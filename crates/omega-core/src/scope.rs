use anyhow::{bail, Context, Result};
use chrono::{DateTime, Duration, Utc};
use fs2::FileExt;
use globset::Glob;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fs::{File, OpenOptions};
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

static CLAIM_COUNTER: AtomicU64 = AtomicU64::new(0);
type LocalClaimReceiptKey = (PathBuf, String);
type LocalClaimReceipts = HashMap<LocalClaimReceiptKey, VecDeque<String>>;
static LOCAL_CLAIM_RECEIPTS: OnceLock<Mutex<LocalClaimReceipts>> = OnceLock::new();

/// A compatibility scope projection. New claims carry both a canonical
/// workspace identity and a unique generation. Missing fields identify a
/// pre-v3 claim and are treated conservatively by authority paths.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeClaim {
    pub session: String,
    pub files_owned: Vec<String>,
    pub claimed_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claim_id: Option<String>,
}

impl ScopeClaim {
    /// Construct an unnamespaced compatibility claim. Authority code should
    /// use [`claim_or_reject_for_workspace`] instead.
    pub fn new(session: &str, files: Vec<String>) -> Result<Self> {
        Self::new_inner(session, None, files)
    }

    fn new_inner(session: &str, workspace_id: Option<String>, files: Vec<String>) -> Result<Self> {
        validate_session_identity(session)?;
        let files_owned = validate_selectors(files)?;
        if files_owned.is_empty() {
            bail!("a writable scope claim must own at least one selector");
        }
        let claim_id = generate_claim_id(session, workspace_id.as_deref())?;
        Ok(Self {
            session: session.to_string(),
            files_owned,
            claimed_at: Utc::now(),
            workspace_id,
            claim_id: Some(claim_id),
        })
    }

    fn path(state_dir: &Path, session: &str) -> Result<PathBuf> {
        validate_session_identity(session)?;
        Ok(state_dir.join(format!("scope-{session}.json")))
    }

    fn validate_for_path(&self, expected_session: &str) -> Result<()> {
        validate_session_identity(&self.session)?;
        if self.session != expected_session {
            bail!(
                "scope claim filename/session mismatch: filename={}, document={}",
                expected_session,
                self.session
            );
        }
        let normalized = validate_selectors(self.files_owned.clone())?;
        if normalized != self.files_owned {
            bail!(
                "scope claim {} contains non-canonical selectors",
                self.session
            );
        }
        match (&self.workspace_id, &self.claim_id) {
            (Some(workspace_id), Some(claim_id)) => {
                validate_workspace_id(workspace_id)?;
                validate_claim_id(claim_id)?;
            }
            (None, None) => {}
            _ => {
                bail!(
                    "scope claim {} mixes legacy and generation-bound authority fields",
                    self.session
                )
            }
        }
        Ok(())
    }

    fn write_locked(&self, state_dir: &Path) -> Result<()> {
        let path = Self::path(state_dir, &self.session)?;
        if crate::config::read_private_optional(&path)?.is_some() {
            // The read verifies type, owner, hard-link count and descriptor/path
            // identity before atomic replacement is permitted.
        }
        let content = serde_json::to_vec_pretty(self)?;
        crate::config::atomic_write_private(&path, &content)?;
        let published: ScopeClaim = read_private_json(&path)?
            .ok_or_else(|| anyhow::anyhow!("scope claim vanished after publish"))?;
        if &published != self {
            bail!("scope claim changed while being published");
        }
        Ok(())
    }

    /// Strict authority read. Unsafe files and malformed claims are errors,
    /// never absence.
    pub fn read_strict(state_dir: &Path, session: &str) -> Result<Option<Self>> {
        ensure_private_state_dir(state_dir)?;
        let path = Self::path(state_dir, session)?;
        let claim: Option<Self> = read_private_json(&path)?;
        if let Some(claim) = &claim {
            claim.validate_for_path(session)?;
        }
        Ok(claim)
    }

    /// Explicitly tolerant diagnostic read retained for TUI/status callers.
    /// Mutation code must use [`ScopeClaim::read_strict`].
    pub fn read(state_dir: &Path, session: &str) -> Option<Self> {
        Self::read_strict(state_dir, session).ok().flatten()
    }

    /// Release exactly the generation returned by the claim operation. A stale
    /// receipt cannot remove a replacement claim (ABA protection).
    pub fn release_exact(state_dir: &Path, expected: &ScopeClaim) -> Result<()> {
        validate_session_identity(&expected.session)?;
        let expected_id = expected
            .claim_id
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("legacy scope claim has no generation receipt"))?;
        let _lock = lock_private_state_file(state_dir, ".scope.lock")?;
        let Some(current) = Self::read_strict(state_dir, &expected.session)? else {
            return Ok(());
        };
        if current.claim_id.as_deref() != Some(expected_id)
            || current.workspace_id != expected.workspace_id
            || current.files_owned != expected.files_owned
        {
            bail!(
                "stale scope release refused for {}: claim generation changed",
                expected.session
            );
        }
        remove_private_file(&Self::path(state_dir, &expected.session)?)?;
        Ok(())
    }

    /// Backward-compatible release for same-process claim/release callsites.
    /// The oldest locally issued receipt is used, so a delayed release cannot
    /// delete a newer generation. After process restart, new claims require an
    /// explicit receipt; only generation-less legacy claims may be removed by
    /// session name.
    pub fn release(state_dir: &Path, session: &str) -> Result<()> {
        validate_session_identity(session)?;
        ensure_private_state_dir(state_dir)?;
        let state_key = std::fs::canonicalize(state_dir)
            .with_context(|| format!("canonicalizing {}", state_dir.display()))?;
        let map = LOCAL_CLAIM_RECEIPTS.get_or_init(|| Mutex::new(HashMap::new()));
        let receipt = {
            let mut guard = map
                .lock()
                .map_err(|_| anyhow::anyhow!("scope receipt registry poisoned"))?;
            let key = (state_key, session.to_string());
            let receipt = guard.get_mut(&key).and_then(VecDeque::pop_front);
            if guard.get(&key).is_some_and(VecDeque::is_empty) {
                guard.remove(&key);
            }
            receipt
        };
        if let Some(claim_id) = receipt {
            let expected = Self::read_strict(state_dir, session)?
                .ok_or_else(|| anyhow::anyhow!("scope claim {session} is already absent"))?;
            if expected.claim_id.as_deref() != Some(claim_id.as_str()) {
                bail!("stale scope release refused for {session}: claim generation changed");
            }
            return Self::release_exact(state_dir, &expected);
        }

        let _lock = lock_private_state_file(state_dir, ".scope.lock")?;
        let Some(current) = Self::read_strict(state_dir, session)? else {
            return Ok(());
        };
        if current.claim_id.is_some() {
            bail!("scope claim {session} has a generation; release requires its exact receipt");
        }
        remove_private_file(&Self::path(state_dir, session)?)
    }

    /// Explicitly tolerant multi-read for diagnostics. Corrupt or unsafe
    /// entries are omitted; authority paths use [`ScopeClaim::read_all_strict`].
    pub fn read_all(state_dir: &Path) -> Vec<Self> {
        Self::read_all_strict(state_dir).unwrap_or_else(|error| {
            tracing::warn!(error = %error, "scope diagnostic view omitted unsafe claims");
            read_all_tolerant(state_dir)
        })
    }

    /// Read every claim or fail the entire authority operation. One corrupt,
    /// unknown or unsafe claim blocks mutation instead of disappearing.
    pub fn read_all_strict(state_dir: &Path) -> Result<Vec<Self>> {
        ensure_private_state_dir(state_dir)?;
        let entries = match std::fs::read_dir(state_dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error).context("reading scope claim directory"),
        };
        let mut claims = Vec::new();
        for entry in entries {
            let entry = entry.context("reading scope claim directory entry")?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| anyhow::anyhow!("scope directory contains a non-UTF-8 filename"))?;
            let Some(session) = name
                .strip_prefix("scope-")
                .and_then(|name| name.strip_suffix(".json"))
            else {
                continue;
            };
            validate_session_identity(session)
                .with_context(|| format!("unsafe scope claim filename {name}"))?;
            let claim: ScopeClaim = read_private_json(&entry.path())?.ok_or_else(|| {
                anyhow::anyhow!(
                    "scope claim {} disappeared during strict read",
                    entry.path().display()
                )
            })?;
            claim.validate_for_path(session)?;
            claims.push(claim);
        }
        claims.sort_by(|left, right| left.session.cmp(&right.session));
        Ok(claims)
    }
}

pub fn check_conflicts(
    state_dir: &Path,
    session: &str,
    files: &[String],
) -> Result<Vec<ScopeConflict>> {
    check_conflicts_inner(state_dir, None, session, files)
}

pub fn check_conflicts_for_workspace(
    state_dir: &Path,
    workspace: &Path,
    session: &str,
    files: &[String],
) -> Result<Vec<ScopeConflict>> {
    let workspace_id = canonical_workspace_identity(workspace)?;
    check_conflicts_inner(state_dir, Some(&workspace_id), session, files)
}

fn check_conflicts_inner(
    state_dir: &Path,
    workspace_id: Option<&str>,
    session: &str,
    files: &[String],
) -> Result<Vec<ScopeConflict>> {
    validate_session_identity(session)?;
    let existing = ScopeClaim::read_all_strict(state_dir)?;
    let requested = validate_selectors(files.to_vec())?;
    let mut conflicts = Vec::new();

    for claim in &existing {
        if claim.session == session {
            continue;
        }
        // Legacy claims have no namespace. They conservatively conflict in
        // every workspace until an operator releases or migrates them.
        if matches!((&claim.workspace_id, workspace_id), (Some(left), Some(right)) if left != right)
        {
            continue;
        }
        let mut overlap = Vec::new();
        for request in &requested {
            for owner in &claim.files_owned {
                if selectors_overlap(request, owner) {
                    overlap.push(format!("{request} <-> {owner}"));
                }
            }
        }
        overlap.sort();
        overlap.dedup();
        if !overlap.is_empty() {
            conflicts.push(ScopeConflict {
                blocking_session: claim.session.clone(),
                overlapping_files: overlap,
            });
        }
    }

    Ok(conflicts)
}

/// Compatibility claim API. It is fail-closed and generation-protected, but
/// unnamespaced, so it conservatively conflicts across all workspaces. New
/// dispatch paths should use [`claim_or_reject_for_workspace`].
pub fn claim_or_reject(state_dir: &Path, session: &str, files: Vec<String>) -> Result<()> {
    claim_or_reject_inner(state_dir, None, session, files).map(|_| ())
}

/// Claim selectors within the canonical identity of one workspace. Aliases to
/// the same directory collide; unrelated workspaces do not.
pub fn claim_or_reject_for_workspace(
    state_dir: &Path,
    workspace: &Path,
    session: &str,
    files: Vec<String>,
) -> Result<ScopeClaim> {
    let workspace_id = canonical_workspace_identity(workspace)?;
    claim_or_reject_inner(state_dir, Some(workspace_id), session, files)
}

/// Prepare a generation-bound workspace claim without publishing it.
///
/// Authority protocols use this to persist the exact receipt in their
/// append-only log before the mutable compatibility file becomes visible.
/// The returned receipt has no effect until [`publish_prepared_claim`] succeeds.
pub fn prepare_claim_for_workspace(
    workspace: &Path,
    session: &str,
    files: Vec<String>,
) -> Result<ScopeClaim> {
    let workspace_id = canonical_workspace_identity(workspace)?;
    ScopeClaim::new_inner(session, Some(workspace_id), files)
}

/// Publish exactly a previously prepared claim under the scope lock.
///
/// Re-publishing the identical receipt is idempotent, including after a
/// process restart. A same-session claim with a different generation is never
/// replaced, which preserves ABA fencing.
pub fn publish_prepared_claim(
    state_dir: &Path,
    workspace: &Path,
    prepared: &ScopeClaim,
) -> Result<ScopeClaim> {
    prepared.validate_for_path(&prepared.session)?;
    let workspace_id = canonical_workspace_identity(workspace)?;
    if prepared.workspace_id.as_deref() != Some(workspace_id.as_str()) {
        bail!(
            "prepared scope claim {} belongs to a different workspace",
            prepared.session
        );
    }
    let claim_id = prepared
        .claim_id
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("prepared scope claim has no generation receipt"))?;
    validate_claim_id(claim_id)?;

    let _lock = lock_private_state_file(state_dir, ".scope.lock")?;
    let existing = ScopeClaim::read_all_strict(state_dir)?;
    if let Some(current) = existing
        .iter()
        .find(|claim| claim.session == prepared.session)
    {
        if current == prepared {
            remember_local_receipt(state_dir, prepared)?;
            return Ok(current.clone());
        }
        bail!(
            "stale prepared scope publication refused for {}: claim generation changed",
            prepared.session
        );
    }
    let conflicts = conflicts_from_claims(
        &existing,
        prepared.workspace_id.as_deref(),
        &prepared.session,
        &prepared.files_owned,
    );
    if !conflicts.is_empty() {
        let details = conflicts
            .iter()
            .map(|conflict| {
                format!(
                    "  {} owns: {}",
                    conflict.blocking_session,
                    conflict.overlapping_files.join(", ")
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        bail!("Scope conflict, files already claimed:\n{details}");
    }
    prepared.write_locked(state_dir)?;
    remember_local_receipt(state_dir, prepared)?;
    Ok(prepared.clone())
}

fn claim_or_reject_inner(
    state_dir: &Path,
    workspace_id: Option<String>,
    session: &str,
    files: Vec<String>,
) -> Result<ScopeClaim> {
    validate_session_identity(session)?;
    let requested = validate_selectors(files)?;
    if requested.is_empty() {
        bail!("a writable scope claim must own at least one selector");
    }
    let _lock = lock_private_state_file(state_dir, ".scope.lock")?;
    let existing = ScopeClaim::read_all_strict(state_dir)?;

    if let Some(current) = existing.iter().find(|claim| claim.session == session) {
        if current.workspace_id != workspace_id {
            bail!("session {session} already owns a claim in a different or legacy workspace");
        }
        if current.files_owned == requested {
            if has_local_receipt(state_dir, current)? {
                return Ok(current.clone());
            }
            bail!(
                "session {session} already owns this scope in another process generation; explicit liveness reconciliation and release_exact are required"
            );
        }
        bail!(
            "session {session} already owns a different scope generation; resize requires replace_claim_exact with the current receipt"
        );
    }

    let conflicts = conflicts_from_claims(&existing, workspace_id.as_deref(), session, &requested);
    if !conflicts.is_empty() {
        let details: Vec<String> = conflicts
            .iter()
            .map(|conflict| {
                format!(
                    "  {} owns: {}",
                    conflict.blocking_session,
                    conflict.overlapping_files.join(", ")
                )
            })
            .collect();
        bail!(
            "Scope conflict, files already claimed:\n{}",
            details.join("\n")
        );
    }

    let claim = ScopeClaim::new_inner(session, workspace_id, requested)?;
    claim.write_locked(state_dir)?;
    remember_local_receipt(state_dir, &claim)?;
    Ok(claim)
}

/// Resize one live claim only when the caller presents the exact current
/// generation. This is the sole same-session replacement path; a reused
/// session name alone cannot steal or broaden scope.
pub fn replace_claim_exact(
    state_dir: &Path,
    workspace: &Path,
    expected: &ScopeClaim,
    files: Vec<String>,
) -> Result<ScopeClaim> {
    let workspace_id = canonical_workspace_identity(workspace)?;
    let requested = validate_selectors(files)?;
    if requested.is_empty() {
        bail!("a writable scope claim must own at least one selector");
    }
    let expected_id = expected
        .claim_id
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("legacy scope claim has no generation receipt"))?;
    let _lock = lock_private_state_file(state_dir, ".scope.lock")?;
    let existing = ScopeClaim::read_all_strict(state_dir)?;
    let current = existing
        .iter()
        .find(|claim| claim.session == expected.session)
        .ok_or_else(|| anyhow::anyhow!("scope claim {} is absent", expected.session))?;
    if current.claim_id.as_deref() != Some(expected_id)
        || current.workspace_id.as_deref() != Some(workspace_id.as_str())
        || current.workspace_id != expected.workspace_id
        || current.files_owned != expected.files_owned
    {
        bail!(
            "scope resize refused for {}: claim generation changed",
            expected.session
        );
    }
    let conflicts = conflicts_from_claims(
        &existing,
        Some(&workspace_id),
        &expected.session,
        &requested,
    );
    if !conflicts.is_empty() {
        bail!(
            "scope resize for {} conflicts with {}",
            expected.session,
            conflicts
                .iter()
                .map(|conflict| conflict.blocking_session.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    let replacement = ScopeClaim::new_inner(&expected.session, Some(workspace_id), requested)?;
    replacement.write_locked(state_dir)?;
    remember_local_receipt(state_dir, &replacement)?;
    Ok(replacement)
}

fn generate_claim_id(session: &str, workspace_id: Option<&str>) -> Result<String> {
    let mut entropy = [0_u8; 32];
    #[cfg(unix)]
    {
        File::open("/dev/urandom")
            .context("opening OS entropy source for scope generation")?
            .read_exact(&mut entropy)
            .context("reading OS entropy for scope generation")?;
    }
    #[cfg(not(unix))]
    {
        bail!("secure scope generation entropy is unavailable on this platform");
    }
    let serial = CLAIM_COUNTER.fetch_add(1, Ordering::Relaxed);
    Ok(claim_id_from_entropy(
        session,
        workspace_id,
        &entropy,
        serial,
    ))
}

fn claim_id_from_entropy(
    session: &str,
    workspace_id: Option<&str>,
    entropy: &[u8; 32],
    serial: u64,
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"omega-scope-claim-v3\0");
    hasher.update(&(session.len() as u64).to_le_bytes());
    hasher.update(session.as_bytes());
    let workspace = workspace_id.unwrap_or("legacy");
    hasher.update(&(workspace.len() as u64).to_le_bytes());
    hasher.update(workspace.as_bytes());
    hasher.update(entropy);
    // The counter is not entropy; it only prevents an accidental duplicate if
    // a broken entropy provider repeats bytes within one process.
    hasher.update(&serial.to_le_bytes());
    hasher.finalize().to_hex().to_string()
}

fn conflicts_from_claims(
    existing: &[ScopeClaim],
    workspace_id: Option<&str>,
    session: &str,
    requested: &[String],
) -> Vec<ScopeConflict> {
    let mut conflicts = Vec::new();
    for claim in existing {
        if claim.session == session {
            continue;
        }
        if matches!((&claim.workspace_id, workspace_id), (Some(left), Some(right)) if left != right)
        {
            continue;
        }
        let mut overlap = Vec::new();
        for request in requested {
            for owner in &claim.files_owned {
                if selectors_overlap(request, owner) {
                    overlap.push(format!("{request} <-> {owner}"));
                }
            }
        }
        overlap.sort();
        overlap.dedup();
        if !overlap.is_empty() {
            conflicts.push(ScopeConflict {
                blocking_session: claim.session.clone(),
                overlapping_files: overlap,
            });
        }
    }
    conflicts
}

fn remember_local_receipt(state_dir: &Path, claim: &ScopeClaim) -> Result<()> {
    let Some(claim_id) = claim.claim_id.clone() else {
        return Ok(());
    };
    let state_key = std::fs::canonicalize(state_dir)
        .with_context(|| format!("canonicalizing {}", state_dir.display()))?;
    let map = LOCAL_CLAIM_RECEIPTS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = map
        .lock()
        .map_err(|_| anyhow::anyhow!("scope receipt registry poisoned"))?;
    let queue = guard.entry((state_key, claim.session.clone())).or_default();
    if queue.back() != Some(&claim_id) {
        queue.push_back(claim_id);
    }
    Ok(())
}

fn has_local_receipt(state_dir: &Path, claim: &ScopeClaim) -> Result<bool> {
    let Some(claim_id) = claim.claim_id.as_ref() else {
        return Ok(false);
    };
    let state_key = std::fs::canonicalize(state_dir)
        .with_context(|| format!("canonicalizing {}", state_dir.display()))?;
    let map = LOCAL_CLAIM_RECEIPTS.get_or_init(|| Mutex::new(HashMap::new()));
    let guard = map
        .lock()
        .map_err(|_| anyhow::anyhow!("scope receipt registry poisoned"))?;
    Ok(guard
        .get(&(state_key, claim.session.clone()))
        .is_some_and(|receipts| receipts.iter().any(|receipt| receipt == claim_id)))
}

#[derive(Debug, Clone)]
pub struct ScopeConflict {
    pub blocking_session: String,
    pub overlapping_files: Vec<String>,
}

/// Validate an rmux/session identity before it becomes part of a filename.
pub(crate) fn validate_session_identity(session: &str) -> Result<()> {
    if session.is_empty() || session.len() > 160 || session == "." || session == ".." {
        bail!("invalid session identity `{session}`");
    }
    if !session
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        bail!(
            "invalid session identity `{session}`: use ASCII letters, digits, dot, dash or underscore"
        );
    }
    if session.starts_with('.') || session.ends_with('.') || session.contains("..") {
        bail!("invalid session identity `{session}`");
    }
    Ok(())
}

fn validate_claim_id(claim_id: &str) -> Result<()> {
    if claim_id.len() != 64 || !claim_id.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("invalid scope claim generation");
    }
    Ok(())
}

fn validate_workspace_id(workspace_id: &str) -> Result<()> {
    let mut parts = workspace_id.split(':');
    if parts.next() != Some("workspace-v1")
        || parts
            .next()
            .and_then(|part| part.parse::<u64>().ok())
            .is_none()
        || parts
            .next()
            .and_then(|part| part.parse::<u64>().ok())
            .is_none()
        || parts.next().is_some()
    {
        bail!("invalid canonical workspace identity");
    }
    Ok(())
}

fn canonical_workspace_identity(workspace: &Path) -> Result<String> {
    let canonical = std::fs::canonicalize(workspace)
        .with_context(|| format!("canonicalizing workspace {}", workspace.display()))?;
    let metadata = std::fs::metadata(&canonical)
        .with_context(|| format!("inspecting workspace {}", canonical.display()))?;
    if !metadata.is_dir() {
        bail!("workspace {} is not a directory", canonical.display());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Ok(format!(
            "workspace-v1:{}:{}",
            metadata.dev(),
            metadata.ino()
        ))
    }
    #[cfg(not(unix))]
    {
        let digest = blake3::hash(canonical.to_string_lossy().as_bytes());
        Ok(format!(
            "workspace-v1:0:{}",
            u64::from_le_bytes(digest.as_bytes()[..8].try_into().expect("eight bytes"))
        ))
    }
}

fn validate_selectors(files: Vec<String>) -> Result<Vec<String>> {
    let mut normalized = Vec::with_capacity(files.len());
    for raw in files {
        let selector = validate_scope_selector(&raw)?;
        normalized.push(selector);
    }
    normalized.sort();
    normalized.dedup();
    Ok(normalized)
}

/// Validate and canonicalize scope selectors for other authority projections.
///
/// Registry and lifecycle documents persist the same selector language as
/// scope claims. Keeping one validator prevents a forged projection from
/// smuggling an absolute/traversing/malformed selector past R-SCOPE even when
/// it is not creating a claim itself.
pub(crate) fn validate_scope_selectors(files: Vec<String>) -> Result<Vec<String>> {
    validate_selectors(files)
}

fn validate_scope_selector(raw: &str) -> Result<String> {
    let selector = raw.trim().replace('\\', "/");
    if selector.is_empty() {
        bail!("scope selector cannot be empty");
    }
    if selector.starts_with('/')
        || selector
            .bytes()
            .any(|byte| byte == 0 || byte.is_ascii_control())
    {
        bail!("scope selector `{raw}` is absolute or contains control bytes");
    }
    for component in Path::new(&selector).components() {
        match component {
            Component::ParentDir => bail!("scope selector `{raw}` may not contain `..`"),
            Component::RootDir | Component::Prefix(_) => {
                bail!("scope selector `{raw}` must be workspace-relative")
            }
            Component::CurDir | Component::Normal(_) => {}
        }
    }
    let normalized = normalize_scope_selector(&selector);
    if normalized.is_empty() || normalized == "." {
        bail!("scope selector `{raw}` does not identify a workspace path");
    }
    Glob::new(&normalized)
        .with_context(|| format!("scope selector `{raw}` is not a valid glob"))?;
    Ok(normalized)
}

/// Lexically normalize a scope selector without touching the filesystem.
/// Authority paths validate the raw selector first with `validate_scope_selector`.
pub fn normalize_scope_selector(raw: &str) -> String {
    let slash_normalized = raw.trim().replace('\\', "/");
    if slash_normalized.is_empty() {
        return String::new();
    }

    let absolute = slash_normalized.starts_with('/');
    let mut parts: Vec<String> = Vec::new();
    for component in Path::new(&slash_normalized).components() {
        match component {
            Component::Prefix(prefix) => {
                parts.push(prefix.as_os_str().to_string_lossy().to_string())
            }
            Component::RootDir | Component::CurDir => {}
            Component::ParentDir => {
                if parts.last().is_some_and(|part| part != "..") {
                    parts.pop();
                } else if !absolute {
                    parts.push("..".to_string());
                }
            }
            Component::Normal(part) => parts.push(part.to_string_lossy().to_string()),
        }
    }
    let joined = parts.join("/");
    if absolute {
        format!("/{joined}")
    } else {
        joined
    }
}

/// Conservative selector intersection.
pub fn selectors_overlap(left: &str, right: &str) -> bool {
    let left = normalize_scope_selector(left);
    let right = normalize_scope_selector(right);
    if left.is_empty() || right.is_empty() {
        return false;
    }
    if left == right || path_contains(&left, &right) || path_contains(&right, &left) {
        return true;
    }

    let left_glob = has_glob_meta(&left);
    let right_glob = has_glob_meta(&right);
    match (left_glob, right_glob) {
        (true, false) => {
            glob_matches(&left, &right) || path_contains(&right, &static_glob_prefix(&left))
        }
        (false, true) => {
            glob_matches(&right, &left) || path_contains(&left, &static_glob_prefix(&right))
        }
        (true, true) => {
            let left_prefix = static_glob_prefix(&left);
            let right_prefix = static_glob_prefix(&right);
            left_prefix.is_empty()
                || right_prefix.is_empty()
                || path_contains(&left_prefix, &right_prefix)
                || path_contains(&right_prefix, &left_prefix)
        }
        (false, false) => false,
    }
}

fn path_contains(parent: &str, child: &str) -> bool {
    !parent.is_empty()
        && (parent == child
            || child
                .strip_prefix(parent)
                .is_some_and(|suffix| suffix.starts_with('/')))
}

fn has_glob_meta(selector: &str) -> bool {
    selector
        .bytes()
        .any(|byte| matches!(byte, b'*' | b'?' | b'[' | b'{'))
}

fn static_glob_prefix(selector: &str) -> String {
    let meta = selector
        .char_indices()
        .find_map(|(index, ch)| "*?[{".contains(ch).then_some(index))
        .unwrap_or(selector.len());
    selector[..meta].trim_end_matches('/').to_string()
}

fn glob_matches(pattern: &str, candidate: &str) -> bool {
    Glob::new(pattern)
        .map(|glob| glob.compile_matcher().is_match(candidate))
        .unwrap_or(true)
}

/// Validate/create a private state directory. The state root itself may not be
/// a symlink because every authority filename is resolved beneath it.
pub(crate) fn ensure_private_state_dir(state_dir: &Path) -> Result<()> {
    match std::fs::symlink_metadata(state_dir) {
        Ok(metadata) => {
            if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
                bail!(
                    "state directory {} is not a real directory",
                    state_dir.display()
                );
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::{MetadataExt, PermissionsExt};
                if metadata.uid() != effective_uid() {
                    bail!(
                        "state directory {} is owned by uid {}, current uid is {}",
                        state_dir.display(),
                        metadata.uid(),
                        effective_uid()
                    );
                }
                if metadata.mode() & 0o077 != 0 {
                    std::fs::set_permissions(state_dir, std::fs::Permissions::from_mode(0o700))
                        .with_context(|| {
                            format!("tightening permissions on {}", state_dir.display())
                        })?;
                }
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            std::fs::create_dir_all(state_dir)
                .with_context(|| format!("creating {}", state_dir.display()))?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(state_dir, std::fs::Permissions::from_mode(0o700))?;
            }
        }
        Err(error) => {
            return Err(error).with_context(|| format!("inspecting {}", state_dir.display()))
        }
    }
    Ok(())
}

/// Hold an exclusive, owner-only, no-follow state mutation lock.
pub(crate) fn lock_private_state_file(state_dir: &Path, name: &str) -> Result<File> {
    ensure_private_state_dir(state_dir)?;
    if !name.starts_with('.') || name.contains('/') || name.contains('\\') || name.contains("..") {
        bail!("invalid internal lock filename `{name}`");
    }
    let path = state_dir.join(name);
    if let Ok(metadata) = std::fs::symlink_metadata(&path) {
        if !metadata.file_type().is_file() {
            bail!("refusing unsafe lock path {}", path.display());
        }
    }
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600).custom_flags(no_follow_flag());
    }
    let file = options
        .open(&path)
        .with_context(|| format!("opening secure lock {}", path.display()))?;
    validate_open_file_identity(&path, &file)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
    }
    file.lock_exclusive()
        .with_context(|| format!("locking {}", path.display()))?;
    validate_open_file_identity(&path, &file)?;
    Ok(file)
}

pub(crate) fn read_private_json<T: DeserializeOwned>(path: &Path) -> Result<Option<T>> {
    crate::config::read_private_optional(path)?
        .map(|bytes| {
            serde_json::from_slice(&bytes)
                .with_context(|| format!("parsing authority file {}", path.display()))
        })
        .transpose()
}

pub(crate) fn remove_private_file(path: &Path) -> Result<()> {
    if crate::config::read_private_optional(path)?.is_none() {
        return Ok(());
    }
    std::fs::remove_file(path).with_context(|| format!("removing {}", path.display()))?;
    if let Some(parent) = path.parent() {
        File::open(parent)
            .and_then(|directory| directory.sync_all())
            .with_context(|| format!("syncing {}", parent.display()))?;
    }
    Ok(())
}

fn validate_open_file_identity(path: &Path, file: &File) -> Result<()> {
    let path_metadata = std::fs::symlink_metadata(path)
        .with_context(|| format!("inspecting lock path {}", path.display()))?;
    let opened = file
        .metadata()
        .with_context(|| format!("inspecting opened lock {}", path.display()))?;
    if !path_metadata.file_type().is_file() || !opened.file_type().is_file() {
        bail!("lock {} is not a regular file", path.display());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if path_metadata.dev() != opened.dev() || path_metadata.ino() != opened.ino() {
            bail!("lock {} changed identity while opening", path.display());
        }
        if opened.nlink() != 1 {
            bail!("lock {} has {} hard links", path.display(), opened.nlink());
        }
        if opened.uid() != effective_uid() {
            bail!("lock {} is owned by another uid", path.display());
        }
    }
    Ok(())
}

fn read_all_tolerant(state_dir: &Path) -> Vec<ScopeClaim> {
    let mut claims = Vec::new();
    let Ok(entries) = std::fs::read_dir(state_dir) else {
        return claims;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some(session) = name
            .strip_prefix("scope-")
            .and_then(|name| name.strip_suffix(".json"))
        else {
            continue;
        };
        let Ok(Some(claim)) = read_private_json::<ScopeClaim>(&path) else {
            continue;
        };
        if claim.validate_for_path(session).is_ok() {
            claims.push(claim);
        }
    }
    claims.sort_by(|left, right| left.session.cmp(&right.session));
    claims
}

#[cfg(unix)]
fn effective_uid() -> u32 {
    unsafe extern "C" {
        fn geteuid() -> u32;
    }
    // SAFETY: geteuid has no preconditions and returns process metadata only.
    unsafe { geteuid() }
}

#[cfg(all(unix, any(target_os = "linux", target_os = "android")))]
fn no_follow_flag() -> i32 {
    0o400000
}

#[cfg(all(
    unix,
    any(
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd"
    )
))]
fn no_follow_flag() -> i32 {
    0x100
}

#[cfg(all(
    unix,
    not(any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd"
    ))
))]
fn no_follow_flag() -> i32 {
    0
}

/// Durable scope lease data carried alongside a ledger lease.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FencedScopeLease {
    pub lease_id: String,
    pub resource: String,
    pub owner: String,
    pub selectors: Vec<String>,
    pub fencing_token: u64,
    pub acquired_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl FencedScopeLease {
    pub fn new(
        lease_id: impl Into<String>,
        resource: impl Into<String>,
        owner: impl Into<String>,
        selectors: Vec<String>,
        fencing_token: u64,
        ttl: Duration,
    ) -> Self {
        let acquired_at = Utc::now();
        let mut selectors: Vec<String> = selectors
            .into_iter()
            .map(|selector| normalize_scope_selector(&selector))
            .filter(|selector| !selector.is_empty())
            .collect();
        selectors.sort();
        selectors.dedup();
        Self {
            lease_id: lease_id.into(),
            resource: resource.into(),
            owner: owner.into(),
            selectors,
            fencing_token,
            acquired_at,
            expires_at: acquired_at + ttl,
        }
    }

    pub fn is_active_at(&self, now: DateTime<Utc>) -> bool {
        self.expires_at > now
    }

    pub fn accepts_fence(&self, supplied_token: u64, now: DateTime<Utc>) -> bool {
        self.is_active_at(now) && supplied_token == self.fencing_token
    }
}

#[cfg(test)]
mod scope_v3_tests {
    use super::*;
    use std::sync::{Arc, Barrier};

    #[test]
    fn lexical_aliases_normalize_to_one_selector() {
        assert_eq!(
            normalize_scope_selector("./crates//omega-core/src/./lib.rs"),
            "crates/omega-core/src/lib.rs"
        );
        assert!(selectors_overlap("./src/lib.rs", "src/lib.rs"));
    }

    #[test]
    fn directory_parent_conflicts_with_child() {
        assert!(selectors_overlap(
            "crates/omega-core",
            "crates/omega-core/src/lib.rs"
        ));
        assert!(!selectors_overlap(
            "crates/omega",
            "crates/omega-core/src/lib.rs"
        ));
    }

    #[test]
    fn glob_conflicts_with_matching_file_and_directory() {
        assert!(selectors_overlap(
            "crates/*/src/**",
            "crates/omega-core/src/lib.rs"
        ));
        assert!(selectors_overlap(
            "crates/omega-core/**",
            "crates/omega-core"
        ));
        assert!(!selectors_overlap(
            "docs/**/*.md",
            "crates/omega-core/src/lib.rs"
        ));
    }

    #[test]
    fn traversal_absolute_and_malformed_glob_are_rejected() {
        let state = tempfile::tempdir().unwrap();
        let workspace = tempfile::tempdir().unwrap();
        for selector in ["../secret", "src/../secret", "/etc/passwd", "src/[abc"] {
            assert!(claim_or_reject_for_workspace(
                state.path(),
                workspace.path(),
                "worker-safe",
                vec![selector.to_string()]
            )
            .is_err());
        }
        assert!(claim_or_reject_for_workspace(
            state.path(),
            workspace.path(),
            "../escape",
            vec!["src/lib.rs".to_string()]
        )
        .is_err());
    }

    #[test]
    fn corrupt_claim_blocks_mutation() {
        let state = tempfile::tempdir().unwrap();
        let workspace = tempfile::tempdir().unwrap();
        std::fs::write(state.path().join("scope-broken.json"), b"{").unwrap();
        assert!(claim_or_reject_for_workspace(
            state.path(),
            workspace.path(),
            "worker-safe",
            vec!["src/lib.rs".to_string()]
        )
        .is_err());
    }

    #[test]
    fn partial_v3_claim_blocks_mutation_instead_of_losing_global_exclusion() {
        let state = tempfile::tempdir().unwrap();
        let left = tempfile::tempdir().unwrap();
        let right = tempfile::tempdir().unwrap();
        let left_workspace_id = canonical_workspace_identity(left.path()).unwrap();
        let partial = serde_json::json!({
            "session": "worker-partial",
            "files_owned": ["src/lib.rs"],
            "claimed_at": Utc::now(),
            "workspace_id": left_workspace_id
        });
        std::fs::write(
            state.path().join("scope-worker-partial.json"),
            serde_json::to_vec_pretty(&partial).unwrap(),
        )
        .unwrap();

        let error = claim_or_reject_for_workspace(
            state.path(),
            right.path(),
            "worker-right",
            vec!["src/lib.rs".to_string()],
        )
        .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("mixes legacy and generation-bound authority fields"),
            "unexpected error: {error:#}"
        );
        assert!(ScopeClaim::read_strict(state.path(), "worker-right")
            .unwrap()
            .is_none());
    }

    #[test]
    fn different_workspaces_do_not_false_conflict() {
        let state = tempfile::tempdir().unwrap();
        let left = tempfile::tempdir().unwrap();
        let right = tempfile::tempdir().unwrap();
        claim_or_reject_for_workspace(
            state.path(),
            left.path(),
            "worker-left",
            vec!["src/lib.rs".to_string()],
        )
        .unwrap();
        claim_or_reject_for_workspace(
            state.path(),
            right.path(),
            "worker-right",
            vec!["src/lib.rs".to_string()],
        )
        .unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn canonical_workspace_aliases_conflict() {
        use std::os::unix::fs::symlink;
        let root = tempfile::tempdir().unwrap();
        let state = root.path().join("state");
        let workspace = root.path().join("workspace");
        let alias = root.path().join("alias");
        std::fs::create_dir_all(&workspace).unwrap();
        symlink(&workspace, &alias).unwrap();
        claim_or_reject_for_workspace(
            &state,
            &workspace,
            "worker-left",
            vec!["src/lib.rs".to_string()],
        )
        .unwrap();
        assert!(claim_or_reject_for_workspace(
            &state,
            &alias,
            "worker-right",
            vec!["src/lib.rs".to_string()]
        )
        .is_err());
    }

    #[test]
    fn simultaneous_overlapping_claims_have_one_winner() {
        let state = Arc::new(tempfile::tempdir().unwrap());
        let workspace = Arc::new(tempfile::tempdir().unwrap());
        let barrier = Arc::new(Barrier::new(2));
        let mut handles = Vec::new();
        for index in 0..2 {
            let state = Arc::clone(&state);
            let workspace = Arc::clone(&workspace);
            let barrier = Arc::clone(&barrier);
            handles.push(std::thread::spawn(move || {
                barrier.wait();
                claim_or_reject_for_workspace(
                    state.path(),
                    workspace.path(),
                    &format!("worker-{index}"),
                    vec!["src/lib.rs".to_string()],
                )
            }));
        }
        let successes = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .filter(Result::is_ok)
            .count();
        assert_eq!(successes, 1);
    }

    #[test]
    fn stale_release_cannot_delete_reclaimed_generation() {
        let state = tempfile::tempdir().unwrap();
        let workspace = tempfile::tempdir().unwrap();
        let first = claim_or_reject_for_workspace(
            state.path(),
            workspace.path(),
            "worker-a",
            vec!["src/a.rs".to_string()],
        )
        .unwrap();
        assert!(claim_or_reject_for_workspace(
            state.path(),
            workspace.path(),
            "worker-a",
            vec!["src/b.rs".to_string()],
        )
        .is_err());
        let second = replace_claim_exact(
            state.path(),
            workspace.path(),
            &first,
            vec!["src/b.rs".to_string()],
        )
        .unwrap();
        assert_ne!(first.claim_id, second.claim_id);
        assert!(ScopeClaim::release_exact(state.path(), &first).is_err());
        assert_eq!(
            ScopeClaim::read_strict(state.path(), "worker-a")
                .unwrap()
                .unwrap()
                .claim_id,
            second.claim_id
        );
        ScopeClaim::release_exact(state.path(), &second).unwrap();
    }

    #[test]
    fn prepared_claim_is_inert_then_publishes_idempotently_by_exact_receipt() {
        let state = tempfile::tempdir().unwrap();
        let workspace = tempfile::tempdir().unwrap();
        let prepared = prepare_claim_for_workspace(
            workspace.path(),
            "worker-prepared",
            vec!["src/lib.rs".to_string()],
        )
        .unwrap();
        assert!(ScopeClaim::read_strict(state.path(), "worker-prepared")
            .unwrap()
            .is_none());

        let first = publish_prepared_claim(state.path(), workspace.path(), &prepared).unwrap();
        let recovered = publish_prepared_claim(state.path(), workspace.path(), &prepared).unwrap();
        assert_eq!(first, prepared);
        assert_eq!(recovered, prepared);
        assert_eq!(
            ScopeClaim::read_strict(state.path(), "worker-prepared")
                .unwrap()
                .unwrap(),
            prepared
        );

        let other_generation = prepare_claim_for_workspace(
            workspace.path(),
            "worker-prepared",
            vec!["src/lib.rs".to_string()],
        )
        .unwrap();
        assert_ne!(other_generation.claim_id, prepared.claim_id);
        assert!(publish_prepared_claim(state.path(), workspace.path(), &other_generation).is_err());
        assert_eq!(
            ScopeClaim::read_strict(state.path(), "worker-prepared")
                .unwrap()
                .unwrap(),
            prepared
        );
        ScopeClaim::release_exact(state.path(), &prepared).unwrap();
    }

    #[test]
    fn claim_generation_uses_os_entropy_not_pid_or_time_identity() {
        // Model two fresh processes that reused every observable process/time
        // input. Independent OS entropy still yields distinct release authority.
        let first = claim_id_from_entropy("worker-a", Some("workspace-v1:1:2"), &[7; 32], 0);
        let second = claim_id_from_entropy("worker-a", Some("workspace-v1:1:2"), &[8; 32], 0);
        assert_ne!(first, second);
        assert_eq!(first.len(), 64);
        assert_eq!(second.len(), 64);
    }

    #[cfg(unix)]
    #[test]
    fn symlink_dangling_hardlink_and_lock_aliases_are_rejected() {
        use std::os::unix::fs::symlink;
        let root = tempfile::tempdir().unwrap();
        let workspace = tempfile::tempdir().unwrap();

        let dangling_state = root.path().join("dangling-state");
        std::fs::create_dir(&dangling_state).unwrap();
        symlink(
            dangling_state.join("missing"),
            dangling_state.join("scope-bad.json"),
        )
        .unwrap();
        assert!(claim_or_reject_for_workspace(
            &dangling_state,
            workspace.path(),
            "worker-a",
            vec!["src/lib.rs".to_string()]
        )
        .is_err());

        let hard_state = root.path().join("hard-state");
        std::fs::create_dir(&hard_state).unwrap();
        let source = hard_state.join("source");
        std::fs::write(&source, b"{}").unwrap();
        std::fs::hard_link(&source, hard_state.join("scope-bad.json")).unwrap();
        assert!(claim_or_reject_for_workspace(
            &hard_state,
            workspace.path(),
            "worker-a",
            vec!["src/lib.rs".to_string()]
        )
        .is_err());

        let lock_state = root.path().join("lock-state");
        std::fs::create_dir(&lock_state).unwrap();
        let lock_target = lock_state.join("target");
        std::fs::write(&lock_target, b"sentinel").unwrap();
        symlink(&lock_target, lock_state.join(".scope.lock")).unwrap();
        assert!(claim_or_reject_for_workspace(
            &lock_state,
            workspace.path(),
            "worker-a",
            vec!["src/lib.rs".to_string()]
        )
        .is_err());
        assert_eq!(std::fs::read(&lock_target).unwrap(), b"sentinel");
    }

    #[test]
    fn expired_or_old_fence_is_rejected() {
        let lease = FencedScopeLease::new(
            "lease-1",
            "repo:scope",
            "worker-a",
            vec!["src/**".to_string()],
            42,
            Duration::seconds(5),
        );
        assert!(!lease.accepts_fence(41, Utc::now()));
        assert!(lease.accepts_fence(42, Utc::now()));
        assert!(!lease.accepts_fence(42, lease.expires_at));
    }
}

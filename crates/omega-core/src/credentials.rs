//! Centralized credential storage for OmegaOS.
//!
//! All provider credentials live in `~/.omega/credentials/`. Each provider has:
//!   - `~/.omega/credentials/<provider>.json`            — active credentials
//!   - `~/.omega/credentials/accounts/<provider>-<name>.json` — saved accounts
//!
//! Legacy LLM CLIs (claude, codex, gemini) expect their credentials at known
//! paths (~/.claude/.credentials.json, ~/.codex/auth.json, etc.). We make those
//! paths symlinks to the canonical files under ~/.omega/credentials/ so:
//!   - The CLI still finds its creds.
//!   - OmegaOS owns the source of truth.
//!   - Account switching = atomic rewrite of the canonical file (symlink stays).
//!
//! ## Relationship to `account.rs` (KNOWN DESIGN-DEBT)
//! `account.rs` is a SEPARATE, Claude-only surface for the `/account` billing +
//! switch view (keyed by `~/.claude/accounts/accounts-meta.json`). The two share
//! the active Claude file (account.rs writes via `oauth::credentials_path()` =
//! this store's `claude.json`), so the ACTIVE account is consistent — but the two
//! SAVED-profile lists are not unified (a Claude account saved here via `/model`
//! is not listed by `/account`, and vice versa). See the account.rs module doc for
//! the full boundary + why a merge is deferred. Keep both lists in sync if you
//! change Claude account-switching.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::cmp::Ordering;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::providers::ProvidersConfig;

static UNIQUE_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Structural validity of a Codex `auth.json`, without exposing credential
/// contents. A valid ChatGPT credential has all three OAuth token fields; a
/// valid API-key credential has a non-empty `OPENAI_API_KEY`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodexCredentialValidity {
    Missing,
    Invalid,
    Valid,
}

/// Safe metadata used to compare Codex credential copies. This deliberately
/// contains no token, account, email, or API-key value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodexCredentialInspection {
    pub validity: CodexCredentialValidity,
    pub last_refresh: Option<DateTime<Utc>>,
    pub modified: Option<SystemTime>,
}

impl CodexCredentialInspection {
    pub fn is_valid(&self) -> bool {
        self.validity == CodexCredentialValidity::Valid
    }
}

/// Which credential copy won a Codex reconciliation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodexReconcileAction {
    NoValidCredential,
    Unchanged,
    RepairedNativeLink,
    AdoptedNative,
    AdoptedCandidate,
}

/// Safe, read-only description of the Codex native/canonical topology.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodexTopologyReport {
    pub native_path: PathBuf,
    pub canonical_path: PathBuf,
    pub native_exists: bool,
    pub canonical_exists: bool,
    pub native_is_symlink: bool,
    pub native_links_to_canonical: bool,
    pub native_validity: CodexCredentialValidity,
    pub canonical_validity: CodexCredentialValidity,
    pub split: bool,
}

/// Result of reconciling Codex credentials. Quarantine paths are safe to print;
/// their file contents are not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodexReconcileReport {
    pub action: CodexReconcileAction,
    pub quarantined: Vec<PathBuf>,
    pub topology: CodexTopologyReport,
}

/// Manager for ~/.omega/credentials/ — every provider's creds + saved accounts.
#[derive(Debug, Clone)]
pub struct CredentialStore {
    base_dir: PathBuf,
    codex_home: PathBuf,
}

impl CredentialStore {
    pub fn new() -> Result<Self> {
        Self::from_roots(&omega_dir(), &codex_home_dir())
    }

    /// Build a store with explicit Omega and Codex roots. This is the fully
    /// deterministic constructor for login flows and tests.
    pub(crate) fn from_roots(root: &Path, codex_home: &Path) -> Result<Self> {
        let base = root.join("credentials");
        ensure_owner_only_dir(&base)?;
        ensure_owner_only_dir(&base.join("accounts"))?;
        Ok(Self {
            base_dir: base,
            codex_home: codex_home.to_path_buf(),
        })
    }

    /// Path to the active credentials file for a provider.
    pub fn active_path(&self, provider: &str) -> PathBuf {
        self.base_dir
            .join(format!("{}.json", safe_provider_component(provider)))
    }

    /// Path to a named saved account for a provider.
    pub fn account_path(&self, provider: &str, name: &str) -> PathBuf {
        self.base_dir.join("accounts").join(format!(
            "{}-{}.json",
            safe_provider_component(provider),
            sanitize(name)
        ))
    }

    /// Read the active credentials for a provider as JSON.
    pub fn read(&self, provider: &str) -> Result<Value> {
        validate_provider_id(provider)?;
        let path = self.active_path(provider);
        let raw = crate::config::read_private_optional_string(&path)?
            .with_context(|| format!("credential file {} does not exist", path.display()))?;
        let v: Value =
            serde_json::from_str(&raw).with_context(|| format!("parsing {}", path.display()))?;
        Ok(v)
    }

    /// Write the active credentials for a provider. Atomic via tmp+rename.
    pub fn write(&self, provider: &str, data: &Value) -> Result<()> {
        validate_provider_id(provider)?;
        let path = self.active_path(provider);
        let json = serde_json::to_string_pretty(data)?;
        crate::config::with_private_file_lock(&path, || {
            // Existing authority must itself be a private regular file. Atomic
            // rename may replace a symlink safely, but silently accepting one
            // would hide a compromised credential topology from the operator.
            let _ = crate::config::read_private_optional(&path)?;
            crate::config::atomic_write_private(&path, json.as_bytes())
                .with_context(|| format!("writing {}", path.display()))
        })
    }

    /// List saved account names for a provider (just the `<name>` part).
    pub fn list_accounts(&self, provider: &str) -> Vec<String> {
        if validate_provider_id(provider).is_err() {
            return Vec::new();
        }
        let accounts = self.base_dir.join("accounts");
        let prefix = format!("{}-", provider);
        let mut out = Vec::new();
        let Ok(entries) = std::fs::read_dir(&accounts) else {
            return out;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let Some(stem) = p.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            if let Some(name) = stem.strip_prefix(&prefix) {
                out.push(name.to_string());
            }
        }
        out.sort();
        out
    }

    /// Switch the active provider credentials to a named saved account.
    /// Copies the account file over the active file (atomic via tmp+rename).
    pub fn switch_account(&self, provider: &str, name: &str) -> Result<()> {
        validate_provider_id(provider)?;
        validate_account_name(name)?;
        let from = self.account_path(provider, name);
        let bytes = crate::config::read_private_optional(&from)?
            .with_context(|| format!("account not found: {name} (looked at {})", from.display()))?;
        serde_json::from_slice::<Value>(&bytes)
            .with_context(|| format!("parsing saved account {}", from.display()))?;
        let to = self.active_path(provider);
        crate::config::with_private_file_lock(&to, || {
            let _ = crate::config::read_private_optional(&to)?;
            crate::config::atomic_write_private(&to, &bytes)
                .with_context(|| format!("switching {} -> {}", from.display(), to.display()))
        })
    }

    /// Save the currently active credentials as a named account.
    pub fn save_as_account(&self, provider: &str, name: &str) -> Result<()> {
        validate_provider_id(provider)?;
        validate_account_name(name)?;
        let from = self.active_path(provider);
        let bytes = crate::config::read_private_optional(&from)?.with_context(|| {
            format!(
                "no active credentials for {provider} (no file at {})",
                from.display()
            )
        })?;
        serde_json::from_slice::<Value>(&bytes)
            .with_context(|| format!("parsing active credential {}", from.display()))?;
        let to = self.account_path(provider, name);
        crate::config::with_private_file_lock(&to, || {
            let _ = crate::config::read_private_optional(&to)?;
            crate::config::atomic_write_private(&to, &bytes)
                .with_context(|| format!("saving account {} -> {}", from.display(), to.display()))
        })
    }

    /// Ensure the legacy path for a provider exists as a symlink to the
    /// canonical file in ~/.omega/credentials/. Creates parent dirs as needed.
    /// No-op if the legacy path already points to the canonical file.
    pub fn ensure_legacy_symlink(&self, provider: &str) -> Result<()> {
        validate_provider_id(provider)?;
        if provider == "codex" {
            self.reconcile_codex()?;
            return Ok(());
        }
        let Some(legacy) = legacy_path_for(provider) else {
            return Ok(());
        };
        let canonical = self.active_path(provider);
        if let Some(parent) = legacy.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        // If legacy is already a symlink pointing at the right target, done.
        if let Ok(meta) = std::fs::symlink_metadata(&legacy) {
            if meta.file_type().is_symlink() {
                if let Ok(target) = std::fs::read_link(&legacy) {
                    if target == canonical {
                        return Ok(());
                    }
                }
                // wrong target — replace
                std::fs::remove_file(&legacy).ok();
            } else if meta.is_file() {
                // Real file at legacy path. If canonical doesn't exist, migrate.
                if !canonical.exists() {
                    if let Some(p) = canonical.parent() {
                        std::fs::create_dir_all(p).ok();
                    }
                    std::fs::rename(&legacy, &canonical).with_context(|| {
                        format!("migrating {} -> {}", legacy.display(), canonical.display())
                    })?;
                } else {
                    // Both exist. The legacy real file is what the CLI just
                    // wrote — Claude's atomic /login and token-rotation writes
                    // replace the symlink with a regular file holding the
                    // FRESHEST tokens. The old behavior (back up + discard
                    // legacy, keep canonical) threw away the only copy of the
                    // rotated refresh token: the canonical kept a consumed one,
                    // the next refresh failed, and every session 401'd into
                    // "Please run /login". Adopt the fresher legacy content
                    // into the canonical's resolved target first, THEN back up
                    // + remove legacy.
                    adopt_fresher_legacy(&legacy, &canonical).with_context(|| {
                        format!(
                            "adopting fresher legacy creds {} into {}",
                            legacy.display(),
                            canonical.display()
                        )
                    })?;
                    let backup = legacy.with_extension("json.pre-omega");
                    let _ = std::fs::rename(&legacy, &backup);
                }
            }
        }

        // At this point legacy does not exist. Create symlink if canonical exists.
        if canonical.exists() {
            #[cfg(unix)]
            std::os::unix::fs::symlink(&canonical, &legacy).with_context(|| {
                format!("symlink {} -> {}", legacy.display(), canonical.display())
            })?;
            #[cfg(not(unix))]
            std::fs::copy(&canonical, &legacy)?;
        }
        Ok(())
    }

    /// Reconcile Codex's native `auth.json` with Omega's canonical credential.
    /// A valid, semantically newer native file wins. When semantic timestamps
    /// are not comparable, a strictly newer filesystem mtime decides instead.
    /// The final native path is atomically restored as a symlink to canonical.
    pub fn reconcile_codex(&self) -> Result<CodexReconcileReport> {
        self.reconcile_codex_inner(None)
    }

    /// Reconcile an additional candidate (normally a device-flow backup)
    /// alongside the native and canonical copies. The candidate is promoted only
    /// when it is the only valid copy or is strictly fresher by the same
    /// `last_refresh`-first ordering. Every distinct losing copy is preserved in
    /// the owner-only quarantine before topology is changed.
    pub fn reconcile_codex_candidate(&self, candidate: &Path) -> Result<CodexReconcileReport> {
        self.reconcile_codex_inner(Some(candidate))
    }

    /// Inspect Codex's current native/canonical topology without changing it.
    pub fn codex_topology(&self) -> CodexTopologyReport {
        codex_topology_for_paths(
            &self.codex_home.join("auth.json"),
            &self.active_path("codex"),
        )
    }

    #[cfg(not(unix))]
    fn reconcile_codex_inner(&self, _candidate: Option<&Path>) -> Result<CodexReconcileReport> {
        anyhow::bail!(
            "Codex credential reconciliation is unsupported on this platform: \
             a safe single-copy native/canonical symlink topology requires Unix"
        )
    }

    #[cfg(unix)]
    fn reconcile_codex_inner(&self, candidate: Option<&Path>) -> Result<CodexReconcileReport> {
        let _lock = self.lock_codex_reconciliation()?;
        let native = self.codex_home.join("auth.json");
        let canonical = self.active_path("codex");
        let native_was_linked = path_links_to(&native, &canonical);

        if let Some(parent) = native.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
        if let Some(parent) = canonical.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }

        let mut quarantined = Vec::new();
        let mut action = if native_was_linked {
            CodexReconcileAction::Unchanged
        } else {
            CodexReconcileAction::RepairedNativeLink
        };

        // Codex refreshes auth.json without taking Omega's reconciliation lock.
        // Never rename a symlink over an observed regular file: first move the
        // exact native path to a durable, recognizable sibling, then create the
        // symlink with create-new semantics. A concurrent replacement therefore
        // either becomes the captured file or makes symlink(2) return EEXIST and
        // is captured on the next iteration. No native credential is overwritten.
        for _ in 0..32 {
            let mut pending = pending_native_captures(&native)?;
            pending.extend(capture_native_and_restore_link(&native, &canonical)?);
            pending.sort();
            pending.dedup();

            let canonical_copy = read_codex_copy(&canonical);
            let extra_copy = candidate.map(read_codex_copy);
            let native_copies = pending
                .iter()
                .filter_map(|path| {
                    let metadata = std::fs::symlink_metadata(path).ok()?;
                    if metadata.file_type().is_symlink() {
                        let _ = std::fs::remove_file(path);
                        return None;
                    }
                    Some((path.clone(), read_codex_copy(path)))
                })
                .collect::<Vec<_>>();

            if native_copies.iter().any(|(_, copy)| copy.bytes.is_none()) {
                anyhow::bail!(
                    "cannot restore Codex topology because a captured native credential is unreadable"
                );
            }

            let mut choices = Vec::new();
            if canonical_copy.inspection.is_valid() {
                choices.push((CodexCopySource::Canonical, &canonical_copy));
            }
            for (_, copy) in &native_copies {
                if copy.inspection.is_valid() {
                    choices.push((CodexCopySource::Native, copy));
                }
            }
            if let Some(copy) = extra_copy
                .as_ref()
                .filter(|copy| copy.inspection.is_valid())
            {
                choices.push((CodexCopySource::Candidate, copy));
            }

            let winner = choices.into_iter().reduce(|current, next| {
                if compare_codex_copies(next.1, current.1) == Ordering::Greater {
                    next
                } else {
                    current
                }
            });

            if let Some((winner_source, winner)) = winner {
                let winner_bytes = winner
                    .bytes
                    .as_deref()
                    .context("valid Codex credential had no readable bytes")?;
                let mut quarantined_contents: Vec<Vec<u8>> = Vec::new();

                let copies = std::iter::once(("canonical", &canonical_copy))
                    .chain(native_copies.iter().map(|(_, copy)| ("native", copy)))
                    .chain(extra_copy.as_ref().map(|copy| ("candidate", copy)));
                for (label, copy) in copies {
                    let Some(bytes) = copy.bytes.as_deref() else {
                        continue;
                    };
                    if bytes == winner_bytes
                        || quarantined_contents
                            .iter()
                            .any(|existing| existing.as_slice() == bytes)
                    {
                        continue;
                    }
                    let saved = self.quarantine_codex_copy(label, bytes)?;
                    quarantined_contents.push(bytes.to_vec());
                    quarantined.push(saved);
                }

                if canonical_copy.bytes.as_deref() != Some(winner_bytes) {
                    atomic_replace_resolved(&canonical, winner_bytes)?;
                }
                action = match winner_source {
                    CodexCopySource::Native => CodexReconcileAction::AdoptedNative,
                    CodexCopySource::Candidate => CodexReconcileAction::AdoptedCandidate,
                    CodexCopySource::Canonical => action,
                };
            } else {
                let mut quarantined_contents: Vec<Vec<u8>> = Vec::new();
                let invalid_copies = native_copies
                    .iter()
                    .map(|(_, copy)| ("native-invalid", copy))
                    .chain(extra_copy.as_ref().map(|copy| ("candidate-invalid", copy)));
                for (label, copy) in invalid_copies {
                    if let Some(bytes) = copy.bytes.as_deref() {
                        let canonical_already_preserves_bytes = label != "candidate-invalid"
                            && canonical_copy.bytes.as_deref() == Some(bytes);
                        if canonical_already_preserves_bytes
                            || quarantined_contents
                                .iter()
                                .any(|existing| existing.as_slice() == bytes)
                        {
                            continue;
                        }
                        quarantined.push(self.quarantine_codex_copy(label, bytes)?);
                        quarantined_contents.push(bytes.to_vec());
                    }
                }
                action = CodexReconcileAction::NoValidCredential;
            }

            // A pending capture is removed only after its contents have either
            // won the atomic canonical write or reached owner-only quarantine.
            for (path, _) in native_copies {
                std::fs::remove_file(&path)
                    .with_context(|| format!("removing reconciled capture {}", path.display()))?;
            }

            if path_links_to(&native, &canonical) {
                return Ok(CodexReconcileReport {
                    action,
                    quarantined,
                    topology: self.codex_topology(),
                });
            }
        }

        anyhow::bail!("Codex kept replacing auth.json while Omega restored canonical topology")
    }

    fn lock_codex_reconciliation(&self) -> Result<crate::config::PrivateFileLock> {
        let omega_root = self
            .base_dir
            .parent()
            .context("credential store has no Omega root")?;
        let locks = omega_root.join("locks");
        ensure_owner_only_dir(&locks)?;
        let path = locks.join("codex-credentials.lock");
        crate::config::acquire_private_lock_path(&path)
    }

    fn quarantine_codex_copy(&self, label: &str, bytes: &[u8]) -> Result<PathBuf> {
        let root = self.base_dir.join("quarantine");
        let dir = root.join("codex");
        ensure_owner_only_dir(&root)?;
        ensure_owner_only_dir(&dir)?;

        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        for _ in 0..32 {
            let serial = UNIQUE_FILE_COUNTER.fetch_add(1, AtomicOrdering::Relaxed);
            let filename = format!(
                "{}-{}-{}-{}.json",
                stamp,
                std::process::id(),
                serial,
                sanitize(label)
            );
            let path = dir.join(filename);
            match atomic_create_new_secret(&path, bytes) {
                Ok(()) => return Ok(path),
                Err(e)
                    if e.downcast_ref::<std::io::Error>()
                        .is_some_and(|io| io.kind() == std::io::ErrorKind::AlreadyExists) =>
                {
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
        anyhow::bail!("could not allocate a unique Codex quarantine path")
    }

    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }
}

/// Both the legacy path (a real file the CLI just wrote) and the canonical
/// store exist. Copy the legacy content into the canonical's RESOLVED target
/// when it is valid JSON and at least as fresh (mtime) as the target — atomic
/// temp+rename in the target's directory, preserving the target's mode (e.g.
/// 0660 group-shared on multi-user hosts), never clobbering the
/// canonical→shared symlink chain. A stale or corrupt legacy file (e.g. a
/// truncated write) is skipped: the caller backs it up without adoption.
/// On error the caller must leave the legacy file in place — it may hold the
/// only copy of the freshest rotated refresh token.
fn adopt_fresher_legacy(legacy: &Path, canonical: &Path) -> Result<()> {
    let bytes = std::fs::read(legacy).with_context(|| format!("reading {}", legacy.display()))?;
    // Corrupt/truncated legacy must never clobber a good canonical.
    if serde_json::from_slice::<Value>(&bytes).is_err() {
        return Ok(());
    }
    let target = std::fs::canonicalize(canonical).unwrap_or_else(|_| canonical.to_path_buf());
    // Freshness: the CLI's write is newer-or-equal → adopt. An older legacy
    // (e.g. a stray leftover) must not roll the canonical back.
    let legacy_mtime = std::fs::metadata(legacy).and_then(|m| m.modified()).ok();
    let target_mtime = std::fs::metadata(&target).and_then(|m| m.modified()).ok();
    if let (Some(l), Some(t)) = (legacy_mtime, target_mtime) {
        if l < t {
            return Ok(());
        }
    }
    let staged = target.with_extension("json.adopt.tmp");
    let _ = std::fs::remove_file(&staged); // clear any stale temp
    std::fs::write(&staged, &bytes).with_context(|| format!("writing {}", staged.display()))?;
    // Preserve the target's existing mode (0660 group-shared on multi-user
    // hosts); fall back to owner-only. chmod the TEMP we own, never the
    // target (which may be root-owned → EPERM), then rename(2) — readers
    // always see the old-or-new complete file, never a truncated one.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&target)
            .map(|m| m.permissions().mode() & 0o777)
            .unwrap_or(0o600);
        let _ = std::fs::set_permissions(&staged, std::fs::Permissions::from_mode(mode));
    }
    if let Err(e) = std::fs::rename(&staged, &target) {
        let _ = std::fs::remove_file(&staged); // don't leak the temp
        return Err(e)
            .with_context(|| format!("renaming {} -> {}", staged.display(), target.display()));
    }
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CodexCopySource {
    Canonical,
    Native,
    Candidate,
}

struct CodexCopy {
    bytes: Option<Vec<u8>>,
    inspection: CodexCredentialInspection,
}

/// Resolve Codex's native home. A non-empty `CODEX_HOME` wins; otherwise use
/// the native CLI default under the current user's home.
pub fn codex_home_dir() -> PathBuf {
    if let Ok(path) = std::env::var("CODEX_HOME") {
        if !path.is_empty() {
            return PathBuf::from(path);
        }
    }
    dirs::home_dir().unwrap_or_default().join(".codex")
}

/// Inspect a Codex credential file without returning or formatting any secret.
pub fn inspect_codex_credential(path: &Path) -> CodexCredentialInspection {
    read_codex_copy(path).inspection
}

/// Whether `candidate` is structurally valid, distinct, and strictly fresher
/// than `baseline`. Two parsed `last_refresh` values outrank filesystem mtime;
/// one-sided or absent semantic timestamps use a strictly newer mtime. A valid
/// candidate is fresh relative to a missing or invalid baseline.
pub fn codex_credential_fresh_relative(candidate: &Path, baseline: &Path) -> bool {
    let candidate = read_codex_copy(candidate);
    if !candidate.inspection.is_valid() {
        return false;
    }
    let baseline = read_codex_copy(baseline);
    if !baseline.inspection.is_valid() {
        return true;
    }
    if candidate.bytes == baseline.bytes {
        return false;
    }
    match (
        &candidate.inspection.last_refresh,
        &baseline.inspection.last_refresh,
    ) {
        (Some(candidate), Some(baseline)) => candidate > baseline,
        _ => matches!(
            (
                candidate.inspection.modified,
                baseline.inspection.modified,
            ),
            (Some(candidate), Some(baseline)) if candidate > baseline
        ),
    }
}

fn read_codex_copy(path: &Path) -> CodexCopy {
    let modified = std::fs::metadata(path).and_then(|m| m.modified()).ok();
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) => {
            let validity = if e.kind() == std::io::ErrorKind::NotFound {
                CodexCredentialValidity::Missing
            } else {
                CodexCredentialValidity::Invalid
            };
            return CodexCopy {
                bytes: None,
                inspection: CodexCredentialInspection {
                    validity,
                    last_refresh: None,
                    modified,
                },
            };
        }
    };

    let Ok(value) = serde_json::from_slice::<Value>(&bytes) else {
        return CodexCopy {
            bytes: Some(bytes),
            inspection: CodexCredentialInspection {
                validity: CodexCredentialValidity::Invalid,
                last_refresh: None,
                modified,
            },
        };
    };

    let last_refresh = match value.get("last_refresh") {
        None | Some(Value::Null) => None,
        Some(Value::String(raw)) => match DateTime::parse_from_rfc3339(raw) {
            Ok(timestamp) => Some(timestamp.with_timezone(&Utc)),
            Err(_) => {
                return CodexCopy {
                    bytes: Some(bytes),
                    inspection: CodexCredentialInspection {
                        validity: CodexCredentialValidity::Invalid,
                        last_refresh: None,
                        modified,
                    },
                };
            }
        },
        Some(_) => {
            return CodexCopy {
                bytes: Some(bytes),
                inspection: CodexCredentialInspection {
                    validity: CodexCredentialValidity::Invalid,
                    last_refresh: None,
                    modified,
                },
            };
        }
    };

    let non_empty = |value: Option<&Value>| {
        value
            .and_then(Value::as_str)
            .is_some_and(|s| !s.trim().is_empty())
    };
    let tokens_valid = value
        .get("tokens")
        .and_then(Value::as_object)
        .is_some_and(|tokens| {
            ["id_token", "access_token", "refresh_token"]
                .iter()
                .all(|field| non_empty(tokens.get(*field)))
        });
    let api_key_valid = non_empty(value.get("OPENAI_API_KEY"));
    let mode = value.get("auth_mode").and_then(Value::as_str);
    let structurally_valid = match mode {
        Some("chatgpt") => tokens_valid,
        Some("apikey") | Some("api_key") => api_key_valid,
        _ => tokens_valid || api_key_valid,
    };

    CodexCopy {
        bytes: Some(bytes),
        inspection: CodexCredentialInspection {
            validity: if structurally_valid {
                CodexCredentialValidity::Valid
            } else {
                CodexCredentialValidity::Invalid
            },
            last_refresh,
            modified,
        },
    }
}

fn compare_codex_copies(left: &CodexCopy, right: &CodexCopy) -> Ordering {
    if left.bytes == right.bytes {
        return Ordering::Equal;
    }
    let modified = || match (left.inspection.modified, right.inspection.modified) {
        (Some(left), Some(right)) => left.cmp(&right),
        _ => Ordering::Equal,
    };
    match (
        &left.inspection.last_refresh,
        &right.inspection.last_refresh,
    ) {
        (Some(left), Some(right)) => {
            let semantic = left.cmp(right);
            if semantic != Ordering::Equal {
                return semantic;
            }
            modified()
        }
        _ => modified(),
    }
}

fn codex_topology_for_paths(native: &Path, canonical: &Path) -> CodexTopologyReport {
    let native_meta = std::fs::symlink_metadata(native).ok();
    let canonical_meta = std::fs::symlink_metadata(canonical).ok();
    let native_exists = native_meta.is_some();
    let canonical_exists = canonical_meta.is_some();
    let native_is_symlink = native_meta
        .as_ref()
        .is_some_and(|metadata| metadata.file_type().is_symlink());
    let native_links_to_canonical = path_links_to(native, canonical);
    CodexTopologyReport {
        native_path: native.to_path_buf(),
        canonical_path: canonical.to_path_buf(),
        native_exists,
        canonical_exists,
        native_is_symlink,
        native_links_to_canonical,
        native_validity: inspect_codex_credential(native).validity,
        canonical_validity: inspect_codex_credential(canonical).validity,
        split: native_exists && canonical_exists && !native_links_to_canonical,
    }
}

fn path_links_to(path: &Path, expected: &Path) -> bool {
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
        return false;
    };
    if !metadata.file_type().is_symlink() {
        return false;
    }
    let Ok(target) = std::fs::read_link(path) else {
        return false;
    };
    let resolved_target = if target.is_absolute() {
        target
    } else {
        path.parent().unwrap_or_else(|| Path::new(".")).join(target)
    };
    let expected = absolute_path(expected);
    absolute_path(&resolved_target) == expected
}

fn absolute_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }
}

fn resolved_write_target(path: &Path) -> Result<PathBuf> {
    let mut current = absolute_path(path);
    for _ in 0..32 {
        let metadata = match std::fs::symlink_metadata(&current) {
            Ok(metadata) => metadata,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(current),
            Err(e) => {
                return Err(e).with_context(|| format!("inspecting {}", current.display()));
            }
        };
        if !metadata.file_type().is_symlink() {
            return Ok(current);
        }
        let target = std::fs::read_link(&current)
            .with_context(|| format!("reading symlink {}", current.display()))?;
        current = if target.is_absolute() {
            target
        } else {
            current
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(target)
        };
    }
    anyhow::bail!("Codex canonical credential symlink chain is too deep")
}

fn unique_sibling(path: &Path, suffix: &str) -> PathBuf {
    let serial = UNIQUE_FILE_COUNTER.fetch_add(1, AtomicOrdering::Relaxed);
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("credential");
    path.with_file_name(format!(
        ".{}.{}.{}.{}",
        name,
        suffix,
        std::process::id(),
        serial
    ))
}

fn secret_open_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
}

fn atomic_replace_resolved(path: &Path, bytes: &[u8]) -> Result<()> {
    let target = resolved_write_target(path)?;
    let parent = target
        .parent()
        .context("Codex canonical credential has no parent directory")?;
    std::fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    let staged = unique_sibling(&target, "omega-adopt");
    let result = (|| -> Result<()> {
        let mut file = secret_open_options()
            .open(&staged)
            .with_context(|| format!("creating {}", staged.display()))?;
        file.write_all(bytes)
            .with_context(|| format!("writing {}", staged.display()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(std::fs::Permissions::from_mode(0o600))
                .with_context(|| format!("setting owner-only mode on {}", staged.display()))?;
        }
        file.sync_all()
            .with_context(|| format!("syncing {}", staged.display()))?;
        drop(file);
        std::fs::rename(&staged, &target)
            .with_context(|| format!("renaming {} -> {}", staged.display(), target.display()))?;
        sync_parent(parent);
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&staged);
    }
    result
}

fn atomic_create_new_secret(path: &Path, bytes: &[u8]) -> Result<()> {
    if std::fs::symlink_metadata(path).is_ok() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "quarantine path already exists",
        )
        .into());
    }
    let parent = path.parent().context("quarantine path has no parent")?;
    let staged = unique_sibling(path, "omega-quarantine");
    let result = (|| -> Result<()> {
        let mut file = secret_open_options()
            .open(&staged)
            .with_context(|| format!("creating {}", staged.display()))?;
        file.write_all(bytes)
            .with_context(|| format!("writing {}", staged.display()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(std::fs::Permissions::from_mode(0o600))
                .with_context(|| format!("setting owner-only mode on {}", staged.display()))?;
        }
        file.sync_all()
            .with_context(|| format!("syncing {}", staged.display()))?;
        drop(file);
        if std::fs::symlink_metadata(path).is_ok() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "quarantine path already exists",
            )
            .into());
        }
        std::fs::rename(&staged, path)
            .with_context(|| format!("renaming {} -> {}", staged.display(), path.display()))?;
        sync_parent(parent);
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&staged);
    }
    result
}

fn pending_native_captures(native: &Path) -> Result<Vec<PathBuf>> {
    let parent = native
        .parent()
        .context("Codex native auth path has no parent")?;
    let filename = native
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("auth.json");
    let prefix = format!(".{filename}.omega-capture.");
    let mut captures: Vec<PathBuf> = Vec::new();
    for entry in
        std::fs::read_dir(parent).with_context(|| format!("reading {}", parent.display()))?
    {
        let entry = entry.with_context(|| format!("reading an entry in {}", parent.display()))?;
        if entry.file_name().to_string_lossy().starts_with(&prefix) {
            captures.push(entry.path());
        }
    }
    Ok(captures)
}

/// Restore `native` without ever rename-overwriting a concurrent Codex write.
/// Regular files are atomically moved to recognizable sibling captures. If a
/// writer recreates `native` before symlink creation, EEXIST makes us capture
/// that file too. Stale captures are deliberately discoverable by the next
/// reconciliation if this process crashes before adoption/quarantine.
fn capture_native_and_restore_link(native: &Path, canonical: &Path) -> Result<Vec<PathBuf>> {
    let parent = native
        .parent()
        .context("Codex native auth path has no parent")?;
    std::fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    #[cfg(unix)]
    {
        let mut captures: Vec<PathBuf> = Vec::new();
        let canonical = absolute_path(canonical);
        for _ in 0..32 {
            if path_links_to(native, &canonical) {
                sync_parent(parent);
                return Ok(captures);
            }
            match std::fs::symlink_metadata(native) {
                Ok(metadata) => {
                    if metadata.file_type().is_file() {
                        let _ =
                            crate::config::read_private_optional(native)?.with_context(|| {
                                format!(
                                    "native Codex credential disappeared before capture: {}",
                                    native.display()
                                )
                            })?;
                    }
                    let capture = unique_sibling(native, "omega-capture");
                    match std::fs::rename(native, &capture) {
                        Ok(()) => {
                            sync_parent(parent);
                            captures.push(capture);
                        }
                        Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                        Err(error) => {
                            return Err(error).with_context(|| {
                                format!("capturing native Codex credential {}", native.display())
                            })
                        }
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    match std::os::unix::fs::symlink(&canonical, native) {
                        Ok(()) => {
                            sync_parent(parent);
                            return Ok(captures);
                        }
                        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                        Err(error) => {
                            return Err(error).with_context(|| {
                                format!(
                                    "creating symlink {} -> {}",
                                    native.display(),
                                    canonical.display()
                                )
                            })
                        }
                    }
                }
                Err(error) => {
                    return Err(error).with_context(|| format!("inspecting {}", native.display()))
                }
            }
        }
        anyhow::bail!("Codex kept recreating auth.json while Omega restored its symlink")
    }
    #[cfg(not(unix))]
    {
        let _ = canonical;
        anyhow::bail!(
            "Codex credential reconciliation is unsupported on this platform: \
             refusing to replace the native credential without symlink support"
        )
    }
}

fn sync_parent(parent: &Path) {
    #[cfg(unix)]
    {
        if let Ok(dir) = std::fs::File::open(parent) {
            let _ = dir.sync_all();
        }
    }
    #[cfg(not(unix))]
    let _ = parent;
}

/// Map a provider id to the legacy path its CLI expects.
/// Returns None for providers without a fixed legacy location (api-key only).
pub fn legacy_path_for(provider: &str) -> Option<PathBuf> {
    match provider {
        "codex" => Some(codex_home_dir().join("auth.json")),
        "claude" => Some(dirs::home_dir()?.join(".claude").join(".credentials.json")),
        // Legacy Gemini CLI file location. Gemini 0.56+ may migrate this into
        // its native hybrid/keyring store; Omega does not try to symlink that
        // modern store, but diagnostics must still name the real legacy path.
        "gemini" => Some(dirs::home_dir()?.join(".gemini").join("oauth_creds.json")),
        // api-key providers store config in providers.toml; no legacy path.
        _ => None,
    }
}

/// Convenience: list providers known to ProvidersConfig.
pub fn known_providers() -> Vec<&'static str> {
    ProvidersConfig::all_providers()
}

/// Delegates to the single canonical resolver (config::omega_dir) so credentials
/// honor $OMEGA_DIR + the consolidated ~/OmegaOS/System layout, instead of
/// hardcoding ~/.omega and diverging from where config.toml actually lives.
fn omega_dir() -> PathBuf {
    crate::config::omega_dir()
}

fn ensure_owner_only_dir(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path).with_context(|| format!("creating {}", path.display()))?;
    let before = std::fs::symlink_metadata(path)
        .with_context(|| format!("inspecting credential directory {}", path.display()))?;
    if !before.file_type().is_dir() {
        anyhow::bail!(
            "credential directory {} is not a real directory",
            path.display()
        );
    }
    let directory = std::fs::File::open(path)
        .with_context(|| format!("opening credential directory {}", path.display()))?;
    let opened = directory
        .metadata()
        .with_context(|| format!("inspecting opened credential directory {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        if before.dev() != opened.dev() || before.ino() != opened.ino() {
            anyhow::bail!(
                "credential directory {} changed identity while being opened",
                path.display()
            );
        }
        if opened.uid() != crate::config::effective_uid() {
            anyhow::bail!(
                "credential directory {} is owned by uid {}, current uid is {}",
                path.display(),
                opened.uid(),
                crate::config::effective_uid()
            );
        }
        directory
            .set_permissions(std::fs::Permissions::from_mode(0o700))
            .with_context(|| format!("setting owner-only mode on {}", path.display()))?;
        let after = std::fs::symlink_metadata(path)
            .with_context(|| format!("re-checking credential directory {}", path.display()))?;
        if after.dev() != opened.dev() || after.ino() != opened.ino() {
            anyhow::bail!(
                "credential directory {} changed identity during validation",
                path.display()
            );
        }
    }
    Ok(())
}

fn validate_provider_id(provider: &str) -> Result<()> {
    if !ProvidersConfig::is_known(provider)
        || provider.is_empty()
        || provider.len() > 32
        || !provider.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
    {
        anyhow::bail!("invalid credential provider id {provider:?}");
    }
    Ok(())
}

fn safe_provider_component(provider: &str) -> &str {
    if validate_provider_id(provider).is_ok() {
        provider
    } else {
        "__invalid_provider"
    }
}

fn validate_account_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name.len() > 64
        || !name.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '_' || character == '-'
        })
    {
        anyhow::bail!(
            "invalid credential account name {name:?}; use 1-64 ASCII letters, digits, '_' or '-'"
        );
    }
    Ok(())
}

fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::ffi::OsString;
    use std::sync::{Mutex, MutexGuard};

    // fresh_store mutates the process-global HOME, and cargo runs tests in
    // parallel, so without this lock two tests clobber each other's HOME and a
    // legacy symlink can land in the wrong tmp dir. Serialize HOME-touching tests.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct LockedEnvironment {
        _guard: MutexGuard<'static, ()>,
        previous: Vec<(&'static str, Option<OsString>)>,
    }

    impl LockedEnvironment {
        fn set(values: Vec<(&'static str, Option<OsString>)>) -> Self {
            let guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            let previous = values
                .iter()
                .map(|(key, _)| (*key, std::env::var_os(key)))
                .collect();
            for (key, value) in values {
                match value {
                    Some(value) => std::env::set_var(key, value),
                    None => std::env::remove_var(key),
                }
            }
            Self {
                _guard: guard,
                previous,
            }
        }
    }

    impl Drop for LockedEnvironment {
        fn drop(&mut self) {
            for (key, value) in self.previous.iter().rev() {
                match value {
                    Some(value) => std::env::set_var(key, value),
                    None => std::env::remove_var(key),
                }
            }
        }
    }

    #[test]
    fn omega_dir_override_relocates_credentials() {
        // $OMEGA_DIR must relocate the WHOLE secret tree, not just config.toml.
        // credentials now share config::omega_dir() (was: a hardcoded ~/.omega),
        // so setting the override lands the credential store under it.
        let tmp = tempfile::tempdir().unwrap();
        let codex_home = tmp.path().join("codex-home");
        let _env = LockedEnvironment::set(vec![
            ("OMEGA_DIR", Some(tmp.path().as_os_str().to_owned())),
            ("CODEX_HOME", Some(codex_home.as_os_str().to_owned())),
        ]);
        let store = CredentialStore::new().unwrap();
        assert!(
            store.base_dir().starts_with(tmp.path()),
            "credentials base {:?} must be under $OMEGA_DIR {:?}",
            store.base_dir(),
            tmp.path()
        );
    }

    #[cfg(unix)]
    #[test]
    fn secret_writes_are_owner_only_0600() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        let (store, _env) = fresh_store(tmp.path());
        // write() — the active-creds path
        store
            .write("codex", &json!({"refresh_token": "sk-secret-xyz"}))
            .unwrap();
        let mode = std::fs::metadata(store.active_path("codex"))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(
            mode & 0o077,
            0,
            "active creds must be 0600, got {:o}",
            mode & 0o777
        );
        // save_as_account() — the saved-profile copy
        store.save_as_account("codex", "work").unwrap();
        let amode = std::fs::metadata(store.account_path("codex", "work"))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(
            amode & 0o077,
            0,
            "saved account must be 0600, got {:o}",
            amode & 0o777
        );
        // switch_account() — the copy-over-active path
        store
            .write("codex", &json!({"refresh_token": "v2"}))
            .unwrap();
        store.switch_account("codex", "work").unwrap();
        let smode = std::fs::metadata(store.active_path("codex"))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(
            smode & 0o077,
            0,
            "switched creds must be 0600, got {:o}",
            smode & 0o777
        );
    }

    fn fresh_store(tmp: &Path) -> (CredentialStore, LockedEnvironment) {
        let omega = tmp.join(".omega");
        let codex_home = tmp.join(".codex");
        let env = LockedEnvironment::set(vec![
            ("HOME", Some(tmp.as_os_str().to_owned())),
            ("OMEGA_DIR", Some(omega.as_os_str().to_owned())),
            ("CODEX_HOME", Some(codex_home.as_os_str().to_owned())),
        ]);
        let native = legacy_path_for("codex").unwrap();
        assert!(
            native.starts_with(&codex_home),
            "native Codex credential path {:?} must stay inside isolated CODEX_HOME {:?}",
            native,
            codex_home
        );
        (CredentialStore::new().unwrap(), env)
    }

    fn codex_store(tmp: &Path) -> (CredentialStore, PathBuf) {
        let codex_home = tmp.join("native-codex");
        let omega = tmp.join("omega");
        let store = CredentialStore::from_roots(&omega, &codex_home).unwrap();
        (store, codex_home)
    }

    fn codex_credential(last_refresh: &str, marker: &str) -> Value {
        json!({
            "auth_mode": "chatgpt",
            "tokens": {
                "id_token": format!("id-{marker}"),
                "access_token": format!("access-{marker}"),
                "refresh_token": format!("refresh-{marker}")
            },
            "last_refresh": last_refresh
        })
    }

    fn write_codex(path: &Path, last_refresh: &str, marker: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(
            path,
            serde_json::to_vec(&codex_credential(last_refresh, marker)).unwrap(),
        )
        .unwrap();
    }

    fn write_codex_api_key(path: &Path, marker: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(
            path,
            serde_json::to_vec(&json!({
                "auth_mode": "apikey",
                "OPENAI_API_KEY": format!("test-{marker}")
            }))
            .unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn write_read_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let (store, _env) = fresh_store(tmp.path());
        let data = json!({"apiKey": "sk-abc123", "model": "gpt-5"});
        store.write("codex", &data).unwrap();
        let read = store.read("codex").unwrap();
        assert_eq!(read, data);
    }

    #[test]
    fn provider_and_account_components_cannot_escape_or_alias() {
        let tmp = tempfile::tempdir().unwrap();
        let (store, _) = codex_store(tmp.path());
        let outside = tmp.path().join("outside.json");

        assert!(store.write("../outside", &json!({"secret": true})).is_err());
        assert!(!outside.exists());
        store.write("codex", &json!({"secret": true})).unwrap();
        for invalid in ["../work", "work/name", "work name", "", "."] {
            assert!(
                store.save_as_account("codex", invalid).is_err(),
                "{invalid:?}"
            );
            assert!(
                store.switch_account("codex", invalid).is_err(),
                "{invalid:?}"
            );
        }
        assert!(store.list_accounts("../outside").is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn credential_authorities_reject_symlinks_hardlinks_and_symlinked_roots() {
        use std::os::unix::fs::{symlink, PermissionsExt};

        let tmp = tempfile::tempdir().unwrap();
        let (store, _) = codex_store(tmp.path());
        let authority = store.active_path("codex");
        let victim = tmp.path().join("victim.json");
        std::fs::write(&victim, r#"{"do_not_touch":true}"#).unwrap();
        let victim_mode = std::fs::metadata(&victim).unwrap().permissions().mode() & 0o777;

        symlink(&victim, &authority).unwrap();
        assert!(store.read("codex").is_err());
        assert!(store.write("codex", &json!({"replacement": true})).is_err());
        assert_eq!(
            std::fs::read_to_string(&victim).unwrap(),
            r#"{"do_not_touch":true}"#
        );
        assert_eq!(
            std::fs::metadata(&victim).unwrap().permissions().mode() & 0o777,
            victim_mode
        );

        std::fs::remove_file(&authority).unwrap();
        std::fs::hard_link(&victim, &authority).unwrap();
        assert!(store.read("codex").is_err());
        assert!(store.write("codex", &json!({"replacement": true})).is_err());
        assert_eq!(
            std::fs::read_to_string(&victim).unwrap(),
            r#"{"do_not_touch":true}"#
        );

        let omega = tmp.path().join("symlink-root");
        let external = tmp.path().join("external-credentials");
        std::fs::create_dir_all(&omega).unwrap();
        std::fs::create_dir_all(&external).unwrap();
        symlink(&external, omega.join("credentials")).unwrap();
        assert!(CredentialStore::from_roots(&omega, &tmp.path().join("codex-home")).is_err());
    }

    #[test]
    fn concurrent_credential_writes_remain_complete_json() {
        let tmp = tempfile::tempdir().unwrap();
        let (store, _) = codex_store(tmp.path());
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(3));
        let mut writers = Vec::new();
        for marker in ["first", "second"] {
            let store = store.clone();
            let barrier = barrier.clone();
            writers.push(std::thread::spawn(move || {
                barrier.wait();
                store
                    .write(
                        "codex",
                        &json!({"marker": marker, "payload": "x".repeat(4096)}),
                    )
                    .unwrap();
            }));
        }
        barrier.wait();
        for writer in writers {
            writer.join().unwrap();
        }
        let final_value = store.read("codex").unwrap();
        assert!(matches!(
            final_value.get("marker").and_then(Value::as_str),
            Some("first" | "second")
        ));
        assert_eq!(
            final_value
                .get("payload")
                .and_then(Value::as_str)
                .map(str::len),
            Some(4096)
        );
    }

    #[cfg(unix)]
    #[test]
    fn codex_reconciliation_lock_rejects_symlink_and_hardlink_poisoning() {
        use std::os::unix::fs::{symlink, PermissionsExt};

        let tmp = tempfile::tempdir().unwrap();
        let (store, _) = codex_store(tmp.path());
        let locks = store.base_dir.parent().unwrap().join("locks");
        std::fs::create_dir_all(&locks).unwrap();
        let lock = locks.join("codex-credentials.lock");
        let victim = tmp.path().join("lock-victim");
        std::fs::write(&victim, "unchanged").unwrap();
        let mode = std::fs::metadata(&victim).unwrap().permissions().mode() & 0o777;

        symlink(&victim, &lock).unwrap();
        assert!(store.reconcile_codex().is_err());
        assert_eq!(std::fs::read_to_string(&victim).unwrap(), "unchanged");
        assert_eq!(
            std::fs::metadata(&victim).unwrap().permissions().mode() & 0o777,
            mode
        );

        std::fs::remove_file(&lock).unwrap();
        std::fs::hard_link(&victim, &lock).unwrap();
        assert!(store.reconcile_codex().is_err());
        assert_eq!(std::fs::read_to_string(&victim).unwrap(), "unchanged");
        assert_eq!(
            std::fs::metadata(&victim).unwrap().permissions().mode() & 0o777,
            mode
        );
    }

    #[test]
    fn save_and_switch_account() {
        let tmp = tempfile::tempdir().unwrap();
        let (store, _env) = fresh_store(tmp.path());
        store.write("codex", &json!({"k": "v1"})).unwrap();
        store.save_as_account("codex", "work").unwrap();
        store.write("codex", &json!({"k": "v2"})).unwrap();
        store.save_as_account("codex", "perso").unwrap();
        let mut accs = store.list_accounts("codex");
        accs.sort();
        assert_eq!(accs, vec!["perso", "work"]);

        store.switch_account("codex", "work").unwrap();
        let read = store.read("codex").unwrap();
        assert_eq!(read["k"], "v1");
    }

    #[test]
    fn ensure_legacy_symlink_creates_link() {
        let tmp = tempfile::tempdir().unwrap();
        let (store, _env) = fresh_store(tmp.path());
        store
            .write("claude", &json!({"claudeAiOauth": {}}))
            .unwrap();
        store.ensure_legacy_symlink("claude").unwrap();
        let legacy = tmp.path().join(".claude").join(".credentials.json");
        let meta = std::fs::symlink_metadata(&legacy).unwrap();
        assert!(meta.file_type().is_symlink());
    }

    #[cfg(unix)]
    #[test]
    fn ensure_legacy_symlink_codex_uses_canonical_reconciler() {
        let tmp = tempfile::tempdir().unwrap();
        let (store, _env) = fresh_store(tmp.path());
        write_codex(
            &store.active_path("codex"),
            "2026-07-21T00:00:00Z",
            "canonical",
        );
        store.ensure_legacy_symlink("codex").unwrap();
        let topology = store.codex_topology();
        assert!(topology.native_links_to_canonical);
        assert_eq!(topology.canonical_validity, CodexCredentialValidity::Valid);
    }

    #[test]
    fn ensure_legacy_migrates_existing_file() {
        let tmp = tempfile::tempdir().unwrap();
        // Pre-create a real file at the legacy path with NO canonical file.
        std::fs::create_dir_all(tmp.path().join(".claude")).unwrap();
        let legacy = tmp.path().join(".claude").join(".credentials.json");
        std::fs::write(&legacy, "{\"k\":1}").unwrap();
        let (store, _env) = fresh_store(tmp.path());
        store.ensure_legacy_symlink("claude").unwrap();
        // canonical should now exist
        assert!(store.active_path("claude").exists());
        // legacy should now be a symlink
        let meta = std::fs::symlink_metadata(&legacy).unwrap();
        assert!(meta.file_type().is_symlink());
    }

    #[cfg(unix)]
    #[test]
    fn ensure_legacy_adopts_fresher_real_file_through_symlink_chain() {
        // Prod shape: canonical claude.json is itself a symlink to a shared
        // file. Claude's atomic /login replaced the legacy symlink with a real
        // file holding FRESH tokens. ensure_legacy_symlink must push that
        // content into the shared target (through the chain, keeping the
        // canonical symlink intact) instead of discarding it — discarding was
        // the root cause of the recurring "Please run /login" 401 loop.
        let tmp = tempfile::tempdir().unwrap();
        let (store, _env) = fresh_store(tmp.path());
        // shared file with OLD creds + canonical symlink pointing at it
        let shared = tmp.path().join("shared-credentials.json");
        std::fs::write(&shared, "{\"claudeAiOauth\":{\"refreshToken\":\"OLD\"}}").unwrap();
        let canonical = store.active_path("claude");
        std::os::unix::fs::symlink(&shared, &canonical).unwrap();
        // legacy real file with FRESH creds (what Claude just wrote)
        std::fs::create_dir_all(tmp.path().join(".claude")).unwrap();
        let legacy = tmp.path().join(".claude").join(".credentials.json");
        std::fs::write(&legacy, "{\"claudeAiOauth\":{\"refreshToken\":\"FRESH\"}}").unwrap();

        store.ensure_legacy_symlink("claude").unwrap();

        // shared target adopted the fresh content
        let shared_content = std::fs::read_to_string(&shared).unwrap();
        assert!(
            shared_content.contains("FRESH"),
            "shared must hold the fresh token, got: {shared_content}"
        );
        // canonical→shared symlink untouched
        assert!(std::fs::symlink_metadata(&canonical)
            .unwrap()
            .file_type()
            .is_symlink());
        // legacy is a symlink again, and a backup of the real file exists
        assert!(std::fs::symlink_metadata(&legacy)
            .unwrap()
            .file_type()
            .is_symlink());
        assert!(legacy.with_extension("json.pre-omega").exists());
    }

    #[cfg(unix)]
    #[test]
    fn ensure_legacy_skips_corrupt_legacy_file() {
        // A truncated/corrupt legacy file must never clobber good canonical
        // creds — it gets backed up and re-linked, canonical stays intact.
        let tmp = tempfile::tempdir().unwrap();
        let (store, _env) = fresh_store(tmp.path());
        store
            .write(
                "claude",
                &json!({"claudeAiOauth": {"refreshToken": "GOOD"}}),
            )
            .unwrap();
        std::fs::create_dir_all(tmp.path().join(".claude")).unwrap();
        let legacy = tmp.path().join(".claude").join(".credentials.json");
        std::fs::write(&legacy, "not-json{{{").unwrap();

        store.ensure_legacy_symlink("claude").unwrap();

        let canonical_content = std::fs::read_to_string(store.active_path("claude")).unwrap();
        assert!(
            canonical_content.contains("GOOD"),
            "canonical must keep good creds, got: {canonical_content}"
        );
        assert!(std::fs::symlink_metadata(&legacy)
            .unwrap()
            .file_type()
            .is_symlink());
    }

    #[test]
    fn codex_home_override_controls_native_auth_path() {
        let tmp = tempfile::tempdir().unwrap();
        let expected = tmp.path().join("alternate-codex");
        let _env =
            LockedEnvironment::set(vec![("CODEX_HOME", Some(expected.as_os_str().to_owned()))]);

        assert_eq!(codex_home_dir(), expected);
        assert_eq!(legacy_path_for("codex"), Some(expected.join("auth.json")));
    }

    #[cfg(unix)]
    #[test]
    fn codex_reconcile_adopts_newer_native_and_quarantines_old_canonical() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::tempdir().unwrap();
        let (store, codex_home) = codex_store(tmp.path());
        let canonical = store.active_path("codex");
        let native = codex_home.join("auth.json");
        write_codex(&canonical, "2026-07-20T10:00:00Z", "canonical-old");
        write_codex(&native, "2026-07-21T10:00:00Z", "native-new");
        std::fs::set_permissions(&canonical, std::fs::Permissions::from_mode(0o644)).unwrap();

        let before = store.codex_topology();
        assert!(before.split);
        let report = store.reconcile_codex().unwrap();

        assert_eq!(report.action, CodexReconcileAction::AdoptedNative);
        assert_eq!(report.quarantined.len(), 1);
        assert_eq!(
            inspect_codex_credential(&canonical).last_refresh,
            DateTime::parse_from_rfc3339("2026-07-21T10:00:00Z")
                .ok()
                .map(|v| v.with_timezone(&Utc))
        );
        assert_eq!(
            inspect_codex_credential(&report.quarantined[0]).last_refresh,
            DateTime::parse_from_rfc3339("2026-07-20T10:00:00Z")
                .ok()
                .map(|v| v.with_timezone(&Utc))
        );
        assert!(report.topology.native_links_to_canonical);
        assert!(!report.topology.split);
        assert_eq!(
            std::fs::metadata(&canonical).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

    #[cfg(unix)]
    #[test]
    fn codex_reconcile_never_lets_newer_mtime_override_newer_last_refresh() {
        let tmp = tempfile::tempdir().unwrap();
        let (store, codex_home) = codex_store(tmp.path());
        let canonical = store.active_path("codex");
        let native = codex_home.join("auth.json");
        write_codex(&canonical, "2026-07-22T10:00:00Z", "canonical-new");
        std::thread::sleep(std::time::Duration::from_millis(5));
        write_codex(&native, "2026-07-21T10:00:00Z", "native-old");

        let report = store.reconcile_codex().unwrap();

        assert_eq!(report.action, CodexReconcileAction::RepairedNativeLink);
        assert_eq!(
            inspect_codex_credential(&canonical).last_refresh,
            DateTime::parse_from_rfc3339("2026-07-22T10:00:00Z")
                .ok()
                .map(|v| v.with_timezone(&Utc))
        );
        assert_eq!(report.quarantined.len(), 1);
        assert_eq!(
            inspect_codex_credential(&report.quarantined[0]).last_refresh,
            DateTime::parse_from_rfc3339("2026-07-21T10:00:00Z")
                .ok()
                .map(|v| v.with_timezone(&Utc))
        );
        assert!(report.topology.native_links_to_canonical);
    }

    #[cfg(unix)]
    #[test]
    fn codex_reconcile_uses_mtime_when_only_one_copy_has_last_refresh() {
        let tmp = tempfile::tempdir().unwrap();
        let (store, codex_home) = codex_store(tmp.path());
        let canonical = store.active_path("codex");
        let native = codex_home.join("auth.json");
        write_codex(&canonical, "2026-07-22T10:00:00Z", "canonical-old");
        std::thread::sleep(std::time::Duration::from_millis(10));
        write_codex_api_key(&native, "native-new");

        let report = store.reconcile_codex().unwrap();

        assert_eq!(report.action, CodexReconcileAction::AdoptedNative);
        assert_eq!(
            inspect_codex_credential(&canonical).last_refresh,
            None,
            "the newer API-key credential should win by mtime"
        );
        assert_eq!(report.quarantined.len(), 1);
        assert!(report.topology.native_links_to_canonical);
    }

    #[cfg(unix)]
    #[test]
    fn codex_reconcile_recovers_a_crash_left_native_capture() {
        let tmp = tempfile::tempdir().unwrap();
        let (store, codex_home) = codex_store(tmp.path());
        let canonical = store.active_path("codex");
        let native = codex_home.join("auth.json");
        let pending = codex_home.join(".auth.json.omega-capture.crash-test");
        write_codex(&canonical, "2026-07-20T10:00:00Z", "canonical-old");
        write_codex(&pending, "2026-07-23T10:00:00Z", "captured-new");
        std::os::unix::fs::symlink(&canonical, &native).unwrap();

        let report = store.reconcile_codex().unwrap();

        assert_eq!(report.action, CodexReconcileAction::AdoptedNative);
        assert!(!pending.exists());
        assert_eq!(
            inspect_codex_credential(&canonical).last_refresh,
            DateTime::parse_from_rfc3339("2026-07-23T10:00:00Z")
                .ok()
                .map(|value| value.with_timezone(&Utc))
        );
        assert!(report.topology.native_links_to_canonical);
    }

    #[cfg(unix)]
    #[test]
    fn codex_reconcile_preserves_corrupt_native_without_clobbering_canonical() {
        let tmp = tempfile::tempdir().unwrap();
        let (store, codex_home) = codex_store(tmp.path());
        let canonical = store.active_path("codex");
        let native = codex_home.join("auth.json");
        write_codex(&canonical, "2026-07-22T10:00:00Z", "canonical-valid");
        std::fs::create_dir_all(&codex_home).unwrap();
        std::fs::write(&native, b"{truncated").unwrap();

        let report = store.reconcile_codex().unwrap();

        assert_eq!(report.action, CodexReconcileAction::RepairedNativeLink);
        assert_eq!(
            inspect_codex_credential(&canonical).validity,
            CodexCredentialValidity::Valid
        );
        assert_eq!(report.quarantined.len(), 1);
        assert_eq!(
            inspect_codex_credential(&report.quarantined[0]).validity,
            CodexCredentialValidity::Invalid
        );
        assert!(report.topology.native_links_to_canonical);
    }

    #[cfg(unix)]
    #[test]
    fn codex_reconcile_restores_single_topology_when_no_copy_is_valid() {
        let tmp = tempfile::tempdir().unwrap();
        let (store, codex_home) = codex_store(tmp.path());
        let native = codex_home.join("auth.json");
        std::fs::create_dir_all(&codex_home).unwrap();
        std::fs::write(&native, b"{incomplete").unwrap();

        let report = store.reconcile_codex().unwrap();

        assert_eq!(report.action, CodexReconcileAction::NoValidCredential);
        assert_eq!(report.quarantined.len(), 1);
        assert!(report.topology.native_is_symlink);
        assert!(report.topology.native_links_to_canonical);
        assert!(!report.topology.split);
        assert_eq!(
            report.topology.canonical_validity,
            CodexCredentialValidity::Missing
        );
        assert_eq!(
            inspect_codex_credential(&report.quarantined[0]).validity,
            CodexCredentialValidity::Invalid
        );
    }

    #[cfg(unix)]
    #[test]
    fn codex_reconcile_quarantines_an_invalid_extra_candidate_without_a_winner() {
        let tmp = tempfile::tempdir().unwrap();
        let (store, _codex_home) = codex_store(tmp.path());
        let candidate = tmp.path().join("flow").join("backup.json");
        std::fs::create_dir_all(candidate.parent().unwrap()).unwrap();
        let invalid = b"{truncated-flow-backup";
        std::fs::write(store.active_path("codex"), invalid).unwrap();
        std::fs::write(&candidate, invalid).unwrap();

        let report = store.reconcile_codex_candidate(&candidate).unwrap();

        assert_eq!(report.action, CodexReconcileAction::NoValidCredential);
        assert_eq!(report.quarantined.len(), 1);
        assert_eq!(std::fs::read(&report.quarantined[0]).unwrap(), invalid);
        assert_eq!(
            inspect_codex_credential(&report.quarantined[0]).validity,
            CodexCredentialValidity::Invalid
        );
        assert!(report.topology.native_links_to_canonical);
    }

    #[cfg(unix)]
    #[test]
    fn codex_reconcile_updates_resolved_target_without_breaking_symlink_chain() {
        let tmp = tempfile::tempdir().unwrap();
        let (store, codex_home) = codex_store(tmp.path());
        let canonical = store.active_path("codex");
        let shared = tmp.path().join("shared").join("codex-auth.json");
        let native = codex_home.join("auth.json");
        write_codex(&shared, "2026-07-20T10:00:00Z", "shared-old");
        std::os::unix::fs::symlink(&shared, &canonical).unwrap();
        write_codex(&native, "2026-07-21T10:00:00Z", "native-new");

        let report = store.reconcile_codex().unwrap();

        assert_eq!(report.action, CodexReconcileAction::AdoptedNative);
        assert_eq!(std::fs::read_link(&canonical).unwrap(), shared);
        assert_eq!(
            inspect_codex_credential(&shared).last_refresh,
            DateTime::parse_from_rfc3339("2026-07-21T10:00:00Z")
                .ok()
                .map(|v| v.with_timezone(&Utc))
        );
        assert!(report.topology.native_links_to_canonical);
    }

    #[cfg(unix)]
    #[test]
    fn codex_candidate_restore_is_monotonic_and_quarantines_every_distinct_loser() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::tempdir().unwrap();
        let (store, codex_home) = codex_store(tmp.path());
        let canonical = store.active_path("codex");
        let native = codex_home.join("auth.json");
        let backup = tmp.path().join("flow").join("backup.json");
        write_codex(&canonical, "2026-07-21T10:00:00Z", "canonical-middle");
        write_codex(&native, "2026-07-22T10:00:00Z", "native-newest");
        std::thread::sleep(std::time::Duration::from_millis(5));
        write_codex(&backup, "2026-07-20T10:00:00Z", "backup-oldest");

        let report = store.reconcile_codex_candidate(&backup).unwrap();

        assert_eq!(report.action, CodexReconcileAction::AdoptedNative);
        assert_eq!(report.quarantined.len(), 2);
        assert_eq!(
            inspect_codex_credential(&canonical).last_refresh,
            DateTime::parse_from_rfc3339("2026-07-22T10:00:00Z")
                .ok()
                .map(|v| v.with_timezone(&Utc))
        );
        let quarantine_dir = store.base_dir().join("quarantine").join("codex");
        let dir_mode = std::fs::metadata(&quarantine_dir)
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(dir_mode & 0o077, 0);
        for path in &report.quarantined {
            let mode = std::fs::metadata(path).unwrap().permissions().mode();
            assert_eq!(mode & 0o077, 0);
        }

        write_codex(&backup, "2026-07-23T10:00:00Z", "backup-newest");
        let promoted = store.reconcile_codex_candidate(&backup).unwrap();
        assert_eq!(promoted.action, CodexReconcileAction::AdoptedCandidate);
        assert_eq!(
            inspect_codex_credential(&canonical).last_refresh,
            DateTime::parse_from_rfc3339("2026-07-23T10:00:00Z")
                .ok()
                .map(|v| v.with_timezone(&Utc))
        );
        assert!(promoted.topology.native_links_to_canonical);
    }

    #[test]
    fn codex_fresh_relative_uses_semantic_time_and_requires_distinct_content() {
        let tmp = tempfile::tempdir().unwrap();
        let baseline = tmp.path().join("baseline.json");
        let candidate = tmp.path().join("candidate.json");
        write_codex(&baseline, "2026-07-22T10:00:00Z", "baseline");
        std::thread::sleep(std::time::Duration::from_millis(5));
        write_codex(&candidate, "2026-07-21T10:00:00Z", "older-candidate");
        assert!(!codex_credential_fresh_relative(&candidate, &baseline));

        write_codex(&candidate, "2026-07-23T10:00:00Z", "newer-candidate");
        assert!(codex_credential_fresh_relative(&candidate, &baseline));

        write_codex(&candidate, "2026-07-22T10:00:00Z", "same-time-candidate");
        assert!(
            !codex_credential_fresh_relative(&candidate, &baseline),
            "equal last_refresh is not proof that a device flow produced a fresh credential"
        );

        std::thread::sleep(std::time::Duration::from_millis(10));
        write_codex_api_key(&candidate, "newer-api-key");
        assert!(
            codex_credential_fresh_relative(&candidate, &baseline),
            "a one-sided last_refresh comparison should use a newer mtime"
        );

        std::fs::copy(&baseline, &candidate).unwrap();
        assert!(!codex_credential_fresh_relative(&candidate, &baseline));

        std::fs::write(&candidate, b"not-json").unwrap();
        assert!(!codex_credential_fresh_relative(&candidate, &baseline));
    }

    #[test]
    fn sanitize_strips_bad_chars() {
        assert_eq!(sanitize("hello world"), "hello_world");
        assert_eq!(sanitize("ok-name_123"), "ok-name_123");
    }
}

//! Project CRUD — create, register, discover, persist projects.

use anyhow::{Context, Result};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::OpenOptions;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ManagedProject {
    pub name: String,
    pub path: PathBuf,
    pub telegram_topic_id: Option<i64>,
    pub oracle_session: Option<String>,
    pub git_email: Option<String>,
    pub created_at: String,
    /// Per-project Telegram visibility toggle. `None` or `Some(true)` = enabled
    /// (the project gets a forum topic on `/sync` and shows normally in the Atlas
    /// bot); `Some(false)` = disabled (sync skips/removes its topic, the bot marks
    /// it 🔕 but keeps it listed). Default ON preserves existing behavior.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub telegram: Option<bool>,
    /// Thematic category used to group the Projects tab. An explicit value wins;
    /// when unset it is derived per-machine from the project's own location. See
    /// `display_category`. Kept out of the serialized form when `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

/// Versioned fields carried alongside the legacy `ManagedProject` view.
///
/// `ManagedProject` is constructed as a public struct by downstream crates, so
/// adding fields to it would be a source-breaking change. The registry therefore
/// owns this forward-compatible fiche and flattens it into each project JSON
/// object. Unknown keys are retained in `extra`, which makes a load/mutate/save
/// cycle lossless for newer or third-party producers.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ProjectFiche {
    pub repo: Option<String>,
    pub slug: Option<String>,
    pub default_branch: Option<String>,
    pub deploy_target: Option<JsonValue>,
    pub credential_refs: Option<JsonValue>,
    pub key_locations: Option<JsonValue>,
    pub extra: JsonMap<String, JsonValue>,
    null_fields: BTreeSet<String>,
}

impl Serialize for ProjectFiche {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut fields = self.extra.clone();
        for name in &self.null_fields {
            fields.insert(name.clone(), JsonValue::Null);
        }
        if let Some(value) = &self.repo {
            fields.insert("repo".to_string(), JsonValue::String(value.clone()));
        }
        if let Some(value) = &self.slug {
            fields.insert("slug".to_string(), JsonValue::String(value.clone()));
        }
        if let Some(value) = &self.default_branch {
            fields.insert(
                "default_branch".to_string(),
                JsonValue::String(value.clone()),
            );
        }
        for (name, value) in [
            ("deploy_target", self.deploy_target.as_ref()),
            ("credential_refs", self.credential_refs.as_ref()),
            ("key_locations", self.key_locations.as_ref()),
        ] {
            if let Some(value) = value {
                fields.insert(name.to_string(), value.clone());
            }
        }
        JsonValue::Object(fields).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ProjectFiche {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut fields = JsonMap::<String, JsonValue>::deserialize(deserializer)?;
        let mut null_fields = BTreeSet::new();
        let mut take_string = |name: &str| -> std::result::Result<Option<String>, D::Error> {
            match fields.remove(name) {
                None => Ok(None),
                Some(JsonValue::Null) => {
                    null_fields.insert(name.to_string());
                    Ok(None)
                }
                Some(JsonValue::String(value)) => Ok(Some(value)),
                Some(_) => Err(serde::de::Error::custom(format!(
                    "project fiche field {name} must be a string or null"
                ))),
            }
        };
        let repo = take_string("repo")?;
        let slug = take_string("slug")?;
        let default_branch = take_string("default_branch")?;
        let mut take_value = |name: &str| -> Option<JsonValue> {
            match fields.remove(name) {
                Some(JsonValue::Null) => {
                    null_fields.insert(name.to_string());
                    None
                }
                value => value,
            }
        };
        let deploy_target = take_value("deploy_target");
        let credential_refs = take_value("credential_refs");
        let key_locations = take_value("key_locations");
        Ok(Self {
            repo,
            slug,
            default_branch,
            deploy_target,
            credential_refs,
            key_locations,
            extra: fields,
            null_fields,
        })
    }
}

impl ManagedProject {
    /// Whether this project participates in Telegram (topic sync + Atlas display).
    /// Enabled unless the toggle was explicitly set to `false`.
    pub fn telegram_enabled(&self) -> bool {
        self.telegram != Some(false)
    }

    /// Thematic category for the Projects-tab grouping. An explicit `category`
    /// (non-empty) wins; otherwise it is derived from the project's own location
    /// on THIS machine — the first path component under the user's configured
    /// projects root (`OmegaConfig::projects_dir`). A project that sits directly
    /// under the root (no category folder) or outside it is `"Other"`.
    ///
    /// Machine-agnostic by design: it mirrors whatever top-level folders the user
    /// organizes their projects into (customers/ side-business/ tools/ …, or any
    /// custom layout), never a hardcoded taxonomy. Pass the root from
    /// `OmegaConfig::projects_dir`.
    pub fn display_category(&self, projects_root: &Path) -> String {
        if let Some(c) = self.category.as_deref() {
            let c = c.trim();
            if !c.is_empty() {
                return c.to_string();
            }
        }
        // The first folder under the projects root, but only when the project
        // actually lives INSIDE a category folder (≥2 remaining components:
        // <category>/<project>[/…]). A bare child of the root has no category.
        if let Ok(rel) = self.path.strip_prefix(projects_root) {
            let mut comps = rel.components();
            if let Some(first) = comps.next() {
                if comps.next().is_some() {
                    return first.as_os_str().to_string_lossy().to_string();
                }
            }
        }
        "Other".to_string()
    }

    /// Sort key for the Projects-tab category sections: named categories
    /// alphabetically (case-insensitive), `"Other"` always last. Deliberately
    /// order-agnostic so it fits any user's folder taxonomy.
    pub fn category_rank(cat: &str) -> (u8, String) {
        if cat == "Other" {
            (1, String::new())
        } else {
            (0, cat.to_lowercase())
        }
    }
}

const PROJECT_REGISTRY_SCHEMA_VERSION: u32 = 1;
const PROJECT_KNOWN_FIELDS: &[&str] = &[
    "name",
    "path",
    "telegram_topic_id",
    "oracle_session",
    "git_email",
    "created_at",
    "telegram",
    "category",
];

#[derive(Debug, Clone, Default)]
pub struct ProjectRegistry {
    pub projects: Vec<ManagedProject>,
    /// True when this registry came from a file that exists but could not be
    /// read or parsed. Such a registry is empty-looking but MUST NOT be saved:
    /// writing it back would erase the user's real project list. Never
    /// serialized — it is a property of this load, not of the data.
    pub poisoned: bool,
    schema_version: u32,
    project_fiches: BTreeMap<String, ProjectFiche>,
    root_extra: JsonMap<String, JsonValue>,
    baseline: Option<RegistrySnapshot>,
}

#[derive(Debug, Clone, Default)]
struct RegistrySnapshot {
    projects: Vec<ManagedProject>,
    project_fiches: BTreeMap<String, ProjectFiche>,
    root_extra: JsonMap<String, JsonValue>,
}

impl RegistrySnapshot {
    fn capture(registry: &ProjectRegistry) -> Self {
        Self {
            projects: registry.projects.clone(),
            project_fiches: registry.project_fiches.clone(),
            root_extra: registry.root_extra.clone(),
        }
    }
}

fn project_key(project: &ManagedProject) -> String {
    format!(
        "{}\u{0}{}",
        project.path.to_string_lossy(),
        project.name.to_lowercase()
    )
}

fn registry_to_value(registry: &ProjectRegistry) -> Result<JsonValue> {
    let mut root = registry.root_extra.clone();
    root.insert(
        "schema_version".to_string(),
        JsonValue::from(registry.schema_version.max(PROJECT_REGISTRY_SCHEMA_VERSION)),
    );

    let mut projects = Vec::with_capacity(registry.projects.len());
    for project in &registry.projects {
        let mut object = match serde_json::to_value(project)? {
            JsonValue::Object(object) => object,
            _ => unreachable!("ManagedProject always serializes as an object"),
        };
        if let Some(fiche) = registry_fiche_for_index(registry, projects.len()) {
            let fiche_value = serde_json::to_value(fiche)?;
            if let JsonValue::Object(fields) = fiche_value {
                for (key, value) in fields {
                    anyhow::ensure!(
                        !PROJECT_KNOWN_FIELDS.contains(&key.as_str()),
                        "project fiche field {key:?} conflicts with a ManagedProject field"
                    );
                    object.insert(key, value);
                }
            }
        }
        projects.push(JsonValue::Object(object));
    }
    root.insert("projects".to_string(), JsonValue::Array(projects));
    Ok(JsonValue::Object(root))
}

impl Serialize for ProjectRegistry {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        registry_to_value(self)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ProjectRegistry {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = JsonValue::deserialize(deserializer)?;
        let mut root = match value {
            JsonValue::Object(root) => root,
            _ => {
                return Err(serde::de::Error::custom(
                    "project registry must be a JSON object",
                ))
            }
        };
        let schema_version = match root.remove("schema_version") {
            None => PROJECT_REGISTRY_SCHEMA_VERSION,
            Some(JsonValue::Number(value)) => value
                .as_u64()
                .and_then(|value| u32::try_from(value).ok())
                .filter(|value| *value >= PROJECT_REGISTRY_SCHEMA_VERSION)
                .ok_or_else(|| serde::de::Error::custom("schema_version must be a positive u32"))?,
            Some(_) => {
                return Err(serde::de::Error::custom(
                    "schema_version must be a positive u32",
                ))
            }
        };
        let project_values = match root
            .remove("projects")
            .ok_or_else(|| serde::de::Error::custom("project registry is missing projects"))?
        {
            JsonValue::Array(projects) => projects,
            _ => return Err(serde::de::Error::custom("projects must be a JSON array")),
        };

        let mut projects = Vec::with_capacity(project_values.len());
        let mut project_fiches = BTreeMap::new();
        for value in project_values {
            let mut object = match value {
                JsonValue::Object(object) => object,
                _ => {
                    return Err(serde::de::Error::custom(
                        "each project must be a JSON object",
                    ))
                }
            };
            let project =
                serde_json::from_value::<ManagedProject>(JsonValue::Object(object.clone()))
                    .map_err(serde::de::Error::custom)?;
            for field in PROJECT_KNOWN_FIELDS {
                object.remove(*field);
            }
            if !object.is_empty() {
                let fiche = serde_json::from_value::<ProjectFiche>(JsonValue::Object(object))
                    .map_err(serde::de::Error::custom)?;
                project_fiches.insert(project_key(&project), fiche);
            }
            projects.push(project);
        }

        let mut registry = Self {
            projects,
            poisoned: false,
            schema_version,
            project_fiches,
            root_extra: root,
            baseline: None,
        };
        registry.baseline = Some(RegistrySnapshot::capture(&registry));
        Ok(registry)
    }
}

const REGISTRY_LOCK_WAIT: Duration = Duration::from_secs(5);
const REGISTRY_LOCK_STALE: Duration = Duration::from_secs(120);
const JSON_LOCK_OWNER_FILE: &str = "owner.json";
static JSON_LOCK_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Serialize, Deserialize)]
struct JsonLockOwner {
    schema_version: u32,
    token: String,
    pid: u32,
    process_start: Option<String>,
    created_ms: u128,
}

/// Cross-language lock shared with `telegram-bot/omega-tg-bot.ts`.
///
/// The owner record makes release generation-exact. A stale owner is first
/// quarantined by atomic rename, and a tokenized lock is reclaimed only after
/// Linux `/proc` proves that exact PID generation is gone. This prevents an old
/// guard from deleting a replacement directory after a stale-lock takeover.
pub(crate) struct JsonDirLock {
    path: PathBuf,
    token: String,
}

fn linux_process_start(pid: u32) -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
        let close = stat.rfind(')')?;
        // Fields after `comm`: state is field 3 (index 0), starttime is field
        // 22 (index 19). `comm` itself may contain spaces or parentheses.
        stat.get(close + 1..)?
            .split_whitespace()
            .nth(19)
            .map(str::to_string)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = pid;
        None
    }
}

#[cfg(target_os = "linux")]
fn linux_lock_owner_is_gone(owner: &JsonLockOwner) -> bool {
    let process_dir = PathBuf::from(format!("/proc/{}", owner.pid));
    if !process_dir.exists() {
        return true;
    }
    match (
        linux_process_start(owner.pid),
        owner.process_start.as_deref(),
    ) {
        (Some(current), Some(expected)) => current != expected,
        // An extant /proc entry whose generation cannot be read is not proof
        // of death. Fail closed instead of stealing a potentially live lock.
        _ => false,
    }
}

fn new_json_lock_owner() -> JsonLockOwner {
    let sequence = JSON_LOCK_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let created_ms = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let pid = std::process::id();
    JsonLockOwner {
        schema_version: 1,
        token: format!("rust-{pid}-{created_ms:x}-{sequence:x}"),
        pid,
        process_start: linux_process_start(pid),
        created_ms,
    }
}

fn read_json_lock_owner(path: &Path) -> Result<Option<JsonLockOwner>> {
    let lock_metadata = std::fs::symlink_metadata(path)
        .with_context(|| format!("inspecting JSON lock directory {}", path.display()))?;
    anyhow::ensure!(
        lock_metadata.file_type().is_dir() && !lock_metadata.file_type().is_symlink(),
        "JSON lock {} must be a real directory",
        path.display()
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        // SAFETY: geteuid has no arguments or memory preconditions and returns
        // the kernel-maintained effective user id for this process.
        let current_uid = unsafe { libc::geteuid() };
        anyhow::ensure!(
            lock_metadata.uid() == current_uid,
            "JSON lock {} is not owned by the current user",
            path.display()
        );
        anyhow::ensure!(
            lock_metadata.permissions().mode() & 0o077 == 0,
            "JSON lock {} is accessible by group or other users",
            path.display()
        );
    }
    let owner_path = path.join(JSON_LOCK_OWNER_FILE);
    match crate::config::read_private_optional(&owner_path)? {
        Some(bytes) => {
            let owner: JsonLockOwner = serde_json::from_slice(&bytes)
                .with_context(|| format!("parsing JSON lock owner {}", owner_path.display()))?;
            anyhow::ensure!(
                owner.schema_version == 1 && !owner.token.is_empty() && owner.pid > 0,
                "invalid JSON lock owner shape {}",
                owner_path.display()
            );
            Ok(Some(owner))
        }
        None => Ok(None),
    }
}

fn json_lock_is_stale(path: &Path) -> bool {
    let old_enough = std::fs::symlink_metadata(path)
        .and_then(|metadata| {
            if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() {
                Ok(metadata)
            } else {
                Err(std::io::Error::other("lock is not a real directory"))
            }
        })
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| SystemTime::now().duration_since(modified).ok())
        .is_some_and(|age| age >= REGISTRY_LOCK_STALE);
    if !old_enough {
        return false;
    }
    match read_json_lock_owner(path) {
        // Legacy empty lock directories have no owner. A legacy guard releases
        // with rmdir, which cannot remove the new non-empty tokenized lock.
        Ok(None) => true,
        #[cfg(target_os = "linux")]
        Ok(Some(owner)) => linux_lock_owner_is_gone(&owner),
        #[cfg(not(target_os = "linux"))]
        Ok(Some(_)) => false,
        // Malformed owner authority is never guessed away.
        Err(_) => false,
    }
}

fn remove_quarantined_lock(path: &Path) -> Result<()> {
    let owner = path.join(JSON_LOCK_OWNER_FILE);
    match std::fs::remove_file(&owner) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(error).with_context(|| format!("removing {}", owner.display())),
    }
    std::fs::remove_dir(path)
        .with_context(|| format!("removing quarantined JSON lock {}", path.display()))
}

impl JsonDirLock {
    pub(crate) fn acquire(data_path: &Path) -> Result<Self> {
        let mut lock_name = data_path.as_os_str().to_os_string();
        lock_name.push(".lock");
        let path = PathBuf::from(lock_name);
        let started = Instant::now();
        loop {
            match std::fs::create_dir(&path) {
                Ok(()) => {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))?;
                    }
                    let owner = new_json_lock_owner();
                    let owner_path = path.join(JSON_LOCK_OWNER_FILE);
                    let owner_result = (|| -> Result<()> {
                        let mut options = OpenOptions::new();
                        options.write(true).create_new(true);
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::OpenOptionsExt;
                            options.mode(0o600);
                        }
                        let mut file = options.open(&owner_path).with_context(|| {
                            format!("creating JSON lock owner {}", owner_path.display())
                        })?;
                        serde_json::to_writer(&mut file, &owner)?;
                        file.write_all(b"\n")?;
                        file.sync_all()?;
                        Ok(())
                    })();
                    if let Err(error) = owner_result {
                        let _ = std::fs::remove_file(&owner_path);
                        let _ = std::fs::remove_dir(&path);
                        return Err(error);
                    }
                    return Ok(Self {
                        path,
                        token: owner.token,
                    });
                }
                Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                    if json_lock_is_stale(&path) {
                        let quarantine = PathBuf::from(format!(
                            "{}.stale.{}",
                            path.display(),
                            new_json_lock_owner().token
                        ));
                        if std::fs::rename(&path, &quarantine).is_ok() {
                            if let Err(cleanup) = remove_quarantined_lock(&quarantine) {
                                if !path.exists() {
                                    let _ = std::fs::rename(&quarantine, &path);
                                }
                                return Err(cleanup);
                            }
                            continue;
                        }
                    }
                    anyhow::ensure!(
                        started.elapsed() < REGISTRY_LOCK_WAIT,
                        "timed out waiting for JSON state lock {}",
                        path.display()
                    );
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(error) => {
                    return Err(error)
                        .with_context(|| format!("creating JSON state lock {}", path.display()));
                }
            }
        }
    }

    fn release_exact(path: &Path, token: &str) -> Result<()> {
        let Some(owner) = read_json_lock_owner(path)? else {
            return Ok(());
        };
        if owner.token != token {
            return Ok(());
        }
        std::fs::remove_file(path.join(JSON_LOCK_OWNER_FILE))?;
        std::fs::remove_dir(path)
            .with_context(|| format!("releasing JSON state lock {}", path.display()))
    }
}

impl Drop for JsonDirLock {
    fn drop(&mut self) {
        if let Err(error) = Self::release_exact(&self.path, &self.token) {
            eprintln!(
                "omega: could not release JSON state lock {}: {error}",
                self.path.display()
            );
        }
    }
}

fn write_registry_atomic(registry: &ProjectRegistry, path: &Path) -> Result<()> {
    let mut json = serde_json::to_vec_pretty(registry)?;
    json.push(b'\n');
    crate::config::atomic_write_private(path, &json)
        .with_context(|| format!("atomically replacing registry {}", path.display()))
}

fn validate_project_name(name: &str) -> Result<()> {
    anyhow::ensure!(!name.is_empty(), "project name cannot be empty");
    anyhow::ensure!(
        name.trim() == name
            && !name.contains('/')
            && !name.contains('\\')
            && !name.chars().any(char::is_control),
        "project name must be one safe path component"
    );
    let mut components = Path::new(name).components();
    anyhow::ensure!(
        matches!(components.next(), Some(std::path::Component::Normal(_)))
            && components.next().is_none(),
        "project name must be one safe path component"
    );
    Ok(())
}

fn same_project_path(left: &Path, right: &Path) -> bool {
    match (std::fs::canonicalize(left), std::fs::canonicalize(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}

fn project_target_path(name: &str, location: &Path) -> Result<PathBuf> {
    validate_project_name(name)?;
    let location = std::fs::canonicalize(location)
        .with_context(|| format!("resolving project location {}", location.display()))?;
    anyhow::ensure!(
        location.is_dir(),
        "project location is not a directory: {}",
        location.display()
    );
    Ok(location.join(name))
}

fn write_new_project_file(path: &Path, content: &[u8]) -> Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    let mut file = options.open(path).with_context(|| {
        format!(
            "refusing to overwrite existing project file {}",
            path.display()
        )
    })?;
    file.write_all(content)
        .with_context(|| format!("writing new project file {}", path.display()))?;
    file.sync_all()
        .with_context(|| format!("syncing new project file {}", path.display()))?;
    Ok(())
}

fn assign_unique_project_matches(
    baseline: &[ManagedProject],
    candidates: &[ManagedProject],
    matches: &mut [Option<usize>],
    used: &mut BTreeSet<usize>,
    predicate: impl Fn(usize, &ManagedProject, &ManagedProject) -> bool,
) {
    for (baseline_index, baseline_project) in baseline.iter().enumerate() {
        if matches[baseline_index].is_some() {
            continue;
        }
        let candidate_indices: Vec<_> = candidates
            .iter()
            .enumerate()
            .filter(|(candidate_index, candidate)| {
                !used.contains(candidate_index)
                    && predicate(baseline_index, baseline_project, candidate)
            })
            .map(|(candidate_index, _)| candidate_index)
            .collect();
        if let [candidate_index] = candidate_indices.as_slice() {
            matches[baseline_index] = Some(*candidate_index);
            used.insert(*candidate_index);
        }
    }
}

/// Match the same logical project across stale registry snapshots.
///
/// Name and path are editable user data, not durable identities. Match their
/// strongest exact forms first, then use `created_at` only among the remaining
/// unmatched records. This disambiguates legacy registries where bulk imports
/// gave many projects the same timestamp. Position is never identity: treating
/// an equal-length delete-plus-add as a rename can leak the removed project's
/// concurrently updated metadata into the new project.
fn project_correspondences(
    baseline: &[ManagedProject],
    candidates: &[ManagedProject],
) -> Vec<Option<usize>> {
    let mut matches = vec![None; baseline.len()];
    let mut used = BTreeSet::new();

    assign_unique_project_matches(
        baseline,
        candidates,
        &mut matches,
        &mut used,
        |_, project, candidate| project_key(project) == project_key(candidate),
    );
    assign_unique_project_matches(
        baseline,
        candidates,
        &mut matches,
        &mut used,
        |_, project, candidate| project.path == candidate.path,
    );
    assign_unique_project_matches(
        baseline,
        candidates,
        &mut matches,
        &mut used,
        |_, project, candidate| project.name.eq_ignore_ascii_case(&candidate.name),
    );
    assign_unique_project_matches(
        baseline,
        candidates,
        &mut matches,
        &mut used,
        |_, project, candidate| {
            !project.created_at.is_empty() && candidate.created_at == project.created_at
        },
    );
    matches
}

fn baseline_key_for_current_index(
    registry: &ProjectRegistry,
    current_index: usize,
) -> Option<String> {
    let baseline = registry.baseline.as_ref()?;
    let matches = project_correspondences(&baseline.projects, &registry.projects);
    let baseline_index = matches
        .iter()
        .position(|candidate| *candidate == Some(current_index))?;
    Some(project_key(&baseline.projects[baseline_index]))
}

fn registry_fiche_for_index(
    registry: &ProjectRegistry,
    project_index: usize,
) -> Option<&ProjectFiche> {
    let project = registry.projects.get(project_index)?;
    let current_key = project_key(project);
    registry.project_fiches.get(&current_key).or_else(|| {
        baseline_key_for_current_index(registry, project_index)
            .and_then(|baseline_key| registry.project_fiches.get(&baseline_key))
    })
}

fn merge_json_map(
    baseline: &JsonMap<String, JsonValue>,
    desired: &JsonMap<String, JsonValue>,
    current: &JsonMap<String, JsonValue>,
) -> JsonMap<String, JsonValue> {
    let mut merged = current.clone();
    let keys: BTreeSet<_> = baseline.keys().chain(desired.keys()).cloned().collect();
    for key in keys {
        if desired.get(&key) == baseline.get(&key) {
            continue;
        }
        match desired.get(&key) {
            Some(value) => {
                merged.insert(key, value.clone());
            }
            None => {
                merged.remove(&key);
            }
        }
    }
    merged
}

fn merge_managed_project(
    baseline: &ManagedProject,
    desired: &ManagedProject,
    current: &ManagedProject,
) -> ManagedProject {
    let mut merged = current.clone();
    macro_rules! apply_local_change {
        ($field:ident) => {
            if desired.$field != baseline.$field {
                merged.$field = desired.$field.clone();
            }
        };
    }
    apply_local_change!(name);
    apply_local_change!(path);
    apply_local_change!(telegram_topic_id);
    apply_local_change!(oracle_session);
    apply_local_change!(git_email);
    apply_local_change!(created_at);
    apply_local_change!(telegram);
    apply_local_change!(category);
    merged
}

fn merge_project_fiche(
    baseline: Option<&ProjectFiche>,
    desired: &ProjectFiche,
    current: Option<&ProjectFiche>,
) -> ProjectFiche {
    let empty = ProjectFiche::default();
    let baseline = baseline.unwrap_or(&empty);
    let mut merged = current.cloned().unwrap_or_default();
    macro_rules! apply_local_change {
        ($field:ident) => {
            if desired.$field != baseline.$field {
                merged.$field = desired.$field.clone();
            }
        };
    }
    apply_local_change!(repo);
    apply_local_change!(slug);
    apply_local_change!(default_branch);
    apply_local_change!(deploy_target);
    apply_local_change!(credential_refs);
    apply_local_change!(key_locations);

    let extra_keys: BTreeSet<_> = baseline
        .extra
        .keys()
        .chain(desired.extra.keys())
        .cloned()
        .collect();
    for key in extra_keys {
        if desired.extra.get(&key) == baseline.extra.get(&key) {
            continue;
        }
        match desired.extra.get(&key) {
            Some(value) => {
                merged.extra.insert(key, value.clone());
            }
            None => {
                merged.extra.remove(&key);
            }
        }
    }

    let null_keys: BTreeSet<_> = baseline
        .null_fields
        .iter()
        .chain(desired.null_fields.iter())
        .cloned()
        .collect();
    for key in null_keys {
        let baseline_has = baseline.null_fields.contains(&key);
        let desired_has = desired.null_fields.contains(&key);
        if baseline_has == desired_has {
            continue;
        }
        if desired_has {
            merged.null_fields.insert(key);
        } else {
            merged.null_fields.remove(&key);
        }
    }
    merged
}

fn merge_registry_change(desired: &ProjectRegistry, latest: &ProjectRegistry) -> ProjectRegistry {
    let Some(baseline) = desired.baseline.as_ref() else {
        let mut authoritative = desired.clone();
        for project in &authoritative.projects {
            let key = project_key(project);
            if let std::collections::btree_map::Entry::Vacant(entry) =
                authoritative.project_fiches.entry(key.clone())
            {
                if let Some(fiche) = latest.project_fiches.get(&key) {
                    entry.insert(fiche.clone());
                }
            }
        }
        authoritative.root_extra = latest.root_extra.clone();
        authoritative.schema_version = desired
            .schema_version
            .max(latest.schema_version)
            .max(PROJECT_REGISTRY_SCHEMA_VERSION);
        authoritative.baseline = None;
        return authoritative;
    };

    let mut merged = latest.clone();
    let baseline_to_desired = project_correspondences(&baseline.projects, &desired.projects);
    let baseline_to_latest = project_correspondences(&baseline.projects, &latest.projects);
    let mut consumed_desired = BTreeSet::new();
    let mut consumed_latest = BTreeSet::new();
    let mut removed_latest = BTreeSet::new();
    let mut fiche_removals = BTreeSet::new();
    let mut fiche_insertions = Vec::new();

    for (baseline_index, baseline_project) in baseline.projects.iter().enumerate() {
        let desired_index = baseline_to_desired[baseline_index];
        let latest_index = baseline_to_latest[baseline_index];
        if let Some(index) = desired_index {
            consumed_desired.insert(index);
        }
        if let Some(index) = latest_index {
            consumed_latest.insert(index);
        }

        let baseline_key = project_key(baseline_project);
        let baseline_fiche = baseline.project_fiches.get(&baseline_key);
        let Some(desired_index) = desired_index else {
            if let Some(latest_index) = latest_index {
                removed_latest.insert(latest_index);
                fiche_removals.insert(project_key(&latest.projects[latest_index]));
            }
            fiche_removals.insert(baseline_key);
            continue;
        };

        let desired_project = &desired.projects[desired_index];
        let desired_key = project_key(desired_project);
        let desired_fiche = desired
            .project_fiches
            .get(&desired_key)
            .or_else(|| desired.project_fiches.get(&baseline_key));

        let (merged_project, latest_key, latest_fiche) = match latest_index {
            Some(index) => {
                let latest_project = &latest.projects[index];
                let latest_key = project_key(latest_project);
                let latest_fiche = latest
                    .project_fiches
                    .get(&latest_key)
                    .or_else(|| latest.project_fiches.get(&baseline_key));
                (
                    merge_managed_project(baseline_project, desired_project, latest_project),
                    Some(latest_key),
                    latest_fiche,
                )
            }
            None => (desired_project.clone(), None, None),
        };
        let merged_key = project_key(&merged_project);
        if let Some(index) = latest_index {
            merged.projects[index] = merged_project;
        } else {
            merged.projects.push(merged_project);
        }

        fiche_removals.insert(baseline_key);
        fiche_removals.insert(desired_key);
        if let Some(key) = latest_key {
            fiche_removals.insert(key);
        }
        let merged_fiche = if desired_fiche != baseline_fiche {
            desired_fiche.map(|fiche| merge_project_fiche(baseline_fiche, fiche, latest_fiche))
        } else {
            latest_fiche.cloned()
        };
        if let Some(fiche) = merged_fiche {
            fiche_insertions.push((merged_key, fiche));
        }
    }

    merged.projects = merged
        .projects
        .into_iter()
        .enumerate()
        .filter_map(|(index, project)| (!removed_latest.contains(&index)).then_some(project))
        .collect();

    for (desired_index, desired_project) in desired.projects.iter().enumerate() {
        if consumed_desired.contains(&desired_index) {
            continue;
        }
        let desired_key = project_key(desired_project);
        let existing_index = latest
            .projects
            .iter()
            .enumerate()
            .find(|(index, candidate)| {
                !consumed_latest.contains(index) && project_key(candidate) == desired_key
            })
            .map(|(index, _)| index);
        let latest_fiche = existing_index.and_then(|index| {
            let key = project_key(&latest.projects[index]);
            latest.project_fiches.get(&key)
        });
        if let Some(existing) = merged
            .projects
            .iter_mut()
            .find(|candidate| project_key(candidate) == desired_key)
        {
            *existing = desired_project.clone();
        } else {
            merged.projects.push(desired_project.clone());
        }
        fiche_removals.insert(desired_key.clone());
        if let Some(fiche) = desired.project_fiches.get(&desired_key) {
            fiche_insertions.push((desired_key, merge_project_fiche(None, fiche, latest_fiche)));
        }
    }

    for key in fiche_removals {
        merged.project_fiches.remove(&key);
    }
    for (key, fiche) in fiche_insertions {
        merged.project_fiches.insert(key, fiche);
    }

    merged.schema_version = desired
        .schema_version
        .max(latest.schema_version)
        .max(PROJECT_REGISTRY_SCHEMA_VERSION);
    merged.root_extra = merge_json_map(
        &baseline.root_extra,
        &desired.root_extra,
        &latest.root_extra,
    );
    merged.poisoned = false;
    merged.baseline = None;
    merged
}

impl ProjectRegistry {
    pub fn registry_path() -> PathBuf {
        crate::config::omega_dir().join("projects.json")
    }

    pub fn load() -> Self {
        Self::load_from(&Self::registry_path())
    }

    /// Load the registry, treating "file is absent" and "file is unreadable"
    /// as DIFFERENT things.
    ///
    /// This used to be `.ok().and_then(parse).unwrap_or_default()`, which
    /// collapsed both into an empty registry. Callers then add one project and
    /// `save()` — so a single transient read failure silently replaced every
    /// project the user had with one entry. That is exactly what happened on
    /// 2026-07-29: a registry of 25 projects became 1.
    ///
    /// Absent → empty is correct (first run). Present-but-unreadable → we
    /// preserve the file, shout on stderr, and return a POISONED registry that
    /// `save()` refuses to persist, so the damage cannot be written back.
    pub fn load_from(path: &Path) -> Self {
        match crate::config::read_private_optional(path) {
            Ok(None) => Self::default(),
            Ok(Some(content)) => match serde_json::from_slice::<Self>(&content) {
                Ok(mut reg) => {
                    reg.poisoned = false;
                    reg
                }
                Err(e) => {
                    let backup = path.with_extension("json.unreadable");
                    match crate::config::atomic_write_private(&backup, &content) {
                        Ok(()) => eprintln!(
                            "omega: projects.json could not be parsed ({e}). Your project list is \
                             NOT lost; the original is untouched and a recovery copy is at {}. \
                             Refusing to overwrite it.",
                            backup.display()
                        ),
                        Err(backup_error) => eprintln!(
                            "omega: projects.json could not be parsed ({e}). The original is \
                             untouched, but the recovery copy failed: {backup_error:#}. Refusing \
                             to overwrite it."
                        ),
                    }
                    Self {
                        poisoned: true,
                        ..Self::default()
                    }
                }
            },
            Err(e) => {
                eprintln!(
                    "omega: projects.json could not be read ({e}). Refusing to overwrite it."
                );
                Self {
                    poisoned: true,
                    ..Self::default()
                }
            }
        }
    }

    pub fn save(&self) -> Result<()> {
        self.save_to(&Self::registry_path())
    }

    pub fn save_to(&self, path: &Path) -> Result<()> {
        // A registry we failed to read is never written back. Persisting it
        // would turn a transient read error into permanent data loss.
        anyhow::ensure!(
            !self.poisoned,
            "refusing to save a projects registry that failed to load — \
             the existing file is preserved"
        );
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let _lock = JsonDirLock::acquire(path)?;
        let latest = if path.exists() {
            let latest = Self::load_from(path);
            anyhow::ensure!(
                !latest.poisoned,
                "refusing to replace a project registry that became unreadable while waiting for its lock"
            );
            latest
        } else {
            Self::default()
        };
        anyhow::ensure!(
            self.baseline.is_some() || latest.baseline.is_none(),
            "refusing to replace an existing project registry from an object that was not loaded from it; use update_locked"
        );
        let merged = merge_registry_change(self, &latest);
        write_registry_atomic(&merged, path)
    }

    /// Execute one complete read-modify-write transaction under the registry's
    /// cross-language directory lock. New mutators should use this API instead
    /// of a separate `load()` and `save()` pair.
    pub fn update_locked<T>(
        path: &Path,
        update: impl FnOnce(&mut ProjectRegistry) -> Result<T>,
    ) -> Result<T> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let _lock = JsonDirLock::acquire(path)?;
        let mut registry = Self::load_from(path);
        anyhow::ensure!(
            !registry.poisoned,
            "refusing to update a project registry that could not be loaded"
        );
        let output = update(&mut registry)?;
        registry.poisoned = false;
        write_registry_atomic(&registry, path)?;
        Ok(output)
    }

    pub fn fiche(&self, name: &str) -> Option<&ProjectFiche> {
        let project_index = self
            .projects
            .iter()
            .position(|project| project.name.eq_ignore_ascii_case(name))?;
        registry_fiche_for_index(self, project_index)
    }

    pub fn fiche_mut(&mut self, name: &str) -> Option<&mut ProjectFiche> {
        let project_index = self
            .projects
            .iter()
            .position(|project| project.name.eq_ignore_ascii_case(name))?;
        let key = project_key(&self.projects[project_index]);
        if !self.project_fiches.contains_key(&key) {
            if let Some(baseline_key) = baseline_key_for_current_index(self, project_index) {
                if baseline_key != key {
                    if let Some(fiche) = self.project_fiches.remove(&baseline_key) {
                        self.project_fiches.insert(key.clone(), fiche);
                    }
                }
            }
        }
        Some(self.project_fiches.entry(key).or_default())
    }

    pub fn find(&self, name: &str) -> Option<&ManagedProject> {
        let lower = name.to_lowercase();
        self.projects
            .iter()
            .find(|p| p.name.to_lowercase() == lower)
    }

    pub fn find_by_path(&self, path: &Path) -> Option<&ManagedProject> {
        self.projects
            .iter()
            .find(|project| same_project_path(&project.path, path))
    }

    pub fn add(&mut self, project: ManagedProject) {
        if self.find_by_path(&project.path).is_none()
            && self
                .projects
                .iter()
                .all(|existing| !existing.name.eq_ignore_ascii_case(&project.name))
        {
            self.projects.push(project);
        }
    }

    pub fn remove(&mut self, name: &str) -> bool {
        let lower = name.to_lowercase();
        let removed_keys: Vec<_> = self
            .projects
            .iter()
            .filter(|project| project.name.to_lowercase() == lower)
            .map(project_key)
            .collect();
        let before = self.projects.len();
        self.projects.retain(|p| p.name.to_lowercase() != lower);
        for key in removed_keys {
            self.project_fiches.remove(&key);
        }
        self.projects.len() < before
    }

    /// Set a project's Telegram toggle (topic sync + Atlas visibility). Returns
    /// `true` if a project with that name existed and was updated.
    pub fn set_telegram(&mut self, name: &str, enabled: bool) -> bool {
        let lower = name.to_lowercase();
        if let Some(p) = self
            .projects
            .iter_mut()
            .find(|p| p.name.to_lowercase() == lower)
        {
            p.telegram = Some(enabled);
            true
        } else {
            false
        }
    }
}

const SCAFFOLD_DIRS: &[&str] = &["docs", "docs/FEATURES", ".planner", ".oracles", ".omega"];

const CLAUDE_MD_TEMPLATE: &str = r#"# {name}

## Project

- **Path:** {path}
- **Created:** {date}

## Development Rules

- Law 1: Code lies. Only runtime tells the truth.
- Law 2: Researcher, not sycophant. Challenge flawed premises.
- Every feature must be verified with live runtime evidence before merge.
"#;

const VISION_TEMPLATE: &str = r#"# Vision — {name}

> Define the emotional foundation and product identity here.

## Internal Compass

_One sentence that acts as a decision filter for everything._

## Soul Statement

_What is this product, at its core?_

## Design Principles

1.
2.
3.

## Personas

### Primary

- **Name:**
- **Role:**
- **Pain:**
- **Goal:**

## What This Is NOT

-
"#;

const PRD_TEMPLATE: &str = r#"# PRD — {name}

> Product Requirements Document

## Overview

_What problem does this solve and for whom?_

## Goals

1.
2.
3.

## Features

| ID | Feature | Priority | Status |
|----|---------|----------|--------|
| F-001 | | P0 | planned |

## Technical Constraints

-

## Success Metrics

-
"#;

/// Scaffold a project on disk and describe it — WITHOUT touching the registry.
///
/// Split out from `create_project` so the scaffolding can be tested without
/// writing the machine's real `~/.omega/projects.json`. The old test called
/// `create_project` directly and rewrote the operator's registry every time the
/// suite ran; on 2026-07-29 that replaced 25 projects with one.
pub fn scaffold_project(name: &str, location: &Path) -> Result<ManagedProject> {
    let project_path = project_target_path(name, location)?;
    std::fs::create_dir(&project_path).with_context(|| {
        format!(
            "creating new project dir at {} (existing directories are never overwritten)",
            project_path.display()
        )
    })?;

    let scaffold_result = (|| -> Result<String> {
        for dir in SCAFFOLD_DIRS {
            std::fs::create_dir_all(project_path.join(dir))?;
        }

        let date = chrono::Utc::now().to_rfc3339();
        let claude_md = CLAUDE_MD_TEMPLATE
            .replace("{name}", name)
            .replace("{path}", &project_path.to_string_lossy())
            .replace("{date}", &date);
        write_new_project_file(
            project_path.join("CLAUDE.md").as_path(),
            claude_md.as_bytes(),
        )?;

        let vision = VISION_TEMPLATE.replace("{name}", name);
        write_new_project_file(
            project_path.join("docs/VISION.md").as_path(),
            vision.as_bytes(),
        )?;

        let prd = PRD_TEMPLATE.replace("{name}", name);
        write_new_project_file(project_path.join("docs/PRD.md").as_path(), prd.as_bytes())?;

        let output = std::process::Command::new("git")
            .args(["init"])
            .current_dir(&project_path)
            .output()
            .context("executing git init for new project")?;
        anyhow::ensure!(
            output.status.success(),
            "git init failed for {}: {}",
            project_path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
        Ok(date)
    })();
    let date = match scaffold_result {
        Ok(date) => date,
        Err(error) => {
            if let Err(rollback_error) = std::fs::remove_dir_all(&project_path) {
                return Err(error).context(format!(
                    "scaffolding failed and rollback of {} also failed: {rollback_error}",
                    project_path.display()
                ));
            }
            return Err(error);
        }
    };

    let project = ManagedProject {
        name: name.to_string(),
        path: project_path,
        telegram_topic_id: None,
        oracle_session: Some(format!("oracle-{}", name)),
        git_email: None,
        created_at: date,
        telegram: None,
        category: None,
    };

    Ok(project)
}

/// Scaffold a project AND register it, so `/projects` sees it. This is the
/// production entry point; tests use `scaffold_project` instead.
pub fn create_project(name: &str, location: &Path) -> Result<ManagedProject> {
    let target = project_target_path(name, location)?;
    let registry_path = ProjectRegistry::registry_path();
    if let Some(parent) = registry_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let _lock = JsonDirLock::acquire(&registry_path)?;
    let mut registry = ProjectRegistry::load_from(&registry_path);
    anyhow::ensure!(
        !registry.poisoned,
        "refusing to create a project while the project registry is unreadable"
    );
    anyhow::ensure!(
        registry.find(name).is_none() && registry.find_by_path(&target).is_none(),
        "project {name:?} or path {} is already registered",
        target.display()
    );

    let project = scaffold_project(name, location)?;
    registry.projects.push(project.clone());
    if let Err(error) = write_registry_atomic(&registry, &registry_path) {
        if let Err(rollback_error) = std::fs::remove_dir_all(&project.path) {
            return Err(error).context(format!(
                "registry update failed and rollback of {} also failed: {rollback_error}",
                project.path.display()
            ));
        }
        return Err(error);
    }
    Ok(project)
}

fn register_existing_project(
    registry: &mut ProjectRegistry,
    project: ManagedProject,
) -> Result<ManagedProject> {
    let same_name = registry
        .projects
        .iter()
        .position(|candidate| candidate.name.eq_ignore_ascii_case(&project.name));
    let same_path = registry
        .projects
        .iter()
        .position(|candidate| same_project_path(&candidate.path, &project.path));
    if let Some(name_index) = same_name {
        anyhow::ensure!(
            same_path == Some(name_index),
            "project name {:?} is already registered at {}",
            project.name,
            registry.projects[name_index].path.display()
        );
    }
    if let Some(index) = same_path {
        let old_key = project_key(&registry.projects[index]);
        let mut existing = registry.projects[index].clone();
        existing.name = project.name;
        existing.path = project.path;
        let new_key = project_key(&existing);
        registry.projects[index] = existing.clone();
        if old_key != new_key {
            if let Some(fiche) = registry.project_fiches.remove(&old_key) {
                registry.project_fiches.insert(new_key, fiche);
            }
        }
        return Ok(existing);
    }
    registry.projects.push(project.clone());
    Ok(project)
}

pub fn add_existing_project(path: &Path) -> Result<ManagedProject> {
    let path = std::fs::canonicalize(path)
        .with_context(|| format!("resolving existing project path {}", path.display()))?;
    anyhow::ensure!(
        path.is_dir(),
        "project path is not a directory: {}",
        path.display()
    );
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .with_context(|| format!("project path has no UTF-8 name: {}", path.display()))?
        .to_string();
    validate_project_name(&name)?;

    let project = ManagedProject {
        name: name.clone(),
        path: path.clone(),
        telegram_topic_id: None,
        oracle_session: Some(format!("oracle-{}", name)),
        git_email: None,
        created_at: chrono::Utc::now().to_rfc3339(),
        telegram: None,
        category: None,
    };

    // Persist to the registry so /projects sees it next time.
    let project = ProjectRegistry::update_locked(&ProjectRegistry::registry_path(), |registry| {
        register_existing_project(registry, project)
    })?;

    Ok(project)
}

pub fn scan_directory(root: &Path) -> Vec<ManagedProject> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(root) else {
        return found;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if !p.is_dir() {
            continue;
        }
        let Some(name) = p.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name.starts_with('.') || name == "node_modules" || name == "target" {
            continue;
        }
        let is_project = p.join(".git").exists()
            || p.join("package.json").exists()
            || p.join("Cargo.toml").exists()
            || p.join("pyproject.toml").exists()
            || p.join("go.mod").exists();
        if is_project {
            found.push(ManagedProject {
                name: name.to_string(),
                path: p,
                telegram_topic_id: None,
                oracle_session: None,
                git_email: None,
                created_at: chrono::Utc::now().to_rfc3339(),
                telegram: None,
                category: None,
            });
        }
    }
    found.sort_by(|a, b| a.name.cmp(&b.name));
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: this test used to call `create_project()`, which persists to the
    // REAL ~/.omega/projects.json. Running the suite therefore rewrote the
    // machine's own project registry — on 2026-07-29 it replaced 25 projects
    // with a single "TestApp". It now exercises the scaffolding and the
    // registry separately, and never touches the user's registry.
    #[test]
    fn scaffolding_creates_the_expected_layout() {
        let tmp = tempfile::tempdir().unwrap();
        let project = scaffold_project("TestApp", tmp.path()).unwrap();
        assert_eq!(project.name, "TestApp");
        assert!(project.path.join("CLAUDE.md").exists());
        assert!(project.path.join("docs/VISION.md").exists());
        assert!(project.path.join("docs/PRD.md").exists());
        assert!(project.path.join(".planner").exists());
    }

    #[test]
    fn registry_add_is_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        let project = scaffold_project("TestApp", tmp.path()).unwrap();

        let mut registry = ProjectRegistry::default();
        registry.add(project.clone());
        assert_eq!(registry.projects.len(), 1);
        assert!(registry.find("testapp").is_some());

        registry.add(project);
        assert_eq!(registry.projects.len(), 1, "duplicate add is a no-op");
    }

    #[test]
    fn malformed_registry_shape_is_poisoned_and_never_replaced() {
        for malformed in [
            r#"{}"#,
            r#"{"schema_version":0,"projects":[]}"#,
            r#"{"schema_version":"1","projects":[]}"#,
            r#"{"schema_version":1,"projects":{}}"#,
        ] {
            let tmp = tempfile::tempdir().unwrap();
            let path = tmp.path().join("projects.json");
            std::fs::write(&path, malformed).unwrap();

            let registry = ProjectRegistry::load_from(&path);
            assert!(
                registry.poisoned,
                "accepted malformed registry: {malformed}"
            );
            assert!(registry.save_to(&path).is_err());
            assert_eq!(std::fs::read_to_string(&path).unwrap(), malformed);
        }
    }

    #[test]
    fn baseline_less_registry_cannot_replace_an_existing_registry() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("projects.json");
        std::fs::write(
            &path,
            r#"{"projects":[{"name":"A","path":"/p/a","telegram_topic_id":null,"oracle_session":null,"git_email":null,"created_at":"a"}]}"#,
        )
        .unwrap();

        let mut detached = ProjectRegistry::default();
        let mut project = proj("/p/b", None);
        project.name = "B".into();
        detached.add(project);
        assert!(detached.save_to(&path).is_err());

        let preserved = ProjectRegistry::load_from(&path);
        assert_eq!(preserved.projects.len(), 1);
        assert!(preserved.find("A").is_some());
        assert!(preserved.find("B").is_none());
    }

    #[cfg(unix)]
    #[test]
    fn malformed_registry_backup_replaces_a_symlink_without_following_it() {
        use std::os::unix::fs::symlink;

        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("projects.json");
        let backup = path.with_extension("json.unreadable");
        let sentinel = tmp.path().join("sentinel");
        std::fs::write(&path, "{not json").unwrap();
        std::fs::write(&sentinel, "do not overwrite").unwrap();
        symlink(&sentinel, &backup).unwrap();

        let registry = ProjectRegistry::load_from(&path);
        assert!(registry.poisoned);
        assert_eq!(
            std::fs::read_to_string(&sentinel).unwrap(),
            "do not overwrite"
        );
        let metadata = std::fs::symlink_metadata(&backup).unwrap();
        assert!(metadata.file_type().is_file());
        assert!(!metadata.file_type().is_symlink());
        assert_eq!(std::fs::read_to_string(backup).unwrap(), "{not json");
    }

    #[cfg(unix)]
    #[test]
    fn registry_authority_symlink_is_rejected_without_touching_its_target() {
        use std::os::unix::fs::symlink;

        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("target.json");
        let path = tmp.path().join("projects.json");
        let original = r#"{"projects":[]}"#;
        std::fs::write(&target, original).unwrap();
        symlink(&target, &path).unwrap();

        let registry = ProjectRegistry::load_from(&path);
        assert!(registry.poisoned);
        assert!(registry.save_to(&path).is_err());
        assert_eq!(std::fs::read_to_string(target).unwrap(), original);
        assert!(std::fs::symlink_metadata(path)
            .unwrap()
            .file_type()
            .is_symlink());
    }

    #[test]
    fn scaffold_rejects_traversal_and_existing_directories_without_mutation() {
        let tmp = tempfile::tempdir().unwrap();
        for name in [
            "",
            " ../escape",
            "../escape",
            "nested/project",
            "nested\\project",
        ] {
            assert!(
                scaffold_project(name, tmp.path()).is_err(),
                "accepted {name:?}"
            );
        }

        let existing = tmp.path().join("Existing");
        std::fs::create_dir(&existing).unwrap();
        let sentinel = existing.join("CLAUDE.md");
        std::fs::write(&sentinel, "operator content").unwrap();
        assert!(scaffold_project("Existing", tmp.path()).is_err());
        assert_eq!(
            std::fs::read_to_string(sentinel).unwrap(),
            "operator content"
        );
    }

    #[cfg(unix)]
    #[test]
    fn re_registering_a_canonical_path_preserves_metadata_and_fiche() {
        use std::os::unix::fs::symlink;

        let tmp = tempfile::tempdir().unwrap();
        let canonical = tmp.path().join("Alpha");
        let alias = tmp.path().join("legacy-alias");
        std::fs::create_dir(&canonical).unwrap();
        symlink(&canonical, &alias).unwrap();

        let input = serde_json::json!({
            "projects": [{
                "name": "Legacy",
                "path": alias,
                "telegram_topic_id": 42,
                "oracle_session": "oracle-legacy",
                "git_email": "agent@example.invalid",
                "created_at": "stable-created-at",
                "telegram": false,
                "repo": "https://example.invalid/alpha.git",
                "future": {"owner": "external"}
            }]
        });
        let mut registry: ProjectRegistry = serde_json::from_value(input).unwrap();
        let discovered = ManagedProject {
            name: "Alpha".into(),
            path: canonical.clone(),
            telegram_topic_id: None,
            oracle_session: Some("oracle-Alpha".into()),
            git_email: None,
            created_at: "new-value-must-not-win".into(),
            telegram: None,
            category: None,
        };

        let registered = register_existing_project(&mut registry, discovered).unwrap();
        assert_eq!(registry.projects.len(), 1);
        assert_eq!(registered.name, "Alpha");
        assert_eq!(registered.path, canonical);
        assert_eq!(registered.telegram_topic_id, Some(42));
        assert_eq!(registered.created_at, "stable-created-at");
        assert_eq!(registered.telegram, Some(false));
        let fiche = registry.fiche("Alpha").unwrap();
        assert_eq!(
            fiche.repo.as_deref(),
            Some("https://example.invalid/alpha.git")
        );
        assert_eq!(
            fiche.extra["future"],
            serde_json::json!({"owner": "external"})
        );
    }

    #[test]
    fn re_registering_a_name_at_a_different_path_is_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let first = tmp.path().join("one");
        let second = tmp.path().join("two");
        std::fs::create_dir(&first).unwrap();
        std::fs::create_dir(&second).unwrap();
        let mut registry = ProjectRegistry::default();
        registry.projects.push(ManagedProject {
            name: "Same".into(),
            path: first,
            telegram_topic_id: None,
            oracle_session: None,
            git_email: None,
            created_at: "original".into(),
            telegram: None,
            category: None,
        });
        let candidate = ManagedProject {
            name: "same".into(),
            path: second,
            telegram_topic_id: None,
            oracle_session: None,
            git_email: None,
            created_at: "candidate".into(),
            telegram: None,
            category: None,
        };

        assert!(register_existing_project(&mut registry, candidate).is_err());
        assert_eq!(registry.projects.len(), 1);
        assert_eq!(registry.projects[0].created_at, "original");
    }

    /// The data-loss guard: a registry that failed to load must never be
    /// written back over the file it failed to read.
    #[test]
    fn an_unreadable_registry_is_never_saved_over() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("projects.json");
        std::fs::write(&path, "{ this is not json").unwrap();

        let reg = ProjectRegistry::load_from(&path);
        assert!(
            reg.poisoned,
            "an unparseable registry must be marked poisoned"
        );
        assert!(reg.projects.is_empty());
        assert!(
            reg.save_to(&path).is_err(),
            "saving a poisoned registry would erase the user's projects"
        );
        // The original bytes survive, and a copy is preserved for recovery.
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "{ this is not json"
        );
        assert!(path.with_extension("json.unreadable").exists());
    }

    /// …but a genuinely absent registry is just a first run.
    #[test]
    fn an_absent_registry_is_a_clean_start() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("projects.json");
        let reg = ProjectRegistry::load_from(&path);
        assert!(!reg.poisoned);
        assert!(
            reg.save_to(&path).is_ok(),
            "a first run must be able to save"
        );
    }

    /// A save must be atomic — a concurrent reader seeing a half-written file
    /// is what poisons a load and starts the whole failure chain.
    #[test]
    fn save_round_trips_and_leaves_no_temp_file() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("projects.json");
        let scaffold_dir = tempfile::tempdir().unwrap();
        let mut reg = ProjectRegistry::default();
        reg.add(scaffold_project("A", scaffold_dir.path()).unwrap());
        reg.save_to(&path).unwrap();

        let back = ProjectRegistry::load_from(&path);
        assert_eq!(back.projects.len(), 1);
        assert!(!back.poisoned);
        assert!(
            !path.with_extension("json.tmp").exists(),
            "temp file must be renamed away"
        );
    }

    #[test]
    fn telegram_toggle_roundtrip() {
        // Absent field → None → enabled (default ON, backward-compatible).
        let legacy = r#"{"projects":[{"name":"A","path":"/p/a","telegram_topic_id":null,"oracle_session":null,"git_email":null,"created_at":"x"}]}"#;
        let reg: ProjectRegistry = serde_json::from_str(legacy).unwrap();
        assert!(
            reg.find("A").unwrap().telegram_enabled(),
            "absent telegram must default to enabled"
        );

        // Set OFF, serialize, and confirm it persists as `false` across a round-trip.
        let mut reg2 = reg.clone();
        assert!(reg2.set_telegram("a", false));
        assert!(!reg2.find("A").unwrap().telegram_enabled());
        let json = serde_json::to_string(&reg2).unwrap();
        assert!(
            json.contains("\"telegram\":false"),
            "OFF must serialize: {json}"
        );
        let reg3: ProjectRegistry = serde_json::from_str(&json).unwrap();
        assert!(
            !reg3.find("A").unwrap().telegram_enabled(),
            "OFF must survive round-trip"
        );

        // Flip back ON → enabled again, and the field is no longer `false`.
        let mut reg4 = reg3.clone();
        assert!(reg4.set_telegram("a", true));
        assert!(reg4.find("A").unwrap().telegram_enabled());
        assert!(!serde_json::to_string(&reg4)
            .unwrap()
            .contains("\"telegram\":false"));
    }

    #[test]
    fn ecosystem_project_fiche_and_unknown_fields_round_trip() {
        let input = serde_json::json!({
            "schema_version": 7,
            "registry_owner": "external-controller",
            "projects": [{
                "name": "A",
                "path": "/p/a",
                "telegram_topic_id": null,
                "oracle_session": null,
                "git_email": null,
                "created_at": "x",
                "repo": "https://example.invalid/acme/a.git",
                "slug": "acme/a",
                "default_branch": "trunk",
                "deploy_target": {"provider": "example", "project": "a"},
                "credential_refs": ["vault://projects/a"],
                "key_locations": {"env": ".env.enc"},
                "future_field": {"nested": [1, 2, 3]},
                "future_null": null
            }]
        });
        let mut registry: ProjectRegistry = serde_json::from_value(input.clone()).unwrap();
        let fiche = registry.fiche("a").expect("fiche must be available");
        assert_eq!(fiche.slug.as_deref(), Some("acme/a"));
        assert_eq!(fiche.default_branch.as_deref(), Some("trunk"));
        assert!(registry.set_telegram("A", false));

        let output = serde_json::to_value(&registry).unwrap();
        assert_eq!(output["schema_version"], 7);
        assert_eq!(output["registry_owner"], input["registry_owner"]);
        for field in [
            "repo",
            "slug",
            "default_branch",
            "deploy_target",
            "credential_refs",
            "key_locations",
            "future_field",
            "future_null",
        ] {
            assert_eq!(output["projects"][0][field], input["projects"][0][field]);
        }
        assert_eq!(output["projects"][0]["telegram"], false);
    }

    #[test]
    fn ecosystem_concurrent_stale_saves_merge_without_data_loss() {
        use std::sync::{Arc, Barrier};

        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("projects.json");
        std::fs::write(
            &path,
            serde_json::to_vec_pretty(&serde_json::json!({
                "root_unknown": "keep",
                "projects": [{
                    "name": "A",
                    "path": "/p/a",
                    "telegram_topic_id": null,
                    "oracle_session": null,
                    "git_email": null,
                    "created_at": "x",
                    "repo": "https://example.invalid/a.git",
                    "future": 42
                }]
            }))
            .unwrap(),
        )
        .unwrap();

        let mut toggle = ProjectRegistry::load_from(&path);
        let mut addition = ProjectRegistry::load_from(&path);
        assert!(toggle.set_telegram("A", false));
        toggle.fiche_mut("A").unwrap().slug = Some("acme/a".into());
        addition
            .projects
            .iter_mut()
            .find(|project| project.name == "A")
            .unwrap()
            .telegram_topic_id = Some(99);
        addition.fiche_mut("A").unwrap().default_branch = Some("main".into());
        addition.add(proj("/p/b", None));
        addition.projects.last_mut().unwrap().name = "B".into();

        let barrier = Arc::new(Barrier::new(3));
        let toggle_path = path.clone();
        let toggle_barrier = barrier.clone();
        let toggle_thread = std::thread::spawn(move || {
            toggle_barrier.wait();
            toggle.save_to(&toggle_path)
        });
        let addition_path = path.clone();
        let addition_barrier = barrier.clone();
        let addition_thread = std::thread::spawn(move || {
            addition_barrier.wait();
            addition.save_to(&addition_path)
        });
        barrier.wait();
        toggle_thread.join().unwrap().unwrap();
        addition_thread.join().unwrap().unwrap();

        let output: JsonValue = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let registry = ProjectRegistry::load_from(&path);
        assert_eq!(registry.projects.len(), 2);
        assert!(!registry.find("A").unwrap().telegram_enabled());
        assert_eq!(registry.find("A").unwrap().telegram_topic_id, Some(99));
        assert!(registry.find("B").is_some());
        let fiche = registry.fiche("A").unwrap();
        assert_eq!(fiche.slug.as_deref(), Some("acme/a"));
        assert_eq!(fiche.default_branch.as_deref(), Some("main"));
        assert_eq!(output["root_unknown"], "keep");
        let a = output["projects"]
            .as_array()
            .unwrap()
            .iter()
            .find(|project| project["name"] == "A")
            .unwrap();
        assert_eq!(a["repo"], "https://example.invalid/a.git");
        assert_eq!(a["future"], 42);
        assert!(!tmp.path().join("projects.json.lock").exists());
        assert!(std::fs::read_dir(tmp.path()).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with("projects.json.tmp.")
        }));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn ecosystem_json_lock_takeover_is_generation_exact_and_aba_safe() {
        let tmp = tempfile::tempdir().unwrap();
        let data_path = tmp.path().join("projects.json");
        let lock_path = tmp.path().join("projects.json.lock");
        std::fs::create_dir(&lock_path).unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&lock_path, std::fs::Permissions::from_mode(0o700)).unwrap();
        let old = JsonLockOwner {
            schema_version: 1,
            token: "dead-generation".into(),
            pid: u32::MAX,
            process_start: Some("definitely-not-live".into()),
            created_ms: 1,
        };
        std::fs::write(
            lock_path.join(JSON_LOCK_OWNER_FILE),
            serde_json::to_vec(&old).unwrap(),
        )
        .unwrap();
        let old_time = SystemTime::now() - REGISTRY_LOCK_STALE - Duration::from_secs(1);
        let directory = OpenOptions::new().read(true).open(&lock_path).unwrap();
        directory
            .set_times(std::fs::FileTimes::new().set_modified(old_time))
            .unwrap();

        let replacement = JsonDirLock::acquire(&data_path).unwrap();
        let replacement_owner = read_json_lock_owner(&lock_path).unwrap().unwrap();
        assert_ne!(replacement_owner.token, old.token);

        // A late destructor from the stale generation cannot unlink the new
        // owner file or remove its directory. Legacy rmdir-only destructors are
        // also fenced because every replacement lock is non-empty.
        JsonDirLock::release_exact(&lock_path, &old.token).unwrap();
        assert_eq!(
            read_json_lock_owner(&lock_path).unwrap().unwrap().token,
            replacement_owner.token
        );
        assert!(std::fs::remove_dir(&lock_path).is_err());

        drop(replacement);
        assert!(!lock_path.exists());
        assert!(std::fs::read_dir(tmp.path()).unwrap().all(|entry| !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .contains(".stale.")));
    }

    #[cfg(unix)]
    #[test]
    fn ecosystem_json_lock_rejects_aliased_owner_authority() {
        use std::os::unix::fs::{symlink, PermissionsExt};

        let tmp = tempfile::tempdir().unwrap();
        let lock_path = tmp.path().join("projects.json.lock");
        let external = tmp.path().join("external-owner.json");
        std::fs::create_dir(&lock_path).unwrap();
        std::fs::set_permissions(&lock_path, std::fs::Permissions::from_mode(0o700)).unwrap();
        std::fs::write(
            &external,
            serde_json::to_vec(&JsonLockOwner {
                schema_version: 1,
                token: "forged-generation".into(),
                pid: u32::MAX,
                process_start: Some("gone".into()),
                created_ms: 1,
            })
            .unwrap(),
        )
        .unwrap();
        std::fs::set_permissions(&external, std::fs::Permissions::from_mode(0o600)).unwrap();
        symlink(&external, lock_path.join(JSON_LOCK_OWNER_FILE)).unwrap();

        assert!(read_json_lock_owner(&lock_path).is_err());
        assert!(!json_lock_is_stale(&lock_path));

        std::fs::remove_file(lock_path.join(JSON_LOCK_OWNER_FILE)).unwrap();
        std::fs::hard_link(&external, lock_path.join(JSON_LOCK_OWNER_FILE)).unwrap();
        assert!(read_json_lock_owner(&lock_path).is_err());
        assert!(!json_lock_is_stale(&lock_path));

        std::fs::remove_file(lock_path.join(JSON_LOCK_OWNER_FILE)).unwrap();
        std::fs::remove_dir(&lock_path).unwrap();
        symlink(tmp.path(), &lock_path).unwrap();
        assert!(read_json_lock_owner(&lock_path).is_err());
        assert!(!json_lock_is_stale(&lock_path));
    }

    #[test]
    fn ecosystem_concurrent_rename_move_and_fiche_changes_preserve_identity() {
        use std::sync::{Arc, Barrier};

        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("projects.json");
        std::fs::write(
            &path,
            serde_json::to_vec_pretty(&serde_json::json!({
                "projects": [{
                    "name": "Alpha",
                    "path": "/projects/alpha",
                    "telegram_topic_id": null,
                    "oracle_session": null,
                    "git_email": null,
                    "created_at": "2026-08-11T12:00:00Z",
                    "repo": "https://example.invalid/alpha.git",
                    "future": {"owner": "external"}
                }]
            }))
            .unwrap(),
        )
        .unwrap();

        let mut renamed = ProjectRegistry::load_from(&path);
        let mut enriched = ProjectRegistry::load_from(&path);
        renamed.fiche_mut("Alpha").unwrap().slug = Some("acme/omega".into());
        let renamed_project = renamed
            .projects
            .iter_mut()
            .find(|candidate| candidate.name == "Alpha")
            .unwrap();
        renamed_project.name = "Omega".into();
        renamed_project.path = PathBuf::from("/projects/moved/omega");
        enriched
            .projects
            .iter_mut()
            .find(|candidate| candidate.name == "Alpha")
            .unwrap()
            .telegram_topic_id = Some(77);
        enriched.fiche_mut("Alpha").unwrap().default_branch = Some("main".into());

        let barrier = Arc::new(Barrier::new(3));
        let renamed_path = path.clone();
        let renamed_barrier = barrier.clone();
        let renamed_thread = std::thread::spawn(move || {
            renamed_barrier.wait();
            renamed.save_to(&renamed_path)
        });
        let enriched_path = path.clone();
        let enriched_barrier = barrier.clone();
        let enriched_thread = std::thread::spawn(move || {
            enriched_barrier.wait();
            enriched.save_to(&enriched_path)
        });
        barrier.wait();
        renamed_thread.join().unwrap().unwrap();
        enriched_thread.join().unwrap().unwrap();

        let registry = ProjectRegistry::load_from(&path);
        assert!(registry.find("Alpha").is_none());
        let project = registry
            .find("Omega")
            .expect("renamed project must survive");
        assert_eq!(project.path, PathBuf::from("/projects/moved/omega"));
        assert_eq!(project.telegram_topic_id, Some(77));
        let fiche = registry
            .fiche("Omega")
            .expect("fiche must follow rename and move");
        assert_eq!(
            fiche.repo.as_deref(),
            Some("https://example.invalid/alpha.git")
        );
        assert_eq!(fiche.slug.as_deref(), Some("acme/omega"));
        assert_eq!(fiche.default_branch.as_deref(), Some("main"));
        assert_eq!(
            fiche.extra["future"],
            serde_json::json!({"owner": "external"})
        );
    }

    #[test]
    fn duplicate_legacy_timestamps_distinguish_rename_from_delete_plus_add() {
        let input = serde_json::json!({
            "projects": [
                {
                    "name": "A",
                    "path": "/projects/a",
                    "telegram_topic_id": null,
                    "oracle_session": null,
                    "git_email": null,
                    "created_at": "bulk-import",
                    "repo": "https://example.invalid/a.git"
                },
                {
                    "name": "B",
                    "path": "/projects/b",
                    "telegram_topic_id": null,
                    "oracle_session": null,
                    "git_email": null,
                    "created_at": "bulk-import",
                    "repo": "https://example.invalid/b.git"
                }
            ]
        });
        let baseline: ProjectRegistry = serde_json::from_value(input).unwrap();

        let mut renamed = baseline.clone();
        let renamed_b = renamed
            .projects
            .iter_mut()
            .find(|project| project.name == "B")
            .unwrap();
        renamed_b.name = "Renamed B".into();
        renamed_b.path = PathBuf::from("/projects/renamed-b");
        let mut concurrently_updated = baseline.clone();
        concurrently_updated
            .projects
            .iter_mut()
            .find(|project| project.name == "B")
            .unwrap()
            .telegram_topic_id = Some(99);
        let merged_rename = merge_registry_change(&renamed, &concurrently_updated);
        let renamed_b = merged_rename.find("Renamed B").unwrap();
        assert_eq!(renamed_b.telegram_topic_id, Some(99));
        assert_eq!(
            merged_rename.fiche("Renamed B").unwrap().repo.as_deref(),
            Some("https://example.invalid/b.git")
        );

        let mut replaced = baseline.clone();
        assert!(replaced.remove("B"));
        replaced.projects.push(ManagedProject {
            name: "C".into(),
            path: PathBuf::from("/projects/c"),
            telegram_topic_id: None,
            oracle_session: None,
            git_email: None,
            created_at: "new-project".into(),
            telegram: None,
            category: None,
        });
        let merged_replacement = merge_registry_change(&replaced, &concurrently_updated);
        assert!(merged_replacement.find("B").is_none());
        let c = merged_replacement.find("C").unwrap();
        assert_eq!(c.telegram_topic_id, None);
        assert!(merged_replacement.fiche("C").is_none());
    }

    #[test]
    fn ecosystem_concurrent_disjoint_root_metadata_changes_merge_per_key() {
        use std::sync::{Arc, Barrier};

        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("projects.json");
        std::fs::write(
            &path,
            serde_json::to_vec_pretty(&serde_json::json!({
                "controller": {"revision": 1},
                "presentation": {"theme": "dark"},
                "projects": []
            }))
            .unwrap(),
        )
        .unwrap();

        let mut controller = ProjectRegistry::load_from(&path);
        let mut presentation = ProjectRegistry::load_from(&path);
        controller
            .root_extra
            .insert("controller".into(), serde_json::json!({"revision": 2}));
        presentation
            .root_extra
            .insert("presentation".into(), serde_json::json!({"theme": "light"}));

        let barrier = Arc::new(Barrier::new(3));
        let controller_path = path.clone();
        let controller_barrier = barrier.clone();
        let controller_thread = std::thread::spawn(move || {
            controller_barrier.wait();
            controller.save_to(&controller_path)
        });
        let presentation_path = path.clone();
        let presentation_barrier = barrier.clone();
        let presentation_thread = std::thread::spawn(move || {
            presentation_barrier.wait();
            presentation.save_to(&presentation_path)
        });
        barrier.wait();
        controller_thread.join().unwrap().unwrap();
        presentation_thread.join().unwrap().unwrap();

        let output: JsonValue = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(output["controller"], serde_json::json!({"revision": 2}));
        assert_eq!(
            output["presentation"],
            serde_json::json!({"theme": "light"})
        );
    }

    #[test]
    fn scan_finds_projects() {
        let tmp = std::env::temp_dir().join("omega-scan-test");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(tmp.join("project-a/.git")).unwrap();
        std::fs::create_dir_all(tmp.join("project-b")).unwrap();
        std::fs::write(tmp.join("project-b/package.json"), "{}").unwrap();
        std::fs::create_dir_all(tmp.join("not-a-project")).unwrap();

        let found = scan_directory(&tmp);
        assert_eq!(found.len(), 2);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    fn proj(path: &str, category: Option<&str>) -> ManagedProject {
        ManagedProject {
            name: "x".into(),
            path: PathBuf::from(path),
            telegram_topic_id: None,
            oracle_session: None,
            git_email: None,
            created_at: "t".into(),
            telegram: None,
            category: category.map(|s| s.into()),
        }
    }

    #[test]
    fn category_is_derived_from_the_users_own_layout() {
        // Machine-agnostic: the category is the first folder under the user's
        // configured projects root — whatever it is named — NOT a fixed taxonomy.
        let root = Path::new("/home/alice/dev");
        assert_eq!(
            proj("/home/alice/dev/customers/acme", None).display_category(root),
            "customers"
        );
        assert_eq!(
            proj("/home/alice/dev/tools/mylib/crate", None).display_category(root),
            "tools"
        );
        // A different user with a different root and different folder names.
        let root2 = Path::new("/Users/bob/Code");
        assert_eq!(
            proj("/Users/bob/Code/Clients/foo", None).display_category(root2),
            "Clients"
        );
    }

    #[test]
    fn category_falls_back_to_other() {
        let root = Path::new("/home/alice/dev");
        // Directly under the root (no category folder) → Other.
        assert_eq!(
            proj("/home/alice/dev/loose-project", None).display_category(root),
            "Other"
        );
        // Outside the root entirely → Other.
        assert_eq!(
            proj("/somewhere/else/proj", None).display_category(root),
            "Other"
        );
    }

    #[test]
    fn explicit_category_overrides_derivation() {
        let root = Path::new("/home/alice/dev");
        // Even though the path would derive "side-business", the pin wins.
        assert_eq!(
            proj("/home/alice/dev/side-business/omega", Some("Framework")).display_category(root),
            "Framework"
        );
        // Blank/whitespace category is ignored → falls back to derivation.
        assert_eq!(
            proj("/home/alice/dev/tools/x", Some("  ")).display_category(root),
            "tools"
        );
    }

    #[test]
    fn category_rank_sorts_named_alpha_other_last() {
        let mut cats = vec!["Partners", "Other", "framework", "Nova"];
        cats.sort_by_key(|c| ManagedProject::category_rank(c));
        assert_eq!(cats, vec!["framework", "Nova", "Partners", "Other"]);
    }
}

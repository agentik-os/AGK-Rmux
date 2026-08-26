use anyhow::{Context, Result};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static ATOMIC_WRITE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Opaque content revision captured at a successful authority-file load.
///
/// Config values can contain credentials, so the digest is deliberately not
/// printable. It exists only to make a later save a compare-and-swap: a stale
/// in-memory copy must fail instead of silently replacing somebody else's
/// concurrent update.
#[derive(Clone, PartialEq, Eq)]
pub struct PrivateRevision([u8; 32]);

impl PrivateRevision {
    fn from_bytes(bytes: &[u8]) -> Self {
        Self(*blake3::hash(bytes).as_bytes())
    }
}

impl fmt::Debug for PrivateRevision {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("<private-revision>")
    }
}

pub(crate) struct PrivateFileLock(File);

impl Drop for PrivateFileLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.0);
    }
}

/// Read an optional authority/config file through a descriptor that cannot
/// follow a final-component symlink. Existing files must be regular, owned by
/// the current user, and have exactly one hard link. Group/world permission
/// drift is tightened on the already-open descriptor before any content is
/// trusted, then descriptor/path identity is checked again after the read.
pub(crate) fn read_private_optional(path: &Path) -> Result<Option<Vec<u8>>> {
    let before = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).with_context(|| format!("inspecting {}", path.display())),
    };
    if !before.file_type().is_file() {
        anyhow::bail!(
            "refusing non-regular authority file {} (symlinks are not trusted)",
            path.display()
        );
    }

    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(no_follow_flag());
    }
    let mut file = options.open(path).with_context(|| {
        format!(
            "opening authority file {} without symlink following",
            path.display()
        )
    })?;
    let opened = file
        .metadata()
        .with_context(|| format!("inspecting opened authority file {}", path.display()))?;
    validate_private_file_identity(path, &before, &opened)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        let mode = opened.mode();
        if mode & 0o077 != 0 {
            tracing::warn!(
                path = %path.display(),
                mode = format_args!("{:o}", mode & 0o777),
                "authority file permissions were broader than owner-only; tightening through the verified descriptor"
            );
            file.set_permissions(std::fs::Permissions::from_mode(mode & !0o077))
                .with_context(|| {
                    format!(
                        "tightening group/world permissions on authority file {}",
                        path.display()
                    )
                })?;
        }
    }

    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .with_context(|| format!("reading authority file {}", path.display()))?;
    let after = std::fs::symlink_metadata(path)
        .with_context(|| format!("re-checking authority file {}", path.display()))?;
    let final_opened = file
        .metadata()
        .with_context(|| format!("re-checking opened authority file {}", path.display()))?;
    validate_private_file_identity(path, &after, &final_opened)?;
    Ok(Some(bytes))
}

pub(crate) fn read_private_optional_string(path: &Path) -> Result<Option<String>> {
    read_private_optional(path)?
        .map(|bytes| {
            String::from_utf8(bytes)
                .with_context(|| format!("authority file {} is not UTF-8", path.display()))
        })
        .transpose()
}

fn validate_private_file_identity(
    path: &Path,
    path_metadata: &std::fs::Metadata,
    opened_metadata: &std::fs::Metadata,
) -> Result<()> {
    if !path_metadata.file_type().is_file() || !opened_metadata.file_type().is_file() {
        anyhow::bail!("authority path {} is not a regular file", path.display());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if path_metadata.dev() != opened_metadata.dev()
            || path_metadata.ino() != opened_metadata.ino()
        {
            anyhow::bail!(
                "authority path {} changed identity while being opened",
                path.display()
            );
        }
        if opened_metadata.nlink() != 1 {
            anyhow::bail!(
                "authority file {} has {} hard links; expected exactly one",
                path.display(),
                opened_metadata.nlink()
            );
        }
        if opened_metadata.uid() != effective_uid() {
            anyhow::bail!(
                "authority file {} is owned by uid {}, current uid is {}",
                path.display(),
                opened_metadata.uid(),
                effective_uid()
            );
        }
    }
    Ok(())
}

#[cfg(unix)]
pub(crate) fn effective_uid() -> u32 {
    unsafe extern "C" {
        fn geteuid() -> u32;
    }
    // SAFETY: geteuid takes no arguments, has no preconditions, and only
    // returns the kernel's effective uid for this process.
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

/// Serialize a read/validate/write transaction for one private authority file.
/// The lock lives beside the target, is never opened through a symlink, and is
/// held until the supplied operation returns.
pub(crate) fn with_private_file_lock<T>(
    path: &Path,
    operation: impl FnOnce() -> Result<T>,
) -> Result<T> {
    let _guard = acquire_private_file_lock(path)?;
    operation()
}

pub(crate) fn acquire_private_file_lock(path: &Path) -> Result<PrivateFileLock> {
    let parent = path
        .parent()
        .with_context(|| format!("{} has no parent directory", path.display()))?;
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("omega-state");
    let lock_path = parent.join(format!(".{filename}.omega-lock"));
    acquire_private_lock_path(&lock_path)
        .with_context(|| format!("locking authority file {}", path.display()))
}

/// Acquire an exact private lock path for subsystems with an established lock
/// filename. Unlike [`acquire_private_file_lock`], this does not derive a
/// sibling name from an authority target.
pub(crate) fn acquire_private_lock_path(lock_path: &Path) -> Result<PrivateFileLock> {
    let parent = lock_path
        .parent()
        .with_context(|| format!("{} has no parent directory", lock_path.display()))?;
    std::fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;

    if let Ok(metadata) = std::fs::symlink_metadata(lock_path) {
        if !metadata.file_type().is_file() {
            anyhow::bail!(
                "refusing non-regular authority lock {}",
                lock_path.display()
            );
        }
    }

    let mut options = std::fs::OpenOptions::new();
    options.read(true).write(true).create(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600).custom_flags(no_follow_flag());
    }
    let file = options
        .open(lock_path)
        .with_context(|| format!("opening authority lock {}", lock_path.display()))?;
    let path_metadata = std::fs::symlink_metadata(lock_path)
        .with_context(|| format!("inspecting authority lock {}", lock_path.display()))?;
    let opened_metadata = file
        .metadata()
        .with_context(|| format!("inspecting opened authority lock {}", lock_path.display()))?;
    validate_private_file_identity(lock_path, &path_metadata, &opened_metadata)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(std::fs::Permissions::from_mode(0o600))
            .with_context(|| format!("tightening authority lock {}", lock_path.display()))?;
    }
    file.lock_exclusive()
        .with_context(|| format!("locking private file {}", lock_path.display()))?;
    Ok(PrivateFileLock(file))
}

pub(crate) fn private_revision(bytes: &[u8]) -> PrivateRevision {
    PrivateRevision::from_bytes(bytes)
}

pub(crate) fn private_toml_error(
    label: &str,
    path: &Path,
    span: Option<std::ops::Range<usize>>,
) -> anyhow::Error {
    let location = span
        .map(|range| format!(" near byte range {}..{}", range.start, range.end))
        .unwrap_or_default();
    anyhow::anyhow!(
        "{label} {} failed strict TOML/schema validation{location}; inspect the owner-only file directly",
        path.display()
    )
}

/// Refuse a stale full-file save. `None` means the caller loaded an absent
/// file, so creation is allowed only while it is still absent.
pub(crate) fn require_private_revision(
    path: &Path,
    expected: Option<&PrivateRevision>,
) -> Result<()> {
    let current = read_private_optional(path)?;
    match (expected, current.as_deref()) {
        (None, None) => Ok(()),
        (Some(expected), Some(bytes)) if &PrivateRevision::from_bytes(bytes) == expected => Ok(()),
        _ => anyhow::bail!(
            "refusing stale write to {}; reload and retry the mutation",
            path.display()
        ),
    }
}

/// Atomically replace a private OmegaOS state/config file.
///
/// The staged file is created beside the destination with `create_new`, synced,
/// owner-locked, then renamed over the destination. A destination symlink is
/// replaced rather than followed. Keeping this helper in the config module
/// gives provider, active-model, and session metadata writes one crash-safe
/// implementation without widening the public API.
pub(crate) fn atomic_write_private(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .with_context(|| format!("{} has no parent directory", path.display()))?;
    std::fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;

    let serial = ATOMIC_WRITE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("omega-state");
    let staged = parent.join(format!(
        ".{filename}.omega-tmp-{}-{serial}",
        std::process::id()
    ));

    let result = (|| -> Result<()> {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options
            .open(&staged)
            .with_context(|| format!("creating staged file {}", staged.display()))?;
        file.write_all(bytes)
            .with_context(|| format!("writing staged file {}", staged.display()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(std::fs::Permissions::from_mode(0o600))
                .with_context(|| format!("setting owner-only mode on {}", staged.display()))?;
        }
        file.sync_all()
            .with_context(|| format!("syncing staged file {}", staged.display()))?;
        drop(file);
        std::fs::rename(&staged, path).with_context(|| {
            format!(
                "atomically replacing {} with {}",
                path.display(),
                staged.display()
            )
        })?;
        if let Ok(directory) = std::fs::File::open(parent) {
            directory
                .sync_all()
                .with_context(|| format!("syncing {}", parent.display()))?;
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&staged);
    }
    result
}

#[derive(Debug, Clone, Serialize, Deserialize)]
// Every field falls back to the manual `Default` impl below, so a partial or
// even empty config.toml deserializes cleanly instead of silently discarding
// the whole file (`OmegaConfig::load()` used to Err on any missing field).
#[serde(default, deny_unknown_fields)]
pub struct OmegaConfig {
    pub state_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub locks_dir: PathBuf,
    /// Root under which project categories (work/clients/...) live. Auto-detected
    /// per-user (see `default_projects_dir`) — never a hardcoded ~/VibeCoding.
    /// Override in ~/.omega/config.toml.
    #[serde(default = "default_projects_dir")]
    pub projects_dir: PathBuf,
    pub projects: Vec<ProjectConfig>,
    /// Runtime used when a caller does not provide an explicit agent.
    ///
    /// Missing fields use Codex. An explicit serialized value is always
    /// preserved, including `claude`: config loading has no provenance proving
    /// whether an old value was untouched or deliberately chosen.
    #[serde(default = "default_agent_command")]
    pub agent_command: String,
    pub default_model: String,
    /// Legacy compatibility field. The `aisb-master` rmux session is now a
    /// read-only viewer and does not launch a provider, so runtime code must
    /// not use this value to imply an active AISB agent.
    #[serde(default = "default_aisb_agent")]
    pub aisb_agent: String,
    #[serde(default = "default_auto_master")]
    pub auto_spawn_master: bool,
    #[serde(default = "default_auto_naming")]
    pub auto_naming: bool,
    /// Single-key rmux session shortcuts in the Sessions tab (x kill, r rename,
    /// b background, . cleanup, …). Default ON. When OFF, the Sessions tab takes
    /// only arrow navigation + Enter/Esc/Tab — so no action fires by accident.
    /// (Arrow-only navigation is always enforced there; j/k never move the cursor.)
    #[serde(default = "default_session_shortcuts")]
    pub session_shortcuts: bool,
    /// TUI color theme slug (Settings → Theme). The registry of valid slugs
    /// lives in omega-tui's theme module; an unknown slug falls back to the
    /// default "omega" palette at load time.
    #[serde(default = "default_theme")]
    pub theme: String,
    /// Paint the active theme's background color across the whole TUI frame
    /// (Settings → Theme). Default ON. When OFF the terminal's own background
    /// shows through (transparency, background images, user terminal colors),
    /// while accent/text colors still follow the theme.
    #[serde(default = "default_theme_background")]
    pub theme_background: bool,
    /// IANA timezone for the on-screen clock (e.g. "Europe/Paris", "America/New_York").
    /// Persisted timestamps stay UTC — this localizes ONLY the displayed clock,
    /// because a headless VPS runs in UTC and the operator usually does not.
    /// `None`/empty → fall back to `$TZ`, then the system local zone.
    #[serde(default)]
    pub timezone: Option<String>,
    /// What the daily update cron is allowed to do (`omega update --auto`).
    /// Default `Apply` — the check runs every day and the update is installed.
    #[serde(default)]
    pub auto_update: AutoUpdatePolicy,
    pub telegram: Option<TelegramConfig>,
    /// Revision of the exact file this value was loaded from. Public only so
    /// downstream crates can continue using struct-update syntax; callers must
    /// treat it as opaque.
    #[doc(hidden)]
    #[serde(skip)]
    pub source_revision: Option<PrivateRevision>,
}

/// How far the daily update cron may go on its own.
///
/// This is the ONE switch between "OmegaOS keeps itself current" and "OmegaOS
/// tells me and I decide". It exists because auto-applying code pulled from a
/// remote is a real trust decision, not a preference: `Apply` means whoever
/// controls the repo controls this machine's OmegaOS — the same trust the
/// installer already required, now renewed daily instead of once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AutoUpdatePolicy {
    /// Check daily and install what is available. The default.
    #[default]
    Apply,
    /// Check daily, alert when an update exists, install nothing.
    Check,
    /// Do nothing at all — the cron exits immediately.
    Off,
}

impl AutoUpdatePolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Apply => "apply",
            Self::Check => "check",
            Self::Off => "off",
        }
    }

    /// Parse a user-typed value (config file, `omega config set`). Unknown text
    /// is NOT silently treated as `off` — a typo must not quietly stop updates,
    /// so it falls back to the default.
    pub fn parse(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "check" | "notify" | "check-only" => Self::Check,
            "off" | "false" | "no" | "disabled" | "never" => Self::Off,
            _ => Self::Apply,
        }
    }
}

fn default_agent_command() -> String {
    "codex".to_string()
}

fn default_aisb_agent() -> String {
    "codex".to_string()
}

fn default_auto_master() -> bool {
    // Retired: the brain is the Atlas Telegram bot (omega-tg-bot.ts), not the legacy
    // `aisb-master` rmux session (a Telegram-conversation viewer). Don't auto-spawn it.
    false
}

fn default_auto_naming() -> bool {
    true
}

fn default_session_shortcuts() -> bool {
    true
}

fn default_theme() -> String {
    "omega".to_string()
}

fn default_theme_background() -> bool {
    true
}

/// Auto-detect the user's project root: the first existing common work
/// container under $HOME, else ~/projects. Cross-user — NO ~/VibeCoding
/// hardcode, so a fresh install adapts to whatever layout the user has.
fn default_projects_dir() -> PathBuf {
    // `home_dir()` already consults $HOME on Unix; if it still fails the home is
    // genuinely unresolvable. Don't return the bare world-writable /tmp as a
    // PERSISTENT projects root (collision + data-loss-on-reboot risk) — scope it
    // to a per-user subdir under the temp dir as a last resort, and warn loudly
    // so the operator knows their projects are not landing under $HOME.
    let home = dirs::home_dir().unwrap_or_else(|| {
        let user = std::env::var("USER").unwrap_or_else(|_| "omega".to_string());
        let fallback = std::env::temp_dir().join(format!("omega-projects-{user}"));
        eprintln!(
            "omega: could not resolve a home directory ($HOME unset) — falling back \
             to a temp projects root at {}. This is EPHEMERAL; set $HOME or \
             projects_dir in config.toml for persistent storage.",
            fallback.display()
        );
        fallback
    });
    for cand in ["VibeCoding", "projects", "Projects", "code", "dev", "work"] {
        let p = home.join(cand);
        if p.is_dir() {
            return p;
        }
    }
    home.join("projects")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfig {
    pub name: String,
    pub path: PathBuf,
    pub category: ProjectCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectCategory {
    Work,
    Client,
    Personal,
    System,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub chat_id: i64,
}

impl fmt::Debug for TelegramConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TelegramConfig")
            .field("has_bot_token", &!self.bot_token.is_empty())
            .field("chat_id", &self.chat_id)
            .finish()
    }
}

/// The OmegaOS system root — the single source of truth for where all state,
/// config, credentials, agents, skills, and logs live.
///
/// Resolution order (first hit wins):
///   1. `$OMEGA_DIR`            — explicit override (install.sh already honors it;
///      this makes the binary agree, fixing a long-standing
///      install.sh-vs-binary split).
///   2. `$HOME/OmegaOS/System`  — the consolidated 3-folder layout, IF it exists
///      (a fresh install or the migration creates it).
///   3. `$HOME/.omega`          — the legacy dotfolder, for machines not migrated.
///
/// Every other path in the codebase derives from this, so relocating the whole
/// system is one env var or one migration, not an N-site rewrite.
pub fn omega_dir() -> PathBuf {
    if let Ok(d) = std::env::var("OMEGA_DIR") {
        if !d.is_empty() {
            let configured = PathBuf::from(&d);
            if configured.is_absolute()
                && configured.parent().is_some()
                && !configured
                    .components()
                    .any(|component| component == std::path::Component::ParentDir)
            {
                return configured;
            }
            eprintln!(
                "omega: ignoring unsafe OMEGA_DIR {:?}; authority roots must be absolute, non-root, and traversal-free",
                d
            );
        }
    }
    let home = dirs::home_dir().unwrap_or_else(|| {
        #[cfg(unix)]
        let identity = effective_uid().to_string();
        #[cfg(not(unix))]
        let identity = std::env::var("USERNAME").unwrap_or_else(|_| "omega".to_string());
        let fallback = std::env::temp_dir().join(format!("omega-{identity}"));
        eprintln!(
            "omega: home directory unavailable; using isolated temporary authority root {}",
            fallback.display()
        );
        fallback
    });
    let consolidated = home.join("OmegaOS").join("System");
    if consolidated.is_dir() {
        return consolidated;
    }
    home.join(".omega")
}

impl Default for OmegaConfig {
    fn default() -> Self {
        let omega_dir = omega_dir();
        Self {
            state_dir: omega_dir.join("state"),
            logs_dir: omega_dir.join("logs"),
            locks_dir: omega_dir.join("locks"),
            projects_dir: default_projects_dir(),
            projects: Vec::new(),
            // Codex is the default runtime for every new session whose caller
            // does not deliberately choose another provider. Deserialization
            // preserves every explicit value, including a legacy `claude`.
            agent_command: default_agent_command(),
            // This field is provider-specific. Codex reads its model/effort
            // from ~/.codex/config.toml; "opus" remains the Claude default for
            // missions that explicitly select Claude.
            default_model: "opus".to_string(),
            aisb_agent: default_aisb_agent(),
            auto_spawn_master: default_auto_master(),
            auto_naming: default_auto_naming(),
            session_shortcuts: default_session_shortcuts(),
            theme: default_theme(),
            theme_background: default_theme_background(),
            timezone: None,
            auto_update: AutoUpdatePolicy::default(),
            telegram: None,
            source_revision: None,
        }
    }
}

impl OmegaConfig {
    pub fn load() -> Result<Self> {
        Self::load_from(&Self::config_path())
    }

    /// Load from an explicit path (testable without touching $OMEGA_DIR).
    ///
    /// A malformed existing config.toml is never discarded into defaults. The
    /// original remains untouched and an owner-only `.corrupt` snapshot is
    /// written atomically for recovery before the error is returned.
    fn load_from(config_path: &Path) -> Result<Self> {
        let Some(content) = read_private_optional_string(config_path)? else {
            return Ok(Self::default());
        };
        match toml::from_str(&content) {
            Ok(cfg) => {
                let mut cfg: Self = cfg;
                cfg.validate().with_context(|| {
                    format!("validating runtime config {}", config_path.display())
                })?;
                cfg.source_revision = Some(private_revision(content.as_bytes()));
                Ok(cfg)
            }
            Err(error) => {
                let safe_error = private_toml_error("runtime config", config_path, error.span());
                let backup = config_path.with_file_name(format!(
                    "{}.corrupt",
                    config_path
                        .file_name()
                        .map(|n| n.to_string_lossy())
                        .unwrap_or_default()
                ));
                if let Err(backup_error) = atomic_write_private(&backup, content.as_bytes()) {
                    tracing::error!(
                        path = %backup.display(),
                        error = %backup_error,
                        "failed to preserve corrupt OmegaOS config snapshot"
                    );
                }
                eprintln!(
                    "omega: config parse failure at {}; runtime authority refused. \
                     Original left untouched; recovery snapshot: {}. {}",
                    config_path.display(),
                    backup.display(),
                    safe_error
                );
                tracing::error!(
                    path = %config_path.display(),
                    backup = %backup.display(),
                    error = %safe_error,
                    "config.toml failed to parse; runtime authority refused"
                );
                Err(safe_error)
            }
        }
    }

    fn validate(&self) -> Result<()> {
        crate::agents::Agent::from_name(&self.agent_command).with_context(|| {
            format!(
                "unknown agent_command {:?}; choose one of: {}",
                self.agent_command,
                crate::agents::Agent::all()
                    .iter()
                    .map(crate::agents::Agent::name)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;
        for (label, path) in [
            ("state_dir", &self.state_dir),
            ("logs_dir", &self.logs_dir),
            ("locks_dir", &self.locks_dir),
            ("projects_dir", &self.projects_dir),
        ] {
            if !path.is_absolute()
                || path.parent().is_none()
                || path
                    .components()
                    .any(|component| component == std::path::Component::ParentDir)
            {
                anyhow::bail!(
                    "{label} must be an absolute, non-root, traversal-free path: {:?}",
                    path
                );
            }
        }
        for project in &self.projects {
            if project.name.trim().is_empty() || project.name.chars().any(char::is_control) {
                anyhow::bail!("project names must be non-empty and contain no control characters");
            }
            if !project.path.is_absolute()
                || project.path.parent().is_none()
                || project
                    .path
                    .components()
                    .any(|component| component == std::path::Component::ParentDir)
            {
                anyhow::bail!(
                    "project path for {:?} must be absolute, non-root, and traversal-free: {:?}",
                    project.name,
                    project.path
                );
            }
        }
        Ok(())
    }

    pub fn config_path() -> PathBuf {
        omega_dir().join("config.toml")
    }

    /// Persist the complete config without exposing a torn or group-readable
    /// file. An unreadable/corrupt existing file is never overwritten.
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        self.save_to(&path)
    }

    fn save_to(&self, path: &Path) -> Result<()> {
        self.validate()
            .with_context(|| format!("validating OmegaOS config for {}", path.display()))?;
        let content = toml::to_string_pretty(self).context("serializing OmegaOS config")?;
        with_private_file_lock(path, || {
            require_private_revision(path, self.source_revision.as_ref())?;
            atomic_write_private(path, content.as_bytes())
                .with_context(|| format!("writing OmegaOS config {}", path.display()))
        })
    }

    /// Persist the auto-update policy into config.toml.
    ///
    /// Rewrites ONLY that one key and leaves every other byte — including the
    /// user's comments and formatting — untouched. Serializing the whole struct
    /// back would be shorter and would silently strip a hand-written config
    /// down to whatever the struct happens to model.
    ///
    /// The key must be written ABOVE the first `[table]` header, or TOML parses
    /// it as a member of that table instead of a top-level key.
    pub fn set_auto_update(policy: AutoUpdatePolicy) -> Result<()> {
        let path = Self::config_path();
        Self::set_auto_update_at(&path, policy)
    }

    fn set_auto_update_at(path: &Path, policy: AutoUpdatePolicy) -> Result<()> {
        with_private_file_lock(path, || {
            let line = format!("auto_update = \"{}\"", policy.as_str());
            let existing = match read_private_optional_string(path)? {
                Some(existing) => {
                    toml::from_str::<toml::Value>(&existing).with_context(|| {
                        format!(
                            "refusing to update auto_update in malformed config {}",
                            path.display()
                        )
                    })?;
                    existing
                }
                None => String::new(),
            };

            let mut out: Vec<String> = Vec::new();
            let mut written = false;
            for raw in existing.lines() {
                let trimmed = raw.trim_start();
                if trimmed.starts_with("auto_update") && trimmed.contains('=') && !written {
                    out.push(line.clone());
                    written = true;
                    continue;
                }
                // First table header: the last point where a top-level key is still
                // top-level. Insert here if we have not written the key yet.
                if !written && trimmed.starts_with('[') {
                    out.push(line.clone());
                    out.push(String::new());
                    written = true;
                }
                out.push(raw.to_string());
            }
            if !written {
                out.push(line);
            }

            let mut body = out.join("\n");
            body.push('\n');
            atomic_write_private(path, body.as_bytes())
        })
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        std::fs::create_dir_all(&self.state_dir)?;
        std::fs::create_dir_all(&self.logs_dir)?;
        std::fs::create_dir_all(&self.locks_dir)?;
        Ok(())
    }

    /// Resolve a project category (customer/side-business/tools/personal/...)
    /// to an absolute directory under `projects_dir`. Cross-user — no hardcoded
    /// paths.
    pub fn resolve_category_path(&self, category: &str) -> PathBuf {
        // New installs use customer / side-business / tools. Older layouts used
        // clients / work / 1-life. Prefer an existing conventional dir so we
        // don't fragment a user's tree, then fall back to the new canonical name.
        let candidates: &[&str] = match category {
            "customer" | "customers" | "client" | "clients" => &["customers", "clients"],
            "side-business" | "side_business" | "sidebusiness" | "work" | "works" => {
                &["side-business", "work"]
            }
            "tool" | "tools" => &["tools"],
            "personal" | "life" | "1-life" => &["1-life", "personal"],
            other => return self.projects_dir.join(other),
        };
        candidates
            .iter()
            .map(|c| self.projects_dir.join(c))
            .find(|p| p.is_dir())
            .unwrap_or_else(|| self.projects_dir.join(candidates[0]))
    }

    pub fn find_project(&self, name: &str) -> Option<&ProjectConfig> {
        let lower = name.to_lowercase();
        self.projects
            .iter()
            .find(|p| p.name.to_lowercase() == lower)
    }

    pub fn discover_projects(scan_dirs: &[&Path]) -> Vec<ProjectConfig> {
        let mut projects = Vec::new();
        for dir in scan_dirs {
            if !dir.exists() {
                continue;
            }
            let category = match dir.file_name().and_then(|n| n.to_str()) {
                Some("clients") | Some("customers") => ProjectCategory::Client,
                Some("1-life") | Some("personal") => ProjectCategory::Personal,
                _ => ProjectCategory::Work, // work, side-business, tools, ...
            };
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }
                    let is_project = path.join(".git").exists()
                        || path.join("package.json").exists()
                        || path.join("Cargo.toml").exists()
                        || path.join("pyproject.toml").exists()
                        || path.join("go.mod").exists();
                    if is_project {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            projects.push(ProjectConfig {
                                name: name.to_string(),
                                path,
                                category: category.clone(),
                            });
                        }
                    }
                }
            }
        }
        projects.sort_by(|a, b| a.name.cmp(&b.name));
        projects
    }
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::*;

    #[test]
    fn auto_update_defaults_to_apply_so_installs_stay_current() {
        assert_eq!(OmegaConfig::default().auto_update, AutoUpdatePolicy::Apply);
        // And an empty config.toml must not silently turn updates off.
        let parsed: OmegaConfig = toml::from_str("").unwrap();
        assert_eq!(parsed.auto_update, AutoUpdatePolicy::Apply);
    }

    #[test]
    fn auto_update_round_trips_through_config_toml() {
        for policy in [
            AutoUpdatePolicy::Off,
            AutoUpdatePolicy::Check,
            AutoUpdatePolicy::Apply,
        ] {
            let toml_str = format!("auto_update = \"{}\"", policy.as_str());
            let parsed: OmegaConfig = toml::from_str(&toml_str).unwrap();
            assert_eq!(parsed.auto_update, policy);
        }
    }

    #[test]
    fn setting_the_policy_preserves_the_rest_of_the_file() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("config.toml");
        std::fs::write(
            &path,
            "# my notes\nagent_command = \"claude\"\n\n[telegram]\nbot_token = \"t\"\nchat_id = 42\n",
        )
        .unwrap();

        OmegaConfig::set_auto_update_at(&path, AutoUpdatePolicy::Off).unwrap();
        let back = std::fs::read_to_string(&path).unwrap();
        assert!(back.contains("# my notes"), "comments survive");
        assert!(back.contains("chat_id = 42"), "telegram survives");
        assert!(back.contains("auto_update = \"off\""));

        // And it must still parse, with the key top-level rather than swallowed
        // into [telegram].
        let cfg: OmegaConfig = toml::from_str(&back).unwrap();
        assert_eq!(cfg.auto_update, AutoUpdatePolicy::Off);
        assert!(cfg.telegram.is_some());
    }

    #[test]
    fn setting_the_policy_twice_replaces_rather_than_appends() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("config.toml");
        OmegaConfig::set_auto_update_at(&path, AutoUpdatePolicy::Off).unwrap();
        OmegaConfig::set_auto_update_at(&path, AutoUpdatePolicy::Check).unwrap();
        let back = std::fs::read_to_string(&path).unwrap();
        assert_eq!(back.matches("auto_update").count(), 1, "one key, not two");
        let cfg: OmegaConfig = toml::from_str(&back).unwrap();
        assert_eq!(cfg.auto_update, AutoUpdatePolicy::Check);
    }

    #[test]
    fn resolve_category_path_is_under_projects_dir_no_vibecoding() {
        // Fresh layout (no category dir exists yet) → the new canonical names.
        let c = OmegaConfig {
            projects_dir: PathBuf::from("/home/someuser/projects"),
            ..Default::default()
        };
        assert_eq!(
            c.resolve_category_path("customer"),
            PathBuf::from("/home/someuser/projects/customers")
        );
        assert_eq!(
            c.resolve_category_path("side-business"),
            PathBuf::from("/home/someuser/projects/side-business")
        );
        assert_eq!(
            c.resolve_category_path("tools"),
            PathBuf::from("/home/someuser/projects/tools")
        );
        assert_eq!(
            c.resolve_category_path("1-life"),
            PathBuf::from("/home/someuser/projects/1-life")
        );
        // Old aliases fall to the new canonical when no legacy dir exists.
        assert_eq!(
            c.resolve_category_path("client"),
            PathBuf::from("/home/someuser/projects/customers")
        );
        assert_eq!(
            c.resolve_category_path("work"),
            PathBuf::from("/home/someuser/projects/side-business")
        );
        // Unknown category → uses its own name as the subdir.
        assert_eq!(
            c.resolve_category_path("research"),
            PathBuf::from("/home/someuser/projects/research")
        );
        // Never a hardcoded ~/VibeCoding.
        assert!(c
            .resolve_category_path("customer")
            .starts_with("/home/someuser/projects"));
    }

    #[test]
    fn resolve_category_path_prefers_existing_legacy_dir() {
        // Backward-compat: a machine already using clients/ + work/ keeps them
        // instead of fragmenting into customers/ + side-business/.
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("clients")).unwrap();
        std::fs::create_dir_all(tmp.path().join("work")).unwrap();
        let c = OmegaConfig {
            projects_dir: tmp.path().to_path_buf(),
            ..Default::default()
        };
        assert_eq!(
            c.resolve_category_path("customer"),
            tmp.path().join("clients")
        );
        assert_eq!(
            c.resolve_category_path("side-business"),
            tmp.path().join("work")
        );
    }

    #[test]
    fn shipped_default_toml_deserializes() {
        // The shipped template MUST parse into OmegaConfig — it used to fail and
        // silently discard the whole config on a fresh install.
        let cfg: OmegaConfig = toml::from_str(include_str!("../../../config/default.toml"))
            .expect("config/default.toml must deserialize into OmegaConfig");
        assert_eq!(cfg.agent_command, "codex");
        assert_eq!(cfg.aisb_agent, "codex");
        assert_eq!(cfg.default_model, "opus");
    }

    #[test]
    fn fresh_and_missing_agent_configuration_defaults_to_codex() {
        let fresh = OmegaConfig::default();
        assert_eq!(fresh.agent_command, "codex");
        assert_eq!(fresh.aisb_agent, "codex");

        // A partial pre-existing config with no agent field adopts the new
        // default without requiring a rewrite of the operator's file.
        let missing: OmegaConfig = toml::from_str("default_model = \"sonnet\"\n").unwrap();
        assert_eq!(missing.agent_command, "codex");
        assert_eq!(missing.aisb_agent, "codex");
        assert_eq!(missing.default_model, "sonnet");
    }

    #[test]
    fn explicit_agent_choices_are_never_migrated_by_config_loading() {
        for explicit in ["claude", "gemini", "codex"] {
            let body = format!("agent_command = \"{explicit}\"\naisb_agent = \"{explicit}\"\n");
            let parsed: OmegaConfig = toml::from_str(&body).unwrap();
            assert_eq!(parsed.agent_command, explicit);
            assert_eq!(parsed.aisb_agent, explicit);
        }

        // `load_from` has no provenance that could distinguish an untouched
        // legacy "claude" from a deliberate operator choice, so it preserves it.
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("config.toml");
        std::fs::write(&path, "agent_command = \"claude\"\n").unwrap();
        let loaded = OmegaConfig::load_from(&path).unwrap();
        assert_eq!(loaded.agent_command, "claude");
    }

    #[test]
    fn theme_background_defaults_on_and_round_trips() {
        // Pre-existing configs (no theme_background key) must keep painting the
        // background — default ON, not serde's bool default (false).
        let cfg: OmegaConfig = toml::from_str("").unwrap();
        assert!(cfg.theme_background);
        // An explicit OFF survives a save/load round-trip.
        let cfg = OmegaConfig {
            theme_background: false,
            ..Default::default()
        };
        let reloaded: OmegaConfig = toml::from_str(&toml::to_string_pretty(&cfg).unwrap()).unwrap();
        assert!(!reloaded.theme_background);
    }

    #[test]
    fn partial_config_overrides_only_its_keys() {
        // A partial config overrides its keys and keeps defaults for the rest,
        // instead of nuking the whole file.
        let cfg: OmegaConfig = toml::from_str("default_model = \"sonnet\"\n").unwrap();
        assert_eq!(cfg.default_model, "sonnet"); // overridden
        assert_eq!(cfg.agent_command, "codex"); // still the default, not empty
    }

    #[test]
    fn malformed_config_surfaces_error_and_preserves_original() {
        // A corrupt config.toml must NOT be silently swallowed into defaults:
        // load_from returns Err (so callers can choose) and copies the original
        // to .corrupt so the operator can recover their telegram token / projects.
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("config.toml");
        std::fs::write(&path, "default_model = \"opus\"\nbroken = = =\n[[[").unwrap();

        let r = OmegaConfig::load_from(&path);
        assert!(
            r.is_err(),
            "malformed config must surface an error, not default silently"
        );

        let backup = tmp.path().join("config.toml.corrupt");
        assert!(
            backup.exists(),
            "the unparseable original must be preserved at .corrupt"
        );
        assert_eq!(
            std::fs::read_to_string(&backup).unwrap(),
            std::fs::read_to_string(&path).unwrap(),
            "the .corrupt backup must be a faithful copy of the original"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(backup).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }

    #[test]
    fn semantic_unknown_agent_config_is_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("config.toml");
        std::fs::write(&path, "agent_command = \"codxe\"\n").unwrap();
        let error = OmegaConfig::load_from(&path).unwrap_err();
        assert!(
            format!("{error:#}").contains("unknown agent_command"),
            "{error:#}"
        );
    }

    #[test]
    fn malformed_config_is_not_overwritten_by_mutations() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("config.toml");
        let corrupt = b"agent_command = = nope\n";
        std::fs::write(&path, corrupt).unwrap();

        assert!(OmegaConfig::set_auto_update_at(&path, AutoUpdatePolicy::Off).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), corrupt);
        assert!(OmegaConfig::default().save_to(&path).is_err());
        assert_eq!(std::fs::read(path).unwrap(), corrupt);
    }

    #[test]
    fn config_rejects_unknown_keys_and_stale_saves() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("config.toml");
        std::fs::write(
            &path,
            "agent_command = 'codex'\n[telegram]\nbot_token = 'top-secret'\nchat_id = 1\ntypo = true\n",
        )
        .unwrap();
        let error = OmegaConfig::load_from(&path).unwrap_err();
        assert!(!format!("{error:#}").contains("top-secret"), "{error:#}");
        assert!(
            format!("{error:#}").contains("strict TOML/schema validation"),
            "{error:#}"
        );

        std::fs::remove_file(&path).unwrap();
        OmegaConfig::default().save_to(&path).unwrap();
        let mut first = OmegaConfig::load_from(&path).unwrap();
        let mut stale = OmegaConfig::load_from(&path).unwrap();
        first.default_model = "first-writer".to_string();
        stale.default_model = "stale-writer".to_string();
        first.save_to(&path).unwrap();
        assert!(stale.save_to(&path).is_err());
        assert_eq!(
            OmegaConfig::load_from(&path).unwrap().default_model,
            "first-writer"
        );
    }

    #[test]
    fn runtime_config_debug_redacts_telegram_token() {
        let config = OmegaConfig {
            telegram: Some(TelegramConfig {
                bot_token: "top-secret-token".to_string(),
                chat_id: 42,
            }),
            ..Default::default()
        };
        let debug = format!("{config:?}");
        assert!(!debug.contains("top-secret-token"));
        assert!(debug.contains("has_bot_token: true"));
    }

    #[test]
    fn runtime_config_rejects_broad_or_traversing_authority_roots() {
        let root = OmegaConfig {
            state_dir: PathBuf::from("/"),
            ..Default::default()
        };
        assert!(root.validate().is_err());

        let traversal = OmegaConfig {
            projects_dir: PathBuf::from("/tmp/omega/../other"),
            ..Default::default()
        };
        assert!(traversal.validate().is_err());

        let project_root = OmegaConfig {
            projects: vec![ProjectConfig {
                name: "unsafe".to_string(),
                path: PathBuf::from("/"),
                category: ProjectCategory::System,
            }],
            ..Default::default()
        };
        assert!(project_root.validate().is_err());
    }

    #[cfg(unix)]
    #[test]
    fn private_authority_lock_rejects_symlink_and_hardlink_poisoning() {
        use std::os::unix::fs::{symlink, PermissionsExt};

        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("config.toml");
        let lock = tmp.path().join(".config.toml.omega-lock");
        let victim = tmp.path().join("victim");
        std::fs::write(&victim, "unchanged").unwrap();
        let mode = std::fs::metadata(&victim).unwrap().permissions().mode() & 0o777;

        symlink(&victim, &lock).unwrap();
        assert!(OmegaConfig::default().save_to(&path).is_err());
        assert_eq!(std::fs::read_to_string(&victim).unwrap(), "unchanged");
        assert_eq!(
            std::fs::metadata(&victim).unwrap().permissions().mode() & 0o777,
            mode
        );

        std::fs::remove_file(&lock).unwrap();
        std::fs::hard_link(&victim, &lock).unwrap();
        assert!(OmegaConfig::default().save_to(&path).is_err());
        assert_eq!(std::fs::read_to_string(&victim).unwrap(), "unchanged");
        assert_eq!(
            std::fs::metadata(&victim).unwrap().permissions().mode() & 0o777,
            mode
        );
    }

    #[test]
    fn full_config_save_is_atomic_and_owner_only() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("config.toml");
        OmegaConfig::default().save_to(&path).unwrap();
        assert_eq!(
            OmegaConfig::load_from(&path).unwrap().agent_command,
            "codex"
        );
        assert_eq!(
            std::fs::read_dir(tmp.path())
                .unwrap()
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.file_name().to_string_lossy().contains("omega-tmp"))
                .count(),
            0
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn atomic_private_write_replaces_destination_symlink_without_following_it() {
        use std::os::unix::fs::symlink;

        let tmp = tempfile::tempdir().unwrap();
        let victim = tmp.path().join("victim");
        let destination = tmp.path().join("config.toml");
        std::fs::write(&victim, "do not replace").unwrap();
        symlink(&victim, &destination).unwrap();

        atomic_write_private(&destination, b"safe").unwrap();
        assert_eq!(std::fs::read_to_string(&victim).unwrap(), "do not replace");
        assert_eq!(std::fs::read_to_string(&destination).unwrap(), "safe");
        assert!(!std::fs::symlink_metadata(destination)
            .unwrap()
            .file_type()
            .is_symlink());
    }

    #[cfg(unix)]
    #[test]
    fn config_load_rejects_symlink_and_dangling_symlink_without_touching_targets() {
        use std::os::unix::fs::{symlink, PermissionsExt};

        let tmp = tempfile::tempdir().unwrap();
        let victim = tmp.path().join("victim.toml");
        let linked = tmp.path().join("config.toml");
        std::fs::write(&victim, "agent_command = \"codex\"\n").unwrap();
        let original_mode = std::fs::metadata(&victim).unwrap().permissions().mode() & 0o777;
        symlink(&victim, &linked).unwrap();

        assert!(OmegaConfig::load_from(&linked).is_err());
        assert_eq!(
            std::fs::read_to_string(&victim).unwrap(),
            "agent_command = \"codex\"\n"
        );
        assert_eq!(
            std::fs::metadata(&victim).unwrap().permissions().mode() & 0o777,
            original_mode,
            "a rejected symlink must not chmod its target"
        );

        std::fs::remove_file(&linked).unwrap();
        symlink(tmp.path().join("missing.toml"), &linked).unwrap();
        assert!(OmegaConfig::load_from(&linked).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn config_load_rejects_hardlinks_before_changing_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::tempdir().unwrap();
        let victim = tmp.path().join("victim.toml");
        let linked = tmp.path().join("config.toml");
        std::fs::write(&victim, "agent_command = \"codex\"\n").unwrap();
        let original_mode = std::fs::metadata(&victim).unwrap().permissions().mode() & 0o777;
        std::fs::hard_link(&victim, &linked).unwrap();

        let error = OmegaConfig::load_from(&linked).unwrap_err();
        assert!(error.to_string().contains("hard links"), "{error:#}");
        assert_eq!(
            std::fs::metadata(&victim).unwrap().permissions().mode() & 0o777,
            original_mode,
            "a rejected hardlink must not chmod the shared inode"
        );
        assert_eq!(
            std::fs::read_to_string(&victim).unwrap(),
            "agent_command = \"codex\"\n"
        );
    }

    #[cfg(unix)]
    #[test]
    fn config_load_tightens_lax_permissions_on_the_verified_descriptor() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("config.toml");
        std::fs::write(&path, "agent_command = \"codex\"\n").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o666)).unwrap();

        OmegaConfig::load_from(&path).unwrap();
        assert_eq!(
            std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

    #[test]
    fn absent_config_is_default_not_error() {
        let tmp = tempfile::tempdir().unwrap();
        let cfg = OmegaConfig::load_from(&tmp.path().join("nonexistent.toml")).unwrap();
        assert_eq!(cfg.agent_command, "codex");
        assert_eq!(cfg.aisb_agent, "codex");
    }

    #[test]
    fn omega_dir_honors_explicit_env_override() {
        // $OMEGA_DIR wins over everything — this is what lets the whole system
        // relocate with one env var (and makes the binary agree with install.sh).
        // Serialize the env mutation; cargo runs tests in parallel.
        static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev = std::env::var("OMEGA_DIR").ok();
        std::env::set_var("OMEGA_DIR", "/tmp/omega-test-root");
        assert_eq!(omega_dir(), PathBuf::from("/tmp/omega-test-root"));
        // empty value is ignored (falls through to a real default, never "")
        std::env::set_var("OMEGA_DIR", "");
        assert_ne!(omega_dir(), PathBuf::from(""));
        match prev {
            Some(v) => std::env::set_var("OMEGA_DIR", v),
            None => std::env::remove_var("OMEGA_DIR"),
        }
    }
}

/// Locate the OmegaOS source checkout on this machine, if there is one.
///
/// Lives here rather than in the CLI because `doctor` needs the same answer:
/// two resolvers would drift, and the one that drifts is the one that decides
/// whether your binary is stale. `$OMEGA_SRC` wins, then the current directory,
/// then the usual install locations. A candidate only counts when it carries
/// BOTH `OMEGA.md` and `crates/omega-core` — a bare directory named OmegaOS is
/// not a checkout.
pub fn resolve_omega_src() -> Option<std::path::PathBuf> {
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(src) = std::env::var("OMEGA_SRC") {
        if !src.is_empty() {
            candidates.push(std::path::PathBuf::from(src));
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd);
    }
    if let Some(home) = dirs::home_dir() {
        candidates.push(home.join("Station/SideBusiness/OmegaOS"));
        candidates.push(home.join("Station/OmegaOS"));
        candidates.push(home.join("OmegaOS"));
    }
    candidates
        .into_iter()
        .find(|p| p.join("OMEGA.md").is_file() && p.join("crates/omega-core").is_dir())
}

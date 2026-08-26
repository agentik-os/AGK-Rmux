use crate::agents::{Agent, AgentLaunch};
use anyhow::{Context, Result};
use rmux_sdk::{
    EnsureSession, EnsureSessionPolicy, Pane, ProcessSpec, Rmux, Session, SessionName,
    SplitDirection, TerminalSizeSpec,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

/// Maximum session-name length. Keeps rmux happy AND keeps any Telegram
/// `callback_data` that embeds a session name under the 64-byte API cap
/// (longest prefix today is `stop_workers:` = 13 bytes; 13 + 48 = 61 < 64).
pub const MAX_SESSION_NAME_LEN: usize = 48;
const TYPED_AGENT_SESSION_POLICY: EnsureSessionPolicy = EnsureSessionPolicy::CreateOnly;

/// Strip the host-shell color killers before rmux can inherit them.
/// Cursor agent terminals set `NO_COLOR=1` and `FORCE_COLOR=0`; if the
/// daemon starts from that process, every Claude/Codex pane goes grayscale.
pub fn sanitize_host_color_env() {
    std::env::remove_var("NO_COLOR");
    match std::env::var("FORCE_COLOR") {
        Ok(value) if value == "0" || value.eq_ignore_ascii_case("false") || value.is_empty() => {
            std::env::set_var("FORCE_COLOR", "1");
        }
        Err(_) => std::env::set_var("FORCE_COLOR", "1"),
        Ok(_) => {}
    }
    if std::env::var("COLORTERM").as_deref().unwrap_or("") != "truecolor" {
        std::env::set_var("COLORTERM", "truecolor");
    }
}

/// Best-effort: force color onto an already-running daemon's session env.
/// Does not rewrite a live agent process — those keep the env they started
/// with until relaunch.
async fn apply_rmux_pane_color_env() {
    for args in [
        ["set-environment", "-g", "FORCE_COLOR", "1"].as_slice(),
        ["set-environment", "-g", "COLORTERM", "truecolor"].as_slice(),
        ["set-option", "-sa", "terminal-features", ",*:RGB"].as_slice(),
    ] {
        let _ = tokio::process::Command::new("rmux")
            .args(args)
            .output()
            .await;
    }
    // Prefer unset. rmux 0.3.1 rejects a valueless assignment; `-u` is the
    // documented form and is ignored if the name is already absent.
    let _ = tokio::process::Command::new("rmux")
        .args(["set-environment", "-gu", "NO_COLOR"])
        .output()
        .await;
}

pub const SESSION_DISPATCH_AUTHORITY_SCHEMA_VERSION: u32 = 1;
pub const DISPATCH_GENERATION_ENV: &str = "OMEGA_DISPATCH_GENERATION";
pub const SCOPE_CLAIM_ID_ENV: &str = "OMEGA_SCOPE_CLAIM_ID";

/// Immutable receipt for one concrete same-name session generation.
///
/// It contains no credential or prompt material. The receipt is safe to pass
/// through a structured child-process environment and lets cleanup distinguish
/// a late generation A signal from the current generation B session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionDispatchAuthority {
    pub schema_version: u32,
    pub session: String,
    pub dispatch_generation: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_claim_id: Option<String>,
}

impl SessionDispatchAuthority {
    pub fn generate(session: &str, scope_claim_id: Option<&str>) -> Result<Self> {
        validate_canonical_session_name(session)?;
        if let Some(claim_id) = scope_claim_id {
            validate_generation_token("scope claim", claim_id)?;
        }
        let mut entropy = [0_u8; 32];
        getrandom::fill(&mut entropy).map_err(|error| {
            anyhow::anyhow!("reading OS entropy for dispatch generation failed: {error}")
        })?;
        let authority = Self {
            schema_version: SESSION_DISPATCH_AUTHORITY_SCHEMA_VERSION,
            session: session.to_string(),
            dispatch_generation: blake3::hash(&entropy).to_hex().to_string(),
            scope_claim_id: scope_claim_id.map(str::to_string),
        };
        authority.validate()?;
        Ok(authority)
    }

    pub fn validate(&self) -> Result<()> {
        if self.schema_version != SESSION_DISPATCH_AUTHORITY_SCHEMA_VERSION {
            anyhow::bail!(
                "unsupported session dispatch authority schema {}",
                self.schema_version
            );
        }
        validate_canonical_session_name(&self.session)?;
        validate_generation_token("dispatch", &self.dispatch_generation)?;
        if let Some(claim_id) = &self.scope_claim_id {
            validate_generation_token("scope claim", claim_id)?;
        }
        Ok(())
    }

    pub fn environment(&self) -> Vec<(String, String)> {
        let mut environment = vec![(
            DISPATCH_GENERATION_ENV.to_string(),
            self.dispatch_generation.clone(),
        )];
        if let Some(claim_id) = &self.scope_claim_id {
            environment.push((SCOPE_CLAIM_ID_ENV.to_string(), claim_id.clone()));
        }
        environment
    }

    pub fn read_strict(state_dir: &Path, session: &str) -> Result<Option<Self>> {
        validate_canonical_session_name(session)?;
        let _lock = session_authority_lock(state_dir, session)?;
        Self::read_locked(state_dir, session)
    }

    pub fn validate_current(&self, state_dir: &Path) -> Result<()> {
        self.validate()?;
        let _lock = session_authority_lock(state_dir, &self.session)?;
        self.validate_current_locked(state_dir)
    }

    pub fn remove_exact(&self, state_dir: &Path) -> Result<()> {
        self.validate()?;
        let _lock = session_authority_lock(state_dir, &self.session)?;
        self.validate_current_locked(state_dir)?;
        crate::scope::remove_private_file(&session_authority_path(state_dir, &self.session)?)
    }

    fn read_locked(state_dir: &Path, session: &str) -> Result<Option<Self>> {
        let authority =
            crate::scope::read_private_json::<Self>(&session_authority_path(state_dir, session)?)?;
        if let Some(authority) = &authority {
            authority.validate()?;
            if authority.session != session {
                anyhow::bail!(
                    "session authority filename/document mismatch: expected {session}, found {}",
                    authority.session
                );
            }
        }
        Ok(authority)
    }

    fn write_locked(&self, state_dir: &Path) -> Result<()> {
        self.validate()?;
        let path = session_authority_path(state_dir, &self.session)?;
        // A create-only launch proved there was no live same-name session.
        // Replacing a valid stale receipt is therefore safe while this lock is
        // held; the strict read prevents overwriting unsafe metadata.
        let _ = Self::read_locked(state_dir, &self.session)?;
        crate::config::atomic_write_private(&path, &serde_json::to_vec_pretty(self)?)?;
        let published = Self::read_locked(state_dir, &self.session)?
            .ok_or_else(|| anyhow::anyhow!("session dispatch authority vanished after publish"))?;
        if published != *self {
            anyhow::bail!("session dispatch authority changed during publish");
        }
        Ok(())
    }

    fn validate_current_locked(&self, state_dir: &Path) -> Result<()> {
        let current = Self::read_locked(state_dir, &self.session)?
            .ok_or_else(|| anyhow::anyhow!("session dispatch authority is absent"))?;
        if current != *self {
            anyhow::bail!(
                "session dispatch generation changed: expected {}, current {}",
                self.dispatch_generation,
                current.dispatch_generation
            );
        }
        Ok(())
    }
}

fn validate_canonical_session_name(session: &str) -> Result<()> {
    crate::scope::validate_session_identity(session)?;
    let canonical = sanitize_session_name(session);
    if canonical != session {
        anyhow::bail!(
            "session authority requires exact canonical identity `{canonical}`, received `{session}`"
        );
    }
    Ok(())
}

fn validate_generation_token(label: &str, generation: &str) -> Result<()> {
    if generation.len() != 64 || !generation.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        anyhow::bail!("invalid {label} generation token");
    }
    Ok(())
}

fn session_authority_path(state_dir: &Path, session: &str) -> Result<PathBuf> {
    validate_canonical_session_name(session)?;
    Ok(state_dir.join(format!("session-authority-{session}.json")))
}

fn session_authority_lock(state_dir: &Path, session: &str) -> Result<std::fs::File> {
    validate_canonical_session_name(session)?;
    crate::scope::lock_private_state_file(state_dir, &format!(".session-authority-{session}.lock"))
}

/// Expand `~` / `~/…` the way the operator typed `--dir ~/Desktop`.
///
/// A literal `~/Desktop` cwd is not a real folder. Codex then splash-exits
/// and the pane looks like bash. TUI New Codex never passed a raw tilde.
/// Codex itself is fine when launched from the Omega menu.
pub fn expand_user_path(raw: &str) -> PathBuf {
    let trimmed = raw.trim();
    if trimmed == "~" {
        return dirs::home_dir().unwrap_or_else(|| PathBuf::from(trimmed));
    }
    if let Some(rest) = trimmed.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(trimmed)
}

/// Resolve `--dir` for `omega new`: omit (`None` = same as TUI, rmux cwd),
/// expand `~`, and refuse a path that is not an existing directory.
pub fn resolve_session_working_dir(raw: Option<&str>) -> Result<Option<PathBuf>> {
    let Some(raw) = raw else {
        return Ok(None);
    };
    let expanded = expand_user_path(raw);
    if !expanded.is_dir() {
        anyhow::bail!(
            "--dir '{}' does not exist (expanded: {}). Create it or pass a real directory.",
            raw,
            expanded.display()
        );
    }
    Ok(Some(expanded))
}

/// Slugify an arbitrary string into a safe rmux session name.
///
/// rmux keys kill/rename/capture on the session name; spaces, non-ASCII, and
/// punctuation (`:` `?` U+202F …) break that lookup and produce un-killable
/// zombie sessions. This collapses anything outside `[A-Za-z0-9._-]` to a
/// single `-`, trims separators, and bounds the length. Idempotent on
/// already-clean names (`oracle-Kommu`, `aisb-master` map to themselves).
pub fn sanitize_session_name(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len().min(MAX_SESSION_NAME_LEN));
    let mut last_dash = false;
    for ch in raw.chars() {
        let ch = fold_accent(ch);
        if ch.is_ascii_alphanumeric() || ch == '.' || ch == '_' || ch == '-' {
            out.push(ch);
            last_dash = false;
        } else if !last_dash {
            // Any space / accent / punctuation collapses to one '-'.
            out.push('-');
            last_dash = true;
        }
        if out.len() >= MAX_SESSION_NAME_LEN {
            break;
        }
    }
    let trimmed = out.trim_matches(|c| c == '-' || c == '.').to_string();
    if trimmed.is_empty() {
        "session".to_string()
    } else {
        trimmed
    }
}

/// Operators name sessions in French ("Camélia") — fold the common Latin-1
/// accents to their ASCII base letter so sanitize yields "Camelia" instead of
/// the mid-word dash "Cam-lia" the generic collapse would produce.
fn fold_accent(ch: char) -> char {
    match ch {
        'à' | 'â' | 'ä' | 'á' | 'ã' => 'a',
        'À' | 'Â' | 'Ä' | 'Á' | 'Ã' => 'A',
        'é' | 'è' | 'ê' | 'ë' => 'e',
        'É' | 'È' | 'Ê' | 'Ë' => 'E',
        'î' | 'ï' | 'í' => 'i',
        'Î' | 'Ï' | 'Í' => 'I',
        'ô' | 'ö' | 'ó' | 'õ' => 'o',
        'Ô' | 'Ö' | 'Ó' | 'Õ' => 'O',
        'ù' | 'û' | 'ü' | 'ú' => 'u',
        'Ù' | 'Û' | 'Ü' | 'Ú' => 'U',
        'ç' => 'c',
        'Ç' => 'C',
        'ñ' => 'n',
        'Ñ' => 'N',
        _ => ch,
    }
}

fn resolve_agent_command(agent_command: &str) -> Result<Agent> {
    Agent::from_name(agent_command).with_context(|| {
        format!(
            "unknown agent provider {agent_command:?}; expected one of: {}",
            Agent::all()
                .iter()
                .map(Agent::name)
                .collect::<Vec<_>>()
                .join(", ")
        )
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionRole {
    Oracle,
    Worker,
    Home,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OmegaSession {
    pub name: String,
    pub role: SessionRole,
    pub project: Option<String>,
    pub oracle_index: Option<u32>,
    pub working_dir: Option<PathBuf>,
    /// Provider recorded at session creation. `None` for generic terminals
    /// and sessions created by older OmegaOS versions.
    #[serde(default)]
    pub provider: Option<String>,
}

impl OmegaSession {
    pub fn classify(name: &str) -> Self {
        let (role, project, oracle_index) = Self::parse_session_name(name);
        Self {
            name: name.to_string(),
            role,
            project,
            oracle_index,
            working_dir: None,
            provider: None,
        }
    }

    fn parse_session_name(name: &str) -> (SessionRole, Option<String>, Option<u32>) {
        // Oracle pattern first — most specific
        if let Some(rest) = name.strip_prefix("oracle-") {
            let (project, idx) = Self::extract_project_and_index(rest);
            return (SessionRole::Oracle, Some(project), idx);
        }

        // Worker pattern: <Project>-(worker|fix|dev|dispatch|...)-
        // Check this BEFORE the system-prefix check so that e.g. AISB-worker-X
        // is correctly identified as a Worker under project AISB.
        let worker_suffixes = [
            "-worker-",
            "-fix-",
            "-dev-",
            "-dispatch-",
            "-work-",
            "-linear",
            "-task-",
            "-audit-",
            "-challenger-",
            "-report-",
            "-verify-",
            "-build-",
            "-deploy-",
            "-team-",
        ];
        for suffix in &worker_suffixes {
            // rfind (RIGHTMOST match): the role suffix is the LAST component, so
            // a suffix string that also appears inside the PROJECT name (e.g.
            // project "my-fix" in "my-fix-worker-X") must not shadow the real
            // role suffix and truncate the project to "my".
            if let Some(pos) = name.rfind(suffix) {
                let project = name[..pos].to_string();
                return (SessionRole::Worker, Some(project), None);
            }
        }

        // Team session: Team-<Project>
        if let Some(rest) = name.strip_prefix("Team-") {
            return (SessionRole::Worker, Some(rest.to_string()), None);
        }

        // Home sessions
        if name.starts_with("Home") || name.starts_with("c-") {
            return (SessionRole::Home, None, None);
        }

        // System daemons (only true daemons, not project-prefixed sessions)
        let system_exact = ["AISB-monitor", "AISB-daemon", "AISB-master"];
        for sys in &system_exact {
            if name == *sys {
                return (SessionRole::System, None, None);
            }
        }
        if name.starts_with("earthbit-") {
            return (SessionRole::System, None, None);
        }

        (SessionRole::Home, None, None)
    }

    fn extract_project_and_index(rest: &str) -> (String, Option<u32>) {
        if let Some(last_dash) = rest.rfind('-') {
            if let Ok(idx) = rest[last_dash + 1..].parse::<u32>() {
                return (rest[..last_dash].to_string(), Some(idx));
            }
        }
        (rest.to_string(), None)
    }

    pub fn display_name(&self) -> &str {
        &self.name
    }
}

fn session_provider_path(name: &str) -> PathBuf {
    crate::config::omega_dir().join("state").join(format!(
        "session-provider-{}.json",
        sanitize_session_name(name)
    ))
}

fn record_session_provider(name: &str, agent: Agent) -> Result<()> {
    let path = session_provider_path(name);
    let payload = serde_json::json!({
        "session": sanitize_session_name(name),
        "provider": agent.name(),
        "recorded_at": chrono::Utc::now().to_rfc3339(),
    });
    let bytes = serde_json::to_vec_pretty(&payload).context("serializing session provider")?;
    crate::config::atomic_write_private(&path, &bytes)
        .with_context(|| format!("recording provider for session {name}"))?;
    // Same chokepoint as provider provenance: TUI New Codex and `omega new
    // --agent` both land here. Launch starts `running`; observe marks `failed`
    // if the agent dies to bash or the pane disappears.
    let state_dir = crate::config::omega_dir().join("state");
    if let Err(error) = crate::session_health::record_launch(&state_dir, name, agent.name()) {
        tracing::warn!(session = %name, error = %error, "failed to record session launch health");
    }
    Ok(())
}

pub fn read_session_provider(name: &str) -> Option<String> {
    let payload: serde_json::Value =
        serde_json::from_slice(&std::fs::read(session_provider_path(name)).ok()?).ok()?;
    let recorded_session = payload.get("session")?.as_str()?;
    if recorded_session != sanitize_session_name(name) {
        return None;
    }
    let provider = payload.get("provider")?.as_str()?;
    Agent::from_name(provider).map(|agent| agent.name().to_string())
}

#[derive(Clone)]
pub struct SessionManager {
    rmux: Arc<Rmux>,
    /// Process-wide Pane handle cache keyed by session name.
    ///
    /// Each keystroke in the TUI chat-focused right panel ultimately calls
    /// `send_text` / `send_text_raw` / `send_key`. Without this cache, every
    /// call resolves the active pane by issuing a fresh `rmux.session(name)`
    /// RPC to the daemon (5–15ms over the local Unix socket). At 60 FPS
    /// typing that's ~120ms of stacked blocking per second — visible as
    /// "typing feels laggy in Hermux".
    ///
    /// The cache stores cloned `Pane` handles (Pane is `#[derive(Clone)]` —
    /// just an Arc<endpoint> + Arc<transport>) so hot-path lookups become a
    /// single mutex acquisition + HashMap get, with zero daemon RPCs. The
    /// cache is invalidated on `kill_session` and on send errors.
    pane_cache: Arc<tokio::sync::Mutex<HashMap<String, Pane>>>,
}

// Process-wide singleton — reused across every Action handler in the TUI
// and Telegram bridge. The original SessionManager::connect() was opening
// a fresh rmux daemon socket every call (~30-50ms latency per call), which
// stacked up to >100ms perceived latency per keystroke in interactive
// passthrough. The cached path serves the same Arc<Rmux> to every caller.
// RwLock<Option<..>> rather than OnceCell: a OnceCell pins the FIRST
// connection forever, so once that socket goes bad (daemon restart, protocol
// desync after a timeout) every caller is stuck on a dead connection with no
// recovery path short of restarting the whole process. reset_cached() lets
// the refresh loop self-heal by forcing the next connect_cached() to redial.
static CACHED_MANAGER: tokio::sync::RwLock<Option<SessionManager>> =
    tokio::sync::RwLock::const_new(None);

impl SessionManager {
    pub async fn connect() -> Result<Self> {
        // Must run BEFORE connect_or_start: a missing daemon is spawned from
        // this process, and Cursor/CI shells start us with NO_COLOR=1
        // FORCE_COLOR=0. That grayscale env then becomes every pane's env.
        sanitize_host_color_env();
        let rmux = Rmux::builder()
            .default_timeout(Duration::from_secs(10))
            .connect_or_start()
            .await
            .context("Failed to connect to rmux daemon")?;
        apply_rmux_pane_color_env().await;
        Ok(Self {
            rmux: Arc::new(rmux),
            pane_cache: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        })
    }

    /// Process-wide cached SessionManager. First call connects, every
    /// subsequent call hands back a clone (Arc<Rmux> share). Use this
    /// in hot paths (per-keystroke forwarding, capture refresh).
    pub async fn connect_cached() -> Result<Self> {
        if let Some(mgr) = CACHED_MANAGER.read().await.as_ref() {
            return Ok(mgr.clone());
        }
        let mut guard = CACHED_MANAGER.write().await;
        // Double-checked: another task may have connected while we waited.
        if let Some(mgr) = guard.as_ref() {
            return Ok(mgr.clone());
        }
        let mgr = Self::connect().await?;
        *guard = Some(mgr.clone());
        Ok(mgr)
    }

    /// Drop the process-wide cached connection so the next connect_cached()
    /// dials the daemon fresh. Call when daemon RPCs start failing — the
    /// cached socket may be dead (daemon restarted) or desynced.
    pub async fn reset_cached() {
        *CACHED_MANAGER.write().await = None;
    }

    pub async fn create_session(
        &self,
        name: &str,
        working_dir: Option<&str>,
        command: Option<&str>,
    ) -> Result<Session> {
        self.create_session_with_environment(name, working_dir, command, &[])
            .await
    }

    pub(crate) async fn create_session_with_environment(
        &self,
        name: &str,
        working_dir: Option<&str>,
        command: Option<&str>,
        environment: &[(String, String)],
    ) -> Result<Session> {
        self.create_session_with_environment_and_policy(
            name,
            working_dir,
            command,
            environment,
            EnsureSessionPolicy::CreateOrReuse,
        )
        .await
    }

    /// Create one explicit-command session bound to an immutable dispatch
    /// generation. Authority variables travel through rmux's structured
    /// environment and the receipt is published only after create-only
    /// succeeds. A metadata failure rolls the new session back.
    pub async fn create_command_session_create_only_with_authority(
        &self,
        state_dir: &Path,
        name: &str,
        working_dir: Option<&str>,
        command: &str,
        authority: &SessionDispatchAuthority,
    ) -> Result<Session> {
        authority.validate()?;
        if authority.session != name {
            anyhow::bail!(
                "session authority identity {} differs from launch target {name}",
                authority.session
            );
        }
        let _lock = session_authority_lock(state_dir, name)?;
        let environment = authority.environment();
        let session = self
            .create_session_with_environment_and_policy(
                name,
                working_dir,
                Some(command),
                &environment,
                EnsureSessionPolicy::CreateOnly,
            )
            .await?;
        if let Err(error) = authority.write_locked(state_dir) {
            let rollback = session.kill().await.err();
            return Err(error).with_context(|| match rollback {
                Some(rollback) => format!(
                    "dispatch authority publication failed; session rollback also failed: {rollback}"
                ),
                None => "dispatch authority publication failed; session was rolled back"
                    .to_string(),
            });
        }
        Ok(session)
    }

    async fn create_session_with_environment_and_policy(
        &self,
        name: &str,
        working_dir: Option<&str>,
        command: Option<&str>,
        environment: &[(String, String)],
        policy: EnsureSessionPolicy,
    ) -> Result<Session> {
        // Defense-in-depth chokepoint: rmux keys every operation (kill, rename,
        // capture) on the session name, and names with spaces, non-ASCII, or
        // punctuation (`:`, `?`, U+202F …) break that lookup — the session
        // becomes permanently un-killable. This happened when raw Telegram
        // message text leaked into a project/oracle name. Slugify here so a
        // dangerous name can NEVER reach rmux, regardless of caller. Idempotent:
        // already-clean names (oracle-Kommu, aisb-master) pass through unchanged.
        let safe = sanitize_session_name(name);
        let session_name = SessionName::new(&safe)?;
        let mut builder = EnsureSession::named(session_name)
            .policy(policy)
            .detached(true)
            .size(TerminalSizeSpec::new(200, 50));

        if let Some(cmd) = command {
            let mut process = ProcessSpec::shell(cmd);
            let mut env = environment.to_vec();
            if !env.iter().any(|(key, _)| key == "OMEGA_SESSION") {
                env.push(("OMEGA_SESSION".to_string(), safe.clone()));
            }
            if !env.is_empty() {
                process.environment = Some(
                    env.iter()
                        .map(|(key, value)| format!("{key}={value}"))
                        .collect(),
                );
            }
            builder = builder.process(process);
        } else if !environment.is_empty() {
            anyhow::bail!("session environment requires an explicit process command");
        }

        if let Some(dir) = working_dir {
            builder = builder.working_directory(dir);
        }

        let session = self
            .rmux
            .ensure_session(builder)
            .await
            .context("Failed to create session")?;
        Ok(session)
    }

    pub async fn create_agent_session(
        &self,
        name: &str,
        working_dir: &str,
        agent_command: &str,
        prompt: Option<&str>,
    ) -> Result<Session> {
        let agent = resolve_agent_command(agent_command)?;
        let launch = agent.try_launch(prompt)?;
        self.create_recorded_agent_session(name, Some(working_dir), agent, launch)
            .await
    }

    pub async fn create_session_with_agent(
        &self,
        name: &str,
        working_dir: Option<&str>,
        agent: Agent,
        prompt: Option<&str>,
    ) -> Result<Session> {
        let launch = agent.try_launch(prompt)?;
        self.create_recorded_agent_session(name, working_dir, agent, launch)
            .await
    }

    /// Spawn an agent session with full LaunchOptions. Claude's interactive
    /// lane intentionally omits its print-only max-turn/max-budget flags.
    pub async fn create_agent_session_with_opts(
        &self,
        name: &str,
        working_dir: &str,
        agent: Agent,
        prompt: Option<&str>,
        opts: crate::agents::LaunchOptions,
    ) -> Result<Session> {
        let launch = agent.try_launch_with(prompt, opts)?;
        self.create_recorded_agent_session(name, Some(working_dir), agent, launch)
            .await
    }

    /// Atomically replace a pane process with a typed agent launch. rmux sends
    /// credentials through its structured environment field, so repair never
    /// types secrets into a shell or exposes them in process argv.
    pub async fn relaunch_agent_session_with_opts(
        &self,
        name: &str,
        working_dir: Option<&str>,
        agent: Agent,
        prompt: Option<&str>,
        opts: crate::agents::LaunchOptions,
    ) -> Result<()> {
        let launch = agent.try_launch_with(prompt, opts)?;
        let session = self.get_session(name).await?;
        let pane = session.pane(0, 0);
        let (command, environment) = launch.into_parts();
        let mut spawn = pane.shell(command).kill_existing(true);
        if let Some(dir) = working_dir {
            spawn = spawn.cwd(dir);
        }
        for (key, value) in environment {
            spawn = spawn.env(key, value);
        }
        spawn
            .await
            .context("Failed to relaunch agent process in session pane")?;
        record_session_provider(name, agent)?;
        self.invalidate_pane(name).await;
        Ok(())
    }

    pub(crate) async fn create_recorded_agent_session(
        &self,
        name: &str,
        working_dir: Option<&str>,
        agent: Agent,
        launch: AgentLaunch,
    ) -> Result<Session> {
        // An agent launch owns authority and process provenance. Reusing an
        // existing same-name pane would return success without starting this
        // command, then overwrite that live pane's provider metadata. Generic
        // terminal creation retains create-or-reuse semantics; typed agent
        // launches are create-only and callers that intend replacement must
        // use the explicit relaunch API.
        self.create_recorded_agent_session_create_only(name, working_dir, agent, launch)
            .await
    }

    /// A fresh authority owner must never inherit a same-name pane. This path
    /// atomically fails on collision instead of reusing an unrelated session.
    pub(crate) async fn create_recorded_agent_session_create_only(
        &self,
        name: &str,
        working_dir: Option<&str>,
        agent: Agent,
        launch: AgentLaunch,
    ) -> Result<Session> {
        let (command, environment) = launch.into_parts();
        let session = self
            .create_session_with_environment_and_policy(
                name,
                working_dir,
                Some(&command),
                &environment,
                TYPED_AGENT_SESSION_POLICY,
            )
            .await?;
        if let Err(error) = record_session_provider(name, agent) {
            let rollback = self.kill_session(name).await.err();
            return Err(error).with_context(|| match rollback {
                Some(rollback) => format!(
                    "provider provenance failed after create-only session launch; rollback also failed: {rollback}"
                ),
                None => "provider provenance failed; create-only session was rolled back".to_string(),
            });
        }
        Ok(session)
    }

    /// Typed-agent counterpart of
    /// [`Self::create_command_session_create_only_with_authority`]. Provider
    /// credentials remain in the existing structured launch environment; the
    /// two authority values are non-secret and duplicate reserved keys are
    /// refused rather than silently shadowed.
    pub async fn create_agent_session_create_only_with_authority(
        &self,
        state_dir: &Path,
        name: &str,
        working_dir: Option<&str>,
        agent: Agent,
        launch: AgentLaunch,
        authority: &SessionDispatchAuthority,
    ) -> Result<Session> {
        authority.validate()?;
        if authority.session != name {
            anyhow::bail!(
                "session authority identity {} differs from launch target {name}",
                authority.session
            );
        }
        let (command, mut environment) = launch.into_parts();
        if environment
            .iter()
            .any(|(key, _)| key == DISPATCH_GENERATION_ENV || key == SCOPE_CLAIM_ID_ENV)
        {
            anyhow::bail!("agent launch attempted to override reserved dispatch authority");
        }
        environment.extend(authority.environment());

        let _lock = session_authority_lock(state_dir, name)?;
        let session = self
            .create_session_with_environment_and_policy(
                name,
                working_dir,
                Some(&command),
                &environment,
                TYPED_AGENT_SESSION_POLICY,
            )
            .await?;
        if let Err(error) = record_session_provider(name, agent) {
            let rollback = session.kill().await.err();
            return Err(error).with_context(|| match rollback {
                Some(rollback) => format!(
                    "provider provenance failed after generated launch; rollback also failed: {rollback}"
                ),
                None => "provider provenance failed; generated launch was rolled back".to_string(),
            });
        }
        if let Err(error) = authority.write_locked(state_dir) {
            let rollback = session.kill().await.err();
            let provider_cleanup =
                crate::scope::remove_private_file(&session_provider_path(name)).err();
            return Err(error).with_context(|| {
                let mut detail = "dispatch authority publication failed".to_string();
                if let Some(rollback) = rollback {
                    detail.push_str(&format!("; session rollback also failed: {rollback}"));
                }
                if let Some(provider_cleanup) = provider_cleanup {
                    detail.push_str(&format!(
                        "; provider metadata rollback also failed: {provider_cleanup}"
                    ));
                }
                detail
            });
        }
        Ok(session)
    }

    /// Kill only the session generation named by `authority`. The per-session
    /// lock serializes OmegaOS create and cleanup paths, so a late A cleanup
    /// cannot validate against or kill a replacement B generation.
    pub async fn kill_session_exact(
        &self,
        state_dir: &Path,
        authority: &SessionDispatchAuthority,
    ) -> Result<()> {
        authority.validate()?;
        let _lock = session_authority_lock(state_dir, &authority.session)?;
        authority.validate_current_locked(state_dir)?;
        let session = self.get_session(&authority.session).await?;
        session.kill().await?;
        crate::scope::remove_private_file(&session_authority_path(state_dir, &authority.session)?)?;
        crate::scope::remove_private_file(&session_provider_path(&authority.session))?;
        self.invalidate_pane(&authority.session).await;
        Ok(())
    }

    pub async fn list_sessions(&self) -> Result<Vec<OmegaSession>> {
        let session_names = self.rmux.list_sessions().await?;
        let mut sessions: Vec<OmegaSession> = session_names
            .iter()
            .map(|name| {
                let mut session = OmegaSession::classify(name.as_ref());
                session.provider = read_session_provider(name.as_ref());
                session
            })
            .collect();

        sessions.sort_by(order_sessions);
        Ok(sessions)
    }

    pub async fn get_session(&self, name: &str) -> Result<Session> {
        // Resolve with the same slug used at creation so a caller that kept the
        // original (possibly dirty) name still finds the session. Idempotent on
        // clean names — every menu/list path already passes the real name.
        let session_name = SessionName::new(sanitize_session_name(name))?;
        self.rmux
            .session(session_name)
            .await
            .context("Session not found")
    }

    pub async fn kill_session(&self, name: &str) -> Result<()> {
        let session = self.get_session(name).await?;
        session.kill().await?;
        let _ = std::fs::remove_file(session_provider_path(name));
        // The pane handle we held is now dangling — drop it so any future
        // send_text/send_key for the same name re-resolves cleanly.
        self.invalidate_pane(name).await;
        Ok(())
    }

    /// Cached lookup of the active pane for `session_name`.
    ///
    /// On a cache hit this is a single mutex acquisition (microseconds).
    /// On a miss it issues one `rmux.session(name)` RPC, stores the result,
    /// and returns the new Pane. The Pane handle itself is cheap to clone
    /// (Arc-wrapped endpoint), so we hand back a clone and keep one in the
    /// cache for the next caller.
    ///
    /// Use this in any hot path where the same session is touched repeatedly
    /// (TUI keystroke forwarding, preview capture). For one-shot operations
    /// `get_active_pane` is still fine but offers no win.
    pub async fn pane_for(&self, session_name: &str) -> Result<Pane> {
        {
            let cache = self.pane_cache.lock().await;
            if let Some(pane) = cache.get(session_name) {
                return Ok(pane.clone());
            }
        }
        // Miss — resolve and cache. We deliberately drop the lock across
        // the get_session await to avoid serialising callers; under
        // contention the worst case is N callers each doing one RPC, the
        // first to re-lock winning the cache slot (or_insert_with), and
        // every caller returning its own equivalent pane for the same name.
        let session = self.get_session(session_name).await?;
        let pane = session.pane(0, 0);
        let mut cache = self.pane_cache.lock().await;
        cache
            .entry(session_name.to_string())
            .or_insert_with(|| pane.clone());
        Ok(pane)
    }

    /// Drop the cached Pane for a session — call after kill/recreate or
    /// after a send error that suggests the daemon-side pane is gone.
    pub async fn invalidate_pane(&self, session_name: &str) {
        self.pane_cache.lock().await.remove(session_name);
    }

    /// Rename a session via the rmux CLI (the SDK doesn't expose rename yet).
    /// Equivalent to: rmux rename-session -t <old> <safe-new>
    ///
    /// The new name goes through the SAME sanitize chokepoint as
    /// create_session: every later lookup (get_session, capture, pane cache)
    /// sanitizes the name it is given, so letting a raw name (spaces,
    /// accents) reach rmux here made the renamed session unaddressable by the
    /// whole TUI — listed, but no preview and no way to enter it. The old
    /// name is passed through verbatim: it must match the daemon's exact
    /// current key, even if that key is a dirty name from a pre-fix rename
    /// (which is also what lets a rename repair such a session).
    ///
    /// Returns the sanitized name actually applied — callers must use it for
    /// selection/status, not the raw input.
    pub async fn rename_session(&self, old_name: &str, new_name: &str) -> Result<String> {
        let safe = sanitize_session_name(new_name);
        let _ = SessionName::new(&safe).context("invalid new session name")?;
        let status = tokio::process::Command::new("rmux")
            .args(["rename-session", "-t", old_name, &safe])
            .status()
            .await
            .context("spawning rmux rename-session")?;
        if !status.success() {
            anyhow::bail!("rmux rename-session failed (exit {:?})", status.code());
        }
        crate::tuilog::log(format!(
            "rename: '{old_name}' → '{safe}' (raw input: '{new_name}')"
        ));
        // Old name is no longer addressable — drop its cached pane. Also drop
        // any entry under new_name: a stale cache slot left from a prior session
        // of the same name would otherwise point at a now-different daemon pane.
        self.invalidate_pane(old_name).await;
        self.invalidate_pane(&safe).await;
        Ok(safe)
    }

    pub async fn get_active_pane(&self, name: &str) -> Result<Pane> {
        let session = self.get_session(name).await?;
        Ok(session.pane(0, 0))
    }

    pub async fn send_text(&self, session_name: &str, text: &str) -> Result<()> {
        // Type, then SUBMIT. Codex/Claude/Hermes composers often absorb a
        // lone Enter as a newline; C-m is the CR the TUI treats as send.
        // Cached pane + one stale-cache retry so kill+recreate self-heals.
        let pane = self.pane_for(session_name).await?;
        match pane.send_text(text).await {
            Ok(()) => {
                submit_composer(&pane).await?;
                Ok(())
            }
            Err(e) if is_pane_stale(&e) => {
                self.invalidate_pane(session_name).await;
                let pane = self.pane_for(session_name).await?;
                pane.send_text(text).await?;
                submit_composer(&pane).await?;
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Raw text send — no auto-Enter. Used by the TUI interactive preview
    /// to forward single chars without injecting a newline the user did not type.
    ///
    /// HOT PATH: invoked per keystroke when the right panel is chat-focused.
    /// Uses the cached pane lookup so the only daemon RPC is the actual
    /// send_text; on a stale-pane error we invalidate + retry once.
    pub async fn send_text_raw(&self, session_name: &str, text: &str) -> Result<()> {
        let pane = self.pane_for(session_name).await?;
        match pane.send_text(text).await {
            Ok(()) => Ok(()),
            Err(e) if is_pane_stale(&e) => {
                self.invalidate_pane(session_name).await;
                let pane = self.pane_for(session_name).await?;
                pane.send_text(text).await?;
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Send a named key event (e.g. "Enter", "BackSpace", "Up", "Escape").
    /// Mirrors the rmux key naming.
    ///
    /// HOT PATH: invoked per arrow/enter/space in chat-focused mode. Same
    /// cached-pane + single-retry strategy as `send_text_raw`.
    pub async fn send_key(&self, session_name: &str, key: &str) -> Result<()> {
        let pane = self.pane_for(session_name).await?;
        match pane.send_key(key).await {
            Ok(()) => Ok(()),
            Err(e) if is_pane_stale(&e) => {
                self.invalidate_pane(session_name).await;
                let pane = self.pane_for(session_name).await?;
                pane.send_key(key).await?;
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Resize a session's active pane to (cols, rows). The TUI calls this
    /// to match the rmux pane to the preview panel's real width — without
    /// it, Claude renders to its spawn-time 200-col width and the preview
    /// shows a clipped slice, making the agent UI look "phone-width".
    pub async fn resize_pane(&self, session_name: &str, cols: u16, rows: u16) -> Result<()> {
        // IMPORTANT (verified empirically): a pane is bounded by its WINDOW.
        // The SDK `pane.resize()` is CLAMPED to the current window size, so it
        // cannot grow the content past the window — calling it to widen a
        // session is a silent no-op. That was the real "Claude isn't
        // responsive to the panel width" bug: every resize call did nothing.
        // Growing the content requires resizing the WINDOW. The SDK exposes no
        // public window-resize, so we shell out to the proven
        // `rmux resize-window`, which sets the absolute size and makes the
        // inner app redraw via SIGWINCH (confirmed live: 200→120→150→100).
        //
        // Cheap in practice: callers guard this behind (session, cols, rows)
        // change-detection, so it fires only when the geometry actually
        // changes, not every frame.
        let out = tokio::process::Command::new("rmux")
            .args([
                "resize-window",
                "-t",
                session_name,
                "-x",
                &cols.to_string(),
                "-y",
                &rows.to_string(),
            ])
            .output()
            .await?;
        if !out.status.success() {
            anyhow::bail!(
                "rmux resize-window failed: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
        Ok(())
    }

    /// Multi-line aware send: wraps `text` in bracketed-paste escape
    /// sequences (`\e[200~ ... \e[201~`) so that an interactive TUI
    /// (e.g. Claude Code) buffers the whole block as one paste rather
    /// than submitting on every embedded newline. After the closing
    /// marker, sends Enter to submit the buffered input.
    ///
    /// Use this whenever the prompt may contain `\n` AND you want a
    /// single coherent user turn (e.g., reply-context + new message).
    ///
    /// SAME PROTECTION AS [`Self::send_paste_raw`], and for the same reason.
    /// This used to be the bare, unretried variant while its twin thirty lines
    /// below chunked the body and replayed the whole block atomically on a
    /// stale pane. Its one caller is the oracle followup, whose body carries
    /// "## Attached files" and runs to several KB — the exact payload the
    /// chunking exists for — and a failure landing BETWEEN `\e[200~` and
    /// `\e[201~` leaves a live oracle's composer stuck in paste mode, eating
    /// the operator's next keystrokes.
    ///
    /// The Enter is deliberately NOT inside the retried block: replaying a
    /// submit could duplicate a turn, whereas replaying the paste alone can
    /// only ever re-fill a composer that never received it.
    pub async fn send_paste_then_submit(&self, session_name: &str, text: &str) -> Result<()> {
        self.send_paste_block(session_name, text).await?;
        let pane = self.pane_for(session_name).await?;
        match submit_composer(&pane).await {
            Ok(()) => Ok(()),
            Err(e) if is_pane_stale(&e) => {
                self.invalidate_pane(session_name).await;
                let pane = self.pane_for(session_name).await?;
                submit_composer(&pane).await?;
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }

    /// One bracketed-paste block (markers + chunked body) sent as a single
    /// unit, replayed in full if the pane went stale mid-flight — never a
    /// half-sent block. Shared by the submit and no-submit variants.
    async fn send_paste_block(&self, session_name: &str, text: &str) -> Result<()> {
        async fn paste_block(
            pane: &Pane,
            text: &str,
        ) -> std::result::Result<(), rmux_sdk::RmuxError> {
            pane.send_text("\u{1b}[200~").await?;
            // Chunk the body so a very large paste isn't sent as one oversized
            // PTY write. Markers are sent once; only the body is split, so the
            // block stays atomic from the target app's perspective.
            const CHUNK: usize = 4096;
            if text.len() <= CHUNK {
                pane.send_text(text).await?;
            } else {
                let mut start = 0;
                let bytes = text.as_bytes();
                while start < bytes.len() {
                    // Advance to a char boundary at/under the chunk limit.
                    let mut end = (start + CHUNK).min(bytes.len());
                    while end < bytes.len() && !text.is_char_boundary(end) {
                        end -= 1;
                    }
                    pane.send_text(&text[start..end]).await?;
                    start = end;
                }
            }
            pane.send_text("\u{1b}[201~").await?;
            Ok(())
        }

        let pane = self.pane_for(session_name).await?;
        match paste_block(&pane, text).await {
            Ok(()) => Ok(()),
            Err(e) if is_pane_stale(&e) => {
                self.invalidate_pane(session_name).await;
                let pane = self.pane_for(session_name).await?;
                paste_block(&pane, text).await?;
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }

    /// HOT PATH (~12 FPS during chat focus): the right-panel preview tick
    /// uses this on every refresh. Cached pane → one daemon RPC per tick
    /// (the snapshot itself), not three.
    pub async fn capture_pane(&self, session_name: &str) -> Result<String> {
        let pane = self.pane_for(session_name).await?;
        let visible = match pane.snapshot().await {
            Ok(snapshot) => snapshot.visible_text(),
            Err(e) if is_pane_stale(&e) => {
                self.invalidate_pane(session_name).await;
                let pane = self.pane_for(session_name).await?;
                pane.snapshot().await?.visible_text()
            }
            Err(e) => return Err(e.into()),
        };
        if !visible.trim().is_empty() {
            return Ok(visible);
        }
        // Alt-screen / splash frames can snapshot empty while the agent is
        // live. Scrollback still has the TUI — prefer that over a blank
        // `omega capture` while Codex/Claude/Hermes are running.
        match self.capture_pane_history(session_name, 200).await {
            Ok(history) if !history.trim().is_empty() => Ok(history),
            _ => Ok(visible),
        }
    }

    /// Capture pane text AND the REAL cursor position from the pane
    /// snapshot. The TUI uses (row, col) to paint the caret exactly where
    /// the agent's input cursor is — not a guessed "last non-empty line".
    /// Returns (visible_text, cursor_row, cursor_col, cursor_visible).
    pub async fn capture_pane_with_cursor(
        &self,
        session_name: &str,
    ) -> Result<(String, u16, u16, bool)> {
        let pane = self.pane_for(session_name).await?;
        let snapshot = match pane.snapshot().await {
            Ok(s) => s,
            Err(e) if is_pane_stale(&e) => {
                self.invalidate_pane(session_name).await;
                let pane = self.pane_for(session_name).await?;
                pane.snapshot().await?
            }
            Err(e) => return Err(e.into()),
        };
        let c = snapshot.cursor;
        Ok((snapshot.visible_text(), c.row, c.col, c.visible))
    }

    /// Capture the pane WITH per-cell styling (fg/bg/bold/reverse), so the
    /// TUI preview can render Claude's colored UI — most importantly the
    /// `/` command-menu selection highlight, which `visible_text()` drops
    /// entirely (the user "can't see what they're selecting"). Also makes
    /// diffs, syntax highlighting, and status colors visible.
    ///
    /// Capture the visible pane as styled rows, GATED on the pane's revision.
    ///
    /// `since_revision` = the revision the caller last rendered. The pane's
    /// `revision` (in every snapshot) bumps on any observable change — output,
    /// resize, clear, cursor move. If it hasn't moved, we return
    /// [`StyledCapture::Unchanged`] WITHOUT running `styled_rows_from_snapshot`
    /// over ~10k cells or flattening to text — the dominant per-frame cost.
    /// During the (frequent) "agent is thinking" pauses the preview redraws for
    /// free off the cached rows. Pass `0` to force a fresh capture (e.g. on a
    /// session switch). Revision `0` (stale/empty snapshot) never gates.
    pub async fn capture_pane_styled(
        &self,
        session_name: &str,
        since_revision: u64,
    ) -> Result<StyledCapture> {
        let pane = self.pane_for(session_name).await?;
        let snapshot = match pane.snapshot().await {
            Ok(s) => s,
            Err(e) if is_pane_stale(&e) => {
                self.invalidate_pane(session_name).await;
                let pane = self.pane_for(session_name).await?;
                pane.snapshot().await?
            }
            Err(e) => return Err(e.into()),
        };
        if snapshot.revision != 0 && snapshot.revision == since_revision {
            return Ok(StyledCapture::Unchanged);
        }
        let c = snapshot.cursor;
        let rows = styled_rows_from_snapshot(&snapshot);
        Ok(StyledCapture::Changed {
            rows,
            cursor_row: c.row,
            cursor_col: c.col,
            cursor_visible: c.visible,
            revision: snapshot.revision,
        })
    }

    /// Capture the pane INCLUDING scrollback history (last `history_lines`
    /// lines, counted up from the live tail). `snapshot().visible_text()`
    /// only renders the current visible screen — so the TUI preview can't
    /// scroll up into history with it. This shells out to the rmux CLI which
    /// supports `-S -<N>` (start N lines into the scrollback buffer).
    ///
    /// SLOW PATH: only used while the user is actively browsing history
    /// (`preview_follow_tail == false`); the live tail keeps the fast
    /// `capture_pane` snapshot path.
    /// `-e` KEEPS THE COLORS. Without it rmux flattens every attribute and the
    /// mirror turned grayscale the instant the user scrolled up — the live tail
    /// goes through `capture_pane_styled` (real cells, real colors) while this
    /// path returned plain text, so scrolling visibly drained the color out of
    /// a Claude or Codex conversation. Callers that want plain text run the
    /// result through [`styled_rows_from_ansi`], which returns both the styled
    /// rows and the stripped text.
    pub async fn capture_pane_history(
        &self,
        session_name: &str,
        history_lines: u32,
    ) -> Result<String> {
        let out = tokio::process::Command::new("rmux")
            .args([
                "capture-pane",
                "-p",
                "-e",
                "-t",
                session_name,
                "-S",
                &format!("-{}", history_lines),
            ])
            .output()
            .await?;
        if !out.status.success() {
            anyhow::bail!(
                "rmux capture-pane -S failed: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }

    /// Forward a paste as ONE bracketed-paste block (`\e[200~ … \e[201~`)
    /// with NO trailing Enter. The target app (Claude Code / a REPL) buffers
    /// the whole block as a single paste instead of treating each embedded
    /// `\n` as a submit — so a 30-line paste lands as 30 lines of one turn,
    /// not 30 separate commands. The user presses Enter themselves.
    ///
    /// Distinct from `send_paste_then_submit` (which auto-submits) — used for
    /// forwarding a *user* paste into the interactive preview.
    pub async fn send_paste_raw(&self, session_name: &str, text: &str) -> Result<()> {
        // Whole bracketed-paste block (markers + chunked body) as one unit, so
        // the stale-pane retry replays the ENTIRE paste atomically — never a
        // half-sent block. Mirrors the single-retry strategy of send_text_raw.
        self.send_paste_block(session_name, text).await
    }

    pub async fn wait_for_text(
        &self,
        session_name: &str,
        text: &str,
        timeout: Duration,
    ) -> Result<()> {
        let pane = self.get_active_pane(session_name).await?;
        pane.expect_visible_text()
            .to_contain(text)
            .timeout(timeout)
            .await?;
        Ok(())
    }

    pub async fn split_pane(&self, session_name: &str, command: Option<&str>) -> Result<Pane> {
        let pane = self.get_active_pane(session_name).await?;

        if let Some(cmd) = command {
            let new_pane = pane.split_with(SplitDirection::Right).shell(cmd).await?;
            Ok(new_pane)
        } else {
            let new_pane = pane.split(SplitDirection::Right).await?;
            Ok(new_pane)
        }
    }

    pub fn rmux(&self) -> &Rmux {
        &self.rmux
    }
}

/// A preview color, PRESERVING its original depth. Critical: converting an
/// ANSI 16-color (what Claude/most CLIs emit) into 24-bit RGB makes the TUI
/// emit truecolor `38;2;…` escapes — which a terminal reached over mosh / a
/// `TERM=xterm` session renders as DEFAULT (grey). Keeping the original index
/// lets the renderer emit the matching 16-color (`9x`) / 256 (`38;5`) escape,
/// which renders everywhere the chrome already does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewColor {
    /// ANSI/xterm palette index (0–15 = the 16 base colors, 16–255 = 256-cube).
    Indexed(u8),
    /// True 24-bit color (only when the source cell was genuinely RGB).
    Rgb(u8, u8, u8),
}

/// One styled run of text within a preview row. `None` = terminal default.
#[derive(Debug, Clone)]
pub struct PreviewSpan {
    pub text: String,
    pub fg: Option<PreviewColor>,
    pub bg: Option<PreviewColor>,
    pub bold: bool,
    /// DIM/ITALIC/UNDERLINE travel with the span: Claude Code renders its
    /// secondary text dim — dropping the bit made it full-brightness in the
    /// preview and visually merged with primary output.
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
}

/// A styled preview row = a sequence of spans.
pub type PreviewLine = Vec<PreviewSpan>;

/// Result of a revision-gated styled capture. `Unchanged` means the pane's
/// revision matched what the caller already rendered — no restyle was done and
/// the caller should keep its cached preview.
pub enum StyledCapture {
    Unchanged,
    Changed {
        rows: Vec<PreviewLine>,
        cursor_row: u16,
        cursor_col: u16,
        cursor_visible: bool,
        revision: u64,
    },
}

/// Convert a pane snapshot into styled rows, merging adjacent same-style
/// cells into spans. Honors REVERSE by swapping fg/bg (that's how the
/// `/` selector + many TUI highlights are drawn).
fn styled_rows_from_snapshot(snapshot: &rmux_sdk::PaneSnapshot) -> Vec<PreviewLine> {
    let cols = snapshot.cols;
    let rows = snapshot.rows;
    let mut out: Vec<PreviewLine> = Vec::with_capacity(rows as usize);
    for r in 0..rows {
        let mut line: PreviewLine = Vec::new();
        let mut cur_text = String::new();
        let mut cur_fg: Option<PreviewColor> = None;
        let mut cur_bg: Option<PreviewColor> = None;
        let mut cur_bold = false;
        let mut cur_dim = false;
        let mut cur_italic = false;
        let mut cur_underline = false;
        let mut started = false;
        for col in 0..cols {
            let Some(cell) = snapshot.cell(r, col) else {
                continue;
            };
            if cell.glyph.is_padding() {
                continue;
            }
            let ch = cell.glyph.text.clone();
            let attr_bits = cell.attributes.bits;
            let reverse = attr_bits & rmux_sdk::PaneAttributes::REVERSE.bits != 0;
            let bold = attr_bits & rmux_sdk::PaneAttributes::BOLD.bits != 0;
            let dim = attr_bits & rmux_sdk::PaneAttributes::DIM.bits != 0;
            let italic = attr_bits & rmux_sdk::PaneAttributes::ITALIC.bits != 0;
            let underline = attr_bits & rmux_sdk::PaneAttributes::UNDERLINE.bits != 0;
            let mut fg = pane_color_to_preview(&cell.foreground);
            let mut bg = pane_color_to_preview(&cell.background);
            if reverse {
                std::mem::swap(&mut fg, &mut bg);
                // ensure a visible swap even when one side was default
                if fg.is_none() {
                    fg = Some(PreviewColor::Indexed(0));
                }
                if bg.is_none() {
                    bg = Some(PreviewColor::Indexed(7));
                }
            }
            let glyph = if ch.is_empty() { " ".to_string() } else { ch };
            if started
                && fg == cur_fg
                && bg == cur_bg
                && bold == cur_bold
                && dim == cur_dim
                && italic == cur_italic
                && underline == cur_underline
            {
                cur_text.push_str(&glyph);
            } else {
                if started {
                    line.push(PreviewSpan {
                        text: std::mem::take(&mut cur_text),
                        fg: cur_fg,
                        bg: cur_bg,
                        bold: cur_bold,
                        dim: cur_dim,
                        italic: cur_italic,
                        underline: cur_underline,
                    });
                }
                cur_fg = fg;
                cur_bg = bg;
                cur_bold = bold;
                cur_dim = dim;
                cur_italic = italic;
                cur_underline = underline;
                cur_text = glyph;
                started = true;
            }
        }
        if started && !cur_text.is_empty() {
            line.push(PreviewSpan {
                text: cur_text,
                fg: cur_fg,
                bg: cur_bg,
                bold: cur_bold,
                dim: cur_dim,
                italic: cur_italic,
                underline: cur_underline,
            });
        }
        // Trim trailing all-blank spans to keep lines tight.
        while line
            .last()
            .is_some_and(|s| s.text.trim().is_empty() && s.bg.is_none())
        {
            line.pop();
        }
        out.push(line);
    }
    // Drop trailing blank rows.
    while out.last().is_some_and(|l| l.is_empty()) {
        out.pop();
    }
    out
}

/// Resolve a rmux PaneColor into a PreviewColor, PRESERVING its depth so the
/// renderer can emit a 16-color / 256 / truecolor escape that matches what the
/// source actually used. `None` = terminal default. Upconverting ANSI→RGB here
/// was the bug that rendered Claude's 16-color output as grey truecolor.
fn pane_color_to_preview(c: &rmux_sdk::PaneColor) -> Option<PreviewColor> {
    use rmux_sdk::PaneColor;
    match c {
        PaneColor::Default | PaneColor::None | PaneColor::Terminal => None,
        PaneColor::Rgb { red, green, blue } => Some(PreviewColor::Rgb(*red, *green, *blue)),
        PaneColor::Ansi { index } => Some(PreviewColor::Indexed(index & 0x0f)),
        PaneColor::BrightAnsi { index } => Some(PreviewColor::Indexed((index & 0x07) + 8)),
        PaneColor::Indexed { index } => Some(PreviewColor::Indexed(*index)),
        _ => None,
    }
}

/// Menu ordering for the session list. Groups by section (Home, project work,
/// System), then by project, then — within a project — oracles BEFORE their
/// workers so workers render nested under the oracle block (the ├/└ tree).
///
/// `role_order` is compared BEFORE `oracle_index` on purpose: workers carry
/// `oracle_index == None`, and `None < Some(_)`, so comparing the index first
/// would float every worker above its governing oracle (the reported bug —
/// "DentistryGPT-worker-…" listed above "oracle-DentistryGPT-1").
fn order_sessions(a: &OmegaSession, b: &OmegaSession) -> std::cmp::Ordering {
    section_order(&a.role)
        .cmp(&section_order(&b.role))
        .then_with(|| a.project.cmp(&b.project))
        .then_with(|| role_order(&a.role).cmp(&role_order(&b.role)))
        .then_with(|| a.oracle_index.cmp(&b.oracle_index))
        .then_with(|| a.name.cmp(&b.name))
}

fn section_order(role: &SessionRole) -> u8 {
    match role {
        SessionRole::Home => 0,
        SessionRole::Oracle | SessionRole::Worker => 1,
        SessionRole::System => 2,
    }
}

fn role_order(role: &SessionRole) -> u8 {
    match role {
        SessionRole::Oracle => 0,
        SessionRole::Worker => 1,
        SessionRole::Home => 2,
        SessionRole::System => 3,
    }
}

/// Cached `Pane` handles can outlive the underlying daemon-side pane when a
/// session is killed-and-recreated under the same name (rename-then-recreate,
/// crash-restart, etc.). The hot-path send/capture methods rely on this
/// predicate to recognise such errors, drop the stale handle, and retry once.
fn is_pane_stale(err: &rmux_sdk::RmuxError) -> bool {
    matches!(
        err,
        rmux_sdk::RmuxError::PaneNotFound { .. }
            | rmux_sdk::RmuxError::OwnedSessionLeaseLost { .. }
    )
}

/// Submit a typed composer: Enter, then C-m. Codex/Claude/Hermes often treat
/// a lone Enter as a newline; the CR is what actually sends.
async fn submit_composer(pane: &Pane) -> std::result::Result<(), rmux_sdk::RmuxError> {
    pane.send_key("Enter").await?;
    pane.send_key("C-m").await?;
    Ok(())
}

/// Parse `capture-pane -e` output into styled rows PLUS the stripped plain
/// text, in one pass.
///
/// The live preview builds its spans from real snapshot cells; the history
/// preview only has the escaped text rmux prints, so it needs the attributes
/// read back OUT of the stream. Both feed the same [`PreviewLine`] renderer, so
/// scrolling up no longer drops to grayscale.
///
/// Deliberately narrow: it understands SGR (`CSI … m`) and SKIPS any other CSI
/// (rmux only emits SGR here, but a stray sequence must not leak into the text)
/// plus OSC strings up to their terminator. Unknown SGR parameters are ignored
/// rather than guessed. The plain text is returned alongside because the
/// selection/copy path must never see escape bytes.
pub fn styled_rows_from_ansi(input: &str) -> (Vec<PreviewLine>, String) {
    let mut rows: Vec<PreviewLine> = Vec::new();
    let mut plain = String::new();

    for raw_line in input.split('\n') {
        let mut line: PreviewLine = Vec::new();
        let mut cur = String::new();
        let (mut fg, mut bg) = (None, None);
        let (mut bold, mut dim, mut italic, mut underline) = (false, false, false, false);

        let flush = |cur: &mut String,
                     line: &mut PreviewLine,
                     fg: Option<PreviewColor>,
                     bg: Option<PreviewColor>,
                     bold: bool,
                     dim: bool,
                     italic: bool,
                     underline: bool| {
            if !cur.is_empty() {
                line.push(PreviewSpan {
                    text: std::mem::take(cur),
                    fg,
                    bg,
                    bold,
                    dim,
                    italic,
                    underline,
                });
            }
        };

        let mut chars = raw_line.chars().peekable();
        while let Some(c) = chars.next() {
            if c != '\u{1b}' {
                cur.push(c);
                plain.push(c);
                continue;
            }
            match chars.peek() {
                Some('[') => {
                    chars.next();
                    let mut params = String::new();
                    let mut final_byte = None;
                    for c in chars.by_ref() {
                        if c.is_ascii_alphabetic() || c == '~' {
                            final_byte = Some(c);
                            break;
                        }
                        params.push(c);
                    }
                    if final_byte != Some('m') {
                        continue; // non-SGR CSI: consumed, never rendered
                    }
                    flush(&mut cur, &mut line, fg, bg, bold, dim, italic, underline);
                    let nums: Vec<i32> = params
                        .split(';')
                        .map(|p| p.trim().parse::<i32>().unwrap_or(0))
                        .collect();
                    let mut i = 0;
                    while i < nums.len() {
                        match nums[i] {
                            0 => {
                                fg = None;
                                bg = None;
                                bold = false;
                                dim = false;
                                italic = false;
                                underline = false;
                            }
                            1 => bold = true,
                            2 => dim = true,
                            3 => italic = true,
                            4 => underline = true,
                            22 => {
                                bold = false;
                                dim = false;
                            }
                            23 => italic = false,
                            24 => underline = false,
                            30..=37 => fg = Some(PreviewColor::Indexed((nums[i] - 30) as u8)),
                            39 => fg = None,
                            40..=47 => bg = Some(PreviewColor::Indexed((nums[i] - 40) as u8)),
                            49 => bg = None,
                            90..=97 => fg = Some(PreviewColor::Indexed((nums[i] - 90 + 8) as u8)),
                            100..=107 => {
                                bg = Some(PreviewColor::Indexed((nums[i] - 100 + 8) as u8))
                            }
                            38 | 48 => {
                                let target_is_fg = nums[i] == 38;
                                let color = match nums.get(i + 1) {
                                    Some(5) => {
                                        let c = nums.get(i + 2).copied().unwrap_or(0);
                                        i += 2;
                                        Some(PreviewColor::Indexed(c.clamp(0, 255) as u8))
                                    }
                                    Some(2) => {
                                        let r = nums.get(i + 2).copied().unwrap_or(0);
                                        let g = nums.get(i + 3).copied().unwrap_or(0);
                                        let b = nums.get(i + 4).copied().unwrap_or(0);
                                        i += 4;
                                        Some(PreviewColor::Rgb(
                                            r.clamp(0, 255) as u8,
                                            g.clamp(0, 255) as u8,
                                            b.clamp(0, 255) as u8,
                                        ))
                                    }
                                    _ => None,
                                };
                                if target_is_fg {
                                    fg = color;
                                } else {
                                    bg = color;
                                }
                            }
                            _ => {}
                        }
                        i += 1;
                    }
                }
                Some(']') => {
                    // OSC: swallow to BEL or ST.
                    chars.next();
                    while let Some(c) = chars.next() {
                        if c == '\u{7}' {
                            break;
                        }
                        if c == '\u{1b}' {
                            if chars.peek() == Some(&'\\') {
                                chars.next();
                            }
                            break;
                        }
                    }
                }
                _ => {
                    chars.next();
                }
            }
        }
        flush(&mut cur, &mut line, fg, bg, bold, dim, italic, underline);
        rows.push(line);
        plain.push('\n');
    }
    if plain.ends_with('\n') {
        plain.pop();
    }
    (rows, plain)
}

#[cfg(test)]
mod ansi_parse_tests {
    use super::{styled_rows_from_ansi, PreviewColor};

    #[test]
    fn strips_escapes_from_plain_text_and_keeps_colors() {
        let (rows, plain) =
            styled_rows_from_ansi("\u{1b}[38;5;196mROUGE\u{1b}[39m ok\n\u{1b}[1;34mBLEU\u{1b}[0m");
        assert_eq!(plain, "ROUGE ok\nBLEU");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0][0].text, "ROUGE");
        assert!(matches!(rows[0][0].fg, Some(PreviewColor::Indexed(196))));
        assert_eq!(rows[0][1].text, " ok");
        assert!(rows[0][1].fg.is_none());
        assert_eq!(rows[1][0].text, "BLEU");
        assert!(rows[1][0].bold);
        assert!(matches!(rows[1][0].fg, Some(PreviewColor::Indexed(4))));
    }

    #[test]
    fn parses_truecolor_and_drops_non_sgr_sequences() {
        let (rows, plain) = styled_rows_from_ansi("\u{1b}[2K\u{1b}[38;2;10;20;30mRGB\u{1b}[m tail");
        assert_eq!(plain, "RGB tail");
        assert!(matches!(rows[0][0].fg, Some(PreviewColor::Rgb(10, 20, 30))));
        assert!(rows[0][1].fg.is_none());
    }
}

#[cfg(test)]
mod sanitize_tests {
    use super::{resolve_agent_command, sanitize_session_name as s};
    use super::{EnsureSessionPolicy, MAX_SESSION_NAME_LEN, TYPED_AGENT_SESSION_POLICY};
    use crate::agents::Agent;
    use std::path::PathBuf;

    #[test]
    fn expand_user_path_resolves_tilde_home() {
        let home = dirs::home_dir().expect("home");
        assert_eq!(crate::session::expand_user_path("~"), home);
        assert_eq!(
            crate::session::expand_user_path("~/Desktop"),
            home.join("Desktop")
        );
        assert_eq!(
            crate::session::expand_user_path("/abs/project"),
            PathBuf::from("/abs/project")
        );
        assert_ne!(
            crate::session::expand_user_path("~/Desktop").as_os_str(),
            std::ffi::OsStr::new("~/Desktop"),
            "omega new --dir ~/Desktop must not chdir into a literal tilde path"
        );
    }

    #[test]
    fn resolve_session_working_dir_matches_tui_when_omitted() {
        assert!(crate::session::resolve_session_working_dir(None)
            .unwrap()
            .is_none());
        let tmp = tempfile::TempDir::new().unwrap();
        let got = crate::session::resolve_session_working_dir(tmp.path().to_str()).unwrap();
        assert_eq!(got.as_deref(), Some(tmp.path()));
        let missing = tmp.path().join("not-a-dir");
        let err = crate::session::resolve_session_working_dir(missing.to_str())
            .expect_err("missing --dir must fail before spawn");
        assert!(
            err.to_string().contains("does not exist"),
            "missing dir error: {err}"
        );
        let home = dirs::home_dir().expect("home");
        assert_eq!(
            crate::session::resolve_session_working_dir(Some("~")).unwrap(),
            Some(home),
            "`--dir ~` must expand to $HOME, not a literal tilde"
        );
    }

    #[test]
    fn clean_names_unchanged() {
        // Idempotency: real session names must pass through untouched.
        for n in [
            "oracle-Kommu",
            "oracle-OmegaOS",
            "aisb-master",
            "omega-telegram-bridge",
            "oracle-DentistryGPT-1",
            "worker-Causio-3",
            "oracle-Agentik-Academy-2",
        ] {
            assert_eq!(s(n), n, "clean name should be unchanged: {n}");
        }
    }

    #[test]
    fn the_three_zombies_become_killable_slugs() {
        // The exact names that became un-killable rmux zombies. After
        // sanitization they are plain ASCII slugs rmux can key on.
        let cases = [
            " stockées les photos",
            "oracle- on met un dossier, on voit que l'analyse va analyser toute la database\u{202f}:",
            "oracle-DentistryGPT-1 la page très lente, est-ce que tu pourrais améliorer\u{202f}?",
        ];
        for raw in cases {
            let out = s(raw);
            assert!(!out.is_empty());
            assert!(out.len() <= MAX_SESSION_NAME_LEN, "too long: {out}");
            assert!(
                !out.starts_with('-') && !out.ends_with('-'),
                "edge dash: {out}"
            );
            assert!(
                out.chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-')),
                "non-slug char survived in {out:?}"
            );
        }
    }

    #[test]
    fn folds_accents_to_ascii() {
        // French session names keep their letters instead of mid-word dashes
        // ("Camélia" → "Camelia", not "Cam-lia").
        assert_eq!(s("Camélia"), "Camelia");
        assert_eq!(s("Éléonore à l'écran"), "Eleonore-a-l-ecran");
        assert_eq!(s("Action Moonbase Capital"), "Action-Moonbase-Capital");
    }

    #[test]
    fn collapses_and_bounds() {
        assert_eq!(s("a   b"), "a-b"); // whitespace run → single dash
        assert_eq!(s("a:::b"), "a-b"); // punctuation run → single dash
        assert_eq!(s("--x--"), "x"); // trim edge dashes
        assert_eq!(s("é"), "e"); // accents fold to ASCII (no longer stripped)
        assert_eq!(s("???"), "session"); // empty-after-strip → fallback
        assert_eq!(s(""), "session");
        assert!(s(&"x".repeat(200)).len() <= MAX_SESSION_NAME_LEN);
    }

    #[test]
    fn unknown_agent_command_is_rejected_without_codex_fallback() {
        assert_eq!(resolve_agent_command("codex").unwrap(), Agent::Codex);
        let error = resolve_agent_command("codxe").unwrap_err();
        assert!(error.to_string().contains("unknown agent provider"));
    }

    #[test]
    fn typed_agent_launch_policy_is_create_only() {
        assert_eq!(TYPED_AGENT_SESSION_POLICY, EnsureSessionPolicy::CreateOnly);
    }
}

#[cfg(test)]
mod dispatch_authority_tests {
    use super::*;

    #[test]
    fn stale_generation_cannot_validate_or_remove_replacement() {
        let state = tempfile::TempDir::new().unwrap();
        let claim_id = "a".repeat(64);
        let first = SessionDispatchAuthority::generate("worker-exact", Some(&claim_id)).unwrap();
        let second = SessionDispatchAuthority::generate("worker-exact", Some(&claim_id)).unwrap();
        assert_ne!(first.dispatch_generation, second.dispatch_generation);

        {
            let _lock = session_authority_lock(state.path(), "worker-exact").unwrap();
            first.write_locked(state.path()).unwrap();
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let path = session_authority_path(state.path(), "worker-exact").unwrap();
            assert_eq!(
                std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        first.validate_current(state.path()).unwrap();

        {
            let _lock = session_authority_lock(state.path(), "worker-exact").unwrap();
            second.write_locked(state.path()).unwrap();
        }
        assert!(first.validate_current(state.path()).is_err());
        assert!(first.remove_exact(state.path()).is_err());
        assert_eq!(
            SessionDispatchAuthority::read_strict(state.path(), "worker-exact")
                .unwrap()
                .unwrap(),
            second
        );
        second.remove_exact(state.path()).unwrap();
        assert!(
            SessionDispatchAuthority::read_strict(state.path(), "worker-exact")
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn authority_environment_contains_only_generation_receipts() {
        let claim_id = "b".repeat(64);
        let authority = SessionDispatchAuthority::generate("worker-env", Some(&claim_id)).unwrap();
        let environment = authority.environment();
        assert_eq!(environment.len(), 2);
        assert_eq!(environment[0].0, DISPATCH_GENERATION_ENV);
        assert_eq!(environment[0].1, authority.dispatch_generation);
        assert_eq!(environment[1].0, SCOPE_CLAIM_ID_ENV);
        assert_eq!(environment[1].1, claim_id);
        assert!(!serde_json::to_string(&authority)
            .unwrap()
            .to_ascii_lowercase()
            .contains("token"));
    }
}

#[cfg(test)]
mod order_tests {
    use super::{order_sessions, OmegaSession};

    #[test]
    fn workers_nest_under_their_project_oracles() {
        // The exact reported case: 2 oracles + 2 workers in DentistryGPT,
        // delivered alphabetically (workers' "D…" sorts before "oracle-…").
        // order_sessions must put oracles first (by index), workers after, so
        // the menu tree renders workers nested under the oracle block.
        let mut v: Vec<OmegaSession> = [
            "DentistryGPT-worker-composio-v3",
            "DentistryGPT-worker-agent-actions",
            "oracle-DentistryGPT-2",
            "oracle-DentistryGPT-1",
        ]
        .iter()
        .map(|n| OmegaSession::classify(n))
        .collect();
        v.sort_by(order_sessions);
        let names: Vec<&str> = v.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "oracle-DentistryGPT-1",             // oracle idx 1 first
                "oracle-DentistryGPT-2",             // then idx 2
                "DentistryGPT-worker-agent-actions", // workers after, by name
                "DentistryGPT-worker-composio-v3",
            ]
        );
    }

    #[test]
    fn projects_stay_grouped_and_oracles_lead_each() {
        // Two projects interleaved on input must come out grouped, each led by
        // its oracle, workers trailing — never a worker above its oracle.
        let mut v: Vec<OmegaSession> = [
            "Kommu-worker-build-1",
            "oracle-DentistryGPT-1",
            "oracle-Kommu-1",
            "DentistryGPT-fix-login-2",
        ]
        .iter()
        .map(|n| OmegaSession::classify(n))
        .collect();
        v.sort_by(order_sessions);
        let names: Vec<&str> = v.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "oracle-DentistryGPT-1",
                "DentistryGPT-fix-login-2",
                "oracle-Kommu-1",
                "Kommu-worker-build-1",
            ]
        );
    }
}

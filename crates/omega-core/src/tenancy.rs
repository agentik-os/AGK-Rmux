//! AGK control layer — four logical profiles, topology-agnostic isolation.
//!
//! Official Hermes profiles isolate Hermes state (`HERMES_HOME`). They are
//! **not** a filesystem sandbox (Nous docs). Mission and Private therefore
//! also hide sibling trees with bubblewrap when `bwrap` is on PATH.
//!
//! Isolation preference (operator docs):
//! 1. four Linux users (not the default installer — needs root)
//! 2. one user + per-scope sandbox (this module)
//! 3. folders/profiles only (discouraged for YOLO mission/private)

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::hermes_sync::{sync_hermes_tree, upsert_marked_file, ScopeTerminal};

pub const ACTIVE_SCOPE_FILE: &str = "active-scope.json";
pub const SCOPE_ENV_FILE: &str = "scope.env";
pub const ENV_SCOPE: &str = "OMEGA_SCOPE";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgkScope {
    Operator,
    Agentik,
    Mission,
    Private,
}

impl AgkScope {
    pub const ALL: [AgkScope; 4] = [
        AgkScope::Operator,
        AgkScope::Agentik,
        AgkScope::Mission,
        AgkScope::Private,
    ];

    pub fn id(self) -> &'static str {
        match self {
            AgkScope::Operator => "operator",
            AgkScope::Agentik => "agentik",
            AgkScope::Mission => "mission",
            AgkScope::Private => "private",
        }
    }

    pub fn parse(raw: &str) -> Result<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "operator" | "op" | "control" => Ok(AgkScope::Operator),
            "agentik" | "agk" | "product" => Ok(AgkScope::Agentik),
            "mission" | "client" | "clients" => Ok(AgkScope::Mission),
            "private" | "perso" | "personal" => Ok(AgkScope::Private),
            other => bail!(
                "unknown AGK scope '{other}' — expected operator, agentik, mission, or private"
            ),
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            AgkScope::Operator => "OPERATOR",
            AgkScope::Agentik => "AGENTIK",
            AgkScope::Mission => "MISSION",
            AgkScope::Private => "PRIVATE",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            AgkScope::Operator => "Operate the machine",
            AgkScope::Agentik => "Operate the organization",
            AgkScope::Mission => "Operate other organizations (clients)",
            AgkScope::Private => "Operate myself",
        }
    }

    /// Mission and Private must not see sibling workspaces or secrets.
    pub fn requires_runtime_isolation(self) -> bool {
        matches!(self, AgkScope::Mission | AgkScope::Private)
    }

    /// Official Hermes `terminal.home_mode: profile` — HOME becomes
    /// `{HERMES_HOME}/home` so host CLIs do not share `~/.ssh` / tokens.
    pub fn hermes_home_mode(self) -> Option<&'static str> {
        self.requires_runtime_isolation().then_some("profile")
    }
}

impl std::fmt::Display for AgkScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.id())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopePaths {
    pub scope: AgkScope,
    pub workspace: PathBuf,
    pub hermes_home: PathBuf,
    pub scope_root: PathBuf,
    pub secrets: PathBuf,
    pub secrets_link: PathBuf,
}

impl ScopePaths {
    pub fn for_home(home: &Path, scope: AgkScope) -> Self {
        let bind = crate::topology::load_resolved(home).binding(scope).clone();
        Self::from_binding(&bind)
    }

    pub fn from_binding(bind: &crate::topology::ProfileBinding) -> Self {
        let id = bind.profile.id();
        Self {
            scope: bind.profile,
            workspace: bind.workspace_root.clone(),
            hermes_home: bind.hermes_home.clone(),
            secrets: bind.secrets_root.clone(),
            secrets_link: bind.home.join(".secrets").join(id),
            scope_root: bind.home.join(".omega").join("scopes").join(id),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ActiveScopeFile {
    scope: AgkScope,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvisionReport {
    pub workspaces: Vec<PathBuf>,
    pub profiles: Vec<PathBuf>,
    pub hermes_created: Vec<String>,
    pub skipped: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxKind {
    Bubblewrap,
    None,
}

pub fn paths_for(home: &Path, scope: AgkScope) -> ScopePaths {
    ScopePaths::from_binding(crate::topology::load_resolved(home).binding(scope))
}

/// Canonical subdirectories under `$HOME/workspace/<scope>/`.
pub fn canonical_dirs(scope: AgkScope) -> &'static [&'static str] {
    match scope {
        AgkScope::Operator => &[
            "infrastructure",
            "security",
            "deployments",
            "monitoring",
            "automation",
            "deposit",
            "docs",
        ],
        AgkScope::Agentik => &[
            "projects",
            "products",
            "missions",
            "research",
            "content",
            "growth",
            "community",
            "knowledge",
            "artifacts",
        ],
        AgkScope::Mission => &["clients", "internal", "shared"],
        AgkScope::Private => &[
            "projects",
            "journal",
            "goals",
            "learning",
            "research",
            "knowledge",
            "artifacts",
        ],
    }
}

fn omega_dir(home: &Path) -> PathBuf {
    home.join(".omega")
}

/// Scope for this process. Env only — persisted `active-scope.json` is
/// applied by `agk` / `scope.env`, never by library tests or bare `omega`
/// invoked without that env (keeps Hermes Home launch tests stable).
pub fn launch_scope() -> Option<AgkScope> {
    if cfg!(test) {
        return None;
    }
    std::env::var(ENV_SCOPE)
        .ok()
        .and_then(|raw| AgkScope::parse(&raw).ok())
}

pub fn persisted_scope(home: &Path) -> AgkScope {
    if let Some(scope) = launch_scope() {
        return scope;
    }
    read_active_scope(&omega_dir(home)).unwrap_or(AgkScope::Operator)
}

pub fn read_active_scope(omega: &Path) -> Option<AgkScope> {
    let raw = fs::read_to_string(omega.join("state").join(ACTIVE_SCOPE_FILE)).ok()?;
    serde_json::from_str::<ActiveScopeFile>(&raw)
        .ok()
        .map(|f| f.scope)
}

pub fn write_active_scope(omega: &Path, scope: AgkScope) -> Result<()> {
    let state = omega.join("state");
    fs::create_dir_all(&state)?;
    let body = serde_json::to_string_pretty(&ActiveScopeFile { scope })?;
    fs::write(state.join(ACTIVE_SCOPE_FILE), body)?;
    Ok(())
}

pub fn write_scope_env(home: &Path, omega: &Path, scope: AgkScope) -> Result<()> {
    let paths = paths_for(home, scope);
    let state = omega.join("state");
    fs::create_dir_all(&state)?;
    let body = format!(
        "# Generated by omega agk switch — no secrets.\n\
         export {ENV_SCOPE}={id}\n\
         export HERMES_HOME={hermes}\n\
         export OMEGA_RMUX_NS={id}\n",
        id = scope.id(),
        hermes = paths.hermes_home.display(),
    );
    fs::write(state.join(SCOPE_ENV_FILE), body)?;
    Ok(())
}

pub fn switch(home: &Path, scope: AgkScope) -> Result<ScopePaths> {
    let omega = omega_dir(home);
    write_active_scope(&omega, scope)?;
    write_scope_env(home, &omega, scope)?;
    Ok(paths_for(home, scope))
}

pub fn provision_all(home: &Path) -> Result<ProvisionReport> {
    let omega = omega_dir(home);
    let agents_md = omega.join("AGENTS.md");
    let mut report = ProvisionReport {
        workspaces: Vec::new(),
        profiles: Vec::new(),
        hermes_created: Vec::new(),
        skipped: Vec::new(),
    };
    let topo = crate::topology::load_resolved(home);
    let logical = topo
        .bindings
        .values()
        .all(|b| b.runtime == crate::topology::RuntimeKind::LogicalProfile);
    if logical {
        fs::create_dir_all(home.join("workspace"))?;
        fs::create_dir_all(home.join(".secrets"))?;
        fs::create_dir_all(home.join(".hermes").join("profiles"))?;
        chmod_private(&home.join(".secrets"))?;
        chmod_private(&omega.join("scopes"))?;
    }

    for scope in AgkScope::ALL {
        let paths = paths_for(home, scope);
        if let Err(e) = seed_workspace(&paths) {
            report
                .skipped
                .push(format!("{} workspace: {e}", scope.id()));
            continue;
        }
        if let Err(e) = fs::create_dir_all(&paths.secrets) {
            report.skipped.push(format!("{} secrets: {e}", scope.id()));
            continue;
        }
        chmod_private(&paths.workspace)?;
        chmod_private(&paths.scope_root)?;
        chmod_private(&paths.secrets)?;
        link_secrets(&paths)?;
        fs::create_dir_all(&paths.hermes_home)?;
        chmod_private(&paths.hermes_home)?;

        if topo.binding(scope).runtime == crate::topology::RuntimeKind::LogicalProfile {
            if let Some(note) = try_official_hermes_profile(home, scope) {
                report.hermes_created.push(note);
            }
        }

        let terminal = ScopeTerminal {
            cwd: paths.workspace.clone(),
            home_mode: scope.hermes_home_mode().map(str::to_string),
        };
        let agents = if agents_md.is_file() {
            Some(agents_md.as_path())
        } else {
            None
        };
        sync_hermes_tree(&paths.hermes_home, &omega, agents, Some(&terminal))?;

        report.workspaces.push(paths.workspace);
        report.profiles.push(paths.hermes_home);
    }

    if read_active_scope(&omega).is_none() {
        switch(home, AgkScope::Operator)?;
    } else {
        write_scope_env(home, &omega, persisted_scope(home))?;
    }

    write_agk_doctrine(&omega)?;

    Ok(report)
}

const AGK_MD_BEGIN: &str = "<!-- OMEGA-AGK:BEGIN -->";
const AGK_MD_END: &str = "<!-- OMEGA-AGK:END -->";
const AGK_MD_BODY: &str = "\
# AGK layers

OmegaOS presents and controls. Agentik Core organizes and governs. \
Hermes thinks and orchestrates. OS define methodology. Agents execute. \
RMUX keeps execution alive. Discord/Desktop/Web/TUI are surfaces. \
Profiles are logical — not Linux users. Route by canonical IDs, never paths.

OmegaOS must not duplicate Hermes or RMUX or become a session backend. \
Ask Hermes via `hermes -p <scope>` or `hermes gateway`. Never scrape `~/.hermes` JSON.

Four profiles: operator (machine), agentik (organization), mission (other organizations), \
private (self). `agk switch` is a full context change. BOT ≠ AGENT.
";

fn write_agk_doctrine(omega: &Path) -> Result<()> {
    upsert_marked_file(&omega.join("AGK.md"), AGK_MD_BEGIN, AGK_MD_END, AGK_MD_BODY)
}

fn seed_workspace(paths: &ScopePaths) -> Result<()> {
    fs::create_dir_all(&paths.workspace)?;
    for dir in canonical_dirs(paths.scope) {
        let path = paths.workspace.join(dir);
        fs::create_dir_all(&path)?;
        chmod_private(&path)?;
    }
    write_if_missing(
        &paths.workspace.join("README.md"),
        scope_readme(paths.scope),
    )?;
    if paths.scope == AgkScope::Mission {
        write_if_missing(
            &paths.workspace.join("clients").join("README.md"),
            "# Clients\n\n\
             One directory per client:\n\
             `clients/<name>/{.client,projects,missions,knowledge,artifacts,infrastructure,automation,docs}`.\n\n\
             Do not invent a sample client. Add a client only when one exists.\n\
             Never mix client work into `$HOME/workspace/private`.\n",
        )?;
    }
    Ok(())
}

fn scope_readme(scope: AgkScope) -> &'static str {
    match scope {
        AgkScope::Operator => {
            "# Operator\n\n\
             Operate the machine. Infrastructure, security, deployments, monitoring, \
             automation, deposit, and docs live here. Never mix operator work into \
             mission or private.\n"
        }
        AgkScope::Agentik => {
            "# Agentik\n\n\
             Operate the organization. Projects, products, missions, research, content, \
             growth, community, knowledge, and artifacts live here.\n"
        }
        AgkScope::Mission => {
            "# Mission\n\n\
             Operate other organizations (clients). Each client lives under \
             `clients/<name>/` with `.client`, `projects`, `missions`, `knowledge`, \
             `artifacts`, `infrastructure`, `automation`, and `docs`. Shared and \
             internal work stay in `shared/` and `internal/`. Never mix client work \
             into private.\n"
        }
        AgkScope::Private => {
            "# Private\n\n\
             Operate myself. Projects, journal, goals, learning, research, knowledge, \
             and artifacts live here. Never mix client work into this tree.\n"
        }
    }
}

fn write_if_missing(path: &Path, body: &str) -> Result<()> {
    if !path.exists() {
        fs::write(path, body)?;
    }
    Ok(())
}

fn link_secrets(paths: &ScopePaths) -> Result<()> {
    if let Some(parent) = paths.secrets_link.parent() {
        fs::create_dir_all(parent)?;
        chmod_private(parent)?;
    }
    if paths.secrets_link.exists() || paths.secrets_link.symlink_metadata().is_ok() {
        return Ok(());
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(&paths.secrets, &paths.secrets_link)?;
    #[cfg(not(unix))]
    {
        let _ = &paths.secrets;
    }
    Ok(())
}

fn chmod_private(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o700);
        fs::set_permissions(path, perms)?;
    }
    Ok(())
}

fn find_hermes(home: &Path) -> Option<PathBuf> {
    for cand in [
        home.join(".local").join("bin").join("hermes"),
        home.join(".hermes").join("bin").join("hermes"),
    ] {
        if cand.is_file() {
            return Some(cand);
        }
    }
    // Never invoke the host Hermes CLI from a TempDir provision test.
    if dirs::home_dir().as_deref() == Some(home) {
        return which("hermes");
    }
    None
}

fn which(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path).find_map(|dir| {
        let cand = dir.join(name);
        cand.is_file().then_some(cand)
    })
}

fn try_official_hermes_profile(home: &Path, scope: AgkScope) -> Option<String> {
    let bin = find_hermes(home)?;
    let marker = paths_for(home, scope).hermes_home.join("profile.yaml");
    if marker.is_file() {
        return Some(format!("{} (official profile present)", scope.id()));
    }
    let attempts: [&[&str]; 3] = [
        &[
            "profile",
            "create",
            scope.id(),
            "--no-alias",
            "--description",
            scope.description(),
        ],
        &[
            "profile",
            "create",
            scope.id(),
            "--description",
            scope.description(),
        ],
        &["profile", "create", scope.id()],
    ];
    for args in attempts {
        let ok = Command::new(&bin)
            .args(args)
            .env("HOME", home)
            .env("HERMES_HOME", home.join(".hermes"))
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if ok {
            return Some(format!("{} (hermes profile create)", scope.id()));
        }
    }
    None
}

pub fn sandbox_kind() -> SandboxKind {
    if which("bwrap").is_some() {
        SandboxKind::Bubblewrap
    } else {
        SandboxKind::None
    }
}

pub fn sandbox_available() -> bool {
    sandbox_kind() != SandboxKind::None
}

/// argv after `bwrap` and before `-- <command>`. Paths are unquoted; the
/// caller shell-quotes when embedding in a pane command.
pub fn sandbox_bind_args(home: &Path, scope: AgkScope) -> Vec<String> {
    let keep = paths_for(home, scope);
    let mut args = vec![
        "--die-with-parent".into(),
        "--bind".into(),
        "/".into(),
        "/".into(),
    ];
    let hides = [
        (home.join("workspace"), keep.workspace.clone()),
        (
            home.join(".hermes").join("profiles"),
            keep.hermes_home.clone(),
        ),
        (omega_dir(home).join("scopes"), keep.scope_root.clone()),
        (home.join(".secrets"), keep.secrets_link.clone()),
    ];
    for (parent, keep_path) in hides {
        args.push("--tmpfs".into());
        args.push(parent.to_string_lossy().into_owned());
        args.push("--bind".into());
        args.push(keep_path.to_string_lossy().into_owned());
        args.push(keep_path.to_string_lossy().into_owned());
    }
    for sensitive in [
        home.join(".hermes").join(".env"),
        omega_dir(home).join("telegram.toml"),
    ] {
        if sensitive.is_file() {
            args.push("--ro-bind".into());
            args.push("/dev/null".into());
            args.push(sensitive.to_string_lossy().into_owned());
        }
    }
    args
}

/// `bwrap … -- ` prefix, or `None` when isolation is not required / unavailable.
pub fn sandbox_exec_prefix(
    home: &Path,
    scope: AgkScope,
    quote: impl Fn(&str) -> String,
) -> Option<String> {
    if !scope.requires_runtime_isolation() || !sandbox_available() {
        return None;
    }
    if crate::topology::load_resolved(home).binding(scope).runtime
        == crate::topology::RuntimeKind::LinuxUser
    {
        return None;
    }
    let args = sandbox_bind_args(home, scope);
    let mut out = String::from("bwrap");
    for arg in args {
        out.push(' ');
        out.push_str(&quote(&arg));
    }
    out.push_str(" -- ");
    Some(out)
}

pub fn hermes_launch_home(home: &Path) -> PathBuf {
    match launch_scope() {
        Some(scope) => paths_for(home, scope).hermes_home,
        None => home.join(".hermes"),
    }
}

pub fn hermes_profile_flag() -> Option<&'static str> {
    launch_scope().map(AgkScope::id)
}

pub fn isolation_gap(home: &Path, yolo: bool) -> Option<String> {
    let scope = persisted_scope(home);
    if !scope.requires_runtime_isolation() {
        return None;
    }
    if crate::topology::load_resolved(home).binding(scope).runtime
        == crate::topology::RuntimeKind::LinuxUser
    {
        return None;
    }
    if sandbox_available() {
        return None;
    }
    if yolo {
        Some(format!(
            "{} YOLO without bwrap — sibling workspaces and secrets are visible to the agent. \
             Install bubblewrap (`apt install bubblewrap`) or do not enable hermes.yolo in this scope.",
            scope.id()
        ))
    } else {
        Some(format!(
            "{} has no bwrap — isolation is logical only. Install bubblewrap for sibling-hide.",
            scope.id()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn parse_accepts_aliases_and_rejects_unknown() {
        assert_eq!(AgkScope::parse("MISSION").unwrap(), AgkScope::Mission);
        assert_eq!(AgkScope::parse("perso").unwrap(), AgkScope::Private);
        assert!(AgkScope::parse("yolo").is_err());
    }

    #[test]
    fn sibling_paths_never_overlap() {
        let home = Path::new("/home/agentik");
        let mission = paths_for(home, AgkScope::Mission);
        let private = paths_for(home, AgkScope::Private);
        assert_ne!(mission.workspace, private.workspace);
        assert_ne!(mission.hermes_home, private.hermes_home);
        assert_ne!(mission.secrets, private.secrets);
        assert!(mission.workspace.ends_with("workspace/mission"));
        assert!(private.hermes_home.ends_with(".hermes/profiles/private"));
        assert!(mission.secrets.ends_with("scopes/mission/secrets"));
    }

    #[test]
    fn provision_is_idempotent_and_private_mode() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path();
        fs::create_dir_all(home.join(".omega")).unwrap();
        fs::write(home.join(".omega").join("AGENTS.md"), "# kernel\n").unwrap();

        let first = provision_all(home).unwrap();
        let second = provision_all(home).unwrap();
        assert_eq!(first.workspaces.len(), 4);
        assert_eq!(second.workspaces.len(), 4);

        for scope in AgkScope::ALL {
            let p = paths_for(home, scope);
            assert!(p.workspace.is_dir(), "{}", p.workspace.display());
            assert!(p.secrets.is_dir());
            assert!(p.hermes_home.join("SOUL.md").is_file());
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = fs::metadata(&p.workspace).unwrap().permissions().mode() & 0o777;
                assert_eq!(mode, 0o700, "{} mode {mode:o}", p.workspace.display());
                let smode = fs::metadata(&p.secrets).unwrap().permissions().mode() & 0o777;
                assert_eq!(smode, 0o700);
            }
        }

        let mission_cfg = fs::read_to_string(
            paths_for(home, AgkScope::Mission)
                .hermes_home
                .join("config.yaml"),
        )
        .unwrap();
        assert!(mission_cfg.contains("home_mode: profile"), "{mission_cfg}");
        assert!(
            mission_cfg.contains(
                &paths_for(home, AgkScope::Mission)
                    .workspace
                    .display()
                    .to_string()
            ),
            "{mission_cfg}"
        );

        let operator_cfg = fs::read_to_string(
            paths_for(home, AgkScope::Operator)
                .hermes_home
                .join("config.yaml"),
        )
        .unwrap();
        assert!(
            !operator_cfg.contains("home_mode: profile"),
            "{operator_cfg}"
        );

        assert_eq!(
            read_active_scope(&omega_dir(home)),
            Some(AgkScope::Operator)
        );
        let switched = switch(home, AgkScope::Mission).unwrap();
        assert_eq!(read_active_scope(&omega_dir(home)), Some(AgkScope::Mission));
        let env = fs::read_to_string(omega_dir(home).join("state").join(SCOPE_ENV_FILE)).unwrap();
        assert!(env.contains("OMEGA_SCOPE=mission"));
        assert!(env.contains(&switched.hermes_home.display().to_string()));
        assert!(!env.contains("TELEGRAM"));
        assert!(!env.contains("token"));
    }

    #[test]
    fn sandbox_args_hide_sibling_trees() {
        let home = Path::new("/home/agentik");
        let args = sandbox_bind_args(home, AgkScope::Mission);
        let joined = args.join(" ");
        assert!(joined.contains("--tmpfs /home/agentik/workspace"));
        assert!(joined.contains("/home/agentik/workspace/mission"));
        assert!(joined.contains("/home/agentik/.hermes/profiles/mission"));
        assert!(joined.contains("/home/agentik/.omega/scopes/mission"));
        assert!(!joined.contains("/home/agentik/workspace/private"));
        assert!(!joined.contains("profiles/private"));
        assert!(!AgkScope::Operator.requires_runtime_isolation());
        assert!(AgkScope::Mission.requires_runtime_isolation());
    }

    #[test]
    fn provision_creates_every_canonical_dir() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path();
        fs::create_dir_all(home.join(".omega")).unwrap();
        provision_all(home).unwrap();
        for scope in AgkScope::ALL {
            let root = paths_for(home, scope).workspace;
            for dir in canonical_dirs(scope) {
                assert!(
                    root.join(dir).is_dir(),
                    "missing {}/{} after provision_all",
                    scope.id(),
                    dir
                );
            }
        }
        assert!(!paths_for(home, AgkScope::Mission)
            .workspace
            .join("clients")
            .join("moonbase")
            .exists());
        let agk = fs::read_to_string(omega_dir(home).join("AGK.md")).unwrap();
        assert!(agk.contains("presents and controls"), "{agk}");
        assert!(agk.contains("hermes -p"), "{agk}");
        assert!(!agk.to_ascii_lowercase().contains("token"));
        assert!(!agk.to_ascii_lowercase().contains("secret"));
    }
}

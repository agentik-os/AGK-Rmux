//! `omega doctor` — a one-shot health check of the whole stack. The first
//! command to run after a fresh install / VPS reset: it tells you, in one
//! screen, whether the daemon, doctrine, agent CLI, Telegram service, secrets,
//! and resources are all in order. Adapts to the host — a missing systemd or
//! crontab is a soft warning, never a hard error.

use crate::config::OmegaConfig;
use crate::session::SessionManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Health {
    Ok,
    Warn,
    Fail,
}

impl Health {
    pub fn glyph(&self) -> &'static str {
        match self {
            Health::Ok => "[+]",
            Health::Warn => "[!]",
            Health::Fail => "[x]",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Check {
    pub name: String,
    pub health: Health,
    pub detail: String,
}

impl Check {
    fn ok(name: &str, detail: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            health: Health::Ok,
            detail: detail.into(),
        }
    }
    fn warn(name: &str, detail: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            health: Health::Warn,
            detail: detail.into(),
        }
    }
    fn fail(name: &str, detail: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            health: Health::Fail,
            detail: detail.into(),
        }
    }
}

/// Is the installed binary the one this checkout would build?
///
/// The comparison is `auto-update.json:last_applied_commit` (what an install
/// last recorded putting on disk) against the checkout's HEAD. There is no
/// build-time sha embedded in the binary, and adding one would only move the
/// question — an install that never records itself is unverifiable either way,
/// so the honest fix is to make BOTH install paths record, then compare.
///
/// Three outcomes, and the middle one matters most:
///   - no checkout on this box (an `npx omega-os` install): nothing to compare,
///     stay quiet rather than invent a complaint;
///   - no recorded provenance: WARN, not FAIL — an install predating the record
///     is not proof of staleness, and failing on it would cry wolf on every
///     machine that upgrades into this version;
///   - recorded commit ≠ HEAD: FAIL, and say the command that fixes it.
fn binary_provenance(config: &OmegaConfig) -> Check {
    let Some(src) = crate::config::resolve_omega_src() else {
        return Check::ok(
            "binary provenance",
            "no source checkout on this box (npx install)",
        );
    };

    let head = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(&src)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();

    if head.is_empty() {
        return Check::warn(
            "binary provenance",
            format!("{} has no readable git HEAD", src.display()),
        );
    }

    let history = crate::auto_update::AutoUpdateState::load(&config.state_dir);
    match history.last_applied_commit.as_deref() {
        Some(installed) if installed == head => {
            Check::ok("binary provenance", format!("built from HEAD ({head})"))
        }
        Some(installed) => {
            // DIRECTION MATTERS, and getting it wrong is how a fixed binary
            // gets silently replaced by a stale one. A box can hold more than
            // one checkout; when the resolved one is BEHIND the installed
            // binary, "run ./install.sh here" is advice to downgrade, and an
            // operator who follows it reinstalls the very bug they just had
            // fixed. Only a checkout that is AHEAD means the binary is stale.
            let count = |range: String| -> Option<u32> {
                std::process::Command::new("git")
                    .args(["rev-list", "--count", &range])
                    .current_dir(&src)
                    .output()
                    .ok()
                    .filter(|out| out.status.success())
                    .and_then(|out| String::from_utf8_lossy(&out.stdout).trim().parse().ok())
            };
            let ahead = count(format!("{installed}..{head}"));
            let behind = count(format!("{head}..{installed}"));
            match (ahead, behind) {
                // The checkout carries work the binary lacks, and nothing the
                // other way: the binary really is stale.
                (Some(a), Some(0)) if a > 0 => Check::fail(
                    "binary provenance",
                    format!(
                        "installed binary is from {installed}, checkout HEAD is {head} \
                         ({a} commit(s) behind) — run: cd {} && ./install.sh",
                        src.display()
                    ),
                ),
                // The binary is ahead. Reinstalling here would DOWNGRADE the
                // box, which is precisely how a just-fixed binary gets
                // silently replaced by a stale one.
                (Some(0), Some(b)) if b > 0 => Check::warn(
                    "binary provenance",
                    format!(
                        "the installed binary ({installed}) is NEWER than {} (HEAD {head}, \
                         {b} commit(s) behind it) — reinstalling from there would DOWNGRADE it; \
                         update that checkout first",
                        src.display()
                    ),
                ),
                (Some(a), Some(b)) if a > 0 && b > 0 => Check::warn(
                    "binary provenance",
                    format!(
                        "{} has DIVERGED from the installed binary: {a} commit(s) it has and the \
                         binary lacks, {b} the binary has and it lacks — reconcile that checkout \
                         before installing from it",
                        src.display()
                    ),
                ),
                // Unknown ancestry (a shallow clone, a missing object): say so
                // rather than guess a direction and hand out the wrong command.
                _ => Check::warn(
                    "binary provenance",
                    format!(
                        "installed binary is from {installed} and {} is at {head}; their \
                         relationship could not be determined here",
                        src.display()
                    ),
                ),
            }
        }
        None => Check::warn(
            "binary provenance",
            format!(
                "no install recorded which commit it installed, so staleness cannot be \
                 proven (HEAD is {head}) — the next ./install.sh records it"
            ),
        ),
    }
}

/// Run a `systemctl --user` query, returning its trimmed stdout (or None if
/// systemd / the unit isn't available — a soft condition, not an error).
fn systemctl_user(args: &[&str]) -> Option<String> {
    // Via service::systemctl_user_cmd so XDG_RUNTIME_DIR is defaulted — cron /
    // hook environments otherwise get "Failed to connect to bus" + empty stdout
    // and misreport a RUNNING service as absent.
    let out = crate::service::systemctl_user_cmd()
        .args(args)
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn ram_available_mb() -> u64 {
    crate::sysinfo::available_ram_mb()
}

/// The rmux daemon socket path, if present — `$RMUX_SOCKET` override, else the
/// conventional `/tmp/rmux-<uid>/default`.
fn rmux_socket_path() -> Option<std::path::PathBuf> {
    if let Ok(s) = std::env::var("RMUX_SOCKET") {
        let p = std::path::PathBuf::from(s);
        if p.exists() {
            return Some(p);
        }
    }
    for e in std::fs::read_dir("/tmp").ok()?.flatten() {
        if e.file_name().to_string_lossy().starts_with("rmux-") {
            let sock = e.path().join("default");
            if sock.exists() {
                return Some(sock);
            }
        }
    }
    None
}

fn check_rmux_color_env() -> Check {
    let output = match std::process::Command::new("rmux")
        .args(["show-environment", "-g"])
        .output()
    {
        Ok(output) if output.status.success() => output,
        _ => {
            return Check::warn("rmux color", "could not read rmux environment");
        }
    };
    let text = String::from_utf8_lossy(&output.stdout);
    let no_color = text
        .lines()
        .any(|line| line == "NO_COLOR" || line.starts_with("NO_COLOR=") && line != "NO_COLOR=");
    let force_off = text
        .lines()
        .any(|line| line == "FORCE_COLOR=0" || line.eq_ignore_ascii_case("FORCE_COLOR=false"));
    if no_color || force_off {
        Check::warn(
            "rmux color",
            "daemon inherited NO_COLOR/FORCE_COLOR=0 — existing panes stay grayscale until relaunch (Menu → R). New panes are sanitized.",
        )
    } else {
        Check::ok(
            "rmux color",
            "FORCE_COLOR=1, NO_COLOR unset (agent panes keep color)",
        )
    }
}

/// Claude/Codex hooks: scripts present under `~/.omega/hooks` and registered
/// on both provider surfaces.
fn check_hooks(config: &OmegaConfig) -> Check {
    let hooks_dir = config
        .state_dir
        .parent()
        .map(|p| p.join("hooks"))
        .unwrap_or_else(|| std::path::PathBuf::from("/nonexistent"));
    // The anti-abandon set is load-bearing doctrine, not decoration: without
    // these an agent can stop mid-mission and nothing notices. Check each by
    // name and say which one is missing — "hooks missing" sent the operator
    // hunting, and the old check only knew two of them.
    const REQUIRED: &[(&str, &str)] = &[
        ("stop-verify-hook.sh", "stop-verify"),
        ("omega-session-contract.sh", "omega-session-contract"),
        ("omega-prompt-scan.sh", "omega-prompt-scan"),
        ("omega-plan-mirror.sh", "omega-plan-mirror"),
        ("omega-audit-guard.sh", "omega-audit-guard"),
        ("omega_plan_state.py", ""), // shared parser, not registered anywhere
        ("track-tool-use.sh", "track-tool-use"),
    ];

    let settings = dirs::home_dir()
        .map(|h| h.join(".claude/settings.json"))
        .and_then(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_default();
    let codex_hooks = dirs::home_dir()
        .map(|home| home.join(".codex/hooks.json"))
        .and_then(|path| std::fs::read_to_string(path).ok())
        .unwrap_or_default();

    let mut missing_files = Vec::new();
    let mut unregistered_claude = Vec::new();
    let mut unregistered_codex = Vec::new();
    for (file, marker) in REQUIRED {
        if !hooks_dir.join(file).exists() {
            missing_files.push(*file);
        } else if !marker.is_empty() && !settings.contains(marker) {
            unregistered_claude.push(*file);
        }
        if !marker.is_empty() && hooks_dir.join(file).exists() && !codex_hooks.contains(marker) {
            unregistered_codex.push(*file);
        }
    }

    if !missing_files.is_empty() {
        return Check::warn(
            "hooks",
            format!(
                "missing from {}: {} (re-run install.sh)",
                hooks_dir.display(),
                missing_files.join(", ")
            ),
        );
    }
    if !unregistered_claude.is_empty() || !unregistered_codex.is_empty() {
        return Check::warn(
            "hooks",
            format!(
                "present but not registered (Claude: {}; Codex: {}) — re-run install.sh; needs jq",
                unregistered_claude.join(", "),
                unregistered_codex.join(", ")
            ),
        );
    }
    Check::ok(
        "hooks",
        format!(
            "{} hooks present + registered for Claude and Codex (finish-guard armed)",
            REQUIRED.len()
        ),
    )
}

fn agk_scope_check(yolo: bool) -> Check {
    let Some(home) = dirs::home_dir() else {
        return Check::warn("agk scopes", "HOME unavailable");
    };
    let missing: Vec<&str> = crate::tenancy::AgkScope::ALL
        .iter()
        .filter(|scope| !crate::tenancy::paths_for(&home, **scope).workspace.is_dir())
        .map(|scope| scope.id())
        .collect();
    if !missing.is_empty() {
        return Check::warn(
            "agk scopes",
            format!("missing {} — omega agk provision", missing.join(", ")),
        );
    }
    let scope = crate::tenancy::persisted_scope(&home);
    let isolation = match crate::tenancy::sandbox_kind() {
        crate::tenancy::SandboxKind::Bubblewrap => "bwrap",
        crate::tenancy::SandboxKind::None => "logical",
    };
    if let Some(gap) = crate::tenancy::isolation_gap(&home, yolo) {
        if yolo && scope.requires_runtime_isolation() {
            return Check::fail("agk scopes", gap);
        }
        return Check::warn("agk scopes", gap);
    }
    Check::ok(
        "agk scopes",
        format!("{} active, isolation={isolation}", scope.id()),
    )
}

fn hermes_gateway_check() -> Check {
    let Some(home) = dirs::home_dir() else {
        return Check::warn("hermes gateway", "HOME unavailable");
    };
    let report = crate::hermes_gateway::inspect(&home);
    if report.cli.is_none() {
        return Check::warn(
            "hermes gateway",
            "hermes CLI missing — optional Home stream: omega install hermes",
        );
    }
    if report.telegram_collision {
        return Check::fail(
            "hermes gateway",
            "TELEGRAM_BOT_TOKEN equals the Omega Atlas bot — create a second @BotFather token",
        );
    }
    if !report.configured() {
        return Check::ok(
            "hermes gateway",
            "idle (omega hermes-gateway setup to connect Telegram/Discord)",
        );
    }
    let platforms = report.platforms.join(", ");
    match report.service {
        crate::hermes_gateway::GatewayService::Running => {
            Check::ok("hermes gateway", format!("running ({platforms})"))
        }
        crate::hermes_gateway::GatewayService::Stopped => Check::warn(
            "hermes gateway",
            format!("{platforms} configured but stopped — omega hermes-gateway start"),
        ),
        crate::hermes_gateway::GatewayService::Missing => Check::warn(
            "hermes gateway",
            format!("{platforms} configured but unit missing — omega hermes-gateway install"),
        ),
    }
}

fn effective_containment(
    config: &OmegaConfig,
    providers: &crate::providers::ProvidersConfig,
) -> Check {
    use crate::agents::Agent;
    let Some(agent) = Agent::from_name(&config.agent_command) else {
        return Check::fail(
            "agent containment",
            format!(
                "unknown provider {:?}; runtime launch is blocked",
                config.agent_command
            ),
        );
    };
    match agent {
        Agent::Claude => {
            if providers.claude.dangerously_skip_permissions {
                Check::warn(
                    "agent containment",
                    "Claude: explicit HIGH-RISK permission bypass enabled in providers.toml",
                )
            } else {
                Check::ok(
                    "agent containment",
                    "Claude: permission-mode auto, inline TTY, project trust preflight",
                )
            }
        }
        Agent::Codex if providers.codex.bypass_hook_trust => Check::ok(
            "agent containment",
            format!(
                "Codex: strict config, approve-for-me preset (workspace-write + auto-review), hook-trust bypass; state+locks and {} configured extra writable root(s)",
                providers.codex.additional_writable_dirs.len()
            ),
        ),
        Agent::Codex => Check::warn(
            "agent containment",
            "Codex hook-trust bypass disabled; a new/changed hook can block a detached pane",
        ),
        Agent::Glm => {
            if providers.glm.dangerously_skip_permissions {
                Check::warn(
                    "agent containment",
                    "GLM/Claude adapter: explicit HIGH-RISK permission bypass enabled",
                )
            } else {
                Check::ok(
                    "agent containment",
                    "GLM/Claude adapter: permission-mode auto, inline TTY, scoped endpoint redirect",
                )
            }
        }
        Agent::Kimi => Check::warn(
            "agent containment",
            "Kimi Code: auto approval mode uses provider-native controls; OmegaOS adds no separate filesystem sandbox; direct credentials stay in KIMI_MODEL_*",
        ),
        Agent::Gemini => Check::warn(
            "agent containment",
            "Gemini: provider-native policy applies; OmegaOS adds no separate filesystem sandbox",
        ),
        Agent::Antigravity => {
            if providers.antigravity.dangerously_skip_permissions {
                Check::warn(
                    "agent containment",
                    "Antigravity: explicit HIGH-RISK permission bypass enabled for detached Omega sessions",
                )
            } else {
                Check::warn(
                    "agent containment",
                    "Antigravity: provider-native approval policy may block a detached session",
                )
            }
        }
        Agent::Pi | Agent::OpenRouter | Agent::Hermes => Check::warn(
            "agent containment",
            format!(
                "{}: provider-native tool policy applies; OmegaOS adds no separate filesystem sandbox",
                agent.name()
            ),
        ),
        Agent::Shell => Check::warn(
            "agent containment",
            "shell: unrestricted local shell selected; no model/tool sandbox applies",
        ),
    }
}

fn minimum_agent_version(agent: crate::agents::Agent) -> Option<semver::Version> {
    use crate::agents::Agent;
    let raw = match agent {
        // Opus 5 support starts here.
        Agent::Claude | Agent::Glm => "2.1.219",
        // --sandbox workspace-write + --ask-for-approval never (never pair
        // sandbox with --approve-for-me; 0.149 dies).
        Agent::Codex => "0.147.0",
        // First stable Gemini 3.1 model support.
        Agent::Gemini => "0.31.0",
        // Stable structured/headless and prompt-interactive contract.
        Agent::Antigravity => "1.1.8",
        // `--` end-of-options support used by the launch adapter.
        Agent::Pi | Agent::OpenRouter => "0.84.3",
        Agent::Hermes => "0.20.0",
        Agent::Kimi => "0.38.0",
        Agent::Shell => return None,
    };
    semver::Version::parse(raw).ok()
}

fn parse_cli_version(raw: &str) -> Option<semver::Version> {
    raw.split_whitespace().find_map(|token| {
        let candidate = token
            .trim_matches(|character: char| {
                !character.is_ascii_alphanumeric()
                    && character != '.'
                    && character != '-'
                    && character != '+'
            })
            .trim_start_matches('v');
        semver::Version::parse(candidate).ok()
    })
}

fn agent_version_check(agent: crate::agents::Agent) -> Check {
    let name = format!("{} version", agent.name());
    let Some(minimum) = minimum_agent_version(agent) else {
        return Check::ok(&name, "local shell");
    };
    let output = std::process::Command::new(agent.binary_name())
        .arg("--version")
        .output();
    let Ok(output) = output else {
        return Check::warn(
            &name,
            format!("could not execute {} --version", agent.binary_name()),
        );
    };
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let Some(version) = parse_cli_version(&combined) else {
        return Check::warn(
            &name,
            format!(
                "unrecognized version output from {}: {}",
                agent.binary_name(),
                combined.trim()
            ),
        );
    };
    if !output.status.success() {
        return Check::warn(
            &name,
            format!(
                "{} --version exited {:?} (reported {version})",
                agent.binary_name(),
                output.status.code()
            ),
        );
    }
    if version < minimum {
        Check::fail(
            &name,
            format!(
                "{version} is older than supported minimum {minimum}; run: omega install {} --force",
                agent.name()
            ),
        )
    } else {
        Check::ok(&name, format!("{version} (minimum {minimum})"))
    }
}

fn agents_override_files(
    cwd: &std::path::Path,
    codex_home: &std::path::Path,
) -> Vec<std::path::PathBuf> {
    let mut found = std::collections::BTreeSet::new();
    let global = codex_home.join("AGENTS.override.md");
    if global.is_file() {
        found.insert(global);
    }
    for directory in cwd.ancestors() {
        let candidate = directory.join("AGENTS.override.md");
        if candidate.is_file() {
            found.insert(candidate);
        }
    }
    found.into_iter().collect()
}

fn check_agents_override_shadowing() -> Check {
    let cwd = match std::env::current_dir() {
        Ok(path) => path,
        Err(error) => {
            return Check::warn(
                "AGENTS override",
                format!("cannot inspect current directory: {error}"),
            )
        }
    };
    let codex_home = std::env::var("CODEX_HOME")
        .map(std::path::PathBuf::from)
        .ok()
        .or_else(|| dirs::home_dir().map(|home| home.join(".codex")));
    let Some(codex_home) = codex_home else {
        return Check::warn("AGENTS override", "home directory unavailable");
    };
    let overrides = agents_override_files(&cwd, &codex_home);
    if overrides.is_empty() {
        Check::ok(
            "AGENTS override",
            "no active AGENTS.override.md shadowing detected",
        )
    } else {
        Check::warn(
            "AGENTS override",
            format!(
                "override precedence active at {}; files were inspected only, never modified",
                overrides
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        )
    }
}

/// Run every health check. Each is independent and never panics.
pub async fn run_all(config: &OmegaConfig) -> Vec<Check> {
    let mut checks = Vec::new();

    // 1. Binary version.
    checks.push(Check::ok(
        "binary",
        format!("omega {}", env!("CARGO_PKG_VERSION")),
    ));

    // Configuration integrity is checked independently from the caller. Older
    // entry points may have constructed a diagnostic default after a load
    // failure; doctor must still expose that negative state.
    match OmegaConfig::load() {
        Ok(_) => checks.push(Check::ok(
            "runtime config",
            format!(
                "{} parsed and validated",
                OmegaConfig::config_path().display()
            ),
        )),
        Err(error) => checks.push(Check::fail(
            "runtime config",
            format!("runtime authority blocked: {error:#}"),
        )),
    }

    let providers = crate::providers::ProvidersConfig::try_load();
    match &providers {
        Ok(provider_config) => {
            checks.push(Check::ok(
                "provider config",
                format!(
                    "{} parsed, {} typed providers available",
                    crate::providers::ProvidersConfig::path().display(),
                    crate::providers::ProvidersConfig::all_providers().len()
                ),
            ));
            checks.push(effective_containment(config, provider_config));
        }
        Err(error) => {
            checks.push(Check::fail(
                "provider config",
                format!("agent launch blocked: {error:#}"),
            ));
            checks.push(Check::fail(
                "agent containment",
                "effective launch profile unavailable because providers.toml is invalid",
            ));
        }
    }

    match crate::providers::ActiveModel::try_load() {
        Ok(model) => checks.push(Check::ok(
            "active model state",
            format!("{} / {}", model.active_provider, model.active_model),
        )),
        Err(error) => checks.push(Check::fail(
            "active model state",
            format!("model routing state invalid: {error:#}"),
        )),
    }
    checks.push(check_agents_override_shadowing());

    // 1b. Binary PROVENANCE — is the installed binary built from the checkout
    // that is on disk right now?
    //
    // Every other check here reads the source tree or ~/.omega, so all of them
    // stay green while the binary drifts behind the checkout it is supposed to
    // come from. That is exactly what happened on the source box: doctor
    // reported "all systems healthy" for five days over a binary thirty commits
    // old. A green board that structurally cannot see the failure is worse than
    // a missing check, because it is read as proof.
    checks.push(binary_provenance(config));

    // 2. rmux daemon reachable.
    match SessionManager::connect().await {
        Ok(mgr) => match mgr.list_sessions().await {
            Ok(sessions) => checks.push(Check::ok(
                "rmux daemon",
                format!("connected, {} live session(s)", sessions.len()),
            )),
            Err(error) => checks.push(Check::fail(
                "rmux daemon",
                format!("connected but list-sessions RPC failed: {error}"),
            )),
        },
        Err(e) => checks.push(Check::fail("rmux daemon", format!("unreachable: {}", e))),
    }

    // 2b. rmux socket file present (distinct from the daemon RPC above —
    // catches a dead daemon that left a stale or missing socket).
    match rmux_socket_path() {
        Some(p) => checks.push(Check::ok("rmux socket", p.display().to_string())),
        None => checks.push(Check::warn(
            "rmux socket",
            "no socket under /tmp/rmux-*/ or $RMUX_SOCKET (daemon down?)",
        )),
    }

    // 2c. Color env. Cursor/CI start rmux with NO_COLOR=1 FORCE_COLOR=0;
    // every pane inherits it and Claude/Codex go grayscale. connect() now
    // sanitizes the daemon session env; this check still fires if that
    // failed, so a gray board is never reported as healthy.
    checks.push(check_rmux_color_env());

    // 3. Doctrine integrity — a FLOOR, not an exact count.
    //
    // This used to hardcode "6 Laws + 36 Rules" and its own comment said to bump
    // the constant on every new rule. Nobody ever does, so the check spent most
    // of its life warning about a healthy system (it fired the day L6 + R-PLAN
    // shipped), and a doctor that always warns is a doctor the operator learns
    // to skim. Exact drift is already caught precisely by check 3b, which
    // compares the on-disk rule files against the compiled registry. What is
    // worth checking HERE is only that the registry is not gutted — a stripped
    // or half-linked binary.
    const MIN_LAWS: usize = 6;
    const MIN_OPS: usize = 30;
    let laws = crate::rules::laws().len();
    let ops = crate::rules::operational_rules().len();
    if laws >= MIN_LAWS && ops >= MIN_OPS {
        checks.push(Check::ok(
            "doctrine",
            format!("{} Laws + {} Rules", laws, ops),
        ));
    } else {
        checks.push(Check::warn(
            "doctrine",
            format!(
                "{} Laws + {} Rules — below the sane floor ({} + {}); registry looks truncated",
                laws, ops, MIN_LAWS, MIN_OPS
            ),
        ));
    }

    // 3b. Doctrine FILES — on-disk ~/.omega/rules vs the compiled registry.
    // The count check above sees only the binary; agents actually LOAD the
    // exported .md files, so a deleted / extra / renamed file drifted
    // silently before this. Cheap id-set diff using the same basename
    // grammar as the rules parity test. Extra ids are reported, not failed —
    // disk-only rules (repo `rules/*.md` not in the registry) are legal.
    {
        let rules_dir = config.state_dir.parent().map(|p| p.join("rules"));
        match rules_dir {
            Some(dir) if dir.is_dir() => {
                let on_disk = crate::rules::markdown_rule_ids(&dir);
                let registry: std::collections::BTreeSet<String> = crate::rules::all_rules()
                    .iter()
                    .map(|r| r.id.to_string())
                    .collect();
                let missing: Vec<String> = registry.difference(&on_disk).cloned().collect();
                let extra: Vec<String> = on_disk.difference(&registry).cloned().collect();
                if missing.is_empty() && extra.is_empty() {
                    checks.push(Check::ok(
                        "doctrine files",
                        format!("{} rule files match the registry", on_disk.len()),
                    ));
                } else {
                    let mut msg = String::from("on-disk rules drift");
                    if !missing.is_empty() {
                        msg.push_str(&format!(" — missing: {}", missing.join(", ")));
                    }
                    if !extra.is_empty() {
                        msg.push_str(&format!(" — extra/disk-only: {}", extra.join(", ")));
                    }
                    msg.push_str(" (fix: omega rules export)");
                    checks.push(Check::warn("doctrine files", msg));
                }
            }
            _ => checks.push(Check::warn(
                "doctrine files",
                "~/.omega/rules missing — run: omega rules export",
            )),
        }
    }

    // 4. Agent CLI available.
    match crate::agents::Agent::from_name(&config.agent_command) {
        Some(agent) if agent.is_available() => checks.push(Check::ok(
            "agent CLI",
            format!("{} available", agent.name()),
        )),
        Some(agent) => {
            let hint = agent
                .install_command()
                .map(|c| format!(" — {}", c))
                .unwrap_or_default();
            checks.push(Check::warn(
                "agent CLI",
                format!("{} not on PATH{}", agent.name(), hint),
            ))
        }
        None => checks.push(Check::fail(
            "agent CLI",
            format!(
                "unknown agent '{}'; runtime launch is blocked",
                config.agent_command
            ),
        )),
    }

    // 4a. Validate every installed provider CLI against the oldest version
    // whose flags Omega emits. Presence alone previously let an old binary
    // fail later with an opaque "unknown option" inside a detached pane.
    let mut checked_binaries = std::collections::BTreeSet::new();
    for agent in crate::agents::Agent::all().iter().copied() {
        if matches!(agent, crate::agents::Agent::Shell)
            || !agent.is_available()
            || !checked_binaries.insert(agent.binary_name())
        {
            continue;
        }
        checks.push(agent_version_check(agent));
    }

    // 4b. Codex topology. A real native file beside a canonical credential is
    // an explicit split, not a healthy login. This is a cheap local check; it
    // never sends a provider request.
    {
        let codex = crate::agents::Agent::Codex;
        if !codex.is_available() {
            let hint = codex
                .install_command()
                .map(|c| format!(" — {}", c))
                .unwrap_or_default();
            checks.push(Check::warn("codex", format!("codex not on PATH{}", hint)));
        } else {
            match crate::credentials::CredentialStore::new()
                .map(|store| store.codex_topology())
            {
                Ok(topology)
                    if topology.canonical_validity
                        != crate::credentials::CodexCredentialValidity::Valid
                        && topology.native_validity
                            != crate::credentials::CodexCredentialValidity::Valid =>
                {
                    checks.push(Check::warn(
                        "codex",
                        "not logged in; run: omega codex-login",
                    ))
                }
                Ok(topology) if topology.split => checks.push(Check::warn(
                    "codex",
                    format!(
                        "native/canonical credential split (native: {}, canonical: {}); no copy was discarded",
                        topology.native_path.display(),
                        topology.canonical_path.display()
                    ),
                )),
                Ok(topology)
                    if topology.native_links_to_canonical
                        && topology.canonical_validity
                            == crate::credentials::CodexCredentialValidity::Valid =>
                {
                    checks.push(Check::ok(
                        "codex",
                        format!(
                            "canonical credential topology via {}",
                            topology.native_path.display()
                        ),
                    ))
                }
                Ok(topology) => checks.push(Check::warn(
                    "codex",
                    format!(
                        "credential topology incomplete (native exists: {}, canonical exists: {}, native linked: {})",
                        topology.native_exists,
                        topology.canonical_exists,
                        topology.native_links_to_canonical
                    ),
                )),
                Err(error) => checks.push(Check::fail(
                    "codex",
                    format!("credential topology unavailable: {error}"),
                )),
            }
        }
    }

    // 4c. Recorded device-flow state and stale backups. Read-only: doctor
    // never settles a flow and never signals any Codex or rmux process.
    {
        let diagnostic = crate::codex_login::diagnostics();
        if diagnostic.active_flow {
            let detail = match diagnostic.active_pid {
                Some(pid) if pid != 0 => format!(
                    "recorded pid {pid} is {}; settle with `omega codex-login-status --pid {pid}`",
                    diagnostic.active_process
                ),
                _ => format!(
                    "{}; run `omega codex-login-status` to reconcile it",
                    diagnostic.active_process
                ),
            };
            checks.push(Check::warn("codex login flow", detail));
        } else if diagnostic.stale_backups > 0 {
            checks.push(Check::warn(
                "codex login flow",
                format!(
                    "{} stale owner-only backup(s) require review; no process was signalled",
                    diagnostic.stale_backups
                ),
            ));
        } else {
            checks.push(Check::ok(
                "codex login flow",
                "no active flow or stale backup",
            ));
        }
    }

    // 5. State dir writable.
    let probe = config.state_dir.join(".doctor-probe");
    match std::fs::write(&probe, b"ok").and_then(|_| std::fs::remove_file(&probe)) {
        Ok(()) => checks.push(Check::ok(
            "state dir",
            config.state_dir.display().to_string(),
        )),
        Err(e) => checks.push(Check::fail(
            "state dir",
            format!("{} not writable: {}", config.state_dir.display(), e),
        )),
    }

    // 6. Telegram service (systemd on Linux, launchd on macOS — soft if absent).
    match crate::service::tg_bot_status() {
        Some(s) if s == "active" => {
            checks.push(Check::ok("telegram service", "omega-tg-bot active"))
        }
        Some(other) => checks.push(Check::warn(
            "telegram service",
            format!(
                "omega-tg-bot {} (start: {})",
                other,
                crate::service::tg_bot_start_hint()
            ),
        )),
        None => checks.push(Check::warn(
            "telegram service",
            "user service not found (optional)",
        )),
    }

    // 6a. Hermes messaging gateway (optional). Idle is fine; a shared
    // Telegram token with Atlas is a hard fail.
    checks.push(hermes_gateway_check());

    // 6a2. AGK scopes (operator/agentik/mission/private). Missing trees are
    // a warn; YOLO mission/private without bwrap is a fail.
    let hermes_yolo = providers.as_ref().map(|p| p.hermes.yolo).unwrap_or(false);
    checks.push(agk_scope_check(hermes_yolo));

    // 6b. Claude Code hooks installed + registered.
    checks.push(check_hooks(config));

    // 7. Secrets present (~/.omega exists + non-empty).
    let omega_dir = config.state_dir.parent().map(|p| p.to_path_buf());
    match omega_dir {
        Some(dir) if dir.exists() => {
            let has_any = std::fs::read_dir(&dir)
                .map(|mut e| e.next().is_some())
                .unwrap_or(false);
            if has_any {
                checks.push(Check::ok(
                    "secrets dir",
                    format!("{} present", dir.display()),
                ));
            } else {
                checks.push(Check::warn(
                    "secrets dir",
                    format!("{} empty", dir.display()),
                ));
            }
        }
        _ => checks.push(Check::warn("secrets dir", "~/.omega not found")),
    }

    // 8. Memory headroom.
    let ram = ram_available_mb();
    if ram == 0 {
        checks.push(Check::warn(
            "memory",
            "memory stats unreadable (/proc/meminfo or vm_stat)",
        ));
    } else if ram < 400 {
        checks.push(Check::warn(
            "memory",
            format!("{}MB available — low (try 'omega cleanup --yes')", ram),
        ));
    } else {
        checks.push(Check::ok("memory", format!("{}MB available", ram)));
    }

    // 9. Usage cache (drives the TUI token toolbar; refreshed by the
    //    `omega usage` cron). Soft — a blank toolbar is not a failure.
    {
        let usage = config.state_dir.join("usage.json");
        match std::fs::metadata(&usage).and_then(|m| m.modified()) {
            Ok(mtime) => {
                let age = std::time::SystemTime::now()
                    .duration_since(mtime)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let mins = age / 60;
                if age > 900 {
                    checks.push(Check::warn(
                        "usage cache",
                        format!(
                            "usage cache stale ({} min) — omega usage cron may be failing",
                            mins
                        ),
                    ));
                } else {
                    checks.push(Check::ok(
                        "usage cache",
                        format!("usage cache {} min old", mins),
                    ));
                }
            }
            Err(_) => checks.push(Check::warn(
                "usage cache",
                "usage.json missing — TUI token toolbar blank (cron not run)",
            )),
        }
    }

    // 10. Claude OAuth credential — the agent CLI needs a live token.
    {
        let omega_dir = config.state_dir.parent().map(|p| p.to_path_buf());
        let cred = omega_dir.map(|d| d.join("credentials/claude.json"));
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i128)
            .unwrap_or(0);
        let parse_expires = |content: &str| {
            serde_json::from_str::<serde_json::Value>(content)
                .ok()
                .and_then(|v| {
                    // The Claude credential nests the token under `claudeAiOauth`;
                    // fall back to a top-level field for older/alternate formats.
                    v.get("claudeAiOauth")
                        .and_then(|o| o.get("expiresAt"))
                        .or_else(|| v.get("expiresAt"))
                        .and_then(|e| e.as_i64())
                        .map(|n| n as i128)
                })
        };
        let file_expires = cred
            .and_then(|p| std::fs::read_to_string(&p).ok())
            .and_then(|c| parse_expires(&c));
        match file_expires {
            Some(exp) if exp < now_ms => checks.push(Check::warn(
                "claude oauth",
                "Claude OAuth expired — refresh required",
            )),
            Some(_) => checks.push(Check::ok("claude oauth", "Claude OAuth valid")),
            // No (parseable) credential FILE. On macOS the Claude CLI stores it
            // in the login Keychain instead — probe it before warning, so a
            // healthy Mac doesn't report a broken credential chain.
            None => {
                let keychain_expires = if cfg!(target_os = "macos") {
                    std::process::Command::new("security")
                        .args([
                            "find-generic-password",
                            "-s",
                            "Claude Code-credentials",
                            "-w",
                        ])
                        .output()
                        .ok()
                        .filter(|o| o.status.success())
                        .and_then(|o| parse_expires(&String::from_utf8_lossy(&o.stdout)))
                } else {
                    None
                };
                match keychain_expires {
                    Some(exp) if exp < now_ms => checks.push(Check::warn(
                        "claude oauth",
                        "Claude OAuth expired (Keychain) — run: claude → /login",
                    )),
                    Some(_) => checks.push(Check::ok(
                        "claude oauth",
                        "Claude OAuth valid (macOS Keychain)",
                    )),
                    None => checks.push(Check::warn(
                        "claude oauth",
                        "claude.json missing/unreadable — agent CLI will fail",
                    )),
                }
            }
        }
    }

    // 11. Single Telegram MAIN-bot poller — two main pollers mean duplicate
    //     messages / getUpdates 409s; only the service manager should run it.
    //     Agent bots (omega-tg-agent-* / os.omega.tg-agent-*) run the SAME
    //     script but are legitimate separate services (own token, per-project)
    //     — `main_bot_pollers` excludes them platform-aware (/proc environ on
    //     Linux, launchd labels on macOS) so they don't inflate the count into
    //     a false "duplicate pollers" warning. A co-tenant's bot under another
    //     user is also excluded (pgrep -u scopes to this user). fix8-T1: a
    //     headless Mac (SSH/cron — no GUI launchctl domain) cannot see the
    //     agent-bot labels at all, so the exclusion list is unavailable there;
    //     report info and skip the duplicate check instead of counting every
    //     agent bot as a duplicate.
    {
        let uid = current_uid();
        let verdict = if cfg!(target_os = "macos") {
            let exclusion = launchctl_gui_domain_accessible(&uid).then(agent_bot_pids_darwin);
            poller_verdict(&raw_tg_bot_pids(&uid), exclusion.as_ref())
        } else {
            // Linux: the /proc-based exclusion is always available — same
            // main_bot_pollers count as before.
            let count = main_bot_pollers(&uid).len();
            if count > 1 {
                PollerVerdict::Duplicates(count)
            } else {
                PollerVerdict::Single(count)
            }
        };
        checks.push(poller_check(verdict));
    }

    // 12. Provisioning tokens — warn only if the file exists but ALL deploy
    //     tokens are blank (deploys will fail). Absent file → skip silently.
    {
        let omega_dir = config.state_dir.parent().map(|p| p.to_path_buf());
        let svc = omega_dir.map(|d| d.join("provisioning/services.env"));
        if let Some(content) = svc.and_then(|p| std::fs::read_to_string(&p).ok()) {
            let pairs: std::collections::HashMap<String, bool> = content
                .lines()
                .filter_map(parse_export_kv)
                .map(|(k, v)| (k, !v.is_empty()))
                .collect();
            let watched = [
                "VERCEL_TOKEN",
                "CONVEX_TEAM_TOKEN",
                "GITHUB_TOKEN",
                "STRIPE_SECRET_KEY",
            ];
            let set: Vec<&str> = watched
                .iter()
                .copied()
                .filter(|k| *pairs.get(*k).unwrap_or(&false))
                .collect();
            if set.is_empty() {
                checks.push(Check::warn(
                    "provisioning",
                    "provisioning tokens blank — fill via TUI Provisioning to enable deploys",
                ));
            } else {
                checks.push(Check::ok(
                    "provisioning",
                    format!("provisioning: {}", set.join(", ")),
                ));
            }
        }
    }

    // 13. Telegram bot parity: the LIVE bot (~/.omega/telegram-bot/omega-tg-bot.ts,
    //     run by the systemd service) must match the source in the OmegaOS repo. A
    //     concurrent linter/edit can revert one side, silently dropping shipped
    //     features. Warn if they diverge so the operator redeploys.
    {
        let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/home"));
        let live = home.join(".omega/telegram-bot/omega-tg-bot.ts");
        let candidates = [
            home.join("Station/SideBusiness/OmegaOS/telegram-bot/omega-tg-bot.ts"),
            home.join("Station/OmegaOS/telegram-bot/omega-tg-bot.ts"),
            home.join("OmegaOS/telegram-bot/omega-tg-bot.ts"),
            std::path::PathBuf::from("/tmp/omega-build/telegram-bot/omega-tg-bot.ts"),
        ];
        match std::fs::read(&live) {
            Ok(live_bytes) => {
                let repo = candidates.iter().find(|p| p.exists());
                match repo.and_then(|p| std::fs::read(p).ok()) {
                    Some(repo_bytes) if repo_bytes == live_bytes => {
                        checks.push(Check::ok("telegram bot parity", "live bot == repo source"))
                    }
                    Some(_) => checks.push(Check::warn(
                        "telegram bot parity",
                        format!(
                            "live bot DIFFERS from repo — redeploy: cp <repo>/telegram-bot/omega-tg-bot.ts ~/.omega/telegram-bot/ && {}",
                            crate::service::tg_bot_restart_hint()
                        ),
                    )),
                    None => checks.push(Check::ok(
                        "telegram bot parity",
                        "live bot present (repo source not found to compare)",
                    )),
                }
            }
            Err(_) => checks.push(Check::warn(
                "telegram bot parity",
                "live bot missing from ~/.omega/telegram-bot",
            )),
        }
    }

    checks
}

/// Explicit deep Codex authentication check. This performs one live provider
/// request and must never be called by ordinary doctor or the self-heal cron.
pub async fn probe_codex_auth() -> Check {
    if !crate::agents::Agent::Codex.is_available() {
        return Check::warn("codex real auth", "probe skipped: codex is not on PATH");
    }
    if crate::codex_login::diagnostics().active_flow {
        return Check::warn(
            "codex real auth",
            "probe skipped while a device-login flow is recorded",
        );
    }
    let probe = tokio::task::spawn_blocking(crate::codex_login::probe_auth)
        .await
        .unwrap_or(crate::codex_login::AuthProbe::Unknown {
            reason: "auth probe task failed".to_string(),
        });
    codex_auth_probe_check(probe)
}

fn codex_auth_probe_check(probe: crate::codex_login::AuthProbe) -> Check {
    match probe {
        crate::codex_login::AuthProbe::Usable => {
            Check::ok("codex real auth", "explicit AUTH_OK probe succeeded")
        }
        crate::codex_login::AuthProbe::Unauthenticated => Check::fail(
            "codex real auth",
            "credential was rejected (401); run: omega codex-login",
        ),
        crate::codex_login::AuthProbe::QuotaLimited => {
            Check::warn("codex real auth", "provider quota is unavailable")
        }
        crate::codex_login::AuthProbe::Unknown { reason } => Check::warn("codex real auth", reason),
    }
}

/// Minimal `export KEY="value"` / `export KEY=value` parse → (KEY, value).
/// Mirrors provisioning::parse_export; kept local to avoid a cross-module
/// `pub` just for the doctor check.
fn parse_export_kv(line: &str) -> Option<(String, String)> {
    let rest = line.trim().strip_prefix("export ")?;
    let (k, v) = rest.split_once('=')?;
    let key = k.trim().to_string();
    if key.is_empty()
        || !key
            .chars()
            .all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit())
    {
        return None;
    }
    let raw = v.trim();
    let val = match raw.chars().next() {
        Some(quote @ ('"' | '\'')) => {
            let tail = &raw[quote.len_utf8()..];
            let mut escaped = false;
            let mut closing = None;
            for (index, character) in tail.char_indices() {
                if quote == '"' && character == '\\' && !escaped {
                    escaped = true;
                    continue;
                }
                if character == quote && !escaped {
                    closing = Some(index);
                    break;
                }
                escaped = false;
            }
            let end = closing?;
            let remainder = &tail[end + quote.len_utf8()..];
            if !remainder.is_empty()
                && !(remainder.chars().next().is_some_and(char::is_whitespace)
                    && remainder.trim_start().starts_with('#'))
            {
                return None;
            }
            tail[..end].to_string()
        }
        _ => {
            let mut end = raw.len();
            for (index, character) in raw.char_indices() {
                if character.is_whitespace() {
                    let remainder = raw[index..].trim();
                    if !remainder.is_empty() && !remainder.starts_with('#') {
                        return None;
                    }
                    end = index;
                    break;
                }
                if index == 0 && character == '#' {
                    end = 0;
                    break;
                }
            }
            raw[..end].to_string()
        }
    };
    Some((key, val))
}

// ───────── auto-fix (omega doctor --fix / the self-heal daemon) ─────────

fn current_uid() -> String {
    std::process::Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

/// The service-managed main-bot PID (the ONE poller we must keep): systemd
/// MainPID on Linux, the launchd job pid on macOS (fix7-T3 — this healer was
/// systemd-only and fail-safe-skipped on every Mac). 0/None if the service
/// isn't running under its manager.
fn service_main_pid() -> Option<u32> {
    if cfg!(target_os = "macos") {
        // `launchctl print gui/<uid>/os.omega.tg-bot` emits a `pid = <N>`
        // line while the job is running (absent when stopped).
        let target = format!(
            "gui/{}/{}",
            current_uid(),
            crate::service::TG_BOT_LAUNCHD_LABEL
        );
        let out = std::process::Command::new("launchctl")
            .args(["print", &target])
            .output()
            .ok()?;
        if !out.status.success() {
            return None; // LaunchAgent not bootstrapped (or no launchd)
        }
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .find_map(|l| {
                l.trim()
                    .strip_prefix("pid =")
                    .and_then(|v| v.trim().parse::<u32>().ok())
            })
            .filter(|p| *p != 0)
    } else {
        let out = systemctl_user(&["show", "omega-tg-bot.service", "-p", "MainPID", "--value"])?;
        out.trim().parse::<u32>().ok().filter(|p| *p != 0)
    }
}

/// PIDs of legitimate agent bots on macOS, where /proc doesn't exist. Every
/// agent bot is a launchd job labelled `os.omega.tg-agent-<id>` (see
/// spawnAgentBot in omega-tg-bot.ts), so collect their pids from
/// `launchctl list` (columns: PID Status Label; "-" when not running).
/// NOTE a command-line check (`ps -o command=`) can NOT replace this: agent
/// bots run the IDENTICAL `bun … omega-tg-bot.ts` command as the main bot
/// and differ only by environment / launchd label.
fn agent_bot_pids_darwin() -> std::collections::HashSet<u32> {
    let Ok(out) = std::process::Command::new("launchctl").arg("list").output() else {
        return Default::default();
    };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| {
            let mut cols = l.split_whitespace();
            let pid = cols.next()?.parse::<u32>().ok()?;
            let _status = cols.next()?;
            let label = cols.next()?;
            label.starts_with("os.omega.tg-agent-").then_some(pid)
        })
        .collect()
}

/// fix9: whether a command line is a GENUINE `bun … omega-tg-bot.ts`
/// invocation — argv[0] is the `bun` (or `…/bun`) executable AND a later token's
/// path basename is exactly `omega-tg-bot.ts`. Rejects command lines that merely
/// CONTAIN both tokens incidentally — e.g. a claude oracle/worker launcher whose
/// own argv[0] is `bash`/`claude` and that mentions `bun` and `omega-tg-bot.ts`
/// only inside a later argv element (a `--brief` string, $PATH, or quoted docs).
/// The loose `bun.*omega-tg-bot\.ts` pgrep pattern counted those as pollers and
/// `--fix` would have KILLED them (live agent sessions).
///
/// Requiring the bun executable to be argv[0] is the argv boundary expressed on
/// the space-joined cmdline: argv[0] is a path with no internal spaces, so it is
/// exactly the first whitespace token. A launcher's argv[0] is never `bun`, so
/// an embedded bot command — even a bare, undecorated one — can never re-admit
/// the false positive. (The brief recommended "any token == bun"; first-token is
/// strictly safer and still matches every documented case.) Agent bots run the
/// IDENTICAL `bun …/omega-tg-bot.ts` argv and pass here too; they are excluded
/// later via /proc environ (OMEGA_AGENT_BOT=), not by this predicate.
fn cmdline_is_tg_bot(cmdline: &str) -> bool {
    let mut tokens = cmdline.split_whitespace();
    let Some(exe) = tokens.next() else {
        return false;
    };
    if exe != "bun" && !exe.ends_with("/bun") {
        return false;
    }
    tokens.any(|t| t.rsplit('/').next() == Some("omega-tg-bot.ts"))
}

/// Raw `bun … omega-tg-bot.ts` PIDs for this user (pgrep), BEFORE any
/// agent-bot exclusion — both the main bot and agent bots match. See
/// `main_bot_pollers` / `poller_verdict` for the exclusion step.
fn raw_tg_bot_pids(uid: &str) -> Vec<u32> {
    // Tight pattern: require a `bun` executable token (start-of-line or after a
    // path separator) immediately followed by args reaching omega-tg-bot.ts —
    // NOT just `bun` and `omega-tg-bot.ts` anywhere on the line.
    let Ok(out) = std::process::Command::new("pgrep")
        .args(["-u", uid, "-f", r"(^|/)bun .*omega-tg-bot\.ts"])
        .output()
    else {
        return Vec::new();
    };
    String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .filter_map(|s| s.parse::<u32>().ok())
        .filter(|pid| {
            // Belt-and-suspenders (fix9): on Linux confirm each pid's real argv
            // is a genuine bun+script invocation by reading /proc/<pid>/cmdline
            // (NUL-separated argv) — platform pgrep quirks can never re-admit
            // the false positive that `--fix` would KILL. macOS has no /proc, so
            // the tightened pgrep match stands there (agent bots excluded later
            // via launchctl).
            if cfg!(target_os = "macos") {
                return true;
            }
            match std::fs::read(format!("/proc/{}/cmdline", pid)) {
                Ok(raw) => {
                    let cmdline = raw
                        .split(|b| *b == 0)
                        .map(String::from_utf8_lossy)
                        .collect::<Vec<_>>()
                        .join(" ");
                    cmdline_is_tg_bot(&cmdline)
                }
                // pid vanished between pgrep and read — drop it.
                Err(_) => false,
            }
        })
        .collect()
}

/// fix8-T1: whether this process can see the user's GUI launchctl domain.
/// Headless sessions (SSH/cron — no Aqua session) cannot: `launchctl print
/// gui/<uid>` fails there and/or `launchctl list` yields nothing. Without the
/// GUI domain the os.omega.tg-agent-* labels are invisible, so agent bots can
/// NOT be excluded from a poller count — callers must skip the duplicate
/// check rather than count agent bots as duplicates.
fn launchctl_gui_domain_accessible(uid: &str) -> bool {
    let print_ok = std::process::Command::new("launchctl")
        .args(["print", &format!("gui/{}", uid)])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    let list_nonempty = std::process::Command::new("launchctl")
        .arg("list")
        .output()
        .map(|o| o.status.success() && o.stdout.iter().any(|b| !b.is_ascii_whitespace()))
        .unwrap_or(false);
    print_ok && list_nonempty
}

/// Outcome of the duplicate-poller decision (see `poller_verdict`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PollerVerdict {
    /// 0 or 1 main poller — healthy.
    Single(usize),
    /// More than one main poller after agent-bot exclusion — warn.
    Duplicates(usize),
    /// Multiple pids but no exclusion list (headless Mac: GUI launchctl
    /// domain inaccessible) — the extras may all be legitimate agent bots; info.
    Undeterminable,
}

fn poller_check(verdict: PollerVerdict) -> Check {
    match verdict {
        PollerVerdict::Duplicates(count) => Check::warn(
            "telegram poller",
            format!(
                "multiple Telegram pollers ({}) — duplicate messages; keep only {}",
                count,
                crate::service::tg_bot_service_desc()
            ),
        ),
        PollerVerdict::Single(count) => {
            Check::ok("telegram poller", format!("{} poller", count))
        }
        PollerVerdict::Undeterminable => Check::warn(
            "telegram poller",
            "poller ownership undeterminable on this host — no GUI launchctl domain; skipping duplicate check",
        ),
    }
}

/// Pure duplicate-poller decision (fix8-T1) — unit-testable without launchctl.
/// `agent_exclusion` is the set of known-legitimate agent-bot pids, or None
/// when the platform cannot provide one. Without an exclusion list a multi-pid
/// count is undeterminable and must NEVER be reported as duplicates.
fn poller_verdict(
    raw_pids: &[u32],
    agent_exclusion: Option<&std::collections::HashSet<u32>>,
) -> PollerVerdict {
    match agent_exclusion {
        Some(excl) => {
            let count = raw_pids.iter().filter(|p| !excl.contains(p)).count();
            if count > 1 {
                PollerVerdict::Duplicates(count)
            } else {
                PollerVerdict::Single(count)
            }
        }
        None if raw_pids.len() > 1 => PollerVerdict::Undeterminable,
        None => PollerVerdict::Single(raw_pids.len()),
    }
}

/// This user's `bun … omega-tg-bot.ts` PIDs that are NOT agent bots. Agent
/// bots (omega-tg-agent-* / os.omega.tg-agent-*) run the SAME script but are
/// legitimate separate services and must NOT be killed. Platform-aware
/// exclusion (fix7-T3): Linux reads OMEGA_AGENT_BOT= from /proc/<pid>/environ;
/// macOS matches the pid against the os.omega.tg-agent-* launchd jobs.
fn main_bot_pollers(uid: &str) -> Vec<u32> {
    let agent_pids = if cfg!(target_os = "macos") {
        agent_bot_pids_darwin()
    } else {
        Default::default()
    };
    raw_tg_bot_pids(uid)
        .into_iter()
        .filter(|pid| {
            if cfg!(target_os = "macos") {
                return !agent_pids.contains(pid);
            }
            // Skip agent bots: their /proc/<pid>/environ contains OMEGA_AGENT_BOT=.
            let environ = std::fs::read(format!("/proc/{}/environ", pid)).unwrap_or_default();
            !environ
                .windows(b"OMEGA_AGENT_BOT=".len())
                .any(|w| w == b"OMEGA_AGENT_BOT=")
        })
        .collect()
}

/// Kill orphan duplicate pollers of the MAIN bot, keeping the service-managed one.
fn fix_duplicate_pollers() -> Vec<String> {
    let uid = current_uid();
    let pollers = main_bot_pollers(&uid);
    if pollers.len() <= 1 {
        return Vec::new();
    }
    // SAFETY: only kill duplicates when we can positively identify the ONE to
    // keep (the service-managed PID). If the service manager can't tell us, do
    // nothing rather than risk killing the live bot.
    let Some(keep_pid) = service_main_pid() else {
        return vec![format!(
            "duplicate pollers found but the service-managed PID is unknown — skipped (manual: keep only {})",
            crate::service::tg_bot_service_desc()
        )];
    };
    let keep = Some(keep_pid);
    let mut log = Vec::new();
    for pid in pollers {
        if Some(pid) == keep {
            continue; // the canonical, service-managed poller
        }
        let ok = std::process::Command::new("kill")
            .arg(pid.to_string())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if ok {
            log.push(format!(
                "killed duplicate Telegram poller pid {}{}",
                pid,
                keep.map(|k| format!(" (kept service-managed {})", k))
                    .unwrap_or_default()
            ));
        }
    }
    log
}

fn fix_restart_tg_service() -> Vec<String> {
    if crate::service::tg_bot_restart() {
        vec!["restarted omega-tg-bot service".into()]
    } else {
        Vec::new()
    }
}

fn fix_agk_scopes() -> Vec<String> {
    let Some(home) = dirs::home_dir() else {
        return Vec::new();
    };
    match crate::tenancy::provision_all(&home) {
        Ok(report) => vec![format!(
            "provisioned {} AGK workspaces + {} Hermes profiles",
            report.workspaces.len(),
            report.profiles.len()
        )],
        Err(_) => Vec::new(),
    }
}

fn fix_hermes_gateway() -> Vec<String> {
    let Some(home) = dirs::home_dir() else {
        return Vec::new();
    };
    let report = crate::hermes_gateway::inspect(&home);
    if report.telegram_collision || !report.configured() {
        return Vec::new();
    }
    match crate::hermes_gateway::start(&home) {
        Ok(()) => vec!["started hermes messaging gateway".into()],
        Err(_) => Vec::new(),
    }
}

fn fix_refresh_usage() -> Vec<String> {
    let exe = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("omega"));
    let ok = std::process::Command::new(exe)
        .args(["usage", "--check"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if ok {
        vec!["refreshed usage cache (omega usage --check)".into()]
    } else {
        Vec::new()
    }
}

fn fix_refresh_oauth() -> Vec<String> {
    let Some(home) = dirs::home_dir() else {
        return Vec::new();
    };
    let script = home.join(".omega/bin/omega-token-refresh.sh");
    if !script.exists() {
        return Vec::new();
    }
    let ok = std::process::Command::new("bash")
        .arg(&script)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if ok {
        vec!["ran omega-token-refresh.sh (oauth)".into()]
    } else {
        Vec::new()
    }
}

/// Apply safe, mechanical fixes for the warnings/fails we know how to resolve
/// (duplicate pollers, dead Telegram service, stale usage cache, expired oauth).
/// Returns one log line per action taken. Never panics; anything it can't fix is
/// left for an oracle (the /status "Fix it" button) or the operator.
pub fn auto_fix(checks: &[Check]) -> Vec<String> {
    let mut log = Vec::new();
    for c in checks.iter().filter(|c| c.health != Health::Ok) {
        match c.name.as_str() {
            "telegram poller" => log.extend(fix_duplicate_pollers()),
            "telegram service" => log.extend(fix_restart_tg_service()),
            "hermes gateway" => log.extend(fix_hermes_gateway()),
            "agk scopes" => log.extend(fix_agk_scopes()),
            "usage cache" => log.extend(fix_refresh_usage()),
            "claude oauth" => log.extend(fix_refresh_oauth()),
            _ => {}
        }
    }
    log
}

/// Worst health across all checks — drives the process exit code.
pub fn overall(checks: &[Check]) -> Health {
    if checks.iter().any(|c| c.health == Health::Fail) {
        Health::Fail
    } else if checks.iter().any(|c| c.health == Health::Warn) {
        Health::Warn
    } else {
        Health::Ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_current_provider_version_output_shapes() {
        assert_eq!(
            parse_cli_version("codex-cli 0.149.1"),
            semver::Version::parse("0.149.1").ok()
        );
        assert_eq!(
            parse_cli_version("Hermes Agent v0.20.5 (2026.8.19)"),
            semver::Version::parse("0.20.5").ok()
        );
        assert_eq!(
            parse_cli_version("2.1.241 (Claude Code)"),
            semver::Version::parse("2.1.241").ok()
        );
    }

    #[test]
    fn agents_override_detection_is_layered_and_read_only() {
        let tmp = tempfile::tempdir().unwrap();
        let codex_home = tmp.path().join("codex-home");
        let project = tmp.path().join("workspace/project/src");
        std::fs::create_dir_all(&codex_home).unwrap();
        std::fs::create_dir_all(&project).unwrap();
        let global = codex_home.join("AGENTS.override.md");
        let local = tmp.path().join("workspace/project/AGENTS.override.md");
        std::fs::write(&global, "global").unwrap();
        std::fs::write(&local, "local").unwrap();

        let before_global = std::fs::read(&global).unwrap();
        let before_local = std::fs::read(&local).unwrap();
        let found = agents_override_files(&project, &codex_home);
        assert_eq!(found.len(), 2);
        assert!(found.contains(&global));
        assert!(found.contains(&local));
        assert_eq!(std::fs::read(global).unwrap(), before_global);
        assert_eq!(std::fs::read(local).unwrap(), before_local);
    }

    #[test]
    fn containment_reports_safe_codex_defaults_and_explicit_claude_risk() {
        let mut config = OmegaConfig {
            agent_command: "codex".to_string(),
            ..Default::default()
        };
        let providers = crate::providers::ProvidersConfig::default();
        let codex = effective_containment(&config, &providers);
        assert_eq!(codex.health, Health::Ok);
        assert!(codex.detail.contains("workspace-write"));

        config.agent_command = "claude".to_string();
        let mut risky = providers;
        risky.claude.dangerously_skip_permissions = true;
        let claude = effective_containment(&config, &risky);
        assert_eq!(claude.health, Health::Warn);
        assert!(claude.detail.contains("HIGH-RISK"));

        config.agent_command = "kimi".to_string();
        let kimi = effective_containment(&config, &risky);
        assert_eq!(kimi.health, Health::Warn);
        assert!(kimi.detail.contains("no separate filesystem sandbox"));

        config.agent_command = "codxe".to_string();
        let unknown = effective_containment(&config, &risky);
        assert_eq!(unknown.health, Health::Fail);
        assert!(unknown.detail.contains("runtime launch is blocked"));
    }

    // fix8-T1: the pure decision behind doctor check #11 (telegram poller).
    #[test]
    fn poller_verdict_excludes_agent_bots() {
        let excl: std::collections::HashSet<u32> = [20, 30].into_iter().collect();
        assert_eq!(
            poller_verdict(&[10, 20, 30], Some(&excl)),
            PollerVerdict::Single(1)
        );
        assert_eq!(
            poller_verdict(&[10, 11, 20], Some(&excl)),
            PollerVerdict::Duplicates(2)
        );
        assert_eq!(poller_verdict(&[], Some(&excl)), PollerVerdict::Single(0));
    }

    #[test]
    fn poller_verdict_without_exclusion_never_warns() {
        // Headless Mac: no GUI launchctl domain → no exclusion list. Several
        // pids may all be legitimate agent bots — undeterminable, never
        // Duplicates (the fix8-T1 false-warning bug).
        assert_eq!(
            poller_verdict(&[10, 20, 30], None),
            PollerVerdict::Undeterminable
        );
        assert_eq!(poller_verdict(&[10], None), PollerVerdict::Single(1));
        assert_eq!(poller_verdict(&[], None), PollerVerdict::Single(0));
    }

    #[test]
    fn doctor_never_reports_unknown_poller_ownership_as_healthy() {
        let check = poller_check(PollerVerdict::Undeterminable);
        assert_eq!(check.health, Health::Warn);
    }

    #[test]
    fn rejected_codex_credentials_fail_the_explicit_probe() {
        let check = codex_auth_probe_check(crate::codex_login::AuthProbe::Unauthenticated);
        assert_eq!(check.health, Health::Fail);
        assert!(check.detail.contains("rejected"));
    }

    #[test]
    fn provisioning_parser_rejects_trailing_shell_and_preserves_quoted_values() {
        assert_eq!(
            parse_export_kv("export TOKEN='value with spaces' # comment"),
            Some(("TOKEN".to_string(), "value with spaces".to_string()))
        );
        assert_eq!(
            parse_export_kv("export TOKEN=plain # comment"),
            Some(("TOKEN".to_string(), "plain".to_string()))
        );
        assert_eq!(
            parse_export_kv("export TOKEN=\"\" # placeholder"),
            Some(("TOKEN".to_string(), String::new()))
        );
        assert_eq!(
            parse_export_kv(r#"export TOKEN="value\"still-set" # comment"#),
            Some(("TOKEN".to_string(), r#"value\"still-set"#.to_string()))
        );
        assert_eq!(parse_export_kv("export TOKEN=value rm -rf target"), None);
        assert_eq!(parse_export_kv("export TOKEN='value'#not-comment"), None);
        assert_eq!(parse_export_kv("export lower=value"), None);
    }

    // fix9: the pure predicate behind raw_tg_bot_pids' /proc/<pid>/cmdline
    // filter (doctor check #11 telegram poller). The loose `bun.*omega-tg-bot
    // \.ts` pattern false-matched claude launchers; `--fix` would KILL them.
    #[test]
    fn cmdline_is_tg_bot_rejects_claude_launcher() {
        // `.bun/bin` in $PATH + `omega-tg-bot.ts` inside a --brief string — the
        // documented false positive. argv[0] is `bash`, not the bun exe → false.
        let line = r#"bash -c export PATH="/home/vibe/.bun/bin:/x"; claude --brief "edit telegram-bot/omega-tg-bot.ts""#;
        assert!(!cmdline_is_tg_bot(line));
    }

    #[test]
    fn cmdline_is_tg_bot_accepts_real_bot() {
        // The genuine systemd poller: `…/bun` executable + script basename.
        assert!(cmdline_is_tg_bot(
            "/usr/local/bin/bun /home/vibe/.omega/telegram-bot/omega-tg-bot.ts"
        ));
    }

    #[test]
    fn cmdline_is_tg_bot_accepts_agent_bot() {
        // Agent bots run the IDENTICAL bun+script command — true here; they are
        // excluded later by /proc environ (OMEGA_AGENT_BOT), not this predicate.
        assert!(cmdline_is_tg_bot(
            "bun /home/vibe/.omega/telegram-bot/omega-tg-bot.ts --agent tg-agent-7"
        ));
    }

    #[test]
    fn cmdline_is_tg_bot_rejects_embedded_bare_bot_command() {
        // Robustness: a launcher (argv[0]=bash) whose --brief embeds the BARE,
        // undecorated real bot command as a substring. An "any token == bun"
        // check would false-match (and --fix would KILL this claude session);
        // requiring argv[0] to be the bun executable rejects it.
        assert!(!cmdline_is_tg_bot(
            "bash -c claude --brief run /usr/local/bin/bun /home/vibe/.omega/telegram-bot/omega-tg-bot.ts"
        ));
    }

    #[test]
    fn overall_is_worst_of() {
        let ok = vec![Check::ok("a", "x")];
        assert_eq!(overall(&ok), Health::Ok);
        let warn = vec![Check::ok("a", "x"), Check::warn("b", "y")];
        assert_eq!(overall(&warn), Health::Warn);
        let fail = vec![Check::warn("b", "y"), Check::fail("c", "z")];
        assert_eq!(overall(&fail), Health::Fail);
    }
}

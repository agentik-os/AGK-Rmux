//! Headless contract for an **external** orchestrator (Grok Bot).
//!
//! Grok Bot is an external orchestrator. Atlas/Telegram is optional. One
//! oracle per project. Review is outside Omega.
//!
//! Loop Grok actually runs:
//! 1. Observe: `oracles`, `workers`, `status --json`, `progress` (read-back).
//! 2. `omega dispatch <PROJECT> "<MISSION>"` (default Codex). Never `--agent hermes`.
//! 3. The oracle plans/verifies and calls `omega spawn-worker` (claude|codex|glm).
//! 4. Reap finish reports. Writer `omega done` is a candidate, not a verdict.
//! 5. Kill / close the mission. Gareth alone may `omega gate --accept`.
//!
//! `omega send` / `omega attach` into a provider setup wizard is forbidden.
//! Dispatch and orchestrate must refuse a launch that *is* a login/wizard
//! rather than starting one and hoping Grok types through it.

use crate::agents::Agent;
use anyhow::Result;

pub fn hermes_is_home_error() -> serde_json::Value {
    serde_json::json!({
        "error": "hermes_is_home",
        "message": "Hermes is Home (`omega new --agent hermes`). Do not dispatch --agent hermes. Use claude, codex, or glm."
    })
}

pub fn wizard_refused_error() -> serde_json::Value {
    serde_json::json!({
        "error": "wizard_refused",
        "message": "dispatch/orchestrate must not launch a provider setup wizard. Log in from a Home pane (`omega new --agent …`) or `omega doctor`. Grok must not type into wizards."
    })
}

/// Explicit `--agent hermes` is refused. A configured Home Hermes falls
/// through to Codex so `omega dispatch` / `omega orchestrate` stay writers.
pub fn resolve_mission_writer(explicit: Option<&str>, configured: &str) -> Result<Agent> {
    if let Some(name) = explicit {
        if name.eq_ignore_ascii_case("hermes") {
            anyhow::bail!("{}", hermes_is_home_error());
        }
        let agent = Agent::from_name(name).ok_or_else(|| {
            anyhow::anyhow!(
                "unknown agent '{}' — expected one of: claude, codex, gemini, pi, glm, kimi, shell",
                name
            )
        })?;
        refuse_hermes_dispatch(agent)?;
        Ok(agent)
    } else {
        let agent = Agent::from_name(configured).ok_or_else(|| {
            anyhow::anyhow!(
                "configured agent `{configured}` is unknown; refusing to dispatch on an implicit provider"
            )
        })?;
        if matches!(agent, Agent::Hermes) {
            return Ok(Agent::Codex);
        }
        Ok(agent)
    }
}

pub fn refuse_hermes_dispatch(agent: Agent) -> Result<()> {
    if matches!(agent, Agent::Hermes) {
        anyhow::bail!("{}", hermes_is_home_error());
    }
    Ok(())
}

/// True when the pane command is a login / device-auth / `/login` wizard
/// rather than the agent TUI. Dispatch must never start these.
pub fn command_starts_provider_wizard(command: &str) -> bool {
    let c = command.to_ascii_lowercase();
    c.contains(" login")
        || c.contains("\tlogin")
        || c.contains("auth login")
        || c.contains("device-auth")
        || c.contains("/login")
        || c.contains(" --login")
}

pub fn refuse_wizard_launch(command: &str) -> Result<()> {
    if command_starts_provider_wizard(command) {
        anyhow::bail!("{}", wizard_refused_error());
    }
    Ok(())
}

/// Writer launch used by dispatch/orchestrate: Hermes refused, no login argv.
pub fn headless_writer_launch(agent: Agent, prompt: Option<&str>) -> Result<String> {
    refuse_hermes_dispatch(agent)?;
    let launch = agent.try_launch(prompt)?;
    refuse_wizard_launch(launch.command())?;
    Ok(launch.command().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hermes_dispatch_is_refused() {
        let err = resolve_mission_writer(Some("hermes"), "codex").unwrap_err();
        let text = err.to_string();
        assert!(text.contains("hermes_is_home"), "{text}");
        assert!(text.contains("omega new --agent hermes"), "{text}");
        assert!(refuse_hermes_dispatch(Agent::Hermes).is_err());
        assert!(headless_writer_launch(Agent::Hermes, None).is_err());
    }

    #[test]
    fn configured_hermes_defaults_to_codex_writer() {
        assert_eq!(
            resolve_mission_writer(None, "hermes").unwrap(),
            Agent::Codex
        );
        assert_eq!(resolve_mission_writer(None, "codex").unwrap(), Agent::Codex);
    }

    #[test]
    fn dispatch_launch_never_starts_a_provider_wizard() {
        for agent in [Agent::Claude, Agent::Codex, Agent::Glm] {
            let cmd = headless_writer_launch(agent, Some("run the lab")).unwrap();
            assert!(
                !command_starts_provider_wizard(&cmd),
                "{} launch must not be a login wizard: {cmd}",
                agent.name()
            );
            assert!(
                !cmd.to_ascii_lowercase().contains("hermes"),
                "writer launch must not invoke Hermes: {cmd}"
            );
        }
        assert!(command_starts_provider_wizard("codex login --device-auth"));
        assert!(command_starts_provider_wizard("claude auth login"));
        assert!(command_starts_provider_wizard("hermes /login"));
        let err = refuse_wizard_launch("codex login --device-auth").unwrap_err();
        assert!(err.to_string().contains("wizard_refused"), "{}", err);
    }
}

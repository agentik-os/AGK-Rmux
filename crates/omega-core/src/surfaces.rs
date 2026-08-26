//! Control surfaces — Discord / Desktop / Web / TUI over one backend.
//!
//! BOT ≠ AGENT. Four Discord identities = four logical profiles.
//! Component IDs carry opaque refs, never secrets or filesystem paths.

use anyhow::{bail, Result};

use crate::canon::{validate_object_id, CanonicalRef};
use crate::tenancy::AgkScope;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceKind {
    Discord,
    Desktop,
    Web,
    Tui,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscordEnv {
    Production,
    Dev,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiscordBot {
    pub mention: &'static str,
    pub profile: AgkScope,
    pub purpose: &'static str,
}

pub fn discord_bots(env: DiscordEnv) -> [DiscordBot; 4] {
    match env {
        DiscordEnv::Production => [
            DiscordBot {
                mention: "@Operator",
                profile: AgkScope::Operator,
                purpose: "Operate the Machine",
            },
            DiscordBot {
                mention: "@Agentik",
                profile: AgkScope::Agentik,
                purpose: "Operate My Organization",
            },
            DiscordBot {
                mention: "@Mission",
                profile: AgkScope::Mission,
                purpose: "Operate Other Organizations",
            },
            DiscordBot {
                mention: "@Private",
                profile: AgkScope::Private,
                purpose: "Operate Myself",
            },
        ],
        DiscordEnv::Dev => [
            DiscordBot {
                mention: "@Operator Dev",
                profile: AgkScope::Operator,
                purpose: "Operate the Machine",
            },
            DiscordBot {
                mention: "@Agentik Dev",
                profile: AgkScope::Agentik,
                purpose: "Operate My Organization",
            },
            DiscordBot {
                mention: "@Mission Dev",
                profile: AgkScope::Mission,
                purpose: "Operate Other Organizations",
            },
            DiscordBot {
                mention: "@Private Dev",
                profile: AgkScope::Private,
                purpose: "Operate Myself",
            },
        ],
    }
}

pub fn bot_for_profile(profile: AgkScope, env: DiscordEnv) -> DiscordBot {
    discord_bots(env)
        .into_iter()
        .find(|b| b.profile == profile)
        .expect("four bots cover four profiles")
}

/// Natural language, slash command, Discord button, desktop click, web click
/// all resolve to the same canonical action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceAction {
    pub surface: SurfaceKind,
    pub action: String,
    pub r#ref: CanonicalRef,
}

/// Opaque component id: `agk1.<action>.<profile>.<optional-ids>`
/// No secrets, no paths, no tokens.
pub fn parse_component_id(raw: &str) -> Result<SurfaceAction> {
    let id = raw.trim();
    if id.len() > 100 {
        bail!("component id too long");
    }
    let lower = id.to_ascii_lowercase();
    if lower.contains("token")
        || lower.contains("secret")
        || lower.contains("bearer")
        || id.contains('/')
        || id.contains('\\')
        || id.contains('~')
    {
        bail!("component id must not carry secrets or paths");
    }
    let mut parts = id.split('.');
    let prefix = parts.next().unwrap_or("");
    if prefix != "agk1" {
        bail!("component id must start with agk1");
    }
    let action = parts.next().unwrap_or("").to_string();
    if action.is_empty() {
        bail!("component id missing action");
    }
    let profile_raw = parts.next().unwrap_or("");
    let profile = crate::canon::parse_profile_id(profile_raw)?;
    let mut r#ref = CanonicalRef {
        profile_id: profile,
        client_id: None,
        project_id: None,
        session_id: None,
        mission_id: None,
        task_id: None,
        run_id: None,
        agent_id: None,
        agent_run_id: None,
        runtime_id: None,
        artifact_id: None,
    };
    if let Some(client) = parts.next() {
        if !client.is_empty() {
            validate_object_id("client", client)?;
            r#ref.client_id = Some(client.to_string());
        }
    }
    if let Some(session) = parts.next() {
        if !session.is_empty() {
            validate_object_id("session", session)?;
            r#ref.session_id = Some(session.to_string());
        }
    }
    Ok(SurfaceAction {
        surface: SurfaceKind::Discord,
        action,
        r#ref,
    })
}

pub fn component_id(action: &str, r#ref: &CanonicalRef) -> String {
    let mut out = format!("agk1.{}.{}", action, r#ref.profile_id.id());
    if let Some(client) = &r#ref.client_id {
        out.push('.');
        out.push_str(client);
    }
    if let Some(session) = &r#ref.session_id {
        out.push('.');
        out.push_str(session);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_bots_are_profiles_not_agents() {
        let bots = discord_bots(DiscordEnv::Production);
        assert_eq!(bots.len(), 4);
        assert_eq!(
            bot_for_profile(AgkScope::Mission, DiscordEnv::Production).mention,
            "@Mission"
        );
        assert_eq!(
            bot_for_profile(AgkScope::Mission, DiscordEnv::Dev).mention,
            "@Mission Dev"
        );
        assert!(parse_component_id("agk1.open.mission.client_moonbase.ses_1").is_ok());
        assert!(parse_component_id("agk1.restart.operator./home/operator").is_err());
        assert!(parse_component_id("agk1.x.operator.token_abc").is_err());
    }
}

//! Canonical IDs — never route by Linux username or filesystem path.
//!
//! Paths are derived from topology. Domain objects keep stable IDs across
//! single-user ↔ multi-user ↔ container migrations.

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::tenancy::AgkScope;

pub const PROFILE_IDS: [&str; 4] = ["operator", "agentik", "mission", "private"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalRef {
    pub profile_id: AgkScope,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mission_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_id: Option<String>,
}

/// A session identity. Linux user is resolved at runtime, never stored as id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalSession {
    pub session_id: String,
    pub profile_id: AgkScope,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mission_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hermes_session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rmux_runtime_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_target: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalProject {
    pub id: String,
    pub profile_id: AgkScope,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    pub slug: String,
}

pub fn is_canonical_profile_id(raw: &str) -> bool {
    PROFILE_IDS.contains(&raw.trim().to_ascii_lowercase().as_str())
}

/// Reject path-shaped or username-shaped stand-ins for an object id.
pub fn validate_object_id(kind: &str, raw: &str) -> Result<()> {
    let id = raw.trim();
    if id.is_empty() {
        bail!("{kind} id is empty");
    }
    if id.contains('/') || id.contains('\\') || id.starts_with('~') {
        bail!("{kind} id must not be a filesystem path");
    }
    if id.contains('\0') || id.contains(' ') {
        bail!("{kind} id must not contain spaces or NUL");
    }
    Ok(())
}

pub fn parse_profile_id(raw: &str) -> Result<AgkScope> {
    if !is_canonical_profile_id(raw) && AgkScope::parse(raw).is_err() {
        bail!("profile_id must be operator|agentik|mission|private — not a Linux username");
    }
    AgkScope::parse(raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_identity_has_no_linux_user_field() {
        let json = serde_json::to_string(&CanonicalSession {
            session_id: "ses_1".into(),
            profile_id: AgkScope::Mission,
            client_id: Some("client_moonbase".into()),
            project_id: Some("proj_ceo".into()),
            mission_id: None,
            provider: Some("hermes".into()),
            hermes_session_id: None,
            rmux_runtime_id: None,
            runtime_target: None,
        })
        .unwrap();
        assert!(!json.contains("linux"));
        assert!(!json.contains("/home/"));
        assert!(validate_object_id("project", "/home/mission/workspace").is_err());
        assert!(validate_object_id("project", "proj_ceo").is_ok());
        assert_eq!(parse_profile_id("mission").unwrap(), AgkScope::Mission);
    }
}

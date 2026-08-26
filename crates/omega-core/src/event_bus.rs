//! Canonical Agentik events. Surfaces subscribe to these — they do not scrape
//! terminals to guess state.

use serde::{Deserialize, Serialize};

use crate::canon::CanonicalRef;
use crate::tenancy::AgkScope;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentikEventKind {
    SessionCreated,
    MissionCreated,
    RunStarted,
    AgentStarted,
    AgentWaiting,
    AgentCompleted,
    ArtifactCreated,
    ApprovalRequired,
    RunFailed,
    RunCompleted,
}

impl AgentikEventKind {
    pub fn as_str(self) -> &'static str {
        match self {
            AgentikEventKind::SessionCreated => "session.created",
            AgentikEventKind::MissionCreated => "mission.created",
            AgentikEventKind::RunStarted => "run.started",
            AgentikEventKind::AgentStarted => "agent.started",
            AgentikEventKind::AgentWaiting => "agent.waiting",
            AgentikEventKind::AgentCompleted => "agent.completed",
            AgentikEventKind::ArtifactCreated => "artifact.created",
            AgentikEventKind::ApprovalRequired => "approval.required",
            AgentikEventKind::RunFailed => "run.failed",
            AgentikEventKind::RunCompleted => "run.completed",
        }
    }

    /// Human-meaningful enough to surface on Discord / Desktop.
    pub fn is_surface_worthy(self) -> bool {
        matches!(
            self,
            AgentikEventKind::MissionCreated
                | AgentikEventKind::AgentWaiting
                | AgentikEventKind::ArtifactCreated
                | AgentikEventKind::ApprovalRequired
                | AgentikEventKind::RunFailed
                | AgentikEventKind::RunCompleted
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentikEvent {
    pub kind: AgentikEventKind,
    pub profile_id: AgkScope,
    pub r#ref: CanonicalRef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

pub fn event_name(kind: AgentikEventKind) -> &'static str {
    kind.as_str()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_run_start_is_not_discord_spam() {
        assert!(!AgentikEventKind::RunStarted.is_surface_worthy());
        assert!(AgentikEventKind::ApprovalRequired.is_surface_worthy());
        assert_eq!(AgentikEventKind::RunFailed.as_str(), "run.failed");
    }
}

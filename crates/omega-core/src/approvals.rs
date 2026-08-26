//! Approval engine — Discord / Desktop buttons are not authority.
//! Server-side validation decides whether a run may resume.

use serde::{Deserialize, Serialize};

use crate::canon::CanonicalRef;
use crate::tenancy::AgkScope;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SensitiveAction {
    ProductionDeploy,
    ClientMessage,
    Delete,
    Financial,
    DangerousShell,
    SystemRestart,
}

impl SensitiveAction {
    pub fn requires_approval(self) -> bool {
        true
    }

    pub fn id(self) -> &'static str {
        match self {
            SensitiveAction::ProductionDeploy => "production_deploy",
            SensitiveAction::ClientMessage => "client_message",
            SensitiveAction::Delete => "delete",
            SensitiveAction::Financial => "financial",
            SensitiveAction::DangerousShell => "dangerous_shell",
            SensitiveAction::SystemRestart => "system_restart",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalDecision {
    Approve,
    Reject,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub approval_id: String,
    pub profile_id: AgkScope,
    pub action: SensitiveAction,
    pub r#ref: CanonicalRef,
    pub risk: ApprovalRisk,
    pub summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalRisk {
    Low,
    Medium,
    High,
}

/// Button / slash / desktop click all become this. Discord state is ignored.
pub fn apply_decision(request: &ApprovalRequest, decision: ApprovalDecision) -> ApprovalOutcome {
    ApprovalOutcome {
        approval_id: request.approval_id.clone(),
        decision,
        resume: decision == ApprovalDecision::Approve,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalOutcome {
    pub approval_id: String,
    pub decision: ApprovalDecision,
    pub resume: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canon::CanonicalRef;

    #[test]
    fn discord_button_is_not_authority() {
        let req = ApprovalRequest {
            approval_id: "appr_1".into(),
            profile_id: AgkScope::Operator,
            action: SensitiveAction::SystemRestart,
            r#ref: CanonicalRef {
                profile_id: AgkScope::Operator,
                client_id: None,
                project_id: None,
                session_id: None,
                mission_id: None,
                task_id: None,
                run_id: Some("run_1".into()),
                agent_id: None,
                agent_run_id: None,
                runtime_id: None,
                artifact_id: None,
            },
            risk: ApprovalRisk::High,
            summary: "Restart Hermes gateway".into(),
        };
        assert!(req.action.requires_approval());
        let out = apply_decision(&req, ApprovalDecision::Reject);
        assert!(!out.resume);
    }
}

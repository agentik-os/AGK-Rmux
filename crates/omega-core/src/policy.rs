//! Policy engine — Profile + Client + OS request + Agent permission
//! = effective authority. Topology isolation informs strictness; it is not
//! a domain object.

use serde::{Deserialize, Serialize};

use crate::approvals::SensitiveAction;
use crate::tenancy::AgkScope;
use crate::topology::RuntimeKind;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Authority {
    pub profile: AgkScope,
    pub isolation: RuntimeKind,
    pub can_read_private: bool,
    pub can_read_mission_clients: bool,
    pub operator_privileged: bool,
}

pub fn can_access(actor: AgkScope, target: AgkScope) -> bool {
    if actor == target {
        return true;
    }
    match (actor, target) {
        (AgkScope::Operator, _) => true,
        (_, AgkScope::Operator) => false,
        (AgkScope::Mission, AgkScope::Private) => false,
        (AgkScope::Private, AgkScope::Mission) => false,
        (AgkScope::Private, AgkScope::Agentik) => false,
        (AgkScope::Agentik, AgkScope::Private) => false,
        (AgkScope::Agentik, AgkScope::Mission) => false,
        (AgkScope::Mission, AgkScope::Agentik) => false,
        _ => false,
    }
}

pub fn operator_privileged(actor: AgkScope) -> bool {
    actor == AgkScope::Operator
}

pub fn allows_sensitive(actor: AgkScope, action: SensitiveAction) -> bool {
    match action {
        SensitiveAction::SystemRestart | SensitiveAction::DangerousShell => {
            operator_privileged(actor)
        }
        SensitiveAction::ClientMessage => {
            matches!(actor, AgkScope::Mission | AgkScope::Operator)
        }
        SensitiveAction::Financial => {
            matches!(
                actor,
                AgkScope::Agentik | AgkScope::Private | AgkScope::Operator
            )
        }
        SensitiveAction::ProductionDeploy | SensitiveAction::Delete => {
            actor != AgkScope::Private || matches!(action, SensitiveAction::Delete)
        }
    }
}

pub fn effective_authority(profile: AgkScope, isolation: RuntimeKind) -> Authority {
    Authority {
        profile,
        isolation,
        can_read_private: can_access(profile, AgkScope::Private),
        can_read_mission_clients: can_access(profile, AgkScope::Mission),
        operator_privileged: operator_privileged(profile),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mission_cannot_read_private_or_restart_host() {
        assert!(!can_access(AgkScope::Mission, AgkScope::Private));
        assert!(!allows_sensitive(
            AgkScope::Mission,
            SensitiveAction::SystemRestart
        ));
        assert!(allows_sensitive(
            AgkScope::Mission,
            SensitiveAction::ClientMessage
        ));
        let auth = effective_authority(AgkScope::Mission, RuntimeKind::LinuxUser);
        assert!(!auth.can_read_private);
        assert!(!auth.operator_privileged);
        assert_eq!(auth.isolation, RuntimeKind::LinuxUser);
    }
}

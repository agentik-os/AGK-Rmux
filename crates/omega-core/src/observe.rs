//! Observability snapshot — orchestration must not be opaque.

use serde::{Deserialize, Serialize};

use crate::tenancy::AgkScope;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ObservabilitySnapshot {
    pub active_sessions: u32,
    pub running_agents: u32,
    pub waiting_approvals: u32,
    pub failed_runs: u32,
    pub rmux_processes: u32,
    pub tool_calls: u32,
    pub retries: u32,
    pub errors: u32,
    pub token_usage: u64,
    pub cost_micros: u64,
    pub latency_ms: u64,
    pub profile: Option<AgkScope>,
}

impl ObservabilitySnapshot {
    pub fn empty() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_starts_empty() {
        let snap = ObservabilitySnapshot::empty();
        assert_eq!(snap.running_agents, 0);
        assert_eq!(snap.waiting_approvals, 0);
    }
}

//! OmegaBudget — iteration & token budget tracker per session.
//!
//! Every Oracle and Worker is born with a budget; spending it down is
//! the only way they finish, get killed, or escalate. Inspired by the
//! tracker pattern that runs inside long-running agent loops (rebrand
//! from the upstream `IterationBudget`). Centralized here so:
//!
//! - Dispatch can stamp a budget on each session at spawn time.
//! - Workers refund a turn when the LLM call was a pure tool dispatch
//!   that doesn't progress the goal (mirroring readline/agent loops
//!   that don't count "free" tool plumbing turns).
//! - The Telegram bridge can surface remaining budget in status cards.
//! - Done.json gets a `budget_used` field so SMITH can correlate
//!   over-budget runs with mission complexity over time.
//!
//! ZERO external state — budgets live in memory of the dispatching
//! process. Persistence (across restarts) is the caller's job.

use serde::{Deserialize, Serialize};

/// Default iteration cap for AISB Master / Oracle sessions. They plan,
/// they decompose, they don't grind. Plenty for the longest mission.
pub const PARENT_TURNS_DEFAULT: u32 = 90;

/// Default iteration cap for Workers. They execute one slice and exit.
pub const WORKER_TURNS_DEFAULT: u32 = 50;

/// A consumable budget for a single agent session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OmegaBudget {
    /// Maximum LLM turns this session is allowed before forced stop.
    pub max_turns: u32,
    /// Turns already consumed.
    pub turns_used: u32,
    /// USD ceiling. None = no monetary cap (rare — only for trusted ops).
    pub max_usd: Option<f32>,
    /// USD already spent (best-effort, from API headers).
    pub usd_used: f32,
    /// Number of refunds granted (debug visibility).
    pub refunds: u32,
}

impl OmegaBudget {
    /// Parent (Oracle / AISB Master) defaults: 90 turns, $25 cap.
    pub fn parent() -> Self {
        Self {
            max_turns: PARENT_TURNS_DEFAULT,
            turns_used: 0,
            max_usd: Some(25.0),
            usd_used: 0.0,
            refunds: 0,
        }
    }

    /// Worker defaults: 50 turns, $8 cap.
    pub fn worker() -> Self {
        Self {
            max_turns: WORKER_TURNS_DEFAULT,
            turns_used: 0,
            max_usd: Some(8.0),
            usd_used: 0.0,
            refunds: 0,
        }
    }

    /// Caller-defined budget. Use sparingly — defaults exist for a reason.
    pub fn custom(max_turns: u32, max_usd: Option<f32>) -> Self {
        Self {
            max_turns,
            turns_used: 0,
            max_usd,
            usd_used: 0.0,
            refunds: 0,
        }
    }

    /// Consume one turn. Returns true if the turn was allowed.
    pub fn consume_turn(&mut self) -> bool {
        if self.turns_used >= self.max_turns {
            return false;
        }
        self.turns_used += 1;
        true
    }

    /// Refund a turn — used when the previous LLM call was a tool-only
    /// dispatch that did not progress the goal. Prevents pure plumbing
    /// turns from eating the budget.
    pub fn refund_turn(&mut self) {
        if self.turns_used > 0 {
            self.turns_used -= 1;
            self.refunds += 1;
        }
    }

    /// Record monetary spend. Returns false if it pushes us over `max_usd`.
    pub fn record_usd(&mut self, amount: f32) -> bool {
        self.usd_used += amount.max(0.0);
        match self.max_usd {
            Some(cap) => self.usd_used <= cap,
            None => true,
        }
    }

    /// Are we out of budget on any axis?
    pub fn exhausted(&self) -> bool {
        if self.turns_used >= self.max_turns {
            return true;
        }
        if let Some(cap) = self.max_usd {
            if self.usd_used >= cap {
                return true;
            }
        }
        false
    }

    /// Remaining turns.
    pub fn remaining_turns(&self) -> u32 {
        self.max_turns.saturating_sub(self.turns_used)
    }

    /// Remaining USD (None = no cap).
    pub fn remaining_usd(&self) -> Option<f32> {
        self.max_usd.map(|cap| (cap - self.usd_used).max(0.0))
    }

    /// Compact human-readable status line for Telegram / status cards.
    /// Example: "turns 12/50 · $1.23/$8.00 · refunds 3"
    pub fn status_line(&self) -> String {
        let usd_part = match self.max_usd {
            Some(cap) => format!(" · ${:.2}/${:.2}", self.usd_used, cap),
            None => format!(" · ${:.2}", self.usd_used),
        };
        let refund_part = if self.refunds > 0 {
            format!(" · refunds {}", self.refunds)
        } else {
            String::new()
        };
        format!(
            "turns {}/{}{}{}",
            self.turns_used, self.max_turns, usd_part, refund_part
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_defaults() {
        let b = OmegaBudget::parent();
        assert_eq!(b.max_turns, 90);
        assert_eq!(b.max_usd, Some(25.0));
        assert!(!b.exhausted());
    }

    #[test]
    fn worker_defaults() {
        let b = OmegaBudget::worker();
        assert_eq!(b.max_turns, 50);
        assert_eq!(b.max_usd, Some(8.0));
    }

    #[test]
    fn consume_and_refund() {
        let mut b = OmegaBudget::worker();
        for _ in 0..3 {
            assert!(b.consume_turn());
        }
        assert_eq!(b.turns_used, 3);
        b.refund_turn();
        assert_eq!(b.turns_used, 2);
        assert_eq!(b.refunds, 1);
    }

    #[test]
    fn refund_floor() {
        let mut b = OmegaBudget::worker();
        b.refund_turn();
        b.refund_turn();
        assert_eq!(b.turns_used, 0);
        assert_eq!(b.refunds, 0);
    }

    #[test]
    fn turn_exhaustion() {
        let mut b = OmegaBudget::custom(2, None);
        assert!(b.consume_turn());
        assert!(b.consume_turn());
        assert!(!b.consume_turn());
        assert!(b.exhausted());
    }

    #[test]
    fn usd_exhaustion() {
        let mut b = OmegaBudget::custom(100, Some(1.0));
        assert!(b.record_usd(0.5));
        assert!(b.record_usd(0.49));
        assert!(!b.record_usd(0.02));
        assert!(b.exhausted());
    }

    #[test]
    fn status_line_format() {
        let mut b = OmegaBudget::worker();
        b.consume_turn();
        b.consume_turn();
        let _ = b.record_usd(0.34);
        let s = b.status_line();
        assert!(s.contains("turns 2/50"));
        assert!(s.contains("$0.34/$8.00"));
    }
}

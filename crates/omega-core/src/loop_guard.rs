//! LoopGuard — bounded-retry-then-escalate-to-human + per-mission timeline.
//!
//! The "Loop Engineering" discipline (Addy Osmani, June 2026; popularized as
//! the 2026 builder skill) makes one demand OmegaOS detected but never
//! ENFORCED: a loop must have a verifiable goal, bounded retries, and a hard
//! ceiling that hands control back to a human. The oracle/worker loops here
//! already SPOT trouble — a worker thrashing the same error, a contested
//! fabrication, a mission running far past its expected window — but every
//! such signal was only recorded (`retry_thrash_count` sat at 0, unread) or
//! escalated to the oracle's own inbox, never to the operator. A loop that
//! polices itself with no exit is exactly the "cognitive surrender" the
//! article warns about. This module is the missing exit.
//!
//! Two primitives, both file-backed in the state dir so the PATROL (a process
//! separate from the oracle/worker it watches) can read and act on them:
//!
//! 1. [`escalate_to_human`] — an idempotent operator alert through the
//!    existing `~/.omega/bin/omega-alert-send.sh` funnel, stamped with a
//!    per-reason cooldown so a stuck loop pings once per window, not on every
//!    patrol tick. It also drops a durable [`EscalationRecord`] so reports and
//!    `omega log` can show "⚠ ESCALATED TO HUMAN" long after the alert fired.
//!
//! 2. [`MissionLog`] — an append-only, human-readable timeline of a mission
//!    (phase changes, dispatches, contests, escalations, gate verdicts)
//!    surfaced by `omega log <oracle>`. It mitigates the article's
//!    "comprehension debt": one file tells the whole story instead of five
//!    scattered JSONs an operator has to cross-reference by hand.
//!
//! Policy lives in the constants below so every call-site agrees on the same
//! ceiling. The wall-clock ceiling is deliberately an ALERT, never a kill: an
//! OmegaOS mission that honestly needs 37 hours gets 37 hours (L5) — the guard
//! tells the operator it is still running, it never murders legitimate work.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Re-dispatch / decision-flip ceiling for a single worker before the loop is
/// declared thrashing and the operator is pulled in. Loop Engineering's "cap
/// attempts (~3) then escalate to a human". L1 already says live runtime
/// evidence is mandatory before the 3rd change to the same bug — this is its
/// orchestration-level twin.
pub const THRASH_CAP: u32 = 3;

/// Verify-phase re-runs after a failed quality gate before escalating. The
/// gate is a firewall, not a correction loop — bound the corrections.
pub const GATE_RETRY_CAP: u32 = 3;

/// Mission wall-clock SOFT ceiling: past this with no closeable done signal,
/// alert the operator once (the mission may be legitimately long — this is a
/// heads-up, not a verdict). 6 hours.
pub const SOFT_WALLCLOCK_SECS: i64 = 6 * 3_600;

/// Mission wall-clock HARD ceiling: past this, escalate with a stronger alert.
/// Still NOT a kill (L5 / ultracode: a 37h mission is allowed) — it forces a
/// human to look, which is the whole point of Loop Engineering. 24 hours.
pub const HARD_WALLCLOCK_SECS: i64 = 24 * 3_600;

/// One alert per (mission, reason) per this window. Matches the usage-monitor
/// cooldown grammar (30 min) so escalating loops re-alert but never spam.
const ESCALATION_COOLDOWN_SECS: u64 = 1_800;

/// Why a loop handed control back to the operator. The slug is the cooldown
/// key; the label is what the operator reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscalationReason {
    /// A worker re-thrashed the same error past [`THRASH_CAP`].
    ThrashCap,
    /// The quality gate failed and the verify phase re-ran past [`GATE_RETRY_CAP`].
    GateRetryCap,
    /// A worker's cited artifact was contested as fabricated repeatedly.
    ContestedFabrication,
    /// The mission passed a wall-clock ceiling without a closeable done signal.
    WallClock,
}

impl EscalationReason {
    pub fn slug(self) -> &'static str {
        match self {
            Self::ThrashCap => "thrash-cap",
            Self::GateRetryCap => "gate-retry-cap",
            Self::ContestedFabrication => "contested-fabrication",
            Self::WallClock => "wall-clock",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::ThrashCap => "worker thrashing — retry cap hit",
            Self::GateRetryCap => "quality gate keeps failing — verify cap hit",
            Self::ContestedFabrication => "repeated fabricated artifact",
            Self::WallClock => "mission running past its wall-clock ceiling",
        }
    }
}

/// Durable record that a mission was escalated to the operator. Written
/// alongside the done signal so the PDF report, the done-notifier, and
/// `omega log` can all surface the same "needs a human" state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationRecord {
    pub mission: String,
    pub reason: EscalationReason,
    pub detail: String,
    pub escalated_at: DateTime<Utc>,
}

impl EscalationRecord {
    fn path(state_dir: &Path, mission_key: &str) -> PathBuf {
        state_dir.join(format!("{}.escalation.json", mission_key))
    }

    pub fn read(state_dir: &Path, mission_key: &str) -> Option<Self> {
        let content = std::fs::read_to_string(Self::path(state_dir, mission_key)).ok()?;
        serde_json::from_str(&content).ok()
    }
}

/// Mission-key normalization: callers hold either the full `oracle-<name>`
/// session name or the bare project key. Strip a single `oracle-` prefix so
/// the writer and reader always agree on one filename — same rule as
/// `OracleDoneSignal::oracle_key`, kept local so this module stays standalone.
fn mission_key(name: &str) -> &str {
    name.strip_prefix("oracle-").unwrap_or(name)
}

fn cooldown_active(path: &Path, window_secs: u64) -> bool {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.elapsed().ok())
        .map(|e| e.as_secs() < window_secs)
        .unwrap_or(false)
}

/// Resolve the operator-alert script. `None` (script absent / no home dir) means
/// "can't reach the operator from here" — the caller still logs + records the
/// escalation; only the Telegram ping is skipped.
fn alert_script() -> Option<PathBuf> {
    let p = dirs::home_dir()?.join(".omega/bin/omega-alert-send.sh");
    p.exists().then_some(p)
}

/// Hand control back to the operator. Idempotent within
/// [`ESCALATION_COOLDOWN_SECS`] per (mission, reason): the first call in a
/// window fires the Telegram alert and (re)writes the [`EscalationRecord`];
/// later calls only refresh the record's detail. Returns `true` iff an alert
/// was actually dispatched this call.
///
/// Never returns an error — escalation is best-effort plumbing on a hot path
/// (the patrol loop); a failed alert must not abort the patrol tick.
pub fn escalate_to_human(
    state_dir: &Path,
    mission: &str,
    reason: EscalationReason,
    detail: &str,
) -> bool {
    let key = mission_key(mission);

    // Durable record — always refreshed so reports show the latest detail.
    let record = EscalationRecord {
        mission: key.to_string(),
        reason,
        detail: detail.to_string(),
        escalated_at: Utc::now(),
    };
    if let Ok(json) = serde_json::to_string_pretty(&record) {
        let _ = std::fs::write(EscalationRecord::path(state_dir, key), json);
    }

    // Timeline entry — visible in `omega log` regardless of whether the ping
    // fires.
    MissionLog::event(
        state_dir,
        key,
        "escalation",
        &format!("⚠ ESCALATED TO HUMAN — {} ({})", reason.label(), detail),
    );

    // Cooldown gate on the Telegram ping only.
    let flag = state_dir.join(format!("escalation-{}-{}.sent", key, reason.slug()));
    if cooldown_active(&flag, ESCALATION_COOLDOWN_SECS) {
        return false;
    }
    let _ = std::fs::write(&flag, Utc::now().to_rfc3339());

    if let Some(script) = alert_script() {
        let msg = format!(
            "🛑 <b>LOOP GUARD · {}</b>\nMission <code>{}</code> needs a human.\n<b>Reason:</b> {}\n{}",
            reason.slug(),
            key,
            reason.label(),
            detail,
        );
        let _ = std::process::Command::new("bash")
            .arg(&script)
            .arg(&msg)
            .status();
    }
    true
}

/// One entry in a mission's timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionEvent {
    pub ts: DateTime<Utc>,
    pub kind: String,
    pub msg: String,
}

/// Append-only, human-readable timeline of a single mission. Backed by a
/// `<mission-key>.mission-log.jsonl` file in the state dir — one JSON object
/// per line, so concurrent appends from the oracle, its workers, and the
/// patrol never corrupt each other (each writes a whole line with O_APPEND).
pub struct MissionLog;

impl MissionLog {
    fn path(state_dir: &Path, mission: &str) -> PathBuf {
        state_dir.join(format!("{}.mission-log.jsonl", mission_key(mission)))
    }

    /// Append one event. Best-effort and never panics: a timeline is an
    /// observability aid, never a correctness dependency. Concurrency-safe via
    /// append-mode writes (each call emits exactly one `\n`-terminated line).
    pub fn event(state_dir: &Path, mission: &str, kind: &str, msg: &str) {
        use std::io::Write;
        let ev = MissionEvent {
            ts: Utc::now(),
            kind: kind.to_string(),
            msg: msg.to_string(),
        };
        let Ok(mut line) = serde_json::to_string(&ev) else {
            return;
        };
        line.push('\n');
        let _ = std::fs::create_dir_all(state_dir);
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(Self::path(state_dir, mission))
        {
            let _ = f.write_all(line.as_bytes());
        }
    }

    /// Read the raw events in chronological (append) order.
    pub fn read(state_dir: &Path, mission: &str) -> Vec<MissionEvent> {
        let Ok(content) = std::fs::read_to_string(Self::path(state_dir, mission)) else {
            return Vec::new();
        };
        content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| serde_json::from_str::<MissionEvent>(l).ok())
            .collect()
    }

    /// Render the timeline as a terminal-friendly block for `omega log`.
    pub fn render(state_dir: &Path, mission: &str) -> String {
        let key = mission_key(mission);
        let events = Self::read(state_dir, mission);
        if events.is_empty() {
            return format!("No mission log for `{}` (nothing recorded yet).", key);
        }
        let mut out = format!("━━ Mission timeline · {} ━━\n", key);
        for ev in &events {
            out.push_str(&format!(
                "{}  {:<11} {}\n",
                ev.ts.format("%Y-%m-%d %H:%M:%S"),
                ev.kind,
                ev.msg,
            ));
        }
        if let Some(esc) = EscalationRecord::read(state_dir, key) {
            out.push_str(&format!(
                "\n⚠ ESCALATED TO HUMAN · {} · {} · {}\n",
                esc.reason.slug(),
                esc.escalated_at.format("%Y-%m-%d %H:%M:%S"),
                esc.detail,
            ));
        }
        out
    }

    /// Remove a mission's timeline + escalation record (called when a stale
    /// signal is cleared before a session is recycled under the same name).
    pub fn clear(state_dir: &Path, mission: &str) {
        let key = mission_key(mission);
        let _ = std::fs::remove_file(Self::path(state_dir, key));
        let _ = std::fs::remove_file(EscalationRecord::path(state_dir, key));
        let _ = std::fs::remove_file(state_dir.join(format!("{}.wallclock-soft", key)));
    }
}

/// Side-marker tracking how many times the patrol has re-observed the SAME
/// worker in a thrash/contest state. The patrol is stateless across ticks, so
/// the count lives on disk keyed by worker session. Crossing [`THRASH_CAP`]
/// triggers a one-shot escalation.
pub fn bump_thrash(state_dir: &Path, worker_session: &str) -> u32 {
    let path = state_dir.join(format!("worker-{}.thrash", worker_session));
    let current = std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
        .unwrap_or(0);
    let next = current.saturating_add(1);
    let _ = std::fs::write(&path, next.to_string());
    next
}

/// Reset a worker's thrash counter (it reported a clean, uncontested done).
pub fn clear_thrash(state_dir: &Path, worker_session: &str) {
    let _ = std::fs::remove_file(state_dir.join(format!("worker-{}.thrash", worker_session)));
}

/// Side-marker counting how many times a mission's quality gate has FAILED.
/// The gate is a firewall that runs once per invocation; bounding the
/// re-verifies here turns it into a correction loop with a ceiling. Crossing
/// [`GATE_RETRY_CAP`] escalates. Keyed by mission so it survives across the
/// separate processes that may re-run the gate.
pub fn bump_gate_attempt(state_dir: &Path, mission: &str) -> u32 {
    let path = state_dir.join(format!("{}.gate-attempts", mission_key(mission)));
    let current = std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
        .unwrap_or(0);
    let next = current.saturating_add(1);
    let _ = std::fs::write(&path, next.to_string());
    next
}

/// Reset the gate-attempt counter (the gate passed).
pub fn clear_gate_attempt(state_dir: &Path, mission: &str) {
    let _ = std::fs::remove_file(state_dir.join(format!("{}.gate-attempts", mission_key(mission))));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mission_key_strips_single_oracle_prefix() {
        assert_eq!(mission_key("oracle-OmegaOS-2"), "OmegaOS-2");
        assert_eq!(mission_key("OmegaOS"), "OmegaOS");
    }

    #[test]
    fn mission_log_appends_and_renders_in_order() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        MissionLog::event(dir, "oracle-Proj", "dispatch", "spawned worker auth");
        MissionLog::event(dir, "Proj", "contest", "worker auth contested");
        let events = MissionLog::read(dir, "Proj");
        assert_eq!(events.len(), 2, "both events recorded under the same key");
        assert_eq!(events[0].kind, "dispatch");
        assert_eq!(events[1].kind, "contest");
        let rendered = MissionLog::render(dir, "oracle-Proj");
        assert!(rendered.contains("spawned worker auth"));
        assert!(rendered.contains("Mission timeline · Proj"));
    }

    #[test]
    fn render_empty_is_friendly() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(MissionLog::render(tmp.path(), "Nope").contains("No mission log"));
    }

    #[test]
    fn escalation_is_idempotent_within_cooldown() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        // First call fires (no script present in test env, but the cooldown
        // marker + record are still written and it returns true).
        assert!(escalate_to_human(
            dir,
            "oracle-Proj",
            EscalationReason::ThrashCap,
            "5 retries"
        ));
        // Second call in the same window is suppressed.
        assert!(!escalate_to_human(
            dir,
            "oracle-Proj",
            EscalationReason::ThrashCap,
            "6 retries"
        ));
        // A DIFFERENT reason is a different cooldown key → still fires.
        assert!(escalate_to_human(
            dir,
            "oracle-Proj",
            EscalationReason::WallClock,
            "ran 25h"
        ));
        // The durable record reflects the latest detail for the thrash reason.
        let rec = EscalationRecord::read(dir, "Proj").unwrap();
        assert_eq!(rec.mission, "Proj");
    }

    #[test]
    fn escalation_record_surfaces_in_render() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        escalate_to_human(
            dir,
            "Proj",
            EscalationReason::GateRetryCap,
            "gate failed 3x",
        );
        let rendered = MissionLog::render(dir, "Proj");
        assert!(rendered.contains("ESCALATED TO HUMAN"));
        assert!(rendered.contains("gate-retry-cap"));
    }

    #[test]
    fn thrash_bump_counts_then_clears() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        assert_eq!(bump_thrash(dir, "w1"), 1);
        assert_eq!(bump_thrash(dir, "w1"), 2);
        assert_eq!(bump_thrash(dir, "w1"), 3);
        clear_thrash(dir, "w1");
        assert_eq!(bump_thrash(dir, "w1"), 1, "counter resets after clear");
    }

    #[test]
    fn clear_removes_log_and_record() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        MissionLog::event(dir, "Proj", "phase", "verify");
        escalate_to_human(dir, "Proj", EscalationReason::WallClock, "long");
        MissionLog::clear(dir, "Proj");
        assert!(MissionLog::read(dir, "Proj").is_empty());
        assert!(EscalationRecord::read(dir, "Proj").is_none());
    }
}

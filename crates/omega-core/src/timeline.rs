//! `omega timeline <oracle>` — replay an oracle's full dispatch→done history.
//!
//! Merges three persisted sources into one chronological view: the oracle's
//! own lifecycle (started_at + phase transitions from `OracleState`), every
//! worker it dispatched (`WorkerEntry.dispatched_at` + current status), and
//! each worker's completion (`DoneSignal.finished_at` + summary + commit).
//! Read-only — purely a debugging lens for stuck or finished missions.

use crate::done::{DoneSignal, DoneStatus};
use crate::oracle_lifecycle::OracleState;
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct TimelineEvent {
    pub at: DateTime<Utc>,
    pub marker: &'static str,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct OracleTimeline {
    pub oracle_name: String,
    pub project: String,
    pub mission: String,
    pub phase: String,
    pub events: Vec<TimelineEvent>,
}

fn truncate(s: &str, max: usize) -> String {
    let one_line = s.replace('\n', " ");
    if one_line.chars().count() <= max {
        one_line
    } else {
        format!("{}…", one_line.chars().take(max).collect::<String>())
    }
}

fn done_marker(status: &DoneStatus) -> &'static str {
    match status {
        DoneStatus::DoneClean => "[+]",
        DoneStatus::Pending => "[~]",
        DoneStatus::Failed => "[x]",
        DoneStatus::Blocked => "[!]",
    }
}

/// Build the merged, chronologically-sorted timeline for `oracle_name`, or
/// None if no OracleState exists for it.
pub fn build(state_dir: &Path, oracle_name: &str) -> Result<Option<OracleTimeline>> {
    let state = match OracleState::read(state_dir, oracle_name)? {
        Some(s) => s,
        None => return Ok(None),
    };

    let mut events = Vec::new();

    events.push(TimelineEvent {
        at: state.started_at,
        marker: "◆",
        text: format!("oracle dispatched — {}", truncate(&state.mission_text, 80)),
    });

    for t in &state.phase_history {
        let reason = if t.reason.is_empty() {
            String::new()
        } else {
            format!("  ({})", truncate(&t.reason, 60))
        };
        events.push(TimelineEvent {
            at: t.at,
            marker: "→",
            text: format!("phase {:?} → {:?}{}", t.from, t.to, reason),
        });
    }

    for w in &state.workers {
        events.push(TimelineEvent {
            at: w.dispatched_at,
            marker: "●",
            text: format!("dispatch worker '{}'  [{:?}]", w.task_name, w.status),
        });
        if let Ok(Some(done)) = DoneSignal::read(state_dir, &w.session_name) {
            let commit = done
                .commit
                .as_deref()
                .map(|c| format!("  @{}", &c[..c.len().min(8)]))
                .unwrap_or_default();
            events.push(TimelineEvent {
                at: done.finished_at,
                marker: done_marker(&done.status),
                text: format!(
                    "worker '{}' {:?}{}  — {}",
                    w.task_name,
                    done.status,
                    commit,
                    truncate(&done.summary, 70)
                ),
            });
        }
    }

    events.sort_by_key(|e| e.at);

    Ok(Some(OracleTimeline {
        oracle_name: state.oracle_name,
        project: state.project,
        mission: state.mission_text,
        phase: format!("{:?}", state.phase),
        events,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mission::Mission;
    use crate::oracle_lifecycle::{WorkerEntry, WorkerEntryStatus};
    use std::path::PathBuf;

    #[test]
    fn timeline_merges_and_sorts_oracle_workers_and_done() {
        let dir = std::env::temp_dir().join(format!("omega-timeline-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();

        let t0 = DateTime::<Utc>::from_timestamp(1_700_000_000, 0).unwrap();
        let mission = Mission::new("Acme", "ship the feature", PathBuf::from("/tmp"));
        let mut state = OracleState::new("oracle-Acme-1", &mission);
        state.started_at = t0;
        state.register_worker(WorkerEntry {
            session_name: "Acme-worker-auth".into(),
            task_id: "t1".into(),
            task_name: "auth".into(),
            attempt_id: None,
            plan_revision: None,
            files_owned: vec![],
            dispatched_at: t0 + chrono::Duration::seconds(10),
            status: WorkerEntryStatus::DoneClean,
        });
        state.write(&dir).unwrap();

        // Worker done signal at t0+20s.
        let mut done = DoneSignal::new("Acme-worker-auth", DoneStatus::DoneClean, "wired auth");
        done.finished_at = t0 + chrono::Duration::seconds(20);
        done.write(&dir).unwrap();

        let tl = build(&dir, "oracle-Acme-1").unwrap().unwrap();
        assert_eq!(tl.project, "Acme");
        // dispatched(t0) < worker-dispatch(t0+10) < worker-done(t0+20), sorted.
        assert_eq!(tl.events.len(), 3);
        assert!(tl.events[0].text.starts_with("oracle dispatched"));
        assert!(tl.events[1].text.contains("dispatch worker 'auth'"));
        assert!(tl.events[2].text.contains("wired auth"));
        assert!(tl.events[0].at <= tl.events[1].at && tl.events[1].at <= tl.events[2].at);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn timeline_none_for_unknown_oracle() {
        let dir = std::env::temp_dir().join(format!("omega-timeline-empty-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert!(build(&dir, "oracle-nope").unwrap().is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
}

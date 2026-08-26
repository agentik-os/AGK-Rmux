//! Progress files — `oracle-<key>.progress.json`.
//!
//! ONE schema, the one the producers actually write. `omega progress
//! <session>` (cmd_progress) merge-writes `{tasks: [{t,s}], done, total,
//! ts}` into `oracle-<key>.progress.json`, where `<key>` is the session
//! name minus a single leading `oracle-` (the same normalization as
//! `OracleDoneSignal::oracle_key`; a worker keeps its full name inside
//! the key). The Telegram bot adds `{chat, thread, msgId, bot, project,
//! oracle, mission}` for its progress card. The previous `ProgressInfo`
//! schema (required `session`, `todos_total`/`todos_completed`) had NO
//! producer anywhere, so every consumer — patrol's stall + auto-done
//! passes, the TUI bars, `omega list` — parsed None forever. The serde
//! aliases below read the real files; the Rust-side field names are kept
//! so consumers didn't have to change.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressInfo {
    /// Full session name. On disk the TG bot writes it as `oracle`
    /// (e.g. `"oracle": "oracle-academy-2"`); files written only by
    /// `omega progress` omit it, so `read`/`read_all` backfill it from
    /// the lookup key / filename instead of failing the whole parse
    /// (the old required field was why every read returned None).
    #[serde(default, alias = "oracle")]
    pub session: String,
    /// `total` on disk (cmd_progress counts the plan's tasks).
    #[serde(default, alias = "total")]
    pub todos_total: u32,
    /// `done` on disk (tasks with status "done").
    #[serde(default, alias = "done")]
    pub todos_completed: u32,
    #[serde(default)]
    pub blocked: bool,
    #[serde(default)]
    pub current_task: Option<String>,
    /// `ts` on disk — cmd_progress stamps RFC3339 on every tick.
    #[serde(default, alias = "ts")]
    pub last_updated: Option<DateTime<Utc>>,
}

impl ProgressInfo {
    pub fn percentage(&self) -> f32 {
        if self.todos_total == 0 {
            return 0.0;
        }
        (self.todos_completed as f32 / self.todos_total as f32) * 100.0
    }

    pub fn bar(&self, width: usize) -> String {
        let pct = self.percentage() / 100.0;
        let filled = (pct * width as f32) as usize;
        let empty = width.saturating_sub(filled);
        format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
    }

    pub fn read(state_dir: &Path, session: &str) -> Option<Self> {
        // Match the producer's path rule: `omega progress` ALWAYS writes
        // `oracle-<key>.progress.json` with <key> = session minus ONE
        // leading `oracle-` — for a worker session the key is its full
        // name, so the file is `oracle-<worker>.progress.json`. Reading
        // `{session}.progress.json` verbatim missed every worker file.
        let key = session.strip_prefix("oracle-").unwrap_or(session);
        let path = state_dir.join(format!("oracle-{}.progress.json", key));
        let content = std::fs::read_to_string(&path).ok()?;
        let mut info: Self = serde_json::from_str(&content).ok()?;
        if info.session.is_empty() {
            info.session = session.to_string();
        }
        Some(info)
    }

    pub fn read_all(state_dir: &Path) -> Vec<Self> {
        let mut results = Vec::new();
        if let Ok(entries) = std::fs::read_dir(state_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.ends_with(".progress.json") {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                if let Ok(mut info) = serde_json::from_str::<ProgressInfo>(&content)
                                {
                                    // Best-effort session backfill from the
                                    // filename stem (`oracle-<key>` — exact
                                    // for oracle sessions, the only real
                                    // session-less producers today).
                                    if info.session.is_empty() {
                                        if let Some(stem) = name.strip_suffix(".progress.json") {
                                            info.session = stem.to_string();
                                        }
                                    }
                                    results.push(info);
                                }
                            }
                        }
                    }
                }
            }
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A real on-disk file: TG-bot card fields merged with cmd_progress's
    /// tasks/done/total/ts. The old schema failed this parse entirely
    /// (required `session`), which is why patrol auto-done / TUI bars /
    /// `omega list` progress were dead paths.
    const REAL_MERGED: &str = r#"{
        "chat": 8626440209,
        "thread": 63,
        "msgId": 1234,
        "bot": "agent",
        "project": "academy",
        "oracle": "oracle-academy-2",
        "mission": "fix the login flow",
        "tasks": [{"t": "analyze", "s": "done"}, {"t": "fix", "s": "todo"}],
        "done": 1,
        "total": 2,
        "ts": "2026-06-10T08:00:00Z"
    }"#;

    #[test]
    fn parses_the_real_producer_schema() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("oracle-academy-2.progress.json"),
            REAL_MERGED,
        )
        .unwrap();

        // Full session name (what patrol/TUI pass) resolves the file.
        let info = ProgressInfo::read(tmp.path(), "oracle-academy-2").expect("must parse");
        assert_eq!(info.session, "oracle-academy-2");
        assert_eq!(info.todos_total, 2);
        assert_eq!(info.todos_completed, 1);
        assert!(info.last_updated.is_some());
        assert!((info.percentage() - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn worker_session_resolves_the_oracle_prefixed_file() {
        // cmd_progress writes oracle-<key>.progress.json even for workers
        // (<key> = the full worker name). The reader must follow the same
        // rule or the worker stall/auto-done passes read nothing.
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("oracle-OmegaOS-worker-fix1.progress.json"),
            r#"{"tasks": [{"t": "a", "s": "done"}], "done": 1, "total": 1, "ts": "2026-06-10T08:00:00Z"}"#,
        )
        .unwrap();
        let info = ProgressInfo::read(tmp.path(), "OmegaOS-worker-fix1").expect("must parse");
        // Session backfilled from the lookup key when the file omits it.
        assert_eq!(info.session, "OmegaOS-worker-fix1");
        assert_eq!(info.todos_completed, 1);
        assert_eq!(info.todos_total, 1);
    }

    #[test]
    fn read_all_backfills_session_from_filename() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("oracle-Acme.progress.json"),
            r#"{"done": 0, "total": 3}"#,
        )
        .unwrap();
        let all = ProgressInfo::read_all(tmp.path());
        assert_eq!(all.len(), 1);
        // Filename stem `oracle-Acme` IS the oracle's session name.
        assert_eq!(all[0].session, "oracle-Acme");
        assert_eq!(all[0].todos_total, 3);
    }
}

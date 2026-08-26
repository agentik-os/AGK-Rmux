//! Missions mirror: a READ-ONLY view over oracle progress ledgers
//! (`~/.omega/state/oracle-<key>.progress.json`, written by
//! `omega progress <session> --plan/--task` per R-ORACLE-LEDGER). This
//! module never writes to the ledger dir — it only mirrors what OmegaOS
//! already persisted there.
//!
//! Real ledger shape (confirmed on-box, not guessed):
//! ```json
//! { "oracle":"oracle-dentistrygpt", "project":"dentistrygpt", "mission":"...",
//!   "done":6, "total":6, "ts":"2026-08-08T08:48:20Z",
//!   "tasks":[ {"s":"done","t":"...","updated_at":"..."} ],
//!   "bot":..., "chat":..., "thread":null, "msgId":... }
//! ```
//! `bot`/`chat`/`thread`/`msgId` are Telegram render coordinates and are
//! ignored. There are also `oracle-<key>-worker-<n>.progress.json` files for
//! individual workers; [`list`] excludes those so the mirror shows missions,
//! not workers (a worker's own progress is a future drill-down).

use crate::protocol::{Mission, MissionTask};
use serde::Deserialize;
use std::path::PathBuf;

/// The maximum length, in characters, of a [`Mission::title`] derived from
/// the ledger's free-text `mission` field.
const TITLE_MAX_CHARS: usize = 120;

/// `$OMEGA_STATE_DIR` when set, else `~/.omega/state` — the directory
/// `omega progress` writes `oracle-*.progress.json` ledgers into.
pub fn ledger_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("OMEGA_STATE_DIR") {
        return PathBuf::from(dir);
    }
    dirs::home_dir()
        .expect("no home dir")
        .join(".omega")
        .join("state")
}

#[derive(Deserialize)]
struct LedgerTask {
    #[serde(rename = "s")]
    status: String,
    #[serde(rename = "t")]
    title: String,
}

#[derive(Deserialize)]
struct LedgerFile {
    oracle: String,
    #[serde(default)]
    project: Option<String>,
    #[serde(default)]
    mission: Option<String>,
    #[serde(default)]
    done: u32,
    #[serde(default)]
    total: u32,
    ts: String,
    #[serde(default)]
    tasks: Vec<LedgerTask>,
}

/// First line of `text`, truncated to `max_chars` characters. Empty for
/// empty/whitespace-only input.
fn first_line_truncated(text: &str, max_chars: usize) -> String {
    text.lines()
        .next()
        .unwrap_or("")
        .chars()
        .take(max_chars)
        .collect()
}

/// A filename counts as a top-level oracle ledger when it is
/// `oracle-*.progress.json` and does NOT contain `-worker-` (those are a
/// single worker's own ledger, excluded from the mission mirror).
fn is_top_level_oracle_ledger(name: &str) -> bool {
    name.starts_with("oracle-") && name.ends_with(".progress.json") && !name.contains("-worker-")
}

/// All top-level oracle missions, most recently updated first. Tolerates a
/// missing ledger dir (empty result) and malformed/foreign JSON in any one
/// file (that file is skipped with a `tracing::warn`, never a panic).
pub fn list() -> Vec<Mission> {
    let dir = ledger_dir();
    let mut missions = Vec::new();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return missions;
    };
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_file() {
            continue;
        }
        let Some(name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        if !is_top_level_oracle_ledger(&name) {
            continue;
        }
        let path = entry.path();
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!("failed to read {}: {e}", path.display());
                continue;
            }
        };
        let parsed: LedgerFile = match serde_json::from_str(&text) {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!("skipping malformed ledger {}: {e}", path.display());
                continue;
            }
        };
        let title = {
            let t = first_line_truncated(parsed.mission.as_deref().unwrap_or(""), TITLE_MAX_CHARS);
            if t.is_empty() {
                None
            } else {
                Some(t)
            }
        };
        missions.push(Mission {
            key: parsed.oracle,
            project: parsed.project,
            title,
            done: parsed.done,
            total: parsed.total,
            tasks: parsed
                .tasks
                .into_iter()
                .map(|t| MissionTask {
                    title: t.title,
                    status: t.status,
                })
                .collect(),
            updated_at: parsed.ts,
        });
    }
    missions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    missions
}

#[cfg(test)]
mod tests {
    use super::*;

    // OMEGA_STATE_DIR is process-global; serialize every test that touches it.
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn write_ledger(dir: &std::path::Path, filename: &str, content: &str) {
        std::fs::write(dir.join(filename), content).unwrap();
    }

    #[test]
    fn ledger_dir_env_override() {
        let _g = LOCK.lock().unwrap();
        std::env::set_var("OMEGA_STATE_DIR", "/tmp/omega-state-test-override");
        assert_eq!(
            ledger_dir(),
            PathBuf::from("/tmp/omega-state-test-override")
        );
        std::env::remove_var("OMEGA_STATE_DIR");
    }

    #[test]
    fn list_on_missing_dir_returns_empty() {
        let _g = LOCK.lock().unwrap();
        std::env::set_var(
            "OMEGA_STATE_DIR",
            "/tmp/nonexistent-omega-state-dir-xyz-123",
        );
        assert!(list().is_empty());
        std::env::remove_var("OMEGA_STATE_DIR");
    }

    #[test]
    fn list_parses_real_schema_excludes_workers_and_skips_malformed() {
        let _g = LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("OMEGA_STATE_DIR", dir.path());

        write_ledger(
            dir.path(),
            "oracle-dentistrygpt.progress.json",
            r#"{
                "oracle":"oracle-dentistrygpt","project":"dentistrygpt",
                "mission":"Audit code reset vs addition\nsome other project detail",
                "done":6,"total":6,"ts":"2026-08-08T08:48:20Z",
                "tasks":[
                    {"s":"done","t":"Audit code reset vs addition","updated_at":"a"},
                    {"s":"doing","t":"Ship the migration","updated_at":"b"}
                ],
                "bot":1,"chat":2,"thread":null,"msgId":3
            }"#,
        );
        write_ledger(
            dir.path(),
            "oracle-verba.progress.json",
            r#"{
                "oracle":"oracle-verba","project":"verba","mission":"Ship the thing",
                "done":1,"total":3,"ts":"2026-08-09T00:00:00Z",
                "tasks":[{"s":"todo","t":"Thing","updated_at":"c"}],
                "bot":1,"chat":2,"thread":null,"msgId":3
            }"#,
        );
        // Worker ledger: must be excluded from the mission mirror.
        write_ledger(
            dir.path(),
            "oracle-verba-worker-1.progress.json",
            r#"{
                "oracle":"oracle-verba-worker-1","project":"verba","mission":"worker task",
                "done":1,"total":1,"ts":"2026-08-09T01:00:00Z","tasks":[],
                "bot":1,"chat":2,"thread":null,"msgId":3
            }"#,
        );
        // Malformed: must be skipped, not fatal.
        write_ledger(
            dir.path(),
            "oracle-broken.progress.json",
            "{ not valid json",
        );
        // Foreign JSON (missing required fields entirely): also skipped.
        write_ledger(
            dir.path(),
            "oracle-foreign.progress.json",
            r#"{"unrelated":"shape"}"#,
        );

        let missions = list();
        std::env::remove_var("OMEGA_STATE_DIR");

        assert_eq!(
            missions.len(),
            2,
            "worker + malformed + foreign ledgers must be excluded"
        );
        assert_eq!(
            missions[0].key, "oracle-verba",
            "sorted updated_at desc: 08-09 before 08-08"
        );
        assert_eq!(missions[1].key, "oracle-dentistrygpt");

        let dentistry = &missions[1];
        assert_eq!(dentistry.project.as_deref(), Some("dentistrygpt"));
        assert_eq!(
            dentistry.title.as_deref(),
            Some("Audit code reset vs addition")
        );
        assert_eq!(dentistry.done, 6);
        assert_eq!(dentistry.total, 6);
        assert_eq!(dentistry.updated_at, "2026-08-08T08:48:20Z");
        assert_eq!(dentistry.tasks.len(), 2);
        assert_eq!(dentistry.tasks[0].status, "done");
        assert_eq!(dentistry.tasks[0].title, "Audit code reset vs addition");
        assert_eq!(dentistry.tasks[1].status, "doing");
        assert_eq!(dentistry.tasks[1].title, "Ship the migration");
    }

    #[test]
    fn title_is_first_line_truncated_to_120_chars() {
        let long = "a".repeat(200);
        let text = format!("{long}\nsecond line");
        let truncated = first_line_truncated(&text, TITLE_MAX_CHARS);
        assert_eq!(truncated.chars().count(), TITLE_MAX_CHARS);
        assert!(truncated.chars().all(|c| c == 'a'));
    }

    #[test]
    fn top_level_oracle_ledger_filename_matching() {
        assert!(is_top_level_oracle_ledger(
            "oracle-dentistrygpt.progress.json"
        ));
        assert!(!is_top_level_oracle_ledger(
            "oracle-dentistrygpt-worker-1.progress.json"
        ));
        assert!(!is_top_level_oracle_ledger("something-else.json"));
        assert!(!is_top_level_oracle_ledger("oracle-x.json"));
    }
}

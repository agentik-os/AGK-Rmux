//! OmegaTrace — append-only trajectory recorder.
//!
//! Every dispatched session (Oracle, Worker, even Telegram master-chat
//! turn) writes a ShareGPT-style JSONL line to
//! `~/.omega/state/trajectory/<YYYY-MM-DD>.jsonl`.
//!
//! Why: future tuning loops, post-mortem analysis, SMITH learning, and
//! a record of what AISB actually said vs. what we expected. Cheap to
//! write, immensely valuable when something goes wrong three weeks
//! later and we want to replay.
//!
//! ShareGPT format = one JSON object per line with:
//!   { "conversations": [ { "from": "system|human|gpt|tool", "value": "..." }, ... ] }
//! Compatible with most fine-tuning pipelines (Axolotl, TRL).
//!
//! 30-day retention enforced by a daily prune called from the orchestrator
//! (not built-in to this module — caller's job).

use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

/// A single turn in a trajectory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Turn {
    pub from: TurnRole,
    pub value: String,
    /// Optional structured metadata (tool name, args, error reason).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TurnRole {
    System,
    Human,
    Gpt,
    Tool,
}

/// A complete trajectory for one session / mission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trajectory {
    /// Stable session identifier (oracle name, worker id, telegram chat id).
    pub session: String,
    /// Which project the session was dispatched against, if any.
    pub project: Option<String>,
    /// The model that produced the assistant turns.
    pub model: String,
    /// ISO 8601 timestamp when the trajectory was opened.
    pub started_at: String,
    /// All turns in order.
    pub conversations: Vec<Turn>,
}

impl Trajectory {
    pub fn new(session: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            session: session.into(),
            project: None,
            model: model.into(),
            started_at: chrono::Utc::now().to_rfc3339(),
            conversations: Vec::new(),
        }
    }

    pub fn with_project(mut self, project: impl Into<String>) -> Self {
        self.project = Some(project.into());
        self
    }

    pub fn push(&mut self, role: TurnRole, value: impl Into<String>) {
        self.conversations.push(Turn {
            from: role,
            value: value.into(),
            meta: None,
        });
    }

    pub fn push_with_meta(
        &mut self,
        role: TurnRole,
        value: impl Into<String>,
        meta: serde_json::Value,
    ) {
        self.conversations.push(Turn {
            from: role,
            value: value.into(),
            meta: Some(meta),
        });
    }
}

/// Default trajectory directory: `~/.omega/state/trajectory/`.
pub fn default_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    home.join(".omega/state/trajectory")
}

/// Today's JSONL file path, e.g. `~/.omega/state/trajectory/2026-05-28.jsonl`.
pub fn today_file(base: &std::path::Path) -> PathBuf {
    let date = chrono::Utc::now().format("%Y-%m-%d");
    base.join(format!("{}.jsonl", date))
}

/// Append a trajectory to today's JSONL file.
/// Creates the directory if needed. Best-effort — silently swallows
/// failures so we never crash a hot path on logging.
pub fn append(traj: &Trajectory) -> anyhow::Result<()> {
    let dir = default_dir();
    std::fs::create_dir_all(&dir)?;
    let file = today_file(&dir);
    let mut f = OpenOptions::new().create(true).append(true).open(&file)?;
    let line = serde_json::to_string(traj)?;
    writeln!(f, "{}", line)?;
    Ok(())
}

/// Best-effort fire-and-forget append for hot paths.
pub fn append_silent(traj: &Trajectory) {
    let _ = append(traj);
}

/// Prune trajectory files older than `retain_days`.
pub fn prune(retain_days: i64) -> anyhow::Result<usize> {
    let dir = default_dir();
    if !dir.exists() {
        return Ok(0);
    }
    let cutoff = chrono::Utc::now() - chrono::Duration::days(retain_days);
    let cutoff_date = cutoff.format("%Y-%m-%d").to_string();
    let mut removed = 0;
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_s = name.to_string_lossy();
        if let Some(stem) = name_s.strip_suffix(".jsonl") {
            if stem < cutoff_date.as_str() {
                std::fs::remove_file(entry.path())?;
                removed += 1;
            }
        }
    }
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_trajectory() {
        let mut t = Trajectory::new("oracle-test", "opus-4.7").with_project("OmegaOS");
        t.push(TurnRole::System, "You are AISB.");
        t.push(TurnRole::Human, "Status?");
        t.push(TurnRole::Gpt, "All green.");
        assert_eq!(t.conversations.len(), 3);
        assert_eq!(t.project.as_deref(), Some("OmegaOS"));
    }

    #[test]
    fn today_file_format() {
        let path = today_file(std::path::Path::new("/tmp"));
        let s = path.to_string_lossy();
        assert!(s.starts_with("/tmp/"));
        assert!(s.ends_with(".jsonl"));
        // Format: YYYY-MM-DD.jsonl
        let stem = path.file_stem().unwrap().to_string_lossy().to_string();
        assert_eq!(stem.len(), 10);
        assert_eq!(stem.chars().nth(4), Some('-'));
        assert_eq!(stem.chars().nth(7), Some('-'));
    }

    #[test]
    fn append_and_read_roundtrip() {
        let dir = std::env::temp_dir().join(format!("omega-traj-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = today_file(&dir);
        let mut t = Trajectory::new("test", "opus-4.7");
        t.push(TurnRole::Human, "hi");
        t.push(TurnRole::Gpt, "hello");
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file)
            .unwrap();
        let line = serde_json::to_string(&t).unwrap();
        writeln!(f, "{}", line).unwrap();
        let read = std::fs::read_to_string(&file).unwrap();
        let parsed: Trajectory = serde_json::from_str(read.trim()).unwrap();
        assert_eq!(parsed.session, "test");
        assert_eq!(parsed.conversations.len(), 2);
        std::fs::remove_dir_all(&dir).ok();
    }
}

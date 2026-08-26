//! Launch contract for agent panes: the session *is* the agent.
//!
//! If the agent dies, Omega records `failed` plus a reason. A silent bash
//! prompt must never look like a running Codex/Claude/Hermes session.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::session::sanitize_session_name;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionHealthStatus {
    Running,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionHealth {
    pub session: String,
    pub provider: String,
    #[serde(rename = "status")]
    pub status: SessionHealthStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub launched_at: DateTime<Utc>,
    pub observed_at: DateTime<Utc>,
}

impl SessionHealth {
    pub fn launch(session: &str, provider: &str) -> Self {
        let now = Utc::now();
        Self {
            session: sanitize_session_name(session),
            provider: provider.to_string(),
            status: SessionHealthStatus::Running,
            reason: None,
            launched_at: now,
            observed_at: now,
        }
    }

    pub fn is_failed(&self) -> bool {
        self.status == SessionHealthStatus::Failed
    }
}

pub fn health_path(state_dir: &Path, session: &str) -> Result<std::path::PathBuf> {
    crate::scope::validate_session_identity(session)?;
    Ok(state_dir.join(format!(
        "session-health-{}.json",
        sanitize_session_name(session)
    )))
}

pub fn record_launch(state_dir: &Path, session: &str, provider: &str) -> Result<SessionHealth> {
    let health = SessionHealth::launch(session, provider);
    write(state_dir, &health)?;
    Ok(health)
}

pub fn write(state_dir: &Path, health: &SessionHealth) -> Result<()> {
    let path = health_path(state_dir, &health.session)?;
    let bytes = serde_json::to_vec_pretty(health).context("serializing session health")?;
    crate::config::atomic_write_private(&path, &bytes)
        .with_context(|| format!("writing session health {}", path.display()))
}

pub fn read(state_dir: &Path, session: &str) -> Result<Option<SessionHealth>> {
    let path = health_path(state_dir, session)?;
    let Some(bytes) = crate::config::read_private_optional(&path)? else {
        return Ok(None);
    };
    let health: SessionHealth = serde_json::from_slice(&bytes)
        .with_context(|| format!("parsing session health {}", path.display()))?;
    Ok(Some(health))
}

/// Refresh health from liveness + a captured pane. Failed is sticky.
pub fn observe(
    state_dir: &Path,
    session: &str,
    provider: &str,
    live: bool,
    pane: Option<&str>,
) -> Result<SessionHealth> {
    let (status, reason) = classify_agent_launch(live, pane);
    let mut health = match read(state_dir, session)? {
        Some(existing) => existing,
        None => SessionHealth::launch(session, provider),
    };
    health.observed_at = Utc::now();
    if health.provider.is_empty() {
        health.provider = provider.to_string();
    }
    if health.status != SessionHealthStatus::Failed && status == SessionHealthStatus::Failed {
        health.status = SessionHealthStatus::Failed;
        health.reason = reason;
    }
    write(state_dir, &health)?;
    Ok(health)
}

/// Empty pane is *not* death (Codex/Claude splash / alt-screen).
/// A live bash under a dead agent *is* death.
pub fn classify_agent_launch(
    live: bool,
    pane: Option<&str>,
) -> (SessionHealthStatus, Option<String>) {
    if !live {
        return (
            SessionHealthStatus::Failed,
            Some("agent_exited: session is not live".to_string()),
        );
    }
    if let Some(pane) = pane {
        if pane_fell_to_silent_bash(pane) {
            return (
                SessionHealthStatus::Failed,
                Some("agent_exited: pane fell through to bash".to_string()),
            );
        }
    }
    (SessionHealthStatus::Running, None)
}

/// True when the pane is a shell, not the agent TUI.
pub fn pane_fell_to_silent_bash(pane: &str) -> bool {
    let trimmed = pane.trim();
    if trimmed.is_empty() {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower.contains("unexpected argument")
        && (lower.contains("approve-for-me") || lower.contains("--sandbox"))
    {
        return true;
    }
    if lower.contains("command not found")
        && (lower.contains("codex") || lower.contains("claude") || lower.contains("hermes"))
    {
        return true;
    }
    pane.lines().any(line_looks_like_shell_ps1)
}

fn line_looks_like_shell_ps1(line: &str) -> bool {
    let t = line.trim();
    if t.is_empty() {
        return false;
    }
    if t.starts_with("bash-") && (t.ends_with('$') || t.ends_with("#")) {
        return true;
    }
    // user@host:path$  — leftover after `; exec bash` or a dead `exec` agent.
    let ends_prompt = t.ends_with('$') || t.ends_with('#');
    ends_prompt && t.contains('@') && t.contains(':') && !t.contains('❯') && !t.contains('›')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_or_splash_pane_is_not_a_failed_launch() {
        let (status, _) = classify_agent_launch(true, Some(""));
        assert_eq!(status, SessionHealthStatus::Running);
        let (status, _) = classify_agent_launch(true, Some("   \n  "));
        assert_eq!(status, SessionHealthStatus::Running);
        let (status, _) = classify_agent_launch(true, Some("✦ Codex v0.149.1\n  Thinking…"));
        assert_eq!(status, SessionHealthStatus::Running);
        assert!(!pane_fell_to_silent_bash(""));
    }

    #[test]
    fn silent_bash_after_codex_is_a_failed_launch() {
        let pane = "✦ Codex\n\nbash-5.3$ ";
        assert!(pane_fell_to_silent_bash(pane));
        let (status, reason) = classify_agent_launch(true, Some(pane));
        assert_eq!(status, SessionHealthStatus::Failed);
        assert!(reason.unwrap().contains("bash"));
    }

    #[test]
    fn dead_agent_last_frame_with_host_ps1_is_failed() {
        let pane = "● Analysis complete.\n\
                    \n\
                    ────────────────────────────────────────────────\n\
                    ❯ \n\
                    ────────────────────────────────────────────────\n\
                      ⏵⏵ bypass permissions on (shift+tab to cycle)\n\
                    vibe@Agentik-os:~/Station/SideBusiness/OmegaOS$ \n";
        assert!(pane_fell_to_silent_bash(pane));
        let (status, _) = classify_agent_launch(true, Some(pane));
        assert_eq!(status, SessionHealthStatus::Failed);
    }

    #[test]
    fn dead_session_is_failed_even_without_a_pane() {
        let (status, reason) = classify_agent_launch(false, None);
        assert_eq!(status, SessionHealthStatus::Failed);
        assert!(reason.unwrap().contains("not live"));
    }

    #[test]
    fn failed_health_is_sticky_and_visible_in_json() {
        let tmp = tempfile::TempDir::new().unwrap();
        record_launch(tmp.path(), "t-codex", "codex").unwrap();
        let failed = observe(tmp.path(), "t-codex", "codex", true, Some("bash-5.3$ ")).unwrap();
        assert!(failed.is_failed());
        let revived = observe(tmp.path(), "t-codex", "codex", true, Some("✦ Codex")).unwrap();
        assert!(
            revived.is_failed(),
            "a later live frame must not hide an observed death"
        );
        let json = serde_json::to_value(&revived).unwrap();
        assert_eq!(json["status"], "failed");
        assert!(json["reason"].as_str().unwrap().contains("bash"));
        assert!(json.get("pane").is_none());
    }
}

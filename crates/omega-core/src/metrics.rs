use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    Working,
    Idle,
    Shell,
    Unknown,
}

impl SessionStatus {
    pub fn icon(&self) -> &'static str {
        match self {
            SessionStatus::Working => "●",
            SessionStatus::Idle => "○",
            SessionStatus::Shell => "·",
            SessionStatus::Unknown => "?",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetrics {
    pub session: String,
    pub status: SessionStatus,
    pub claude_count: u32,
    pub subagent_count: u32,
    pub ram_mb: u64,
    pub age: Duration,
    pub git_branch: Option<String>,
    pub cwd: Option<PathBuf>,
}

impl SessionMetrics {
    pub fn detect_from_pane_content(content: &str) -> SessionStatus {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return SessionStatus::Unknown;
        }

        let last_lines: Vec<&str> = trimmed.lines().rev().take(15).collect();
        let last_section = last_lines.join("\n");

        if last_section.contains('❯') {
            return SessionStatus::Idle;
        }

        let working_signals = [
            "Tool call:",
            "Spawning",
            "Running",
            "Executing",
            "esc to interrupt",
            "ctrl+c to stop",
            "✻",
            "Hatching",
            "Cooking",
            "Thinking",
            "Processing",
            "Working",
            "tokens",
        ];

        for sig in &working_signals {
            if last_section.contains(sig) {
                return SessionStatus::Working;
            }
        }

        let shell_signals = ["$ ", "# ", "% ", "@"];
        for sig in &shell_signals {
            if last_lines.first().is_some_and(|l| l.contains(sig)) {
                return SessionStatus::Shell;
            }
        }

        SessionStatus::Idle
    }

    pub fn count_subagents(content: &str) -> u32 {
        let mut count = 0;
        for line in content.lines() {
            if line.contains("subagent") || line.contains("Subagent") {
                count += 1;
            }
            if line.contains("Agent(") {
                count += 1;
            }
        }
        count.min(99)
    }

    pub fn read_ram_mb(pid: u32) -> u64 {
        let status_path = format!("/proc/{}/status", pid);
        if let Ok(content) = std::fs::read_to_string(&status_path) {
            for line in content.lines() {
                if let Some(rest) = line.strip_prefix("VmRSS:") {
                    let kb: u64 = rest
                        .split_whitespace()
                        .next()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                    return kb / 1024;
                }
            }
            return 0;
        }
        // No /proc (macOS) — `ps` reports RSS in KB.
        std::process::Command::new("ps")
            .args(["-o", "rss=", "-p", &pid.to_string()])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .and_then(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .trim()
                    .parse::<u64>()
                    .ok()
            })
            .map(|kb| kb / 1024)
            .unwrap_or(0)
    }

    pub fn detect_git_branch(cwd: &std::path::Path) -> Option<String> {
        let head_path = cwd.join(".git").join("HEAD");
        if !head_path.exists() {
            return None;
        }
        let content = std::fs::read_to_string(&head_path).ok()?;
        let line = content.lines().next()?;
        if let Some(rest) = line.strip_prefix("ref: refs/heads/") {
            return Some(rest.trim().to_string());
        }
        if line.len() >= 8 {
            return Some(line[..8].to_string());
        }
        None
    }

    pub fn format_age(age: Duration) -> String {
        let secs = age.as_secs();
        if secs < 60 {
            format!("{}s", secs)
        } else if secs < 3600 {
            format!("{}m", secs / 60)
        } else if secs < 86400 {
            format!("{}h", secs / 3600)
        } else {
            format!("{}d", secs / 86400)
        }
    }

    pub fn format_ram(ram_mb: u64) -> String {
        if ram_mb == 0 {
            String::new()
        } else if ram_mb < 1024 {
            format!("{}M", ram_mb)
        } else {
            format!("{:.1}G", ram_mb as f64 / 1024.0)
        }
    }
}

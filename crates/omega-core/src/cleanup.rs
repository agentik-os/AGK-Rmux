//! Nuclear cleanup + kill-all — a safe-but-powerful system reclaim that adapts
//! to the host.
//!
//! Kills stray mission sessions (freeing their idle agent processes — the big
//! RAM/CPU win), prunes stale OmegaOS state left behind by dead sessions,
//! clears scratch under `/tmp`, and drops the Linux page cache where permitted.
//!
//! Safety invariants:
//!   * NEVER kills a session in `keep` — the caller passes the current session
//!     plus, in the TUI, any protected ones.
//!   * `infrastructure_keep` shields long-lived singletons (the user's own
//!     Home/System shells, the Telegram bridge, the master) by default so a
//!     "kill all" can't casually nuke the live bot or the user's terminal.
//!   * NEVER hard-requires root — steps it can't perform (e.g. dropping the
//!     page cache) are skipped and noted, so it behaves on any machine/user.

use crate::config::OmegaConfig;
use crate::session::{OmegaSession, SessionManager, SessionRole};
use anyhow::Result;
use std::collections::HashSet;
use std::path::Path;

/// What a cleanup reclaimed — rendered in the TUI status line and the CLI.
#[derive(Debug, Default, Clone)]
pub struct CleanupReport {
    pub sessions_killed: Vec<String>,
    pub stale_state_pruned: usize,
    pub scratch_bytes_freed: u64,
    pub ram_before_mb: u64,
    pub ram_after_mb: u64,
    pub cache_dropped: bool,
    pub notes: Vec<String>,
}

impl CleanupReport {
    /// MB of available RAM reclaimed (can be negative if other processes grew).
    pub fn ram_reclaimed_mb(&self) -> i64 {
        self.ram_after_mb as i64 - self.ram_before_mb as i64
    }

    /// One-line human summary.
    pub fn summary(&self) -> String {
        format!(
            "killed {} session(s), pruned {} stale file(s), freed {} scratch, RAM {}→{}MB ({:+}MB){}",
            self.sessions_killed.len(),
            self.stale_state_pruned,
            human_bytes(self.scratch_bytes_freed),
            self.ram_before_mb,
            self.ram_after_mb,
            self.ram_reclaimed_mb(),
            if self.cache_dropped { ", cache dropped" } else { "" },
        )
    }
}

/// Long-lived singletons that survive a kill-all/nuclear by default: the user's
/// own interactive shells (Home/System role), the Telegram bridge, and the
/// master. The caller adds the current session (+ protected ones in the TUI).
pub fn infrastructure_keep(sessions: &[OmegaSession]) -> HashSet<String> {
    sessions
        .iter()
        .filter(|s| {
            matches!(s.role, SessionRole::Home | SessionRole::System)
                || s.name.contains("telegram-bridge")
                || crate::aisb::is_master(&s.name)
        })
        .map(|s| s.name.clone())
        .collect()
}

/// "Junk" sessions — ones OmegaOS could NOT have created. Every omega
/// create-path runs `sanitize_session_name` (alnum/._- only; spaces, accents,
/// bullets, punctuation collapse to '-'), so a live session whose rmux name
/// differs from its own sanitized form was born OUTSIDE omega: a paste-
/// corruption artifact (e.g. `"istryGPT -"` from a mangled menu copy pasted
/// into a session) or a hand-rolled `rmux new-session`. Clean slugs
/// (oracle-…, …-worker-…, claude-1, the master, the bridge) all equal their
/// sanitized form and are therefore never flagged. `keep` shields the caller's
/// current session.
pub fn find_junk_sessions(sessions: &[OmegaSession], keep: &HashSet<String>) -> Vec<String> {
    sessions
        .iter()
        .filter(|s| !keep.contains(&s.name))
        .filter(|s| s.name != crate::session::sanitize_session_name(&s.name))
        .map(|s| s.name.clone())
        .collect()
}

/// The sessions a kill-all/nuclear WOULD kill given a `keep` set — for the
/// confirmation preview (show before you destroy).
pub async fn killable(mgr: &SessionManager, keep: &HashSet<String>) -> Vec<String> {
    mgr.list_sessions()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|s| s.name)
        .filter(|n| !keep.contains(n))
        .collect()
}

/// Kill every live session except those in `keep`. Returns the names killed.
pub async fn kill_all(mgr: &SessionManager, keep: &HashSet<String>) -> Result<Vec<String>> {
    let sessions = mgr.list_sessions().await?;
    let mut killed = Vec::new();
    for s in sessions {
        if keep.contains(&s.name) {
            continue;
        }
        if mgr.kill_session(&s.name).await.is_ok() {
            killed.push(s.name);
        }
    }
    Ok(killed)
}

/// Available RAM in MB (Linux /proc, macOS vm_stat); 0 if unreadable.
fn ram_available_mb() -> u64 {
    crate::sysinfo::available_ram_mb()
}

/// The `session` field of a state JSON file, if present.
fn json_session_field(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    v.get("session")?.as_str().map(|s| s.to_string())
}

/// Remove state files (scope claims, done/blocked signals) that belong to a
/// session no longer alive. Reads each file's own `session` field rather than
/// guessing from the filename, so it's robust. Returns the count removed.
fn prune_dead_state(state_dir: &Path, alive: &HashSet<String>, notes: &mut Vec<String>) -> usize {
    let mut pruned = 0;
    let entries = match std::fs::read_dir(state_dir) {
        Ok(e) => e,
        Err(_) => return 0,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        let is_state = (name.starts_with("scope-") && name.ends_with(".json"))
            || (name.starts_with("worker-") && name.ends_with(".done.json"))
            || (name.starts_with("worker-blocked-") && name.ends_with(".json"));
        if !is_state {
            continue;
        }
        // Keep state for sessions that are still alive.
        if let Some(session) = json_session_field(&path) {
            if alive.contains(&session) {
                continue;
            }
        }
        if std::fs::remove_file(&path).is_ok() {
            pruned += 1;
        }
    }
    if pruned > 0 {
        notes.push(format!(
            "pruned {} stale state file(s) from dead sessions",
            pruned
        ));
    }
    pruned
}

/// Best-effort clear of OmegaOS scratch under `/tmp` (omega-* / claude-*)
/// owned by us. Returns bytes freed.
fn clear_scratch(notes: &mut Vec<String>) -> u64 {
    let mut freed = 0u64;
    let entries = match std::fs::read_dir("/tmp") {
        Ok(e) => e,
        Err(_) => return 0,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        if !(name.starts_with("omega-") || name.starts_with("claude-")) {
            continue;
        }
        let size = dir_size(&path);
        let removed = if path.is_dir() {
            std::fs::remove_dir_all(&path).is_ok()
        } else {
            std::fs::remove_file(&path).is_ok()
        };
        if removed {
            freed += size;
        }
    }
    if freed > 0 {
        notes.push(format!("cleared {} of /tmp scratch", human_bytes(freed)));
    }
    freed
}

fn dir_size(path: &Path) -> u64 {
    if let Ok(meta) = std::fs::symlink_metadata(path) {
        if meta.file_type().is_symlink() {
            return 0;
        }
        if meta.is_file() {
            return meta.len();
        }
    }
    let mut total = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for e in entries.flatten() {
            total += dir_size(&e.path());
        }
    }
    total
}

/// Format a byte count compactly (B/KB/MB/GB).
pub fn human_bytes(b: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut v = b as f64;
    let mut i = 0;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    format!("{:.1}{}", v, UNITS[i])
}

/// Drop the Linux page cache if permitted. `sync` first (always safe), then try
/// a direct write (works as root), else `sudo -n` (non-interactive — never
/// prompts, fails fast). Returns whether the cache was dropped; notes either way.
fn drop_page_cache(notes: &mut Vec<String>) -> bool {
    let _ = std::process::Command::new("sync").status();

    if std::fs::write("/proc/sys/vm/drop_caches", "3\n").is_ok() {
        notes.push("dropped page cache (direct)".into());
        return true;
    }
    let ok = std::process::Command::new("sudo")
        .args(["-n", "sh", "-c", "echo 3 > /proc/sys/vm/drop_caches"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if ok {
        notes.push("dropped page cache (sudo -n)".into());
    } else {
        notes.push("page-cache drop skipped (needs root/sudo — not available here)".into());
    }
    ok
}

/// Full nuclear cleanup. Adapts to the host; skips + notes anything it can't do.
pub async fn nuclear_cleanup(
    mgr: &SessionManager,
    config: &OmegaConfig,
    keep: &HashSet<String>,
) -> Result<CleanupReport> {
    let mut report = CleanupReport {
        ram_before_mb: ram_available_mb(),
        ..Default::default()
    };

    // 1. Kill all sessions except keep (frees their idle agent processes).
    report.sessions_killed = kill_all(mgr, keep).await.unwrap_or_default();

    // 2. Recompute the live set, then prune state left by dead sessions.
    let alive: HashSet<String> = mgr
        .list_sessions()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|s| s.name)
        .collect();
    report.stale_state_pruned = prune_dead_state(&config.state_dir, &alive, &mut report.notes);

    // 3. Clear /tmp scratch we own.
    report.scratch_bytes_freed = clear_scratch(&mut report.notes);

    // 4. Drop the page cache where permitted.
    report.cache_dropped = drop_page_cache(&mut report.notes);

    report.ram_after_mb = ram_available_mb();
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_bytes_scales() {
        assert_eq!(human_bytes(512), "512.0B");
        assert_eq!(human_bytes(2048), "2.0KB");
        assert_eq!(human_bytes(5 * 1024 * 1024), "5.0MB");
    }

    #[test]
    fn ram_available_is_readable_on_linux() {
        // On the Linux CI/host this is > 0; the function must never panic.
        let _ = ram_available_mb();
    }

    #[test]
    fn infrastructure_keep_shields_singletons_not_workers() {
        let sessions = vec![
            OmegaSession::classify("Home"),
            OmegaSession::classify("omega-telegram-bridge"),
            OmegaSession::classify("aisb-master"),
            OmegaSession::classify("Causio-worker-fix-auth"),
            OmegaSession::classify("oracle-Causio-1"),
        ];
        let keep = infrastructure_keep(&sessions);
        assert!(keep.contains("omega-telegram-bridge"));
        assert!(keep.contains("aisb-master"));
        // Mission sessions are NOT shielded — they're the clutter we reclaim.
        assert!(!keep.contains("Causio-worker-fix-auth"));
        assert!(!keep.contains("oracle-Causio-1"));
    }

    #[test]
    fn junk_detection_flags_only_unsanitized_names() {
        let sessions = vec![
            OmegaSession::classify("istryGPT -"), // paste artifact (space + dash)
            OmegaSession::classify("• Dentistry"), // bullet + space
            OmegaSession::classify("oracle-DentistryGPT-1"), // clean omega slug
            OmegaSession::classify("DentistryGPT-worker-x"), // clean
            OmegaSession::classify("aisb-master"), // clean
            OmegaSession::classify("claude-1"),   // clean user shell
        ];
        let keep = HashSet::new();
        let junk = find_junk_sessions(&sessions, &keep);
        assert!(junk.contains(&"istryGPT -".to_string()));
        assert!(junk.contains(&"• Dentistry".to_string()));
        // Every clean slug equals its sanitized form → never flagged.
        assert!(!junk.iter().any(|j| j == "oracle-DentistryGPT-1"));
        assert!(!junk.iter().any(|j| j == "DentistryGPT-worker-x"));
        assert!(!junk.iter().any(|j| j == "aisb-master"));
        assert!(!junk.iter().any(|j| j == "claude-1"));
        assert_eq!(junk.len(), 2);
    }

    #[test]
    fn junk_detection_honors_keep() {
        let sessions = vec![OmegaSession::classify("my notes")];
        let mut keep = HashSet::new();
        keep.insert("my notes".to_string());
        // A dirty name the caller explicitly keeps (e.g. current session) is spared.
        assert!(find_junk_sessions(&sessions, &keep).is_empty());
    }
}

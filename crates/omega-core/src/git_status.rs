//! Compact branch + unpushed-age summary per session, shown in the TUI's
//! status bar on the Sessions tab as e.g. `↑4h • main`.
//!
//! Mirrors the "ahead-of-remote" indicator the user is used to on the tmux
//! VPS dashboard. `↑Nh` is the time since the OLDEST unpushed commit on
//! the current branch — i.e. how long the working branch has been "drifting"
//! from origin. When everything is pushed, only the branch is shown.

use std::path::PathBuf;

fn pane_cwd(session_name: &str) -> Option<PathBuf> {
    let out = std::process::Command::new("rmux")
        .args([
            "list-panes",
            "-t",
            session_name,
            "-F",
            "#{pane_current_path}",
        ])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let path = String::from_utf8_lossy(&out.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    if path.is_empty() {
        None
    } else {
        Some(PathBuf::from(path))
    }
}

fn git(cwd: &std::path::Path, args: &[&str]) -> Option<String> {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// "↑Nh • branch" when commits are unpushed, "branch" when in sync, None
/// when the cwd is not a git repo. Time unit auto-scales: `↑12m`, `↑4h`,
/// `↑3d`. Falls back gracefully when there's no upstream configured.
pub fn status_for_session(session_name: &str) -> Option<String> {
    let cwd = pane_cwd(session_name)?;
    // Must be inside a worktree.
    git(&cwd, &["rev-parse", "--is-inside-work-tree"])?;
    let branch = git(&cwd, &["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_else(|| "?".into());
    // Oldest unpushed commit's unix timestamp (only commits ahead of upstream).
    // git's --reverse on log applied AFTER limits, so we list all then take the
    // first line — robust for small ahead counts (typical case).
    let oldest = git(
        &cwd,
        &["log", "@{upstream}..HEAD", "--format=%ct", "--reverse"],
    )
    .and_then(|s| s.lines().next().and_then(|l| l.parse::<u64>().ok()));
    let oldest_ts = match oldest {
        Some(ts) => ts,
        None => return Some(branch), // up to date with upstream, or no upstream
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(oldest_ts);
    let secs = now.saturating_sub(oldest_ts);
    let age = fmt_age(secs);
    Some(format!("↑{} • {}", age, branch))
}

fn fmt_age(secs: u64) -> String {
    if secs < 60 {
        return format!("{}s", secs);
    }
    let mins = secs / 60;
    if mins < 60 {
        return format!("{}m", mins);
    }
    let hours = mins / 60;
    if hours < 48 {
        return format!("{}h", hours);
    }
    let days = hours / 24;
    format!("{}d", days)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn age_formatting() {
        assert_eq!(fmt_age(30), "30s");
        assert_eq!(fmt_age(60), "1m");
        assert_eq!(fmt_age(125 * 60), "2h"); // 2h05m → 2h
        assert_eq!(fmt_age(4 * 3600), "4h");
        assert_eq!(fmt_age(48 * 3600), "2d");
        assert_eq!(fmt_age(72 * 3600), "3d");
    }
}

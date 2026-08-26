//! Pull-before-work preflight (R-FICHE / anti-clobber doctrine).
//!
//! Every dispatched agent (oracle at dispatch time, worker at spawn time)
//! must start from the CURRENT origin state — a mission built on a stale
//! checkout silently rebuilds or overwrites work that cloud sessions /
//! other machines already pushed. This module is the single, runtime-enforced
//! version of the "GIT FIRST" doctrine line: fetch, measure the drift, and
//! fast-forward when (and only when) it is provably safe.
//!
//! Safety rules:
//!   * never touches a dirty working tree (a pull there could conflict with
//!     another live session's WIP — see the concurrent-sessions incident);
//!   * `--ff-only`, never a merge/rebase — a diverged branch is surfaced,
//!     not "resolved" silently;
//!   * everything is best-effort with a bounded fetch: a network failure
//!     degrades to a warning, it never blocks a dispatch.

use std::path::Path;
use std::process::Command;

/// Outcome of the preflight, in decreasing order of "all good".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitSyncOutcome {
    /// Not a git repository (scratch dirs, fresh scaffolds) — nothing to do.
    NotARepo,
    /// No upstream tracking branch — nothing to compare against.
    NoUpstream,
    /// Already at (or ahead of) origin.
    UpToDate,
    /// Was N commits behind; fast-forwarded cleanly.
    Pulled(u64),
    /// Behind origin but the tree is dirty — NOT pulled (never clobber WIP).
    BehindDirty(u64),
    /// Behind origin, clean tree, but the ff-only pull failed (diverged
    /// history or a lock) — the divergence needs a human/oracle decision.
    PullFailed(u64),
    /// `git fetch` itself failed (offline, auth) — drift unknown.
    FetchFailed,
    /// A git repo with no `origin` remote (local-only). Not a fetch failure.
    LocalOnly,
}

impl GitSyncOutcome {
    /// One-line, log/prompt-ready description.
    pub fn describe(&self) -> String {
        match self {
            Self::NotARepo => "not a git repo — no sync needed".into(),
            Self::NoUpstream => "no upstream branch — nothing to pull".into(),
            Self::UpToDate => "up to date with origin".into(),
            Self::Pulled(n) => format!("pulled {n} commit(s) from origin (ff-only)"),
            Self::BehindDirty(n) => format!(
                "{n} commit(s) behind origin but the tree is DIRTY — not pulled; \
                 reconcile (stash/commit, then git pull --ff-only) before editing"
            ),
            Self::PullFailed(n) => format!(
                "{n} commit(s) behind origin and ff-only pull FAILED (diverged?) — \
                 reconcile manually before editing"
            ),
            Self::FetchFailed => {
                "git fetch failed — origin drift UNKNOWN; verify before pushing".into()
            }
            Self::LocalOnly => "local-only git repo — no origin remote; skipping fetch".into(),
        }
    }

    /// A warning to surface in the agent's prompt, when the work dir is NOT
    /// known-current. `None` means safe to proceed silently.
    pub fn warning(&self) -> Option<String> {
        match self {
            Self::NotARepo
            | Self::NoUpstream
            | Self::UpToDate
            | Self::Pulled(_)
            | Self::LocalOnly => None,
            other => Some(format!("⚠ GIT SYNC: {}", other.describe())),
        }
    }
}

fn git(dir: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Fetch with a bounded wall clock so a dead network never hangs a dispatch.
/// `timeout(1)` is coreutils — when absent (stock macOS) fall back to a plain
/// fetch (git's own http timeouts still apply eventually).
fn fetch_bounded(dir: &Path) -> bool {
    let bounded = Command::new("timeout")
        .args(["30", "git", "fetch", "--quiet", "origin"])
        .current_dir(dir)
        .output();
    match bounded {
        Ok(o) => o.status.success(),
        Err(_) => Command::new("git")
            .args(["fetch", "--quiet", "origin"])
            .current_dir(dir)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false),
    }
}

/// Run the preflight in `dir`: fetch origin, measure how far behind the
/// current branch is, and fast-forward iff the tree is clean. Pure I/O,
/// never panics, never blocks beyond the bounded fetch.
pub fn pull_preflight(dir: &Path) -> GitSyncOutcome {
    if git(dir, &["rev-parse", "--is-inside-work-tree"]).as_deref() != Some("true") {
        return GitSyncOutcome::NotARepo;
    }
    // Probe origin BEFORE fetch. A local-only repo has no remotes; `git fetch
    // origin` fails and used to paint every `omega dispatch` with
    // "origin drift UNKNOWN" on Mac scratch projects.
    if git(dir, &["remote", "get-url", "origin"]).is_none() {
        return GitSyncOutcome::LocalOnly;
    }
    if !fetch_bounded(dir) {
        return GitSyncOutcome::FetchFailed;
    }
    let behind = match git(dir, &["rev-list", "--count", "HEAD..@{upstream}"]) {
        None => return GitSyncOutcome::NoUpstream,
        Some(s) => s.parse::<u64>().unwrap_or(0),
    };
    if behind == 0 {
        return GitSyncOutcome::UpToDate;
    }
    let dirty = git(dir, &["status", "--porcelain"])
        .map(|s| !s.is_empty())
        .unwrap_or(true);
    if dirty {
        return GitSyncOutcome::BehindDirty(behind);
    }
    let pulled = Command::new("git")
        .args(["pull", "--ff-only", "--quiet"])
        .current_dir(dir)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if pulled {
        GitSyncOutcome::Pulled(behind)
    } else {
        GitSyncOutcome::PullFailed(behind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_repo_is_a_silent_no_op() {
        let tmp = tempfile::tempdir().unwrap();
        let out = pull_preflight(tmp.path());
        assert_eq!(out, GitSyncOutcome::NotARepo);
        assert!(out.warning().is_none());
    }

    #[test]
    fn clean_local_repo_without_upstream_is_silent() {
        let tmp = tempfile::tempdir().unwrap();
        let run = |args: &[&str]| {
            assert!(Command::new("git")
                .args(args)
                .current_dir(tmp.path())
                .output()
                .unwrap()
                .status
                .success());
        };
        run(&["init", "-q"]);
        // No origin remote: local-only, silent, never a blocking warning.
        let out = pull_preflight(tmp.path());
        assert!(matches!(out, GitSyncOutcome::LocalOnly));
        assert!(out.warning().is_none());
    }

    #[test]
    fn dirty_behind_and_pull_failed_warn() {
        assert!(GitSyncOutcome::BehindDirty(3).warning().is_some());
        assert!(GitSyncOutcome::PullFailed(2).warning().is_some());
        assert!(GitSyncOutcome::Pulled(5).warning().is_none());
        assert!(GitSyncOutcome::UpToDate.warning().is_none());
    }
}

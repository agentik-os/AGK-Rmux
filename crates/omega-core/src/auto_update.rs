//! Daily update check + automatic apply — the decision layer.
//!
//! `omega update --auto` runs from cron on every install. This module owns the
//! part worth testing: given the state of the checkout, the machine and the
//! previous runs, WHAT should happen? The CLI does the git and install.sh work.
//!
//! Auto-applying code fetched from a remote is a real trust decision, so the
//! decision is deliberately conservative and every refusal is named:
//!
//! * nothing to pull → do nothing, cheaply and silently (the common case)
//! * local edits or unpushed commits → never touch them, ever
//! * an agent is mid-turn → defer to tomorrow rather than rebuild under it
//! * the same commit failed [`FAILURE_CAP`] times → stop, escalate to a human
//!   (R-LOOP: retrying a 4th time is thrash, not progress)
//!
//! The failure counter is keyed by TARGET COMMIT, not by a bare tally: a new
//! commit on the remote is a genuinely new attempt and starts from zero, while
//! one bad commit can never be retried forever.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::config::AutoUpdatePolicy;

/// Consecutive failed applies of the SAME target commit before the cron stops
/// trying and asks for a human. Mirrors `loop_guard::THRASH_CAP` — same reason.
pub const FAILURE_CAP: u32 = 3;

/// What the caller observed about the checkout before deciding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckoutState {
    /// Commits on origin that we do not have.
    pub behind: usize,
    /// Local commits not on origin.
    pub ahead: usize,
    /// Uncommitted changes present.
    pub dirty: bool,
    /// Short sha of the remote tip — the update target. Empty when unknown.
    pub target: String,
    /// Short sha of the local HEAD. Needed because a fast-forward moves HEAD
    /// BEFORE install.sh runs: if the install then fails, the checkout is
    /// current while the installed binary is not, and `behind` alone would
    /// report the machine as up to date forever.
    pub head: String,
}

/// What the cron decided to do, and why. Every variant carries the sentence
/// that goes into the log, so the reason can never drift from the decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// Policy is `off` — the cron does nothing at all.
    Disabled,
    /// Already current. The overwhelmingly common daily outcome.
    UpToDate,
    /// An update exists and will be installed now.
    Apply { behind: usize, target: String },
    /// An update exists; policy is `check`, so only report it.
    NotifyOnly { behind: usize, target: String },
    /// An update exists but applying it would risk something. Never retried
    /// blindly — the reason says what the user has to resolve.
    Skip { reason: SkipReason },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    /// Uncommitted local changes — an update would have to clobber them.
    DirtyTree,
    /// Local commits not pushed — fast-forward is impossible without losing them.
    LocalCommits { ahead: usize },
    /// An agent session is mid-turn; rebuilding under it can wait a day.
    AgentWorking { session: String },
    /// This exact commit already failed to install `FAILURE_CAP` times.
    RepeatedFailure { target: String, failures: u32 },
}

impl SkipReason {
    /// One line, plain words, for the log and the alert.
    pub fn describe(&self) -> String {
        match self {
            Self::DirtyTree => "the checkout has uncommitted changes — they are never touched, so the update is skipped".to_string(),
            Self::LocalCommits { ahead } => format!(
                "the checkout has {} local commit(s) not on origin — push or rebase them, then it resumes",
                ahead
            ),
            Self::AgentWorking { session } => format!(
                "an agent is working right now ({}) — deferred to the next daily run",
                session
            ),
            Self::RepeatedFailure { target, failures } => format!(
                "commit {} failed to install {} times — not retrying, this needs a human",
                target, failures
            ),
        }
    }

    /// True when the operator has to act before updates can resume. These are
    /// worth an alert; a one-day deferral is not.
    pub fn needs_human(&self) -> bool {
        !matches!(self, Self::AgentWorking { .. })
    }
}

impl Decision {
    pub fn describe(&self) -> String {
        match self {
            Self::Disabled => "auto-update is off (config: auto_update)".to_string(),
            Self::UpToDate => "already up to date".to_string(),
            // behind == 0 here means nothing was pulled, yet an install is
            // still owed: either the fast-forward landed and the install then
            // failed, or the checkout moved on its own (a locally authored
            // commit) and no install ever ran for it. The wording must cover
            // both — claiming a failed install on a machine that simply
            // committed would send the reader hunting for an error that never
            // happened.
            Self::Apply { behind, target } if *behind == 0 => format!(
                "source is at {} but the installed binary is not — installing it",
                target
            ),
            Self::Apply { behind, target } => {
                format!("{} commit(s) behind — installing {}", behind, target)
            }
            Self::NotifyOnly { behind, target } if *behind == 0 => format!(
                "source is at {} but the installed binary is not — check-only policy, not installing",
                target
            ),
            Self::NotifyOnly { behind, target } => format!(
                "{} commit(s) behind ({}) — check-only policy, not installing",
                behind, target
            ),
            Self::Skip { reason } => reason.describe(),
        }
    }
}

/// The persisted memory of the daily cron: what it last saw, what it last
/// installed, and how often the current target has failed.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AutoUpdateState {
    pub last_check: Option<DateTime<Utc>>,
    pub last_applied: Option<DateTime<Utc>>,
    /// Commit installed by the last successful apply.
    pub last_applied_commit: Option<String>,
    /// Target commit the failures below refer to. A different target resets them.
    pub failing_commit: Option<String>,
    pub consecutive_failures: u32,
    /// Human-readable outcome of the last run, for `omega update --check`.
    pub last_outcome: Option<String>,
    /// Fingerprint of the doctrine the last install put on disk.
    ///
    /// Kept so a rebuild can be asked whether the RULES moved, which a commit
    /// sha cannot answer: most commits change no rule at all.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doctrine_fingerprint: Option<String>,
    /// When the fingerprint above last CHANGED — not when it was last written.
    ///
    /// This is the cutoff a session check needs. A session started after this
    /// carries the current doctrine no matter how many binaries have been
    /// installed since, so a doctrine-neutral rebuild stops flagging live work.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doctrine_changed_at: Option<DateTime<Utc>>,
}

impl AutoUpdateState {
    pub fn path(state_dir: &Path) -> PathBuf {
        state_dir.join("auto-update.json")
    }

    /// Read the state, treating any error (missing, truncated by a crash mid-
    /// write, hand-edited) as "no history". A corrupt file must never be the
    /// thing that stops a machine from updating.
    pub fn load(state_dir: &Path) -> Self {
        std::fs::read_to_string(Self::path(state_dir))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, state_dir: &Path) -> Result<()> {
        std::fs::create_dir_all(state_dir)?;
        let path = Self::path(state_dir);
        // Write-then-rename: a cron killed mid-write must not leave a truncated
        // file behind (load() tolerates it, but silently losing the failure
        // count would let a bad commit be retried forever).
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_string_pretty(self)?)?;
        std::fs::rename(&tmp, &path)?;
        Ok(())
    }

    /// How many times `target` has failed to install in a row (0 for a target
    /// we have never failed on).
    pub fn failures_for(&self, target: &str) -> u32 {
        match &self.failing_commit {
            Some(c) if c == target => self.consecutive_failures,
            _ => 0,
        }
    }

    /// Record a failed apply of `target`. A new target restarts the count.
    pub fn record_failure(&mut self, target: &str) {
        if self.failing_commit.as_deref() == Some(target) {
            self.consecutive_failures = self.consecutive_failures.saturating_add(1);
        } else {
            self.failing_commit = Some(target.to_string());
            self.consecutive_failures = 1;
        }
    }

    /// Record a successful apply — clears the failure memory entirely.
    pub fn record_success(&mut self, target: &str, at: DateTime<Utc>) {
        self.last_applied = Some(at);
        self.last_applied_commit = Some(target.to_string());
        self.failing_commit = None;
        self.consecutive_failures = 0;
        self.record_doctrine(&crate::rules::doctrine_fingerprint(), at);
    }

    /// Note the doctrine this install carries, moving `doctrine_changed_at`
    /// ONLY when the fingerprint actually differs.
    ///
    /// Stamping the time on every install would defeat the point: the cutoff
    /// would advance on doctrine-neutral rebuilds and the session check would
    /// flag live work again, which is the false positive this exists to kill.
    ///
    /// A FIRST observation (no fingerprint on record) is not a change. There is
    /// nothing to compare against, and calling it one would flag every session
    /// running when a machine upgrades into this version, once, for no reason.
    pub fn record_doctrine(&mut self, fingerprint: &str, at: DateTime<Utc>) {
        let changed = self
            .doctrine_fingerprint
            .as_deref()
            .is_some_and(|previous| previous != fingerprint);
        self.doctrine_fingerprint = Some(fingerprint.to_string());
        if changed {
            self.doctrine_changed_at = Some(at);
        }
    }
}

/// The whole decision, in one pure function so every branch is testable
/// without a git checkout, a network, or a running agent.
///
/// `working_session` is `Some(name)` when an agent is mid-turn.
pub fn decide(
    policy: AutoUpdatePolicy,
    state: &CheckoutState,
    history: &AutoUpdateState,
    working_session: Option<&str>,
) -> Decision {
    if policy == AutoUpdatePolicy::Off {
        return Decision::Disabled;
    }

    // Nothing to pull is checked BEFORE the refusals: a dirty checkout that is
    // already current is not a problem worth alerting anyone about, and that is
    // the normal state of a developer's own machine.
    //
    // "Nothing to pull" is NOT the same as "nothing to install". The
    // fast-forward moves HEAD before install.sh runs, so an install that fails
    // afterwards leaves the checkout current and the BINARY stale. Reading
    // `behind` alone reported that machine as up to date every night after,
    // and the install was never retried — the failure cap never even engaged
    // because this branch returned first. An install is still owed whenever the
    // commit we last failed on is the one checked out.
    //
    // The SECOND way to owe an install has nothing to do with failure: the
    // checkout can move forward without this cron ever pulling it. A machine
    // that AUTHORS commits — the box OmegaOS is developed on, or anyone who
    // commits locally and pushes — is never `behind`, so `behind == 0` returned
    // UpToDate forever and the binary was only ever refreshed by someone
    // remembering to run install.sh by hand. Measured on the source box on
    // 2026-08-05: `last_applied_commit` still read a commit from five days
    // earlier while HEAD had moved 30+ commits, and `omega update --check`
    // cheerfully reported "up to date" against a demonstrably stale binary.
    //
    // So: HEAD not being the commit we last installed is itself an owed
    // install. `None` is deliberately NOT an owed install — it means no install
    // ever recorded its provenance, and owing one on every run would put a
    // machine with an unwritable state file into a nightly rebuild loop.
    let installed_head_mismatch = !state.head.is_empty()
        && history
            .last_applied_commit
            .as_deref()
            .is_some_and(|installed| installed != state.head.as_str());

    let install_owed = !state.head.is_empty()
        && ((history.failing_commit.as_deref() == Some(state.head.as_str())
            && history.consecutive_failures > 0)
            || installed_head_mismatch);

    if state.behind == 0 && !install_owed {
        return Decision::UpToDate;
    }

    if policy == AutoUpdatePolicy::Check {
        return Decision::NotifyOnly {
            behind: state.behind,
            target: state.target.clone(),
        };
    }

    // Order matters: report the condition the user must FIX before the one they
    // merely have to wait out.
    if state.dirty {
        return Decision::Skip {
            reason: SkipReason::DirtyTree,
        };
    }
    if state.ahead > 0 {
        return Decision::Skip {
            reason: SkipReason::LocalCommits { ahead: state.ahead },
        };
    }
    let failures = history.failures_for(&state.target);
    if failures >= FAILURE_CAP {
        return Decision::Skip {
            reason: SkipReason::RepeatedFailure {
                target: state.target.clone(),
                failures,
            },
        };
    }
    if let Some(session) = working_session {
        return Decision::Skip {
            reason: SkipReason::AgentWorking {
                session: session.to_string(),
            },
        };
    }

    Decision::Apply {
        behind: state.behind,
        target: state.target.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn behind(n: usize) -> CheckoutState {
        CheckoutState {
            behind: n,
            ahead: 0,
            dirty: false,
            target: "abc1234".to_string(),
            // Behind means HEAD is NOT the target yet.
            head: "0000000".to_string(),
        }
    }

    /// The state a machine is in after a fast-forward whose install then
    /// failed: the checkout is current, the installed binary is not.
    fn ff_done_install_failed() -> CheckoutState {
        CheckoutState {
            behind: 0,
            ahead: 0,
            dirty: false,
            target: "abc1234".to_string(),
            head: "abc1234".to_string(),
        }
    }

    #[test]
    fn nothing_to_pull_is_the_quiet_daily_outcome() {
        let d = decide(
            AutoUpdatePolicy::Apply,
            &behind(0),
            &AutoUpdateState::default(),
            None,
        );
        assert_eq!(d, Decision::UpToDate);
    }

    #[test]
    fn an_available_update_is_installed() {
        let d = decide(
            AutoUpdatePolicy::Apply,
            &behind(3),
            &AutoUpdateState::default(),
            None,
        );
        assert_eq!(
            d,
            Decision::Apply {
                behind: 3,
                target: "abc1234".to_string()
            }
        );
    }

    #[test]
    fn check_policy_reports_but_never_installs() {
        let d = decide(
            AutoUpdatePolicy::Check,
            &behind(2),
            &AutoUpdateState::default(),
            None,
        );
        assert!(matches!(d, Decision::NotifyOnly { behind: 2, .. }));
    }

    #[test]
    fn off_policy_does_nothing_at_all() {
        // Not even the "you are behind" notice — off means off.
        let d = decide(
            AutoUpdatePolicy::Off,
            &behind(9),
            &AutoUpdateState::default(),
            None,
        );
        assert_eq!(d, Decision::Disabled);
    }

    #[test]
    fn local_work_is_never_clobbered() {
        let mut dirty = behind(2);
        dirty.dirty = true;
        assert!(matches!(
            decide(
                AutoUpdatePolicy::Apply,
                &dirty,
                &AutoUpdateState::default(),
                None
            ),
            Decision::Skip {
                reason: SkipReason::DirtyTree
            }
        ));

        let mut ahead = behind(2);
        ahead.ahead = 4;
        assert!(matches!(
            decide(
                AutoUpdatePolicy::Apply,
                &ahead,
                &AutoUpdateState::default(),
                None
            ),
            Decision::Skip {
                reason: SkipReason::LocalCommits { ahead: 4 }
            }
        ));
    }

    // Found by /codeaudit, proven at runtime: the fast-forward moves HEAD
    // BEFORE install.sh runs. When the install then failed, `behind` was 0 on
    // every later run, this returned UpToDate, and the install was NEVER
    // retried — the machine sat on new source with a stale binary while the
    // updater reported "already up to date". The failure cap never engaged
    // because the Apply branch was unreachable.
    #[test]
    fn an_install_that_failed_after_the_fast_forward_is_retried() {
        let mut history = AutoUpdateState::default();
        history.record_failure("abc1234");

        match decide(
            AutoUpdatePolicy::Apply,
            &ff_done_install_failed(),
            &history,
            None,
        ) {
            Decision::Apply { behind, target } => {
                assert_eq!(behind, 0, "nothing to pull — only the install is owed");
                assert_eq!(target, "abc1234");
            }
            other => panic!("a failed install must be retried, got {:?}", other),
        }
    }

    /// …but the retry is still bounded. An install that keeps failing on the
    /// checked-out commit must stop at the cap, not rebuild every night forever.
    #[test]
    fn the_owed_install_still_stops_at_the_cap() {
        let mut history = AutoUpdateState::default();
        for _ in 0..FAILURE_CAP {
            history.record_failure("abc1234");
        }
        match decide(
            AutoUpdatePolicy::Apply,
            &ff_done_install_failed(),
            &history,
            None,
        ) {
            Decision::Skip { reason } => assert!(reason.needs_human()),
            other => panic!("expected the cap to hold, got {:?}", other),
        }
    }

    /// A machine that is genuinely current — nothing pulled, nothing failed —
    /// must still take the cheap path. The retry must not fire on every box.
    #[test]
    fn a_clean_current_machine_still_does_nothing() {
        let state = ff_done_install_failed(); // same shape, but no failure history
        assert_eq!(
            decide(
                AutoUpdatePolicy::Apply,
                &state,
                &AutoUpdateState::default(),
                None
            ),
            Decision::UpToDate
        );
    }

    /// And a failure recorded against a DIFFERENT commit than the one checked
    /// out must not trigger a pointless reinstall.
    #[test]
    fn a_failure_on_another_commit_does_not_force_a_reinstall() {
        let mut history = AutoUpdateState::default();
        history.record_failure("0ldc0de");
        assert_eq!(
            decide(
                AutoUpdatePolicy::Apply,
                &ff_done_install_failed(),
                &history,
                None
            ),
            Decision::UpToDate
        );
    }

    #[test]
    fn a_dirty_checkout_that_is_current_is_not_an_alert() {
        // A developer's own machine sits dirty all day. Only a dirty checkout
        // that BLOCKS a real update is worth saying anything about.
        let mut s = behind(0);
        s.dirty = true;
        assert_eq!(
            decide(
                AutoUpdatePolicy::Apply,
                &s,
                &AutoUpdateState::default(),
                None
            ),
            Decision::UpToDate
        );
    }

    /// The state a machine is in when it AUTHORED the commit: nothing to pull,
    /// no failure ever, but the binary predates HEAD.
    fn installed(commit: &str) -> AutoUpdateState {
        AutoUpdateState {
            last_applied_commit: Some(commit.to_string()),
            ..Default::default()
        }
    }

    fn at(secs: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(secs, 0).expect("valid timestamp")
    }

    #[test]
    fn a_first_doctrine_observation_is_not_a_change() {
        // Otherwise every machine upgrading INTO this version would flag every
        // session it happens to be running, once, for nothing.
        let mut st = AutoUpdateState::default();
        st.record_doctrine("aaaa1111", at(1_000));
        assert_eq!(st.doctrine_fingerprint.as_deref(), Some("aaaa1111"));
        assert_eq!(
            st.doctrine_changed_at, None,
            "nothing to compare against yet"
        );
    }

    #[test]
    fn a_doctrine_neutral_rebuild_does_not_move_the_cutoff() {
        // This is the whole point: the cutoff must not advance on a rebuild
        // that changed no rule, or the session check goes back to crying wolf.
        let mut st = AutoUpdateState::default();
        st.record_doctrine("aaaa1111", at(1_000));
        st.record_doctrine("aaaa1111", at(2_000));
        st.record_doctrine("aaaa1111", at(3_000));
        assert_eq!(st.doctrine_changed_at, None);
    }

    #[test]
    fn a_real_doctrine_change_moves_the_cutoff_once() {
        let mut st = AutoUpdateState::default();
        st.record_doctrine("aaaa1111", at(1_000));
        st.record_doctrine("bbbb2222", at(2_000));
        assert_eq!(st.doctrine_changed_at, Some(at(2_000)));

        // ...and a neutral rebuild after it leaves the mark where it was, so a
        // session started at 2_500 stays correctly unflagged.
        st.record_doctrine("bbbb2222", at(3_000));
        assert_eq!(st.doctrine_changed_at, Some(at(2_000)));
    }

    #[test]
    fn a_successful_apply_records_the_doctrine_it_installed() {
        let mut st = AutoUpdateState::default();
        st.record_success("abc1234", at(1_000));
        assert_eq!(
            st.doctrine_fingerprint.as_deref(),
            Some(crate::rules::doctrine_fingerprint().as_str()),
            "an install must record the doctrine it actually put on disk"
        );
    }

    #[test]
    fn the_new_doctrine_fields_survive_an_old_state_file() {
        // A state file written before these fields existed must still load, or
        // upgrading would wipe the failure memory it does carry.
        let json = r#"{"last_check":null,"last_applied":null,"last_applied_commit":"abc1234",
                       "failing_commit":null,"consecutive_failures":0,"last_outcome":null}"#;
        let st: AutoUpdateState = serde_json::from_str(json).expect("old file loads");
        assert_eq!(st.last_applied_commit.as_deref(), Some("abc1234"));
        assert_eq!(st.doctrine_fingerprint, None);
        assert_eq!(st.doctrine_changed_at, None);
    }

    #[test]
    fn a_locally_authored_commit_owes_an_install() {
        // The defect this closes: the box OmegaOS is developed on commits and
        // pushes, so it is NEVER behind, and every night returned UpToDate
        // against a binary built from an older commit. Nothing failed here —
        // the install simply never ran for this commit.
        let d = decide(
            AutoUpdatePolicy::Apply,
            &ff_done_install_failed(), // head == "abc1234", behind 0, clean
            &installed("52ddd33"),     // but the binary came from an older one
            None,
        );
        assert_eq!(
            d,
            Decision::Apply {
                behind: 0,
                target: "abc1234".to_string()
            }
        );
    }

    #[test]
    fn the_message_for_an_owed_install_claims_no_failure() {
        // It must read the same whether the install failed or never ran: on a
        // machine that merely committed, "its install did not finish" sends the
        // reader hunting for an error that does not exist.
        let msg = Decision::Apply {
            behind: 0,
            target: "abc1234".to_string(),
        }
        .describe();
        assert!(msg.contains("the installed binary is not"), "{msg}");
        assert!(!msg.to_lowercase().contains("did not finish"), "{msg}");
    }

    #[test]
    fn the_installed_commit_matching_head_is_the_quiet_outcome() {
        // The overwhelmingly common case must stay silent, or the new check
        // turns every night into a rebuild.
        assert_eq!(
            decide(
                AutoUpdatePolicy::Apply,
                &ff_done_install_failed(),
                &installed("abc1234"),
                None
            ),
            Decision::UpToDate
        );
    }

    #[test]
    fn unknown_provenance_never_forces_a_rebuild_loop() {
        // `None` means no install ever recorded what it installed — an install
        // predating this field, or a state file that cannot be written. Owing an
        // install on that would rebuild the workspace every single night,
        // forever, on a machine that is perfectly current.
        assert_eq!(
            decide(
                AutoUpdatePolicy::Apply,
                &ff_done_install_failed(),
                &AutoUpdateState::default(), // last_applied_commit: None
                None
            ),
            Decision::UpToDate
        );
    }

    #[test]
    fn an_owed_install_still_never_clobbers_unpushed_work() {
        // Committed locally but not pushed: the install is owed, yet a
        // fast-forward would destroy the local commits. The refusal wins.
        let mut s = ff_done_install_failed();
        s.ahead = 2;
        assert_eq!(
            decide(AutoUpdatePolicy::Apply, &s, &installed("52ddd33"), None),
            Decision::Skip {
                reason: SkipReason::LocalCommits { ahead: 2 }
            }
        );
    }

    #[test]
    fn an_owed_install_is_reported_under_the_check_policy() {
        let d = decide(
            AutoUpdatePolicy::Check,
            &ff_done_install_failed(),
            &installed("52ddd33"),
            None,
        );
        assert_eq!(
            d,
            Decision::NotifyOnly {
                behind: 0,
                target: "abc1234".to_string()
            }
        );
    }

    #[test]
    fn a_working_agent_defers_the_rebuild() {
        let d = decide(
            AutoUpdatePolicy::Apply,
            &behind(1),
            &AutoUpdateState::default(),
            Some("oracle-Camelia-1"),
        );
        match d {
            Decision::Skip { reason } => {
                assert!(!reason.needs_human(), "waiting a day is not an escalation");
                assert!(reason.describe().contains("oracle-Camelia-1"));
            }
            other => panic!("expected a deferral, got {:?}", other),
        }
    }

    #[test]
    fn the_same_bad_commit_is_not_retried_forever() {
        let mut history = AutoUpdateState::default();
        let state = behind(1);
        for attempt in 1..=FAILURE_CAP {
            assert!(
                matches!(
                    decide(AutoUpdatePolicy::Apply, &state, &history, None),
                    Decision::Apply { .. }
                ),
                "attempt {} must still be tried",
                attempt
            );
            history.record_failure(&state.target);
        }
        // Cap reached — it stops and asks for a human.
        match decide(AutoUpdatePolicy::Apply, &state, &history, None) {
            Decision::Skip { reason } => {
                assert!(reason.needs_human());
                assert!(
                    matches!(reason, SkipReason::RepeatedFailure { failures, .. } if failures == FAILURE_CAP)
                );
            }
            other => panic!("expected the cap to stop it, got {:?}", other),
        }
    }

    #[test]
    fn a_new_commit_on_the_remote_is_a_fresh_attempt() {
        let mut history = AutoUpdateState::default();
        for _ in 0..FAILURE_CAP {
            history.record_failure("abc1234");
        }
        let mut moved_on = behind(1);
        moved_on.target = "def5678".to_string();
        assert!(
            matches!(
                decide(AutoUpdatePolicy::Apply, &moved_on, &history, None),
                Decision::Apply { .. }
            ),
            "a different target must not inherit the old target's failures"
        );
        assert_eq!(history.failures_for("def5678"), 0);
    }

    #[test]
    fn success_clears_the_failure_memory() {
        let mut history = AutoUpdateState::default();
        history.record_failure("abc1234");
        history.record_failure("abc1234");
        assert_eq!(history.failures_for("abc1234"), 2);
        history.record_success("abc1234", Utc::now());
        assert_eq!(history.failures_for("abc1234"), 0);
        assert_eq!(history.last_applied_commit.as_deref(), Some("abc1234"));
    }

    #[test]
    fn state_round_trips_through_disk() {
        let tmp = tempfile::tempdir().unwrap();
        let mut s = AutoUpdateState::default();
        s.record_failure("abc1234");
        s.last_outcome = Some("installed".to_string());
        s.save(tmp.path()).unwrap();

        let back = AutoUpdateState::load(tmp.path());
        assert_eq!(back.failures_for("abc1234"), 1);
        assert_eq!(back.last_outcome.as_deref(), Some("installed"));
    }

    #[test]
    fn a_corrupt_state_file_never_blocks_an_update() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(AutoUpdateState::path(tmp.path()), "{ truncated").unwrap();
        let back = AutoUpdateState::load(tmp.path());
        assert_eq!(back.consecutive_failures, 0, "corrupt reads as no history");
        assert!(matches!(
            decide(AutoUpdatePolicy::Apply, &behind(1), &back, None),
            Decision::Apply { .. }
        ));
    }

    #[test]
    fn policy_parses_what_users_actually_type() {
        assert_eq!(AutoUpdatePolicy::parse("off"), AutoUpdatePolicy::Off);
        assert_eq!(AutoUpdatePolicy::parse("  CHECK "), AutoUpdatePolicy::Check);
        assert_eq!(AutoUpdatePolicy::parse("apply"), AutoUpdatePolicy::Apply);
        // A typo must not silently disable updates.
        assert_eq!(AutoUpdatePolicy::parse("aply"), AutoUpdatePolicy::Apply);
    }
}

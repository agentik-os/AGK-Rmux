use crate::config::OmegaConfig;
use crate::done::{DoneSignal, DoneStatus, OracleDoneSignal, WorkerBlocked};
use crate::inbox::{Inbox, InboxEvent};
use crate::oracle_lifecycle::{
    OracleRegistry, OracleRegistryStatus, OracleState, SignalWatcher, StallAction, StallThresholds,
    WorkerEntryStatus, WorkerStallDetector,
};
use crate::scope::ScopeClaim;
use crate::session::{SessionManager, SessionRole};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::time::Duration;

const STALL_THRESHOLD_SECS: i64 = 900; // 15 minutes without progress = stalled (file-based)
const AUTO_DONE_IDLE_SECS: i64 = 120; // 2 minutes idle after 100% todos = patrol auto-done

// Deterministic worker close (Task#6). After a worker's done_clean clears the
// ground-truth gate, patrol marks the rmux session Closeable and reaps it (kill
// + lock release) once the parent oracle has CONSUMED/ack'd the worker_done
// event (its inbox no longer carries it) OR this bounded grace window elapses —
// whichever comes first. This removes reliance on the idle/CPU heuristic for the
// honest-done case (an honest worker used to linger as a zombie because the
// primary path never killed it). Const, not a config.rs knob, by design.
const WORKER_CLOSE_GRACE_SECS: i64 = 45;
// Grace window before a closeable (done_clean, no pending actions) oracle is
// deterministically reaped. Longer than the worker grace: the inline auto-close
// in `omega done` / `omega progress` normally fires within seconds, so patrol's
// reap is the backstop for a missed close, not the primary path.
const ORACLE_CLOSE_GRACE_SECS: i64 = 120;

// Orphan-worker sweep: a worker whose governing oracle is GONE (session dead)
// while that oracle's mission is declared done_clean is a zombie — nothing
// will ever consume its output. Generous grace after the oracle's finished_at
// so a same-name re-dispatch (which clears the stale signal first) can never
// race the sweep.
const ORPHAN_WORKER_GRACE_SECS: i64 = 300;

// Ceiling on how old a done signal may be and still authorize an orphan reap.
//
// The grace above is a FLOOR and was the only bound, which is safe on the
// registered-parent branch (the signal is read by exact oracle name, and a
// same-name re-dispatch clears it first) but NOT on the project-match branch
// below, which accepts any closeable signal sharing the worker's project. A
// project accumulates done.json files forever, so the oldest one kept matching
// and authorized killing workers it never governed.
//
// Measured incident (2026-08-04): `oracle-OmegaOS-4.done.json`, done_clean and
// finished 2026-06-28, matched every freshly spawned `OmegaOS-worker-*` and the
// sweep closed three README workers within 60s of spawn each, before any of
// them could write a file or a done signal.
//
// A genuine orphan is swept within GRACE + one patrol tick of its mission
// finishing, so a signal older than this ceiling cannot still have live
// legitimate orphans: anything running under it is a NEW worker. The window is
// deliberately generous (24h) to tolerate patrol downtime, and erring toward
// leaving a worker alive matches this module's own doctrine that losing a
// worker's commits is far worse than leaking one - a leak is recoverable with
// `omega reap`, a wrongful kill destroys unsaved work.
const ORPHAN_SIGNAL_MAX_AGE_SECS: i64 = 86_400;

#[derive(Debug)]
pub struct PatrolReport {
    pub total_sessions: usize,
    pub oracles: usize,
    pub workers: usize,
    pub done_workers: Vec<String>,
    pub stalled_workers: Vec<String>,
    pub blocked_workers: Vec<String>,
    pub orphaned_sessions: Vec<String>,
    pub done_oracles: Vec<String>,
    pub actions_taken: Vec<String>,
}

pub struct Patrol {
    config: OmegaConfig,
    stall_detector: WorkerStallDetector,
    signal_watcher: SignalWatcher,
}

impl Patrol {
    pub fn new(config: OmegaConfig) -> Self {
        let signal_watcher = SignalWatcher::new(config.state_dir.clone());
        Self {
            stall_detector: WorkerStallDetector::new(StallThresholds::default()),
            signal_watcher,
            config,
        }
    }

    /// Resolve V3 task authority for one worker. `None` means the session is a
    /// legacy/unregistered worker; any malformed V3 binding is an error, never
    /// permission to fall back to compatibility state.
    fn v3_worker_attempt_state(
        &self,
        oracle_states: &[OracleState],
        session: &str,
    ) -> Result<Option<crate::mission::TaskAttemptState>> {
        let Some((oracle, worker)) = oracle_states.iter().find_map(|oracle| {
            oracle
                .workers
                .iter()
                .find(|worker| worker.session_name == session)
                .map(|worker| (oracle, worker))
        }) else {
            return Ok(None);
        };
        let attempt_id = worker
            .attempt_id
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("registered V3 worker has no attempt id"))?;
        let revision = worker
            .plan_revision
            .ok_or_else(|| anyhow::anyhow!("registered V3 worker has no plan revision"))?;
        let ledger = crate::mission_ledger::MissionLedger::open(
            self.config.state_dir.join("mission-engine-v3.sqlite3"),
        )?;
        oracle.require_ledger_authority(&ledger)?;
        let attempt = ledger
            .task_attempt(attempt_id)?
            .ok_or_else(|| anyhow::anyhow!("registered V3 attempt is missing"))?;
        if attempt.mission_id != oracle.mission_id
            || attempt.task_id != worker.task_id
            || attempt.plan_revision != revision
        {
            anyhow::bail!("registered worker differs from exact ledger task authority");
        }
        Ok(Some(attempt.state))
    }

    fn release_exact_accepted_worker_scopes(
        &self,
        oracle: &OracleState,
        worker: &crate::oracle_lifecycle::WorkerEntry,
    ) -> Result<()> {
        let attempt_id = worker
            .attempt_id
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("registered V3 worker has no attempt id"))?;
        let plan_revision = worker
            .plan_revision
            .ok_or_else(|| anyhow::anyhow!("registered V3 worker has no plan revision"))?;
        let ledger = crate::mission_ledger::MissionLedger::open(
            self.config.state_dir.join("mission-engine-v3.sqlite3"),
        )?;
        oracle.require_ledger_authority(&ledger)?;
        let projection = ledger
            .task_attempt(attempt_id)?
            .ok_or_else(|| anyhow::anyhow!("registered V3 attempt is missing"))?;
        if projection.mission_id != oracle.mission_id
            || projection.task_id != worker.task_id
            || projection.plan_revision != plan_revision
            || projection.state != crate::mission::TaskAttemptState::Accepted
        {
            anyhow::bail!("scope release requires the exact Accepted V3 attempt");
        }
        let attempt = crate::orchestration::AuthoritativeTaskAttempt {
            mission_id: oracle.mission_id.clone(),
            task_id: worker.task_id.clone(),
            attempt_id: attempt_id.to_string(),
            plan_revision,
            owner: Some(worker.session_name.clone()),
            leases: ledger.active_leases_for_attempt(
                &oracle.mission_id,
                &worker.task_id,
                attempt_id,
            )?,
            scope_receipt: None,
        };
        crate::orchestration::release_authoritative_scopes(
            &ledger,
            &self.config.state_dir,
            &attempt,
        )
    }

    fn require_delivered_oracle_authority(
        &self,
        oracle_states: &[OracleState],
        oracle_name: &str,
        signal: &OracleDoneSignal,
    ) -> Result<OracleState> {
        let matches: Vec<_> = oracle_states
            .iter()
            .filter(|state| state.oracle_name == oracle_name)
            .collect();
        if matches.len() != 1 {
            anyhow::bail!(
                "expected one strict V3 OracleState for {oracle_name}, found {}",
                matches.len()
            );
        }
        let state = matches[0];
        let ledger = crate::mission_ledger::MissionLedger::open(
            self.config.state_dir.join("mission-engine-v3.sqlite3"),
        )?;
        state.require_ledger_authority(&ledger)?;
        ledger
            .mission_record(&state.mission_id)?
            .ok_or_else(|| anyhow::anyhow!("immutable V3 mission is missing"))?;
        let projection = ledger
            .mission(&state.mission_id)?
            .ok_or_else(|| anyhow::anyhow!("V3 mission projection is missing"))?;
        if projection.state != crate::mission::MissionState::Delivered {
            anyhow::bail!("V3 mission is {:?}, not Delivered", projection.state);
        }
        ledger.validate_mission_acceptance(&state.mission_id)?;
        crate::orchestration::validate_oracle_done_signal_authority(
            &ledger,
            &state.mission_id,
            oracle_name,
            signal,
        )?;
        Ok(state.clone())
    }

    pub async fn run_once(&mut self) -> Result<PatrolReport> {
        // Heartbeat — proves the patrol actually fired. Lets the user (and
        // `omega doctor`) verify the self-improvement loop is alive rather
        // than silently dead (the failure mode of the old Smith agent).
        let hb = self.config.state_dir.join("patrol-heartbeat.txt");
        crate::config::atomic_write_private(&hb, Utc::now().to_rfc3339().as_bytes())?;

        // connect_cached: the patrol daemon calls run_once every tick — reuse one
        // process-wide rmux connection instead of opening a fresh socket per tick.
        let mgr = SessionManager::connect_cached().await?;
        let sessions = mgr.list_sessions().await?;

        let mut report = PatrolReport {
            total_sessions: sessions.len(),
            oracles: sessions
                .iter()
                .filter(|s| s.role == SessionRole::Oracle)
                .count(),
            workers: sessions
                .iter()
                .filter(|s| s.role == SessionRole::Worker)
                .count(),
            done_workers: Vec::new(),
            stalled_workers: Vec::new(),
            blocked_workers: Vec::new(),
            orphaned_sessions: Vec::new(),
            done_oracles: Vec::new(),
            actions_taken: Vec::new(),
        };

        let oracle_sessions: Vec<_> = sessions
            .iter()
            .filter(|s| s.role == SessionRole::Oracle)
            .collect();

        // Read every oracle's persisted state ONCE per tick. find_parent_oracle
        // needs it to resolve a worker -> its governing oracle, and it's called
        // once per signaling worker; reading it per call was an O(W×O) disk scan +
        // JSON parse every tick. Compute it here and pass the slice down.
        let oracle_states =
            crate::oracle_lifecycle::OracleState::read_all_strict(&self.config.state_dir)?;
        let team_runs = crate::team::TeamRunState::read_all_strict(&self.config.state_dir)?;
        let team_session_names: std::collections::HashSet<String> = team_runs
            .iter()
            .map(|run| run.team_session.clone())
            .collect();
        let live_team_snapshot: std::collections::HashSet<String> = sessions
            .iter()
            .map(|session| session.name.clone())
            .collect();
        let team_report = crate::team::reconcile_team_runs(
            &self.config.state_dir,
            &mgr,
            &team_runs,
            &live_team_snapshot,
        )
        .await?;
        report.actions_taken.extend(team_report.actions);

        // ── Broken-pane sweep: panes whose terminal object the daemon lost ──
        // rmux (≤0.3.1) can lose a pane's in-memory terminal while the pane
        // process keeps running (2026-06-12: recreated same-name sessions
        // listed fine but every capture/attach/status failed with "missing
        // pane terminal" — invisible in the TUI, unreachable by send-keys).
        // The pane is unusable either way, so repair beats preserving it:
        // respawn the pane (rebuilds the terminal, keeps the session and its
        // start dir), then for agent-bearing sessions relaunch the configured
        // agent with --continue so the conversation resumes where it stopped.
        // System/plain-shell sessions just get their shell back.
        for session in &sessions {
            match mgr.capture_pane(&session.name).await {
                Err(e) if format!("{e:#}").contains("missing pane terminal") => {}
                _ => continue,
            }
            if team_session_names.contains(&session.name) {
                report.actions_taken.push(format!(
                    "Team {} has a broken pane; held for exact team-run reconciliation",
                    session.name
                ));
                continue;
            }
            tracing::warn!(session = %session.name, "Broken pane (terminal lost) — respawning");
            let respawned = std::process::Command::new("rmux")
                .args(["respawn-pane", "-k", "-t", &session.name])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if !respawned {
                report.actions_taken.push(format!(
                    "Session {}: pane terminal lost, respawn FAILED — repair manually \
                     (rmux respawn-pane -k -t {})",
                    session.name, session.name
                ));
                continue;
            }
            mgr.invalidate_pane(&session.name).await;
            let relaunch_agent = session.project.is_some()
                || matches!(session.role, SessionRole::Oracle | SessionRole::Worker);
            if relaunch_agent {
                let configured = session
                    .provider
                    .as_deref()
                    .unwrap_or(self.config.agent_command.as_str());
                let Some(agent) = crate::agents::Agent::from_name(configured) else {
                    report.actions_taken.push(format!(
                        "Session {}: pane respawned but agent relaunch REFUSED; unknown configured provider {:?}",
                        session.name, configured
                    ));
                    continue;
                };
                let working_dir = session
                    .working_dir
                    .as_deref()
                    .and_then(std::path::Path::to_str);
                if let Err(error) = mgr
                    .relaunch_agent_session_with_opts(
                        &session.name,
                        working_dir,
                        agent,
                        None,
                        crate::agents::LaunchOptions {
                            resume_conversation: true,
                            ..Default::default()
                        },
                    )
                    .await
                {
                    report.actions_taken.push(format!(
                        "Session {}: pane respawned but typed agent relaunch FAILED ({error})",
                        session.name
                    ));
                    continue;
                }
            }
            report.actions_taken.push(format!(
                "Session {}: pane terminal lost (rmux bug) — respawned pane{}",
                session.name,
                if relaunch_agent {
                    " + relaunched agent (--continue)"
                } else {
                    ""
                }
            ));
        }

        // ── Worker patrol: done signals ──
        for session in &sessions {
            if session.role == SessionRole::Worker && !team_session_names.contains(&session.name) {
                // ── Freshness guard (worker twin of the oracle guard below) ──
                // Worker names are deterministic (`<project>-worker-<task>`)
                // and the done.json survives its session, so a re-dispatch
                // under the same name would otherwise be insta-finished — and
                // then reaped — on its PREDECESSOR's stale signal. Date the
                // signal against the worker's `dispatched_at` from its
                // oracle's persisted state (register_worker refreshes it on a
                // re-dispatch). No WorkerEntry → treat as fresh: hand-spawned
                // workers have no registry entry, and dropping their signal
                // would break done delivery for them entirely.
                let fresh_done = match DoneSignal::read(&self.config.state_dir, &session.name)? {
                    Some(done) => {
                        let dispatched_at = oracle_states
                            .iter()
                            .flat_map(|s| s.workers.iter())
                            .filter(|w| w.session_name == session.name)
                            .map(|w| w.dispatched_at)
                            .max();
                        if worker_signal_is_stale(done.finished_at, dispatched_at) {
                            tracing::warn!(
                                worker = %session.name,
                                finished_at = %done.finished_at,
                                dispatched_at = ?dispatched_at,
                                "stale worker done signal predates dispatch — ignored"
                            );
                            report.actions_taken.push(format!(
                                "Ignored stale done signal for {} (predates dispatch)",
                                session.name
                            ));
                            None
                        } else {
                            Some(done)
                        }
                    }
                    None => None,
                };
                if let Some(done) = fresh_done {
                    report.done_workers.push(session.name.clone());

                    // A worker's narration is only a CandidateDone. Patrol uses
                    // the same exact-plan independent finalizer as native
                    // orchestration; no compatibility-only artifact path may
                    // write Accepted.
                    let repo_root = crate::session::OmegaSession::classify(&session.name)
                        .project
                        .as_deref()
                        .and_then(|p| resolve_repo_root(&self.config, p));
                    let mut effective_status = done.status;
                    let mut contest_reason: Option<String> = None;
                    let mut accepted_persisted = false;
                    let v3_worker = oracle_states.iter().find_map(|state| {
                        state
                            .workers
                            .iter()
                            .find(|worker| worker.session_name == session.name)
                            .map(|worker| (state, worker))
                    });
                    if done.status == DoneStatus::DoneClean {
                        match (v3_worker, repo_root.as_deref()) {
                            (Some((oracle, worker)), Some(root)) => {
                                let attempt_id = worker.attempt_id.as_deref();
                                let plan_revision = worker.plan_revision;
                                let result = match (attempt_id, plan_revision) {
                                    (Some(attempt_id), Some(plan_revision)) => {
                                        let ledger = crate::mission_ledger::MissionLedger::open(
                                            self.config.state_dir.join("mission-engine-v3.sqlite3"),
                                        );
                                        ledger.map_err(anyhow::Error::from).and_then(|ledger| {
                                            crate::orchestration::verify_and_finalize_candidate(
                                                &ledger,
                                                &oracle.mission_id,
                                                &worker.task_id,
                                                attempt_id,
                                                plan_revision,
                                                &session.name,
                                                &done,
                                                root,
                                            )
                                        })
                                    }
                                    _ => Err(anyhow::anyhow!(
                                        "worker registry has no exact V3 attempt binding"
                                    )),
                                };
                                match result {
                                    Ok(outcome)
                                        if outcome.accepted
                                            && outcome.attempt_state
                                                == crate::mission::TaskAttemptState::Accepted =>
                                    {
                                        match self
                                            .release_exact_accepted_worker_scopes(oracle, worker)
                                        {
                                            Ok(()) => {
                                                accepted_persisted = true;
                                                effective_status = DoneStatus::DoneClean;
                                            }
                                            Err(error) => {
                                                effective_status = DoneStatus::Pending;
                                                report.actions_taken.push(format!(
                                                    "{}: Accepted persisted but exact scope release failed closed ({error})",
                                                    session.name
                                                ));
                                            }
                                        }
                                    }
                                    Ok(outcome) => {
                                        effective_status = DoneStatus::Pending;
                                        let reason = if outcome.verification.failures.is_empty() {
                                            format!(
                                                "authoritative attempt settled as {:?}",
                                                outcome.attempt_state
                                            )
                                        } else {
                                            outcome.verification.failures.join("; ")
                                        };
                                        let contradicted = outcome
                                            .verification
                                            .observations
                                            .iter()
                                            .filter(|observation| !observation.passed)
                                            .map(|observation| observation.detail.as_str())
                                            .any(|detail| {
                                                detail.contains(" exited ")
                                                    || detail.contains("does NOT exist")
                                                    || detail.contains("returned ")
                                            });
                                        if contradicted {
                                            contest_reason = Some(reason.clone());
                                        }
                                        report.actions_taken.push(format!(
                                            "{}: candidate retained as pending; exact independent verification rejected ({reason})",
                                            session.name
                                        ));
                                    }
                                    Err(error) => {
                                        effective_status = DoneStatus::Pending;
                                        report.actions_taken.push(format!(
                                            "{}: candidate authority/verification failed closed; scope held ({error})",
                                            session.name
                                        ));
                                    }
                                }
                            }
                            (None, _) => {
                                effective_status = DoneStatus::Pending;
                                report.actions_taken.push(format!(
                                    "{}: no V3 worker binding; candidate remains pending and scope held",
                                    session.name
                                ));
                            }
                            (_, None) => {
                                effective_status = DoneStatus::Pending;
                                report.actions_taken.push(format!(
                                    "{}: project root is unresolved; candidate remains pending and scope held",
                                    session.name
                                ));
                            }
                        }
                    } else if let Some((oracle, worker)) = v3_worker {
                        let target = match done.status {
                            DoneStatus::Failed => crate::mission::TaskAttemptState::Failed,
                            DoneStatus::Blocked | DoneStatus::Pending => {
                                crate::mission::TaskAttemptState::Blocked
                            }
                            DoneStatus::DoneClean => unreachable!("handled above"),
                        };
                        let result = match (worker.attempt_id.as_deref(), worker.plan_revision) {
                            (Some(attempt_id), Some(plan_revision)) => {
                                let ledger = crate::mission_ledger::MissionLedger::open(
                                    self.config.state_dir.join("mission-engine-v3.sqlite3"),
                                );
                                ledger.map_err(anyhow::Error::from).and_then(|ledger| {
                                    oracle.require_ledger_authority(&ledger)?;
                                    crate::orchestration::finalize_nonclean_candidate(
                                        &ledger,
                                        &oracle.mission_id,
                                        &worker.task_id,
                                        attempt_id,
                                        plan_revision,
                                        &session.name,
                                        &done,
                                        target,
                                    )
                                })
                            }
                            _ => Err(anyhow::anyhow!(
                                "worker registry has no exact V3 attempt binding"
                            )),
                        };
                        if let Err(error) = result {
                            effective_status = DoneStatus::Pending;
                            report.actions_taken.push(format!(
                                "{}: non-clean authoritative transition failed; scope held ({error})",
                                session.name
                            ));
                        }
                    }

                    if let Some(oracle) =
                        self.find_parent_oracle(&session.name, &oracle_sessions, &oracle_states)
                    {
                        let inbox = Inbox::for_oracle(&self.config.state_dir, &oracle.name);
                        let status_str = if contest_reason.is_some() {
                            "contested"
                        } else {
                            match effective_status {
                                DoneStatus::DoneClean => "done_clean",
                                DoneStatus::Pending => "pending",
                                DoneStatus::Failed => "failed",
                                DoneStatus::Blocked => "blocked",
                            }
                        };
                        // Push the worker_done event ONCE per signal, not once
                        // per tick. The reap pass treats "event absent from
                        // the oracle inbox" as the ack — a re-push every tick
                        // made the ack unobservable (only the grace timer ever
                        // fired) and delivered the same event to the oracle
                        // repeatedly. The marker is keyed on status+finished_at
                        // so a NEW or upgraded signal re-arms automatically.
                        let event_key = format!("{}:{}", status_str, done.finished_at.timestamp());
                        if !inbox_event_already_sent(
                            &self.config.state_dir,
                            &session.name,
                            "done",
                            &event_key,
                        )? {
                            let pushed = match inbox
                                .push(&InboxEvent::worker_done(&session.name, status_str))
                            {
                                Ok(()) => true,
                                Err(error) => {
                                    report.actions_taken.push(format!(
                                        "Worker {} done-event delivery FAILED ({error})",
                                        session.name
                                    ));
                                    false
                                }
                            };
                            // Surface the fabrication detail so the oracle can
                            // re-dispatch with eyes open.
                            if let Some(reason) = &contest_reason {
                                if let Err(error) = inbox.push(&InboxEvent::worker_blocked(
                                    &session.name,
                                    &format!("GROUND-TRUTH CONTEST: {}", reason),
                                )) {
                                    report.actions_taken.push(format!(
                                        "Worker {} contest-detail delivery FAILED ({error})",
                                        session.name
                                    ));
                                }
                            }
                            // Record only on a successful push — a failed one
                            // must retry next tick, not be marked delivered.
                            if pushed {
                                if let Err(error) = record_inbox_event_sent(
                                    &self.config.state_dir,
                                    &session.name,
                                    "done",
                                    &event_key,
                                ) {
                                    report.actions_taken.push(format!(
                                        "Worker {} done event delivered, but delivery marker FAILED ({error})",
                                        session.name
                                    ));
                                }
                            }
                        }

                        // Update oracle state with worker completion
                        match OracleState::read(&self.config.state_dir, &oracle.name) {
                            Ok(Some(mut oracle_state)) => {
                                let ws = match effective_status {
                                    DoneStatus::DoneClean => WorkerEntryStatus::DoneClean,
                                    DoneStatus::Pending => WorkerEntryStatus::Pending,
                                    DoneStatus::Failed => WorkerEntryStatus::Failed,
                                    DoneStatus::Blocked => WorkerEntryStatus::Blocked,
                                };
                                oracle_state.update_worker_status(&session.name, ws);
                                if let Err(error) = oracle_state.write(&self.config.state_dir) {
                                    report.actions_taken.push(format!(
                                        "Worker {} delivered, but oracle-state update FAILED ({error})",
                                        session.name
                                    ));
                                }
                            }
                            Ok(None) => report.actions_taken.push(format!(
                                "Worker {} delivered, but parent oracle state is missing",
                                session.name
                            )),
                            Err(error) => report.actions_taken.push(format!(
                                "Worker {} delivered, but parent oracle state is unreadable ({error})",
                                session.name
                            )),
                        }
                    }

                    if let Some(reason) = contest_reason {
                        // Fabrication: keep the scope claim HELD (work is not
                        // actually done) and flag it loudly.
                        report.actions_taken.push(format!(
                            "CONTESTED {}: done_clean failed ground-truth — {}",
                            session.name, reason
                        ));
                        // Loop Engineering — bounded retries then escalate. A
                        // worker that re-writes a contested done_clean is
                        // thrashing; count consecutive contested attempts and at
                        // THRASH_CAP hand the loop to a human instead of letting
                        // it re-fabricate forever (L1's "3rd change → runtime
                        // evidence", enforced at the orchestration layer).
                        let thrash =
                            crate::loop_guard::bump_thrash(&self.config.state_dir, &session.name);
                        if let Some(oracle) =
                            self.find_parent_oracle(&session.name, &oracle_sessions, &oracle_states)
                        {
                            crate::loop_guard::MissionLog::event(
                                &self.config.state_dir,
                                &oracle.name,
                                "contest",
                                &format!(
                                    "{} contested ({}/{}): {}",
                                    session.name,
                                    thrash,
                                    crate::loop_guard::THRASH_CAP,
                                    reason
                                ),
                            );
                            if thrash >= crate::loop_guard::THRASH_CAP
                                && crate::loop_guard::escalate_to_human(
                                    &self.config.state_dir,
                                    &oracle.name,
                                    crate::loop_guard::EscalationReason::ContestedFabrication,
                                    &format!(
                                        "worker {} contested {}× — {}",
                                        session.name, thrash, reason
                                    ),
                                )
                            {
                                report.actions_taken.push(format!(
                                    "ESCALATED TO HUMAN: {} thrashed {}× (contested fabrication)",
                                    session.name, thrash
                                ));
                            }
                        }
                    } else if accepted_persisted && effective_status == DoneStatus::DoneClean {
                        self.stall_detector.forget(&session.name);
                        // Clean, uncontested close → the loop converged; reset
                        // the worker's thrash counter so a future reuse of the
                        // name starts fresh.
                        crate::loop_guard::clear_thrash(&self.config.state_dir, &session.name);
                        // Task#6 — deterministic close: an honest worker that
                        // wrote a verified done_clean used to keep its rmux
                        // session ALIVE (the only kill_session was the idle
                        // heuristic), leaving a zombie. Mark it Closeable now; the
                        // reap pass below kills it once the parent oracle ack's
                        // the worker_done event OR the grace window elapses.
                        let parent = self
                            .find_parent_oracle(&session.name, &oracle_sessions, &oracle_states)
                            .map(|o| o.name.clone());
                        match WorkerCloseMarker::ensure(
                            &self.config.state_dir,
                            &session.name,
                            parent.as_deref(),
                        ) {
                            Ok(()) => report.actions_taken.push(format!(
                                "Released scope for {} (ground-truth [+]); marked Closeable",
                                session.name
                            )),
                            Err(error) => report.actions_taken.push(format!(
                                "Released scope for {}, but Closeable marker FAILED ({error})",
                                session.name
                            )),
                        }
                    }
                }

                // Check for blocked workers
                if let Some(blocked) = WorkerBlocked::read(&self.config.state_dir, &session.name)? {
                    report.blocked_workers.push(session.name.clone());
                    if let Some(oracle) =
                        self.find_parent_oracle(&session.name, &oracle_sessions, &oracle_states)
                    {
                        let inbox = Inbox::for_oracle(&self.config.state_dir, &oracle.name);
                        // Same push-once contract as worker_done above: the
                        // blocked file persists across ticks, so an unguarded
                        // push re-delivered the question every minute. Keyed
                        // on blocked_at — a NEW block re-arms.
                        let bkey = blocked.blocked_at.timestamp().to_string();
                        if !inbox_event_already_sent(
                            &self.config.state_dir,
                            &session.name,
                            "blocked",
                            &bkey,
                        )? && inbox
                            .push(&InboxEvent::worker_blocked(
                                &session.name,
                                &blocked.question,
                            ))
                            .is_ok()
                        {
                            if let Err(error) = record_inbox_event_sent(
                                &self.config.state_dir,
                                &session.name,
                                "blocked",
                                &bkey,
                            ) {
                                report.actions_taken.push(format!(
                                    "Worker {} blocked event delivered, but delivery marker FAILED ({error})",
                                    session.name
                                ));
                            }
                        }
                    }
                }
            }
        }

        // ── Worker patrol: pane-based stall detection (30s nudge / 5min escalate) ──
        for session in &sessions {
            if session.role != SessionRole::Worker || team_session_names.contains(&session.name) {
                continue;
            }
            let has_done = DoneSignal::read(&self.config.state_dir, &session.name)?.is_some();
            if has_done {
                continue;
            }

            match mgr.capture_pane(&session.name).await {
                Ok(content) => {
                    // A content-filter block or hard API error means the agent is
                    // stuck on an error, not merely idle. Escalate to the oracle now
                    // (with the reason) instead of waiting out the stall thresholds,
                    // and stop tracking it as a potential stall.
                    if let Some(reason) = detect_fatal_agent_error(&content) {
                        report.blocked_workers.push(session.name.clone());
                        if let Some(oracle) =
                            self.find_parent_oracle(&session.name, &oracle_sessions, &oracle_states)
                        {
                            let inbox = Inbox::for_oracle(&self.config.state_dir, &oracle.name);
                            if let Err(error) =
                                inbox.push(&InboxEvent::worker_blocked(&session.name, reason))
                            {
                                report.actions_taken.push(format!(
                                    "Worker {} fatal error detected, but oracle escalation FAILED ({error})",
                                    session.name
                                ));
                            }
                        }
                        report.actions_taken.push(format!(
                            "Worker {} blocked by {}; oracle escalation attempted",
                            session.name, reason
                        ));
                        self.stall_detector.forget(&session.name);
                        continue;
                    }
                    let action = self.stall_detector.check(&session.name, &content);
                    match action {
                        StallAction::Nudge {
                            ref session,
                            idle_secs,
                        } => {
                            tracing::info!(worker = %session, idle_secs, "Worker idle — nudge");
                            // Send a nudge via the session pane
                            match mgr
                                .send_text(
                                    session,
                                    "You appear idle. Continue your mission or report done.",
                                )
                                .await
                            {
                                Ok(()) => report
                                    .actions_taken
                                    .push(format!("Nudged {} (idle {}s)", session, idle_secs)),
                                Err(error) => report.actions_taken.push(format!(
                                    "Nudge FAILED for {} (idle {}s): {error}",
                                    session, idle_secs
                                )),
                            }
                        }
                        StallAction::Escalate {
                            ref session,
                            idle_secs,
                        } => {
                            report.stalled_workers.push(session.clone());
                            if let Some(oracle) =
                                self.find_parent_oracle(session, &oracle_sessions, &oracle_states)
                            {
                                let inbox = Inbox::for_oracle(&self.config.state_dir, &oracle.name);
                                if let Err(error) =
                                    inbox.push(&InboxEvent::worker_stalled(session, idle_secs))
                                {
                                    report.actions_taken.push(format!(
                                        "Stall escalation delivery FAILED for {}: {error}",
                                        session
                                    ));
                                }
                            }
                            report.actions_taken.push(format!(
                                "Stall detected: {} (idle {}s); oracle escalation attempted",
                                session, idle_secs
                            ));
                        }
                        StallAction::Active => {}
                    }
                }
                Err(_) => {
                    // Can't capture pane — session might be dead
                    report.orphaned_sessions.push(session.name.clone());
                }
            }
        }

        // ── Worker patrol: file-based stall detection (progress files) ──
        for session in &sessions {
            if session.role == SessionRole::Worker && !team_session_names.contains(&session.name) {
                if let Some(progress) =
                    crate::progress::ProgressInfo::read(&self.config.state_dir, &session.name)
                {
                    let has_done =
                        DoneSignal::read(&self.config.state_dir, &session.name)?.is_some();
                    if !has_done
                        && progress.todos_completed < progress.todos_total
                        && !progress.blocked
                    {
                        if let Some(last_update) = progress.last_updated {
                            let idle_secs = (Utc::now() - last_update).num_seconds();
                            if idle_secs > STALL_THRESHOLD_SECS
                                && !report.stalled_workers.contains(&session.name)
                            {
                                report.stalled_workers.push(session.name.clone());
                                if let Some(oracle) = self.find_parent_oracle(
                                    &session.name,
                                    &oracle_sessions,
                                    &oracle_states,
                                ) {
                                    let inbox =
                                        Inbox::for_oracle(&self.config.state_dir, &oracle.name);
                                    if let Err(error) = inbox.push(&InboxEvent::worker_stalled(
                                        &session.name,
                                        idle_secs as u64,
                                    )) {
                                        report.actions_taken.push(format!(
                                            "Progress-stall escalation delivery FAILED for {}: {error}",
                                            session.name
                                        ));
                                    }
                                }
                                report.actions_taken.push(format!(
                                    "Stall detected (progress): {} (idle {}s)",
                                    session.name, idle_secs
                                ));
                            }
                        }
                    }

                    // Auto-done: worker completed all todos but exited without calling
                    // worker-mark-done.sh. After AUTO_DONE_IDLE_SECS of inactivity, patrol
                    // writes DoneSignal::done_clean on the worker's behalf.
                    if !has_done
                        && progress.todos_total > 0
                        && progress.todos_completed >= progress.todos_total
                        && !report.done_workers.contains(&session.name)
                    {
                        if let Some(last_update) = progress.last_updated {
                            let idle_secs = (Utc::now() - last_update).num_seconds();
                            // ── Conservative ground-truth gate ──
                            // Ticking all todos is NOT proof the worker finished
                            // cleanly — it may have crashed mid-edit right after
                            // the last tick. The strongest available "finished
                            // cleanly" signal is the rmux session being GONE (the
                            // process actually exited), not merely idle at a
                            // prompt. Re-probe liveness via the SessionManager:
                            // `capture_pane` returns Err when the session/pane no
                            // longer resolves — the same dead-session idiom used
                            // by the pane stall + orphan passes above/below.
                            //
                            // Thresholds split by liveness: a GONE session is safe
                            // to record after AUTO_DONE_IDLE_SECS, but an ALIVE
                            // worker that just ticked its last todo is routinely
                            // deep in its verify step (build/test > 2 min, writes
                            // no progress ticks) — killing it that early aborts
                            // real work mid-verification. Alive sessions get the
                            // full file-stall bar (STALL_THRESHOLD_SECS), the same
                            // patience as the stall pass. (This branch was dead
                            // code until the progress-schema fix made these files
                            // parse — the 120s tuning never ran against a live
                            // worker.)
                            let session_gone = mgr.capture_pane(&session.name).await.is_err();
                            let idle_threshold = if session_gone {
                                AUTO_DONE_IDLE_SECS
                            } else {
                                STALL_THRESHOLD_SECS
                            };
                            if idle_secs > idle_threshold {
                                if session_gone {
                                    tracing::info!(
                                        worker = %session.name,
                                        idle_secs,
                                        "Auto-done: rmux session GONE — clean-exit confirmed"
                                    );
                                } else {
                                    tracing::warn!(
                                        worker = %session.name,
                                        idle_secs,
                                        "Auto-done HEURISTIC: session still alive but idle past \
                                         threshold with all todos ticked — proceeding (may have \
                                         stalled mid-edit; kill+auto-done as before)"
                                    );
                                    report.actions_taken.push(format!(
                                        "Auto-done HEURISTIC (session alive, idle): {} ({}/{} todos, idle {}s)",
                                        session.name,
                                        progress.todos_completed,
                                        progress.todos_total,
                                        idle_secs,
                                    ));
                                }
                                // Kill the still-live idle worker so it cannot keep
                                // editing files while we record its (un-trusted)
                                // state. N8: scope is NOT released here — it stays
                                // HELD until a real done_clean clears the gate, so
                                // no other worker can claim these files yet.
                                // A still-live worker must be stopped before we
                                // persist the heuristic signal. If the kill
                                // fails, claiming otherwise would allow it to
                                // keep editing behind patrol's recorded state.
                                if !session_gone {
                                    if let Err(error) = mgr.kill_session(&session.name).await {
                                        report.actions_taken.push(format!(
                                            "Auto-done held for {}: live worker kill FAILED ({error}); no signal written",
                                            session.name
                                        ));
                                        continue;
                                    }
                                }
                                // N8: the idle-heuristic NEVER claims done_clean
                                // on a silently-exited worker's behalf. Ticking
                                // todos + going idle is not ground truth — only a
                                // worker-written done-signal that survives the
                                // ground-truth gate is. We write Pending instead:
                                // it re-confirms next tick, preserves the contest
                                // mechanism, and — critically — does NOT release
                                // the scope claim as if the work were clean.
                                let reason = if session_gone {
                                    "auto-done HEURISTIC: todos completed + session gone — recorded PENDING (not clean; re-confirm next tick), scope HELD"
                                } else {
                                    "auto-done HEURISTIC: todos completed + idle past threshold (session still alive) — patrol killed the worker, recorded PENDING (not clean), scope HELD"
                                };
                                let mut signal =
                                    DoneSignal::new(&session.name, DoneStatus::Pending, reason);
                                signal.todos_total = progress.todos_total;
                                signal.todos_completed = progress.todos_completed;
                                match signal.write(&self.config.state_dir) {
                                    Ok(()) => {
                                        report.done_workers.push(session.name.clone());
                                        // Do NOT release scope here — the heuristic is
                                        // not proof of clean completion. Scope stays
                                        // held until a real done_clean clears the
                                        // ground-truth gate in the primary path.
                                        self.stall_detector.forget(&session.name);
                                        if let Some(oracle) = self.find_parent_oracle(
                                            &session.name,
                                            &oracle_sessions,
                                            &oracle_states,
                                        ) {
                                            let inbox = Inbox::for_oracle(
                                                &self.config.state_dir,
                                                &oracle.name,
                                            );
                                            // Mark the event sent under the SAME
                                            // key the main done pass will compute
                                            // for this signal next tick, so it
                                            // doesn't re-deliver it (only on a
                                            // successful push — a failure retries).
                                            if inbox
                                                .push(&InboxEvent::worker_done(
                                                    &session.name,
                                                    "pending",
                                                ))
                                                .is_ok()
                                            {
                                                if let Err(error) = record_inbox_event_sent(
                                                    &self.config.state_dir,
                                                    &session.name,
                                                    "done",
                                                    &format!(
                                                        "pending:{}",
                                                        signal.finished_at.timestamp()
                                                    ),
                                                ) {
                                                    report.actions_taken.push(format!(
                                                        "Auto-done event delivered for {}, but delivery marker FAILED ({error})",
                                                        session.name
                                                    ));
                                                }
                                            }
                                            match OracleState::read(
                                                &self.config.state_dir,
                                                &oracle.name,
                                            ) {
                                                Ok(Some(mut oracle_state)) => {
                                                    oracle_state.update_worker_status(
                                                        &session.name,
                                                        WorkerEntryStatus::Pending,
                                                    );
                                                    if let Err(error) = oracle_state
                                                        .write(&self.config.state_dir)
                                                    {
                                                        report.actions_taken.push(format!(
                                                            "Auto-done signal persisted for {}, but oracle-state update FAILED ({error})",
                                                            session.name
                                                        ));
                                                    }
                                                }
                                                Ok(None) => report.actions_taken.push(format!(
                                                    "Auto-done signal persisted for {}, but parent oracle state is missing",
                                                    session.name
                                                )),
                                                Err(error) => report.actions_taken.push(format!(
                                                    "Auto-done signal persisted for {}, but parent oracle state is unreadable ({error})",
                                                    session.name
                                                )),
                                            }
                                        }
                                        tracing::info!(
                                            worker = %session.name,
                                            todos = progress.todos_completed,
                                            idle_secs,
                                            "Patrol auto-done HEURISTIC: worker recorded PENDING (scope held)"
                                        );
                                        report.actions_taken.push(format!(
                                        "Auto-done HEURISTIC -> PENDING {} ({}/{} todos, idle {}s, scope held)",
                                        session.name,
                                        progress.todos_completed,
                                        progress.todos_total,
                                        idle_secs,
                                    ));
                                    }
                                    Err(write_error) => {
                                        // The worker was already killed above but the
                                        // PENDING signal could not be persisted. Without
                                        // a signal the worker is invisible: not in
                                        // done_workers, no inbox event, no oracle-state
                                        // update — yet its scope claim stays HELD,
                                        // blocking re-dispatch. Surface it loudly (error
                                        // log + report action) so the orphan is observable
                                        // instead of failing silently.
                                        tracing::error!(
                                            worker = %session.name,
                                            error = %write_error,
                                            "Patrol auto-done FAILED to write PENDING signal — \
                                             worker killed but scope HELD with no recorded signal"
                                        );
                                        report.actions_taken.push(format!(
                                            "Auto-done FAILED to write signal for {}: {} (worker killed, scope HELD, no signal recorded)",
                                            session.name, write_error
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // ── Deterministic worker reap (Task#6) ──
        // For each Closeable worker, reap (kill rmux session + release any
        // remaining locks) once the parent oracle has CONSUMED its worker_done
        // event (its inbox no longer carries one for this worker) OR the grace
        // window elapsed. Honest-done workers no longer linger as zombies, and
        // the reap is deterministic rather than gated on the idle/CPU heuristic.
        let live_session_names: std::collections::HashSet<&str> =
            sessions.iter().map(|s| s.name.as_str()).collect();
        for marker in WorkerCloseMarker::read_all(&self.config.state_dir)? {
            match self.v3_worker_attempt_state(&oracle_states, &marker.session) {
                Ok(Some(crate::mission::TaskAttemptState::Accepted)) => {}
                Ok(None) => {
                    report.actions_taken.push(format!(
                        "Held close marker and scope for {}: no exact V3 worker authority",
                        marker.session
                    ));
                    continue;
                }
                Ok(Some(state)) => {
                    report.actions_taken.push(format!(
                        "Held close marker and scope for {}: V3 attempt is {:?}, not Accepted",
                        marker.session, state
                    ));
                    continue;
                }
                Err(error) => {
                    report.actions_taken.push(format!(
                        "Held close marker and scope for {}: V3 authority failed closed ({error})",
                        marker.session
                    ));
                    continue;
                }
            }
            // Oracle ack = the worker_done event was drained from its inbox.
            // peek() lists what's still queued; absence after we pushed it ⇒
            // the oracle consumed it (its drain deletes the file).
            let oracle_acked = match &marker.oracle {
                Some(oracle_name) => {
                    let inbox = Inbox::for_oracle(&self.config.state_dir, oracle_name);
                    match inbox.peek() {
                        Ok(evts) => !evts.iter().any(|e| {
                            e.event_type == crate::inbox::EventType::WorkerDone
                                && e.payload.get("session").and_then(|v| v.as_str())
                                    == Some(marker.session.as_str())
                        }),
                        Err(error) => {
                            // An unreadable inbox is not an acknowledgement.
                            // Fail closed so storage corruption cannot authorize
                            // an early worker reap before the grace window.
                            report.actions_taken.push(format!(
                                "Held {}: oracle inbox acknowledgement unreadable ({error})",
                                marker.session
                            ));
                            false
                        }
                    }
                }
                // No known parent ⇒ rely solely on the grace window.
                None => false,
            };
            let closeable_secs = (Utc::now() - marker.since).num_seconds();
            if should_reap_closeable(oracle_acked, closeable_secs) {
                let scope_receipt =
                    match ScopeClaim::read_strict(&self.config.state_dir, &marker.session) {
                        Ok(receipt) => receipt,
                        Err(error) => {
                            report.actions_taken.push(format!(
                                "Held {}: exact scope receipt could not be read ({error})",
                                marker.session
                            ));
                            continue;
                        }
                    };
                // Kill the rmux session (no-op/Err if already gone) and release
                // any remaining scope lock, atomically from patrol's view.
                if live_session_names.contains(marker.session.as_str()) {
                    if let Err(error) = mgr.kill_session(&marker.session).await {
                        report.actions_taken.push(format!(
                            "Held scope/marker for {}: session reap failed ({error})",
                            marker.session
                        ));
                        continue;
                    }
                }
                if let Some(receipt) = &scope_receipt {
                    if let Err(error) = ScopeClaim::release_exact(&self.config.state_dir, receipt) {
                        report.actions_taken.push(format!(
                            "Reaped {} but retained close marker: exact scope release failed ({error})",
                            marker.session
                        ));
                        continue;
                    }
                }
                self.stall_detector.forget(&marker.session);
                if let Err(error) =
                    WorkerCloseMarker::remove(&self.config.state_dir, &marker.session)
                {
                    report.actions_taken.push(format!(
                        "Reaped {} but close marker removal FAILED ({error})",
                        marker.session
                    ));
                    continue;
                }
                if let Err(error) =
                    remove_inbox_event_markers(&self.config.state_dir, &marker.session)
                {
                    report.actions_taken.push(format!(
                        "Reaped {} but inbox marker cleanup FAILED ({error})",
                        marker.session
                    ));
                }
                let trigger = if oracle_acked {
                    "oracle ack'd"
                } else {
                    "grace elapsed"
                };
                tracing::info!(
                    worker = %marker.session,
                    trigger,
                    closeable_secs,
                    "Deterministic reap: honest-done worker closed"
                );
                report.actions_taken.push(format!(
                    "Reaped done_clean worker {} ({}, {}s closeable)",
                    marker.session, trigger, closeable_secs
                ));
            } else if !live_session_names.contains(marker.session.as_str()) {
                // Session already gone (e.g. the worker exited on its own before
                // the reap fired). Nothing to kill — just clear the marker + lock.
                match ScopeClaim::read_strict(&self.config.state_dir, &marker.session) {
                    Ok(Some(receipt)) => {
                        if let Err(error) =
                            ScopeClaim::release_exact(&self.config.state_dir, &receipt)
                        {
                            report.actions_taken.push(format!(
                                "Held close marker for {}: exact scope release failed ({error})",
                                marker.session
                            ));
                            continue;
                        }
                    }
                    Ok(None) => {}
                    Err(error) => {
                        report.actions_taken.push(format!(
                            "Held close marker for {}: scope receipt read failed ({error})",
                            marker.session
                        ));
                        continue;
                    }
                }
                if let Err(error) =
                    WorkerCloseMarker::remove(&self.config.state_dir, &marker.session)
                {
                    report.actions_taken.push(format!(
                        "Session {} was gone and scope released, but close marker removal FAILED ({error})",
                        marker.session
                    ));
                }
                if let Err(error) =
                    remove_inbox_event_markers(&self.config.state_dir, &marker.session)
                {
                    report.actions_taken.push(format!(
                        "Session {} was gone, but inbox marker cleanup FAILED ({error})",
                        marker.session
                    ));
                }
            }
        }

        // ── Scope-claim janitor ──
        // Only the done_clean path releases a worker's scope; failed /
        // blocked / contested workers and patrol's auto-done (PENDING, scope
        // deliberately HELD pending a real done_clean) leave the claim
        // behind. Once the owning session is DEAD that hold can never be
        // cleared by the worker itself — the files stay locked and every
        // re-dispatch over them bails on "Scope conflict" until a manual
        // `omega cleanup`. A dead owner cannot write, so releasing is safe.
        // Two guards against racing a spawn-in-progress (spawn-worker claims
        // scope an instant BEFORE its rmux session appears): require a
        // recorded done/blocked signal on disk AND a minimum claim age.
        const SCOPE_RELEASE_MIN_AGE_SECS: i64 = 300;
        for claim in ScopeClaim::read_all_strict(&self.config.state_dir)? {
            if live_session_names.contains(claim.session.as_str()) {
                continue;
            }
            if (Utc::now() - claim.claimed_at).num_seconds() < SCOPE_RELEASE_MIN_AGE_SECS {
                continue;
            }
            let has_done_signal = match DoneSignal::read(&self.config.state_dir, &claim.session) {
                Ok(signal) => signal.is_some(),
                Err(error) => {
                    report.actions_taken.push(format!(
                        "Held scope of dead session {}: done signal read failed ({error})",
                        claim.session
                    ));
                    continue;
                }
            };
            let has_signal = if has_done_signal {
                true
            } else {
                match WorkerBlocked::read(&self.config.state_dir, &claim.session) {
                    Ok(signal) => signal.is_some(),
                    Err(error) => {
                        report.actions_taken.push(format!(
                            "Held scope of dead session {}: blocked signal read failed ({error})",
                            claim.session
                        ));
                        continue;
                    }
                }
            };
            if has_signal {
                match self.v3_worker_attempt_state(&oracle_states, &claim.session) {
                    Ok(Some(crate::mission::TaskAttemptState::Accepted)) => {}
                    Ok(None) => {
                        report.actions_taken.push(format!(
                            "Held scope of dead session {}: no exact V3 worker authority",
                            claim.session
                        ));
                        continue;
                    }
                    Ok(Some(state)) => {
                        report.actions_taken.push(format!(
                            "Held scope of dead session {}: V3 attempt is {:?}, not Accepted",
                            claim.session, state
                        ));
                        continue;
                    }
                    Err(error) => {
                        report.actions_taken.push(format!(
                            "Held scope of dead session {}: V3 authority failed closed ({error})",
                            claim.session
                        ));
                        continue;
                    }
                }
                match ScopeClaim::release_exact(&self.config.state_dir, &claim) {
                    Ok(()) => report.actions_taken.push(format!(
                        "Released scope of dead Accepted V3 session {}",
                        claim.session
                    )),
                    Err(error) => report.actions_taken.push(format!(
                        "Held scope of dead session {}: release failed ({error})",
                        claim.session
                    )),
                }
            }
        }

        // ── Orphan detection: sessions with no done/progress and empty pane ──
        for session in &sessions {
            if session.role == SessionRole::Worker && !team_session_names.contains(&session.name) {
                let has_done = DoneSignal::read(&self.config.state_dir, &session.name)?.is_some();
                let has_progress =
                    crate::progress::ProgressInfo::read(&self.config.state_dir, &session.name)
                        .is_some();

                if !has_done && !has_progress && !report.orphaned_sessions.contains(&session.name) {
                    match mgr.capture_pane(&session.name).await {
                        Ok(content) => {
                            let trimmed = content.trim();
                            if trimmed.is_empty() || trimmed.lines().count() <= 1 {
                                report.orphaned_sessions.push(session.name.clone());
                            }
                        }
                        Err(_) => {
                            report.orphaned_sessions.push(session.name.clone());
                        }
                    }
                }
            }
        }

        // ── Oracle patrol: check done signals + registry cleanup ──
        self.patrol_oracles(&mgr, &sessions, &oracle_states, &mut report)
            .await?;

        // ── Orphan-worker sweep: workers whose done_clean oracle is gone ──
        // The cascade close above only fires while the oracle SESSION is still
        // alive to be reaped. When the oracle already closed (inline auto-close,
        // manual kill, crash-after-done) its leftover workers had NO reaper at
        // all — the 7-zombie dentistrygpt incident. Sweep them here.
        self.sweep_orphan_workers(&mgr, &sessions, &oracle_states, &mut report)
            .await?;

        // ── Oracle recovery: resurrect crashed-mid-mission oracles (guarded) ──
        if let Err(error) = self.resurrect_dead_oracles(&mut report).await {
            report.actions_taken.push(format!(
                "Oracle resurrection authority failed closed ({error})"
            ));
        }

        // ── Signal file watcher: detect new oracle result files ──
        match self.signal_watcher.poll() {
            Ok(new_signals) => {
                for (oracle_name, signal) in &new_signals {
                    report.actions_taken.push(format!(
                        "Signal file detected: {} (status: {:?})",
                        oracle_name, signal.status
                    ));
                }
            }
            Err(error) => report
                .actions_taken
                .push(format!("Oracle signal watcher failed closed ({error})")),
        }

        // ── State-dir GC (bounded, age-gated) ──
        self.gc_state_dir(&live_session_names, &mut report);

        self.log_patrol(&report)?;

        Ok(report)
    }

    /// Mission wall-clock breaker (Loop Engineering bounded-runtime). A running
    /// oracle with no closeable done signal gets a timeline note at the SOFT
    /// ceiling and an operator escalation at the HARD ceiling. Never a kill —
    /// the goal is to force a human to LOOK at a loop that has run unusually
    /// long, not to murder legitimately long work (L5). Soft is marker-gated so
    /// the timeline gets one note, not one per patrol tick.
    fn check_mission_wallclock(
        &self,
        session_name: &str,
        spawned_at: Option<DateTime<Utc>>,
        has_closeable_done: bool,
        report: &mut PatrolReport,
    ) {
        use crate::loop_guard::{self, EscalationReason};
        if has_closeable_done {
            return;
        }
        let Some(start) = spawned_at else {
            return;
        };
        let elapsed = (Utc::now() - start).num_seconds();
        let hrs = elapsed / 3_600;
        if elapsed >= loop_guard::HARD_WALLCLOCK_SECS {
            if loop_guard::escalate_to_human(
                &self.config.state_dir,
                session_name,
                EscalationReason::WallClock,
                &format!(
                    "running {}h with no closeable done signal (hard ceiling {}h)",
                    hrs,
                    loop_guard::HARD_WALLCLOCK_SECS / 3_600
                ),
            ) {
                report.actions_taken.push(format!(
                    "ESCALATED TO HUMAN: {} past hard wall-clock ceiling ({}h)",
                    session_name, hrs
                ));
            }
        } else if elapsed >= loop_guard::SOFT_WALLCLOCK_SECS {
            // One-shot timeline note (no operator ping at soft).
            let key = session_name.strip_prefix("oracle-").unwrap_or(session_name);
            let marker = self
                .config
                .state_dir
                .join(format!("{}.wallclock-soft", key));
            match crate::config::read_private_optional(&marker) {
                Ok(Some(_)) => {}
                Ok(None) => {
                    match crate::config::atomic_write_private(
                        &marker,
                        Utc::now().to_rfc3339().as_bytes(),
                    ) {
                        Ok(()) => loop_guard::MissionLog::event(
                            &self.config.state_dir,
                            session_name,
                            "wallclock",
                            &format!(
                                "mission running {}h (soft ceiling {}h) — still no closeable done signal",
                                hrs,
                                loop_guard::SOFT_WALLCLOCK_SECS / 3_600
                            ),
                        ),
                        Err(error) => report.actions_taken.push(format!(
                            "Wall-clock soft marker FAILED for {} ({error})",
                            session_name
                        )),
                    }
                }
                Err(error) => report.actions_taken.push(format!(
                    "Wall-clock soft marker unreadable for {} ({error})",
                    session_name
                )),
            }
        }
    }

    /// Patrol oracle sessions: check for done oracles, update registry, handle close.
    async fn patrol_oracles(
        &mut self,
        mgr: &SessionManager,
        sessions: &[crate::session::OmegaSession],
        oracle_states: &[OracleState],
        report: &mut PatrolReport,
    ) -> Result<()> {
        let live_names: Vec<String> = sessions.iter().map(|s| s.name.clone()).collect();
        // Read-only SNAPSHOT for the spawned_at lookups below. All mutations
        // (cleanup + status changes) are collected during the loop and applied
        // at the END under the `.oracle-registry.lock` — the old pattern
        // (load here, mutate across the kill_session awaits, save at the end,
        // no lock) clobbered any oracle a concurrent locked dispatch
        // registered mid-tick, erasing the spawned_at its freshness guard
        // depends on.
        let registry = OracleRegistry::load_strict(&self.config.state_dir)?;
        let mut status_changes: Vec<(String, OracleRegistryStatus)> = Vec::new();

        for session in sessions {
            if session.role != SessionRole::Oracle {
                continue;
            }

            // ── Loop Engineering: mission wall-clock breaker ──
            // A loop with no ceiling is the article's "cognitive surrender".
            // We do NOT kill long missions (L5 / ultracode: a 37h mission is
            // legitimate) — at the SOFT ceiling we drop a timeline note, and
            // only at the HARD ceiling do we ping the operator to come look.
            let spawned_at = registry
                .oracles
                .iter()
                .find(|e| e.session_name == session.name)
                .map(|e| e.spawned_at);
            let done_signal = match OracleDoneSignal::read(&self.config.state_dir, &session.name) {
                Ok(signal) => signal,
                Err(error) => {
                    report.actions_taken.push(format!(
                        "Held oracle {}: done signal read failed ({error})",
                        session.name
                    ));
                    continue;
                }
            };
            let has_closeable_done = done_signal
                .as_ref()
                .map(|done| done.is_closeable())
                .unwrap_or(false);
            self.check_mission_wallclock(&session.name, spawned_at, has_closeable_done, report);

            // Check oracle done signal
            if let Some(mut done) = done_signal {
                // ── Freshness guard (layered defense, stale-reap audit) ──
                // Oracle names recycle (Dead-purged registry entries make
                // next_oracle_name re-issue the base name) and the done.json
                // survives its session, so the signal on disk can belong to a
                // PREVIOUS mission. Acting on a stale signal killed brand-new
                // oracles (reap) and forged completions (upgrade). Date the
                // signal against the live session's registry spawned_at:
                // signal older than the session → ignore + warn. Unknown spawn
                // time (no registry entry) → ignore too: never act on a signal
                // you cannot date — the inline auto-close in `omega done` /
                // `omega progress` remains the primary close path.
                let spawned_at = registry
                    .oracles
                    .iter()
                    .find(|e| e.session_name == session.name)
                    .map(|e| e.spawned_at);
                let stale = signal_predates_session(done.finished_at, spawned_at);
                if stale {
                    tracing::warn!(
                        oracle = %session.name,
                        finished_at = %done.finished_at,
                        spawned_at = ?spawned_at,
                        "stale done signal predates session spawn — ignored (no upgrade, no reap)"
                    );
                    report.actions_taken.push(format!(
                        "Ignored stale done signal for {} (predates session spawn)",
                        session.name
                    ));
                }

                // ── L4 gate-pending upgrade (backstop for a missed progress tick) ──
                // `omega done` downgrades done_clean → Pending for EITHER of two
                // reasons: the plan is <100%, or the independent quality gate is
                // absent / not accepted. `omega progress` upgrades it back only
                // when BOTH have since been satisfied. If that tick was missed,
                // resolve it here — against the same two conditions, so the two
                // copies of the rule can never disagree.
                //
                // This used to read the raw `done` / `total` counters off the
                // progress JSON and ignore the gate entirely, so a mission
                // refused ONLY by the missing gate was auto-accepted within one
                // patrol cycle and the gate's refusal text was cleared with it.
                // The ledger is now the source of truth (the counters can be
                // stale or hand-edited, and `is_complete` also refuses a plan
                // holding a failed task), and an unreadable ledger or an absent
                // gate leaves the signal HONESTLY pending.
                if !stale && done.status == DoneStatus::Pending && done.gate_pending {
                    let plan_complete = match crate::oracle_todo::OracleTodo::load(
                        &self.config.state_dir,
                        &session.name,
                    ) {
                        Ok(todo) => todo.is_complete(),
                        Err(error) => {
                            report.actions_taken.push(format!(
                                "Held gate-pending oracle {}: plan read failed ({error})",
                                session.name
                            ));
                            false
                        }
                    };
                    // `None` = no gate result on disk at all, which is a
                    // REFUSAL here, never a pass (L5: an absent verdict is not
                    // an accepted one).
                    let gate_overall_pass = match crate::gate::GateResult::read(
                        &self.config.state_dir,
                        &session.name,
                    ) {
                        Ok(gate) => gate.map(|gate| gate.overall_pass),
                        Err(error) => {
                            report.actions_taken.push(format!(
                                "Held gate-pending oracle {}: gate result read failed ({error})",
                                session.name
                            ));
                            None
                        }
                    };
                    if may_upgrade_gate_pending(plan_complete, gate_overall_pass) {
                        done.status = DoneStatus::DoneClean;
                        done.pending_actions.clear();
                        done.gate_pending = false;
                        done.finished_at = Utc::now();
                        done.duration_secs =
                            (done.finished_at - done.started_at).num_seconds().max(0) as u64;
                        if let Err(error) = done.write(&self.config.state_dir) {
                            report.actions_taken.push(format!(
                                "Held gate-pending oracle {}: secure signal rewrite failed ({error})",
                                session.name
                            ));
                            continue;
                        }
                        // The notifier may have already reported the
                        // transient Pending state and written its per-path
                        // marker — invalidate it so the corrected
                        // done_clean is notified exactly once.
                        if let Err(error) = OracleDoneSignal::invalidate_notified_strict(
                            &self.config.state_dir,
                            &session.name,
                        ) {
                            report.actions_taken.push(format!(
                                "Oracle {} upgraded but notification re-arm failed ({error})",
                                session.name
                            ));
                            continue;
                        }
                        tracing::info!(
                            oracle = %session.name,
                            "L4 gate satisfied — pending upgraded to done_clean"
                        );
                        report.actions_taken.push(format!(
                            "L4 gate satisfied for {} — upgraded pending to done_clean",
                            session.name
                        ));
                    }
                }

                if !stale && done.is_closeable() {
                    let delivered_state = match self.require_delivered_oracle_authority(
                        oracle_states,
                        &session.name,
                        &done,
                    ) {
                        Ok(state) => state,
                        Err(error) => {
                            report.actions_taken.push(format!(
                                "Held closeable oracle {}: exact Delivered V3 authority failed ({error})",
                                session.name
                            ));
                            continue;
                        }
                    };
                    report.done_oracles.push(session.name.clone());
                    status_changes.push((session.name.clone(), OracleRegistryStatus::Done));

                    // ── Deterministic oracle reap (mirror of the worker reap) ──
                    // The inline auto-close in `omega done` / `omega progress`
                    // normally closes the session within seconds; if that was
                    // missed, reap here once the grace window elapsed so an
                    // honest-done oracle never lingers as a zombie.
                    let closeable_secs = (Utc::now() - done.finished_at).num_seconds();
                    if should_reap_oracle(done.is_closeable(), closeable_secs) {
                        // ── Cascade close — an oracle NEVER leaves orphan
                        // workers behind. The mission is declared done_clean
                        // and the grace elapsed: any worker session still
                        // alive is a zombie by definition (nothing will ever
                        // consume its output), so close them all WITH the
                        // oracle. The `omega done` close-gate refuses
                        // done_clean while a worker still runs, so a running
                        // worker here means an old-binary or hand-written
                        // signal — reaped too, loudly.
                        let lw = match crate::oracle_lifecycle::live_workers_of_oracle_strict(
                            &self.config.state_dir,
                            &session.name,
                            sessions,
                        ) {
                            Ok(workers) => workers,
                            Err(error) => {
                                report.actions_taken.push(format!(
                                    "Held oracle {}: strict V3 worker authority failed ({error})",
                                    session.name
                                ));
                                continue;
                            }
                        };
                        let state = &delivered_state;
                        let worker_names = lw.all();
                        let mut worker_receipts = Vec::with_capacity(worker_names.len());
                        let mut receipt_error = None;
                        for worker in &worker_names {
                            match ScopeClaim::read_strict(&self.config.state_dir, worker) {
                                Ok(receipt) => worker_receipts.push((worker.clone(), receipt)),
                                Err(error) => {
                                    receipt_error = Some(format!(
                                        "worker {worker} scope receipt failed: {error}"
                                    ));
                                    break;
                                }
                            }
                        }
                        let oracle_receipt =
                            match ScopeClaim::read_strict(&self.config.state_dir, &session.name) {
                                Ok(receipt) => receipt,
                                Err(error) => {
                                    receipt_error =
                                        Some(format!("oracle scope receipt failed: {error}"));
                                    None
                                }
                            };
                        if let Some(error) = receipt_error {
                            report.actions_taken.push(format!(
                                "Held oracle {} before cascade kill: {error}",
                                session.name
                            ));
                            continue;
                        }
                        let mut close_failed = false;
                        for (w, receipt) in worker_receipts {
                            if let Err(error) = mgr.kill_session(&w).await {
                                close_failed = true;
                                report.actions_taken.push(format!(
                                    "Held scope for {} and oracle {}: worker kill failed ({error})",
                                    w, session.name
                                ));
                                continue;
                            }
                            if let Some(receipt) = &receipt {
                                if let Err(error) =
                                    ScopeClaim::release_exact(&self.config.state_dir, receipt)
                                {
                                    close_failed = true;
                                    report.actions_taken.push(format!(
                                        "Worker {} closed but exact scope release failed ({error})",
                                        w
                                    ));
                                    continue;
                                }
                            }
                            self.stall_detector.forget(&w);
                            if let Err(error) =
                                WorkerCloseMarker::remove(&self.config.state_dir, &w)
                            {
                                close_failed = true;
                                report.actions_taken.push(format!(
                                    "Worker {} closed but close marker removal FAILED ({error})",
                                    w
                                ));
                                continue;
                            }
                            if let Err(error) =
                                remove_inbox_event_markers(&self.config.state_dir, &w)
                            {
                                report.actions_taken.push(format!(
                                    "Worker {} closed but inbox marker cleanup FAILED ({error})",
                                    w
                                ));
                            }
                            let was_running = lw.running.contains(&w);
                            tracing::info!(
                                oracle = %session.name, worker = %w, was_running,
                                "Cascade close: worker closed with its done_clean oracle"
                            );
                            report.actions_taken.push(format!(
                                "Cascade-closed worker {} with done_clean oracle {}{}",
                                w,
                                session.name,
                                if was_running {
                                    " (was still running!)"
                                } else {
                                    ""
                                }
                            ));
                        }
                        if close_failed {
                            continue;
                        }
                        if let Err(error) = mgr.kill_session(&session.name).await {
                            report.actions_taken.push(format!(
                                "Held oracle scope {}: oracle kill failed ({error})",
                                session.name
                            ));
                            continue;
                        }
                        // Release any scope claim the oracle still held —
                        // parity with the worker reap above (a gate-pending
                        // oracle skips the cmd_done-time release because its
                        // signal was not closeable yet, so the claim would
                        // otherwise leak until a manual cleanup).
                        if let Some(receipt) = &oracle_receipt {
                            if let Err(error) =
                                ScopeClaim::release_exact(&self.config.state_dir, receipt)
                            {
                                report.actions_taken.push(format!(
                                    "Oracle {} closed but exact scope release failed ({error})",
                                    session.name
                                ));
                                continue;
                            }
                        }
                        tracing::info!(
                            oracle = %session.name,
                            closeable_secs,
                            "Deterministic reap: done_clean oracle closed"
                        );
                        report.actions_taken.push(format!(
                            "Reaped done_clean oracle {} ({}s past finished_at)",
                            session.name, closeable_secs
                        ));
                        if let Err(error) =
                            self.maybe_trigger_curator(mgr, state, &session.name).await
                        {
                            report.actions_taken.push(format!(
                                "Oracle {} closed, curator trigger failed closed ({error})",
                                session.name
                            ));
                        }
                    }
                }
            }

            // Check oracle state for all-workers-terminal
            if let Some(oracle_state) = oracle_states
                .iter()
                .find(|state| state.oracle_name == session.name)
            {
                if oracle_state.all_workers_terminal()
                    && !report.done_oracles.contains(&session.name)
                {
                    // All workers are done but oracle hasn't written done signal yet — mark idle
                    status_changes.push((session.name.clone(), OracleRegistryStatus::Idle));
                }
            }
        }

        // Apply cleanup + the collected status changes atomically on a FRESH
        // reload under the registry lock, so a registration made by a
        // concurrent dispatch during this tick is merged, never lost.
        OracleRegistry::update_locked(&self.config.state_dir, |reg| {
            reg.cleanup(&live_names);
            for (name, status) in &status_changes {
                reg.mark_status(name, *status);
            }
        })?;
        Ok(())
    }

    /// Reap live WORKER sessions whose governing oracle is dead and whose
    /// mission is over (a closeable oracle done-signal past the grace).
    ///
    /// Parent resolution mirrors `live_workers_of_oracle`: the OracleState
    /// registry is authoritative; unregistered workers fall back to their
    /// project name. The fallback is vetoed while ANY oracle session of that
    /// project is live — a running mission may legitimately own them — and a
    /// worker with no signal to date is left alone (resurrect handles a
    /// crashed-mid-mission oracle; a signal-less orphan is its evidence).
    async fn sweep_orphan_workers(
        &mut self,
        mgr: &SessionManager,
        sessions: &[crate::session::OmegaSession],
        oracle_states: &[crate::oracle_lifecycle::OracleState],
        report: &mut PatrolReport,
    ) -> Result<()> {
        let live_oracles: std::collections::HashSet<&str> = sessions
            .iter()
            .filter(|s| s.role == SessionRole::Oracle)
            .map(|s| s.name.as_str())
            .collect();
        let live_oracle_projects: std::collections::HashSet<&str> = sessions
            .iter()
            .filter(|s| s.role == SessionRole::Oracle)
            .filter_map(|s| s.project.as_deref())
            .collect();
        let done_signals = OracleDoneSignal::read_all(&self.config.state_dir);

        for w in sessions.iter().filter(|s| s.role == SessionRole::Worker) {
            let registered_parent = oracle_states
                .iter()
                .find(|st| st.workers.iter().any(|e| e.session_name == w.name))
                .map(|st| st.oracle_name.clone());
            let (signal, governed_by) = match &registered_parent {
                Some(oracle_name) => {
                    if live_oracles.contains(oracle_name.as_str()) {
                        continue; // parent alive — cascade/ack paths own this
                    }
                    let signal = match OracleDoneSignal::read(&self.config.state_dir, oracle_name) {
                        Ok(signal) => signal,
                        Err(error) => {
                            report.actions_taken.push(format!(
                                "Orphan sweep held {}: parent done signal read failed ({error})",
                                w.name
                            ));
                            continue;
                        }
                    };
                    (signal, oracle_name.clone())
                }
                None => {
                    let Some(project) = w.project.as_deref() else {
                        continue;
                    };
                    if live_oracle_projects.contains(project) {
                        continue; // a live oracle of this project may own it
                    }
                    (
                        done_signals
                            .iter()
                            .find(|d| d.project == project && d.is_closeable())
                            .cloned(),
                        format!("project {project}"),
                    )
                }
            };
            let Some(sig) = signal else { continue };
            let finished_secs = (Utc::now() - sig.finished_at).num_seconds();
            if !should_reap_orphan(sig.is_closeable(), finished_secs) {
                continue;
            }
            match self.v3_worker_attempt_state(oracle_states, &w.name) {
                Ok(Some(crate::mission::TaskAttemptState::Accepted)) => {}
                Ok(None) => {
                    report.actions_taken.push(format!(
                        "Orphan sweep held {}: no exact V3 worker authority",
                        w.name
                    ));
                    continue;
                }
                Ok(Some(state)) => {
                    report.actions_taken.push(format!(
                        "Orphan sweep held {}: V3 attempt is {:?}, not Accepted",
                        w.name, state
                    ));
                    continue;
                }
                Err(error) => {
                    report.actions_taken.push(format!(
                        "Orphan sweep held {}: V3 authority failed closed ({error})",
                        w.name
                    ));
                    continue;
                }
            }
            let scope_receipt = match ScopeClaim::read_strict(&self.config.state_dir, &w.name) {
                Ok(receipt) => receipt,
                Err(error) => {
                    report.actions_taken.push(format!(
                        "Orphan sweep held {}: exact scope receipt failed ({error})",
                        w.name
                    ));
                    continue;
                }
            };
            if let Err(error) = mgr.kill_session(&w.name).await {
                report.actions_taken.push(format!(
                    "Orphan sweep held scope for {}: kill failed ({error})",
                    w.name
                ));
                continue;
            }
            if let Some(receipt) = &scope_receipt {
                if let Err(error) = ScopeClaim::release_exact(&self.config.state_dir, receipt) {
                    report.actions_taken.push(format!(
                        "Orphan {} closed but exact scope release failed ({error})",
                        w.name
                    ));
                    continue;
                }
            }
            self.stall_detector.forget(&w.name);
            if let Err(error) = WorkerCloseMarker::remove(&self.config.state_dir, &w.name) {
                report.actions_taken.push(format!(
                    "Orphan {} closed but close marker removal FAILED ({error})",
                    w.name
                ));
                continue;
            }
            if let Err(error) = remove_inbox_event_markers(&self.config.state_dir, &w.name) {
                report.actions_taken.push(format!(
                    "Orphan {} closed but inbox marker cleanup FAILED ({error})",
                    w.name
                ));
            }
            tracing::info!(
                worker = %w.name, governed_by = %governed_by, finished_secs,
                "Orphan sweep: worker closed (oracle gone, mission done_clean)"
            );
            report.actions_taken.push(format!(
                "Orphan sweep: closed worker {} ({} done_clean {}s ago, oracle session gone)",
                w.name, governed_by, finished_secs
            ));
        }
        Ok(())
    }

    /// Auto-resurrect oracles that crashed mid-mission — the install-time
    /// equivalent of an oracle-watchdog. Guarded against thrash: an oracle is
    /// only brought back if it has unfinished work (workers not all terminal AND
    /// no closeable done signal), its mission is still recent (phase changed
    /// within 24h), and we have not already tried within the last 5 minutes. A
    /// finished, abandoned, or stale-stopped oracle stays dead.
    async fn resurrect_dead_oracles(&self, report: &mut PatrolReport) -> Result<()> {
        let dispatcher = crate::dispatch::Dispatcher::new(
            SessionManager::connect_cached().await?,
            self.config.clone(),
        );
        for name in dispatcher.dead_oracles().await {
            let state = match OracleState::read(&self.config.state_dir, &name) {
                Ok(Some(s)) => s,
                Ok(None) => continue,
                Err(error) => {
                    report.actions_taken.push(format!(
                        "Oracle resurrection held for {name}: state unreadable ({error})"
                    ));
                    continue;
                }
            };
            // Finished → leave it dead.
            if state.all_workers_terminal() {
                continue;
            }
            // Never started → leave it dead. An oracle that registered ZERO
            // workers never decomposed a mission, so there is nothing to resume:
            // resurrecting it only replays the original (often malformed) dispatch
            // and spawns an empty oracle shell, which patrol then resurrects again
            // every 5 min — an infinite "empty session keeps reopening" loop.
            // (all_workers_terminal() is false for an empty worker list, so this
            // case is NOT caught above.)
            if state.workers.is_empty() {
                continue;
            }
            match OracleDoneSignal::read(&self.config.state_dir, &name) {
                Ok(Some(done)) if done.is_closeable() => continue,
                Ok(_) => {}
                Err(error) => {
                    report.actions_taken.push(format!(
                        "Oracle resurrection held for {name}: done signal unreadable ({error})"
                    ));
                    continue;
                }
            }
            // Abandoned (no activity in 24h) → leave it dead.
            if (Utc::now() - state.phase_entered_at).num_hours() > 24 {
                continue;
            }
            // Anti-thrash: don't retry within 5 minutes.
            let marker = self
                .config
                .state_dir
                .join(format!("oracle-{}.resurrect-attempt", name));
            let recently_tried = match crate::config::read_private_optional(&marker) {
                Ok(None) => false,
                Ok(Some(_)) => match std::fs::symlink_metadata(&marker)
                    .and_then(|metadata| metadata.modified())
                    .and_then(|modified| modified.elapsed().map_err(std::io::Error::other))
                {
                    Ok(age) => age.as_secs() < 300,
                    Err(error) => {
                        report.actions_taken.push(format!(
                            "Oracle resurrection held for {name}: attempt marker timestamp unreadable ({error})"
                        ));
                        continue;
                    }
                },
                Err(error) => {
                    report.actions_taken.push(format!(
                        "Oracle resurrection held for {name}: attempt marker unreadable ({error})"
                    ));
                    continue;
                }
            };
            if recently_tried {
                continue;
            }
            if let Err(error) =
                crate::config::atomic_write_private(&marker, Utc::now().to_rfc3339().as_bytes())
            {
                report.actions_taken.push(format!(
                    "Oracle resurrection held for {name}: attempt marker write FAILED ({error})"
                ));
                continue;
            }
            match dispatcher.resurrect_oracle(&name).await {
                Ok(crate::dispatch::ResurrectOutcome::Resurrected) => report.actions_taken.push(
                    format!("Resurrected crashed oracle {} (mission unfinished)", name),
                ),
                Ok(other) => report
                    .actions_taken
                    .push(format!("Oracle resurrection skipped for {name}: {other:?}")),
                Err(error) => report.actions_taken.push(format!(
                    "Oracle resurrection attempt FAILED for {name} ({error})"
                )),
            }
        }
        Ok(())
    }

    /// Self-improvement hook: when an oracle's done.json flips to a
    /// closeable status, spawn a curator worker that reads the trajectory
    /// + done.json and proposes NEW_SKILL / EDIT_SKILL / NEW_RULE /
    ///   NEW_MEMORY items. Output lands in
    ///   `~/.omega/state/curator/<oracle>-<timestamp>.md`.
    ///
    /// Idempotent: marker file `~/.omega/state/curator-triggered/<oracle>.flag`
    /// prevents re-trigger on subsequent patrol ticks.
    async fn maybe_trigger_curator(
        &self,
        mgr: &SessionManager,
        oracle_state: &OracleState,
        oracle_name: &str,
    ) -> Result<()> {
        crate::scope::validate_session_identity(oracle_name)?;
        let ledger = crate::mission_ledger::MissionLedger::open(
            self.config.state_dir.join("mission-engine-v3.sqlite3"),
        )?;
        oracle_state.require_ledger_authority(&ledger)?;
        let mission = ledger
            .mission_record(&oracle_state.mission_id)?
            .ok_or_else(|| anyhow::anyhow!("immutable mission is missing"))?;
        let projection = ledger
            .mission(&oracle_state.mission_id)?
            .ok_or_else(|| anyhow::anyhow!("mission projection is missing"))?;
        if projection.state != crate::mission::MissionState::Delivered {
            anyhow::bail!("curator dispatch requires exact Delivered V3 authority");
        }
        let signal = OracleDoneSignal::read(&self.config.state_dir, oracle_name)?
            .filter(|signal| signal.is_closeable())
            .ok_or_else(|| {
                anyhow::anyhow!("curator dispatch requires a strict closeable signal")
            })?;
        crate::orchestration::validate_oracle_done_signal_authority(
            &ledger,
            &oracle_state.mission_id,
            oracle_name,
            &signal,
        )?;
        let flag_dir = self.config.state_dir.join("curator-triggered");
        let flag = flag_dir.join(format!("{}.flag", oracle_name));
        {
            let _lock = crate::scope::lock_private_state_file(
                &self.config.state_dir,
                ".curator-trigger.lock",
            )?;
            if crate::config::read_private_optional(&flag)?.is_some() {
                return Ok(());
            }
        }
        std::fs::create_dir_all(&flag_dir)?;
        std::fs::create_dir_all(self.config.state_dir.join("curator"))?;

        let curator_session =
            crate::session::sanitize_session_name(&format!("curator-{}", oracle_name));
        let done_key = oracle_name.strip_prefix("oracle-").unwrap_or(oracle_name);
        let done_path = self
            .config
            .state_dir
            .join(format!("oracle-{}.done.json", done_key));
        let prompt = format!("/omega-curate {}", done_path.to_string_lossy());
        let agent =
            crate::agents::Agent::from_name(&self.config.agent_command).ok_or_else(|| {
                anyhow::anyhow!(
                    "unknown configured curator provider {:?}",
                    self.config.agent_command
                )
            })?;
        let working_dir = mission
            .working_dir
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("mission working directory is not UTF-8"))?;
        mgr.create_agent_session_with_opts(
            &curator_session,
            working_dir,
            agent,
            Some(&prompt),
            crate::agents::LaunchOptions::default(),
        )
        .await?;
        {
            let _lock = crate::scope::lock_private_state_file(
                &self.config.state_dir,
                ".curator-trigger.lock",
            )?;
            if crate::config::read_private_optional(&flag)?.is_none() {
                crate::config::atomic_write_private(&flag, Utc::now().to_rfc3339().as_bytes())?;
            }
        }
        tracing::info!(
            oracle = %oracle_name,
            curator = %curator_session,
            "curator dispatched through typed provider launch"
        );
        Ok(())
    }

    /// Bounded state-dir garbage collection. The spawn paths write per-session
    /// side files (`{name}.mcp.json`, `{name}.debug.log`), and the protocol
    /// leaves per-signal files behind (worker done.json, push-once `.sent`
    /// markers, resurrect/stuck markers, retired `-prev` done.json) — nothing
    /// deleted any of them automatically, so a live host accumulated 460+
    /// files. Every rule below is conservative: keyed on the owning session
    /// being DEAD and an age floor, so an in-flight spawn (side files written
    /// an instant before the rmux session appears) is never swept. Also
    /// migrates legacy double-prefixed oracle state files in passing (see
    /// `OracleState::state_key`).
    fn gc_state_dir(&self, live: &std::collections::HashSet<&str>, report: &mut PatrolReport) {
        const HOUR: u64 = 3_600;
        const DAY: u64 = 86_400;
        let dir = &self.config.state_dir;
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        let mut removed = 0usize;
        for entry in entries.flatten() {
            let path = entry.path();
            let name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };
            let age = file_age_secs(&path).unwrap_or(0);
            let dead_after =
                |session: &str, min_age: u64| !live.contains(session) && age >= min_age;

            // Legacy double-prefixed oracle state → migrate once (rename into
            // the canonical single-prefix name; drop it if the canonical file
            // already exists — the live binary has been writing there).
            if name.starts_with("oracle-oracle-") && name.ends_with(".state.json") {
                let target = dir.join(&name["oracle-".len()..]);
                if target.exists() {
                    let _ = std::fs::remove_file(&path);
                } else {
                    let _ = std::fs::rename(&path, &target);
                }
                continue;
            }

            let stale = if let Some(session) = name.strip_suffix(".mcp.json") {
                // One written per spawn (oracle AND worker), never read after
                // launch — the single largest garbage class.
                dead_after(session, HOUR)
            } else if let Some(session) = name.strip_suffix(".debug.log") {
                // Post-mortem value decays fast; keep a week.
                dead_after(session, 7 * DAY)
            } else if name.starts_with("worker-") && name.ends_with(".done.json") {
                // Long-consumed worker signals. The spawn-time clear protects a
                // re-dispatch immediately; this only bounds the pile.
                let session = &name["worker-".len()..name.len() - ".done.json".len()];
                dead_after(session, 7 * DAY)
            } else if name.ends_with(".resurrect-attempt") {
                // Pure anti-thrash stamp with a 5-minute window — a day-old
                // marker is garbage regardless of session state.
                age >= DAY
            } else if name.starts_with("oracle-")
                && (name.ends_with(".state.json") || name.ends_with(".progress.json"))
            {
                // Lifecycle state of a DEAD oracle. Resurrect abandons a
                // mission after 24h (phase_entered_at), so two-days-dead
                // state is a pure phantom: its only remaining effect was
                // feeding the stuck-alert cron endless "oracle bloqué"
                // pings for sessions the operator can't even see (216 such
                // files had accumulated by 2026-06-11). The .state.json
                // basename IS the session name (state_key strips the
                // oracle- prefix before formatting).
                let stem = name
                    .trim_end_matches(".state.json")
                    .trim_end_matches(".progress.json");
                dead_after(stem, 2 * DAY)
            } else if name.starts_with("oracle-")
                && (name.ends_with(".report.json")
                    || name.ends_with(".report.pdf")
                    || name.ends_with(".findings.md"))
            {
                // Delivered mission artifacts — same 14-day record window
                // as retired done.json signals.
                let stem = name
                    .trim_end_matches(".report.json")
                    .trim_end_matches(".report.pdf")
                    .trim_end_matches(".findings.md");
                dead_after(stem, 14 * DAY)
            } else if name.ends_with(".inbox.lock") || name.ends_with(".inbox.jsonl") {
                // Inbox side files are keyed on the FULL session name with an
                // extra oracle- prefix ("oracle-<session>.inbox.lock", giving
                // oracle-oracle-X for oracles) — strip one prefix to recover
                // the owning session.
                let stem = name
                    .trim_end_matches(".inbox.lock")
                    .trim_end_matches(".inbox.jsonl");
                let session = stem.strip_prefix("oracle-").unwrap_or(stem);
                dead_after(session, 7 * DAY)
            } else if let Some(stem) = name.strip_suffix(".stuck-alerted") {
                // The cron keys this on the state-file basename: `oracle-X`,
                // or legacy `oracle-oracle-X`. Live when either form maps to
                // a live session; dead → removing re-arms the alert for a
                // recycled name (mirrors OracleDoneSignal::clear).
                let owner_live = live.contains(stem)
                    || stem
                        .strip_prefix("oracle-")
                        .map(|s| live.contains(s))
                        .unwrap_or(false);
                !owner_live && age >= HOUR
            } else if name.starts_with("worker-") && name.ends_with(".sent") {
                // Push-once markers — the reap removes them; this catches leaks.
                let stem = &name["worker-".len()..];
                let session = stem
                    .strip_suffix(".done.sent")
                    .or_else(|| stem.strip_suffix(".blocked.sent"))
                    .unwrap_or(stem);
                dead_after(session, 7 * DAY)
            } else if name.starts_with("oracle-")
                && name.ends_with(".done.json")
                && is_retired_done_name(&name)
            {
                // Retired signals: only once DELIVERED (.notified sibling) and
                // past a 14-day record window — never destroy an unsent report.
                let notified = dir.join(format!("{}.notified", name));
                if notified.exists() && age >= 14 * DAY {
                    let _ = std::fs::remove_file(&notified);
                    true
                } else {
                    false
                }
            } else {
                false
            };

            if stale && std::fs::remove_file(&path).is_ok() {
                removed += 1;
            }
        }
        if removed > 0 {
            report
                .actions_taken
                .push(format!("GC: removed {} stale state file(s)", removed));
        }
    }

    fn find_parent_oracle<'a>(
        &self,
        worker_name: &str,
        oracles: &'a [&crate::session::OmegaSession],
        states: &[crate::oracle_lifecycle::OracleState],
    ) -> Option<&'a crate::session::OmegaSession> {
        // Authoritative: the oracle whose OracleState registry actually lists
        // this worker. This is correct even with multiple oracles per project.
        // `states` is read ONCE per tick by the caller (was an O(W×O) per-call
        // disk scan before).
        if let Some(state) = states
            .iter()
            .find(|s| s.workers.iter().any(|w| w.session_name == worker_name))
        {
            if let Some(o) = oracles.iter().find(|o| o.name == state.oracle_name) {
                return Some(o);
            }
        }
        // Fallback: first oracle of the same project (best-effort if no registry hit).
        let worker_session = crate::session::OmegaSession::classify(worker_name);
        let worker_project = worker_session.project.as_deref()?;
        oracles
            .iter()
            .find(|o| o.project.as_deref() == Some(worker_project))
            .copied()
    }

    pub async fn run_loop(&mut self, interval: Duration) -> Result<()> {
        tracing::info!(interval_secs = interval.as_secs(), "Patrol daemon started");
        loop {
            match self.run_once().await {
                Ok(report) => {
                    tracing::info!(
                        sessions = report.total_sessions,
                        done_workers = report.done_workers.len(),
                        stalled = report.stalled_workers.len(),
                        done_oracles = report.done_oracles.len(),
                        orphaned = report.orphaned_sessions.len(),
                        actions = report.actions_taken.len(),
                        "Patrol tick"
                    );
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Patrol tick failed");
                }
            }
            tokio::time::sleep(interval).await;
        }
    }

    fn log_patrol(&self, report: &PatrolReport) -> Result<()> {
        let log_line = format!(
            "[{}] sessions={} oracles={} workers={} done_w={} stalled={} blocked={} orphaned={} done_o={} actions={}\n",
            Utc::now().format("%Y-%m-%dT%H:%M:%S"),
            report.total_sessions,
            report.oracles,
            report.workers,
            report.done_workers.len(),
            report.stalled_workers.len(),
            report.blocked_workers.len(),
            report.orphaned_sessions.len(),
            report.done_oracles.len(),
            report.actions_taken.len(),
        );

        let log_path = self.config.logs_dir.join("patrol.log");
        std::fs::create_dir_all(&self.config.logs_dir)?;

        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;
        file.write_all(log_line.as_bytes())?;
        Ok(())
    }
}

/// Deterministic worker-close marker (Task#6). Written when a worker's
/// done_clean clears the ground-truth gate; consumed by the reap pass. Persisted
/// so a patrol restart still reaps a pending close instead of leaking a zombie.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct WorkerCloseMarker {
    session: String,
    /// Parent oracle to watch for the worker_done ack (inbox consumption).
    oracle: Option<String>,
    /// When the worker became Closeable — start of the grace window.
    since: chrono::DateTime<Utc>,
}

impl WorkerCloseMarker {
    fn path(state_dir: &std::path::Path, session: &str) -> std::path::PathBuf {
        state_dir.join(format!("worker-close-{}.json", session))
    }

    /// Write the marker once. Idempotent: if it already exists, keep the
    /// original `since` so the grace clock isn't reset every tick.
    fn ensure(state_dir: &std::path::Path, session: &str, oracle: Option<&str>) -> Result<()> {
        crate::scope::validate_session_identity(session)?;
        let path = Self::path(state_dir, session);
        let _lock = crate::scope::lock_private_state_file(state_dir, ".worker-close.lock")?;
        if crate::config::read_private_optional(&path)?.is_some() {
            return Ok(());
        }
        let marker = WorkerCloseMarker {
            session: session.to_string(),
            oracle: oracle.map(|s| s.to_string()),
            since: Utc::now(),
        };
        let content = serde_json::to_vec(&marker).context("serializing worker close marker")?;
        crate::config::atomic_write_private(&path, &content)
            .with_context(|| format!("writing worker close marker for {session}"))
    }

    fn read_all(state_dir: &std::path::Path) -> Result<Vec<WorkerCloseMarker>> {
        let mut out = Vec::new();
        let entries = match std::fs::read_dir(state_dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(out),
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("reading patrol state dir {}", state_dir.display()))
            }
        };
        for entry in entries {
            let entry = entry.context("reading worker close marker directory entry")?;
            let p = entry.path();
            let Some(name) = p.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if !name.starts_with("worker-close-") || !name.ends_with(".json") {
                continue;
            }
            let content = crate::config::read_private_optional_string(&p)?
                .with_context(|| format!("worker close marker disappeared: {}", p.display()))?;
            let marker: WorkerCloseMarker = serde_json::from_str(&content)
                .with_context(|| format!("parsing worker close marker {}", p.display()))?;
            crate::scope::validate_session_identity(&marker.session)?;
            if Self::path(state_dir, &marker.session) != p {
                anyhow::bail!(
                    "worker close marker filename/session mismatch at {}",
                    p.display()
                );
            }
            out.push(marker);
        }
        Ok(out)
    }

    fn remove(state_dir: &std::path::Path, session: &str) -> Result<()> {
        crate::scope::validate_session_identity(session)?;
        let path = Self::path(state_dir, session);
        let _lock = crate::scope::lock_private_state_file(state_dir, ".worker-close.lock")?;
        if crate::config::read_private_optional(&path)?.is_none() {
            return Ok(());
        }
        std::fs::remove_file(&path)
            .with_context(|| format!("removing worker close marker {}", path.display()))
    }
}

/// L4 gate-pending upgrade predicate (pure + testable). Patrol may rewrite a
/// `gate_pending` signal back to `done_clean` only when BOTH refusals `omega
/// done` can record have since been lifted: the plan is genuinely complete
/// (`OracleTodo::is_complete`, not the raw counters) AND the independent
/// quality gate reports `overall_pass`.
///
/// The gate argument is an `Option` on purpose, because ABSENT and FAILED are
/// two different states that must not be collapsed at the call site: `None` is
/// "no `GateResult` on disk", `Some(false)` is "the gate ran and refused", and
/// only `Some(true)` is an accepted verdict. Both refusals leave the signal
/// pending, which is the whole point — a `done_clean` nobody verified is worse
/// than an honest pending that waits (R-VERIFY, L5).
fn may_upgrade_gate_pending(plan_complete: bool, gate_overall_pass: Option<bool>) -> bool {
    plan_complete && gate_overall_pass == Some(true)
}

/// Reap predicate (Task#6, pure + testable). Given whether the parent oracle
/// has consumed the worker_done event (`oracle_acked`) and how long the worker
/// has been Closeable, decide whether to reap NOW. Reap when the oracle ack'd OR
/// the bounded grace window elapsed — whichever first.
fn should_reap_closeable(oracle_acked: bool, closeable_secs: i64) -> bool {
    oracle_acked || closeable_secs >= WORKER_CLOSE_GRACE_SECS
}

/// Oracle reap predicate (pure + testable). An oracle is reaped only when its
/// done signal is closeable (done_clean, no pending actions) AND the grace
/// window since `finished_at` has elapsed — the grace gives the inline
/// auto-close (and the done-notifier cron) time to act first.
fn should_reap_oracle(closeable: bool, secs: i64) -> bool {
    closeable && secs >= ORACLE_CLOSE_GRACE_SECS
}

/// Orphan-worker predicate (pure + testable). A live worker whose governing
/// oracle session is GONE is reaped only when that oracle's mission is over
/// (closeable done signal) AND `finished_at` sits inside the reap WINDOW:
/// at least the generous grace has elapsed (so a same-name re-dispatch, which
/// clears the stale signal first, can never race the sweep) and no more than
/// [`ORPHAN_SIGNAL_MAX_AGE_SECS`] has elapsed.
///
/// The ceiling is what makes this safe on the project-match branch. Matching by
/// project alone means the signal was NOT necessarily written by this worker's
/// parent, so "grace elapsed" cannot imply "this signal governs this worker" —
/// an ancient done_clean stayed eligible forever and reaped brand-new workers.
/// A real orphan is swept within grace plus one tick; past the ceiling, a live
/// worker is by definition a newer one that this mission never dispatched.
fn should_reap_orphan(closeable: bool, finished_secs: i64) -> bool {
    closeable && (ORPHAN_WORKER_GRACE_SECS..=ORPHAN_SIGNAL_MAX_AGE_SECS).contains(&finished_secs)
}

/// Freshness guard predicate (pure + testable). A done signal whose
/// `finished_at` predates the live session's spawn belongs to a PREVIOUS
/// mission that recycled the name — patrol must never upgrade or reap on it.
/// Unknown spawn time (no registry entry for the session) is treated as stale
/// too: never act on a signal you cannot date. Dispatch registers spawned_at
/// via reserve_oracle and resurrect via register_resurrected, so a live
/// OmegaOS-launched oracle always has one; the conservative default only
/// affects hand-made sessions, where killing would be worse than lingering.
fn signal_predates_session(
    finished_at: chrono::DateTime<Utc>,
    session_spawned_at: Option<chrono::DateTime<Utc>>,
) -> bool {
    match session_spawned_at {
        Some(spawned_at) => finished_at < spawned_at,
        None => true,
    }
}

/// Worker-signal freshness predicate (pure + testable, the worker twin of
/// `signal_predates_session`). A done.json whose `finished_at` predates the
/// worker's `dispatched_at` belongs to a PREVIOUS mission that recycled the
/// deterministic worker name — acting on it insta-finishes (and reaps) the
/// new worker. Unlike the oracle guard, an UNKNOWN dispatch time is treated
/// as FRESH: workers without a registry entry (hand-spawned) have no other
/// done-delivery path, so dropping their signal would silence them entirely.
fn worker_signal_is_stale(
    finished_at: chrono::DateTime<Utc>,
    dispatched_at: Option<chrono::DateTime<Utc>>,
) -> bool {
    matches!(dispatched_at, Some(d) if finished_at < d)
}

/// Push-once markers for the per-tick inbox pushes. Patrol re-detects the
/// same done/blocked file every tick while it exists; the marker records the
/// content key (status + signal timestamp) of the event last pushed for a
/// session, so the event reaches the oracle exactly once per signal. A new
/// or upgraded signal carries a different key and re-arms automatically —
/// no coordination with the spawn-time stale-signal clear is needed.
fn event_sent_path(state_dir: &std::path::Path, session: &str, kind: &str) -> std::path::PathBuf {
    state_dir.join(format!("worker-{}.{}.sent", session, kind))
}

fn inbox_event_already_sent(
    state_dir: &std::path::Path,
    session: &str,
    kind: &str,
    key: &str,
) -> Result<bool> {
    let path = event_sent_path(state_dir, session, kind);
    Ok(crate::config::read_private_optional_string(&path)?
        .is_some_and(|content| content.trim() == key))
}

fn record_inbox_event_sent(
    state_dir: &std::path::Path,
    session: &str,
    kind: &str,
    key: &str,
) -> Result<()> {
    crate::config::atomic_write_private(&event_sent_path(state_dir, session, kind), key.as_bytes())
}

fn remove_inbox_event_markers(state_dir: &std::path::Path, session: &str) -> Result<()> {
    for kind in ["done", "blocked"] {
        let path = event_sent_path(state_dir, session, kind);
        if crate::config::read_private_optional(&path)?.is_some() {
            std::fs::remove_file(&path)
                .with_context(|| format!("removing inbox delivery marker {}", path.display()))?;
        }
    }
    Ok(())
}

/// Age of a file in seconds via mtime — `None` when unreadable (the GC then
/// treats it as age 0, i.e. never deletes on an unknown clock).
fn file_age_secs(path: &std::path::Path) -> Option<u64> {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.elapsed().ok())
        .map(|d| d.as_secs())
}

/// True for a RETIRED oracle signal — `oracle-<key>-prev<ts>.done.json`, the
/// rename `OracleDoneSignal::clear` performs on an un-notified signal. The
/// `<ts>` digits requirement keeps a project whose name merely contains
/// "-prev" out of the GC's reach.
fn is_retired_done_name(name: &str) -> bool {
    let Some(stem) = name.strip_suffix(".done.json") else {
        return false;
    };
    match stem.rfind("-prev") {
        Some(i) => {
            let ts = &stem[i + "-prev".len()..];
            !ts.is_empty() && ts.chars().all(|c| c.is_ascii_digit())
        }
        None => false,
    }
}

/// The ONLY predicate that may contest a worker's `done_clean`.
///
/// A check that RAN and refuted the claim is a concrete fabrication. A check
/// that could not run — no repo root, `git` absent, a command that was never
/// predeclared — proves nothing and must not contest anything: it still fails
/// the verdict, so the work falls through to `Pending` with its scope held.
/// This used to be `any(|c| !c.passed)`, which could not tell those apart and
/// escalated four unverifiable checks to a human as fabrications on mission
/// OmegaOS-m-8fe7d35df5bf.
#[cfg(test)]
fn verdict_contests_worker(verdict: &crate::done::GroundTruthVerdict) -> bool {
    verdict.checks.iter().any(|c| c.outcome.is_contradicted())
}

/// Resolve the repo root the ground-truth gate must check a worker's artifacts
/// against.
///
/// There were two project registries and the gate read the wrong one.
/// `~/.omega/projects.json` (`ProjectRegistry`) is the live one — 25 projects on
/// this machine — while `config.projects` from `~/.omega/config.toml` is
/// literally `projects = []`. Reading only the config handed the gate a `None`
/// root, every git/file check went unverified, and an unverifiable check was
/// then branded a fabrication. The registry is consulted FIRST; the config
/// remains the fallback so nothing regresses where it IS populated.
fn resolve_repo_root(config: &OmegaConfig, project: &str) -> Option<std::path::PathBuf> {
    resolve_repo_root_in(
        &crate::project_manager::ProjectRegistry::registry_path(),
        config,
        project,
    )
}

/// Testable core of [`resolve_repo_root`]: the registry path is explicit so a
/// test never reads (or writes) the machine's real `~/.omega/projects.json`.
fn resolve_repo_root_in(
    registry_path: &std::path::Path,
    config: &OmegaConfig,
    project: &str,
) -> Option<std::path::PathBuf> {
    crate::project_manager::ProjectRegistry::load_from(registry_path)
        .find(project)
        .map(|managed| managed.path.clone())
        .or_else(|| config.find_project(project).map(|pc| pc.path.clone()))
}

/// Detect a fatal, non-recoverable agent error in a session's pane output — the
/// agent is stuck on an error rather than working or idle. Only the tail (the
/// live error, not old scrollback) is inspected. A content-filter block and a
/// hard API error qualify; a line that says it is retrying does not. The
/// returned string is a short reason for the oracle's inbox.
fn detect_fatal_agent_error(content: &str) -> Option<&'static str> {
    let tail: String = content.lines().rev().take(8).collect::<Vec<_>>().join("\n");
    if tail.contains("content filtering policy") || tail.contains("Output blocked by content") {
        Some("content-filter block")
    } else if tail.contains("API Error") && !tail.contains("retry") && !tail.contains("Retrying") {
        Some("API error")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_content_filter_block() {
        let pane = "working\nAPI Error: Output blocked by content filtering policy\n❯";
        assert_eq!(detect_fatal_agent_error(pane), Some("content-filter block"));
    }

    #[test]
    fn detects_hard_api_error() {
        assert_eq!(
            detect_fatal_agent_error("boom\nAPI Error: 500 internal\n❯"),
            Some("API error")
        );
    }

    #[test]
    fn ignores_retrying_and_normal_output() {
        assert_eq!(
            detect_fatal_agent_error("API Error: 529 overloaded, Retrying in 5s"),
            None
        );
        assert_eq!(detect_fatal_agent_error("just working on it\n❯"), None);
    }

    #[test]
    fn gate_upgrade_needs_complete_plan_and_passing_gate() {
        // The ONLY accepting shape: the plan is 100% and the independent gate
        // ran and passed.
        assert!(may_upgrade_gate_pending(true, Some(true)));
    }

    #[test]
    fn gate_upgrade_refused_when_gate_absent() {
        // The defect this predicate exists to close: patrol used to read only
        // the plan counters, so a mission refused ONLY because no GateResult
        // was ever written was auto-accepted on the next cycle.
        assert!(!may_upgrade_gate_pending(true, None));
    }

    #[test]
    fn gate_upgrade_refused_when_gate_failed() {
        // The gate ran and said no. An explicit refusal is at least as binding
        // as a missing one.
        assert!(!may_upgrade_gate_pending(true, Some(false)));
    }

    #[test]
    fn gate_upgrade_refused_when_plan_incomplete_whatever_the_gate() {
        // The plan condition is independent: no gate verdict rescues an
        // unfinished (or failed, per `is_complete`) plan.
        assert!(!may_upgrade_gate_pending(false, Some(true)));
        assert!(!may_upgrade_gate_pending(false, Some(false)));
        assert!(!may_upgrade_gate_pending(false, None));
    }

    #[test]
    fn orphan_sweep_needs_closeable_signal_and_grace() {
        // No closeable signal → never reap, however old (resurrect's domain).
        assert!(!should_reap_orphan(false, 0));
        assert!(!should_reap_orphan(false, ORPHAN_WORKER_GRACE_SECS * 10));
        // Closeable but inside the grace → wait (re-dispatch race window).
        assert!(!should_reap_orphan(true, 0));
        assert!(!should_reap_orphan(true, ORPHAN_WORKER_GRACE_SECS - 1));
        // Closeable + grace elapsed → reap.
        assert!(should_reap_orphan(true, ORPHAN_WORKER_GRACE_SECS));
        assert!(should_reap_orphan(true, ORPHAN_WORKER_GRACE_SECS + 600));
    }

    #[test]
    fn orphan_sweep_ignores_an_ancient_signal_that_never_governed_the_worker() {
        // Regression, measured 2026-08-04. `oracle-OmegaOS-4.done.json` was
        // done_clean and 3_185_365s (36.9 days) old. On the project-match branch
        // it kept matching every newly spawned `OmegaOS-worker-*`, and the sweep
        // closed three README workers within 60s of spawn — before any of them
        // could write a file or a done signal. Grace-elapsed alone said "reap".
        const INCIDENT_AGE_SECS: i64 = 3_185_365;
        const _: () = assert!(INCIDENT_AGE_SECS > ORPHAN_SIGNAL_MAX_AGE_SECS);
        assert!(
            !should_reap_orphan(true, INCIDENT_AGE_SECS),
            "a 36-day-old done signal must never authorize reaping a live worker"
        );

        // The window is closed at the top and stays open right up to it, so a
        // genuine orphan (swept within grace + one tick) is still reaped.
        assert!(should_reap_orphan(true, ORPHAN_SIGNAL_MAX_AGE_SECS));
        assert!(!should_reap_orphan(true, ORPHAN_SIGNAL_MAX_AGE_SECS + 1));
    }

    #[test]
    fn reap_fires_on_oracle_ack_or_grace() {
        // Task#6 reap predicate: reap as soon as the oracle ack's, regardless of
        // how little time has elapsed.
        assert!(should_reap_closeable(true, 0));
        assert!(should_reap_closeable(true, WORKER_CLOSE_GRACE_SECS - 1));
        // Without an ack, reap only after the bounded grace window elapses.
        assert!(!should_reap_closeable(false, 0));
        assert!(!should_reap_closeable(false, WORKER_CLOSE_GRACE_SECS - 1));
        assert!(should_reap_closeable(false, WORKER_CLOSE_GRACE_SECS));
        assert!(should_reap_closeable(false, WORKER_CLOSE_GRACE_SECS + 10));
    }

    #[test]
    fn oracle_reap_fires_only_when_closeable_and_grace_elapsed() {
        // Mirrors should_reap_closeable: a non-closeable oracle is NEVER reaped,
        // no matter how long it has been finished.
        assert!(!should_reap_oracle(false, 0));
        assert!(!should_reap_oracle(false, ORACLE_CLOSE_GRACE_SECS + 600));
        // Closeable but inside the grace window — give the inline auto-close
        // a chance first.
        assert!(!should_reap_oracle(true, 0));
        assert!(!should_reap_oracle(true, ORACLE_CLOSE_GRACE_SECS - 1));
        // Closeable + grace elapsed → reap.
        assert!(should_reap_oracle(true, ORACLE_CLOSE_GRACE_SECS));
        assert!(should_reap_oracle(true, ORACLE_CLOSE_GRACE_SECS + 10));
    }

    #[test]
    fn stale_signal_predates_session_guard() {
        let spawn = Utc::now();
        // Signal from a PRIOR mission (finished before this session spawned)
        // → stale: no reap, no gate-pending upgrade.
        assert!(signal_predates_session(
            spawn - chrono::Duration::hours(3),
            Some(spawn)
        ));
        assert!(signal_predates_session(
            spawn - chrono::Duration::seconds(1),
            Some(spawn)
        ));
        // Signal written BY this session (at or after spawn) → fresh.
        assert!(!signal_predates_session(spawn, Some(spawn)));
        assert!(!signal_predates_session(
            spawn + chrono::Duration::seconds(30),
            Some(spawn)
        ));
        // Unknown spawn time (no registry entry) → conservatively stale:
        // never kill a session you cannot date.
        assert!(signal_predates_session(Utc::now(), None));
    }

    #[test]
    fn worker_stale_signal_guard() {
        let dispatch = Utc::now();
        // Predecessor's signal (finished before this dispatch) → stale.
        assert!(worker_signal_is_stale(
            dispatch - chrono::Duration::hours(2),
            Some(dispatch)
        ));
        // Signal written by THIS dispatch → fresh.
        assert!(!worker_signal_is_stale(
            dispatch + chrono::Duration::seconds(30),
            Some(dispatch)
        ));
        // Unknown dispatch time (hand-spawned worker, no registry entry) →
        // FRESH: the opposite default to the oracle guard, because dropping
        // the signal would break done delivery for unregistered workers.
        assert!(!worker_signal_is_stale(Utc::now(), None));
    }

    #[test]
    fn inbox_event_markers_are_content_keyed() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        // Nothing sent yet.
        assert!(!inbox_event_already_sent(dir, "w1", "done", "done_clean:100").unwrap());
        record_inbox_event_sent(dir, "w1", "done", "done_clean:100").unwrap();
        // Same signal → already sent (no per-tick re-push).
        assert!(inbox_event_already_sent(dir, "w1", "done", "done_clean:100").unwrap());
        // Upgraded / new signal (different key) → re-armed.
        assert!(!inbox_event_already_sent(dir, "w1", "done", "done_clean:200").unwrap());
        assert!(!inbox_event_already_sent(dir, "w1", "done", "pending:100").unwrap());
        // Kinds are independent.
        assert!(!inbox_event_already_sent(dir, "w1", "blocked", "100").unwrap());
        remove_inbox_event_markers(dir, "w1").unwrap();
        assert!(!inbox_event_already_sent(dir, "w1", "done", "done_clean:100").unwrap());
    }

    #[test]
    fn retired_done_name_matcher() {
        assert!(is_retired_done_name(
            "oracle-OmegaOS-prev1765432100.done.json"
        ));
        // A live signal is never "retired", even for a project containing -prev.
        assert!(!is_retired_done_name("oracle-OmegaOS.done.json"));
        assert!(!is_retired_done_name("oracle-x-prevention.done.json"));
        assert!(!is_retired_done_name("oracle-x-prev.done.json"));
    }

    #[test]
    fn close_marker_is_idempotent_keeps_since() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        WorkerCloseMarker::ensure(dir, "worker-x", Some("oracle-X")).unwrap();
        let first = WorkerCloseMarker::read_all(dir).unwrap();
        assert_eq!(first.len(), 1);
        let since0 = first[0].since;
        // Re-ensure must NOT reset `since` (grace clock stability).
        WorkerCloseMarker::ensure(dir, "worker-x", Some("oracle-X")).unwrap();
        let second = WorkerCloseMarker::read_all(dir).unwrap();
        assert_eq!(second.len(), 1);
        assert_eq!(second[0].since, since0);
        WorkerCloseMarker::remove(dir, "worker-x").unwrap();
        assert!(WorkerCloseMarker::read_all(dir).unwrap().is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn close_marker_reader_rejects_symlink_and_hardlink_authority() {
        let temp = tempfile::tempdir().unwrap();
        let state = temp.path();
        let target = state.join("target.json");
        std::fs::write(
            &target,
            serde_json::to_vec(&WorkerCloseMarker {
                session: "worker-x".to_string(),
                oracle: None,
                since: Utc::now(),
            })
            .unwrap(),
        )
        .unwrap();
        let marker = WorkerCloseMarker::path(state, "worker-x");
        std::os::unix::fs::symlink(&target, &marker).unwrap();
        assert!(WorkerCloseMarker::read_all(state).is_err());
        std::fs::remove_file(&marker).unwrap();
        std::fs::hard_link(&target, &marker).unwrap();
        assert!(WorkerCloseMarker::read_all(state).is_err());
    }

    #[test]
    fn ignores_stale_error_in_scrollback() {
        let mut lines = vec!["API Error: Output blocked by content filtering policy"];
        lines.extend(std::iter::repeat_n("normal output line", 20));
        assert_eq!(detect_fatal_agent_error(&lines.join("\n")), None);
    }

    fn done_citing_sha(sha: &str) -> crate::done::DoneSignal {
        let mut done = crate::done::DoneSignal::stub("Proj-worker-x", DoneStatus::DoneClean);
        done.todos_total = 1;
        done.todos_completed = 1;
        done.artifacts.push(crate::done::DoneArtifact::GitSha {
            sha: sha.to_string(),
            branch: None,
        });
        done
    }

    /// MANDATORY TEST 1 (patrol half). A real SHA that could not be looked up
    /// (no repo root) must NOT contest the worker: patrol falls through to the
    /// `!verdict.passes` branch, which is `Pending` with the scope held — never
    /// `Failed`, never a contested-fabrication escalation.
    #[test]
    fn unverifiable_check_never_contests_a_worker() {
        let done = done_citing_sha("0123456789abcdef0123456789abcdef01234567");
        let verdict = crate::done::verify_done_against_repo(&done, None);
        assert!(
            !verdict_contests_worker(&verdict),
            "an unrun check was branded a fabrication: {:?}",
            verdict.failures
        );
        // The classification patrol then applies, verbatim from the call site.
        let effective = if verdict_contests_worker(&verdict) {
            DoneStatus::Failed
        } else if !verdict.passes {
            DoneStatus::Pending
        } else {
            done.status
        };
        assert_eq!(effective, DoneStatus::Pending);
    }

    /// MANDATORY TEST 2 (patrol half). The detector is NOT weakened: a bogus
    /// SHA looked up in a REAL repo still contests the worker and still lands
    /// on `Failed`.
    #[test]
    fn contradicted_check_still_contests_a_worker() {
        let temp = tempfile::tempdir().unwrap();
        let status = std::process::Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(temp.path())
            .status()
            .expect("git init");
        assert!(status.success());
        let done = done_citing_sha("0123456789abcdef0123456789abcdef01234567");
        let verdict = crate::done::verify_done_against_repo(&done, Some(temp.path()));
        assert!(
            verdict_contests_worker(&verdict),
            "a proven fabrication must still be caught: {:?}",
            verdict.failures
        );
        let effective = if verdict_contests_worker(&verdict) {
            DoneStatus::Failed
        } else if !verdict.passes {
            DoneStatus::Pending
        } else {
            done.status
        };
        assert_eq!(effective, DoneStatus::Failed);
    }

    /// MANDATORY TEST 3. The registry (`projects.json`) is the real one, and
    /// `config.projects` is empty on a real machine — that split-brain is why
    /// `repo_root` was `None` in the first place. Uses a tempdir registry: the
    /// machine's own `~/.omega/projects.json` is never read or written
    /// (project_manager.rs:429 records the day a test clobbered it).
    #[test]
    fn repo_root_resolves_from_the_projects_registry_when_config_is_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let registry_path = tmp.path().join("projects.json");
        let project_path = tmp.path().join("Station/SideBusiness/OmegaOS");
        std::fs::create_dir_all(&project_path).unwrap();
        std::fs::write(
            &registry_path,
            serde_json::json!({
                "projects": [{
                    "name": "OmegaOS",
                    "path": project_path,
                    "telegram_topic_id": null,
                    "oracle_session": null,
                    "git_email": null,
                    "created_at": "2026-08-05T00:00:00Z"
                }]
            })
            .to_string(),
        )
        .unwrap();

        let config = OmegaConfig::default();
        assert!(
            config.find_project("OmegaOS").is_none(),
            "premise: config.projects does not carry the project"
        );
        assert_eq!(
            resolve_repo_root_in(&registry_path, &config, "OmegaOS"),
            Some(project_path),
            "the real registry must be consulted first"
        );
        // Unknown project, empty config → still None (no invented root).
        assert_eq!(
            resolve_repo_root_in(&registry_path, &config, "NotAProject"),
            None
        );
    }
}

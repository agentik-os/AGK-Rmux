//! Executor — the autonomous Driver. Selects work ONLY via
//! PlanTracker::ready_steps (the LLM is never asked "what next?"), spawns one
//! worker per step through a WorkerRuntime, and gates every completion through
//! the Guardian before marking a step Done.

use crate::agents::Agent;
use crate::done::{DoneSignal, DoneStatus};
use crate::guardian::{Guardian, Verdict};
use crate::planner::{
    PlanStep, PlanTracker, StepStatus, WorkerDispatchBinding, WORKER_DISPATCH_SCHEMA_VERSION,
};
use crate::scope;
use crate::session::SessionManager;
use anyhow::{bail, Context, Result};
use fs2::FileExt;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Pluggable worker backend — real rmux sessions in prod, scripted in tests.
#[allow(async_fn_in_trait)]
pub trait WorkerRuntime {
    /// Prepare an inert, generation-bound dispatch receipt. Production
    /// runtimes override this to include a prepared (unpublished) scope claim.
    async fn prepare(&self, step: &PlanStep, _cwd: &Path) -> Result<WorkerDispatchBinding> {
        Ok(WorkerDispatchBinding {
            schema_version: WORKER_DISPATCH_SCHEMA_VERSION,
            generation: new_dispatch_generation()?,
            session: format!("fake-{}", step.step_id),
            scope_claim: None,
        })
    }
    /// Publish and spawn exactly the persisted binding. A runtime must never
    /// invent or rediscover mutable authority here.
    async fn spawn(
        &self,
        step: &PlanStep,
        binding: &WorkerDispatchBinding,
        brief: &str,
        cwd: &Path,
    ) -> Result<()>;
    /// Block until the worker's done.json appears (or timeout).
    async fn wait_done(&self, session: &str, timeout: Duration) -> Result<DoneSignal>;
    /// Reconcile a worker whose wait_done failed (timeout / bad done.json):
    /// kill its session and release its file-scope claim so the next run starts
    /// clean. Default no-op (scripted test runtimes have nothing to clean).
    async fn cleanup_failed(&self, _binding: &WorkerDispatchBinding) -> Result<()> {
        Ok(())
    }
    /// Release a worker's file-scope claim after a TERMINAL verdict without
    /// killing its session. A Pass-verdict worker has finished its job but its
    /// claim must still be freed, or the next run that touches the same files is
    /// rejected (claim_or_reject) and that step can never start. cleanup_failed
    /// kills+releases (right for Retry/Fail); this releases only (right for a
    /// successful worker we leave for inspection). Default no-op.
    async fn release_scope(&self, _binding: &WorkerDispatchBinding) -> Result<()> {
        Ok(())
    }
    /// Crash-resume adoption probe: if this step's worker from a PREVIOUS run
    /// is still observable (its detached session is alive, or its done.json
    /// already landed), return the session name so run() WAITS on it instead
    /// of resetting the step — re-dispatch would kill live work mid-edit or
    /// discard a finished-but-unprocessed result (spawn clears both). Default
    /// None: scripted test runtimes own no detached sessions.
    async fn adoptable_session(
        &self,
        _step: &PlanStep,
        _binding: &WorkerDispatchBinding,
    ) -> Result<bool> {
        Ok(false)
    }
    /// Retire the exact generation named by an orphaned tracker entry before
    /// returning that step to Pending. A runtime that cannot prove cleanup
    /// must fail rather than permit a second writer.
    async fn cleanup_orphan(
        &self,
        _step: &PlanStep,
        _binding: Option<&WorkerDispatchBinding>,
    ) -> Result<()> {
        Ok(())
    }
}

fn new_dispatch_generation() -> Result<String> {
    let mut entropy = [0_u8; 32];
    getrandom::fill(&mut entropy)
        .map_err(|error| anyhow::anyhow!("reading OS entropy for worker generation: {error}"))?;
    let mut generation = String::with_capacity(67);
    generation.push_str("wg-");
    for byte in entropy {
        use std::fmt::Write as _;
        write!(&mut generation, "{byte:02x}").expect("writing to String cannot fail");
    }
    Ok(generation)
}

fn validate_dispatch_binding(binding: &WorkerDispatchBinding) -> Result<()> {
    if binding.schema_version != WORKER_DISPATCH_SCHEMA_VERSION {
        bail!(
            "unsupported worker dispatch binding schema {}",
            binding.schema_version
        );
    }
    if binding.generation.len() != 67
        || !binding.generation.starts_with("wg-")
        || !binding.generation[3..]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        bail!("worker dispatch binding has an invalid generation");
    }
    crate::scope::validate_session_identity(&binding.session)?;
    if let Some(claim) = &binding.scope_claim {
        if claim.session != binding.session {
            bail!("worker dispatch binding scope/session mismatch");
        }
        if claim.claim_id.is_none() || claim.workspace_id.is_none() {
            bail!("worker dispatch binding carries a legacy scope receipt");
        }
    }
    Ok(())
}

/// Exclusive lifetime guard for one `.planner/tracker.json`. Without this,
/// two executors can both reconcile and dispatch from the same persisted
/// generation even though individual tracker writes are atomic.
struct PlanRunLock {
    _file: File,
}

impl Drop for PlanRunLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self._file);
    }
}

impl PlanRunLock {
    fn acquire(project_dir: &Path) -> Result<Self> {
        let planner_dir = project_dir.join(".planner");
        std::fs::create_dir_all(&planner_dir)
            .with_context(|| format!("creating {}", planner_dir.display()))?;
        let path = planner_dir.join("plan-run.lock");
        if let Ok(metadata) = std::fs::symlink_metadata(&path) {
            if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
                bail!("refusing unsafe plan run lock {}", path.display());
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                if metadata.nlink() != 1 {
                    bail!("refusing multiply linked plan run lock {}", path.display());
                }
            }
        }
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
        }
        let file = options
            .open(&path)
            .with_context(|| format!("opening {}", path.display()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::{MetadataExt, PermissionsExt};
            let descriptor = file.metadata()?;
            let named = std::fs::symlink_metadata(&path)?;
            if !descriptor.file_type().is_file()
                || descriptor.nlink() != 1
                || named.file_type().is_symlink()
                || descriptor.dev() != named.dev()
                || descriptor.ino() != named.ino()
            {
                bail!("refusing unsafe plan run lock {}", path.display());
            }
            file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
        }
        file.try_lock_exclusive()
            .with_context(|| format!("another plan executor already holds {}", path.display()))?;
        Ok(Self { _file: file })
    }
}

#[derive(Debug, Clone)]
pub struct RunOptions {
    pub parallelism: usize,
    pub max_attempts: u8,
    pub worker_timeout: Duration,
    /// Cap on one Guardian re-run of a verify_command. Without it a verify
    /// that never exits (serve-without-exit, watch mode) froze the whole run.
    pub verify_timeout: Duration,
    /// Explicitly authorize plan steps that mutate no files. Empty
    /// `files_to_touch` is otherwise rejected because it would silently skip
    /// R-SCOPE. This flag is for genuinely read-only/deploy-only steps whose
    /// worker brief and verification command have no writable project surface.
    pub allow_read_only_steps: bool,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            parallelism: 3,
            max_attempts: 2,
            worker_timeout: Duration::from_secs(60 * 30),
            verify_timeout: Duration::from_secs(60 * 10),
            allow_read_only_steps: false,
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct RunReport {
    pub completed: Vec<String>,
    pub failed: Vec<String>,
    pub blocked: Vec<String>,
    pub success: bool,
}

/// Render the worker prompt for one step from its typed fields.
pub fn render_brief(step: &PlanStep) -> String {
    let mut brief = format!(
        "{title}\n\n{description}\n\nFiles you own (touch ONLY these): {files}\n\nDone criteria: {criteria}\n\nVerify before done.json: {verify}",
        title = step.title,
        description = step.description,
        files = step.files_to_touch.join(", "),
        criteria = step.done_criteria,
        verify = step.verify_command,
    );
    // A retry must not be blind: without the prior attempt's verification
    // evidence in the brief, max_attempts just repeats the same mistake.
    if let Some(feedback) = &step.last_feedback {
        brief.push_str(&format!(
            "\n\nPREVIOUS ATTEMPT FAILED VERIFICATION — diagnose and fix this first:\n{feedback}"
        ));
    }
    brief
}

/// Worker timeout: explicit per-step override > terminal-wave default >
/// RunOptions. Audit/deploy-wave steps run forensic audits / the acceptance
/// browser sweep, which routinely outlive the 30-min default — killing them
/// mid-audit falsely fails the heaviest, most valuable steps, so terminal
/// waves get 4x.
fn step_timeout(step: &PlanStep, opts: &RunOptions) -> Duration {
    step.timeout_mins
        .map(|m| Duration::from_secs(m * 60))
        .unwrap_or_else(|| {
            if step.wave.is_some_and(|w| w.is_terminal()) {
                opts.worker_timeout * 4
            } else {
                opts.worker_timeout
            }
        })
}

/// Drive a plan to completion. Selects work via ready_steps only; gates every
/// completion through the Guardian. Resumable: persists after each transition.
pub async fn run<R: WorkerRuntime>(
    project_dir: &Path,
    runtime: &R,
    opts: RunOptions,
) -> Result<RunReport> {
    let _run_lock = PlanRunLock::acquire(project_dir)?;
    // load_strict so a malformed tracker surfaces its parse error instead of
    // masquerading as "no tracker" (the operator must never be told the plan
    // doesn't exist while it sits there corrupt).
    let mut tracker = PlanTracker::load_strict(project_dir)
        .context("loading .planner/tracker.json")?
        .context("no .planner/tracker.json — run `omega plan` first")?;
    // Structural gate BEFORE any worker spawns: cycles, dangling deps,
    // duplicate ids and trivial verify_commands on REMAINING steps are refused
    // here — so a malformed plan can never silently mis-sequence, deadlock, or
    // fake-complete the build. Run-time validation deliberately skips terminal
    // steps and demotes files_to_touch issues to warnings: a mid-flight
    // tracker from an older schema must stay resumable (validate_for_run doc).
    tracker
        .validate_for_run()
        .context("plan failed validation — fix the tracker.json")?;
    if !opts.allow_read_only_steps {
        let unscoped: Vec<&str> = tracker
            .steps
            .iter()
            .filter(|step| {
                matches!(step.status, StepStatus::Pending | StepStatus::InProgress)
                    && step.files_to_touch.is_empty()
            })
            .map(|step| step.step_id.as_str())
            .collect();
        if !unscoped.is_empty() {
            bail!(
                "plan has writable-scope ambiguity on step(s) {}: list exact files, or explicitly set allow_read_only_steps for a genuinely non-writing run",
                unscoped.join(", ")
            );
        }
    }
    // Crash-resume reconciliation: a previous plan-run that died mid-flight
    // persisted its dispatched steps as InProgress (the tracker saves at spawn
    // time, completion only later). ready_steps selects Pending ONLY, so
    // without this pass those steps — and every dependent — stay stuck
    // forever. BUT step workers are DETACHED rmux sessions that survive the
    // engine dying: an InProgress step whose worker is still alive (or whose
    // done.json already landed) is NOT an orphan — re-dispatching it would
    // kill live work mid-edit / discard the unconsumed result (spawn clears
    // both). Those steps are ADOPTED: kept InProgress and fed into the normal
    // wait→verify path on the first loop iteration. Only steps with no
    // observable worker are reset to Pending for re-dispatch (spawn() clears
    // the stale done.json + session, and a same-name scope claim never
    // self-conflicts, so no further cleanup is needed there).
    let mut adopted: Vec<(String, WorkerDispatchBinding, Duration)> = Vec::new();
    let orphaned: Vec<String> = tracker
        .steps
        .iter()
        .filter(|s| s.status == StepStatus::InProgress)
        .map(|s| s.step_id.clone())
        .collect();
    if !orphaned.is_empty() {
        tracing::warn!(steps = ?orphaned, "reconciling in_progress steps left by a previous run");
        for sid in &orphaned {
            let step = tracker.get_step(sid).context("step vanished")?.clone();
            match step.worker_binding.clone() {
                Some(binding) => {
                    validate_dispatch_binding(&binding).with_context(|| {
                        format!("validating persisted worker binding for {sid}")
                    })?;
                    if runtime.adoptable_session(&step, &binding).await? {
                        tracing::info!(step = %sid, session = %binding.session,
                            generation = %binding.generation,
                            "adopting exact persisted worker generation");
                        let timeout = step_timeout(&step, &opts);
                        adopted.push((sid.clone(), binding, timeout));
                    } else {
                        runtime
                            .cleanup_orphan(&step, Some(&binding))
                            .await
                            .with_context(|| {
                                format!("retiring orphaned worker generation for {sid}")
                            })?;
                        tracker.reset_to_pending(sid)?;
                    }
                }
                None => {
                    // Schema migration: old trackers never persisted a scope
                    // receipt. The runtime may retire only the old deterministic
                    // name, then the step is reset and receives a fresh binding.
                    runtime
                        .cleanup_orphan(&step, None)
                        .await
                        .with_context(|| format!("retiring legacy orphan for {sid}"))?;
                    tracker.reset_to_pending(sid)?;
                }
            }
        }
        tracker.save(project_dir)?;
    }
    let guardian = Guardian::new(opts.max_attempts, opts.verify_timeout);
    let mut report = RunReport::default();

    loop {
        let ready: Vec<String> = tracker
            .ready_steps(opts.parallelism)
            .iter()
            .map(|s| s.step_id.clone())
            .collect();

        if ready.is_empty() && adopted.is_empty() {
            let st = tracker.status();
            report.success = st.is_complete();
            if !st.is_complete() {
                // Terminal sweep: classify whatever never reached Done.
                // A step the Guardian failed mid-loop was already recorded in
                // `report.failed`, so guard against double-counting it here.
                for s in &tracker.steps {
                    match s.status {
                        StepStatus::Failed if !report.failed.contains(&s.step_id) => {
                            report.failed.push(s.step_id.clone());
                        }
                        StepStatus::Pending => report.blocked.push(s.step_id.clone()),
                        // Defensive: the start-of-run reconciliation makes a
                        // leftover InProgress unreachable in this loop, but a
                        // stuck step must still be VISIBLE in the report, never
                        // silently dropped from the failure accounting.
                        StepStatus::InProgress => report.blocked.push(s.step_id.clone()),
                        _ => {}
                    }
                }
            }
            return Ok(report);
        }

        // Adopted steps (crash-resume) drain into the first batch: they are
        // already InProgress with a live/finished worker — no spawn needed.
        let mut inflight: Vec<(String, WorkerDispatchBinding, Duration)> =
            std::mem::take(&mut adopted);
        for sid in &ready {
            let pending = tracker.get_step(sid).context("step vanished")?.clone();
            if pending.worker_binding.is_some() {
                bail!("pending step {sid} retained a worker binding without cleanup");
            }
            // Prepare is deliberately inert. If this process dies before the
            // following atomic save, there is no external claim or session to
            // reconcile. The exact prepared receipt is then persisted before
            // spawn publishes any authority.
            let binding = runtime
                .prepare(&pending, project_dir)
                .await
                .with_context(|| format!("preparing worker binding for step {sid}"))?;
            validate_dispatch_binding(&binding)
                .with_context(|| format!("validating worker binding for step {sid}"))?;
            tracker.start_step(sid)?;
            let step = tracker.get_step_mut(sid).context("step vanished")?;
            step.worker_binding = Some(binding.clone());
            // Persist the dispatch intent BEFORE the external spawn. If the
            // process dies after rmux creates the worker, the next run must see
            // InProgress and adopt that exact deterministic session instead of
            // treating the step as Pending and retiring live work as "stale".
            tracker
                .save(project_dir)
                .with_context(|| format!("persisting dispatch intent for step {sid}"))?;
            let step = tracker.get_step(sid).context("step vanished")?.clone();
            let timeout = step_timeout(&step, &opts);
            let brief = render_brief(&step);
            match runtime.spawn(&step, &binding, &brief, project_dir).await {
                Ok(()) => {}
                Err(spawn_error) => {
                    // Cleanup consumes the persisted receipt. If cleanup cannot
                    // be proven, leave InProgress+binding durable so a later run
                    // cannot dispatch a second writer.
                    if let Err(cleanup_error) = runtime.cleanup_orphan(&step, Some(&binding)).await
                    {
                        return Err(cleanup_error).with_context(|| {
                            format!(
                                "spawn for {sid} failed ({spawn_error:#}); exact rollback also failed"
                            )
                        });
                    }
                    tracker
                        .get_step_mut(sid)
                        .context("step vanished")?
                        .worker_binding = None;
                    tracker.mark_failed(sid)?;
                    tracker
                        .save(project_dir)
                        .with_context(|| format!("persisting failed spawn state for step {sid}"))?;
                    return Err(spawn_error).with_context(|| format!("spawning step {sid}"));
                }
            }
            inflight.push((sid.clone(), binding, timeout));
        }
        tracker.save(project_dir)?;

        for (sid, binding, timeout) in inflight {
            let session = binding.session.as_str();
            // A single worker's wait_done error (timeout / unreadable / unparseable
            // done.json) must NOT abort the whole plan: that would strand every
            // sibling step persisted as Running, never reconciled. Treat it as a
            // failed step, record it, release its scope claim, and continue the
            // batch so the rest of the plan still drives to completion.
            let done = match runtime.wait_done(session, timeout).await {
                Ok(done) => done,
                Err(e) => {
                    tracing::error!(step = %sid, session = %session, error = %e, "worker wait failed — marking step failed");
                    // Runtime-specific cleanup: kill the (likely hung) session and
                    // free its file-scope claim so the next run starts clean.
                    if let Err(cleanup_error) = runtime.cleanup_failed(&binding).await {
                        tracker.save(project_dir)?;
                        return Err(cleanup_error).with_context(|| {
                            format!("fail-closed cleanup for worker {session} after wait failure")
                        });
                    }
                    tracker
                        .get_step_mut(&sid)
                        .context("step vanished")?
                        .worker_binding = None;
                    tracker.mark_failed(&sid)?;
                    report.failed.push(sid.clone());
                    tracker.save(project_dir)?;
                    continue;
                }
            };
            let step = tracker.get_step(&sid).context("step vanished")?.clone();
            let attempt = step.attempt + 1;
            // A worker's done.json is an input, never the verdict (R-VERIFY) —
            // but a worker that DECLARES blocked/failed must never reach the
            // Guardian: a verify weaker than the step's actual work (e.g. a
            // build that already passed BEFORE the step) would convert that
            // honest negative into a Pass and mark the step Done. Treat any
            // non-done_clean signal like a failed verification: retry with the
            // worker's stated blocker as feedback while attempts remain.
            if done.status != DoneStatus::DoneClean {
                let label = match done.status {
                    DoneStatus::DoneClean => unreachable!(),
                    DoneStatus::Pending => "pending",
                    DoneStatus::Failed => "failed",
                    DoneStatus::Blocked => "blocked",
                };
                tracing::warn!(step = %sid, status = %label, summary = %done.summary, "worker did not signal done_clean");
                runtime
                    .cleanup_failed(&binding)
                    .await
                    .with_context(|| format!("fail-closed cleanup for worker {session}"))?;
                if attempt < opts.max_attempts {
                    tracker.bump_attempt(&sid)?;
                    tracker.reset_to_pending(&sid)?;
                    if let Some(s) = tracker.get_step_mut(&sid) {
                        s.last_feedback = Some(format!(
                            "previous worker signalled `{label}`: {}",
                            done.summary
                        ));
                    }
                } else {
                    tracker
                        .get_step_mut(&sid)
                        .context("step vanished")?
                        .worker_binding = None;
                    tracker.mark_failed(&sid)?;
                    report.failed.push(sid.clone());
                }
                tracker.save(project_dir)?;
                continue;
            }
            match guardian.verify(&step, project_dir, attempt).await {
                Verdict::Pass => {
                    // Worker announced done + Guardian re-verified → the engine has
                    // advanced past this step, so CLOSE the worker: kill its session
                    // and free the file-scope claim (otherwise finished workers pile
                    // up as idle panes and the lock outlives them, blocking re-runs).
                    // Release BEFORE persisting Done: a failed release leaves the
                    // step resumable rather than publishing completion while its
                    // writable scope remains ambiguously held.
                    runtime
                        .release_scope(&binding)
                        .await
                        .with_context(|| format!("releasing accepted worker {session}"))?;
                    tracker
                        .get_step_mut(&sid)
                        .context("step vanished")?
                        .worker_binding = None;
                    tracker.mark_done(&sid)?;
                    report.completed.push(sid.clone());
                }
                Verdict::Retry { feedback } => {
                    tracing::warn!(step = %sid, %feedback, "guardian: retry");
                    runtime
                        .cleanup_failed(&binding)
                        .await
                        .with_context(|| format!("cleaning retry worker {session}"))?;
                    tracker.bump_attempt(&sid)?;
                    tracker.reset_to_pending(&sid)?;
                    // Persist the Guardian's evidence ON the step: render_brief
                    // appends it to the retry worker's prompt. Logging alone
                    // delivered nothing to the worker — retries were blind.
                    if let Some(s) = tracker.get_step_mut(&sid) {
                        s.last_feedback = Some(feedback);
                    }
                    // Release the file-scope claim from this attempt so the retry
                    // re-spawn can re-claim it. Without this, claim_or_reject in the
                    // next spawn() rejects (files still locked by the prior attempt)
                    // and the step can never actually retry. spawn() also clears the
                    // stale done.json + kills the old session.
                }
                Verdict::Fail { reason } => {
                    tracing::error!(step = %sid, %reason, "guardian: fail");
                    runtime
                        .cleanup_failed(&binding)
                        .await
                        .with_context(|| format!("cleaning failed worker {session}"))?;
                    tracker
                        .get_step_mut(&sid)
                        .context("step vanished")?
                        .worker_binding = None;
                    tracker.mark_failed(&sid)?;
                    report.failed.push(sid.clone());
                    // Terminal failure: kill the worker + free its scope claim
                    // (same cleanup as Retry), else the lock leaks forever.
                }
            }
            tracker.save(project_dir)?;
        }
    }
}

/// Real backend: spawns rmux worker sessions and waits on their done.json —
/// the exact contract the Orchestrator already uses
/// (`create_session_with_agent` + polling `state_dir/worker-{session}.done.json`).
pub struct RmuxRuntime<'a> {
    pub mgr: &'a SessionManager,
    pub state_dir: PathBuf,
    pub project: String,
    pub agent: Agent,
    pub poll: Duration,
}

fn read_done_signal_strict(state_dir: &Path, session: &str) -> Result<Option<DoneSignal>> {
    let signal = DoneSignal::read(state_dir, session)?;
    if let Some(signal) = &signal {
        if signal.session != session {
            bail!(
                "done signal path/session mismatch: expected {}, document names {}",
                session,
                signal.session
            );
        }
        if let Some(projection) = &signal.projection {
            if projection.source != "mission-engine-v3.sqlite3"
                || projection.event_id.trim().is_empty()
                || projection.event_sequence == 0
                || projection.mission_version < projection.event_sequence
                || projection.projection_hash.len() != 64
                || !projection
                    .projection_hash
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit())
            {
                bail!("done signal {session} carries malformed ledger provenance");
            }
        }
    }
    Ok(signal)
}

fn legacy_step_session_name(project: &str, step: &PlanStep) -> String {
    crate::session::sanitize_session_name(&format!(
        "{}-step-{}",
        project,
        step.step_id.to_lowercase()
    ))
}

/// Bind a detached worker name to the tracker generation while retaining a
/// readable prefix. The generation digest is kept at the end deliberately:
/// `sanitize_session_name` truncates from the right, which previously allowed
/// long project/step names to erase the only fencing component.
fn generated_step_session_name(project: &str, step: &PlanStep, generation: &str) -> Result<String> {
    let probe = WorkerDispatchBinding {
        schema_version: WORKER_DISPATCH_SCHEMA_VERSION,
        generation: generation.to_string(),
        session: "generation-probe".to_string(),
        scope_claim: None,
    };
    validate_dispatch_binding(&probe)?;
    let digest = blake3::hash(
        format!(
            "omega-plan-worker-v1\0{project}\0{}\0{generation}",
            step.step_id
        )
        .as_bytes(),
    )
    .to_hex();
    let suffix = &digest[..20];
    let mut prefix = legacy_step_session_name(project, step);
    let prefix_limit = crate::session::MAX_SESSION_NAME_LEN - suffix.len() - 1;
    prefix.truncate(prefix_limit);
    let prefix = prefix.trim_end_matches(['-', '.']);
    Ok(format!("{prefix}-{suffix}"))
}

impl RmuxRuntime<'_> {
    fn validate_binding(&self, step: &PlanStep, binding: &WorkerDispatchBinding) -> Result<()> {
        validate_dispatch_binding(binding)?;
        let expected = generated_step_session_name(&self.project, step, &binding.generation)?;
        if binding.session != expected {
            bail!(
                "worker binding session mismatch: expected {expected}, got {}",
                binding.session
            );
        }
        match (step.files_to_touch.is_empty(), &binding.scope_claim) {
            (true, None) => {}
            (true, Some(_)) => bail!("read-only worker binding unexpectedly owns file scope"),
            (false, None) => bail!("writable worker binding has no scope receipt"),
            (false, Some(claim)) if claim.files_owned != step.files_to_touch => {
                bail!("worker binding scope does not match the plan step")
            }
            (false, Some(_)) => {}
        }
        Ok(())
    }

    /// Migration-only cleanup for the pre-binding deterministic session name.
    /// New dispatches never use this mutable name as authority.
    async fn retire_legacy_session_generation(&self, session: &str) -> Result<()> {
        crate::scope::validate_session_identity(session)?;
        let observed_claim = scope::ScopeClaim::read_strict(&self.state_dir, session)?;
        let live = self
            .mgr
            .list_sessions()
            .await
            .with_context(|| format!("listing sessions before retiring {session}"))?;
        if live.iter().any(|candidate| candidate.name == session) {
            self.mgr
                .kill_session(session)
                .await
                .with_context(|| format!("killing previous session {session}"))?;
            let after = self
                .mgr
                .list_sessions()
                .await
                .with_context(|| format!("confirming retirement of {session}"))?;
            if after.iter().any(|candidate| candidate.name == session) {
                bail!("session {session} remained live after kill acknowledgement");
            }
        }
        if let Some(claim) = observed_claim {
            scope::ScopeClaim::release_exact(&self.state_dir, &claim)
                .with_context(|| format!("releasing exact scope generation for {session}"))?;
        }
        Ok(())
    }

    async fn retire_legacy_session_artifacts(&self, session: &str) -> Result<()> {
        let done_path = self.state_dir.join(format!("worker-{session}.done.json"));
        let stale_done = read_done_signal_strict(&self.state_dir, session)?;
        self.retire_legacy_session_generation(session).await?;
        if stale_done.is_some() {
            crate::scope::remove_private_file(&done_path)
                .with_context(|| format!("removing stale done signal for {session}"))?;
        }
        Ok(())
    }

    /// Consume only the persisted generation receipt. A delayed cleanup for A
    /// cannot kill B because each generation has a unique session, and cannot
    /// release B because ScopeClaim::release_exact fences its claim id.
    async fn retire_bound_artifacts(&self, binding: &WorkerDispatchBinding) -> Result<()> {
        validate_dispatch_binding(binding)?;
        let session = &binding.session;
        let done_path = self.state_dir.join(format!("worker-{session}.done.json"));
        let stale_done = read_done_signal_strict(&self.state_dir, session)?;
        let live = self
            .mgr
            .list_sessions()
            .await
            .with_context(|| format!("listing sessions before retiring {session}"))?;
        if live.iter().any(|candidate| candidate.name == *session) {
            self.mgr
                .kill_session(session)
                .await
                .with_context(|| format!("killing worker generation {session}"))?;
            let after = self
                .mgr
                .list_sessions()
                .await
                .with_context(|| format!("confirming retirement of {session}"))?;
            if after.iter().any(|candidate| candidate.name == *session) {
                bail!("session {session} remained live after kill acknowledgement");
            }
        }
        match &binding.scope_claim {
            Some(expected) => scope::ScopeClaim::release_exact(&self.state_dir, expected)
                .with_context(|| format!("releasing persisted scope receipt for {session}"))?,
            None => {
                if scope::ScopeClaim::read_strict(&self.state_dir, session)?.is_some() {
                    bail!("read-only worker {session} unexpectedly owns file scope");
                }
            }
        }
        if stale_done.is_some() {
            scope::remove_private_file(&done_path)
                .with_context(|| format!("removing done signal for {session}"))?;
        }
        Ok(())
    }
}

impl WorkerRuntime for RmuxRuntime<'_> {
    async fn prepare(&self, step: &PlanStep, cwd: &Path) -> Result<WorkerDispatchBinding> {
        let generation = new_dispatch_generation()?;
        let session = generated_step_session_name(&self.project, step, &generation)?;
        let scope_claim = if step.files_to_touch.is_empty() {
            None
        } else {
            Some(
                scope::prepare_claim_for_workspace(cwd, &session, step.files_to_touch.clone())
                    .with_context(|| format!("preparing scope receipt for {session}"))?,
            )
        };
        Ok(WorkerDispatchBinding {
            schema_version: WORKER_DISPATCH_SCHEMA_VERSION,
            generation,
            session,
            scope_claim,
        })
    }

    async fn adoptable_session(
        &self,
        step: &PlanStep,
        binding: &WorkerDispatchBinding,
    ) -> Result<bool> {
        self.validate_binding(step, binding)?;
        let session = &binding.session;
        let done_exists = read_done_signal_strict(&self.state_dir, session)?.is_some();
        let live = self
            .mgr
            .list_sessions()
            .await
            .with_context(|| format!("listing sessions before adopting {session}"))?;
        let observable = done_exists || live.iter().any(|candidate| candidate.name == *session);
        if observable {
            let current = scope::ScopeClaim::read_strict(&self.state_dir, session)?;
            match (&binding.scope_claim, current) {
                (Some(expected), Some(current)) if *expected == current => {}
                (Some(_), Some(_)) => {
                    bail!("worker {session} is observable but its scope generation changed")
                }
                (Some(_), None) => {
                    bail!("worker {session} is observable without its persisted scope claim")
                }
                (None, Some(_)) => {
                    bail!("read-only worker {session} unexpectedly owns file scope")
                }
                (None, None) => {}
            }
        }
        Ok(observable)
    }

    async fn cleanup_orphan(
        &self,
        step: &PlanStep,
        binding: Option<&WorkerDispatchBinding>,
    ) -> Result<()> {
        match binding {
            Some(binding) => {
                self.validate_binding(step, binding)?;
                self.retire_bound_artifacts(binding).await
            }
            None => {
                let legacy = legacy_step_session_name(&self.project, step);
                self.retire_legacy_session_artifacts(&legacy).await
            }
        }
    }

    async fn spawn(
        &self,
        step: &PlanStep,
        binding: &WorkerDispatchBinding,
        brief: &str,
        cwd: &Path,
    ) -> Result<()> {
        self.validate_binding(step, binding)?;
        let session = binding.session.clone();
        let legacy_session = legacy_step_session_name(&self.project, step);

        // Retry-safety: the session name is deterministic per step, so on a
        // guardian-Retry re-dispatch the OLD attempt's worker-<session>.done.json
        // still exists (wait_done would return it instantly and re-verify
        // unchanged state) and the rmux session may still be alive (create-or-reuse
        // would attach to the stale pane and never deliver the new brief). Clear
        // both so a retry actually re-runs the step with the fresh brief:
        // Migration cleanup: pre-generation executors used one deterministic
        // name forever. Retire it before any generation-bound worker starts,
        // otherwise a legacy writer can survive beside the new generation.
        if legacy_session != session {
            self.retire_legacy_session_artifacts(&legacy_session)
                .await?;
        }
        if read_done_signal_strict(&self.state_dir, &session)?.is_some() {
            bail!("fresh worker generation {session} already has a done signal");
        }
        let live = self.mgr.list_sessions().await?;
        if live.iter().any(|candidate| candidate.name == session) {
            bail!("fresh worker generation {session} already has a live session");
        }
        if let Some(prepared) = &binding.scope_claim {
            scope::publish_prepared_claim(&self.state_dir, cwd, prepared)
                .with_context(|| format!("publishing scope receipt for {session}"))?;
        }
        // THE FUNNEL — inject the Worker-scoped Laws + operational rules. This
        // plan-run/executor dispatch path previously spawned workers with NO
        // doctrine; mirror cmd_spawn_worker so every dispatched agent gets it.
        let mut full_brief = brief.to_string();
        // CRITICAL: tell the worker HOW to signal completion. Without this the
        // worker does the work, verifies, writes a "Resume:" line, then idles at the
        // prompt — never emitting worker-<session>.done.json, so the executor's
        // wait_done() blocks forever and the ENTIRE plan stalls at this step. The
        // `omega done <session> …` invocation (with the exact session baked in) is the
        // signal the engine polls for. This is the #1 reason a build "never finishes".
        full_brief.push_str(&crate::rules::worker_session_identity_block(&session));
        full_brief.push_str(&format!(
            "\n## SIGNAL COMPLETION — REQUIRED (the build engine BLOCKS until you do this)\n\
             This is an AUTOMATED build step. As your VERY LAST action, after your Verify \
             command passes, you MUST run exactly:\n  \
             omega done {session} done_clean \"<one-line summary of what you changed>\"\n\
             That writes the done signal the engine waits for. If you genuinely cannot finish, run\n  \
             omega done {session} blocked \"<what blocks you>\"   (or: failed)\n\
             Do NOT stop at the prompt, ask a question, or write only a 'Resume:' line and idle — \
             the whole plan halts on this step until `omega done {session} …` runs. No exceptions.",
            session = session
        ));
        let ctx = crate::orchestration::policy_context_for_agent(
            crate::rules::RuleScope::Worker,
            &full_brief,
            self.agent,
        );
        if !ctx.is_empty() {
            full_brief.push_str("\n\n");
            full_brief.push_str(&ctx);
        }
        let cwd = cwd.to_string_lossy();
        // Claude-side session label (`--name`): mirror the rmux session name so the
        // step-worker's conversation is addressable/resumable by the same
        // deterministic identity (non-Claude providers ignore the field).
        let opts = crate::agents::LaunchOptions {
            session_name: Some(session.clone()),
            ..Default::default()
        };
        if let Err(e) = self
            .mgr
            .create_agent_session_with_opts(&session, &cwd, self.agent, Some(&full_brief), opts)
            .await
        {
            if let Some(claim) = &binding.scope_claim {
                if let Err(release_error) = scope::ScopeClaim::release_exact(&self.state_dir, claim)
                {
                    bail!(
                        "spawning {session} failed: {e:#}; exact scope rollback also failed: {release_error:#}"
                    );
                }
            }
            return Err(e).with_context(|| format!("spawning {session}"));
        }
        Ok(())
    }

    async fn wait_done(&self, session: &str, timeout: Duration) -> Result<DoneSignal> {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(signal) = read_done_signal_strict(&self.state_dir, session)? {
                return Ok(signal);
            }
            if Instant::now() >= deadline {
                bail!("worker {session} timed out after {}s", timeout.as_secs());
            }
            tokio::time::sleep(self.poll).await;
        }
    }

    async fn cleanup_failed(&self, binding: &WorkerDispatchBinding) -> Result<()> {
        // Kill the hung/failed worker session (best-effort) and free its scope
        // claim so a resumed plan run does not see a locked file or a zombie pane.
        self.retire_bound_artifacts(binding).await
    }

    async fn release_scope(&self, binding: &WorkerDispatchBinding) -> Result<()> {
        // A worker whose step is DONE has announced completion (its done.json was
        // accepted and the Guardian re-verified) — so CLOSE it: kill the session
        // and free its scope claim. Leaving Pass workers alive piled up dozens of
        // idle `*-step-*` panes per build; the oracle/engine has already advanced
        // past this step, so the worker has no reason to stay open.
        self.retire_bound_artifacts(binding).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::done::DoneStatus;
    use std::collections::HashMap;

    /// Scripts a DoneStatus per step_id; session names are "fake-<step_id>".
    pub struct FakeRuntime {
        pub script: HashMap<String, DoneStatus>,
    }

    impl WorkerRuntime for FakeRuntime {
        async fn spawn(
            &self,
            _step: &PlanStep,
            _binding: &WorkerDispatchBinding,
            _brief: &str,
            _cwd: &Path,
        ) -> Result<()> {
            Ok(())
        }
        async fn wait_done(&self, session: &str, _t: Duration) -> Result<DoneSignal> {
            let step_id = session.strip_prefix("fake-").unwrap_or(session);
            let status = self
                .script
                .get(step_id)
                .cloned()
                .unwrap_or(DoneStatus::DoneClean);
            Ok(DoneSignal::stub(step_id, status))
        }
    }

    struct ReleaseFailureRuntime;

    impl WorkerRuntime for ReleaseFailureRuntime {
        async fn spawn(
            &self,
            _step: &PlanStep,
            _binding: &WorkerDispatchBinding,
            _brief: &str,
            _cwd: &Path,
        ) -> Result<()> {
            Ok(())
        }

        async fn wait_done(&self, session: &str, _timeout: Duration) -> Result<DoneSignal> {
            Ok(DoneSignal::stub(session, DoneStatus::DoneClean))
        }

        async fn release_scope(&self, _binding: &WorkerDispatchBinding) -> Result<()> {
            bail!("synthetic exact-release failure")
        }
    }

    #[derive(Default)]
    struct SpawnResumeState {
        spawn_calls: Vec<String>,
        adoption_calls: Vec<String>,
        spawned_bindings: Vec<WorkerDispatchBinding>,
        adopted_bindings: Vec<WorkerDispatchBinding>,
        live_sessions: std::collections::HashSet<String>,
    }

    #[derive(Clone, Default)]
    struct FailSecondSpawnRuntime {
        state: std::sync::Arc<Mutex<SpawnResumeState>>,
    }

    impl WorkerRuntime for FailSecondSpawnRuntime {
        async fn prepare(&self, step: &PlanStep, _cwd: &Path) -> Result<WorkerDispatchBinding> {
            Ok(WorkerDispatchBinding {
                schema_version: WORKER_DISPATCH_SCHEMA_VERSION,
                generation: new_dispatch_generation()?,
                session: format!("resume-{}", step.step_id),
                scope_claim: None,
            })
        }

        async fn spawn(
            &self,
            step: &PlanStep,
            binding: &WorkerDispatchBinding,
            _brief: &str,
            _cwd: &Path,
        ) -> Result<()> {
            let mut state = self.state.lock().unwrap();
            state.spawn_calls.push(step.step_id.clone());
            state.spawned_bindings.push(binding.clone());
            if step.step_id == "STEP-002" {
                bail!("synthetic second spawn failure");
            }
            state.live_sessions.insert(binding.session.clone());
            Ok(())
        }

        async fn adoptable_session(
            &self,
            step: &PlanStep,
            binding: &WorkerDispatchBinding,
        ) -> Result<bool> {
            let mut state = self.state.lock().unwrap();
            if state.live_sessions.contains(&binding.session) {
                state.adoption_calls.push(step.step_id.clone());
                state.adopted_bindings.push(binding.clone());
                Ok(true)
            } else {
                Ok(false)
            }
        }

        async fn wait_done(&self, session: &str, _timeout: Duration) -> Result<DoneSignal> {
            if !self.state.lock().unwrap().live_sessions.contains(session) {
                bail!("session {session} was not live for adoption");
            }
            Ok(DoneSignal::stub(session, DoneStatus::DoneClean))
        }

        async fn release_scope(&self, binding: &WorkerDispatchBinding) -> Result<()> {
            self.state
                .lock()
                .unwrap()
                .live_sessions
                .remove(&binding.session);
            Ok(())
        }
    }

    #[tokio::test]
    async fn spawn_failure_persists_started_sibling_for_resume_without_duplicate() {
        let dir = tempfile::tempdir().unwrap();
        let mut tracker = PlanTracker::new("parallel spawn crash consistency");
        let phase = tracker.add_phase("parallel", "dispatch independent steps");
        for (step_id, file) in [("STEP-001", "a.rs"), ("STEP-002", "b.rs")] {
            tracker
                .add_step(
                    phase,
                    step(step_id, phase)
                        .title(step_id)
                        .files(&[file])
                        .criteria("verified")
                        .verify("test -d .")
                        .build(),
                )
                .unwrap();
        }
        tracker.save(dir.path()).unwrap();

        let runtime = FailSecondSpawnRuntime::default();
        let error = run(dir.path(), &runtime, RunOptions::default())
            .await
            .unwrap_err();
        assert!(error.to_string().contains("spawning step STEP-002"));

        let persisted = PlanTracker::load_strict(dir.path()).unwrap().unwrap();
        assert_eq!(
            persisted.get_step("STEP-001").unwrap().status,
            StepStatus::InProgress,
            "the already spawned sibling must remain durably adoptable"
        );
        assert_eq!(
            persisted.get_step("STEP-002").unwrap().status,
            StepStatus::Failed,
            "the failed external spawn must have an explicit durable terminal state"
        );
        let persisted_binding = persisted
            .get_step("STEP-001")
            .unwrap()
            .worker_binding
            .clone()
            .expect("spawned sibling must retain its exact binding");

        let report = run(dir.path(), &runtime, RunOptions::default())
            .await
            .unwrap();
        assert!(!report.success, "the failed sibling remains visible");
        assert_eq!(report.completed, vec!["STEP-001"]);
        assert_eq!(report.failed, vec!["STEP-002"]);

        let state = runtime.state.lock().unwrap();
        assert_eq!(state.spawn_calls, vec!["STEP-001", "STEP-002"]);
        assert_eq!(state.adoption_calls, vec!["STEP-001"]);
        assert_eq!(state.adopted_bindings, vec![persisted_binding.clone()]);
        assert_eq!(state.spawned_bindings[0], persisted_binding);
        assert!(state.live_sessions.is_empty());
    }

    fn unscoped_binding(session: &str) -> WorkerDispatchBinding {
        WorkerDispatchBinding {
            schema_version: WORKER_DISPATCH_SCHEMA_VERSION,
            generation: new_dispatch_generation().unwrap(),
            session: session.to_string(),
            scope_claim: None,
        }
    }

    #[derive(Default)]
    struct GenerationState {
        live_sessions: std::collections::HashSet<String>,
        adopted: Vec<WorkerDispatchBinding>,
        cleaned: Vec<WorkerDispatchBinding>,
        spawned: Vec<WorkerDispatchBinding>,
    }

    #[derive(Clone, Default)]
    struct GenerationRuntime {
        state: std::sync::Arc<Mutex<GenerationState>>,
    }

    impl WorkerRuntime for GenerationRuntime {
        async fn prepare(&self, _step: &PlanStep, _cwd: &Path) -> Result<WorkerDispatchBinding> {
            let generation = new_dispatch_generation()?;
            Ok(WorkerDispatchBinding {
                schema_version: WORKER_DISPATCH_SCHEMA_VERSION,
                session: format!("fresh-{}", &generation[3..19]),
                generation,
                scope_claim: None,
            })
        }

        async fn spawn(
            &self,
            _step: &PlanStep,
            binding: &WorkerDispatchBinding,
            _brief: &str,
            _cwd: &Path,
        ) -> Result<()> {
            let mut state = self.state.lock().unwrap();
            state.spawned.push(binding.clone());
            state.live_sessions.insert(binding.session.clone());
            Ok(())
        }

        async fn adoptable_session(
            &self,
            _step: &PlanStep,
            binding: &WorkerDispatchBinding,
        ) -> Result<bool> {
            let mut state = self.state.lock().unwrap();
            state.adopted.push(binding.clone());
            Ok(state.live_sessions.contains(&binding.session))
        }

        async fn cleanup_orphan(
            &self,
            _step: &PlanStep,
            binding: Option<&WorkerDispatchBinding>,
        ) -> Result<()> {
            if let Some(binding) = binding {
                let mut state = self.state.lock().unwrap();
                state.cleaned.push(binding.clone());
                state.live_sessions.remove(&binding.session);
            }
            Ok(())
        }

        async fn wait_done(&self, session: &str, _timeout: Duration) -> Result<DoneSignal> {
            Ok(DoneSignal::stub(session, DoneStatus::DoneClean))
        }

        async fn cleanup_failed(&self, binding: &WorkerDispatchBinding) -> Result<()> {
            self.state
                .lock()
                .unwrap()
                .live_sessions
                .remove(&binding.session);
            Ok(())
        }

        async fn release_scope(&self, binding: &WorkerDispatchBinding) -> Result<()> {
            self.state
                .lock()
                .unwrap()
                .live_sessions
                .remove(&binding.session);
            Ok(())
        }
    }

    #[tokio::test]
    async fn resume_rejects_stale_generation_after_crash_before_spawn() {
        let dir = tempfile::tempdir().unwrap();
        let mut tracker = PlanTracker::new("generation fencing");
        let phase = tracker.add_phase("run", "one exact worker");
        tracker
            .add_step(
                phase,
                step("STEP-001", phase)
                    .title("fenced")
                    .files(&["a.rs"])
                    .criteria("verified")
                    .verify("test -d .")
                    .build(),
            )
            .unwrap();
        tracker.start_step("STEP-001").unwrap();
        let stale_a = unscoped_binding("stale-a");
        let persisted_b = unscoped_binding("persisted-b");
        tracker.get_step_mut("STEP-001").unwrap().worker_binding = Some(persisted_b.clone());
        tracker.save(dir.path()).unwrap();

        let runtime = GenerationRuntime::default();
        runtime
            .state
            .lock()
            .unwrap()
            .live_sessions
            .insert(stale_a.session.clone());
        let report = run(dir.path(), &runtime, RunOptions::default())
            .await
            .unwrap();
        assert!(report.success);
        let state = runtime.state.lock().unwrap();
        assert_eq!(state.adopted, vec![persisted_b.clone()]);
        assert_eq!(state.cleaned, vec![persisted_b.clone()]);
        assert_eq!(state.spawned.len(), 1);
        assert_ne!(state.spawned[0].generation, persisted_b.generation);
        assert!(state.live_sessions.contains(&stale_a.session));
    }

    #[tokio::test]
    async fn late_cleanup_of_a_cannot_kill_or_release_b() {
        let runtime = GenerationRuntime::default();
        let generation_a = unscoped_binding("generation-a");
        let generation_b = unscoped_binding("generation-b");
        runtime
            .state
            .lock()
            .unwrap()
            .live_sessions
            .insert(generation_b.session.clone());
        runtime.cleanup_failed(&generation_a).await.unwrap();
        assert!(runtime
            .state
            .lock()
            .unwrap()
            .live_sessions
            .contains(&generation_b.session));
    }

    #[derive(Clone)]
    struct ScopeRecoveryRuntime {
        state_dir: PathBuf,
    }

    impl WorkerRuntime for ScopeRecoveryRuntime {
        async fn prepare(&self, step: &PlanStep, cwd: &Path) -> Result<WorkerDispatchBinding> {
            let generation = new_dispatch_generation()?;
            let session = format!("scope-{}", &generation[3..19]);
            Ok(WorkerDispatchBinding {
                schema_version: WORKER_DISPATCH_SCHEMA_VERSION,
                generation,
                scope_claim: Some(scope::prepare_claim_for_workspace(
                    cwd,
                    &session,
                    step.files_to_touch.clone(),
                )?),
                session,
            })
        }

        async fn spawn(
            &self,
            _step: &PlanStep,
            binding: &WorkerDispatchBinding,
            _brief: &str,
            cwd: &Path,
        ) -> Result<()> {
            scope::publish_prepared_claim(
                &self.state_dir,
                cwd,
                binding
                    .scope_claim
                    .as_ref()
                    .context("missing scope receipt")?,
            )?;
            Ok(())
        }

        async fn wait_done(&self, session: &str, _timeout: Duration) -> Result<DoneSignal> {
            Ok(DoneSignal::stub(session, DoneStatus::DoneClean))
        }

        async fn cleanup_orphan(
            &self,
            _step: &PlanStep,
            binding: Option<&WorkerDispatchBinding>,
        ) -> Result<()> {
            if let Some(claim) = binding.and_then(|binding| binding.scope_claim.as_ref()) {
                scope::ScopeClaim::release_exact(&self.state_dir, claim)?;
            }
            Ok(())
        }

        async fn release_scope(&self, binding: &WorkerDispatchBinding) -> Result<()> {
            scope::ScopeClaim::release_exact(
                &self.state_dir,
                binding
                    .scope_claim
                    .as_ref()
                    .context("missing scope receipt")?,
            )
        }
    }

    #[tokio::test]
    async fn resume_cleans_exact_claim_after_publish_before_session() {
        let workspace = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let mut tracker = PlanTracker::new("publish crash");
        let phase = tracker.add_phase("run", "recover claim");
        tracker
            .add_step(
                phase,
                step("STEP-001", phase)
                    .title("claim")
                    .files(&["a.rs"])
                    .criteria("verified")
                    .verify("test -d .")
                    .build(),
            )
            .unwrap();
        tracker.start_step("STEP-001").unwrap();
        let generation = new_dispatch_generation().unwrap();
        let session = format!("published-{}", &generation[3..19]);
        let claim = scope::prepare_claim_for_workspace(
            workspace.path(),
            &session,
            vec!["a.rs".to_string()],
        )
        .unwrap();
        scope::publish_prepared_claim(state.path(), workspace.path(), &claim).unwrap();
        let interrupted = WorkerDispatchBinding {
            schema_version: WORKER_DISPATCH_SCHEMA_VERSION,
            generation,
            session: session.clone(),
            scope_claim: Some(claim),
        };
        tracker.get_step_mut("STEP-001").unwrap().worker_binding = Some(interrupted);
        tracker.save(workspace.path()).unwrap();

        let runtime = ScopeRecoveryRuntime {
            state_dir: state.path().to_path_buf(),
        };
        let report = run(workspace.path(), &runtime, RunOptions::default())
            .await
            .unwrap();
        assert!(report.success);
        assert!(scope::ScopeClaim::read_strict(state.path(), &session)
            .unwrap()
            .is_none());
        assert!(scope::ScopeClaim::read_all_strict(state.path())
            .unwrap()
            .is_empty());
    }

    #[derive(Clone, Default)]
    struct BlockingRuntime {
        spawned: std::sync::Arc<tokio::sync::Notify>,
        release: std::sync::Arc<tokio::sync::Notify>,
        spawn_count: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    }

    impl WorkerRuntime for BlockingRuntime {
        async fn spawn(
            &self,
            _step: &PlanStep,
            _binding: &WorkerDispatchBinding,
            _brief: &str,
            _cwd: &Path,
        ) -> Result<()> {
            self.spawn_count
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            self.spawned.notify_one();
            Ok(())
        }

        async fn wait_done(&self, session: &str, _timeout: Duration) -> Result<DoneSignal> {
            self.release.notified().await;
            Ok(DoneSignal::stub(session, DoneStatus::DoneClean))
        }
    }

    #[tokio::test]
    async fn concurrent_plan_run_is_refused_before_spawn() {
        let dir = tempfile::tempdir().unwrap();
        let mut tracker = PlanTracker::new("single locked run");
        let phase = tracker.add_phase("run", "hold lock");
        tracker
            .add_step(
                phase,
                step("STEP-001", phase)
                    .title("locked")
                    .files(&["a.rs"])
                    .criteria("verified")
                    .verify("test -d .")
                    .build(),
            )
            .unwrap();
        tracker.save(dir.path()).unwrap();
        let runtime = BlockingRuntime::default();
        let first_dir = dir.path().to_path_buf();
        let first_runtime = runtime.clone();
        let first =
            tokio::spawn(
                async move { run(&first_dir, &first_runtime, RunOptions::default()).await },
            );
        runtime.spawned.notified().await;

        let error = run(dir.path(), &runtime, RunOptions::default())
            .await
            .unwrap_err();
        assert!(error
            .to_string()
            .contains("another plan executor already holds"));
        assert_eq!(
            runtime
                .spawn_count
                .load(std::sync::atomic::Ordering::SeqCst),
            1
        );
        runtime.release.notify_one();
        let first_report = first.await.unwrap().unwrap();
        assert!(first_report.success);
    }

    #[cfg(unix)]
    #[test]
    fn plan_run_lock_rejects_symlink_and_hardlink_authority() {
        use std::os::unix::fs::symlink;

        let symlink_project = tempfile::tempdir().unwrap();
        let planner = symlink_project.path().join(".planner");
        std::fs::create_dir_all(&planner).unwrap();
        let victim = symlink_project.path().join("victim");
        std::fs::write(&victim, b"do not lock").unwrap();
        symlink(&victim, planner.join("plan-run.lock")).unwrap();
        assert!(PlanRunLock::acquire(symlink_project.path()).is_err());
        assert_eq!(std::fs::read(&victim).unwrap(), b"do not lock");

        let hardlink_project = tempfile::tempdir().unwrap();
        let planner = hardlink_project.path().join(".planner");
        std::fs::create_dir_all(&planner).unwrap();
        let shared = hardlink_project.path().join("shared");
        std::fs::write(&shared, b"shared").unwrap();
        std::fs::hard_link(&shared, planner.join("plan-run.lock")).unwrap();
        assert!(PlanRunLock::acquire(hardlink_project.path()).is_err());
        assert_eq!(std::fs::read(&shared).unwrap(), b"shared");
    }

    #[test]
    fn strict_done_signal_rejects_path_alias_and_forged_provenance() {
        let tmp = tempfile::tempdir().unwrap();
        let mut signal = DoneSignal::stub("worker-safe", DoneStatus::DoneClean);
        signal.write(tmp.path()).unwrap();
        assert!(read_done_signal_strict(tmp.path(), "worker-safe")
            .unwrap()
            .is_some());

        signal.session = "worker-other".to_string();
        std::fs::write(
            tmp.path().join("worker-worker-safe.done.json"),
            serde_json::to_vec(&signal).unwrap(),
        )
        .unwrap();
        assert!(read_done_signal_strict(tmp.path(), "worker-safe").is_err());

        let mut forged = DoneSignal::stub("worker-forged", DoneStatus::DoneClean);
        forged.projection = Some(crate::done::ProjectionProvenance {
            source: "mission-engine-v3.sqlite3".to_string(),
            event_id: "event".to_string(),
            event_sequence: 1,
            mission_version: 1,
            projection_hash: "not-a-hash".to_string(),
        });
        forged.write(tmp.path()).unwrap();
        assert!(read_done_signal_strict(tmp.path(), "worker-forged").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn strict_done_signal_rejects_symlink_and_hardlink() {
        use std::os::unix::fs::symlink;
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("target");
        std::fs::write(&target, b"{}").unwrap();
        symlink(&target, root.path().join("worker-worker-link.done.json")).unwrap();
        assert!(read_done_signal_strict(root.path(), "worker-link").is_err());

        let hard_target = root.path().join("hard-target");
        std::fs::write(&hard_target, b"{}").unwrap();
        std::fs::hard_link(
            &hard_target,
            root.path().join("worker-worker-hard.done.json"),
        )
        .unwrap();
        assert!(read_done_signal_strict(root.path(), "worker-hard").is_err());
    }

    #[tokio::test]
    async fn empty_scope_requires_explicit_read_only_authorization() {
        let dir = tempfile::tempdir().unwrap();
        let mut tracker = PlanTracker::new("P");
        let phase = tracker.add_phase("read", "diagnostic");
        tracker
            .add_step(
                phase,
                step("STEP-READ", phase)
                    .title("read only")
                    .files(&[])
                    .criteria("observed")
                    .verify("test -d .")
                    .build(),
            )
            .unwrap();
        tracker.save(dir.path()).unwrap();
        let runtime = FakeRuntime {
            script: HashMap::new(),
        };
        assert!(run(dir.path(), &runtime, RunOptions::default())
            .await
            .is_err());
        let report = run(
            dir.path(),
            &runtime,
            RunOptions {
                allow_read_only_steps: true,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(report.success);
    }

    #[tokio::test]
    async fn release_failure_prevents_persisted_completion() {
        let dir = tempfile::tempdir().unwrap();
        save_linear_plan(dir.path(), "test -d .");
        let error = run(dir.path(), &ReleaseFailureRuntime, RunOptions::default())
            .await
            .unwrap_err();
        assert!(error.to_string().contains("releasing accepted worker"));
        let tracker = PlanTracker::load_strict(dir.path()).unwrap().unwrap();
        assert_eq!(
            tracker.get_step("STEP-001").unwrap().status,
            StepStatus::InProgress,
            "completion must not publish while exact scope release failed"
        );
    }

    #[test]
    fn render_brief_includes_files_and_verify() {
        use crate::planner::step;
        let s = step("STEP-001", 1)
            .title("T")
            .description("D")
            .files(&["a.rs"])
            .criteria("ok")
            .verify("true")
            .build();
        let b = render_brief(&s);
        assert!(b.contains("a.rs"));
        assert!(b.contains("true"));
    }

    use crate::planner::step;

    fn save_linear_plan(dir: &Path, verify: &str) {
        let mut t = PlanTracker::new("P");
        let p = t.add_phase("F", "g");
        t.add_step(
            p,
            step("STEP-001", p)
                .title("a")
                .files(&["a.rs"])
                .criteria("ok")
                .verify(verify)
                .build(),
        )
        .unwrap();
        t.add_step(
            p,
            step("STEP-002", p)
                .title("b")
                .files(&["b.rs"])
                .criteria("ok")
                .verify(verify)
                .depends(&["STEP-001"])
                .build(),
        )
        .unwrap();
        t.add_step(
            p,
            step("STEP-003", p)
                .title("c")
                .files(&["c.rs"])
                .criteria("ok")
                .verify(verify)
                .depends(&["STEP-002"])
                .build(),
        )
        .unwrap();
        t.save(dir).unwrap();
    }

    #[tokio::test]
    async fn run_completes_all_steps_in_order() {
        let dir = tempfile::tempdir().unwrap();
        save_linear_plan(dir.path(), "test -d .");
        let rt = FakeRuntime {
            script: HashMap::new(),
        };
        let report = run(dir.path(), &rt, RunOptions::default()).await.unwrap();
        assert!(report.success);
        assert_eq!(report.completed, vec!["STEP-001", "STEP-002", "STEP-003"]);
        let t = PlanTracker::load(dir.path()).unwrap();
        assert!(t.status().is_complete());
    }

    #[tokio::test]
    async fn resume_reconciles_orphaned_in_progress_steps() {
        let dir = tempfile::tempdir().unwrap();
        save_linear_plan(dir.path(), "test -d .");
        // Simulate a previous plan-run that died after dispatch: the step was
        // persisted as in_progress with no live worker behind it.
        let mut t = PlanTracker::load(dir.path()).unwrap();
        t.start_step("STEP-001").unwrap();
        t.save(dir.path()).unwrap();
        let rt = FakeRuntime {
            script: HashMap::new(),
        };
        let report = run(dir.path(), &rt, RunOptions::default()).await.unwrap();
        assert!(
            report.success,
            "orphaned in_progress step must be re-dispatched, not stuck"
        );
        assert_eq!(report.completed, vec!["STEP-001", "STEP-002", "STEP-003"]);
    }

    #[tokio::test]
    async fn blocked_worker_is_never_marked_done() {
        let dir = tempfile::tempdir().unwrap();
        // The trap: a verify that PASSES regardless of the step's work. The
        // worker's honest `blocked` must not be laundered into Done by it.
        save_linear_plan(dir.path(), "test -d .");
        let mut script = HashMap::new();
        script.insert("STEP-001".to_string(), DoneStatus::Blocked);
        let rt = FakeRuntime { script };
        let report = run(
            dir.path(),
            &rt,
            RunOptions {
                max_attempts: 1,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(!report.success);
        assert_eq!(report.failed, vec!["STEP-001"]);
        assert!(report.completed.is_empty());
    }

    /// Records every brief it is handed, so feedback delivery is provable.
    #[derive(Default)]
    struct BriefRecorder {
        briefs: Mutex<Vec<String>>,
    }

    impl WorkerRuntime for BriefRecorder {
        async fn spawn(
            &self,
            _step: &PlanStep,
            _binding: &WorkerDispatchBinding,
            brief: &str,
            _cwd: &Path,
        ) -> Result<()> {
            self.briefs.lock().unwrap().push(brief.to_string());
            Ok(())
        }
        async fn wait_done(&self, session: &str, _t: Duration) -> Result<DoneSignal> {
            let step_id = session.strip_prefix("fake-").unwrap_or(session);
            Ok(DoneSignal::stub(step_id, DoneStatus::DoneClean))
        }
    }

    #[tokio::test]
    async fn guardian_retry_feedback_reaches_the_retry_brief() {
        let dir = tempfile::tempdir().unwrap();
        // Fail-once verify: the first Guardian run plants a marker and exits 1;
        // the re-run sees the marker and passes.
        let verify = "test -f marker || { touch marker; exit 1; }";
        let mut t = PlanTracker::new("P");
        let p = t.add_phase("F", "g");
        t.add_step(
            p,
            step("STEP-001", p)
                .title("a")
                .files(&["a.rs"])
                .criteria("ok")
                .verify(verify)
                .build(),
        )
        .unwrap();
        t.save(dir.path()).unwrap();
        let rt = BriefRecorder::default();
        let report = run(dir.path(), &rt, RunOptions::default()).await.unwrap();
        assert!(report.success);
        let briefs = rt.briefs.lock().unwrap();
        assert_eq!(briefs.len(), 2, "expected exactly one retry dispatch");
        assert!(
            !briefs[0].contains("PREVIOUS ATTEMPT"),
            "first brief must be clean"
        );
        assert!(
            briefs[1].contains("PREVIOUS ATTEMPT FAILED VERIFICATION"),
            "retry brief must carry the Guardian feedback, got:\n{}",
            briefs[1]
        );
    }

    #[tokio::test]
    async fn guardian_overrides_worker_done_claim() {
        let dir = tempfile::tempdir().unwrap();
        save_linear_plan(dir.path(), "false"); // worker claims DoneClean but verify fails
        let rt = FakeRuntime {
            script: HashMap::new(),
        };
        let report = run(
            dir.path(),
            &rt,
            RunOptions {
                max_attempts: 1,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(!report.success);
        assert_eq!(report.failed, vec!["STEP-001"]);
        assert!(report.completed.is_empty());
    }

    use std::sync::Mutex;

    /// Records the scope-lifecycle calls so the file-scope-leak regression is
    /// guarded: a Pass must release its claim, a Fail must clean it up.
    #[derive(Default)]
    struct TrackingRuntime {
        script: HashMap<String, DoneStatus>,
        released: Mutex<Vec<String>>,
        cleaned: Mutex<Vec<String>>,
    }

    impl WorkerRuntime for TrackingRuntime {
        async fn spawn(
            &self,
            _step: &PlanStep,
            _binding: &WorkerDispatchBinding,
            _brief: &str,
            _cwd: &Path,
        ) -> Result<()> {
            Ok(())
        }
        async fn wait_done(&self, session: &str, _t: Duration) -> Result<DoneSignal> {
            let step_id = session.strip_prefix("fake-").unwrap_or(session);
            let status = self
                .script
                .get(step_id)
                .cloned()
                .unwrap_or(DoneStatus::DoneClean);
            Ok(DoneSignal::stub(step_id, status))
        }
        async fn cleanup_failed(&self, binding: &WorkerDispatchBinding) -> Result<()> {
            self.cleaned.lock().unwrap().push(binding.session.clone());
            Ok(())
        }
        async fn release_scope(&self, binding: &WorkerDispatchBinding) -> Result<()> {
            self.released.lock().unwrap().push(binding.session.clone());
            Ok(())
        }
    }

    #[tokio::test]
    async fn pass_releases_scope_claim() {
        let dir = tempfile::tempdir().unwrap();
        save_linear_plan(dir.path(), "test -d .");
        let rt = TrackingRuntime::default();
        let report = run(dir.path(), &rt, RunOptions::default()).await.unwrap();
        assert!(report.success);
        // Every completed step freed its file-scope claim (release-only)...
        assert_eq!(
            rt.released.lock().unwrap().clone(),
            vec!["fake-STEP-001", "fake-STEP-002", "fake-STEP-003"]
        );
        // ...and no successful step needed the kill+release cleanup path.
        assert!(rt.cleaned.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn fail_releases_scope_via_cleanup() {
        let dir = tempfile::tempdir().unwrap();
        save_linear_plan(dir.path(), "false"); // verify fails -> Fail at max_attempts=1
        let rt = TrackingRuntime::default();
        let report = run(
            dir.path(),
            &rt,
            RunOptions {
                max_attempts: 1,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(!report.success);
        // The failed step's claim was freed via cleanup_failed (kill+release);
        // dependents stay blocked so only STEP-001 was cleaned.
        assert_eq!(rt.cleaned.lock().unwrap().clone(), vec!["fake-STEP-001"]);
        // A Fail is not a Pass: the release-only path was NOT taken.
        assert!(rt.released.lock().unwrap().is_empty());
    }
}

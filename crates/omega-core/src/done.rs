use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::net::IpAddr;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration as StdDuration, Instant};

use crate::mission::{VerifierCheck, VerifierCheckKind};

const MAX_COMPLETION_SIGNAL_BYTES: usize = 1024 * 1024;
const MAX_VERIFIER_OUTPUT_BYTES: u64 = 1024 * 1024;
const COMPLETION_SIGNAL_LOCK: &str = ".completion-signals.lock";

fn read_bounded_private_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Option<T>> {
    let Some(bytes) = crate::config::read_private_optional(path)? else {
        return Ok(None);
    };
    if bytes.len() > MAX_COMPLETION_SIGNAL_BYTES {
        bail!(
            "authority signal {} exceeds {} bytes",
            path.display(),
            MAX_COMPLETION_SIGNAL_BYTES
        );
    }
    serde_json::from_slice(&bytes)
        .with_context(|| format!("parsing authority signal {}", path.display()))
        .map(Some)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoneSignal {
    pub session: String,
    pub status: DoneStatus,
    pub summary: String,
    #[serde(default)]
    pub commit: Option<String>,
    pub finished_at: DateTime<Utc>,
    #[serde(default)]
    pub todos_total: u32,
    #[serde(default)]
    pub todos_completed: u32,
    #[serde(default)]
    pub pending_actions: Vec<String>,

    // ── Opus-4.8 system-card hardening (L6) ──
    // Mechanisms #31, #65, #79, #90, #100, #167/168 mapped into a single
    // ground-truth-oriented done schema. Worker narration alone is no
    // longer admissible — these fields require artifact citations OR
    // explicit honest negatives.
    /// What is NOT done, failing, or unsigned-off — REQUIRED even when the
    /// worker thinks everything passed. The "great job" suppression that
    /// the model card catches. (#90)
    #[serde(default)]
    pub not_done: Vec<String>,

    /// Scope + limits framing — what this verification PROVED vs what it
    /// did NOT cover. Bounded honesty. (#100)
    #[serde(default)]
    pub scope: ScopeFraming,

    /// Cross-source corroboration: which independent signals agreed on
    /// the verdict (e.g. ["worker_self_report", "git_ls_remote", "ci_exit_code"]).
    /// A `done_clean` accepted with only `worker_self_report` is
    /// "single-source/unverified" and must be flagged. (#65, #66)
    #[serde(default)]
    pub corroboration: Vec<CorroborationSource>,

    /// Four-failure-mode taxonomy when status != done_clean — durable
    /// comparable failure ledger across the fleet. (#31)
    #[serde(default)]
    pub failure_mode: Option<FailureMode>,

    /// Retry-thrash / decision-flip counter — non-zero suggests the
    /// worker was looping. (#167, #168) Cross [`crate::loop_guard::THRASH_CAP`]
    /// and the patrol escalates the mission to the operator.
    #[serde(default)]
    pub retry_thrash_count: u32,

    /// Set when the worker (or the patrol on its behalf) has decided this loop
    /// cannot close itself and a human must look — bounded-retry exhausted,
    /// repeated contested fabrication, etc. Surfaced in the report and by
    /// `omega log`. serde default keeps pre-existing done.json files readable.
    #[serde(default)]
    pub escalate_to_human: bool,

    /// Ground-truth artifact citations. Each load-bearing claim in
    /// `summary` MUST map to one of these. Verified by the gate before
    /// done_clean is accepted. (#8, #18, #25, #26, #72, #76)
    #[serde(default)]
    pub artifacts: Vec<DoneArtifact>,
    /// Provenance of this compatibility JSON when it was projected from the
    /// V3 mission ledger. `None` identifies a pre-V3 or standalone worker.
    #[serde(default)]
    pub projection: Option<ProjectionProvenance>,
    /// Exact session generation that produced this signal. Legacy signals
    /// deserialize as `None` but cannot authorize automated exact cleanup.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dispatch_generation: Option<String>,
    /// Exact writable scope generation owned by this dispatch, when any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_claim_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DoneStatus {
    DoneClean,
    Pending,
    Failed,
    Blocked,
}

/// What the worker's verification actually covered, vs what it did NOT.
/// Forces the report to state the BOUNDS of its claims (#100).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScopeFraming {
    #[serde(default)]
    pub confirms: Vec<String>,
    #[serde(default)]
    pub does_not_confirm: Vec<String>,
}

/// One independent signal that agreed on the verdict. Multiple = the
/// "corroborated" badge on the report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorroborationSource {
    /// The worker's own narrative — never sufficient on its own.
    WorkerSelfReport,
    /// `git ls-remote` / `git cat-file -e` confirmed the SHA/branch exists on origin.
    GitRemote,
    /// CI / build exit code from a real run (logged).
    CiExitCode,
    /// HTTP probe against the deployed URL returned the expected status.
    ProdHealthcheck,
    /// Filesystem check confirmed an asserted file/dir actually exists.
    FilesystemCheck,
    /// A separate Oracle/auditor signed off (multi-grader consensus).
    IndependentAuditor,
    /// Free-form named signal (when none of the above fits).
    Other(String),
}

/// Anthropic's 4-failure-mode taxonomy from the Opus 4.8 card (#31).
/// Trivial to add, makes the failure ledger comparable across runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureMode {
    /// Worker stated something that doesn't exist / didn't happen.
    Fabrication,
    /// Worker did not follow the brief's actual instructions.
    InstructionFollowing,
    /// Worker only verified the cheap/checkable surface and skipped the
    /// hard one.
    CheapVerificationSkipped,
    /// Worker spotted a problem, was told to fix it, and didn't.
    IgnoredCorrection,
}

/// A single ground-truth artifact cited as evidence for a claim in the
/// summary. `verify_done_against_repo` will check each of these against
/// the real environment before accepting done_clean.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum DoneArtifact {
    /// Full Git commit SHA reachable from the live origin/main (or named
    /// origin branch). A merely-local object is not proof.
    GitSha {
        sha: String,
        #[serde(default)]
        branch: Option<String>,
    },
    /// Existing branch confirmed against the live `origin` remote.
    GitBranch { name: String },
    /// Path that must exist after canonicalization inside the supplied repo
    /// root. Absolute paths outside that root and symlink escapes are rejected.
    FilePath { path: String },
    /// Command + expected exit code (recorded from the actual run).
    Command { cmd: String, exit_code: i32 },
    /// URL that must return 2xx when probed.
    Url { url: String, expected_status: u16 },
    /// Free-form note (no automated verification).
    Note(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionProvenance {
    pub source: String,
    pub event_id: String,
    pub event_sequence: u64,
    pub mission_version: u64,
    pub projection_hash: String,
}

impl DoneSignal {
    pub fn new(session: &str, status: DoneStatus, summary: &str) -> Self {
        Self {
            session: session.to_string(),
            status,
            summary: summary.to_string(),
            commit: None,
            finished_at: Utc::now(),
            todos_total: 0,
            todos_completed: 0,
            pending_actions: Vec::new(),
            not_done: Vec::new(),
            scope: ScopeFraming::default(),
            corroboration: Vec::new(),
            failure_mode: None,
            retry_thrash_count: 0,
            escalate_to_human: false,
            artifacts: Vec::new(),
            projection: None,
            dispatch_generation: None,
            scope_claim_id: None,
        }
    }

    /// Minimal constructor for tests and synthetic signals. Sets `session`
    /// and `status` to the args and zeroes/empties every other field.
    pub fn stub(session: &str, status: DoneStatus) -> Self {
        Self {
            session: session.to_string(),
            status,
            summary: String::new(),
            commit: None,
            finished_at: Utc::now(),
            todos_total: 0,
            todos_completed: 0,
            pending_actions: Vec::new(),
            not_done: Vec::new(),
            scope: ScopeFraming::default(),
            corroboration: Vec::new(),
            failure_mode: None,
            retry_thrash_count: 0,
            escalate_to_human: false,
            artifacts: Vec::new(),
            projection: None,
            dispatch_generation: None,
            scope_claim_id: None,
        }
    }

    /// Stamp this completion with the immutable launch receipt. The caller
    /// must obtain the receipt before spawning; reading whatever scope happens
    /// to be current at done time would let a late A process bind itself to B.
    pub fn bind_dispatch_authority(
        &mut self,
        authority: &crate::session::SessionDispatchAuthority,
    ) -> Result<()> {
        authority.validate()?;
        if self.session != authority.session {
            bail!(
                "done signal session {} differs from dispatch authority {}",
                self.session,
                authority.session
            );
        }
        self.dispatch_generation = Some(authority.dispatch_generation.clone());
        self.scope_claim_id = authority.scope_claim_id.clone();
        Ok(())
    }

    /// Require this signal to come from exactly `authority`. Missing fields
    /// are a legacy/unprovenanced state and fail closed for automated cleanup.
    pub fn validate_dispatch_authority(
        &self,
        authority: &crate::session::SessionDispatchAuthority,
    ) -> Result<()> {
        authority.validate()?;
        if self.session != authority.session
            || self.dispatch_generation.as_deref() != Some(authority.dispatch_generation.as_str())
            || self.scope_claim_id.as_deref() != authority.scope_claim_id.as_deref()
        {
            bail!("done signal does not match the exact session dispatch generation");
        }
        Ok(())
    }

    fn validate_dispatch_binding_shape(&self) -> Result<()> {
        match (&self.dispatch_generation, &self.scope_claim_id) {
            (None, None) => Ok(()),
            (None, Some(_)) => bail!("scope claim generation has no dispatch generation"),
            (Some(generation), scope_claim_id) => {
                if generation.len() != 64
                    || !generation.bytes().all(|byte| byte.is_ascii_hexdigit())
                {
                    bail!("done signal contains an invalid dispatch generation");
                }
                if let Some(claim_id) = scope_claim_id {
                    if claim_id.len() != 64
                        || !claim_id.bytes().all(|byte| byte.is_ascii_hexdigit())
                    {
                        bail!("done signal contains an invalid scope claim generation");
                    }
                }
                Ok(())
            }
        }
    }

    fn validate_against_current_dispatch(&self, state_dir: &Path, writing: bool) -> Result<()> {
        self.validate_dispatch_binding_shape()?;
        match crate::session::SessionDispatchAuthority::read_strict(state_dir, &self.session)? {
            Some(authority) => self.validate_dispatch_authority(&authority),
            None if writing && self.dispatch_generation.is_some() => {
                bail!("generated done signal has no current session dispatch authority")
            }
            None => Ok(()),
        }
    }

    /// Single-source = the worker's word only, no independent ground-truth.
    /// Reports flagged single-source must be shown to the user as such (#65).
    pub fn is_single_source(&self) -> bool {
        self.corroboration.is_empty()
            || self.corroboration == vec![CorroborationSource::WorkerSelfReport]
    }

    pub fn write(&self, state_dir: &Path) -> Result<()> {
        crate::scope::validate_session_identity(&self.session)?;
        self.validate_against_current_dispatch(state_dir, true)?;
        let path = state_dir.join(format!("worker-{}.done.json", self.session));
        let _lock = crate::scope::lock_private_state_file(state_dir, COMPLETION_SIGNAL_LOCK)?;
        if let Some(existing) = read_bounded_private_json::<Self>(&path)? {
            if existing.session != self.session {
                bail!("existing done signal identity differs from its filename");
            }
        }
        let content = serde_json::to_vec_pretty(self)?;
        if content.len() > MAX_COMPLETION_SIGNAL_BYTES {
            bail!("done signal exceeds the authority size limit");
        }
        crate::config::atomic_write_private(&path, &content)?;
        Ok(())
    }

    pub fn read(state_dir: &Path, session: &str) -> Result<Option<Self>> {
        crate::scope::validate_session_identity(session)?;
        let path = state_dir.join(format!("worker-{}.done.json", session));
        let signal = read_bounded_private_json::<Self>(&path)?;
        if signal
            .as_ref()
            .is_some_and(|signal| signal.session != session)
        {
            bail!("done signal identity differs from requested session");
        }
        if let Some(signal) = &signal {
            signal.validate_against_current_dispatch(state_dir, false)?;
        }
        Ok(signal)
    }

    pub fn is_complete(&self) -> bool {
        self.status == DoneStatus::DoneClean
            && self.todos_total > 0
            && self.todos_completed == self.todos_total
            && self.pending_actions.is_empty()
            && self.not_done.is_empty()
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self.status, DoneStatus::DoneClean | DoneStatus::Failed)
    }

    /// Read all done signals in state directory.
    pub fn read_all(state_dir: &Path) -> Vec<Self> {
        let mut results = Vec::new();
        if let Ok(entries) = std::fs::read_dir(state_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("worker-") && name.ends_with(".done.json") {
                        let Some(session) = name
                            .strip_prefix("worker-")
                            .and_then(|name| name.strip_suffix(".done.json"))
                        else {
                            continue;
                        };
                        if let Ok(Some(signal)) = Self::read(state_dir, session) {
                            results.push(signal);
                        }
                    }
                }
            }
        }
        results
    }

    /// Clean up done signal file after processing.
    pub fn remove(state_dir: &Path, session: &str) -> Result<()> {
        crate::scope::validate_session_identity(session)?;
        let path = state_dir.join(format!("worker-{}.done.json", session));
        let _lock = crate::scope::lock_private_state_file(state_dir, COMPLETION_SIGNAL_LOCK)?;
        if let Some(signal) = read_bounded_private_json::<Self>(&path)? {
            if signal.session != session {
                bail!("done signal identity differs from requested removal");
            }
            crate::scope::remove_private_file(&path)?;
        }
        Ok(())
    }
}

/// Result of running the ground-truth gate against a done.json.
/// (#8, #18, #25, #26, #72, #76, #82 — "worker narration is inadmissible
/// as proof; every claim must map to a real, runnable artifact.")
#[derive(Debug, Clone)]
pub struct GroundTruthVerdict {
    /// True iff every artifact citation in the done signal exists in
    /// the real environment AND no fabricated reference was detected.
    pub passes: bool,
    /// Per-artifact verification result, in the order the worker
    /// declared them.
    pub checks: Vec<ArtifactCheck>,
    /// Human-readable failure summary (empty when passes == true).
    pub failures: Vec<String>,
}

/// What a single artifact check actually established.
///
/// A two-valued `passed: bool` cannot say "the check could not run", so an
/// UNRUN check (no repo root supplied, `git` missing, an io error) used to be
/// indistinguishable from a proven fabrication. Patrol read that `false` as a
/// concrete fabrication, contested the worker, and escalated a human — four
/// spurious escalations on mission OmegaOS-m-8fe7d35df5bf. The doctrine it
/// broke is patrol's own: a concrete fabrication is a failure, weak or absent
/// proof stays a candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckOutcome {
    /// The check ran and the artifact is really there.
    Verified,
    /// The check could NOT run, so it establishes nothing either way:
    /// no repo root was supplied, the tool failed to execute, the claim was
    /// never predeclared and was therefore not rerun. Absent proof — never
    /// evidence of fabrication.
    Unverifiable,
    /// The check ran against the real environment and the artifact is NOT
    /// there (or the rerun refuted the claim). This is the fabrication.
    Contradicted,
}

impl CheckOutcome {
    /// True only for a check that ran and confirmed the artifact.
    pub fn is_verified(self) -> bool {
        matches!(self, CheckOutcome::Verified)
    }

    /// True only for a check that ran and REFUTED the claim. This is the sole
    /// predicate that may contest a worker.
    pub fn is_contradicted(self) -> bool {
        matches!(self, CheckOutcome::Contradicted)
    }
}

#[derive(Debug, Clone)]
pub struct ArtifactCheck {
    pub artifact: DoneArtifact,
    /// Kept for every existing caller: exactly `outcome.is_verified()`.
    /// Never read it to decide "fabricated" — use `outcome`.
    pub passed: bool,
    pub outcome: CheckOutcome,
    pub detail: String,
}

/// Verify every artifact a worker cited in its done.json against the
/// real environment. Returns `passes = false` if any citation refers to
/// something that doesn't actually exist — the "hallucinated commits"
/// failure mode the Opus 4.8 card flags as the demo-worthy keystone
/// (BLUF: "Worker narration inadmissible as proof").
///
/// `repo_root` is the directory the worker was supposed to operate in
/// (typically the project working_dir). Pass `None` to skip git/path
/// checks (e.g. for non-code workers).
pub fn verify_done_against_repo(done: &DoneSignal, repo_root: Option<&Path>) -> GroundTruthVerdict {
    verify_done_internal(done, repo_root, &[])
}

/// Verify a done signal and rerun only command/HTTP checks that were frozen in
/// the accepted task contract. Legacy callers remain fail-closed: a command or
/// URL cited only by worker narration is not executed and is not accepted.
pub fn verify_done_against_contract(
    done: &DoneSignal,
    repo_root: Option<&Path>,
    verifier_checks: &[VerifierCheck],
) -> GroundTruthVerdict {
    verify_done_internal(done, repo_root, verifier_checks)
}

fn verify_done_internal(
    done: &DoneSignal,
    repo_root: Option<&Path>,
    verifier_checks: &[VerifierCheck],
) -> GroundTruthVerdict {
    let mut checks = Vec::with_capacity(done.artifacts.len());
    let mut failures = Vec::new();

    for art in &done.artifacts {
        let (outcome, detail) = match art {
            DoneArtifact::GitSha { sha, branch } => {
                check_git_sha(sha, branch.as_deref(), repo_root)
            }
            DoneArtifact::GitBranch { name } => check_git_branch(name, repo_root),
            DoneArtifact::FilePath { path } => check_file_path(path, repo_root),
            DoneArtifact::Command { cmd, exit_code } => {
                verify_predeclared_command(cmd, *exit_code, repo_root, verifier_checks)
            }
            DoneArtifact::Url {
                url,
                expected_status,
            } => verify_predeclared_http(url, *expected_status, verifier_checks),
            // A note proves nothing, but it also refutes nothing: it is the
            // absence of evidence, not evidence of fabrication.
            DoneArtifact::Note(s) => (
                CheckOutcome::Unverifiable,
                format!("note is context only, never proof: {}", s),
            ),
        };
        if !outcome.is_verified() {
            failures.push(detail.clone());
        }
        checks.push(ArtifactCheck {
            artifact: art.clone(),
            passed: outcome.is_verified(),
            outcome,
            detail,
        });
    }

    if done.status == DoneStatus::DoneClean {
        if done.todos_total == 0 {
            failures.push(
                "done_clean rejected: the 0/0 task count proves no work and fails closed"
                    .to_string(),
            );
        } else if done.todos_completed != done.todos_total {
            failures.push(format!(
                "done_clean rejected: completed {}/{} declared tasks",
                done.todos_completed, done.todos_total
            ));
        }
        if !done.pending_actions.is_empty() {
            failures.push(format!(
                "done_clean rejected: {} pending action(s) remain",
                done.pending_actions.len()
            ));
        }
        if !done.not_done.is_empty() {
            failures.push(format!(
                "done_clean rejected: {} item(s) explicitly not done",
                done.not_done.len()
            ));
        }
        if done.artifacts.is_empty() && done.is_single_source() {
            failures.push(
                "done_clean asserted with zero artifacts and no independent corroboration (single-source). Worker narration is inadmissible.".to_string(),
            );
        }
    }

    GroundTruthVerdict {
        passes: failures.is_empty(),
        checks,
        failures,
    }
}

fn verify_predeclared_command(
    claimed_cmd: &str,
    claimed_exit: i32,
    repo_root: Option<&Path>,
    verifier_checks: &[VerifierCheck],
) -> (CheckOutcome, String) {
    let approved = verifier_checks.iter().find(|check| {
        matches!(
            &check.kind,
            VerifierCheckKind::Command {
                argv,
                expected_exit_code,
                ..
            } if argv.join(" ") == claimed_cmd && *expected_exit_code == claimed_exit
        )
    });
    let Some(approved) = approved else {
        return (
            CheckOutcome::Unverifiable,
            format!(
                "command was not an exact predeclared verifier and was not executed: `{claimed_cmd}`"
            ),
        );
    };
    let VerifierCheckKind::Command {
        argv,
        cwd,
        expected_exit_code,
    } = &approved.kind
    else {
        unreachable!("matched command verifier");
    };
    let Some(program) = argv.first().filter(|program| !program.is_empty()) else {
        return (
            CheckOutcome::Unverifiable,
            "predeclared verifier command has empty argv".to_string(),
        );
    };

    let mut command = Command::new(program);
    command
        .args(&argv[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Some(cwd) = cwd {
        let Some(root) = repo_root else {
            return (
                CheckOutcome::Unverifiable,
                format!("verifier `{claimed_cmd}` declares cwd but no repo root was supplied"),
            );
        };
        let root = match root.canonicalize() {
            Ok(root) => root,
            Err(error) => {
                return (
                    CheckOutcome::Unverifiable,
                    format!("cannot canonicalize verifier repo root: {error}"),
                )
            }
        };
        let requested = root.join(cwd);
        let requested = match requested.canonicalize() {
            Ok(requested) if requested.starts_with(&root) => requested,
            Ok(_) => {
                return (
                    CheckOutcome::Unverifiable,
                    format!("predeclared verifier cwd escapes repo root: {cwd}"),
                )
            }
            Err(error) => {
                return (
                    CheckOutcome::Unverifiable,
                    format!("cannot resolve predeclared verifier cwd `{cwd}`: {error}"),
                )
            }
        };
        command.current_dir(requested);
    } else if let Some(root) = repo_root {
        command.current_dir(root);
    }

    let output = match run_bounded_command(
        command,
        StdDuration::from_secs(approved.timeout_secs.max(1)),
    ) {
        Ok(output) => output,
        Err(error) => {
            return (
                CheckOutcome::Unverifiable,
                format!("failed to execute predeclared verifier `{claimed_cmd}`: {error}"),
            )
        }
    };
    let actual = output.status.code().unwrap_or(-1);
    if actual == *expected_exit_code {
        (
            CheckOutcome::Verified,
            format!("predeclared verifier `{claimed_cmd}` reran with exit code {actual}"),
        )
    } else {
        (
            CheckOutcome::Contradicted,
            format!(
                "predeclared verifier `{claimed_cmd}` exited {actual}, expected {expected_exit_code}"
            ),
        )
    }
}

struct BoundedCommandOutput {
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn read_bounded_pipe(mut pipe: impl Read) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    pipe.by_ref()
        .take(MAX_VERIFIER_OUTPUT_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_VERIFIER_OUTPUT_BYTES {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "verifier output exceeded the 1 MiB limit",
        ));
    }
    Ok(bytes)
}

fn run_bounded_command(
    mut command: Command,
    timeout: StdDuration,
) -> std::io::Result<BoundedCommandOutput> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command.spawn()?;
    let process_group = child.id();
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| std::io::Error::other("bounded command stdout pipe was not created"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| std::io::Error::other("bounded command stderr pipe was not created"))?;
    let stdout_reader = std::thread::spawn(move || read_bounded_pipe(stdout));
    let stderr_reader = std::thread::spawn(move || read_bounded_pipe(stderr));
    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait()? {
            Some(status) => {
                terminate_verifier_process_tree(&mut child, process_group)?;
                break status;
            }
            None if Instant::now() < deadline => {
                std::thread::sleep(StdDuration::from_millis(10));
            }
            None => {
                terminate_verifier_process_tree(&mut child, process_group)?;
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    format!("command timed out after {}s", timeout.as_secs()),
                ));
            }
        }
    };
    let stdout = stdout_reader
        .join()
        .map_err(|_| std::io::Error::other("stdout reader panicked"))??;
    let stderr = stderr_reader
        .join()
        .map_err(|_| std::io::Error::other("stderr reader panicked"))??;
    Ok(BoundedCommandOutput {
        status,
        stdout,
        stderr,
    })
}

#[cfg(unix)]
fn signal_process_group(process_group: u32, signal: i32) -> std::io::Result<bool> {
    unsafe extern "C" {
        fn kill(pid: i32, signal: i32) -> i32;
    }
    let Ok(process_group) = i32::try_from(process_group) else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "process group id exceeds i32",
        ));
    };
    // Negative pid addresses the complete process group. ESRCH means every
    // member already exited and is therefore success for containment.
    let result = unsafe { kill(-process_group, signal) };
    if result == 0 {
        Ok(true)
    } else {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(3) {
            Ok(false)
        } else {
            Err(error)
        }
    }
}

#[cfg(unix)]
fn process_group_exists(process_group: u32) -> std::io::Result<bool> {
    signal_process_group(process_group, 0)
}

#[cfg(unix)]
fn terminate_verifier_process_tree(
    child: &mut std::process::Child,
    process_group: u32,
) -> std::io::Result<()> {
    const SIGTERM: i32 = 15;
    const SIGKILL: i32 = 9;
    signal_process_group(process_group, SIGTERM)?;
    let deadline = Instant::now() + StdDuration::from_millis(250);
    while process_group_exists(process_group)? && Instant::now() < deadline {
        let _ = child.try_wait()?;
        std::thread::sleep(StdDuration::from_millis(10));
    }
    if process_group_exists(process_group)? {
        signal_process_group(process_group, SIGKILL)?;
        let _ = child.wait()?;
        let kill_deadline = Instant::now() + StdDuration::from_millis(250);
        while process_group_exists(process_group)? && Instant::now() < kill_deadline {
            std::thread::sleep(StdDuration::from_millis(10));
        }
    }
    if process_group_exists(process_group)? {
        return Err(std::io::Error::other(
            "verifier process group remained alive after SIGKILL",
        ));
    }
    Ok(())
}

#[cfg(not(unix))]
fn terminate_verifier_process_tree(
    child: &mut std::process::Child,
    _process_group: u32,
) -> std::io::Result<()> {
    match child.try_wait()? {
        Some(_) => Ok(()),
        None => {
            child.kill()?;
            child.wait()?;
            Ok(())
        }
    }
}

fn verify_predeclared_http(
    claimed_url: &str,
    claimed_status: u16,
    verifier_checks: &[VerifierCheck],
) -> (CheckOutcome, String) {
    let approved = verifier_checks.iter().find(|check| {
        matches!(
            &check.kind,
            VerifierCheckKind::Http {
                url,
                expected_status
            } if url == claimed_url && *expected_status == claimed_status
        )
    });
    let Some(approved) = approved else {
        return (
            CheckOutcome::Unverifiable,
            format!(
                "URL was not an exact predeclared verifier and was not requested: {claimed_url}"
            ),
        );
    };
    let (host, port, pinned_ip) = match resolve_public_http_target(claimed_url) {
        Ok(target) => target,
        Err(error) => return (CheckOutcome::Unverifiable, error),
    };

    let timeout = approved.timeout_secs.max(1).to_string();
    let resolve = match pinned_ip {
        IpAddr::V4(address) => format!("{host}:{port}:{address}"),
        IpAddr::V6(address) => format!("{host}:{port}:[{address}]"),
    };
    let mut command = Command::new("curl");
    command.args([
        "--silent",
        "--show-error",
        "--output",
        "/dev/null",
        "--write-out",
        "%{http_code}",
        "--proto",
        "=http,https",
        "--proto-redir",
        "=http,https",
        "--noproxy",
        "*",
        "--max-redirs",
        "0",
        "--resolve",
        &resolve,
        "--max-time",
        &timeout,
        claimed_url,
    ]);
    let output = run_bounded_command(
        command,
        StdDuration::from_secs(approved.timeout_secs.max(1).saturating_add(1)),
    );
    match output {
        Ok(output) if output.status.success() => {
            let actual = String::from_utf8_lossy(&output.stdout)
                .trim()
                .parse::<u16>();
            match actual {
                Ok(actual) if matches!(actual, 401 | 403) => (
                    CheckOutcome::Contradicted,
                    format!(
                        "predeclared HTTP verifier was unauthorized ({actual}): {claimed_url}"
                    ),
                ),
                Ok(actual) if (300..400).contains(&actual) => (
                    CheckOutcome::Contradicted,
                    format!(
                        "predeclared HTTP verifier returned a redirect ({actual}); redirects are never followed or accepted: {claimed_url}"
                    ),
                ),
                Ok(actual) if actual == claimed_status => (
                    CheckOutcome::Verified,
                    format!(
                        "predeclared HTTP verifier returned {actual}: {claimed_url}"
                    ),
                ),
                Ok(actual) => (
                    CheckOutcome::Contradicted,
                    format!(
                        "predeclared HTTP verifier returned {actual}, expected {claimed_status}: {claimed_url}"
                    ),
                ),
                Err(error) => (
                    CheckOutcome::Unverifiable,
                    format!("HTTP verifier returned invalid status output: {error}"),
                ),
            }
        }
        Ok(output) => (
            CheckOutcome::Unverifiable,
            format!(
                "predeclared HTTP verifier failed with exit {:?}: {}",
                output.status.code(),
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        ),
        Err(error) => (
            CheckOutcome::Unverifiable,
            format!("could not run predeclared HTTP verifier: {error}"),
        ),
    }
}

fn resolve_public_http_target(url: &str) -> Result<(String, u16, IpAddr), String> {
    if url.chars().any(char::is_whitespace)
        || url.chars().any(char::is_control)
        || url.contains('\\')
    {
        return Err(format!("predeclared URL is unsafe or malformed: {url}"));
    }
    let (remainder, default_port) = if let Some(remainder) = url.strip_prefix("https://") {
        (remainder, 443)
    } else if let Some(remainder) = url.strip_prefix("http://") {
        (remainder, 80)
    } else {
        return Err(format!("predeclared URL has a non-HTTP scheme: {url}"));
    };
    let authority_end = remainder.find(['/', '?', '#']).unwrap_or(remainder.len());
    let authority = &remainder[..authority_end];
    if authority.is_empty() || authority.contains('@') {
        return Err(format!("predeclared URL has unsafe authority: {url}"));
    }
    let (host, port) = if let Some(bracketed) = authority.strip_prefix('[') {
        let close = bracketed
            .find(']')
            .ok_or_else(|| format!("predeclared URL has malformed IPv6 authority: {url}"))?;
        let host = &bracketed[..close];
        let suffix = &bracketed[close + 1..];
        let port = if suffix.is_empty() {
            default_port
        } else {
            suffix
                .strip_prefix(':')
                .ok_or_else(|| format!("predeclared URL has malformed authority: {url}"))?
                .parse::<u16>()
                .map_err(|_| format!("predeclared URL has invalid port: {url}"))?
        };
        (host.to_string(), port)
    } else {
        let colon_count = authority.bytes().filter(|byte| *byte == b':').count();
        if colon_count > 1 {
            return Err(format!("predeclared IPv6 URL must use brackets: {url}"));
        }
        match authority.rsplit_once(':') {
            Some((host, port)) => (
                host.to_string(),
                port.parse::<u16>()
                    .map_err(|_| format!("predeclared URL has invalid port: {url}"))?,
            ),
            None => (authority.to_string(), default_port),
        }
    };
    if port == 0 || host.is_empty() {
        return Err(format!(
            "predeclared URL has empty host or zero port: {url}"
        ));
    }
    let normalized = host.trim_end_matches('.').to_ascii_lowercase();
    if normalized.is_empty()
        || normalized == "localhost"
        || normalized.ends_with(".localhost")
        || matches!(
            normalized.as_str(),
            "metadata" | "metadata.google.internal" | "instance-data"
        )
    {
        return Err(format!(
            "predeclared URL targets a local/metadata host: {url}"
        ));
    }
    if !normalized
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b':'))
        || normalized.starts_with('-')
    {
        return Err(format!("predeclared URL host has unsafe syntax: {url}"));
    }
    let addresses: Vec<IpAddr> = if let Ok(address) = normalized.parse() {
        vec![address]
    } else {
        let mut resolver = Command::new("getent");
        resolver.args(["ahosts", &normalized]);
        let output = run_bounded_command(resolver, StdDuration::from_secs(10))
            .map_err(|error| format!("bounded resolver failed for {host}: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "cannot resolve predeclared URL host {host}: getent exited {:?}: {}",
                output.status.code(),
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        let mut addresses = Vec::new();
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            if let Some(address) = line
                .split_whitespace()
                .next()
                .and_then(|value| value.parse::<IpAddr>().ok())
            {
                if !addresses.contains(&address) {
                    addresses.push(address);
                }
            }
        }
        addresses
    };
    if addresses.is_empty() {
        return Err(format!(
            "predeclared URL host resolved to no addresses: {host}"
        ));
    }
    if let Some(address) = addresses.iter().find(|address| !is_public_ip(**address)) {
        return Err(format!(
            "predeclared URL resolves to forbidden non-public address {address}: {url}"
        ));
    }
    let pinned = addresses[0];
    Ok((host, port, pinned))
}

fn is_public_ip(address: IpAddr) -> bool {
    match address {
        IpAddr::V4(address) => {
            let octets = address.octets();
            !matches!(
                octets,
                [0, ..]
                    | [10, ..]
                    | [127, ..]
                    | [169, 254, ..]
                    | [192, 168, ..]
                    | [192, 0, 0, ..]
                    | [192, 0, 2, ..]
                    | [198, 51, 100, ..]
                    | [203, 0, 113, ..]
            ) && !(octets[0] == 100 && (64..=127).contains(&octets[1]))
                && !(octets[0] == 172 && (16..=31).contains(&octets[1]))
                && !(octets[0] == 198 && matches!(octets[1], 18 | 19))
                && octets[0] < 224
        }
        IpAddr::V6(address) => {
            if let Some(mapped) = address.to_ipv4_mapped() {
                return is_public_ip(IpAddr::V4(mapped));
            }
            let segments = address.segments();
            (segments[0] & 0xe000) == 0x2000 && !(segments[0] == 0x2001 && segments[1] == 0x0db8)
        }
    }
}

fn check_git_sha(
    sha: &str,
    branch: Option<&str>,
    repo_root: Option<&Path>,
) -> (CheckOutcome, String) {
    let Some(root) = repo_root else {
        // Nothing was looked up. Saying FABRICATED here is a lie.
        return (
            CheckOutcome::Unverifiable,
            format!("no repo root supplied; SHA {sha} was not verified"),
        );
    };
    if !(sha.len() == 40 || sha.len() == 64) || !sha.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return (
            CheckOutcome::Contradicted,
            format!("claimed git SHA has invalid full-object syntax: {sha}"),
        );
    }
    let branch = match validate_git_branch(branch.unwrap_or("main"), root) {
        Ok(branch) => branch,
        Err(error) => return (CheckOutcome::Contradicted, error),
    };
    let mut cat_file = Command::new("git");
    cat_file.args(["cat-file", "-e", sha]).current_dir(root);
    let out = run_bounded_command(cat_file, StdDuration::from_secs(15));
    match out {
        Ok(output) if output.status.success() => {}
        // git ran and answered: the object is not in this repo.
        Ok(_) => {
            return (
                CheckOutcome::Contradicted,
                format!(
                    "claimed git SHA {} does NOT exist in repo — FABRICATED",
                    sha
                ),
            )
        }
        // git itself never ran (missing binary, io error) — that is our
        // failure, not the worker's.
        Err(e) => {
            return (
                CheckOutcome::Unverifiable,
                format!("git lookup failed for {}: {}", sha, e),
            )
        }
    }
    let remote_ref = format!("refs/heads/{branch}");
    let mut ls_remote = Command::new("git");
    ls_remote
        .args(["ls-remote", "--exit-code", "origin", &remote_ref])
        .current_dir(root)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_SSH_COMMAND", "ssh -oBatchMode=yes -oConnectTimeout=10");
    let remote = run_bounded_command(ls_remote, StdDuration::from_secs(30));
    let remote = match remote {
        Ok(output) if output.status.success() => output,
        Ok(output) => {
            return (
                CheckOutcome::Unverifiable,
                format!(
                    "could not confirm origin/{branch}: git ls-remote exited {:?}: {}",
                    output.status.code(),
                    String::from_utf8_lossy(&output.stderr).trim()
                ),
            )
        }
        Err(error) => {
            return (
                CheckOutcome::Unverifiable,
                format!("could not run git ls-remote for origin/{branch}: {error}"),
            )
        }
    };
    let remote_tip = String::from_utf8_lossy(&remote.stdout)
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string();
    if remote_tip.is_empty() || !remote_tip.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return (
            CheckOutcome::Unverifiable,
            format!("origin/{branch} returned no valid object id"),
        );
    }
    let mut merge_base = Command::new("git");
    merge_base
        .args(["merge-base", "--is-ancestor", sha, &remote_tip])
        .current_dir(root);
    match run_bounded_command(merge_base, StdDuration::from_secs(15)) {
        Ok(output) if output.status.success() => (
            CheckOutcome::Verified,
            format!("git SHA {sha} is reachable from live origin/{branch} tip {remote_tip}"),
        ),
        Ok(output) if output.status.code() == Some(1) => (
            CheckOutcome::Contradicted,
            format!("claimed git SHA {sha} is not reachable from origin/{branch}"),
        ),
        Ok(output) => (
            CheckOutcome::Unverifiable,
            format!(
                "git could not compare SHA {sha} with origin/{branch}: exit {:?}",
                output.status.code()
            ),
        ),
        Err(error) => (
            CheckOutcome::Unverifiable,
            format!("git ancestry check failed for {sha}: {error}"),
        ),
    }
}

fn check_git_branch(name: &str, repo_root: Option<&Path>) -> (CheckOutcome, String) {
    let Some(root) = repo_root else {
        return (
            CheckOutcome::Unverifiable,
            format!("no repo root supplied; branch {name} was not verified"),
        );
    };
    let branch = match validate_git_branch(name, root) {
        Ok(branch) => branch,
        Err(error) => return (CheckOutcome::Contradicted, error),
    };
    let remote_ref = format!("refs/heads/{branch}");
    let mut ls_remote = Command::new("git");
    ls_remote
        .args(["ls-remote", "--exit-code", "origin", &remote_ref])
        .current_dir(root)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_SSH_COMMAND", "ssh -oBatchMode=yes -oConnectTimeout=10");
    match run_bounded_command(ls_remote, StdDuration::from_secs(30)) {
        Ok(output) if output.status.success() => (
            CheckOutcome::Verified,
            format!("git branch origin/{branch} exists on the live remote"),
        ),
        Ok(output) if output.status.code() == Some(2) => (
            CheckOutcome::Contradicted,
            format!("claimed git branch origin/{branch} NOT found — FABRICATED"),
        ),
        Ok(output) => (
            CheckOutcome::Unverifiable,
            format!(
                "live origin lookup failed for branch {branch}: exit {:?}: {}",
                output.status.code(),
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        ),
        Err(e) => (
            CheckOutcome::Unverifiable,
            format!("bounded git lookup failed for branch {branch}: {e}"),
        ),
    }
}

fn validate_git_branch(name: &str, repo_root: &Path) -> Result<String, String> {
    let branch = name
        .strip_prefix("refs/heads/")
        .or_else(|| name.strip_prefix("refs/remotes/origin/"))
        .or_else(|| name.strip_prefix("origin/"))
        .unwrap_or(name);
    if branch.is_empty() || branch.starts_with('-') || branch.chars().any(char::is_control) {
        return Err(format!("claimed git branch is unsafe or malformed: {name}"));
    }
    let mut command = Command::new("git");
    command
        .args(["check-ref-format", "--branch", branch])
        .current_dir(repo_root);
    match run_bounded_command(command, StdDuration::from_secs(10)) {
        Ok(output) if output.status.success() => Ok(branch.to_string()),
        Ok(_) => Err(format!("claimed git branch is invalid: {name}")),
        Err(error) => Err(format!("could not validate git branch {name}: {error}")),
    }
}

fn check_file_path(path: &str, repo_root: Option<&Path>) -> (CheckOutcome, String) {
    let Some(root) = repo_root else {
        return (
            CheckOutcome::Unverifiable,
            format!("no repo root supplied; artifact path {path} was not verified"),
        );
    };
    let root = match root.canonicalize() {
        Ok(root) => root,
        Err(error) => {
            return (
                CheckOutcome::Unverifiable,
                format!("cannot canonicalize artifact repo root: {error}"),
            )
        }
    };
    let candidate = Path::new(path);
    let full = if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        root.join(candidate)
    };
    let canonical = match full.canonicalize() {
        Ok(canonical) => canonical,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return (
                CheckOutcome::Contradicted,
                format!(
                    "claimed file path {} does NOT exist — FABRICATED",
                    full.display()
                ),
            )
        }
        Err(error) => {
            return (
                CheckOutcome::Unverifiable,
                format!(
                    "cannot canonicalize claimed path {}: {error}",
                    full.display()
                ),
            )
        }
    };
    if !canonical.starts_with(&root) {
        return (
            CheckOutcome::Contradicted,
            format!("claimed artifact path escapes canonical repo root: {path}"),
        );
    }
    (
        CheckOutcome::Verified,
        format!("artifact {} exists inside repo", canonical.display()),
    )
}

#[cfg(test)]
mod done_v3_tests {
    use super::*;

    fn complete_signal() -> DoneSignal {
        let mut done = DoneSignal::stub("worker-a", DoneStatus::DoneClean);
        done.todos_total = 1;
        done.todos_completed = 1;
        done
    }

    #[test]
    fn done_signal_is_bound_to_exact_dispatch_and_scope_generation() {
        let claim_id = "c".repeat(64);
        let first = crate::session::SessionDispatchAuthority::generate("worker-a", Some(&claim_id))
            .unwrap();
        let replacement =
            crate::session::SessionDispatchAuthority::generate("worker-a", Some(&claim_id))
                .unwrap();
        let mut done = complete_signal();
        done.bind_dispatch_authority(&first).unwrap();
        done.validate_dispatch_authority(&first).unwrap();
        assert!(done.validate_dispatch_authority(&replacement).is_err());

        let legacy = DoneSignal::stub("worker-a", DoneStatus::Pending);
        assert!(legacy.validate_dispatch_authority(&first).is_err());
    }

    #[test]
    fn done_clean_zero_of_zero_fails_closed() {
        let done = DoneSignal::stub("worker-a", DoneStatus::DoneClean);
        assert!(!done.is_complete());
        let verdict = verify_done_against_repo(&done, None);
        assert!(!verdict.passes);
        assert!(verdict
            .failures
            .iter()
            .any(|failure| failure.contains("0/0")));
    }

    #[test]
    fn pending_or_not_done_items_reject_completion() {
        let mut done = complete_signal();
        done.pending_actions.push("deploy".to_string());
        done.not_done.push("production check".to_string());
        assert!(!done.is_complete());
        let verdict = verify_done_against_repo(&done, None);
        assert!(!verdict.passes);
        assert!(verdict
            .failures
            .iter()
            .any(|failure| failure.contains("pending action")));
    }

    #[test]
    fn note_is_context_and_never_proof() {
        let mut done = complete_signal();
        done.artifacts
            .push(DoneArtifact::Note("trust me".to_string()));
        let verdict = verify_done_against_repo(&done, None);
        assert!(!verdict.passes);
        assert!(!verdict.checks[0].passed);
    }

    #[test]
    fn undeclared_command_is_not_executed() {
        let temp = tempfile::tempdir().unwrap();
        let marker = temp.path().join("must-not-exist");
        let mut done = complete_signal();
        done.artifacts.push(DoneArtifact::Command {
            cmd: format!("touch {}", marker.display()),
            exit_code: 0,
        });
        let verdict = verify_done_against_repo(&done, Some(temp.path()));
        assert!(!verdict.passes);
        assert!(!marker.exists());
    }

    /// Create a throwaway repo with a real origin/main commit. Git artifacts
    /// are accepted only when the object is reachable from the live remote
    /// branch, not merely present in the local object database.
    fn repo_with_a_real_object() -> (tempfile::TempDir, String) {
        let temp = tempfile::tempdir().unwrap();
        let status = std::process::Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(temp.path())
            .status()
            .expect("git init");
        assert!(status.success(), "git init failed");
        for (key, value) in [
            ("user.email", "omega-test@example.invalid"),
            ("user.name", "Omega Test"),
        ] {
            assert!(std::process::Command::new("git")
                .args(["config", key, value])
                .current_dir(temp.path())
                .status()
                .unwrap()
                .success());
        }
        std::fs::write(temp.path().join("payload.txt"), b"ground truth").unwrap();
        assert!(std::process::Command::new("git")
            .args(["add", "payload.txt"])
            .current_dir(temp.path())
            .status()
            .unwrap()
            .success());
        assert!(std::process::Command::new("git")
            .args(["commit", "--quiet", "-m", "ground truth"])
            .current_dir(temp.path())
            .status()
            .unwrap()
            .success());
        assert!(std::process::Command::new("git")
            .args(["branch", "-M", "main"])
            .current_dir(temp.path())
            .status()
            .unwrap()
            .success());
        let origin = temp.path().join("origin.git");
        assert!(std::process::Command::new("git")
            .args(["init", "--quiet", "--bare"])
            .arg(&origin)
            .status()
            .unwrap()
            .success());
        assert!(std::process::Command::new("git")
            .args(["remote", "add", "origin"])
            .arg(&origin)
            .current_dir(temp.path())
            .status()
            .unwrap()
            .success());
        assert!(std::process::Command::new("git")
            .args(["push", "--quiet", "-u", "origin", "main"])
            .current_dir(temp.path())
            .status()
            .unwrap()
            .success());
        let out = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(temp.path())
            .output()
            .expect("git rev-parse");
        assert!(out.status.success(), "git rev-parse failed");
        let sha = String::from_utf8_lossy(&out.stdout).trim().to_string();
        assert_eq!(sha.len(), 40, "expected a full object SHA, got {sha:?}");
        (temp, sha)
    }

    /// MANDATORY TEST 1 (done.rs half). A REAL sha checked with NO repo root
    /// was never looked up, so it is `Unverifiable` — absent proof. Branding
    /// it `Contradicted` is the false positive that escalated four honest
    /// workers to a human on mission OmegaOS-m-8fe7d35df5bf.
    #[test]
    fn real_sha_without_a_repo_root_is_unverifiable_not_contradicted() {
        let (temp, sha) = repo_with_a_real_object();
        // The SHA is genuinely real: with the root, the same check verifies it.
        let mut done = complete_signal();
        done.artifacts.push(DoneArtifact::GitSha {
            sha: sha.clone(),
            branch: None,
        });
        let grounded = verify_done_against_repo(&done, Some(temp.path()));
        assert_eq!(grounded.checks[0].outcome, CheckOutcome::Verified);

        // Same real SHA, no root supplied: nothing ran, nothing is refuted.
        let verdict = verify_done_against_repo(&done, None);
        assert_eq!(
            verdict.checks[0].outcome,
            CheckOutcome::Unverifiable,
            "an unrun check must never be reported as a fabrication: {}",
            verdict.checks[0].detail
        );
        assert!(!verdict.checks[0].outcome.is_contradicted());
        assert!(
            !verdict.checks[0].passed,
            "`passed` stays outcome-is-Verified"
        );
        assert!(!verdict.passes, "absent proof still fails the verdict");
    }

    /// MANDATORY TEST 2 (done.rs half). The detector is NOT weakened: a bogus
    /// SHA looked up in a REAL repo is `Contradicted` — a concrete fabrication.
    #[test]
    fn bogus_sha_with_a_real_repo_root_is_contradicted() {
        let (temp, _real) = repo_with_a_real_object();
        let mut done = complete_signal();
        done.artifacts.push(DoneArtifact::GitSha {
            sha: "0123456789abcdef0123456789abcdef01234567".to_string(),
            branch: None,
        });
        let verdict = verify_done_against_repo(&done, Some(temp.path()));
        assert_eq!(
            verdict.checks[0].outcome,
            CheckOutcome::Contradicted,
            "a lookup that ran and found nothing IS the fabrication: {}",
            verdict.checks[0].detail
        );
        assert!(verdict.checks[0].outcome.is_contradicted());
        assert!(verdict.checks[0].detail.contains("FABRICATED"));
        assert!(!verdict.passes);
    }

    #[test]
    fn exact_predeclared_command_is_rerun_without_a_shell() {
        let mut done = complete_signal();
        done.artifacts.push(DoneArtifact::Command {
            cmd: "/bin/true".to_string(),
            exit_code: 0,
        });
        let verifier = VerifierCheck {
            schema_version: crate::mission::CONTRACT_SCHEMA_VERSION,
            check_id: "true-check".to_string(),
            kind: VerifierCheckKind::Command {
                argv: vec!["/bin/true".to_string()],
                cwd: None,
                expected_exit_code: 0,
            },
            timeout_secs: 2,
        };
        let verdict = verify_done_against_contract(&done, None, &[verifier]);
        assert!(verdict.passes, "{:?}", verdict.failures);
        assert!(verdict.checks[0].passed);
    }

    #[cfg(unix)]
    #[test]
    fn verifier_reaps_background_descendants_before_accepting() {
        let temp = tempfile::tempdir().unwrap();
        let marker = temp.path().join("escaped-descendant");
        let script = format!("(sleep 1; touch '{}') & exit 0", marker.display());
        let command_text = ["/bin/sh", "-c", script.as_str()].join(" ");
        let mut done = complete_signal();
        done.artifacts.push(DoneArtifact::Command {
            cmd: command_text.clone(),
            exit_code: 0,
        });
        let verifier = VerifierCheck {
            schema_version: crate::mission::CONTRACT_SCHEMA_VERSION,
            check_id: "descendant-containment".to_string(),
            kind: VerifierCheckKind::Command {
                argv: vec!["/bin/sh".to_string(), "-c".to_string(), script],
                cwd: None,
                expected_exit_code: 0,
            },
            timeout_secs: 2,
        };
        let verdict = verify_done_against_contract(&done, None, &[verifier]);
        assert!(verdict.passes, "{:?}", verdict.failures);
        std::thread::sleep(StdDuration::from_millis(1_200));
        assert!(
            !marker.exists(),
            "background verifier descendant escaped its process group"
        );
    }

    #[test]
    fn http_target_policy_rejects_ssrf_authorities() {
        for url in [
            "http://localhost/",
            "http://127.0.0.1/",
            "http://169.254.169.254/latest/meta-data/",
            "http://10.1.2.3/",
            "http://[::1]/",
            "http://user@example.com/",
            "http://example.com\\@127.0.0.1/",
        ] {
            assert!(
                resolve_public_http_target(url).is_err(),
                "unsafe target unexpectedly accepted: {url}"
            );
        }
        assert_eq!(
            resolve_public_http_target("https://8.8.8.8/health")
                .unwrap()
                .2,
            "8.8.8.8".parse::<IpAddr>().unwrap()
        );
    }

    #[test]
    fn artifact_path_is_confined_to_canonical_repo() {
        let repo = tempfile::tempdir().unwrap();
        let outside = tempfile::NamedTempFile::new().unwrap();
        let (outcome, _) = check_file_path(outside.path().to_str().unwrap(), Some(repo.path()));
        assert_eq!(outcome, CheckOutcome::Contradicted);
    }

    #[test]
    fn completion_signal_authority_rejects_traversal_corruption_and_mismatch() {
        let state = tempfile::tempdir().unwrap();
        assert!(DoneSignal::read(state.path(), "../escape").is_err());
        let mut traversal = DoneSignal::stub("../escape", DoneStatus::Pending);
        assert!(traversal.write(state.path()).is_err());

        let path = state.path().join("worker-worker-a.done.json");
        crate::config::atomic_write_private(&path, b"not-json").unwrap();
        assert!(DoneSignal::read(state.path(), "worker-a").is_err());

        traversal.session = "worker-b".to_string();
        crate::config::atomic_write_private(&path, &serde_json::to_vec_pretty(&traversal).unwrap())
            .unwrap();
        assert!(DoneSignal::read(state.path(), "worker-a").is_err());
        assert!(DoneSignal::remove(state.path(), "worker-a").is_err());
        assert!(path.exists(), "mismatched authority must not be deleted");
    }

    #[cfg(unix)]
    #[test]
    fn completion_signal_is_private_and_rejects_symlink_or_hardlink() {
        use std::os::unix::fs::{symlink, PermissionsExt};

        let state = tempfile::tempdir().unwrap();
        let signal = DoneSignal::stub("worker-a", DoneStatus::Pending);
        signal.write(state.path()).unwrap();
        let path = state.path().join("worker-worker-a.done.json");
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );

        let hardlink = state.path().join("hardlink-copy");
        std::fs::hard_link(&path, &hardlink).unwrap();
        assert!(DoneSignal::read(state.path(), "worker-a").is_err());
        std::fs::remove_file(&hardlink).unwrap();
        DoneSignal::remove(state.path(), "worker-a").unwrap();

        let target = state.path().join("target");
        std::fs::write(&target, b"do-not-follow").unwrap();
        symlink(&target, &path).unwrap();
        assert!(DoneSignal::read(state.path(), "worker-a").is_err());
        assert!(signal.write(state.path()).is_err());
        assert_eq!(std::fs::read(&target).unwrap(), b"do-not-follow");
    }
}

/// A structured record for when a worker is blocked but still executing a fallback.
/// Mirrors the live system's worker-blocked-*.json protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerBlocked {
    pub session: String,
    pub blocked_at: DateTime<Utc>,
    pub question: String,
    pub best_guess: String,
    pub fallback_action: String,
    pub can_resume_without_answer: bool,
}

impl WorkerBlocked {
    pub fn new(session: &str, question: &str, best_guess: &str, fallback_action: &str) -> Self {
        Self {
            session: session.to_string(),
            blocked_at: Utc::now(),
            question: question.to_string(),
            best_guess: best_guess.to_string(),
            fallback_action: fallback_action.to_string(),
            can_resume_without_answer: true,
        }
    }

    pub fn write(&self, state_dir: &Path) -> Result<()> {
        crate::scope::validate_session_identity(&self.session)?;
        let path = state_dir.join(format!("worker-blocked-{}.json", self.session));
        let _lock = crate::scope::lock_private_state_file(state_dir, COMPLETION_SIGNAL_LOCK)?;
        if let Some(existing) = read_bounded_private_json::<Self>(&path)? {
            if existing.session != self.session {
                bail!("existing blocked signal identity differs from its filename");
            }
        }
        let content = serde_json::to_vec_pretty(self)?;
        if content.len() > MAX_COMPLETION_SIGNAL_BYTES {
            bail!("blocked signal exceeds the authority size limit");
        }
        crate::config::atomic_write_private(&path, &content)?;
        Ok(())
    }

    pub fn read(state_dir: &Path, session: &str) -> Result<Option<Self>> {
        crate::scope::validate_session_identity(session)?;
        let path = state_dir.join(format!("worker-blocked-{}.json", session));
        let signal = read_bounded_private_json::<Self>(&path)?;
        if signal
            .as_ref()
            .is_some_and(|signal| signal.session != session)
        {
            bail!("blocked signal identity differs from requested session");
        }
        Ok(signal)
    }

    pub fn clear(state_dir: &Path, session: &str) -> Result<()> {
        crate::scope::validate_session_identity(session)?;
        let path = state_dir.join(format!("worker-blocked-{}.json", session));
        let _lock = crate::scope::lock_private_state_file(state_dir, COMPLETION_SIGNAL_LOCK)?;
        if let Some(signal) = read_bounded_private_json::<Self>(&path)? {
            if signal.session != session {
                bail!("blocked signal identity differs from requested clear");
            }
            crate::scope::remove_private_file(&path)?;
        }
        Ok(())
    }

    /// Read all blocked signals in state directory.
    pub fn read_all(state_dir: &Path) -> Vec<Self> {
        let mut results = Vec::new();
        if let Ok(entries) = std::fs::read_dir(state_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("worker-blocked-") && name.ends_with(".json") {
                        let Some(session) = name
                            .strip_prefix("worker-blocked-")
                            .and_then(|name| name.strip_suffix(".json"))
                        else {
                            continue;
                        };
                        if let Ok(Some(blocked)) = Self::read(state_dir, session) {
                            results.push(blocked);
                        }
                    }
                }
            }
        }
        results
    }
}

/// Oracle-level done signal — written when an oracle completes its mission.
/// Mirrors the VPS oracle-*.done.json schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleDoneSignal {
    pub oracle: String,
    pub project: String,
    pub status: DoneStatus,
    pub mission: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub duration_secs: u64,
    pub summary: String,
    pub ship: Option<OracleShipResult>,
    #[serde(default)]
    pub pending_actions: Vec<String>,
    #[serde(default)]
    pub lifecycle: OracleLifecycle,
    /// True when the L4 completeness gate downgraded a done_clean to Pending
    /// ONLY because the progress plan was not yet 100% (the oracle's own final
    /// "report" task is by contract still unfinished at `omega done` time —
    /// chicken-and-egg). `omega progress` and patrol upgrade such a signal back
    /// to DoneClean once the plan reaches 100% with no failed task. serde
    /// default keeps pre-existing done.json files readable (false = no gate).
    #[serde(default)]
    pub gate_pending: bool,
    /// Provenance of this compatibility JSON when it was projected from the
    /// V3 mission ledger. `None` identifies a pre-V3 Oracle.
    #[serde(default)]
    pub projection: Option<ProjectionProvenance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleShipResult {
    pub requested: bool,
    pub result: String,
    pub commit: Option<String>,
    pub push_url: Option<String>,
    pub deploy_url: Option<String>,
    pub deploy_status: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OracleLifecycle {
    Persistent,
    #[default]
    Ephemeral,
}

impl OracleDoneSignal {
    pub fn new(oracle: &str, project: &str, status: DoneStatus, mission: &str) -> Self {
        let now = Utc::now();
        Self {
            oracle: oracle.to_string(),
            project: project.to_string(),
            status,
            mission: mission.to_string(),
            started_at: now,
            finished_at: now,
            duration_secs: 0,
            summary: String::new(),
            ship: None,
            pending_actions: Vec::new(),
            lifecycle: OracleLifecycle::Ephemeral,
            gate_pending: false,
            projection: None,
        }
    }

    /// Canonical on-disk name is `oracle-<key>.done.json`, where `<key>` is the
    /// session name minus a single leading `oracle-` prefix (any numeric index
    /// is RETAINED: `oracle-OmegaOS-2` -> key `OmegaOS-2`). Callers legitimately
    /// hold the name in either form — `omega done` runs inside an oracle session
    /// and knows only its full `oracle-<name>` session name, while the close-gate
    /// passes the bare project key — so both `read` and `write` normalize through
    /// this one rule. Without it the writer and patrol's reader (which passes the
    /// full `session.name`) disagreed by one `oracle-` prefix and the signal was
    /// silently invisible to whichever side guessed wrong. Project keys never
    /// themselves begin with `oracle-`, so stripping the prefix is unambiguous.
    fn oracle_key(oracle: &str) -> &str {
        oracle.strip_prefix("oracle-").unwrap_or(oracle)
    }

    fn validated_oracle_key(oracle: &str) -> Result<&str> {
        crate::scope::validate_session_identity(oracle)?;
        let key = Self::oracle_key(oracle);
        crate::scope::validate_session_identity(key)?;
        Ok(key)
    }

    pub fn write(&self, state_dir: &Path) -> Result<()> {
        let key = Self::validated_oracle_key(&self.oracle)?;
        let path = state_dir.join(format!("oracle-{}.done.json", key));
        let _lock = crate::scope::lock_private_state_file(state_dir, COMPLETION_SIGNAL_LOCK)?;
        if let Some(existing) = read_bounded_private_json::<Self>(&path)? {
            if Self::validated_oracle_key(&existing.oracle)? != key {
                bail!("existing oracle done signal identity differs from its filename");
            }
        }
        let content = serde_json::to_vec_pretty(self)?;
        if content.len() > MAX_COMPLETION_SIGNAL_BYTES {
            bail!("oracle done signal exceeds the authority size limit");
        }
        crate::config::atomic_write_private(&path, &content)?;
        Ok(())
    }

    pub fn read(state_dir: &Path, oracle: &str) -> Result<Option<Self>> {
        let key = Self::validated_oracle_key(oracle)?;
        let path = state_dir.join(format!("oracle-{}.done.json", key));
        let signal = read_bounded_private_json::<Self>(&path)?;
        if let Some(signal) = &signal {
            if Self::validated_oracle_key(&signal.oracle)? != key {
                bail!("oracle done signal identity differs from requested oracle");
            }
        }
        Ok(signal)
    }

    /// The per-path "already notified" side-marker the done-notify cron writes
    /// next to the signal (`oracle-<key>.done.json.notified`).
    fn notified_path(state_dir: &Path, oracle: &str) -> std::path::PathBuf {
        let key = Self::oracle_key(oracle);
        state_dir.join(format!("oracle-{}.done.json.notified", key))
    }

    /// Clear a STALE signal before (re)launching a session under this name —
    /// the oracle mirror of the worker-side stale-signal clear (c1f0858).
    /// Oracle session names are deterministic per project and the done.json is
    /// otherwise never deleted, so a leftover closeable signal from a PRIOR
    /// mission would make patrol's reap kill the brand-new session within one
    /// tick. Removes the signal AND its `.notified` marker (a surviving marker
    /// would silently suppress the new mission's report).
    ///
    /// An UN-notified signal (no `.notified` marker — the cron marks only on a
    /// confirmed send) is a report the operator never received: it is RETIRED,
    /// not deleted — renamed to `oracle-<key>-prev<ts>.done.json`, which still
    /// matches the notifier's `oracle-*.done.json` glob (delivered once under
    /// its own marker path) but never collides with the new session's signal.
    /// Returns whether a stale signal actually existed.
    pub fn clear_strict(state_dir: &Path, oracle: &str) -> Result<bool> {
        let key = Self::validated_oracle_key(oracle)?;
        let _lock = crate::scope::lock_private_state_file(state_dir, COMPLETION_SIGNAL_LOCK)?;
        // Re-arm the stuck-oracle alert for the recycled name: the cron's
        // once-per-oracle `<state-basename>.stuck-alerted` marker survives
        // the signal otherwise, silencing a genuine future stall under the
        // same name. Cover the legacy double-prefixed basename too (state
        // files were `oracle-oracle-X.state.json` before normalization).
        crate::scope::remove_private_file(
            &state_dir.join(format!("oracle-{}.stuck-alerted", key)),
        )?;
        crate::scope::remove_private_file(
            &state_dir.join(format!("oracle-oracle-{}.stuck-alerted", key)),
        )?;
        let path = state_dir.join(format!("oracle-{}.done.json", key));
        let Some(signal) = read_bounded_private_json::<Self>(&path)? else {
            crate::scope::remove_private_file(&Self::notified_path(state_dir, oracle))?;
            return Ok(false);
        };
        if Self::validated_oracle_key(&signal.oracle)? != key {
            bail!("oracle done signal identity differs from requested clear");
        }
        if crate::config::read_private_optional(&Self::notified_path(state_dir, oracle))?.is_some()
        {
            // Already delivered → safe to drop both.
            crate::scope::remove_private_file(&path)?;
            crate::scope::remove_private_file(&Self::notified_path(state_dir, oracle))?;
        } else {
            // Not yet delivered → retire so the notifier still sends it.
            let retired = state_dir.join(format!(
                "oracle-{}-prev{}.done.json",
                key,
                Utc::now().timestamp_micros()
            ));
            if crate::config::read_private_optional(&retired)?.is_some() {
                bail!("refusing to overwrite existing retired oracle signal");
            }
            std::fs::rename(&path, &retired).with_context(|| {
                format!(
                    "retiring oracle signal {} to {}",
                    path.display(),
                    retired.display()
                )
            })?;
            std::fs::File::open(state_dir)?.sync_all()?;
        }
        Ok(true)
    }

    /// Compatibility wrapper. Authority paths should use `clear_strict` so an
    /// unsafe or corrupt signal cannot be mistaken for "nothing to clear".
    pub fn clear(state_dir: &Path, oracle: &str) -> bool {
        Self::clear_strict(state_dir, oracle).unwrap_or(false)
    }

    /// Invalidate the notified marker after an upgrade rewrites the signal
    /// (Pending+gate_pending → DoneClean). The notifier cron may have already
    /// reported the transient Pending state and written the per-path marker;
    /// without this, the corrected done_clean would NEVER be sent and the
    /// operator's record would permanently say the mission was incomplete.
    pub fn invalidate_notified_strict(state_dir: &Path, oracle: &str) -> Result<()> {
        Self::validated_oracle_key(oracle)?;
        let _lock = crate::scope::lock_private_state_file(state_dir, COMPLETION_SIGNAL_LOCK)?;
        crate::scope::remove_private_file(&Self::notified_path(state_dir, oracle))
    }

    pub fn invalidate_notified(state_dir: &Path, oracle: &str) {
        let _ = Self::invalidate_notified_strict(state_dir, oracle);
    }

    pub fn is_closeable(&self) -> bool {
        self.status == DoneStatus::DoneClean && self.pending_actions.is_empty()
    }

    /// Every CURRENT oracle done-signal in the state dir. Retired signals
    /// (`oracle-<key>-prev<ts>.done.json`, parked by `clear` until the
    /// notifier delivers them) describe a SUPERSEDED mission and are skipped —
    /// acting on one (e.g. the orphan-worker sweep) would judge live workers
    /// against a previous mission's outcome.
    pub fn read_all(state_dir: &Path) -> Vec<Self> {
        let mut out = Vec::new();
        let Ok(entries) = std::fs::read_dir(state_dir) else {
            return out;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let Some(key) = name
                .strip_prefix("oracle-")
                .and_then(|r| r.strip_suffix(".done.json"))
            else {
                continue;
            };
            let retired = key
                .rsplit_once("-prev")
                .is_some_and(|(_, ts)| !ts.is_empty() && ts.chars().all(|c| c.is_ascii_digit()));
            if retired {
                continue;
            }
            if let Ok(Some(sig)) = Self::read(state_dir, key) {
                out.push(sig);
            }
        }
        out
    }
}

#[cfg(test)]
mod oracle_done_tests {
    use super::*;

    #[test]
    fn oracle_done_signal_prefix_normalized_read_write() {
        // The writer (`omega done` inside an oracle session, which knows only
        // its full `oracle-<name>` session name) and patrol's reader (which
        // also passes the full `session.name`) must agree on the filename. Both
        // normalize through oracle_key, so a bare-key write resolves on a
        // full-name read and vice-versa — no `oracle-oracle-` double prefix.
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();

        let sig = OracleDoneSignal::new("OmegaOS", "OmegaOS", DoneStatus::DoneClean, "mission");
        sig.write(dir).unwrap();

        let f = dir.join("oracle-OmegaOS.done.json");
        assert!(f.exists(), "expected {:?} to exist", f);
        assert!(
            !dir.join("oracle-oracle-OmegaOS.done.json").exists(),
            "double-prefixed file must NOT be created"
        );

        // Full session name (what patrol passes) resolves the bare-keyed file.
        let via_full = OracleDoneSignal::read(dir, "oracle-OmegaOS").unwrap();
        assert!(
            via_full.is_some(),
            "patrol's full-name read must find the signal"
        );
        assert!(via_full.unwrap().is_closeable());

        // Bare key (what the close-gate passes) resolves the same file.
        assert!(OracleDoneSignal::read(dir, "OmegaOS").unwrap().is_some());
    }

    #[test]
    fn oracle_done_signal_retains_index() {
        // Only the single `oracle-` prefix is stripped; a numeric index stays.
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        let sig = OracleDoneSignal::new("OmegaOS-2", "OmegaOS", DoneStatus::DoneClean, "mission");
        sig.write(dir).unwrap();
        assert!(dir.join("oracle-OmegaOS-2.done.json").exists());
        assert!(OracleDoneSignal::read(dir, "oracle-OmegaOS-2")
            .unwrap()
            .is_some());
    }

    #[test]
    fn clear_removes_stale_signal_and_notified_marker() {
        // dispatch_oracle / resurrect_oracle call this before launching a
        // session: BOTH the stale done.json and its .notified marker must go,
        // for either key form (bare key at dispatch, full session name at
        // resurrect).
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        let sig = OracleDoneSignal::new("OmegaOS", "OmegaOS", DoneStatus::DoneClean, "mission");
        sig.write(dir).unwrap();
        std::fs::write(dir.join("oracle-OmegaOS.done.json.notified"), "").unwrap();

        // Notified signal + full session name (resurrect path): both removed.
        assert!(OracleDoneSignal::clear(dir, "oracle-OmegaOS"));
        assert!(!dir.join("oracle-OmegaOS.done.json").exists());
        assert!(!dir.join("oracle-OmegaOS.done.json.notified").exists());

        // No stale signal → reports false, still a no-op success.
        assert!(!OracleDoneSignal::clear(dir, "oracle-OmegaOS"));

        // Bare key (dispatch path) clears too.
        sig.write(dir).unwrap();
        std::fs::write(dir.join("oracle-OmegaOS.done.json.notified"), "").unwrap();
        assert!(OracleDoneSignal::clear(dir, "OmegaOS"));
        assert!(!dir.join("oracle-OmegaOS.done.json").exists());
    }

    #[test]
    fn clear_retires_unnotified_signal_instead_of_deleting() {
        // An UN-notified signal (no .notified marker) is a report the operator
        // never received: clear must RETIRE it (rename, still matching the
        // notifier's oracle-*.done.json glob) rather than destroy it.
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        let sig = OracleDoneSignal::new("OmegaOS", "OmegaOS", DoneStatus::DoneClean, "mission");
        sig.write(dir).unwrap();

        assert!(OracleDoneSignal::clear(dir, "oracle-OmegaOS"));
        // The canonical path is free for the new session's signal…
        assert!(!dir.join("oracle-OmegaOS.done.json").exists());
        // …and the report survived under a retired name the notifier will scan.
        let retired: Vec<_> = std::fs::read_dir(dir)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|n| n.starts_with("oracle-OmegaOS-prev") && n.ends_with(".done.json"))
            .collect();
        assert_eq!(
            retired.len(),
            1,
            "expected exactly one retired signal, got {:?}",
            retired
        );
    }

    #[test]
    fn invalidate_notified_removes_only_the_marker() {
        // The gate-pending upgrade rewrites the signal in place; the marker
        // must be invalidated so the corrected done_clean is re-notified,
        // while the signal itself stays on disk for the notifier to read.
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        let sig = OracleDoneSignal::new("OmegaOS", "OmegaOS", DoneStatus::DoneClean, "mission");
        sig.write(dir).unwrap();
        std::fs::write(dir.join("oracle-OmegaOS.done.json.notified"), "").unwrap();

        OracleDoneSignal::invalidate_notified(dir, "oracle-OmegaOS");
        assert!(!dir.join("oracle-OmegaOS.done.json.notified").exists());
        assert!(dir.join("oracle-OmegaOS.done.json").exists());

        // Idempotent when no marker exists.
        OracleDoneSignal::invalidate_notified(dir, "OmegaOS");
    }
}

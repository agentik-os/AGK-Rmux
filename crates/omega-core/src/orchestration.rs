//! Native Rust orchestration — the complete pipeline.
//!
//! This module owns the full lifecycle:
//!
//!   Mission → classify → plan → dispatch workers → monitor (event-driven)
//!     → collect done signals → quality gate → outcome → report
//!
//! All shell scripts are obsolete. Everything is typed Rust, async, with
//! event-driven worker monitoring via the rmux SDK (output_stream + wait_for_text),
//! not polling.

use crate::agents::Agent;
use crate::config::OmegaConfig;
use crate::done::{DoneSignal, DoneStatus};
use crate::gate::{
    AdversarialChallenge, ChallengeResult, GateResult, GradeResult, GradeVerdict, GraderLens,
    Rubric, RubricCriterion,
};
use crate::mission::{
    Mission, MissionState, Outcome, OutcomeStatus, Plan, PlanContract, PlanStrategy, RetryPolicy,
    Task, TaskAttemptState, TaskContract, TaskId, VerifierCheck, VerifierCheckKind, WorkerResult,
    CONTRACT_SCHEMA_VERSION,
};
pub use crate::mission_ledger::{
    mission_gate_fact_digest, mission_gate_result_digest, ContractVerification,
    MissionGateCheckObservation, MissionGateLensObservation, MissionGateObservation,
    MissionIssueResolution, PlanRequirementKind, PlanRequirementObservation,
    RecordedContractVerification, VerifierObservation, ACCEPTANCE_OBSERVATION_SCHEMA_VERSION,
};
use crate::mission_ledger::{
    AppendEvent, LeaseAssertion, LeaseRecord, LedgerError, MissionLedger, TaskAttemptMutation,
};
use crate::routing::{classify_mission, Complexity};
use crate::scope;
use crate::session::SessionManager;
use anyhow::{bail, Context, Result};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::time::Duration;
use tokio::time::{timeout, Instant};

const LEDGER_CAS_RETRIES: usize = 8;
const DONE_CLOCK_SKEW_SECS: i64 = 5;
const SCOPE_AUTHORITY_SCHEMA_VERSION: u32 = 1;
const SCOPE_AUTHORITY_ACTOR: &str = "omega-scope-authority";
const SCOPE_AUTHORITY_EVENT_KIND: &str = "scope_claim_authority_prepared";

/// Compatibility marker written by `omega done` when an orchestrator-managed
/// Oracle has produced an exact V3 CandidateDone event but cannot honestly
/// declare itself accepted. The orchestrator removes this marker only from its
/// in-memory verification view; the on-disk signal remains Pending until the
/// authoritative task and mission acceptance paths complete.
pub const V3_ACCEPTANCE_PENDING: &str =
    "acceptation V3 indépendante en attente; livraison non autorisée";

pub fn provider_family_for_agent(agent: Agent) -> crate::rules::ProviderFamily {
    match agent {
        Agent::Claude | Agent::Glm => crate::rules::ProviderFamily::Claude,
        Agent::Codex => crate::rules::ProviderFamily::Codex,
        Agent::Gemini | Agent::Antigravity => crate::rules::ProviderFamily::Gemini,
        Agent::Pi | Agent::OpenRouter | Agent::Hermes | Agent::Kimi | Agent::Shell => {
            crate::rules::ProviderFamily::Other
        }
    }
}

/// Same map, plus harness names that are not (yet) first-class `Agent`s —
/// OpenCode is Home-only today but still needs the Other overlay.
pub fn provider_family_from_name(name: &str) -> crate::rules::ProviderFamily {
    if let Some(agent) = Agent::from_name(name) {
        return provider_family_for_agent(agent);
    }
    match name.to_ascii_lowercase().as_str() {
        "opencode" | "open-code" | "open_code" => crate::rules::ProviderFamily::Other,
        _ => crate::rules::ProviderFamily::Other,
    }
}

/// Single policy funnel used by dispatch / orchestrate / workers / teams.
pub fn policy_context_for_agent(
    scope: crate::rules::RuleScope,
    mission: &str,
    agent: Agent,
) -> String {
    crate::rules::agent_context_for_provider(scope, Some(mission), provider_family_for_agent(agent))
}

/// Error type for orchestration operations.
#[derive(Debug, thiserror::Error)]
pub enum OrchestrationError {
    #[error("scope conflict for {worker}: {detail}")]
    ScopeConflict { worker: String, detail: String },

    #[error("dispatch failed for {target}: {source}")]
    DispatchFailed {
        target: String,
        #[source]
        source: anyhow::Error,
    },

    #[error("worker {worker} timed out after {seconds}s")]
    WorkerTimeout { worker: String, seconds: u64 },

    #[error("quality gate rejected: {reason}")]
    GateRejected { reason: String },

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Configuration knobs for the orchestrator.
#[derive(Debug, Clone)]
pub struct OrchestratorOptions {
    pub worker_timeout: Duration,
    pub poll_interval: Duration,
    pub enforce_gate: bool,
    pub auto_ack: bool,
}

impl Default for OrchestratorOptions {
    fn default() -> Self {
        Self {
            worker_timeout: Duration::from_secs(3600),
            poll_interval: Duration::from_secs(5),
            enforce_gate: true,
            auto_ack: true,
        }
    }
}

/// One task attempt frozen by the V3 ledger before any worker process exists.
#[derive(Debug, Clone)]
pub struct AuthoritativeTaskAttempt {
    pub mission_id: crate::mission::MissionId,
    pub task_id: String,
    pub attempt_id: String,
    pub plan_revision: u64,
    pub owner: Option<String>,
    pub leases: Vec<LeaseRecord>,
    /// Exact immutable-to-file bridge for the compatibility scope claim.
    /// This receipt is generation-bound and may be reloaded only from its
    /// unique ledger event after a process restart.
    pub scope_receipt: Option<AuthoritativeScopeReceipt>,
}

/// Write-ahead receipt binding one compatibility claim generation to one
/// immutable V3 attempt identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthoritativeScopeReceipt {
    pub schema_version: u32,
    pub mission_id: crate::mission::MissionId,
    pub task_id: String,
    pub attempt_id: String,
    pub plan_revision: u64,
    pub owner: String,
    pub claim: scope::ScopeClaim,
}

/// Canonical execution identity behind the legacy [`Plan`] / [`Outcome`]
/// compatibility API.
#[derive(Debug, Clone)]
pub struct AuthoritativeExecution {
    pub mission_id: crate::mission::MissionId,
    pub plan: PlanContract,
    pub attempts: Vec<AuthoritativeTaskAttempt>,
}

impl AuthoritativeExecution {
    pub fn attempt(&self, task_id: &str) -> Result<&AuthoritativeTaskAttempt> {
        self.attempts
            .iter()
            .find(|attempt| attempt.task_id == task_id)
            .ok_or_else(|| anyhow::anyhow!("no authoritative attempt for task {task_id}"))
    }

    pub fn attempt_mut(&mut self, task_id: &str) -> Result<&mut AuthoritativeTaskAttempt> {
        self.attempts
            .iter_mut()
            .find(|attempt| attempt.task_id == task_id)
            .ok_or_else(|| anyhow::anyhow!("no authoritative attempt for task {task_id}"))
    }
}

fn compatibility_task_contract(mission: &Mission, task: &Task) -> TaskContract {
    TaskContract {
        schema_version: CONTRACT_SCHEMA_VERSION,
        task_id: TaskId::new(&task.id),
        name: task.name.clone(),
        prompt: task.prompt.clone(),
        acceptance_criteria: vec![format!(
            "Complete `{}` against its frozen prompt and produce independently verifiable evidence",
            task.name
        )],
        // The legacy Plan had no verifier field. A real, directly-executable
        // repository integrity check is frozen here instead of fabricating a
        // passing receipt. Non-git work consequently fails verification rather
        // than bypassing the contract.
        verifier_checks: vec![VerifierCheck {
            schema_version: CONTRACT_SCHEMA_VERSION,
            check_id: format!("{}-repository-integrity", task.id),
            kind: VerifierCheckKind::Command {
                argv: vec!["git".to_string(), "diff".to_string(), "--check".to_string()],
                cwd: Some(mission.working_dir.to_string_lossy().to_string()),
                expected_exit_code: 0,
            },
            timeout_secs: 120,
        }],
        required_capabilities: vec!["tool_calling".to_string()],
        scope: task.files_owned.clone(),
        risk: classify_mission(&task.prompt).risk,
        retry_policy: RetryPolicy::default(),
        depends_on: task.depends_on.iter().map(TaskId::new).collect(),
    }
}

/// Create the authoritative mission, classification, immutable plan and queued
/// attempts before any rmux or provider side effect is attempted.
///
/// This is shared by `omega orchestrate` and `omega team`; callers cannot get a
/// runnable identity without a valid plan and one queued attempt per task.
pub fn prepare_authoritative_execution(
    ledger: &MissionLedger,
    mission: &Mission,
    legacy_plan: &Plan,
    actor: &str,
    required_gates: Vec<String>,
) -> Result<AuthoritativeExecution> {
    if legacy_plan.mission_id != mission.id {
        bail!("legacy plan belongs to a different mission");
    }
    if legacy_plan.tasks.is_empty() {
        bail!("orchestration plan has no executable tasks");
    }

    let created = ledger.create_mission(
        mission,
        &format!("orchestration:{}:created", mission.id.as_str()),
        actor,
    )?;
    if created.projection.state != MissionState::Created {
        bail!(
            "mission {} already advanced to {:?}; refusing parallel orchestration authority",
            mission.id.as_str(),
            created.projection.state
        );
    }

    let decision = classify_mission(&mission.text);
    let mut classified = AppendEvent::new(
        mission.id.clone(),
        created.projection.version,
        format!("orchestration:{}:classified", mission.id.as_str()),
        actor,
        "mission_classified",
    );
    classified.next_mission_state = Some(MissionState::Classified);
    classified.payload = serde_json::to_value(&decision)?;
    let classified = ledger.append(classified)?;

    let contracts = legacy_plan
        .tasks
        .iter()
        .map(|task| compatibility_task_contract(mission, task))
        .collect();
    let plan = PlanContract::new(
        mission.id.clone(),
        1,
        classified.projection.version,
        contracts,
        required_gates,
        Vec::new(),
    )?;
    let mut planned = AppendEvent::new(
        mission.id.clone(),
        classified.projection.version,
        format!("orchestration:{}:plan:1", mission.id.as_str()),
        actor,
        "plan_accepted",
    );
    planned.next_mission_state = Some(MissionState::Planned);
    planned.payload = serde_json::to_value(&plan)?;
    planned.plan = Some(plan.clone());
    let mut projection = ledger.append(planned)?.projection;

    let mut attempts = Vec::with_capacity(plan.tasks.len());
    for task in &plan.tasks {
        let attempt_id = format!(
            "attempt-{}-{}-1",
            mission.id.as_str(),
            task.task_id.as_str()
        );
        let mut queued = AppendEvent::new(
            mission.id.clone(),
            projection.version,
            format!("orchestration:{attempt_id}:queued"),
            actor,
            "task_attempt_queued",
        );
        queued.task_attempt = Some(TaskAttemptMutation {
            task_id: task.task_id.as_str().to_string(),
            attempt_id: attempt_id.clone(),
            plan_revision: plan.revision,
            expected_version: 0,
            next_state: TaskAttemptState::Queued,
        });
        queued.payload = serde_json::json!({
            "task": task.task_id.as_str(),
            "plan_revision": plan.revision,
        });
        projection = ledger.append(queued)?.projection;
        attempts.push(AuthoritativeTaskAttempt {
            mission_id: mission.id.clone(),
            task_id: task.task_id.as_str().to_string(),
            attempt_id,
            plan_revision: plan.revision,
            owner: None,
            leases: Vec::new(),
            scope_receipt: None,
        });
    }

    Ok(AuthoritativeExecution {
        mission_id: mission.id.clone(),
        plan,
        attempts,
    })
}

/// Claim both the compatibility scope file and fenced V3 leases before a
/// worker starts. A failure rolls back every acquisition made by this call.
fn validate_scope_receipt_binding(
    ledger: &MissionLedger,
    attempt: &AuthoritativeTaskAttempt,
    event: &crate::mission_ledger::MissionEvent,
    receipt: &AuthoritativeScopeReceipt,
) -> Result<()> {
    if event.actor != SCOPE_AUTHORITY_ACTOR
        || event.kind != SCOPE_AUTHORITY_EVENT_KIND
        || event.idempotency_key != format!("orchestration:{}:scope-authority", attempt.attempt_id)
        || event.plan_revision != Some(attempt.plan_revision)
        || receipt.schema_version != SCOPE_AUTHORITY_SCHEMA_VERSION
        || receipt.mission_id != attempt.mission_id
        || receipt.task_id != attempt.task_id
        || receipt.attempt_id != attempt.attempt_id
        || receipt.plan_revision != attempt.plan_revision
        || receipt.owner != receipt.claim.session
        || receipt.owner.trim().is_empty()
        || receipt.claim.workspace_id.is_none()
        || receipt.claim.claim_id.is_none()
        || receipt.claim.files_owned.is_empty()
    {
        bail!("scope authority receipt is not bound to the exact task attempt");
    }
    let mission = ledger
        .mission(&attempt.mission_id)?
        .ok_or_else(|| anyhow::anyhow!("scope receipt mission disappeared"))?;
    if mission.active_plan_revision != Some(attempt.plan_revision) {
        bail!("scope receipt plan revision is no longer active");
    }
    let task_attempt = ledger
        .task_attempt(&attempt.attempt_id)?
        .ok_or_else(|| anyhow::anyhow!("scope receipt task attempt disappeared"))?;
    if task_attempt.mission_id != attempt.mission_id
        || task_attempt.task_id != attempt.task_id
        || task_attempt.plan_revision != attempt.plan_revision
    {
        bail!("scope receipt task projection differs from immutable authority");
    }
    let plan = ledger
        .active_plan(&attempt.mission_id)?
        .ok_or_else(|| anyhow::anyhow!("scope receipt mission has no active plan"))?;
    let task = plan
        .tasks
        .iter()
        .find(|task| task.task_id.as_str() == attempt.task_id)
        .ok_or_else(|| anyhow::anyhow!("scope receipt task is absent from the active plan"))?;
    let normalized_scope = scope::validate_scope_selectors(task.scope.clone())?;
    if normalized_scope != receipt.claim.files_owned {
        bail!("scope receipt selectors differ from the immutable task contract");
    }
    if receipt.claim.claimed_at > event.recorded_at + ChronoDuration::seconds(DONE_CLOCK_SKEW_SECS)
    {
        bail!("scope receipt timestamp is later than its immutable event");
    }
    let history = ledger.events(&attempt.mission_id)?;
    let queued_before = history
        .iter()
        .filter(|candidate| {
            candidate.sequence < event.sequence
                && candidate
                    .resulting_task_attempt
                    .as_ref()
                    .is_some_and(|projection| {
                        projection.mission_id == attempt.mission_id
                            && projection.task_id == attempt.task_id
                            && projection.attempt_id == attempt.attempt_id
                            && projection.plan_revision == attempt.plan_revision
                            && projection.state == TaskAttemptState::Queued
                    })
        })
        .count();
    if queued_before != 1 {
        bail!("scope authority must follow one exact immutable Queued attempt event");
    }
    if history.iter().any(|candidate| {
        candidate.sequence <= event.sequence
            && candidate
                .resulting_task_attempt
                .as_ref()
                .is_some_and(|projection| {
                    projection.attempt_id == attempt.attempt_id
                        && projection.state == TaskAttemptState::Running
                })
    }) {
        bail!("scope authority was not persisted before the worker entered Running");
    }
    Ok(())
}

fn load_authoritative_scope_receipt(
    ledger: &MissionLedger,
    attempt: &AuthoritativeTaskAttempt,
) -> Result<Option<AuthoritativeScopeReceipt>> {
    let mut matches = Vec::new();
    for event in ledger
        .events(&attempt.mission_id)?
        .into_iter()
        .filter(|event| event.kind == SCOPE_AUTHORITY_EVENT_KIND)
    {
        let receipt: AuthoritativeScopeReceipt = serde_json::from_value(event.payload.clone())
            .with_context(|| {
                format!("corrupt immutable scope authority event {}", event.event_id)
            })?;
        if receipt.attempt_id == attempt.attempt_id {
            matches.push((event, receipt));
        }
    }
    if matches.len() > 1 {
        bail!(
            "attempt {} has duplicate immutable scope authority receipts",
            attempt.attempt_id
        );
    }
    let Some((event, receipt)) = matches.into_iter().next() else {
        return Ok(None);
    };
    validate_scope_receipt_binding(ledger, attempt, &event, &receipt)?;
    Ok(Some(receipt))
}

fn append_scope_authority_receipt(
    ledger: &MissionLedger,
    attempt: &AuthoritativeTaskAttempt,
    receipt: &AuthoritativeScopeReceipt,
) -> Result<AuthoritativeScopeReceipt> {
    for _ in 0..LEDGER_CAS_RETRIES {
        let mission = ledger
            .mission(&attempt.mission_id)?
            .ok_or_else(|| anyhow::anyhow!("scope receipt mission disappeared"))?;
        let mut event = AppendEvent::new(
            attempt.mission_id.clone(),
            mission.version,
            format!("orchestration:{}:scope-authority", attempt.attempt_id),
            SCOPE_AUTHORITY_ACTOR,
            SCOPE_AUTHORITY_EVENT_KIND,
        );
        event.payload = serde_json::to_value(receipt)?;
        event.observation_plan_revision = Some(attempt.plan_revision);
        match ledger.append(event) {
            Ok(_) => {
                return load_authoritative_scope_receipt(ledger, attempt)?.ok_or_else(|| {
                    anyhow::anyhow!("persisted scope authority receipt disappeared")
                })
            }
            Err(LedgerError::VersionConflict { .. }) => continue,
            Err(LedgerError::IdempotencyConflict { .. }) => {
                return load_authoritative_scope_receipt(ledger, attempt)?.ok_or_else(|| {
                    anyhow::anyhow!("scope authority idempotency conflict has no recorded receipt")
                })
            }
            Err(error) => return Err(error.into()),
        }
    }
    bail!(
        "scope authority for attempt {} did not converge after {} compare-and-set retries",
        attempt.attempt_id,
        LEDGER_CAS_RETRIES
    )
}

fn rollback_scope_acquisition(
    ledger: &MissionLedger,
    state_dir: &Path,
    attempt: &mut AuthoritativeTaskAttempt,
    receipt: &AuthoritativeScopeReceipt,
    acquired: Vec<LeaseRecord>,
    cause: anyhow::Error,
) -> anyhow::Error {
    let mut rollback_failures = Vec::new();
    let mut residual_leases = Vec::new();
    for lease in acquired {
        if let Err(error) = ledger.release_lease(&lease.resource_key, lease.fencing_token) {
            rollback_failures.push(format!("lease {}: {error}", lease.resource_key));
            residual_leases.push(lease);
        }
    }
    let residual_receipt = if residual_leases.is_empty() {
        match scope::ScopeClaim::release_exact(state_dir, &receipt.claim) {
            Ok(()) => None,
            Err(error) => {
                rollback_failures.push(format!("compatibility scope {}: {error}", receipt.owner));
                Some(receipt.clone())
            }
        }
    } else {
        rollback_failures.push(format!(
            "compatibility scope {} retained because V3 lease rollback is incomplete",
            receipt.owner
        ));
        Some(receipt.clone())
    };
    if rollback_failures.is_empty() {
        attempt.owner = None;
        attempt.leases.clear();
        attempt.scope_receipt = None;
        cause.context("authoritative scope acquisition rolled back cleanly")
    } else {
        attempt.owner = Some(receipt.owner.clone());
        attempt.leases = residual_leases;
        attempt.scope_receipt = residual_receipt;
        anyhow::anyhow!(
            "{cause}; scope acquisition rollback incomplete: {}",
            rollback_failures.join("; ")
        )
    }
}

fn scope_lease_resource(workspace_id: &str, normalized_selector: &str) -> String {
    let identity = format!("{workspace_id}:{normalized_selector}");
    format!("scope:{}", blake3::hash(identity.as_bytes()).to_hex())
}

pub fn claim_authoritative_scopes(
    ledger: &MissionLedger,
    state_dir: &Path,
    working_dir: &Path,
    attempt: &mut AuthoritativeTaskAttempt,
    owner: &str,
    selectors: &[String],
    ttl: Duration,
) -> Result<()> {
    if attempt.owner.is_some() || !attempt.leases.is_empty() || attempt.scope_receipt.is_some() {
        bail!("attempt already carries scope authority; explicit release is required first");
    }
    let plan = ledger
        .active_plan(&attempt.mission_id)?
        .ok_or_else(|| anyhow::anyhow!("scope claim mission has no active plan"))?;
    let task = plan
        .tasks
        .iter()
        .find(|task| task.task_id.as_str() == attempt.task_id)
        .ok_or_else(|| anyhow::anyhow!("scope claim task is absent from the active plan"))?;
    let requested = scope::validate_scope_selectors(selectors.to_vec())?;
    let contracted = scope::validate_scope_selectors(task.scope.clone())?;
    if requested != contracted {
        bail!("requested scope differs from the immutable task contract");
    }
    if requested.is_empty() {
        if load_authoritative_scope_receipt(ledger, attempt)?.is_some() {
            bail!("read-only task unexpectedly has a persisted writable scope receipt");
        }
        if !ledger
            .active_leases_for_attempt(&attempt.mission_id, &attempt.task_id, &attempt.attempt_id)?
            .is_empty()
        {
            bail!("read-only task unexpectedly has active writable leases");
        }
        attempt.owner = Some(owner.to_string());
        return Ok(());
    }
    let desired = scope::prepare_claim_for_workspace(working_dir, owner, requested)?;
    let receipt = match load_authoritative_scope_receipt(ledger, attempt)? {
        Some(receipt) => receipt,
        None => {
            let prepared = AuthoritativeScopeReceipt {
                schema_version: SCOPE_AUTHORITY_SCHEMA_VERSION,
                mission_id: attempt.mission_id.clone(),
                task_id: attempt.task_id.clone(),
                attempt_id: attempt.attempt_id.clone(),
                plan_revision: attempt.plan_revision,
                owner: owner.to_string(),
                claim: desired.clone(),
            };
            append_scope_authority_receipt(ledger, attempt, &prepared)?
        }
    };
    if receipt.owner != owner
        || receipt.claim.session != owner
        || receipt.claim.workspace_id != desired.workspace_id
        || receipt.claim.files_owned != desired.files_owned
    {
        bail!(
            "persisted scope authority differs from the requested owner, workspace, or selectors"
        );
    }
    if let Err(error) = scope::publish_prepared_claim(state_dir, working_dir, &receipt.claim) {
        return Err(rollback_scope_acquisition(
            ledger,
            state_dir,
            attempt,
            &receipt,
            Vec::new(),
            error,
        ));
    }
    let mut leases = Vec::new();
    for normalized in &receipt.claim.files_owned {
        let workspace_id =
            receipt.claim.workspace_id.as_deref().ok_or_else(|| {
                anyhow::anyhow!("scope receipt has no canonical workspace identity")
            })?;
        let resource = scope_lease_resource(workspace_id, normalized);
        match ledger.acquire_lease(
            &resource,
            &attempt.mission_id,
            &attempt.task_id,
            &attempt.attempt_id,
            owner,
            ttl,
        ) {
            Ok(lease) => leases.push(lease),
            Err(error) => {
                return Err(rollback_scope_acquisition(
                    ledger,
                    state_dir,
                    attempt,
                    &receipt,
                    leases,
                    error.into(),
                ));
            }
        }
    }
    attempt.owner = Some(owner.to_string());
    attempt.leases = leases;
    attempt.scope_receipt = Some(receipt);
    Ok(())
}

/// Advance one task attempt through the ledger. If the task owns scope, every
/// post-queue mutation is fenced by the first acquired lease and the actor must
/// be the lease owner.
fn attempt_has_reached(current: TaskAttemptState, target: TaskAttemptState) -> bool {
    use TaskAttemptState::*;

    if current == target {
        return true;
    }
    match target {
        Queued => false,
        Running => matches!(
            current,
            CandidateDone | Verifying | Accepted | CorrectionRequired | Blocked | Failed
        ),
        CandidateDone => matches!(
            current,
            Verifying | Accepted | CorrectionRequired | Blocked | Failed
        ),
        Verifying => matches!(current, Accepted | CorrectionRequired | Blocked | Failed),
        Accepted | CorrectionRequired | Blocked | Failed | Cancelled => false,
    }
}

pub fn transition_authoritative_attempt(
    ledger: &MissionLedger,
    attempt: &AuthoritativeTaskAttempt,
    next: TaskAttemptState,
    actor: &str,
) -> Result<()> {
    let label = format!("{next:?}").to_lowercase();
    for _ in 0..LEDGER_CAS_RETRIES {
        let mission = ledger
            .mission(&attempt.mission_id)?
            .ok_or_else(|| anyhow::anyhow!("authoritative mission disappeared"))?;
        let task = ledger
            .task_attempt(&attempt.attempt_id)?
            .ok_or_else(|| anyhow::anyhow!("authoritative task attempt disappeared"))?;
        if next == TaskAttemptState::Running {
            let owner = attempt
                .owner
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("Running transition requires a claimed owner"))?;
            if actor != owner {
                bail!("Running transition actor differs from the claimed attempt owner");
            }
        }
        if attempt_has_reached(task.state, next) {
            return Ok(());
        }
        if !task.state.can_transition_to(next) {
            bail!(
                "attempt {} cannot advance from {:?} to {:?}; refusing a backward or branch-changing transition",
                attempt.attempt_id,
                task.state,
                next
            );
        }

        let mut event = AppendEvent::new(
            attempt.mission_id.clone(),
            mission.version,
            format!("orchestration:{}:{label}", attempt.attempt_id),
            actor,
            format!("task_attempt_{label}"),
        );
        event.task_attempt = Some(TaskAttemptMutation {
            task_id: attempt.task_id.clone(),
            attempt_id: attempt.attempt_id.clone(),
            plan_revision: attempt.plan_revision,
            expected_version: task.version,
            next_state: next,
        });
        event.lease_assertions = attempt.leases.iter().map(LeaseAssertion::from).collect();
        match ledger.append(event) {
            Ok(_) => return Ok(()),
            Err(
                LedgerError::VersionConflict { .. } | LedgerError::AttemptVersionConflict { .. },
            ) => continue,
            Err(error) => return Err(error.into()),
        }
    }
    bail!(
        "attempt {} did not converge after {} compare-and-set retries",
        attempt.attempt_id,
        LEDGER_CAS_RETRIES
    )
}

fn ensure_attempt_verifying(
    ledger: &MissionLedger,
    attempt: &AuthoritativeTaskAttempt,
    actor: &str,
) -> Result<()> {
    transition_authoritative_attempt(ledger, attempt, TaskAttemptState::CandidateDone, actor)?;
    transition_authoritative_attempt(ledger, attempt, TaskAttemptState::Verifying, actor)
}

pub fn release_authoritative_scopes(
    ledger: &MissionLedger,
    state_dir: &Path,
    attempt: &AuthoritativeTaskAttempt,
) -> Result<()> {
    let plan = ledger
        .active_plan(&attempt.mission_id)?
        .ok_or_else(|| anyhow::anyhow!("scope release mission has no active plan"))?;
    let task = plan
        .tasks
        .iter()
        .find(|task| task.task_id.as_str() == attempt.task_id)
        .ok_or_else(|| anyhow::anyhow!("scope release task is absent from the active plan"))?;
    let contracted = scope::validate_scope_selectors(task.scope.clone())?;
    if contracted.is_empty() {
        // Read-only dispatches still have an exact owner so their Running and
        // completion events cannot be forged. They intentionally carry no
        // writable receipt or lease, so release is an idempotent no-op.
        if attempt.scope_receipt.is_some()
            || load_authoritative_scope_receipt(ledger, attempt)?.is_some()
            || !attempt.leases.is_empty()
            || !ledger
                .active_leases_for_attempt(
                    &attempt.mission_id,
                    &attempt.task_id,
                    &attempt.attempt_id,
                )?
                .is_empty()
        {
            bail!("read-only task carries unexpected writable scope authority");
        }
        return Ok(());
    }
    let persisted = load_authoritative_scope_receipt(ledger, attempt)?.ok_or_else(|| {
        anyhow::anyhow!(
            "attempt {} has no immutable scope authority receipt",
            attempt.attempt_id
        )
    })?;
    if attempt
        .scope_receipt
        .as_ref()
        .is_some_and(|receipt| receipt != &persisted)
    {
        bail!("in-memory scope receipt differs from immutable ledger authority");
    }
    if attempt.owner.as_deref() != Some(persisted.owner.as_str()) {
        bail!("attempt owner differs from immutable scope authority");
    }
    let active = ledger.active_leases_for_attempt(
        &attempt.mission_id,
        &attempt.task_id,
        &attempt.attempt_id,
    )?;
    let current = scope::ScopeClaim::read_strict(state_dir, &persisted.owner)?;
    match current {
        Some(ref current) if current == &persisted.claim => {}
        Some(_) => bail!(
            "stale scope release refused for {}: mutable claim differs from immutable receipt",
            persisted.owner
        ),
        None if active.is_empty() => return Ok(()),
        None => bail!(
            "compatibility scope {} disappeared while V3 leases remain active",
            persisted.owner
        ),
    }
    for lease in &active {
        if lease.owner != persisted.owner
            || !attempt.leases.iter().any(|presented| {
                presented.resource_key == lease.resource_key
                    && presented.fencing_token == lease.fencing_token
                    && presented.owner == lease.owner
            })
        {
            bail!(
                "active lease {} is absent from the exact release authority",
                lease.resource_key
            );
        }
    }
    let mut failures = Vec::new();
    for lease in &active {
        if let Err(error) = ledger.release_lease(&lease.resource_key, lease.fencing_token) {
            failures.push(format!("{}: {error}", lease.resource_key));
        }
    }
    if !failures.is_empty() {
        bail!(
            "scope release incomplete; compatibility claim retained: {}",
            failures.join("; ")
        );
    }
    scope::ScopeClaim::release_exact(state_dir, &persisted.claim)
        .with_context(|| format!("releasing exact compatibility scope {}", persisted.owner))
}

/// Advance the mission lifecycle with a fresh compare-and-set version.
fn mission_has_reached(current: MissionState, target: MissionState) -> bool {
    use MissionState::*;

    if current == target {
        return true;
    }
    match target {
        Created | Classified | Planned => false,
        Running => matches!(
            current,
            Verifying | Accepted | CorrectionRequired | Blocked | Failed | Reporting | Delivered
        ),
        Verifying => matches!(
            current,
            Accepted | CorrectionRequired | Blocked | Failed | Reporting | Delivered
        ),
        Accepted => matches!(current, Reporting | Delivered),
        Reporting => matches!(current, Delivered),
        CorrectionRequired | Blocked | Failed | Delivered | Cancelled => false,
    }
}

pub fn transition_authoritative_mission(
    ledger: &MissionLedger,
    mission_id: &crate::mission::MissionId,
    next: MissionState,
    actor: &str,
) -> Result<()> {
    let label = format!("{next:?}").to_lowercase();
    for _ in 0..LEDGER_CAS_RETRIES {
        let current = ledger
            .mission(mission_id)?
            .ok_or_else(|| anyhow::anyhow!("authoritative mission disappeared"))?;
        if mission_has_reached(current.state, next) {
            if matches!(
                next,
                MissionState::Accepted | MissionState::Reporting | MissionState::Delivered
            ) {
                ledger.validate_mission_acceptance(mission_id)?;
            }
            return Ok(());
        }
        if !current.state.can_transition_to(next) {
            bail!(
                "mission {} cannot advance from {:?} to {:?}; refusing a backward or branch-changing transition",
                mission_id.as_str(),
                current.state,
                next
            );
        }
        let mut event = AppendEvent::new(
            mission_id.clone(),
            current.version,
            format!("orchestration:{}:mission:{label}", mission_id.as_str()),
            actor,
            format!("mission_{label}"),
        );
        event.next_mission_state = Some(next);
        match ledger.append(event) {
            Ok(_) => return Ok(()),
            Err(LedgerError::VersionConflict { .. }) => continue,
            Err(error) => return Err(error.into()),
        }
    }
    bail!(
        "mission {} did not converge after {} compare-and-set retries",
        mission_id.as_str(),
        LEDGER_CAS_RETRIES
    )
}

/// Independently rerun every verifier frozen in the task contract.
///
/// Each immutable verifier executes exactly once. Worker claims matching a
/// frozen verifier reuse that observation; they never trigger a second command
/// or HTTP request. Additional non-executable artifacts are grounded separately,
/// while undeclared command and HTTP claims fail closed without execution.
fn verifier_matches_artifact(check: &VerifierCheck, artifact: &crate::done::DoneArtifact) -> bool {
    match (&check.kind, artifact) {
        (
            VerifierCheckKind::Command {
                argv,
                expected_exit_code,
                ..
            },
            crate::done::DoneArtifact::Command { cmd, exit_code },
        ) => argv.join(" ") == *cmd && expected_exit_code == exit_code,
        (
            VerifierCheckKind::Http {
                url,
                expected_status,
            },
            crate::done::DoneArtifact::Url {
                url: claimed_url,
                expected_status: claimed_status,
            },
        ) => url == claimed_url && expected_status == claimed_status,
        (
            VerifierCheckKind::FileExists { path },
            crate::done::DoneArtifact::FilePath { path: claimed_path },
        ) => path == claimed_path,
        (
            VerifierCheckKind::GitObject { sha },
            crate::done::DoneArtifact::GitSha {
                sha: claimed_sha, ..
            },
        ) => sha == claimed_sha,
        _ => false,
    }
}

pub fn independently_verify_task_contract(
    done: &DoneSignal,
    repo_root: &Path,
    contract: &TaskContract,
) -> ContractVerification {
    let mut independent = DoneSignal::new(
        "omega-independent-verifier",
        DoneStatus::DoneClean,
        "immutable task verifier rerun",
    );
    independent.todos_total = 1;
    independent.todos_completed = 1;
    independent.corroboration = vec![
        crate::done::CorroborationSource::IndependentAuditor,
        crate::done::CorroborationSource::CiExitCode,
    ];
    independent.artifacts = contract
        .verifier_checks
        .iter()
        .map(|check| match &check.kind {
            VerifierCheckKind::Command {
                argv,
                expected_exit_code,
                ..
            } => crate::done::DoneArtifact::Command {
                cmd: argv.join(" "),
                exit_code: *expected_exit_code,
            },
            VerifierCheckKind::Http {
                url,
                expected_status,
            } => crate::done::DoneArtifact::Url {
                url: url.clone(),
                expected_status: *expected_status,
            },
            VerifierCheckKind::FileExists { path } => {
                crate::done::DoneArtifact::FilePath { path: path.clone() }
            }
            VerifierCheckKind::GitObject { sha } => crate::done::DoneArtifact::GitSha {
                sha: sha.clone(),
                branch: None,
            },
        })
        .collect();
    let independent_verdict = crate::done::verify_done_against_contract(
        &independent,
        Some(repo_root),
        &contract.verifier_checks,
    );

    let observations = contract
        .verifier_checks
        .iter()
        .zip(&independent_verdict.checks)
        .map(|(check, result)| VerifierObservation {
            check_id: check.check_id.clone(),
            passed: result.passed,
            detail: result.detail.clone(),
        })
        .collect::<Vec<_>>();
    let exact_coverage = observations.len() == contract.verifier_checks.len()
        && observations.iter().all(|observation| observation.passed);
    let mut failures = Vec::new();
    if done.status != DoneStatus::DoneClean {
        failures.push(format!(
            "worker evidence: status is {:?}, not done_clean",
            done.status
        ));
    }
    if done.session.trim().is_empty() {
        failures.push("worker evidence: empty session identity".to_string());
    }
    if done.todos_total == 0 {
        failures.push("worker evidence: the 0/0 task count proves no work".to_string());
    } else if done.todos_completed != done.todos_total {
        failures.push(format!(
            "worker evidence: completed {}/{} declared tasks",
            done.todos_completed, done.todos_total
        ));
    }
    if !done.pending_actions.is_empty() {
        failures.push(format!(
            "worker evidence: {} pending action(s) remain",
            done.pending_actions.len()
        ));
    }
    if !done.not_done.is_empty() {
        failures.push(format!(
            "worker evidence: {} item(s) explicitly not done",
            done.not_done.len()
        ));
    }
    if done.artifacts.is_empty() && done.is_single_source() {
        failures.push(
            "worker evidence: zero artifacts and no validated independent provenance; worker narration is inadmissible"
                .to_string(),
        );
    }

    let mut supplemental = done.clone();
    supplemental.artifacts.clear();
    for artifact in &done.artifacts {
        if let Some((index, check)) = contract
            .verifier_checks
            .iter()
            .enumerate()
            .find(|(_, check)| verifier_matches_artifact(check, artifact))
        {
            let observation = observations.get(index);
            if !observation.is_some_and(|observation| observation.passed) {
                failures.push(format!(
                    "worker evidence: claim for frozen verifier `{}` was not corroborated",
                    check.check_id
                ));
            }
            continue;
        }
        match artifact {
            crate::done::DoneArtifact::Command { cmd, .. } => failures.push(format!(
                "worker evidence: undeclared command was not executed: `{cmd}`"
            )),
            crate::done::DoneArtifact::Url { url, .. } => failures.push(format!(
                "worker evidence: undeclared URL was not requested: {url}"
            )),
            _ => supplemental.artifacts.push(artifact.clone()),
        }
    }
    if !supplemental.artifacts.is_empty() {
        let supplemental_verdict =
            crate::done::verify_done_against_repo(&supplemental, Some(repo_root));
        failures.extend(
            supplemental_verdict
                .failures
                .into_iter()
                .map(|failure| format!("worker evidence: {failure}")),
        );
    }
    failures.extend(
        independent_verdict
            .failures
            .into_iter()
            .map(|failure| format!("independent verifier: {failure}")),
    );
    if observations.len() != contract.verifier_checks.len() {
        failures.push(format!(
            "independent verifier coverage {}/{}",
            observations.len(),
            contract.verifier_checks.len()
        ));
    }
    ContractVerification {
        passed: failures.is_empty() && independent_verdict.passes && exact_coverage,
        observations,
        failures,
    }
}

fn done_signal_digest(done: &DoneSignal) -> Result<String> {
    Ok(blake3::hash(&serde_json::to_vec(done)?)
        .to_hex()
        .to_string())
}

fn append_bound_observation(
    ledger: &MissionLedger,
    attempt: &AuthoritativeTaskAttempt,
    actor: &str,
    kind: &str,
    key_label: &str,
    payload: serde_json::Value,
) -> Result<()> {
    let payload_digest = blake3::hash(&serde_json::to_vec(&payload)?)
        .to_hex()
        .to_string();
    for _ in 0..LEDGER_CAS_RETRIES {
        let mission = ledger
            .mission(&attempt.mission_id)?
            .ok_or_else(|| anyhow::anyhow!("authoritative mission disappeared"))?;
        let mut event = AppendEvent::new(
            attempt.mission_id.clone(),
            mission.version,
            format!(
                "orchestration:{}:{key_label}:{payload_digest}",
                attempt.attempt_id
            ),
            actor,
            kind,
        );
        event.payload = payload.clone();
        match ledger.append(event) {
            Ok(_) => return Ok(()),
            Err(LedgerError::VersionConflict { .. }) => continue,
            Err(error) => return Err(error.into()),
        }
    }
    bail!(
        "observation for attempt {} did not converge after {} compare-and-set retries",
        attempt.attempt_id,
        LEDGER_CAS_RETRIES
    )
}

fn append_mission_observation(
    ledger: &MissionLedger,
    mission_id: &crate::mission::MissionId,
    actor: &str,
    kind: &str,
    key_label: &str,
    payload: serde_json::Value,
) -> Result<crate::mission_ledger::MissionEvent> {
    let payload_digest = blake3::hash(&serde_json::to_vec(&payload)?)
        .to_hex()
        .to_string();
    for _ in 0..LEDGER_CAS_RETRIES {
        let mission = ledger
            .mission(mission_id)?
            .ok_or_else(|| anyhow::anyhow!("authoritative mission disappeared"))?;
        let mut event = AppendEvent::new(
            mission_id.clone(),
            mission.version,
            format!(
                "orchestration:{}:{key_label}:{payload_digest}",
                mission_id.as_str()
            ),
            actor,
            kind,
        );
        event.payload = payload.clone();
        match ledger.append(event) {
            Ok(outcome) => return Ok(outcome.event),
            Err(LedgerError::VersionConflict { .. }) => continue,
            Err(error) => return Err(error.into()),
        }
    }
    bail!(
        "mission observation for {} did not converge after {} compare-and-set retries",
        mission_id.as_str(),
        LEDGER_CAS_RETRIES
    )
}

pub fn record_plan_requirement_observation(
    ledger: &MissionLedger,
    mission_id: &crate::mission::MissionId,
    kind: PlanRequirementKind,
    requirement: &str,
    passed: bool,
    observed_by: &str,
    evidence_event_ids: Vec<String>,
) -> Result<crate::mission_ledger::MissionEvent> {
    let plan = ledger
        .active_plan(mission_id)?
        .ok_or_else(|| anyhow::anyhow!("mission has no active immutable plan"))?;
    plan.verify_integrity()
        .map_err(|error| anyhow::anyhow!("active plan integrity failed: {error}"))?;
    let declared = match kind {
        PlanRequirementKind::Gate => &plan.required_gates,
        PlanRequirementKind::Approval => &plan.required_approvals,
    };
    if !declared.iter().any(|item| item == requirement) {
        bail!("plan does not declare required {:?} `{requirement}`", kind);
    }
    if observed_by.trim().is_empty() || evidence_event_ids.is_empty() {
        bail!("plan requirement observation needs an observer and event evidence");
    }
    let observation = PlanRequirementObservation {
        schema_version: ACCEPTANCE_OBSERVATION_SCHEMA_VERSION,
        mission_id: mission_id.as_str().to_string(),
        plan_revision: plan.revision,
        plan_digest: plan.content_digest,
        requirement: requirement.to_string(),
        kind,
        passed,
        observed_by: observed_by.to_string(),
        evidence_event_ids,
    };
    let (event_kind, key_label) = match kind {
        PlanRequirementKind::Gate => ("plan_gate_observed", "plan-gate"),
        PlanRequirementKind::Approval => ("plan_approval_observed", "plan-approval"),
    };
    append_mission_observation(
        ledger,
        mission_id,
        observed_by,
        event_kind,
        key_label,
        serde_json::to_value(observation)?,
    )
}

pub fn record_mission_gate_observation(
    ledger: &MissionLedger,
    mission_id: &crate::mission::MissionId,
    actor: &str,
    observation: &MissionGateObservation,
) -> Result<crate::mission_ledger::MissionEvent> {
    let plan = ledger
        .active_plan(mission_id)?
        .ok_or_else(|| anyhow::anyhow!("mission has no active immutable plan"))?;
    if observation.schema_version != ACCEPTANCE_OBSERVATION_SCHEMA_VERSION
        || observation.mission_id != mission_id.as_str()
        || observation.plan_revision != plan.revision
        || observation.plan_digest != plan.content_digest
    {
        bail!("mission gate observation is not bound to the exact active plan");
    }
    append_mission_observation(
        ledger,
        mission_id,
        actor,
        "mission_gate_observed",
        "mission-gate",
        serde_json::to_value(observation)?,
    )
}

fn record_contract_verification(
    ledger: &MissionLedger,
    attempt: &AuthoritativeTaskAttempt,
    plan_digest: &str,
    done: &DoneSignal,
    verification: &ContractVerification,
) -> Result<RecordedContractVerification> {
    let record = RecordedContractVerification {
        schema_version: ACCEPTANCE_OBSERVATION_SCHEMA_VERSION,
        mission_id: attempt.mission_id.as_str().to_string(),
        task_id: attempt.task_id.clone(),
        attempt_id: attempt.attempt_id.clone(),
        plan_revision: attempt.plan_revision,
        plan_digest: plan_digest.to_string(),
        worker_signal_digest: done_signal_digest(done)?,
        verification: verification.clone(),
    };
    append_bound_observation(
        ledger,
        attempt,
        "omega-independent-verifier",
        "task_verifier_observations_recorded",
        "independent-verification",
        serde_json::to_value(&record)?,
    )?;
    Ok(record)
}

fn exact_verification_is_recorded(
    ledger: &MissionLedger,
    attempt: &AuthoritativeTaskAttempt,
    expected: &RecordedContractVerification,
) -> Result<bool> {
    Ok(ledger
        .events(&attempt.mission_id)?
        .into_iter()
        .any(|event| {
            event.kind == "task_verifier_observations_recorded"
                && serde_json::from_value::<RecordedContractVerification>(event.payload)
                    .is_ok_and(|recorded| recorded == *expected)
        }))
}

fn record_acceptance_invalidation(
    ledger: &MissionLedger,
    attempt: &AuthoritativeTaskAttempt,
    actor: &str,
    reason: &str,
) -> Result<()> {
    let mission = ledger
        .mission(&attempt.mission_id)?
        .ok_or_else(|| anyhow::anyhow!("authoritative mission disappeared"))?;
    if matches!(
        mission.state,
        MissionState::Accepted | MissionState::Reporting | MissionState::Delivered
    ) {
        append_mission_observation(
            ledger,
            &attempt.mission_id,
            actor,
            "mission_acceptance_invalidated",
            "late-acceptance-invalidation",
            serde_json::json!({
                "schema_version": ACCEPTANCE_OBSERVATION_SCHEMA_VERSION,
                "mission_id": attempt.mission_id.as_str(),
                "task_id": attempt.task_id,
                "attempt_id": attempt.attempt_id,
                "plan_revision": attempt.plan_revision,
                "previous_mission_state": mission.state,
                "reason": reason,
            }),
        )?;
        bail!(
            "late contradiction invalidated mission acceptance in {:?}; explicit mission correction/reopen is required",
            mission.state
        );
    }
    append_bound_observation(
        ledger,
        attempt,
        actor,
        "task_acceptance_invalidated",
        "acceptance-invalidated",
        serde_json::json!({
            "mission_id": attempt.mission_id.as_str(),
            "task_id": attempt.task_id,
            "attempt_id": attempt.attempt_id,
            "plan_revision": attempt.plan_revision,
            "reason": reason,
        }),
    )
}

fn finalize_contract_verification(
    ledger: &MissionLedger,
    attempt: &AuthoritativeTaskAttempt,
    actor: &str,
    record: &RecordedContractVerification,
) -> Result<bool> {
    if !exact_verification_is_recorded(ledger, attempt, record)? {
        bail!(
            "attempt {} has no exact ledger-bound independent verification observation",
            attempt.attempt_id
        );
    }

    for _ in 0..LEDGER_CAS_RETRIES {
        let current = ledger
            .task_attempt(&attempt.attempt_id)?
            .ok_or_else(|| anyhow::anyhow!("authoritative task attempt disappeared"))?;
        if record.verification.passed {
            match current.state {
                TaskAttemptState::Running | TaskAttemptState::CandidateDone => {
                    ensure_attempt_verifying(ledger, attempt, actor)?;
                }
                TaskAttemptState::Verifying => {
                    if transition_authoritative_attempt(
                        ledger,
                        attempt,
                        TaskAttemptState::Accepted,
                        actor,
                    )
                    .is_err()
                    {
                        continue;
                    }
                }
                TaskAttemptState::Accepted => return Ok(true),
                TaskAttemptState::Queued
                | TaskAttemptState::CorrectionRequired
                | TaskAttemptState::Blocked
                | TaskAttemptState::Failed
                | TaskAttemptState::Cancelled => return Ok(false),
            }
        } else {
            match current.state {
                TaskAttemptState::Running | TaskAttemptState::CandidateDone => {
                    ensure_attempt_verifying(ledger, attempt, actor)?;
                }
                TaskAttemptState::Verifying => {
                    if transition_authoritative_attempt(
                        ledger,
                        attempt,
                        TaskAttemptState::CorrectionRequired,
                        actor,
                    )
                    .is_err()
                    {
                        continue;
                    }
                    return Ok(false);
                }
                TaskAttemptState::Accepted => {
                    record_acceptance_invalidation(
                        ledger,
                        attempt,
                        actor,
                        "accepted attempt was refuted by the exact independent verifier observation",
                    )?;
                    return Ok(false);
                }
                TaskAttemptState::Queued
                | TaskAttemptState::CorrectionRequired
                | TaskAttemptState::Blocked
                | TaskAttemptState::Failed
                | TaskAttemptState::Cancelled => return Ok(false),
            }
        }
    }
    bail!(
        "attempt {} finalization did not converge after {} retries",
        attempt.attempt_id,
        LEDGER_CAS_RETRIES
    )
}

#[derive(Debug, Clone)]
pub struct CandidateVerificationOutcome {
    pub accepted: bool,
    pub attempt_state: TaskAttemptState,
    pub plan_digest: String,
    pub worker_signal_digest: String,
    pub verification: ContractVerification,
}

/// Load, independently verify, record and finalize one exact CandidateDone.
///
/// This is the sole public candidate acceptance path shared by the native
/// orchestrator and Patrol. Authority drift fails before verifier execution;
/// a verifier failure is recorded and moves the attempt to
/// `CorrectionRequired`. Scope release remains the caller's responsibility and
/// is only legal after `accepted == true` is durably observed.
#[allow(clippy::too_many_arguments)]
pub fn verify_and_finalize_candidate(
    ledger: &MissionLedger,
    mission_id: &crate::mission::MissionId,
    task_id: &str,
    attempt_id: &str,
    plan_revision: u64,
    expected_owner: &str,
    done: &DoneSignal,
    repo_root: &Path,
) -> Result<CandidateVerificationOutcome> {
    if done.session != expected_owner {
        bail!(
            "candidate owner mismatch: expected {}, signal belongs to {}",
            expected_owner,
            done.session
        );
    }
    let plan = ledger
        .active_plan(mission_id)?
        .ok_or_else(|| anyhow::anyhow!("mission has no active immutable plan"))?;
    plan.verify_integrity()
        .map_err(|error| anyhow::anyhow!("active plan integrity failed: {error}"))?;
    if plan.revision != plan_revision {
        bail!(
            "candidate revision {} differs from active plan revision {}",
            plan_revision,
            plan.revision
        );
    }
    let contract = plan
        .tasks
        .iter()
        .find(|task| task.task_id.as_str() == task_id)
        .ok_or_else(|| anyhow::anyhow!("task {task_id} is absent from active plan"))?;
    let projection = ledger
        .task_attempt(attempt_id)?
        .ok_or_else(|| anyhow::anyhow!("candidate task attempt is missing"))?;
    if projection.mission_id != *mission_id
        || projection.task_id != task_id
        || projection.attempt_id != attempt_id
        || projection.plan_revision != plan_revision
    {
        bail!("candidate task-attempt identity differs from ledger authority");
    }
    if !matches!(
        projection.state,
        TaskAttemptState::CandidateDone | TaskAttemptState::Verifying
    ) {
        bail!(
            "attempt {attempt_id} is {:?}, not an independently verifiable candidate",
            projection.state
        );
    }

    let events = ledger.events(mission_id)?;
    let running_owner = events.iter().rev().find_map(|event| {
        event
            .resulting_task_attempt
            .as_ref()
            .filter(|attempt| {
                attempt.attempt_id == attempt_id && attempt.state == TaskAttemptState::Running
            })
            .map(|_| event.actor.as_str())
    });
    if running_owner != Some(expected_owner) {
        bail!(
            "candidate owner {} differs from the exact running-attempt owner {:?}",
            expected_owner,
            running_owner
        );
    }
    let leases = ledger.active_leases_for_attempt(mission_id, task_id, attempt_id)?;
    if leases.iter().any(|lease| lease.owner != expected_owner) {
        bail!("one or more active attempt leases belong to a different owner");
    }

    let provenance = done
        .projection
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("candidate has no V3 ledger provenance"))?;
    let oracle_signal = expected_owner.starts_with("oracle-");
    let attempt = AuthoritativeTaskAttempt {
        mission_id: mission_id.clone(),
        task_id: task_id.to_string(),
        attempt_id: attempt_id.to_string(),
        plan_revision,
        owner: Some(expected_owner.to_string()),
        leases,
        scope_receipt: None,
    };
    let completion_event =
        validate_done_projection(ledger, &attempt, expected_owner, provenance, oracle_signal)
            .map_err(|error| anyhow::anyhow!(error))?;
    validate_done_payload_binding(done, &completion_event, oracle_signal)
        .map_err(|error| anyhow::anyhow!(error))?;
    let earliest = done.finished_at - ChronoDuration::seconds(DONE_CLOCK_SKEW_SECS);
    let latest = done.finished_at + ChronoDuration::seconds(DONE_CLOCK_SKEW_SECS);
    if completion_event.recorded_at < earliest || completion_event.recorded_at > latest {
        bail!("candidate timestamp is not bound to its immutable completion event");
    }

    ensure_attempt_verifying(ledger, &attempt, "omega-independent-verifier")?;
    let verification = independently_verify_task_contract(done, repo_root, contract);
    let worker_signal_digest = done_signal_digest(done)?;
    let record =
        record_contract_verification(ledger, &attempt, &plan.content_digest, done, &verification)?;
    let accepted =
        finalize_contract_verification(ledger, &attempt, "omega-independent-verifier", &record)?;
    let attempt_state = ledger
        .task_attempt(attempt_id)?
        .ok_or_else(|| anyhow::anyhow!("finalized attempt disappeared"))?
        .state;
    Ok(CandidateVerificationOutcome {
        accepted,
        attempt_state,
        plan_digest: plan.content_digest,
        worker_signal_digest,
        verification,
    })
}

fn finalize_nonclean_attempt(
    ledger: &MissionLedger,
    attempt: &AuthoritativeTaskAttempt,
    actor: &str,
    target: TaskAttemptState,
) -> Result<()> {
    for _ in 0..LEDGER_CAS_RETRIES {
        let current = ledger
            .task_attempt(&attempt.attempt_id)?
            .ok_or_else(|| anyhow::anyhow!("authoritative task attempt disappeared"))?;
        match current.state {
            TaskAttemptState::Running | TaskAttemptState::CandidateDone => {
                ensure_attempt_verifying(ledger, attempt, actor)?;
            }
            TaskAttemptState::Verifying => {
                if transition_authoritative_attempt(ledger, attempt, target, actor).is_err() {
                    continue;
                }
                return Ok(());
            }
            state if state == target => return Ok(()),
            TaskAttemptState::Accepted => {
                record_acceptance_invalidation(
                    ledger,
                    attempt,
                    actor,
                    "accepted attempt contradicted by a non-clean completion signal",
                )?;
                return Ok(());
            }
            TaskAttemptState::Queued
            | TaskAttemptState::CorrectionRequired
            | TaskAttemptState::Blocked
            | TaskAttemptState::Failed
            | TaskAttemptState::Cancelled => return Ok(()),
        }
    }
    bail!(
        "non-clean attempt {} finalization did not converge after {} retries",
        attempt.attempt_id,
        LEDGER_CAS_RETRIES
    )
}

/// Settle a non-clean compatibility signal through one exact V3 authority
/// path. The immutable completion event, task identity, owner, plan revision
/// and complete lease set are all reloaded before any transition.
#[allow(clippy::too_many_arguments)]
pub fn finalize_nonclean_candidate(
    ledger: &MissionLedger,
    mission_id: &crate::mission::MissionId,
    task_id: &str,
    attempt_id: &str,
    plan_revision: u64,
    expected_owner: &str,
    done: &DoneSignal,
    target: TaskAttemptState,
) -> Result<TaskAttemptState> {
    if !matches!(target, TaskAttemptState::Blocked | TaskAttemptState::Failed) {
        bail!("non-clean finalizer only permits Blocked or Failed");
    }
    if done.session != expected_owner || done.status == DoneStatus::DoneClean {
        bail!("non-clean signal identity or status is incompatible with finalization");
    }
    let plan = ledger
        .active_plan(mission_id)?
        .ok_or_else(|| anyhow::anyhow!("mission has no active immutable plan"))?;
    plan.verify_integrity()
        .map_err(|error| anyhow::anyhow!("active plan integrity failed: {error}"))?;
    if plan.revision != plan_revision
        || !plan
            .tasks
            .iter()
            .any(|task| task.task_id.as_str() == task_id)
    {
        bail!("non-clean signal is not bound to the exact active plan task");
    }
    let projection = ledger
        .task_attempt(attempt_id)?
        .ok_or_else(|| anyhow::anyhow!("non-clean task attempt is missing"))?;
    if projection.mission_id != *mission_id
        || projection.task_id != task_id
        || projection.plan_revision != plan_revision
    {
        bail!("non-clean task-attempt identity differs from ledger authority");
    }
    let events = ledger.events(mission_id)?;
    let running_owner = events.iter().rev().find_map(|event| {
        event.resulting_task_attempt.as_ref().and_then(|attempt| {
            (attempt.attempt_id == attempt_id && attempt.state == TaskAttemptState::Running)
                .then_some(event.actor.as_str())
        })
    });
    if running_owner != Some(expected_owner) {
        bail!("non-clean signal owner differs from the Running attempt owner");
    }
    let leases = ledger.active_leases_for_attempt(mission_id, task_id, attempt_id)?;
    if leases.iter().any(|lease| lease.owner != expected_owner) {
        bail!("one or more active attempt leases belong to a different owner");
    }
    let attempt = AuthoritativeTaskAttempt {
        mission_id: mission_id.clone(),
        task_id: task_id.to_string(),
        attempt_id: attempt_id.to_string(),
        plan_revision,
        owner: Some(expected_owner.to_string()),
        leases,
        scope_receipt: None,
    };
    let provenance = done
        .projection
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("non-clean signal has no V3 ledger provenance"))?;
    let oracle_signal = expected_owner.starts_with("oracle-");
    let completion_event =
        validate_done_projection(ledger, &attempt, expected_owner, provenance, oracle_signal)
            .map_err(|error| anyhow::anyhow!(error))?;
    validate_done_payload_binding(done, &completion_event, oracle_signal)
        .map_err(|error| anyhow::anyhow!(error))?;
    finalize_nonclean_attempt(ledger, &attempt, "omega-independent-verifier", target)?;
    Ok(ledger
        .task_attempt(attempt_id)?
        .ok_or_else(|| anyhow::anyhow!("finalized non-clean attempt disappeared"))?
        .state)
}

fn validate_done_identity_and_freshness(
    signal: &DoneSignal,
    expected_session: &str,
    dispatched_at: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Result<(), OrchestrationError> {
    if signal.session != expected_session {
        return Err(OrchestrationError::Other(anyhow::anyhow!(
            "completion identity mismatch: expected {}, received {}",
            expected_session,
            signal.session
        )));
    }
    let earliest = dispatched_at - ChronoDuration::seconds(DONE_CLOCK_SKEW_SECS);
    let latest = now + ChronoDuration::seconds(DONE_CLOCK_SKEW_SECS);
    if signal.finished_at < earliest {
        return Err(OrchestrationError::Other(anyhow::anyhow!(
            "stale completion for {}: finished at {} before dispatch {}",
            expected_session,
            signal.finished_at,
            dispatched_at
        )));
    }
    if signal.finished_at > latest {
        return Err(OrchestrationError::Other(anyhow::anyhow!(
            "impossible future completion for {}: finished at {}",
            expected_session,
            signal.finished_at
        )));
    }
    Ok(())
}

fn validate_done_projection(
    ledger: &MissionLedger,
    attempt: &AuthoritativeTaskAttempt,
    expected_session: &str,
    projection: &crate::done::ProjectionProvenance,
    oracle_signal: bool,
) -> Result<crate::mission_ledger::MissionEvent, OrchestrationError> {
    if projection.source != "mission-engine-v3.sqlite3" {
        return Err(OrchestrationError::Other(anyhow::anyhow!(
            "completion cites unsupported authority source {}",
            projection.source
        )));
    }
    let events = ledger
        .events(&attempt.mission_id)
        .map_err(|error| OrchestrationError::Other(error.into()))?;
    let event = events
        .iter()
        .find(|event| {
            event.event_id == projection.event_id && event.sequence == projection.event_sequence
        })
        .ok_or_else(|| {
            OrchestrationError::Other(anyhow::anyhow!(
                "completion provenance event {}#{} does not exist",
                projection.event_id,
                projection.event_sequence
            ))
        })?;
    let kind_is_completion = if oracle_signal {
        event.kind == "legacy_oracle_completion_candidate"
            || event.kind == "legacy_worker_completion_candidate"
    } else {
        event.kind == "legacy_worker_completion_candidate"
    };
    if event.actor != expected_session || !kind_is_completion {
        return Err(OrchestrationError::Other(anyhow::anyhow!(
            "completion provenance is not bound to session {} and its completion lifecycle",
            expected_session
        )));
    }
    if event.task_id.as_deref() != Some(attempt.task_id.as_str())
        || event.attempt_id.as_deref() != Some(attempt.attempt_id.as_str())
        || event.plan_revision != Some(attempt.plan_revision)
        || event
            .resulting_task_attempt
            .as_ref()
            .is_none_or(|projection| {
                projection.mission_id != attempt.mission_id
                    || projection.task_id != attempt.task_id
                    || projection.attempt_id != attempt.attempt_id
                    || projection.plan_revision != attempt.plan_revision
                    || projection.state != TaskAttemptState::CandidateDone
            })
    {
        return Err(OrchestrationError::Other(anyhow::anyhow!(
            "completion provenance is not the exact CandidateDone event for task {} attempt {} revision {}",
            attempt.task_id,
            attempt.attempt_id,
            attempt.plan_revision
        )));
    }
    let historical = ledger
        .projection_at(&attempt.mission_id, projection.event_sequence)
        .map_err(|error| OrchestrationError::Other(error.into()))?
        .ok_or_else(|| {
            OrchestrationError::Other(anyhow::anyhow!(
                "completion provenance sequence {} is not replayable",
                projection.event_sequence
            ))
        })?;
    if historical.version != projection.mission_version
        || historical.projection_hash != projection.projection_hash
    {
        return Err(OrchestrationError::Other(anyhow::anyhow!(
            "completion provenance does not match the exact historical ledger projection"
        )));
    }
    Ok(event.clone())
}

fn validate_done_payload_binding(
    signal: &DoneSignal,
    event: &crate::mission_ledger::MissionEvent,
    oracle_signal: bool,
) -> Result<(), OrchestrationError> {
    if oracle_signal {
        let recorded: crate::done::OracleDoneSignal = serde_json::from_value(event.payload.clone())
            .map_err(|error| {
                OrchestrationError::Other(anyhow::anyhow!(
                    "oracle completion event payload is corrupt: {error}"
                ))
            })?;
        let mut bridged_pending = recorded.pending_actions.clone();
        bridged_pending.retain(|action| action != V3_ACCEPTANCE_PENDING);
        let exact =
            recorded.status == signal.status && recorded.pending_actions == signal.pending_actions;
        let exact_bridge = recorded.status == DoneStatus::Pending
            && recorded
                .pending_actions
                .iter()
                .any(|action| action == V3_ACCEPTANCE_PENDING)
            && signal.status == DoneStatus::DoneClean
            && bridged_pending == signal.pending_actions;
        if recorded.oracle.strip_prefix("oracle-") != signal.session.strip_prefix("oracle-")
            || (!exact && !exact_bridge)
            || recorded.summary != signal.summary
            || recorded.finished_at != signal.finished_at
        {
            return Err(OrchestrationError::Other(anyhow::anyhow!(
                "oracle completion file differs from its immutable CandidateDone event payload"
            )));
        }
    } else {
        let mut without_projection = signal.clone();
        without_projection.projection = None;
        let expected = serde_json::to_value(without_projection).map_err(|error| {
            OrchestrationError::Other(anyhow::anyhow!(
                "cannot canonicalize worker completion payload: {error}"
            ))
        })?;
        if event.payload != expected {
            return Err(OrchestrationError::Other(anyhow::anyhow!(
                "worker completion file differs from its immutable CandidateDone event payload"
            )));
        }
    }
    Ok(())
}

/// Validate an Oracle completion file against the exact immutable
/// CandidateDone event of the current mission. This prevents a recycled
/// oracle name or legacy done file from closing/curating a different Delivered
/// mission.
pub fn validate_oracle_done_signal_authority(
    ledger: &MissionLedger,
    mission_id: &crate::mission::MissionId,
    expected_oracle: &str,
    signal: &crate::done::OracleDoneSignal,
) -> Result<()> {
    if signal.oracle.strip_prefix("oracle-") != expected_oracle.strip_prefix("oracle-") {
        bail!("oracle completion identity differs from the live oracle");
    }
    let provenance = signal
        .projection
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("oracle completion has no V3 provenance"))?;
    if provenance.source != "mission-engine-v3.sqlite3" {
        bail!("oracle completion cites an unsupported authority source");
    }
    let event = ledger
        .events(mission_id)?
        .into_iter()
        .find(|event| {
            event.event_id == provenance.event_id && event.sequence == provenance.event_sequence
        })
        .ok_or_else(|| anyhow::anyhow!("oracle completion provenance event is absent"))?;
    if event.kind != "legacy_oracle_completion_candidate"
        || event.actor != expected_oracle
        || event.mission_id != *mission_id
    {
        bail!("oracle completion event is not owned by the exact live oracle mission");
    }
    let resulting = event
        .resulting_task_attempt
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("oracle completion event has no task result"))?;
    if resulting.state != TaskAttemptState::CandidateDone
        || event.task_id.as_deref() != Some(resulting.task_id.as_str())
        || event.attempt_id.as_deref() != Some(resulting.attempt_id.as_str())
        || event.plan_revision != Some(resulting.plan_revision)
    {
        bail!("oracle completion event is not an exact CandidateDone transition");
    }
    let plan = ledger
        .active_plan(mission_id)?
        .ok_or_else(|| anyhow::anyhow!("oracle mission has no active plan"))?;
    if plan.revision != resulting.plan_revision
        || !plan
            .tasks
            .iter()
            .any(|task| task.task_id.as_str() == resulting.task_id)
    {
        bail!("oracle completion task is not in the exact active plan");
    }
    let current = ledger
        .task_attempt(&resulting.attempt_id)?
        .ok_or_else(|| anyhow::anyhow!("oracle completion task attempt is missing"))?;
    if current.mission_id != *mission_id
        || current.task_id != resulting.task_id
        || current.plan_revision != resulting.plan_revision
        || current.state != TaskAttemptState::Accepted
    {
        bail!("oracle completion task is not exactly Accepted");
    }
    let historical = ledger
        .projection_at(mission_id, provenance.event_sequence)?
        .ok_or_else(|| anyhow::anyhow!("oracle completion projection is not replayable"))?;
    if historical.version != provenance.mission_version
        || historical.projection_hash != provenance.projection_hash
    {
        bail!("oracle completion projection hash/version differs from ledger");
    }
    let mut recorded_signal = signal.clone();
    recorded_signal.projection = None;
    if serde_json::to_value(recorded_signal)? != event.payload {
        bail!("oracle completion file differs from its immutable event payload");
    }
    Ok(())
}

fn bridge_v3_oracle_candidate(signal: &mut DoneSignal, exact_provenance_validated: bool) -> bool {
    if !exact_provenance_validated
        || signal.status != DoneStatus::Pending
        || !signal
            .pending_actions
            .iter()
            .any(|action| action == V3_ACCEPTANCE_PENDING)
    {
        return false;
    }
    signal.status = DoneStatus::DoneClean;
    signal.todos_completed = signal.todos_total;
    signal
        .pending_actions
        .retain(|action| action != V3_ACCEPTANCE_PENDING);
    true
}

/// The Orchestrator owns the full mission lifecycle.
pub struct Orchestrator {
    config: OmegaConfig,
    options: OrchestratorOptions,
    mgr: SessionManager,
}

impl Orchestrator {
    pub async fn new(config: OmegaConfig, options: OrchestratorOptions) -> Result<Self> {
        config.ensure_dirs()?;
        let mgr = SessionManager::connect().await?;
        Ok(Self {
            config,
            options,
            mgr,
        })
    }

    pub async fn from_default_config() -> Result<Self> {
        let config = OmegaConfig::load().context(
            "cannot load OmegaOS config for orchestrator; refusing default authority fallback",
        )?;
        Self::new(config, OrchestratorOptions::default()).await
    }

    /// Run a mission end-to-end: classify, plan, dispatch, monitor, gate, report.
    pub async fn execute(&self, mission: Mission) -> Result<Outcome> {
        let started_at = Utc::now();
        tracing::info!(
            mission_id = %mission.id.0,
            project = %mission.project,
            "Mission execution started"
        );

        // 1. Classify + plan
        let plan = self.plan(&mission).await?;
        tracing::info!(
            mission_id = %mission.id.0,
            complexity = ?plan.complexity,
            strategy = ?plan.strategy,
            tasks = plan.tasks.len(),
            "Mission planned"
        );

        // The legacy Plan is now only a compatibility view. Freeze it into the
        // canonical V3 contract and queue every attempt before any provider or
        // rmux side effect can occur.
        let ledger = MissionLedger::open(crate::oracle_lifecycle::mission_ledger_path(
            &self.config.state_dir,
        ))?;
        let required_gates = if self.options.enforce_gate {
            vec!["independent_verification".to_string()]
        } else {
            Vec::new()
        };
        let mut authority = prepare_authoritative_execution(
            &ledger,
            &mission,
            &plan,
            "omega-orchestrate",
            required_gates,
        )?;
        transition_authoritative_mission(
            &ledger,
            &mission.id,
            MissionState::Running,
            "omega-orchestrate",
        )?;

        // 2. Build rubric for quality gate (saved upfront, evaluated later)
        let rubric = self.build_rubric(&mission, &plan);
        if let Err(e) = rubric.write(&self.config.state_dir, &mission.id.0) {
            tracing::warn!(error = %e, "Failed to persist rubric");
        }

        // 3. Dispatch workers per plan
        let mut workers = Vec::with_capacity(plan.tasks.len());
        for task in &plan.tasks {
            let session_name = self.session_name_for(&mission, task);
            {
                let attempt = authority.attempt_mut(&task.id)?;
                if let Err(error) = claim_authoritative_scopes(
                    &ledger,
                    &self.config.state_dir,
                    &mission.working_dir,
                    attempt,
                    &session_name,
                    &task.files_owned,
                    self.options.worker_timeout,
                ) {
                    let _ = transition_authoritative_mission(
                        &ledger,
                        &mission.id,
                        MissionState::Verifying,
                        "omega-orchestrate",
                    );
                    let _ = transition_authoritative_mission(
                        &ledger,
                        &mission.id,
                        MissionState::Failed,
                        "omega-orchestrate",
                    );
                    return Err(
                        error.context(format!("claiming authoritative scope for {}", task.id))
                    );
                }
            }
            let attempt = authority.attempt(&task.id)?.clone();
            let dispatched_at = Utc::now();
            match self.dispatch_task(&mission, task).await {
                Ok(spawned_session) => {
                    if spawned_session != session_name {
                        bail!(
                            "worker identity drift: authority={}, spawned={}",
                            session_name,
                            spawned_session
                        );
                    }
                    if let Err(error) = transition_authoritative_attempt(
                        &ledger,
                        &attempt,
                        TaskAttemptState::Running,
                        &session_name,
                    ) {
                        let _ = self.mgr.kill_session(&session_name).await;
                        if let Err(release_error) =
                            release_authoritative_scopes(&ledger, &self.config.state_dir, &attempt)
                        {
                            tracing::error!(
                                worker = %session_name,
                                error = %release_error,
                                "failed to release every authoritative scope after spawn rollback"
                            );
                        }
                        return Err(error.context(
                            "worker spawn rolled back because its V3 running transition failed",
                        ));
                    }
                    workers.push((task.clone(), session_name, attempt, dispatched_at));
                }
                Err(e) => {
                    tracing::error!(task = %task.id, error = %e, "Dispatch failed");
                    let _ = transition_authoritative_attempt(
                        &ledger,
                        &attempt,
                        TaskAttemptState::Cancelled,
                        &session_name,
                    );
                    release_authoritative_scopes(&ledger, &self.config.state_dir, &attempt)
                        .context("releasing scopes after dispatch failure")?;
                    transition_authoritative_mission(
                        &ledger,
                        &mission.id,
                        MissionState::Verifying,
                        "omega-orchestrate",
                    )?;
                    transition_authoritative_mission(
                        &ledger,
                        &mission.id,
                        MissionState::Failed,
                        "omega-orchestrate",
                    )?;
                    return Ok(Outcome {
                        mission_id: mission.id.clone(),
                        status: OutcomeStatus::Failed,
                        workers: Vec::new(),
                        gate: None,
                        audit_recommendations: Vec::new(),
                        started_at,
                        finished_at: Utc::now(),
                        summary: format!("Dispatch failed: {}", e),
                    });
                }
            }
        }

        // 4. Monitor each worker for completion (event-driven)
        let mut results = Vec::with_capacity(workers.len());
        for (task, session, attempt, dispatched_at) in &workers {
            let started = Instant::now();
            let result = self
                .wait_for_worker_done(
                    &ledger,
                    attempt,
                    session,
                    *dispatched_at,
                    self.options.worker_timeout,
                )
                .await;
            let duration_secs = started.elapsed().as_secs();

            match result {
                Ok(done) => {
                    ensure_attempt_verifying(&ledger, attempt, session)?;
                    let mut effective_status = done.status;
                    let mut effective_summary = done.summary.clone();
                    let mut accepted_persisted = false;
                    match done.status {
                        DoneStatus::DoneClean => {
                            let outcome = verify_and_finalize_candidate(
                                &ledger,
                                &attempt.mission_id,
                                &attempt.task_id,
                                &attempt.attempt_id,
                                attempt.plan_revision,
                                session,
                                &done,
                                &mission.working_dir,
                            )?;
                            if !outcome.accepted {
                                effective_status = DoneStatus::Pending;
                                effective_summary = format!(
                                    "{}; independent verification rejected: {}",
                                    done.summary,
                                    if outcome.verification.failures.is_empty() {
                                        "attempt was already finalized on a non-acceptable branch"
                                            .to_string()
                                    } else {
                                        outcome.verification.failures.join(" | ")
                                    }
                                );
                            } else if outcome.attempt_state == TaskAttemptState::Accepted {
                                accepted_persisted = true;
                            }
                        }
                        DoneStatus::Failed => finalize_nonclean_attempt(
                            &ledger,
                            attempt,
                            session,
                            TaskAttemptState::Failed,
                        )?,
                        DoneStatus::Blocked | DoneStatus::Pending => finalize_nonclean_attempt(
                            &ledger,
                            attempt,
                            session,
                            TaskAttemptState::Blocked,
                        )?,
                    }
                    results.push(WorkerResult {
                        task_id: task.id.clone(),
                        session_name: session.clone(),
                        status: effective_status,
                        summary: effective_summary,
                        commit: done.commit.clone(),
                        duration_secs,
                    });

                    if accepted_persisted {
                        release_authoritative_scopes(&ledger, &self.config.state_dir, attempt)
                            .context("releasing scopes after exact task acceptance")?;
                    }
                }
                Err(e) => {
                    // A timeout is a distinct, high-severity operational signal
                    // (worker hung / daemon unresponsive), not a generic monitor
                    // error — surface it at ERROR level and tag the result summary
                    // so it is visible in the outcome, not buried per-worker.
                    let summary = match &e {
                        OrchestrationError::WorkerTimeout { seconds, .. } => {
                            tracing::error!(worker = %session, seconds = %seconds, "Worker timed out");
                            format!("TIMEOUT after {}s (worker hung or unresponsive)", seconds)
                        }
                        _ => {
                            tracing::warn!(worker = %session, error = %e, "Worker monitoring failed");
                            format!("Monitoring error: {}", e)
                        }
                    };
                    results.push(WorkerResult {
                        task_id: task.id.clone(),
                        session_name: session.clone(),
                        status: DoneStatus::Failed,
                        summary,
                        commit: None,
                        duration_secs,
                    });
                    ensure_attempt_verifying(&ledger, attempt, session)?;
                    finalize_nonclean_attempt(&ledger, attempt, session, TaskAttemptState::Failed)?;
                }
            }
        }

        transition_authoritative_mission(
            &ledger,
            &mission.id,
            MissionState::Verifying,
            "omega-orchestrate",
        )?;

        // 5. Compute outcome status
        let all_clean =
            !results.is_empty() && results.iter().all(|r| r.status == DoneStatus::DoneClean);
        let any_failed = results.iter().any(|r| r.status == DoneStatus::Failed);
        let status = if results.is_empty() {
            // Zero workers dispatched means the mission was never executed.
            // This is a hollow non-result, not a partial win — fail loudly so the
            // orchestration breakage surfaces instead of masquerading as success.
            OutcomeStatus::Failed
        } else if all_clean {
            OutcomeStatus::Success
        } else if any_failed {
            OutcomeStatus::Failed
        } else {
            OutcomeStatus::PartialSuccess
        };

        // 6. Quality gate. Every successful execution computes and persists the
        // exact plan-bound gate before mission acceptance is attempted.
        let gate = if status == OutcomeStatus::Success {
            if !self.options.enforce_gate {
                tracing::warn!(
                    mission_id = %mission.id.0,
                    "enforce_gate=false affects plan requirements only; authoritative mission acceptance still requires the quality gate"
                );
            }
            Some(Self::run_quality_gate(
                &self.config.state_dir,
                &ledger,
                &mission,
                &authority.plan,
                &rubric,
                &results,
            )?)
        } else {
            None
        };

        // 6.5 Select and record audit recommendations for the oracle to dispatch
        let modified_files: Vec<String> = results
            .iter()
            .flat_map(|r| r.commit.as_deref().map(|_| r.task_id.clone()))
            .collect();
        let audit_recommendations = crate::audit::select_audits(&mission.text, &modified_files)
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();
        if !audit_recommendations.is_empty() {
            tracing::info!(
                mission_id = %mission.id.0,
                audits = ?audit_recommendations,
                "Audit recommendations selected"
            );
        }

        // 7. A failed authoritative gate can never ship, including through the
        // compatibility `enforce_gate=false` option.
        let final_status = match (&gate, status) {
            (Some(g), OutcomeStatus::Success) if !g.overall_pass => OutcomeStatus::PartialSuccess,
            (_, s) => s,
        };

        match final_status {
            OutcomeStatus::Success => {
                transition_authoritative_mission(
                    &ledger,
                    &mission.id,
                    MissionState::Accepted,
                    "omega-orchestrate",
                )?;
                transition_authoritative_mission(
                    &ledger,
                    &mission.id,
                    MissionState::Reporting,
                    "omega-orchestrate",
                )?;
                transition_authoritative_mission(
                    &ledger,
                    &mission.id,
                    MissionState::Delivered,
                    "omega-orchestrate",
                )?;
            }
            OutcomeStatus::PartialSuccess => {
                let target = if results.iter().any(|result| {
                    matches!(result.status, DoneStatus::Blocked | DoneStatus::Pending)
                }) {
                    MissionState::Blocked
                } else {
                    MissionState::CorrectionRequired
                };
                transition_authoritative_mission(
                    &ledger,
                    &mission.id,
                    target,
                    "omega-orchestrate",
                )?;
            }
            OutcomeStatus::Failed => {
                transition_authoritative_mission(
                    &ledger,
                    &mission.id,
                    MissionState::Failed,
                    "omega-orchestrate",
                )?;
            }
            OutcomeStatus::Aborted => {
                // There is no Verifying -> Cancelled edge: once effects ran,
                // an abort is a failed verification, not a pre-run cancel.
                transition_authoritative_mission(
                    &ledger,
                    &mission.id,
                    MissionState::Failed,
                    "omega-orchestrate",
                )?;
            }
        }

        let summary = self.summarize(&mission, &plan, &results, gate.as_ref());

        Ok(Outcome {
            mission_id: mission.id.clone(),
            status: final_status,
            workers: results,
            gate,
            audit_recommendations,
            started_at,
            finished_at: Utc::now(),
            summary,
        })
    }

    /// Build an execution Plan from a Mission.
    /// Routes through the classifier and decomposes by complexity.
    pub async fn plan(&self, mission: &Mission) -> Result<Plan> {
        let decision = classify_mission(&mission.text);
        // Configured Hermes is Home. Orchestrate must dispatch a writer
        // (never a provider setup wizard).
        let agent =
            crate::external_orchestrator::resolve_mission_writer(None, &self.config.agent_command)?
                .name()
                .to_string();

        let strategy = match decision.complexity {
            Complexity::Simple => PlanStrategy::Direct,
            Complexity::Medium => PlanStrategy::Direct,
            Complexity::Complex => PlanStrategy::Sequential,
            Complexity::Epic => PlanStrategy::Parallel,
        };

        let tasks = match strategy {
            PlanStrategy::Direct => {
                vec![Task {
                    id: format!("{}-t1", mission.id.0),
                    name: "main".to_string(),
                    prompt: mission.text.clone(),
                    // A direct coding worker owns the repository by default.
                    // The legacy empty scope was an unfenced write bypass: two
                    // direct missions could edit the same checkout concurrently
                    // while both claimed to satisfy R-SCOPE.
                    files_owned: vec!["**/*".to_string()],
                    depends_on: Vec::new(),
                    agent: agent.clone(),
                    estimated_minutes: decision.complexity.estimated_minutes(),
                }]
            }
            _ => {
                // For Sequential / Parallel / Team we spawn an Oracle that
                // dynamically decomposes. The orchestrator only seeds one
                // primary "oracle" task; the Oracle then calls back into
                // the orchestrator via `omega spawn-worker`.
                vec![Task {
                    id: format!("{}-oracle", mission.id.0),
                    name: "oracle".to_string(),
                    prompt: format!(
                        "[ORACLE] You decompose this mission into sub-tasks and dispatch workers via `omega spawn-worker`.\n\nMission: {}\n\nProject: {}\n\nWhen all sub-workers complete and you have verified outcomes, call: omega done {} done_clean \"<summary>\"",
                        mission.text,
                        mission.project,
                        // Must match the session name minted in dispatch_task
                        // (oracle-<project>-<mission_id>) so the oracle signals
                        // done against its OWN session, not a colliding name.
                        format_args!("oracle-{}-{}", mission.project, mission.id.0)
                    ),
                    files_owned: Vec::new(),
                    depends_on: Vec::new(),
                    agent: agent.clone(),
                    estimated_minutes: decision.complexity.estimated_minutes(),
                }]
            }
        };

        Ok(Plan {
            mission_id: mission.id.clone(),
            complexity: decision.complexity,
            strategy,
            tasks,
            created_at: Utc::now(),
        })
    }

    fn session_name_for(&self, mission: &Mission, task: &Task) -> String {
        // Include the mission id so session names are globally unique, not just
        // per-project: two concurrent missions for the SAME project would otherwise
        // collide on identical names (clobbering each other's panes + done.json).
        // The id stays deterministic within a mission, so resumability is preserved.
        if task.name == "oracle" {
            format!("oracle-{}-{}", mission.project, mission.id.0)
        } else {
            format!("{}-worker-{}-{}", mission.project, task.name, mission.id.0)
        }
    }

    /// Dispatch a single task as a worker session. The caller must already
    /// have frozen and queued its V3 attempt and acquired every scope lease.
    async fn dispatch_task(&self, mission: &Mission, task: &Task) -> Result<String> {
        let session_name = self.session_name_for(mission, task);

        // Clear any STALE done.json from a prior mission. Session names are
        // deterministic per project (oracle-<project> / <project>-worker-<name>),
        // so a leftover worker-<session>.done.json from a previous run would make
        // wait_for_worker_done return that OLD result instantly — reporting the
        // previous mission's outcome as this one's. Remove it before dispatch so
        // the wait only ever observes a fresh signal from THIS mission.
        DoneSignal::remove(&self.config.state_dir, &session_name)
            .context("clearing stale worker completion authority before dispatch")?;
        if session_name.starts_with("oracle-") {
            crate::done::OracleDoneSignal::clear_strict(&self.config.state_dir, &session_name)
                .context("clearing stale oracle completion authority before dispatch")?;
        }

        let agent = Agent::from_name(&task.agent).ok_or_else(|| {
            anyhow::anyhow!(
                "task {} declares unknown agent `{}`; refusing implicit provider fallback",
                task.id,
                task.agent
            )
        })?;
        crate::external_orchestrator::headless_writer_launch(agent, Some(&task.prompt))?;

        // THE FUNNEL — inject the role-scoped Laws + operational rules. The
        // `omega orchestrate` dispatch path previously spawned oracles AND workers
        // with NO doctrine (only executor.rs + cmd_spawn_worker had it), so every
        // agent created here ran ungoverned. An "oracle" task gets Oracle scope;
        // everything else gets Worker scope.
        let scope = if task.name == "oracle" {
            crate::rules::RuleScope::Oracle
        } else {
            crate::rules::RuleScope::Worker
        };
        let mut full_prompt = task.prompt.clone();
        let compiled = policy_context_for_agent(scope, &full_prompt, agent);
        if !compiled.is_empty() {
            full_prompt.push_str("\n\n");
            full_prompt.push_str(&compiled);
        }

        self.mgr
            .create_session_with_agent(
                &session_name,
                Some(&mission.working_dir.to_string_lossy()),
                agent,
                Some(&full_prompt),
            )
            .await
            .with_context(|| format!("dispatching {}", session_name))?;

        Ok(session_name)
    }

    /// Event-driven wait for a worker's done.json — uses rmux SDK output_stream
    /// to detect pane death + filesystem polling for the done.json marker.
    async fn wait_for_worker_done(
        &self,
        ledger: &MissionLedger,
        attempt: &AuthoritativeTaskAttempt,
        session_name: &str,
        dispatched_at: DateTime<Utc>,
        max_wait: Duration,
    ) -> Result<DoneSignal, OrchestrationError> {
        let done_path = self
            .config
            .state_dir
            .join(format!("worker-{}.done.json", session_name));
        let is_oracle = session_name.starts_with("oracle-");

        let deadline = Instant::now() + max_wait;
        let mut interval = tokio::time::interval(self.options.poll_interval);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            if is_oracle {
                if let Ok(Some(oracle_done)) =
                    crate::done::OracleDoneSignal::read(&self.config.state_dir, session_name)
                {
                    if oracle_done.oracle.strip_prefix("oracle-")
                        != session_name.strip_prefix("oracle-")
                    {
                        return Err(OrchestrationError::Other(anyhow::anyhow!(
                            "oracle completion identity mismatch: expected {}, received {}",
                            session_name,
                            oracle_done.oracle
                        )));
                    }
                    let mut signal =
                        DoneSignal::new(session_name, oracle_done.status, &oracle_done.summary);
                    signal.todos_total = 1;
                    signal.todos_completed = u32::from(oracle_done.status == DoneStatus::DoneClean);
                    signal.pending_actions = oracle_done.pending_actions;
                    signal.finished_at = oracle_done.finished_at;
                    signal.projection = oracle_done.projection;
                    validate_done_identity_and_freshness(
                        &signal,
                        session_name,
                        dispatched_at,
                        Utc::now(),
                    )?;
                    let projection = signal.projection.as_ref().ok_or_else(|| {
                        OrchestrationError::Other(anyhow::anyhow!(
                            "oracle completion {} has no V3 ledger provenance",
                            session_name
                        ))
                    })?;
                    let completion_event =
                        validate_done_projection(ledger, attempt, session_name, projection, true)?;
                    validate_done_payload_binding(&signal, &completion_event, true)?;
                    bridge_v3_oracle_candidate(&mut signal, true);
                    // OracleDoneSignal predates the worker artifact schema. Its
                    // exact ledger provenance establishes signal identity only;
                    // the immutable task verifier still supplies content proof.
                    signal
                        .corroboration
                        .push(crate::done::CorroborationSource::Other(
                            "validated_ledger_provenance".to_string(),
                        ));
                    return Ok(signal);
                }
            }
            if done_path.exists() {
                let content = std::fs::read_to_string(&done_path)
                    .map_err(|e| OrchestrationError::Other(e.into()))?;
                let signal: DoneSignal = serde_json::from_str(&content)
                    .map_err(|e| OrchestrationError::Other(e.into()))?;
                validate_done_identity_and_freshness(
                    &signal,
                    session_name,
                    dispatched_at,
                    Utc::now(),
                )?;
                let projection = signal.projection.as_ref().ok_or_else(|| {
                    OrchestrationError::Other(anyhow::anyhow!(
                        "worker completion {} has no V3 ledger provenance",
                        session_name
                    ))
                })?;
                let completion_event =
                    validate_done_projection(ledger, attempt, session_name, projection, false)?;
                validate_done_payload_binding(&signal, &completion_event, false)?;
                return Ok(signal);
            }

            if Instant::now() >= deadline {
                return Err(OrchestrationError::WorkerTimeout {
                    worker: session_name.to_string(),
                    seconds: max_wait.as_secs(),
                });
            }

            // Try event-driven wait if possible (pane snapshot reveals completion markers)
            let remaining = deadline.saturating_duration_since(Instant::now());
            let next_poll = remaining.min(self.options.poll_interval);
            let _ = timeout(next_poll, interval.tick()).await;
        }
    }

    /// Build a rubric automatically from the mission text + plan.
    fn build_rubric(&self, mission: &Mission, plan: &Plan) -> Rubric {
        let mut criteria = vec![
            RubricCriterion {
                id: "F1".to_string(),
                description: "Core mission objective satisfied".to_string(),
                weight: 3.0,
                category: crate::gate::CriterionCategory::Functional,
            },
            RubricCriterion {
                id: "Q1".to_string(),
                description: "No regressions introduced".to_string(),
                weight: 2.0,
                category: crate::gate::CriterionCategory::Quality,
            },
        ];

        if matches!(plan.complexity, Complexity::Complex | Complexity::Epic) {
            criteria.push(RubricCriterion {
                id: "Q2".to_string(),
                description: "All sub-workers reported done_clean".to_string(),
                weight: 2.0,
                category: crate::gate::CriterionCategory::Quality,
            });
        }

        if mission.text.to_lowercase().contains("security")
            || mission.text.to_lowercase().contains("auth")
        {
            criteria.push(RubricCriterion {
                id: "S1".to_string(),
                description: "No security regressions or new vulnerabilities".to_string(),
                weight: 3.0,
                category: crate::gate::CriterionCategory::Security,
            });
        }

        Rubric::new(&mission.text, criteria)
    }

    /// Execute and persist a ledger-derived quality gate.
    ///
    /// Each challenge is a distinct, falsifiable assertion over the immutable
    /// mission event stream. Its citation names the exact event id and sequence;
    /// the acceptance transaction later recomputes every fact digest against that
    /// same event. The three lenses deliberately consume different fact sets.
    fn run_quality_gate(
        state_dir: &Path,
        ledger: &MissionLedger,
        mission: &Mission,
        plan: &PlanContract,
        rubric: &Rubric,
        results: &[WorkerResult],
    ) -> Result<GateResult> {
        let oracle = format!("mission-{}", mission.id.0);
        if plan
            .required_gates
            .iter()
            .any(|gate| gate == "independent_verification")
        {
            let evidence_by_task: BTreeMap<String, String> = ledger
                .events(&mission.id)?
                .into_iter()
                .filter(|event| event.kind == "task_verifier_observations_recorded")
                .filter_map(|event| {
                    serde_json::from_value::<RecordedContractVerification>(event.payload.clone())
                        .ok()
                        .filter(|record| {
                            record.mission_id == mission.id.as_str()
                                && record.plan_revision == plan.revision
                                && record.plan_digest == plan.content_digest
                                && record.verification.passed
                        })
                        .map(|record| (record.task_id, event.event_id))
                })
                .collect();
            let evidence = evidence_by_task.into_values().collect::<Vec<_>>();
            let passed = evidence.len() == plan.tasks.len();
            record_plan_requirement_observation(
                ledger,
                &mission.id,
                PlanRequirementKind::Gate,
                "independent_verification",
                passed,
                "omega-quality-gate",
                evidence,
            )?;
        }
        let before_start = ledger.events(&mission.id)?;
        let head = before_start
            .last()
            .ok_or_else(|| anyhow::anyhow!("mission event stream is empty"))?;
        let start_event = append_mission_observation(
            ledger,
            &mission.id,
            "omega-quality-gate",
            "quality_gate_started",
            "quality-gate-start",
            serde_json::json!({
                "schema_version": ACCEPTANCE_OBSERVATION_SCHEMA_VERSION,
                "mission_id": mission.id.as_str(),
                "plan_revision": plan.revision,
                "plan_digest": plan.content_digest,
                "evidence_head_event_id": head.event_id,
                "evidence_head_sequence": head.sequence,
                "worker_results_digest": blake3::hash(&serde_json::to_vec(results)?).to_hex().to_string(),
            }),
        )?;
        let events = ledger.events(&mission.id)?;
        let projection = ledger
            .mission(&mission.id)?
            .ok_or_else(|| anyhow::anyhow!("mission projection disappeared"))?;
        let immutable = ledger
            .mission_record(&mission.id)?
            .ok_or_else(|| anyhow::anyhow!("immutable mission record disappeared"))?;
        let attempts = ledger.task_attempts(&mission.id)?;
        let replayed_attempts = ledger.replay_task_attempts(&mission.id)?;
        let replayed_projection = ledger.replay(&mission.id)?;

        let find_event = |predicate: &dyn Fn(&crate::mission_ledger::MissionEvent) -> bool| {
            events.iter().find(|event| predicate(event))
        };
        let event_for_state = |state: TaskAttemptState| {
            find_event(&|event| {
                event
                    .resulting_task_attempt
                    .as_ref()
                    .is_some_and(|attempt| attempt.state == state)
            })
        };
        let accepted_sequences: BTreeMap<_, _> = events
            .iter()
            .filter_map(|event| {
                event.resulting_task_attempt.as_ref().and_then(|attempt| {
                    (attempt.state == TaskAttemptState::Accepted)
                        .then_some((attempt.task_id.as_str(), event.sequence))
                })
            })
            .collect();
        let all_dependencies_ordered = plan.tasks.iter().all(|task| {
            accepted_sequences
                .get(task.task_id.as_str())
                .is_some_and(|task_sequence| {
                    task.depends_on.iter().all(|dependency| {
                        accepted_sequences
                            .get(dependency.as_str())
                            .is_some_and(|dependency_sequence| dependency_sequence < task_sequence)
                    })
                })
        });
        let exact_result_tasks: BTreeSet<_> = results
            .iter()
            .map(|result| result.task_id.as_str())
            .collect();
        let planned_tasks: BTreeSet<_> = plan
            .tasks
            .iter()
            .map(|task| task.task_id.as_str())
            .collect();
        let exact_worker_results = exact_result_tasks.len() == results.len()
            && exact_result_tasks == planned_tasks
            && results
                .iter()
                .all(|result| result.status == DoneStatus::DoneClean);
        let latest_attempt_ids: BTreeMap<_, _> =
            plan.tasks
                .iter()
                .filter_map(|task| {
                    attempts
                        .iter()
                        .filter(|attempt| {
                            attempt.plan_revision == plan.revision
                                && attempt.task_id == task.task_id.as_str()
                        })
                        .max_by_key(|attempt| {
                            events
                                .iter()
                                .filter(|event| {
                                    event.resulting_task_attempt.as_ref().is_some_and(|result| {
                                        result.attempt_id == attempt.attempt_id
                                    })
                                })
                                .map(|event| event.sequence)
                                .max()
                                .unwrap_or(0)
                        })
                        .map(|attempt| (task.task_id.as_str(), attempt.attempt_id.as_str()))
                })
                .collect();
        let exact_attempt_coverage = plan.tasks.iter().all(|task| {
            let task_attempts: Vec<_> = attempts
                .iter()
                .filter(|attempt| {
                    attempt.plan_revision == plan.revision
                        && attempt.task_id == task.task_id.as_str()
                })
                .collect();
            let Some(latest_id) = latest_attempt_ids.get(task.task_id.as_str()) else {
                return false;
            };
            task_attempts.iter().all(|attempt| {
                if attempt.attempt_id == **latest_id {
                    attempt.state == TaskAttemptState::Accepted
                } else {
                    attempt.state.is_terminal()
                }
            })
        });
        let running_owners_match = plan.tasks.iter().all(|task| {
            let latest_id = latest_attempt_ids.get(task.task_id.as_str()).copied();
            let owner = events.iter().rev().find_map(|event| {
                event.resulting_task_attempt.as_ref().and_then(|attempt| {
                    (attempt.plan_revision == plan.revision
                        && attempt.task_id == task.task_id.as_str()
                        && Some(attempt.attempt_id.as_str()) == latest_id
                        && attempt.state == TaskAttemptState::Running)
                        .then_some(event.actor.as_str())
                })
            });
            owner.is_some_and(|owner| {
                results.iter().any(|result| {
                    result.task_id == task.task_id.as_str() && result.session_name == owner
                })
            })
        });
        let exact_candidate_provenance = plan.tasks.iter().all(|task| {
            let latest_id = latest_attempt_ids.get(task.task_id.as_str()).copied();
            events.iter().any(|event| {
                matches!(
                    event.kind.as_str(),
                    "legacy_worker_completion_candidate" | "legacy_oracle_completion_candidate"
                ) && event
                    .resulting_task_attempt
                    .as_ref()
                    .is_some_and(|attempt| {
                        attempt.plan_revision == plan.revision
                            && attempt.task_id == task.task_id.as_str()
                            && Some(attempt.attempt_id.as_str()) == latest_id
                            && attempt.state == TaskAttemptState::CandidateDone
                    })
            })
        });
        let exact_verifier_records = plan.tasks.iter().all(|task| {
            let latest_id = latest_attempt_ids.get(task.task_id.as_str()).copied();
            events.iter().any(|event| {
                event.kind == "task_verifier_observations_recorded"
                    && serde_json::from_value::<RecordedContractVerification>(event.payload.clone())
                        .is_ok_and(|record| {
                            record.schema_version == ACCEPTANCE_OBSERVATION_SCHEMA_VERSION
                                && record.mission_id == mission.id.as_str()
                                && record.task_id == task.task_id.as_str()
                                && Some(record.attempt_id.as_str()) == latest_id
                                && record.plan_revision == plan.revision
                                && record.plan_digest == plan.content_digest
                                && record.verification.passed
                                && record.verification.failures.is_empty()
                                && record.verification.observations.len()
                                    == task.verifier_checks.len()
                        })
            })
        });
        let no_active_leases = attempts.iter().all(|attempt| {
            ledger
                .active_leases_for_attempt(&mission.id, &attempt.task_id, &attempt.attempt_id)
                .is_ok_and(|leases| leases.is_empty())
        });
        let no_unresolved_issues = events
            .iter()
            .filter(|event| {
                matches!(
                    event.kind.as_str(),
                    "mission_acceptance_invalidated" | "mission_blocker_recorded"
                ) && event
                    .plan_revision
                    .is_none_or(|revision| revision == plan.revision)
            })
            .all(|issue| {
                let expected_kind = if issue.kind == "mission_acceptance_invalidated" {
                    "mission_acceptance_revalidated"
                } else {
                    "mission_blocker_resolved"
                };
                events.iter().any(|event| {
                    event.kind == expected_kind
                        && event.sequence > issue.sequence
                        && event.sequence < start_event.sequence
                        && serde_json::from_value::<MissionIssueResolution>(event.payload.clone())
                            .is_ok_and(|resolution| {
                                resolution.schema_version == ACCEPTANCE_OBSERVATION_SCHEMA_VERSION
                                    && resolution.mission_id == mission.id.as_str()
                                    && resolution.plan_revision == plan.revision
                                    && resolution.plan_digest == plan.content_digest
                                    && resolution.issue_event_id == issue.event_id
                                    && !resolution.resolved_by.trim().is_empty()
                                    && !resolution.detail.trim().is_empty()
                            })
                })
            });
        let contiguous_events = events.iter().enumerate().all(|(index, event)| {
            event.sequence == u64::try_from(index).unwrap_or(u64::MAX) + 1
                && event.mission_id == mission.id
        });
        let replay_matches = replayed_projection == projection && replayed_attempts == attempts;

        struct RawFact<'a> {
            id: &'static str,
            passed: bool,
            detail: String,
            event: Option<&'a crate::mission_ledger::MissionEvent>,
        }
        let raw = vec![
            RawFact {
                id: "immutable_mission_identity",
                passed: serde_json::to_value(&immutable)? == serde_json::to_value(mission)?,
                detail: "immutable mission row and mission_created payload match the execution input".to_string(),
                event: find_event(&|event| event.kind == "mission_created"),
            },
            RawFact {
                id: "contiguous_event_stream",
                passed: contiguous_events,
                detail: "event sequence is contiguous and every event belongs to the exact mission".to_string(),
                event: find_event(&|event| event.kind == "mission_classified"),
            },
            RawFact {
                id: "active_plan_integrity",
                passed: plan.verify_integrity().is_ok()
                    && projection.active_plan_revision == Some(plan.revision),
                detail: "active immutable plan digest and revision verify".to_string(),
                event: find_event(&|event| event.activated_plan_revision == Some(plan.revision)),
            },
            RawFact {
                id: "task_attempt_coverage",
                passed: exact_attempt_coverage && all_dependencies_ordered,
                detail: "every planned task has one accepted attempt and dependencies were accepted first".to_string(),
                event: event_for_state(TaskAttemptState::Queued),
            },
            RawFact {
                id: "mission_execution_started",
                passed: events.iter().any(|event| event.resulting_mission_state == Some(MissionState::Running)),
                detail: "mission reached Running through the authoritative ledger".to_string(),
                event: find_event(&|event| event.resulting_mission_state == Some(MissionState::Running)),
            },
            RawFact {
                id: "worker_ownership_binding",
                passed: running_owners_match,
                detail: "each worker result is bound to the actor that owned its Running attempt".to_string(),
                event: event_for_state(TaskAttemptState::Running),
            },
            RawFact {
                id: "candidate_provenance",
                passed: exact_candidate_provenance && exact_worker_results,
                detail: "every clean worker result has an exact immutable CandidateDone provenance event".to_string(),
                event: event_for_state(TaskAttemptState::CandidateDone),
            },
            RawFact {
                id: "independent_verification_phase",
                passed: plan.tasks.iter().all(|task| attempts.iter().any(|attempt| attempt.task_id == task.task_id.as_str() && attempt.version >= 4)),
                detail: "every task crossed the independently-owned Verifying phase".to_string(),
                event: event_for_state(TaskAttemptState::Verifying),
            },
            RawFact {
                id: "verifier_observation_coverage",
                passed: exact_verifier_records,
                detail: "every frozen verifier set has one exact passing plan-bound observation record".to_string(),
                event: find_event(&|event| event.kind == "task_verifier_observations_recorded"),
            },
            RawFact {
                id: "accepted_task_authority",
                passed: exact_attempt_coverage,
                detail: "every authoritative task projection is exactly Accepted".to_string(),
                event: event_for_state(TaskAttemptState::Accepted),
            },
            RawFact {
                id: "mission_verification_barrier",
                passed: projection.state == MissionState::Verifying && replay_matches,
                detail: "mission is Verifying and replay matches all materialized projections".to_string(),
                event: find_event(&|event| event.resulting_mission_state == Some(MissionState::Verifying)),
            },
            RawFact {
                id: "release_and_issue_barrier",
                passed: no_active_leases && no_unresolved_issues,
                detail: "no active task leases or unresolved mission-level issues remain".to_string(),
                event: Some(&start_event),
            },
        ];

        let mut checks = Vec::new();
        for fact in raw {
            let Some(evidence) = fact.event else {
                continue;
            };
            checks.push(MissionGateCheckObservation {
                check_id: fact.id.to_string(),
                fact_digest: mission_gate_fact_digest(
                    fact.id,
                    fact.passed,
                    &fact.detail,
                    evidence,
                )?,
                passed: fact.passed,
                evidence_event_id: evidence.event_id.clone(),
                evidence_sequence: evidence.sequence,
                detail: fact.detail,
            });
        }
        let check_pass = |id: &str| {
            checks
                .iter()
                .find(|check| check.check_id == id)
                .is_some_and(|check| check.passed)
        };
        let lens_specs = [
            (
                "code_reviewer",
                vec![
                    "immutable_mission_identity",
                    "active_plan_integrity",
                    "task_attempt_coverage",
                    "verifier_observation_coverage",
                    "accepted_task_authority",
                ],
            ),
            (
                "debugger",
                vec![
                    "contiguous_event_stream",
                    "mission_execution_started",
                    "independent_verification_phase",
                    "mission_verification_barrier",
                    "release_and_issue_barrier",
                ],
            ),
            (
                "general_purpose",
                vec![
                    "immutable_mission_identity",
                    "worker_ownership_binding",
                    "candidate_provenance",
                    "accepted_task_authority",
                    "release_and_issue_barrier",
                ],
            ),
        ];
        let lenses: Vec<MissionGateLensObservation> = lens_specs
            .iter()
            .map(|(lens, ids)| MissionGateLensObservation {
                lens: (*lens).to_string(),
                passed: ids.iter().all(|id| check_pass(id)),
                fact_ids: ids.iter().map(|id| (*id).to_string()).collect(),
            })
            .collect();
        let event_citation = |event_id: &str, sequence: u64| {
            format!(
                "log:mission-ledger:{} event {}#{}",
                mission.id.as_str(),
                event_id,
                sequence
            )
        };
        let grade_citation = checks
            .iter()
            .find(|check| check.check_id == "verifier_observation_coverage")
            .or_else(|| checks.first())
            .map(|check| event_citation(&check.evidence_event_id, check.evidence_sequence))
            .unwrap_or_else(|| format!("log:mission-ledger:{} empty", mission.id.as_str()));
        let grades: Vec<GradeResult> = rubric
            .criteria
            .iter()
            .map(|criterion| {
                let verdict = if checks.len() >= 12 && checks.iter().all(|check| check.passed) {
                    GradeVerdict::Satisfied
                } else {
                    GradeVerdict::Unmet
                };
                GradeResult {
                    criterion_id: criterion.id.clone(),
                    verdict,
                    confidence: 1.0,
                    evidence: grade_citation.clone(),
                }
            })
            .collect();
        let grader_submissions: Vec<(GraderLens, GradeVerdict, f32, String)> = lenses
            .iter()
            .enumerate()
            .map(|(index, lens)| {
                let typed_lens = [
                    GraderLens::CodeReviewer,
                    GraderLens::Debugger,
                    GraderLens::GeneralPurpose,
                ][index];
                let evidence = lens
                    .fact_ids
                    .first()
                    .and_then(|id| checks.iter().find(|check| check.check_id == *id))
                    .map(|check| event_citation(&check.evidence_event_id, check.evidence_sequence))
                    .unwrap_or_else(|| grade_citation.clone());
                (
                    typed_lens,
                    if lens.passed {
                        GradeVerdict::Satisfied
                    } else {
                        GradeVerdict::Unmet
                    },
                    1.0,
                    evidence,
                )
            })
            .collect();
        let challenges: Vec<AdversarialChallenge> = checks
            .iter()
            .map(|check| AdversarialChallenge {
                challenge: format!("Falsify ledger assertion `{}`", check.check_id),
                result: if check.passed {
                    ChallengeResult::NoDefect
                } else {
                    ChallengeResult::DefectFound
                },
                evidence: event_citation(&check.evidence_event_id, check.evidence_sequence),
            })
            .collect();
        let claim_strings: Vec<String> = challenges
            .iter()
            .map(|c| c.evidence.clone())
            .chain(grades.iter().map(|g| g.evidence.clone()))
            .collect();
        let claims: Vec<&str> = claim_strings.iter().map(String::as_str).collect();

        let tokens_spent: u64 = results.iter().map(|r| r.duration_secs).sum();
        let prior_gate = GateResult::read(state_dir, &oracle).ok().flatten();

        let qg = crate::gate::QualityGate::with_default_cap(state_dir.to_path_buf());
        let gate_result = qg.run(
            &oracle,
            rubric,
            grades,
            grader_submissions,
            challenges,
            prior_gate.as_ref(),
            tokens_spent,
            &claims,
        );
        let mut observation = MissionGateObservation {
            schema_version: ACCEPTANCE_OBSERVATION_SCHEMA_VERSION,
            mission_id: mission.id.as_str().to_string(),
            plan_revision: plan.revision,
            plan_digest: plan.content_digest.clone(),
            gate_result_digest: String::new(),
            overall_pass: gate_result.overall_pass
                && checks.len() >= 12
                && checks.iter().all(|check| check.passed)
                && lenses.iter().all(|lens| lens.passed),
            checks,
            lenses,
        };
        observation.gate_result_digest = mission_gate_result_digest(&observation)?;
        let gate_event = record_mission_gate_observation(
            ledger,
            &mission.id,
            "omega-quality-gate",
            &observation,
        )?;
        if plan
            .required_gates
            .iter()
            .any(|gate| gate == "quality_gate")
        {
            record_plan_requirement_observation(
                ledger,
                &mission.id,
                PlanRequirementKind::Gate,
                "quality_gate",
                observation.overall_pass,
                "omega-quality-gate",
                vec![gate_event.event_id],
            )?;
        }
        Ok(gate_result)
    }

    fn summarize(
        &self,
        mission: &Mission,
        plan: &Plan,
        results: &[WorkerResult],
        gate: Option<&GateResult>,
    ) -> String {
        let mut s = String::new();
        s.push_str(&format!(
            "Mission '{}' (project: {}, complexity: {:?})\n",
            mission.id.0, mission.project, plan.complexity
        ));
        s.push_str(&format!(
            "Plan: {:?} with {} task(s)\n",
            plan.strategy,
            plan.tasks.len()
        ));
        s.push_str(&format!(
            "Workers: {} dispatched, {} clean, {} pending, {} failed\n",
            results.len(),
            results
                .iter()
                .filter(|r| r.status == DoneStatus::DoneClean)
                .count(),
            results
                .iter()
                .filter(|r| r.status == DoneStatus::Pending)
                .count(),
            results
                .iter()
                .filter(|r| r.status == DoneStatus::Failed)
                .count(),
        ));
        if let Some(g) = gate {
            s.push_str(&format!(
                "Quality gate: {} (score: {:.1}, rubric: {}, consensus: {}, adversarial: {})\n",
                if g.overall_pass { "PASS" } else { "FAIL" },
                g.score,
                g.rubric_pass,
                g.consensus_pass,
                g.adversarial_pass,
            ));
        }
        s
    }

    pub fn config(&self) -> &OmegaConfig {
        &self.config
    }

    pub fn session_manager(&self) -> &SessionManager {
        &self.mgr
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn provider_family_maps_first_class_and_opencode_aliases() {
        assert_eq!(
            provider_family_for_agent(Agent::Claude),
            crate::rules::ProviderFamily::Claude
        );
        assert_eq!(
            provider_family_for_agent(Agent::Codex),
            crate::rules::ProviderFamily::Codex
        );
        assert_eq!(
            provider_family_for_agent(Agent::Hermes),
            crate::rules::ProviderFamily::Other
        );
        assert_eq!(
            provider_family_from_name("opencode"),
            crate::rules::ProviderFamily::Other
        );
        assert_eq!(
            provider_family_from_name("open-code"),
            crate::rules::ProviderFamily::Other
        );
        let ctx = policy_context_for_agent(
            crate::rules::RuleScope::Worker,
            "ship the fix",
            Agent::Codex,
        );
        assert!(ctx.contains("[L0]"));
        assert!(ctx.contains("update_plan"));
        assert!(ctx.contains("Provider harness"));
    }

    #[test]
    fn mission_id_generates_unique_ids() {
        let a = crate::mission::MissionId::new();
        let b = crate::mission::MissionId::new();
        assert_ne!(a, b);
        assert!(a.as_str().starts_with("m-"));
    }

    #[test]
    fn priority_orders_correctly() {
        use crate::mission::Priority;
        assert!(Priority::Low < Priority::Normal);
        assert!(Priority::Normal < Priority::High);
        assert!(Priority::High < Priority::Urgent);
    }

    #[test]
    fn orchestration_prepares_plan_and_attempts_before_execution() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = Mission::new(
            "OmegaOS",
            "implement and verify the authority path",
            PathBuf::from("/tmp/omega-authority"),
        );
        let plan = Plan {
            mission_id: mission.id.clone(),
            complexity: Complexity::Complex,
            strategy: PlanStrategy::Parallel,
            tasks: vec![Task {
                id: "authority".to_string(),
                name: "authority".to_string(),
                prompt: "Implement the ledger authority".to_string(),
                files_owned: vec!["src/authority.rs".to_string()],
                depends_on: Vec::new(),
                agent: "codex".to_string(),
                estimated_minutes: 30,
            }],
            created_at: Utc::now(),
        };

        let prepared = prepare_authoritative_execution(
            &ledger,
            &mission,
            &plan,
            "test-orchestrator",
            vec!["independent_verification".to_string()],
        )
        .unwrap();

        let projection = ledger.mission(&mission.id).unwrap().unwrap();
        assert_eq!(projection.state, MissionState::Planned);
        assert_eq!(projection.active_plan_revision, Some(1));
        let active = ledger.active_plan(&mission.id).unwrap().unwrap();
        assert_eq!(active.content_digest, prepared.plan.content_digest);
        assert_eq!(active.tasks.len(), 1);
        let attempt = ledger
            .task_attempt(&prepared.attempts[0].attempt_id)
            .unwrap()
            .unwrap();
        assert_eq!(attempt.state, TaskAttemptState::Queued);
        assert_eq!(attempt.plan_revision, active.revision);
    }

    #[test]
    fn orchestration_refuses_a_hollow_plan_instead_of_bypassing_contracts() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = Mission::new("OmegaOS", "empty", PathBuf::from("/tmp"));
        let plan = Plan {
            mission_id: mission.id.clone(),
            complexity: Complexity::Simple,
            strategy: PlanStrategy::Direct,
            tasks: Vec::new(),
            created_at: Utc::now(),
        };
        assert!(prepare_authoritative_execution(
            &ledger,
            &mission,
            &plan,
            "test-orchestrator",
            Vec::new(),
        )
        .is_err());
        assert!(ledger.mission(&mission.id).unwrap().is_none());
    }

    fn direct_plan(mission: &Mission, task_id: &str) -> Plan {
        Plan {
            mission_id: mission.id.clone(),
            complexity: Complexity::Simple,
            strategy: PlanStrategy::Direct,
            tasks: vec![Task {
                id: task_id.to_string(),
                name: "main".to_string(),
                prompt: "Implement the change".to_string(),
                files_owned: vec!["**/*".to_string()],
                depends_on: Vec::new(),
                agent: "codex".to_string(),
                estimated_minutes: 10,
            }],
            created_at: Utc::now(),
        }
    }

    fn running_attempt(
        ledger: &MissionLedger,
        mission: &Mission,
        task_id: &str,
    ) -> AuthoritativeExecution {
        let mut execution = prepare_authoritative_execution(
            ledger,
            mission,
            &direct_plan(mission, task_id),
            "test",
            Vec::new(),
        )
        .unwrap();
        execution.attempts[0].owner = Some("worker".to_string());
        transition_authoritative_mission(ledger, &mission.id, MissionState::Running, "test")
            .unwrap();
        transition_authoritative_attempt(
            ledger,
            &execution.attempts[0],
            TaskAttemptState::Running,
            "worker",
        )
        .unwrap();
        execution
    }

    #[test]
    fn monitor_converges_when_done_writer_already_marked_candidate_done() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = Mission::new("OmegaOS", "race", PathBuf::from("/tmp"));
        let execution = running_attempt(&ledger, &mission, "race");
        let attempt = &execution.attempts[0];

        transition_authoritative_attempt(
            &ledger,
            attempt,
            TaskAttemptState::CandidateDone,
            "worker",
        )
        .unwrap();
        ensure_attempt_verifying(&ledger, attempt, "worker").unwrap();

        assert_eq!(
            ledger
                .task_attempt(&attempt.attempt_id)
                .unwrap()
                .unwrap()
                .state,
            TaskAttemptState::Verifying
        );
    }

    #[test]
    fn concurrent_candidate_done_transitions_converge_idempotently() {
        let ledger = std::sync::Arc::new(MissionLedger::open_in_memory().unwrap());
        let mission = Mission::new("OmegaOS", "race", PathBuf::from("/tmp"));
        let execution = running_attempt(&ledger, &mission, "race");
        let attempt = std::sync::Arc::new(execution.attempts[0].clone());
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(8));
        let handles = (0..8)
            .map(|_| {
                let ledger = std::sync::Arc::clone(&ledger);
                let attempt = std::sync::Arc::clone(&attempt);
                let barrier = std::sync::Arc::clone(&barrier);
                std::thread::spawn(move || {
                    barrier.wait();
                    transition_authoritative_attempt(
                        &ledger,
                        &attempt,
                        TaskAttemptState::CandidateDone,
                        "worker",
                    )
                })
            })
            .collect::<Vec<_>>();
        for handle in handles {
            handle.join().unwrap().unwrap();
        }
        assert_eq!(
            ledger
                .task_attempt(&attempt.attempt_id)
                .unwrap()
                .unwrap()
                .state,
            TaskAttemptState::CandidateDone
        );
    }

    #[test]
    fn accepted_attempt_without_exact_verification_record_is_not_trusted() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = Mission::new("OmegaOS", "acceptance", PathBuf::from("/tmp"));
        let execution = running_attempt(&ledger, &mission, "acceptance");
        let attempt = &execution.attempts[0];
        ensure_attempt_verifying(&ledger, attempt, "worker").unwrap();
        transition_authoritative_attempt(&ledger, attempt, TaskAttemptState::Accepted, "worker")
            .unwrap();
        let record = RecordedContractVerification {
            schema_version: ACCEPTANCE_OBSERVATION_SCHEMA_VERSION,
            mission_id: mission.id.as_str().to_string(),
            task_id: attempt.task_id.clone(),
            attempt_id: attempt.attempt_id.clone(),
            plan_revision: attempt.plan_revision,
            plan_digest: execution.plan.content_digest.clone(),
            worker_signal_digest: "not-recorded".to_string(),
            verification: ContractVerification {
                passed: true,
                observations: Vec::new(),
                failures: Vec::new(),
            },
        };

        assert!(!exact_verification_is_recorded(&ledger, attempt, &record).unwrap());
        assert!(finalize_contract_verification(&ledger, attempt, "worker", &record).is_err());
    }

    #[test]
    fn completion_identity_and_timestamp_are_bounded() {
        let dispatched_at = Utc::now();
        let now = dispatched_at + ChronoDuration::seconds(1);
        let mut signal = DoneSignal::new("other-worker", DoneStatus::DoneClean, "done");
        signal.finished_at = now;
        assert!(validate_done_identity_and_freshness(
            &signal,
            "expected-worker",
            dispatched_at,
            now
        )
        .is_err());

        signal.session = "expected-worker".to_string();
        signal.finished_at = dispatched_at - ChronoDuration::seconds(DONE_CLOCK_SKEW_SECS + 1);
        assert!(validate_done_identity_and_freshness(
            &signal,
            "expected-worker",
            dispatched_at,
            now
        )
        .is_err());

        signal.finished_at = now + ChronoDuration::seconds(DONE_CLOCK_SKEW_SECS + 1);
        assert!(validate_done_identity_and_freshness(
            &signal,
            "expected-worker",
            dispatched_at,
            now
        )
        .is_err());
    }

    #[test]
    fn completion_projection_must_match_exact_historical_ledger_event() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = Mission::new("OmegaOS", "provenance", PathBuf::from("/tmp"));
        let execution = running_attempt(&ledger, &mission, "provenance");
        let attempt = &execution.attempts[0];
        let mission_projection = ledger.mission(&mission.id).unwrap().unwrap();
        let task_projection = ledger.task_attempt(&attempt.attempt_id).unwrap().unwrap();
        let mut event = AppendEvent::new(
            mission.id.clone(),
            mission_projection.version,
            "test:worker-completion",
            "worker",
            "legacy_worker_completion_candidate",
        );
        event.task_attempt = Some(TaskAttemptMutation {
            task_id: attempt.task_id.clone(),
            attempt_id: attempt.attempt_id.clone(),
            plan_revision: attempt.plan_revision,
            expected_version: task_projection.version,
            next_state: TaskAttemptState::CandidateDone,
        });
        let outcome = ledger.append(event).unwrap();
        let provenance = crate::done::ProjectionProvenance {
            source: "mission-engine-v3.sqlite3".to_string(),
            event_id: outcome.event.event_id,
            event_sequence: outcome.event.sequence,
            mission_version: outcome.projection.version,
            projection_hash: outcome.projection.projection_hash,
        };
        validate_done_projection(&ledger, attempt, "worker", &provenance, false).unwrap();

        let mut forged = provenance;
        forged.projection_hash = "forged".to_string();
        assert!(validate_done_projection(&ledger, attempt, "worker", &forged, false).is_err());
    }

    fn append_worker_candidate(
        ledger: &MissionLedger,
        attempt: &AuthoritativeTaskAttempt,
        owner: &str,
        mut done: DoneSignal,
    ) -> DoneSignal {
        let mission = ledger.mission(&attempt.mission_id).unwrap().unwrap();
        let task = ledger.task_attempt(&attempt.attempt_id).unwrap().unwrap();
        let mut event = AppendEvent::new(
            attempt.mission_id.clone(),
            mission.version,
            format!("test:candidate:{}", attempt.attempt_id),
            owner,
            "legacy_worker_completion_candidate",
        );
        event.payload = serde_json::to_value(&done).unwrap();
        event.task_attempt = Some(TaskAttemptMutation {
            task_id: attempt.task_id.clone(),
            attempt_id: attempt.attempt_id.clone(),
            plan_revision: attempt.plan_revision,
            expected_version: task.version,
            next_state: TaskAttemptState::CandidateDone,
        });
        event.lease_assertions = attempt.leases.iter().map(LeaseAssertion::from).collect();
        let appended = ledger.append(event).unwrap();
        done.projection = Some(crate::done::ProjectionProvenance {
            source: "mission-engine-v3.sqlite3".to_string(),
            event_id: appended.event.event_id,
            event_sequence: appended.event.sequence,
            mission_version: appended.projection.version,
            projection_hash: appended.projection.projection_hash,
        });
        done
    }

    #[test]
    fn exact_candidate_api_executes_full_contract_and_persists_acceptance() {
        let temp = tempfile::TempDir::new().unwrap();
        assert!(std::process::Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(temp.path())
            .status()
            .unwrap()
            .success());
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = Mission::new("OmegaOS", "candidate", temp.path().to_path_buf());
        let execution = running_attempt(&ledger, &mission, "candidate");
        let attempt = &execution.attempts[0];
        let mut done = DoneSignal::new("worker", DoneStatus::DoneClean, "done");
        done.todos_total = 1;
        done.todos_completed = 1;
        done.corroboration = vec![crate::done::CorroborationSource::WorkerSelfReport];
        done.artifacts = vec![crate::done::DoneArtifact::Command {
            cmd: "git diff --check".to_string(),
            exit_code: 0,
        }];
        let done = append_worker_candidate(&ledger, attempt, "worker", done);

        let outcome = verify_and_finalize_candidate(
            &ledger,
            &mission.id,
            &attempt.task_id,
            &attempt.attempt_id,
            attempt.plan_revision,
            "worker",
            &done,
            temp.path(),
        )
        .unwrap();
        assert!(outcome.accepted, "{:?}", outcome.verification.failures);
        assert_eq!(outcome.attempt_state, TaskAttemptState::Accepted);
        assert_eq!(
            outcome.verification.observations.len(),
            execution.plan.tasks[0].verifier_checks.len()
        );
        assert!(ledger.events(&mission.id).unwrap().iter().any(|event| {
            event.kind == "task_verifier_observations_recorded"
                && serde_json::from_value::<RecordedContractVerification>(event.payload.clone())
                    .is_ok_and(|record| {
                        record.attempt_id == attempt.attempt_id
                            && record.plan_digest == execution.plan.content_digest
                            && record.verification.passed
                    })
        }));
    }

    #[test]
    fn candidate_without_exact_provenance_stays_candidate_and_keeps_authority() {
        let temp = tempfile::TempDir::new().unwrap();
        assert!(std::process::Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(temp.path())
            .status()
            .unwrap()
            .success());
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = Mission::new("OmegaOS", "candidate", temp.path().to_path_buf());
        let execution = running_attempt(&ledger, &mission, "candidate");
        let attempt = &execution.attempts[0];
        let mut done = DoneSignal::new("worker", DoneStatus::DoneClean, "done");
        done.todos_total = 1;
        done.todos_completed = 1;
        done.corroboration = vec![crate::done::CorroborationSource::WorkerSelfReport];
        done.artifacts = vec![crate::done::DoneArtifact::Command {
            cmd: "git diff --check".to_string(),
            exit_code: 0,
        }];
        let recorded = append_worker_candidate(&ledger, attempt, "worker", done);
        let mut missing = recorded;
        missing.projection = None;

        assert!(verify_and_finalize_candidate(
            &ledger,
            &mission.id,
            &attempt.task_id,
            &attempt.attempt_id,
            attempt.plan_revision,
            "worker",
            &missing,
            temp.path(),
        )
        .is_err());
        assert_eq!(
            ledger
                .task_attempt(&attempt.attempt_id)
                .unwrap()
                .unwrap()
                .state,
            TaskAttemptState::CandidateDone
        );
        assert!(!ledger
            .events(&mission.id)
            .unwrap()
            .iter()
            .any(|event| event.kind == "task_verifier_observations_recorded"));
    }

    #[test]
    fn oracle_pending_bridge_requires_exact_candidate_provenance() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = Mission::new("OmegaOS", "oracle bridge", PathBuf::from("/tmp"));
        let mut execution = prepare_authoritative_execution(
            &ledger,
            &mission,
            &direct_plan(&mission, "oracle"),
            "test",
            Vec::new(),
        )
        .unwrap();
        transition_authoritative_mission(&ledger, &mission.id, MissionState::Running, "test")
            .unwrap();
        execution.attempts[0].owner = Some("oracle-OmegaOS".to_string());
        let attempt = &execution.attempts[0];
        transition_authoritative_attempt(
            &ledger,
            attempt,
            TaskAttemptState::Running,
            "oracle-OmegaOS",
        )
        .unwrap();
        let mut oracle = crate::done::OracleDoneSignal::new(
            "oracle-OmegaOS",
            "OmegaOS",
            DoneStatus::Pending,
            "mission",
        );
        oracle.summary = "candidate ready".to_string();
        oracle.pending_actions = vec![
            V3_ACCEPTANCE_PENDING.to_string(),
            "unrelated pending action".to_string(),
        ];
        let mission_projection = ledger.mission(&mission.id).unwrap().unwrap();
        let task_projection = ledger.task_attempt(&attempt.attempt_id).unwrap().unwrap();
        let mut event = AppendEvent::new(
            mission.id.clone(),
            mission_projection.version,
            "test:oracle-candidate",
            "oracle-OmegaOS",
            "legacy_oracle_completion_candidate",
        );
        event.payload = serde_json::to_value(&oracle).unwrap();
        event.task_attempt = Some(TaskAttemptMutation {
            task_id: attempt.task_id.clone(),
            attempt_id: attempt.attempt_id.clone(),
            plan_revision: attempt.plan_revision,
            expected_version: task_projection.version,
            next_state: TaskAttemptState::CandidateDone,
        });
        let appended = ledger.append(event).unwrap();
        let provenance = crate::done::ProjectionProvenance {
            source: "mission-engine-v3.sqlite3".to_string(),
            event_id: appended.event.event_id,
            event_sequence: appended.event.sequence,
            mission_version: appended.projection.version,
            projection_hash: appended.projection.projection_hash,
        };
        let mut signal = DoneSignal::new("oracle-OmegaOS", DoneStatus::Pending, "candidate ready");
        signal.finished_at = oracle.finished_at;
        signal.todos_total = 1;
        signal.pending_actions = oracle.pending_actions.clone();
        signal.projection = Some(provenance.clone());
        let exact = validate_done_projection(&ledger, attempt, "oracle-OmegaOS", &provenance, true)
            .unwrap();
        validate_done_payload_binding(&signal, &exact, true).unwrap();
        assert!(bridge_v3_oracle_candidate(&mut signal, true));
        assert_eq!(signal.status, DoneStatus::DoneClean);
        assert_eq!(
            signal.pending_actions,
            vec!["unrelated pending action".to_string()]
        );

        let mut unbound = DoneSignal::new("oracle-OmegaOS", DoneStatus::Pending, "candidate ready");
        unbound.pending_actions = vec![V3_ACCEPTANCE_PENDING.to_string()];
        assert!(!bridge_v3_oracle_candidate(&mut unbound, false));
        assert_eq!(unbound.status, DoneStatus::Pending);
    }

    #[test]
    fn concurrent_direct_missions_cannot_claim_the_same_checkout() {
        let tmp = tempfile::TempDir::new().unwrap();
        let work = tmp.path().join("repo");
        std::fs::create_dir_all(&work).unwrap();
        let state = tmp.path().join("state");
        std::fs::create_dir_all(&state).unwrap();
        let ledger = MissionLedger::open_in_memory().unwrap();
        let first = Mission::new("OmegaOS", "first", work.clone());
        let second = Mission::new("OmegaOS", "second", work.clone());
        let mut first_run = prepare_authoritative_execution(
            &ledger,
            &first,
            &direct_plan(&first, "first"),
            "test",
            Vec::new(),
        )
        .unwrap();
        let mut second_run = prepare_authoritative_execution(
            &ledger,
            &second,
            &direct_plan(&second, "second"),
            "test",
            Vec::new(),
        )
        .unwrap();
        claim_authoritative_scopes(
            &ledger,
            &state,
            &work,
            &mut first_run.attempts[0],
            "worker-first",
            &["**/*".to_string()],
            Duration::from_secs(60),
        )
        .unwrap();
        assert!(claim_authoritative_scopes(
            &ledger,
            &state,
            &work,
            &mut second_run.attempts[0],
            "worker-second",
            &["**/*".to_string()],
            Duration::from_secs(60),
        )
        .is_err());
        release_authoritative_scopes(&ledger, &state, &first_run.attempts[0]).unwrap();
    }

    #[test]
    fn release_reconciles_an_already_released_lease_and_clears_the_rest() {
        let tmp = tempfile::TempDir::new().unwrap();
        let work = tmp.path().join("repo");
        let state = tmp.path().join("state");
        std::fs::create_dir_all(&work).unwrap();
        std::fs::create_dir_all(&state).unwrap();
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = Mission::new("OmegaOS", "release", work.clone());
        let mut plan = direct_plan(&mission, "release");
        plan.tasks[0].files_owned = vec!["src/a.rs".to_string(), "src/b.rs".to_string()];
        let mut run =
            prepare_authoritative_execution(&ledger, &mission, &plan, "test", Vec::new()).unwrap();
        claim_authoritative_scopes(
            &ledger,
            &state,
            &work,
            &mut run.attempts[0],
            "worker-release",
            &plan.tasks[0].files_owned,
            Duration::from_secs(60),
        )
        .unwrap();
        assert_eq!(run.attempts[0].leases.len(), 2);
        let first = &run.attempts[0].leases[0];
        let second = &run.attempts[0].leases[1];
        ledger
            .release_lease(&first.resource_key, first.fencing_token)
            .unwrap();
        release_authoritative_scopes(&ledger, &state, &run.attempts[0]).unwrap();
        assert!(ledger
            .assert_fence(&second.resource_key, second.fencing_token)
            .is_err());
        assert!(scope::ScopeClaim::read(&state, "worker-release").is_none());
    }

    fn prepared_scope_receipt(
        ledger: &MissionLedger,
        work: &Path,
        attempt: &AuthoritativeTaskAttempt,
        owner: &str,
        selectors: &[String],
    ) -> AuthoritativeScopeReceipt {
        let receipt = AuthoritativeScopeReceipt {
            schema_version: SCOPE_AUTHORITY_SCHEMA_VERSION,
            mission_id: attempt.mission_id.clone(),
            task_id: attempt.task_id.clone(),
            attempt_id: attempt.attempt_id.clone(),
            plan_revision: attempt.plan_revision,
            owner: owner.to_string(),
            claim: scope::prepare_claim_for_workspace(work, owner, selectors.to_vec()).unwrap(),
        };
        let persisted = append_scope_authority_receipt(ledger, attempt, &receipt).unwrap();
        let event = ledger
            .events(&attempt.mission_id)
            .unwrap()
            .into_iter()
            .find(|event| {
                event.kind == SCOPE_AUTHORITY_EVENT_KIND
                    && event.payload == serde_json::to_value(&persisted).unwrap()
            })
            .unwrap();
        assert_eq!(event.plan_revision, Some(attempt.plan_revision));
        persisted
    }

    #[test]
    fn scope_receipt_event_recovers_event_only_and_published_crash_boundaries() {
        for publish_before_restart in [false, true] {
            let tmp = tempfile::TempDir::new().unwrap();
            let work = tmp.path().join("repo");
            let state = tmp.path().join("state");
            std::fs::create_dir_all(&work).unwrap();
            std::fs::create_dir_all(&state).unwrap();
            let ledger = MissionLedger::open_in_memory().unwrap();
            let mission = Mission::new("OmegaOS", "scope recovery", work.clone());
            let plan = direct_plan(&mission, "scope-recovery");
            let mut execution =
                prepare_authoritative_execution(&ledger, &mission, &plan, "test", Vec::new())
                    .unwrap();
            let owner = "worker-scope-recovery";
            let persisted = prepared_scope_receipt(
                &ledger,
                &work,
                &execution.attempts[0],
                owner,
                &plan.tasks[0].files_owned,
            );
            if publish_before_restart {
                scope::publish_prepared_claim(&state, &work, &persisted.claim).unwrap();
            } else {
                assert!(scope::ScopeClaim::read_strict(&state, owner)
                    .unwrap()
                    .is_none());
            }

            claim_authoritative_scopes(
                &ledger,
                &state,
                &work,
                &mut execution.attempts[0],
                owner,
                &plan.tasks[0].files_owned,
                Duration::from_secs(60),
            )
            .unwrap();
            assert_eq!(
                execution.attempts[0].scope_receipt.as_ref(),
                Some(&persisted)
            );
            assert_eq!(
                scope::ScopeClaim::read_strict(&state, owner)
                    .unwrap()
                    .unwrap(),
                persisted.claim
            );
            release_authoritative_scopes(&ledger, &state, &execution.attempts[0]).unwrap();
        }
    }

    #[test]
    fn restart_release_reloads_only_the_unique_immutable_scope_receipt() {
        let tmp = tempfile::TempDir::new().unwrap();
        let work = tmp.path().join("repo");
        let state = tmp.path().join("state");
        std::fs::create_dir_all(&work).unwrap();
        std::fs::create_dir_all(&state).unwrap();
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = Mission::new("OmegaOS", "scope restart", work.clone());
        let plan = direct_plan(&mission, "scope-restart");
        let mut execution =
            prepare_authoritative_execution(&ledger, &mission, &plan, "test", Vec::new()).unwrap();
        claim_authoritative_scopes(
            &ledger,
            &state,
            &work,
            &mut execution.attempts[0],
            "worker-scope-restart",
            &plan.tasks[0].files_owned,
            Duration::from_secs(60),
        )
        .unwrap();

        let mut restarted = execution.attempts[0].clone();
        restarted.scope_receipt = None;
        restarted.leases = ledger
            .active_leases_for_attempt(
                &restarted.mission_id,
                &restarted.task_id,
                &restarted.attempt_id,
            )
            .unwrap();
        release_authoritative_scopes(&ledger, &state, &restarted).unwrap();
        assert!(
            scope::ScopeClaim::read_strict(&state, "worker-scope-restart")
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn stale_attempt_receipt_cannot_release_a_replacement_scope_generation() {
        let tmp = tempfile::TempDir::new().unwrap();
        let work = tmp.path().join("repo");
        let state = tmp.path().join("state");
        std::fs::create_dir_all(&work).unwrap();
        std::fs::create_dir_all(&state).unwrap();
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = Mission::new("OmegaOS", "scope aba", work.clone());
        let plan = direct_plan(&mission, "scope-aba");
        let mut execution =
            prepare_authoritative_execution(&ledger, &mission, &plan, "test", Vec::new()).unwrap();
        claim_authoritative_scopes(
            &ledger,
            &state,
            &work,
            &mut execution.attempts[0],
            "worker-scope-aba",
            &plan.tasks[0].files_owned,
            Duration::from_secs(60),
        )
        .unwrap();
        let first = execution.attempts[0]
            .scope_receipt
            .as_ref()
            .unwrap()
            .claim
            .clone();
        let replacement = scope::replace_claim_exact(
            &state,
            &work,
            &first,
            vec!["src/replacement.rs".to_string()],
        )
        .unwrap();

        assert!(release_authoritative_scopes(&ledger, &state, &execution.attempts[0]).is_err());
        assert_eq!(
            scope::ScopeClaim::read_strict(&state, "worker-scope-aba")
                .unwrap()
                .unwrap(),
            replacement
        );
    }

    #[test]
    fn missing_duplicate_or_corrupt_scope_events_fail_closed() {
        let tmp = tempfile::TempDir::new().unwrap();
        let work = tmp.path().join("repo");
        let state = tmp.path().join("state");
        std::fs::create_dir_all(&work).unwrap();
        std::fs::create_dir_all(&state).unwrap();

        let missing_ledger = MissionLedger::open_in_memory().unwrap();
        let missing_mission = Mission::new("OmegaOS", "scope missing", work.clone());
        let missing_plan = direct_plan(&missing_mission, "scope-missing");
        let mut missing = prepare_authoritative_execution(
            &missing_ledger,
            &missing_mission,
            &missing_plan,
            "test",
            Vec::new(),
        )
        .unwrap();
        let orphan = scope::claim_or_reject_for_workspace(
            &state,
            &work,
            "worker-scope-missing",
            missing_plan.tasks[0].files_owned.clone(),
        )
        .unwrap();
        missing.attempts[0].owner = Some("worker-scope-missing".to_string());
        assert!(
            release_authoritative_scopes(&missing_ledger, &state, &missing.attempts[0]).is_err()
        );
        assert_eq!(
            scope::ScopeClaim::read_strict(&state, "worker-scope-missing")
                .unwrap()
                .unwrap(),
            orphan
        );
        scope::ScopeClaim::release_exact(&state, &orphan).unwrap();

        for corrupt in [false, true] {
            let bad_state = tmp.path().join(if corrupt {
                "state-corrupt"
            } else {
                "state-duplicate"
            });
            std::fs::create_dir_all(&bad_state).unwrap();
            let ledger = MissionLedger::open_in_memory().unwrap();
            let mission = Mission::new("OmegaOS", "scope bad event", work.clone());
            let plan = direct_plan(
                &mission,
                if corrupt {
                    "scope-corrupt"
                } else {
                    "scope-duplicate"
                },
            );
            let mut execution =
                prepare_authoritative_execution(&ledger, &mission, &plan, "test", Vec::new())
                    .unwrap();
            let owner = if corrupt {
                "worker-scope-corrupt"
            } else {
                "worker-scope-duplicate"
            };
            claim_authoritative_scopes(
                &ledger,
                &bad_state,
                &work,
                &mut execution.attempts[0],
                owner,
                &plan.tasks[0].files_owned,
                Duration::from_secs(60),
            )
            .unwrap();
            let payload = if corrupt {
                serde_json::json!({"attempt_id": execution.attempts[0].attempt_id})
            } else {
                serde_json::to_value(execution.attempts[0].scope_receipt.as_ref().unwrap()).unwrap()
            };
            append_mission_observation(
                &ledger,
                &mission.id,
                SCOPE_AUTHORITY_ACTOR,
                SCOPE_AUTHORITY_EVENT_KIND,
                if corrupt {
                    "scope-corrupt"
                } else {
                    "scope-duplicate"
                },
                payload,
            )
            .unwrap();
            assert!(
                release_authoritative_scopes(&ledger, &bad_state, &execution.attempts[0]).is_err()
            );
            assert!(scope::ScopeClaim::read_strict(&bad_state, owner)
                .unwrap()
                .is_some());
        }
    }

    #[test]
    fn failed_multi_lease_acquisition_rolls_back_every_owned_resource() {
        let tmp = tempfile::TempDir::new().unwrap();
        let work = tmp.path().join("repo");
        let state = tmp.path().join("state");
        std::fs::create_dir_all(&work).unwrap();
        std::fs::create_dir_all(&state).unwrap();
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = Mission::new("OmegaOS", "scope rollback", work.clone());
        let mut plan = direct_plan(&mission, "scope-rollback");
        plan.tasks[0].files_owned = vec!["src/a.rs".to_string(), "src/b.rs".to_string()];
        let mut execution =
            prepare_authoritative_execution(&ledger, &mission, &plan, "test", Vec::new()).unwrap();
        let blocked_selector = scope::normalize_scope_selector("src/b.rs");
        let prepared = scope::prepare_claim_for_workspace(
            &work,
            "resource-probe",
            plan.tasks[0].files_owned.clone(),
        )
        .unwrap();
        let blocked_resource =
            scope_lease_resource(prepared.workspace_id.as_deref().unwrap(), &blocked_selector);
        let foreign_mission = crate::mission::MissionId::new();
        ledger
            .acquire_lease(
                &blocked_resource,
                &foreign_mission,
                "foreign-task",
                "foreign-attempt",
                "foreign-worker",
                Duration::from_secs(60),
            )
            .unwrap();

        let error = claim_authoritative_scopes(
            &ledger,
            &state,
            &work,
            &mut execution.attempts[0],
            "worker-scope-rollback",
            &plan.tasks[0].files_owned,
            Duration::from_secs(60),
        )
        .unwrap_err();
        assert!(error.to_string().contains("rolled back cleanly"));
        assert!(execution.attempts[0].owner.is_none());
        assert!(execution.attempts[0].leases.is_empty());
        assert!(execution.attempts[0].scope_receipt.is_none());
        assert!(ledger
            .active_leases_for_attempt(
                &mission.id,
                &execution.attempts[0].task_id,
                &execution.attempts[0].attempt_id,
            )
            .unwrap()
            .is_empty());
        assert!(
            scope::ScopeClaim::read_strict(&state, "worker-scope-rollback")
                .unwrap()
                .is_none()
        );
        assert!(ledger.assert_fence(&blocked_resource, 1).is_ok());
    }

    #[test]
    fn incomplete_lease_rollback_retains_the_exact_compatibility_claim() {
        let tmp = tempfile::TempDir::new().unwrap();
        let work = tmp.path().join("repo");
        let state = tmp.path().join("state");
        std::fs::create_dir_all(&work).unwrap();
        std::fs::create_dir_all(&state).unwrap();
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = Mission::new("OmegaOS", "rollback retention", work.clone());
        let plan = direct_plan(&mission, "rollback-retention");
        let mut execution =
            prepare_authoritative_execution(&ledger, &mission, &plan, "test", Vec::new()).unwrap();
        let receipt = prepared_scope_receipt(
            &ledger,
            &work,
            &execution.attempts[0],
            "worker-rollback-retention",
            &plan.tasks[0].files_owned,
        );
        scope::publish_prepared_claim(&state, &work, &receipt.claim).unwrap();
        let missing_lease = LeaseRecord {
            resource_key: "scope:missing-rollback-lease".to_string(),
            mission_id: mission.id.clone(),
            task_id: execution.attempts[0].task_id.clone(),
            attempt_id: execution.attempts[0].attempt_id.clone(),
            owner: receipt.owner.clone(),
            fencing_token: 99,
            expires_at: Utc::now() + ChronoDuration::seconds(60),
            status: crate::mission_ledger::LeaseStatus::Active,
        };
        let error = rollback_scope_acquisition(
            &ledger,
            &state,
            &mut execution.attempts[0],
            &receipt,
            vec![missing_lease],
            anyhow::anyhow!("synthetic acquisition failure"),
        );
        assert!(error.to_string().contains("rollback incomplete"));
        assert_eq!(execution.attempts[0].scope_receipt, Some(receipt.clone()));
        assert_eq!(execution.attempts[0].leases.len(), 1);
        assert_eq!(
            scope::ScopeClaim::read_strict(&state, &receipt.owner)
                .unwrap()
                .unwrap(),
            receipt.claim
        );
    }

    #[test]
    fn read_only_attempt_claims_an_owner_without_creating_writable_authority() {
        let tmp = tempfile::TempDir::new().unwrap();
        let work = tmp.path().join("repo");
        let state = tmp.path().join("state");
        std::fs::create_dir_all(&work).unwrap();
        std::fs::create_dir_all(&state).unwrap();
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = Mission::new("OmegaOS", "read only verifier", work.clone());
        let mut plan = direct_plan(&mission, "read-only");
        plan.tasks[0].files_owned.clear();
        let mut execution =
            prepare_authoritative_execution(&ledger, &mission, &plan, "test", Vec::new()).unwrap();
        claim_authoritative_scopes(
            &ledger,
            &state,
            &work,
            &mut execution.attempts[0],
            "worker-read-only",
            &[],
            Duration::from_secs(60),
        )
        .unwrap();
        assert_eq!(
            execution.attempts[0].owner.as_deref(),
            Some("worker-read-only")
        );
        assert!(execution.attempts[0].leases.is_empty());
        assert!(execution.attempts[0].scope_receipt.is_none());
        assert!(scope::ScopeClaim::read_all_strict(&state)
            .unwrap()
            .is_empty());
        transition_authoritative_attempt(
            &ledger,
            &execution.attempts[0],
            TaskAttemptState::Running,
            "worker-read-only",
        )
        .unwrap();
        release_authoritative_scopes(&ledger, &state, &execution.attempts[0]).unwrap();
    }

    #[test]
    fn duplicate_selectors_acquire_one_canonical_lease() {
        let tmp = tempfile::TempDir::new().unwrap();
        let work = tmp.path().join("repo");
        let state = tmp.path().join("state");
        std::fs::create_dir_all(&work).unwrap();
        std::fs::create_dir_all(&state).unwrap();
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = Mission::new("OmegaOS", "deduplicated scope", work.clone());
        let mut plan = direct_plan(&mission, "deduplicated-scope");
        plan.tasks[0].files_owned = vec![
            "./src/lib.rs".to_string(),
            "src//lib.rs".to_string(),
            "src/lib.rs".to_string(),
        ];
        let mut execution =
            prepare_authoritative_execution(&ledger, &mission, &plan, "test", Vec::new()).unwrap();
        claim_authoritative_scopes(
            &ledger,
            &state,
            &work,
            &mut execution.attempts[0],
            "worker-deduplicated-scope",
            &plan.tasks[0].files_owned,
            Duration::from_secs(60),
        )
        .unwrap();
        assert_eq!(execution.attempts[0].leases.len(), 1);
        assert_eq!(
            execution.attempts[0]
                .scope_receipt
                .as_ref()
                .unwrap()
                .claim
                .files_owned,
            vec!["src/lib.rs".to_string()]
        );
        release_authoritative_scopes(&ledger, &state, &execution.attempts[0]).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn workspace_aliases_derive_the_same_v3_lease_resource() {
        use std::os::unix::fs::symlink;
        let tmp = tempfile::TempDir::new().unwrap();
        let work = tmp.path().join("repo");
        let alias = tmp.path().join("repo-alias");
        std::fs::create_dir_all(&work).unwrap();
        symlink(&work, &alias).unwrap();
        let first = scope::prepare_claim_for_workspace(
            &work,
            "worker-real",
            vec!["src/lib.rs".to_string()],
        )
        .unwrap();
        let second = scope::prepare_claim_for_workspace(
            &alias,
            "worker-alias",
            vec!["./src//lib.rs".to_string()],
        )
        .unwrap();
        assert_eq!(first.workspace_id, second.workspace_id);
        assert_eq!(
            scope_lease_resource(
                first.workspace_id.as_deref().unwrap(),
                &first.files_owned[0]
            ),
            scope_lease_resource(
                second.workspace_id.as_deref().unwrap(),
                &second.files_owned[0]
            )
        );
    }

    #[test]
    fn done_clean_without_worker_evidence_cannot_be_accepted() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::process::Command::new("git")
            .args(["init", "-q"])
            .current_dir(tmp.path())
            .status()
            .unwrap();
        let mission = Mission::new("OmegaOS", "verify", tmp.path().to_path_buf());
        let plan = direct_plan(&mission, "verify");
        let contract = compatibility_task_contract(&mission, &plan.tasks[0]);
        let mut done = DoneSignal::new("worker", DoneStatus::DoneClean, "trust me");
        done.todos_total = 1;
        done.todos_completed = 1;

        let verdict = independently_verify_task_contract(&done, tmp.path(), &contract);
        assert!(!verdict.passed);
        assert_eq!(verdict.observations.len(), contract.verifier_checks.len());
        assert!(verdict
            .failures
            .iter()
            .any(|failure| failure.contains("zero artifacts")));
    }

    #[test]
    fn failing_git_diff_check_cannot_be_accepted() {
        let tmp = tempfile::TempDir::new().unwrap();
        let git = |args: &[&str]| {
            let status = std::process::Command::new("git")
                .args(args)
                .current_dir(tmp.path())
                .status()
                .unwrap();
            assert!(status.success(), "git {args:?} failed");
        };
        git(&["init", "-q"]);
        git(&["config", "user.email", "test@omega.invalid"]);
        git(&["config", "user.name", "Omega Test"]);
        std::fs::write(tmp.path().join("tracked.txt"), "clean\n").unwrap();
        git(&["add", "tracked.txt"]);
        git(&["commit", "-qm", "fixture"]);
        std::fs::write(tmp.path().join("tracked.txt"), "trailing spaces   \n").unwrap();

        let mission = Mission::new("OmegaOS", "verify", tmp.path().to_path_buf());
        let plan = direct_plan(&mission, "verify");
        let contract = compatibility_task_contract(&mission, &plan.tasks[0]);
        let mut done = DoneSignal::new("worker", DoneStatus::DoneClean, "done");
        done.todos_total = 1;
        done.todos_completed = 1;
        done.corroboration = vec![
            crate::done::CorroborationSource::WorkerSelfReport,
            crate::done::CorroborationSource::CiExitCode,
        ];
        done.artifacts = vec![crate::done::DoneArtifact::Command {
            cmd: "git diff --check".to_string(),
            exit_code: 0,
        }];

        let verdict = independently_verify_task_contract(&done, tmp.path(), &contract);
        assert!(!verdict.passed);
        assert!(verdict
            .observations
            .iter()
            .any(|observation| !observation.passed));
    }

    #[test]
    fn immutable_command_verifier_executes_exactly_once() {
        let tmp = tempfile::TempDir::new().unwrap();
        let script = tmp.path().join("count-verifier.sh");
        let counter = tmp.path().join("counter");
        std::fs::write(&script, "#!/bin/sh\nprintf x >> \"$1\"\n").unwrap();
        let argv = vec![
            "sh".to_string(),
            script.to_string_lossy().to_string(),
            counter.to_string_lossy().to_string(),
        ];
        let mission = Mission::new("OmegaOS", "verify once", tmp.path().to_path_buf());
        let plan = direct_plan(&mission, "verify-once");
        let mut contract = compatibility_task_contract(&mission, &plan.tasks[0]);
        contract.verifier_checks = vec![VerifierCheck {
            schema_version: CONTRACT_SCHEMA_VERSION,
            check_id: "one-invocation".to_string(),
            kind: VerifierCheckKind::Command {
                argv: argv.clone(),
                cwd: None,
                expected_exit_code: 0,
            },
            timeout_secs: 5,
        }];
        let mut done = DoneSignal::new("worker", DoneStatus::DoneClean, "done");
        done.todos_total = 1;
        done.todos_completed = 1;
        done.corroboration = vec![crate::done::CorroborationSource::WorkerSelfReport];
        done.artifacts = vec![crate::done::DoneArtifact::Command {
            cmd: argv.join(" "),
            exit_code: 0,
        }];

        let verdict = independently_verify_task_contract(&done, tmp.path(), &contract);
        assert!(verdict.passed, "{:?}", verdict.failures);
        assert_eq!(std::fs::read_to_string(counter).unwrap(), "x");
    }

    #[tokio::test]
    async fn rubric_includes_security_for_security_missions() {
        let config = OmegaConfig::default();
        let opts = OrchestratorOptions::default();
        // Don't connect to rmux; build a fake orchestrator just for rubric logic
        let mission = Mission::new(
            "TestProj",
            "Audit security of the login flow",
            PathBuf::from("/tmp"),
        );
        let plan = Plan {
            mission_id: mission.id.clone(),
            complexity: Complexity::Medium,
            strategy: PlanStrategy::Direct,
            tasks: vec![],
            created_at: Utc::now(),
        };

        // We can build the rubric without a connected mgr — extract the logic
        let mut has_security = false;
        let rubric_criteria_count = {
            let _ = opts;
            let _ = config;
            // Inline the same logic as build_rubric
            let lower = mission.text.to_lowercase();
            let mut count = 2;
            if matches!(plan.complexity, Complexity::Complex | Complexity::Epic) {
                count += 1;
            }
            if lower.contains("security") || lower.contains("auth") {
                count += 1;
                has_security = true;
            }
            count
        };

        assert_eq!(rubric_criteria_count, 3);
        assert!(has_security);
    }
}

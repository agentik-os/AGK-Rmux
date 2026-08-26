//! Transactional event ledger for OmegaOS Orchestration V3.
//!
//! The ledger is the only write authority. JSON files, Telegram cards, task
//! lists and timelines are projections that can be rebuilt from these events.
//! SQLite gives the single-host runtime atomic event + projection + outbox
//! commits. External effects remain truthfully at-least-once.

use crate::mission::{
    InvalidTransition, Mission, MissionId, MissionState, PlanContract, TaskAttemptState,
};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use rusqlite::{
    params, Connection, OpenFlags, OptionalExtension, Transaction, TransactionBehavior,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;

#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};

const LEGACY_SCHEMA_VERSION: u32 = 1;
const SCHEMA_VERSION: u32 = 2;
const LEGACY_ATTEMPT_PROVENANCE: &str = "legacy_unverified";
const LEGACY_ATTEMPT_MIGRATION: &str = "archive_schema_v1_task_attempts";

#[derive(Debug)]
pub enum LedgerError {
    Io(std::io::Error),
    Sqlite(rusqlite::Error),
    Json(serde_json::Error),
    Chrono(chrono::ParseError),
    MissionNotFound(String),
    MissionAlreadyExists(String),
    VersionConflict {
        expected: u64,
        actual: u64,
    },
    AttemptVersionConflict {
        attempt_id: String,
        expected: u64,
        actual: u64,
    },
    IdempotencyConflict {
        mission_id: String,
        key: String,
        recorded_digest: String,
        supplied_digest: String,
    },
    PlanRevisionNotActive {
        mission_id: String,
        supplied: u64,
        active: Option<u64>,
    },
    TaskNotInPlan {
        mission_id: String,
        revision: u64,
        task_id: String,
    },
    TaskNoLongerActive {
        mission_id: String,
        task_id: String,
        attempt_revision: u64,
        active_revision: u64,
    },
    ProjectionHashMismatch {
        mission_id: String,
        stored: String,
        recomputed: String,
    },
    ReplayDivergence {
        mission_id: String,
        reason: String,
    },
    InvalidTransition(InvalidTransition),
    InvalidTaskTransition(InvalidTransition),
    InvalidInput(String),
    LeaseHeld {
        resource: String,
        owner: String,
        token: u64,
    },
    LeaseContextMismatch {
        resource: String,
        reason: String,
    },
    StaleFence {
        resource: String,
        expected: u64,
        actual: Option<u64>,
    },
    OutboxClaimConflict(String),
}

impl fmt::Display for LedgerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use LedgerError::*;
        match self {
            Io(error) => write!(f, "ledger filesystem error: {error}"),
            Sqlite(error) => write!(f, "sqlite error: {error}"),
            Json(error) => write!(f, "json error: {error}"),
            Chrono(error) => write!(f, "timestamp error: {error}"),
            MissionNotFound(id) => write!(f, "mission not found: {id}"),
            MissionAlreadyExists(id) => write!(f, "mission already exists: {id}"),
            VersionConflict { expected, actual } => {
                write!(
                    f,
                    "mission version conflict: expected {expected}, actual {actual}"
                )
            }
            AttemptVersionConflict {
                attempt_id,
                expected,
                actual,
            } => write!(
                f,
                "task attempt {attempt_id} version conflict: expected {expected}, actual {actual}"
            ),
            IdempotencyConflict {
                mission_id,
                key,
                recorded_digest,
                supplied_digest,
            } => write!(
                f,
                "idempotency key {key} for mission {mission_id} was reused for a different command (recorded {recorded_digest}, supplied {supplied_digest})"
            ),
            PlanRevisionNotActive {
                mission_id,
                supplied,
                active,
            } => write!(
                f,
                "plan revision {supplied} is not active for mission {mission_id}; active revision is {active:?}"
            ),
            TaskNotInPlan {
                mission_id,
                revision,
                task_id,
            } => write!(
                f,
                "task {task_id} does not exist in mission {mission_id} plan revision {revision}"
            ),
            TaskNoLongerActive {
                mission_id,
                task_id,
                attempt_revision,
                active_revision,
            } => write!(
                f,
                "task {task_id} from mission {mission_id} plan revision {attempt_revision} is not unchanged in active revision {active_revision}"
            ),
            ProjectionHashMismatch {
                mission_id,
                stored,
                recomputed,
            } => write!(
                f,
                "mission {mission_id} projection hash mismatch: stored {stored}, recomputed {recomputed}"
            ),
            ReplayDivergence { mission_id, reason } => {
                write!(f, "mission {mission_id} materialized projection diverges from replay: {reason}")
            }
            InvalidTransition(error) | InvalidTaskTransition(error) => error.fmt(f),
            InvalidInput(message) => write!(f, "invalid ledger input: {message}"),
            LeaseHeld {
                resource,
                owner,
                token,
            } => write!(
                f,
                "lease {resource} is held by {owner} with fencing token {token}"
            ),
            LeaseContextMismatch { resource, reason } => {
                write!(f, "lease {resource} context mismatch: {reason}")
            }
            StaleFence {
                resource,
                expected,
                actual,
            } => write!(
                f,
                "stale fencing token for {resource}: supplied {expected}, current {:?}",
                actual
            ),
            OutboxClaimConflict(id) => {
                write!(f, "outbox record {id} is not claimed by this worker")
            }
        }
    }
}

impl std::error::Error for LedgerError {}

impl From<std::io::Error> for LedgerError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<rusqlite::Error> for LedgerError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<serde_json::Error> for LedgerError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<chrono::ParseError> for LedgerError {
    fn from(value: chrono::ParseError) -> Self {
        Self::Chrono(value)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionEvent {
    pub event_id: String,
    pub mission_id: MissionId,
    pub task_id: Option<String>,
    pub attempt_id: Option<String>,
    pub sequence: u64,
    pub expected_version: u64,
    pub schema_version: u32,
    pub idempotency_key: String,
    #[serde(default)]
    pub command_digest: String,
    pub actor: String,
    pub provider: Option<String>,
    pub causation_id: Option<String>,
    pub correlation_id: Option<String>,
    pub fencing_token: Option<u64>,
    #[serde(default)]
    pub plan_revision: Option<u64>,
    /// Set only when this event activates a newly persisted plan. Task attempt
    /// events use `plan_revision` without changing the mission's active plan.
    #[serde(default)]
    pub activated_plan_revision: Option<u64>,
    /// Complete task-attempt command recorded by the append-only authority.
    ///
    /// Schema-v1 events did not persist this value. A legacy event without a
    /// task binding remains replayable, while a legacy task-attempt event is
    /// rejected because its materialized row cannot be promoted to authority.
    #[serde(default)]
    pub task_attempt_mutation: Option<TaskAttemptMutation>,
    /// Exact materialized attempt produced by `task_attempt_mutation`.
    /// Replay derives every task-attempt projection from this immutable value
    /// and validates it against the command, aliases, transition and timestamp.
    #[serde(default)]
    pub resulting_task_attempt: Option<TaskAttemptProjection>,
    pub recorded_at: DateTime<Utc>,
    pub kind: String,
    pub payload: Value,
    pub resulting_mission_state: Option<MissionState>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionProjection {
    pub mission_id: MissionId,
    pub state: MissionState,
    pub version: u64,
    pub active_plan_revision: Option<u64>,
    pub projection_hash: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskAttemptProjection {
    pub mission_id: MissionId,
    pub task_id: String,
    pub attempt_id: String,
    pub plan_revision: u64,
    pub state: TaskAttemptState,
    pub version: u64,
    pub fencing_token: Option<u64>,
    pub updated_at: DateTime<Utc>,
}

/// Auditable, non-authoritative copy of a schema-v1 materialized attempt.
///
/// These records are intentionally a distinct type. In particular, they
/// cannot be passed to code expecting an authoritative
/// [`TaskAttemptProjection`] and therefore cannot satisfy delivery gates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LegacyTaskAttemptRecord {
    pub mission_id: MissionId,
    pub task_id: String,
    pub attempt_id: String,
    pub plan_revision: u64,
    pub historical_state: TaskAttemptState,
    pub historical_version: u64,
    pub historical_fencing_token: Option<u64>,
    pub historical_updated_at: DateTime<Utc>,
    pub imported_at: DateTime<Utc>,
    pub provenance: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskAttemptMutation {
    pub task_id: String,
    pub attempt_id: String,
    pub plan_revision: u64,
    pub expected_version: u64,
    pub next_state: TaskAttemptState,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewOutboxEffect {
    pub idempotency_key: String,
    pub kind: String,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppendEvent {
    pub event_id: String,
    pub mission_id: MissionId,
    pub expected_version: u64,
    pub idempotency_key: String,
    pub actor: String,
    pub provider: Option<String>,
    pub causation_id: Option<String>,
    pub correlation_id: Option<String>,
    pub kind: String,
    pub payload: Value,
    pub next_mission_state: Option<MissionState>,
    pub task_attempt: Option<TaskAttemptMutation>,
    pub plan: Option<PlanContract>,
    /// Plan revision explicitly bound to a non-task observation event.
    /// Task mutations derive this value from their typed mutation instead.
    pub observation_plan_revision: Option<u64>,
    pub lease_resource: Option<String>,
    pub fencing_token: Option<u64>,
    /// Complete lease authority presented by this command.
    ///
    /// `lease_resource`/`fencing_token` remain as a single-lease compatibility
    /// surface. New callers must present every active lease held by the task
    /// attempt here. The full set is authenticated by `command_digest` and is
    /// checked under the same IMMEDIATE transaction as the attempt mutation.
    pub lease_assertions: Vec<LeaseAssertion>,
    pub outbox: Vec<NewOutboxEffect>,
}

impl AppendEvent {
    pub fn new(
        mission_id: MissionId,
        expected_version: u64,
        idempotency_key: impl Into<String>,
        actor: impl Into<String>,
        kind: impl Into<String>,
    ) -> Self {
        Self {
            event_id: stable_id("event"),
            mission_id,
            expected_version,
            idempotency_key: idempotency_key.into(),
            actor: actor.into(),
            provider: None,
            causation_id: None,
            correlation_id: None,
            kind: kind.into(),
            payload: Value::Null,
            next_mission_state: None,
            task_attempt: None,
            plan: None,
            observation_plan_revision: None,
            lease_resource: None,
            fencing_token: None,
            lease_assertions: Vec::new(),
            outbox: Vec::new(),
        }
    }
}

#[derive(Serialize)]
struct AppendCommandDigest<'a> {
    mission_id: &'a MissionId,
    expected_version: u64,
    actor: &'a str,
    provider: &'a Option<String>,
    causation_id: &'a Option<String>,
    correlation_id: &'a Option<String>,
    kind: &'a str,
    payload: &'a Value,
    next_mission_state: &'a Option<MissionState>,
    task_attempt: &'a Option<TaskAttemptMutation>,
    plan: &'a Option<PlanContract>,
    #[serde(skip_serializing_if = "Option::is_none")]
    observation_plan_revision: &'a Option<u64>,
    lease_resource: &'a Option<String>,
    fencing_token: &'a Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lease_assertions: Option<&'a [LeaseAssertion]>,
    outbox: &'a [NewOutboxEffect],
}

fn append_command_digest(request: &AppendEvent) -> Result<String, LedgerError> {
    canonical_digest(&AppendCommandDigest {
        mission_id: &request.mission_id,
        expected_version: request.expected_version,
        actor: &request.actor,
        provider: &request.provider,
        causation_id: &request.causation_id,
        correlation_id: &request.correlation_id,
        kind: &request.kind,
        payload: &request.payload,
        next_mission_state: &request.next_mission_state,
        task_attempt: &request.task_attempt,
        plan: &request.plan,
        observation_plan_revision: &request.observation_plan_revision,
        lease_resource: &request.lease_resource,
        fencing_token: &request.fencing_token,
        // Preserve the historical digest for commands that do not use the new
        // aggregate authority field. This keeps retries of already-recorded
        // unleased and legacy single-fence commands idempotent across upgrade.
        lease_assertions: (!request.lease_assertions.is_empty())
            .then_some(request.lease_assertions.as_slice()),
        outbox: &request.outbox,
    })
}

fn create_mission_command_digest(mission: &Mission, actor: &str) -> Result<String, LedgerError> {
    #[derive(Serialize)]
    struct CreateMissionCommand<'a> {
        mission: &'a Mission,
        actor: &'a str,
        kind: &'static str,
    }
    canonical_digest(&CreateMissionCommand {
        mission,
        actor,
        kind: "mission_created",
    })
}

fn canonical_digest(value: &impl Serialize) -> Result<String, LedgerError> {
    let value = canonicalize_json(serde_json::to_value(value)?);
    let bytes = serde_json::to_vec(&value)?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn canonicalize_json(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let sorted: BTreeMap<String, Value> = map
                .into_iter()
                .map(|(key, value)| (key, canonicalize_json(value)))
                .collect();
            Value::Object(sorted.into_iter().collect())
        }
        Value::Array(values) => Value::Array(values.into_iter().map(canonicalize_json).collect()),
        other => other,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppendOutcome {
    pub event: MissionEvent,
    pub projection: MissionProjection,
    pub idempotent_replay: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaseStatus {
    Active,
    Released,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaseRecord {
    pub resource_key: String,
    pub mission_id: MissionId,
    pub task_id: String,
    pub attempt_id: String,
    pub owner: String,
    pub fencing_token: u64,
    pub expires_at: DateTime<Utc>,
    pub status: LeaseStatus,
}

/// One exact lease credential supplied with an attempt mutation.
///
/// The owner is explicit because an independent verifier may append the
/// verdict while presenting the worker's still-current lease authority. The
/// event actor therefore remains truthful (`omega-independent-verifier`)
/// instead of impersonating the worker that owns the scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaseAssertion {
    pub resource_key: String,
    pub owner: String,
    pub fencing_token: u64,
}

impl From<&LeaseRecord> for LeaseAssertion {
    fn from(lease: &LeaseRecord) -> Self {
        Self {
            resource_key: lease.resource_key.clone(),
            owner: lease.owner.clone(),
            fencing_token: lease.fencing_token,
        }
    }
}

pub const ACCEPTANCE_OBSERVATION_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifierObservation {
    pub check_id: String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractVerification {
    pub passed: bool,
    pub observations: Vec<VerifierObservation>,
    pub failures: Vec<String>,
}

/// Immutable, plan-bound result of executing every verifier in one task
/// contract. Mission acceptance consumes this exact payload, never an
/// `Accepted` projection alone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordedContractVerification {
    #[serde(default = "acceptance_observation_schema_version")]
    pub schema_version: u32,
    pub mission_id: String,
    pub task_id: String,
    pub attempt_id: String,
    pub plan_revision: u64,
    pub plan_digest: String,
    pub worker_signal_digest: String,
    pub verification: ContractVerification,
}

fn acceptance_observation_schema_version() -> u32 {
    ACCEPTANCE_OBSERVATION_SCHEMA_VERSION
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanRequirementKind {
    Gate,
    Approval,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanRequirementObservation {
    #[serde(default = "acceptance_observation_schema_version")]
    pub schema_version: u32,
    pub mission_id: String,
    pub plan_revision: u64,
    pub plan_digest: String,
    pub requirement: String,
    pub kind: PlanRequirementKind,
    pub passed: bool,
    pub observed_by: String,
    pub evidence_event_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissionGateCheckObservation {
    pub check_id: String,
    pub fact_digest: String,
    pub passed: bool,
    pub evidence_event_id: String,
    pub evidence_sequence: u64,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissionGateLensObservation {
    pub lens: String,
    pub passed: bool,
    pub fact_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissionGateObservation {
    #[serde(default = "acceptance_observation_schema_version")]
    pub schema_version: u32,
    pub mission_id: String,
    pub plan_revision: u64,
    pub plan_digest: String,
    pub gate_result_digest: String,
    pub overall_pass: bool,
    pub checks: Vec<MissionGateCheckObservation>,
    pub lenses: Vec<MissionGateLensObservation>,
}

/// Explicit closure of a mission-level invalidation or blocker. Historical
/// issue events are never called "unresolved" merely because they exist: the
/// resolution must name the exact immutable event and active plan it closes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissionIssueResolution {
    #[serde(default = "acceptance_observation_schema_version")]
    pub schema_version: u32,
    pub mission_id: String,
    pub plan_revision: u64,
    pub plan_digest: String,
    pub issue_event_id: String,
    pub resolved_by: String,
    pub detail: String,
}

/// Deterministically bind one gate assertion to the immutable ledger event it
/// cites. Recomputing this in the acceptance transaction prevents callers from
/// supplying an arbitrary 64-character string as a fact digest.
pub fn mission_gate_fact_digest(
    check_id: &str,
    passed: bool,
    detail: &str,
    evidence: &MissionEvent,
) -> Result<String, LedgerError> {
    let bytes = serde_json::to_vec(&(
        "omega.mission-gate-fact.v1",
        check_id,
        passed,
        detail,
        evidence,
    ))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

/// Hash the complete plan-bound gate receipt while excluding the digest field
/// itself. This is stable across serialization formatting and is recomputed at
/// acceptance time.
pub fn mission_gate_result_digest(
    observation: &MissionGateObservation,
) -> Result<String, LedgerError> {
    let bytes = serde_json::to_vec(&(
        "omega.mission-gate-result.v1",
        observation.schema_version,
        &observation.mission_id,
        observation.plan_revision,
        &observation.plan_digest,
        observation.overall_pass,
        &observation.checks,
        &observation.lenses,
    ))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutboxStatus {
    Pending,
    Processing,
    Delivered,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutboxRecord {
    pub outbox_id: String,
    pub mission_id: MissionId,
    pub event_id: String,
    pub idempotency_key: String,
    pub kind: String,
    pub payload: Value,
    pub status: OutboxStatus,
    pub attempts: u32,
    pub available_at: DateTime<Utc>,
    pub claim_owner: Option<String>,
    pub claim_until: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub remote_ref: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LedgerFileIdentity {
    #[cfg(unix)]
    device: u64,
    #[cfg(unix)]
    inode: u64,
    #[cfg(not(unix))]
    length: u64,
}

fn ledger_file_identity(metadata: &fs::Metadata) -> LedgerFileIdentity {
    LedgerFileIdentity {
        #[cfg(unix)]
        device: metadata.dev(),
        #[cfg(unix)]
        inode: metadata.ino(),
        #[cfg(not(unix))]
        length: metadata.len(),
    }
}

fn invalid_ledger_path(path: &Path, reason: impl fmt::Display) -> LedgerError {
    LedgerError::InvalidInput(format!("unsafe ledger path {}: {reason}", path.display()))
}

fn validate_ledger_parent(path: &Path) -> Result<(), LedgerError> {
    if path.file_name().is_none() {
        return Err(invalid_ledger_path(path, "missing database file name"));
    }
    let parent = path
        .parent()
        .filter(|candidate| !candidate.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let metadata = fs::symlink_metadata(parent).map_err(|error| {
        invalid_ledger_path(
            path,
            format!("parent {} is unavailable: {error}", parent.display()),
        )
    })?;
    if metadata.file_type().is_symlink() {
        return Err(invalid_ledger_path(
            path,
            format!("parent {} is a symlink", parent.display()),
        ));
    }
    if !metadata.is_dir() {
        return Err(invalid_ledger_path(
            path,
            format!("parent {} is not a directory", parent.display()),
        ));
    }
    Ok(())
}

fn validate_regular_ledger_file(path: &Path) -> Result<fs::Metadata, LedgerError> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Err(invalid_ledger_path(path, "database file is a symlink"));
    }
    if !metadata.is_file() {
        return Err(invalid_ledger_path(
            path,
            "database path is not a regular file",
        ));
    }
    Ok(metadata)
}

fn sqlite_sidecar(path: &Path, suffix: &str) -> PathBuf {
    let mut value = path.as_os_str().to_os_string();
    value.push(suffix);
    PathBuf::from(value)
}

#[cfg(unix)]
fn set_owner_only(path: &Path) -> Result<(), LedgerError> {
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_owner_only(_path: &Path) -> Result<(), LedgerError> {
    Ok(())
}

fn secure_existing_sqlite_file(path: &Path) -> Result<(), LedgerError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(invalid_ledger_path(
                    path,
                    "SQLite file is a symlink or non-regular file",
                ));
            }
            set_owner_only(path)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn prepare_ledger_path(path: &Path) -> Result<LedgerFileIdentity, LedgerError> {
    validate_ledger_parent(path)?;
    let metadata = match fs::symlink_metadata(path) {
        Ok(_) => validate_regular_ledger_file(path)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let mut options = OpenOptions::new();
            options.read(true).write(true).create_new(true);
            #[cfg(unix)]
            options.mode(0o600);
            options.open(path)?;
            validate_regular_ledger_file(path)?
        }
        Err(error) => return Err(error.into()),
    };
    set_owner_only(path)?;
    for suffix in ["-wal", "-shm"] {
        secure_existing_sqlite_file(&sqlite_sidecar(path, suffix))?;
    }
    validate_ledger_parent(path)?;
    Ok(ledger_file_identity(&metadata))
}

fn verify_ledger_identity(path: &Path, expected: LedgerFileIdentity) -> Result<(), LedgerError> {
    validate_ledger_parent(path)?;
    let actual = ledger_file_identity(&validate_regular_ledger_file(path)?);
    if actual != expected {
        return Err(invalid_ledger_path(
            path,
            "database file changed while it was being opened",
        ));
    }
    Ok(())
}

fn secure_sqlite_files(path: &Path) -> Result<(), LedgerError> {
    validate_ledger_parent(path)?;
    validate_regular_ledger_file(path)?;
    set_owner_only(path)?;
    for suffix in ["-wal", "-shm"] {
        secure_existing_sqlite_file(&sqlite_sidecar(path, suffix))?;
    }
    Ok(())
}

fn archive_unverified_legacy_attempts(connection: &Connection) -> Result<(), LedgerError> {
    let imported_at = Utc::now().to_rfc3339();
    connection.execute(
        "INSERT INTO legacy_task_attempts (
            attempt_id, mission_id, task_id, plan_revision, state_json,
            version, fencing_token, updated_at, imported_at, provenance, source
         )
         SELECT attempts.attempt_id, attempts.mission_id, attempts.task_id,
                attempts.plan_revision, attempts.state_json, attempts.version,
                attempts.fencing_token, attempts.updated_at, ?1,
                'legacy_unverified',
                CASE WHEN EXISTS (
                    SELECT 1 FROM events AS legacy_events
                    WHERE legacy_events.schema_version = 1
                      AND legacy_events.mission_id = attempts.mission_id
                      AND legacy_events.task_id = attempts.task_id
                      AND legacy_events.attempt_id = attempts.attempt_id
                ) THEN 'schema_v1_event_projection'
                  ELSE 'unbound_projection'
                END
         FROM task_attempts AS attempts
         WHERE NOT EXISTS (
             SELECT 1 FROM events AS authoritative_events
             WHERE authoritative_events.schema_version = 2
               AND authoritative_events.mission_id = attempts.mission_id
               AND authoritative_events.task_id = attempts.task_id
               AND authoritative_events.attempt_id = attempts.attempt_id
               AND authoritative_events.task_attempt_mutation_json IS NOT NULL
               AND authoritative_events.task_attempt_projection_json IS NOT NULL
         )
         ON CONFLICT(attempt_id) DO NOTHING",
        params![imported_at],
    )?;

    let mismatch: Option<String> = connection
        .query_row(
            "SELECT attempts.attempt_id
             FROM task_attempts AS attempts
             JOIN legacy_task_attempts AS archived
               ON archived.attempt_id = attempts.attempt_id
             WHERE NOT EXISTS (
                 SELECT 1 FROM events AS authoritative_events
                 WHERE authoritative_events.schema_version = 2
                   AND authoritative_events.mission_id = attempts.mission_id
                   AND authoritative_events.task_id = attempts.task_id
                   AND authoritative_events.attempt_id = attempts.attempt_id
                   AND authoritative_events.task_attempt_mutation_json IS NOT NULL
                   AND authoritative_events.task_attempt_projection_json IS NOT NULL
             )
               AND (
                   archived.mission_id != attempts.mission_id
                   OR archived.task_id != attempts.task_id
                   OR archived.plan_revision != attempts.plan_revision
                   OR archived.state_json != attempts.state_json
                   OR archived.version != attempts.version
                   OR archived.fencing_token IS NOT attempts.fencing_token
                   OR archived.updated_at != attempts.updated_at
                   OR archived.provenance != 'legacy_unverified'
                   OR archived.source != CASE WHEN EXISTS (
                       SELECT 1 FROM events AS legacy_events
                       WHERE legacy_events.schema_version = 1
                         AND legacy_events.mission_id = attempts.mission_id
                         AND legacy_events.task_id = attempts.task_id
                         AND legacy_events.attempt_id = attempts.attempt_id
                   ) THEN 'schema_v1_event_projection'
                     ELSE 'unbound_projection'
                   END
               )
             ORDER BY attempts.attempt_id LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(attempt_id) = mismatch {
        return Err(LedgerError::InvalidInput(format!(
            "legacy archive conflict for task attempt {attempt_id}"
        )));
    }

    let unarchived: Option<String> = connection
        .query_row(
            "SELECT attempts.attempt_id
             FROM task_attempts AS attempts
             LEFT JOIN legacy_task_attempts AS archived
               ON archived.attempt_id = attempts.attempt_id
             WHERE archived.attempt_id IS NULL
               AND NOT EXISTS (
                   SELECT 1 FROM events AS authoritative_events
                   WHERE authoritative_events.schema_version = 2
                     AND authoritative_events.mission_id = attempts.mission_id
                     AND authoritative_events.task_id = attempts.task_id
                     AND authoritative_events.attempt_id = attempts.attempt_id
                     AND authoritative_events.task_attempt_mutation_json IS NOT NULL
                     AND authoritative_events.task_attempt_projection_json IS NOT NULL
               )
             ORDER BY attempts.attempt_id LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(attempt_id) = unarchived {
        return Err(LedgerError::InvalidInput(format!(
            "legacy task attempt {attempt_id} could not be archived"
        )));
    }

    connection.execute(
        "DELETE FROM task_attempts AS attempts
         WHERE NOT EXISTS (
             SELECT 1 FROM events AS authoritative_events
             WHERE authoritative_events.schema_version = 2
               AND authoritative_events.mission_id = attempts.mission_id
               AND authoritative_events.task_id = attempts.task_id
               AND authoritative_events.attempt_id = attempts.attempt_id
               AND authoritative_events.task_attempt_mutation_json IS NOT NULL
               AND authoritative_events.task_attempt_projection_json IS NOT NULL
         )",
        [],
    )?;
    Ok(())
}

pub struct MissionLedger {
    connection: Mutex<Connection>,
}

impl MissionLedger {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, LedgerError> {
        let path = path.as_ref();
        let identity = prepare_ledger_path(path)?;
        let connection = Connection::open_with_flags(
            path,
            OpenFlags::default() | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )?;
        verify_ledger_identity(path, identity)?;
        let ledger = Self::from_connection(connection)?;
        verify_ledger_identity(path, identity)?;
        secure_sqlite_files(path)?;
        Ok(ledger)
    }

    pub fn open_in_memory() -> Result<Self, LedgerError> {
        let connection = Connection::open_in_memory()?;
        Self::from_connection(connection)
    }

    fn from_connection(mut connection: Connection) -> Result<Self, LedgerError> {
        connection.busy_timeout(Duration::from_secs(5))?;
        connection.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = FULL;
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS missions (
                mission_id TEXT PRIMARY KEY,
                mission_json TEXT NOT NULL,
                state_json TEXT NOT NULL,
                version INTEGER NOT NULL,
                active_plan_revision INTEGER,
                projection_hash TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS events (
                event_id TEXT PRIMARY KEY,
                mission_id TEXT NOT NULL,
                task_id TEXT,
                attempt_id TEXT,
                sequence INTEGER NOT NULL,
                expected_version INTEGER NOT NULL,
                schema_version INTEGER NOT NULL,
                idempotency_key TEXT NOT NULL,
                command_digest TEXT NOT NULL DEFAULT '',
                actor TEXT NOT NULL,
                provider TEXT,
                causation_id TEXT,
                correlation_id TEXT,
                fencing_token INTEGER,
                plan_revision INTEGER,
                activated_plan_revision INTEGER,
                task_attempt_mutation_json TEXT,
                task_attempt_projection_json TEXT,
                recorded_at TEXT NOT NULL,
                kind TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                mission_state_json TEXT,
                UNIQUE(mission_id, sequence),
                UNIQUE(mission_id, idempotency_key),
                FOREIGN KEY(mission_id) REFERENCES missions(mission_id)
            );

            CREATE TABLE IF NOT EXISTS plans (
                mission_id TEXT NOT NULL,
                plan_id TEXT NOT NULL,
                revision INTEGER NOT NULL,
                contract_json TEXT NOT NULL,
                content_digest TEXT NOT NULL,
                PRIMARY KEY(mission_id, revision),
                UNIQUE(mission_id, content_digest),
                FOREIGN KEY(mission_id) REFERENCES missions(mission_id)
            );

            CREATE TABLE IF NOT EXISTS task_attempts (
                attempt_id TEXT PRIMARY KEY,
                mission_id TEXT NOT NULL,
                task_id TEXT NOT NULL,
                plan_revision INTEGER NOT NULL,
                state_json TEXT NOT NULL,
                version INTEGER NOT NULL,
                fencing_token INTEGER,
                updated_at TEXT NOT NULL,
                UNIQUE(mission_id, task_id, attempt_id),
                FOREIGN KEY(mission_id) REFERENCES missions(mission_id)
            );

            CREATE TABLE IF NOT EXISTS legacy_task_attempts (
                attempt_id TEXT PRIMARY KEY,
                mission_id TEXT NOT NULL,
                task_id TEXT NOT NULL,
                plan_revision INTEGER NOT NULL,
                state_json TEXT NOT NULL,
                version INTEGER NOT NULL,
                fencing_token INTEGER,
                updated_at TEXT NOT NULL,
                imported_at TEXT NOT NULL,
                provenance TEXT NOT NULL CHECK (provenance = 'legacy_unverified'),
                source TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS ledger_migrations (
                name TEXT PRIMARY KEY,
                applied_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS leases (
                resource_key TEXT PRIMARY KEY,
                mission_id TEXT NOT NULL,
                task_id TEXT NOT NULL,
                attempt_id TEXT NOT NULL,
                owner TEXT NOT NULL,
                fencing_token INTEGER NOT NULL,
                expires_at TEXT NOT NULL,
                status TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS outbox (
                outbox_id TEXT PRIMARY KEY,
                mission_id TEXT NOT NULL,
                event_id TEXT NOT NULL,
                idempotency_key TEXT NOT NULL UNIQUE,
                kind TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                status TEXT NOT NULL,
                attempts INTEGER NOT NULL DEFAULT 0,
                available_at TEXT NOT NULL,
                claim_owner TEXT,
                claim_until TEXT,
                last_error TEXT,
                remote_ref TEXT,
                FOREIGN KEY(mission_id) REFERENCES missions(mission_id),
                FOREIGN KEY(event_id) REFERENCES events(event_id)
            );

            CREATE INDEX IF NOT EXISTS idx_events_mission_sequence
                ON events(mission_id, sequence);
            CREATE INDEX IF NOT EXISTS idx_outbox_delivery
                ON outbox(status, available_at);
            CREATE INDEX IF NOT EXISTS idx_legacy_attempts_mission
                ON legacy_task_attempts(mission_id, task_id, attempt_id);
            "#,
        )?;
        // Forward-compatible migration for ledgers created by the first V3
        // draft, before plan_revision became part of the immutable event.
        let has_plan_revision = {
            let mut statement = connection.prepare("PRAGMA table_info(events)")?;
            let columns = statement.query_map([], |row| row.get::<_, String>(1))?;
            let mut found = false;
            for column in columns {
                if column? == "plan_revision" {
                    found = true;
                    break;
                }
            }
            found
        };
        if !has_plan_revision {
            connection.execute("ALTER TABLE events ADD COLUMN plan_revision INTEGER", [])?;
        }
        let has_activated_plan_revision = {
            let mut statement = connection.prepare("PRAGMA table_info(events)")?;
            let columns = statement.query_map([], |row| row.get::<_, String>(1))?;
            let mut found = false;
            for column in columns {
                if column? == "activated_plan_revision" {
                    found = true;
                    break;
                }
            }
            found
        };
        if !has_activated_plan_revision {
            connection.execute(
                "ALTER TABLE events ADD COLUMN activated_plan_revision INTEGER",
                [],
            )?;
        }
        // Safely distinguish legacy plan activations from task-attempt bindings.
        // A plan activation has no attempt identity and its immutable plan was
        // created from the event's expected mission version.
        connection.execute(
            "UPDATE events
             SET activated_plan_revision = plan_revision
             WHERE activated_plan_revision IS NULL
               AND plan_revision IS NOT NULL
               AND task_id IS NULL
               AND attempt_id IS NULL
               AND EXISTS (
                 SELECT 1 FROM plans
                 WHERE plans.mission_id = events.mission_id
                   AND plans.revision = events.plan_revision
                   AND json_extract(plans.contract_json, '$.created_from_version') = events.expected_version
               )",
            [],
        )?;
        // The digest binds an idempotency key to the full semantic command.
        // Existing rows intentionally migrate to an empty digest: a retry against
        // one of them cannot be proven equivalent and therefore fails closed.
        let has_command_digest = {
            let mut statement = connection.prepare("PRAGMA table_info(events)")?;
            let columns = statement.query_map([], |row| row.get::<_, String>(1))?;
            let mut found = false;
            for column in columns {
                if column? == "command_digest" {
                    found = true;
                    break;
                }
            }
            found
        };
        if !has_command_digest {
            connection.execute(
                "ALTER TABLE events ADD COLUMN command_digest TEXT NOT NULL DEFAULT ''",
                [],
            )?;
        }
        let has_task_attempt_mutation = {
            let mut statement = connection.prepare("PRAGMA table_info(events)")?;
            let columns = statement.query_map([], |row| row.get::<_, String>(1))?;
            let mut found = false;
            for column in columns {
                if column? == "task_attempt_mutation_json" {
                    found = true;
                    break;
                }
            }
            found
        };
        if !has_task_attempt_mutation {
            connection.execute(
                "ALTER TABLE events ADD COLUMN task_attempt_mutation_json TEXT",
                [],
            )?;
        }
        let has_task_attempt_projection = {
            let mut statement = connection.prepare("PRAGMA table_info(events)")?;
            let columns = statement.query_map([], |row| row.get::<_, String>(1))?;
            let mut found = false;
            for column in columns {
                if column? == "task_attempt_projection_json" {
                    found = true;
                    break;
                }
            }
            found
        };
        if !has_task_attempt_projection {
            connection.execute(
                "ALTER TABLE events ADD COLUMN task_attempt_projection_json TEXT",
                [],
            )?;
        }
        // Opening a ledger is both the legacy quarantine boundary and an
        // integrity boundary. Archive + delete + verification share one
        // immediate transaction: no legacy row can remain authoritative and a
        // failed verification cannot leave a half-migrated ledger behind.
        {
            let transaction =
                connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let migration_applied: bool = transaction.query_row(
                "SELECT EXISTS(SELECT 1 FROM ledger_migrations WHERE name = ?1)",
                params![LEGACY_ATTEMPT_MIGRATION],
                |row| row.get(0),
            )?;
            if !migration_applied {
                archive_unverified_legacy_attempts(&transaction)?;
            }
            verify_all_projection_coherence(&transaction)?;
            if !migration_applied {
                transaction.execute(
                    "INSERT INTO ledger_migrations (name, applied_at) VALUES (?1, ?2)",
                    params![LEGACY_ATTEMPT_MIGRATION, Utc::now().to_rfc3339()],
                )?;
            }
            transaction.commit()?;
        }
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    pub fn create_mission(
        &self,
        mission: &Mission,
        idempotency_key: &str,
        actor: &str,
    ) -> Result<AppendOutcome, LedgerError> {
        validate_key(idempotency_key, "idempotency_key")?;
        let command_digest = create_mission_command_digest(mission, actor)?;
        let mut connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;

        if let Some(event) =
            read_event_by_idempotency(&transaction, mission.id.as_str(), idempotency_key)?
        {
            ensure_idempotent_match(&event, &command_digest)?;
            let projection = read_projection(&transaction, mission.id.as_str())?
                .ok_or_else(|| LedgerError::MissionNotFound(mission.id.0.clone()))?;
            verify_projection_coherence(&transaction, &projection)?;
            transaction.commit()?;
            return Ok(AppendOutcome {
                event,
                projection,
                idempotent_replay: true,
            });
        }

        let exists: bool = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM missions WHERE mission_id = ?1)",
            params![mission.id.as_str()],
            |row| row.get(0),
        )?;
        if exists {
            return Err(LedgerError::MissionAlreadyExists(mission.id.0.clone()));
        }

        let now = Utc::now();
        let state = MissionState::Created;
        let projection_hash = projection_hash(&mission.id, state, 1, None)?;
        transaction.execute(
            "INSERT INTO missions (
                mission_id, mission_json, state_json, version,
                active_plan_revision, projection_hash, created_at, updated_at
             ) VALUES (?1, ?2, ?3, 1, NULL, ?4, ?5, ?5)",
            params![
                mission.id.as_str(),
                serde_json::to_string(mission)?,
                serde_json::to_string(&state)?,
                projection_hash,
                now.to_rfc3339(),
            ],
        )?;
        let event = MissionEvent {
            event_id: stable_id("event"),
            mission_id: mission.id.clone(),
            task_id: None,
            attempt_id: None,
            sequence: 1,
            expected_version: 0,
            schema_version: SCHEMA_VERSION,
            idempotency_key: idempotency_key.to_string(),
            command_digest,
            actor: actor.to_string(),
            provider: None,
            causation_id: None,
            correlation_id: None,
            fencing_token: None,
            plan_revision: None,
            activated_plan_revision: None,
            task_attempt_mutation: None,
            resulting_task_attempt: None,
            recorded_at: now,
            kind: "mission_created".to_string(),
            payload: serde_json::to_value(mission)?,
            resulting_mission_state: Some(state),
        };
        insert_event(&transaction, &event)?;
        let projection = read_projection(&transaction, mission.id.as_str())?
            .ok_or_else(|| LedgerError::MissionNotFound(mission.id.0.clone()))?;
        verify_projection_coherence(&transaction, &projection)?;
        transaction.commit()?;
        Ok(AppendOutcome {
            event,
            projection,
            idempotent_replay: false,
        })
    }

    /// Atomically append an event, update materialized projections, persist an
    /// optional immutable plan revision, and enqueue external effects.
    pub fn append(&self, request: AppendEvent) -> Result<AppendOutcome, LedgerError> {
        validate_key(&request.idempotency_key, "idempotency_key")?;
        validate_key(&request.event_id, "event_id")?;
        let command_digest = append_command_digest(&request)?;
        if let Some(plan) = &request.plan {
            plan.verify_integrity()
                .map_err(|error| LedgerError::InvalidInput(error.to_string()))?;
            if plan.mission_id != request.mission_id {
                return Err(LedgerError::InvalidInput(
                    "plan mission_id differs from event mission_id".to_string(),
                ));
            }
            if plan.created_from_version != request.expected_version {
                return Err(LedgerError::InvalidInput(format!(
                    "plan created_from_version {} differs from expected mission version {}",
                    plan.created_from_version, request.expected_version
                )));
            }
        }

        let mut connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;

        if let Some(event) = read_event_by_idempotency(
            &transaction,
            request.mission_id.as_str(),
            &request.idempotency_key,
        )? {
            ensure_idempotent_match(&event, &command_digest)?;
            let projection = read_projection(&transaction, request.mission_id.as_str())?
                .ok_or_else(|| LedgerError::MissionNotFound(request.mission_id.0.clone()))?;
            verify_projection_coherence(&transaction, &projection)?;
            transaction.commit()?;
            return Ok(AppendOutcome {
                event,
                projection,
                idempotent_replay: true,
            });
        }

        let current = read_projection(&transaction, request.mission_id.as_str())?
            .ok_or_else(|| LedgerError::MissionNotFound(request.mission_id.0.clone()))?;
        verify_projection_coherence(&transaction, &current)?;
        if current.version != request.expected_version {
            return Err(LedgerError::VersionConflict {
                expected: request.expected_version,
                actual: current.version,
            });
        }
        if let Some(plan) = &request.plan {
            validate_plan_activation(&transaction, &current, plan)?;
        }
        if let Some(revision) = request.observation_plan_revision {
            if request.task_attempt.is_some() || request.plan.is_some() {
                return Err(LedgerError::InvalidInput(
                    "observation_plan_revision is only valid for non-task observations".to_string(),
                ));
            }
            if current.active_plan_revision != Some(revision) {
                return Err(LedgerError::PlanRevisionNotActive {
                    mission_id: request.mission_id.0.clone(),
                    supplied: revision,
                    active: current.active_plan_revision,
                });
            }
        }
        let supplied_leases = supplied_lease_assertions(&request)?;
        let projection_fence = if let Some(mutation) = request.task_attempt.as_ref() {
            assert_complete_attempt_lease_authority_tx(
                &transaction,
                &request.mission_id,
                mutation,
                &supplied_leases,
            )?;
            if supplied_leases.len() == 1 {
                Some(supplied_leases[0].fencing_token)
            } else {
                None
            }
        } else {
            if !supplied_leases.is_empty() {
                return Err(LedgerError::InvalidInput(
                    "lease authority requires a task-attempt mutation".to_string(),
                ));
            }
            None
        };

        let next_state = match request.next_mission_state {
            Some(next) => current
                .state
                .transition(next)
                .map_err(LedgerError::InvalidTransition)?,
            None => current.state,
        };
        if matches!(
            request.next_mission_state,
            Some(MissionState::Accepted | MissionState::Reporting | MissionState::Delivered)
        ) {
            validate_mission_acceptance_connection(&transaction, &request.mission_id)?;
        }
        let now = Utc::now();
        let next_version = current.version.checked_add(1).ok_or_else(|| {
            LedgerError::InvalidInput("mission projection version overflow".to_string())
        })?;
        let task_projection = if let Some(mutation) = &request.task_attempt {
            Some(apply_task_mutation(
                &transaction,
                &request.mission_id,
                mutation,
                projection_fence,
                now,
            )?)
        } else {
            None
        };

        let active_plan_revision = if let Some(plan) = &request.plan {
            persist_plan(&transaction, plan)?;
            Some(plan.revision)
        } else {
            current.active_plan_revision
        };
        let hash = projection_hash(
            &request.mission_id,
            next_state,
            next_version,
            active_plan_revision,
        )?;
        transaction.execute(
            "UPDATE missions
             SET state_json = ?1, version = ?2, active_plan_revision = ?3,
                 projection_hash = ?4, updated_at = ?5
             WHERE mission_id = ?6 AND version = ?7",
            params![
                serde_json::to_string(&next_state)?,
                as_i64(next_version)?,
                active_plan_revision.map(as_i64).transpose()?,
                hash,
                now.to_rfc3339(),
                request.mission_id.as_str(),
                as_i64(request.expected_version)?,
            ],
        )?;

        let event = MissionEvent {
            event_id: request.event_id,
            mission_id: request.mission_id.clone(),
            task_id: task_projection.as_ref().map(|task| task.task_id.clone()),
            attempt_id: task_projection.as_ref().map(|task| task.attempt_id.clone()),
            sequence: next_version,
            expected_version: request.expected_version,
            schema_version: SCHEMA_VERSION,
            idempotency_key: request.idempotency_key,
            command_digest,
            actor: request.actor,
            provider: request.provider,
            causation_id: request.causation_id,
            correlation_id: request.correlation_id,
            // The materialized/replayed projection retains a token only when
            // authority consisted of exactly one lease. Multi-lease commands
            // are authenticated as a set in the command digest instead of
            // pretending one token represents the whole scope.
            fencing_token: projection_fence,
            plan_revision: request
                .task_attempt
                .as_ref()
                .map(|attempt| attempt.plan_revision)
                .or(request.observation_plan_revision),
            activated_plan_revision: request.plan.as_ref().map(|plan| plan.revision),
            task_attempt_mutation: request.task_attempt,
            resulting_task_attempt: task_projection,
            recorded_at: now,
            kind: request.kind,
            payload: request.payload,
            resulting_mission_state: request.next_mission_state,
        };
        insert_event(&transaction, &event)?;
        for (index, effect) in request.outbox.iter().enumerate() {
            insert_outbox(&transaction, &event, effect, index, now)?;
        }
        let projection = read_projection(&transaction, request.mission_id.as_str())?
            .ok_or_else(|| LedgerError::MissionNotFound(request.mission_id.0.clone()))?;
        verify_projection_coherence(&transaction, &projection)?;
        transaction.commit()?;
        Ok(AppendOutcome {
            event,
            projection,
            idempotent_replay: false,
        })
    }

    pub fn mission(
        &self,
        mission_id: &MissionId,
    ) -> Result<Option<MissionProjection>, LedgerError> {
        let connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        let projection = read_projection(&connection, mission_id.as_str())?;
        if let Some(projection) = &projection {
            verify_projection_coherence(&connection, projection)?;
        }
        Ok(projection)
    }

    /// Re-evaluate the complete mission-acceptance predicate against the
    /// current immutable stream without appending a transition. Delivery and
    /// status readers use this after acceptance so a late contradiction cannot
    /// remain truthfully green merely because the projection is terminal.
    pub fn validate_mission_acceptance(&self, mission_id: &MissionId) -> Result<(), LedgerError> {
        validate_key(mission_id.as_str(), "mission_id")?;
        let connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        validate_mission_acceptance_connection(&connection, mission_id)
    }

    /// Return the immutable mission contract stored when the ledger authority
    /// was created. Unlike [`MissionLedger::mission`], this exposes the frozen
    /// project, text and working directory rather than only their lifecycle
    /// projection. The row and creation event must agree exactly before the
    /// value is returned.
    pub fn mission_record(&self, mission_id: &MissionId) -> Result<Option<Mission>, LedgerError> {
        validate_key(mission_id.as_str(), "mission_id")?;
        let connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        let Some((mission_json, projection)) = connection
            .query_row(
                "SELECT mission_json, state_json, version, active_plan_revision,
                        projection_hash, updated_at
                 FROM missions WHERE mission_id = ?1",
                params![mission_id.as_str()],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        MissionProjection {
                            mission_id: mission_id.clone(),
                            state: serde_json::from_str::<MissionState>(&row.get::<_, String>(1)?)
                                .map_err(|error| {
                                    rusqlite::Error::FromSqlConversionFailure(
                                        1,
                                        rusqlite::types::Type::Text,
                                        Box::new(error),
                                    )
                                })?,
                            version: as_u64(row.get::<_, i64>(2)?).map_err(|error| {
                                rusqlite::Error::FromSqlConversionFailure(
                                    2,
                                    rusqlite::types::Type::Integer,
                                    Box::new(error),
                                )
                            })?,
                            active_plan_revision: row
                                .get::<_, Option<i64>>(3)?
                                .map(as_u64)
                                .transpose()
                                .map_err(|error| {
                                    rusqlite::Error::FromSqlConversionFailure(
                                        3,
                                        rusqlite::types::Type::Integer,
                                        Box::new(error),
                                    )
                                })?,
                            projection_hash: row.get(4)?,
                            updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                                .map_err(|error| {
                                    rusqlite::Error::FromSqlConversionFailure(
                                        5,
                                        rusqlite::types::Type::Text,
                                        Box::new(error),
                                    )
                                })?
                                .with_timezone(&Utc),
                        },
                    ))
                },
            )
            .optional()?
        else {
            return Ok(None);
        };
        verify_projection_coherence(&connection, &projection)?;
        let mission: Mission = serde_json::from_str(&mission_json)?;
        if mission.id != *mission_id {
            return Err(LedgerError::ReplayDivergence {
                mission_id: mission_id.0.clone(),
                reason: format!("immutable mission row belongs to {}", mission.id.as_str()),
            });
        }
        let created = read_events(&connection, mission_id)?
            .into_iter()
            .find(|event| event.kind == "mission_created")
            .ok_or_else(|| LedgerError::ReplayDivergence {
                mission_id: mission_id.0.clone(),
                reason: "immutable mission has no creation event".to_string(),
            })?;
        let event_mission: Mission = serde_json::from_value(created.payload)?;
        if serde_json::to_value(&event_mission)? != serde_json::to_value(&mission)? {
            return Err(LedgerError::ReplayDivergence {
                mission_id: mission_id.0.clone(),
                reason: "immutable mission row differs from its creation event".to_string(),
            });
        }
        Ok(Some(mission))
    }

    /// Return the immutable plan revision currently selected by the mission
    /// projection. Compatibility readers use this to verify a legacy
    /// done.json against the checks that existed before the worker ran.
    pub fn active_plan(&self, mission_id: &MissionId) -> Result<Option<PlanContract>, LedgerError> {
        let connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        let Some(projection) = read_projection(&connection, mission_id.as_str())? else {
            return Ok(None);
        };
        verify_projection_coherence(&connection, &projection)?;
        let Some(revision) = projection.active_plan_revision else {
            return Ok(None);
        };
        let contract = read_plan_contract(&connection, mission_id, revision)?.ok_or_else(|| {
            LedgerError::InvalidInput(format!(
                "active plan revision {revision} is missing for mission {}",
                mission_id.as_str()
            ))
        })?;
        Ok(Some(contract))
    }

    pub fn task_attempt(
        &self,
        attempt_id: &str,
    ) -> Result<Option<TaskAttemptProjection>, LedgerError> {
        let connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        let attempt = read_task_attempt(&connection, attempt_id)?;
        let mission_id = match &attempt {
            Some(attempt) => Some(attempt.mission_id.clone()),
            None => connection
                .query_row(
                    "SELECT mission_id FROM events
                     WHERE attempt_id = ?1
                        OR json_extract(task_attempt_mutation_json, '$.attempt_id') = ?1
                     LIMIT 1",
                    params![attempt_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()?
                .map(MissionId),
        };
        if let Some(mission_id) = mission_id {
            let projection =
                read_projection(&connection, mission_id.as_str())?.ok_or_else(|| {
                    LedgerError::ReplayDivergence {
                        mission_id: mission_id.0.clone(),
                        reason: format!(
                            "task attempt {attempt_id} refers to a missing mission projection"
                        ),
                    }
                })?;
            verify_projection_coherence(&connection, &projection)?;
        }
        Ok(attempt)
    }

    pub fn task_attempts(
        &self,
        mission_id: &MissionId,
    ) -> Result<Vec<TaskAttemptProjection>, LedgerError> {
        let connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        let projection = read_projection(&connection, mission_id.as_str())?
            .ok_or_else(|| LedgerError::MissionNotFound(mission_id.0.clone()))?;
        verify_projection_coherence(&connection, &projection)?;
        read_task_attempts_for_mission(&connection, mission_id)
    }

    /// Historical schema-v1 attempts quarantined during migration.
    ///
    /// This diagnostic surface is deliberately separate from `task_attempt*`:
    /// callers can inspect old state, but cannot accidentally treat it as an
    /// authoritative attempt accepted by the current event stream.
    pub fn legacy_task_attempts(
        &self,
        mission_id: &MissionId,
    ) -> Result<Vec<LegacyTaskAttemptRecord>, LedgerError> {
        let connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        read_legacy_task_attempts(&connection, mission_id)
    }

    pub fn legacy_task_attempt_count(&self) -> Result<u64, LedgerError> {
        let connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        let count: i64 =
            connection.query_row("SELECT COUNT(*) FROM legacy_task_attempts", [], |row| {
                row.get(0)
            })?;
        as_u64(count)
    }

    pub fn events(&self, mission_id: &MissionId) -> Result<Vec<MissionEvent>, LedgerError> {
        let connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        read_events(&connection, mission_id)
    }

    /// Replay only the append-only event sequence. Materialized mission rows are
    /// not consulted, so this is an independent corruption/drift check.
    pub fn replay(&self, mission_id: &MissionId) -> Result<MissionProjection, LedgerError> {
        let connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        replay_from_connection(&connection, mission_id)
    }

    /// Reconstruct all task attempts from the immutable event stream without
    /// consulting the materialized `task_attempts` table.
    pub fn replay_task_attempts(
        &self,
        mission_id: &MissionId,
    ) -> Result<Vec<TaskAttemptProjection>, LedgerError> {
        let connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        let mut attempts: Vec<_> = replay_all_from_connection(&connection, mission_id)?
            .task_attempts
            .into_values()
            .collect();
        attempts.sort_by(|left, right| {
            (&left.task_id, &left.attempt_id).cmp(&(&right.task_id, &right.attempt_id))
        });
        Ok(attempts)
    }

    /// Restore mission and task-attempt projections from the append-only event
    /// stream. Event and plan rows are never rewritten. The replacement is one
    /// immediate transaction and is accepted only after coherence revalidation.
    pub fn rebuild_projections(
        &self,
        mission_id: &MissionId,
    ) -> Result<MissionProjection, LedgerError> {
        let mut connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let replayed = replay_all_from_connection(&transaction, mission_id)?;
        let changed = transaction.execute(
            "UPDATE missions
             SET state_json = ?1, version = ?2, active_plan_revision = ?3,
                 projection_hash = ?4, updated_at = ?5
             WHERE mission_id = ?6",
            params![
                serde_json::to_string(&replayed.mission.state)?,
                as_i64(replayed.mission.version)?,
                replayed
                    .mission
                    .active_plan_revision
                    .map(as_i64)
                    .transpose()?,
                replayed.mission.projection_hash,
                replayed.mission.updated_at.to_rfc3339(),
                mission_id.as_str(),
            ],
        )?;
        if changed != 1 {
            return Err(LedgerError::MissionNotFound(mission_id.0.clone()));
        }

        transaction.execute(
            "DELETE FROM task_attempts WHERE mission_id = ?1",
            params![mission_id.as_str()],
        )?;
        for attempt in replayed.task_attempts.values() {
            // Also remove a projection whose globally unique attempt id was
            // maliciously rebound to another mission.
            transaction.execute(
                "DELETE FROM task_attempts WHERE attempt_id = ?1",
                params![attempt.attempt_id],
            )?;
            insert_task_attempt_projection(&transaction, attempt)?;
        }
        verify_projection_coherence(&transaction, &replayed.mission)?;
        transaction.commit()?;
        Ok(replayed.mission)
    }

    /// Rebuild the authoritative projection through exactly `sequence`.
    ///
    /// This is the trust boundary for stale compatibility views: callers can
    /// compare their persisted source hash to the hash of the immutable event
    /// prefix they cite. `None` means that exact sequence does not exist (and
    /// includes sequence zero and an unknown mission).
    pub fn projection_at(
        &self,
        mission_id: &MissionId,
        sequence: u64,
    ) -> Result<Option<MissionProjection>, LedgerError> {
        if sequence == 0 {
            return Ok(None);
        }
        let connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        replay_at_from_connection(&connection, mission_id, sequence)
    }

    pub fn acquire_lease(
        &self,
        resource_key: &str,
        mission_id: &MissionId,
        task_id: &str,
        attempt_id: &str,
        owner: &str,
        ttl: Duration,
    ) -> Result<LeaseRecord, LedgerError> {
        validate_key(resource_key, "resource_key")?;
        validate_key(mission_id.as_str(), "mission_id")?;
        validate_key(task_id, "task_id")?;
        validate_key(attempt_id, "attempt_id")?;
        validate_key(owner, "lease owner")?;
        let mut connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let existing = read_lease(&transaction, resource_key)?;
        let now = Utc::now();
        if let Some(lease) = &existing {
            if lease.status == LeaseStatus::Active && lease.expires_at > now {
                if lease.owner == owner
                    && lease.mission_id == *mission_id
                    && lease.task_id == task_id
                    && lease.attempt_id == attempt_id
                {
                    transaction.commit()?;
                    return Ok(lease.clone());
                }
                return Err(LedgerError::LeaseHeld {
                    resource: resource_key.to_string(),
                    owner: lease.owner.clone(),
                    token: lease.fencing_token,
                });
            }
        }
        let token = existing
            .as_ref()
            .map(|lease| lease.fencing_token.saturating_add(1))
            .unwrap_or(1);
        let expires_at = now
            + ChronoDuration::from_std(ttl)
                .map_err(|error| LedgerError::InvalidInput(error.to_string()))?;
        let record = LeaseRecord {
            resource_key: resource_key.to_string(),
            mission_id: mission_id.clone(),
            task_id: task_id.to_string(),
            attempt_id: attempt_id.to_string(),
            owner: owner.to_string(),
            fencing_token: token,
            expires_at,
            status: LeaseStatus::Active,
        };
        transaction.execute(
            "INSERT INTO leases (
                resource_key, mission_id, task_id, attempt_id, owner,
                fencing_token, expires_at, status
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'active')
             ON CONFLICT(resource_key) DO UPDATE SET
                mission_id = excluded.mission_id,
                task_id = excluded.task_id,
                attempt_id = excluded.attempt_id,
                owner = excluded.owner,
                fencing_token = excluded.fencing_token,
                expires_at = excluded.expires_at,
                status = 'active'",
            params![
                record.resource_key,
                record.mission_id.as_str(),
                record.task_id,
                record.attempt_id,
                record.owner,
                as_i64(record.fencing_token)?,
                record.expires_at.to_rfc3339(),
            ],
        )?;
        transaction.commit()?;
        Ok(record)
    }

    pub fn renew_lease(
        &self,
        resource_key: &str,
        owner: &str,
        fencing_token: u64,
        ttl: Duration,
    ) -> Result<LeaseRecord, LedgerError> {
        let mut connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut lease =
            read_lease(&transaction, resource_key)?.ok_or_else(|| LedgerError::StaleFence {
                resource: resource_key.to_string(),
                expected: fencing_token,
                actual: None,
            })?;
        if lease.status != LeaseStatus::Active
            || lease.fencing_token != fencing_token
            || lease.owner != owner
            || lease.expires_at <= Utc::now()
        {
            return Err(LedgerError::StaleFence {
                resource: resource_key.to_string(),
                expected: fencing_token,
                actual: Some(lease.fencing_token),
            });
        }
        lease.expires_at = Utc::now()
            + ChronoDuration::from_std(ttl)
                .map_err(|error| LedgerError::InvalidInput(error.to_string()))?;
        transaction.execute(
            "UPDATE leases SET expires_at = ?1
             WHERE resource_key = ?2 AND owner = ?3
               AND fencing_token = ?4 AND status = 'active'",
            params![
                lease.expires_at.to_rfc3339(),
                resource_key,
                owner,
                as_i64(fencing_token)?,
            ],
        )?;
        transaction.commit()?;
        Ok(lease)
    }

    pub fn release_lease(&self, resource_key: &str, fencing_token: u64) -> Result<(), LedgerError> {
        let mut connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        assert_fence_tx(&transaction, resource_key, fencing_token)?;
        transaction.execute(
            "UPDATE leases SET status = 'released', expires_at = ?1
             WHERE resource_key = ?2 AND fencing_token = ?3",
            params![
                Utc::now().to_rfc3339(),
                resource_key,
                as_i64(fencing_token)?,
            ],
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub fn assert_fence(&self, resource_key: &str, fencing_token: u64) -> Result<(), LedgerError> {
        let connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        assert_fence_tx(&connection, resource_key, fencing_token)
    }

    /// Resolve every unexpired lease currently bound to one exact attempt.
    ///
    /// The deterministic resource order lets callers persist/retry the exact
    /// credential set. Mutation authority is still checked again atomically by
    /// [`MissionLedger::append`]; this read API is discovery, never a TOCTOU
    /// substitute for the transactional check.
    pub fn active_leases_for_attempt(
        &self,
        mission_id: &MissionId,
        task_id: &str,
        attempt_id: &str,
    ) -> Result<Vec<LeaseRecord>, LedgerError> {
        validate_key(mission_id.as_str(), "mission_id")?;
        validate_key(task_id, "task_id")?;
        validate_key(attempt_id, "attempt_id")?;
        let connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        read_active_leases_for_attempt(&connection, mission_id, task_id, attempt_id)
    }

    /// Claim pending effects for at-least-once delivery. A crash after a remote
    /// send and before `mark_outbox_delivered` may cause a duplicate; handlers
    /// must reconcile or record that possibility rather than claim exactly-once.
    pub fn claim_outbox(
        &self,
        worker: &str,
        limit: usize,
        claim_ttl: Duration,
    ) -> Result<Vec<OutboxRecord>, LedgerError> {
        let mut connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let now = Utc::now();
        let claim_until = now
            + ChronoDuration::from_std(claim_ttl)
                .map_err(|error| LedgerError::InvalidInput(error.to_string()))?;
        let ids = {
            let mut statement = transaction.prepare(
                "SELECT outbox_id FROM outbox
                 WHERE available_at <= ?1
                   AND (
                     status = 'pending'
                     OR (status = 'processing' AND claim_until < ?1)
                   )
                 ORDER BY available_at, outbox_id
                 LIMIT ?2",
            )?;
            let rows = statement
                .query_map(params![now.to_rfc3339(), as_i64(limit as u64)?], |row| {
                    row.get::<_, String>(0)
                })?;
            rows.collect::<Result<Vec<_>, _>>()?
        };
        for id in &ids {
            transaction.execute(
                "UPDATE outbox
                 SET status = 'processing', claim_owner = ?1, claim_until = ?2,
                     attempts = attempts + 1
                 WHERE outbox_id = ?3",
                params![worker, claim_until.to_rfc3339(), id],
            )?;
        }
        let mut records = Vec::new();
        for id in ids {
            records.push(read_outbox(&transaction, &id)?.ok_or_else(|| {
                LedgerError::InvalidInput(format!("claimed outbox row disappeared: {id}"))
            })?);
        }
        transaction.commit()?;
        Ok(records)
    }

    pub fn mark_outbox_delivered(
        &self,
        outbox_id: &str,
        worker: &str,
        remote_ref: Option<&str>,
    ) -> Result<(), LedgerError> {
        let connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        let changed = connection.execute(
            "UPDATE outbox
             SET status = 'delivered', remote_ref = ?1,
                 claim_owner = NULL, claim_until = NULL, last_error = NULL
             WHERE outbox_id = ?2 AND status = 'processing' AND claim_owner = ?3",
            params![remote_ref, outbox_id, worker],
        )?;
        if changed == 0 {
            return Err(LedgerError::OutboxClaimConflict(outbox_id.to_string()));
        }
        Ok(())
    }

    pub fn mark_outbox_retry(
        &self,
        outbox_id: &str,
        worker: &str,
        error: &str,
        retry_after: Duration,
    ) -> Result<(), LedgerError> {
        let available_at = Utc::now()
            + ChronoDuration::from_std(retry_after)
                .map_err(|failure| LedgerError::InvalidInput(failure.to_string()))?;
        let connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        let changed = connection.execute(
            "UPDATE outbox
             SET status = 'pending', available_at = ?1, last_error = ?2,
                 claim_owner = NULL, claim_until = NULL
             WHERE outbox_id = ?3 AND status = 'processing' AND claim_owner = ?4",
            params![available_at.to_rfc3339(), error, outbox_id, worker],
        )?;
        if changed == 0 {
            return Err(LedgerError::OutboxClaimConflict(outbox_id.to_string()));
        }
        Ok(())
    }
}

fn read_events(
    connection: &Connection,
    mission_id: &MissionId,
) -> Result<Vec<MissionEvent>, LedgerError> {
    let mut statement = connection.prepare(
        "SELECT event_id, mission_id, task_id, attempt_id, sequence,
                expected_version, schema_version, idempotency_key, command_digest, actor,
                provider, causation_id, correlation_id, fencing_token,
                plan_revision, activated_plan_revision, task_attempt_mutation_json,
                task_attempt_projection_json, recorded_at, kind, payload_json,
                mission_state_json
         FROM events WHERE mission_id = ?1 ORDER BY sequence ASC",
    )?;
    let rows = statement.query_map(params![mission_id.as_str()], event_row)?;
    let mut events = Vec::new();
    for row in rows {
        events.push(row??);
    }
    Ok(events)
}

fn replay_from_connection(
    connection: &Connection,
    mission_id: &MissionId,
) -> Result<MissionProjection, LedgerError> {
    Ok(replay_all_from_connection(connection, mission_id)?.mission)
}

#[derive(Debug, Clone, PartialEq)]
struct ReplayedProjection {
    mission: MissionProjection,
    task_attempts: BTreeMap<String, TaskAttemptProjection>,
}

fn replay_all_from_connection(
    connection: &Connection,
    mission_id: &MissionId,
) -> Result<ReplayedProjection, LedgerError> {
    let events = read_events(connection, mission_id)?;
    replay_events(connection, mission_id, events)
}

fn replay_at_from_connection(
    connection: &Connection,
    mission_id: &MissionId,
    sequence: u64,
) -> Result<Option<MissionProjection>, LedgerError> {
    let events: Vec<MissionEvent> = read_events(connection, mission_id)?
        .into_iter()
        .take_while(|event| event.sequence <= sequence)
        .collect();
    if events.last().map(|event| event.sequence) != Some(sequence) {
        return Ok(None);
    }
    replay_events(connection, mission_id, events)
        .map(|replayed| replayed.mission)
        .map(Some)
}

fn replay_events(
    connection: &Connection,
    mission_id: &MissionId,
    events: Vec<MissionEvent>,
) -> Result<ReplayedProjection, LedgerError> {
    if events.is_empty() {
        return Err(LedgerError::MissionNotFound(mission_id.0.clone()));
    }
    let mut state: Option<MissionState> = None;
    let mut version = 0_u64;
    let mut active_plan_revision: Option<u64> = None;
    let mut task_attempts = BTreeMap::new();
    let mut updated_at = events[0].recorded_at;
    for event in events {
        if event.mission_id != *mission_id {
            return Err(LedgerError::ReplayDivergence {
                mission_id: mission_id.0.clone(),
                reason: format!("event {} belongs to another mission", event.event_id),
            });
        }
        if event.schema_version != LEGACY_SCHEMA_VERSION && event.schema_version != SCHEMA_VERSION {
            return Err(LedgerError::ReplayDivergence {
                mission_id: mission_id.0.clone(),
                reason: format!(
                    "event {} uses unsupported schema {}",
                    event.event_id, event.schema_version
                ),
            });
        }
        let expected_sequence =
            version
                .checked_add(1)
                .ok_or_else(|| LedgerError::ReplayDivergence {
                    mission_id: mission_id.0.clone(),
                    reason: "event sequence overflow".to_string(),
                })?;
        if event.sequence != expected_sequence || event.expected_version != version {
            return Err(LedgerError::ReplayDivergence {
                mission_id: mission_id.0.clone(),
                reason: format!(
                    "event {} has sequence {} / expected_version {}; expected {} / {}",
                    event.event_id,
                    event.sequence,
                    event.expected_version,
                    expected_sequence,
                    version
                ),
            });
        }
        if let Some(next) = event.resulting_mission_state {
            state = Some(match state {
                None if next == MissionState::Created => next,
                Some(current) => current
                    .transition(next)
                    .map_err(LedgerError::InvalidTransition)?,
                _ => {
                    return Err(LedgerError::ReplayDivergence {
                        mission_id: mission_id.0.clone(),
                        reason: "first stateful event must create the mission".to_string(),
                    })
                }
            });
        }
        replay_task_attempt_event(
            connection,
            mission_id,
            &event,
            active_plan_revision,
            &mut task_attempts,
        )?;
        if let Some(revision) = event.activated_plan_revision {
            let expected_revision = active_plan_revision
                .unwrap_or(0)
                .checked_add(1)
                .ok_or_else(|| LedgerError::ReplayDivergence {
                    mission_id: mission_id.0.clone(),
                    reason: "active plan revision overflow".to_string(),
                })?;
            if revision != expected_revision {
                return Err(LedgerError::ReplayDivergence {
                    mission_id: mission_id.0.clone(),
                    reason: format!(
                        "event {} activates revision {revision}; expected {expected_revision}",
                        event.event_id
                    ),
                });
            }
            let plan = read_plan_contract(connection, mission_id, revision)?.ok_or_else(|| {
                LedgerError::ReplayDivergence {
                    mission_id: mission_id.0.clone(),
                    reason: format!(
                        "event {} activates missing plan revision {revision}",
                        event.event_id
                    ),
                }
            })?;
            if plan.created_from_version != event.expected_version {
                return Err(LedgerError::ReplayDivergence {
                    mission_id: mission_id.0.clone(),
                    reason: format!(
                        "plan revision {revision} was not created from activating event version {}",
                        event.expected_version
                    ),
                });
            }
            active_plan_revision = Some(revision);
        }
        version = event.sequence;
        updated_at = event.recorded_at;
    }
    let state = state.ok_or_else(|| LedgerError::ReplayDivergence {
        mission_id: mission_id.0.clone(),
        reason: "event stream contains no mission state".to_string(),
    })?;
    Ok(ReplayedProjection {
        mission: MissionProjection {
            mission_id: mission_id.clone(),
            state,
            version,
            active_plan_revision,
            projection_hash: projection_hash(mission_id, state, version, active_plan_revision)?,
            updated_at,
        },
        task_attempts,
    })
}

fn replay_task_attempt_event(
    connection: &Connection,
    mission_id: &MissionId,
    event: &MissionEvent,
    active_plan_revision: Option<u64>,
    task_attempts: &mut BTreeMap<String, TaskAttemptProjection>,
) -> Result<(), LedgerError> {
    let has_any_task_identity = event.task_id.is_some() || event.attempt_id.is_some();
    let has_complete_task_alias =
        event.task_id.is_some() && event.attempt_id.is_some() && event.plan_revision.is_some();
    let plan_bound_scope_observation = event.schema_version == SCHEMA_VERSION
        && !has_any_task_identity
        && event.plan_revision.is_some()
        && event.kind == "scope_claim_authority_prepared";
    let legacy_plan_revision_alias = event.schema_version == LEGACY_SCHEMA_VERSION
        && !has_any_task_identity
        && event.plan_revision.is_some()
        && event.plan_revision == event.activated_plan_revision;
    if (has_any_task_identity && !has_complete_task_alias)
        || (!has_any_task_identity
            && event.plan_revision.is_some()
            && !legacy_plan_revision_alias
            && !plan_bound_scope_observation)
    {
        return Err(replay_divergence(
            mission_id,
            event,
            "incomplete task_id/attempt_id/plan_revision aliases",
        ));
    }

    let has_any_authoritative_binding =
        event.task_attempt_mutation.is_some() || event.resulting_task_attempt.is_some();
    let has_full_authoritative_binding =
        event.task_attempt_mutation.is_some() && event.resulting_task_attempt.is_some();
    if has_any_authoritative_binding != has_full_authoritative_binding {
        return Err(replay_divergence(
            mission_id,
            event,
            "incomplete task-attempt mutation/projection authority",
        ));
    }

    if event.schema_version == LEGACY_SCHEMA_VERSION {
        if has_any_authoritative_binding {
            return Err(replay_divergence(
                mission_id,
                event,
                "schema-v1 event unexpectedly carries schema-v2 task-attempt authority",
            ));
        }
        if event.fencing_token.is_some() && !has_complete_task_alias {
            return Err(replay_divergence(
                mission_id,
                event,
                "legacy non-task event carries a fencing token",
            ));
        }
        // Schema-v1 task aliases are retained only as historical audit data.
        // They deliberately contribute no TaskAttemptProjection. The old
        // materialized row has been quarantined in legacy_task_attempts and a
        // new schema-v2 queued event is required before this id can influence
        // orchestration or delivery again.
        return Ok(());
    }

    let (mutation, resulting) = match (
        event.task_attempt_mutation.as_ref(),
        event.resulting_task_attempt.as_ref(),
    ) {
        (None, None) if plan_bound_scope_observation => {
            if event.plan_revision != active_plan_revision {
                return Err(replay_divergence(
                    mission_id,
                    event,
                    "scope observation is not bound to the active plan revision",
                ));
            }
            if event.fencing_token.is_some() {
                return Err(replay_divergence(
                    mission_id,
                    event,
                    "scope observation carries a fencing token",
                ));
            }
            return Ok(());
        }
        (None, None) if !has_any_task_identity && event.plan_revision.is_none() => {
            if event.fencing_token.is_some() {
                return Err(replay_divergence(
                    mission_id,
                    event,
                    "non-task event carries a fencing token",
                ));
            }
            return Ok(());
        }
        (Some(mutation), Some(resulting)) if has_complete_task_alias => (mutation, resulting),
        (None, None) => {
            return Err(replay_divergence(
                mission_id,
                event,
                "task aliases are not backed by an authoritative mutation/projection",
            ))
        }
        _ => {
            return Err(replay_divergence(
                mission_id,
                event,
                "task-attempt authority is not backed by complete aliases",
            ))
        }
    };

    if event.task_id.as_deref() != Some(mutation.task_id.as_str())
        || event.attempt_id.as_deref() != Some(mutation.attempt_id.as_str())
        || event.plan_revision != Some(mutation.plan_revision)
    {
        return Err(replay_divergence(
            mission_id,
            event,
            "task-attempt aliases differ from the immutable mutation",
        ));
    }
    if resulting.mission_id != *mission_id
        || resulting.task_id != mutation.task_id
        || resulting.attempt_id != mutation.attempt_id
        || resulting.plan_revision != mutation.plan_revision
    {
        return Err(replay_divergence(
            mission_id,
            event,
            "resulting task-attempt identity differs from its mutation",
        ));
    }

    let current = task_attempts.get(&mutation.attempt_id);
    validate_replayed_task_authorization(
        connection,
        mission_id,
        mutation,
        active_plan_revision,
        current,
        event,
    )?;
    let (next_state, next_version) = match current {
        None => {
            if mutation.expected_version != 0 || mutation.next_state != TaskAttemptState::Queued {
                return Err(replay_divergence(
                    mission_id,
                    event,
                    "new task attempt must bind expected_version 0 to queued/version 1",
                ));
            }
            (TaskAttemptState::Queued, 1)
        }
        Some(current) => {
            if current.version != mutation.expected_version {
                return Err(replay_divergence(
                    mission_id,
                    event,
                    format!(
                        "task attempt {} expected version {}, replay has {}",
                        mutation.attempt_id, mutation.expected_version, current.version
                    ),
                ));
            }
            if current.mission_id != *mission_id
                || current.task_id != mutation.task_id
                || current.plan_revision != mutation.plan_revision
            {
                return Err(replay_divergence(
                    mission_id,
                    event,
                    "task attempt identity or plan revision changed during replay",
                ));
            }
            let next_state = current
                .state
                .transition(mutation.next_state)
                .map_err(|error| {
                    replay_divergence(
                        mission_id,
                        event,
                        format!("invalid task-attempt transition: {error}"),
                    )
                })?;
            let next_version = current.version.checked_add(1).ok_or_else(|| {
                replay_divergence(mission_id, event, "task-attempt version overflow")
            })?;
            (next_state, next_version)
        }
    };
    let expected = TaskAttemptProjection {
        mission_id: mission_id.clone(),
        task_id: mutation.task_id.clone(),
        attempt_id: mutation.attempt_id.clone(),
        plan_revision: mutation.plan_revision,
        state: next_state,
        version: next_version,
        fencing_token: event.fencing_token,
        updated_at: event.recorded_at,
    };
    if *resulting != expected {
        return Err(replay_divergence(
            mission_id,
            event,
            format!(
                "resulting task attempt {:?} differs from replay-derived {:?}",
                resulting, expected
            ),
        ));
    }
    task_attempts.insert(mutation.attempt_id.clone(), expected);
    Ok(())
}

fn validate_replayed_task_authorization(
    connection: &Connection,
    mission_id: &MissionId,
    mutation: &TaskAttemptMutation,
    active_plan_revision: Option<u64>,
    existing: Option<&TaskAttemptProjection>,
    event: &MissionEvent,
) -> Result<(), LedgerError> {
    let active_revision = active_plan_revision.ok_or_else(|| {
        replay_divergence(
            mission_id,
            event,
            "task attempt was recorded before an active plan existed",
        )
    })?;
    let active_plan =
        read_plan_contract(connection, mission_id, active_revision)?.ok_or_else(|| {
            replay_divergence(
                mission_id,
                event,
                format!("active plan revision {active_revision} is missing"),
            )
        })?;
    let active_task = active_plan
        .tasks
        .iter()
        .find(|task| task.task_id.as_str() == mutation.task_id);

    if existing.is_none() {
        if mutation.plan_revision != active_revision || active_task.is_none() {
            return Err(replay_divergence(
                mission_id,
                event,
                format!(
                    "new task {} is not declared in active plan revision {active_revision}",
                    mutation.task_id
                ),
            ));
        }
        return Ok(());
    }

    let attempt_plan = read_plan_contract(connection, mission_id, mutation.plan_revision)?
        .ok_or_else(|| {
            replay_divergence(
                mission_id,
                event,
                format!(
                    "attempt plan revision {} is missing",
                    mutation.plan_revision
                ),
            )
        })?;
    let attempt_task = attempt_plan
        .tasks
        .iter()
        .find(|task| task.task_id.as_str() == mutation.task_id)
        .ok_or_else(|| {
            replay_divergence(
                mission_id,
                event,
                format!(
                    "task {} is missing from attempt plan revision {}",
                    mutation.task_id, mutation.plan_revision
                ),
            )
        })?;
    if active_task != Some(attempt_task) {
        return Err(replay_divergence(
            mission_id,
            event,
            format!(
                "task {} is no longer unchanged in active plan revision {active_revision}",
                mutation.task_id
            ),
        ));
    }
    Ok(())
}

fn replay_divergence(
    mission_id: &MissionId,
    event: &MissionEvent,
    reason: impl fmt::Display,
) -> LedgerError {
    LedgerError::ReplayDivergence {
        mission_id: mission_id.0.clone(),
        reason: format!("event {}: {reason}", event.event_id),
    }
}

fn verify_projection_coherence(
    connection: &Connection,
    projection: &MissionProjection,
) -> Result<(), LedgerError> {
    let replayed = replay_all_from_connection(connection, &projection.mission_id)?;
    if projection.state != replayed.mission.state
        || projection.version != replayed.mission.version
        || projection.active_plan_revision != replayed.mission.active_plan_revision
        || projection.projection_hash != replayed.mission.projection_hash
        || projection.updated_at != replayed.mission.updated_at
    {
        return Err(LedgerError::ReplayDivergence {
            mission_id: projection.mission_id.0.clone(),
            reason: format!(
                "stored state/version/plan/hash/time = {:?}/{}/{:?}/{}/{}; replay = {:?}/{}/{:?}/{}/{}",
                projection.state,
                projection.version,
                projection.active_plan_revision,
                projection.projection_hash,
                projection.updated_at.to_rfc3339(),
                replayed.mission.state,
                replayed.mission.version,
                replayed.mission.active_plan_revision,
                replayed.mission.projection_hash,
                replayed.mission.updated_at.to_rfc3339()
            ),
        });
    }
    let materialized = read_task_attempts_for_mission(connection, &projection.mission_id)?
        .into_iter()
        .map(|attempt| (attempt.attempt_id.clone(), attempt))
        .collect::<BTreeMap<_, _>>();
    if materialized != replayed.task_attempts {
        return Err(LedgerError::ReplayDivergence {
            mission_id: projection.mission_id.0.clone(),
            reason: format!(
                "materialized task attempts differ from event replay (stored {}, replayed {}, stored_digest {}, replayed_digest {})",
                materialized.len(),
                replayed.task_attempts.len(),
                canonical_digest(&materialized)?,
                canonical_digest(&replayed.task_attempts)?,
            ),
        });
    }
    Ok(())
}

fn verify_all_projection_coherence(connection: &Connection) -> Result<(), LedgerError> {
    let mission_ids = {
        let mut statement =
            connection.prepare("SELECT mission_id FROM missions ORDER BY mission_id")?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
        rows.collect::<Result<Vec<_>, _>>()?
    };
    for mission_id in mission_ids {
        let projection = read_projection(connection, &mission_id)?
            .ok_or_else(|| LedgerError::MissionNotFound(mission_id.clone()))?;
        verify_projection_coherence(connection, &projection)?;
    }

    let orphan: Option<(String, String)> = connection
        .query_row(
            "SELECT attempts.attempt_id, attempts.mission_id
             FROM task_attempts AS attempts
             LEFT JOIN missions ON missions.mission_id = attempts.mission_id
             WHERE missions.mission_id IS NULL
             ORDER BY attempts.attempt_id LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    if let Some((attempt_id, mission_id)) = orphan {
        return Err(LedgerError::ReplayDivergence {
            mission_id,
            reason: format!(
                "orphan materialized task attempt {attempt_id} has no mission projection"
            ),
        });
    }
    Ok(())
}

fn ensure_idempotent_match(
    recorded: &MissionEvent,
    supplied_digest: &str,
) -> Result<(), LedgerError> {
    if !recorded.command_digest.is_empty() && recorded.command_digest == supplied_digest {
        return Ok(());
    }
    Err(LedgerError::IdempotencyConflict {
        mission_id: recorded.mission_id.0.clone(),
        key: recorded.idempotency_key.clone(),
        recorded_digest: if recorded.command_digest.is_empty() {
            "legacy-unbound".to_string()
        } else {
            recorded.command_digest.clone()
        },
        supplied_digest: supplied_digest.to_string(),
    })
}

fn stable_id(prefix: &str) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    let (wall_direction, wall_nanos) = crate::mission::wall_clock_nanos();
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"omega.ledger-id.v2");
    hasher.update(&(prefix.len() as u64).to_le_bytes());
    hasher.update(prefix.as_bytes());
    hasher.update(crate::mission::process_discriminator());
    hasher.update(&std::process::id().to_le_bytes());
    hasher.update(&counter.to_le_bytes());
    hasher.update(&[wall_direction]);
    hasher.update(&wall_nanos.to_le_bytes());
    let encoded = hasher.finalize().to_hex();
    format!("{prefix}-{}", &encoded[..32])
}

fn validate_key(value: &str, name: &str) -> Result<(), LedgerError> {
    if value.trim().is_empty() {
        Err(LedgerError::InvalidInput(format!(
            "{name} must not be empty"
        )))
    } else {
        Ok(())
    }
}

fn as_i64(value: u64) -> Result<i64, LedgerError> {
    i64::try_from(value)
        .map_err(|_| LedgerError::InvalidInput(format!("value exceeds SQLite INTEGER: {value}")))
}

fn as_u64(value: i64) -> Result<u64, LedgerError> {
    u64::try_from(value)
        .map_err(|_| LedgerError::InvalidInput(format!("negative SQLite INTEGER: {value}")))
}

fn projection_hash(
    mission_id: &MissionId,
    state: MissionState,
    version: u64,
    plan_revision: Option<u64>,
) -> Result<String, LedgerError> {
    let bytes = serde_json::to_vec(&(mission_id, state, version, plan_revision))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn validate_plan_activation(
    transaction: &Transaction<'_>,
    current: &MissionProjection,
    plan: &PlanContract,
) -> Result<(), LedgerError> {
    let expected_revision = current
        .active_plan_revision
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(|| LedgerError::InvalidInput("plan revision overflow".to_string()))?;
    if plan.revision != expected_revision {
        return Err(LedgerError::InvalidInput(format!(
            "activated plan revision must advance from {:?} to {expected_revision}, got {}",
            current.active_plan_revision, plan.revision
        )));
    }

    // The ledger derives protection from durable attempts, independently of
    // whichever helper the caller used to construct the new PlanContract.
    let protected = {
        let mut statement = transaction.prepare(
            "SELECT DISTINCT task_id, plan_revision
             FROM task_attempts WHERE mission_id = ?1
             ORDER BY task_id, plan_revision",
        )?;
        let rows = statement.query_map(params![current.mission_id.as_str()], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        rows.collect::<Result<Vec<_>, _>>()?
    };
    for (task_id, source_revision) in protected {
        let source_revision = as_u64(source_revision)?;
        let source_plan = read_plan_contract(transaction, &current.mission_id, source_revision)?
            .ok_or_else(|| {
                LedgerError::InvalidInput(format!(
                    "protected task {task_id} refers to missing plan revision {source_revision}"
                ))
            })?;
        let original = source_plan
            .tasks
            .iter()
            .find(|task| task.task_id.as_str() == task_id)
            .ok_or_else(|| {
                LedgerError::InvalidInput(format!(
                    "protected task {task_id} is missing from source plan revision {source_revision}"
                ))
            })?;
        let replacement = plan
            .tasks
            .iter()
            .find(|task| task.task_id.as_str() == task_id)
            .ok_or_else(|| {
                LedgerError::InvalidInput(format!("protected task removed: {task_id}"))
            })?;
        if replacement != original {
            return Err(LedgerError::InvalidInput(format!(
                "protected task changed: {task_id}"
            )));
        }
    }
    Ok(())
}

fn acceptance_error(message: impl Into<String>) -> LedgerError {
    LedgerError::InvalidInput(format!("mission acceptance rejected: {}", message.into()))
}

fn looks_like_digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn validate_evidence_event_ids(
    events: &[MissionEvent],
    observation_sequence: u64,
    evidence_event_ids: &[String],
) -> Result<(), LedgerError> {
    if evidence_event_ids.is_empty() {
        return Err(acceptance_error(
            "requirement observation has no evidence events",
        ));
    }
    let unique: BTreeSet<_> = evidence_event_ids.iter().collect();
    if unique.len() != evidence_event_ids.len() {
        return Err(acceptance_error(
            "requirement observation repeats an evidence event",
        ));
    }
    for event_id in evidence_event_ids {
        let evidence = events
            .iter()
            .find(|event| event.event_id == *event_id)
            .ok_or_else(|| acceptance_error(format!("evidence event {event_id} is missing")))?;
        if evidence.sequence >= observation_sequence {
            return Err(acceptance_error(format!(
                "evidence event {event_id} does not precede its observation"
            )));
        }
    }
    Ok(())
}

fn valid_verification_for_attempt(
    events: &[MissionEvent],
    plan: &PlanContract,
    task: &crate::mission::TaskContract,
    attempt: &TaskAttemptProjection,
) -> Result<Option<u64>, LedgerError> {
    let accepted_sequence = events
        .iter()
        .filter(|event| {
            event.resulting_task_attempt.as_ref().is_some_and(|result| {
                result.attempt_id == attempt.attempt_id
                    && result.state == TaskAttemptState::Accepted
            })
        })
        .map(|event| event.sequence)
        .max();
    let Some(accepted_sequence) = accepted_sequence else {
        return Ok(None);
    };

    let mut valid_record = false;
    for event in events
        .iter()
        .filter(|event| event.kind == "task_verifier_observations_recorded")
    {
        let record: RecordedContractVerification = serde_json::from_value(event.payload.clone())
            .map_err(|error| {
                acceptance_error(format!(
                    "corrupt task verifier observation {}: {error}",
                    event.event_id
                ))
            })?;
        if record.mission_id != plan.mission_id.as_str()
            || record.task_id != task.task_id.as_str()
            || record.attempt_id != attempt.attempt_id
            || record.plan_revision != plan.revision
        {
            continue;
        }
        let expected_checks: BTreeSet<_> = task
            .verifier_checks
            .iter()
            .map(|check| check.check_id.as_str())
            .collect();
        let observed_checks: BTreeSet<_> = record
            .verification
            .observations
            .iter()
            .map(|observation| observation.check_id.as_str())
            .collect();
        let exact_checks = observed_checks.len() == record.verification.observations.len()
            && observed_checks == expected_checks;
        if record.schema_version == ACCEPTANCE_OBSERVATION_SCHEMA_VERSION
            && record.plan_digest == plan.content_digest
            && looks_like_digest(&record.worker_signal_digest)
            && record.verification.passed
            && record.verification.failures.is_empty()
            && exact_checks
            && record
                .verification
                .observations
                .iter()
                .all(|observation| observation.passed && !observation.detail.trim().is_empty())
            && event.actor == "omega-independent-verifier"
            && event.sequence < accepted_sequence
        {
            valid_record = true;
        }
    }
    if events.iter().any(|event| {
        event.kind == "task_acceptance_invalidated"
            && serde_json::from_value::<Value>(event.payload.clone())
                .ok()
                .and_then(|payload| {
                    payload
                        .get("attempt_id")
                        .and_then(Value::as_str)
                        .map(str::to_string)
                })
                .as_deref()
                == Some(attempt.attempt_id.as_str())
    }) {
        return Ok(None);
    }
    Ok(valid_record.then_some(accepted_sequence))
}

fn validate_requirement_observation(
    events: &[MissionEvent],
    plan: &PlanContract,
    requirement: &str,
    kind: PlanRequirementKind,
) -> Result<(), LedgerError> {
    let expected_event_kind = match kind {
        PlanRequirementKind::Gate => "plan_gate_observed",
        PlanRequirementKind::Approval => "plan_approval_observed",
    };
    let mut latest = None;
    for event in events
        .iter()
        .filter(|event| event.kind == expected_event_kind)
    {
        let observation: PlanRequirementObservation = serde_json::from_value(event.payload.clone())
            .map_err(|error| {
                acceptance_error(format!(
                    "corrupt {expected_event_kind} event {}: {error}",
                    event.event_id
                ))
            })?;
        if observation.mission_id == plan.mission_id.as_str()
            && observation.plan_revision == plan.revision
            && observation.plan_digest == plan.content_digest
            && observation.requirement == requirement
            && observation.kind == kind
        {
            latest = Some((event, observation));
        }
    }
    let Some((event, observation)) = latest else {
        return Err(acceptance_error(format!(
            "required {:?} `{requirement}` has no exact plan-bound observation",
            kind
        )));
    };
    if observation.schema_version != ACCEPTANCE_OBSERVATION_SCHEMA_VERSION
        || !observation.passed
        || observation.observed_by.trim().is_empty()
    {
        return Err(acceptance_error(format!(
            "required {:?} `{requirement}` did not pass",
            kind
        )));
    }
    validate_evidence_event_ids(events, event.sequence, &observation.evidence_event_ids)
}

fn validate_mission_gate_observation(
    events: &[MissionEvent],
    plan: &PlanContract,
) -> Result<u64, LedgerError> {
    let mut latest = None;
    for event in events
        .iter()
        .filter(|event| event.kind == "mission_gate_observed")
    {
        let observation: MissionGateObservation = serde_json::from_value(event.payload.clone())
            .map_err(|error| {
                acceptance_error(format!(
                    "corrupt mission gate observation {}: {error}",
                    event.event_id
                ))
            })?;
        if observation.mission_id == plan.mission_id.as_str()
            && observation.plan_revision == plan.revision
            && observation.plan_digest == plan.content_digest
        {
            latest = Some((event, observation));
        }
    }
    let Some((gate_event, observation)) = latest else {
        return Err(acceptance_error(
            "no exact plan-bound mission_gate_observed event",
        ));
    };
    if observation.schema_version != ACCEPTANCE_OBSERVATION_SCHEMA_VERSION
        || !observation.overall_pass
        || !looks_like_digest(&observation.gate_result_digest)
        || observation.checks.len() < 12
    {
        return Err(acceptance_error(
            "mission gate is missing, failed, malformed, or has fewer than 12 checks",
        ));
    }
    let check_ids: BTreeSet<_> = observation
        .checks
        .iter()
        .map(|check| check.check_id.as_str())
        .collect();
    let fact_digests: BTreeSet<_> = observation
        .checks
        .iter()
        .map(|check| check.fact_digest.as_str())
        .collect();
    if check_ids.len() != observation.checks.len()
        || fact_digests.len() != observation.checks.len()
        || observation
            .checks
            .iter()
            .any(|check| !check.passed || !looks_like_digest(&check.fact_digest))
    {
        return Err(acceptance_error(
            "mission gate checks are duplicated, unfalsifiable, or failing",
        ));
    }
    let evidence_event_ids: BTreeSet<_> = observation
        .checks
        .iter()
        .map(|check| check.evidence_event_id.as_str())
        .collect();
    if evidence_event_ids.len() != observation.checks.len() {
        return Err(acceptance_error(
            "mission gate checks do not cite distinct immutable evidence events",
        ));
    }
    for check in &observation.checks {
        let evidence = events
            .iter()
            .find(|event| event.event_id == check.evidence_event_id)
            .ok_or_else(|| {
                acceptance_error(format!(
                    "mission gate evidence event {} is missing",
                    check.evidence_event_id
                ))
            })?;
        if evidence.sequence != check.evidence_sequence
            || evidence.sequence >= gate_event.sequence
            || check.detail.trim().is_empty()
            || mission_gate_fact_digest(&check.check_id, check.passed, &check.detail, evidence)?
                != check.fact_digest
        {
            return Err(acceptance_error(format!(
                "mission gate check {} has invalid event evidence",
                check.check_id
            )));
        }
    }

    let expected_lenses = ["code_reviewer", "debugger", "general_purpose"];
    let lens_names: BTreeSet<_> = observation
        .lenses
        .iter()
        .map(|lens| lens.lens.as_str())
        .collect();
    if observation.lenses.len() != expected_lenses.len()
        || lens_names != expected_lenses.into_iter().collect()
        || observation
            .lenses
            .iter()
            .any(|lens| !lens.passed || lens.fact_ids.is_empty())
    {
        return Err(acceptance_error(
            "mission gate does not contain three passing independent lenses",
        ));
    }
    let mut lens_fact_sets = Vec::new();
    for lens in &observation.lenses {
        let facts: BTreeSet<_> = lens.fact_ids.iter().map(String::as_str).collect();
        if facts.len() != lens.fact_ids.len() || !facts.iter().all(|fact| check_ids.contains(*fact))
        {
            return Err(acceptance_error(format!(
                "mission gate lens {} cites unknown or duplicate facts",
                lens.lens
            )));
        }
        lens_fact_sets.push(facts);
    }
    if lens_fact_sets[0] == lens_fact_sets[1]
        || lens_fact_sets[0] == lens_fact_sets[2]
        || lens_fact_sets[1] == lens_fact_sets[2]
    {
        return Err(acceptance_error(
            "mission gate lenses consume identical evidence sets",
        ));
    }
    if mission_gate_result_digest(&observation)? != observation.gate_result_digest {
        return Err(acceptance_error(
            "mission gate result digest does not bind the exact checks and lenses",
        ));
    }
    Ok(gate_event.sequence)
}

fn validate_mission_issue_resolution(
    events: &[MissionEvent],
    plan: &PlanContract,
    gate_sequence: u64,
) -> Result<(), LedgerError> {
    for issue in events.iter().filter(|event| {
        matches!(
            event.kind.as_str(),
            "mission_acceptance_invalidated" | "mission_blocker_recorded"
        ) && event
            .plan_revision
            .is_none_or(|revision| revision == plan.revision)
    }) {
        let expected_kind = if issue.kind == "mission_acceptance_invalidated" {
            "mission_acceptance_revalidated"
        } else {
            "mission_blocker_resolved"
        };
        let resolved = events.iter().any(|event| {
            if event.kind != expected_kind
                || event.sequence <= issue.sequence
                || event.sequence >= gate_sequence
            {
                return false;
            }
            serde_json::from_value::<MissionIssueResolution>(event.payload.clone()).is_ok_and(
                |resolution| {
                    resolution.schema_version == ACCEPTANCE_OBSERVATION_SCHEMA_VERSION
                        && resolution.mission_id == plan.mission_id.as_str()
                        && resolution.plan_revision == plan.revision
                        && resolution.plan_digest == plan.content_digest
                        && resolution.issue_event_id == issue.event_id
                        && !resolution.resolved_by.trim().is_empty()
                        && !resolution.detail.trim().is_empty()
                },
            )
        });
        if !resolved {
            return Err(acceptance_error(format!(
                "mission issue {} ({}) has no exact plan-bound resolution before the current gate",
                issue.event_id, issue.kind
            )));
        }
    }
    Ok(())
}

fn validate_mission_acceptance_connection(
    transaction: &Connection,
    mission_id: &MissionId,
) -> Result<(), LedgerError> {
    let projection = read_projection(transaction, mission_id.as_str())?
        .ok_or_else(|| LedgerError::MissionNotFound(mission_id.0.clone()))?;
    let revision = projection
        .active_plan_revision
        .ok_or_else(|| acceptance_error("mission has no active immutable plan revision"))?;
    let plan = read_plan_contract(transaction, mission_id, revision)?
        .ok_or_else(|| acceptance_error("active immutable plan row is missing"))?;
    plan.verify_integrity()
        .map_err(|error| acceptance_error(format!("plan integrity failed: {error}")))?;
    let attempts = read_task_attempts_for_mission(transaction, mission_id)?;
    let events = read_events(transaction, mission_id)?;

    let mut accepted_sequences = BTreeMap::new();
    for task in &plan.tasks {
        let task_attempts: Vec<_> = attempts
            .iter()
            .filter(|attempt| {
                attempt.task_id == task.task_id.as_str() && attempt.plan_revision == plan.revision
            })
            .collect();
        if task_attempts
            .iter()
            .any(|attempt| !attempt.state.is_terminal())
        {
            return Err(acceptance_error(format!(
                "task {} still has a nonterminal attempt",
                task.task_id.as_str()
            )));
        }
        let latest_attempt = task_attempts
            .iter()
            .max_by_key(|attempt| {
                events
                    .iter()
                    .filter(|event| {
                        event
                            .resulting_task_attempt
                            .as_ref()
                            .is_some_and(|result| result.attempt_id == attempt.attempt_id)
                    })
                    .map(|event| event.sequence)
                    .max()
                    .unwrap_or(0)
            })
            .copied()
            .ok_or_else(|| {
                acceptance_error(format!(
                    "task {} has no attempt in the active plan",
                    task.task_id.as_str()
                ))
            })?;
        if latest_attempt.state != TaskAttemptState::Accepted {
            return Err(acceptance_error(format!(
                "task {} latest authoritative attempt {} is {:?}, not accepted",
                task.task_id.as_str(),
                latest_attempt.attempt_id,
                latest_attempt.state
            )));
        }
        let mut accepted = Vec::new();
        for attempt in task_attempts
            .into_iter()
            .filter(|attempt| attempt.state == TaskAttemptState::Accepted)
        {
            if let Some(sequence) = valid_verification_for_attempt(&events, &plan, task, attempt)? {
                accepted.push((attempt.attempt_id.as_str(), sequence));
            }
        }
        if accepted.len() != 1 || accepted[0].0 != latest_attempt.attempt_id {
            return Err(acceptance_error(format!(
                "task {} does not have one unambiguous latest accepted attempt with a passing recorded verifier observation",
                task.task_id.as_str()
            )));
        }
        let sequence = accepted[0].1;
        accepted_sequences.insert(task.task_id.as_str(), sequence);
    }
    for task in &plan.tasks {
        let task_sequence = accepted_sequences[task.task_id.as_str()];
        for dependency in &task.depends_on {
            let dependency_sequence = accepted_sequences
                .get(dependency.as_str())
                .ok_or_else(|| acceptance_error("accepted dependency projection is missing"))?;
            if *dependency_sequence >= task_sequence {
                return Err(acceptance_error(format!(
                    "task {} was accepted before dependency {}",
                    task.task_id.as_str(),
                    dependency.as_str()
                )));
            }
        }
    }

    for gate in &plan.required_gates {
        validate_requirement_observation(&events, &plan, gate, PlanRequirementKind::Gate)?;
    }
    for approval in &plan.required_approvals {
        validate_requirement_observation(&events, &plan, approval, PlanRequirementKind::Approval)?;
    }
    let active_lease_count: i64 = transaction.query_row(
        "SELECT COUNT(*) FROM leases
         WHERE mission_id = ?1 AND status = 'active' AND expires_at > ?2",
        params![mission_id.as_str(), Utc::now().to_rfc3339()],
        |row| row.get(0),
    )?;
    if active_lease_count != 0 {
        return Err(acceptance_error(format!(
            "{active_lease_count} active task lease(s) remain"
        )));
    }

    let gate_sequence = validate_mission_gate_observation(&events, &plan)?;
    validate_mission_issue_resolution(&events, &plan, gate_sequence)
}

fn persist_plan(transaction: &Transaction<'_>, plan: &PlanContract) -> Result<(), LedgerError> {
    let previous: Option<i64> = transaction
        .query_row(
            "SELECT MAX(revision) FROM plans WHERE mission_id = ?1",
            params![plan.mission_id.as_str()],
            |row| row.get(0),
        )
        .optional()?
        .flatten();
    if let Some(previous) = previous {
        let previous = as_u64(previous)?;
        if plan.revision != previous.saturating_add(1) {
            return Err(LedgerError::InvalidInput(format!(
                "plan revision must advance contiguously: previous {previous}, new {}",
                plan.revision
            )));
        }
    } else if plan.revision != 1 {
        return Err(LedgerError::InvalidInput(format!(
            "first plan revision must be 1, got {}",
            plan.revision
        )));
    }
    transaction.execute(
        "INSERT INTO plans (
            mission_id, plan_id, revision, contract_json, content_digest
         ) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            plan.mission_id.as_str(),
            plan.plan_id.0,
            as_i64(plan.revision)?,
            serde_json::to_string(plan)?,
            plan.content_digest,
        ],
    )?;
    Ok(())
}

fn read_plan_contract(
    connection: &Connection,
    mission_id: &MissionId,
    revision: u64,
) -> Result<Option<PlanContract>, LedgerError> {
    let stored: Option<(String, String)> = connection
        .query_row(
            "SELECT contract_json, content_digest
             FROM plans WHERE mission_id = ?1 AND revision = ?2",
            params![mission_id.as_str(), as_i64(revision)?],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    let Some((contract_json, stored_digest)) = stored else {
        return Ok(None);
    };
    let contract: PlanContract = serde_json::from_str(&contract_json)?;
    if contract.mission_id != *mission_id || contract.revision != revision {
        return Err(LedgerError::InvalidInput(format!(
            "plan row identity mismatch for mission {} revision {revision}",
            mission_id.as_str()
        )));
    }
    contract
        .verify_integrity()
        .map_err(|error| LedgerError::InvalidInput(error.to_string()))?;
    if stored_digest != contract.content_digest {
        return Err(LedgerError::InvalidInput(format!(
            "stored digest differs from plan contract digest for mission {} revision {revision}",
            mission_id.as_str()
        )));
    }
    Ok(Some(contract))
}

fn active_plan_revision(
    connection: &Connection,
    mission_id: &MissionId,
) -> Result<Option<u64>, LedgerError> {
    let revision: Option<i64> = connection
        .query_row(
            "SELECT active_plan_revision FROM missions WHERE mission_id = ?1",
            params![mission_id.as_str()],
            |row| row.get(0),
        )
        .optional()?
        .flatten();
    revision.map(as_u64).transpose()
}

fn validate_task_authorization(
    transaction: &Transaction<'_>,
    mission_id: &MissionId,
    mutation: &TaskAttemptMutation,
    existing: Option<&TaskAttemptProjection>,
) -> Result<(), LedgerError> {
    let active_revision = active_plan_revision(transaction, mission_id)?;
    let Some(active_revision) = active_revision else {
        return Err(LedgerError::PlanRevisionNotActive {
            mission_id: mission_id.0.clone(),
            supplied: mutation.plan_revision,
            active: None,
        });
    };
    let active_plan =
        read_plan_contract(transaction, mission_id, active_revision)?.ok_or_else(|| {
            LedgerError::InvalidInput(format!(
                "active plan revision {active_revision} is missing for mission {}",
                mission_id.as_str()
            ))
        })?;
    let active_task = active_plan
        .tasks
        .iter()
        .find(|task| task.task_id.as_str() == mutation.task_id.as_str());

    if existing.is_none() {
        if mutation.plan_revision != active_revision {
            return Err(LedgerError::PlanRevisionNotActive {
                mission_id: mission_id.0.clone(),
                supplied: mutation.plan_revision,
                active: Some(active_revision),
            });
        }
        if active_task.is_none() {
            return Err(LedgerError::TaskNotInPlan {
                mission_id: mission_id.0.clone(),
                revision: active_revision,
                task_id: mutation.task_id.clone(),
            });
        }
        return Ok(());
    }

    let attempt_plan = read_plan_contract(transaction, mission_id, mutation.plan_revision)?
        .ok_or_else(|| LedgerError::TaskNotInPlan {
            mission_id: mission_id.0.clone(),
            revision: mutation.plan_revision,
            task_id: mutation.task_id.clone(),
        })?;
    let attempt_task = attempt_plan
        .tasks
        .iter()
        .find(|task| task.task_id.as_str() == mutation.task_id.as_str())
        .ok_or_else(|| LedgerError::TaskNotInPlan {
            mission_id: mission_id.0.clone(),
            revision: mutation.plan_revision,
            task_id: mutation.task_id.clone(),
        })?;
    match active_task {
        Some(task) if task == attempt_task => Ok(()),
        _ => Err(LedgerError::TaskNoLongerActive {
            mission_id: mission_id.0.clone(),
            task_id: mutation.task_id.clone(),
            attempt_revision: mutation.plan_revision,
            active_revision,
        }),
    }
}

fn apply_task_mutation(
    transaction: &Transaction<'_>,
    mission_id: &MissionId,
    mutation: &TaskAttemptMutation,
    fencing_token: Option<u64>,
    now: DateTime<Utc>,
) -> Result<TaskAttemptProjection, LedgerError> {
    let existing = read_task_attempt(transaction, &mutation.attempt_id)?;
    validate_task_authorization(transaction, mission_id, mutation, existing.as_ref())?;
    let (next_state, next_version) = match existing {
        None => {
            if mutation.expected_version != 0 {
                return Err(LedgerError::AttemptVersionConflict {
                    attempt_id: mutation.attempt_id.clone(),
                    expected: mutation.expected_version,
                    actual: 0,
                });
            }
            if mutation.next_state != TaskAttemptState::Queued {
                return Err(LedgerError::InvalidInput(
                    "a new task attempt must start in queued".to_string(),
                ));
            }
            (TaskAttemptState::Queued, 1)
        }
        Some(ref current) => {
            if current.version != mutation.expected_version {
                return Err(LedgerError::AttemptVersionConflict {
                    attempt_id: mutation.attempt_id.clone(),
                    expected: mutation.expected_version,
                    actual: current.version,
                });
            }
            if current.mission_id != *mission_id
                || current.task_id != mutation.task_id
                || current.plan_revision != mutation.plan_revision
            {
                return Err(LedgerError::InvalidInput(
                    "task attempt identity or plan revision changed".to_string(),
                ));
            }
            (
                current
                    .state
                    .transition(mutation.next_state)
                    .map_err(LedgerError::InvalidTaskTransition)?,
                current.version.checked_add(1).ok_or_else(|| {
                    LedgerError::InvalidInput("task-attempt version overflow".to_string())
                })?,
            )
        }
    };
    let projection = TaskAttemptProjection {
        mission_id: mission_id.clone(),
        task_id: mutation.task_id.clone(),
        attempt_id: mutation.attempt_id.clone(),
        plan_revision: mutation.plan_revision,
        state: next_state,
        version: next_version,
        fencing_token,
        updated_at: now,
    };
    transaction.execute(
        "INSERT INTO task_attempts (
            attempt_id, mission_id, task_id, plan_revision, state_json,
            version, fencing_token, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(attempt_id) DO UPDATE SET
            state_json = excluded.state_json,
            version = excluded.version,
            fencing_token = excluded.fencing_token,
            updated_at = excluded.updated_at",
        params![
            projection.attempt_id,
            projection.mission_id.as_str(),
            projection.task_id,
            as_i64(projection.plan_revision)?,
            serde_json::to_string(&projection.state)?,
            as_i64(projection.version)?,
            projection.fencing_token.map(as_i64).transpose()?,
            projection.updated_at.to_rfc3339(),
        ],
    )?;
    Ok(projection)
}

fn insert_event(transaction: &Transaction<'_>, event: &MissionEvent) -> Result<(), LedgerError> {
    transaction.execute(
        "INSERT INTO events (
            event_id, mission_id, task_id, attempt_id, sequence,
            expected_version, schema_version, idempotency_key, command_digest, actor,
            provider, causation_id, correlation_id, fencing_token,
            plan_revision, activated_plan_revision, task_attempt_mutation_json,
            task_attempt_projection_json, recorded_at, kind, payload_json,
            mission_state_json
         ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
            ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20,
            ?21, ?22
         )",
        params![
            event.event_id,
            event.mission_id.as_str(),
            event.task_id,
            event.attempt_id,
            as_i64(event.sequence)?,
            as_i64(event.expected_version)?,
            i64::from(event.schema_version),
            event.idempotency_key,
            event.command_digest,
            event.actor,
            event.provider,
            event.causation_id,
            event.correlation_id,
            event.fencing_token.map(as_i64).transpose()?,
            event.plan_revision.map(as_i64).transpose()?,
            event.activated_plan_revision.map(as_i64).transpose()?,
            event
                .task_attempt_mutation
                .as_ref()
                .map(serde_json::to_string)
                .transpose()?,
            event
                .resulting_task_attempt
                .as_ref()
                .map(serde_json::to_string)
                .transpose()?,
            event.recorded_at.to_rfc3339(),
            event.kind,
            serde_json::to_string(&event.payload)?,
            event
                .resulting_mission_state
                .map(|state| serde_json::to_string(&state))
                .transpose()?,
        ],
    )?;
    Ok(())
}

fn insert_outbox(
    transaction: &Transaction<'_>,
    event: &MissionEvent,
    effect: &NewOutboxEffect,
    index: usize,
    now: DateTime<Utc>,
) -> Result<(), LedgerError> {
    validate_key(&effect.idempotency_key, "outbox idempotency_key")?;
    transaction.execute(
        "INSERT INTO outbox (
            outbox_id, mission_id, event_id, idempotency_key, kind,
            payload_json, status, attempts, available_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pending', 0, ?7)",
        params![
            format!("outbox-{}-{index}", event.event_id),
            event.mission_id.as_str(),
            event.event_id,
            effect.idempotency_key,
            effect.kind,
            serde_json::to_string(&effect.payload)?,
            now.to_rfc3339(),
        ],
    )?;
    Ok(())
}

fn read_projection(
    connection: &Connection,
    mission_id: &str,
) -> Result<Option<MissionProjection>, LedgerError> {
    let row = connection
        .query_row(
            "SELECT state_json, version, active_plan_revision,
                    projection_hash, updated_at
             FROM missions WHERE mission_id = ?1",
            params![mission_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            },
        )
        .optional()?;
    row.map(|(state, version, plan_revision, hash, updated_at)| {
        let mission_id = MissionId(mission_id.to_string());
        let state = serde_json::from_str(&state)?;
        let version = as_u64(version)?;
        let active_plan_revision = plan_revision.map(as_u64).transpose()?;
        let recomputed = projection_hash(&mission_id, state, version, active_plan_revision)?;
        if hash != recomputed {
            return Err(LedgerError::ProjectionHashMismatch {
                mission_id: mission_id.0.clone(),
                stored: hash,
                recomputed,
            });
        }
        Ok(MissionProjection {
            mission_id,
            state,
            version,
            active_plan_revision,
            projection_hash: recomputed,
            updated_at: DateTime::parse_from_rfc3339(&updated_at)?.with_timezone(&Utc),
        })
    })
    .transpose()
}

fn read_task_attempt(
    connection: &Connection,
    attempt_id: &str,
) -> Result<Option<TaskAttemptProjection>, LedgerError> {
    let row = connection
        .query_row(
            "SELECT mission_id, task_id, plan_revision, state_json,
                    version, fencing_token, updated_at
             FROM task_attempts WHERE attempt_id = ?1",
            params![attempt_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, Option<i64>>(5)?,
                    row.get::<_, String>(6)?,
                ))
            },
        )
        .optional()?;
    row.map(
        |(mission_id, task_id, plan_revision, state, version, token, updated_at)| {
            Ok(TaskAttemptProjection {
                mission_id: MissionId(mission_id),
                task_id,
                attempt_id: attempt_id.to_string(),
                plan_revision: as_u64(plan_revision)?,
                state: serde_json::from_str(&state)?,
                version: as_u64(version)?,
                fencing_token: token.map(as_u64).transpose()?,
                updated_at: DateTime::parse_from_rfc3339(&updated_at)?.with_timezone(&Utc),
            })
        },
    )
    .transpose()
}

fn read_task_attempts_for_mission(
    connection: &Connection,
    mission_id: &MissionId,
) -> Result<Vec<TaskAttemptProjection>, LedgerError> {
    let mut statement = connection.prepare(
        "SELECT mission_id, task_id, attempt_id, plan_revision, state_json,
                version, fencing_token, updated_at
         FROM task_attempts WHERE mission_id = ?1
         ORDER BY task_id, attempt_id",
    )?;
    let rows = statement.query_map(params![mission_id.as_str()], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, i64>(5)?,
            row.get::<_, Option<i64>>(6)?,
            row.get::<_, String>(7)?,
        ))
    })?;
    let mut attempts = Vec::new();
    for row in rows {
        let (
            stored_mission_id,
            task_id,
            attempt_id,
            plan_revision,
            state_json,
            version,
            fencing_token,
            updated_at,
        ) = row?;
        attempts.push(TaskAttemptProjection {
            mission_id: MissionId(stored_mission_id),
            task_id,
            attempt_id,
            plan_revision: as_u64(plan_revision)?,
            state: serde_json::from_str(&state_json)?,
            version: as_u64(version)?,
            fencing_token: fencing_token.map(as_u64).transpose()?,
            updated_at: DateTime::parse_from_rfc3339(&updated_at)?.with_timezone(&Utc),
        });
    }
    Ok(attempts)
}

fn read_legacy_task_attempts(
    connection: &Connection,
    mission_id: &MissionId,
) -> Result<Vec<LegacyTaskAttemptRecord>, LedgerError> {
    let mut statement = connection.prepare(
        "SELECT mission_id, task_id, attempt_id, plan_revision, state_json,
                version, fencing_token, updated_at, imported_at, provenance, source
         FROM legacy_task_attempts WHERE mission_id = ?1
         ORDER BY task_id, attempt_id",
    )?;
    let rows = statement.query_map(params![mission_id.as_str()], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, i64>(5)?,
            row.get::<_, Option<i64>>(6)?,
            row.get::<_, String>(7)?,
            row.get::<_, String>(8)?,
            row.get::<_, String>(9)?,
            row.get::<_, String>(10)?,
        ))
    })?;
    let mut attempts = Vec::new();
    for row in rows {
        let (
            stored_mission_id,
            task_id,
            attempt_id,
            plan_revision,
            state_json,
            version,
            fencing_token,
            updated_at,
            imported_at,
            provenance,
            source,
        ) = row?;
        if provenance != LEGACY_ATTEMPT_PROVENANCE {
            return Err(LedgerError::InvalidInput(format!(
                "unknown legacy attempt provenance for {attempt_id}: {provenance}"
            )));
        }
        attempts.push(LegacyTaskAttemptRecord {
            mission_id: MissionId(stored_mission_id),
            task_id,
            attempt_id,
            plan_revision: as_u64(plan_revision)?,
            historical_state: serde_json::from_str(&state_json)?,
            historical_version: as_u64(version)?,
            historical_fencing_token: fencing_token.map(as_u64).transpose()?,
            historical_updated_at: DateTime::parse_from_rfc3339(&updated_at)?.with_timezone(&Utc),
            imported_at: DateTime::parse_from_rfc3339(&imported_at)?.with_timezone(&Utc),
            provenance,
            source,
        });
    }
    Ok(attempts)
}

fn insert_task_attempt_projection(
    connection: &Connection,
    projection: &TaskAttemptProjection,
) -> Result<(), LedgerError> {
    connection.execute(
        "INSERT INTO task_attempts (
            attempt_id, mission_id, task_id, plan_revision, state_json,
            version, fencing_token, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            projection.attempt_id,
            projection.mission_id.as_str(),
            projection.task_id,
            as_i64(projection.plan_revision)?,
            serde_json::to_string(&projection.state)?,
            as_i64(projection.version)?,
            projection.fencing_token.map(as_i64).transpose()?,
            projection.updated_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

fn read_event_by_idempotency(
    connection: &Connection,
    mission_id: &str,
    idempotency_key: &str,
) -> Result<Option<MissionEvent>, LedgerError> {
    let row = connection
        .query_row(
            "SELECT event_id, mission_id, task_id, attempt_id, sequence,
                    expected_version, schema_version, idempotency_key, command_digest, actor,
                    provider, causation_id, correlation_id, fencing_token,
                    plan_revision, activated_plan_revision, task_attempt_mutation_json,
                    task_attempt_projection_json, recorded_at, kind, payload_json,
                    mission_state_json
             FROM events WHERE mission_id = ?1 AND idempotency_key = ?2",
            params![mission_id, idempotency_key],
            event_row,
        )
        .optional()?;
    row.transpose()
}

fn event_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Result<MissionEvent, LedgerError>> {
    let event_id = row.get::<_, String>(0)?;
    let mission_id = row.get::<_, String>(1)?;
    let task_id = row.get::<_, Option<String>>(2)?;
    let attempt_id = row.get::<_, Option<String>>(3)?;
    let sequence = row.get::<_, i64>(4)?;
    let expected_version = row.get::<_, i64>(5)?;
    let schema_version = row.get::<_, i64>(6)?;
    let idempotency_key = row.get::<_, String>(7)?;
    let command_digest = row.get::<_, String>(8)?;
    let actor = row.get::<_, String>(9)?;
    let provider = row.get::<_, Option<String>>(10)?;
    let causation_id = row.get::<_, Option<String>>(11)?;
    let correlation_id = row.get::<_, Option<String>>(12)?;
    let fencing_token = row.get::<_, Option<i64>>(13)?;
    let plan_revision = row.get::<_, Option<i64>>(14)?;
    let activated_plan_revision = row.get::<_, Option<i64>>(15)?;
    let task_attempt_mutation = row.get::<_, Option<String>>(16)?;
    let task_attempt_projection = row.get::<_, Option<String>>(17)?;
    let recorded_at = row.get::<_, String>(18)?;
    let kind = row.get::<_, String>(19)?;
    let payload = row.get::<_, String>(20)?;
    let state = row.get::<_, Option<String>>(21)?;
    Ok((|| {
        Ok(MissionEvent {
            event_id,
            mission_id: MissionId(mission_id),
            task_id,
            attempt_id,
            sequence: as_u64(sequence)?,
            expected_version: as_u64(expected_version)?,
            schema_version: u32::try_from(schema_version).map_err(|_| {
                LedgerError::InvalidInput(format!("invalid schema version: {schema_version}"))
            })?,
            idempotency_key,
            command_digest,
            actor,
            provider,
            causation_id,
            correlation_id,
            fencing_token: fencing_token.map(as_u64).transpose()?,
            plan_revision: plan_revision.map(as_u64).transpose()?,
            activated_plan_revision: activated_plan_revision.map(as_u64).transpose()?,
            task_attempt_mutation: task_attempt_mutation
                .map(|value| serde_json::from_str(&value))
                .transpose()?,
            resulting_task_attempt: task_attempt_projection
                .map(|value| serde_json::from_str(&value))
                .transpose()?,
            recorded_at: DateTime::parse_from_rfc3339(&recorded_at)?.with_timezone(&Utc),
            kind,
            payload: serde_json::from_str(&payload)?,
            resulting_mission_state: state
                .map(|value| serde_json::from_str(&value))
                .transpose()?,
        })
    })())
}

fn read_lease(
    connection: &Connection,
    resource_key: &str,
) -> Result<Option<LeaseRecord>, LedgerError> {
    let row = connection
        .query_row(
            "SELECT mission_id, task_id, attempt_id, owner,
                    fencing_token, expires_at, status
             FROM leases WHERE resource_key = ?1",
            params![resource_key],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                ))
            },
        )
        .optional()?;
    row.map(
        |(mission_id, task_id, attempt_id, owner, token, expires_at, status)| {
            Ok(LeaseRecord {
                resource_key: resource_key.to_string(),
                mission_id: MissionId(mission_id),
                task_id,
                attempt_id,
                owner,
                fencing_token: as_u64(token)?,
                expires_at: DateTime::parse_from_rfc3339(&expires_at)?.with_timezone(&Utc),
                status: match status.as_str() {
                    "active" => LeaseStatus::Active,
                    "released" => LeaseStatus::Released,
                    other => {
                        return Err(LedgerError::InvalidInput(format!(
                            "unknown lease status: {other}"
                        )))
                    }
                },
            })
        },
    )
    .transpose()
}

fn supplied_lease_assertions(request: &AppendEvent) -> Result<Vec<LeaseAssertion>, LedgerError> {
    let mut supplied = request.lease_assertions.clone();
    match (&request.lease_resource, request.fencing_token) {
        (Some(resource), Some(token)) => {
            let legacy = LeaseAssertion {
                resource_key: resource.clone(),
                owner: request.actor.clone(),
                fencing_token: token,
            };
            match supplied
                .iter()
                .find(|assertion| assertion.resource_key == *resource)
            {
                Some(existing) if existing != &legacy => {
                    return Err(LedgerError::LeaseContextMismatch {
                        resource: resource.clone(),
                        reason: "legacy and aggregate lease credentials disagree".to_string(),
                    });
                }
                Some(_) => {}
                None => supplied.push(legacy),
            }
        }
        (Some(resource), None) => {
            return Err(LedgerError::StaleFence {
                resource: resource.clone(),
                expected: 0,
                actual: None,
            });
        }
        (None, Some(_)) => {
            return Err(LedgerError::InvalidInput(
                "fencing_token requires lease_resource".to_string(),
            ));
        }
        (None, None) => {}
    }
    supplied.sort_by(|left, right| left.resource_key.cmp(&right.resource_key));
    for assertion in &supplied {
        validate_key(&assertion.resource_key, "lease assertion resource")?;
        validate_key(&assertion.owner, "lease assertion owner")?;
        if assertion.fencing_token == 0 {
            return Err(LedgerError::InvalidInput(format!(
                "lease assertion {} has zero fencing token",
                assertion.resource_key
            )));
        }
    }
    if supplied
        .windows(2)
        .any(|pair| pair[0].resource_key == pair[1].resource_key)
    {
        return Err(LedgerError::InvalidInput(
            "duplicate lease assertion resource".to_string(),
        ));
    }
    Ok(supplied)
}

fn read_active_leases_for_attempt(
    connection: &Connection,
    mission_id: &MissionId,
    task_id: &str,
    attempt_id: &str,
) -> Result<Vec<LeaseRecord>, LedgerError> {
    let mut statement = connection.prepare(
        "SELECT resource_key, mission_id, task_id, attempt_id, owner,
                fencing_token, expires_at, status
         FROM leases
         WHERE mission_id = ?1 AND task_id = ?2 AND attempt_id = ?3
           AND status = 'active' AND expires_at > ?4
         ORDER BY resource_key",
    )?;
    let rows = statement.query_map(
        params![
            mission_id.as_str(),
            task_id,
            attempt_id,
            Utc::now().to_rfc3339()
        ],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
            ))
        },
    )?;
    let mut leases = Vec::new();
    for row in rows {
        let (
            resource_key,
            stored_mission,
            stored_task,
            stored_attempt,
            owner,
            token,
            expires,
            status,
        ) = row?;
        let status = match status.as_str() {
            "active" => LeaseStatus::Active,
            "released" => LeaseStatus::Released,
            other => {
                return Err(LedgerError::InvalidInput(format!(
                    "unknown lease status: {other}"
                )))
            }
        };
        leases.push(LeaseRecord {
            resource_key,
            mission_id: MissionId(stored_mission),
            task_id: stored_task,
            attempt_id: stored_attempt,
            owner,
            fencing_token: as_u64(token)?,
            expires_at: DateTime::parse_from_rfc3339(&expires)?.with_timezone(&Utc),
            status,
        });
    }
    Ok(leases)
}

fn assert_complete_attempt_lease_authority_tx(
    connection: &Connection,
    mission_id: &MissionId,
    mutation: &TaskAttemptMutation,
    supplied: &[LeaseAssertion],
) -> Result<(), LedgerError> {
    // Diagnose a supplied credential against its own durable row first. This
    // preserves precise mission/task/attempt/owner errors and also rejects an
    // extra stale credential before comparing the aggregate set.
    for assertion in supplied {
        assert_fence_context_tx(
            connection,
            &assertion.resource_key,
            assertion.fencing_token,
            mission_id,
            mutation,
            &assertion.owner,
        )?;
    }
    let active = read_active_leases_for_attempt(
        connection,
        mission_id,
        &mutation.task_id,
        &mutation.attempt_id,
    )?;
    if active.len() != supplied.len() {
        return Err(LedgerError::LeaseContextMismatch {
            resource: format!("attempt:{}", mutation.attempt_id),
            reason: format!(
                "complete active lease set required: ledger has {}, command supplied {}",
                active.len(),
                supplied.len()
            ),
        });
    }
    for (lease, assertion) in active.iter().zip(supplied) {
        if lease.resource_key != assertion.resource_key {
            return Err(LedgerError::LeaseContextMismatch {
                resource: assertion.resource_key.clone(),
                reason: format!(
                    "lease resource set differs; expected {}",
                    lease.resource_key
                ),
            });
        }
    }
    Ok(())
}

fn assert_fence_tx(
    connection: &Connection,
    resource_key: &str,
    fencing_token: u64,
) -> Result<(), LedgerError> {
    let lease = read_lease(connection, resource_key)?;
    match lease {
        Some(lease)
            if lease.status == LeaseStatus::Active
                && lease.fencing_token == fencing_token
                && lease.expires_at > Utc::now() =>
        {
            Ok(())
        }
        other => Err(LedgerError::StaleFence {
            resource: resource_key.to_string(),
            expected: fencing_token,
            actual: other.map(|lease| lease.fencing_token),
        }),
    }
}

fn assert_fence_context_tx(
    connection: &Connection,
    resource_key: &str,
    fencing_token: u64,
    mission_id: &MissionId,
    mutation: &TaskAttemptMutation,
    owner: &str,
) -> Result<(), LedgerError> {
    let lease = read_lease(connection, resource_key)?;
    let Some(lease) = lease else {
        return Err(LedgerError::StaleFence {
            resource: resource_key.to_string(),
            expected: fencing_token,
            actual: None,
        });
    };
    if lease.status != LeaseStatus::Active
        || lease.fencing_token != fencing_token
        || lease.expires_at <= Utc::now()
    {
        return Err(LedgerError::StaleFence {
            resource: resource_key.to_string(),
            expected: fencing_token,
            actual: Some(lease.fencing_token),
        });
    }
    let mismatch = if lease.mission_id != *mission_id {
        Some("mission_id")
    } else if lease.task_id != mutation.task_id {
        Some("task_id")
    } else if lease.attempt_id != mutation.attempt_id {
        Some("attempt_id")
    } else if lease.owner != owner {
        Some("owner")
    } else {
        None
    };
    if let Some(field) = mismatch {
        return Err(LedgerError::LeaseContextMismatch {
            resource: resource_key.to_string(),
            reason: format!("{field} does not match the append command"),
        });
    }
    Ok(())
}

fn read_outbox(
    connection: &Connection,
    outbox_id: &str,
) -> Result<Option<OutboxRecord>, LedgerError> {
    let row = connection
        .query_row(
            "SELECT mission_id, event_id, idempotency_key, kind, payload_json,
                    status, attempts, available_at, claim_owner, claim_until,
                    last_error, remote_ref
             FROM outbox WHERE outbox_id = ?1",
            params![outbox_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, Option<String>>(9)?,
                    row.get::<_, Option<String>>(10)?,
                    row.get::<_, Option<String>>(11)?,
                ))
            },
        )
        .optional()?;
    row.map(
        |(
            mission_id,
            event_id,
            idempotency_key,
            kind,
            payload,
            status,
            attempts,
            available_at,
            claim_owner,
            claim_until,
            last_error,
            remote_ref,
        )| {
            Ok(OutboxRecord {
                outbox_id: outbox_id.to_string(),
                mission_id: MissionId(mission_id),
                event_id,
                idempotency_key,
                kind,
                payload: serde_json::from_str(&payload)?,
                status: match status.as_str() {
                    "pending" => OutboxStatus::Pending,
                    "processing" => OutboxStatus::Processing,
                    "delivered" => OutboxStatus::Delivered,
                    other => {
                        return Err(LedgerError::InvalidInput(format!(
                            "unknown outbox status: {other}"
                        )))
                    }
                },
                attempts: u32::try_from(attempts).map_err(|_| {
                    LedgerError::InvalidInput(format!("invalid outbox attempts value: {attempts}"))
                })?,
                available_at: DateTime::parse_from_rfc3339(&available_at)?.with_timezone(&Utc),
                claim_owner,
                claim_until: claim_until
                    .map(|value| {
                        DateTime::parse_from_rfc3339(&value).map(|dt| dt.with_timezone(&Utc))
                    })
                    .transpose()?,
                last_error,
                remote_ref,
            })
        },
    )
    .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mission::{
        RetryPolicy, TaskContract, TaskId, VerifierCheck, VerifierCheckKind,
        CONTRACT_SCHEMA_VERSION,
    };
    use crate::routing::RiskLevel;
    use std::path::PathBuf;
    use std::sync::{Arc, Barrier};
    use std::thread;

    #[cfg(unix)]
    use std::os::unix::fs::symlink;

    fn mission(id: &str) -> Mission {
        let mut mission = Mission::new("OmegaOS", "test mission", PathBuf::from("/tmp"));
        mission.id = MissionId(id.to_string());
        mission
    }

    fn transition(
        mission_id: &MissionId,
        expected_version: u64,
        key: &str,
        next: MissionState,
    ) -> AppendEvent {
        let mut request = AppendEvent::new(
            mission_id.clone(),
            expected_version,
            key,
            "test",
            "mission_transition",
        );
        request.next_mission_state = Some(next);
        request
    }

    fn task(id: &str) -> TaskContract {
        TaskContract {
            schema_version: CONTRACT_SCHEMA_VERSION,
            task_id: TaskId::new(id),
            name: id.to_string(),
            prompt: format!("implement {id}"),
            acceptance_criteria: vec![format!("{id} is verified")],
            verifier_checks: vec![VerifierCheck {
                schema_version: CONTRACT_SCHEMA_VERSION,
                check_id: format!("verify-{id}"),
                kind: VerifierCheckKind::FileExists {
                    path: format!("src/{id}.rs"),
                },
                timeout_secs: 10,
            }],
            required_capabilities: vec![],
            scope: vec![format!("src/{id}.rs")],
            risk: RiskLevel::Medium,
            retry_policy: RetryPolicy::default(),
            depends_on: vec![],
        }
    }

    fn persist_first_plan(ledger: &MissionLedger, mission: &Mission, tasks: Vec<TaskContract>) {
        let plan = PlanContract::new(mission.id.clone(), 1, 1, tasks, vec![], vec![]).unwrap();
        let mut event = transition(&mission.id, 1, "persist-plan", MissionState::Classified);
        event.plan = Some(plan);
        ledger.append(event).unwrap();
    }

    /// Write the exact schema deployed before event schema v2. Attempt rows
    /// intentionally contain information that the old events cannot prove.
    fn create_schema_v1_fixture(
        path: &Path,
        mission_count: usize,
        attempt_count: usize,
    ) -> Vec<Mission> {
        assert!(mission_count > 0);
        assert!(attempt_count >= mission_count);
        let connection = Connection::open(path).unwrap();
        connection
            .execute_batch(
                r#"
                PRAGMA foreign_keys = ON;
                CREATE TABLE missions (
                    mission_id TEXT PRIMARY KEY,
                    mission_json TEXT NOT NULL,
                    state_json TEXT NOT NULL,
                    version INTEGER NOT NULL,
                    active_plan_revision INTEGER,
                    projection_hash TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );
                CREATE TABLE events (
                    event_id TEXT PRIMARY KEY,
                    mission_id TEXT NOT NULL,
                    task_id TEXT,
                    attempt_id TEXT,
                    sequence INTEGER NOT NULL,
                    expected_version INTEGER NOT NULL,
                    schema_version INTEGER NOT NULL,
                    idempotency_key TEXT NOT NULL,
                    actor TEXT NOT NULL,
                    provider TEXT,
                    causation_id TEXT,
                    correlation_id TEXT,
                    fencing_token INTEGER,
                    plan_revision INTEGER,
                    recorded_at TEXT NOT NULL,
                    kind TEXT NOT NULL,
                    payload_json TEXT NOT NULL,
                    mission_state_json TEXT,
                    UNIQUE(mission_id, sequence),
                    UNIQUE(mission_id, idempotency_key),
                    FOREIGN KEY(mission_id) REFERENCES missions(mission_id)
                );
                CREATE TABLE plans (
                    mission_id TEXT NOT NULL,
                    plan_id TEXT NOT NULL,
                    revision INTEGER NOT NULL,
                    contract_json TEXT NOT NULL,
                    content_digest TEXT NOT NULL,
                    PRIMARY KEY(mission_id, revision),
                    UNIQUE(mission_id, content_digest),
                    FOREIGN KEY(mission_id) REFERENCES missions(mission_id)
                );
                CREATE TABLE task_attempts (
                    attempt_id TEXT PRIMARY KEY,
                    mission_id TEXT NOT NULL,
                    task_id TEXT NOT NULL,
                    plan_revision INTEGER NOT NULL,
                    state_json TEXT NOT NULL,
                    version INTEGER NOT NULL,
                    fencing_token INTEGER,
                    updated_at TEXT NOT NULL,
                    UNIQUE(mission_id, task_id, attempt_id),
                    FOREIGN KEY(mission_id) REFERENCES missions(mission_id)
                );
                "#,
            )
            .unwrap();

        let base = attempt_count / mission_count;
        let remainder = attempt_count % mission_count;
        let origin = Utc::now() - ChronoDuration::hours(1);
        let mut missions = Vec::new();
        let mut global_attempt = 0_usize;
        for mission_index in 0..mission_count {
            let attempts_for_mission = base + usize::from(mission_index < remainder);
            let mission = mission(&format!("legacy-mission-{mission_index}"));
            let tasks = (0..attempts_for_mission)
                .map(|task_index| task(&format!("legacy-task-{mission_index}-{task_index}")))
                .collect::<Vec<_>>();
            let plan = PlanContract::new(
                mission.id.clone(),
                1,
                1,
                tasks.clone(),
                Vec::new(),
                Vec::new(),
            )
            .unwrap();
            let final_version = 2_u64 + attempts_for_mission as u64;
            let created_at = origin + ChronoDuration::seconds((mission_index * 100) as i64);
            let updated_at = created_at + ChronoDuration::seconds(final_version as i64 - 1);
            let hash = projection_hash(
                &mission.id,
                MissionState::Classified,
                final_version,
                Some(1),
            )
            .unwrap();
            connection
                .execute(
                    "INSERT INTO missions (
                        mission_id, mission_json, state_json, version,
                        active_plan_revision, projection_hash, created_at, updated_at
                     ) VALUES (?1, ?2, ?3, ?4, 1, ?5, ?6, ?7)",
                    params![
                        mission.id.as_str(),
                        serde_json::to_string(&mission).unwrap(),
                        serde_json::to_string(&MissionState::Classified).unwrap(),
                        as_i64(final_version).unwrap(),
                        hash,
                        created_at.to_rfc3339(),
                        updated_at.to_rfc3339(),
                    ],
                )
                .unwrap();
            connection
                .execute(
                    "INSERT INTO plans (
                        mission_id, plan_id, revision, contract_json, content_digest
                     ) VALUES (?1, ?2, 1, ?3, ?4)",
                    params![
                        mission.id.as_str(),
                        plan.plan_id.0.as_str(),
                        serde_json::to_string(&plan).unwrap(),
                        plan.content_digest,
                    ],
                )
                .unwrap();

            let insert_event = |sequence: u64,
                                task_id: Option<&str>,
                                attempt_id: Option<&str>,
                                plan_revision: Option<u64>,
                                kind: &str,
                                payload: Value,
                                state: Option<MissionState>| {
                let recorded_at = created_at + ChronoDuration::seconds(sequence as i64 - 1);
                connection
                    .execute(
                        "INSERT INTO events (
                            event_id, mission_id, task_id, attempt_id, sequence,
                            expected_version, schema_version, idempotency_key, actor,
                            provider, causation_id, correlation_id, fencing_token,
                            plan_revision, recorded_at, kind, payload_json,
                            mission_state_json
                         ) VALUES (
                            ?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, 'legacy-engine',
                            NULL, NULL, NULL, NULL, ?8, ?9, ?10, ?11, ?12
                         )",
                        params![
                            format!("legacy-event-{mission_index}-{sequence}"),
                            mission.id.as_str(),
                            task_id,
                            attempt_id,
                            as_i64(sequence).unwrap(),
                            as_i64(sequence - 1).unwrap(),
                            format!("legacy-key-{mission_index}-{sequence}"),
                            plan_revision.map(as_i64).transpose().unwrap(),
                            recorded_at.to_rfc3339(),
                            kind,
                            serde_json::to_string(&payload).unwrap(),
                            state
                                .map(|value| serde_json::to_string(&value))
                                .transpose()
                                .unwrap(),
                        ],
                    )
                    .unwrap();
                recorded_at
            };
            insert_event(
                1,
                None,
                None,
                None,
                "mission_created",
                serde_json::to_value(&mission).unwrap(),
                Some(MissionState::Created),
            );
            insert_event(
                2,
                None,
                None,
                Some(1),
                "plan_accepted",
                Value::Null,
                Some(MissionState::Classified),
            );
            for (task_index, task) in tasks.iter().enumerate() {
                let sequence = 3 + task_index as u64;
                let attempt_id = format!("legacy-attempt-{mission_index}-{task_index}");
                let attempt_updated_at = insert_event(
                    sequence,
                    Some(task.task_id.as_str()),
                    Some(&attempt_id),
                    Some(1),
                    "task_attempt_historical",
                    Value::Null,
                    None,
                );
                let historical_state = match global_attempt {
                    0 => TaskAttemptState::Accepted,
                    1 => TaskAttemptState::Running,
                    _ => TaskAttemptState::CandidateDone,
                };
                connection
                    .execute(
                        "INSERT INTO task_attempts (
                            attempt_id, mission_id, task_id, plan_revision,
                            state_json, version, fencing_token, updated_at
                         ) VALUES (?1, ?2, ?3, 1, ?4, ?5, NULL, ?6)",
                        params![
                            attempt_id,
                            mission.id.as_str(),
                            task.task_id.as_str(),
                            serde_json::to_string(&historical_state).unwrap(),
                            as_i64(7 + global_attempt as u64).unwrap(),
                            attempt_updated_at.to_rfc3339(),
                        ],
                    )
                    .unwrap();
                global_attempt += 1;
            }
            missions.push(mission);
        }
        assert_eq!(global_attempt, attempt_count);
        missions
    }

    #[test]
    fn ledger_ids_are_fixed_format_and_unique_under_parallel_generation() {
        const WORKERS: usize = 16;
        const IDS_PER_WORKER: usize = 5_000;
        let barrier = Arc::new(Barrier::new(WORKERS));
        let joins: Vec<_> = (0..WORKERS)
            .map(|_| {
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    barrier.wait();
                    (0..IDS_PER_WORKER)
                        .map(|_| stable_id("event"))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        let ids: std::collections::HashSet<String> = joins
            .into_iter()
            .flat_map(|join| join.join().unwrap())
            .collect();
        assert_eq!(ids.len(), WORKERS * IDS_PER_WORKER);
        assert!(ids.iter().all(|id| {
            let Some(hex) = id.strip_prefix("event-") else {
                return false;
            };
            hex.len() == 32
                && hex
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        }));
    }

    #[cfg(unix)]
    #[test]
    fn filesystem_ledger_refuses_symlinked_or_non_regular_paths() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("target.sqlite");
        drop(MissionLedger::open(&target).unwrap());

        let file_link = temp.path().join("linked.sqlite");
        symlink(&target, &file_link).unwrap();
        assert!(matches!(
            MissionLedger::open(&file_link),
            Err(LedgerError::InvalidInput(message)) if message.contains("symlink")
        ));

        let directory_path = temp.path().join("directory.sqlite");
        fs::create_dir(&directory_path).unwrap();
        assert!(matches!(
            MissionLedger::open(&directory_path),
            Err(LedgerError::InvalidInput(message)) if message.contains("not a regular file")
        ));

        let real_parent = temp.path().join("real-parent");
        fs::create_dir(&real_parent).unwrap();
        let linked_parent = temp.path().join("linked-parent");
        symlink(&real_parent, &linked_parent).unwrap();
        assert!(matches!(
            MissionLedger::open(linked_parent.join("mission.sqlite")),
            Err(LedgerError::InvalidInput(message)) if message.contains("parent") && message.contains("symlink")
        ));
    }

    #[test]
    fn open_creates_missing_mission_engine_sqlite3() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("mission-engine-v3.sqlite3");
        assert!(!path.exists(), "fresh state dir has no ledger");
        drop(MissionLedger::open(&path).unwrap());
        assert!(
            path.is_file(),
            "omega must create mission-engine-v3.sqlite3 instead of crashing"
        );
        drop(MissionLedger::open(&path).unwrap());
    }

    #[cfg(unix)]
    #[test]
    fn filesystem_ledger_enforces_owner_only_database_and_sidecar_modes() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("private.sqlite");
        {
            let ledger = MissionLedger::open(&path).unwrap();
            let mission = mission("m-private-ledger");
            ledger
                .create_mission(&mission, "create-private-ledger", "test")
                .unwrap();
        }
        fs::set_permissions(&path, fs::Permissions::from_mode(0o666)).unwrap();

        let ledger = MissionLedger::open(&path).unwrap();
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        let mission = mission("m-private-ledger-second-write");
        ledger
            .create_mission(&mission, "create-private-ledger-second-write", "test")
            .unwrap();

        let sidecars = [sqlite_sidecar(&path, "-wal"), sqlite_sidecar(&path, "-shm")];
        assert!(
            sidecars.iter().all(|sidecar| sidecar.exists()),
            "WAL mode must expose both sidecars while the ledger is open"
        );
        for sidecar in sidecars {
            assert_eq!(
                fs::metadata(sidecar).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }

    #[test]
    fn schema_v1_14_mission_72_attempt_fixture_is_quarantined_and_requeueable() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("legacy-v1.sqlite");
        let missions = create_schema_v1_fixture(&path, 14, 72);

        {
            let ledger = MissionLedger::open(&path).unwrap();
            assert_eq!(ledger.legacy_task_attempt_count().unwrap(), 72);
            assert!(ledger.task_attempts(&missions[0].id).unwrap().is_empty());
            assert!(ledger
                .replay_task_attempts(&missions[0].id)
                .unwrap()
                .is_empty());
            assert!(ledger.task_attempt("legacy-attempt-0-0").unwrap().is_none());

            let historical = ledger.legacy_task_attempts(&missions[0].id).unwrap();
            assert_eq!(historical.len(), 6);
            assert_eq!(historical[0].attempt_id, "legacy-attempt-0-0");
            assert_eq!(historical[0].historical_state, TaskAttemptState::Accepted);
            assert_eq!(historical[0].historical_version, 7);
            assert_eq!(historical[0].provenance, LEGACY_ATTEMPT_PROVENANCE);
            assert_eq!(historical[0].source, "schema_v1_event_projection");
            assert_eq!(historical[1].historical_state, TaskAttemptState::Running);

            let projection = ledger.mission(&missions[0].id).unwrap().unwrap();
            let mut requeue = AppendEvent::new(
                missions[0].id.clone(),
                projection.version,
                "requeue-after-schema-v1-quarantine",
                "verifier",
                "task_attempt_requeued_for_reverification",
            );
            requeue.task_attempt = Some(TaskAttemptMutation {
                task_id: "legacy-task-0-0".to_string(),
                attempt_id: "legacy-attempt-0-0".to_string(),
                plan_revision: 1,
                expected_version: 0,
                next_state: TaskAttemptState::Queued,
            });
            let appended = ledger.append(requeue.clone()).unwrap();
            assert_eq!(appended.event.schema_version, SCHEMA_VERSION);
            assert_eq!(
                ledger
                    .task_attempt("legacy-attempt-0-0")
                    .unwrap()
                    .unwrap()
                    .state,
                TaskAttemptState::Queued
            );
            assert!(ledger.append(requeue).unwrap().idempotent_replay);

            let mut active_requeue = AppendEvent::new(
                missions[0].id.clone(),
                appended.projection.version,
                "requeue-active-schema-v1-attempt",
                "verifier",
                "task_attempt_requeued_for_reverification",
            );
            active_requeue.task_attempt = Some(TaskAttemptMutation {
                task_id: "legacy-task-0-1".to_string(),
                attempt_id: "legacy-attempt-0-1".to_string(),
                plan_revision: 1,
                expected_version: 0,
                next_state: TaskAttemptState::Queued,
            });
            ledger.append(active_requeue).unwrap();
            assert_eq!(ledger.task_attempts(&missions[0].id).unwrap().len(), 2);
            assert_eq!(ledger.legacy_task_attempt_count().unwrap(), 72);
        }

        // Migration and quarantine are idempotent. The schema-v2 requeue stays
        // authoritative while the exact schema-v1 Accepted row stays archived.
        let reopened = MissionLedger::open(&path).unwrap();
        assert_eq!(reopened.legacy_task_attempt_count().unwrap(), 72);
        assert_eq!(
            reopened
                .task_attempt("legacy-attempt-0-0")
                .unwrap()
                .unwrap()
                .state,
            TaskAttemptState::Queued
        );
        assert_eq!(
            reopened.legacy_task_attempts(&missions[0].id).unwrap()[0].historical_state,
            TaskAttemptState::Accepted
        );
    }

    #[test]
    fn legacy_archive_failure_rolls_back_before_authoritative_rows_are_deleted() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("legacy-rollback.sqlite");
        create_schema_v1_fixture(&path, 1, 1);
        {
            let connection = Connection::open(&path).unwrap();
            connection
                .execute_batch(
                    "CREATE TABLE legacy_task_attempts (
                        attempt_id TEXT PRIMARY KEY,
                        mission_id TEXT NOT NULL,
                        task_id TEXT NOT NULL,
                        plan_revision INTEGER NOT NULL,
                        state_json TEXT NOT NULL,
                        version INTEGER NOT NULL,
                        fencing_token INTEGER,
                        updated_at TEXT NOT NULL,
                        imported_at TEXT NOT NULL,
                        provenance TEXT NOT NULL CHECK (provenance = 'forced_failure'),
                        source TEXT NOT NULL
                    );",
                )
                .unwrap();
        }

        assert!(matches!(
            MissionLedger::open(&path),
            Err(LedgerError::Sqlite(_))
        ));
        let connection = Connection::open(&path).unwrap();
        let authoritative_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM task_attempts", [], |row| row.get(0))
            .unwrap();
        let archived_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM legacy_task_attempts", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(authoritative_count, 1);
        assert_eq!(archived_count, 0);
    }

    #[test]
    fn cas_and_idempotency_are_fail_closed() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = mission("m-cas");
        let created = ledger
            .create_mission(&mission, "create-m-cas", "test")
            .unwrap();
        assert_eq!(created.projection.version, 1);

        let first = ledger
            .append(transition(
                &mission.id,
                1,
                "classify-once",
                MissionState::Classified,
            ))
            .unwrap();
        assert_eq!(first.projection.version, 2);
        let replay = ledger
            .append(transition(
                &mission.id,
                1,
                "classify-once",
                MissionState::Classified,
            ))
            .unwrap();
        assert!(replay.idempotent_replay);
        assert_eq!(replay.event.event_id, first.event.event_id);
        assert!(matches!(
            ledger.append(transition(
                &mission.id,
                1,
                "different-command",
                MissionState::Classified,
            )),
            Err(LedgerError::VersionConflict {
                expected: 1,
                actual: 2
            })
        ));
    }

    #[test]
    fn reusing_an_idempotency_key_for_a_different_command_is_rejected() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = mission("m-idempotency-collision");
        ledger
            .create_mission(&mission, "create-idempotency-collision", "test")
            .unwrap();
        ledger
            .append(transition(
                &mission.id,
                1,
                "same-key",
                MissionState::Classified,
            ))
            .unwrap();

        let collision = transition(&mission.id, 1, "same-key", MissionState::Cancelled);
        assert!(matches!(
            ledger.append(collision),
            Err(LedgerError::IdempotencyConflict { key, .. }) if key == "same-key"
        ));
    }

    #[test]
    fn canonical_payload_key_order_replays_the_same_command() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = mission("m-canonical-idempotency");
        ledger
            .create_mission(&mission, "create-canonical-idempotency", "test")
            .unwrap();
        let mut first = transition(&mission.id, 1, "canonical", MissionState::Classified);
        let mut first_payload = serde_json::Map::new();
        first_payload.insert("b".to_string(), Value::from(2));
        first_payload.insert("a".to_string(), Value::from(1));
        first.payload = Value::Object(first_payload);
        let mut replay = transition(&mission.id, 1, "canonical", MissionState::Classified);
        let mut replay_payload = serde_json::Map::new();
        replay_payload.insert("a".to_string(), Value::from(1));
        replay_payload.insert("b".to_string(), Value::from(2));
        replay.payload = Value::Object(replay_payload);

        ledger.append(first).unwrap();
        assert!(ledger.append(replay).unwrap().idempotent_replay);
    }

    #[test]
    fn concurrent_expected_version_has_one_winner() {
        let temp = tempfile::tempdir().unwrap();
        let db = temp.path().join("ledger.sqlite");
        let ledger = MissionLedger::open(&db).unwrap();
        let mission = mission("m-race");
        ledger
            .create_mission(&mission, "create-m-race", "test")
            .unwrap();
        drop(ledger);

        let workers = 12;
        let barrier = Arc::new(Barrier::new(workers));
        let mut joins = Vec::new();
        for index in 0..workers {
            let db = db.clone();
            let barrier = Arc::clone(&barrier);
            let mission_id = mission.id.clone();
            joins.push(thread::spawn(move || {
                let ledger = MissionLedger::open(db).unwrap();
                barrier.wait();
                ledger.append(transition(
                    &mission_id,
                    1,
                    &format!("race-{index}"),
                    MissionState::Classified,
                ))
            }));
        }
        let results: Vec<_> = joins.into_iter().map(|join| join.join().unwrap()).collect();
        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(
            results
                .iter()
                .filter(|result| matches!(
                    result,
                    Err(LedgerError::VersionConflict {
                        expected: 1,
                        actual: 2
                    })
                ))
                .count(),
            workers - 1
        );
    }

    #[test]
    fn replay_matches_materialized_projection() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = mission("m-replay");
        ledger
            .create_mission(&mission, "create-m-replay", "test")
            .unwrap();
        ledger
            .append(transition(
                &mission.id,
                1,
                "classify-replay",
                MissionState::Classified,
            ))
            .unwrap();
        ledger
            .append(transition(
                &mission.id,
                2,
                "plan-replay",
                MissionState::Planned,
            ))
            .unwrap();
        let materialized = ledger.mission(&mission.id).unwrap().unwrap();
        let replayed = ledger.replay(&mission.id).unwrap();
        assert_eq!(materialized.state, replayed.state);
        assert_eq!(materialized.version, replayed.version);
        assert_eq!(materialized.projection_hash, replayed.projection_hash);
    }

    #[test]
    fn projection_at_replays_the_exact_historical_prefix_and_hash() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = mission("m-historical-projection");
        let created = ledger
            .create_mission(&mission, "create-historical-projection", "test")
            .unwrap();
        ledger
            .append(transition(
                &mission.id,
                1,
                "classify-after-history",
                MissionState::Classified,
            ))
            .unwrap();

        let historical = ledger.projection_at(&mission.id, 1).unwrap().unwrap();
        assert_eq!(historical, created.projection);
        assert_ne!(historical.projection_hash, "forged-stale-hash");
        assert!(ledger.projection_at(&mission.id, 3).unwrap().is_none());
        assert!(ledger
            .projection_at(&MissionId("absent".to_string()), 1)
            .unwrap()
            .is_none());
    }

    #[test]
    fn stored_projection_hash_tampering_is_rejected_before_replay() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = mission("m-projection-hash-tamper");
        ledger
            .create_mission(&mission, "create-projection-hash-tamper", "test")
            .unwrap();
        ledger
            .connection
            .lock()
            .unwrap()
            .execute(
                "UPDATE missions SET projection_hash = 'forged' WHERE mission_id = ?1",
                params![mission.id.as_str()],
            )
            .unwrap();

        assert!(matches!(
            ledger.mission(&mission.id),
            Err(LedgerError::ProjectionHashMismatch { stored, .. }) if stored == "forged"
        ));
    }

    #[test]
    fn recomputed_hash_cannot_hide_materialized_projection_divergence() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = mission("m-materialized-divergence");
        ledger
            .create_mission(&mission, "create-materialized-divergence", "test")
            .unwrap();
        ledger
            .append(transition(
                &mission.id,
                1,
                "classify-materialized-divergence",
                MissionState::Classified,
            ))
            .unwrap();
        let forged_hash = projection_hash(&mission.id, MissionState::Cancelled, 2, None).unwrap();
        ledger
            .connection
            .lock()
            .unwrap()
            .execute(
                "UPDATE missions SET state_json = ?1, projection_hash = ?2 WHERE mission_id = ?3",
                params![
                    serde_json::to_string(&MissionState::Cancelled).unwrap(),
                    forged_hash,
                    mission.id.as_str()
                ],
            )
            .unwrap();

        assert!(matches!(
            ledger.mission(&mission.id),
            Err(LedgerError::ReplayDivergence { reason, .. })
                if reason.contains("stored state/version/plan/hash/time")
        ));
    }

    #[test]
    fn replay_rejects_tampered_event_version_binding() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = mission("m-event-version-tamper");
        ledger
            .create_mission(&mission, "create-event-version-tamper", "test")
            .unwrap();
        ledger
            .append(transition(
                &mission.id,
                1,
                "classify-event-version-tamper",
                MissionState::Classified,
            ))
            .unwrap();
        ledger
            .connection
            .lock()
            .unwrap()
            .execute(
                "UPDATE events SET expected_version = 9 WHERE mission_id = ?1 AND sequence = 2",
                params![mission.id.as_str()],
            )
            .unwrap();

        assert!(matches!(
            ledger.replay(&mission.id),
            Err(LedgerError::ReplayDivergence { reason, .. })
                if reason.contains("expected_version")
        ));
    }

    #[test]
    fn replay_reconstructs_plan_revision_from_typed_event_field() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = mission("m-plan-replay");
        ledger
            .create_mission(&mission, "create-plan-replay", "test")
            .unwrap();
        let plan = PlanContract::new(
            mission.id.clone(),
            1,
            1,
            vec![task("task-a")],
            Vec::new(),
            Vec::new(),
        )
        .unwrap();
        let mut event = AppendEvent::new(
            mission.id.clone(),
            1,
            "record-plan",
            "test",
            "arbitrary_event_name",
        );
        event.next_mission_state = Some(MissionState::Classified);
        event.plan = Some(plan);
        let materialized = ledger.append(event).unwrap().projection;
        let replayed = ledger.replay(&mission.id).unwrap();
        assert_eq!(materialized.active_plan_revision, Some(1));
        assert_eq!(replayed.active_plan_revision, Some(1));
        assert_eq!(materialized.projection_hash, replayed.projection_hash);
    }

    #[test]
    fn task_attempt_event_is_complete_and_idempotent_replay_is_projection_neutral() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = mission("m-attempt-event-authority");
        ledger
            .create_mission(&mission, "create-attempt-event-authority", "test")
            .unwrap();
        persist_first_plan(&ledger, &mission, vec![task("task-a")]);

        let mutation = TaskAttemptMutation {
            task_id: "task-a".to_string(),
            attempt_id: "attempt-authoritative".to_string(),
            plan_revision: 1,
            expected_version: 0,
            next_state: TaskAttemptState::Queued,
        };
        let mut request = AppendEvent::new(
            mission.id.clone(),
            2,
            "queue-authoritative",
            "engine",
            "task_queued",
        );
        request.task_attempt = Some(mutation.clone());
        let first = ledger.append(request.clone()).unwrap();
        let duplicate = ledger.append(request).unwrap();

        assert_eq!(first.event.schema_version, SCHEMA_VERSION);
        assert_eq!(first.event.task_attempt_mutation, Some(mutation));
        let resulting = first.event.resulting_task_attempt.as_ref().unwrap();
        assert_eq!(resulting.state, TaskAttemptState::Queued);
        assert_eq!(resulting.version, 1);
        assert!(duplicate.idempotent_replay);
        assert_eq!(duplicate.event.event_id, first.event.event_id);
        assert_eq!(ledger.events(&mission.id).unwrap().len(), 3);
        assert_eq!(
            ledger.replay_task_attempts(&mission.id).unwrap(),
            ledger.task_attempts(&mission.id).unwrap()
        );
    }

    #[test]
    fn tampered_or_missing_attempt_projection_is_rejected_and_rebuild_restores_truth() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = mission("m-attempt-rebuild");
        ledger
            .create_mission(&mission, "create-attempt-rebuild", "test")
            .unwrap();
        persist_first_plan(&ledger, &mission, vec![task("task-a")]);
        let mut queued = AppendEvent::new(
            mission.id.clone(),
            2,
            "queue-attempt-rebuild",
            "engine",
            "task_queued",
        );
        queued.task_attempt = Some(TaskAttemptMutation {
            task_id: "task-a".to_string(),
            attempt_id: "attempt-rebuild".to_string(),
            plan_revision: 1,
            expected_version: 0,
            next_state: TaskAttemptState::Queued,
        });
        ledger.append(queued).unwrap();

        ledger
            .connection
            .lock()
            .unwrap()
            .execute(
                "UPDATE task_attempts SET state_json = ?1, version = 99
                 WHERE attempt_id = 'attempt-rebuild'",
                params![serde_json::to_string(&TaskAttemptState::Accepted).unwrap()],
            )
            .unwrap();
        assert!(matches!(
            ledger.mission(&mission.id),
            Err(LedgerError::ReplayDivergence { reason, .. })
                if reason.contains("materialized task attempts")
        ));
        let replayed = ledger.replay_task_attempts(&mission.id).unwrap();
        assert_eq!(replayed[0].state, TaskAttemptState::Queued);
        assert_eq!(replayed[0].version, 1);

        ledger.rebuild_projections(&mission.id).unwrap();
        assert_eq!(
            ledger.task_attempt("attempt-rebuild").unwrap().unwrap(),
            replayed[0]
        );

        ledger
            .connection
            .lock()
            .unwrap()
            .execute(
                "DELETE FROM task_attempts WHERE attempt_id = 'attempt-rebuild'",
                [],
            )
            .unwrap();
        assert!(matches!(
            ledger.task_attempt("attempt-rebuild"),
            Err(LedgerError::ReplayDivergence { reason, .. })
                if reason.contains("materialized task attempts")
        ));
        ledger.rebuild_projections(&mission.id).unwrap();
        assert_eq!(ledger.task_attempts(&mission.id).unwrap().len(), 1);
    }

    #[test]
    fn orphan_attempt_projection_is_rejected_and_removed_by_rebuild() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = mission("m-attempt-orphan");
        ledger
            .create_mission(&mission, "create-attempt-orphan", "test")
            .unwrap();
        persist_first_plan(&ledger, &mission, vec![task("task-a")]);
        let now = Utc::now();
        ledger
            .connection
            .lock()
            .unwrap()
            .execute(
                "INSERT INTO task_attempts (
                    attempt_id, mission_id, task_id, plan_revision, state_json,
                    version, fencing_token, updated_at
                 ) VALUES ('orphan-attempt', ?1, 'task-a', 1, ?2, 1, NULL, ?3)",
                params![
                    mission.id.as_str(),
                    serde_json::to_string(&TaskAttemptState::Queued).unwrap(),
                    now.to_rfc3339(),
                ],
            )
            .unwrap();

        assert!(matches!(
            ledger.task_attempts(&mission.id),
            Err(LedgerError::ReplayDivergence { reason, .. })
                if reason.contains("materialized task attempts")
        ));
        ledger.rebuild_projections(&mission.id).unwrap();
        assert!(ledger.task_attempts(&mission.id).unwrap().is_empty());
    }

    #[test]
    fn tampered_authoritative_attempt_event_fails_replay_and_cannot_rebuild() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = mission("m-attempt-event-tamper");
        ledger
            .create_mission(&mission, "create-attempt-event-tamper", "test")
            .unwrap();
        persist_first_plan(&ledger, &mission, vec![task("task-a")]);
        let mut queued = AppendEvent::new(
            mission.id.clone(),
            2,
            "queue-attempt-event-tamper",
            "engine",
            "task_queued",
        );
        queued.task_attempt = Some(TaskAttemptMutation {
            task_id: "task-a".to_string(),
            attempt_id: "attempt-event-tamper".to_string(),
            plan_revision: 1,
            expected_version: 0,
            next_state: TaskAttemptState::Queued,
        });
        ledger.append(queued).unwrap();

        let mut projection: TaskAttemptProjection = {
            let connection = ledger.connection.lock().unwrap();
            let json: String = connection
                .query_row(
                    "SELECT task_attempt_projection_json FROM events
                     WHERE attempt_id = 'attempt-event-tamper'",
                    [],
                    |row| row.get(0),
                )
                .unwrap();
            serde_json::from_str(&json).unwrap()
        };
        projection.state = TaskAttemptState::Accepted;
        projection.version = 99;
        ledger
            .connection
            .lock()
            .unwrap()
            .execute(
                "UPDATE events SET task_attempt_projection_json = ?1
                 WHERE attempt_id = 'attempt-event-tamper'",
                params![serde_json::to_string(&projection).unwrap()],
            )
            .unwrap();

        assert!(matches!(
            ledger.replay_task_attempts(&mission.id),
            Err(LedgerError::ReplayDivergence { reason, .. })
                if reason.contains("resulting task attempt")
        ));
        assert!(matches!(
            ledger.rebuild_projections(&mission.id),
            Err(LedgerError::ReplayDivergence { .. })
        ));
    }

    #[test]
    fn opening_a_ledger_rejects_attempt_projection_drift() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("attempt-integrity.sqlite");
        let mission = mission("m-attempt-open-integrity");
        {
            let ledger = MissionLedger::open(&path).unwrap();
            ledger
                .create_mission(&mission, "create-attempt-open-integrity", "test")
                .unwrap();
            persist_first_plan(&ledger, &mission, vec![task("task-a")]);
            let mut queued = AppendEvent::new(
                mission.id.clone(),
                2,
                "queue-attempt-open-integrity",
                "engine",
                "task_queued",
            );
            queued.task_attempt = Some(TaskAttemptMutation {
                task_id: "task-a".to_string(),
                attempt_id: "attempt-open-integrity".to_string(),
                plan_revision: 1,
                expected_version: 0,
                next_state: TaskAttemptState::Queued,
            });
            ledger.append(queued).unwrap();
        }
        {
            let connection = Connection::open(&path).unwrap();
            connection
                .execute(
                    "UPDATE task_attempts SET state_json = ?1, version = 99
                     WHERE attempt_id = 'attempt-open-integrity'",
                    params![serde_json::to_string(&TaskAttemptState::Accepted).unwrap()],
                )
                .unwrap();
        }

        assert!(matches!(
            MissionLedger::open(&path),
            Err(LedgerError::ReplayDivergence { reason, .. })
                if reason.contains("materialized task attempts")
        ));
    }

    #[test]
    fn schema_v1_attempt_events_are_historical_and_never_replay_as_authority() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = mission("m-legacy-attempt-event");
        ledger
            .create_mission(&mission, "create-legacy-attempt-event", "test")
            .unwrap();
        persist_first_plan(&ledger, &mission, vec![task("task-a")]);
        ledger
            .connection
            .lock()
            .unwrap()
            .execute(
                "UPDATE events
                 SET schema_version = ?1,
                     plan_revision = CASE WHEN sequence = 2 THEN 1 ELSE plan_revision END
                 WHERE sequence IN (1, 2)",
                params![i64::from(LEGACY_SCHEMA_VERSION)],
            )
            .unwrap();
        assert_eq!(ledger.replay(&mission.id).unwrap().version, 2);

        let mut queued = AppendEvent::new(
            mission.id.clone(),
            2,
            "queue-legacy-attempt-event",
            "engine",
            "task_queued",
        );
        queued.task_attempt = Some(TaskAttemptMutation {
            task_id: "task-a".to_string(),
            attempt_id: "attempt-legacy".to_string(),
            plan_revision: 1,
            expected_version: 0,
            next_state: TaskAttemptState::Queued,
        });
        ledger.append(queued).unwrap();
        ledger
            .connection
            .lock()
            .unwrap()
            .execute(
                "UPDATE events
                 SET schema_version = ?1,
                     task_attempt_mutation_json = NULL,
                     task_attempt_projection_json = NULL
                 WHERE attempt_id = 'attempt-legacy'",
                params![i64::from(LEGACY_SCHEMA_VERSION)],
            )
            .unwrap();
        assert_eq!(ledger.replay(&mission.id).unwrap().version, 3);
        assert!(ledger.replay_task_attempts(&mission.id).unwrap().is_empty());
        assert!(matches!(
            ledger.task_attempt("attempt-legacy"),
            Err(LedgerError::ReplayDivergence { reason, .. })
                if reason.contains("materialized task attempts")
        ));
    }

    #[test]
    fn outbox_is_transactional_and_idempotent() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = mission("m-outbox");
        ledger
            .create_mission(&mission, "create-m-outbox", "test")
            .unwrap();
        let mut request = transition(
            &mission.id,
            1,
            "classify-with-notify",
            MissionState::Classified,
        );
        request.outbox.push(NewOutboxEffect {
            idempotency_key: "notify-m-outbox".to_string(),
            kind: "telegram_message".to_string(),
            payload: serde_json::json!({"text": "classified"}),
        });
        ledger.append(request.clone()).unwrap();
        ledger.append(request).unwrap();

        let claimed = ledger
            .claim_outbox("notifier", 10, Duration::from_secs(30))
            .unwrap();
        assert_eq!(claimed.len(), 1);
        assert_eq!(claimed[0].attempts, 1);
        ledger
            .mark_outbox_delivered(&claimed[0].outbox_id, "notifier", Some("telegram:42"))
            .unwrap();
        assert!(ledger
            .claim_outbox("notifier", 10, Duration::from_secs(30))
            .unwrap()
            .is_empty());
    }

    #[test]
    fn lease_fencing_rejects_aba_writer() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = mission("m-lease");
        ledger
            .create_mission(&mission, "create-m-lease", "test")
            .unwrap();
        let first = ledger
            .acquire_lease(
                "src/auth.rs",
                &mission.id,
                "auth",
                "attempt-1",
                "worker-a",
                Duration::ZERO,
            )
            .unwrap();
        let second = ledger
            .acquire_lease(
                "src/auth.rs",
                &mission.id,
                "auth",
                "attempt-2",
                "worker-b",
                Duration::from_secs(30),
            )
            .unwrap();
        assert!(second.fencing_token > first.fencing_token);
        assert!(matches!(
            ledger.assert_fence("src/auth.rs", first.fencing_token),
            Err(LedgerError::StaleFence { .. })
        ));
        ledger
            .assert_fence("src/auth.rs", second.fencing_token)
            .unwrap();
    }

    #[test]
    fn leased_append_is_bound_to_mission_task_attempt_and_owner() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission_a = mission("m-lease-context-a");
        let mission_b = mission("m-lease-context-b");
        ledger
            .create_mission(&mission_a, "create-lease-context-a", "test")
            .unwrap();
        ledger
            .create_mission(&mission_b, "create-lease-context-b", "test")
            .unwrap();
        persist_first_plan(&ledger, &mission_a, vec![task("task-a")]);
        persist_first_plan(&ledger, &mission_b, vec![task("task-b")]);

        for (mission, task_id, attempt_id, key) in [
            (&mission_a, "task-a", "attempt-a", "queue-lease-context-a"),
            (&mission_b, "task-b", "attempt-b", "queue-lease-context-b"),
        ] {
            let mut queued = AppendEvent::new(mission.id.clone(), 2, key, "engine", "task_queued");
            queued.task_attempt = Some(TaskAttemptMutation {
                task_id: task_id.to_string(),
                attempt_id: attempt_id.to_string(),
                plan_revision: 1,
                expected_version: 0,
                next_state: TaskAttemptState::Queued,
            });
            ledger.append(queued).unwrap();
        }

        let lease = ledger
            .acquire_lease(
                "src/leased.rs",
                &mission_a.id,
                "task-a",
                "attempt-a",
                "worker-a",
                Duration::from_secs(300),
            )
            .unwrap();

        let leased_running =
            |mission_id: MissionId, task_id: &str, attempt_id: &str, actor: &str, key: &str| {
                let mut event = AppendEvent::new(mission_id, 3, key, actor, "task_running");
                event.task_attempt = Some(TaskAttemptMutation {
                    task_id: task_id.to_string(),
                    attempt_id: attempt_id.to_string(),
                    plan_revision: 1,
                    expected_version: 1,
                    next_state: TaskAttemptState::Running,
                });
                event.lease_resource = Some("src/leased.rs".to_string());
                event.fencing_token = Some(lease.fencing_token);
                event
            };

        for (event, field) in [
            (
                leased_running(
                    mission_a.id.clone(),
                    "task-a",
                    "attempt-a",
                    "worker-b",
                    "wrong-lease-owner",
                ),
                "owner",
            ),
            (
                leased_running(
                    mission_a.id.clone(),
                    "task-b",
                    "attempt-a",
                    "worker-a",
                    "wrong-lease-task",
                ),
                "task_id",
            ),
            (
                leased_running(
                    mission_a.id.clone(),
                    "task-a",
                    "attempt-other",
                    "worker-a",
                    "wrong-lease-attempt",
                ),
                "attempt_id",
            ),
            (
                leased_running(
                    mission_b.id.clone(),
                    "task-b",
                    "attempt-b",
                    "worker-a",
                    "wrong-lease-mission",
                ),
                "mission_id",
            ),
        ] {
            assert!(matches!(
                ledger.append(event),
                Err(LedgerError::LeaseContextMismatch { reason, .. })
                    if reason.contains(field)
            ));
        }

        let accepted = ledger
            .append(leased_running(
                mission_a.id.clone(),
                "task-a",
                "attempt-a",
                "worker-a",
                "correct-lease-context",
            ))
            .unwrap();
        assert_eq!(accepted.projection.version, 4);
        assert_eq!(
            ledger.task_attempt("attempt-a").unwrap().unwrap().state,
            TaskAttemptState::Running
        );
    }

    #[test]
    fn attempt_mutation_requires_every_active_lease_and_rejects_one_stolen_scope() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = mission("m-aggregate-lease");
        ledger
            .create_mission(&mission, "create-aggregate-lease", "test")
            .unwrap();
        persist_first_plan(&ledger, &mission, vec![task("task-a")]);
        let mut queued = AppendEvent::new(
            mission.id.clone(),
            2,
            "queue-aggregate-lease",
            "engine",
            "task_attempt_queued",
        );
        queued.task_attempt = Some(TaskAttemptMutation {
            task_id: "task-a".to_string(),
            attempt_id: "attempt-a".to_string(),
            plan_revision: 1,
            expected_version: 0,
            next_state: TaskAttemptState::Queued,
        });
        ledger.append(queued).unwrap();

        let first = ledger
            .acquire_lease(
                "scope:first",
                &mission.id,
                "task-a",
                "attempt-a",
                "worker-a",
                Duration::from_secs(300),
            )
            .unwrap();
        let second = ledger
            .acquire_lease(
                "scope:second",
                &mission.id,
                "task-a",
                "attempt-a",
                "worker-a",
                Duration::from_secs(300),
            )
            .unwrap();
        assert_eq!(
            ledger
                .active_leases_for_attempt(&mission.id, "task-a", "attempt-a")
                .unwrap(),
            vec![first.clone(), second.clone()]
        );

        let running = |key: &str, assertions: Vec<LeaseAssertion>| {
            let mut event = AppendEvent::new(
                mission.id.clone(),
                3,
                key,
                "omega-independent-verifier",
                "task_attempt_running",
            );
            event.task_attempt = Some(TaskAttemptMutation {
                task_id: "task-a".to_string(),
                attempt_id: "attempt-a".to_string(),
                plan_revision: 1,
                expected_version: 1,
                next_state: TaskAttemptState::Running,
            });
            event.lease_assertions = assertions;
            event
        };
        assert!(matches!(
            ledger.append(running("aggregate-none", Vec::new())),
            Err(LedgerError::LeaseContextMismatch { .. })
        ));
        assert!(matches!(
            ledger.append(running("aggregate-one", vec![LeaseAssertion::from(&first)])),
            Err(LedgerError::LeaseContextMismatch { .. })
        ));
        ledger
            .append(running(
                "aggregate-all",
                vec![LeaseAssertion::from(&first), LeaseAssertion::from(&second)],
            ))
            .unwrap();

        ledger
            .release_lease(&first.resource_key, first.fencing_token)
            .unwrap();
        let stolen = ledger
            .acquire_lease(
                &first.resource_key,
                &mission.id,
                "task-a",
                "attempt-stolen",
                "worker-b",
                Duration::from_secs(300),
            )
            .unwrap();
        assert!(stolen.fencing_token > first.fencing_token);
        assert_eq!(
            ledger
                .active_leases_for_attempt(&mission.id, "task-a", "attempt-a")
                .unwrap(),
            vec![second.clone()]
        );

        let mut candidate = AppendEvent::new(
            mission.id.clone(),
            4,
            "aggregate-stale-candidate",
            "omega-independent-verifier",
            "task_attempt_candidate_done",
        );
        candidate.task_attempt = Some(TaskAttemptMutation {
            task_id: "task-a".to_string(),
            attempt_id: "attempt-a".to_string(),
            plan_revision: 1,
            expected_version: 2,
            next_state: TaskAttemptState::CandidateDone,
        });
        candidate.lease_assertions =
            vec![LeaseAssertion::from(&first), LeaseAssertion::from(&second)];
        assert!(matches!(
            ledger.append(candidate),
            Err(LedgerError::StaleFence { resource, .. }) if resource == first.resource_key
        ));
        assert_eq!(
            ledger.task_attempt("attempt-a").unwrap().unwrap().state,
            TaskAttemptState::Running
        );
        ledger
            .assert_fence(&second.resource_key, second.fencing_token)
            .unwrap();
    }

    #[test]
    fn task_attempt_cannot_skip_candidate_verification() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = mission("m-task");
        ledger
            .create_mission(&mission, "create-m-task", "test")
            .unwrap();
        persist_first_plan(&ledger, &mission, vec![task("task-a")]);
        let mut queued =
            AppendEvent::new(mission.id.clone(), 2, "queue-task", "engine", "task_queued");
        queued.task_attempt = Some(TaskAttemptMutation {
            task_id: "task-a".to_string(),
            attempt_id: "attempt-a1".to_string(),
            plan_revision: 1,
            expected_version: 0,
            next_state: TaskAttemptState::Queued,
        });
        ledger.append(queued).unwrap();

        let mut invalid = AppendEvent::new(
            mission.id.clone(),
            3,
            "accept-task-directly",
            "engine",
            "task_accepted",
        );
        invalid.task_attempt = Some(TaskAttemptMutation {
            task_id: "task-a".to_string(),
            attempt_id: "attempt-a1".to_string(),
            plan_revision: 1,
            expected_version: 1,
            next_state: TaskAttemptState::Accepted,
        });
        assert!(matches!(
            ledger.append(invalid),
            Err(LedgerError::InvalidTaskTransition(_))
        ));
    }

    #[test]
    fn task_attempt_requires_an_active_plan_and_a_declared_task() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = mission("m-task-auth");
        ledger
            .create_mission(&mission, "create-m-task-auth", "test")
            .unwrap();

        let mut no_plan = AppendEvent::new(
            mission.id.clone(),
            1,
            "queue-no-plan",
            "engine",
            "task_queued",
        );
        no_plan.task_attempt = Some(TaskAttemptMutation {
            task_id: "task-a".to_string(),
            attempt_id: "attempt-no-plan".to_string(),
            plan_revision: 1,
            expected_version: 0,
            next_state: TaskAttemptState::Queued,
        });
        assert!(matches!(
            ledger.append(no_plan),
            Err(LedgerError::PlanRevisionNotActive { active: None, .. })
        ));

        persist_first_plan(&ledger, &mission, vec![task("task-a")]);
        let mut unknown = AppendEvent::new(
            mission.id.clone(),
            2,
            "queue-unknown",
            "engine",
            "task_queued",
        );
        unknown.task_attempt = Some(TaskAttemptMutation {
            task_id: "ghost".to_string(),
            attempt_id: "attempt-ghost".to_string(),
            plan_revision: 1,
            expected_version: 0,
            next_state: TaskAttemptState::Queued,
        });
        assert!(matches!(
            ledger.append(unknown),
            Err(LedgerError::TaskNotInPlan { task_id, .. }) if task_id == "ghost"
        ));

        let mut stale = AppendEvent::new(
            mission.id.clone(),
            2,
            "queue-stale",
            "engine",
            "task_queued",
        );
        stale.task_attempt = Some(TaskAttemptMutation {
            task_id: "task-a".to_string(),
            attempt_id: "attempt-stale".to_string(),
            plan_revision: 2,
            expected_version: 0,
            next_state: TaskAttemptState::Queued,
        });
        assert!(matches!(
            ledger.append(stale),
            Err(LedgerError::PlanRevisionNotActive {
                supplied: 2,
                active: Some(1),
                ..
            })
        ));
    }

    #[test]
    fn active_plan_fails_closed_when_persisted_contract_is_tampered() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = mission("m-plan-integrity");
        ledger
            .create_mission(&mission, "create-m-plan-integrity", "test")
            .unwrap();
        persist_first_plan(&ledger, &mission, vec![task("task-a")]);
        {
            let connection = ledger.connection.lock().unwrap();
            let contract_json: String = connection
                .query_row(
                    "SELECT contract_json FROM plans WHERE mission_id = ?1 AND revision = 1",
                    params![mission.id.as_str()],
                    |row| row.get(0),
                )
                .unwrap();
            let mut contract: Value = serde_json::from_str(&contract_json).unwrap();
            contract["tasks"][0]["prompt"] = Value::from("tampered after persistence");
            connection
                .execute(
                    "UPDATE plans SET contract_json = ?1 WHERE mission_id = ?2 AND revision = 1",
                    params![
                        serde_json::to_string(&contract).unwrap(),
                        mission.id.as_str()
                    ],
                )
                .unwrap();
        }
        assert!(matches!(
            ledger.active_plan(&mission.id),
            Err(LedgerError::InvalidInput(message)) if message.contains("digest mismatch")
        ));
    }

    #[test]
    fn ledger_protects_attempted_tasks_even_when_amend_helper_is_bypassed() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = mission("m-ledger-derived-protection");
        ledger
            .create_mission(&mission, "create-ledger-derived-protection", "test")
            .unwrap();
        persist_first_plan(&ledger, &mission, vec![task("task-a")]);

        let mut queued = AppendEvent::new(
            mission.id.clone(),
            2,
            "queue-before-bypass",
            "engine",
            "task_queued",
        );
        queued.task_attempt = Some(TaskAttemptMutation {
            task_id: "task-a".to_string(),
            attempt_id: "attempt-before-bypass".to_string(),
            plan_revision: 1,
            expected_version: 0,
            next_state: TaskAttemptState::Queued,
        });
        ledger.append(queued).unwrap();

        let mut changed_task = task("task-a");
        changed_task.prompt = "silently changed after dispatch".to_string();
        let bypassed = PlanContract::new(
            mission.id.clone(),
            2,
            3,
            vec![changed_task],
            Vec::new(),
            Vec::new(),
        )
        .unwrap();
        let mut amendment = AppendEvent::new(
            mission.id.clone(),
            3,
            "bypass-plan-amend-helper",
            "engine",
            "plan_amended",
        );
        amendment.plan = Some(bypassed);

        assert!(matches!(
            ledger.append(amendment),
            Err(LedgerError::InvalidInput(message))
                if message.contains("protected task changed: task-a")
        ));
        assert_eq!(
            ledger.mission(&mission.id).unwrap().unwrap().version,
            3,
            "a rejected amendment must be transactionally inert"
        );
    }

    #[test]
    fn in_flight_attempt_may_continue_only_when_its_task_is_unchanged_in_active_plan() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = mission("m-active-amendment");
        ledger
            .create_mission(&mission, "create-m-active-amendment", "test")
            .unwrap();
        persist_first_plan(&ledger, &mission, vec![task("task-a")]);

        let mut queued = AppendEvent::new(
            mission.id.clone(),
            2,
            "queue-before-amendment",
            "engine",
            "task_queued",
        );
        queued.task_attempt = Some(TaskAttemptMutation {
            task_id: "task-a".to_string(),
            attempt_id: "attempt-before-amendment".to_string(),
            plan_revision: 1,
            expected_version: 0,
            next_state: TaskAttemptState::Queued,
        });
        ledger.append(queued).unwrap();

        let plan = ledger.active_plan(&mission.id).unwrap().unwrap();
        let amended = plan
            .amend(
                1,
                3,
                vec![task("task-a"), task("task-b")],
                &[TaskId::new("task-a")],
            )
            .unwrap();
        let mut amendment = AppendEvent::new(
            mission.id.clone(),
            3,
            "amend-with-protected-task",
            "engine",
            "plan_amended",
        );
        amendment.plan = Some(amended);
        ledger.append(amendment).unwrap();

        let mut running = AppendEvent::new(
            mission.id.clone(),
            4,
            "continue-after-amendment",
            "engine",
            "task_running",
        );
        running.task_attempt = Some(TaskAttemptMutation {
            task_id: "task-a".to_string(),
            attempt_id: "attempt-before-amendment".to_string(),
            plan_revision: 1,
            expected_version: 1,
            next_state: TaskAttemptState::Running,
        });
        ledger.append(running).unwrap();
        assert_eq!(
            ledger.replay(&mission.id).unwrap().active_plan_revision,
            Some(2),
            "an event bound to an older task plan must not regress the active plan"
        );
    }
}

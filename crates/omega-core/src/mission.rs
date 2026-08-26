//! Mission types — the unit of work that flows through OmegaOS.
//!
//! A Mission is what the human (or Master AISB) hands to the orchestrator.
//! It is then classified, decomposed into a Plan, and executed by Workers.

use crate::routing::Complexity;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::io::Read;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Opaque mission identifier.
///
/// Newly generated values retain the historical `m-` prefix but use a
/// 128-bit digest. They are collision-resistant identifiers, not secrets or
/// authorization tokens. Deserialization remains compatible with every legacy
/// string because the wrapper intentionally imposes no format on stored IDs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MissionId(pub String);

impl MissionId {
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
        let context = mission_id_process_context();
        let (wall_direction, wall_nanos) = wall_clock_nanos();
        let monotonic_nanos = context.monotonic_origin.elapsed().as_nanos();
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"omega.mission-id.v2");
        hasher.update(&context.discriminator);
        hasher.update(&std::process::id().to_le_bytes());
        hasher.update(&counter.to_le_bytes());
        hasher.update(&[wall_direction]);
        hasher.update(&wall_nanos.to_le_bytes());
        hasher.update(&monotonic_nanos.to_le_bytes());
        let encoded = hasher.finalize().to_hex();
        Self(format!("m-{}", &encoded[..32]))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

struct MissionIdProcessContext {
    monotonic_origin: Instant,
    discriminator: [u8; 32],
}

fn mission_id_process_context() -> &'static MissionIdProcessContext {
    static CONTEXT: OnceLock<MissionIdProcessContext> = OnceLock::new();
    CONTEXT.get_or_init(|| {
        let monotonic_origin = Instant::now();
        let (wall_direction, wall_nanos) = wall_clock_nanos();
        let mut os_seed = [0_u8; 32];
        let has_os_seed = std::fs::File::open("/dev/urandom")
            .and_then(|mut source| source.read_exact(&mut os_seed))
            .is_ok();
        let context_address = (&CONTEXT as *const OnceLock<MissionIdProcessContext>) as usize;
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"omega.mission-process.v1");
        hasher.update(&std::process::id().to_le_bytes());
        hasher.update(&[wall_direction]);
        hasher.update(&wall_nanos.to_le_bytes());
        hasher.update(&context_address.to_le_bytes());
        hasher.update(&[u8::from(has_os_seed)]);
        hasher.update(&os_seed);
        MissionIdProcessContext {
            monotonic_origin,
            discriminator: *hasher.finalize().as_bytes(),
        }
    })
}

/// Shared only inside omega-core so other process-wide identifiers can bind
/// themselves to the same once-per-process discriminator.
pub(crate) fn process_discriminator() -> &'static [u8; 32] {
    &mission_id_process_context().discriminator
}

/// Preserve the direction as well as the full 128-bit nanosecond magnitude so
/// clocks before the Unix epoch do not collapse to the same zero value.
pub(crate) fn wall_clock_nanos() -> (u8, u128) {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => (1, duration.as_nanos()),
        Err(error) => (0, error.duration().as_nanos()),
    }
}

impl Default for MissionId {
    fn default() -> Self {
        Self::new()
    }
}

/// A unit of work submitted by the human or AISB.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mission {
    pub id: MissionId,
    pub project: String,
    pub text: String,
    pub working_dir: PathBuf,
    pub created_at: DateTime<Utc>,
    pub created_by: MissionSource,
    pub priority: Priority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissionSource {
    Human,
    AisbMaster,
    Oracle,
    Scheduled,
    External,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Low,
    Normal,
    High,
    Urgent,
}

impl Mission {
    pub fn new(project: impl Into<String>, text: impl Into<String>, working_dir: PathBuf) -> Self {
        Self {
            id: MissionId::new(),
            project: project.into(),
            text: text.into(),
            working_dir,
            created_at: Utc::now(),
            created_by: MissionSource::Human,
            priority: Priority::Normal,
        }
    }

    pub fn from_aisb_master(
        project: impl Into<String>,
        text: impl Into<String>,
        working_dir: PathBuf,
    ) -> Self {
        let mut m = Self::new(project, text, working_dir);
        m.created_by = MissionSource::AisbMaster;
        m
    }

    pub fn with_priority(mut self, p: Priority) -> Self {
        self.priority = p;
        self
    }
}

// ---------------------------------------------------------------------------
// Orchestration V3 contracts
// ---------------------------------------------------------------------------

pub const CONTRACT_SCHEMA_VERSION: u32 = 1;

fn contract_schema_version() -> u32 {
    CONTRACT_SCHEMA_VERSION
}

/// Mission lifecycle owned by the V3 event ledger.
///
/// The legacy [`OutcomeStatus`] remains below as a compatibility projection.
/// It is not an authority for lifecycle transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissionState {
    Created,
    Classified,
    Planned,
    Running,
    Verifying,
    Accepted,
    CorrectionRequired,
    Blocked,
    Failed,
    Reporting,
    Delivered,
    Cancelled,
}

impl MissionState {
    pub fn can_transition_to(self, next: Self) -> bool {
        use MissionState::*;
        matches!(
            (self, next),
            (Created, Classified | Cancelled)
                | (Classified, Planned | Cancelled)
                | (Planned, Running | Cancelled)
                | (Running, Verifying | Cancelled)
                | (Verifying, Accepted | CorrectionRequired | Blocked | Failed)
                | (CorrectionRequired, Running | Cancelled)
                | (Blocked, Planned | Running | Failed | Cancelled)
                | (Accepted, Reporting)
                | (Reporting, Delivered)
        )
    }

    pub fn transition(self, next: Self) -> Result<Self, InvalidTransition> {
        if self.can_transition_to(next) {
            Ok(next)
        } else {
            Err(InvalidTransition::new("mission", self, next))
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Delivered | Self::Failed | Self::Cancelled)
    }
}

/// Lifecycle of one concrete attempt at one task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskAttemptState {
    Queued,
    Running,
    CandidateDone,
    Verifying,
    Accepted,
    CorrectionRequired,
    Blocked,
    Failed,
    Cancelled,
}

impl TaskAttemptState {
    pub fn can_transition_to(self, next: Self) -> bool {
        use TaskAttemptState::*;
        matches!(
            (self, next),
            (Queued, Running | Cancelled)
                | (Running, CandidateDone | Cancelled)
                | (CandidateDone, Verifying)
                | (Verifying, Accepted | CorrectionRequired | Blocked | Failed)
                | (CorrectionRequired, Running | Cancelled)
                | (Blocked, Queued | Running | Failed | Cancelled)
        )
    }

    pub fn transition(self, next: Self) -> Result<Self, InvalidTransition> {
        if self.can_transition_to(next) {
            Ok(next)
        } else {
            Err(InvalidTransition::new("task_attempt", self, next))
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Accepted | Self::Failed | Self::Cancelled)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidTransition {
    pub machine: &'static str,
    pub from: String,
    pub to: String,
}

impl InvalidTransition {
    fn new(machine: &'static str, from: impl fmt::Debug, to: impl fmt::Debug) -> Self {
        Self {
            machine,
            from: format!("{from:?}"),
            to: format!("{to:?}"),
        }
    }
}

impl fmt::Display for InvalidTransition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid {} transition: {} -> {}",
            self.machine, self.from, self.to
        )
    }
}

impl std::error::Error for InvalidTransition {}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlanId(pub String);

impl PlanId {
    pub fn for_mission(mission_id: &MissionId) -> Self {
        Self(format!("plan-{}", mission_id.as_str()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub String);

impl TaskId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AttemptId(pub String);

impl AttemptId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

/// A verifier check fixed in the task contract before execution begins.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifierCheck {
    #[serde(default = "contract_schema_version")]
    pub schema_version: u32,
    pub check_id: String,
    pub kind: VerifierCheckKind,
    #[serde(default = "default_verifier_timeout")]
    pub timeout_secs: u64,
}

fn default_verifier_timeout() -> u64 {
    60
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum VerifierCheckKind {
    Command {
        /// Executed directly, never through a shell.
        argv: Vec<String>,
        #[serde(default)]
        cwd: Option<String>,
        #[serde(default)]
        expected_exit_code: i32,
    },
    Http {
        url: String,
        expected_status: u16,
    },
    FileExists {
        path: String,
    },
    GitObject {
        sha: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub backoff_secs: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            backoff_secs: 5,
        }
    }
}

/// Immutable task definition. Runtime attempts reference both `task_id` and
/// the exact `PlanContract.revision` that authorized them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskContract {
    #[serde(default = "contract_schema_version")]
    pub schema_version: u32,
    pub task_id: TaskId,
    pub name: String,
    pub prompt: String,
    pub acceptance_criteria: Vec<String>,
    pub verifier_checks: Vec<VerifierCheck>,
    #[serde(default)]
    pub required_capabilities: Vec<String>,
    #[serde(default)]
    pub scope: Vec<String>,
    pub risk: crate::routing::RiskLevel,
    #[serde(default)]
    pub retry_policy: RetryPolicy,
    #[serde(default)]
    pub depends_on: Vec<TaskId>,
}

/// Authoritative immutable DAG for one mission revision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanContract {
    pub plan_id: PlanId,
    pub mission_id: MissionId,
    pub revision: u64,
    #[serde(default = "contract_schema_version")]
    pub schema_version: u32,
    pub created_from_version: u64,
    pub tasks: Vec<TaskContract>,
    #[serde(default)]
    pub required_gates: Vec<String>,
    #[serde(default)]
    pub required_approvals: Vec<String>,
    pub content_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanContractError {
    UnsupportedSchema {
        contract: String,
        version: u32,
    },
    InvalidRevision {
        expected: u64,
        actual: u64,
    },
    EmptyPlan,
    EmptyTaskId,
    EmptyTaskField {
        task: String,
        field: &'static str,
    },
    EmptyAcceptanceCriteria(String),
    EmptyAcceptanceCriterion {
        task: String,
        index: usize,
    },
    EmptyVerifierChecks(String),
    EmptyVerifierId(String),
    DuplicateVerifierId {
        task: String,
        check: String,
    },
    ZeroVerifierTimeout {
        task: String,
        check: String,
    },
    InvalidVerifier {
        task: String,
        check: String,
        reason: String,
    },
    InvalidRetryPolicy(String),
    EmptyPlanRequirement {
        kind: &'static str,
    },
    DuplicatePlanRequirement {
        kind: &'static str,
        value: String,
    },
    DuplicateTaskId(String),
    UnknownDependency {
        task: String,
        dependency: String,
    },
    DependencyCycle,
    ProtectedTaskRemoved(String),
    ProtectedTaskChanged(String),
    DigestMismatch {
        expected: String,
        actual: String,
    },
    Serialization(String),
}

impl fmt::Display for PlanContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use PlanContractError::*;
        match self {
            UnsupportedSchema { contract, version } => {
                write!(f, "unsupported {contract} schema version: {version}")
            }
            InvalidRevision { expected, actual } => {
                write!(
                    f,
                    "plan revision conflict: expected {expected}, got {actual}"
                )
            }
            EmptyPlan => write!(f, "plan must contain at least one task"),
            EmptyTaskId => write!(f, "task_id must not be empty"),
            EmptyTaskField { task, field } => {
                write!(f, "task {task} has an empty {field}")
            }
            EmptyAcceptanceCriteria(task) => {
                write!(f, "task {task} must declare acceptance criteria")
            }
            EmptyAcceptanceCriterion { task, index } => {
                write!(
                    f,
                    "task {task} has an empty acceptance criterion at index {index}"
                )
            }
            EmptyVerifierChecks(task) => {
                write!(f, "task {task} must declare at least one verifier check")
            }
            EmptyVerifierId(task) => write!(f, "task {task} has an empty verifier check id"),
            DuplicateVerifierId { task, check } => {
                write!(f, "task {task} has duplicate verifier check id {check}")
            }
            ZeroVerifierTimeout { task, check } => {
                write!(f, "task {task} verifier {check} has a zero timeout")
            }
            InvalidVerifier {
                task,
                check,
                reason,
            } => write!(f, "task {task} verifier {check} is invalid: {reason}"),
            InvalidRetryPolicy(task) => {
                write!(
                    f,
                    "task {task} retry policy must allow at least one attempt"
                )
            }
            EmptyPlanRequirement { kind } => {
                write!(f, "plan contains an empty required {kind}")
            }
            DuplicatePlanRequirement { kind, value } => {
                write!(f, "plan contains duplicate required {kind}: {value}")
            }
            DuplicateTaskId(id) => write!(f, "duplicate task_id: {id}"),
            UnknownDependency { task, dependency } => {
                write!(f, "task {task} depends on unknown task {dependency}")
            }
            DependencyCycle => write!(f, "plan dependency graph contains a cycle"),
            ProtectedTaskRemoved(id) => write!(f, "protected task removed: {id}"),
            ProtectedTaskChanged(id) => write!(f, "protected task changed: {id}"),
            DigestMismatch { expected, actual } => {
                write!(
                    f,
                    "plan digest mismatch: expected {expected}, actual {actual}"
                )
            }
            Serialization(error) => write!(f, "plan serialization failed: {error}"),
        }
    }
}

impl std::error::Error for PlanContractError {}

impl PlanContract {
    pub fn new(
        mission_id: MissionId,
        revision: u64,
        created_from_version: u64,
        mut tasks: Vec<TaskContract>,
        mut required_gates: Vec<String>,
        mut required_approvals: Vec<String>,
    ) -> Result<Self, PlanContractError> {
        tasks.sort_by(|a, b| a.task_id.0.cmp(&b.task_id.0));
        for task in &mut tasks {
            task.depends_on.sort_by(|a, b| a.0.cmp(&b.0));
            task.verifier_checks
                .sort_by(|a, b| a.check_id.cmp(&b.check_id));
            task.required_capabilities.sort();
            task.scope.sort();
        }
        required_gates.sort();
        required_approvals.sort();
        let mut plan = Self {
            plan_id: PlanId::for_mission(&mission_id),
            mission_id,
            revision,
            schema_version: CONTRACT_SCHEMA_VERSION,
            created_from_version,
            tasks,
            required_gates,
            required_approvals,
            content_digest: String::new(),
        };
        plan.validate()?;
        plan.content_digest = plan.compute_digest()?;
        Ok(plan)
    }

    pub fn validate(&self) -> Result<(), PlanContractError> {
        if self.schema_version != CONTRACT_SCHEMA_VERSION {
            return Err(PlanContractError::UnsupportedSchema {
                contract: "plan".to_string(),
                version: self.schema_version,
            });
        }
        if self.revision == 0 {
            return Err(PlanContractError::InvalidRevision {
                expected: 1,
                actual: 0,
            });
        }
        if self.tasks.is_empty() {
            return Err(PlanContractError::EmptyPlan);
        }
        validate_plan_requirements("gate", &self.required_gates)?;
        validate_plan_requirements("approval", &self.required_approvals)?;
        let mut ids = HashSet::new();
        for task in &self.tasks {
            if task.schema_version != CONTRACT_SCHEMA_VERSION {
                return Err(PlanContractError::UnsupportedSchema {
                    contract: format!("task {}", task.task_id.0),
                    version: task.schema_version,
                });
            }
            if task.task_id.0.trim().is_empty() {
                return Err(PlanContractError::EmptyTaskId);
            }
            if !ids.insert(task.task_id.0.as_str()) {
                return Err(PlanContractError::DuplicateTaskId(task.task_id.0.clone()));
            }
            let task_id = task.task_id.0.clone();
            if task.name.trim().is_empty() {
                return Err(PlanContractError::EmptyTaskField {
                    task: task_id,
                    field: "name",
                });
            }
            if task.prompt.trim().is_empty() {
                return Err(PlanContractError::EmptyTaskField {
                    task: task.task_id.0.clone(),
                    field: "prompt",
                });
            }
            if task.acceptance_criteria.is_empty() {
                return Err(PlanContractError::EmptyAcceptanceCriteria(
                    task.task_id.0.clone(),
                ));
            }
            if let Some(index) = task
                .acceptance_criteria
                .iter()
                .position(|criterion| criterion.trim().is_empty())
            {
                return Err(PlanContractError::EmptyAcceptanceCriterion {
                    task: task.task_id.0.clone(),
                    index,
                });
            }
            if task.verifier_checks.is_empty() {
                return Err(PlanContractError::EmptyVerifierChecks(
                    task.task_id.0.clone(),
                ));
            }
            if task.retry_policy.max_attempts == 0 {
                return Err(PlanContractError::InvalidRetryPolicy(
                    task.task_id.0.clone(),
                ));
            }
            let mut check_ids = HashSet::new();
            for check in &task.verifier_checks {
                validate_verifier_check(&task.task_id.0, check, &mut check_ids)?;
            }
        }
        for task in &self.tasks {
            for dependency in &task.depends_on {
                if !ids.contains(dependency.0.as_str()) {
                    return Err(PlanContractError::UnknownDependency {
                        task: task.task_id.0.clone(),
                        dependency: dependency.0.clone(),
                    });
                }
            }
        }
        if self.has_cycle() {
            return Err(PlanContractError::DependencyCycle);
        }
        Ok(())
    }

    /// Validate both structure and the immutable content fingerprint. This is
    /// required at trust boundaries after deserialization.
    pub fn verify_integrity(&self) -> Result<(), PlanContractError> {
        self.validate()?;
        let actual = self.compute_digest()?;
        if actual != self.content_digest {
            return Err(PlanContractError::DigestMismatch {
                expected: self.content_digest.clone(),
                actual,
            });
        }
        Ok(())
    }

    /// Create a new immutable revision. Started or accepted tasks are protected:
    /// their IDs and definitions cannot disappear or change.
    pub fn amend(
        &self,
        expected_revision: u64,
        created_from_version: u64,
        tasks: Vec<TaskContract>,
        protected_task_ids: &[TaskId],
    ) -> Result<Self, PlanContractError> {
        if self.revision != expected_revision {
            return Err(PlanContractError::InvalidRevision {
                expected: expected_revision,
                actual: self.revision,
            });
        }
        let replacements: HashMap<&str, &TaskContract> = tasks
            .iter()
            .map(|task| (task.task_id.as_str(), task))
            .collect();
        for protected in protected_task_ids {
            let previous = self
                .tasks
                .iter()
                .find(|task| task.task_id == *protected)
                .ok_or_else(|| PlanContractError::ProtectedTaskRemoved(protected.0.clone()))?;
            let replacement = replacements
                .get(protected.as_str())
                .ok_or_else(|| PlanContractError::ProtectedTaskRemoved(protected.0.clone()))?;
            if previous != *replacement {
                return Err(PlanContractError::ProtectedTaskChanged(protected.0.clone()));
            }
        }
        Self::new(
            self.mission_id.clone(),
            self.revision.saturating_add(1),
            created_from_version,
            tasks,
            self.required_gates.clone(),
            self.required_approvals.clone(),
        )
    }

    fn compute_digest(&self) -> Result<String, PlanContractError> {
        #[derive(Serialize)]
        struct DigestInput<'a> {
            plan_id: &'a PlanId,
            mission_id: &'a MissionId,
            revision: u64,
            schema_version: u32,
            created_from_version: u64,
            tasks: &'a [TaskContract],
            required_gates: &'a [String],
            required_approvals: &'a [String],
        }
        let input = DigestInput {
            plan_id: &self.plan_id,
            mission_id: &self.mission_id,
            revision: self.revision,
            schema_version: self.schema_version,
            created_from_version: self.created_from_version,
            tasks: &self.tasks,
            required_gates: &self.required_gates,
            required_approvals: &self.required_approvals,
        };
        let bytes = serde_json::to_vec(&input)
            .map_err(|error| PlanContractError::Serialization(error.to_string()))?;
        Ok(blake3::hash(&bytes).to_hex().to_string())
    }

    fn has_cycle(&self) -> bool {
        fn visit(
            id: &str,
            tasks: &HashMap<&str, &TaskContract>,
            visiting: &mut HashSet<String>,
            visited: &mut HashSet<String>,
        ) -> bool {
            if visited.contains(id) {
                return false;
            }
            if !visiting.insert(id.to_string()) {
                return true;
            }
            if let Some(task) = tasks.get(id) {
                for dependency in &task.depends_on {
                    if visit(dependency.as_str(), tasks, visiting, visited) {
                        return true;
                    }
                }
            }
            visiting.remove(id);
            visited.insert(id.to_string());
            false
        }

        let tasks: HashMap<&str, &TaskContract> = self
            .tasks
            .iter()
            .map(|task| (task.task_id.as_str(), task))
            .collect();
        let mut visiting = HashSet::new();
        let mut visited = HashSet::new();
        tasks
            .keys()
            .any(|id| visit(id, &tasks, &mut visiting, &mut visited))
    }
}

fn validate_plan_requirements(
    kind: &'static str,
    requirements: &[String],
) -> Result<(), PlanContractError> {
    let mut seen = HashSet::new();
    for requirement in requirements {
        let trimmed = requirement.trim();
        if trimmed.is_empty() {
            return Err(PlanContractError::EmptyPlanRequirement { kind });
        }
        if !seen.insert(trimmed) {
            return Err(PlanContractError::DuplicatePlanRequirement {
                kind,
                value: trimmed.to_string(),
            });
        }
    }
    Ok(())
}

fn validate_verifier_check<'a>(
    task_id: &str,
    check: &'a VerifierCheck,
    seen: &mut HashSet<&'a str>,
) -> Result<(), PlanContractError> {
    if check.schema_version != CONTRACT_SCHEMA_VERSION {
        return Err(PlanContractError::UnsupportedSchema {
            contract: format!("verifier {}", check.check_id),
            version: check.schema_version,
        });
    }
    let check_id = check.check_id.trim();
    if check_id.is_empty() {
        return Err(PlanContractError::EmptyVerifierId(task_id.to_string()));
    }
    if !seen.insert(check_id) {
        return Err(PlanContractError::DuplicateVerifierId {
            task: task_id.to_string(),
            check: check_id.to_string(),
        });
    }
    if check.timeout_secs == 0 {
        return Err(PlanContractError::ZeroVerifierTimeout {
            task: task_id.to_string(),
            check: check_id.to_string(),
        });
    }
    let invalid = match &check.kind {
        VerifierCheckKind::Command { argv, .. } => argv
            .first()
            .filter(|program| !program.trim().is_empty())
            .is_none()
            .then_some("command argv must name a program"),
        VerifierCheckKind::Http {
            url,
            expected_status,
        } => {
            if url.trim().is_empty() {
                Some("HTTP URL must not be empty")
            } else if !(100..=599).contains(expected_status) {
                Some("HTTP expected_status must be between 100 and 599")
            } else {
                None
            }
        }
        VerifierCheckKind::FileExists { path } => path
            .trim()
            .is_empty()
            .then_some("file path must not be empty"),
        VerifierCheckKind::GitObject { sha } => sha
            .trim()
            .is_empty()
            .then_some("git object id must not be empty"),
    };
    if let Some(reason) = invalid {
        return Err(PlanContractError::InvalidVerifier {
            task: task_id.to_string(),
            check: check_id.to_string(),
            reason: reason.to_string(),
        });
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceClaim {
    #[serde(default = "contract_schema_version")]
    pub schema_version: u32,
    pub claim_id: String,
    pub mission_id: MissionId,
    pub task_id: TaskId,
    pub attempt_id: AttemptId,
    pub check_id: String,
    pub claimed_at: DateTime<Utc>,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationOutcome {
    Passed,
    Failed,
    Rejected,
    TimedOut,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    #[serde(default = "contract_schema_version")]
    pub schema_version: u32,
    pub observation_id: String,
    pub mission_id: MissionId,
    pub task_id: TaskId,
    pub attempt_id: AttemptId,
    pub check_id: String,
    pub verifier: String,
    pub observed_at: DateTime<Utc>,
    pub outcome: ObservationOutcome,
    pub output_hash: Option<String>,
    pub detail: String,
}

/// Compatibility name retained for callers that adopted the earlier draft.
pub type EvidenceObservation = Observation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcceptanceVerdict {
    Accepted,
    CorrectionRequired,
    Blocked,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AcceptanceDecision {
    #[serde(default = "contract_schema_version")]
    pub schema_version: u32,
    pub decision_id: String,
    pub mission_id: MissionId,
    pub task_id: Option<TaskId>,
    pub attempt_id: Option<AttemptId>,
    pub plan_revision: u64,
    pub verdict: AcceptanceVerdict,
    pub observation_ids: Vec<String>,
    pub reasons: Vec<String>,
    pub decided_by: String,
    pub decided_at: DateTime<Utc>,
}

/// A decomposition of a Mission into executable tasks.
/// Built by Oracle/KEYMAKER for COMPLEX or EPIC missions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub mission_id: MissionId,
    pub complexity: Complexity,
    pub strategy: PlanStrategy,
    pub tasks: Vec<Task>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanStrategy {
    /// Single worker handles the whole mission
    Direct,
    /// Oracle decomposes and dispatches sequentially
    Sequential,
    /// Multiple workers run in parallel with shared file scopes
    Parallel,
    /// Team in split panes coordinated by a lead
    Team,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub prompt: String,
    pub files_owned: Vec<String>,
    pub depends_on: Vec<String>,
    pub agent: String,
    #[serde(default)]
    pub estimated_minutes: u32,
}

impl Task {
    pub fn new(id: impl Into<String>, name: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            prompt: prompt.into(),
            files_owned: Vec::new(),
            depends_on: Vec::new(),
            agent: "codex".to_string(),
            estimated_minutes: 15,
        }
    }
}

/// Outcome of a Mission after execution + verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    pub mission_id: MissionId,
    pub status: OutcomeStatus,
    pub workers: Vec<WorkerResult>,
    pub gate: Option<crate::gate::GateResult>,
    pub audit_recommendations: Vec<String>,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub summary: String,
}

impl Outcome {
    pub fn is_success(&self) -> bool {
        matches!(self.status, OutcomeStatus::Success)
    }

    pub fn duration_secs(&self) -> u64 {
        (self.finished_at - self.started_at).num_seconds().max(0) as u64
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeStatus {
    Success,
    PartialSuccess,
    Failed,
    Aborted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerResult {
    pub task_id: String,
    pub session_name: String,
    pub status: crate::done::DoneStatus,
    pub summary: String,
    pub commit: Option<String>,
    pub duration_secs: u64,
}

// NOTE: a `MissionTracker` (missions.json — "persists active missions for
// AISB visibility") used to live here. It had ZERO callers — no dispatch
// path ever tracked, nothing ever completed, missions.json never existed
// on any install — and there is no mission QUEUE anywhere (dispatch_oracle
// always reserves + spawns immediately). Deleted as dead code rather than
// speculatively wired; if a real queue ever lands, gate dispatch on
// `OracleRegistry::count_active(project)` and persist the overflow there.

#[cfg(test)]
mod v3_contract_tests {
    use super::*;
    use std::collections::HashSet as StdHashSet;

    fn task(id: &str, depends_on: &[&str]) -> TaskContract {
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
            required_capabilities: vec!["launch".to_string()],
            scope: vec![format!("src/{id}.rs")],
            risk: crate::routing::RiskLevel::Medium,
            retry_policy: RetryPolicy::default(),
            depends_on: depends_on.iter().map(|id| TaskId::new(*id)).collect(),
        }
    }

    #[test]
    fn generated_mission_ids_have_a_stable_versioned_shape() {
        for _ in 0..256 {
            let id = MissionId::new();
            let hex = id.0.strip_prefix("m-").expect("stable mission prefix");
            assert_eq!(hex.len(), 32, "128 bits must be emitted as 32 hex digits");
            assert!(
                hex.bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
                "mission digest must use canonical lowercase hexadecimal"
            );
        }

        let legacy: MissionId = serde_json::from_str("\"m-0123abcdef\"").unwrap();
        assert_eq!(legacy.as_str(), "m-0123abcdef");
    }

    #[test]
    fn generated_mission_ids_are_unique_at_high_volume() {
        const SAMPLE_SIZE: usize = 100_000;
        let ids: StdHashSet<String> = (0..SAMPLE_SIZE).map(|_| MissionId::new().0).collect();
        assert_eq!(ids.len(), SAMPLE_SIZE);
    }

    #[test]
    fn mission_state_rejects_shortcuts_and_allows_blocked_resume_cancel() {
        assert!(MissionState::Created
            .transition(MissionState::Accepted)
            .is_err());
        assert_eq!(
            MissionState::Blocked.transition(MissionState::Planned),
            Ok(MissionState::Planned)
        );
        assert_eq!(
            MissionState::Blocked.transition(MissionState::Running),
            Ok(MissionState::Running)
        );
        assert_eq!(
            MissionState::Blocked.transition(MissionState::Cancelled),
            Ok(MissionState::Cancelled)
        );
        for terminal in [
            MissionState::Delivered,
            MissionState::Failed,
            MissionState::Cancelled,
        ] {
            assert!(terminal.is_terminal());
            for next in [
                MissionState::Created,
                MissionState::Running,
                MissionState::Accepted,
            ] {
                assert!(!terminal.can_transition_to(next));
            }
        }
    }

    #[test]
    fn task_attempt_requires_candidate_and_verification() {
        assert!(TaskAttemptState::Running
            .transition(TaskAttemptState::Accepted)
            .is_err());
        assert_eq!(
            TaskAttemptState::Running
                .transition(TaskAttemptState::CandidateDone)
                .unwrap()
                .transition(TaskAttemptState::Verifying)
                .unwrap()
                .transition(TaskAttemptState::Accepted)
                .unwrap(),
            TaskAttemptState::Accepted
        );
        assert_eq!(
            TaskAttemptState::Blocked.transition(TaskAttemptState::Running),
            Ok(TaskAttemptState::Running)
        );
        assert_eq!(
            TaskAttemptState::Blocked.transition(TaskAttemptState::Cancelled),
            Ok(TaskAttemptState::Cancelled)
        );
    }

    #[test]
    fn transition_api_and_transition_matrix_agree_for_every_state_pair() {
        let mission_states = [
            MissionState::Created,
            MissionState::Classified,
            MissionState::Planned,
            MissionState::Running,
            MissionState::Verifying,
            MissionState::Accepted,
            MissionState::CorrectionRequired,
            MissionState::Blocked,
            MissionState::Failed,
            MissionState::Reporting,
            MissionState::Delivered,
            MissionState::Cancelled,
        ];
        for from in mission_states {
            for to in mission_states {
                assert_eq!(
                    from.transition(to).is_ok(),
                    from.can_transition_to(to),
                    "{from:?} -> {to:?}"
                );
            }
        }

        let attempt_states = [
            TaskAttemptState::Queued,
            TaskAttemptState::Running,
            TaskAttemptState::CandidateDone,
            TaskAttemptState::Verifying,
            TaskAttemptState::Accepted,
            TaskAttemptState::CorrectionRequired,
            TaskAttemptState::Blocked,
            TaskAttemptState::Failed,
            TaskAttemptState::Cancelled,
        ];
        for from in attempt_states {
            for to in attempt_states {
                assert_eq!(
                    from.transition(to).is_ok(),
                    from.can_transition_to(to),
                    "{from:?} -> {to:?}"
                );
            }
        }
    }

    #[test]
    fn plan_contract_is_stable_and_rejects_cycles() {
        let mission = MissionId("m-contract".to_string());
        let plan_a = PlanContract::new(
            mission.clone(),
            1,
            3,
            vec![task("b", &["a"]), task("a", &[])],
            vec!["gate".to_string()],
            vec![],
        )
        .unwrap();
        let plan_b = PlanContract::new(
            mission.clone(),
            1,
            3,
            vec![task("a", &[]), task("b", &["a"])],
            vec!["gate".to_string()],
            vec![],
        )
        .unwrap();
        assert_eq!(plan_a.content_digest, plan_b.content_digest);
        plan_a.verify_integrity().unwrap();

        let cycle = PlanContract::new(
            mission,
            1,
            3,
            vec![task("a", &["b"]), task("b", &["a"])],
            vec![],
            vec![],
        );
        assert_eq!(cycle.unwrap_err(), PlanContractError::DependencyCycle);
    }

    #[test]
    fn plan_integrity_rejects_post_creation_mutation() {
        let mission_id = MissionId("mission-integrity".to_string());
        let mut plan =
            PlanContract::new(mission_id, 1, 4, vec![task("a", &[])], vec![], vec![]).unwrap();
        plan.tasks[0].prompt.push_str(" silently changed");
        assert!(matches!(
            plan.verify_integrity(),
            Err(PlanContractError::DigestMismatch { .. })
        ));
    }

    #[test]
    fn amendment_cannot_change_protected_task() {
        let plan = PlanContract::new(
            MissionId("m-amend".to_string()),
            1,
            2,
            vec![task("a", &[])],
            vec![],
            vec![],
        )
        .unwrap();
        let mut changed = task("a", &[]);
        changed.acceptance_criteria = vec!["weaker criterion".to_string()];
        let result = plan.amend(1, 3, vec![changed], &[TaskId::new("a")]);
        assert_eq!(
            result.unwrap_err(),
            PlanContractError::ProtectedTaskChanged("a".to_string())
        );
    }

    #[test]
    fn plan_contract_rejects_empty_or_unverifiable_work() {
        let mission = MissionId("m-invalid-plan".to_string());
        assert_eq!(
            PlanContract::new(mission.clone(), 1, 1, vec![], vec![], vec![]).unwrap_err(),
            PlanContractError::EmptyPlan
        );

        let mut invalid = task("a", &[]);
        invalid.acceptance_criteria.clear();
        assert_eq!(
            PlanContract::new(mission.clone(), 1, 1, vec![invalid], vec![], vec![]).unwrap_err(),
            PlanContractError::EmptyAcceptanceCriteria("a".to_string())
        );

        let mut invalid = task("a", &[]);
        invalid.verifier_checks.clear();
        assert_eq!(
            PlanContract::new(mission.clone(), 1, 1, vec![invalid], vec![], vec![]).unwrap_err(),
            PlanContractError::EmptyVerifierChecks("a".to_string())
        );

        let mut invalid = task("a", &[]);
        invalid.retry_policy.max_attempts = 0;
        assert_eq!(
            PlanContract::new(mission, 1, 1, vec![invalid], vec![], vec![]).unwrap_err(),
            PlanContractError::InvalidRetryPolicy("a".to_string())
        );
    }

    #[test]
    fn plan_contract_rejects_malformed_verifiers_and_requirements() {
        let mission = MissionId("m-invalid-verifier".to_string());
        let mut invalid = task("a", &[]);
        invalid.verifier_checks[0].timeout_secs = 0;
        assert!(matches!(
            PlanContract::new(mission.clone(), 1, 1, vec![invalid], vec![], vec![]),
            Err(PlanContractError::ZeroVerifierTimeout { .. })
        ));

        let mut invalid = task("a", &[]);
        invalid.verifier_checks[0].kind = VerifierCheckKind::Command {
            argv: vec![],
            cwd: None,
            expected_exit_code: 0,
        };
        assert!(matches!(
            PlanContract::new(mission.clone(), 1, 1, vec![invalid], vec![], vec![]),
            Err(PlanContractError::InvalidVerifier { .. })
        ));

        assert!(matches!(
            PlanContract::new(
                mission,
                1,
                1,
                vec![task("a", &[])],
                vec!["gate".to_string(), "gate".to_string()],
                vec![]
            ),
            Err(PlanContractError::DuplicatePlanRequirement { kind: "gate", .. })
        ));
    }
}

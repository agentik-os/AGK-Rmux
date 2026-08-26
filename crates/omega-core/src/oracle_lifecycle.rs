//! Oracle Lifecycle — 5-step workflow state machine.
//!
//! Implements the VPS Oracle pattern: Analyze → Dispatch → Monitor → Verify → Report.
//! Oracles are on-demand project managers: they decompose, delegate, monitor, and verify.
//! They NEVER edit code directly — all code changes go through workers.

use crate::done::DoneStatus;
use crate::mission::{Mission, MissionId, MissionState, TaskAttemptState, WorkerResult};
use crate::mission_ledger::{AppendOutcome, MissionLedger, MissionProjection};
use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Schema of the JSON compatibility projections consumed by the TUI, patrol,
/// Telegram and older automation. These files are deliberately not an input to
/// the V3 state machine: their provenance only points back to the append-only
/// mission ledger that produced them.
pub const COMPATIBILITY_PROJECTION_SCHEMA_VERSION: u32 = 1;

/// Provenance carried by every compatibility projection produced by a new V3
/// mission.
///
/// `source_version` and `source_event_sequence` are intentionally both stored.
/// They are equal for the single-stream ledger today, but naming both prevents
/// a future multi-stream projection from silently changing the meaning of the
/// field. `source_projection_hash` is the hash committed by the ledger, never a
/// digest recomputed from this mutable JSON view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompatibilityProjection {
    pub projection_schema_version: u32,
    pub source: String,
    pub source_schema_version: u32,
    pub source_mission_id: MissionId,
    pub source_version: u64,
    pub source_event_sequence: u64,
    pub source_projection_hash: String,
}

impl CompatibilityProjection {
    pub fn from_append(outcome: &AppendOutcome) -> Self {
        Self {
            projection_schema_version: COMPATIBILITY_PROJECTION_SCHEMA_VERSION,
            source: "mission-engine-v3.sqlite3".to_string(),
            source_schema_version: outcome.event.schema_version,
            source_mission_id: outcome.projection.mission_id.clone(),
            source_version: outcome.projection.version,
            source_event_sequence: outcome.event.sequence,
            source_projection_hash: outcome.projection.projection_hash.clone(),
        }
    }

    /// Prove that this mutable JSON view descends from a real ledger event.
    ///
    /// A compatibility projection may be older than the current mission
    /// version, so exact equality with the live projection would reject every
    /// oracle as soon as its first worker is queued. Instead, the referenced
    /// immutable event must exist and the live stream must not have moved
    /// backwards. When the stamp is current, its projection hash must match
    /// exactly as an additional corruption check.
    pub fn validate(
        &self,
        expected_mission_id: &MissionId,
        ledger: &MissionLedger,
    ) -> Result<MissionProjection> {
        if self.projection_schema_version != COMPATIBILITY_PROJECTION_SCHEMA_VERSION {
            bail!(
                "unsupported compatibility projection schema {}",
                self.projection_schema_version
            );
        }
        if self.source != "mission-engine-v3.sqlite3" {
            bail!(
                "unsupported compatibility projection source `{}`",
                self.source
            );
        }
        if &self.source_mission_id != expected_mission_id {
            bail!(
                "compatibility projection mission mismatch: state={}, source={}",
                expected_mission_id.as_str(),
                self.source_mission_id.as_str()
            );
        }
        if self.source_version == 0
            || self.source_event_sequence == 0
            || self.source_version != self.source_event_sequence
            || self.source_projection_hash.trim().is_empty()
        {
            bail!("invalid compatibility projection provenance");
        }
        let projection = ledger.mission(expected_mission_id)?.ok_or_else(|| {
            anyhow::anyhow!(
                "mission {} is absent from the authoritative ledger",
                expected_mission_id.as_str()
            )
        })?;
        if projection.version < self.source_version {
            bail!(
                "compatibility projection is from impossible future version {} (ledger {})",
                self.source_version,
                projection.version
            );
        }
        let event = ledger
            .events(expected_mission_id)?
            .into_iter()
            .find(|event| event.sequence == self.source_event_sequence)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "compatibility projection source event {} is absent",
                    self.source_event_sequence
                )
            })?;
        if event.schema_version != self.source_schema_version {
            bail!(
                "compatibility projection source schema mismatch: stamp={}, event={}",
                self.source_schema_version,
                event.schema_version
            );
        }
        let historical = ledger
            .projection_at(expected_mission_id, self.source_event_sequence)?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "compatibility projection source version {} cannot be replayed",
                    self.source_event_sequence
                )
            })?;
        if historical.version != self.source_version
            || historical.projection_hash != self.source_projection_hash
        {
            bail!("compatibility projection hash differs from authoritative ledger");
        }
        Ok(projection)
    }
}

/// Canonical on-host ledger location. Keeping this in core removes the path
/// literals that previously drifted across dispatch, worker, team and done
/// commands.
pub fn mission_ledger_path(state_dir: &Path) -> PathBuf {
    state_dir.join("mission-engine-v3.sqlite3")
}

// ---------------------------------------------------------------------------
// Oracle State Machine
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OraclePhase {
    /// Initial: reading CLAUDE.md, decomposing, defining success criteria
    Analyze,
    /// Dispatching workers via structured context template
    Dispatch,
    /// Monitoring worker panes, stall detection, inbox events
    Monitor,
    /// All workers done — running quality gate + verification
    Verify,
    /// Writing signal file, reporting results
    Report,
    /// Terminal: oracle has completed its mission
    Done,
}

impl OraclePhase {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Analyze => "ANALYSE",
            Self::Dispatch => "DISPATCH",
            Self::Monitor => "MONITORING",
            Self::Verify => "VERIFY",
            Self::Report => "REPORT",
            Self::Done => "DONE",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Done)
    }
}

// ---------------------------------------------------------------------------
// Oracle State — persisted to state dir
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleState {
    pub oracle_name: String,
    pub project: String,
    pub mission_id: MissionId,
    pub mission_text: String,
    pub working_dir: PathBuf,
    pub phase: OraclePhase,
    pub workers: Vec<WorkerEntry>,
    pub god_mode: Option<GodModeState>,
    pub ship_requested: bool,
    pub started_at: DateTime<Utc>,
    pub phase_entered_at: DateTime<Utc>,
    #[serde(default)]
    pub phase_history: Vec<PhaseTransition>,
    /// N10: timestamp at which this oracle reached a real *closeable* state —
    /// i.e. it wrote a genuine done-signal / finished its mission, NOT merely
    /// that `all_workers_terminal()` became true. An oracle is reusable
    /// (`dispatch::find_available`) ONLY when this is set. `None` means the
    /// oracle is still live work, even if every worker is in a terminal status.
    #[serde(default)]
    pub closeable_since: Option<DateTime<Utc>>,
    /// Claude `--session-id` UUID for this oracle, persisted for the record.
    /// NOTE: every dispatch/resurrect mints a FRESH id (resolve_session_id —
    /// reusing one collides: `--session-id` CREATES, it does not resume), so
    /// this field is bookkeeping, not lineage. `None` for states written
    /// before this field existed.
    #[serde(default)]
    pub session_id: Option<String>,
    /// Provenance of this mutable legacy view. `None` means a pre-V3 state file
    /// and is accepted only by legacy read-only consumers. Every new mutation
    /// path must call [`OracleState::require_ledger_authority`] and therefore
    /// fails closed when this stamp is absent or corrupt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compatibility_projection: Option<CompatibilityProjection>,
    /// Optimistic concurrency revision for this compatibility projection.
    /// It is independent from the authoritative mission version.
    #[serde(default, rename = "_omega_storage_revision")]
    pub storage_revision: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerEntry {
    pub session_name: String,
    pub task_id: String,
    pub task_name: String,
    /// V3 task-attempt identity. Absent on pre-ledger workers.
    #[serde(default)]
    pub attempt_id: Option<String>,
    /// Immutable plan revision that authorized this attempt.
    #[serde(default)]
    pub plan_revision: Option<u64>,
    pub files_owned: Vec<String>,
    pub dispatched_at: DateTime<Utc>,
    pub status: WorkerEntryStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerEntryStatus {
    Running,
    DoneClean,
    Pending,
    Failed,
    Blocked,
    Stalled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseTransition {
    pub from: OraclePhase,
    pub to: OraclePhase,
    pub at: DateTime<Utc>,
    pub reason: String,
}

impl OracleState {
    pub fn new(oracle_name: &str, mission: &Mission) -> Self {
        let now = Utc::now();
        Self {
            oracle_name: oracle_name.to_string(),
            project: mission.project.clone(),
            mission_id: mission.id.clone(),
            mission_text: mission.text.clone(),
            working_dir: mission.working_dir.clone(),
            phase: OraclePhase::Analyze,
            workers: Vec::new(),
            god_mode: None,
            ship_requested: false,
            started_at: now,
            phase_entered_at: now,
            phase_history: Vec::new(),
            closeable_since: None,
            session_id: None,
            compatibility_projection: None,
            storage_revision: 0,
        }
    }

    /// Minimal state for a live oracle that has no persisted mission yet.
    /// Lets `spawn-worker` ALWAYS record the worker→oracle link (the session
    /// menu nests workers under their governing oracle from it) even when the
    /// oracle never wrote a full `OracleState`. Placeholders are overwritten by
    /// a later full `new()` / `transition()`.
    pub fn new_minimal(oracle_name: &str, project: &str, working_dir: PathBuf) -> Self {
        let now = Utc::now();
        Self {
            oracle_name: oracle_name.to_string(),
            project: project.to_string(),
            mission_id: MissionId(String::new()),
            mission_text: String::new(),
            working_dir,
            // A placeholder oracle has not declared a mission yet, so the only
            // truthful initial phase is the first one (`Analyze`, matching
            // `new()`). `Dispatch` falsely implies workers are already being
            // dispatched; a later full `new()`/`transition()` sets the real phase.
            phase: OraclePhase::Analyze,
            workers: Vec::new(),
            god_mode: None,
            ship_requested: false,
            started_at: now,
            phase_entered_at: now,
            phase_history: Vec::new(),
            closeable_since: None,
            session_id: None,
            compatibility_projection: None,
            storage_revision: 0,
        }
    }

    /// Build the legacy OracleState only after a V3 ledger append committed.
    /// This is the sole constructor for new dispatches; [`OracleState::new`]
    /// remains only for parsing/tests and historical compatibility.
    pub fn from_ledger(
        oracle_name: &str,
        mission: &Mission,
        source: &AppendOutcome,
    ) -> Result<Self> {
        if source.projection.mission_id != mission.id
            || source.event.mission_id != mission.id
            || source.event.sequence != source.projection.version
        {
            bail!("cannot project OracleState from a different ledger mission");
        }
        let mut state = Self::new(oracle_name, mission);
        state.compatibility_projection = Some(CompatibilityProjection::from_append(source));
        Ok(state)
    }

    /// Validate the compatibility stamp against the ledger. Mutating callers
    /// use this rather than trusting `mission_id`, phase or workers from JSON.
    pub fn require_ledger_authority(&self, ledger: &MissionLedger) -> Result<MissionProjection> {
        let stamp = self.compatibility_projection.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "oracle {} is a legacy projection without V3 authority",
                self.oracle_name
            )
        })?;
        let projection = stamp.validate(&self.mission_id, ledger)?;
        let mission = ledger.mission_record(&self.mission_id)?.ok_or_else(|| {
            anyhow::anyhow!(
                "mission {} has no immutable mission record",
                self.mission_id.as_str()
            )
        })?;
        if self.project != mission.project || self.mission_text != mission.text {
            bail!(
                "OracleState {} immutable mission fields differ from ledger",
                self.oracle_name
            );
        }
        if !same_canonical_workspace(&self.working_dir, &mission.working_dir) {
            bail!(
                "OracleState {} working directory differs from immutable mission: state={}, ledger={}",
                self.oracle_name,
                self.working_dir.display(),
                mission.working_dir.display()
            );
        }
        if !oracle_phase_compatible(self.phase, projection.state) {
            bail!(
                "OracleState {} phase {:?} is incompatible with ledger state {:?}",
                self.oracle_name,
                self.phase,
                projection.state
            );
        }
        self.validate_worker_bindings(ledger, &projection)?;
        Ok(projection)
    }

    pub fn transition(&mut self, to: OraclePhase, reason: &str) {
        let from = self.phase;
        self.phase_history.push(PhaseTransition {
            from,
            to,
            at: Utc::now(),
            reason: reason.to_string(),
        });
        self.phase = to;
        self.phase_entered_at = Utc::now();
        tracing::info!(
            oracle = %self.oracle_name,
            from = ?from,
            to = ?to,
            reason = %reason,
            "Oracle phase transition"
        );
    }

    pub fn register_worker(&mut self, entry: WorkerEntry) {
        // Upsert — never DUPLICATE the same session (retry / concurrent
        // dispatch would otherwise break terminal detection), but a
        // RE-DISPATCH under the same deterministic worker name must refresh
        // the entry: patrol's worker freshness guard dates done signals
        // against `dispatched_at`, and keeping the old timestamp would let
        // the PREVIOUS mission's stale done.json pass as fresh and
        // insta-finish (then reap) the new worker.
        if let Some(w) = self
            .workers
            .iter_mut()
            .find(|w| w.session_name == entry.session_name)
        {
            *w = entry;
            return;
        }
        self.workers.push(entry);
    }

    pub fn update_worker_status(&mut self, session: &str, status: WorkerEntryStatus) {
        if let Some(w) = self.workers.iter_mut().find(|w| w.session_name == session) {
            w.status = status;
        }
    }

    pub fn all_workers_terminal(&self) -> bool {
        // Blocked + Stalled are terminal too — a worker in either state will not
        // progress on its own, so the oracle must move on (to Verify/Report)
        // instead of hanging in Monitor forever waiting for done_clean/failed.
        !self.workers.is_empty()
            && self.workers.iter().all(|w| {
                matches!(
                    w.status,
                    WorkerEntryStatus::DoneClean
                        | WorkerEntryStatus::Failed
                        | WorkerEntryStatus::Blocked
                        | WorkerEntryStatus::Stalled
                )
            })
    }

    /// N10: mark this oracle as having reached a real, closeable done-state
    /// (it wrote a genuine done-signal / finished its mission). Idempotent —
    /// the first timestamp wins so the close-grace clock isn't reset on every
    /// patrol tick. Reaching the terminal `Done` phase implies this too.
    pub fn mark_closeable(&mut self) {
        if self.closeable_since.is_none() {
            self.closeable_since = Some(Utc::now());
        }
    }

    /// N10: an oracle has a real done-signal / is in a closeable state when its
    /// phase is terminal `Done` OR `mark_closeable` was recorded. This is the
    /// authoritative "safe to reuse / close" predicate — STRICTLY stronger than
    /// `all_workers_terminal()`, which only means no worker will progress on its
    /// own (the oracle may still owe Verify/Report work).
    pub fn is_closeable(&self) -> bool {
        self.phase.is_terminal() || self.closeable_since.is_some()
    }

    /// Convenience alias for `is_closeable` matching the N10 wording — true when
    /// the oracle has written a real done-signal / reached a closeable state.
    pub fn has_done_signal(&self) -> bool {
        self.is_closeable()
    }

    pub fn any_worker_failed(&self) -> bool {
        self.workers
            .iter()
            .any(|w| w.status == WorkerEntryStatus::Failed)
    }

    pub fn running_workers(&self) -> Vec<&WorkerEntry> {
        self.workers
            .iter()
            .filter(|w| w.status == WorkerEntryStatus::Running)
            .collect()
    }

    pub fn duration_secs(&self) -> u64 {
        (Utc::now() - self.started_at).num_seconds().max(0) as u64
    }

    /// Filename key — the oracle name minus a single leading `oracle-`
    /// prefix, the SAME rule as `OracleDoneSignal::oracle_key`. Every
    /// caller holds the FULL session name (`oracle-X`), and formatting it
    /// straight into `oracle-{}.state.json` produced double-prefixed files
    /// (`oracle-oracle-X.state.json`) that the shell-side consumers
    /// (stuck-oracle-alert derives its done/progress probe paths from the
    /// state basename) then mis-keyed — finished oracles were never
    /// recognized as done and false stall alerts fired forever.
    fn state_key(oracle_name: &str) -> Result<&str> {
        crate::scope::validate_session_identity(oracle_name)?;
        let key = oracle_name.strip_prefix("oracle-").unwrap_or(oracle_name);
        crate::scope::validate_session_identity(key)?;
        Ok(key)
    }

    fn paths(state_dir: &Path, oracle_name: &str) -> Result<(PathBuf, PathBuf, String)> {
        let key = Self::state_key(oracle_name)?.to_string();
        Ok((
            state_dir.join(format!("oracle-{key}.state.json")),
            state_dir.join(format!("oracle-oracle-{key}.state.json")),
            key,
        ))
    }

    fn validate_document_identity(&self, expected_key: &str) -> Result<()> {
        let actual_key = Self::state_key(&self.oracle_name)?;
        if actual_key != expected_key {
            bail!(
                "oracle state filename/session mismatch: filename={}, document={}",
                expected_key,
                self.oracle_name
            );
        }
        Ok(())
    }

    /// Validate every authority-relevant worker reference against the current
    /// ledger plan and exact task-attempt tuple. A compatibility stamp proves
    /// ancestry only; it never authorizes a forged worker mutation by itself.
    fn validate_authoritative_workers(&self, ledger: &MissionLedger) -> Result<()> {
        let Some(_) = &self.compatibility_projection else {
            if self
                .workers
                .iter()
                .any(|worker| worker.attempt_id.is_some() || worker.plan_revision.is_some())
            {
                bail!(
                    "legacy oracle {} cannot persist V3 worker authority",
                    self.oracle_name
                );
            }
            return Ok(());
        };

        self.require_ledger_authority(ledger)?;
        Ok(())
    }

    fn validate_worker_bindings(
        &self,
        ledger: &MissionLedger,
        projection: &MissionProjection,
    ) -> Result<()> {
        let active_plan = ledger.active_plan(&self.mission_id)?;
        let mut sessions = HashSet::new();
        let mut attempts = HashSet::new();
        for worker in &self.workers {
            crate::scope::validate_session_identity(&worker.session_name)?;
            if !sessions.insert(worker.session_name.as_str()) {
                bail!(
                    "oracle {} has duplicate worker session {}",
                    self.oracle_name,
                    worker.session_name
                );
            }
            let attempt_id = worker.attempt_id.as_deref().ok_or_else(|| {
                anyhow::anyhow!(
                    "worker {} has no authoritative attempt id",
                    worker.session_name
                )
            })?;
            let plan_revision = worker.plan_revision.ok_or_else(|| {
                anyhow::anyhow!(
                    "worker {} has no authoritative plan revision",
                    worker.session_name
                )
            })?;
            if worker.task_id.trim().is_empty() || !attempts.insert(attempt_id) {
                bail!("oracle worker identity is empty or duplicated");
            }
            let attempt = ledger.task_attempt(attempt_id)?.ok_or_else(|| {
                anyhow::anyhow!("worker attempt {attempt_id} is absent from the ledger")
            })?;
            if attempt.mission_id != self.mission_id
                || attempt.task_id != worker.task_id
                || attempt.plan_revision != plan_revision
            {
                bail!(
                    "worker {} identity differs from authoritative attempt {}",
                    worker.session_name,
                    attempt_id
                );
            }
            let plan = active_plan.as_ref().ok_or_else(|| {
                anyhow::anyhow!(
                    "mission {} has workers but no active plan",
                    self.mission_id.as_str()
                )
            })?;
            let task = plan
                .tasks
                .iter()
                .find(|task| task.task_id.as_str() == worker.task_id);
            if projection.active_plan_revision != Some(plan_revision)
                || plan.revision != plan_revision
                || task.is_none()
            {
                bail!(
                    "worker {} is not an exact task in active plan revision {}",
                    worker.session_name,
                    plan_revision
                );
            }
            let task = task.expect("task presence checked above");
            if worker.task_name != task.name {
                bail!(
                    "worker {} task name differs from active plan",
                    worker.session_name
                );
            }
            let mut projected_scope: Vec<String> = worker
                .files_owned
                .iter()
                .map(|scope| crate::scope::normalize_scope_selector(scope))
                .collect();
            let mut contracted_scope: Vec<String> = task
                .scope
                .iter()
                .map(|scope| crate::scope::normalize_scope_selector(scope))
                .collect();
            projected_scope.sort();
            projected_scope.dedup();
            contracted_scope.sort();
            contracted_scope.dedup();
            if projected_scope != contracted_scope {
                bail!(
                    "worker {} scope differs from active task contract",
                    worker.session_name
                );
            }
            if !worker_status_compatible(worker.status, attempt.state) {
                bail!(
                    "worker {} status {:?} is incompatible with ledger attempt {:?}",
                    worker.session_name,
                    worker.status,
                    attempt.state
                );
            }
        }
        Ok(())
    }

    fn read_locked(state_dir: &Path, oracle_name: &str) -> Result<Option<Self>> {
        let (path, legacy, key) = Self::paths(state_dir, oracle_name)?;
        if let Some(state) = crate::scope::read_private_json::<Self>(&path)? {
            state.validate_document_identity(&key)?;
            return Ok(Some(state));
        }

        let Some(mut state) = crate::scope::read_private_json::<Self>(&legacy)? else {
            return Ok(None);
        };
        state.validate_document_identity(&key)?;
        state.storage_revision = state.storage_revision.max(1);
        crate::config::atomic_write_private(&path, &serde_json::to_vec_pretty(&state)?)?;
        crate::scope::remove_private_file(&legacy)?;
        Ok(Some(state))
    }

    fn write_locked(&self, state_dir: &Path) -> Result<Self> {
        let (path, legacy, key) = Self::paths(state_dir, &self.oracle_name)?;
        self.validate_document_identity(&key)?;

        // Refuse an unsafe or ambiguous legacy twin before publishing.
        let legacy_state = crate::scope::read_private_json::<Self>(&legacy)?;
        if let Some(legacy_state) = &legacy_state {
            legacy_state.validate_document_identity(&key)?;
        }
        let current = crate::scope::read_private_json::<Self>(&path)?;
        if let Some(current) = &current {
            current.validate_document_identity(&key)?;
            if current.mission_id == self.mission_id
                && current.storage_revision != self.storage_revision
            {
                bail!(
                    "stale OracleState write for {}: expected storage revision {}, found {}",
                    self.oracle_name,
                    self.storage_revision,
                    current.storage_revision
                );
            }
            if current.mission_id != self.mission_id && !current.is_closeable() {
                bail!(
                    "oracle {} still owns mission {}; refusing replacement with {}",
                    self.oracle_name,
                    current.mission_id.as_str(),
                    self.mission_id.as_str()
                );
            }
        } else if self.storage_revision != 0 {
            bail!(
                "OracleState {} disappeared after revision {}",
                self.oracle_name,
                self.storage_revision
            );
        }

        let mut published = self.clone();
        published.storage_revision = current
            .as_ref()
            .map_or(1, |state| state.storage_revision.saturating_add(1));
        crate::config::atomic_write_private(&path, &serde_json::to_vec_pretty(&published)?)?;
        if legacy_state.is_some() {
            crate::scope::remove_private_file(&legacy)?;
        }
        let round_trip = crate::scope::read_private_json::<Self>(&path)?
            .ok_or_else(|| anyhow::anyhow!("OracleState vanished after publish"))?;
        if round_trip.storage_revision != published.storage_revision
            || round_trip.oracle_name != published.oracle_name
            || round_trip.mission_id != published.mission_id
        {
            bail!("OracleState changed while being published");
        }
        Ok(published)
    }

    /// Persist one compatibility projection with serialized, owner-only,
    /// no-follow and CAS-protected publication.
    pub fn write(&self, state_dir: &Path) -> Result<()> {
        let (_, _, key) = Self::paths(state_dir, &self.oracle_name)?;
        let _lock =
            crate::scope::lock_private_state_file(state_dir, &format!(".oracle-{key}.state.lock"))?;
        if self.compatibility_projection.is_some()
            || self
                .workers
                .iter()
                .any(|worker| worker.attempt_id.is_some() || worker.plan_revision.is_some())
        {
            let ledger = MissionLedger::open(mission_ledger_path(state_dir))?;
            self.validate_authoritative_workers(&ledger)?;
        }
        self.write_locked(state_dir).map(|_| ())
    }

    /// Register one spawned worker through a single locked read-modify-write.
    /// `fallback` is used only when no state exists and must be a legacy
    /// minimal projection; a V3 worker can never be attached to invented
    /// authority after its governing projection disappears.
    pub fn register_worker_locked(
        state_dir: &Path,
        oracle_name: &str,
        fallback: Option<Self>,
        entry: WorkerEntry,
    ) -> Result<()> {
        let (_, _, key) = Self::paths(state_dir, oracle_name)?;
        let _lock =
            crate::scope::lock_private_state_file(state_dir, &format!(".oracle-{key}.state.lock"))?;
        let mut state = match Self::read_locked(state_dir, oracle_name)? {
            Some(state) => state,
            None => {
                if entry.attempt_id.is_some() || entry.plan_revision.is_some() {
                    bail!(
                        "oracle {oracle_name} authority disappeared before V3 worker registration"
                    );
                }
                let fallback = fallback.ok_or_else(|| {
                    anyhow::anyhow!("oracle {oracle_name} has no state projection")
                })?;
                if fallback.oracle_name != oracle_name
                    || fallback.compatibility_projection.is_some()
                    || !fallback.workers.is_empty()
                {
                    bail!("invalid fallback projection for oracle {oracle_name}");
                }
                fallback
            }
        };

        if state.compatibility_projection.is_some()
            || entry.attempt_id.is_some()
            || entry.plan_revision.is_some()
        {
            let ledger = MissionLedger::open(mission_ledger_path(state_dir))?;
            state.validate_authoritative_workers(&ledger)?;
            state.register_worker(entry);
            state.validate_authoritative_workers(&ledger)?;
        } else {
            state.register_worker(entry);
        }
        state.write_locked(state_dir).map(|_| ())
    }

    /// Locked read-modify-write for authority callers. The source projection,
    /// mission identity and every worker tuple are revalidated under the same
    /// file lock immediately before publication.
    pub fn mutate_authoritative<T, F>(state_dir: &Path, oracle_name: &str, mutate: F) -> Result<T>
    where
        F: FnOnce(&mut Self) -> Result<T>,
    {
        let (_, _, key) = Self::paths(state_dir, oracle_name)?;
        let _lock =
            crate::scope::lock_private_state_file(state_dir, &format!(".oracle-{key}.state.lock"))?;
        let mut state = Self::read_locked(state_dir, oracle_name)?
            .ok_or_else(|| anyhow::anyhow!("oracle {oracle_name} has no state projection"))?;
        let ledger = MissionLedger::open(mission_ledger_path(state_dir))?;
        state.validate_authoritative_workers(&ledger)?;
        let identity = (state.oracle_name.clone(), state.mission_id.clone());
        let output = mutate(&mut state)?;
        if (state.oracle_name.clone(), state.mission_id.clone()) != identity {
            bail!("OracleState mutation may not change oracle or mission identity");
        }
        state.validate_authoritative_workers(&ledger)?;
        state.write_locked(state_dir)?;
        Ok(output)
    }

    /// Strict single-state read. Legacy double-prefixed files are migrated
    /// atomically while holding the same per-oracle lock.
    pub fn read(state_dir: &Path, oracle_name: &str) -> Result<Option<Self>> {
        let (_, _, key) = Self::paths(state_dir, oracle_name)?;
        let _lock =
            crate::scope::lock_private_state_file(state_dir, &format!(".oracle-{key}.state.lock"))?;
        Self::read_locked(state_dir, oracle_name)
    }

    /// Strict authority sweep. One malformed, unsafe or duplicate projection
    /// blocks the whole read rather than becoming invisible.
    pub fn read_all_strict(state_dir: &Path) -> Result<Vec<Self>> {
        crate::scope::ensure_private_state_dir(state_dir)?;
        let entries = std::fs::read_dir(state_dir)?;
        let mut by_oracle: HashMap<String, Self> = HashMap::new();
        for entry in entries {
            let entry = entry?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| anyhow::anyhow!("oracle state directory has a non-UTF-8 filename"))?;
            if !name.starts_with("oracle-") || !name.ends_with(".state.json") {
                continue;
            }
            let state: Self = crate::scope::read_private_json(&entry.path())?.ok_or_else(|| {
                anyhow::anyhow!(
                    "oracle state {} vanished during strict read",
                    entry.path().display()
                )
            })?;
            let key = Self::state_key(&state.oracle_name)?.to_string();
            let canonical = format!("oracle-{key}.state.json");
            let legacy = format!("oracle-oracle-{key}.state.json");
            if name != canonical && name != legacy {
                bail!(
                    "oracle state filename {} does not match document {}",
                    name,
                    state.oracle_name
                );
            }
            if state.compatibility_projection.is_some() {
                let ledger = MissionLedger::open(mission_ledger_path(state_dir))?;
                state.require_ledger_authority(&ledger)?;
            }
            if by_oracle.insert(state.oracle_name.clone(), state).is_some() {
                bail!("duplicate canonical/legacy OracleState for {key}");
            }
        }
        let mut states: Vec<Self> = by_oracle.into_values().collect();
        states.sort_by(|left, right| left.oracle_name.cmp(&right.oracle_name));
        Ok(states)
    }

    /// Explicitly tolerant diagnostics for TUI/status/liveness views.
    ///
    /// The TUI refresh loop calls this on every tick, so the skipped-entry
    /// signal is emitted ONCE per process. Warning per tick turned a single
    /// unsafe entry into an unbounded log stream that, before logging was moved
    /// off the terminal for TUI runs, rendered on top of the interface.
    pub fn read_all(state_dir: &Path) -> Vec<Self> {
        static SWEEP_REPORTED: std::sync::Once = std::sync::Once::new();
        match Self::read_all_strict(state_dir) {
            Ok(states) => states,
            Err(error) => {
                SWEEP_REPORTED.call_once(|| {
                    tracing::warn!(error = %error, "OracleState diagnostic sweep omitted unsafe entries");
                });
                read_oracle_states_tolerant(state_dir)
            }
        }
    }

    pub fn remove(state_dir: &Path, oracle_name: &str) -> Result<()> {
        let (path, legacy, key) = Self::paths(state_dir, oracle_name)?;
        let _lock =
            crate::scope::lock_private_state_file(state_dir, &format!(".oracle-{key}.state.lock"))?;
        crate::scope::remove_private_file(&path)?;
        crate::scope::remove_private_file(&legacy)
    }
}

fn read_oracle_states_tolerant(state_dir: &Path) -> Vec<OracleState> {
    let mut by_oracle = HashMap::new();
    let Ok(entries) = std::fs::read_dir(state_dir) else {
        return Vec::new();
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !name.starts_with("oracle-") || !name.ends_with(".state.json") {
            continue;
        }
        let Ok(Some(state)) = crate::scope::read_private_json::<OracleState>(&path) else {
            continue;
        };
        let Ok(key) = OracleState::state_key(&state.oracle_name) else {
            continue;
        };
        let canonical = format!("oracle-{key}.state.json");
        let legacy = format!("oracle-oracle-{key}.state.json");
        let ledger_valid = if state.compatibility_projection.is_some() {
            match MissionLedger::open(mission_ledger_path(state_dir)) {
                Ok(ledger) => state.require_ledger_authority(&ledger).is_ok(),
                Err(_) => false,
            }
        } else {
            true
        };
        if ledger_valid && (name == canonical || name == legacy) {
            by_oracle.entry(state.oracle_name.clone()).or_insert(state);
        }
    }
    let mut states: Vec<_> = by_oracle.into_values().collect();
    states.sort_by(|left, right| left.oracle_name.cmp(&right.oracle_name));
    states
}

fn same_canonical_workspace(left: &Path, right: &Path) -> bool {
    if left == right {
        return true;
    }
    match (std::fs::canonicalize(left), std::fs::canonicalize(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}

/// A compatibility phase may lag the ledger, but it may never claim a later
/// lifecycle stage than the authoritative mission has reached.
fn oracle_phase_compatible(phase: OraclePhase, mission: MissionState) -> bool {
    let phase_stage = match phase {
        OraclePhase::Analyze => 0,
        OraclePhase::Dispatch => 1,
        OraclePhase::Monitor => 2,
        OraclePhase::Verify => 3,
        OraclePhase::Report => 4,
        OraclePhase::Done => 5,
    };
    let mission_stage = match mission {
        MissionState::Created | MissionState::Classified => 0,
        MissionState::Planned => 1,
        MissionState::Running | MissionState::CorrectionRequired => 2,
        MissionState::Verifying => 3,
        MissionState::Accepted
        | MissionState::Blocked
        | MissionState::Failed
        | MissionState::Reporting
        | MissionState::Cancelled => 4,
        MissionState::Delivered => 5,
    };
    phase_stage <= mission_stage
}

/// Non-terminal compatibility statuses may lag safely. Any status that makes
/// `all_workers_terminal` true must be backed by the corresponding ledger
/// terminal/blocked state; in particular only Accepted may project DoneClean.
fn worker_status_compatible(status: WorkerEntryStatus, attempt: TaskAttemptState) -> bool {
    match status {
        WorkerEntryStatus::Running | WorkerEntryStatus::Pending => true,
        WorkerEntryStatus::DoneClean => attempt == TaskAttemptState::Accepted,
        WorkerEntryStatus::Failed => {
            matches!(
                attempt,
                TaskAttemptState::Failed | TaskAttemptState::Cancelled
            )
        }
        WorkerEntryStatus::Blocked | WorkerEntryStatus::Stalled => matches!(
            attempt,
            TaskAttemptState::Blocked | TaskAttemptState::Failed | TaskAttemptState::Cancelled
        ),
    }
}

/// True when a worker entry will not progress on its own (the per-entry twin
/// of `OracleState::all_workers_terminal`).
pub fn worker_entry_terminal(status: WorkerEntryStatus) -> bool {
    matches!(
        status,
        WorkerEntryStatus::DoneClean
            | WorkerEntryStatus::Failed
            | WorkerEntryStatus::Blocked
            | WorkerEntryStatus::Stalled
    )
}

/// An oracle's LIVE worker sessions, split by whether they are finished.
#[derive(Debug, Default)]
pub struct LiveWorkers {
    /// Live session but the worker is finished (terminal registry status OR a
    /// written done-signal) — safe to cascade-close together with the oracle.
    pub terminal: Vec<String>,
    /// Live and still working — these BLOCK an oracle's done_clean (an oracle
    /// may not close itself while its workers run).
    pub running: Vec<String>,
}

impl LiveWorkers {
    pub fn all(&self) -> Vec<String> {
        self.terminal
            .iter()
            .chain(self.running.iter())
            .cloned()
            .collect()
    }
}

/// Resolve a tolerant diagnostic view of live workers governed by
/// `oracle_name`. Mutation/close paths must use
/// [`live_workers_of_oracle_strict`].
///
/// Authoritative source: the oracle's own `OracleState.workers` registry.
/// Fallback (state file GC'd or never written — the exact shape of the
/// dentistrygpt orphan incident): live `<project>-worker-*` sessions of the
/// oracle's project that no OTHER oracle's state claims. Without the
/// fallback, a lost state file silently exempts every worker from the
/// close-gate and the cascade, recreating the zombie leak this exists to fix.
pub fn live_workers_of_oracle(
    state_dir: &Path,
    oracle_name: &str,
    live_sessions: &[crate::session::OmegaSession],
) -> LiveWorkers {
    use crate::session::{OmegaSession, SessionRole};

    let live: std::collections::HashSet<&str> =
        live_sessions.iter().map(|s| s.name.as_str()).collect();
    let all_states = OracleState::read_all(state_dir);
    let own_state = all_states.iter().find(|s| s.oracle_name == oracle_name);

    // A worker is "finished" when its registry status is terminal or it wrote
    // ANY done-signal (the signal marks the end of its run, whatever the
    // verdict; spawn-time stale-clear guarantees the file postdates dispatch).
    let finished = |name: &str, entry_status: Option<WorkerEntryStatus>| -> bool {
        if entry_status.is_some_and(worker_entry_terminal) {
            return true;
        }
        matches!(crate::done::DoneSignal::read(state_dir, name), Ok(Some(_)))
    };

    let mut out = LiveWorkers::default();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    if let Some(state) = own_state {
        for w in &state.workers {
            if !live.contains(w.session_name.as_str()) {
                continue;
            }
            seen.insert(w.session_name.clone());
            if finished(&w.session_name, Some(w.status)) {
                out.terminal.push(w.session_name.clone());
            } else {
                out.running.push(w.session_name.clone());
            }
        }
    }

    // Project-name fallback for unregistered live workers.
    if let Some(project) = OmegaSession::classify(oracle_name).project {
        let owned_elsewhere: std::collections::HashSet<&str> = all_states
            .iter()
            .filter(|s| s.oracle_name != oracle_name)
            .flat_map(|s| s.workers.iter().map(|w| w.session_name.as_str()))
            .collect();
        for s in live_sessions {
            if s.role != SessionRole::Worker
                || s.project.as_deref() != Some(project.as_str())
                || seen.contains(&s.name)
                || owned_elsewhere.contains(s.name.as_str())
            {
                continue;
            }
            if finished(&s.name, None) {
                out.terminal.push(s.name.clone());
            } else {
                out.running.push(s.name.clone());
            }
        }
    }
    out
}

fn read_worker_done_strict(
    state_dir: &Path,
    session: &str,
) -> Result<Option<crate::done::DoneSignal>> {
    crate::done::DoneSignal::read(state_dir, session)
}

fn validate_worker_done_provenance(
    ledger: &MissionLedger,
    state: &OracleState,
    worker: &WorkerEntry,
    signal: &crate::done::DoneSignal,
) -> Result<()> {
    let Some(provenance) = &signal.projection else {
        // Legacy signals remain visible, but never decide strict terminality.
        return Ok(());
    };
    if provenance.source != "mission-engine-v3.sqlite3" {
        bail!(
            "worker {} done signal has an unknown projection source",
            worker.session_name
        );
    }
    let event = ledger
        .events(&state.mission_id)?
        .into_iter()
        .find(|event| event.event_id == provenance.event_id)
        .ok_or_else(|| anyhow::anyhow!("worker done projection event is absent"))?;
    if event.sequence != provenance.event_sequence
        || event.attempt_id.as_deref() != worker.attempt_id.as_deref()
        || event.task_id.as_deref() != Some(worker.task_id.as_str())
    {
        bail!(
            "worker {} done projection is bound to another task attempt",
            worker.session_name
        );
    }
    let projection = ledger
        .projection_at(&state.mission_id, event.sequence)?
        .ok_or_else(|| anyhow::anyhow!("worker done projection cannot be replayed"))?;
    if projection.version != provenance.mission_version
        || projection.projection_hash != provenance.projection_hash
    {
        bail!(
            "worker {} done projection hash/version differs from ledger",
            worker.session_name
        );
    }
    Ok(())
}

/// Authority-safe worker resolution for close/kill/release decisions.
///
/// Every V3 OracleState and done signal is read through a private no-follow
/// descriptor and reconciled to the immutable mission, active plan and exact
/// task attempt. A done.json alone never makes a worker terminal; only ledger
/// Accepted/Blocked/Failed/Cancelled state can authorize cascading it.
pub fn live_workers_of_oracle_strict(
    state_dir: &Path,
    oracle_name: &str,
    live_sessions: &[crate::session::OmegaSession],
) -> Result<LiveWorkers> {
    use crate::session::SessionRole;

    let all_states = OracleState::read_all_strict(state_dir)?;
    let own_state = all_states
        .iter()
        .find(|state| state.oracle_name == oracle_name)
        .ok_or_else(|| anyhow::anyhow!("oracle {oracle_name} has no strict state projection"))?;
    if own_state.compatibility_projection.is_none() {
        bail!("oracle {oracle_name} is a legacy projection without close authority");
    }
    let ledger = MissionLedger::open(mission_ledger_path(state_dir))?;
    own_state.require_ledger_authority(&ledger)?;

    let live: HashSet<&str> = live_sessions
        .iter()
        .map(|session| session.name.as_str())
        .collect();
    let mut seen = HashSet::new();
    let mut out = LiveWorkers::default();
    for worker in &own_state.workers {
        if !live.contains(worker.session_name.as_str()) {
            continue;
        }
        seen.insert(worker.session_name.clone());
        if let Some(signal) = read_worker_done_strict(state_dir, &worker.session_name)? {
            validate_worker_done_provenance(&ledger, own_state, worker, &signal)?;
        }
        let attempt_id = worker
            .attempt_id
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("worker {} has no attempt id", worker.session_name))?;
        let attempt = ledger
            .task_attempt(attempt_id)?
            .ok_or_else(|| anyhow::anyhow!("worker attempt {attempt_id} is absent"))?;
        if matches!(
            attempt.state,
            TaskAttemptState::Accepted
                | TaskAttemptState::Blocked
                | TaskAttemptState::Failed
                | TaskAttemptState::Cancelled
        ) {
            out.terminal.push(worker.session_name.clone());
        } else {
            out.running.push(worker.session_name.clone());
        }
    }

    let mut owned_elsewhere: HashSet<&str> = HashSet::new();
    for state in all_states
        .iter()
        .filter(|state| state.oracle_name != oracle_name)
    {
        // A legacy projection has no ledger ancestry and therefore no right to
        // hide a live project worker from the target oracle's close accounting.
        // Strict V3 projections were already checked by read_all_strict; repeat
        // the explicit authority predicate here so this safety property remains
        // local if the sweep's diagnostic behavior ever changes.
        if state.compatibility_projection.is_none() {
            continue;
        }
        state.require_ledger_authority(&ledger)?;
        owned_elsewhere.extend(
            state
                .workers
                .iter()
                .map(|worker| worker.session_name.as_str()),
        );
    }
    for session in live_sessions {
        if session.role != SessionRole::Worker
            || session.project.as_deref() != Some(own_state.project.as_str())
            || seen.contains(&session.name)
            || owned_elsewhere.contains(session.name.as_str())
        {
            continue;
        }
        // Parse any signal strictly so corruption blocks the close, but an
        // unregistered worker can never be terminal authority.
        let _ = read_worker_done_strict(state_dir, &session.name)?;
        out.running.push(session.name.clone());
    }
    out.terminal.sort();
    out.terminal.dedup();
    out.running.sort();
    out.running.dedup();
    Ok(out)
}

// ---------------------------------------------------------------------------
// God Mode State Machine: WORK → VERIFYING → DONE (loop on failure)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GodModeState {
    pub phase: GodModePhase,
    pub iteration: u32,
    pub max_iterations: u32,
    pub plan_phases_total: u32,
    pub plan_phases_completed: u32,
    pub previous_issues: Vec<String>,
    pub entered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GodModePhase {
    /// Planning: /planner generates task DAG
    Planning,
    /// Executing plan phases sequentially, workers per phase
    Working,
    /// Post-work: running /debugaudit verification
    Verifying,
    /// All phases done + verification passed
    Done,
}

impl GodModeState {
    pub fn new(max_iterations: u32) -> Self {
        Self {
            phase: GodModePhase::Planning,
            iteration: 1,
            max_iterations,
            plan_phases_total: 0,
            plan_phases_completed: 0,
            previous_issues: Vec::new(),
            entered_at: Utc::now(),
        }
    }

    pub fn transition(&mut self, to: GodModePhase) {
        self.phase = to;
        self.entered_at = Utc::now();
    }

    pub fn start_new_iteration(&mut self) -> bool {
        if self.iteration >= self.max_iterations {
            return false;
        }
        self.iteration += 1;
        self.phase = GodModePhase::Working;
        self.entered_at = Utc::now();
        true
    }

    pub fn record_issue(&mut self, issue: String) {
        self.previous_issues.push(issue);
    }
}

// ---------------------------------------------------------------------------
// Oracle Prompt Generator — per-project system prompt with context
// ---------------------------------------------------------------------------

pub struct OraclePromptGenerator;

impl OraclePromptGenerator {
    /// Generate the system prompt for an oracle session.
    ///
    /// Single source of truth: renders the shared `agents/oracle.md` v2
    /// template (installed `~/.omega/agents/oracle.md`, falling back to the
    /// repo copy) with the {{PROJECT}}/{{WORKDIR}}/{{SESSION}} placeholders,
    /// prepends the amplified mission as the Layer-3 body, and appends the
    /// per-mission dynamic blocks (ship / god mode). When the template file is
    /// missing (pre-install), falls back to a self-contained inline prompt so
    /// the oracle is never left without an identity.
    pub fn generate(
        project: &str,
        working_dir: &Path,
        oracle_name: &str,
        mission: &str,
        ship: bool,
        god_mode: bool,
    ) -> String {
        let mut prompt = String::with_capacity(4096);

        // Layer 3 — the mission body comes first so it's unmissable.
        prompt.push_str(&format!("## Mission\n{}\n\n---\n\n", mission));

        // Layer 3b — HOW to run this particular mission. Sits right under the
        // mission and above the standing identity, because the shape of the
        // work is mission-specific while the identity is not. Without it, an
        // oracle given "audit this" and one given "build this" received the
        // same generic advice and both defaulted to doing the work themselves
        // instead of spawning and supervising workers.
        prompt.push_str(&crate::mission_patterns::orchestration_block(mission));
        prompt.push_str(&crate::lab::oracle_lab_block_for_mission(mission));
        prompt.push_str("\n---\n\n");

        // Layer 2 — the shared v2 identity/protocol template.
        match Self::load_template(project, working_dir, oracle_name) {
            Some(tpl) => prompt.push_str(&tpl),
            None => Self::push_inline_fallback(&mut prompt, project, working_dir, oracle_name),
        }

        // Per-mission dynamic blocks.
        if ship {
            prompt.push_str(
                "## Ship Pipeline\n\
                 This mission requires shipping. After verification:\n\
                 build → gitleaks → commit → push → deploy → verify deploy.\n\
                 Use `omega ship` when ready.\n\n",
            );
        }

        // God Mode
        if god_mode {
            prompt.push_str(
                "## GOD MODE Active\n\
                 Multi-phase autonomous execution:\n\
                 1. PLAN: Generate task DAG with /planner\n\
                 2. EXECUTE: Phase by phase — dispatch workers, monitor, verify per phase\n\
                 3. VERIFY: Run /debugaudit after each phase, fix-loop until clean\n\
                 4. COMPLETE: All phases done + final verification → write signal file\n\
                 Pre-check: if 'Not Done' items exist from prior iteration → CONTINUE (don't ask)\n\n",
            );
        }

        prompt
    }

    /// Load + render the shared `agents/oracle.md` v2 template. Prefers the
    /// installed copy, falls back to the repo copy. Returns None if neither
    /// exists (the caller then uses the inline fallback).
    fn load_template(project: &str, working_dir: &Path, oracle_name: &str) -> Option<String> {
        let home = dirs::home_dir().unwrap_or_else(|| {
            std::env::var("HOME")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
        });
        let candidates = [
            home.join(".omega/agents/oracle.md"),
            std::path::PathBuf::from("agents/oracle.md"),
        ];
        let template = candidates
            .iter()
            .find_map(|p| std::fs::read_to_string(p).ok())?;
        Some(
            template
                .replace("{{PROJECT}}", project)
                .replace("{{WORKDIR}}", &working_dir.display().to_string())
                .replace("{{SESSION}}", oracle_name),
        )
    }

    /// Self-contained inline identity/protocol, used only when the shared v2
    /// template file is missing (pre-install). Kept terse but complete.
    fn push_inline_fallback(
        prompt: &mut String,
        project: &str,
        working_dir: &Path,
        oracle_name: &str,
    ) {
        prompt.push_str(&format!(
            "## Project: {} ({})\n## Role: ORACLE\n## Session: {}\n\n\
             You are the Oracle — a PROJECT MANAGER, never edit code directly. Analyze,\n\
             decompose, dispatch workers, monitor, verify, report.\n\n\
             ## Three Laws\n\
             1. Code lies. Only runtime tells the truth.\n\
             2. Be a researcher, not a sycophant. Challenge flawed premises.\n\
             3. Decide and proceed, never wait.\n\n\
             ## Workflow\n\
             ANALYZE → DISPATCH (`omega spawn-worker <task> \"<prompt>\" --dir <dir> --files a,b`) → MONITOR\n\
             (`omega status`, inbox) → VERIFY (build + tests + audit + ground-truth) → REPORT\n\
             (`omega done {} done_clean \"<summary>\"`).\n\n\
             ## Quality Gate (before done)\n\
             - All workers done_clean AND survived the ground-truth gate\n\
             - Build = 0 errors, no runtime errors\n\
             - `omega gate {}` criteria satisfied\n",
            project,
            working_dir.display(),
            oracle_name,
            oracle_name,
            oracle_name,
        ));
    }

    /// Detect if a mission text implies shipping.
    pub fn should_ship(text: &str) -> bool {
        let lower = text.to_lowercase();
        let ship_keywords = [
            "ship",
            "deploy",
            "push",
            "merge",
            "en prod",
            "envoie en prod",
            "livre",
        ];
        ship_keywords.iter().any(|kw| lower.contains(kw))
    }

    /// Detect if a mission text requests god mode.
    pub fn is_god_mode(text: &str) -> bool {
        let lower = text.to_lowercase();
        lower.contains("god mode") || lower.contains("godmode") || lower.contains("/godmode")
    }
}

// ---------------------------------------------------------------------------
// Signal File — oracle writes result, watcher detects completion
// ---------------------------------------------------------------------------

fn validate_project_identity(project: &str) -> Result<()> {
    crate::scope::validate_session_identity(project)
        .map_err(|error| anyhow::anyhow!("invalid project identity `{project}`: {error}"))
}

fn validate_bounded_identity(value: &str, field: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 512
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        bail!("invalid {field}");
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleSignalFile {
    pub oracle: String,
    pub project: String,
    pub status: SignalStatus,
    pub build: SignalBuild,
    pub summary: String,
    pub not_done: Vec<String>,
    pub next_steps: Vec<String>,
    pub worker_results: Vec<WorkerResult>,
    pub duration_secs: u64,
    pub ship: Option<ShipResult>,
    pub created_at: DateTime<Utc>,
    /// Monotonic generation of the compatibility document. Legacy signals
    /// deserialize as generation zero and are upgraded on their next write.
    #[serde(default, rename = "_omega_revision")]
    storage_revision: u64,
    /// Exact on-disk generation/digest observed by [`OracleSignalFile::read`].
    /// These never serialize; they are the CAS receipt for a later write.
    #[serde(skip)]
    source_revision: Option<u64>,
    #[serde(skip)]
    source_digest: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SignalStatus {
    Done,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SignalBuild {
    Pass,
    Fail,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipResult {
    pub result: ShipOutcome,
    pub commit: Option<String>,
    pub deploy_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShipOutcome {
    Ok,
    Failed,
    Skipped,
    Frozen,
}

impl OracleSignalFile {
    pub fn from_oracle_state(state: &OracleState, build: SignalBuild) -> Self {
        let worker_results: Vec<WorkerResult> = state
            .workers
            .iter()
            .map(|w| WorkerResult {
                task_id: w.task_id.clone(),
                session_name: w.session_name.clone(),
                status: match w.status {
                    WorkerEntryStatus::DoneClean => DoneStatus::DoneClean,
                    WorkerEntryStatus::Pending => DoneStatus::Pending,
                    WorkerEntryStatus::Failed => DoneStatus::Failed,
                    WorkerEntryStatus::Blocked => DoneStatus::Blocked,
                    _ => DoneStatus::Pending,
                },
                summary: String::new(),
                commit: None,
                duration_secs: 0,
            })
            .collect();

        let status = if state.any_worker_failed() || build == SignalBuild::Fail {
            SignalStatus::Failed
        } else {
            SignalStatus::Done
        };

        Self {
            oracle: state.oracle_name.clone(),
            project: state.project.clone(),
            status,
            build,
            summary: String::new(),
            not_done: Vec::new(),
            next_steps: Vec::new(),
            worker_results,
            duration_secs: state.duration_secs(),
            ship: None,
            created_at: Utc::now(),
            storage_revision: 0,
            source_revision: None,
            source_digest: None,
        }
    }

    fn paths(state_dir: &Path, oracle: &str) -> Result<(PathBuf, PathBuf, String)> {
        let key = OracleState::state_key(oracle)?.to_string();
        Ok((
            state_dir.join(format!("oracle-{key}.result.json")),
            state_dir.join(format!("oracle-oracle-{key}.result.json")),
            key,
        ))
    }

    fn validate_for_key(&self, expected_key: &str) -> Result<()> {
        let actual_key = OracleState::state_key(&self.oracle)?;
        if actual_key != expected_key {
            bail!(
                "oracle signal filename/session mismatch: filename={}, document={}",
                expected_key,
                self.oracle
            );
        }
        validate_project_identity(&self.project)?;
        if self.status == SignalStatus::Done && self.build == SignalBuild::Fail {
            bail!("oracle signal cannot be DONE while its build is FAIL");
        }
        let mut sessions = HashSet::new();
        let mut tasks = HashSet::new();
        for worker in &self.worker_results {
            crate::scope::validate_session_identity(&worker.session_name)?;
            validate_bounded_identity(&worker.task_id, "worker task id")?;
            if !sessions.insert(worker.session_name.as_str()) {
                bail!(
                    "oracle signal contains duplicate worker session {}",
                    worker.session_name
                );
            }
            if !tasks.insert(worker.task_id.as_str()) {
                bail!(
                    "oracle signal contains duplicate worker task {}",
                    worker.task_id
                );
            }
        }
        Ok(())
    }

    fn authority_digest(&self) -> Result<String> {
        let mut canonical = self.clone();
        canonical.source_revision = None;
        canonical.source_digest = None;
        Ok(blake3::hash(&serde_json::to_vec(&canonical)?)
            .to_hex()
            .to_string())
    }

    /// Read through canonical and pre-normalization paths while the caller
    /// holds `.oracle-signal.lock`. A sole legacy double-prefix file is renamed
    /// atomically before it is returned; two files are ambiguous authority.
    fn read_optional_strict_locked(state_dir: &Path, oracle: &str) -> Result<Option<Self>> {
        crate::scope::ensure_private_state_dir(state_dir)?;
        let (canonical, legacy, key) = Self::paths(state_dir, oracle)?;
        let canonical_signal = crate::scope::read_private_json::<Self>(&canonical)?;
        let legacy_signal = crate::scope::read_private_json::<Self>(&legacy)?;
        let mut signal = match (canonical_signal, legacy_signal) {
            (Some(_), Some(_)) => bail!(
                "oracle signal {} has both canonical and legacy documents",
                key
            ),
            (Some(signal), None) => signal,
            (None, Some(signal)) => {
                signal.validate_for_key(&key)?;
                std::fs::rename(&legacy, &canonical)?;
                std::fs::File::open(state_dir)?.sync_all()?;
                crate::scope::read_private_json::<Self>(&canonical)?.ok_or_else(|| {
                    anyhow::anyhow!("oracle signal vanished during legacy migration")
                })?
            }
            (None, None) => return Ok(None),
        };
        signal.validate_for_key(&key)?;
        if crate::scope::read_private_json::<Self>(&canonical)?.is_none() {
            // The only way a signal can be returned is from the canonical path
            // (either originally or after the atomic legacy rename).
            bail!("oracle signal canonical path disappeared during strict read");
        }
        if crate::scope::read_private_json::<Self>(&legacy)?.is_some() {
            bail!("oracle signal legacy path reappeared during strict read");
        }
        if signal.storage_revision == u64::MAX {
            // Reading remains possible for diagnostics, but no future CAS write
            // can advance this generation. Surface it before mutation instead
            // of wrapping below.
            tracing::warn!(oracle = %signal.oracle, "oracle signal revision is exhausted");
        }
        signal.source_revision = Some(signal.storage_revision);
        signal.source_digest = Some(signal.authority_digest()?);
        Ok(Some(signal))
    }

    fn read_optional_strict(state_dir: &Path, oracle: &str) -> Result<Option<Self>> {
        let _lock = crate::scope::lock_private_state_file(state_dir, ".oracle-signal.lock")?;
        Self::read_optional_strict_locked(state_dir, oracle)
    }

    fn canonical_path(state_dir: &Path, oracle: &str) -> Result<PathBuf> {
        let (canonical, _, _) = Self::paths(state_dir, oracle)?;
        Ok(canonical)
    }

    fn current_under_lock(state_dir: &Path, oracle: &str) -> Result<Option<Self>> {
        let signal = Self::read_optional_strict_locked(state_dir, oracle)?;
        if signal.is_none() {
            return Ok(None);
        }
        Ok(signal)
    }

    pub fn write(&self, state_dir: &Path) -> Result<PathBuf> {
        let (_, _, key) = Self::paths(state_dir, &self.oracle)?;
        self.validate_for_key(&key)?;
        let _lock = crate::scope::lock_private_state_file(state_dir, ".oracle-signal.lock")?;
        let current = Self::current_under_lock(state_dir, &self.oracle)?;
        match (&self.source_digest, self.source_revision, current.as_ref()) {
            (Some(expected_digest), Some(expected_revision), Some(observed))
                if observed.storage_revision == expected_revision
                    && observed.source_digest.as_deref() == Some(expected_digest.as_str()) => {}
            (None, None, None) => {}
            (Some(_), Some(_), None) => {
                bail!("oracle signal disappeared before compare-and-swap")
            }
            (None, None, Some(_)) => {
                bail!(
                    "oracle signal {} already exists; read it before replacing it",
                    self.oracle
                )
            }
            _ => bail!(
                "stale oracle signal write refused for {}: generation or digest changed",
                self.oracle
            ),
        }
        let next_revision = current
            .as_ref()
            .map(|signal| signal.storage_revision)
            .unwrap_or(0)
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("oracle signal revision overflow"))?;
        let mut published = self.clone();
        published.storage_revision = next_revision;
        published.source_revision = None;
        published.source_digest = None;
        published.validate_for_key(&key)?;
        let content = serde_json::to_vec_pretty(&published)?;
        let path = Self::canonical_path(state_dir, &self.oracle)?;
        // If a destination already exists, the strict read above proved it is
        // a private single-link regular file before atomic replacement.
        crate::config::atomic_write_private(&path, &content)?;
        let observed = Self::current_under_lock(state_dir, &self.oracle)?
            .ok_or_else(|| anyhow::anyhow!("oracle signal vanished after publish"))?;
        if observed.storage_revision != next_revision
            || observed.source_digest.as_deref() != Some(published.authority_digest()?.as_str())
        {
            bail!("oracle signal changed while being published");
        }
        Ok(path)
    }

    /// Write a markdown signal file (human-readable, for backward compat with VPS watcher).
    pub fn write_markdown(&self, state_dir: &Path) -> Result<PathBuf> {
        let (_, _, key) = Self::paths(state_dir, &self.oracle)?;
        self.validate_for_key(&key)?;
        let _lock = crate::scope::lock_private_state_file(state_dir, ".oracle-signal.lock")?;
        let path = state_dir.join(format!("oracle-result-{}.md", self.project));

        let status_str = match self.status {
            SignalStatus::Done => "DONE",
            SignalStatus::Failed => "FAILED",
        };
        let build_str = match self.build {
            SignalBuild::Pass => "PASS",
            SignalBuild::Fail => "FAIL",
            SignalBuild::Skipped => "SKIPPED",
        };

        let mut md = format!(
            "# Oracle Report -- {}\nPROJECT:{}\nSTATUS:{}\nBUILD:{}\n",
            self.project, self.project, status_str, build_str
        );

        md.push_str("## Resume\n");
        if self.summary.is_empty() {
            md.push_str("Mission completed.\n");
        } else {
            md.push_str(&self.summary);
            md.push('\n');
        }

        if !self.not_done.is_empty() {
            md.push_str("## Not Done\n");
            for item in &self.not_done {
                md.push_str(&format!("- {}\n", item));
            }
        }

        if !self.next_steps.is_empty() {
            md.push_str("## Next Steps\n");
            for step in &self.next_steps {
                md.push_str(&format!("- {}\n", step));
            }
        }

        // Verify an existing compatibility file before replacing its directory
        // entry. A symlink/hardlink/foreign-owner file is authority corruption,
        // not an invitation to overwrite it.
        let _ = crate::config::read_private_optional(&path)?;
        crate::config::atomic_write_private(&path, md.as_bytes())?;
        let observed = crate::config::read_private_optional(&path)?
            .ok_or_else(|| anyhow::anyhow!("oracle markdown signal vanished after publish"))?;
        if observed != md.as_bytes() {
            bail!("oracle markdown signal changed while being published");
        }
        Ok(path)
    }

    /// Strict authority read. Corrupt, aliased, foreign or mismatched files
    /// return an error and must block close/reuse decisions.
    pub fn read(state_dir: &Path, oracle: &str) -> Result<Option<Self>> {
        Self::read_optional_strict(state_dir, oracle)
    }

    /// Explicitly tolerant projection for status-only surfaces. It never
    /// authorizes a mutation or lifecycle transition.
    pub fn read_diagnostic(state_dir: &Path, oracle: &str) -> Option<Self> {
        Self::read(state_dir, oracle).unwrap_or_else(|error| {
            tracing::warn!(oracle, error = %error, "diagnostic signal omitted unsafe document");
            None
        })
    }
}

// ---------------------------------------------------------------------------
// Signal File Watcher — poll for new oracle result files
// ---------------------------------------------------------------------------

pub struct SignalWatcher {
    state_dir: PathBuf,
    known: HashMap<String, (u64, DateTime<Utc>)>,
}

impl SignalWatcher {
    pub fn new(state_dir: PathBuf) -> Self {
        Self {
            state_dir,
            known: HashMap::new(),
        }
    }

    /// Scan for new or updated signal files since last check.
    /// Returns list of (oracle_name, signal_file) pairs that are new.
    pub fn poll(&mut self) -> Result<Vec<(String, OracleSignalFile)>> {
        crate::scope::ensure_private_state_dir(&self.state_dir)?;
        let mut new_signals = Vec::new();
        for entry in std::fs::read_dir(&self.state_dir)? {
            let entry = entry?;
            let Some(name) = entry.file_name().to_str().map(str::to_string) else {
                continue;
            };
            let Some(oracle) = name
                .strip_prefix("oracle-")
                .and_then(|name| name.strip_suffix(".result.json"))
            else {
                continue;
            };
            crate::scope::validate_session_identity(oracle)?;
            let signal = OracleSignalFile::read(&self.state_dir, oracle)?.ok_or_else(|| {
                anyhow::anyhow!("oracle signal {name} disappeared during strict watcher poll")
            })?;
            let oracle_identity = signal.oracle.clone();
            let observation = (signal.storage_revision, signal.created_at);
            let is_new = match self.known.get(&oracle_identity) {
                None => true,
                Some((previous_revision, previous_created))
                    if signal.storage_revision < *previous_revision =>
                {
                    bail!("oracle signal {oracle_identity} revision moved backwards")
                }
                Some((previous_revision, previous_created))
                    if signal.storage_revision == *previous_revision =>
                {
                    if signal.created_at != *previous_created {
                        bail!(
                            "oracle signal {oracle_identity} changed without advancing its revision"
                        );
                    }
                    false
                }
                Some(_) => true,
            };
            if is_new {
                self.known.insert(oracle_identity.clone(), observation);
                new_signals.push((oracle_identity, signal));
            }
        }
        new_signals.sort_by(|left, right| left.0.cmp(&right.0));
        Ok(new_signals)
    }
}

// ---------------------------------------------------------------------------
// Worker Stall Detector — captures pane output, detects idle prompt
// ---------------------------------------------------------------------------

/// Thresholds for stall detection, matching the VPS Shadow Manager signals.
pub struct StallThresholds {
    /// Seconds of idle at prompt before sending a nudge
    pub nudge_after_secs: u64,
    /// Seconds of idle at prompt before escalating to oracle
    pub escalate_after_secs: u64,
}

impl Default for StallThresholds {
    fn default() -> Self {
        Self {
            nudge_after_secs: 30,
            escalate_after_secs: 300,
        }
    }
}

pub struct WorkerStallDetector {
    thresholds: StallThresholds,
    last_active: HashMap<String, std::time::Instant>,
    idle_since: HashMap<String, std::time::Instant>,
}

#[derive(Debug, Clone)]
pub enum StallAction {
    Active,
    Nudge { session: String, idle_secs: u64 },
    Escalate { session: String, idle_secs: u64 },
}

impl WorkerStallDetector {
    pub fn new(thresholds: StallThresholds) -> Self {
        Self {
            thresholds,
            last_active: HashMap::new(),
            idle_since: HashMap::new(),
        }
    }

    /// Analyze pane content to determine if the worker is idle.
    /// Returns the appropriate stall action.
    pub fn check(&mut self, session: &str, pane_content: &str) -> StallAction {
        let is_idle = Self::detect_idle_prompt(pane_content);
        let now = std::time::Instant::now();

        if is_idle {
            let idle_start = self.idle_since.entry(session.to_string()).or_insert(now);
            let idle_secs = now.duration_since(*idle_start).as_secs();

            if idle_secs >= self.thresholds.escalate_after_secs {
                return StallAction::Escalate {
                    session: session.to_string(),
                    idle_secs,
                };
            } else if idle_secs >= self.thresholds.nudge_after_secs {
                return StallAction::Nudge {
                    session: session.to_string(),
                    idle_secs,
                };
            }
        } else {
            // Worker is active — reset idle tracking
            self.idle_since.remove(session);
            self.last_active.insert(session.to_string(), now);
        }

        StallAction::Active
    }

    /// Detect if pane shows an idle prompt (worker finished or stuck).
    /// Checks for common prompt patterns: `❯`, `$`, `>` at end of visible output.
    fn detect_idle_prompt(content: &str) -> bool {
        let trimmed = content.trim();
        // N11: an EMPTY pane capture is NOT proof of an idle prompt — it usually
        // means the capture raced a redraw or the session is mid-spawn. Treating
        // it as idle was a false-positive that mis-escalated live workers. Empty
        // => not idle.
        if trimmed.is_empty() {
            return false;
        }

        // Check the last non-empty line, trimming trailing whitespace
        let last_line = trimmed.lines().last().unwrap_or("");
        let last_trimmed = last_line.trim_end();

        // Claude Code / zsh prompt indicator. '❯' is a prompt glyph that
        // virtually never terminates a code/markup line, so a line ending in it
        // is an idle prompt (unchanged behaviour). The ambiguous case the N11
        // exclusions target is the ASCII '>' below.
        if last_trimmed.ends_with('❯') {
            return true;
        }

        // Standard shell prompt patterns (with or without trailing space)
        if last_trimmed.ends_with('$') || last_trimmed.ends_with('%') || last_trimmed.ends_with('#')
        {
            return true;
        }

        // Bare '>' / '❯' prompt — but ONLY when it's a standalone prompt, never
        // when the line ends in code/markup punctuation that merely closes with
        // '>': '=>', '->', '/>', '});', generic-type '<T>', JSX/HTML tags, etc.
        // Those are output, not an idle prompt.
        if last_trimmed.ends_with('>') {
            return is_standalone_prompt(last_trimmed, '>');
        }

        false
    }

    /// Remove tracking for a session (e.g., after it's killed).
    pub fn forget(&mut self, session: &str) {
        self.last_active.remove(session);
        self.idle_since.remove(session);
    }
}

/// True only when `line` is a *standalone* prompt ending in `glyph` (one of
/// `>` / `❯`), not a code/markup line that merely happens to end with it.
///
/// Excludes the common code/markup line-endings that close with `>`: `=>`,
/// `->`, `/>`, `<T>` / `<Foo>` generics, JSX/HTML tags, and a `});`-style
/// statement that still ends with `>` after a fat-arrow body. A genuine idle
/// prompt is a short line: either just the glyph, or a shell-style
/// `user@host …>` style prefix followed by the glyph.
fn is_standalone_prompt(line: &str, glyph: char) -> bool {
    let line = line.trim_end();
    if !line.ends_with(glyph) {
        return false;
    }
    // The character immediately before the prompt glyph decides intent.
    // Code/markup endings put punctuation right before the closing '>'.
    let before = line[..line.len() - glyph.len_utf8()].trim_end_matches(' ');
    if let Some(prev) = before.chars().last() {
        // '=>' '->' '/>' and tag/generic closers like 'foo>' are NOT prompts;
        // a real prompt has whitespace (or nothing) before the glyph.
        if matches!(prev, '=' | '-' | '/' | ')' | ';' | '"' | '\'' | '`') {
            return false;
        }
        // An alphanumeric immediately before '>' is a closing tag / generic
        // (e.g. `</div>`, `Vec<T>`), not a bare prompt.
        if prev.is_alphanumeric() {
            return false;
        }
    }
    // Markup/JSX tag lines also typically OPEN with '<' somewhere — a line that
    // both contains '<' and ends with '>' is overwhelmingly a tag, not a prompt.
    if glyph == '>' && before.contains('<') {
        return false;
    }
    true
}

// ---------------------------------------------------------------------------
// Oracle Registry — tracks all active oracles across projects
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OracleRegistry {
    pub oracles: Vec<OracleRegistryEntry>,
    #[serde(default, rename = "_omega_revision")]
    storage_revision: u64,
    #[serde(skip)]
    source_revision: Option<u64>,
    #[serde(skip)]
    source_digest: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleRegistryEntry {
    pub oracle_name: String,
    pub project: String,
    pub session_name: String,
    pub status: OracleRegistryStatus,
    pub spawned_at: DateTime<Utc>,
    pub files_owned: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OracleRegistryStatus {
    Active,
    Idle,
    Done,
    Dead,
}

impl OracleRegistry {
    fn path(state_dir: &Path) -> PathBuf {
        state_dir.join("oracle-registry.json")
    }

    fn authority_digest(&self) -> Result<String> {
        let mut canonical = self.clone();
        canonical.source_revision = None;
        canonical.source_digest = None;
        Ok(blake3::hash(&serde_json::to_vec(&canonical)?)
            .to_hex()
            .to_string())
    }

    fn validate_entry(entry: &OracleRegistryEntry) -> Result<()> {
        crate::scope::validate_session_identity(&entry.oracle_name)?;
        crate::scope::validate_session_identity(&entry.session_name)?;
        validate_project_identity(&entry.project)?;
        if entry.oracle_name != entry.session_name {
            bail!(
                "oracle registry entry binds {} to a different session {}",
                entry.oracle_name,
                entry.session_name
            );
        }
        let project_oracle = format!("oracle-{}", entry.project);
        if entry.oracle_name != project_oracle
            && !entry
                .oracle_name
                .strip_prefix(&(project_oracle + "-"))
                .is_some_and(|suffix| !suffix.is_empty())
        {
            bail!(
                "oracle {} is not namespaced to project {}",
                entry.oracle_name,
                entry.project
            );
        }
        let normalized = crate::scope::validate_scope_selectors(entry.files_owned.clone())?;
        if normalized != entry.files_owned {
            bail!(
                "oracle registry entry {} contains non-canonical file selectors",
                entry.oracle_name
            );
        }
        Ok(())
    }

    fn validate(&self) -> Result<()> {
        let mut oracle_names = HashSet::new();
        let mut session_names = HashSet::new();
        for entry in &self.oracles {
            Self::validate_entry(entry)?;
            if !oracle_names.insert(entry.oracle_name.as_str()) {
                bail!(
                    "oracle registry contains duplicate oracle {}",
                    entry.oracle_name
                );
            }
            if !session_names.insert(entry.session_name.as_str()) {
                bail!(
                    "oracle registry contains duplicate session {}",
                    entry.session_name
                );
            }
        }
        Ok(())
    }

    fn load_optional_strict(state_dir: &Path) -> Result<Option<Self>> {
        crate::scope::ensure_private_state_dir(state_dir)?;
        let Some(mut registry) = crate::scope::read_private_json::<Self>(&Self::path(state_dir))?
        else {
            return Ok(None);
        };
        registry.validate()?;
        registry.source_revision = Some(registry.storage_revision);
        registry.source_digest = Some(registry.authority_digest()?);
        Ok(Some(registry))
    }

    /// Strict authority loader. Missing state is an empty registry; malformed,
    /// aliased, foreign, duplicate or cross-project state is an error.
    pub fn load_strict(state_dir: &Path) -> Result<Self> {
        Ok(Self::load_optional_strict(state_dir)?.unwrap_or_default())
    }

    /// Explicitly tolerant status projection. Mutation and dispatch decisions
    /// must use [`OracleRegistry::load_strict`] or a locked mutation helper.
    pub fn load_diagnostic(state_dir: &Path) -> Self {
        Self::load_strict(state_dir).unwrap_or_else(|error| {
            tracing::warn!(error = %error, "diagnostic registry omitted unsafe document");
            Self::default()
        })
    }

    /// Backward-compatible status-only alias. New authority callsites must use
    /// [`OracleRegistry::load_strict`].
    pub fn load(state_dir: &Path) -> Self {
        Self::load_diagnostic(state_dir)
    }

    pub fn save(&self, state_dir: &Path) -> Result<()> {
        let _lock = crate::scope::lock_private_state_file(state_dir, ".oracle-registry.lock")?;
        self.save_locked(state_dir)
    }

    fn save_locked(&self, state_dir: &Path) -> Result<()> {
        self.validate()?;
        let current = Self::load_optional_strict(state_dir)?;
        match (&self.source_digest, self.source_revision, current.as_ref()) {
            (Some(expected_digest), Some(expected_revision), Some(observed))
                if observed.storage_revision == expected_revision
                    && observed.source_digest.as_deref() == Some(expected_digest.as_str()) => {}
            (None, None, None) => {}
            (Some(_), Some(_), None) => {
                bail!("oracle registry disappeared before compare-and-swap")
            }
            (None, None, Some(_)) => {
                bail!("oracle registry already exists; reload before replacing it")
            }
            _ => bail!("stale oracle registry write refused: generation or digest changed"),
        }
        let next_revision = current
            .as_ref()
            .map(|registry| registry.storage_revision)
            .unwrap_or(0)
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("oracle registry revision overflow"))?;
        let mut published = self.clone();
        published.storage_revision = next_revision;
        published.source_revision = None;
        published.source_digest = None;
        published.validate()?;
        crate::config::atomic_write_private(
            &Self::path(state_dir),
            &serde_json::to_vec_pretty(&published)?,
        )?;
        let observed = Self::load_optional_strict(state_dir)?
            .ok_or_else(|| anyhow::anyhow!("oracle registry vanished after publish"))?;
        let expected_digest = published.authority_digest()?;
        if observed.storage_revision != next_revision
            || observed.source_digest.as_deref() != Some(expected_digest.as_str())
        {
            bail!("oracle registry changed while being published");
        }
        Ok(())
    }

    /// Atomically reserve + register an Active oracle name for `project`,
    /// serialized across processes by an exclusive advisory lock (fs2, the same
    /// pattern as scope.rs / inbox.rs). Without it two concurrent dispatches
    /// both load the registry, both compute the SAME next_oracle_name, and both
    /// save — colliding on one name and losing a registration. The lock guards
    /// a fresh reload + name pick + register + save, so each caller sees the
    /// other's reservation. `preferred` (a caller-verified live idle oracle to
    /// reuse) is honored only if still Idle in the reloaded registry. The lock
    /// guards only synchronous IO — do any async liveness check BEFORE calling.
    pub fn reserve_oracle(
        state_dir: &Path,
        project: &str,
        preferred: Option<&str>,
    ) -> Result<String> {
        validate_project_identity(project)?;
        if let Some(preferred) = preferred {
            crate::scope::validate_session_identity(preferred)?;
        }
        let _lock = crate::scope::lock_private_state_file(state_dir, ".oracle-registry.lock")?;

        // Reload under the lock so a concurrent reservation is merged, not lost.
        let mut reg = Self::load_strict(state_dir)?;
        let name = match preferred {
            Some(p)
                if reg.oracles.iter().any(|e| {
                    e.oracle_name == p
                        && e.session_name == p
                        && e.project == project
                        && e.status == OracleRegistryStatus::Idle
                }) =>
            {
                p.to_string()
            }
            _ => reg.next_oracle_name(project),
        };
        reg.register_checked(OracleRegistryEntry {
            oracle_name: name.clone(),
            project: project.to_string(),
            session_name: name.clone(),
            status: OracleRegistryStatus::Active,
            spawned_at: Utc::now(),
            files_owned: Vec::new(),
        })?;
        reg.save_locked(state_dir)?;
        Ok(name)
        // lock drops here -> advisory lock released
    }

    fn register_checked(&mut self, entry: OracleRegistryEntry) -> Result<()> {
        Self::validate_entry(&entry)?;
        if let Some(existing) = self
            .oracles
            .iter()
            .find(|existing| existing.oracle_name == entry.oracle_name)
        {
            if existing.project != entry.project || existing.session_name != entry.session_name {
                bail!(
                    "oracle {} is already bound to project {} / session {}",
                    entry.oracle_name,
                    existing.project,
                    existing.session_name
                );
            }
        }
        if self.oracles.iter().any(|existing| {
            existing.session_name == entry.session_name && existing.oracle_name != entry.oracle_name
        }) {
            bail!(
                "session {} is already bound to another oracle",
                entry.session_name
            );
        }
        self.oracles.retain(|e| e.oracle_name != entry.oracle_name);
        self.oracles.push(entry);
        self.validate()
    }

    pub fn register(&mut self, entry: OracleRegistryEntry) -> Result<()> {
        self.register_checked(entry)
    }

    /// Apply mutations to the registry atomically, under the same exclusive
    /// advisory lock as `reserve_oracle`/`register_resurrected`. Patrol's
    /// previous pattern — load at tick start, mutate across awaits, save at
    /// the end — clobbered any oracle a concurrent locked dispatch
    /// registered mid-tick: the lost entry took its `spawned_at` with it,
    /// the freshness guard then treated EVERY signal of that oracle as
    /// stale (never upgraded, never reaped), and `next_oracle_name` could
    /// re-issue its name while the session was live. The closure runs on a
    /// FRESH reload while the lock is held — keep it synchronous and quick.
    pub fn update_locked<F>(state_dir: &Path, mutate: F) -> Result<()>
    where
        F: FnOnce(&mut OracleRegistry),
    {
        let _lock = crate::scope::lock_private_state_file(state_dir, ".oracle-registry.lock")?;
        let mut reg = Self::load_strict(state_dir)?;
        mutate(&mut reg);
        reg.save_locked(state_dir)
        // lock drops here -> advisory lock released
    }

    /// Re-register a resurrected oracle as Active with a FRESH `spawned_at`,
    /// under the same exclusive lock as reserve_oracle. The resurrect path
    /// previously never re-registered (the dead entry had been purged by
    /// cleanup), so the resurrected oracle was invisible to the registry and —
    /// critically — patrol had no `spawned_at` to date the session's done
    /// signal against: its freshness guard then treats every signal as stale.
    pub fn register_resurrected(state_dir: &Path, oracle_name: &str, project: &str) -> Result<()> {
        crate::scope::validate_session_identity(oracle_name)?;
        validate_project_identity(project)?;
        let _lock = crate::scope::lock_private_state_file(state_dir, ".oracle-registry.lock")?;
        let mut reg = Self::load_strict(state_dir)?;
        reg.register_checked(OracleRegistryEntry {
            oracle_name: oracle_name.to_string(),
            project: project.to_string(),
            session_name: oracle_name.to_string(),
            status: OracleRegistryStatus::Active,
            spawned_at: Utc::now(),
            files_owned: Vec::new(),
        })?;
        reg.save_locked(state_dir)
        // lock drops here -> advisory lock released
    }

    pub fn find_available(&self, project: &str) -> Option<&OracleRegistryEntry> {
        self.oracles
            .iter()
            .find(|e| e.project == project && e.status == OracleRegistryStatus::Idle)
    }

    pub fn find_active(&self, project: &str) -> Vec<&OracleRegistryEntry> {
        self.oracles
            .iter()
            .filter(|e| e.project == project && e.status == OracleRegistryStatus::Active)
            .collect()
    }

    pub fn count_active(&self, project: &str) -> usize {
        self.oracles
            .iter()
            .filter(|e| {
                e.project == project
                    && matches!(
                        e.status,
                        OracleRegistryStatus::Active | OracleRegistryStatus::Idle
                    )
            })
            .count()
    }

    pub fn mark_status(&mut self, oracle_name: &str, status: OracleRegistryStatus) {
        if let Some(entry) = self
            .oracles
            .iter_mut()
            .find(|e| e.oracle_name == oracle_name)
        {
            entry.status = status;
        }
    }

    /// Remove dead entries (sessions that no longer exist in rmux).
    /// `Done` entries are purged too once their session is gone — retaining
    /// them forever grew `next_oracle_name`'s index without bound and
    /// contradicted the dispatch-time "Dead-purged → name re-issued"
    /// contract (the stale-signal clear exists precisely because names
    /// recycle). A freshly-spawned entry is exempt for a short grace
    /// window: the caller's session list may be a snapshot taken BEFORE
    /// the spawn (patrol snapshots sessions at tick start), and purging it
    /// would erase the `spawned_at` the freshness guard depends on.
    pub fn cleanup(&mut self, live_sessions: &[String]) {
        const SPAWN_GRACE_SECS: i64 = 120;
        let now = Utc::now();
        for entry in &mut self.oracles {
            if !live_sessions.contains(&entry.session_name)
                && (now - entry.spawned_at).num_seconds() > SPAWN_GRACE_SECS
            {
                entry.status = OracleRegistryStatus::Dead;
            }
        }
        self.oracles
            .retain(|e| e.status != OracleRegistryStatus::Dead);
    }

    /// Next available oracle name for a project (oracle-Proj, oracle-Proj-2, ...).
    pub fn next_oracle_name(&self, project: &str) -> String {
        let base = format!("oracle-{project}");
        if !self
            .oracles
            .iter()
            .any(|entry| entry.oracle_name == base || entry.session_name == base)
        {
            return base;
        }
        // With N entries, at least one of N+1 consecutive suffixes is free.
        // Bound the search instead of relying on an effectively-infinite loop.
        for index in 2..=self.oracles.len().saturating_add(2) {
            let candidate = format!("{base}-{index}");
            if !self
                .oracles
                .iter()
                .any(|entry| entry.oracle_name == candidate || entry.session_name == candidate)
            {
                return candidate;
            }
        }
        unreachable!("N registry entries cannot occupy N+1 candidate names")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oracle_state_rejects_session_traversal_and_corrupt_authority_sweeps() {
        let tmp = tempfile::TempDir::new().unwrap();
        let mission = Mission::new("Acme", "safe", PathBuf::from("/tmp"));
        let state = OracleState::new("../oracle-Acme", &mission);
        assert!(state.write(tmp.path()).is_err());
        assert!(OracleState::read(tmp.path(), "../oracle-Acme").is_err());

        std::fs::write(tmp.path().join("oracle-broken.state.json"), b"{").unwrap();
        assert!(OracleState::read_all_strict(tmp.path()).is_err());
        assert!(OracleState::read_all(tmp.path()).is_empty());
    }

    #[test]
    fn read_all_skips_dead_purge_projections_without_crashing() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("oracle-mac-purge-20260822.state.json"),
            b"{",
        )
        .unwrap();
        let mission = Mission::new("demo", "safe", PathBuf::from("/tmp"));
        OracleState::new("oracle-demo", &mission)
            .write(tmp.path())
            .unwrap();
        let names: Vec<String> = OracleState::read_all(tmp.path())
            .into_iter()
            .map(|state| state.oracle_name)
            .collect();
        assert!(
            !names.iter().any(|name| name.contains("purge")),
            "broken purge projection must not abort the sweep: {names:?}"
        );
        assert!(
            names.iter().any(|name| name == "oracle-demo"),
            "valid oracles must still appear beside a dead purge file: {names:?}"
        );
    }

    #[test]
    fn oracle_state_cas_refuses_stale_full_document_writer() {
        let tmp = tempfile::TempDir::new().unwrap();
        let mission = Mission::new("Acme", "safe", PathBuf::from("/tmp"));
        OracleState::new("oracle-Acme", &mission)
            .write(tmp.path())
            .unwrap();
        let mut first = OracleState::read(tmp.path(), "oracle-Acme")
            .unwrap()
            .unwrap();
        let mut stale = first.clone();
        first.session_id = Some("first".to_string());
        first.write(tmp.path()).unwrap();
        stale.session_id = Some("stale".to_string());
        assert!(stale.write(tmp.path()).is_err());
        assert_eq!(
            OracleState::read(tmp.path(), "oracle-Acme")
                .unwrap()
                .unwrap()
                .session_id
                .as_deref(),
            Some("first")
        );
    }

    #[test]
    fn minimal_projection_cannot_persist_v3_worker_authority() {
        let tmp = tempfile::TempDir::new().unwrap();
        let mut state = OracleState::new_minimal("oracle-Acme", "Acme", PathBuf::from("/tmp/Acme"));
        state.register_worker(WorkerEntry {
            session_name: "Acme-worker-1".to_string(),
            task_id: "task-1".to_string(),
            task_name: "task".to_string(),
            attempt_id: Some("attempt-1".to_string()),
            plan_revision: Some(1),
            files_owned: vec!["src/lib.rs".to_string()],
            dispatched_at: Utc::now(),
            status: WorkerEntryStatus::Running,
        });
        assert!(state.write(tmp.path()).is_err());
        let ledger = MissionLedger::open_in_memory().unwrap();
        assert!(state.require_ledger_authority(&ledger).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn oracle_state_rejects_symlink_hardlink_and_unsafe_lock_paths() {
        use std::os::unix::fs::symlink;
        let root = tempfile::TempDir::new().unwrap();

        let symlink_dir = root.path().join("symlink");
        std::fs::create_dir(&symlink_dir).unwrap();
        let target = symlink_dir.join("target");
        std::fs::write(&target, b"sentinel").unwrap();
        symlink(&target, symlink_dir.join("oracle-Acme.state.json")).unwrap();
        assert!(OracleState::read(&symlink_dir, "oracle-Acme").is_err());
        assert_eq!(std::fs::read(&target).unwrap(), b"sentinel");

        let hardlink_dir = root.path().join("hardlink");
        std::fs::create_dir(&hardlink_dir).unwrap();
        let hard_target = hardlink_dir.join("target");
        std::fs::write(&hard_target, b"{}").unwrap();
        std::fs::hard_link(&hard_target, hardlink_dir.join("oracle-Acme.state.json")).unwrap();
        assert!(OracleState::read(&hardlink_dir, "oracle-Acme").is_err());

        let lock_dir = root.path().join("lock");
        std::fs::create_dir(&lock_dir).unwrap();
        let lock_target = lock_dir.join("target");
        std::fs::write(&lock_target, b"sentinel").unwrap();
        symlink(&lock_target, lock_dir.join(".oracle-Acme.state.lock")).unwrap();
        let mission = Mission::new("Acme", "safe", PathBuf::from("/tmp"));
        assert!(OracleState::new("oracle-Acme", &mission)
            .write(&lock_dir)
            .is_err());
        assert_eq!(std::fs::read(&lock_target).unwrap(), b"sentinel");
    }

    #[test]
    fn oracle_projection_is_derived_from_and_validated_against_ledger() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = Mission::new("OmegaOS", "projection", PathBuf::from("/tmp/OmegaOS"));
        let created = ledger
            .create_mission(
                &mission,
                &format!("test:{}:created", mission.id.as_str()),
                "test",
            )
            .unwrap();
        let state = OracleState::from_ledger("oracle-OmegaOS", &mission, &created).unwrap();
        assert_eq!(
            state.require_ledger_authority(&ledger).unwrap(),
            created.projection
        );
    }

    #[test]
    fn forged_projection_metadata_cannot_override_ledger() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = Mission::new("OmegaOS", "projection", PathBuf::from("/tmp/OmegaOS"));
        let created = ledger
            .create_mission(
                &mission,
                &format!("test:{}:created", mission.id.as_str()),
                "test",
            )
            .unwrap();
        let mut state = OracleState::from_ledger("oracle-OmegaOS", &mission, &created).unwrap();
        state
            .compatibility_projection
            .as_mut()
            .unwrap()
            .source_projection_hash = "forged".to_string();
        assert!(state.require_ledger_authority(&ledger).is_err());
        assert_eq!(
            ledger.mission(&mission.id).unwrap().unwrap(),
            created.projection,
            "mutating the JSON projection must not mutate ledger authority"
        );
    }

    #[test]
    fn forged_hash_on_a_stale_projection_is_rejected() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = Mission::new("OmegaOS", "projection", PathBuf::from("/tmp/OmegaOS"));
        let created = ledger
            .create_mission(
                &mission,
                &format!("test:{}:created", mission.id.as_str()),
                "test",
            )
            .unwrap();
        let mut state = OracleState::from_ledger("oracle-OmegaOS", &mission, &created).unwrap();
        let mut cancelled = crate::mission_ledger::AppendEvent::new(
            mission.id.clone(),
            created.projection.version,
            format!("test:{}:cancelled", mission.id.as_str()),
            "test",
            "mission_cancelled",
        );
        cancelled.next_mission_state = Some(crate::mission::MissionState::Cancelled);
        ledger.append(cancelled).unwrap();
        state
            .compatibility_projection
            .as_mut()
            .unwrap()
            .source_projection_hash = "forged-stale-hash".to_string();
        assert!(
            state.require_ledger_authority(&ledger).is_err(),
            "a stale projection must still prove the exact historical hash it cites"
        );
    }

    #[test]
    fn stamped_state_cannot_forge_immutable_mission_or_future_phase() {
        let tmp = tempfile::TempDir::new().unwrap();
        let ledger = MissionLedger::open(mission_ledger_path(tmp.path())).unwrap();
        let mission = Mission::new(
            "OmegaOS",
            "immutable mission",
            PathBuf::from("/tmp/OmegaOS-immutable"),
        );
        let created = ledger
            .create_mission(
                &mission,
                &format!("test:{}:created", mission.id.as_str()),
                "test",
            )
            .unwrap();
        let state = OracleState::from_ledger("oracle-OmegaOS", &mission, &created).unwrap();

        let mut forged_project = state.clone();
        forged_project.project = "Other".to_string();
        assert!(forged_project.require_ledger_authority(&ledger).is_err());

        let mut forged_text = state.clone();
        forged_text.mission_text = "different mission".to_string();
        assert!(forged_text.require_ledger_authority(&ledger).is_err());

        let mut forged_worktree = state.clone();
        forged_worktree.working_dir = PathBuf::from("/tmp/Other-worktree");
        assert!(forged_worktree.require_ledger_authority(&ledger).is_err());

        let mut forged_phase = state;
        forged_phase.phase = OraclePhase::Done;
        assert!(forged_phase.require_ledger_authority(&ledger).is_err());
    }

    #[test]
    fn done_clean_worker_requires_exact_accepted_attempt() {
        use crate::mission::{
            PlanContract, RetryPolicy, TaskContract, TaskId, VerifierCheck, VerifierCheckKind,
            CONTRACT_SCHEMA_VERSION,
        };
        use crate::mission_ledger::{AppendEvent, TaskAttemptMutation};

        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = Mission::new("OmegaOS", "worker status", PathBuf::from("/tmp/OmegaOS"));
        ledger
            .create_mission(
                &mission,
                &format!("test:{}:created", mission.id.as_str()),
                "test",
            )
            .unwrap();
        let mut classified = AppendEvent::new(
            mission.id.clone(),
            1,
            format!("test:{}:classified", mission.id.as_str()),
            "test",
            "mission_classified",
        );
        classified.next_mission_state = Some(MissionState::Classified);
        ledger.append(classified).unwrap();

        let task = TaskContract {
            schema_version: CONTRACT_SCHEMA_VERSION,
            task_id: TaskId::new("task-1"),
            name: "implement".to_string(),
            prompt: "implement safely".to_string(),
            acceptance_criteria: vec!["verified".to_string()],
            verifier_checks: vec![VerifierCheck {
                schema_version: CONTRACT_SCHEMA_VERSION,
                check_id: "file".to_string(),
                kind: VerifierCheckKind::FileExists {
                    path: "src/lib.rs".to_string(),
                },
                timeout_secs: 30,
            }],
            required_capabilities: Vec::new(),
            scope: vec!["src/lib.rs".to_string()],
            risk: crate::routing::RiskLevel::Low,
            retry_policy: RetryPolicy::default(),
            depends_on: Vec::new(),
        };
        let plan = PlanContract::new(mission.id.clone(), 1, 2, vec![task], Vec::new(), Vec::new())
            .unwrap();
        let mut planned = AppendEvent::new(
            mission.id.clone(),
            2,
            format!("test:{}:planned", mission.id.as_str()),
            "test",
            "mission_planned",
        );
        planned.next_mission_state = Some(MissionState::Planned);
        planned.plan = Some(plan);
        ledger.append(planned).unwrap();

        let attempt_id = "attempt-1".to_string();
        let mut queued = AppendEvent::new(
            mission.id.clone(),
            3,
            "test:attempt:queued",
            "test",
            "task_attempt_queued",
        );
        queued.task_attempt = Some(TaskAttemptMutation {
            task_id: "task-1".to_string(),
            attempt_id: attempt_id.clone(),
            plan_revision: 1,
            expected_version: 0,
            next_state: TaskAttemptState::Queued,
        });
        ledger.append(queued).unwrap();
        let mut running = AppendEvent::new(
            mission.id.clone(),
            4,
            "test:attempt:running",
            "test",
            "task_attempt_running",
        );
        running.next_mission_state = Some(MissionState::Running);
        running.task_attempt = Some(TaskAttemptMutation {
            task_id: "task-1".to_string(),
            attempt_id: attempt_id.clone(),
            plan_revision: 1,
            expected_version: 1,
            next_state: TaskAttemptState::Running,
        });
        let running = ledger.append(running).unwrap();

        let mut state = OracleState::from_ledger("oracle-OmegaOS", &mission, &running).unwrap();
        state.phase = OraclePhase::Monitor;
        state.register_worker(WorkerEntry {
            session_name: "OmegaOS-worker-1".to_string(),
            task_id: "task-1".to_string(),
            task_name: "implement".to_string(),
            attempt_id: Some(attempt_id),
            plan_revision: Some(1),
            files_owned: vec!["src/lib.rs".to_string()],
            dispatched_at: Utc::now(),
            status: WorkerEntryStatus::DoneClean,
        });
        assert!(state.require_ledger_authority(&ledger).is_err());
        state.workers[0].status = WorkerEntryStatus::Running;
        assert!(state.require_ledger_authority(&ledger).is_ok());
    }

    #[test]
    fn legacy_oracle_cannot_hide_unregistered_live_worker_from_strict_close_gate() {
        let tmp = tempfile::TempDir::new().unwrap();
        let ledger = MissionLedger::open(mission_ledger_path(tmp.path())).unwrap();
        let mission = Mission::new(
            "OmegaOS",
            "strict worker ownership",
            PathBuf::from("/tmp/OmegaOS"),
        );
        let created = ledger
            .create_mission(
                &mission,
                &format!("test:{}:created", mission.id.as_str()),
                "test",
            )
            .unwrap();
        OracleState::from_ledger("oracle-OmegaOS", &mission, &created)
            .unwrap()
            .write(tmp.path())
            .unwrap();

        let mut legacy =
            OracleState::new_minimal("oracle-OmegaOS-2", "OmegaOS", PathBuf::from("/tmp/OmegaOS"));
        legacy.register_worker(WorkerEntry {
            session_name: "OmegaOS-worker-orphan".to_string(),
            task_id: "legacy-task".to_string(),
            task_name: "untrusted legacy claim".to_string(),
            attempt_id: None,
            plan_revision: None,
            files_owned: vec!["src/lib.rs".to_string()],
            dispatched_at: Utc::now(),
            status: WorkerEntryStatus::DoneClean,
        });
        legacy.write(tmp.path()).unwrap();

        let live = vec![crate::session::OmegaSession::classify(
            "OmegaOS-worker-orphan",
        )];
        let workers = live_workers_of_oracle_strict(tmp.path(), "oracle-OmegaOS", &live).unwrap();
        assert_eq!(workers.running, vec!["OmegaOS-worker-orphan"]);
        assert!(workers.terminal.is_empty());
    }

    #[test]
    fn stall_detector_idle_prompt() {
        assert!(WorkerStallDetector::detect_idle_prompt("some output\n❯"));
        assert!(WorkerStallDetector::detect_idle_prompt("some output\n❯ "));
        assert!(WorkerStallDetector::detect_idle_prompt("hacker@vps ~ $ "));
        // N11: an empty pane capture is NOT an idle prompt (was a false-positive).
        assert!(!WorkerStallDetector::detect_idle_prompt(""));
        assert!(!WorkerStallDetector::detect_idle_prompt("Thinking..."));
        assert!(!WorkerStallDetector::detect_idle_prompt("Writing file.rs"));
        assert!(!WorkerStallDetector::detect_idle_prompt("Running tests..."));
    }

    #[test]
    fn idle_prompt_excludes_code_and_markup_endings() {
        // N11: a line that ends in '>' but is code/markup is NOT an idle prompt.
        assert!(!WorkerStallDetector::detect_idle_prompt(
            "let f = |x| x => x + 1"
        ));
        assert!(!WorkerStallDetector::detect_idle_prompt(
            "fn foo() -> u32 {\nreturn 1 ->"
        ));
        assert!(!WorkerStallDetector::detect_idle_prompt(
            "<input type=\"text\" />"
        ));
        assert!(!WorkerStallDetector::detect_idle_prompt("    </div>"));
        assert!(!WorkerStallDetector::detect_idle_prompt("let v: Vec<T>"));
        assert!(!WorkerStallDetector::detect_idle_prompt("arr.map(x => x);"));
        // A genuine bare '>' prompt still counts.
        assert!(WorkerStallDetector::detect_idle_prompt("output\n>"));
        assert!(WorkerStallDetector::detect_idle_prompt("output\n> "));
    }

    #[test]
    fn oracle_phase_transitions() {
        let mission = Mission::new("TestProject", "Fix a bug", PathBuf::from("/tmp"));
        let mut state = OracleState::new("oracle-TestProject", &mission);

        assert_eq!(state.phase, OraclePhase::Analyze);
        state.transition(OraclePhase::Dispatch, "Analysis complete");
        assert_eq!(state.phase, OraclePhase::Dispatch);
        assert_eq!(state.phase_history.len(), 1);
    }

    #[test]
    fn god_mode_iteration_limit() {
        let mut gm = GodModeState::new(3);
        assert!(gm.start_new_iteration()); // iter 2
        assert!(gm.start_new_iteration()); // iter 3
        assert!(!gm.start_new_iteration()); // at max, returns false
    }

    #[test]
    fn signal_file_markdown_format() {
        let mission = Mission::new("MyProject", "Do something", PathBuf::from("/tmp"));
        let state = OracleState::new("oracle-MyProject", &mission);
        let mut signal = OracleSignalFile::from_oracle_state(&state, SignalBuild::Pass);
        signal.summary = "Fixed the auth flow.".to_string();
        signal.next_steps = vec!["Deploy to prod".to_string()];

        let tmp = tempfile::TempDir::new().unwrap();
        let path = signal.write_markdown(tmp.path()).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();

        assert!(content.contains("PROJECT:MyProject"));
        assert!(content.contains("STATUS:DONE"));
        assert!(content.contains("BUILD:PASS"));
        assert!(content.contains("Fixed the auth flow."));
        assert!(content.contains("Deploy to prod"));
    }

    #[test]
    fn oracle_signal_is_strict_private_cas_and_watcher_fails_closed() {
        let tmp = tempfile::TempDir::new().unwrap();
        let mission = Mission::new("Acme", "signal", PathBuf::from("/tmp/Acme"));
        let state = OracleState::new("oracle-Acme", &mission);
        let signal = OracleSignalFile::from_oracle_state(&state, SignalBuild::Pass);
        let path = signal.write(tmp.path()).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
            assert_eq!(
                std::fs::metadata(tmp.path()).unwrap().permissions().mode() & 0o777,
                0o700
            );
        }

        let mut first = OracleSignalFile::read(tmp.path(), "oracle-Acme")
            .unwrap()
            .unwrap();
        let mut stale = first.clone();
        first.summary = "accepted".to_string();
        first.write(tmp.path()).unwrap();
        stale.summary = "stale".to_string();
        assert!(stale.write(tmp.path()).is_err());

        let mut watcher = SignalWatcher::new(tmp.path().to_path_buf());
        assert_eq!(watcher.poll().unwrap().len(), 1);
        assert!(watcher.poll().unwrap().is_empty());
        std::fs::write(&path, b"{").unwrap();
        assert!(watcher.poll().is_err(), "corruption must not disappear");
    }

    #[cfg(unix)]
    #[test]
    fn oracle_signal_rejects_symlink_and_hardlink_authority_paths() {
        use std::os::unix::fs::symlink;
        let root = tempfile::TempDir::new().unwrap();
        let mission = Mission::new("Acme", "signal", PathBuf::from("/tmp/Acme"));
        let state = OracleState::new("oracle-Acme", &mission);
        let signal = OracleSignalFile::from_oracle_state(&state, SignalBuild::Pass);

        let symlink_dir = root.path().join("symlink");
        std::fs::create_dir(&symlink_dir).unwrap();
        let symlink_target = symlink_dir.join("target");
        std::fs::write(&symlink_target, b"sentinel").unwrap();
        symlink(&symlink_target, symlink_dir.join("oracle-Acme.result.json")).unwrap();
        assert!(signal.write(&symlink_dir).is_err());
        assert_eq!(std::fs::read(&symlink_target).unwrap(), b"sentinel");

        let hardlink_dir = root.path().join("hardlink");
        std::fs::create_dir(&hardlink_dir).unwrap();
        let hardlink_target = hardlink_dir.join("target");
        std::fs::write(&hardlink_target, b"{}").unwrap();
        std::fs::hard_link(
            &hardlink_target,
            hardlink_dir.join("oracle-Acme.result.json"),
        )
        .unwrap();
        assert!(signal.write(&hardlink_dir).is_err());

        let markdown_dir = root.path().join("markdown");
        std::fs::create_dir(&markdown_dir).unwrap();
        let markdown_target = markdown_dir.join("target");
        std::fs::write(&markdown_target, b"sentinel").unwrap();
        symlink(&markdown_target, markdown_dir.join("oracle-result-Acme.md")).unwrap();
        assert!(signal.write_markdown(&markdown_dir).is_err());
        assert_eq!(std::fs::read(&markdown_target).unwrap(), b"sentinel");
    }

    #[test]
    fn oracle_signal_read_migrates_legacy_double_prefix_under_lock() {
        let tmp = tempfile::TempDir::new().unwrap();
        let mission = Mission::new("Acme", "signal", PathBuf::from("/tmp/Acme"));
        let state = OracleState::new("oracle-Acme", &mission);
        let signal = OracleSignalFile::from_oracle_state(&state, SignalBuild::Pass);
        let legacy = tmp.path().join("oracle-oracle-Acme.result.json");
        crate::config::atomic_write_private(&legacy, &serde_json::to_vec_pretty(&signal).unwrap())
            .unwrap();

        let observed = OracleSignalFile::read(tmp.path(), "oracle-Acme")
            .unwrap()
            .unwrap();
        assert_eq!(observed.oracle, "oracle-Acme");
        assert!(tmp.path().join("oracle-Acme.result.json").exists());
        assert!(!legacy.exists());
    }

    #[test]
    fn oracle_registry_next_name() {
        let mut reg = OracleRegistry::default();

        assert_eq!(reg.next_oracle_name("Kommu"), "oracle-Kommu");

        reg.register(OracleRegistryEntry {
            oracle_name: "oracle-Kommu".to_string(),
            project: "Kommu".to_string(),
            session_name: "oracle-Kommu".to_string(),
            status: OracleRegistryStatus::Active,
            spawned_at: Utc::now(),
            files_owned: Vec::new(),
        })
        .unwrap();

        assert_eq!(reg.next_oracle_name("Kommu"), "oracle-Kommu-2");
    }

    #[test]
    fn reserve_oracle_concurrent_names_are_unique() {
        use std::collections::HashSet;
        use std::sync::Arc;
        use std::thread;
        const N: usize = 8;

        let tmp = tempfile::TempDir::new().unwrap();
        let dir = Arc::new(tmp.path().to_path_buf());

        // N threads race to reserve an oracle for the same project at once.
        let mut handles = Vec::new();
        for _ in 0..N {
            let dir = Arc::clone(&dir);
            handles.push(thread::spawn(move || {
                OracleRegistry::reserve_oracle(dir.as_path(), "Race", None).unwrap()
            }));
        }
        let names: Vec<String> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        // The exclusive lock serializes name allocation, so every concurrent
        // reservation gets a DISTINCT name (pre-fix they all read the same stale
        // registry and collided on oracle-Race / -2 …).
        let unique: HashSet<&String> = names.iter().collect();
        assert_eq!(
            unique.len(),
            N,
            "concurrent reservations must be unique: {:?}",
            names
        );

        // All N registrations survive (none clobbered by a racing save).
        let reg = OracleRegistry::load_strict(dir.as_path()).unwrap();
        assert_eq!(
            reg.oracles.iter().filter(|e| e.project == "Race").count(),
            N
        );
    }

    #[test]
    fn oracle_registry_strict_load_cas_and_cross_project_reservation() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        assert_eq!(
            OracleRegistry::reserve_oracle(dir, "Foo", None).unwrap(),
            "oracle-Foo"
        );
        OracleRegistry::update_locked(dir, |registry| {
            registry.mark_status("oracle-Foo", OracleRegistryStatus::Idle);
        })
        .unwrap();
        assert_eq!(
            OracleRegistry::reserve_oracle(dir, "Bar", Some("oracle-Foo")).unwrap(),
            "oracle-Bar",
            "an idle oracle from another project must never be reused"
        );

        let mut first = OracleRegistry::load_strict(dir).unwrap();
        let mut stale = first.clone();
        first.mark_status("oracle-Foo", OracleRegistryStatus::Done);
        first.save(dir).unwrap();
        stale.mark_status("oracle-Foo", OracleRegistryStatus::Active);
        assert!(stale.save(dir).is_err());
        assert_eq!(
            OracleRegistry::load_strict(dir)
                .unwrap()
                .oracles
                .iter()
                .find(|entry| entry.oracle_name == "oracle-Foo")
                .unwrap()
                .status,
            OracleRegistryStatus::Done
        );

        assert!(OracleRegistry::reserve_oracle(dir, "../escape", None).is_err());
        assert!(OracleRegistry::reserve_oracle(dir, "Safe", Some("../escape")).is_err());
    }

    #[test]
    fn oracle_registry_corruption_blocks_mutation_and_project_prefix_collision() {
        let corrupt = tempfile::TempDir::new().unwrap();
        let path = corrupt.path().join("oracle-registry.json");
        std::fs::write(&path, b"{").unwrap();
        assert!(OracleRegistry::load_strict(corrupt.path()).is_err());
        assert!(OracleRegistry::reserve_oracle(corrupt.path(), "Acme", None).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"{");

        let collision = tempfile::TempDir::new().unwrap();
        assert_eq!(
            OracleRegistry::reserve_oracle(collision.path(), "Foo", None).unwrap(),
            "oracle-Foo"
        );
        assert_eq!(
            OracleRegistry::reserve_oracle(collision.path(), "Foo", None).unwrap(),
            "oracle-Foo-2"
        );
        assert_eq!(
            OracleRegistry::reserve_oracle(collision.path(), "Foo-2", None).unwrap(),
            "oracle-Foo-2-2",
            "project names that overlap another project's numeric suffix need a globally unique session"
        );
    }

    #[cfg(unix)]
    #[test]
    fn oracle_registry_rejects_symlink_hardlink_and_lock_aliases() {
        use std::os::unix::fs::symlink;
        let root = tempfile::TempDir::new().unwrap();

        let symlink_dir = root.path().join("symlink");
        std::fs::create_dir(&symlink_dir).unwrap();
        let symlink_target = symlink_dir.join("target");
        std::fs::write(&symlink_target, b"sentinel").unwrap();
        symlink(&symlink_target, symlink_dir.join("oracle-registry.json")).unwrap();
        assert!(OracleRegistry::load_strict(&symlink_dir).is_err());
        assert!(OracleRegistry::reserve_oracle(&symlink_dir, "Acme", None).is_err());
        assert_eq!(std::fs::read(&symlink_target).unwrap(), b"sentinel");

        let hardlink_dir = root.path().join("hardlink");
        std::fs::create_dir(&hardlink_dir).unwrap();
        let hardlink_target = hardlink_dir.join("target");
        std::fs::write(&hardlink_target, b"{}").unwrap();
        std::fs::hard_link(&hardlink_target, hardlink_dir.join("oracle-registry.json")).unwrap();
        assert!(OracleRegistry::load_strict(&hardlink_dir).is_err());

        let lock_dir = root.path().join("lock");
        std::fs::create_dir(&lock_dir).unwrap();
        let lock_target = lock_dir.join("target");
        std::fs::write(&lock_target, b"sentinel").unwrap();
        symlink(&lock_target, lock_dir.join(".oracle-registry.lock")).unwrap();
        assert!(OracleRegistry::reserve_oracle(&lock_dir, "Acme", None).is_err());
        assert_eq!(std::fs::read(&lock_target).unwrap(), b"sentinel");
    }

    #[test]
    fn oracle_state_filename_is_prefix_normalized() {
        // Callers hold the FULL session name (`oracle-X`); the file on disk
        // must be single-prefixed (`oracle-X.state.json`), never
        // `oracle-oracle-X.state.json` — same rule as OracleDoneSignal.
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        let mission = Mission::new("Acme", "do it", PathBuf::from("/tmp"));
        let state = OracleState::new("oracle-Acme-2", &mission);
        state.write(dir).unwrap();

        assert!(dir.join("oracle-Acme-2.state.json").exists());
        assert!(!dir.join("oracle-oracle-Acme-2.state.json").exists());

        // Both name forms resolve the same file.
        assert!(OracleState::read(dir, "oracle-Acme-2").unwrap().is_some());
        assert!(OracleState::read(dir, "Acme-2").unwrap().is_some());
    }

    #[test]
    fn oracle_state_read_migrates_legacy_double_prefix() {
        // A state written by a pre-normalization binary lives at the
        // double-prefixed name; read() must find it AND rename it into place.
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        let mission = Mission::new("Acme", "do it", PathBuf::from("/tmp"));
        let state = OracleState::new("oracle-Acme", &mission);
        let legacy = dir.join("oracle-oracle-Acme.state.json");
        std::fs::write(&legacy, serde_json::to_string_pretty(&state).unwrap()).unwrap();

        let read = OracleState::read(dir, "oracle-Acme").unwrap();
        assert!(read.is_some(), "legacy state must remain readable");
        assert!(
            dir.join("oracle-Acme.state.json").exists(),
            "must migrate in place"
        );
        assert!(!legacy.exists(), "legacy file must be renamed away");
    }

    #[test]
    fn register_worker_upsert_refreshes_dispatched_at() {
        // A re-dispatch under the same deterministic worker name must
        // refresh dispatched_at (the worker freshness guard dates done
        // signals against it) without duplicating the entry.
        let mission = Mission::new("Acme", "do it", PathBuf::from("/tmp"));
        let mut state = OracleState::new("oracle-Acme", &mission);
        let t0 = Utc::now() - chrono::Duration::hours(2);
        state.register_worker(WorkerEntry {
            session_name: "Acme-worker-x".into(),
            task_id: "t1".into(),
            task_name: "x".into(),
            attempt_id: None,
            plan_revision: None,
            files_owned: vec![],
            dispatched_at: t0,
            status: WorkerEntryStatus::DoneClean,
        });
        let t1 = Utc::now();
        state.register_worker(WorkerEntry {
            session_name: "Acme-worker-x".into(),
            task_id: "t1".into(),
            task_name: "x".into(),
            attempt_id: None,
            plan_revision: None,
            files_owned: vec![],
            dispatched_at: t1,
            status: WorkerEntryStatus::Running,
        });
        assert_eq!(state.workers.len(), 1, "no duplicate entry");
        assert_eq!(state.workers[0].dispatched_at, t1);
        assert_eq!(state.workers[0].status, WorkerEntryStatus::Running);
    }

    #[test]
    fn locked_worker_registration_preserves_concurrent_legacy_workers() {
        let tmp = tempfile::TempDir::new().unwrap();
        let state_dir = std::sync::Arc::new(tmp.path().to_path_buf());
        let mut threads = Vec::new();
        for index in 0..2 {
            let state_dir = state_dir.clone();
            threads.push(std::thread::spawn(move || {
                let oracle = "oracle-Acme";
                let fallback = OracleState::new_minimal(oracle, "Acme", PathBuf::from("/tmp/Acme"));
                OracleState::register_worker_locked(
                    &state_dir,
                    oracle,
                    Some(fallback),
                    WorkerEntry {
                        session_name: format!("Acme-worker-{index}"),
                        task_id: format!("task-{index}"),
                        task_name: format!("task {index}"),
                        attempt_id: None,
                        plan_revision: None,
                        files_owned: Vec::new(),
                        dispatched_at: Utc::now(),
                        status: WorkerEntryStatus::Running,
                    },
                )
            }));
        }
        for thread in threads {
            thread.join().unwrap().unwrap();
        }
        let state = OracleState::read(&state_dir, "oracle-Acme")
            .unwrap()
            .unwrap();
        assert_eq!(state.workers.len(), 2);
        assert!(state
            .workers
            .iter()
            .any(|worker| worker.session_name == "Acme-worker-0"));
        assert!(state
            .workers
            .iter()
            .any(|worker| worker.session_name == "Acme-worker-1"));
    }

    #[test]
    fn cleanup_purges_done_entries_with_dead_sessions_after_grace() {
        let mut reg = OracleRegistry::default();
        let old = Utc::now() - chrono::Duration::hours(1);
        // Done + dead session + past the spawn grace → purged.
        reg.register(OracleRegistryEntry {
            oracle_name: "oracle-A".into(),
            project: "A".into(),
            session_name: "oracle-A".into(),
            status: OracleRegistryStatus::Done,
            spawned_at: old,
            files_owned: vec![],
        })
        .unwrap();
        // Active + dead but JUST spawned → kept (snapshot may predate spawn).
        reg.register(OracleRegistryEntry {
            oracle_name: "oracle-B".into(),
            project: "B".into(),
            session_name: "oracle-B".into(),
            status: OracleRegistryStatus::Active,
            spawned_at: Utc::now(),
            files_owned: vec![],
        })
        .unwrap();
        // Done + LIVE session → kept.
        reg.register(OracleRegistryEntry {
            oracle_name: "oracle-C".into(),
            project: "C".into(),
            session_name: "oracle-C".into(),
            status: OracleRegistryStatus::Done,
            spawned_at: old,
            files_owned: vec![],
        })
        .unwrap();
        reg.cleanup(&["oracle-C".to_string()]);
        let names: Vec<&str> = reg.oracles.iter().map(|e| e.oracle_name.as_str()).collect();
        assert!(
            !names.contains(&"oracle-A"),
            "Done+dead+aged must be purged"
        );
        assert!(
            names.contains(&"oracle-B"),
            "fresh spawn must survive a stale snapshot"
        );
        assert!(names.contains(&"oracle-C"), "Done+live must be retained");
    }

    #[test]
    fn update_locked_persists_mutations() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        OracleRegistry::reserve_oracle(dir, "Acme", None).unwrap();
        OracleRegistry::update_locked(dir, |reg| {
            reg.mark_status("oracle-Acme", OracleRegistryStatus::Done);
        })
        .unwrap();
        let reg = OracleRegistry::load_strict(dir).unwrap();
        assert_eq!(reg.oracles[0].status, OracleRegistryStatus::Done);
    }

    #[test]
    fn ship_detection() {
        assert!(OraclePromptGenerator::should_ship("Fix auth and deploy"));
        assert!(OraclePromptGenerator::should_ship("envoie en prod"));
        assert!(OraclePromptGenerator::should_ship("ship the feature"));
        assert!(!OraclePromptGenerator::should_ship("just fix the bug"));
    }

    #[test]
    fn god_mode_detection() {
        assert!(OraclePromptGenerator::is_god_mode("Run in god mode"));
        assert!(OraclePromptGenerator::is_god_mode(
            "/godmode fix everything"
        ));
        assert!(!OraclePromptGenerator::is_god_mode("Fix the login flow"));
    }

    // The operator's stated objective: oracles must LAUNCH worker sessions they
    // supervise, not do the work themselves. That only happens if the generated
    // prompt actually says so, per mission. This test fails the moment the
    // orchestration block stops reaching a real oracle prompt.
    #[test]
    fn the_generated_oracle_prompt_carries_the_mission_shape() {
        let prompt = OraclePromptGenerator::generate(
            "camelia",
            std::path::Path::new("/tmp/camelia"),
            "oracle-camelia-1",
            "audite le code et corrige ce que tu trouves en parallele",
            false,
            false,
        );
        assert!(
            prompt.contains("How to run THIS mission"),
            "shape block missing"
        );
        assert!(
            prompt.contains("P-AUDIT"),
            "an audit mission must match the audit shape"
        );
        assert!(
            prompt.contains("P-PARALLEL"),
            "\"en parallele\" must match the parallel shape"
        );
        assert!(
            prompt.contains("spawn-worker"),
            "it must tell the oracle to dispatch"
        );
        assert!(
            prompt.contains("Done when"),
            "it must carry a stop condition"
        );
        // And the mission itself still leads.
        assert!(prompt.starts_with("## Mission"));
    }

    /// A mission that matches nothing still gets the orchestration floor —
    /// never a prompt with no shape at all.
    #[test]
    fn an_unclassifiable_mission_still_gets_a_shape() {
        let prompt = OraclePromptGenerator::generate(
            "p",
            std::path::Path::new("/tmp/p"),
            "oracle-p-1",
            "do the thing",
            false,
            false,
        );
        assert!(prompt.contains("How to run THIS mission"));
        assert!(prompt.contains("spawn-worker"));
    }
}

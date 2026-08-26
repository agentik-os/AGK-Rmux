use crate::session::SessionManager;
use anyhow::{Context, Result};
use rmux_sdk::SplitDirection;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub name: String,
    pub role: String,
    pub prompt: String,
    pub files_owned: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamConfig {
    pub project: String,
    pub session_name: String,
    pub working_dir: String,
    pub agent_command: String,
    pub members: Vec<TeamMember>,
}

pub const TEAM_RUN_SCHEMA_VERSION: u32 = 1;

/// Persisted bridge between one multi-pane rmux team and the immutable V3
/// attempts owned by its logical members. Member identities are deliberately
/// unique per run, so a delayed completion from an older team can never bind
/// to a replacement run for the same project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamRunMember {
    pub session: String,
    pub task_id: String,
    pub attempt_id: String,
    pub plan_revision: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamRunState {
    pub schema_version: u32,
    pub team_session: String,
    pub project: String,
    pub mission_id: crate::mission::MissionId,
    pub working_dir: PathBuf,
    pub members: Vec<TeamRunMember>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl TeamRunState {
    fn path(state_dir: &Path, team_session: &str) -> Result<PathBuf> {
        validate_team_session_name(team_session)?;
        Ok(state_dir.join(format!("team-run-{team_session}.json")))
    }

    pub fn validate(&self) -> Result<()> {
        if self.schema_version != TEAM_RUN_SCHEMA_VERSION {
            anyhow::bail!("unsupported team run schema {}", self.schema_version);
        }
        validate_team_session_name(&self.team_session)?;
        if self.mission_id.as_str().trim().is_empty() {
            anyhow::bail!("team run has an empty mission id");
        }
        if !self.working_dir.is_absolute() || !self.working_dir.is_dir() {
            anyhow::bail!(
                "team run working directory {} is not an accessible absolute directory",
                self.working_dir.display()
            );
        }
        if self.members.is_empty() {
            anyhow::bail!("team run has no members");
        }
        let mut sessions = HashSet::new();
        let mut tasks = HashSet::new();
        let mut attempts = HashSet::new();
        for member in &self.members {
            crate::scope::validate_session_identity(&member.session)?;
            if !member
                .session
                .strip_prefix(&format!("{}-", self.team_session))
                .is_some_and(|suffix| !suffix.is_empty())
            {
                anyhow::bail!(
                    "team member {} is not owned by team session {}",
                    member.session,
                    self.team_session
                );
            }
            if member.task_id.trim().is_empty()
                || member.attempt_id.trim().is_empty()
                || member.plan_revision == 0
            {
                anyhow::bail!(
                    "team member {} has an incomplete V3 binding",
                    member.session
                );
            }
            if !sessions.insert(member.session.clone())
                || !tasks.insert(member.task_id.clone())
                || !attempts.insert(member.attempt_id.clone())
            {
                anyhow::bail!("team run contains duplicate member authority");
            }
        }
        Ok(())
    }

    pub fn write(&self, state_dir: &Path) -> Result<()> {
        self.validate()?;
        std::fs::create_dir_all(state_dir)?;
        crate::config::atomic_write_private(
            &Self::path(state_dir, &self.team_session)?,
            &serde_json::to_vec_pretty(self)?,
        )
    }

    pub fn remove(&self, state_dir: &Path) -> Result<()> {
        crate::scope::remove_private_file(&Self::path(state_dir, &self.team_session)?)
    }

    pub fn read_all_strict(state_dir: &Path) -> Result<Vec<Self>> {
        let mut runs = Vec::new();
        let entries = match std::fs::read_dir(state_dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(runs),
            Err(error) => return Err(error.into()),
        };
        for entry in entries {
            let entry = entry?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.starts_with("team-run-") || !name.ends_with(".json") {
                continue;
            }
            let run = crate::scope::read_private_json::<Self>(&entry.path())?
                .ok_or_else(|| anyhow::anyhow!("team run {} disappeared while reading", name))?;
            run.validate()?;
            if Self::path(state_dir, &run.team_session)? != entry.path() {
                anyhow::bail!("team run filename/document identity mismatch for {name}");
            }
            runs.push(run);
        }
        runs.sort_by(|left, right| left.team_session.cmp(&right.team_session));
        Ok(runs)
    }

    pub fn find_member(state_dir: &Path, session: &str) -> Result<Option<(Self, TeamRunMember)>> {
        let mut matches = Vec::new();
        for run in Self::read_all_strict(state_dir)? {
            if let Some(member) = run
                .members
                .iter()
                .find(|member| member.session == session)
                .cloned()
            {
                matches.push((run, member));
            }
        }
        match matches.len() {
            0 => Ok(None),
            1 => Ok(matches.pop()),
            count => anyhow::bail!("member {session} appears in {count} team run bindings"),
        }
    }
}

/// Append the immutable completion-candidate event for a logical team member.
/// The persisted run binding is the only lookup authority; project names and
/// session prefixes are never reverse-engineered.
pub fn record_team_member_projection<T: Serialize>(
    state_dir: &Path,
    session: &str,
    value: &T,
    idempotency_suffix: &str,
    kind: &str,
    provider: &str,
) -> Result<Option<crate::done::ProjectionProvenance>> {
    const CAS_ATTEMPTS: usize = 8;
    let Some((run, member)) = TeamRunState::find_member(state_dir, session)? else {
        return Ok(None);
    };
    let ledger = crate::mission_ledger::MissionLedger::open(
        crate::oracle_lifecycle::mission_ledger_path(state_dir),
    )?;
    let payload = serde_json::to_value(value)?;
    for _ in 0..CAS_ATTEMPTS {
        let mission = ledger
            .mission(&run.mission_id)?
            .ok_or_else(|| anyhow::anyhow!("team mission projection disappeared"))?;
        if mission.state != crate::mission::MissionState::Running {
            anyhow::bail!(
                "team completion refused: mission {} is {:?}, not Running",
                run.mission_id.as_str(),
                mission.state
            );
        }
        let plan = ledger
            .active_plan(&run.mission_id)?
            .ok_or_else(|| anyhow::anyhow!("team active plan disappeared"))?;
        if plan.revision != member.plan_revision
            || !plan
                .tasks
                .iter()
                .any(|task| task.task_id.as_str() == member.task_id)
        {
            anyhow::bail!("team member completion differs from its active plan binding");
        }
        let attempt = ledger
            .task_attempt(&member.attempt_id)?
            .ok_or_else(|| anyhow::anyhow!("team task attempt disappeared"))?;
        if attempt.mission_id != run.mission_id
            || attempt.task_id != member.task_id
            || attempt.plan_revision != member.plan_revision
        {
            anyhow::bail!("team task attempt differs from its persisted run binding");
        }
        let running_actor_matches = ledger.events(&run.mission_id)?.iter().any(|event| {
            event.actor == session
                && event.resulting_task_attempt.as_ref().is_some_and(|result| {
                    result.attempt_id == member.attempt_id
                        && result.state == crate::mission::TaskAttemptState::Running
                })
        });
        if !running_actor_matches {
            anyhow::bail!("team member did not author its exact Running transition");
        }
        let mut event = crate::mission_ledger::AppendEvent::new(
            run.mission_id.clone(),
            mission.version,
            format!("{kind}:{session}:{idempotency_suffix}"),
            session,
            kind,
        );
        event.provider = Some(provider.to_string());
        event.correlation_id = Some(run.team_session.clone());
        event.payload = payload.clone();
        match attempt.state {
            crate::mission::TaskAttemptState::Running => {
                event.task_attempt = Some(crate::mission_ledger::TaskAttemptMutation {
                    task_id: member.task_id.clone(),
                    attempt_id: member.attempt_id.clone(),
                    plan_revision: member.plan_revision,
                    expected_version: attempt.version,
                    next_state: crate::mission::TaskAttemptState::CandidateDone,
                });
            }
            crate::mission::TaskAttemptState::CandidateDone => {}
            other => {
                anyhow::bail!("team completion refused from non-candidate attempt state {other:?}")
            }
        }
        match ledger.append(event) {
            Ok(appended) => {
                return Ok(Some(crate::done::ProjectionProvenance {
                    source: "mission-engine-v3.sqlite3".to_string(),
                    event_id: appended.event.event_id,
                    event_sequence: appended.event.sequence,
                    mission_version: appended.projection.version,
                    projection_hash: appended.projection.projection_hash,
                }));
            }
            Err(crate::mission_ledger::LedgerError::VersionConflict { .. }) => continue,
            Err(error) => return Err(error.into()),
        }
    }
    anyhow::bail!(
        "team member completion did not converge after {CAS_ATTEMPTS} compare-and-set attempts"
    )
}

#[derive(Debug, Default)]
pub struct TeamReconcileReport {
    pub actions: Vec<String>,
    pub completed_runs: Vec<String>,
    pub failed_runs: Vec<String>,
}

fn authoritative_attempt_for_member(
    ledger: &crate::mission_ledger::MissionLedger,
    run: &TeamRunState,
    member: &TeamRunMember,
) -> Result<crate::orchestration::AuthoritativeTaskAttempt> {
    let projection = ledger
        .task_attempt(&member.attempt_id)?
        .ok_or_else(|| anyhow::anyhow!("team member attempt {} is missing", member.attempt_id))?;
    if projection.mission_id != run.mission_id
        || projection.task_id != member.task_id
        || projection.plan_revision != member.plan_revision
    {
        anyhow::bail!("team member attempt differs from its persisted binding");
    }
    Ok(crate::orchestration::AuthoritativeTaskAttempt {
        mission_id: run.mission_id.clone(),
        task_id: member.task_id.clone(),
        attempt_id: member.attempt_id.clone(),
        plan_revision: member.plan_revision,
        owner: Some(member.session.clone()),
        leases: ledger.active_leases_for_attempt(
            &run.mission_id,
            &member.task_id,
            &member.attempt_id,
        )?,
        scope_receipt: None,
    })
}

/// Independently settle persisted team candidates from patrol. A team command
/// returns immediately after dispatch, so this is the durable resume executor
/// that converts member narration into verified V3 outcomes and closes the
/// shared rmux session only after every member is accepted or the run fails.
pub async fn reconcile_team_runs(
    state_dir: &Path,
    session_mgr: &SessionManager,
    runs: &[TeamRunState],
    live_sessions: &HashSet<String>,
) -> Result<TeamReconcileReport> {
    const ABANDONED_RUN_GRACE_SECS: i64 = 300;
    let mut report = TeamReconcileReport::default();
    for run in runs {
        run.validate()?;
        let ledger = crate::mission_ledger::MissionLedger::open(
            crate::oracle_lifecycle::mission_ledger_path(state_dir),
        )?;
        let mut accepted = 0usize;
        let mut failed = false;
        let mut has_signal = false;

        for member in &run.members {
            let Some(done) = crate::done::DoneSignal::read(state_dir, &member.session)? else {
                continue;
            };
            has_signal = true;
            let earliest = run.created_at - chrono::Duration::minutes(5);
            let latest = chrono::Utc::now() + chrono::Duration::minutes(5);
            if done.session != member.session
                || done.finished_at < earliest
                || done.finished_at > latest
            {
                anyhow::bail!(
                    "team member {} has stale or mismatched completion",
                    member.session
                );
            }
            let mut projection = ledger
                .task_attempt(&member.attempt_id)?
                .ok_or_else(|| anyhow::anyhow!("team member attempt disappeared"))?;
            match done.status {
                crate::done::DoneStatus::DoneClean => {
                    if matches!(
                        projection.state,
                        crate::mission::TaskAttemptState::CandidateDone
                            | crate::mission::TaskAttemptState::Verifying
                    ) {
                        let outcome = crate::orchestration::verify_and_finalize_candidate(
                            &ledger,
                            &run.mission_id,
                            &member.task_id,
                            &member.attempt_id,
                            member.plan_revision,
                            &member.session,
                            &done,
                            &run.working_dir,
                        )?;
                        projection = ledger
                            .task_attempt(&member.attempt_id)?
                            .ok_or_else(|| anyhow::anyhow!("finalized team attempt disappeared"))?;
                        if !outcome.accepted {
                            report.actions.push(format!(
                                "Team member {} failed independent verification: {}",
                                member.session,
                                outcome.verification.failures.join("; ")
                            ));
                        }
                    }
                }
                crate::done::DoneStatus::Failed
                | crate::done::DoneStatus::Blocked
                | crate::done::DoneStatus::Pending => {
                    let target = if done.status == crate::done::DoneStatus::Failed {
                        crate::mission::TaskAttemptState::Failed
                    } else {
                        crate::mission::TaskAttemptState::Blocked
                    };
                    crate::orchestration::finalize_nonclean_candidate(
                        &ledger,
                        &run.mission_id,
                        &member.task_id,
                        &member.attempt_id,
                        member.plan_revision,
                        &member.session,
                        &done,
                        target,
                    )?;
                    projection = ledger
                        .task_attempt(&member.attempt_id)?
                        .ok_or_else(|| anyhow::anyhow!("settled team attempt disappeared"))?;
                }
            }

            if projection.state == crate::mission::TaskAttemptState::Accepted {
                let authority = authoritative_attempt_for_member(&ledger, run, member)?;
                crate::orchestration::release_authoritative_scopes(&ledger, state_dir, &authority)?;
                accepted += 1;
            } else if matches!(
                projection.state,
                crate::mission::TaskAttemptState::CorrectionRequired
                    | crate::mission::TaskAttemptState::Blocked
                    | crate::mission::TaskAttemptState::Failed
                    | crate::mission::TaskAttemptState::Cancelled
            ) {
                failed = true;
            }
        }

        let abandoned = !live_sessions.contains(&run.team_session)
            && !has_signal
            && (chrono::Utc::now() - run.created_at).num_seconds() >= ABANDONED_RUN_GRACE_SECS;
        if accepted == run.members.len() {
            for next in [
                crate::mission::MissionState::Verifying,
                crate::mission::MissionState::Accepted,
                crate::mission::MissionState::Reporting,
                crate::mission::MissionState::Delivered,
            ] {
                crate::orchestration::transition_authoritative_mission(
                    &ledger,
                    &run.mission_id,
                    next,
                    "omega-team-patrol",
                )?;
            }
            if live_sessions.contains(&run.team_session) {
                session_mgr.kill_session(&run.team_session).await?;
            }
            run.remove(state_dir)?;
            report.completed_runs.push(run.team_session.clone());
            report.actions.push(format!(
                "Team {} independently accepted and delivered",
                run.team_session
            ));
        } else if failed || abandoned {
            let authority = crate::orchestration::AuthoritativeExecution {
                mission_id: run.mission_id.clone(),
                plan: ledger
                    .active_plan(&run.mission_id)?
                    .ok_or_else(|| anyhow::anyhow!("team active plan disappeared"))?,
                attempts: run
                    .members
                    .iter()
                    .map(|member| authoritative_attempt_for_member(&ledger, run, member))
                    .collect::<Result<Vec<_>>>()?,
            };
            abort_team_authority(&ledger, state_dir, &authority);
            if live_sessions.contains(&run.team_session) {
                session_mgr.kill_session(&run.team_session).await?;
            }
            run.remove(state_dir)?;
            report.failed_runs.push(run.team_session.clone());
            report.actions.push(format!(
                "Team {} failed closed{}",
                run.team_session,
                if abandoned {
                    " after its rmux session disappeared"
                } else {
                    ""
                }
            ));
        }
    }
    Ok(report)
}

/// Mint a non-reusable rmux name for a team run. Entropy, rather than
/// wall-clock/PID material, makes delayed done files and cleanup unambiguous.
pub fn generate_team_session_name(project: &str) -> Result<String> {
    crate::scope::validate_session_identity(project)?;
    let canonical = crate::session::sanitize_session_name(project);
    if canonical != project {
        anyhow::bail!("team project identity `{project}` is not canonical; use `{canonical}`");
    }
    let mut entropy = [0_u8; 32];
    getrandom::fill(&mut entropy).map_err(|error| {
        anyhow::anyhow!("reading OS entropy for team generation failed: {error}")
    })?;
    let digest = blake3::hash(&entropy).to_hex();
    let suffix = &digest.as_str()[..16];
    let available = crate::session::MAX_SESSION_NAME_LEN - "Team--".len() - suffix.len();
    let project_prefix = &project[..project.len().min(available)];
    let name = format!("Team-{project_prefix}-{suffix}");
    validate_team_session_name(&name)?;
    Ok(name)
}

#[derive(Debug, Clone)]
pub struct PreparedTeamAuthority {
    pub mission: crate::mission::Mission,
    pub legacy_plan: crate::mission::Plan,
    pub authority: crate::orchestration::AuthoritativeExecution,
}

/// Freeze a team into the same V3 mission/plan/attempt contracts used by the
/// orchestrator. This function performs no rmux or provider effect, making it
/// the fail-closed preparation surface for CLI and tests.
pub fn prepare_team_authority(
    state_dir: &Path,
    config: &TeamConfig,
) -> Result<PreparedTeamAuthority> {
    if config.members.is_empty() {
        anyhow::bail!("team has no members");
    }
    let agent = resolve_team_agent(&config.agent_command)?;
    validate_team_session_name(&config.session_name)?;
    let requested_working_dir = PathBuf::from(&config.working_dir);
    let metadata = std::fs::metadata(&requested_working_dir).with_context(|| {
        format!(
            "team working directory {} is not accessible",
            requested_working_dir.display()
        )
    })?;
    if !metadata.is_dir() {
        anyhow::bail!(
            "team working directory {} is not a directory",
            requested_working_dir.display()
        );
    }
    let working_dir = std::fs::canonicalize(&requested_working_dir).with_context(|| {
        format!(
            "canonicalizing team working directory {}",
            requested_working_dir.display()
        )
    })?;

    let mut member_ids = HashSet::new();
    let mut owners = HashSet::new();
    let mut normalized_scopes = Vec::with_capacity(config.members.len());
    for member in &config.members {
        crate::scope::validate_session_identity(&member.name)
            .with_context(|| format!("invalid team member identity `{}`", member.name))?;
        let member_id = sanitize_identity(&member.name);
        if !member_ids.insert(member_id.clone()) {
            anyhow::bail!(
                "team member identities collide after canonicalization: `{}`",
                member.name
            );
        }
        let owner = format!("{}-{}", config.session_name, member.name);
        crate::scope::validate_session_identity(&owner)
            .with_context(|| format!("invalid team authority owner `{owner}`"))?;
        if !owners.insert(owner) {
            anyhow::bail!("duplicate team authority owner for `{}`", member.name);
        }
        if member.files_owned.is_empty() && !is_explicit_read_only_role(&member.role) {
            anyhow::bail!(
                "team member `{}` has writable role `{}` but no files_owned scope; declare scope or use an explicit read-only role",
                member.name,
                member.role
            );
        }
        normalized_scopes.push(
            crate::scope::validate_scope_selectors(member.files_owned.clone())
                .with_context(|| format!("invalid scope for team member `{}`", member.name))?,
        );
    }
    for left in 0..normalized_scopes.len() {
        for right in (left + 1)..normalized_scopes.len() {
            if normalized_scopes[left].iter().any(|left_selector| {
                normalized_scopes[right].iter().any(|right_selector| {
                    crate::scope::selectors_overlap(left_selector, right_selector)
                })
            }) {
                anyhow::bail!(
                    "team members `{}` and `{}` declare overlapping writable scopes",
                    config.members[left].name,
                    config.members[right].name
                );
            }
        }
    }
    let mission = crate::mission::Mission::new(
        &config.project,
        format!(
            "Team {}: {}",
            config.session_name,
            config
                .members
                .iter()
                .map(|member| format!("{} ({})", member.name, member.role))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        working_dir,
    );
    let tasks = config
        .members
        .iter()
        .zip(&normalized_scopes)
        .enumerate()
        .map(|(index, (member, scope))| crate::mission::Task {
            id: format!("team-{}-{}", index + 1, sanitize_identity(&member.name)),
            name: member.name.clone(),
            prompt: member.prompt.clone(),
            files_owned: scope.clone(),
            depends_on: Vec::new(),
            agent: agent.name().to_string(),
            estimated_minutes: 60,
        })
        .collect::<Vec<_>>();
    let legacy_plan = crate::mission::Plan {
        mission_id: mission.id.clone(),
        complexity: crate::routing::Complexity::Complex,
        strategy: crate::mission::PlanStrategy::Team,
        tasks,
        created_at: chrono::Utc::now(),
    };
    let ledger = crate::mission_ledger::MissionLedger::open(
        crate::oracle_lifecycle::mission_ledger_path(state_dir),
    )?;
    let authority = crate::orchestration::prepare_authoritative_execution(
        &ledger,
        &mission,
        &legacy_plan,
        "omega-team",
        vec!["independent_verification".to_string()],
    )?;
    Ok(PreparedTeamAuthority {
        mission,
        legacy_plan,
        authority,
    })
}

fn validate_team_session_name(name: &str) -> Result<()> {
    crate::scope::validate_session_identity(name)?;
    let canonical = crate::session::sanitize_session_name(name);
    if canonical != name {
        anyhow::bail!(
            "team session name `{name}` is not canonical; use `{canonical}` so scope, done and rmux identities stay exact"
        );
    }
    Ok(())
}

pub struct TeamSpawner<'a> {
    session_mgr: &'a SessionManager,
    state_dir: Result<PathBuf, String>,
}

impl<'a> TeamSpawner<'a> {
    pub fn new(session_mgr: &'a SessionManager) -> Self {
        Self {
            session_mgr,
            state_dir: crate::config::OmegaConfig::load()
                .map(|config| config.state_dir)
                .map_err(|error| error.to_string()),
        }
    }

    /// Pin the exact state directory resolved by the caller. CLI surfaces
    /// should use this so tests/relocated installs cannot drift from the
    /// ledger used by the rest of the command.
    pub fn with_state_dir(mut self, state_dir: impl Into<PathBuf>) -> Self {
        self.state_dir = Ok(state_dir.into());
        self
    }

    pub async fn spawn_team(&self, config: &TeamConfig) -> Result<Vec<String>> {
        let state_dir = self.state_dir.as_ref().map_err(|error| {
            anyhow::anyhow!("cannot resolve authoritative OmegaOS state directory: {error}")
        })?;
        let working_dir = PathBuf::from(&config.working_dir);
        let agent = resolve_team_agent(&config.agent_command)?;
        let providers = crate::providers::ProvidersConfig::try_load()
            .context("loading one immutable provider snapshot for the team")?;
        let prepared = prepare_team_authority(state_dir, config)?;
        let mission = prepared.mission;
        let legacy_plan = prepared.legacy_plan;
        let mut authority = prepared.authority;
        let ledger = crate::mission_ledger::MissionLedger::open(
            crate::oracle_lifecycle::mission_ledger_path(state_dir),
        )?;
        crate::orchestration::transition_authoritative_mission(
            &ledger,
            &mission.id,
            crate::mission::MissionState::Running,
            "omega-team",
        )?;

        for (member, task) in config.members.iter().zip(&legacy_plan.tasks) {
            let owner = format!("{}-{}", config.session_name, member.name);
            let attempt = authority.attempt_mut(&task.id)?;
            if let Err(error) = crate::orchestration::claim_authoritative_scopes(
                &ledger,
                state_dir,
                &working_dir,
                attempt,
                &owner,
                &member.files_owned,
                Duration::from_secs(24 * 60 * 60),
            ) {
                abort_team_authority(&ledger, state_dir, &authority);
                return Err(error.context(format!("claiming team scope for {}", member.name)));
            }
        }

        let run = TeamRunState {
            schema_version: TEAM_RUN_SCHEMA_VERSION,
            team_session: config.session_name.clone(),
            project: config.project.clone(),
            mission_id: mission.id.clone(),
            working_dir: mission.working_dir.clone(),
            members: config
                .members
                .iter()
                .zip(&legacy_plan.tasks)
                .map(|(member, task)| {
                    let attempt = authority.attempt(&task.id)?;
                    Ok(TeamRunMember {
                        session: format!("{}-{}", config.session_name, member.name),
                        task_id: task.id.clone(),
                        attempt_id: attempt.attempt_id.clone(),
                        plan_revision: attempt.plan_revision,
                    })
                })
                .collect::<Result<Vec<_>>>()?,
            created_at: chrono::Utc::now(),
        };
        if let Err(error) = run.write(state_dir) {
            abort_team_authority(&ledger, state_dir, &authority);
            return Err(error).context("persisting team run authority before rmux launch");
        }

        // Freeze every provider launch before the first rmux side effect. The
        // command is non-secret; provider credentials remain in the structured
        // environment carried by AgentLaunch.
        let mut launches = Vec::with_capacity(config.members.len());
        for (member, task) in config.members.iter().zip(&legacy_plan.tasks) {
            let attempt = authority.attempt(&task.id)?;
            let fences = attempt
                .leases
                .iter()
                .map(|lease| format!("{}#{}", lease.resource_key, lease.fencing_token))
                .collect::<Vec<_>>()
                .join(", ");
            let agent_prompt =
                build_team_member_prompt(config, &mission, member, task, attempt, &fences, agent)?;
            let launch = match agent.launch_with_providers(
                Some(&agent_prompt),
                crate::agents::LaunchOptions::default(),
                &providers,
            ) {
                Ok(launch) => launch,
                Err(error) => {
                    abort_team_authority(&ledger, state_dir, &authority);
                    if let Err(remove_error) = run.remove(state_dir) {
                        tracing::error!(error = %remove_error, "failed to remove rolled-back team run");
                    }
                    return Err(error).context(format!(
                        "building typed provider launch for team member {}",
                        member.name
                    ));
                }
            };
            launches.push(launch);
        }

        let session = match self
            .session_mgr
            .create_recorded_agent_session_create_only(
                &config.session_name,
                Some(&config.working_dir),
                agent,
                launches[0].clone(),
            )
            .await
        {
            Ok(session) => session,
            Err(error) => {
                abort_team_authority(&ledger, state_dir, &authority);
                if let Err(remove_error) = run.remove(state_dir) {
                    tracing::error!(error = %remove_error, "failed to remove rolled-back team run");
                }
                return Err(error).context("Failed to create team session");
            }
        };

        let first_pane = session.pane(0, 0);
        let mut pane_names = Vec::new();

        for (i, ((member, task), launch)) in config
            .members
            .iter()
            .zip(&legacy_plan.tasks)
            .zip(&launches)
            .enumerate()
        {
            let attempt = authority.attempt(&task.id)?;

            let pane_result = if i == 0 {
                Ok(())
            } else {
                let direction = if i % 2 == 1 {
                    SplitDirection::Right
                } else {
                    SplitDirection::Down
                };
                let mut split = first_pane.split_with(direction).shell(launch.command());
                for (key, value) in launch.environment() {
                    split = split.env(key, value);
                }
                match split.await {
                    Ok(new_pane) => new_pane.set_title(&member.name).await,
                    Err(error) => Err(error),
                }
            };
            if let Err(error) = pane_result {
                let _ = self.session_mgr.kill_session(&config.session_name).await;
                abort_team_authority(&ledger, state_dir, &authority);
                if let Err(remove_error) = run.remove(state_dir) {
                    tracing::error!(error = %remove_error, "failed to remove rolled-back team run");
                }
                return Err(error).context(format!("starting team member {}", member.name));
            }

            if let Err(error) = crate::orchestration::transition_authoritative_attempt(
                &ledger,
                attempt,
                crate::mission::TaskAttemptState::Running,
                &format!("{}-{}", config.session_name, member.name),
            ) {
                let _ = self.session_mgr.kill_session(&config.session_name).await;
                abort_team_authority(&ledger, state_dir, &authority);
                if let Err(remove_error) = run.remove(state_dir) {
                    tracing::error!(error = %remove_error, "failed to remove rolled-back team run");
                }
                return Err(error).context(format!(
                    "team member {} spawned without a V3 running transition",
                    member.name
                ));
            }

            pane_names.push(format!("{}-{}", config.session_name, member.name));
        }

        // Even out the grid. The alternating Right/Down splits above produce a
        // lopsided binary-tree layout (pane 0 stays huge, later panes get
        // cramped) — on a client smaller than the spawn size that reads as
        // "empty space + agents you have to scroll to find". `tiled` arranges
        // every member in an even grid that reflows proportionally when the
        // window resizes to the attaching client, so the grid stays balanced
        // at attach. Best-effort: a layout hiccup must never fail the spawn.
        if config.members.len() > 1 {
            match tokio::process::Command::new("rmux")
                .args(["select-layout", "-t", &config.session_name, "tiled"])
                .output()
                .await
            {
                Ok(o) if !o.status.success() => tracing::warn!(
                    team = %config.session_name,
                    stderr = %String::from_utf8_lossy(&o.stderr),
                    "select-layout tiled failed (panes left in zigzag layout)"
                ),
                Err(e) => tracing::warn!(
                    team = %config.session_name,
                    error = %e,
                    "could not run rmux select-layout (panes left in zigzag layout)"
                ),
                _ => {}
            }
        }

        tracing::info!(
            team = %config.session_name,
            members = config.members.len(),
            "Team spawned"
        );

        Ok(pane_names)
    }
}

fn abort_team_authority(
    ledger: &crate::mission_ledger::MissionLedger,
    state_dir: &Path,
    authority: &crate::orchestration::AuthoritativeExecution,
) {
    for attempt in &authority.attempts {
        if let Ok(Some(projection)) = ledger.task_attempt(&attempt.attempt_id) {
            let actor = attempt.owner.as_deref().unwrap_or("omega-team");
            let result = match projection.state {
                crate::mission::TaskAttemptState::Queued
                | crate::mission::TaskAttemptState::Running
                | crate::mission::TaskAttemptState::CorrectionRequired
                | crate::mission::TaskAttemptState::Blocked => {
                    crate::orchestration::transition_authoritative_attempt(
                        ledger,
                        attempt,
                        crate::mission::TaskAttemptState::Cancelled,
                        actor,
                    )
                }
                crate::mission::TaskAttemptState::CandidateDone => {
                    crate::orchestration::transition_authoritative_attempt(
                        ledger,
                        attempt,
                        crate::mission::TaskAttemptState::Verifying,
                        "omega-team-rollback",
                    )
                    .and_then(|()| {
                        crate::orchestration::transition_authoritative_attempt(
                            ledger,
                            attempt,
                            crate::mission::TaskAttemptState::Failed,
                            "omega-team-rollback",
                        )
                    })
                }
                crate::mission::TaskAttemptState::Verifying => {
                    crate::orchestration::transition_authoritative_attempt(
                        ledger,
                        attempt,
                        crate::mission::TaskAttemptState::Failed,
                        "omega-team-rollback",
                    )
                }
                crate::mission::TaskAttemptState::Accepted
                | crate::mission::TaskAttemptState::Failed
                | crate::mission::TaskAttemptState::Cancelled => Ok(()),
            };
            if let Err(error) = result {
                tracing::error!(
                    attempt = %attempt.attempt_id,
                    error = %error,
                    "failed to settle team attempt during rollback"
                );
            }
        }
        if let Err(error) =
            crate::orchestration::release_authoritative_scopes(ledger, state_dir, attempt)
        {
            tracing::error!(
                attempt = %attempt.attempt_id,
                error = %error,
                "failed to release every team scope during rollback"
            );
        }
    }
    if let Ok(Some(projection)) = ledger.mission(&authority.mission_id) {
        if projection.state == crate::mission::MissionState::Running {
            if let Err(error) = crate::orchestration::transition_authoritative_mission(
                ledger,
                &authority.mission_id,
                crate::mission::MissionState::Verifying,
                "omega-team",
            ) {
                tracing::error!(error = %error, "failed to enter team rollback verification state");
            }
        }
    }
    if let Err(error) = crate::orchestration::transition_authoritative_mission(
        ledger,
        &authority.mission_id,
        crate::mission::MissionState::Failed,
        "omega-team",
    ) {
        tracing::error!(error = %error, "failed to mark rolled-back team mission failed");
    }
}

fn sanitize_identity(raw: &str) -> String {
    let sanitized = raw
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if sanitized.is_empty() {
        "member".to_string()
    } else {
        sanitized
    }
}

fn resolve_team_agent(raw: &str) -> Result<crate::agents::Agent> {
    let agent = crate::agents::Agent::from_name(raw).ok_or_else(|| {
        anyhow::anyhow!(
            "unknown team agent `{raw}`; refusing an untyped or injection-shaped launch command"
        )
    })?;
    if agent == crate::agents::Agent::Shell {
        anyhow::bail!("shell is not an agent provider and cannot own a team task");
    }
    Ok(agent)
}

fn is_explicit_read_only_role(role: &str) -> bool {
    matches!(
        role.trim().to_ascii_lowercase().as_str(),
        "read-only" | "readonly" | "reviewer" | "verifier" | "researcher" | "oracle"
    )
}

fn build_team_member_prompt(
    config: &TeamConfig,
    mission: &crate::mission::Mission,
    member: &TeamMember,
    task: &crate::mission::Task,
    attempt: &crate::orchestration::AuthoritativeTaskAttempt,
    fences: &str,
    agent: crate::agents::Agent,
) -> Result<String> {
    let mut prompt = format!(
        "[DISPATCHED] Team member: {} ({})\n\
         Third Law: decide and proceed, never wait.\n\n\
         {}\n\n\
         Mission ID: {}\n\
         Task ID: {}\n\
         Attempt ID: {}\n\
         Plan revision: {}\n\
         Fenced scopes: {}\n\
         Files owned: {}\n\
         When done: omega done {}-{} done_clean \"<summary>\"",
        member.name,
        member.role,
        member.prompt,
        mission.id.as_str(),
        task.id,
        attempt.attempt_id,
        attempt.plan_revision,
        if fences.is_empty() { "none" } else { fences },
        if member.files_owned.is_empty() {
            "none (read-only)".to_string()
        } else {
            member.files_owned.join(", ")
        },
        config.session_name,
        member.name,
    );
    let compiled = crate::orchestration::policy_context_for_agent(
        crate::rules::RuleScope::Worker,
        &prompt,
        agent,
    );
    if !compiled.is_empty() {
        prompt.push_str("\n\n");
        prompt.push_str(&compiled);
    }
    Ok(prompt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn team_preparation_cannot_exist_without_plan_and_attempt_contracts() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config = TeamConfig {
            project: "OmegaOS".to_string(),
            session_name: "Team-OmegaOS".to_string(),
            working_dir: tmp.path().to_string_lossy().to_string(),
            agent_command: "codex".to_string(),
            members: vec![
                TeamMember {
                    name: "core".to_string(),
                    role: "worker".to_string(),
                    prompt: "Implement core".to_string(),
                    files_owned: vec!["src/core.rs".to_string()],
                },
                TeamMember {
                    name: "tests".to_string(),
                    role: "verifier".to_string(),
                    prompt: "Verify core".to_string(),
                    files_owned: vec!["tests/core.rs".to_string()],
                },
            ],
        };

        let prepared = prepare_team_authority(tmp.path(), &config).unwrap();
        let ledger = crate::mission_ledger::MissionLedger::open(
            crate::oracle_lifecycle::mission_ledger_path(tmp.path()),
        )
        .unwrap();
        let mission = ledger.mission(&prepared.mission.id).unwrap().unwrap();
        assert_eq!(mission.state, crate::mission::MissionState::Planned);
        let plan = ledger.active_plan(&prepared.mission.id).unwrap().unwrap();
        assert_eq!(plan.tasks.len(), config.members.len());
        assert_eq!(prepared.authority.attempts.len(), config.members.len());
        for attempt in &prepared.authority.attempts {
            let projection = ledger.task_attempt(&attempt.attempt_id).unwrap().unwrap();
            assert_eq!(projection.state, crate::mission::TaskAttemptState::Queued);
            assert_eq!(projection.plan_revision, plan.revision);
        }
    }

    #[test]
    fn empty_team_is_rejected_without_creating_a_mission() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config = TeamConfig {
            project: "OmegaOS".to_string(),
            session_name: "Team-OmegaOS".to_string(),
            working_dir: tmp.path().to_string_lossy().to_string(),
            agent_command: "codex".to_string(),
            members: Vec::new(),
        };
        assert!(prepare_team_authority(tmp.path(), &config).is_err());
        assert!(!crate::oracle_lifecycle::mission_ledger_path(tmp.path()).exists());
    }

    #[test]
    fn injection_shaped_or_unknown_agent_is_rejected_before_authority_creation() {
        for raw in ["codex; touch /tmp/pwned", "unknown-provider", "shell"] {
            let tmp = tempfile::TempDir::new().unwrap();
            let config = TeamConfig {
                project: "OmegaOS".to_string(),
                session_name: "Team-OmegaOS".to_string(),
                working_dir: "/tmp/OmegaOS".to_string(),
                agent_command: raw.to_string(),
                members: vec![TeamMember {
                    name: "core".to_string(),
                    role: "worker".to_string(),
                    prompt: "Implement core".to_string(),
                    files_owned: vec!["src/core.rs".to_string()],
                }],
            };
            assert!(prepare_team_authority(tmp.path(), &config).is_err());
            assert!(!crate::oracle_lifecycle::mission_ledger_path(tmp.path()).exists());
        }
    }

    #[test]
    fn writable_team_member_without_scope_is_rejected() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config = TeamConfig {
            project: "OmegaOS".to_string(),
            session_name: "Team-OmegaOS".to_string(),
            working_dir: tmp.path().to_string_lossy().to_string(),
            agent_command: "codex".to_string(),
            members: vec![TeamMember {
                name: "core".to_string(),
                role: "worker".to_string(),
                prompt: "Implement core".to_string(),
                files_owned: Vec::new(),
            }],
        };
        assert!(prepare_team_authority(tmp.path(), &config).is_err());
        assert!(!crate::oracle_lifecycle::mission_ledger_path(tmp.path()).exists());

        let mut read_only = config;
        read_only.members[0].role = "verifier".to_string();
        assert!(prepare_team_authority(tmp.path(), &read_only).is_ok());
    }

    #[test]
    fn unsafe_identity_scope_or_overlap_is_rejected_before_ledger_creation() {
        let cases = [
            TeamConfig {
                project: "OmegaOS".to_string(),
                session_name: "Team OmegaOS".to_string(),
                working_dir: String::new(),
                agent_command: "codex".to_string(),
                members: vec![TeamMember {
                    name: "reviewer".to_string(),
                    role: "reviewer".to_string(),
                    prompt: "Review".to_string(),
                    files_owned: Vec::new(),
                }],
            },
            TeamConfig {
                project: "OmegaOS".to_string(),
                session_name: "Team-OmegaOS".to_string(),
                working_dir: String::new(),
                agent_command: "codex".to_string(),
                members: vec![TeamMember {
                    name: "core/member".to_string(),
                    role: "worker".to_string(),
                    prompt: "Build".to_string(),
                    files_owned: vec!["src/core.rs".to_string()],
                }],
            },
            TeamConfig {
                project: "OmegaOS".to_string(),
                session_name: "Team-OmegaOS".to_string(),
                working_dir: String::new(),
                agent_command: "codex".to_string(),
                members: vec![TeamMember {
                    name: "core".to_string(),
                    role: "worker".to_string(),
                    prompt: "Build".to_string(),
                    files_owned: vec!["../escape".to_string()],
                }],
            },
            TeamConfig {
                project: "OmegaOS".to_string(),
                session_name: "Team-OmegaOS".to_string(),
                working_dir: String::new(),
                agent_command: "codex".to_string(),
                members: vec![
                    TeamMember {
                        name: "core".to_string(),
                        role: "worker".to_string(),
                        prompt: "Build core".to_string(),
                        files_owned: vec!["src".to_string()],
                    },
                    TeamMember {
                        name: "tests".to_string(),
                        role: "worker".to_string(),
                        prompt: "Build tests".to_string(),
                        files_owned: vec!["src/lib.rs".to_string()],
                    },
                ],
            },
        ];

        for mut config in cases {
            let state = tempfile::TempDir::new().unwrap();
            config.working_dir = state.path().to_string_lossy().to_string();
            assert!(prepare_team_authority(state.path(), &config).is_err());
            assert!(
                !crate::oracle_lifecycle::mission_ledger_path(state.path()).exists(),
                "invalid team authority must fail before ledger creation"
            );
        }
    }

    #[test]
    fn team_prompt_uses_typed_provider_launch_and_canonical_rule_funnel() {
        let config = TeamConfig {
            project: "OmegaOS".to_string(),
            session_name: "Team-OmegaOS".to_string(),
            working_dir: "/tmp/OmegaOS".to_string(),
            agent_command: "codex".to_string(),
            members: vec![TeamMember {
                name: "core".to_string(),
                role: "worker".to_string(),
                prompt: "Implement core".to_string(),
                files_owned: vec!["src/core.rs".to_string()],
            }],
        };
        let mission =
            crate::mission::Mission::new("OmegaOS", "team", PathBuf::from("/tmp/OmegaOS"));
        let task = crate::mission::Task {
            id: "team-1-core".to_string(),
            name: "core".to_string(),
            prompt: "Implement core".to_string(),
            files_owned: vec!["src/core.rs".to_string()],
            depends_on: Vec::new(),
            agent: "codex".to_string(),
            estimated_minutes: 10,
        };
        let attempt = crate::orchestration::AuthoritativeTaskAttempt {
            mission_id: mission.id.clone(),
            task_id: task.id.clone(),
            attempt_id: "attempt-team-core-1".to_string(),
            plan_revision: 1,
            owner: Some("Team-OmegaOS-core".to_string()),
            leases: Vec::new(),
            scope_receipt: None,
        };
        let prompt = build_team_member_prompt(
            &config,
            &mission,
            &config.members[0],
            &task,
            &attempt,
            "",
            crate::agents::Agent::Codex,
        )
        .unwrap();
        assert!(prompt.contains("[L0]"), "worker rules were not injected");
        assert!(prompt.contains("Attempt ID: attempt-team-core-1"));
        assert!(prompt.contains("[R-GOAL]"));
        assert!(prompt.contains("[R-MODEL]"));
        let command = resolve_team_agent("codex")
            .unwrap()
            .launch_command(Some(&prompt));
        assert!(command.contains("codex"));
        assert!(command.contains("--no-alt-screen"));
        assert!(!command.starts_with("codex -p "));
    }

    #[test]
    fn team_prompt_compiles_provider_neutral_goal_and_model_rules() {
        let config = TeamConfig {
            project: "OmegaOS".to_string(),
            session_name: "Team-OmegaOS".to_string(),
            working_dir: "/tmp/OmegaOS".to_string(),
            agent_command: "claude".to_string(),
            members: vec![TeamMember {
                name: "core".to_string(),
                role: "worker".to_string(),
                prompt: "Implement core".to_string(),
                files_owned: vec!["src/core.rs".to_string()],
            }],
        };
        let mission =
            crate::mission::Mission::new("OmegaOS", "team", PathBuf::from("/tmp/OmegaOS"));
        let task = crate::mission::Task {
            id: "team-1-core".to_string(),
            name: "core".to_string(),
            prompt: "Implement core".to_string(),
            files_owned: vec!["src/core.rs".to_string()],
            depends_on: Vec::new(),
            agent: "claude".to_string(),
            estimated_minutes: 10,
        };
        let attempt = crate::orchestration::AuthoritativeTaskAttempt {
            mission_id: mission.id.clone(),
            task_id: task.id.clone(),
            attempt_id: "attempt-team-core-1".to_string(),
            plan_revision: 1,
            owner: Some("Team-OmegaOS-core".to_string()),
            leases: Vec::new(),
            scope_receipt: None,
        };

        for agent in [
            crate::agents::Agent::Claude,
            crate::agents::Agent::Codex,
            crate::agents::Agent::Gemini,
        ] {
            let prompt = build_team_member_prompt(
                &config,
                &mission,
                &config.members[0],
                &task,
                &attempt,
                "",
                agent,
            )
            .unwrap();
            assert!(prompt.contains("[R-GOAL]"));
            assert!(prompt.contains("[R-MODEL]"));
        }
    }

    #[test]
    fn generated_team_sessions_are_unique_canonical_and_bounded() {
        let first = generate_team_session_name("OmegaOS").unwrap();
        let second = generate_team_session_name("OmegaOS").unwrap();
        assert_ne!(first, second);
        assert!(first.starts_with("Team-OmegaOS-"));
        assert!(first.len() <= crate::session::MAX_SESSION_NAME_LEN);
        validate_team_session_name(&first).unwrap();
        assert!(generate_team_session_name("../OmegaOS").is_err());
    }

    #[test]
    fn persisted_team_binding_records_the_exact_member_candidate() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config = TeamConfig {
            project: "OmegaOS".to_string(),
            session_name: "Team-OmegaOS-generation".to_string(),
            working_dir: tmp.path().to_string_lossy().to_string(),
            agent_command: "codex".to_string(),
            members: vec![TeamMember {
                name: "reviewer".to_string(),
                role: "reviewer".to_string(),
                prompt: "Review core".to_string(),
                files_owned: Vec::new(),
            }],
        };
        let mut prepared = prepare_team_authority(tmp.path(), &config).unwrap();
        let ledger = crate::mission_ledger::MissionLedger::open(
            crate::oracle_lifecycle::mission_ledger_path(tmp.path()),
        )
        .unwrap();
        crate::orchestration::transition_authoritative_mission(
            &ledger,
            &prepared.mission.id,
            crate::mission::MissionState::Running,
            "test",
        )
        .unwrap();
        let task = &prepared.legacy_plan.tasks[0];
        let owner = format!("{}-{}", config.session_name, config.members[0].name);
        let attempt = prepared.authority.attempt_mut(&task.id).unwrap();
        crate::orchestration::claim_authoritative_scopes(
            &ledger,
            tmp.path(),
            tmp.path(),
            attempt,
            &owner,
            &[],
            Duration::from_secs(60),
        )
        .unwrap();
        crate::orchestration::transition_authoritative_attempt(
            &ledger,
            attempt,
            crate::mission::TaskAttemptState::Running,
            &owner,
        )
        .unwrap();
        let run = TeamRunState {
            schema_version: TEAM_RUN_SCHEMA_VERSION,
            team_session: config.session_name.clone(),
            project: config.project.clone(),
            mission_id: prepared.mission.id.clone(),
            working_dir: std::fs::canonicalize(tmp.path()).unwrap(),
            members: vec![TeamRunMember {
                session: owner.clone(),
                task_id: task.id.clone(),
                attempt_id: attempt.attempt_id.clone(),
                plan_revision: attempt.plan_revision,
            }],
            created_at: chrono::Utc::now(),
        };
        run.write(tmp.path()).unwrap();
        let found = TeamRunState::find_member(tmp.path(), &owner)
            .unwrap()
            .unwrap();
        assert_eq!(found.0, run);
        assert_eq!(found.1, run.members[0]);

        let done = crate::done::DoneSignal::new(
            &owner,
            crate::done::DoneStatus::DoneClean,
            "review complete",
        );
        let provenance = record_team_member_projection(
            tmp.path(),
            &owner,
            &done,
            "candidate",
            "legacy_worker_completion_candidate",
            "codex",
        )
        .unwrap()
        .unwrap();
        assert_eq!(provenance.source, "mission-engine-v3.sqlite3");
        assert_eq!(
            ledger
                .task_attempt(&attempt.attempt_id)
                .unwrap()
                .unwrap()
                .state,
            crate::mission::TaskAttemptState::CandidateDone
        );
    }

    #[test]
    fn read_only_team_scope_release_is_an_idempotent_no_op() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config = TeamConfig {
            project: "OmegaOS".to_string(),
            session_name: "Team-OmegaOS-readonly".to_string(),
            working_dir: tmp.path().to_string_lossy().to_string(),
            agent_command: "codex".to_string(),
            members: vec![TeamMember {
                name: "reviewer".to_string(),
                role: "reviewer".to_string(),
                prompt: "Review".to_string(),
                files_owned: Vec::new(),
            }],
        };
        let mut prepared = prepare_team_authority(tmp.path(), &config).unwrap();
        let ledger = crate::mission_ledger::MissionLedger::open(
            crate::oracle_lifecycle::mission_ledger_path(tmp.path()),
        )
        .unwrap();
        let task = &prepared.legacy_plan.tasks[0];
        let attempt = prepared.authority.attempt_mut(&task.id).unwrap();
        crate::orchestration::claim_authoritative_scopes(
            &ledger,
            tmp.path(),
            tmp.path(),
            attempt,
            "Team-OmegaOS-readonly-reviewer",
            &[],
            Duration::from_secs(60),
        )
        .unwrap();
        crate::orchestration::release_authoritative_scopes(&ledger, tmp.path(), attempt).unwrap();
        crate::orchestration::release_authoritative_scopes(&ledger, tmp.path(), attempt).unwrap();
    }
}

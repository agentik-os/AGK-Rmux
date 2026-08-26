//! Quality gate — rubric check, multi-grader consensus, Popper falsification,
//! regression detection, token budget, citation enforcement.
//!
//! Implements R-RUBRIC (rubric before execution), R-VERIFY (multi-grader ≥2/3
//! plus adversarial Popper pass), R-BUDGET (token budget), and R-CITE
//! (citation enforcement). Ship verification is the deploy URL → 200 gate.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ── Rubric Types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rubric {
    pub mission: String,
    pub criteria: Vec<RubricCriterion>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RubricCriterion {
    pub id: String,
    pub description: String,
    pub weight: f32,
    pub category: CriterionCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CriterionCategory {
    Functional,
    Quality,
    Performance,
    Security,
}

// ── Grade Types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradeResult {
    pub criterion_id: String,
    pub verdict: GradeVerdict,
    pub confidence: f32,
    pub evidence: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GradeVerdict {
    Satisfied,
    NeedsRevision,
    Unmet,
    Blocked,
}

// ── Gate Result ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResult {
    pub oracle: String,
    pub timestamp: DateTime<Utc>,
    pub rubric_pass: bool,
    pub consensus_pass: bool,
    pub adversarial_pass: bool,
    pub regression_pass: bool,
    pub audit_results: Vec<crate::audit::AuditResult>,
    pub audit_pass: bool,
    pub token_budget_pass: bool,
    pub citation_pass: bool,
    pub overall_pass: bool,
    pub score: f32,
    pub details: GateDetails,
    /// Set only on a HUMAN acceptance (`omega gate <oracle> --accept`): who
    /// signed it off. `None` on every machine-produced result, so a reader can
    /// always tell a graded pass from an accepted one. Additive and defaulted,
    /// so results written before this field still parse.
    #[serde(default)]
    pub accepted_by: Option<String>,
    /// What the approver says they verified. Recorded verbatim beside the name:
    /// an acceptance with no evidence is not an audit trail.
    #[serde(default)]
    pub accepted_evidence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateDetails {
    pub grades: Vec<GradeResult>,
    pub consensus_votes: Vec<ConsensusVote>,
    pub adversarial_challenges: Vec<AdversarialChallenge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusVote {
    pub grader: String,
    pub verdict: GradeVerdict,
    pub confidence: f32,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdversarialChallenge {
    pub challenge: String,
    pub result: ChallengeResult,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChallengeResult {
    DefectFound,
    NoDefect,
    Inconclusive,
}

// ── Rubric impl ──

impl Rubric {
    pub fn new(mission: &str, criteria: Vec<RubricCriterion>) -> Self {
        Self {
            mission: mission.to_string(),
            criteria,
            created_at: Utc::now(),
        }
    }

    pub fn write(&self, state_dir: &Path, oracle: &str) -> Result<()> {
        let path = state_dir.join(format!("{}.rubric.json", oracle));
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn read(state_dir: &Path, oracle: &str) -> Result<Option<Self>> {
        let path = state_dir.join(format!("{}.rubric.json", oracle));
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)?;
        Ok(Some(serde_json::from_str(&content)?))
    }
}

/// The writer (oracle or worker) cannot gate-accept its own mission.
/// Gareth / a human on a regular terminal can. `caller_session` is
/// `OMEGA_SESSION` when `omega gate` is typed from inside an agent pane.
pub fn refuse_writer_self_approval(
    oracle: &str,
    approver: &str,
    caller_session: Option<&str>,
) -> Result<()> {
    if looks_like_writer_identity(approver, oracle) {
        anyhow::bail!(
            "{}",
            serde_json::json!({
                "error": "writer_cannot_self_approve",
                "oracle": oracle,
                "approver": approver,
                "message": "the writer cannot gate-accept its own work. A human signs off with omega gate --accept --approver <human> --evidence <what you verified>."
            })
        );
    }
    if let Some(caller) = caller_session {
        if looks_like_writer_identity(caller, oracle) {
            anyhow::bail!(
                "{}",
                serde_json::json!({
                    "error": "writer_cannot_self_approve",
                    "oracle": oracle,
                    "caller": caller,
                    "message": "omega gate --accept from an oracle/worker pane is refused. Sign off from a human terminal."
                })
            );
        }
    }
    Ok(())
}

fn looks_like_writer_identity(name: &str, oracle: &str) -> bool {
    let n = name.trim().to_ascii_lowercase();
    if n.is_empty() {
        return false;
    }
    let oracle = oracle.trim().to_ascii_lowercase();
    n == oracle || n.starts_with("oracle-") || n.contains("-worker-") || n.starts_with("worker-")
}

// ── GateResult impl ──

impl GateResult {
    #[allow(clippy::too_many_arguments)]
    pub fn evaluate(
        rubric: &Rubric,
        grades: Vec<GradeResult>,
        consensus_votes: Vec<ConsensusVote>,
        falsification: &FalsificationReport,
        audit_results: Vec<crate::audit::AuditResult>,
        regression_pass: bool,
        token_budget_pass: bool,
        citation_pass: bool,
    ) -> Self {
        let rubric_pass = grades.iter().all(|g| g.verdict == GradeVerdict::Satisfied);

        let satisfied_votes = consensus_votes
            .iter()
            .filter(|v| v.verdict == GradeVerdict::Satisfied)
            .count();
        // R-21: ≥2/3 graders must agree SATISFIED. An empty vote list cannot meet
        // quorum — `.max(1)` forces 0 >= 2/3 to be false (mirrors MultiGrader::evaluate),
        // so a gate with no multi-grader input fails instead of silently passing.
        let consensus_pass = satisfied_votes * 3 >= consensus_votes.len().max(1) * 2;

        // R-30: adversarial_pass is the REAL Popper falsification verdict —
        // ≥12 challenges, zero defects, zero uncited (FalsificationReport::pass).
        // Previously this only checked defects==0 on raw challenges, so a gate
        // with 0 (or uncited) challenges still "passed" the adversarial check.
        let adversarial_challenges = falsification.challenges.clone();
        let adversarial_pass = falsification.pass;

        let total_weight: f32 = rubric.criteria.iter().map(|c| c.weight).sum();
        let earned_weight: f32 = grades
            .iter()
            .filter(|g| g.verdict == GradeVerdict::Satisfied)
            .filter_map(|g| {
                rubric
                    .criteria
                    .iter()
                    .find(|c| c.id == g.criterion_id)
                    .map(|c| c.weight)
            })
            .sum();

        let score = if total_weight > 0.0 {
            (earned_weight / total_weight) * 100.0
        } else {
            0.0
        };

        // R-AUDIT: audit_pass reflects the REAL audit results, not a hardcoded
        // `true`. With no audits run it stays true (nothing to fail); once an
        // audit chain ran, every audit must clear NeedsWork/Fail (verdict==Pass).
        use crate::audit::AuditVerdict;
        let audit_pass = audit_results
            .iter()
            .all(|a| a.verdict == AuditVerdict::Pass);
        let overall_pass = rubric_pass
            && consensus_pass
            && adversarial_pass
            && regression_pass
            && audit_pass
            && token_budget_pass
            && citation_pass;

        Self {
            oracle: String::new(),
            timestamp: Utc::now(),
            rubric_pass,
            consensus_pass,
            adversarial_pass,
            regression_pass,
            audit_results,
            audit_pass,
            token_budget_pass,
            citation_pass,
            overall_pass,
            score,
            details: GateDetails {
                grades,
                consensus_votes,
                adversarial_challenges,
            },
            accepted_by: None,
            accepted_evidence: None,
        }
    }

    /// A HUMAN acceptance of the quality gate, signed and evidenced.
    ///
    /// WHY THIS EXISTS. `closure_verdict` refuses a mission until an independent
    /// `GateResult` says `overall_pass`, and the only code that ever produces one
    /// is the `omega orchestrate` pipeline (`orchestration.rs`). A mission
    /// dispatched with `omega dispatch` — which is what the Telegram bot and
    /// every operator do — therefore had NO reachable way to satisfy the gate,
    /// so its close was refused for as long as it existed. Three oracles were
    /// stuck that way on this box, one of them for 59 hours.
    ///
    /// It stays honest in three ways: the approver is REQUIRED and never
    /// defaulted (an agent must not sign its own work off), the evidence is
    /// recorded verbatim, and `accepted_by` marks the result as a human
    /// acceptance so nothing downstream can mistake it for a graded pass.
    pub fn human_acceptance(oracle: &str, approver: &str, evidence: &str) -> Result<Self> {
        let approver = approver.trim();
        let evidence = evidence.trim();
        if approver.is_empty() {
            anyhow::bail!("an acceptance needs an approver: pass --approver \"<who>\"");
        }
        if evidence.is_empty() {
            anyhow::bail!("an acceptance needs evidence: pass --evidence \"<what you verified>\"");
        }
        refuse_writer_self_approval(oracle, approver, None)?;
        Ok(Self {
            oracle: oracle.to_string(),
            timestamp: Utc::now(),
            // Every machine sub-verdict stays FALSE: nothing was graded, and
            // claiming otherwise would forge a rubric pass nobody ran. Only
            // `overall_pass` is true, and `accepted_by` says exactly why.
            rubric_pass: false,
            consensus_pass: false,
            adversarial_pass: false,
            regression_pass: false,
            audit_results: Vec::new(),
            audit_pass: false,
            token_budget_pass: false,
            citation_pass: false,
            overall_pass: true,
            score: 0.0,
            details: GateDetails {
                grades: Vec::new(),
                consensus_votes: Vec::new(),
                adversarial_challenges: Vec::new(),
            },
            accepted_by: Some(approver.to_string()),
            accepted_evidence: Some(evidence.to_string()),
        })
    }

    pub fn write(&self, state_dir: &Path) -> Result<()> {
        let path = state_dir.join(format!("{}.gate-result.json", self.oracle));
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Read a previously-persisted gate result for `oracle`, if any. Used by the
    /// regression detector (R-22) to compare the current run against the prior one.
    pub fn read(state_dir: &Path, oracle: &str) -> Result<Option<Self>> {
        let path = state_dir.join(format!("{}.gate-result.json", oracle));
        if !path.exists() {
            return Ok(None);
        }
        Ok(Some(serde_json::from_str(&std::fs::read_to_string(
            &path,
        )?)?))
    }
}

// ── Multi-Grader (R-21) ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraderLens {
    CodeReviewer,
    Debugger,
    GeneralPurpose,
}

impl GraderLens {
    pub fn all() -> [Self; 3] {
        [Self::CodeReviewer, Self::Debugger, Self::GeneralPurpose]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::CodeReviewer => "code-reviewer",
            Self::Debugger => "debugger",
            Self::GeneralPurpose => "general-purpose",
        }
    }
}

/// 3 independent verification lenses, ≥2/3 must agree SATISFIED (R-21).
pub struct MultiGrader;

impl MultiGrader {
    pub fn evaluate(
        submissions: &[(GraderLens, GradeVerdict, f32, String)],
    ) -> (Vec<ConsensusVote>, bool) {
        let votes: Vec<ConsensusVote> = submissions
            .iter()
            .map(|(lens, verdict, confidence, reasoning)| ConsensusVote {
                grader: lens.label().to_string(),
                verdict: *verdict,
                confidence: *confidence,
                reasoning: reasoning.clone(),
            })
            .collect();

        let satisfied = votes
            .iter()
            .filter(|v| v.verdict == GradeVerdict::Satisfied)
            .count();
        let total = votes.len().max(1);
        let pass = satisfied * 3 >= total * 2;

        (votes, pass)
    }
}

// ── Popper Falsifier (R-30) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FalsificationReport {
    pub challenges: Vec<AdversarialChallenge>,
    pub minimum_required: usize,
    pub total_attempted: usize,
    pub defects_found: usize,
    pub uncited_rejected: usize,
    pub pass: bool,
}

/// ≥12 adversarial challenges with file:line evidence (R-30 + R-35).
pub struct PopperFalsifier;

impl PopperFalsifier {
    pub const MINIMUM_CHALLENGES: usize = 12;

    pub fn validate(challenges: &[AdversarialChallenge]) -> FalsificationReport {
        let uncited = challenges
            .iter()
            .filter(|c| c.evidence.trim().is_empty())
            .count();
        let defects = challenges
            .iter()
            .filter(|c| matches!(c.result, ChallengeResult::DefectFound))
            .count();
        let enough = challenges.len() >= Self::MINIMUM_CHALLENGES;

        FalsificationReport {
            challenges: challenges.to_vec(),
            minimum_required: Self::MINIMUM_CHALLENGES,
            total_attempted: challenges.len(),
            defects_found: defects,
            uncited_rejected: uncited,
            pass: enough && defects == 0 && uncited == 0,
        }
    }
}

// ── Regression Detector (R-22) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionReport {
    pub previous_score: Option<f32>,
    pub current_score: f32,
    pub regressions: Vec<RegressionItem>,
    pub pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionItem {
    pub criterion_id: String,
    pub was: GradeVerdict,
    pub now: GradeVerdict,
}

/// Semantic diff current vs previous iteration — zero regressions required (R-22).
pub struct RegressionDetector;

impl RegressionDetector {
    pub fn detect(prior: Option<&GateResult>, current_grades: &[GradeResult]) -> RegressionReport {
        let current_score = Self::score_from_grades(current_grades);

        let prior_gate = match prior {
            None => {
                return RegressionReport {
                    previous_score: None,
                    current_score,
                    regressions: Vec::new(),
                    pass: true,
                }
            }
            Some(g) => g,
        };

        let mut regressions = Vec::new();
        for grade in current_grades {
            if let Some(prev) = prior_gate
                .details
                .grades
                .iter()
                .find(|g| g.criterion_id == grade.criterion_id)
            {
                if prev.verdict == GradeVerdict::Satisfied
                    && grade.verdict != GradeVerdict::Satisfied
                {
                    regressions.push(RegressionItem {
                        criterion_id: grade.criterion_id.clone(),
                        was: prev.verdict,
                        now: grade.verdict,
                    });
                }
            }
        }

        // R-22: a previously-satisfied criterion that DISAPPEARS from the current
        // rubric is a structural regression — the criterion is no longer verified.
        // The loop above only sees criteria present in BOTH runs, so removed ones
        // would otherwise pass silently.
        for prev in &prior_gate.details.grades {
            if prev.verdict == GradeVerdict::Satisfied
                && !current_grades
                    .iter()
                    .any(|g| g.criterion_id == prev.criterion_id)
            {
                regressions.push(RegressionItem {
                    criterion_id: prev.criterion_id.clone(),
                    was: prev.verdict,
                    now: GradeVerdict::Unmet,
                });
            }
        }

        RegressionReport {
            previous_score: Some(prior_gate.score),
            current_score,
            regressions: regressions.clone(),
            pass: regressions.is_empty(),
        }
    }

    fn score_from_grades(grades: &[GradeResult]) -> f32 {
        if grades.is_empty() {
            return 0.0;
        }
        let satisfied = grades
            .iter()
            .filter(|g| g.verdict == GradeVerdict::Satisfied)
            .count();
        (satisfied as f32 / grades.len() as f32) * 100.0
    }
}

// ── Token Budget (R-28) ──

/// Track spend per mission — hard stop at cap (R-28). Default 500K tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBudget {
    pub cap: u64,
    pub spent: u64,
}

impl TokenBudget {
    pub const DEFAULT_CAP: u64 = 500_000;

    pub fn new(cap: u64) -> Self {
        Self { cap, spent: 0 }
    }

    pub fn record(&mut self, tokens: u64) {
        self.spent = self.spent.saturating_add(tokens);
    }

    pub fn check(&self) -> bool {
        self.spent <= self.cap
    }

    pub fn remaining(&self) -> u64 {
        self.cap.saturating_sub(self.spent)
    }

    pub fn utilization(&self) -> f32 {
        if self.cap == 0 {
            return 100.0;
        }
        (self.spent as f32 / self.cap as f32) * 100.0
    }

    pub fn write(&self, state_dir: &Path, mission_id: &str) -> Result<()> {
        let path = state_dir.join(format!("{}.token-budget.json", mission_id));
        std::fs::write(&path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn read(state_dir: &Path, mission_id: &str) -> Result<Option<Self>> {
        let path = state_dir.join(format!("{}.token-budget.json", mission_id));
        if !path.exists() {
            return Ok(None);
        }
        Ok(Some(serde_json::from_str(&std::fs::read_to_string(
            &path,
        )?)?))
    }
}

// ── Citation Enforcer (R-35) ──

/// Every claim in audits/grading requires citations — no citation = rejected (R-35).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationReport {
    pub total_claims: usize,
    pub cited: usize,
    pub uncited: Vec<String>,
    pub pass: bool,
}

pub struct CitationEnforcer;

impl CitationEnforcer {
    pub fn validate(claims: &[&str]) -> CitationReport {
        let mut uncited = Vec::new();
        for claim in claims {
            if !Self::has_citation(claim) {
                uncited.push(claim.to_string());
            }
        }
        CitationReport {
            total_claims: claims.len(),
            cited: claims.len() - uncited.len(),
            uncited: uncited.clone(),
            pass: uncited.is_empty(),
        }
    }

    fn has_citation(claim: &str) -> bool {
        let source_exts = [
            ".rs:", ".ts:", ".tsx:", ".js:", ".jsx:", ".py:", ".toml:", ".json:", ".md:", ".css:",
        ];
        if source_exts.iter().any(|p| claim.contains(p)) {
            return true;
        }
        if claim.contains("screenshot") || claim.contains("log:") {
            return true;
        }
        if let Some(idx) = claim.find("line ") {
            let after = &claim[idx + 5..];
            if after.starts_with(|c: char| c.is_ascii_digit()) {
                return true;
            }
        }
        false
    }
}

// ── Quality Gate Orchestrator ──

/// Orchestrates the full quality gate: rubric + multi-grader + Popper +
/// regression + budget + citations → GateResult.
pub struct QualityGate {
    state_dir: PathBuf,
    token_cap: u64,
}

impl QualityGate {
    pub fn new(state_dir: PathBuf, token_cap: u64) -> Self {
        Self {
            state_dir,
            token_cap,
        }
    }

    pub fn with_default_cap(state_dir: PathBuf) -> Self {
        Self::new(state_dir, TokenBudget::DEFAULT_CAP)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn run(
        &self,
        oracle: &str,
        rubric: &Rubric,
        grades: Vec<GradeResult>,
        grader_submissions: Vec<(GraderLens, GradeVerdict, f32, String)>,
        challenges: Vec<AdversarialChallenge>,
        prior_gate: Option<&GateResult>,
        tokens_spent: u64,
        claims: &[&str],
    ) -> GateResult {
        self.run_with_audits(
            oracle,
            rubric,
            grades,
            grader_submissions,
            challenges,
            Vec::new(),
            prior_gate,
            tokens_spent,
            claims,
        )
    }

    /// Same as [`run`] but threads real audit results into the gate so
    /// `audit_pass` reflects them (R-AUDIT). `run` delegates here with no
    /// audits — keeping the existing call sites unchanged.
    #[allow(clippy::too_many_arguments)]
    pub fn run_with_audits(
        &self,
        oracle: &str,
        rubric: &Rubric,
        grades: Vec<GradeResult>,
        grader_submissions: Vec<(GraderLens, GradeVerdict, f32, String)>,
        challenges: Vec<AdversarialChallenge>,
        audit_results: Vec<crate::audit::AuditResult>,
        prior_gate: Option<&GateResult>,
        tokens_spent: u64,
        claims: &[&str],
    ) -> GateResult {
        let (consensus_votes, _) = MultiGrader::evaluate(&grader_submissions);
        // R-30: the Popper falsification verdict actually drives the gate now —
        // its `pass` becomes adversarial_pass inside GateResult::evaluate
        // (previously the report was computed then discarded as `_popper`).
        let falsification = PopperFalsifier::validate(&challenges);
        let regression = RegressionDetector::detect(prior_gate, &grades);

        let mut budget = TokenBudget::new(self.token_cap);
        budget.record(tokens_spent);
        let _ = budget.write(&self.state_dir, oracle);

        let citations = CitationEnforcer::validate(claims);

        let mut result = GateResult::evaluate(
            rubric,
            grades,
            consensus_votes,
            &falsification,
            audit_results,
            regression.pass,
            budget.check(),
            citations.pass,
        );
        result.oracle = oracle.to_string();
        let _ = result.write(&self.state_dir);

        // ── Loop Engineering: bound the gate's re-verifies (R-LOOP) ──
        // The gate is a firewall, not a correction loop. A mission that keeps
        // failing it should not re-verify forever — count consecutive failures
        // and at GATE_RETRY_CAP hand the loop to a human. A pass resets the
        // counter. Every run leaves a timeline note for `omega log`.
        if result.overall_pass {
            crate::loop_guard::clear_gate_attempt(&self.state_dir, oracle);
            crate::loop_guard::MissionLog::event(
                &self.state_dir,
                oracle,
                "gate",
                "quality gate PASSED",
            );
        } else {
            let attempts = crate::loop_guard::bump_gate_attempt(&self.state_dir, oracle);
            crate::loop_guard::MissionLog::event(
                &self.state_dir,
                oracle,
                "gate",
                &format!(
                    "quality gate FAILED (attempt {}/{})",
                    attempts,
                    crate::loop_guard::GATE_RETRY_CAP
                ),
            );
            if attempts >= crate::loop_guard::GATE_RETRY_CAP {
                crate::loop_guard::escalate_to_human(
                    &self.state_dir,
                    oracle,
                    crate::loop_guard::EscalationReason::GateRetryCap,
                    &format!(
                        "quality gate failed {}× — re-verify cap hit, needs a human",
                        attempts
                    ),
                );
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_challenge(i: usize, defect: bool) -> AdversarialChallenge {
        AdversarialChallenge {
            challenge: format!("edge case {}", i),
            result: if defect {
                ChallengeResult::DefectFound
            } else {
                ChallengeResult::NoDefect
            },
            evidence: format!("src/main.rs:{}", i + 1),
        }
    }

    fn passing_votes() -> Vec<ConsensusVote> {
        GraderLens::all()
            .iter()
            .map(|lens| ConsensusVote {
                grader: lens.label().to_string(),
                verdict: GradeVerdict::Satisfied,
                confidence: 0.9,
                reasoning: "ok".into(),
            })
            .collect()
    }

    fn make_grade(id: &str, verdict: GradeVerdict) -> GradeResult {
        GradeResult {
            criterion_id: id.into(),
            verdict,
            confidence: 0.9,
            evidence: format!("src/lib.rs:1 — {}", id),
        }
    }

    fn make_gate(grades: Vec<GradeResult>, score: f32) -> GateResult {
        GateResult {
            oracle: "prior".into(),
            timestamp: Utc::now(),
            rubric_pass: true,
            consensus_pass: true,
            adversarial_pass: true,
            regression_pass: true,
            audit_results: vec![],
            audit_pass: true,
            token_budget_pass: true,
            citation_pass: true,
            overall_pass: true,
            score,
            details: GateDetails {
                grades,
                consensus_votes: vec![],
                adversarial_challenges: vec![],
            },
            accepted_by: None,
            accepted_evidence: None,
        }
    }

    // ── Multi-Grader ──

    #[test]
    fn multi_grader_two_of_three_passes() {
        let submissions = vec![
            (
                GraderLens::CodeReviewer,
                GradeVerdict::Satisfied,
                0.9,
                "ok".into(),
            ),
            (
                GraderLens::Debugger,
                GradeVerdict::NeedsRevision,
                0.7,
                "issue".into(),
            ),
            (
                GraderLens::GeneralPurpose,
                GradeVerdict::Satisfied,
                0.85,
                "ok".into(),
            ),
        ];
        let (votes, pass) = MultiGrader::evaluate(&submissions);
        assert_eq!(votes.len(), 3);
        assert!(pass);
    }

    #[test]
    fn multi_grader_one_of_three_fails() {
        let submissions = vec![
            (
                GraderLens::CodeReviewer,
                GradeVerdict::Satisfied,
                0.9,
                "ok".into(),
            ),
            (
                GraderLens::Debugger,
                GradeVerdict::NeedsRevision,
                0.7,
                "no".into(),
            ),
            (
                GraderLens::GeneralPurpose,
                GradeVerdict::NeedsRevision,
                0.6,
                "no".into(),
            ),
        ];
        let (_, pass) = MultiGrader::evaluate(&submissions);
        assert!(!pass);
    }

    #[test]
    fn multi_grader_all_three_passes() {
        let submissions = vec![
            (
                GraderLens::CodeReviewer,
                GradeVerdict::Satisfied,
                0.9,
                "ok".into(),
            ),
            (
                GraderLens::Debugger,
                GradeVerdict::Satisfied,
                0.9,
                "ok".into(),
            ),
            (
                GraderLens::GeneralPurpose,
                GradeVerdict::Satisfied,
                0.9,
                "ok".into(),
            ),
        ];
        let (_, pass) = MultiGrader::evaluate(&submissions);
        assert!(pass);
    }

    // ── Popper Falsifier ──

    #[test]
    fn popper_needs_12_challenges() {
        let challenges: Vec<_> = (0..11).map(|i| make_challenge(i, false)).collect();
        let report = PopperFalsifier::validate(&challenges);
        assert!(!report.pass);
        assert_eq!(report.total_attempted, 11);
    }

    #[test]
    fn popper_12_no_defects_passes() {
        let challenges: Vec<_> = (0..12).map(|i| make_challenge(i, false)).collect();
        let report = PopperFalsifier::validate(&challenges);
        assert!(report.pass);
    }

    #[test]
    fn popper_defect_found_fails() {
        let mut challenges: Vec<_> = (0..12).map(|i| make_challenge(i, false)).collect();
        challenges[5].result = ChallengeResult::DefectFound;
        let report = PopperFalsifier::validate(&challenges);
        assert!(!report.pass);
        assert_eq!(report.defects_found, 1);
    }

    #[test]
    fn popper_uncited_rejected() {
        let mut challenges: Vec<_> = (0..12).map(|i| make_challenge(i, false)).collect();
        challenges[3].evidence = String::new();
        let report = PopperFalsifier::validate(&challenges);
        assert!(!report.pass);
        assert_eq!(report.uncited_rejected, 1);
    }

    // ── Regression Detector ──

    #[test]
    fn regression_no_prior_passes() {
        let grades = vec![make_grade("c1", GradeVerdict::Satisfied)];
        let report = RegressionDetector::detect(None, &grades);
        assert!(report.pass);
        assert!(report.previous_score.is_none());
    }

    #[test]
    fn regression_detected_when_criterion_degrades() {
        let prior = make_gate(vec![make_grade("c1", GradeVerdict::Satisfied)], 100.0);
        let current = vec![make_grade("c1", GradeVerdict::NeedsRevision)];
        let report = RegressionDetector::detect(Some(&prior), &current);
        assert!(!report.pass);
        assert_eq!(report.regressions.len(), 1);
        assert_eq!(report.regressions[0].criterion_id, "c1");
    }

    #[test]
    fn regression_detected_when_satisfied_criterion_removed() {
        // R-22 structural regression: a previously-satisfied criterion dropped from
        // the rubric must be flagged, not silently passed.
        let prior = make_gate(
            vec![
                make_grade("c1", GradeVerdict::Satisfied),
                make_grade("c2", GradeVerdict::Satisfied),
            ],
            100.0,
        );
        let current = vec![make_grade("c1", GradeVerdict::Satisfied)];
        let report = RegressionDetector::detect(Some(&prior), &current);
        assert!(!report.pass);
        assert_eq!(report.regressions.len(), 1);
        assert_eq!(report.regressions[0].criterion_id, "c2");
        assert_eq!(report.regressions[0].now, GradeVerdict::Unmet);
    }

    #[test]
    fn no_regression_when_improvement() {
        let prior = make_gate(vec![make_grade("c1", GradeVerdict::NeedsRevision)], 0.0);
        let current = vec![make_grade("c1", GradeVerdict::Satisfied)];
        let report = RegressionDetector::detect(Some(&prior), &current);
        assert!(report.pass);
    }

    // ── Token Budget ──

    #[test]
    fn budget_under_cap() {
        let mut budget = TokenBudget::new(500_000);
        budget.record(300_000);
        assert!(budget.check());
        assert_eq!(budget.remaining(), 200_000);
    }

    #[test]
    fn budget_over_cap() {
        let mut budget = TokenBudget::new(500_000);
        budget.record(600_000);
        assert!(!budget.check());
        assert_eq!(budget.remaining(), 0);
    }

    #[test]
    fn budget_at_cap() {
        let mut budget = TokenBudget::new(500_000);
        budget.record(500_000);
        assert!(budget.check());
    }

    #[test]
    fn budget_utilization() {
        let mut budget = TokenBudget::new(1000);
        budget.record(250);
        assert!((budget.utilization() - 25.0).abs() < 0.1);
    }

    #[test]
    fn budget_persistence() {
        let dir = tempfile::tempdir().unwrap();
        let mut budget = TokenBudget::new(500_000);
        budget.record(100_000);
        budget.write(dir.path(), "m-test").unwrap();
        let loaded = TokenBudget::read(dir.path(), "m-test").unwrap().unwrap();
        assert_eq!(loaded.spent, 100_000);
        assert_eq!(loaded.cap, 500_000);
    }

    // ── Citation Enforcer ──

    #[test]
    fn citation_file_line_passes() {
        let claims = &["Found issue in src/main.rs:42"];
        let report = CitationEnforcer::validate(claims);
        assert!(report.pass);
    }

    #[test]
    fn citation_log_passes() {
        let claims = &["Error visible in log: connection refused"];
        let report = CitationEnforcer::validate(claims);
        assert!(report.pass);
    }

    #[test]
    fn citation_screenshot_passes() {
        let claims = &["Confirmed via screenshot at /tmp/shot.png"];
        let report = CitationEnforcer::validate(claims);
        assert!(report.pass);
    }

    #[test]
    fn citation_line_number_passes() {
        let claims = &["Problem at line 42 in the auth module"];
        let report = CitationEnforcer::validate(claims);
        assert!(report.pass);
    }

    #[test]
    fn citation_uncited_fails() {
        let claims = &["The code is bad"];
        let report = CitationEnforcer::validate(claims);
        assert!(!report.pass);
        assert_eq!(report.uncited.len(), 1);
    }

    #[test]
    fn citation_mixed() {
        let claims = &["Found bug in src/main.rs:42", "The code is bad"];
        let report = CitationEnforcer::validate(claims);
        assert!(!report.pass);
        assert_eq!(report.cited, 1);
        assert_eq!(report.uncited.len(), 1);
    }

    #[test]
    fn citation_empty_passes() {
        let claims: &[&str] = &[];
        let report = CitationEnforcer::validate(claims);
        assert!(report.pass);
    }

    // ── Gate Evaluate ──

    #[test]
    fn gate_all_pass() {
        let rubric = Rubric::new(
            "test",
            vec![RubricCriterion {
                id: "c1".into(),
                description: "test".into(),
                weight: 1.0,
                category: CriterionCategory::Functional,
            }],
        );
        let grades = vec![make_grade("c1", GradeVerdict::Satisfied)];
        // R-30: a passing gate needs a passing Popper report (≥12 cited, no defects).
        let challenges: Vec<_> = (0..12).map(|i| make_challenge(i, false)).collect();
        let falsification = PopperFalsifier::validate(&challenges);
        // R-21: a passing gate needs a quorum of SATISFIED consensus votes.
        let result = GateResult::evaluate(
            &rubric,
            grades,
            passing_votes(),
            &falsification,
            vec![],
            true,
            true,
            true,
        );
        assert!(result.overall_pass);
        assert!(result.adversarial_pass);
        assert!(result.consensus_pass);
        assert!((result.score - 100.0).abs() < 0.1);
    }

    #[test]
    fn gate_empty_consensus_blocks() {
        // R-21: zero multi-grader votes cannot meet ≥2/3 quorum, so the gate fails.
        let rubric = Rubric::new("test", vec![]);
        let challenges: Vec<_> = (0..12).map(|i| make_challenge(i, false)).collect();
        let falsification = PopperFalsifier::validate(&challenges);
        let result = GateResult::evaluate(
            &rubric,
            vec![],
            vec![],
            &falsification,
            vec![],
            true,
            true,
            true,
        );
        assert!(!result.consensus_pass);
        assert!(!result.overall_pass);
    }

    #[test]
    fn gate_adversarial_fail_blocks() {
        // Fewer than 12 challenges → Popper fails → adversarial_pass false → gate blocks.
        let rubric = Rubric::new("test", vec![]);
        let challenges: Vec<_> = (0..5).map(|i| make_challenge(i, false)).collect();
        let falsification = PopperFalsifier::validate(&challenges);
        let result = GateResult::evaluate(
            &rubric,
            vec![],
            vec![],
            &falsification,
            vec![],
            true,
            true,
            true,
        );
        assert!(!result.overall_pass);
        assert!(!result.adversarial_pass);
    }

    #[test]
    fn gate_audit_fail_blocks() {
        // A failing audit result must block the gate (audit_pass is real, not a stub).
        use crate::audit::AuditResult;
        let rubric = Rubric::new("test", vec![]);
        let challenges: Vec<_> = (0..12).map(|i| make_challenge(i, false)).collect();
        let falsification = PopperFalsifier::validate(&challenges);
        // 40/100 → Fail verdict.
        let audits = vec![AuditResult::new("codeaudit", 168.0, 420)];
        let result = GateResult::evaluate(
            &rubric,
            vec![],
            vec![],
            &falsification,
            audits,
            true,
            true,
            true,
        );
        assert!(!result.audit_pass);
        assert!(!result.overall_pass);
    }

    #[test]
    fn gate_token_budget_fail_blocks() {
        let rubric = Rubric::new("test", vec![]);
        let challenges: Vec<_> = (0..12).map(|i| make_challenge(i, false)).collect();
        let falsification = PopperFalsifier::validate(&challenges);
        let result = GateResult::evaluate(
            &rubric,
            vec![],
            vec![],
            &falsification,
            vec![],
            true,
            false,
            true,
        );
        assert!(!result.overall_pass);
        assert!(!result.token_budget_pass);
    }

    #[test]
    fn gate_citation_fail_blocks() {
        let rubric = Rubric::new("test", vec![]);
        let challenges: Vec<_> = (0..12).map(|i| make_challenge(i, false)).collect();
        let falsification = PopperFalsifier::validate(&challenges);
        let result = GateResult::evaluate(
            &rubric,
            vec![],
            vec![],
            &falsification,
            vec![],
            true,
            true,
            false,
        );
        assert!(!result.overall_pass);
        assert!(!result.citation_pass);
    }

    #[test]
    fn gate_regression_fail_blocks() {
        let rubric = Rubric::new("test", vec![]);
        let challenges: Vec<_> = (0..12).map(|i| make_challenge(i, false)).collect();
        let falsification = PopperFalsifier::validate(&challenges);
        let result = GateResult::evaluate(
            &rubric,
            vec![],
            vec![],
            &falsification,
            vec![],
            false,
            true,
            true,
        );
        assert!(!result.overall_pass);
        assert!(!result.regression_pass);
    }

    // ── Quality Gate Orchestrator ──

    #[test]
    fn quality_gate_full_pass() {
        let dir = tempfile::tempdir().unwrap();
        let qg = QualityGate::with_default_cap(dir.path().to_path_buf());
        let rubric = Rubric::new(
            "test mission",
            vec![RubricCriterion {
                id: "c1".into(),
                description: "it works".into(),
                weight: 1.0,
                category: CriterionCategory::Functional,
            }],
        );
        let grades = vec![make_grade("c1", GradeVerdict::Satisfied)];
        let submissions = vec![
            (
                GraderLens::CodeReviewer,
                GradeVerdict::Satisfied,
                0.9,
                "ok".into(),
            ),
            (
                GraderLens::Debugger,
                GradeVerdict::Satisfied,
                0.9,
                "ok".into(),
            ),
            (
                GraderLens::GeneralPurpose,
                GradeVerdict::Satisfied,
                0.9,
                "ok".into(),
            ),
        ];
        let challenges: Vec<_> = (0..12).map(|i| make_challenge(i, false)).collect();
        let claims = &["Verified at src/main.rs:1"];

        let result = qg.run(
            "test-oracle",
            &rubric,
            grades,
            submissions,
            challenges,
            None,
            1000,
            claims,
        );
        assert!(result.overall_pass);
        assert!(result.adversarial_pass);
        assert!(dir.path().join("test-oracle.gate-result.json").exists());
        assert!(dir.path().join("test-oracle.token-budget.json").exists());
    }

    #[test]
    fn grader_lens_labels() {
        assert_eq!(GraderLens::CodeReviewer.label(), "code-reviewer");
        assert_eq!(GraderLens::Debugger.label(), "debugger");
        assert_eq!(GraderLens::GeneralPurpose.label(), "general-purpose");
        assert_eq!(GraderLens::all().len(), 3);
    }

    // ── Human acceptance of the gate ──

    #[test]
    fn a_human_acceptance_passes_the_gate_and_says_who_signed_it() {
        let g = GateResult::human_acceptance("oracle-p-1", "gs", "prod 200 + 14/15 verified")
            .expect("a signed acceptance is accepted");
        assert!(g.overall_pass, "the close gate must now be satisfiable");
        assert_eq!(g.accepted_by.as_deref(), Some("gs"));
        assert_eq!(
            g.accepted_evidence.as_deref(),
            Some("prod 200 + 14/15 verified")
        );
    }

    #[test]
    fn a_human_acceptance_forges_no_machine_verdict() {
        // The honesty property: nothing was graded, so every machine sub-verdict
        // stays false and a reader can always tell an acceptance from a pass.
        let g = GateResult::human_acceptance("oracle-p-1", "gs", "read the diff").unwrap();
        assert!(!g.rubric_pass);
        assert!(!g.consensus_pass);
        assert!(!g.adversarial_pass);
        assert!(!g.audit_pass);
        assert!(g.details.grades.is_empty());
        assert!(g.details.consensus_votes.is_empty());
    }

    #[test]
    fn an_unsigned_or_unevidenced_acceptance_is_refused() {
        // An agent must not write its own permission slip.
        assert!(GateResult::human_acceptance("oracle-p-1", "", "evidence").is_err());
        assert!(GateResult::human_acceptance("oracle-p-1", "   ", "evidence").is_err());
        assert!(GateResult::human_acceptance("oracle-p-1", "gs", "").is_err());
        assert!(GateResult::human_acceptance("oracle-p-1", "gs", "  ").is_err());
    }

    #[test]
    fn a_writer_cannot_gate_accept_itself() {
        assert!(GateResult::human_acceptance("oracle-p-1", "oracle-p-1", "trust me").is_err());
        assert!(GateResult::human_acceptance("oracle-p-1", "p-1-worker-auth", "trust me").is_err());
        assert!(refuse_writer_self_approval("oracle-p-1", "gs", Some("oracle-p-1")).is_err());
        assert!(refuse_writer_self_approval("oracle-p-1", "gs", None).is_ok());
    }

    #[test]
    fn a_gate_result_written_before_the_acceptance_fields_still_parses() {
        // Additive + defaulted: the field landed on a struct that already had
        // results on disk.
        let legacy = r#"{
            "oracle":"oracle-p-1","timestamp":"2026-01-01T00:00:00Z",
            "rubric_pass":true,"consensus_pass":true,"adversarial_pass":true,
            "regression_pass":true,"audit_results":[],"audit_pass":true,
            "token_budget_pass":true,"citation_pass":true,"overall_pass":true,
            "score":1.0,
            "details":{"grades":[],"consensus_votes":[],"adversarial_challenges":[]}
        }"#;
        let g: GateResult = serde_json::from_str(legacy).expect("legacy result must still parse");
        assert!(g.overall_pass);
        assert!(
            g.accepted_by.is_none(),
            "a graded pass is not an acceptance"
        );
    }
}

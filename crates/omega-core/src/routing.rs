use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Complexity {
    Simple,
    Medium,
    Complex,
    Epic,
}

pub const ROUTER_VERSION: &str = "v3-topology-risk-1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    #[default]
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RoutingTopology {
    #[default]
    SingleAgent,
    ManagerTools,
    Handoff,
    ParallelWorkers,
    Council,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum QualityPolicy {
    #[default]
    Standard,
    Verified,
    Adversarial,
    ApprovalRequired,
}

impl Complexity {
    pub fn label(&self) -> &'static str {
        match self {
            Complexity::Simple => "SIMPLE",
            Complexity::Medium => "MEDIUM",
            Complexity::Complex => "COMPLEX",
            Complexity::Epic => "EPIC",
        }
    }

    pub fn recommended_agents(&self) -> usize {
        match self {
            Complexity::Simple => 1,
            Complexity::Medium => 1,
            Complexity::Complex => 3,
            Complexity::Epic => 5,
        }
    }

    pub fn estimated_minutes(&self) -> u32 {
        match self {
            Complexity::Simple => 5,
            Complexity::Medium => 20,
            Complexity::Complex => 60,
            Complexity::Epic => 240,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub complexity: Complexity,
    pub reasoning: Vec<String>,
    pub suggested_agent: String,
    pub decompose: bool,
    pub use_team: bool,
    pub use_quality_gate: bool,
    pub audit_skills: Vec<AuditSkill>,
    #[serde(default)]
    pub topology: RoutingTopology,
    #[serde(default)]
    pub risk: RiskLevel,
    #[serde(default)]
    pub confidence: f32,
    #[serde(default)]
    pub required_capabilities: Vec<String>,
    #[serde(default = "default_parallelism")]
    pub recommended_parallelism: usize,
    #[serde(default)]
    pub quality_policy: QualityPolicy,
    #[serde(default = "default_router_version")]
    pub router_version: String,
}

fn default_parallelism() -> usize {
    1
}

fn default_router_version() -> String {
    ROUTER_VERSION.to_string()
}

/// Maps audit keywords to specific forensic skill invocations.
/// Mirrors the live system's Quality Arsenal (see `crate::audit::all_audits`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditSkill {
    pub skill: String,
    pub trigger: String,
}

/// Detect audit keywords and return matching skills.
fn detect_audit_skills(text: &str) -> Vec<AuditSkill> {
    let lower = text.to_lowercase();
    let mut skills = Vec::new();

    let audit_table: &[(&[&str], &str)] = &[
        (
            &[
                "ux",
                "ui",
                "design audit",
                "audit visuel",
                "audit design",
                "ui/ux",
            ],
            "uiuxaudit",
        ),
        (
            &["refonte", "refontaudit", "redesign dashboard"],
            "refontaudit",
        ),
        (
            &["flow", "user flow", "parcours", "audit flow"],
            "flowaudit",
        ),
        (&["code audit", "code quality", "audit code"], "codeaudit"),
        (
            &["perf", "performance", "core web vitals", "audit perf"],
            "perfaudit",
        ),
        (&["security", "vulnerab", "owasp", "audit sec"], "secaudit"),
        (&["a11y", "accessibility", "wcag"], "a11yaudit"),
        (&["seo", "audit seo", "crawlability"], "seoaudit"),
        (
            &["feature audit", "completeness", "audit feature"],
            "featureaudit",
        ),
        (
            &["copy audit", "messaging audit", "audit copy"],
            "copyaudit",
        ),
        (&["dx audit", "developer experience", "audit dx"], "dxaudit"),
        (
            &["motion audit", "animation audit", "audit motion"],
            "motionaudit",
        ),
        (
            &["data integrity", "data audit", "audit data", "schema audit"],
            "dataaudit",
        ),
        (&["api audit", "audit api", "api contracts"], "apiaudit"),
        (&["debugaudit", "runtime bug", "debug audit"], "debugaudit"),
        (
            &[
                "automation",
                "cron",
                "crontab",
                "scripts audit",
                "daemon health",
            ],
            "automationaudit",
        ),
        (
            &[
                "logic",
                "optimize logic",
                "system optimization",
                "architecture logic",
            ],
            "logicaudit",
        ),
        (
            &[
                "retention",
                "retentionaudit",
                "feature opportunities",
                "make it sticky",
            ],
            "retentionaudit",
        ),
    ];

    for (triggers, skill) in audit_table {
        for trigger in *triggers {
            if lower.contains(trigger) {
                skills.push(AuditSkill {
                    skill: skill.to_string(),
                    trigger: trigger.to_string(),
                });
                break;
            }
        }
    }

    // "full audit" is authoritative: it means run EVERY audit, regardless of
    // any skills already detected above. Source the full list from the audit
    // registry (single source of truth) so routing never drifts out of sync
    // with the Quality Arsenal as audits are added or removed.
    if lower.contains("full audit")
        || lower.contains("audit complet")
        || lower.contains("toutes les audits")
    {
        skills.clear();
        for audit in crate::audit::all_audits() {
            skills.push(AuditSkill {
                skill: audit.id.to_string(),
                trigger: "full audit".to_string(),
            });
        }
    }

    // CODE-TOUCH BASELINE: any mission that changes code gets `/codeaudit` as the
    // baseline floor — even when no audit keyword matched — so a code change is NEVER
    // shipped without the quality gate. Skip for read-only/research missions and avoid
    // duplicating an already-detected codeaudit (or a full-audit run that includes it).
    let touches_code = [
        "fix",
        "implement",
        "build",
        "add",
        "create",
        "refactor",
        "feature",
        "bug",
        "patch",
        "update",
        "change",
        "rewrite",
        "migrate",
        "corrige",
        "implémente",
        "ajoute",
        "code",
        "develop",
        "développe",
        "ship",
        "deploy",
    ]
    .iter()
    .any(|k| lower.contains(k));
    let read_only = lower.contains("research")
        || lower.contains("recherche")
        || lower.contains("read-only")
        || lower.contains("investigate")
        || lower.contains("explain");
    if touches_code && !read_only && !skills.iter().any(|s| s.skill == "codeaudit") {
        skills.push(AuditSkill {
            skill: "codeaudit".to_string(),
            trigger: "code-touch baseline".to_string(),
        });
    }

    skills
}

pub fn classify_mission(mission: &str) -> RoutingDecision {
    // An empty or whitespace-only mission is a user/intake error (typo, empty
    // API body). Reject it explicitly rather than scoring "" as a Simple task
    // and wasting an agent on an empty objective. Signature stays infallible
    // (8 call sites treat it as such); the rejection is surfaced via reasoning.
    if mission.trim().is_empty() {
        return RoutingDecision {
            complexity: Complexity::Simple,
            reasoning: vec!["INVALID: mission text is empty — nothing to route".to_string()],
            suggested_agent: "codex".to_string(),
            decompose: false,
            use_team: false,
            use_quality_gate: false,
            audit_skills: Vec::new(),
            topology: RoutingTopology::SingleAgent,
            risk: RiskLevel::Low,
            confidence: 1.0,
            required_capabilities: Vec::new(),
            recommended_parallelism: 0,
            quality_policy: QualityPolicy::Standard,
            router_version: ROUTER_VERSION.to_string(),
        };
    }

    let lower = mission.to_lowercase();
    let mut reasoning = Vec::new();
    let mut score: i32 = 0;

    // These features describe blast radius and topology in both languages.
    // They intentionally avoid treating a bare "build" or "create" as EPIC.
    let broad_scope_signals = [
        "complete overhaul",
        "refactor everything",
        "redesign all",
        "rewrite everything",
        "entire system",
        "whole system",
        "all laws",
        "all rules",
        "from scratch",
        "ground up",
        "from zero to",
        "0 to 100",
        "refonte complète",
        "refondre tout",
        "revois tout",
        "absolument tout",
        "tout le système",
        "toutes les lois",
        "toutes les règles",
        "partant de 0",
        "de 0 à 100",
    ];
    let architecture_signals = [
        "architecture",
        "architect",
        "orchestration",
        "state machine",
        "event ledger",
        "event store",
        "mission engine",
        "provider",
        "authentication",
        "oauth",
        "jwt",
        "oracle sessions",
        "session oracle",
        "logique du système",
        "gestion des oracles",
        "génération des skills",
        "skills sont générés",
    ];
    let complex_signals = [
        "audit",
        "implement",
        "implément",
        "refactor",
        "redesign",
        "migrate",
        "migration",
        "design",
        "multi-step",
        "multiple files",
        "across multiple",
        "plusieurs fichiers",
        "intégr",
        "refonte",
        "améliore",
    ];
    let simple_signals = [
        "rename",
        "typo",
        "format",
        "lint",
        "comment",
        "delete unused",
        "quick fix",
        "one-line",
        "fix typo",
        "faute de frappe",
        "renommer",
    ];

    let broad_matches: Vec<_> = broad_scope_signals
        .iter()
        .filter(|signal| lower.contains(**signal))
        .collect();
    let architecture_matches: Vec<_> = architecture_signals
        .iter()
        .filter(|signal| lower.contains(**signal))
        .collect();
    let complex_matches: Vec<_> = complex_signals
        .iter()
        .filter(|signal| lower.contains(**signal))
        .collect();

    if let Some(signal) = broad_matches.first() {
        score += 4;
        reasoning.push(format!("Broad-system signal: '{}'", signal));
    }
    if let Some(signal) = architecture_matches.first() {
        score += 2;
        reasoning.push(format!("Architecture signal: '{}'", signal));
    }
    if !architecture_matches.is_empty() && touches_code_mission(&lower) {
        score += 1;
        reasoning.push("Architecture change requires implementation".to_string());
    }
    if let Some(signal) = complex_matches.first() {
        score += 2;
        reasoning.push(format!("Complex-work signal: '{}'", signal));
    }
    if complex_matches.len() >= 3 {
        score += 1;
        reasoning.push(format!(
            "Multi-stage mission ({} independent work signals)",
            complex_matches.len()
        ));
    }
    if let Some(signal) = simple_signals
        .iter()
        .find(|signal| lower.contains(**signal))
    {
        if broad_matches.is_empty() && architecture_matches.is_empty() {
            score -= 2;
            reasoning.push(format!("Bounded/simple signal: '{}'", signal));
        }
    }

    let word_count = mission.split_whitespace().count();
    if word_count > 50 {
        score += 2;
        reasoning.push(format!("Long mission ({} words)", word_count));
    } else if word_count > 20 {
        score += 1;
        reasoning.push(format!("Medium-length mission ({} words)", word_count));
    } else if word_count < 5 {
        score -= 1;
        reasoning.push(format!("Very short mission ({} words)", word_count));
    }

    let file_mentions = lower.matches("file").count()
        + lower.matches(".rs").count()
        + lower.matches(".py").count()
        + lower.matches(".ts").count();
    if file_mentions > 3 {
        score += 1;
        reasoning.push(format!("Multiple file references ({})", file_mentions));
    }

    if lower.contains("audit") {
        reasoning.push("Audit detected — recommend quality gate".to_string());
    }

    let mut complexity = if score >= 5 {
        Complexity::Epic
    } else if score >= 3 {
        Complexity::Complex
    } else if score >= 1 {
        Complexity::Medium
    } else {
        Complexity::Simple
    };

    // @council — an explicit @council mention or a high-stakes / contested decision
    // routes to the COUNCIL — the multi-model Claude council (R-COUNCIL). Advisory only (printed by
    // `omega route`); the council itself is convened via the llm-council skill / Atlas.
    let council_signals = [
        "@council",
        "irreversible",
        "prod-wide",
        "production-wide",
        "force-push",
        "drop the database",
        "drop the prod",
        "architecture decision",
        "cross-project",
        "contested",
        "high-stakes",
        "second opinion",
        "tie-break",
        "conflicting findings",
    ];
    let council_requested = council_signals.iter().any(|s| lower.contains(s));
    if council_requested {
        reasoning.push("Council signal — routing to @council (multi-model council)".to_string());
    }

    let destructive = [
        "drop database",
        "drop the database",
        "truncate",
        "force-push",
        "force push",
        "delete production",
        "wipe",
        "supprimer la production",
        "réinitialiser la base",
    ]
    .iter()
    .any(|signal| lower.contains(signal));
    let production = [
        "production",
        "prod-wide",
        "deploy",
        "ship",
        "déployer",
        "mise en prod",
    ]
    .iter()
    .any(|signal| lower.contains(signal));
    let security = [
        "security",
        "vulnerability",
        "pentest",
        "auth",
        "secret",
        "sécurité",
        "vulnérabilité",
    ]
    .iter()
    .any(|signal| lower.contains(signal));

    let risk = if destructive {
        RiskLevel::Critical
    } else if production
        || security
        || (!broad_matches.is_empty() && !architecture_matches.is_empty())
    {
        RiskLevel::High
    } else if !complex_matches.is_empty() || touches_code_mission(&lower) {
        RiskLevel::Medium
    } else {
        RiskLevel::Low
    };

    let feature_count = broad_matches.len() + architecture_matches.len() + complex_matches.len();
    let mut confidence = (0.55 + feature_count as f32 * 0.07).min(0.98);
    if !broad_matches.is_empty() && !architecture_matches.is_empty() {
        confidence = confidence.max(0.9);
    }
    // Unknown-but-long missions are routed conservatively, not optimistically.
    if confidence < 0.6 && word_count > 20 && matches!(complexity, Complexity::Simple) {
        complexity = Complexity::Medium;
        reasoning.push("Low-confidence long mission escalated conservatively".to_string());
    }

    let topology = if council_requested || risk == RiskLevel::Critical {
        RoutingTopology::Council
    } else {
        match complexity {
            Complexity::Epic => RoutingTopology::ParallelWorkers,
            Complexity::Complex => RoutingTopology::ManagerTools,
            Complexity::Medium | Complexity::Simple => RoutingTopology::SingleAgent,
        }
    };
    let suggested_agent = match topology {
        RoutingTopology::Council => "council",
        RoutingTopology::ManagerTools | RoutingTopology::ParallelWorkers => "codex",
        RoutingTopology::SingleAgent | RoutingTopology::Handoff => "codex",
    };

    let audit_skills = detect_audit_skills(mission);
    if !audit_skills.is_empty() {
        reasoning.push(format!(
            "Audit skills detected: {}",
            audit_skills
                .iter()
                .map(|a| a.skill.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    let quality_policy = match risk {
        RiskLevel::Critical => QualityPolicy::ApprovalRequired,
        RiskLevel::High => QualityPolicy::Adversarial,
        RiskLevel::Medium => QualityPolicy::Verified,
        RiskLevel::Low => QualityPolicy::Standard,
    };
    let mut required_capabilities = vec![
        "launch".to_string(),
        "inspect".to_string(),
        "cancel".to_string(),
        "timeout".to_string(),
    ];
    if matches!(
        topology,
        RoutingTopology::ManagerTools | RoutingTopology::ParallelWorkers
    ) {
        required_capabilities.extend([
            "stable_session_identity".to_string(),
            "progress_events".to_string(),
            "plan_projection".to_string(),
        ]);
    }
    if lower.contains("resume") || lower.contains("reprise") || lower.contains("recovery") {
        required_capabilities.push("resume".to_string());
    }
    if risk >= RiskLevel::High {
        required_capabilities.push("tool_mcp_policy".to_string());
    }
    required_capabilities.sort();
    required_capabilities.dedup();

    let use_quality_gate = complexity >= Complexity::Complex
        || risk >= RiskLevel::High
        || lower.contains("audit")
        || !audit_skills.is_empty();

    RoutingDecision {
        complexity,
        reasoning,
        suggested_agent: suggested_agent.to_string(),
        decompose: matches!(
            topology,
            RoutingTopology::ManagerTools
                | RoutingTopology::ParallelWorkers
                | RoutingTopology::Council
        ),
        use_team: matches!(
            topology,
            RoutingTopology::ParallelWorkers | RoutingTopology::Council
        ),
        use_quality_gate,
        audit_skills,
        topology,
        risk,
        confidence,
        required_capabilities,
        recommended_parallelism: complexity.recommended_agents(),
        quality_policy,
        router_version: ROUTER_VERSION.to_string(),
    }
}

fn touches_code_mission(lower: &str) -> bool {
    [
        "fix",
        "implement",
        "build",
        "add",
        "create",
        "refactor",
        "feature",
        "patch",
        "update",
        "change",
        "rewrite",
        "migrate",
        "corrige",
        "implémente",
        "ajoute",
        "code",
        "develop",
        "développe",
    ]
    .iter()
    .any(|signal| lower.contains(signal))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fix_typo_is_simple() {
        let d = classify_mission("fix typo in README");
        assert_eq!(d.complexity, Complexity::Simple);
    }

    #[test]
    fn build_auth_is_complex_or_epic() {
        let d = classify_mission(
            "build the entire authentication system with OAuth2 and JWT refresh tokens",
        );
        assert!(matches!(
            d.complexity,
            Complexity::Complex | Complexity::Epic
        ));
    }

    #[test]
    fn audit_triggers_quality_gate() {
        let d = classify_mission("audit the security of the API endpoints");
        assert!(d.use_quality_gate);
    }

    #[test]
    fn empty_mission_is_rejected() {
        let d = classify_mission("   ");
        assert!(d.audit_skills.is_empty());
        assert!(!d.use_quality_gate);
        assert!(d.reasoning.iter().any(|r| r.contains("INVALID")));
    }

    #[test]
    fn full_audit_runs_entire_registry() {
        let d = classify_mission("please run a full audit on everything");
        // "full audit" is authoritative and sourced from the registry.
        assert_eq!(d.audit_skills.len(), crate::audit::all_audits().len());
        assert!(d.audit_skills.iter().all(|a| a.trigger == "full audit"));
    }

    #[test]
    fn full_audit_overrides_partial_detection() {
        // Even when specific audits are named, "full audit" expands to all.
        let d = classify_mission("do a security and seo full audit");
        assert_eq!(d.audit_skills.len(), crate::audit::all_audits().len());
    }

    #[test]
    fn council_mention_routes_to_council() {
        let d =
            classify_mission("@council should we drop the prod database — this is irreversible");
        assert_eq!(d.suggested_agent, "council");
    }

    #[test]
    fn normal_mission_does_not_route_to_council() {
        let d = classify_mission("fix typo in README");
        assert_ne!(d.suggested_agent, "council");
        assert_ne!(d.suggested_agent, "morpheus");
        assert_ne!(d.suggested_agent, "oracle");
        assert_eq!(d.suggested_agent, "codex");
    }

    #[test]
    fn french_and_english_system_overhaul_have_routing_parity() {
        let french = classify_mission(
            "Revois toutes les lois, toutes les règles, la génération des skills et la \
             gestion des oracles dans les sessions pour améliorer absolument tout \
             l'orchestration et la logique du système, en partant de 0 à 100.",
        );
        let english = classify_mission(
            "Redesign all laws, all rules, skill generation and Oracle session management \
             to improve the entire system orchestration and logic from zero to 100.",
        );
        assert_eq!(french.complexity, Complexity::Epic);
        assert_eq!(english.complexity, Complexity::Epic);
        assert_eq!(french.topology, english.topology);
        assert_eq!(french.risk, english.risk);
        assert_eq!(french.quality_policy, english.quality_policy);
        assert!(french.use_quality_gate && english.use_quality_gate);
        assert!(french.confidence >= 0.9 && english.confidence >= 0.9);
    }

    #[test]
    fn destructive_mission_requires_council_and_approval() {
        let decision = classify_mission("drop the database in production");
        assert_eq!(decision.risk, RiskLevel::Critical);
        assert_eq!(decision.topology, RoutingTopology::Council);
        assert_eq!(decision.quality_policy, QualityPolicy::ApprovalRequired);
        assert!(decision
            .required_capabilities
            .contains(&"tool_mcp_policy".to_string()));
    }
}

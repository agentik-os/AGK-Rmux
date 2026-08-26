//! Intent parser: transforms casual text into structured IntentJson.
//!
//! Mirrors the VPS `intent-parser.sh` from the Wave-4 Intent-Loop Pipeline.
//! Regex fast-path handles ~80% of messages (zero LLM cost, <1ms).
//! Structured fallback catches the rest.

use serde::{Deserialize, Serialize};

/// Canonical action types that the system can route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntentAction {
    BugFix,
    Feature,
    Audit,
    Ship,
    Refactor,
    Docs,
    Question,
    GodMode,
    Status,
    Other,
}

impl IntentAction {
    pub fn label(&self) -> &'static str {
        match self {
            Self::BugFix => "bug-fix",
            Self::Feature => "feature",
            Self::Audit => "audit",
            Self::Ship => "ship",
            Self::Refactor => "refactor",
            Self::Docs => "docs",
            Self::Question => "question",
            Self::GodMode => "god-mode",
            Self::Status => "status",
            Self::Other => "other",
        }
    }
}

/// Structured intent extracted from raw user text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentJson {
    pub action: IntentAction,
    pub target: Option<String>,
    pub success_criteria: Vec<String>,
    pub verify_commands: Vec<String>,
    pub raw_text: String,
    pub confidence: f32,
    pub detected_projects: Vec<String>,
    pub detected_audits: Vec<String>,
    pub is_multi_project: bool,
}

/// Fast-path regex patterns: (keywords, action, default confidence).
/// Longest match first within each group to avoid partial hits.
const PATTERNS: &[(&[&str], IntentAction, f32)] = &[
    // God mode — check before feature/ship
    (
        &["/godmode", "god mode", "godmode"],
        IntentAction::GodMode,
        0.95,
    ),
    // Status queries
    (
        &["status", "health", "how is", "comment va", "état"],
        IntentAction::Status,
        0.85,
    ),
    // Ship / deploy
    (
        &[
            "ship",
            "deploy",
            "push",
            "merge",
            "envoie en prod",
            "met en prod",
            "livre",
            "publish",
        ],
        IntentAction::Ship,
        0.90,
    ),
    // Audit keywords (high confidence — well-defined)
    (
        &[
            "audit",
            "/codeaudit",
            "/flowaudit",
            "/uiuxaudit",
            "/debugaudit",
            "/featureaudit",
            "/perfaudit",
            "/secaudit",
            "/a11yaudit",
            "/seoaudit",
            "/dataaudit",
            "/apiaudit",
            "/copyaudit",
            "/dxaudit",
            "/motionaudit",
            "/automationaudit",
            "/logicaudit",
            "/retentionaudit",
        ],
        IntentAction::Audit,
        0.95,
    ),
    // Bug fix
    (
        &[
            "fix",
            "bug",
            "broken",
            "crash",
            "error",
            "fail",
            "issue",
            "doesn't work",
            "ne marche pas",
            "ne fonctionne pas",
            "cassé",
            "problème",
            "problem",
            "regression",
            "revert",
        ],
        IntentAction::BugFix,
        0.80,
    ),
    // Feature
    (
        &[
            "add",
            "create",
            "build",
            "implement",
            "new feature",
            "ajoute",
            "crée",
            "construis",
            "implémente",
            "nouvelle feature",
        ],
        IntentAction::Feature,
        0.75,
    ),
    // Refactor
    (
        &[
            "refactor",
            "clean up",
            "restructure",
            "reorganize",
            "simplify",
            "optimize",
            "améliore",
            "nettoie",
            "réorganise",
        ],
        IntentAction::Refactor,
        0.80,
    ),
    // Docs
    (
        &[
            "document",
            "readme",
            "docs",
            "documentation",
            "write docs",
            "update docs",
            "changelog",
        ],
        IntentAction::Docs,
        0.85,
    ),
    // Question — weakest signal, checked last
    (
        &[
            "what",
            "how",
            "why",
            "where",
            "when",
            "can you",
            "explain",
            "tell me",
            "show me",
            "list",
            "which",
            "est-ce que",
            "comment",
            "pourquoi",
            "qu'est-ce",
        ],
        IntentAction::Question,
        0.60,
    ),
];

pub struct IntentParser {
    project_keywords: Vec<(String, Vec<String>)>,
}

impl IntentParser {
    pub fn new() -> Self {
        Self {
            project_keywords: Vec::new(),
        }
    }

    /// Register a project with its name and keyword aliases for detection.
    pub fn register_project(&mut self, name: &str, aliases: Vec<String>) {
        self.project_keywords.push((name.to_string(), aliases));
    }

    /// Parse raw text into structured IntentJson.
    pub fn parse(&self, text: &str) -> IntentJson {
        let lower = text.to_lowercase();
        let trimmed = text.trim();

        // Detect projects mentioned in the text
        let detected_projects = self.detect_projects(&lower);
        let is_multi_project = detected_projects.len() > 1;

        // Detect audit skill invocations
        let detected_audits = detect_audit_names(&lower);

        // Fast-path: regex keyword matching
        let (action, confidence) = self.classify_fast(&lower, &detected_audits);

        // Extract target (the "what" after the action verb)
        let target = extract_target(trimmed, action);

        // Generate success criteria from action + target
        let success_criteria = generate_criteria(action, target.as_deref());

        // Generate verify commands
        let verify_commands = generate_verify_commands(action);

        IntentJson {
            action,
            target,
            success_criteria,
            verify_commands,
            raw_text: trimmed.to_string(),
            confidence,
            detected_projects,
            detected_audits,
            is_multi_project,
        }
    }

    fn classify_fast(&self, lower: &str, audits: &[String]) -> (IntentAction, f32) {
        // Direct slash command → highest confidence
        if lower.starts_with('/') {
            for &(keywords, action, conf) in PATTERNS {
                for kw in keywords {
                    if lower.starts_with(kw) {
                        return (action, conf);
                    }
                }
            }
        }

        // Audit detection overrides other signals
        if !audits.is_empty() {
            return (IntentAction::Audit, 0.95);
        }

        // Score each pattern group
        let mut best: Option<(IntentAction, f32)> = None;
        for &(keywords, action, base_conf) in PATTERNS {
            for kw in keywords {
                if lower.contains(kw) {
                    let conf = if lower.starts_with(kw) {
                        base_conf
                    } else {
                        base_conf * 0.9
                    };
                    if best.is_none_or(|(_, c)| conf > c) {
                        best = Some((action, conf));
                    }
                    break;
                }
            }
        }

        best.unwrap_or((IntentAction::Other, 0.30))
    }

    fn detect_projects(&self, lower: &str) -> Vec<String> {
        let mut found = Vec::new();
        for (name, aliases) in &self.project_keywords {
            // Check name with word boundary approximation
            if word_boundary_match(lower, &name.to_lowercase()) {
                found.push(name.clone());
                continue;
            }
            for alias in aliases {
                if word_boundary_match(lower, &alias.to_lowercase()) {
                    found.push(name.clone());
                    break;
                }
            }
        }
        found
    }
}

impl Default for IntentParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Word-boundary check: the keyword must not be a substring of a larger word.
fn word_boundary_match(haystack: &str, needle: &str) -> bool {
    let bytes = haystack.as_bytes();
    let needle_bytes = needle.as_bytes();
    let nlen = needle_bytes.len();

    let mut start = 0;
    while let Some(pos) = haystack[start..].find(needle) {
        let abs = start + pos;
        let before_ok = abs == 0 || !bytes[abs - 1].is_ascii_alphanumeric();
        let after_pos = abs + nlen;
        let after_ok = after_pos >= bytes.len() || !bytes[after_pos].is_ascii_alphanumeric();
        if before_ok && after_ok {
            return true;
        }
        start = abs + 1;
        if start >= haystack.len() {
            break;
        }
    }
    false
}

/// Detect which audit skills are referenced in the text.
fn detect_audit_names(lower: &str) -> Vec<String> {
    let table: &[(&[&str], &str)] = &[
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
            &["logic", "optimize logic", "system optimization"],
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

    // "full audit" → EVERY Quality Arsenal audit. Delegate to the audit
    // registry (crate::audit::all_audits — the single source of truth) rather
    // than this local trigger table: the table had drifted to 17 while the
    // registry grew to 23, so the two "full audit" paths disagreed on which
    // audits run. The registry's own test (select_full_audit_returns_all) locks
    // the count, so this path now always matches it.
    if lower.contains("full audit")
        || lower.contains("audit complet")
        || lower.contains("toutes les audits")
    {
        return crate::audit::all_audits()
            .iter()
            .map(|a| a.id.to_string())
            .collect();
    }

    let mut found = Vec::new();
    for (triggers, name) in table {
        for trigger in *triggers {
            if lower.contains(trigger) {
                found.push(name.to_string());
                break;
            }
        }
    }
    found
}

/// Extract the target object from the raw text based on the action.
fn extract_target(text: &str, action: IntentAction) -> Option<String> {
    let lower = text.to_lowercase();

    // For slash commands, target is everything after the command
    if lower.starts_with('/') {
        let rest = text.split_once(' ')?.1;
        let trimmed = rest.trim();
        if trimmed.is_empty() {
            return None;
        }
        return Some(trimmed.to_string());
    }

    // For natural language, extract what comes after the action verb
    let verbs: &[&str] = match action {
        IntentAction::BugFix => &["fix", "repair", "debug", "resolve", "corrige", "répare"],
        IntentAction::Feature => &["add", "create", "build", "implement", "ajoute", "crée"],
        IntentAction::Refactor => &["refactor", "clean", "restructure", "simplify", "optimize"],
        IntentAction::Ship => &["ship", "deploy", "push", "merge", "publish"],
        IntentAction::Docs => &["document", "write docs for", "update docs for"],
        _ => return Some(text.to_string()),
    };

    for verb in verbs {
        if let Some(pos) = lower.find(verb) {
            let after = &text[pos + verb.len()..].trim_start();
            if !after.is_empty() {
                return Some(after.to_string());
            }
        }
    }

    Some(text.to_string())
}

/// Generate success criteria based on action type.
fn generate_criteria(action: IntentAction, target: Option<&str>) -> Vec<String> {
    let mut criteria = Vec::new();

    match action {
        IntentAction::BugFix => {
            if let Some(t) = target {
                criteria.push(format!("The issue with {} is resolved", t));
            }
            criteria.push("npm run build / cargo build = 0 errors".to_string());
            criteria.push("No console errors related to the fix".to_string());
        }
        IntentAction::Feature => {
            if let Some(t) = target {
                criteria.push(format!("{} is implemented and functional", t));
            }
            criteria.push("Build passes with no errors".to_string());
            criteria.push("Feature is visually verifiable or test-covered".to_string());
        }
        IntentAction::Audit => {
            criteria.push("Audit completes all phases with score".to_string());
            criteria.push("Findings documented with file:line citations".to_string());
        }
        IntentAction::Ship => {
            criteria.push("Build passes".to_string());
            criteria.push("Git push succeeds".to_string());
            criteria.push("Deploy URL returns 200".to_string());
        }
        IntentAction::Refactor => {
            criteria.push("Existing tests still pass".to_string());
            criteria.push("Build passes with no errors".to_string());
            criteria.push("No behavioral regressions".to_string());
        }
        IntentAction::Docs => {
            criteria.push("Documentation is accurate and complete".to_string());
        }
        IntentAction::GodMode => {
            criteria.push("All phases complete".to_string());
            criteria.push("Build passes".to_string());
            criteria.push("Verification audit passes".to_string());
        }
        IntentAction::Question | IntentAction::Status | IntentAction::Other => {}
    }

    criteria
}

/// Generate verify commands based on action type.
fn generate_verify_commands(action: IntentAction) -> Vec<String> {
    match action {
        IntentAction::BugFix | IntentAction::Feature | IntentAction::Refactor => {
            vec!["cargo build --release 2>&1 | tail -3".to_string()]
        }
        IntentAction::Ship => {
            vec![
                "cargo build --release 2>&1 | tail -3".to_string(),
                "git log --oneline -1".to_string(),
            ]
        }
        IntentAction::Audit => {
            vec!["echo 'Audit score recorded in verdict.json'".to_string()]
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parser_with_projects() -> IntentParser {
        let mut p = IntentParser::new();
        p.register_project("Causio", vec!["causio".into()]);
        p.register_project("Kommu", vec!["kommu".into()]);
        p.register_project("DentistryGPT", vec!["dent".into(), "dentistry".into()]);
        p.register_project("OmegaVPS", vec!["omega".into()]);
        p
    }

    #[test]
    fn fix_login_is_bugfix() {
        let p = parser_with_projects();
        let intent = p.parse("fix le login");
        assert_eq!(intent.action, IntentAction::BugFix);
        assert!(intent.confidence >= 0.70);
    }

    #[test]
    fn slash_godmode_detected() {
        let p = IntentParser::new();
        let intent = p.parse("/godmode Kommu fix all auth");
        assert_eq!(intent.action, IntentAction::GodMode);
        assert!(intent.confidence >= 0.90);
    }

    #[test]
    fn audit_keyword_detected() {
        let p = IntentParser::new();
        let intent = p.parse("fais un audit UX et code sur Causio");
        assert_eq!(intent.action, IntentAction::Audit);
        assert!(!intent.detected_audits.is_empty());
    }

    #[test]
    fn full_audit_returns_all_registry_audits() {
        let p = IntentParser::new();
        let intent = p.parse("lance un full audit");
        // Must equal the audit registry exactly (single source of truth), not a
        // hardcoded count that drifts when audits are added. Pre-fix this path
        // returned 17 (a stale local table) while the registry held 23.
        assert_eq!(
            intent.detected_audits.len(),
            crate::audit::all_audits().len()
        );
        assert_eq!(intent.detected_audits.len(), 23);
    }

    #[test]
    fn multi_project_detection() {
        let p = parser_with_projects();
        let intent = p.parse("fix le login sur Causio et Kommu");
        assert!(intent.is_multi_project);
        assert!(intent.detected_projects.contains(&"Causio".to_string()));
        assert!(intent.detected_projects.contains(&"Kommu".to_string()));
    }

    #[test]
    fn single_project_detection() {
        let p = parser_with_projects();
        let intent = p.parse("deploy Causio en prod");
        assert!(!intent.is_multi_project);
        assert_eq!(intent.detected_projects, vec!["Causio".to_string()]);
    }

    #[test]
    fn no_project_when_none_mentioned() {
        let p = parser_with_projects();
        let intent = p.parse("fix the sidebar layout");
        assert!(intent.detected_projects.is_empty());
    }

    #[test]
    fn ship_action_detected() {
        let p = IntentParser::new();
        let intent = p.parse("ship it to production");
        assert_eq!(intent.action, IntentAction::Ship);
    }

    #[test]
    fn question_is_low_confidence() {
        let p = IntentParser::new();
        let intent = p.parse("what is the current state of auth?");
        assert_eq!(intent.action, IntentAction::Question);
        assert!(intent.confidence < 0.70);
    }

    #[test]
    fn word_boundary_avoids_false_positives() {
        // "perform" contains "perf" — should NOT trigger perfaudit
        // But our audit detection uses `contains` which will match.
        // The word_boundary_match function is for project names, not audit triggers.
        let p = parser_with_projects();
        let intent = p.parse("DentistryGPT needs a fix");
        assert!(intent
            .detected_projects
            .contains(&"DentistryGPT".to_string()));
        // "dent" alias should NOT match "indent"
        let intent2 = p.parse("indent the code properly");
        assert!(!intent2
            .detected_projects
            .contains(&"DentistryGPT".to_string()));
    }

    #[test]
    fn target_extraction_for_bugfix() {
        let p = IntentParser::new();
        let intent = p.parse("fix the login redirect loop");
        assert_eq!(intent.target.as_deref(), Some("the login redirect loop"));
    }

    #[test]
    fn criteria_generated_for_ship() {
        let p = IntentParser::new();
        let intent = p.parse("deploy to production");
        assert!(intent.success_criteria.iter().any(|c| c.contains("200")));
    }

    #[test]
    fn slash_command_target_extraction() {
        let p = IntentParser::new();
        let intent = p.parse("/codeaudit --files=src/auth.rs");
        assert_eq!(intent.action, IntentAction::Audit);
        assert_eq!(intent.target.as_deref(), Some("--files=src/auth.rs"));
    }
}

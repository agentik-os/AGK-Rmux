//! Quality Arsenal audit registry — typed catalogue of the 23 Gestalt-Popper
//! forensic audits that form OmegaOS's quality infrastructure.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditDomain {
    Code,
    Flow,
    Design,
    Runtime,
    Feature,
    Performance,
    Security,
    Accessibility,
    Seo,
    Data,
    Api,
    Copy,
    Dx,
    Motion,
    Automation,
    Logic,
    Retention,
    Observability,
    Dependencies,
    Localization,
    Release,
    Privacy,
}

impl AuditDomain {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Code => "Code",
            Self::Flow => "Flow",
            Self::Design => "Design",
            Self::Runtime => "Runtime",
            Self::Feature => "Feature",
            Self::Performance => "Performance",
            Self::Security => "Security",
            Self::Accessibility => "Accessibility",
            Self::Seo => "SEO",
            Self::Data => "Data",
            Self::Api => "API",
            Self::Copy => "Copy",
            Self::Dx => "DX",
            Self::Motion => "Motion",
            Self::Automation => "Automation",
            Self::Logic => "Logic",
            Self::Retention => "Retention",
            Self::Observability => "Observability",
            Self::Dependencies => "Dependencies",
            Self::Localization => "Localization",
            Self::Release => "Release",
            Self::Privacy => "Privacy",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuditSkill {
    pub id: &'static str,
    pub name: &'static str,
    pub domain: AuditDomain,
    pub phases: u32,
    pub max_score: u32,
    pub normalized_max: u32,
    pub description: &'static str,
    pub triggers: &'static [&'static str],
    pub skill_path: &'static str,
    pub read_only: bool,
}

impl AuditSkill {
    pub fn normalized_score(&self, raw: f32) -> f32 {
        if self.max_score == 0 {
            return 0.0;
        }
        (raw / self.max_score as f32) * self.normalized_max as f32
    }

    pub fn matches_text(&self, text: &str) -> bool {
        let lower = text.to_lowercase();
        self.triggers.iter().any(|t| lower.contains(t))
    }
}

#[derive(Debug, Deserialize)]
struct AuditRegistry {
    meta: AuditRegistryMeta,
    audits: Vec<AuditRegistryEntry>,
}

#[derive(Debug, Deserialize)]
struct AuditRegistryMeta {
    total_audits: usize,
}

#[derive(Debug, Deserialize)]
struct AuditRegistryEntry {
    id: String,
    name: String,
    domain: String,
    phases: u32,
    max_score: u32,
    triggers: Vec<String>,
    read_only: bool,
    answers: String,
}

const AUDIT_REGISTRY_TOML: &str = include_str!("../../../skills/audits/registry.toml");

fn parse_domain(value: &str) -> Result<AuditDomain, String> {
    match value {
        "code" => Ok(AuditDomain::Code),
        "flow" | "flows" => Ok(AuditDomain::Flow),
        "design" => Ok(AuditDomain::Design),
        "runtime" => Ok(AuditDomain::Runtime),
        "feature" | "features" => Ok(AuditDomain::Feature),
        "performance" => Ok(AuditDomain::Performance),
        "security" => Ok(AuditDomain::Security),
        "accessibility" => Ok(AuditDomain::Accessibility),
        "seo" => Ok(AuditDomain::Seo),
        "data" => Ok(AuditDomain::Data),
        "api" => Ok(AuditDomain::Api),
        "copy" => Ok(AuditDomain::Copy),
        "dx" => Ok(AuditDomain::Dx),
        "motion" => Ok(AuditDomain::Motion),
        "automation" => Ok(AuditDomain::Automation),
        "logic" => Ok(AuditDomain::Logic),
        "retention" => Ok(AuditDomain::Retention),
        "observability" => Ok(AuditDomain::Observability),
        "dependencies" => Ok(AuditDomain::Dependencies),
        "localization" => Ok(AuditDomain::Localization),
        "release" => Ok(AuditDomain::Release),
        "privacy" => Ok(AuditDomain::Privacy),
        other => Err(format!("unknown audit domain: {other}")),
    }
}

fn leak_string(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}

fn load_canonical_audits() -> Result<Vec<AuditSkill>, String> {
    let registry: AuditRegistry =
        toml::from_str(AUDIT_REGISTRY_TOML).map_err(|error| error.to_string())?;
    if registry.meta.total_audits != registry.audits.len() {
        return Err(format!(
            "registry meta declares {} audits but contains {}",
            registry.meta.total_audits,
            registry.audits.len()
        ));
    }

    let mut ids = HashSet::new();
    let mut audits = Vec::with_capacity(registry.audits.len());
    for entry in registry.audits {
        if entry.id.trim().is_empty()
            || entry.name.trim().is_empty()
            || entry.answers.trim().is_empty()
            || entry.triggers.is_empty()
            || entry.phases == 0
            || entry.max_score == 0
        {
            return Err(format!("audit {} has incomplete metadata", entry.id));
        }
        if !ids.insert(entry.id.clone()) {
            return Err(format!("duplicate audit id: {}", entry.id));
        }

        let domain = parse_domain(&entry.domain)?;
        let id = leak_string(entry.id);
        let triggers: Vec<&'static str> = entry.triggers.into_iter().map(leak_string).collect();
        audits.push(AuditSkill {
            id,
            name: leak_string(entry.name),
            domain,
            phases: entry.phases,
            max_score: entry.max_score,
            normalized_max: 100,
            description: leak_string(entry.answers),
            triggers: Box::leak(triggers.into_boxed_slice()),
            skill_path: leak_string(format!("audits/{id}/SKILL.md")),
            read_only: entry.read_only,
        });
    }
    Ok(audits)
}

fn canonical_audits() -> &'static Vec<AuditSkill> {
    static AUDITS: OnceLock<Vec<AuditSkill>> = OnceLock::new();
    AUDITS.get_or_init(|| {
        load_canonical_audits()
            .unwrap_or_else(|error| panic!("invalid embedded audit registry: {error}"))
    })
}

/// Return the Quality Arsenal from the machine-readable TOML registry.
///
/// `skills/audits/registry.toml` is the only source of truth. The Rust
/// catalogue is parsed from that file at runtime and cached for the process.
pub fn all_audits() -> Vec<AuditSkill> {
    canonical_audits().clone()
}

/// Tokenize text into lowercase alphanumeric words (splitting on any
/// non-alphanumeric character).
fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(|w| w.to_string())
        .collect()
}

/// True if `phrase` (one or more words) appears as a contiguous run of whole
/// words inside `words`. Single-word phrases match a single token; multi-word
/// phrases must match consecutively.
fn phrase_matches(words: &[String], phrase: &str) -> bool {
    let needle = tokenize(phrase);
    if needle.is_empty() {
        return false;
    }
    words.windows(needle.len()).any(|w| w == needle.as_slice())
}

/// Select audits relevant to a mission based on keyword matching.
pub fn select_audits(mission_text: &str, _files: &[String]) -> Vec<&'static str> {
    let audits = canonical_audits();
    let lower = mission_text.to_lowercase();
    let words = tokenize(mission_text);

    // "full audit" / "audit complet" → every registered audit (all 23)
    if lower.contains("full audit")
        || lower.contains("audit complet")
        || lower.contains("toutes les audits")
    {
        return audits.iter().map(|a| a.id).collect();
    }

    let mut selected: Vec<&'static str> = audits
        .iter()
        .filter(|a| a.triggers.iter().any(|t| phrase_matches(&words, t)))
        .map(|a| a.id)
        .collect();

    // Composite bundles: security → secaudit + apiaudit + dataaudit
    if phrase_matches(&words, "security") || phrase_matches(&words, "auth") {
        for extra in &["secaudit", "apiaudit", "dataaudit"] {
            if !selected.contains(extra) {
                selected.push(extra);
            }
        }
    }

    // UI changes → uiuxaudit + a11yaudit + motionaudit
    if phrase_matches(&words, "ui")
        || phrase_matches(&words, "ux")
        || phrase_matches(&words, "design")
    {
        for extra in &["uiuxaudit", "a11yaudit", "motionaudit"] {
            if !selected.contains(extra) {
                selected.push(extra);
            }
        }
    }

    // Default end-of-mission minimum quality gate
    if selected.is_empty() {
        selected.push("codeaudit");
        selected.push("debugaudit");
    }

    selected
}

/// Select all audits for a specific domain.
pub fn select_audits_for_domain(domain: AuditDomain) -> Vec<AuditSkill> {
    all_audits()
        .into_iter()
        .filter(|a| a.domain == domain)
        .collect()
}

/// Find a single audit by id.
pub fn find_audit(id: &str) -> Option<AuditSkill> {
    all_audits().into_iter().find(|a| a.id == id)
}

// ── Result types ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditConfidence {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditVerdict {
    Pass,
    NeedsWork,
    Fail,
    Aborted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditResult {
    pub audit_id: String,
    pub raw_score: f32,
    pub max_score: u32,
    pub normalized_score: f32,
    pub confidence: AuditConfidence,
    pub verdict: AuditVerdict,
    pub findings_count: u32,
    pub critical_findings: u32,
    pub worker_session: Option<String>,
    pub completed_at: DateTime<Utc>,
}

impl AuditResult {
    pub fn new(audit_id: &str, raw_score: f32, max_score: u32) -> Self {
        let normalized = if max_score > 0 {
            (raw_score / max_score as f32) * 100.0
        } else {
            0.0
        };
        let verdict = if normalized >= 80.0 {
            AuditVerdict::Pass
        } else if normalized >= 50.0 {
            AuditVerdict::NeedsWork
        } else {
            AuditVerdict::Fail
        };
        Self {
            audit_id: audit_id.to_string(),
            raw_score,
            max_score,
            normalized_score: normalized,
            confidence: AuditConfidence::Medium,
            verdict,
            findings_count: 0,
            critical_findings: 0,
            worker_session: None,
            completed_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub mission_id: String,
    pub audits: Vec<AuditResult>,
    pub overall_score: f32,
    pub overall_verdict: AuditVerdict,
    pub created_at: DateTime<Utc>,
}

impl AuditReport {
    pub fn from_results(mission_id: &str, audits: Vec<AuditResult>) -> Self {
        let overall_score = if audits.is_empty() {
            0.0
        } else {
            audits.iter().map(|a| a.normalized_score).sum::<f32>() / audits.len() as f32
        };
        let overall_verdict = if audits.is_empty() {
            AuditVerdict::Aborted
        } else if audits.iter().all(|a| a.verdict == AuditVerdict::Pass) {
            AuditVerdict::Pass
        } else if audits.iter().any(|a| a.verdict == AuditVerdict::Fail) {
            AuditVerdict::Fail
        } else {
            AuditVerdict::NeedsWork
        };
        Self {
            mission_id: mission_id.to_string(),
            audits,
            overall_score,
            overall_verdict,
            created_at: Utc::now(),
        }
    }

    pub fn all_pass(&self, threshold: f32) -> bool {
        self.audits.iter().all(|a| a.normalized_score >= threshold)
    }

    /// Format the report as Telegram-ready HTML using the formatting engine.
    pub fn to_telegram_html(&self) -> String {
        use crate::formatting;

        let verdict_icon = match self.overall_verdict {
            AuditVerdict::Pass => "✅",
            AuditVerdict::NeedsWork => "🟡",
            AuditVerdict::Fail => "🔴",
            AuditVerdict::Aborted => "⚫",
        };

        let mut out = format!(
            "{} <b>Quality Report — {}</b>\n\
             <b>Overall:</b> <code>{:.0}/100</code>\n\
             ━━━━━━━━━━\n",
            verdict_icon,
            formatting::escape_html(&self.mission_id),
            self.overall_score,
        );

        let scores: Vec<(&str, f32, f32)> = self
            .audits
            .iter()
            .map(|a| {
                let name = find_audit(&a.audit_id).map(|s| s.name).unwrap_or("Unknown");
                (name, a.raw_score, a.max_score as f32)
            })
            .collect();

        out.push_str(&formatting::format_audit_scores(&scores));
        out
    }
}

impl AuditSkill {
    /// Generate a dispatch prompt for invoking this audit in a worker session.
    /// The first line is always the skill slash command.
    pub fn dispatch_prompt(&self, scope: &AuditScope) -> String {
        let mut prompt = format!("/{}", self.id);

        if let Some(url) = &scope.url {
            prompt.push_str(&format!(" --url={}", url));
        }
        if !scope.files.is_empty() {
            prompt.push_str(&format!(" --files={}", scope.files.join(",")));
        }
        if let Some(desc) = &scope.description {
            prompt.push_str(&format!(" --scope=\"{}\"", desc));
        }

        prompt.push_str("\n\ndo not expand beyond this scope");
        prompt
    }
}

/// Scoping parameters for a targeted audit.
#[derive(Debug, Clone, Default)]
pub struct AuditScope {
    pub url: Option<String>,
    pub files: Vec<String>,
    pub description: Option<String>,
}

/// Build a list of dispatch prompts for all audits selected for a mission.
pub fn build_dispatch_prompts(mission_text: &str, scope: &AuditScope) -> Vec<(String, String)> {
    let selected_ids = select_audits(mission_text, &scope.files.clone());
    selected_ids
        .into_iter()
        .filter_map(|id| {
            find_audit(id).map(|audit| {
                let prompt = audit.dispatch_prompt(scope);
                (id.to_string(), prompt)
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_23_audits() {
        assert_eq!(all_audits().len(), 23);
    }

    #[test]
    fn every_audit_has_metadata() {
        for a in all_audits() {
            assert!(!a.id.is_empty());
            assert!(!a.name.is_empty());
            assert!(!a.description.is_empty());
            assert!(a.phases > 0);
            assert!(a.max_score > 0);
            assert_eq!(a.normalized_max, 100);
            assert!(!a.triggers.is_empty());
            assert!(!a.skill_path.is_empty());
        }
    }

    #[test]
    fn unique_ids() {
        let audits = all_audits();
        let ids: Vec<&str> = audits.iter().map(|a| a.id).collect();
        let mut deduped = ids.clone();
        deduped.sort();
        deduped.dedup();
        assert_eq!(ids.len(), deduped.len());
    }

    #[test]
    fn retentionaudit_is_read_only() {
        let audits = all_audits();
        let retention = audits.iter().find(|a| a.id == "retentionaudit").unwrap();
        assert!(retention.read_only);
        let non_readonly = audits.iter().filter(|a| a.id != "retentionaudit");
        for a in non_readonly {
            assert!(!a.read_only, "{} should not be read_only", a.id);
        }
    }

    #[test]
    fn select_full_audit_returns_all() {
        let selected = select_audits("full audit of the project", &[]);
        assert_eq!(selected.len(), 23);
    }

    #[test]
    fn select_security_mission() {
        let selected = select_audits("fix auth flow security", &[]);
        assert!(selected.contains(&"secaudit"));
        assert!(selected.contains(&"apiaudit"));
        assert!(selected.contains(&"dataaudit"));
    }

    #[test]
    fn select_default_minimum() {
        let selected = select_audits("do something generic", &[]);
        assert!(selected.contains(&"codeaudit"));
        assert!(selected.contains(&"debugaudit"));
    }

    #[test]
    fn select_ui_mission() {
        let selected = select_audits("redesign the UI dashboard", &[]);
        assert!(selected.contains(&"uiuxaudit"));
        assert!(selected.contains(&"a11yaudit"));
        assert!(selected.contains(&"motionaudit"));
    }

    #[test]
    fn select_audits_for_domain_works() {
        let code_audits = select_audits_for_domain(AuditDomain::Code);
        assert_eq!(code_audits.len(), 1);
        assert_eq!(code_audits[0].id, "codeaudit");
    }

    #[test]
    fn find_audit_works() {
        assert!(find_audit("codeaudit").is_some());
        assert!(find_audit("nonexistent").is_none());
    }

    #[test]
    fn audit_result_scoring() {
        let r = AuditResult::new("codeaudit", 336.0, 420);
        assert!((r.normalized_score - 80.0).abs() < 0.1);
        assert_eq!(r.verdict, AuditVerdict::Pass);
    }

    #[test]
    fn audit_report_all_pass() {
        let results = vec![
            AuditResult::new("codeaudit", 336.0, 420),
            AuditResult::new("debugaudit", 288.0, 360),
        ];
        let report = AuditReport::from_results("m-test", results);
        assert!(report.all_pass(70.0));
        assert_eq!(report.overall_verdict, AuditVerdict::Pass);
    }

    #[test]
    fn normalized_score_calculation() {
        let skill = find_audit("codeaudit").unwrap();
        let score = skill.normalized_score(210.0);
        assert!((score - 50.0).abs() < 0.1);
    }

    #[test]
    fn audit_report_telegram_html() {
        let results = vec![
            AuditResult::new("codeaudit", 336.0, 420),
            AuditResult::new("secaudit", 200.0, 400),
        ];
        let report = AuditReport::from_results("m-test", results);
        let html = report.to_telegram_html();
        assert!(html.contains("Quality Report"));
        assert!(html.contains("m-test"));
    }

    #[test]
    fn dispatch_prompt_with_scope() {
        let audit = find_audit("codeaudit").unwrap();
        let scope = AuditScope {
            url: Some("https://app.example.com/dashboard".to_string()),
            files: vec!["src/auth.rs".to_string()],
            description: Some("auth module only".to_string()),
        };
        let prompt = audit.dispatch_prompt(&scope);
        assert!(prompt.starts_with("/codeaudit"));
        assert!(prompt.contains("--url="));
        assert!(prompt.contains("--files="));
        assert!(prompt.contains("--scope="));
        assert!(prompt.contains("do not expand beyond this scope"));
    }

    #[test]
    fn dispatch_prompt_minimal_scope() {
        let audit = find_audit("debugaudit").unwrap();
        let scope = AuditScope::default();
        let prompt = audit.dispatch_prompt(&scope);
        assert!(prompt.starts_with("/debugaudit"));
        assert!(!prompt.contains("--url="));
        assert!(!prompt.contains("--files="));
    }

    #[test]
    fn build_dispatch_prompts_security() {
        let scope = AuditScope {
            url: Some("https://app.example.com".to_string()),
            ..Default::default()
        };
        let prompts = build_dispatch_prompts("fix auth flow security", &scope);
        assert!(prompts.iter().any(|(id, _)| id == "secaudit"));
        assert!(prompts.iter().any(|(id, _)| id == "apiaudit"));
        for (_, prompt) in &prompts {
            assert!(prompt.contains("--url="));
        }
    }
}

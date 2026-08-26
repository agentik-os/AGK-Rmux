//! Agent Registry + Team Builder.
//!
//! Agent Definition = reusable specialist (this registry).
//! Sub-Agent = delegated actor for one mission.
//! Agent Run = one concrete execution.
//! Ephemeral Worker = temporary specialist created for one mission.
//!
//! Distinct from the 15 Matrix AISB templates in `aisb_agents`.

use crate::tenancy::AgkScope;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorKind {
    Definition,
    SubAgent,
    AgentRun,
    EphemeralWorker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Specialist {
    pub id: &'static str,
    pub role: &'static str,
    pub scopes: &'static [AgkScope],
    pub capabilities: &'static [&'static str],
    pub skills: &'static [&'static str],
    pub tools: &'static [&'static str],
    pub preferred_models: &'static [&'static str],
    pub permissions: &'static [&'static str],
    pub inputs: &'static [&'static str],
    pub outputs: &'static [&'static str],
    pub handoff: &'static str,
    pub eval: &'static str,
}

const AGK: &[AgkScope] = &[AgkScope::Agentik, AgkScope::Mission, AgkScope::Private];
const AGK_MISSION: &[AgkScope] = &[AgkScope::Agentik, AgkScope::Mission];
const OP: &[AgkScope] = &[AgkScope::Operator];

fn all_specialists() -> Vec<Specialist> {
    vec![
        spec(
            "researcher",
            "Researcher",
            AGK,
            &["research", "sources"],
            "Pass citations to synthesizer or critic",
        ),
        spec(
            "strategist",
            "Strategist",
            AGK,
            &["strategy", "roadmap"],
            "Hand plan to architect or builder",
        ),
        spec(
            "architect",
            "Architect",
            AGK_MISSION,
            &["design", "interfaces"],
            "Hand spec to builder",
        ),
        spec(
            "builder",
            "Builder",
            AGK_MISSION,
            &["implement"],
            "Hand diff to reviewer",
        ),
        spec(
            "claude-builder",
            "Claude Builder",
            AGK_MISSION,
            &["implement"],
            "Prefer Claude; handoff to reviewer",
        ),
        spec(
            "codex-builder",
            "Codex Builder",
            AGK_MISSION,
            &["implement"],
            "Prefer Codex; handoff to reviewer",
        ),
        spec(
            "reviewer",
            "Reviewer",
            AGK,
            &["review"],
            "Approve or return to builder",
        ),
        spec(
            "critic",
            "Critic",
            AGK,
            &["challenge", "sources"],
            "Falsify claims before report",
        ),
        spec(
            "qa",
            "QA",
            AGK_MISSION,
            &["test"],
            "File failures to builder",
        ),
        spec(
            "security",
            "Security",
            AGK_MISSION,
            &["security"],
            "Block ship on critical findings",
        ),
        spec(
            "deployment",
            "Deployment",
            AGK_MISSION,
            &["deploy"],
            "Requires approval on production",
        ),
        spec(
            "content",
            "Content",
            AGK,
            &["write"],
            "Hand draft to reviewer",
        ),
        spec(
            "growth",
            "Growth",
            AGK,
            &["growth"],
            "Hand experiments to reporter",
        ),
        spec(
            "finance",
            "Finance",
            &[AgkScope::Agentik, AgkScope::Private],
            &["finance"],
            "Financial actions need approval",
        ),
        spec(
            "reporter",
            "Reporter",
            AGK,
            &["report"],
            "Publish artifacts, never raw logs",
        ),
        spec(
            "research-planner",
            "Research Planner",
            AGK,
            &["plan"],
            "Spawn researchers then critic",
        ),
        spec(
            "source-critic",
            "Source Critic",
            AGK,
            &["sources"],
            "Reject weak sources",
        ),
        spec(
            "synthesizer",
            "Synthesizer",
            AGK,
            &["synthesize"],
            "Hand brief to strategist or reporter",
        ),
        spec(
            "tester",
            "Tester",
            AGK_MISSION,
            &["test"],
            "Same as QA for builder OS",
        ),
        spec(
            "sre",
            "Operator SRE",
            OP,
            &["sre"],
            "Admin Action Registry only — never raw root",
        ),
    ]
}

pub fn registry() -> &'static [Specialist] {
    static REG: std::sync::OnceLock<Vec<Specialist>> = std::sync::OnceLock::new();
    REG.get_or_init(all_specialists).as_slice()
}

fn spec(
    id: &'static str,
    role: &'static str,
    scopes: &'static [AgkScope],
    capabilities: &'static [&'static str],
    handoff: &'static str,
) -> Specialist {
    Specialist {
        id,
        role,
        scopes,
        capabilities,
        skills: capabilities,
        tools: &["hermes", "rmux"],
        preferred_models: preferred_for(id),
        permissions: &["workspace"],
        inputs: &["brief", "context"],
        outputs: &["artifact", "handoff"],
        handoff,
        eval: "task complete + evidence cited",
    }
}

fn preferred_for(id: &str) -> &'static [&'static str] {
    match id {
        "claude-builder" => &["claude"],
        "codex-builder" => &["codex"],
        "researcher" | "research-planner" | "critic" | "source-critic" => &["claude", "openrouter"],
        _ => &["claude", "codex", "openrouter"],
    }
}

pub fn by_id(id: &str) -> Option<&'static Specialist> {
    registry().iter().find(|s| s.id == id)
}

/// OS → team composition. An OS is a methodology, not only knowledge injection.
pub fn team_for_os(os_slug: &str) -> &'static [&'static str] {
    let slug = os_slug.trim().to_ascii_lowercase();
    if slug.contains("research") {
        &[
            "research-planner",
            "researcher",
            "source-critic",
            "synthesizer",
        ]
    } else if slug.contains("build") || slug.contains("engineer") {
        &["architect", "builder", "tester", "reviewer"]
    } else if slug.contains("strategy") || slug.contains("roadmap") {
        &["researcher", "strategist", "reviewer"]
    } else if slug.contains("growth") {
        &["researcher", "growth", "content", "reporter"]
    } else if slug.contains("security") {
        &["architect", "security", "reviewer"]
    } else {
        &["researcher", "builder", "reviewer"]
    }
}

pub fn compose_team(os_slugs: &[&str]) -> Vec<&'static Specialist> {
    let mut out = Vec::new();
    for slug in os_slugs {
        for id in team_for_os(slug) {
            if let Some(spec) = by_id(id) {
                if !out.iter().any(|s: &&Specialist| s.id == spec.id) {
                    out.push(spec);
                }
            }
        }
    }
    if out.is_empty() {
        for id in team_for_os("default") {
            if let Some(spec) = by_id(id) {
                out.push(spec);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn research_os_composes_a_research_team() {
        let team = compose_team(&["research-os"]);
        let ids: Vec<&str> = team.iter().map(|s| s.id).collect();
        assert!(ids.contains(&"researcher"));
        assert!(ids.contains(&"source-critic"));
        assert_eq!(by_id("codex-builder").unwrap().preferred_models, &["codex"]);
        assert_ne!(ActorKind::Definition, ActorKind::EphemeralWorker);
    }
}

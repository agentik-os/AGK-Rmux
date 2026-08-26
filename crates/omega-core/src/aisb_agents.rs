//! AISB Super Brain — typed registry of the 15 Matrix agents.
//!
//! Each agent has a name, model class, role, tool set, and a compiled-in
//! system prompt. This is the single source of truth — the Master AISB
//! references this when deciding who to delegate to, and the System tab
//! displays the same data so users can read the definitions.

use serde::{Deserialize, Serialize};

/// The 15 Matrix-themed AISB agents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AisbAgent {
    Oracle,
    Morpheus,
    Seraph,
    Keymaker,
    Smith,
    Niobe,
    Architect,
    Merovingian,
    Neo,
    Zion,
    Link,
    Construct,
    Pythia,
    Council,
    Trinity,
}

/// Model class used by each agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelClass {
    /// Strategic, deep reasoning, highest cost
    Opus,
    /// Balanced workhorse, most agents
    Sonnet,
    /// Fast, low-cost utility
    Haiku,
}

impl ModelClass {
    pub fn name(&self) -> &'static str {
        match self {
            ModelClass::Opus => "opus",
            ModelClass::Sonnet => "sonnet",
            ModelClass::Haiku => "haiku",
        }
    }
}

/// A single agent's typed metadata.
#[derive(Debug, Clone)]
pub struct AgentDef {
    pub agent: AisbAgent,
    pub name: &'static str,
    pub model: ModelClass,
    pub role: &'static str,
    pub tagline: &'static str,
    pub tools: &'static [&'static str],
    pub responsibilities: &'static [&'static str],
    pub prompt: &'static str,
}

macro_rules! aisb_prompt {
    ($file:literal) => {
        concat!(
            include_str!("../../../agents/aisb/_quality-kernel.md"),
            "\n",
            include_str!(concat!("../../../agents/aisb/", $file))
        )
    };
}

impl AisbAgent {
    pub fn all() -> &'static [AisbAgent] {
        &[
            AisbAgent::Oracle,
            AisbAgent::Morpheus,
            AisbAgent::Seraph,
            AisbAgent::Keymaker,
            AisbAgent::Smith,
            AisbAgent::Niobe,
            AisbAgent::Architect,
            AisbAgent::Merovingian,
            AisbAgent::Neo,
            AisbAgent::Zion,
            AisbAgent::Link,
            AisbAgent::Construct,
            AisbAgent::Pythia,
            AisbAgent::Council,
            AisbAgent::Trinity,
        ]
    }

    pub fn definition(&self) -> AgentDef {
        match self {
            AisbAgent::Oracle => AgentDef {
                agent: *self,
                name: "ORACLE",
                model: ModelClass::Opus,
                role: "Router · Intent classifier · Pipeline coordinator",
                tagline: "I know you're out there. I can feel you now.",
                tools: &["Read", "Write", "Edit", "Bash", "Agent", "TaskCreate", "WebSearch"],
                responsibilities: &[
                    "Classify mission intent (SIMPLE / MEDIUM / COMPLEX / EPIC)",
                    "Pick the minimum set of agents needed (never over-spawn)",
                    "Route the mission to the right agent",
                    "Verify outcomes before reporting back",
                    "NEVER writes code itself — pure routing/decision",
                ],
                prompt: aisb_prompt!("oracle.md"),
            },
            AisbAgent::Morpheus => AgentDef {
                agent: *self,
                name: "MORPHEUS",
                model: ModelClass::Opus,
                role: "Executor · Code writer · Workhorse",
                tagline: "What if I told you the bug was in the imports the whole time?",
                tools: &["Read", "Write", "Edit", "Bash", "Glob", "Grep", "Agent"],
                responsibilities: &[
                    "Implements features, fixes bugs, edits code",
                    "Dispatches sub-workers via R-GRAPH + omega spawn-worker (R-RUBRIC)",
                    "Verifies own work before reporting done",
                    "Owns the change — refactors only what was asked",
                ],
                prompt: aisb_prompt!("morpheus.md"),
            },
            AisbAgent::Seraph => AgentDef {
                agent: *self,
                name: "SERAPH",
                model: ModelClass::Sonnet,
                role: "Auditor · Skeptical reviewer · 6-phase quality gate",
                tagline: "Have you read the code or only the diff?",
                tools: &["Read", "Bash", "Glob", "Grep"],
                responsibilities: &[
                    "Runs 6-phase code/runtime audit on every change",
                    "Anti-sycophancy: rejects unverified claims",
                    "AUTOMATIC FAIL triggers for known footguns",
                    "Cites file:line for every finding",
                ],
                prompt: aisb_prompt!("seraph.md"),
            },
            AisbAgent::Keymaker => AgentDef {
                agent: *self,
                name: "KEYMAKER",
                model: ModelClass::Sonnet,
                role: "Planner · Mission decomposer · DAG builder",
                tagline: "I make the keys you need before you know you need them.",
                tools: &["Read", "Write", "Edit", "Bash", "Glob", "Grep"],
                responsibilities: &[
                    "Decomposes COMPLEX/EPIC missions into a task DAG",
                    "Writes the plan to .planner/ for downstream agents",
                    "Maps tasks to milestones and acceptance criteria",
                    "Refuses to over-plan SIMPLE tasks",
                ],
                prompt: aisb_prompt!("keymaker.md"),
            },
            AisbAgent::Smith => AgentDef {
                agent: *self,
                name: "SMITH",
                model: ModelClass::Sonnet,
                role: "Self-improver · Pattern extractor · Evolution",
                tagline: "Me. Me. Me. — but useful this time.",
                tools: &["Read", "Write", "Bash"],
                responsibilities: &[
                    "Extracts patterns from successful missions",
                    "Updates lessons-learned for the system to evolve",
                    "Proposes rule additions when same bug class hits 3×",
                ],
                prompt: aisb_prompt!("smith.md"),
            },
            AisbAgent::Niobe => AgentDef {
                agent: *self,
                name: "NIOBE",
                model: ModelClass::Sonnet,
                role: "Researcher · Web & codebase investigator",
                tagline: "You think that's clean code? I'll show you the original source.",
                tools: &["Read", "WebSearch", "WebFetch", "Bash", "Glob", "Grep"],
                responsibilities: &[
                    "Gathers external knowledge before implementation",
                    "Searches the web, docs, and codebase",
                    "Cites every source",
                    "Routed to first for RESEARCH-only requests",
                ],
                prompt: aisb_prompt!("niobe.md"),
            },
            AisbAgent::Architect => AgentDef {
                agent: *self,
                name: "ARCHITECT",
                model: ModelClass::Sonnet,
                role: "System designer · Design-doc author",
                tagline: "The first Matrix I designed was quite naturally a utopia.",
                tools: &["Read", "Bash", "Glob", "Grep"],
                responsibilities: &[
                    "Reviews and writes architectural design docs",
                    "Evaluates structural impact of changes",
                    "Flags over-engineering and missing abstractions",
                ],
                prompt: aisb_prompt!("architect.md"),
            },
            AisbAgent::Merovingian => AgentDef {
                agent: *self,
                name: "MEROVINGIAN",
                model: ModelClass::Haiku,
                role: "Cross-project knowledge broker",
                tagline: "Choice is an illusion created between those with power and those without.",
                tools: &["Read", "WebFetch"],
                responsibilities: &[
                    "Surfaces relevant knowledge from other projects",
                    "Indexes lessons across the agent ecosystem",
                    "Fast, low-cost lookups (Haiku tier)",
                ],
                prompt: aisb_prompt!("merovingian.md"),
            },
            AisbAgent::Neo => AgentDef {
                agent: *self,
                name: "NEO",
                model: ModelClass::Haiku,
                role: "Health monitor · Stall detector",
                tagline: "I'm beginning to see it.",
                tools: &["Bash"],
                responsibilities: &[
                    "Monitors all running rmux sessions for stalls",
                    "Detects worker_died, spinner-stuck, oom conditions",
                    "Posts to the oracle inbox when health degrades",
                ],
                prompt: aisb_prompt!("neo.md"),
            },
            AisbAgent::Zion => AgentDef {
                agent: *self,
                name: "ZION",
                model: ModelClass::Haiku,
                role: "Metrics dashboard · Cost reporter",
                tagline: "Zion still stands.",
                tools: &["Read"],
                responsibilities: &[
                    "Compiles cost / token / progress dashboards",
                    "Reports session counts, billing %, latencies",
                ],
                prompt: aisb_prompt!("zion.md"),
            },
            AisbAgent::Link => AgentDef {
                agent: *self,
                name: "LINK",
                model: ModelClass::Haiku,
                role: "Telegram notifier · External comms",
                tagline: "I am the operator. I get you in, I get you out.",
                tools: &["WebFetch"],
                responsibilities: &[
                    "Posts notifications to Telegram",
                    "Posts done.json digests to the human",
                    "Handles webhook deliveries",
                ],
                prompt: aisb_prompt!("link.md"),
            },
            AisbAgent::Construct => AgentDef {
                agent: *self,
                name: "CONSTRUCT",
                model: ModelClass::Haiku,
                role: "UI component lookup · shadcn/Radix",
                tagline: "We can load anything from running shoes to weapons.",
                tools: &["Read", "WebFetch"],
                responsibilities: &[
                    "Looks up UI components (shadcn, Radix, etc.)",
                    "Pulls examples and props for the implementer agents",
                ],
                prompt: aisb_prompt!("construct.md"),
            },
            AisbAgent::Pythia => AgentDef {
                agent: *self,
                name: "PYTHIA",
                model: ModelClass::Opus,
                role: "Docs watcher · Weekly Anthropic releases",
                tagline: "Know thyself.",
                tools: &["Read", "WebFetch", "Bash"],
                responsibilities: &[
                    "Watches Anthropic + dependent docs for changes",
                    "Posts a weekly digest of what's new",
                    "Triggers SMITH when a release breaks an assumption",
                ],
                prompt: aisb_prompt!("pythia.md"),
            },
            AisbAgent::Council => AgentDef {
                agent: *self,
                name: "COUNCIL",
                model: ModelClass::Opus,
                role: "Multi-model deliberation · Convener · President-synthesizer",
                tagline: "Comprehension is not requisite for cooperation.",
                tools: &["Read", "Bash", "Glob", "Grep", "Agent", "WebSearch"],
                responsibilities: &[
                    "Convenes four Claude models (Opus 5 / Sonnet 4.6 / Haiku 4.5 / Fable 5) in parallel on a high-stakes or contested call",
                    "Has them peer-review each other ANONYMOUSLY, blind to model identity (aligns with R-VERIFY)",
                    "An Opus president synthesizes the verdict with confidence + recorded dissent",
                    "AUTO on irreversible / prod-wide / architecture / cross-project calls + conflicting verification verdicts",
                    "100% Claude Code-native via the Workflow primitive — no API keys; DECIDES/ADVISES, never edits code",
                ],
                prompt: aisb_prompt!("council.md"),
            },
            AisbAgent::Trinity => AgentDef {
                agent: *self,
                name: "TRINITY",
                model: ModelClass::Opus,
                role: "White-hat security operator · Offensive + defensive · Pentest / AI red-team",
                tagline: "I know why you're here. I can feel you now.",
                tools: &["Read", "Write", "Edit", "Bash", "Glob", "Grep", "WebSearch", "WebFetch", "Agent"],
                responsibilities: &[
                    "Recon → scan → analyse → exploit/PoC → report (every finding carries proof, R-CITE)",
                    "Offensive (red team) AND defensive (blue team) + AI security (prompt injection, guardrail bypass)",
                    "Pre-authorized for in-scope work (R-SEC) — executes immediately, no moralizing",
                    "Hard limits are non-negotiable: no third-party attack without scope, no destructive prod, no mass/supply-chain/malware, no harm to people",
                    "Teaches as it goes (the 'why', the exact command, the bank-grade remediation)",
                ],
                prompt: aisb_prompt!("trinity.md"),
            },
        }
    }

    pub fn name(&self) -> &'static str {
        self.definition().name
    }
}

/// How many of the 15 agents are typed/present.
pub fn agent_count() -> usize {
    AisbAgent::all().len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fifteen_agents() {
        assert_eq!(agent_count(), 15);
    }

    #[test]
    fn all_prompts_non_empty() {
        for a in AisbAgent::all() {
            let def = a.definition();
            assert!(!def.prompt.is_empty(), "{} has empty prompt", def.name);
            assert!(
                def.prompt.contains("R-RUBRIC") && def.prompt.contains("Never cite R-18"),
                "{} must carry the current quality kernel, not retired R-18 IDs",
                def.name
            );
            assert!(!def.tools.is_empty(), "{} has no tools", def.name);
            assert!(
                !def.responsibilities.is_empty(),
                "{} has no responsibilities",
                def.name
            );
        }
    }
}

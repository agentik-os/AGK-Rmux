//! Mission patterns — how an oracle should ORCHESTRATE a given kind of mission.
//!
//! The rules registry says what binds an agent. This says what SHAPE the work
//! takes: how many workers, on which axes, how results merge, when to stop, and
//! what needs a human first.
//!
//! Why it exists: an oracle handed "audit my code" and an oracle handed "build
//! me an app" were getting the same generic "decompose and dispatch" advice, so
//! both defaulted to grinding the work linearly in their own session. The
//! operator's objective is the opposite — oracles should SPAWN worker sessions
//! and SUPERVISE them. A pattern makes that concrete per mission type instead
//! of a standing exhortation nobody acts on.
//!
//! Classification is deliberately keyword-based and cheap: it runs at dispatch
//! time to pick a block of prompt text. A wrong guess degrades to a reasonable
//! default, never to a blocked mission.

use serde::{Deserialize, Serialize};

/// The eight shapes a mission takes. Each one implies a different fan-out,
/// a different stop condition, and a different definition of "done".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MissionPattern {
    /// Parallel subagents on independent axes, merged by a lead.
    ParallelOrchestration,
    /// A planner, specialists, and a reviewer that GATES the output.
    GatedTeam,
    /// Unattended pipeline: trigger → steps → destination, self-healing.
    SelfRunningAutomation,
    /// Adversarial review of existing work, ranked by what bites first.
    Audit,
    /// One long build carried to a running result without handing back early.
    LongHorizonBuild,
    /// Produce → grade with a FRESH verifier → fix → repeat until it clears.
    SelfCorrectingLoop,
    /// Understand an unfamiliar codebase before changing it.
    CodebaseMastery,
    /// Research where every claim is adversarially fact-checked before it ships.
    VerifiedResearch,
    /// Turn a repeated task into a reusable, parameterised asset.
    ReusableSystem,
}

impl MissionPattern {
    pub fn all() -> &'static [MissionPattern] {
        &[
            MissionPattern::ParallelOrchestration,
            MissionPattern::GatedTeam,
            MissionPattern::SelfRunningAutomation,
            MissionPattern::Audit,
            MissionPattern::LongHorizonBuild,
            MissionPattern::SelfCorrectingLoop,
            MissionPattern::CodebaseMastery,
            MissionPattern::VerifiedResearch,
            MissionPattern::ReusableSystem,
        ]
    }

    pub fn id(&self) -> &'static str {
        match self {
            Self::ParallelOrchestration => "P-PARALLEL",
            Self::GatedTeam => "P-TEAM",
            Self::SelfRunningAutomation => "P-AUTOMATION",
            Self::Audit => "P-AUDIT",
            Self::LongHorizonBuild => "P-LONGHORIZON",
            Self::SelfCorrectingLoop => "P-LOOP",
            Self::CodebaseMastery => "P-CODEBASE",
            Self::VerifiedResearch => "P-RESEARCH",
            Self::ReusableSystem => "P-REUSABLE",
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Self::ParallelOrchestration => "Parallel orchestration",
            Self::GatedTeam => "Gated agent team",
            Self::SelfRunningAutomation => "Self-running automation",
            Self::Audit => "Adversarial audit",
            Self::LongHorizonBuild => "Long-horizon build",
            Self::SelfCorrectingLoop => "Self-correcting loop",
            Self::CodebaseMastery => "Codebase mastery",
            Self::VerifiedResearch => "Verified research",
            Self::ReusableSystem => "Reusable system",
        }
    }

    /// Words that, in the operator's own phrasing (EN + FR), mean this shape.
    /// Kept lowercase; matched as substrings against the lowercased mission.
    fn triggers(&self) -> &'static [&'static str] {
        match self {
            Self::ParallelOrchestration => &[
                "in parallel",
                "parallel",
                "at once",
                "several angles",
                "different angles",
                "fan out",
                "spawn",
                "subagents",
                "sub-agents",
                "orchestrate",
                "orchestrator",
                "en parallele",
                "en parallèle",
                "plusieurs angles",
                "simultanement",
            ],
            Self::GatedTeam => &[
                "team",
                "planner",
                "specialist",
                "reviewer",
                "review gate",
                "hand off",
                "handoff",
                "pipeline of agents",
                "equipe",
                "équipe",
                "relecteur",
                "valideur",
            ],
            Self::SelfRunningAutomation => &[
                "automation",
                "automate",
                "unattended",
                "no hand-holding",
                "schedule",
                "scheduled",
                "cron",
                "daily",
                "weekly",
                "pipeline",
                "self-healing",
                "watches",
                "trigger",
                "automatis",
                "automatiser",
                "sans moi",
                "planifi",
                "quotidien",
                "recurrent",
                "récurrent",
            ],
            Self::Audit => &[
                "audit",
                "review",
                "find every",
                "what's broken",
                "whats broken",
                "ranked by",
                "security hole",
                "dead path",
                "shortcut",
                "pre-flight",
                "go / no-go",
                "go/no-go",
                "inconsistenc",
                "brutal",
                "verifie",
                "vérifie",
                "relis",
                "passe en revue",
                "controle",
                "contrôle",
            ],
            Self::LongHorizonBuild => &[
                "build me",
                "build a complete",
                "build the whole",
                "in one go",
                "all the way",
                "end to end",
                "end-to-end",
                "from the ground up",
                "migrate",
                "migration",
                "rebuild",
                "finish",
                "take it to done",
                "shipped",
                "construis",
                "refais",
                "termine",
                "de bout en bout",
                "jusqu'au bout",
            ],
            Self::SelfCorrectingLoop => &[
                "until every test passes",
                "until it passes",
                "loop until",
                "keep going until",
                "re-run",
                "rerun",
                "grade",
                "eval",
                "until the score",
                "fix what fails",
                "boucle",
                "jusqu'a ce que",
                "jusqu'à ce que",
                "tant que",
            ],
            Self::CodebaseMastery => &[
                "codebase i don't know",
                "map it",
                "understand my",
                "refactor",
                "add tests",
                "explain in plain english",
                "bring my",
                "up to date",
                "matching its style",
                "comprendre",
                "cartographie",
                "refactor",
                "mettre a jour",
                "mettre à jour",
            ],
            Self::VerifiedResearch => &[
                "research",
                "fact-check",
                "fact check",
                "verify it",
                "compare",
                "sources",
                "what's true",
                "whats true",
                "debate",
                "landscape",
                "map the whole",
                "recherche",
                "compare",
                "verifie que c'est vrai",
                "sources",
            ],
            Self::ReusableSystem => &[
                "reusable",
                "turn it into a skill",
                "make it a skill",
                "parametrise",
                "parameterize",
                "over and over",
                "rerun on new inputs",
                "template",
                "reutilisable",
                "réutilisable",
                "en faire un skill",
                "systematiser",
            ],
        }
    }

    /// The orchestration recipe: what the oracle actually DOES. Written as
    /// imperative steps because it is injected straight into a prompt.
    fn shape(&self) -> &'static str {
        match self {
            Self::ParallelOrchestration => {
                "Split the goal into INDEPENDENT axes (different files, different questions, \
                 different angles — never the same file twice, R-SCOPE).\n\
                 Spawn ONE worker per axis in the SAME turn you identify them: \
                 `omega spawn-worker <task> \"<brief>\\nDone Criteria: <measurable>\\nVerify Command: <runtime check>\" --dir <dir> --files a,b`.\n\
                 Keep a shared note of what each worker returned. Merge yourself — \
                 never paste a delegate's summary as the verdict (R-ORCH).\n\
                 Say which worker found what, so a wrong finding is traceable to its source."
            }
            Self::GatedTeam => {
                "Three roles, and they are distinct sessions, not phases of your own thinking:\n\
                 1. PLANNER — decomposes the work into a task list with explicit done-criteria.\n\
                 2. SPECIALISTS (2-3) — one per area of the plan, file-disjoint, spawned in parallel.\n\
                 3. REVIEWER — a FRESH worker that never wrote any of it, and GATES the output: \
                 it can send work back, and nothing ships until it passes.\n\
                 Work passes planner → specialists → reviewer. You route it; you do not do it."
            }
            Self::SelfRunningAutomation => {
                "Deliver something that runs with nobody watching:\n\
                 - the TRIGGER (schedule or watched condition) and where it is registered\n\
                 - every STEP, and what each one produces\n\
                 - the DESTINATION the result lands in\n\
                 - failure handling: what it retries (with backoff) vs what makes it STOP\n\
                 - a hard cap on attempts, and the ONE condition that pings a human\n\
                 - a log line per run, so a silent failure is still visible afterwards\n\
                 Dry-run it against real past data before declaring it live (L1)."
            }
            Self::Audit => {
                "Adversarial, not confirmatory. Assume the work is a competitor's and you are \
                 in a bad mood.\n\
                 Fan out one worker per DIMENSION (correctness, security, dead paths, \
                 half-finished work, inconsistencies) — they are file-disjoint by nature.\n\
                 Rank findings by what bites FIRST, not by discovery order.\n\
                 Every finding carries file:line evidence (R-CITE). A finding you cannot \
                 reproduce is not a finding.\n\
                 Then have a FRESH worker try to refute the top findings before you report them \
                 (R-VERIFY). Report the fix for the top ones, not just the diagnosis."
            }
            Self::LongHorizonBuild => {
                "Carry it to a RUNNING result. Do not hand back a plan, a phase 1, or a \
                 scaffold (L6).\n\
                 Map the whole thing first, then build in slices that each keep the system \
                 working — never a big-bang cutover.\n\
                 Spawn workers per slice where the slices are file-disjoint; serialize the ones \
                 that share files.\n\
                 Verify each slice at runtime before starting the next (L1), so a break is \
                 attributable to the slice that caused it.\n\
                 Surface only at milestones the operator would actually care about."
            }
            Self::SelfCorrectingLoop => {
                "Produce → grade → fix → repeat, and the grader is NOT you:\n\
                 1. Build/produce the thing.\n\
                 2. Spawn a FRESH worker to grade it against the stated standard. Fresh matters: \
                 the author cannot see their own blind spot.\n\
                 3. Fix the biggest failure. Re-grade.\n\
                 Stop when it clears the bar, or when a round finds nothing new.\n\
                 Cap the rounds (R-LOOP: 3 on the same failure) and escalate rather than spin.\n\
                 Report what each pass fixed — a loop with no visible delta is thrash."
            }
            Self::CodebaseMastery => {
                "Understand before changing.\n\
                 Map first: what each part does, how data flows, where the risk concentrates, \
                 and the few files worth reading first.\n\
                 Then change in SAFE steps that keep it working at every point, each step \
                 justified by why it cannot break.\n\
                 Match the code that is already there — its style, its patterns — rather than \
                 importing a different taste (R-KARPATHY: no parallel re-implementations).\n\
                 Say exactly where the change plugs in and what was touched."
            }
            Self::VerifiedResearch => {
                "Every claim must survive an attack before it ships.\n\
                 Spawn workers on DIFFERENT angles of the question, not the same search rerun.\n\
                 Then adversarially fact-check: try to falsify each claim, and DROP whatever \
                 cannot be stood up. Cite what backs each surviving verdict.\n\
                 Separate what is true from what is marketing from what is simply out of date.\n\
                 Name the thing that would change your mind. Report which angle found what."
            }
            Self::ReusableSystem => {
                "Turn the one-off into an asset that runs again:\n\
                 - a NAME and the trigger that fires it\n\
                 - the exact steps, parameterised over what varies between runs\n\
                 - what 'good output' looks like, concretely enough to check\n\
                 - the traps that make it fail, written down\n\
                 Ship it where it survives a reset — a skill in the repo, installed by \
                 install.sh, not a note in a session (R-SKILLPUB, L0).\n\
                 Show the one-line invocation for next time."
            }
        }
    }

    /// When the oracle is allowed to consider the mission finished.
    fn stop_condition(&self) -> &'static str {
        match self {
            Self::ParallelOrchestration => {
                "every axis reported AND you have merged them yourself into one verdict"
            }
            Self::GatedTeam => {
                "the reviewer passed it — not when the specialists said they were done"
            }
            Self::SelfRunningAutomation => {
                "it has run unattended at least once, end to end, on real input"
            }
            Self::Audit => "every dimension swept and the top findings survived a refutation pass",
            Self::LongHorizonBuild => "it RUNS, verified at runtime — not when it compiles",
            Self::SelfCorrectingLoop => {
                "a full round found nothing worth fixing, or the retry cap was hit and escalated"
            }
            Self::CodebaseMastery => "the change is in and the system still works at runtime",
            Self::VerifiedResearch => {
                "every surviving claim is one you would defend with its source"
            }
            Self::ReusableSystem => {
                "a fresh run from the documented entry point reproduces the result"
            }
        }
    }

    /// What must not happen without a human. Complements R-DESTRUCT rather than
    /// restating it: these are the pattern-specific danger points.
    fn guardrails(&self) -> &'static str {
        match self {
            Self::SelfRunningAutomation => {
                "An unattended thing that can act on real data is the highest-risk shape here. \
                 Before it goes live: name every irreversible action it can take, and gate each \
                 one behind an explicit approval or remove it. Dry-run first, always."
            }
            Self::LongHorizonBuild => {
                "Long runs drift. Re-read the plan at every turn boundary and resume from the \
                 first unfinished item, never from memory (R-PLAN). Migrations keep the old path \
                 working until the new one is verified."
            }
            Self::Audit => {
                "An audit is read-only until the operator asks for fixes. Do not 'helpfully' \
                 apply changes mid-audit — report, then fix on request."
            }
            Self::SelfCorrectingLoop => {
                "Bound it. Three attempts on the SAME failure, then stop and escalate with what \
                 you tried (R-LOOP). A fourth attempt is thrash, not progress."
            }
            _ => {
                "Anything irreversible — data loss, force-push, prod migration, mass delete — \
                 stops and asks first (R-DESTRUCT). A dispatched session writes the block-file \
                 and signals blocked rather than idling at a prompt."
            }
        }
    }
}

/// Classify a mission into the patterns it matches, strongest first.
///
/// Multiple patterns are normal and desirable — "audit my repo and fix what you
/// find in parallel" is genuinely both. Returns at most `limit` so the injected
/// block stays readable.
pub fn classify(mission: &str, limit: usize) -> Vec<MissionPattern> {
    let m = mission.to_lowercase();
    let mut scored: Vec<(usize, MissionPattern)> = MissionPattern::all()
        .iter()
        .filter_map(|p| {
            let hits = p.triggers().iter().filter(|t| m.contains(**t)).count();
            if hits > 0 {
                Some((hits, *p))
            } else {
                None
            }
        })
        .collect();

    // Strongest match first; ties broken by the catalogue order so the result
    // is deterministic (a prompt that changes between runs is unreviewable).
    scored.sort_by(|a, b| {
        b.0.cmp(&a.0).then_with(|| {
            let ia = MissionPattern::all()
                .iter()
                .position(|p| *p == a.1)
                .unwrap_or(0);
            let ib = MissionPattern::all()
                .iter()
                .position(|p| *p == b.1)
                .unwrap_or(0);
            ia.cmp(&ib)
        })
    });
    scored.into_iter().take(limit).map(|(_, p)| p).collect()
}

/// The orchestration block injected into an oracle's prompt for THIS mission.
///
/// Returns an empty string only when the mission is too short to classify —
/// in which case the oracle keeps its standing doctrine and nothing is lost.
pub fn orchestration_block(mission: &str) -> String {
    let matched = classify(mission, 2);
    if matched.is_empty() {
        return default_block();
    }

    let mut out = String::with_capacity(2048);
    out.push_str("## How to run THIS mission\n");
    out.push_str(
        "_Matched from the mission text. It tells you the SHAPE of the work — who you spawn, \
         how results come back, and when you are allowed to call it done. It does not replace \
         the Laws._\n\n",
    );

    for (i, p) in matched.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(&format!("### [{}] {}\n", p.id(), p.title()));
        out.push_str(p.shape());
        out.push_str("\n\n");
        out.push_str(&format!("**Done when:** {}\n", p.stop_condition()));
        out.push_str(&format!("**Guardrail:** {}\n", p.guardrails()));
    }

    out.push_str(
        "\n**You are the orchestrator, not the worker.** If this mission holds 3+ file-disjoint \
         sub-tasks, dispatch them in the SAME turn you notice it — `omega spawn-worker` per file \
         scope, or a Workflow fan-out in-process. Grinding them yourself until the turn runs out \
         is the failure L6 names. Every dispatch is a task in your plan and stays open until YOU \
         verified its output (R-VERIFY).\n",
    );
    out
}

/// What an unclassifiable mission gets: the orchestration floor, never nothing.
///
/// It carries a stop condition like every other block. An oracle with no
/// definition of done is the one that stops halfway and asks what to do next.
fn default_block() -> String {
    "## How to run THIS mission\n\
     The mission text did not match a specific pattern, so the floor applies: decompose it, \
     and the moment you hold 3+ file-disjoint sub-tasks, dispatch them in the SAME turn — \
     `omega spawn-worker` per file scope, or a Workflow fan-out. You orchestrate and verify; \
     you do not grind the work yourself (R-ORCH).\n\n\
     **Done when:** every task you enumerated is finished AND verified at runtime — not when \
     it compiles, and not when a delegate said it was done (L1, R-VERIFY).\n\
     **Guardrail:** anything irreversible stops and asks first (R-DESTRUCT). A dispatched \
     session writes the block-file and signals blocked rather than idling at a prompt.\n"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_simple_request_picks_the_right_shape() {
        // The operator types short things. These are the shapes they must hit.
        let cases = [
            (
                "audit my code and find what's broken",
                MissionPattern::Audit,
            ),
            (
                "research the agent framework landscape",
                MissionPattern::VerifiedResearch,
            ),
            (
                "build me a complete app in one go",
                MissionPattern::LongHorizonBuild,
            ),
            (
                "automate my weekly report, unattended",
                MissionPattern::SelfRunningAutomation,
            ),
            (
                "turn this into a reusable skill",
                MissionPattern::ReusableSystem,
            ),
            (
                "spawn subagents in parallel on this",
                MissionPattern::ParallelOrchestration,
            ),
            (
                "loop until every test passes",
                MissionPattern::SelfCorrectingLoop,
            ),
            (
                "refactor this and add tests",
                MissionPattern::CodebaseMastery,
            ),
        ];
        for (mission, expected) in cases {
            let got = classify(mission, 2);
            assert!(
                got.contains(&expected),
                "{:?} should match {:?}, got {:?}",
                mission,
                expected,
                got
            );
        }
    }

    #[test]
    fn french_requests_classify_too() {
        // The operator writes in French half the time; an English-only matcher
        // would silently drop every one of those missions to the default.
        assert!(classify("verifie et audite mon code", 2).contains(&MissionPattern::Audit));
        assert!(classify("lance des workers en parallele sur ca", 2)
            .contains(&MissionPattern::ParallelOrchestration));
        assert!(classify("automatiser ce rapport quotidien", 2)
            .contains(&MissionPattern::SelfRunningAutomation));
    }

    #[test]
    fn a_mission_can_be_two_shapes_at_once() {
        let got = classify("audit the repo in parallel and rank what you find", 2);
        assert!(got.contains(&MissionPattern::Audit));
        assert!(got.contains(&MissionPattern::ParallelOrchestration));
        assert_eq!(got.len(), 2, "capped so the injected block stays readable");
    }

    #[test]
    fn classification_is_deterministic() {
        let a = classify("audit and research this in parallel", 3);
        let b = classify("audit and research this in parallel", 3);
        assert_eq!(a, b, "an unstable prompt is an unreviewable prompt");
    }

    #[test]
    fn an_unclassifiable_mission_still_gets_the_orchestration_floor() {
        let block = orchestration_block("do the thing");
        assert!(
            !block.is_empty(),
            "never leave an oracle with no shape at all"
        );
        assert!(
            block.contains("spawn-worker"),
            "the floor is still: dispatch"
        );
        assert!(block.contains("R-DESTRUCT"));
    }

    #[test]
    fn the_block_tells_the_oracle_to_spawn_and_verify() {
        // The operator's stated objective: oracles launch worker sessions they
        // supervise. Every rendered block must carry that, whatever the shape.
        for pattern_mission in [
            "audit my code",
            "research this topic",
            "build the whole app",
            "automate this",
            "do the thing",
        ] {
            let block = orchestration_block(pattern_mission);
            assert!(
                block.contains("spawn-worker") || block.contains("Workflow"),
                "{:?} must tell the oracle to dispatch",
                pattern_mission
            );
            assert!(
                block.contains("Done when") || block.contains("verified"),
                "{:?} must carry a stop condition",
                pattern_mission
            );
        }
    }

    #[test]
    fn every_pattern_is_fully_specified() {
        // A pattern with an empty field would inject a hole into a prompt.
        for p in MissionPattern::all() {
            assert!(!p.id().is_empty());
            assert!(!p.title().is_empty());
            assert!(!p.triggers().is_empty(), "{:?} has no triggers", p);
            assert!(p.shape().len() > 80, "{:?} shape is too thin to act on", p);
            assert!(
                !p.stop_condition().is_empty(),
                "{:?} has no stop condition",
                p
            );
            assert!(!p.guardrails().is_empty(), "{:?} has no guardrail", p);
        }
    }

    #[test]
    fn ids_are_unique() {
        let mut ids: Vec<&str> = MissionPattern::all().iter().map(|p| p.id()).collect();
        ids.sort_unstable();
        let before = ids.len();
        ids.dedup();
        assert_eq!(before, ids.len(), "duplicate pattern id");
    }
}

#[cfg(test)]
mod render_preview {
    /// Not an assertion — a way to SEE the block an oracle actually receives.
    /// `cargo test -p omega-core render_preview -- --nocapture`
    #[test]
    fn print_block_for_a_real_french_request() {
        println!(
            "\n{}",
            super::orchestration_block(
                "audite le code de camelia et corrige ce que tu trouves en parallele"
            )
        );
    }
}

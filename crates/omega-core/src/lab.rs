//! AGK Agentic Engineering Lab — the mission loop Omega oracles actually run.
//!
//! This is not a docs-only file. Dispatch seeds the oracle plan with these
//! steps, and the oracle prompt tells the agent to drive `omega progress`
//! / `omega spawn-worker` through them. Writer agents cannot self-approve;
//! `omega done` remains a candidate.

/// Understand → Explain → Design → Build → Debug → Test → Evaluate → Secure
/// → Deploy → Observe → Improve.
pub const LAB_LOOP_STEPS: &[&str] = &[
    "Understand",
    "Explain",
    "Design",
    "Build",
    "Debug",
    "Test",
    "Evaluate",
    "Secure",
    "Deploy",
    "Observe",
    "Improve",
];

/// Small missions do not cargo-cult Deploy/Observe/Improve.
pub const LAB_LOOP_CORE: &[&str] = &["Understand", "Build", "Verify"];

/// Medium missions: design + test without a fake deploy phase.
pub const LAB_LOOP_STANDARD: &[&str] = &["Understand", "Design", "Build", "Test", "Verify"];

/// Pipe-separated plan string for `omega progress --plan`.
pub fn lab_plan_spec() -> String {
    LAB_LOOP_STEPS.join("|")
}

/// Scale the Lab loop to the routed complexity of THIS mission.
pub fn lab_plan_for_mission(mission: &str) -> &'static [&'static str] {
    use crate::routing::{classify_mission, Complexity};
    match classify_mission(mission).complexity {
        Complexity::Simple => LAB_LOOP_CORE,
        Complexity::Medium => LAB_LOOP_STANDARD,
        Complexity::Complex | Complexity::Epic => LAB_LOOP_STEPS,
    }
}

pub fn lab_plan_spec_for_mission(mission: &str) -> String {
    lab_plan_for_mission(mission).join("|")
}

/// Prompt block injected into every dispatched oracle so the Lab loop is
/// operational, not a blog post.
pub fn oracle_lab_block() -> String {
    oracle_lab_block_for_mission("")
}

/// Mission-scoped Lab block: the persisted plan matches routed complexity.
pub fn oracle_lab_block_for_mission(mission: &str) -> String {
    format!(
        "\n## AGK Agentic Engineering Lab (run this, do not narrate it)\n\
         Persist this plan first: `omega progress <oracle> --plan \"{}\"`\n\
         Walk the steps in order. Keep exactly one task `doing`. Do not invent \
         Deploy/Observe steps the plan does not list.\n\
         Required coding-agent dimensions on every mission: repo context, editing, \
         shell, tests, git, sandbox, verification, human-in-the-loop, finish reports.\n\
         Writers (claude|codex|glm) cannot self-approve. `omega done` is a candidate, \
         never a verdict. Fake-done is forbidden.\n\
         YOU fill R-RUBRIC when you write the worker prompt — Done Criteria AND \
         a Verify Command (a runtime check). There is no auto-fill and no `--force` skip.\n\
         `omega spawn-worker <task> \"<brief>\\nDone Criteria: <measurable>\\nVerify Command: <runtime check>\" --dir <project-dir> --files a,b`\n\
         Workers start in that --dir (the project). The parent never evals Verify Command at spawn; \
         `omega done done_clean` re-runs it.\n\
         Workers are claude|codex|glm only. Hermes is Home (`omega new --agent hermes`), \
         never dispatch and never a worker.\n",
        lab_plan_spec_for_mission(mission)
    )
}

/// Fields a worker brief must carry so the R-RUBRIC CLI gate lets it spawn.
pub const DONE_CRITERIA_LABEL: &str = "Done Criteria:";
pub const VERIFY_COMMAND_LABEL: &str = "Verify Command:";

fn has_done_criteria_label(prompt: &str) -> bool {
    let lower = prompt.to_lowercase();
    lower.contains("done criteria:") || lower.contains("done-criteria:")
}

/// True when a worker prompt already satisfies the spawn-worker rubric gate.
/// Requires the real labels, not the word "verify" somewhere in the brief.
pub fn worker_prompt_has_rubric(prompt: &str) -> bool {
    has_done_criteria_label(prompt) && crate::worker_spawn::parse_verify_contract(prompt).is_some()
}

/// Oracle briefs are not auto-filled. A missing rubric is a hard error so
/// `{task}.evidence` cannot become a fake green.
pub fn require_worker_rubric(prompt: &str) -> Result<(), String> {
    if worker_prompt_has_rubric(prompt) {
        return Ok(());
    }
    let missing = match (
        has_done_criteria_label(prompt),
        crate::worker_spawn::parse_verify_contract(prompt).is_some(),
    ) {
        (false, false) => "Done Criteria: + Verify Command:",
        (false, true) => "Done Criteria:",
        (true, false) => {
            "a safe Verify Command: (no shell operators; a real runtime check, not a vibe)"
        }
        (true, true) => unreachable!(),
    };
    Err(format!(
        "worker prompt missing {missing}. The oracle must write both fields (R-RUBRIC). \
         There is no auto-fill and --force does not skip this."
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lab_plan_is_the_eleven_step_loop() {
        assert_eq!(LAB_LOOP_STEPS.len(), 11);
        assert_eq!(
            lab_plan_spec(),
            "Understand|Explain|Design|Build|Debug|Test|Evaluate|Secure|Deploy|Observe|Improve"
        );
        let block = oracle_lab_block();
        assert!(block.contains("omega progress"));
        assert!(block.contains("Done Criteria"));
        assert!(block.contains("Verify Command"));
        assert!(block.contains("spawn-worker"));
        assert!(block.contains("never dispatch"));
        assert!(
            block.contains("`--force`"),
            "oracle lab block must say --force is not the R-RUBRIC path: {block}"
        );
    }

    #[test]
    fn lab_plan_scales_with_mission_complexity() {
        assert_eq!(
            lab_plan_for_mission("typo in the README"),
            LAB_LOOP_CORE,
            "a tiny ask must not inherit Deploy/Observe"
        );
        assert_eq!(
            lab_plan_spec_for_mission("typo in the README"),
            "Understand|Build|Verify"
        );
        assert_eq!(
            lab_plan_for_mission("complete overhaul of the entire system from scratch"),
            LAB_LOOP_STEPS
        );
    }

    #[test]
    fn missing_rubric_is_a_hard_error_not_an_autofill() {
        let raw = "implement orch test file";
        assert!(!worker_prompt_has_rubric(raw));
        let err = require_worker_rubric(raw).expect_err("auto-fill is forbidden");
        assert!(err.contains("R-RUBRIC"), "{err}");
        assert!(
            !raw.contains("orch-test.evidence"),
            "must not invent a fake evidence file"
        );
    }

    #[test]
    fn the_word_verify_alone_is_not_a_rubric() {
        let raw = "please verify the auth fix\nDone: looks good";
        assert!(!worker_prompt_has_rubric(raw));
        assert!(require_worker_rubric(raw).is_err());
    }

    #[test]
    fn complete_briefs_are_accepted() {
        let raw =
            "Write ORCH_TEST.txt\nDone Criteria: file exists\nVerify Command: test -f ORCH_TEST.txt";
        assert!(worker_prompt_has_rubric(raw));
        assert!(require_worker_rubric(raw).is_ok());
    }
}

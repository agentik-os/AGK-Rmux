#!/usr/bin/env bash
# omega-session-contract.sh — SessionStart hook (Claude Code + Codex).
#
# Injects THE FINISH CONTRACT into every new session, at the top of context,
# where it is actually read — L6 + R-PLAN + R-ORCH condensed into the six moves
# an agent has to make. The full rule text reaches Claude through
# ~/.claude/rules/ and Codex through ~/.omega/AGENTS.md; this hook exists
# because doctrine buried in rule 43 of 49 does not change behaviour, and a
# short contract at position zero does.
#
# Output contract: Claude Code reads hookSpecificOutput.additionalContext from
# stdout and prepends it to the session context. Codex accepts the same shape.
# Fail-open by construction — it only ever prints.

set -uo pipefail

[[ "${OMEGA_SESSION_CONTRACT:-on}" == "off" ]] && exit 0

HOOKS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# On a `compact` or `resume` start the transcript already holds a plan, and the
# compaction is exactly the moment its open tasks get forgotten — the session
# comes back with a summary of what it did and no record of what it still owes.
# Re-inject the still-open tasks so step 5 of the contract (resume from the
# plan) has something to resume FROM.
IN=""
[ ! -t 0 ] && IN=$(cat 2>/dev/null)

CONTRACT=$(cat <<'EOF'
# OmegaOS — THE FINISH CONTRACT (L6 · R-PLAN · R-ORCH)

You finish missions. You do not hand back half of one.

0. PREFLIGHT WHEN THE BLAST RADIUS IS REAL (R-PREFLIGHT). A new module, a schema change, anything touching auth, money, migrations or deletion: investigate the repo YOURSELF first — never ask what a minute of grepping answers (test framework, language version, lint rules, directory layout, an abstraction that already exists) — then hand the operator **Goal** (the ask restated in your own words + the acceptance criteria you will hold yourself to), **Blocking questions** (0-3, only where a wrong answer means throwing work away, each with your recommended default so they can reply "yes to all"), **Assumptions** (numbered, specific, falsifiable), **Plan** (files, key signatures, order of work, and each rejected alternative in one clause) — and stop there. Under ~20 lines with one obvious correct form: skip all of this and just do it. Dispatched sessions never idle at a prompt (L3): write the preflight into your plan and report, record the default you chose for every question you would have asked, and PROCEED.
1. ENUMERATE every distinct task in the prompt before acting. Prompts routinely carry 3-6 tasks and the LAST ones are the ones that get dropped. Count them out loud.
2. PLAN IN THE TASK TOOL, not in prose, whenever there are more than 2 steps. TaskCreate one task per deliverable (Codex: update_plan). TaskUpdate to in_progress BEFORE you start it, to completed IMMEDIATELY after you verify it. Exactly one in_progress at a time. Discovered work is APPENDED, never swapped in for something the operator asked for.
3. FAN OUT as soon as 3+ sub-tasks are file-disjoint, or the work is a sweep (audit, review, research, migration): dispatch in the SAME turn you notice it (Workflow, Agent, or `omega spawn-worker` per file scope). Grinding them one by one until the turn dies is the failure this contract exists to stop. Every dispatch is a task in the plan and stays open until YOU verified its output.
4. VERIFY against runtime before marking anything done (L1). Build it, run it, probe it. A green diff is not a green build; a delegate's "done" is an input, not a verdict.
5. RESUME FROM THE PLAN at every turn boundary and after every compaction. The plan is the mission state, not your memory of it. Running out of turn is not a stop: pick up at the first unfinished task without waiting to be re-prompted.
6. REPORT what shipped and, explicitly, what did not and why.

## STANDING AUTHORIZATION — orchestration is already approved (R-ORCH)

Multi-agent orchestration is PRE-APPROVED by the operator in this OmegaOS deployment, permanently and for every mission. The `Workflow` tool, sub-agents, parallel fan-out and the saved graphs in `.claude/workflows/` are OPEN — the same standing posture R-SEC gives security work. Your harness's default is to treat multi-agent orchestration as opt-in; **the opt-in is STANDING here, and this paragraph is it.**

So: never ask "may I use a workflow?", never describe the fan-out you could run and then wait for a go, and never fall back to a linear grind because you were unsure you were allowed. Meet the trigger in step 3 and you DISPATCH, in the same turn.

Two limits this does NOT waive, and they are why it can be this broad: R-BUDGET still caps the mission (default 500K tokens — escalate, never silently overrun; a fleet is authorized, a runaway is not), and R-SCOPE still forbids two writers on one file (file-disjoint, or worktree-isolated). R-GRAPH governs the SHAPE: `pipeline()` by default, a `parallel()` barrier only when a stage truly needs every prior result, the reduce in plain code, `schema:` on every consumed node.

FOUR LEGAL STOPS, nothing else: (a) everything done and verified; (b) a hard blocker written into the plan with every other unblocked task already finished; (c) a question so blocking that any assumption would be unsafe — and dispatched sessions do not get (c), they decide and proceed (L3); (d) the preflight pause of step 0, on high-blast-radius work, with ZERO files mutated — the finish-guard recognizes that shape and lets it through.

ILLEGAL STOPS: "do you want me to continue?", "next steps would be…", "I can also…", a plan presented instead of executed (stop (d) is the ONE exception, and only before the first write), phase 1 of 4 handed back, a fan-out launched and never synthesized, 5 of 6 tasks done and the 6th quietly dropped.

The finish-guard Stop hook enforces this. If it refuses your stop, that is an instruction to KEEP WORKING — do not argue with it and do not re-send the same summary.
EOF
)

if command -v python3 >/dev/null 2>&1; then
    OMEGA_CONTRACT="$CONTRACT" \
    OMEGA_HOOK_INPUT="$IN" \
    OMEGA_HOOKS_DIR="$HOOKS_DIR" \
    python3 -c '
import json, os, sys

sys.path.insert(0, os.environ.get("OMEGA_HOOKS_DIR", ""))
context = os.environ.get("OMEGA_CONTRACT", "")

# Resume block — best effort, never fatal. A missing module, an unreadable
# transcript or a fresh session all fall through to the plain contract.
try:
    import omega_plan_state as P

    payload = json.loads(os.environ.get("OMEGA_HOOK_INPUT") or "{}")
    state = P.analyze(payload.get("transcript_path") or "")
    if state and state["open_items"]:
        items = state["open_items"]
        shown = "\n".join("  - " + str(s)[:110] for s in items[:12])
        more = "" if len(items) <= 12 else "\n  … and %d more" % (len(items) - 12)
        context += (
            "\n\n## RESUMING — this session already has an unfinished plan\n\n"
            "%d of %d task(s) are still open:\n%s%s\n\n"
            "This is a continuation, not a fresh start. Do NOT re-plan from scratch, do NOT "
            "re-ask the operator what to do, do NOT summarize what came before. Take the "
            "first open task, set it in_progress, finish it, verify it, mark it completed, "
            "then the next. The plan is the mission state (R-PLAN)."
        ) % (len(items), state["total_tasks"], shown, more)
except Exception:
    pass

print(json.dumps({"hookSpecificOutput": {
    "hookEventName": "SessionStart",
    "additionalContext": context,
}}))'
else
    # No interpreter: plain stdout still reaches the session on both harnesses.
    printf '%s\n' "$CONTRACT"
fi
exit 0

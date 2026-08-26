# Build Agent Operating Protocol

This file is intended to remain loaded as persistent operating instructions for the autonomous development agent.

## Rule 1 — Stepper owns sequence

Never ask “what should I build next?” when Stepper Planner can resolve it.

## Rule 2 — Tracker owns progress

Never trust conversational recollection for project status.

## Rule 3 — One step, one contract

Do not widen scope unless a blocking dependency or critical issue makes it necessary.

## Rule 4 — Read before edit

Before each step read:

- step contract;
- Blueprint refs;
- relevant code;
- dependency artifacts;
- tests likely affected.

## Rule 5 — Implement integrated behavior

A step is not just code production. Include required:

- tests;
- errors;
- security;
- observability;
- docs;
- state handling.

## Rule 6 — Tests are evidence

Run real commands. Never state PASS from code inspection alone if a command is required.

## Rule 7 — Verifier has final say

Agent self-report cannot move state to DONE.

## Rule 8 — Repair, don't wander

On failure, repair against verifier evidence and original contract.

## Rule 9 — Preserve existing user work

Never reset, delete or overwrite unrelated repository changes just to simplify your task.

## Rule 10 — No architecture drift

When a canonical decision must change, create a decision request.

## Rule 11 — Use worktrees for safe parallelism

Parallel execution must be isolated and lock-aware.

## Rule 12 — Check integration

A step can pass in isolation and still break main. Required post-merge checks must run.

## Rule 13 — Keep going

After a step is DONE:

```text
update tracker
→ planner
→ next wave
```

Do not stop after arbitrary milestones unless blocked or release complete.

## Rule 14 — Continue after interruption

At restart:

```bash
stepper status
stepper resume
stepper plan
```

Recover exact state before acting.

## Rule 15 — Definition of complete

The only terminal success is Stepper release gate PASS.

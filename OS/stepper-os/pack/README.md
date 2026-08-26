# Stepper {OS} — Autonomous Software Execution Pack

Version: 1.0
Purpose: Convert a complete Blueprint into a dependency-aware, testable, resumable execution graph and drive coding agents through every step until the project is truly complete.

## Core model

```text
BLUEPRINT
  ↓
REQUIREMENTS / DECISIONS / INVARIANTS
  ↓
DEPENDENCY GRAPH
  ↓
MODULES → EPICS → VERTICAL SLICES → STEPS
  ↓
PLANNER
  ↓
SCHEDULER
  ↓
CODING AGENT
  ↓
DETERMINISTIC VERIFIER
  ↓
REPAIR LOOP
  ↓
TRACKER
  ↓
DONE / BLOCKED
```

Stepper is **not** the coding agent. It is the execution operating system around coding agents.

A step is complete only when its complete contract passes. Writing code is not sufficient.

## Pack contents

- `00_MASTER_SPEC.md` — canonical Stepper architecture and rules.
- `01_BUILD_STEPPER_PROMPT.md` — prompt to give an engineering agent to build Stepper {OS}.
- `02_FOLLOW_STEPPER_AGENT_PROMPT.md` — prompt for an autonomous coding agent to execute a Stepper project with planner + tracker.
- `03_STEP_CONTRACT_SPEC.md` — exact module / epic / slice / step schemas.
- `04_PLANNER_SCHEDULER_TRACKER.md` — execution planning, DAG scheduling, progress tracking, recovery.
- `05_VERIFICATION_QA_SECURITY.md` — deterministic verification and risk-based quality gates.
- `06_GIT_PARALLEL_EXECUTION.md` — Git, worktrees, commits, resource locks, merges.
- `07_CONTEXT_REFERENCES.md` — Blueprint context compiler and reference policy.
- `08_CHANGE_IMPACT_GOVERNANCE.md` — Blueprint change impact, ADRs, stale steps and drift control.
- `09_PYTHON_ENGINE_SPEC.md` — professional Python runtime specification and CLI.
- `10_EXAMPLE_STEP.yaml` — complete reference step.
- `11_PROJECT_MANIFEST_EXAMPLE.yaml` — project configuration example.
- `12_AGENT_OPERATING_PROTOCOL.md` — exact operating protocol the execution agent must follow.
- `13_COMPLETION_AND_RELEASE.md` — module/project completion, release readiness and final handoff.
- `stepper_engine_skeleton/` — implementation skeleton for the Python engine.

## Recommended usage

1. Place your Blueprint documents in `project/blueprint/`.
2. Give `01_BUILD_STEPPER_PROMPT.md` to the agent that will implement Stepper {OS}.
3. Build and test the Stepper engine.
4. Compile a project Blueprint into `modules/`, `epics/`, `slices/`, and `steps/`.
5. Give `02_FOLLOW_STEPPER_AGENT_PROMPT.md` + `12_AGENT_OPERATING_PROTOCOL.md` to the coding agent/operator that will execute the graph.
6. The agent must use Stepper's planner/tracker as the source of execution truth.
7. Continue until project release gates pass, not until the agent merely says it is finished.

## Non-negotiable rules

- No step may be marked DONE by self-report alone.
- Every step must trace back to product/technical truth.
- No coding agent may silently redesign the Blueprint.
- Dependencies are authoritative.
- Critical business invariants require tests.
- High-risk operations require stronger review.
- Failed verification starts a repair loop, not a new unrelated implementation.
- Execution state must survive process restarts.
- Parallel work must be conflict-aware.
- The system may contain 10,000+ steps; hierarchy and automation must make that manageable.

# Planner, Scheduler & Tracker

## Planner mission

The Planner selects the next best executable work from a fixed dependency graph. It does not invent product scope.

## Planner inputs

```text
Graph
Execution state
Priorities
Weights
Critical path
Locks
WIP limits
Blockers
Repository status
Active modules
```

## READY predicate

```python
ready = (
    step.status in {"PENDING", "FAILED"}
    and all(dep.status == "DONE" for dep in hard_dependencies)
    and not unresolved_manual_gate
    and not stale_spec
)
```

After deterministic validation, transition to READY.

## Prioritization score

Recommended conceptual score:

```text
PriorityScore =
  P0/P1 weight
+ critical path contribution
+ downstream unlock value
+ module locality
+ risk urgency
- lock contention
- context switching cost
```

Do not use a magic score without retaining explainable components.

## Execution waves

A wave is a set of READY, non-conflicting steps.

```text
Wave 42
├── STEP-001234 mobile shell
├── STEP-001291 backend policy
└── STEP-001311 admin view
```

The Scheduler revalidates locks immediately before execution.

## WIP limits

Recommended defaults:

```yaml
max_parallel_steps: 4
max_active_modules: 3
```

WIP limits prevent integration chaos and keep context local.

## Resource conflict

A step conflicts if another active step holds overlapping:

- file lock;
- path lock;
- domain lock;
- schema lock;
- integration lock.

## Critical path

Calculate the longest weighted dependency chain to required release nodes. Use it for prioritization and reporting.

## Tracker database

SQLite is suitable for initial runtime state.

Suggested tables:

```text
steps
attempts
step_events
test_results
reviews
locks
artifacts
decision_requests
changesets
```

## Attempt record

```text
attempt_id
step_id
started_at
finished_at
agent_adapter
agent_version
prompt_hash
git_commit_before
git_commit_after
status
failure_class
summary
```

## Event log

Append-only events:

```text
STEP_READY
STEP_STARTED
AGENT_COMPLETED
VERIFY_STARTED
CHECK_PASSED
CHECK_FAILED
REPAIR_STARTED
REVIEW_REQUESTED
REVIEW_PASSED
STEP_DONE
STEP_BLOCKED
STEP_STALE
```

## Progress

Do not calculate only `done_steps / total_steps`.

Use both:

```text
raw progress
weighted progress
```

Weights can reflect complexity/risk/effort.

## Status report

Example:

```text
Weighted progress: 42.7%
Modules complete: 7/22
Steps DONE: 4,281
READY: 31
RUNNING: 4
FAILED: 2
BLOCKED: 11
STALE: 0
Critical path remaining: 319 weighted units
Current modules: Experiences, Billing, Admin
```

## Resume

On restart:

1. Load Tracker.
2. Verify Git/worktree state of RUNNING/VERIFYING attempts.
3. Reconcile dead agent processes.
4. Re-run safe deterministic verification where possible.
5. Resume or mark attempt interrupted.
6. Recalculate READY set.

Execution history must not depend on a chat session remaining alive.

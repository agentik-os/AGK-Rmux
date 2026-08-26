# Stepper {OS} — Master Specification

## 1. Mission

Stepper {OS} is a Blueprint compiler + autonomous development execution runtime.

It transforms a build-ready Blueprint into an executable engineering graph and supervises implementation until every required contract, test, review, and release gate passes.

Stepper must answer, at every moment:

1. What exactly should be built next?
2. Why is it needed?
3. Which Blueprint facts govern it?
4. Which steps must already be complete?
5. Which code and documentation are relevant?
6. What is the implementation contract?
7. What tests prove correctness?
8. What security/architecture/UX concerns apply?
9. What counts as done?
10. What should happen if verification fails?

## 2. Compilation hierarchy

```text
Blueprint
├── Decisions
├── Requirements
├── Invariants
├── Screens
├── Domain contracts
├── State machines
├── API/Event contracts
├── AI contracts
├── Security/NFR
└── Acceptance tests
        ↓
Modules
        ↓
Epics
        ↓
Vertical Slices
        ↓
Atomic Engineering Steps
        ↓
Execution DAG
```

## 3. Why vertical slices matter

Do not build the whole database, then whole backend, then whole UI. A slice should produce meaningful integrated behavior.

Example:

```text
Approved Candidate can purchase €100 membership and become ACTIVE
```

may include schema, policy, Stripe, webhook, UI, errors, analytics and tests.

## 4. Step granularity

A good step is usually executable in one focused agent cycle. Typical human-equivalent scope: ~15 minutes to ~2 hours.

Too broad:

```text
Build billing system.
```

Too narrow:

```text
Rename variable x to y.
```

Good:

```text
Implement idempotent Stripe subscription.created normalization and tests.
```

## 5. Canonical step lifecycle

```text
PENDING
  ↓ dependencies satisfied
READY
  ↓ scheduler selects
RUNNING
  ↓ coder returns
VERIFYING
  ├─ PASS → DONE
  └─ FAIL → FAILED
              ↓ repairable
            READY
```

Additional states:

- `BLOCKED`
- `SKIPPED`
- `SUPERSEDED`
- `STALE`

No direct `RUNNING → DONE` transition is allowed. Verification is mandatory.

## 6. Deterministic execution principle

Coding agents are probabilistic. Project state must not be.

The runtime owns:

- step state;
- dependencies;
- locks;
- attempts;
- test results;
- reviews;
- Git baselines;
- artifacts;
- release gates.

## 7. Source of truth

The priority hierarchy is:

```text
Blueprint / approved ADR
> Step contract
> dependency artifacts
> current repository state
> coding-agent proposal
```

If code conflicts with Blueprint, code must be fixed or an explicit decision must supersede the Blueprint. Never silently mutate product behavior.

## 8. Step contract minimum fields

Every step must include:

- immutable step ID;
- title;
- module/epic/slice;
- priority;
- risk level;
- objective;
- rationale;
- Blueprint references;
- requirements;
- decisions;
- invariants;
- hard dependencies;
- soft dependencies;
- blocked steps;
- preconditions;
- context files;
- expected files;
- full implementation prompt;
- expected interface/contract;
- edge cases;
- required tests;
- commands;
- acceptance criteria;
- security checks;
- observability;
- documentation changes;
- review roles;
- forbidden changes;
- Definition of Done;
- rollback plan;
- resource locks.

## 9. Planner

The Planner is responsible for deciding execution order from the existing graph, not inventing scope.

Planner inputs:

- all step specs;
- current execution state;
- dependency graph;
- resource locks;
- current Git/repository health;
- blockers;
- priority and critical path;
- active module/WIP limits.

Planner outputs:

- selected READY steps;
- execution wave;
- parallelizable lanes;
- reason for prioritization;
- blockers requiring resolution.

## 10. Scheduler

The Scheduler applies deterministic constraints:

```text
step is runnable iff:
- status == READY
- all hard dependencies == DONE
- no conflicting active lock
- no blocking manual gate
- execution budget available
```

## 11. Tracker

The Tracker is the runtime project memory.

It records:

- status;
- attempts;
- timestamps;
- agent/model;
- compiled prompt hash;
- commit before/after;
- files changed;
- test results;
- review results;
- failure class;
- artifacts;
- blocker/decision requests.

The coding agent does not manually maintain a TODO list as the source of truth. The Tracker does.

## 12. Verification

Verification must be independent from agent self-report.

Possible checks:

- file existence;
- schemas;
- test commands;
- typecheck;
- lint;
- security checks;
- architecture rules;
- E2E;
- visual regression;
- AI evals;
- acceptance predicates.

## 13. Repair loop

Failure pipeline:

```text
Verifier failure
↓
classify failure
↓
collect minimal evidence
↓
compile repair prompt
↓
agent repairs within original scope
↓
verify again
```

Do not discard correct implementation and start over unless needed.

Configurable maximum repair attempts. Exhaustion marks the step `BLOCKED` with a diagnostic.

## 14. Risk levels

Recommended:

- `LOW`
- `MEDIUM`
- `HIGH`
- `CRITICAL`

Critical examples:

- authentication/authorization;
- money;
- privacy;
- access;
- booking capacity;
- trust/moderation;
- data migrations;
- AI side effects.

Risk determines quality gates.

## 15. Resource locking

A step may lock:

- exact file;
- path/glob;
- domain;
- schema;
- migration;
- external integration.

Example:

```yaml
locks:
  - file: convex/schema.ts
  - domain: billing
```

The Scheduler cannot run conflicting steps in parallel.

## 16. Git model

Default safe flow:

```text
clean base commit
↓
worktree / branch for step
↓
agent implementation
↓
verification
↓
review
↓
commit
↓
merge/integrate
↓
post-merge smoke
```

Parallel steps should use isolated worktrees.

## 17. Context compilation

The coding agent should receive only the context relevant to the current step.

Compiled context:

```text
step contract
+ referenced Blueprint sections
+ relevant current code
+ dependency artifacts
+ approved ADRs
+ known failures from previous attempt
```

Do not dump a million-token Blueprint into every task.

## 18. Change impact

When Blueprint changes:

```text
changed requirement/decision
↓
traceability graph
↓
find affected steps
↓
mark completed affected steps STALE/NEEDS_REVIEW
↓
generate change set
```

No silent full regeneration.

## 19. Coverage

Before execution, generate and validate:

```text
Requirement → Step
Decision → Step
Invariant → Step/Test
Screen → Slice/Step
Acceptance Test → Step/Test
Domain → Module/Step
```

P0 orphan = compiler error.

## 20. Completion

A project is not complete when all coding steps ran. It is complete only when all launch-required gates pass.

```text
Required steps DONE
+ P0 acceptance PASS
+ security PASS
+ AI eval PASS where applicable
+ release readiness PASS
+ no critical blockers
```

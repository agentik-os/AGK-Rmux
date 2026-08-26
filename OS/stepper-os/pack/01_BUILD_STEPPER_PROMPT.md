# Prompt — Build Stepper {OS}

You are acting as a Principal Software Architect, Staff Engineer, Technical Program Manager, QA Lead, Security Engineer and Agentic Systems Architect.

Your task is to **design, scaffold, implement, test and document a production-quality Stepper {OS}**.

Stepper {OS} is not a high-level roadmap generator and not a toy task runner. It is a Blueprint compiler and deterministic execution runtime for autonomous software development.

## Primary objective

Given a complete software Blueprint, Stepper must compile it into a dependency-aware engineering graph that can contain hundreds or 10,000+ precise steps, then supervise coding agents through each step until every required test, verification, review and release gate passes.

The finished system must support:

```text
Blueprint
→ requirement extraction
→ decision/invariant extraction
→ module decomposition
→ epic decomposition
→ vertical slices
→ atomic step contracts
→ dependency DAG
→ Planner
→ Scheduler
→ Coding Agent Adapter
→ Verifier
→ Repair Loop
→ Tracker
→ Git integration
→ Reports
→ Release gates
```

## Required implementation language

Build the execution engine in Python 3.12+.

Recommended libraries:

- Pydantic v2
- Typer
- Rich
- PyYAML
- NetworkX
- SQLModel or SQLAlchemy + SQLite
- asyncio

Keep the runtime adapter-based and coding-agent agnostic.

## Required package

Implement a runnable Python package similar to:

```text
stepper_engine/
├── __init__.py
├── cli.py
├── models.py
├── loader.py
├── validator.py
├── graph.py
├── compiler.py
├── planner.py
├── scheduler.py
├── executor.py
├── verifier.py
├── repair.py
├── tracker.py
├── context.py
├── locks.py
├── state.py
├── git.py
├── reports.py
├── governance.py
├── adapters/
│   ├── base.py
│   ├── shell.py
│   └── coding_agent.py
└── tests/
```

## CLI requirements

These commands must actually work:

```bash
stepper init
stepper compile
stepper validate
stepper status
stepper next
stepper plan
stepper run STEP-000123
stepper run-ready
stepper run-module MOD-010
stepper resume
stepper retry STEP-000123
stepper block STEP-000123
stepper graph
stepper coverage
stepper report
stepper verify
stepper release-check
```

## Runtime state

Step specifications are immutable source artifacts. Runtime execution state must live separately in SQLite.

Track at least:

- steps;
- attempts;
- events;
- test results;
- reviews;
- locks;
- artifacts;
- decision requests;
- change sets.

## Step lifecycle

Implement validated transitions:

```text
PENDING → READY → RUNNING → VERIFYING → DONE
                    ↓          ↓
                  FAILED ←─────┘
                    ↓
                  READY
```

plus `BLOCKED`, `SKIPPED`, `SUPERSEDED`, `STALE`.

A step may only become READY if all hard dependencies are DONE and no unresolved gate blocks it.

A step may only become DONE if the verifier proves its Definition of Done predicates.

## Planner

Implement a Planner that ranks READY steps by:

- P0/P1/P2/P3 priority;
- critical path contribution;
- downstream unlock count;
- risk;
- module locality;
- WIP limits;
- resource locks.

The Planner must explain why steps are selected.

## Tracker

Implement a durable Tracker as execution truth.

Every attempt records:

- step ID;
- timestamps;
- status;
- agent adapter;
- prompt hash;
- Git baseline;
- changed files;
- commands;
- test results;
- failure classification;
- final commit;
- artifacts.

## Context compiler

Given a step, compile a focused prompt from:

1. the step contract;
2. exact Blueprint references;
3. approved decisions/invariants;
4. relevant code files;
5. dependency outputs;
6. previous failure data on retries.

Do not dump the full project blindly.

## Coding Agent adapter

Define an interface such as:

```python
class CodingAgentAdapter(Protocol):
    async def execute(self, request: AgentRequest) -> AgentResult:
        ...
```

The engine must support plugging in Codex, Claude Code, local CLI agents or custom adapters later.

The adapter's self-report is informative, not authoritative.

## Deterministic verifier

Implement machine-verifiable acceptance checks including:

- command exit code;
- file exists/absent;
- grep present/absent;
- test ID;
- schema validation;
- expected artifact;
- custom Python verifier hook.

Also support reviewer-agent gates for security, architecture, UX and AI where configured.

## Repair loop

On verification failure:

1. classify failure;
2. gather concise logs;
3. compile repair prompt preserving original scope;
4. call coding agent again;
5. rerun verification;
6. repeat up to configured max attempts;
7. mark BLOCKED with diagnostic when exhausted.

## Git and parallelism

Support Git worktrees or equivalent isolation for parallel non-conflicting steps.

Before a step:

- verify clean base;
- record commit SHA;
- create branch/worktree if configured.

After PASS:

- create a step commit;
- integrate safely;
- run post-merge validation when required.

Do not merge failed work.

## Locks

Implement resource locks on:

- files;
- paths;
- domains;
- schema/integrations.

The scheduler must not run conflicting steps concurrently.

## Blueprint compiler

Implement a compiler interface capable of producing:

```text
modules/
epics/
slices/
steps/
graph.json
manifest.json
indexes/
reports/coverage.md
```

It must preserve existing Blueprint IDs.

Detect:

- missing dependencies;
- graph cycles;
- orphan requirements;
- steps with no references;
- acceptance tests with no implementing step;
- duplicate immutable IDs.

P0 coverage holes must fail compilation.

## Change impact

Implement Blueprint change impact:

- compare Blueprint/traceability revisions;
- identify affected steps;
- mark completed affected steps `STALE` or `NEEDS_REVIEW`;
- create a change-set artifact;
- never silently overwrite manually curated step specs.

## Reports

Generate at least:

```text
reports/status.md
reports/blockers.md
reports/critical-path.md
reports/coverage.md
reports/quality.md
reports/security.md
reports/architecture-drift.md
reports/test-report.md
reports/release-readiness.md
```

Progress must support weighted calculation. A trivial styling step must not count equally with payment architecture.

## Safety rules

Never let model-generated arbitrary shell text execute without validation.

Shell execution must define:

- cwd;
- timeout;
- captured stdout/stderr;
- exit status.

Do not expose secrets in prompts or logs.

## Tests for Stepper itself

Write comprehensive tests for:

- schema validation;
- DAG cycle detection;
- READY resolution;
- planner ordering;
- lock conflicts;
- state transition validation;
- retry/repair loop;
- persistence/restart;
- idempotent attempt recording;
- coverage detection;
- change impact;
- worktree logic where testable.

Provide a sample project demonstrating:

- dependencies;
- parallel steps;
- failed verification;
- repair;
- blocked step;
- manual review;
- successful completion.

## Required documentation

Document:

- architecture;
- CLI;
- schemas;
- adding a coding-agent adapter;
- building a Blueprint compiler;
- writing steps;
- execution lifecycle;
- debugging failures;
- recovery after process crash;
- Git/worktree workflow;
- release readiness.

## Working style

Do not stop after writing architecture notes.

Actually implement the engine, run its test suite, fix failures, and leave a runnable repository.

Use the companion documents in this pack as canonical requirements.

## Final acceptance

The Stepper implementation is complete only when:

```text
CLI works
+ unit/integration tests pass
+ sample project executes
+ failed step repairs correctly
+ restart/resume works
+ graph validation works
+ tracker is durable
+ reports generate
+ release-check works
```

Do not declare completion before this.

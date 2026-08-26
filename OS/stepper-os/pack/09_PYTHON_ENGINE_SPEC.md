# Python Stepper Engine — Detailed Specification

## Runtime

Python 3.12+

## Suggested dependencies

```text
pydantic>=2
typer
rich
pyyaml
networkx
sqlmodel
```

Keep shell/Git logic explicit and testable.

## Core models

```python
class StepStatus(str, Enum):
    PENDING = "PENDING"
    READY = "READY"
    RUNNING = "RUNNING"
    VERIFYING = "VERIFYING"
    FAILED = "FAILED"
    BLOCKED = "BLOCKED"
    DONE = "DONE"
    SKIPPED = "SKIPPED"
    SUPERSEDED = "SUPERSEDED"
    STALE = "STALE"
```

Use Pydantic for immutable specs and SQLModel/DB rows for mutable runtime state.

## Key services

### Loader

Loads:

- manifest;
- modules;
- epics;
- slices;
- steps.

Validates schema and uniqueness.

### Graph

NetworkX DAG.

Functions:

```python
validate_acyclic()
hard_dependencies(step_id)
downstream(step_id)
critical_path()
```

### Planner

Produces ranked candidates + explanation.

### Scheduler

Turns candidates into safe execution wave based on locks/WIP.

### ContextCompiler

Builds agent request.

### Executor

Coordinates:

```text
state transition
worktree
agent call
artifact capture
verification
repair
integration
```

### Verifier

Executes acceptance checks deterministically.

### Tracker

Persists events/attempts/reviews/artifacts.

### Reporter

Generates markdown and JSON status reports.

## CLI

Typer recommended.

```python
app = typer.Typer()
```

Commands should return nonzero exit code when validation/release check fails.

## Safe shell

Use `asyncio.create_subprocess_exec` or `subprocess.run` with argv lists.

Avoid `shell=True` for model-generated commands.

Capture stdout/stderr with truncation/redaction policy.

## Restart safety

On startup reconcile runtime state:

- RUNNING step with no live process becomes INTERRUPTED/FAILED or resumable attempt;
- worktree state is inspected;
- locks held by dead workers are reconciled;
- committed successful work is never repeated blindly.

## Idempotency

Runtime operations such as recording attempts and step completion should be transactionally consistent.

A second process should not claim the same step while it is locked/running.

## Multi-worker

If supporting multiple workers, use DB-backed lease/lock ownership with TTL/heartbeat rather than only in-memory locks.

## Release check

A configurable release target identifies required modules/slices/steps and gates.

`stepper release-check` must produce clear PASS/FAIL plus exact blockers.

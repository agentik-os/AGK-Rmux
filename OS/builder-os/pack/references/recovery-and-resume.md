# Recovery, Reconciliation, and Resume

## Contents

1. Recovery principle
2. Start-of-recovery protocol
3. State reconciliation matrix
4. Worktree and lock recovery
5. Command/migration recovery
6. Safe resume
7. Checkpoints

## 1. Recovery principle

Assume processes, sessions, networks, tests, and agent calls can stop at any point. Persist state before and after every consequential boundary. Recovery must derive truth from Tracker, Builder journal, repository, process/worktree state, and external receipts—not conversation history.

## 2. Start-of-recovery protocol

1. load the last valid Builder checkpoint and append-only events after it;
2. verify state checksum and input fingerprints;
3. inspect Stepper status and reconcile its RUNNING/VERIFYING steps;
4. inspect repository branch/HEAD/index/worktrees and attempt markers;
5. identify live/dead workers and leases;
6. inspect incomplete command, migration, integration, and external-action receipts;
7. classify each interrupted attempt;
8. re-run only safe idempotent verification or resume from an explicit checkpoint;
9. release/reclaim locks transactionally;
10. ask Stepper Planner for the next wave.

## 3. State reconciliation matrix

| Builder state | Repository/process evidence | Action |
| --- | --- | --- |
| CLAIMED/CONTEXT_READY | no edits/process | expire or resume same attempt |
| IMPLEMENTING | dirty attempt worktree, worker dead | preserve diff; mark INTERRUPTED; inspect before resume |
| IMPLEMENTED | diff/head present | verify context/spec hash, then continue to VERIFYING |
| VERIFYING | command outcome absent | rerun only if safe/idempotent; otherwise inspect side effects |
| REVIEWING | review artifact absent | reissue review with same immutable inputs |
| INTEGRATING | target unchanged, commit absent | retry policy transaction; otherwise inspect conflict/partial state |
| POST_MERGE_VERIFYING | integrated revision known | rerun required post-merge checks |
| SUCCEEDED | integrated revision/evidence missing | invalidate success and block/reconcile |

Never repeat a potentially external or destructive action solely because its response was lost. Search for an idempotency key, provider receipt, migration marker, commit, or other durable evidence first.

## 4. Worktree and lock recovery

- Match worktree path, branch, HEAD, step, attempt, and lease.
- Preserve uncommitted changes until ownership is known.
- Reclaim a dead lease only after its worker/process cannot still mutate resources.
- Detect orphaned worktrees and pending commits; do not delete automatically.
- Recalculate resource conflicts before new claims.
- Keep integration locks narrow and time-bounded with heartbeat/owner.

## 5. Command/migration recovery

For ordinary read-only checks, rerun when inputs are unchanged. For generators/builds, verify outputs before rerun. For database migrations/backfills:

- inspect migration table/checkpoint/progress marker;
- verify transaction semantics and partial side effects;
- use the documented resume/forward-recovery path;
- never reverse or rerun against production without explicit authorization;
- run integrity checks before continuing dependent work.

For payments, messages, bookings, infrastructure, or deployment actions, require idempotency/receipt reconciliation.

## 6. Safe resume

Resume the same attempt when contract/context/base remain valid and preserved work is trustworthy. Start a new attempt when the spec/context/base changed, the prior environment is invalid, or evidence cannot be trusted. Supersede rather than erase the prior attempt.

If Blueprint or Stepper fingerprints changed, stop normal resume and apply change governance first.

## 7. Checkpoints

Checkpoint after:

- preflight completion;
- step claim;
- implementation result;
- each verification/review batch;
- integration;
- Stepper transition;
- gate snapshot;
- pause/block/decision request;
- final handoff.

A checkpoint includes revision, checksum, current wave, active attempts/locks, exact next action, blockers, input fingerprints, repository revisions, and event-log offset.

# Oracle lifecycle contract

An oracle owns a mission from classification through accepted delivery. The
authoritative state is the transactional V3 mission ledger, not an rmux pane,
transcript, Telegram card, or JSON file.

The governing doctrine is compiled from `crates/omega-core/src/rules.rs`. Print
the current oracle context with:

```bash
omega rules context oracle
```

## State authority and projections

The canonical on-host ledger is:

```text
~/.omega/state/mission-engine-v3.sqlite3
```

`crates/omega-core/src/mission_ledger.rs` is the only mission write authority.
It atomically commits events, materialized mission projections, task attempts,
leases, and delivery outbox work. External effects are at-least-once and must be
idempotent.

Other surfaces are compatibility or presentation projections:

| Surface | Purpose | Authority |
|---|---|---|
| `mission-engine-v3.sqlite3` | events, mission state, plan revisions, attempts, leases, outbox | authoritative |
| `oracle-<key>.progress.json` | live task checklist and Telegram rendering | mutable projection |
| `oracle-<key>.done.json` | oracle completion candidate | compatibility projection |
| `worker-<session>.done.json` | worker completion candidate | compatibility projection |
| `scope-<session>.json` | human-readable scope view | compatibility projection backed by a lease |
| timeline and Telegram cards | operator visibility | rebuildable projections |

A V3 compatibility projection carries its ledger source, mission identity,
event/version position, and projection hash. A consumer validates those fields
against the authoritative ledger before acting. A forged, stale, future, or
legacy projection cannot override ledger state or make a worker terminal.

## 1. Freeze mission identity

At creation, persist the immutable mission identity: mission id, project,
original request, and resolved worktree. Resume and mutation paths must reject a
projection whose project, request, worktree, phase, or worker binding does not
match the ledger.

Project names alone are not workspace identity. Dispatch and resume use the
exact resolved repository/worktree recorded for the mission.

## 2. Persist a typed plan

Enumerate every operator deliverable in the requested order. Persist a
`PlanContract` with a monotonically increasing revision, typed task ids,
dependencies, acceptance criteria, and real verifier checks. Discovered work is
appended or introduced by a new plan revision; it never silently replaces an
original deliverable.

Task attempts bind to the exact `(mission, plan revision, task id, attempt)`
tuple. A stale attempt from an older revision can be accepted only when the
ledger proves that task is unchanged and still active under the current plan.

For operator visibility, mirror the checklist with:

```bash
omega progress <session> --plan "audit|implement|verify|deliver"
omega progress <session> --task "audit" --status doing
```

This JSON progress surface is not the mission authority. After restart or
compaction, resume from the ledger-backed plan and attempts; use progress JSON
only as a presentation aid.

## 3. Dispatch exact attempts

Before a worker writes:

- resolve the authoritative project and worktree;
- bind the worker to an active task attempt;
- acquire the declared file scope with its generation/fencing identity;
- record the dispatch transition in the ledger;
- pass explicit acceptance and verification checks to the worker.

Overlapping file writers are serialized or isolated. A scope release must match
the current owner and generation, so an old process cannot release a newer
worker's claim after an ABA cycle.

## 4. Verify candidates independently

`omega done` writes a completion candidate. It is an input, never the verdict.
The oracle or independent verifier must run the declared checks against the
exact revision and workspace, then append the accepted or rejected transition
to the ledger. A `done.json` without valid ledger ancestry never makes an
attempt terminal.

Evidence records what a command confirmed and what it did not. Missing
credentials, unreachable delivery, a warning, or an unrun production path is a
negative or unverified state, not a pass.

## 5. Resume and crash recovery

On process restart, rmux restart, or context compaction:

1. open the canonical ledger;
2. validate its filesystem identity and replay/materialized projection parity;
3. load the immutable mission, active plan revision, and task attempts;
4. reconcile compatibility JSON only when its ledger receipt validates;
5. reacquire or inspect leases using current fencing tokens;
6. continue the first required nonterminal task.

Never infer completion from a missing pane. Never reconstruct authority from a
Telegram card or transcript.

## 6. Closure and delivery

Clean closure requires every required task attempt to have an accepted terminal
ledger state, every bound worker to be accounted for, required scope/worktree
leases to be safely released, and delivery work to be recorded. A pending,
failed, or blocked mission stays honest about what remains.

Delivery uses the ledger outbox so a crash between external send and local
acknowledgement can be retried. Retries are bounded and idempotency-aware;
at-least-once transport must not create multiple logical completions.

Re-running reconciliation or closure must be safe. It may repair projections or
finish an unacknowledged outbox item, but it must not repeat a destructive
cleanup, accept an obsolete task attempt, or release another generation's
scope.

## Operator inspection

```bash
omega status <session>
omega timeline <oracle>
omega rules context oracle
omega doctor --deep
```

These commands provide evidence and projections. The ledger remains the
authority for acceptance, resume, and delivery decisions.

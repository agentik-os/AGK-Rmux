# OmegaOS Orchestration V3

Status: implemented release candidate, final verification in progress
Date: 2026-07-30
Decision record: OMEGA-ORCH-V3

## Objective

OmegaOS must make mission progress, ownership, evidence, verification, and delivery explicit and replayable. It must not infer correctness from an agent narrative, a session name, a pane transcript, or the presence of a Git object.

The design favors the simplest topology that meets the mission. A single agent remains the default for bounded work. Parallel workers, handoffs, and councils are selected only when task topology, risk, or required capabilities justify their cost.

## Non-negotiable invariants

1. The event store is the only write authority for mission and task state.
2. A worker can claim completion, but cannot accept its own work.
3. Scope, dependencies, final notification, and 100% reporting remain locked until acceptance.
4. Every state transition uses an expected version, an idempotency key, and a current fencing token where a lease applies.
5. External effects are delivered at least once from a transactional outbox and are idempotent at the handler boundary.
6. Legacy JSON, timeline, Telegram, and harness plan surfaces are projections, never authorities.
7. Provider adapters expose capabilities; policy stays in the engine.
8. A missing required capability fails before spawn or routes to a compatible provider. There is no silent fallback.
9. Verification executes only checks approved before work began, inside a constrained environment.
10. Reports derive from accepted evidence and never display completion before acceptance.

## State machines

### Task attempt

```text
queued -> running -> candidate_done -> verifying
verifying -> accepted | correction_required | blocked | failed
correction_required -> running
blocked -> queued | running | failed | cancelled
queued | running | correction_required -> cancelled
accepted | failed | cancelled are terminal
```

Direct transitions from `running` or `candidate_done` to `accepted` are invalid.

### Mission

```text
created -> classified -> planned -> running -> verifying
verifying -> accepted | correction_required | blocked | failed
correction_required -> running
blocked -> planned | running | failed | cancelled
accepted -> reporting -> delivered
created | classified | planned | running | correction_required -> cancelled
delivered | failed | cancelled are terminal
```

A mission can enter `accepted` only when all required task attempts are accepted, the DAG is satisfied, required gates and audits pass, and no approval or blocker remains open.

`blocked` always carries a typed reason, required authority, and resume condition. Resuming it requires a recorded approval or resolved dependency event. It can never remain an ambiguous dead state.

## Plan contract

The authoritative DAG is an immutable, versioned `PlanContract` persisted in the event store:

```text
plan_id
mission_id
revision
schema_version
created_from_version
tasks[]
dependencies[]
required_gates[]
required_approvals[]
content_digest
```

Each task has a stable task ID, acceptance criteria, predeclared verifier checks, required capabilities, scope, risk, retry policy, and dependency IDs. Attempts are separate records and always reference the plan revision they implement.

A plan may be replaced freely before execution begins. Once a task has started, changes require a `PlanAmended` event under expected-version CAS. An amendment cannot remove or weaken an accepted task, erase evidence, create a dependency cycle, or reinterpret an existing attempt. Material changes create new task IDs and attempts. Mission acceptance evaluates exactly one recorded plan revision and its complete dependency closure.

## Event envelope

Every event carries:

```text
event_id
mission_id
task_id?
attempt_id?
sequence
expected_version
schema_version
idempotency_key
actor
provider?
causation_id?
correlation_id
fencing_token?
recorded_at
kind
payload
```

The reducer is pure. Given the same ordered events, it must produce the same projection hash.

## Persistence

The single-host implementation uses SQLite WAL:

- `missions`: stable identity and current materialized version
- `events`: append-only ordered events, unique `(mission_id, sequence)`
- `task_attempts`: materialized task and attempt state
- `leases`: owner, resource scope, expiry, monotonically increasing fencing token
- `outbox`: pending external effects with a unique idempotency key

Appending an event, updating its materialized state, and enqueuing resulting effects happen in one transaction. A writer supplies `expected_version`; a stale writer receives a conflict and must reload. Retrying the same `idempotency_key` returns the already-recorded event without applying it again.

External systems cannot participate in that transaction. The outbox worker therefore provides a truthful, duplicate-tolerant at-least-once guarantee. Each operation has a reconciliation strategy: edit an existing Telegram message when its remote ID is known, query remote state when the API supports it, and otherwise record a possible duplicate after a send-success/local-ack crash. OmegaOS never claims exactly-once delivery without a remote idempotency primitive.

## Evidence boundary

Three records are distinct:

- `EvidenceClaim`: what a worker says exists.
- `EvidenceObservation`: what an independent verifier actually observed.
- `AcceptanceDecision`: what the MissionEngine accepted or rejected.

Verifier commands are fixed in the task contract before execution. The verifier uses a pinned revision or worktree, explicit cwd, clean environment without inherited secrets by default, time and output caps, redaction, and immutable output hashes. Worker-supplied arbitrary commands are never executed merely because they appeared in a done message.

URL checks allow only declared HTTP(S) targets. Loopback, link-local, RFC1918, cloud metadata, `file://`, redirects outside policy, and unapproved hosts are rejected.

## Leases and scope

Each task attempt acquires a fenced lease. The lease token increases whenever ownership changes. Writes, candidate completion, acceptance, renewal, and release require the current token. A paused old worker cannot write after a replacement owns a newer token.

Scopes are normalized to repository-relative paths. Exact files, directories, normalized aliases, and declared globs are checked for overlap. Parallel mutation requires isolated worktrees. If isolation cannot be established, parallel dispatch fails closed.

## Routing

Routing emits a versioned decision:

```text
topology
risk
confidence
reasons
required_capabilities
recommended_parallelism
quality_policy
router_version
```

Deterministic guards inspect blast radius, irreversibility, data and production effects, security, ambiguity, file topology, acceptance surface, and explicit user enumeration. French and English equivalents must preserve topology and risk. Low confidence escalates conservatively rather than under-routing.

Provider selection happens after the required capabilities are known.

## Provider capability contract

Each provider adapter declares support for:

- launch
- stable session identity
- send
- inspect
- cancel
- timeout
- resume or explicit unsupported status
- model and effort selection
- tool and MCP policy
- progress event normalization
- skill discovery convention
- plan projection

The adapter implements mechanisms only. Mission policy, safety, verification, and completion stay provider-neutral.

## Rules context compiler

The doctrine is compiled from:

1. a concise normative law kernel;
2. one role contract;
3. mission and project rules selected by typed domains and risk;
4. provider mechanics selected from capability metadata;
5. skill references, with full skill bodies loaded only when selected.

Rule metadata includes category, scope, risk, enforcement mechanism, providers, dependencies, conflicts, runbook, version, and lifecycle status. The compiler measures the complete injected context, including platform overlap. Exceeding the 24 KB OmegaOS budget is an error with diagnostics, never silent truncation.

## Skill catalog compiler

`SkillCatalogV1` scans explicit OmegaOS-owned roots. It excludes `node_modules`, `.venv`, build output, vendor trees, and escaping symlinks. It validates current universal fields first, then migrates version, provenance, compatibility, risk, dependencies, and verification metadata from warnings to required fields.

The parser boundary is replaceable and must use a maintained implementation with explicit byte, depth, document, and alias budgets. The first implementation uses `serde-saphyr`, not the deprecated `serde_yaml 0.9` parser.

Each skill and provider pair has exactly one state:

```text
enabled
excluded(reason)
unsupported(missing_capability)
```

The canonical intermediate representation has stable ordering, normalized relative paths, canonical serialization, and a cryptographic content digest that excludes timestamps, mtimes, and absolute paths. Atlas, lexical retrieval, optional embeddings, Claude, Codex, documentation, and install generations derive from this IR.

Compilation validates collision by name, alias, case, Unicode normalization, and provider slug. Dependency references must exist and the DAG must be acyclic. Verification references are validated at compile time but executed only by the constrained runtime.

Install output is generation-based. A complete catalog is staged in a new owned generation, validated, fsynced, then exposed through an atomic `current` pointer swap. The previous generation remains available for rollback. A crash produces either the old complete generation or the new complete generation, never a mixture. The ownership manifest lists every generated path; install and rollback never delete unowned local skills.

## Compatibility migration

1. Freeze legacy fixtures and enumerate all readers, writers, and scope-release sites.
2. Add the event store, reducers, contracts, and catalog compiler in shadow mode.
3. Mirror legacy actions into events through one ingress adapter. Imported `DoneClean` becomes `legacy_candidate_unverified`.
4. Generate legacy files from the ledger and compare semantic projections.
5. Run the verifier in observe-only mode, then enforce it for canary missions.
6. Move one complete ingress path at a time to the MissionEngine. The engine choice is fixed when a mission is created.
7. Switch readers one at a time. A rollback switches readers, never rewrites ledger history.
8. Remove legacy writers only after zero divergence, crash tests, a clean install, and rollback tests.

Every generated projection carries `projection_source=mission_engine`, mission ID, source sequence, and schema version. The legacy ingress rejects those marked projections, preventing a generated file from being reingested as a new event.

## Release gates

The release is not called 100% until all of these pass:

1. CAS contention and duplicate idempotency tests.
2. Event replay and projection hash determinism.
3. Crash injection around event commit and outbox delivery.
4. Lease ABA and stale worker rejection.
5. No-op done becomes correction required and retains scope.
6. Aggregate mission acceptance across task, gate, audit, and approval states.
7. Verifier RCE, SSRF, timeout, redaction, and evidence-freshness tests.
8. Legacy projection rebuild and split-brain ingress tests.
9. At least 100 paired French and English routing cases with zero high-risk under-classification.
10. Provider capability conformance.
11. Property test proving no release, final notification, 100%, or delivery before acceptance.
12. Skill catalog closure, deterministic digest, rollback, and explicit provider-state tests.
13. Context compiler below 24 KB with no duplicate body.
14. Fresh clone, install, sync, build, full test suite, and live golden paths.

## Implemented release-candidate surfaces

- `mission_ledger.rs` owns mission events, plan revisions, task attempts, fenced leases, and the transactional outbox.
- `omega done` records candidate evidence and compatibility projections with source provenance. It does not self-accept work.
- Oracle delivery requires accepted attempts plus recorded gate results.
- The rules compiler emits provider-aware, role-scoped contexts below a hard byte budget.
- `SkillCatalogV1` is the source for install, provider links, Atlas, and RAG generation.
- `skills/audits/registry.toml` is the only audit catalogue consumed by Rust and the shell runner.
- The rmux view reflows grapheme clusters, preserves ANSI styles, maps the cursor after reflow, and identifies Codex from persisted session metadata.
- Fresh configurations select Codex. Existing explicit provider selections are preserved.

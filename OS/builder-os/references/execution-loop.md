# Deterministic Execution and Repair Loop

## Contents

1. Execution state machine
2. Wave protocol
3. Per-step transaction
4. Context compilation
5. Implementation discipline
6. Evidence model
7. Repair loop
8. Integration transaction

## 1. Execution state machine

Keep Stepper status authoritative. Maintain a Builder attempt state underneath it:

```text
CREATED
→ CLAIMED
→ CONTEXT_READY
→ IMPLEMENTING
→ IMPLEMENTED
→ VERIFYING
→ REVIEWING
→ INTEGRATING
→ POST_MERGE_VERIFYING
→ SUCCEEDED
```

Failure branches:

```text
VERIFYING/REVIEWING/INTEGRATING
→ FAILED
→ REPAIRING
→ IMPLEMENTED

any unsafe state → BLOCKED
process loss → INTERRUPTED
failed integration recovery → ROLLED_BACK or BLOCKED
```

Builder must never set Stepper `DONE` directly unless it is the configured Verifier adapter applying an allowed transition after all gates pass.

## 2. Wave protocol

For each Stepper Planner wave:

1. verify every candidate remains `READY`;
2. verify all hard dependencies are `DONE` at the same Tracker revision;
3. recalculate overlapping file/path/domain/schema/migration/integration locks;
4. respect WIP, active-module, compute, API, and review limits;
5. allocate isolated worktrees/branches when configured;
6. atomically claim steps and leases;
7. execute independent lanes;
8. integrate in dependency-safe order;
9. re-plan after each completion, blocker, stale event, or material repository change.

Parallelism is an optimization, not a requirement. Reduce concurrency when shared schemas, migrations, cross-cutting types, generated clients, or unstable tests create integration risk.

## 3. Per-step transaction

### Claim

Persist step ID, spec hash, Blueprint/Stepper fingerprints, base revision, worker identity, lease, locks, attempt number, and timestamp. Reject double claims.

### Hydrate

Load:

- the complete immutable step spec;
- exact accepted Blueprint records referenced by the step;
- dependency output contracts and evidence;
- relevant repository files/tests/configuration;
- active ADRs/change sets;
- failure evidence from the immediately relevant prior attempts;
- applicable repository instructions.

### Preflight

Check dependencies, paths, environment capabilities, base cleanliness/preservation, lock ownership, contract hash, and manual gates. Block on mismatch; do not edit first and explain later.

### Micro-plan

Record a concise plan with:

- interfaces/invariants to preserve;
- files expected to create/modify;
- implementation order;
- test additions and commands;
- security/UX/AI/data concerns;
- docs and observability updates;
- rollback approach.

The micro-plan may refine tactics, not scope.

### Implement

Produce the smallest complete vertical change. Include required errors, permissions, states, events, instrumentation, tests, migration compatibility, and documentation. Reuse canonical domain logic and established abstractions.

### Diff self-check

Before independent verification:

- inspect the full diff against the attempt base;
- identify accidental generated/binary/lockfile changes;
- confirm forbidden paths and unrelated behavior were untouched;
- run focused developer checks;
- summarize unresolved issues honestly.

## 4. Context compilation

Use a step-scoped context bundle:

```text
authority header
+ immutable step contract and hash
+ exact Blueprint refs and prohibited shortcuts
+ dependency contracts/artifacts
+ targeted current code and tests
+ active ADR/change-set excerpts
+ relevant prior failure evidence
+ repository instructions
```

Attach source locators and versions. Keep secrets out. Do not dump the full Blueprint or entire repository when targeted retrieval is possible.

Invalidate a compiled context bundle when any hashed source changes.

## 5. Implementation discipline

- Read before edit.
- Preserve architecture and naming unless the contract changes them.
- Keep domain authority server-side where specified.
- Avoid duplicate business logic and vendor leakage.
- Treat types, errors, reason codes, events, analytics, and audit records as contracts.
- Preserve backward compatibility across rolling deployments where required.
- Make migrations resumable/idempotent where required.
- Do not mock away required integration truth.
- Do not broaden a step for attractive refactors.
- Register incidental work instead of smuggling it into the diff.

## 6. Evidence model

Capture for every attempt:

- input and prompt/context hashes;
- base/head revisions and diff digest;
- files created/modified/deleted;
- commands as argv or normalized safe command descriptions;
- start/end times, exit code, redacted stdout/stderr summary;
- tests/checks and artifact digests;
- reviews with reviewer role, result, findings, and evidence;
- docs and runbooks updated;
- known issues and decision/blocker links;
- integration and post-integration results.

Evidence must be append-only. Corrections supersede prior evidence; they do not erase it.

## 7. Repair loop

1. classify the failed check;
2. retain the minimal reproducible evidence;
3. decide whether the failure is in implementation, test, environment, dependency, specification, security, data, integration, infrastructure, or external service;
4. preserve correct work and original step scope;
5. compile a repair context from the contract plus failure evidence;
6. fix the root cause;
7. rerun the failed check and required regression set;
8. update the attempt journal;
9. stop at configured limit or sooner if further mutation is unsafe.

Never weaken requirements, delete a valid failing test, alter expected output without governance, or repeatedly rewrite architecture at random.

## 8. Integration transaction

After isolated verification and review:

1. ensure the base/integration target has not changed incompatibly;
2. rebase or refresh only through the configured policy;
3. resolve conflicts with both step contracts in view;
4. integrate in dependency-safe order;
5. run required post-integration smoke/regression/schema/generated-code checks;
6. record the integrated revision;
7. roll back/revert or block on failure according to the step rollback contract;
8. only then submit completion evidence to Stepper Verifier.

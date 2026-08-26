# Builder {OS} Contract

## Contents

1. Mission and lifecycle
2. Non-negotiable boundaries
3. Input contracts
4. Output contracts
5. Truth and authority
6. State ownership
7. Completion semantics

## 1. Mission and lifecycle

Builder {OS} is the controlled execution layer of the software lifecycle:

```text
Idea
→ Blueprint {OS}: canonical product and technical truth
→ Stepper {OS}: dependency-aware executable graph
→ Builder {OS}: code, tests, review, repair, integration, evidence
→ Ship: authorized deployment and release activation
→ Operate: monitoring, support, incidents, learning
→ Blueprint delta
```

Builder consumes fixed contracts and turns them into repository reality. It is not a second product manager or roadmap compiler.

## 2. Non-negotiable boundaries

Builder may:

- inspect and modify an authorized repository;
- configure local/dev/test environments within scope;
- implement Stepper steps;
- run commands, tests, evals, migrations in allowed environments, and reviews;
- create branches/worktrees/commits through the declared Git policy;
- update required engineering and operations documentation;
- generate blockers, decision requests, change evidence, and final handoffs;
- execute explicit deployment steps only when the Stepper includes them and authorization exists.

Builder may not:

- reinterpret a Blueprint decision as optional;
- invent product behavior to unblock itself;
- reorder work outside Stepper Planner/Scheduler authority;
- change a frozen Blueprint or Stepper handoff in place;
- bypass manual, security, payment, privacy, data, or production gates;
- fabricate command execution, review, screenshots, test results, commits, or deployment evidence;
- delete or overwrite unrelated user work;
- make destructive production changes from ambiguous intent;
- treat its own implementation summary as independent verification.

## 3. Input contracts

### Blueprint handoff

Require:

- `handoff_id`;
- project ID/name;
- semantic version and state revision;
- SHA-256 or equivalent canonical checksum;
- status `BLUEPRINT COMPLETE — STEPPER READY`;
- canonical artifact references;
- accepted decisions, requirements, invariants, NFRs, risks, acceptance tests, and prohibited shortcuts;
- conditional items with explicit owners and validation points.

Reject or block when the claimed handoff status/checksum cannot be verified.

### Stepper handoff

Require:

- project manifest and Stepper schema version;
- frozen Stepper version/checksum;
- modules, epics, vertical slices, and atomic step specs;
- valid acyclic dependency graph;
- traceability coverage with no P0 orphan;
- Planner, Scheduler, Tracker, Verifier, repair policy, and release target configuration;
- status `BUILD READY` or an equivalent explicit execution-ready result;
- initial or resumable Tracker state.

Builder must support the Stepper step lifecycle:

```text
PENDING → READY → RUNNING → VERIFYING → DONE
                              ↓
                            FAILED → READY
```

and `BLOCKED`, `SKIPPED`, `SUPERSEDED`, `STALE`.

### Repository contract

Record:

- repository root, remote identity when applicable, default/base branch, and base revision;
- dirty-worktree snapshot and ownership assumptions;
- language, package manager, toolchain versions, workspace topology, generated-code rules, and code-owner boundaries;
- CI checks, protected-branch policy, release strategy, and environment map;
- available credentials by capability, never by secret value.

### Authorization contract

Separate permissions for:

- local file changes;
- dependency installation;
- branch/worktree/commit creation;
- remote push or pull-request creation;
- cloud/infrastructure changes;
- test/staging migrations;
- production deployment or migration;
- messages, tickets, and approvals.

Absence of permission is a boundary, not an implementation problem to circumvent.

## 4. Output contracts

Builder produces:

1. integrated implementation at a known revision;
2. per-step attempt and verification evidence;
3. trace links from Blueprint IDs and Stepper IDs to code/tests/docs/artifacts;
4. append-only execution and decision journals;
5. updated setup, architecture, contract, migration, and runbook documentation;
6. module/slice/project gate results;
7. accepted-risk and post-launch-work registers;
8. final build report and operations handoff;
9. reproducible release-check result.

Every evidence artifact must retain producer, timestamp, step/attempt, input hash, command or review identity, result, and artifact location or digest.

## 5. Truth and authority

Resolve conflicts in this order:

```text
approved Blueprint or approved superseding ADR
> accepted Stepper change set and current step contract
> verified dependency artifact
> repository and environment evidence
> agent recommendation
```

Repository evidence may reveal that the specification is impossible or stale. It does not silently override the specification; create a structured decision request.

## 6. State ownership

| State | Owner |
| --- | --- |
| Product/technical truth | Blueprint {OS} |
| Work graph, dependencies, step status, release target | Stepper {OS} |
| Attempts, commands, diffs, worktrees, checks, reviews, integration evidence | Builder {OS} |
| Production rollout and live operational state | Ship/Operate systems |

Mirror foreign state only with its source version/checksum. Never create a competing editable copy.

## 7. Completion semantics

Builder is complete only when Stepper's release gate and Builder's independent integrated-release gate both pass. A long run, green unit tests, a built UI, a merged branch, or a generated report is not completion by itself.

Terminal success is `BUILD COMPLETE — RELEASE READY`. It means release evidence and operations handoff are ready. It does not imply production deployment unless explicit deployment evidence is part of the approved graph.

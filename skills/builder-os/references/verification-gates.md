# Verification, Reviews, and Quality Gates

## Contents

1. Independent verification
2. Risk matrix
3. Check evidence
4. Domain-specific gates
5. Twenty project gates
6. No-fake-done tests

## 1. Independent verification

Treat implementation output as a candidate. The Verifier executes objective checks against the repository/worktree and immutable step contract. Reviews assess risks that commands alone cannot prove.

Verification order:

```text
artifact and diff checks
→ focused unit/domain tests
→ integration/contract tests
→ typecheck/build/lint
→ security/architecture/data/UX/AI reviews
→ acceptance predicates
→ isolated-step PASS
→ integration
→ post-integration checks
→ Stepper DONE
```

## 2. Risk matrix

| Risk | Required minimum |
| --- | --- |
| LOW | focused tests, typecheck/lint or equivalent, diff/forbidden-change check |
| MEDIUM | LOW + integration/contract evidence + architecture sanity |
| HIGH | MEDIUM + relevant E2E + specialist reviews + rollback evidence |
| CRITICAL | all applicable automation + security/architecture/data review + manual gates where declared + release/canary constraints |

Never downgrade risk merely to reduce verification cost.

## 3. Check evidence

For each check persist:

```json
{
  "check_id": "CHK-...",
  "step_id": "STEP-...",
  "attempt_id": "ATT-...",
  "kind": "command|review|predicate|visual|eval",
  "input_hash": "sha256:...",
  "result": "PASS|FAIL|BLOCKED|N/A",
  "exit_code": 0,
  "summary": "...",
  "artifact_refs": [],
  "started_at": "...",
  "finished_at": "..."
}
```

Retain actual command identity, environment label, revision, redacted output, and artifact digest. `N/A` requires rationale and gate owner acceptance when the check was expected by policy.

## 4. Domain-specific gates

### Code and architecture

- required files/contracts exist;
- no forbidden changes or imports;
- dependency direction and domain ownership hold;
- business logic is not duplicated;
- public APIs, errors, events, and reason codes match contracts;
- compatibility and deprecation rules are respected;
- new architecture decisions are documented and approved.

### Security, privacy, and abuse

- authn/authz and cross-tenant isolation;
- server authority and least privilege;
- injection/path traversal/unsafe deserialization/shell execution;
- CSRF/SSRF/XSS and platform-relevant threats;
- secret and sensitive-data leakage;
- encryption/retention/deletion/export/consent where required;
- rate limits, abuse controls, auditability, and dependency/supply-chain risk;
- no security claim without executable evidence or review.

### UX, visual, and accessibility

- success, loading, empty, error, denied, partial, retry/offline, and destructive-confirmation states;
- keyboard/screen-reader/focus/contrast/touch-target behavior;
- responsive layout, safe areas, reduced motion, localization, and text overflow;
- design tokens/components and project aesthetic;
- browser/device evidence or visual regression for critical flows;
- no mocked action presented as live functionality.

### Data and migrations

- forward migration on representative schema/data;
- backward/rolling-deploy compatibility when required;
- idempotent/resumable backfill;
- integrity, uniqueness, isolation, locking, and concurrency;
- rollback or documented forward-recovery procedure;
- backup/restore and observability for high-risk changes;
- no production migration without explicit authorization.

### AI and agents

- typed tool contract and server-side authorization;
- context minimization, provenance, isolation, and prompt-injection resistance;
- memory write/read/delete/consent rules;
- uncertainty, abstention, escalation, confirmation, and human override;
- golden/adversarial/safety eval sets with versioned thresholds;
- latency/cost/token/tool-call budgets;
- traces and receipts without sensitive leakage;
- idempotency, compensation, and rollback for side effects.

### Performance, reliability, and operations

- measurable latency/throughput/resource budgets;
- load, concurrency, timeout, retry, circuit-breaker, queue, cache, and idempotency behavior;
- graceful degradation and dependency failure;
- logging, metrics, traces, alerts, dashboards, SLOs, and runbooks;
- backup, restore, disaster recovery, and incident procedure where required.

## 5. Twenty project gates

| ID | Gate | Pass condition |
| --- | --- | --- |
| BG01 | Input Integrity | frozen Blueprint and Stepper identities/checksums verified |
| BG02 | Repository Baseline | base revision and pre-existing state recorded; no unsafe unknown changes |
| BG03 | Setup and Toolchain | reproducible install/build/test setup documented and passing |
| BG04 | Graph Alignment | required Stepper graph valid; no P0 orphan/stale unreviewed step |
| BG05 | Code Quality | build/typecheck/lint/static rules pass as applicable |
| BG06 | Unit and Domain | required focused tests pass |
| BG07 | Integration and Contract | service/API/event/provider contracts pass |
| BG08 | E2E and Acceptance | required P0 user/operator flows and acceptance predicates pass |
| BG09 | Security, Privacy, Abuse | release security gate passes; no critical finding |
| BG10 | Architecture | boundaries, decisions, compatibility, and drift review pass |
| BG11 | UX, Accessibility, Visual | required states and target-device/browser evidence pass |
| BG12 | Data and Migration | schema, migration, backfill, integrity, and recovery gate pass |
| BG13 | AI and Evaluation | applicable AI eval/safety/tooling thresholds pass |
| BG14 | Performance and Reliability | applicable budgets, load, failure, and resilience checks pass |
| BG15 | Observability and Operations | monitoring, alerts, runbooks, backup/recovery readiness pass |
| BG16 | Documentation | setup/contracts/ADRs/migrations/runbooks are current and verified |
| BG17 | Integrated Revision Health | post-integration test matrix passes on the release revision |
| BG18 | Traceability and Evidence | required Blueprint/Stepper/code/test/docs evidence coverage meets thresholds |
| BG19 | Risks and Follow-up | no unaccepted critical risk; residual work registered and owned |
| BG20 | Release and Handoff | final report, rollback, deployment procedure, environment and operations handoff complete |

Critical gates cannot be `CONDITIONAL`. A noncritical conditional gate must identify condition, owner, due point, blocking boundary, and verifying step.

## 6. No-fake-done tests

Fail completion when any required truth is missing, including:

- a UI exists but uses mocks/placeholders for the required backend;
- code compiles but required commands were not run;
- hidden buttons substitute for server authorization;
- payment success lacks webhook reconciliation/idempotency;
- booking lacks capacity/concurrency protection;
- migration exists but rollback/recovery or compatibility was required and untested;
- AI text claims an action that no authorized tool executed;
- screenshots show one happy state while error/denied/loading states are absent;
- isolated branch checks pass but release revision checks fail or never ran;
- documentation/report says complete while the Tracker or gates disagree.

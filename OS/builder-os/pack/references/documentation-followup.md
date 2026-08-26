# Documentation, Reporting, and Follow-up

## Contents

1. Documentation as code
2. Required document classes
3. Step documentation ledger
4. Status reporting
5. Follow-up register
6. Evidence retention
7. Documentation verification

## 1. Documentation as code

Treat documentation as part of the implemented contract. Update it in the same step when code makes it false. Keep source-controlled docs close to the owner domain when practical. Link rather than duplicate canonical truth.

Documentation completion does not mean generating many pages. It means an authorized engineer/operator can set up, understand, test, release, recover, and operate the implemented behavior without guessing.

## 2. Required document classes

Maintain applicable:

- local setup and prerequisites;
- environment-variable names, purpose, scope, and acquisition process without values;
- architecture overview and approved ADRs;
- domain rules and invariants;
- API, tool, event, webhook, error, and reason-code contracts;
- data model, migrations, backfills, retention, and recovery;
- test/eval strategy and commands;
- deployment and rollback procedure;
- monitoring, alerts, SLOs, dashboards, and incident runbooks;
- security/privacy threat and control notes;
- AI model/tool/context/memory/eval/rollback contracts;
- known risks, debt, post-launch work, and accepted deviations.

## 3. Step documentation ledger

For each step record:

```yaml
step_id: STEP-000123
docs_required:
  - docs/domain/experience.md
docs_changed:
  - path: docs/domain/experience.md
    reason: eligibility reason-code contract changed
    verification: links-and-example-check
docs_not_applicable_reason: null
```

Do not mark `documentation_updated` from a code comment unless the contract explicitly treats it as sufficient.

## 4. Status reporting

Generate status from canonical state/Tracker, not memory. Include:

- Builder status and input fingerprints;
- weighted/raw progress from Stepper;
- modules/slices/steps by state;
- current wave and active attempts;
- critical path and lock contention;
- recent integrated revisions;
- verification trend and flaky/pre-existing failures;
- blockers/decisions/manual gates;
- release-gate snapshot;
- next Planner-selected work.

Do not estimate percentages when Stepper has authoritative weights.

## 5. Follow-up register

Every follow-up item requires:

```yaml
followup_id: FU-0042
kind: bug|debt|risk|enhancement|ops|security|experiment
severity: low|medium|high|critical
statement: precise work or risk
evidence_refs: []
blueprint_refs: []
step_refs: []
owner: role-or-person
target: pre-release|release|post-launch|date-or-version
blocking: false
acceptance: observable closure condition
status: open|accepted|scheduled|resolved|rejected
```

Critical unresolved follow-up blocks release. High items require explicit disposition. Do not hide deferred mandatory work under “future improvement.”

## 6. Evidence retention

Retain digests and durable references for commands, test reports, coverage, evals, screenshots, visual diffs, security scans, migration rehearsals, reviews, commits, and final release artifacts. Apply redaction and retention policies; do not store secrets or unnecessary personal data.

## 7. Documentation verification

Check applicable:

- commands and paths still exist;
- examples compile or execute where practical;
- links and anchors resolve;
- schemas/contracts match generated or source definitions;
- environment names match code/config;
- rollback/runbook steps are safe and ordered;
- version/status references are current;
- no document claims deployment or completion beyond evidence.

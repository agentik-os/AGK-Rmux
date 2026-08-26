# Change Impact, Decisions, and Scope Governance

## Contents

1. Frozen inputs
2. Blueprint delta
3. Stepper change set
4. Decision request
5. Incidental findings
6. Critical findings
7. Scope control

## 1. Frozen inputs

Bind every attempt to Blueprint and Stepper versions/checksums. Never follow a moving `latest` pointer during execution. Detect mismatches at session start, context compilation, step claim, and integration.

## 2. Blueprint delta

Accept only an explicit new Blueprint version and delta. Trace:

```text
changed decision/requirement/invariant/NFR/acceptance
→ affected modules/slices/steps
→ code/tests/data/docs/infrastructure/evals
→ completed steps requiring STALE/NEEDS_REVIEW
→ release-gate impact
```

Preserve original handoff, attempts, commits, evidence, and gate results as historical truth.

## 3. Stepper change set

Require an accepted change-set artifact containing:

- previous/new Stepper fingerprint;
- changed or new step IDs;
- stale, superseded, regenerated, and supplemental steps;
- dependency/lock/release-target changes;
- migration/integration ordering;
- rationale and Blueprint/ADR refs.

Builder executes the changed graph after deterministic validation. It does not regenerate the graph itself.

## 4. Decision request

Raise a structured request when implementation reveals a real contract gap or conflict:

```yaml
decision_request_id: DR-0021
step_id: STEP-001231
problem: precise contradiction or missing decision
evidence: [artifact references]
blueprint_refs: [DEC-..., REQ-...]
options:
  - option: ...
    product_impact: ...
    technical_impact: ...
    risk: ...
recommendation: ...
blocks: [STEP-...]
independent_work_available: true
```

Do not smuggle the recommendation into production code before approval.

## 5. Incidental findings

For an unrelated bug, refactor, debt item, flaky test, or enhancement:

- preserve evidence;
- assign severity and affected scope;
- register a Stepper candidate/follow-up item;
- do not widen the current step unless it blocks correctness or creates critical security/data risk;
- identify whether the finding is pre-existing.

## 6. Critical findings

Stop unsafe affected execution for:

- active secret or personal-data exposure;
- exploitable authorization/authentication bypass;
- payment/accounting inconsistency;
- likely data loss/corruption;
- destructive or irreversible ambiguity;
- production-impacting migration risk;
- compromised dependency/build pipeline.

Redact sensitive evidence, create a critical blocker/incident record, and continue only independent safe work.

## 7. Scope control

Apply the smallest-sufficient-change principle. Refactoring is permitted only when required to satisfy the step safely or explicitly represented in Stepper. Record every file outside `expected_files` with a reason. Treat changes to public contracts, schemas, auth, money, privacy, permissions, migrations, or infrastructure as high-impact even when the diff is small.

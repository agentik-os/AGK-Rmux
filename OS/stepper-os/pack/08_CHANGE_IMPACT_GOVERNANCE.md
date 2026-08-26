# Change Impact, ADRs & Governance

## Blueprint changes after Stepper generation

Blueprints evolve. Stepper must detect impact without destroying history.

```text
Blueprint diff
↓
changed requirement/decision/invariant
↓
traceability graph
↓
affected modules/slices/steps
↓
change set
```

## Completed affected step

Do not silently leave DONE.

Possible transition:

```text
DONE → STALE
```

or attach `NEEDS_REVIEW` depending on implementation.

## Change set artifact

```yaml
changeset_id: CHANGE-0007
blueprint_from: 3.0
blueprint_to: 3.1
changed_refs:
  - DEC-080
  - EXP-003
affected_steps:
  stale:
    - STEP-000123
  regenerate_candidates:
    - STEP-000124
new_steps: []
superseded_steps: []
```

## Architecture Decision Request

If implementation discovers a genuine conflict:

```yaml
decision_request_id: DR-0021
step_id: STEP-001231
problem: Current provider cannot support required atomic reservation hold.
blueprint_refs:
  - DEC-080
  - EXP-SYS-001
options:
  - option: Change provider
    impact: medium
  - option: Change product semantics
    impact: high
recommendation: Change provider
blocks:
  - STEP-001231
```

Do not let a coding agent silently choose a product-semantic change.

## Supersession

Approved decision change should be explicit:

```text
DEC-080 superseded by DEC-112
```

Historical trace remains available.

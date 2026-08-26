---
type: north-star
product: OmegaOS
status: Active
updated: 2026-07-30
---

# OmegaOS North Star

## North Star Metric

**Verified Mission Completion Rate:** percentage of accepted missions that reach a valid terminal state with every requested deliverable complete and independently verified, without silent scope loss or operator rescue.

## Input metrics

- Mission contracts created before execution.
- Requested deliverables represented in tracked state.
- File ownership conflicts rejected before mutation.
- Relevant skills selected with validated manifests.
- Runtime checks attached to terminal claims.
- High-risk decisions routed through an independent review gate.

## Output metrics

- Verified completion rate.
- First-pass acceptance rate.
- Resume success after interruption.
- Mean coordination tax per mission.
- Mean doctrine tokens loaded per mission.
- Mean quality-gate retries.
- Escalations caused by true blockers versus orchestration defects.

## Leading indicators

- Rule activation precision and recall.
- Percentage of mission transitions emitted as structured events.
- Percentage of worker briefs with measurable done criteria.
- Percentage of findings that survive reproduce, refute, and cross-check review.

## Lagging indicators

- Repeated incident rate.
- Operator intervention rate.
- Regressions after verified completion.
- Fresh-install parity failures.

## Initial target

The orchestration v3 release is successful when all deterministic tests pass, install verification passes, doctrine context is materially reduced, provider-specific instructions are isolated behind adapters, and three representative mission classes complete through runtime tests: simple inline, parallel delegated, and blocked destructive.

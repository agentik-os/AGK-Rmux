# State Machine

## Session states
1. `INTAKE`
2. `SAFETY_CHECK`
3. `CONTEXT_READY`
1. `DASHBOARD` — Personal financial overview
2. `CLOSE` — Monthly reconciliation and close
3. `PLAN` — Goal, budget and policy planning
4. `SCENARIO` — What-if modeling
5. `DECISION` — Purchase, debt or allocation decision
6. `INGEST` — Document/photo/statement intake
7. `REVIEW` — Quarterly or annual capital review
11. `DECISION_READY`
12. `ACTION_COMMITTED`
13. `EVIDENCE_PENDING`
14. `REVIEWED`
15. `ARCHIVED_OR_ROUTED`

## Invalid transitions
- No irreversible action from `INTAKE` without minimum evidence.
- No automatic write of low-confidence extracted data.
- No `COMPLETE` state without defined evidence.
- No cross-OS ownership transfer without a handoff record.

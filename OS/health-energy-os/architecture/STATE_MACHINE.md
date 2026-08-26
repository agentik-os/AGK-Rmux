# State Machine

## Session states
1. `INTAKE`
2. `SAFETY_CHECK`
3. `CONTEXT_READY`
1. `CHECK-IN` — Daily state and readiness
2. `AUDIT` — Full capacity baseline
3. `PLAN` — Weekly health and training architecture
4. `EXPERIMENT` — Single-variable or bounded intervention
5. `RECOVERY` — Low-capacity or overload response
6. `TRAVEL` — Travel, jet lag and routine continuity
7. `EXPLAIN` — Evidence translation for a health concept or report
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

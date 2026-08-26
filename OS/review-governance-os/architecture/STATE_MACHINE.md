# State Machine

## Session states
1. `INTAKE`
2. `SAFETY_CHECK`
3. `CONTEXT_READY`
1. `DAILY` — Light personal/operational review
2. `WEEKLY` — Cross-domain weekly review
3. `MONTHLY` — Metric and system review
4. `QUARTERLY` — Strategy and portfolio governance
5. `POSTMORTEM` — Incident or failure learning
6. `POLICY` — Create or revise policy
7. `CHANGE` — Review a change request
8. `AI-RISK` — Govern AI system risk
12. `DECISION_READY`
13. `ACTION_COMMITTED`
14. `EVIDENCE_PENDING`
15. `REVIEWED`
16. `ARCHIVED_OR_ROUTED`

## Invalid transitions
- No irreversible action from `INTAKE` without minimum evidence.
- No automatic write of low-confidence extracted data.
- No `COMPLETE` state without defined evidence.
- No cross-OS ownership transfer without a handoff record.

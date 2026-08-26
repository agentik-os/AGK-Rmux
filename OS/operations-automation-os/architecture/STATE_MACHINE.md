# State Machine

## Session states
1. `INTAKE`
2. `SAFETY_CHECK`
3. `CONTEXT_READY`
1. `DIAGNOSE` — Interview and understand the system
2. `MAP` — Create current-state process and value-stream maps
3. `CHALLENGE` — Remove waste and redesign work
4. `SCORE` — Assess automation candidates
5. `DESIGN` — Create future-state and automation blueprint
6. `AGENT` — Assess/build AI-agent operating contract
7. `DEPLOY` — Prepare rollout, controls and runbook
8. `AUDIT` — Review live automations and operational health
9. `INCIDENT` — Recover failed automation
13. `DECISION_READY`
14. `ACTION_COMMITTED`
15. `EVIDENCE_PENDING`
16. `REVIEWED`
17. `ARCHIVED_OR_ROUTED`

## Invalid transitions
- No irreversible action from `INTAKE` without minimum evidence.
- No automatic write of low-confidence extracted data.
- No `COMPLETE` state without defined evidence.
- No cross-OS ownership transfer without a handoff record.

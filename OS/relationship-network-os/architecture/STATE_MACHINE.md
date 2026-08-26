# State Machine

## Session states
1. `INTAKE`
2. `SAFETY_CHECK`
3. `CONTEXT_READY`
1. `BRIEF` — Prepare for a person or meeting
2. `CAPTURE` — Record an interaction and promises
3. `FOLLOW-UP` — Write or plan a follow-up
4. `CONNECT` — Design an introduction
5. `NURTURE` — Maintain important relationships
6. `CONFLICT` — Prepare a boundary or difficult conversation
7. `GATHER` — Design a meaningful gathering
8. `AUDIT` — Review relationship portfolio ethically
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

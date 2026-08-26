# State Machine

## Session states
1. `INTAKE`
2. `SAFETY_CHECK`
3. `CONTEXT_READY`
1. `DIAGNOSE` — Understand the strategic challenge
2. `DESIGN` — Create a strategy kernel
3. `PORTFOLIO` — Rank and allocate across bets/projects
4. `SCENARIO` — Explore futures and contingencies
5. `DECISION` — Make a consequential strategic choice
6. `QUARTER` — Build quarterly strategic commitments
7. `REVIEW` — Continue, pivot, pause or kill
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

# State Machine

## Session states
1. `INTAKE`
2. `SAFETY_CHECK`
3. `CONTEXT_READY`
1. `INTAKE` — Establish contracts and release scope
2. `PLAN` — Create risk-based quality plan
3. `TEST` — Execute product tests
4. `EVAL` — Evaluate AI/agent behavior
5. `AUDIT` — Security/privacy/accessibility/supply-chain audit
6. `CANDIDATE` — Assemble release candidate
7. `RELEASE` — Deploy and verify
8. `INCIDENT` — Contain, rollback and learn
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

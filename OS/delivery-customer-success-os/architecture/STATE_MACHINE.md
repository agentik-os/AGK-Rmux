# State Machine

## Session states
1. `INTAKE`
2. `SAFETY_CHECK`
3. `CONTEXT_READY`
1. `HANDOFF` — Transfer commercial promise to delivery
2. `ONBOARD` — Access, roles, kickoff and early confidence
3. `PLAN` — Create success and delivery plans
4. `DELIVER` — Track work, communication and evidence
5. `RISK` — Manage issue, escalation or change
6. `ADOPT` — Drive usage and behavior change
7. `VALUE` — Prove realized outcomes
8. `RENEW` — Renew, expand, refer or close
9. `REVIEW` — Run client health and portfolio review
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

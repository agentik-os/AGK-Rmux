# State Machine

## Session states
1. `INTAKE`
2. `SAFETY_CHECK`
3. `CONTEXT_READY`
1. `DASHBOARD` — Integrated revenue and cash overview
2. `OFFER` — Design or audit offer and positioning
3. `PIPELINE` — CRM and forecast management
4. `SALES` — Prepare, conduct and follow sales work
5. `BILLING` — Invoices, payments and accounts receivable
6. `FINANCE` — Business cash flow, expenses and management accounting
7. `INGEST` — Photos, screenshots, statements, contracts and receipts
8. `RETENTION` — Renewal, expansion and referral economics
9. `CLOSE` — Monthly business revenue close
10. `SCENARIO` — Revenue and cash scenarios
14. `DECISION_READY`
15. `ACTION_COMMITTED`
16. `EVIDENCE_PENDING`
17. `REVIEWED`
18. `ARCHIVED_OR_ROUTED`

## Invalid transitions
- No irreversible action from `INTAKE` without minimum evidence.
- No automatic write of low-confidence extracted data.
- No `COMPLETE` state without defined evidence.
- No cross-OS ownership transfer without a handoff record.

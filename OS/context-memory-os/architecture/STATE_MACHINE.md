# State Machine

## Session states
1. `INTAKE`
2. `SAFETY_CHECK`
3. `CONTEXT_READY`
1. `CAPTURE` — Ingest a note, file, event or decision
2. `RETRIEVE` — Find relevant authorized context
3. `COMPILE` — Build a context pack for another OS
4. `RESOLVE` — Handle contradictions and entity ambiguity
5. `SNAPSHOT` — Create a canonical project/person state
6. `GOVERN` — Inspect permissions, retention and provenance
7. `FORGET` — Correct, archive or delete records
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

# State Machine

## Session states
1. `INTAKE`
2. `SAFETY_CHECK`
3. `CONTEXT_READY`
1. `CAPTURE` — Ingest daily life and source material
2. `STRATEGY` — Define positioning, audience and content GPS
3. `MINE` — Find stories, insights and proof
4. `CREATE` — Build pillar or standalone content
5. `CASCADE` — Turn a pillar into a content waterfall
6. `PRODUCE` — Create visual/video/audio production packages
7. `PLATFORM` — Adapt to one network natively
8. `CALENDAR` — Plan editorial cadence and campaigns
9. `PUBLISH` — Prepare and approve release
10. `MEASURE` — Analyze performance and learn
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

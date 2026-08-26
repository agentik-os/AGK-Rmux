# Autonomy-Readiness Gate — {{company}}

> Transfer is complete only when this gate passes. Run it adversarially — you are trying to FALSIFY "they're autonomous", not confirm it (R-VERIFY). Never round a PARTIAL up to PASS to close the engagement.
> Part 1 (adoption) must pass before Part 2 (transfer): you don't certify a team to extend a system the team isn't even using.

- **Date:** {{date}} · **Run by:** {{CAIO}} · **Target:** {{date}}

---

## Part 1 — Adoption gate (precondition)
| Item | Target | Result | PASS/PARTIAL/FAIL |
|---|---|---|---|
| Adoption NSM (value-received, not vanity) per component | > {{target}} / active operator | {{...}} | {{...}} |
| Retention curve by cohort | not collapsing; plateau > target | {{...}} | {{...}} |
| Validated use cases (real, accepted, unaided, evidenced) | ≥ {{N, default 5}} | {{count}} | {{...}} |
| Skeptics addressed (converted OR re-onboard scheduled) | none silently dropped | {{...}} | {{...}} |

---

## Part 2 — Transfer gate (the three motions, UNAIDED, real conditions, CAIO hands-off)
> Evidence required for each (R-CITE — no evidence, no pass). If the owner had to ask the CAIO HOW → the motion does NOT pass; fix the DOCS first, then re-run.

| Motion | Named owner | What they did (real instance) | Evidence | PASS/FAIL |
|---|---|---|---|---|
| **ADD AN AGENT** (HITL wired, staging, acceptance pass) | {{name}} | {{...}} | {{PR/commit/recording}} | {{...}} |
| **CONNECT A TOOL** (read-only first, permissioned, logged) | {{name}} | {{...}} | {{commit/log}} | {{...}} |
| **ADJUST A REPORT** (verified vs runtime, not the label — L1) | {{name}} | {{...}} | {{before/after}} | {{...}} |

---

## Part 3 — Ownership conditions (no bus factor of one)
| Item | Result | PASS/FAIL |
|---|---|---|
| Named owner per component; backup on critical (bus factor ≥ 2) | {{...}} | {{...}} |
| ZERO CAIO-only credentials (rotated to client vault) | {{count remaining}} | {{...}} |
| Escalation path documented | {{...}} | {{...}} |
| Evolution process run ≥ once by the team | {{...}} | {{...}} |
| Weekly guardian routine run ≥ twice without the CAIO | {{dates}} | {{...}} |
| HITL approver = client employee on every sensitive decision | {{...}} | {{...}} |

---

## Verdict
```
[ ] TRANSFER COMPLETE   — Part 1 all PASS + all three motions PASS + all ownership PASS → caio-run-and-optimize
[ ] PARTIAL / DATED     — most pass; named gap with owner + date: {{gap → owner → due}}
[ ] TRANSFER BLOCKED    — a motion FAILed OR a CAIO-only credential remains OR a component has no owner
                          → fix the specific gap (usually re-document + re-teach), then re-run
[ ] ADOPTION NOT READY  — Part 1 failed (curve collapsing OR < N use cases) → back to adoption phase
```

**Verdict:** {{...}}
**Outstanding (owner + date):** {{...}}
**Next:** {{re-run date / hand to caio-run-and-optimize}}

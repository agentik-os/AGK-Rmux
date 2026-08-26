# 05 — Adoption Measurement & the Autonomy-Readiness Gate

Phases 3 & 6 of the skill. Outputs `caio-enablement/08-Adoption-Tracker.md` and `07-Autonomy-Readiness-Gate.md`.

Two jobs: (A) measure adoption HONESTLY so "they use it" is a fact, not a hope (mm-11), and (B) define the objective, falsifiable gate that decides whether transfer is complete. This is where the offer's two promises get receipts.

> If you can't show the retention curve, you don't know if it was adopted. If you can't point at an unaided extension under real conditions, you don't know if it was transferred. Everything else is a feeling.

---

## A. Measuring adoption (mm-11, applied internally)

mm-11's entire apparatus — North Star Metric, retention curve, cohorts, the leaky bucket — transfers one-to-one to internal adoption. The "customers" are your operators; "churn" is reverting to the old way.

### A.1 The adoption North Star Metric (value received, not vanity)
The adoption NSM must capture the **value the system delivers to the operator**, not activity. mm-11's discriminating test, internalized: *if this metric doubles, is the team objectively getting more of their real work done through the system?*

| Vanity (REFUSED as adoption proof) | Real adoption NSM (per system) |
|---|---|
| Logins / sign-ins | Workflows completed through the system per active operator per week |
| Seats provisioned | Agent outputs accepted by the human (drafts approved, briefs shipped) |
| Training sessions attended | Real tasks done via the system that used to be done by hand |
| Dashboard page-views | Decisions made off the dashboard's numbers |

Pick ONE NSM per system component (an agent's NSM differs from a dashboard's), exactly as mm-11 insists each product gets its own NSM. The portfolio-level read is "is the whole system being used"; the component-level read is each component's NSM. A metric that can rise while operators are frustrated (e.g. "agent runs" climbing because reps keep retrying a bad draft) is a trap, not an NSM.

### A.2 The adoption retention curve (the real proof)
Do NOT measure adoption as an average ("80% adoption"). An average hides everything. Measure it as mm-11 does — a curve, by cohort:
```
x-axis: weeks since the operator's onboarding (W1, W2, W4, W8)
y-axis: % of that cohort still using the system for real work
```
The SHAPE is the diagnosis (mm-11):
- **Collapses toward zero** -> the system doesn't fit the real work, or never reached the aha. More training won't fix it. Re-diagnose fit (ref 01 §G); possibly back to implementation/architect. Do NOT declare adoption.
- **Flattens on a plateau** -> a core of operators for whom the system is now part of the job. The height of the plateau is your real adoption. **Plateau > target = adoption proven.**

### A.3 Cohorts (is onboarding improving?)
Group operators by onboarding week and compare curves. If the cohort onboarded last week retains better than the one from a month ago, your onboarding is improving. If later cohorts retain worse while you keep onboarding more people, you're accelerating toward a wall (mm-11) — fix onboarding before adding audiences.

### A.4 The aha-moment + the leaky-bucket rule
Most internal abandonment happens before the first unaided win (mm-11 — onboarding is chantier n°1). The Adoption-Tracker therefore instruments time-to-first-unaided-run per operator, and flags anyone who hasn't reached it. And the hierarchy holds: **retention before expansion** — stabilize the first cohort's curve before rolling to the next department. Expanding on a leaky bucket multiplies the leak.

### A.5 The minimal measurement stack (don't drown)
mm-11's warning applies: the risk isn't under-instrumenting, it's a hundred dashboards nobody reads. The minimum:
- Three events per component: the operator's **action done**, the **acceptance** (human approved/used it), the **NSM** event.
- Cohort retention (most product-analytics tools compute it natively once the events exist).
- One weekly view: NSM, last cohort's retention, lapsing-operator flags. If the team looks at one screen, it's that one — and it's the same screen the weekly guardian routine reads (ref 03 §E) and that `caio-run-and-optimize` inherits.

The Adoption-Tracker template (`assets/templates/Adoption-Tracker.md`) is the human-readable version; the events above are the machine version. The tracker is handed to run-and-optimize as the usage baseline.

---

## B. The Autonomy-Readiness Gate (the objective transfer test)

Transfer is NOT complete on a feeling, an applause, or a quiz. It is complete when the gate passes. The gate has two parts; Part 1 (adoption) must pass before Part 2 (transfer) — you don't certify a team to extend a system the team isn't even using (mm-11 — don't expand before the bucket holds).

### B.1 Part 1 — Adoption gate (precondition)
```
[ ] Adoption NSM defined per component, value-received (not vanity), and > target per active operator.
[ ] Retention curve by cohort is NOT collapsing (week-1 cohort still using weeks later; plateau > target).
[ ] 04-Validated-Use-Cases-Log.md has >= N real, accepted, evidenced, UNAIDED runs (default N=5; scaled).
[ ] Every named skeptic either converted to a user OR has a re-onboard scheduled (none silently dropped).
```

### B.2 Part 2 — Transfer gate (the three motions, UNAIDED, real conditions)
Each motion performed by a NAMED client owner, on a REAL instance, with the CAIO observing and **hands off the keyboard**. Evidence captured for each (R-CITE — no evidence, no pass).
```
[ ] ADD AN AGENT   :: owner ____ added/extended an agent, wired HITL, shipped to staging, passed its
                      acceptance check. Evidence: ____ (PR / commit / recording).
[ ] CONNECT A TOOL :: owner ____ connected a real new integration (read-only first), permissioned,
                      appears in the logs. Evidence: ____.
[ ] ADJUST A REPORT:: owner ____ changed a report/metric and verified it against runtime (not the
                      label — L1). Evidence: ____ (before/after).
```

### B.3 Ownership conditions (no bus factor of one)
```
[ ] Named owner per component; backup owner on anything critical (bus factor >= 2).
[ ] ZERO CAIO-only credentials — all keys rotated to the client's vault; CAIO access advisory only.
[ ] Escalation path documented (in-house vs. agentic-systems-builder vs. agentik-skill-forge).
[ ] The evolution process run at least once by the team.
[ ] The weekly guardian routine run >= twice WITHOUT the CAIO present.
[ ] HITL approver on every sensitive decision is a CLIENT employee (Iron Law 7).
```

### B.4 The load-bearing gate rule
If a client owner had to ask the CAIO **how** during any motion, that motion does **NOT** pass — and the corrective action is, first, to fix the **documentation** (the docs failed the legibility test, ref 02 §A), and only then to re-coach. This rule is what makes the gate a test of the *system's transferability*, not just the person's memory. Re-document, then re-run the motion.

---

## C. Gate scoring rubric & verdict

Score each item PASS / PARTIAL / FAIL. The verdicts:

```
TRANSFER COMPLETE       :: Part 1 all PASS, Part 2 all three motions PASS, all ownership conditions PASS.
                           -> hand to caio-run-and-optimize.
TRANSFER BLOCKED        :: any Part 2 motion FAILs, OR any CAIO-only credential remains, OR no named
                           owner on a component. -> fix the specific gap (usually re-document + re-teach),
                           then re-run. Do NOT declare done (L4 — done means 100%, verified).
ADOPTION NOT READY      :: Part 1 fails (curve collapsing OR < N validated use cases). -> back to the
                           adoption phase; do not attempt transfer on an unadopted system.
PARTIAL / DATED         :: most pass, a named gap with a date + owner (e.g. "1 skeptic re-onboard by Fri").
                           -> a legitimate interim state; the engagement isn't closed until it clears.
```

Never round a PARTIAL up to a PASS to close the engagement. A gate that certifies autonomy the team doesn't have is the exact failure (the consultant leaves, the system rots, the dependency was never actually broken) the whole offer exists to prevent. The gate is adversarial on purpose: you are trying to *falsify* "they're autonomous", not confirm it (R-VERIFY).

---

## D. Worked excerpt (the gate as run)

```
AUTONOMY-READINESS GATE — Acme Corp — 2026-07-18

PART 1 ADOPTION:
- NSM (triage agent): "tickets triaged & accepted/rep/wk". W1 0 -> W4 214/wk (62%).   PASS (>50%)
- Retention: wk1 cohort (11 reps) still active wk4 = 9/11. Curve flattening, not collapsing. PASS (>7)
- Validated use cases: 6 real, accepted, unaided, evidenced.                            PASS (>=5)
- Skeptics: 2/3 converted; 1 re-onboard scheduled Fri.                                   PARTIAL

PART 2 TRANSFER (unaided, real, CAIO hands-off):
1. ADD AGENT: J. cloned triage->refund-triage, HITL wired, staging, acceptance pass. PR#142+rec.  PASS
2. CONNECT TOOL: J. added read-only Zendesk-macro source via Composio, logged. commit+log.         PASS
3. ADJUST REPORT: M. changed "resolved" threshold + fixed a mislabeled metric via config UI,
   verified vs runtime. before/after screenshots.                                                  PASS

OWNERSHIP:
- Owners: brief=COO+M, triage=M+J, dashboard=J; backups set on triage+dashboard.     PASS
- CAIO-only credentials: 0 (rotated to client 1Password).                            PASS
- Weekly routine: run twice w/o CAIO.                                                PASS
- HITL approvers all client employees.                                               PASS

VERDICT: PARTIAL / DATED — transfer motions + ownership all PASS; adoption blocked only on 1 skeptic
re-onboard (owner: M., due Fri). Clears to TRANSFER COMPLETE on re-onboard; then -> caio-run-and-optimize.
```

---

## E. Discipline checks for this phase

| Check | Pass = |
|---|---|
| Adoption NSM is value-received per component; vanity metrics excluded | yes |
| Retention measured as a curve by cohort, not an average; shape diagnosed | yes |
| Lapsing operators flagged (time-to-first-unaided-run instrumented) | yes |
| Part 1 (adoption) passes before Part 2 (transfer) is attempted | yes |
| All three transfer motions demonstrated UNAIDED on real instances, evidence captured | yes |
| The "had to ask how -> fix the docs" rule applied (legibility, not just coaching) | yes |
| Zero CAIO-only credentials; named owners (+ backups on critical) | yes |
| HITL approvers are client employees on every sensitive decision | yes |
| No PARTIAL rounded up to PASS; the gate run adversarially | yes |
| Adoption-Tracker + Validated-Use-Cases log handed to caio-run-and-optimize | yes |

The gate is the receipt for the offer's whole promise: not "we trained them" but "they own it and can extend it without us, and here's the proof".

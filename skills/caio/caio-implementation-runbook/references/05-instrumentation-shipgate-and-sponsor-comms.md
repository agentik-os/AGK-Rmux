# Reference 05 — Instrumentation (5.8 + mm-11) · Ship-Gate · Sponsor Communication (mm-04)

> The build's three cross-cutting disciplines. **§A** — instrument the baseline (mm-11): the North-Star + cost/usage events wired at build time so ROI is measurable later. **§B** — monitoring/observability (5.8): operational transparency by default. **§C** — the ship-gate: value in week 1, acceptance pulled from the architect's specs. **§D** — sponsor communication (mm-04): keep the sponsor bought-in through the longest, most vulnerable phase.

---

# §A — Instrument the baseline (the mm-11 slice)

> *(Grounded by mm-11 — measure-loops-retention, the **instrument-for-baseline** slice ONLY. This skill does **not** measure ROI; `caio-run-and-optimize` does. But measurement is impossible later if the substrate is not laid now.)*

## A.0 Why instrument at build time

mm-11's core arithmetic lesson is that a system either **compounds** or **leaks**, and you cannot tell which without measurement. The CAIO offer makes a ROI projection (the architect's `09-ROI-Governance-And-Risks.md`). That projection is a **hypothesis**. To later confirm or falsify it (mm-11's Popperian discipline — *a test must be able to prove you wrong*), the measurement substrate must exist **from t0 (go-live)**. Bolt it on three months later and you have no baseline — you can never compute the delta. So you build the instrument now and hand the *measuring* to Phase 5.

> You build the **instrument**, not the **verdict**. The verdict ("did ROI hold?") belongs to `caio-run-and-optimize`.

## A.1 Three clean events per dashboard (mm-11 — "three events beat a hundred")

mm-11 is explicit: *trois events bien posés valent mieux que cent mal nommés* — three well-placed events beat a hundred badly named. Per dashboard, you instrument **exactly three**:

| Event | What it is | Example |
|---|---|---|
| **North-Star (`nsm`)** | the architect's success metric for that opportunity — the **value the dashboard delivers** | "executive brief shipped on time"; "support ticket auto-triaged + accepted" |
| **Cost/usage (`usage`)** | model cost + tokens + agent run per execution | `{model, tokensIn, tokensOut, costUsd, runId}` |
| **Value-delivered (`value`)** | the workflow outcome that proves real work happened | "12h→30min report cycle"; "renewal-risk flagged before churn" |

Written to `baselineEvents{seatKey, eventType, value, ts, t0}` (reference 02 §4). That is the whole baseline. Resist instrumenting fifty events — you will drown in dashboards you never read (mm-11's explicit warning: *le risque n'est pas de sous-instrumenter, c'est de te noyer dans des dashboards que tu ne regardes jamais*).

## A.2 The no-vanity-metric gate (mm-11 — the North-Star test)

mm-11's discriminating test: *if this metric doubled, would the client be objectively better off?* Apply it to every North-Star event you wire:

- **"Dashboard opened" — REJECTED.** It can double while nothing improves (mm-11: a metric that climbs while users are frustrated is a trap). It is a vanity metric.
- **"Executive brief shipped on time + read" — ACCEPTED.** If it doubles, the org is demonstrably more informed and faster.
- **"Agent runs" — REJECTED as an NSM** (it is a *cost* event, not value). High agent-run counts with low accepted-outcomes is a leaky bucket.
- **"Outcomes accepted by the human" — ACCEPTED.** Real value delivered.

> The North-Star event must *precede revenue and represent value received* (mm-11). The dashboard's job is to deliver a real outcome; instrument that outcome, never the vanity proxy.

## A.3 t0 — the baseline anchor

- **t0 = each dashboard's go-live timestamp.** Record it in `baselineEvents.t0` and in `metadata.json`.
- Where possible, capture the **pre-build manual baseline** too (e.g. "report cycle was 12h/week before go-live" — from the discovery rollup / architect's ROI math) so Phase 5 compares against the *real* prior state, not zero.
- The handoff to `caio-run-and-optimize` is: "here is t0, here are the three events per dashboard, here is the architect's projection in `09-` — compute the delta." You set the experiment up; they read it.

## A.4 What you do NOT do (the boundary)

You do **not** compute ROI, draw the retention curve, run the cohort analysis, pick growth loops, or declare PMF. Those are mm-11's *full* doctrine and they belong to `caio-run-and-optimize` (Phase 5). Here you wire the **substrate** only. Crossing into measurement is scope creep — and you would be measuring a system with one day of data, which is meaningless.

---

# §B — Monitoring / observability (5.8)

> The offer's 5.8: operational transparency built in **by default** — which agents execute, model costs, real usage, reports consulted. This is the architect's Iron Law 8 ("the dashboard must expose sources, logs, status, errors, costs, confidence") made into a running surface.

## B.1 The monitoring view (built in STEP 6 of provisioning, before features)

A dedicated observability surface (CIO/CTO-owned, CAIO-visible) reading the `agentRuns`, `costEvents`, `reports`, and `integrations` tables:

| Panel | Source | Answers |
|---|---|---|
| **Agent activity** | `agentRuns` | which agents executed, when, status, errors, confidence |
| **Model cost** | `costEvents` | $ + tokens by agent, by seat, by day; trend vs budget |
| **Real usage** | `baselineEvents(usage)` | what's actually being used vs built-but-idle |
| **Reports consulted** | `reports` + read receipts | which reports shipped + were opened (a built-but-unread report is a leak) |
| **Integration health** | `integrations` | last live read, rateRemaining, auth status per connector |
| **Errors / failures** | `agentRuns.error`, `alerts` | what's broken right now, owned not hidden |

## B.2 Observability rules

- **Built before features (Iron Law 6).** The skeleton stands in provisioning STEP 6 so every feature is instrumented by construction.
- **Cost is visible to the CFO.** Model spend is a line item, not a surprise. The CIO/CTO and CFO dashboards both surface `costEvents` (model cost per workflow) — the offer's promise of "model costs, real usage" transparency.
- **Errors are owned, not hidden.** A failed agent run shows in the monitoring view and (if it feeds a report/alert) marks the downstream panel "delayed" — never a stale number shown as fresh (L1).
- **Confidence travels.** Every LLM-derived value carries its confidence from the agent through to the panel (reference 03 §A.2).

## B.3 Monitoring acceptance

| Check | Pass = |
|---|---|
| Monitoring view live before any micro-SaaS feature (STEP 6) | yes |
| Agent runs, model cost, usage, reports, integration health all visible | yes |
| Model cost surfaced to CFO + CIO/CTO (no hidden spend) | yes |
| Failed runs owned + surfaced; no stale-as-fresh numbers | yes |
| The 3 baseline events per dashboard firing since t0 (mm-11) | yes |

---

# §C — The ship-gate (value in week 1)

> The offer's promise: each micro-SaaS goes live **only when its acceptance test passes** — value in week 1, not a POC/demo.

## C.0 The acceptance criteria are the architect's, not yours

You do **not** invent acceptance criteria. They are pulled **verbatim** from `company-ai-os/07-Dashboard-Feature-Specs.md` (the architect's 12-field specs end in an `acceptance` field). If a feature has no acceptance criterion, you cannot ship-gate it — route back to the architect. This keeps the gate honest: the bar was set *before* the builder had an incentive to clear it (R-RUBRIC).

## C.1 The gate, per deliverable

```
SHIP-GATE (per micro-SaaS / report / federation rule)
1. Criteria — copy the acceptance criterion verbatim from 07-Dashboard-Feature-Specs.md.
2. Real data — run against the client's REAL connected data, not seeded/mock (L1).
3. Acceptance gate — for the dashboard apps, run /omg-acceptance:
     - every route 200 + renders
     - every console error owned (app-bundle/backend; third-party noise ignored)
     - the authenticated golden path walked with a REAL persisted write
4. Anti-black-box — every asserted number shows source + freshness + confidence (Iron Law 6).
5. Baseline — the 3 mm-11 events fire (reference 05 §A).
6. HITL — every sensitive action renders an approval step, not auto-execute (Iron Law 9).
7. Verdict — GREEN: ship now (value in week 1). RED: back to the builder; do NOT ship "mostly working."
8. Record — 08-Ship-Gate-Ledger.md: criteria, run date, verdict, evidence (log/screenshot — R-CITE).
```

## C.2 "It builds" is never "it works" (L1)

A green compile with a red console is not shipped (R-PROD). The ship-gate runs the **real golden path on real data** and observes the **browser console + network**. A demo that "looks done" but has a 500 on the CFO's actual data is RED. Runtime is the only truth.

## C.3 Adversarial verification (R-VERIFY)

A builder's own "done" is an input, never the verdict. Before a deliverable is marked shipped, three skeptic lenses (2-of-3 consensus) try to falsify it:
- **Runtime skeptic** — does acceptance pass on *real* data, console clean?
- **Anti-black-box skeptic** — sources/logs/status/errors/costs/confidence all exposed?
- **Baseline skeptic** — are the three mm-11 events firing at t0?

Ships only on 2-of-3. (For a `full-build`, this is the fan-out's verify stage — SKILL.md "Dynamic Workflow orchestration".)

---

# §D — Sponsor communication (the mm-04 slice)

> *(Grounded by mm-04 — messaging/copy/offer, the **build-milestone communication** slice. The build is the longest and most vulnerable phase; executive sponsorship and budget confidence decay in silence. An un-communicated build loses the sponsor — and a lost sponsor kills a working system.)*

## D.0 The mechanism (mm-04 — the copy canalizes existing desire)

mm-04's foundational principle (Schwartz): *copy does not create desire — it channels the desire that already exists.* The sponsor **already wants** the legible, automated company (that is why they signed). Your milestone communication does not manufacture enthusiasm — it **channels the sponsor's existing desire onto the evidence that it is happening.** You do not "sell" the build; you make the progress *legible* so the existing belief stays alive.

## D.1 The value equation, applied to the sponsor (mm-04 — Hormozi)

mm-04 carries Hormozi's value equation: **Value = (Dream outcome × Perceived likelihood) ÷ (Time delay × Effort)**. The sponsor is the "buyer" of *continued budget and air-cover*. Maximize their perceived value of staying bought-in:

| Lever | Move |
|---|---|
| **Dream outcome ↑** | restate it: "the company runs itself, legibly" — at every milestone, tie the demo to the dream, not the feature. |
| **Perceived likelihood ↑** | show a **live demo on real data** (runtime, never slideware — Iron Law 8). A working CFO dashboard on *their* numbers beats any deck. |
| **Time delay ↓** | the ship-gate delivers the **first quick victory in week 1** — a real dashboard live, the offer's "value in week 1." mm-04's "première victoire rapide." |
| **Effort & sacrifice ↓** | the build runs without the sponsor chasing it; the progress brief **comes to them**, one page, decision pre-framed. |

## D.2 Clear beats clever (mm-04 — Ogilvy): the progress brief

mm-04 (Ogilvy): *clear beats clever; if it doesn't sell, it isn't creative.* The progress brief is **one page**, jargon-free:

```
PROGRESS BRIEF — [week N]
SHIPPED:   [the specific deliverable that went live + its evidence link] (e.g. "CFO cash dashboard live; first weekly brief shipped Mon 7am on real Stripe data")
NEXT:      [the next deliverable + its ship date]
BLOCKED:   [the one blocker, if any, owned + the unblock ask]
DECISION:  [the ONE decision needed from you, if any — pre-framed]
```

No "we're tracking well." A **specific shipped outcome with its evidence** (R-CITE). The brief that says "the CFO dashboard is live and the Monday brief shipped automatically on real data" keeps the sponsor; the brief that says "good progress this sprint" loses them.

## D.3 The realization gate as a value-prop the sponsor signs (mm-04 — Dunford)

mm-04 carries Dunford's positioning one-liner. The Architecture-Realization approval (reference 01 §7) is framed as a value proposition, not a spec dump:

> "For [your C-suite] who [need the company legible *and* automated], this is the [centralized federated Company-AI-OS] that [turns seven blind tools into one organism that alerts itself], unlike [disconnected dashboards] that [keep each C-Level guessing in their own silo]."

Lead with the value-prop + a one-page topology diagram; attach the technical detail. The sponsor approves what they understand (mm-04 — the 5-second test).

## D.4 The go-live announcement (mm-04 — BAB)

mm-04's Before/After/Bridge framework structures the go-live announcement:
- **Before:** "Every Monday, 12 person-hours assembling the executive report by hand."
- **After:** "The brief falls out automatically at 7am, with every number sourced."
- **Bridge:** "The CFO dashboard and the report engine you just approved."

Concrete, their words, evidence attached. Not "we have launched our AI transformation."

## D.5 The honesty gate (mm-04 — Schwartz/Cialdini + L1)

mm-04's hard rule: *no fabricated scarcity, no manufactured social proof, no invented stat — trust is the only compounding asset.* In a build that means:
- **No fabricated progress.** A brief reports what *actually shipped*, verified by the ship-gate. "Done" claims trace to a green acceptance run (L1).
- **No fake-green demo.** A milestone demo shows the acceptance gate actually green on real data — never a hard-coded screen, never seeded data dressed as live (Iron Law 8).
- **No manufactured urgency** to extract more budget. The value is real or it is not; manufacture nothing.

A single faked demo ends the engagement (mm-04: *une fausse urgence est un suicide de marque* — a fabrication is brand suicide). The whole point of communicating the build is to make *real* progress visible — which only works if it is real.

## D.6 Cadence + the plan

Captured in `09-Sponsor-Communication-Plan.md` (template: `assets/templates/Sponsor-Communication-Plan.md`):
- **Realization gate** — one approval meeting, value-prop framed (D.3).
- **Milestone demos** — at each micro-SaaS ship (week 1 first), live on real data (D.1).
- **Progress brief** — weekly, one page (D.2).
- **Go-live announcement** — BAB, to the broader exec team (D.4).
- **Recipients + channel + cadence** recorded; every brief logged in `00-Build-Log.md` so the communication is itself auditable (and a missed brief is visible).

## D.7 Sponsor-communication acceptance

| Check | Pass = |
|---|---|
| Realization gate framed as a value-prop + sponsor-approved with a date (mm-04, Iron Law 1) | yes |
| Every milestone demo ran live on real data (no slideware) — logged | yes |
| Weekly one-page progress brief delivered; specific shipped outcomes cited | yes |
| No fabricated progress; every "done" traces to a green ship-gate (L1) | yes |
| Go-live announcement uses BAB with real before/after numbers | yes |
| Every brief/demo logged in the build log (auditable communication) | yes |

The sponsor staying bought-in through go-live is item 7 of the SKILL.md Iron Test. A perfect system with a lost sponsor does not get renewed — mm-04 is not a nicety here, it is what protects the build.

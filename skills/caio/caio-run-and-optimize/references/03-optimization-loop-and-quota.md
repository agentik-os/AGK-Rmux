# 03 — The Optimization Loop + the 1h/Week Quota

Phases 4-5 of CAIO Run & Optimize. Outputs `caio-run/Optimization-Loop-Cadence.md`, `caio-run/Weekly-Quota-Agenda.md`, and the `optimization_backlog` + `quota` blocks of `metadata.json`.

Two doctrines drive this reference. **mm-11** (the system that compounds) governs Part A — the cadence that turns yesterday's measured work into a sharper, cheaper choice this month, instead of letting the system drift into stale dashboards. **mm-08** (pricing = positioning) governs Part B — why the strategic quota is deliberately one hour, what that hour is for, and where the boundary sits before a request becomes a scoped engagement.

---

# Part A — The Optimization Loop (mm-11)

> Each tour rend le suivant moins cher et plus gros (mm-11). A delivered system either compounds — every month's measurement makes the next month's choice sharper — or it drifts. The loop is what keeps it compounding.

## A.1 The flywheel, applied to a delivered system

mm-11's four-piece flywheel maps directly onto the run:

```
system NSM (the progress definition)
   → cohort savings-retention (each shipped workflow keeps delivering)
      → optimization picks the next-highest-value improvement
         → a falsifiable hypothesis improves the next cohort
            → (back to a higher NSM)
```

The loop assembles these in order. The discipline is the *order*: NSM and cohort retention are the foundation (you do not optimize what you have not measured); the next-best improvement and its hypothesis are the engine.

## A.2 The weekly read (15 minutes, one screen — no new projects)

Open the operator one-screen (reference 02 §C.1). Read five things, triage only:
1. **NSM this week + trend** — up, flat, down?
2. **Last cohort's savings-retention** — is the newest wave holding, or starting to decay?
3. **Cost vs budget** — any margin drift?
4. **Top adoption mover** — what rose or fell most?
5. **The single most important open alert** — does it need a human this week?

The weekly read does NOT start work. It decides whether anything is on fire and what the monthly loop should look at. Discipline: a weekly that turns into a project meeting destroys the lightness the whole engagement is positioned on (see Part B).

## A.3 The monthly re-score (the compounding step)

This is where the system gets better instead of older. You **re-score the architect's `05-Automation-Opportunity-Backlog.md` against ACTUAL data** — not the projection-time estimates.

Why re-score: the architect scored opportunities on *estimated* impact, frequency, and adoption. You now have *measured* numbers. The #6 opportunity (modest estimated impact) may be #1 once you've seen its real frequency is 3× the estimate and the team loves it. The #1 may be dead (falsified, reference 01 §D). The backlog is a live document now, re-ranked on telemetry.

Steps:
1. **Pull the open backlog** (un-shipped opportunities) + any falsified/leaking shipped workflows.
2. **Re-score each on actual-informed ICE or RICE:**
   - **ICE** = Impact × Confidence × Ease (1-10 each) — fast, fine for a solo CAIO.
   - **RICE** = (Reach × Impact × Confidence) ÷ Effort — when reach varies a lot across candidates.
   - The discipline is not the formula (the scores are subjective) — it is **forcing the explicit reason** one candidate beats another, and logging it (mm-11).
3. **Apply the retention-first weight (Iron Law 5).** While ANY cohort is leaking, a retention/adoption fix outranks a new build of equal raw score. You do not add a workflow to a company that is abandoning the last one. This is mm-11's `retention → monetization → acquisition` made operational.
4. **Pick ONE next-highest-value improvement** and write it as a falsifiable hypothesis.

## A.4 The hypothesis format (mm-11 — a test must be able to give you wrong)

> *Because [telemetry observation], I believe [change] will move [the NSM / a specific cohort's savings-retention] from [current] to [target]. I'll know within [window] if [threshold].*

Worked example:
> *Because the support-triage agent's WAU dropped 35% in weeks 6-8 while its error rate stayed flat, I believe the team stopped trusting it after the March prompt change — not that it broke. I believe reverting the prompt + a 30-min re-onboarding will restore WAU from 41% to >70% and pull wave-2's savings-retention off its decay. I'll know within 3 weeks if WAU clears 70% and the cohort saving stops falling.*

Now the action *teaches you something whatever the result*: if WAU recovers, trust was the issue; if it doesn't, the workflow may be genuinely obsolete (a kill candidate). A change with no hypothesis is a slot machine — you pull the lever, a number moves, you learn nothing.

## A.5 The three verdicts the monthly loop emits

Every monthly loop ends in exactly one of these for the chosen improvement:

| Verdict | Meaning | Routes to |
|---|---|---|
| **Tweak** | optimize an existing workflow (prompt, threshold, handoff, re-onboard) | you or the client's system owner; tracked in the loop doc |
| **Build** | a genuinely new workflow worth shipping | `agentic-systems-builder` / `agentik-skill-forge` with an F-XXX spec (this skill does NOT build) |
| **Expand** | the system is healthy and saturating its current scope | `caio-enterprise-workflow-architect` for the next-wave audit — **the chain loop closes** |

The **Expand** verdict is the chain's compounding moment: a healthy, measured, retention-positive system is the precondition to re-enter the architect for the next department. Reaching it means the run worked. Note the gate: you only emit **Expand** when no cohort is leaking and the realization rate is healthy — otherwise the verdict is **Tweak** (fix the leak first).

## A.6 Worked monthly loop (the re-score in action)

Continuing the worked client from reference 01 (F002 proven, F001 falsified/leaking):

```
Open candidates re-scored on ACTUAL data (not projection-time estimates):
| Candidate                          | Type             | Projected rank | Actual-informed ICE | Note
| Restore F001 support-triage trust  | Tweak/retention  | (not ranked)   | I9·C8·E7 = 8.0      | wave-1 LEAKING
| Knowledge-base RAG search (was #4)  | Build            | #4             | I6·C6·E4 = 5.3      | new build
| Sales-followup sequencer (was #3)   | Build            | #3             | I7·C7·E6 = 6.7      | new build

Retention-first weight (Iron Law 5): F001 is a leaking cohort → its retention fix outranks
both new builds this month, even though "sales-followup" had a higher projected rank.

Chosen improvement (ONE): restore F001 trust.
Hypothesis: "Because F001's WAU fell 35% in weeks 6-8 while error rate stayed flat, I believe
the team distrusted the agent after the March prompt change — not that it broke. Reverting the
prompt + a 30-min re-onboarding will restore WAU from 41% to >70% and pull wave-2's
savings-retention off its decay. I'll know within 3 weeks if WAU clears 70%."
Verdict: TWEAK (owner: CAIO + system owner). Sales-followup and RAG stay parked until F001's
cohort stops leaking — THEN the loop can emit a Build, and only after that an Expand.
```

The lesson: the projection-time ranking said "ship sales-followup next". The *measured* data said "you have a leaking workflow; fixing it beats any new build". That inversion is the entire value of re-scoring on actuals — it stops the company pouring water into a holed bucket (mm-11).

## A.7 The loop document (`Optimization-Loop-Cadence.md`)

```
# Optimization Loop — [Client] — [Quarter]

## Cadence
- Weekly read: [day/time], owner: [CAIO], one-screen only
- Monthly re-score: [date], inputs: actual ROI + cohort table + open backlog

## This month's re-scored backlog (top 5)
| Rank | Item | Type | Actual-informed ICE | Retention-first? | Note |
|---|---|---|---|---|---|
| 1 | Restore support-triage trust | Tweak/retention | 8.4 | YES (wave-2 leaking) | beats new builds this month |
| 2 | ... |

## This month's hypothesis (the ONE improvement)
[the Because… I believe… I'll know if… statement]

## Verdict: Tweak / Build / Expand
[+ routing]

## Last month's hypothesis — result
[confirmed / refuted + what it taught]
```

---

# Part B — The 1h/Week Strategic Quota (mm-08)

> Le prix (et le format) EST le positionnement (mm-08). The quota is one hour by design. A heavy retainer would contradict the autonomy the chain delivered. The lightness is the product, not a discount.

## B.1 Why one hour and not ten (the positioning argument)

The whole accompaniment chain promised: make the company legible → build the system → enable the team → **make the team autonomous**. A 40-hour-a-month retainer at this stage would *say*, in mm-08's terms, "your system isn't really autonomous; you still need me operationally." That contradicts everything the chain delivered and quietly tells the C-Level the autonomy was a fiction.

The 1h/week is a **category statement**: the system runs itself; the team owns it; the hour buys *strategy*, not *babysitting*. It is the safety net **without** the dependency. Pricing here is not about extracting more hours — a price/format that contradicts the positioning is a self-inflicted wound (mm-08).

## B.2 What the hour IS for

Three uses, all strategic, none operational:
1. **Strategic questions** — "should we expand the support agent into billing?", "is this new regulation a governance change?", "what's the next department to make legible?"
2. **Technical arbitrations** — "build-vs-buy this new integration?", "is this vendor change worth the migration?", "model A vs model B for this workflow?"
3. **New-integration scoping** — sizing a new connection or workflow enough to decide *whether* it becomes a scoped engagement.

## B.3 What the hour is NOT for (and the overage boundary — Iron Law 8)

The hour is **not** for: operational firefighting, running reports by hand, debugging that belongs to the system owner, or — critically — *delivering a new build inside the retainer*.

mm-08's margin discipline applies to services exactly as it does to AI tokens: **unbounded usage is a margin bomb.** In SaaS, one power-user on unmetered usage turns a profitable account into a loss. In consulting, one client treating the 1h/week as "and also build me this" turns a clean retainer into unpaid full-time work.

The boundary:
- Anything that **fits in the hour** (a question, an arbitration, a scoping conversation) → inside the quota.
- Anything **bigger** (a new workflow, a migration, a department expansion, a multi-week build) → a **scoped mini-engagement** with its own SOW via `/market-proposal`. Not absorbed. Not "I'll just do it this once."
- The overage is **priced and capped**, the way mm-08 caps token overage: protect the margin, protect the boundary, keep the lightness real.

Saying "that's a scoped piece, here's a one-page SOW" is not friction — it is what keeps the 1h/week format honest. A retainer that silently absorbs builds is one that will be quietly resented and then churned.

## B.4 The quota agenda (`Weekly-Quota-Agenda.md`)

A fixed 60-minute structure so the hour stays strategic and never drifts into ops:

```
[0-10 min]  The one-screen review
            - NSM + trend, top cohort retention, top alert, cost vs budget
            - Decision: is anything on fire? (if yes → is it a Tweak or a scoped piece?)

[10-50 min] 1-2 strategic items (the real value of the hour)
            - The client's strategic question / arbitration, OR
            - This month's optimization hypothesis (if it's a decision, not a build)
            - New-integration scoping → decide: inside-hour answer, or scope a SOW?

[50-60 min] Capture + boundary
            - Log decisions + the next-loop hypothesis
            - Name anything that became a scoped mini-engagement (→ /market-proposal)
            - Confirm owners for any action the CLIENT'S team takes (not you)
```

Senior-client courtesy: if they're time-pressed, run [0-10] + the single most important item, defer the rest to next week. The format is light by design — honour that.

## B.5 The economics: NRR on the account (mm-08)

The recurring base (the 1h/week retainer) + scoped expansions produces **net-revenue-retention** on the account: it grows in value year over year without a new logo.
- The **expansion value metric** is *departments / scope under management* — NOT hours. The account expands by covering more of the company (next department, next C-Level), not by billing more time on the same scope. This keeps the lightness intact while the revenue grows.
- mm-08's grandfather-and-raise: as the system's *measured* value grows (the QBR proves it), new scope is priced to current value; existing scope is grandfathered. You raise on expansion, not by squeezing the base.
- A healthy account shows NRR > 100% of delivered value: the same engagement, more of the company piloted, the retainer base steady and the scoped expansions compounding. That is the bridge to reference 04's retention-and-expansion play.

## B.5b Worked example (the boundary in practice)

A live client brings three things to the same weekly hour. The discipline is sorting them correctly in real time:

```
Request A: "Should we point the support agent at billing tickets too?"
  → strategic question. INSIDE the hour. Answer: directionally yes, but it's a new workflow —
    so the BUILD of it is a scoped piece (see C).

Request B: "The agent mislabeled three tickets yesterday, can you look?"
  → operational. NOT the hour's job. Route to the system owner; if it's a real defect,
    it's a Tweak in the loop, not a live debugging session that eats the strategic hour.

Request C: "Great — so build the billing extension."
  → a new workflow = a margin-bomb if absorbed. Scope it: ~10 dev-days, F-XXX spec,
    one-page SOW via /market-proposal, priced to current measured value. NOT "I'll just do it."
```

The CAIO who answers A in the hour, deflects B to the owner, and scopes C as a SOW has protected three things at once: the strategic value of the hour, the margin (mm-08), and the positioning (the system stays autonomous; the CAIO stays strategic). The CAIO who quietly "just builds C this once" has started the slide from a clean 1h/week retainer into unpaid full-time work — and taught the client that builds are free. That is the exact services-analogue of mm-08's unbounded-token-usage margin bomb.

## B.6 Phase 4-5 discipline checks

| Check | Pass = |
|---|---|
| Weekly read is one-screen, triage-only, starts no projects | yes |
| Monthly loop re-scores the architect backlog on ACTUAL data, not estimates | yes |
| Retention/adoption fixes outrank new builds while any cohort leaks | yes |
| The chosen improvement is a falsifiable hypothesis (Because… I believe… I'll know if…) | yes |
| The verdict is exactly one of Tweak / Build / Expand, with routing | yes |
| Build verdicts route OUT to agentic-systems-builder / skill-forge (this skill doesn't build) | yes |
| Expand verdicts gate on no-leak + healthy realization, then loop to the architect | yes |
| The 1h/week agenda has a 0-10 / 10-50 / 50-60 structure | yes |
| The overage-is-a-mini-SOW boundary is explicit (mm-08 margin) | yes |
| Expansion value metric = scope/departments, not hours | yes |

---

*The loop keeps the system compounding. The hour keeps the engagement honest about its own lightness. Both are doctrine, not preference.*

# 01 — ROI Measurement Methodology (baseline → actual, by cohort, vs the projection)

Phases 1-2 of CAIO Run & Optimize. Outputs `caio-run/ROI-Measurement-Model.md` and the `roi_actual_vs_projected` block of `metadata.json`.

This is the heart of the run. The architect *projected* ROI in `company-ai-os/09-ROI-Governance-And-Risks.md`. Your job is to measure what actually happened — from telemetry and receipts, never from estimates — and to put the actual beside the projection so the engagement either proves or honestly falsifies its own thesis. The doctrine lens is **mm-11** (measure-loops-retention): a North Star Metric, cohort retention curves, and the leaky-bucket hierarchy applied to a *delivered system* instead of a SaaS product.

> The architect sold a number. You are the auditor of that number. If you cannot trace it to a logged event, an invoice, or a timesheet, it does not exist.

---

## Part A — Define the system NSM + reconcile the baseline (Phase 1, run ONCE)

### A.1 Pick the system North Star Metric (mm-11)

The architect produced 10-50 scored opportunities, each with its own metric. You need ONE number above them that answers, weekly: *is the delivered system delivering value?*

mm-11's rule: the NSM **precedes revenue** and **represents value the client receives**, not activity the system emits. Calibrate it to the dominant business objective the architect recorded in `01-required-inputs`:

| Architect's objective | Candidate system NSM | Why it is value-received |
|---|---|---|
| Save time | Net hours redeployed to higher-value work / week | Hours given back AND reinvested — not "hours the agent ran" |
| Reduce cost | Verified cost avoided / month (rework, overtime, vendor, error-cost) | A receipt-backed cost line that fell |
| Increase revenue | Qualified pipeline actions the system completed / week | The output that *precedes* booked revenue |
| Improve quality | Decisions improved / week (human-confirmed "this changed the call") | The judgment the AI augmented, counted at the point of decision |
| Centralize operations | Workflows piloted from the dashboard / week (vs run blind) | Operations made legible and steered |

**The vanity guard (mm-11, mandatory).** Run the test on your candidate: *if it doubles, are the client's people objectively better off?*
- "Agent runs", "tokens consumed", "API calls", "reports generated" → FAIL. They can double because the system loops uselessly, or because people are fighting it. A metric that can climb while the company is frustrated is a trap.
- "Net hours redeployed", "decisions improved", "cost avoided" → PASS. They cannot double without the company actually being better off.

Write the chosen NSM, its telemetry source (the exact event), and its vanity-test result at the top of `ROI-Measurement-Model.md`. Everything downstream rolls up to it.

### A.2 Reconcile the baseline (do NOT trust an assumed baseline)

ROI is `baseline − actual`. If the baseline is wrong, the whole model is wrong. The baseline should already exist — enablement (S4) was supposed to *instrument for baseline* before go-live, and the architect's 09-ROI recorded the pre-go-live `hours × loaded-cost × frequency` per workflow.

Your job is to **reconcile**, not re-invent:
1. Pull the architect's baseline per workflow from 09-ROI.
2. Confirm each baseline number has a real source (a timesheet, an interview quote with a quantified task from the discovery dossier, a process-map step-count). If a baseline was an *estimate with no source*, flag it `baseline-unverified` — you will report its ROI with a confidence penalty, never as hard proof.
3. If enablement captured a clean pre-go-live measurement window, use THAT as the baseline (it beats the architect's projection-time estimate).

**Refusal:** if there is no baseline at all — neither a measured window nor a sourced 09-ROI figure — you cannot compute honest ROI for that workflow. Report it as `no-baseline → ROI unmeasurable, re-instrument` rather than inventing a before-number. A fabricated baseline is the most seductive lie in this whole methodology because it makes every "saving" look huge.

### A.3 The loaded-cost reference (consistent with the architect)

Use the SAME loaded-cost methodology the architect used, so actual and projected are comparable. Loaded hourly cost = annual salary × loading factor (1.3-1.5×, default 1.4×) ÷ ~2080 working hours. Use the client's own loaded-cost figure if they provide one — it ends the argument before it starts.

```
Role example      | Annual salary | Loaded (1.4x) | Hourly ($)
Junior IC         | $50k          | $70k          | $34
Mid IC            | $80k          | $112k         | $54
Senior IC         | $130k         | $182k         | $87
Manager           | $150k         | $210k         | $101
VP                | $250k         | $350k         | $168
C-Level           | $400k+        | $560k+        | $269+
```

---

## Part B — Measure the actuals from telemetry/receipts (Phase 2)

For each shipped workflow, you measure FIVE actual quantities. Each has a *source rule* — where the number must come from. No source, no number.

### B.1 Hours saved (actual)

```
actual hours/week = baseline hours/week − residual manual hours/week
```
- **Residual manual hours** come from event telemetry (how many items the system now handles end-to-end vs how many still route to a human) cross-checked with a short timesheet or a spot interview. The system rarely automates 100% — there is HITL, edge cases, and an adoption ramp. The *honest* saving is net of the residual.
- **Source rule:** event log (items auto-handled) + a human-confirmed residual estimate. Telemetry alone over-counts (it doesn't see the human still double-checking); a timesheet alone under-counts (people forget). Use both.

### B.2 Cost avoided (actual)

- Direct cost lines that fell: overtime hours, contractor/agency spend, vendor tool retired, error-remediation cost, SLA penalties avoided.
- **Source rule:** an invoice, a payroll line, or a ticket count × cost-per-ticket. A "we probably saved" is not cost avoided.
- **Net it against the system's own cost** (B.5). A workflow that saves $4k/mo in labour but costs $1.5k/mo in model spend nets $2.5k — report the net.

### B.3 Value created (actual)

- Revenue-side or capability-side value the system enabled: pipeline actions completed, faster cycle time that closed deals sooner, a new capability the company simply could not do before.
- **Source rule:** the CRM/product telemetry for the action, attributed conservatively. Use *directional* attribution (mm-11 — don't chase perfect multi-touch attribution; "the system did this action, which precedes this outcome" is enough), and never claim revenue the system only *touched*.

### B.4 Adoption rate (actual)

- `active users of the feature ÷ target users` from product telemetry (WAU/MAU per feature), plus runs-per-workflow.
- Adoption is the multiplier on every other number: a workflow with 30% adoption delivers ~30% of its modelled saving regardless of how good it is. mm-11's leaky-bucket logic — a feature people stop opening is churning even if it "works".
- **Source rule:** active-user telemetry, not a license count. Seats sold ≠ seats used.

### B.5 The system's own cost (the denominator)

- Model spend (from the cost meter wired in runbook §5.8), infra, monitoring, and the maintenance/ops time.
- mm-08 discipline: **model cost should stay a fraction of the value** — if cost-per-NSM-unit creeps above ~20-30% of the value captured, the workflow's economics are degrading (flag it for the loop). One run-away power-user workflow can quietly turn a profitable system into a loss (mm-08 token-margin bomb).

### B.6 Decisions improved (quality objective)

When the objective is quality, hours are the wrong unit. Count **decisions improved**: instances where the AI surfaced an option, a risk, or a number that a human acted on, with a human-confirmed "this changed the call".
- **Source rule:** a logged decision artefact + a periodic human confirmation (a tick in the approval queue, a monthly 3-question survey to the decision-makers). Never infer "improved" from "the agent produced output" — output is activity, improvement is outcome (mm-11).

---

## Part C — Cohort ROI: the savings-retention curve (mm-11 cohort slice)

Do **not** average ROI across the system. An average of "+40% here, −0% there (dead workflow)" reads as "+20%" and hides the corpse. Measure by **cohort = go-live wave.**

### C.1 Build the cohort table

```
Cohort (go-live)  | Workflows | M1 saving | M3 saving | M6 saving | shape    | durable saving
2026-01 (wave 1)  | F001,F002 | $18k/mo   | $16k/mo   | $15k/mo   | plateau  | $15k/mo  (count this)
2026-03 (wave 2)  | F003      | $9k/mo    | $4k/mo    | $1k/mo    | decay    | ~$1k/mo  (LEAKING)
2026-05 (wave 3)  | F004,F005 | $22k/mo   | $21k/mo   | —         | plateau  | $21k/mo
```

### C.2 Read the shape (the diagnosis)

- **Plateau** (saving holds): the workflow became part of how the team works. The *plateau height* is the durable contribution — count THAT in the ROI model, not the month-1 spike. A plateau > 0 is the delivered-system equivalent of mm-11's product-market-fit signal: the team has "kept" the automation.
- **Decay toward zero**: the automation did not stick. People reverted to manual, or stopped trusting the output. This is the leaky bucket. The month-1 saving was a mirage. *Do not finance the next wave on top of a decaying cohort* (Iron Law 5).

### C.3 Compare cohorts (is delivery learning?)

- If wave-3 retains better than wave-1 → the delivery is improving; the team absorbs each wave better. Good — you can increase cadence.
- If later cohorts decay *faster* while you keep shipping → you are shipping faster than the company can absorb (mm-11 — "accelerating into the wall"). Slow down, fix retention, then resume.

### C.4 The strategic consequence

Cohort retention sets the optimization loop's priority order (see reference 03): a leaking cohort forces **retention work** (re-onboard the team, fix the agent, redesign the handoff) *before* any **acquisition** (next department). This is mm-11's `retention → monetization → acquisition` made operational on a delivered system.

---

## Part D — The Re-Measure: prove or falsify the projection

### D.1 Lay actual beside projected

For each workflow, put your measured actual next to the architect's 09-ROI projection:

```
| Workflow | Projected $/yr | Actual $/yr (durable) | Ratio | Verdict    | Why the delta |
|----------|----------------|-----------------------|-------|------------|---------------|
| F001     | $187k          | $171k                 | 91%   | Proven     | adoption 88%, slightly higher residual |
| F002     | $75k           | $58k                  | 77%   | Partial    | slower adoption ramp on C-suite side |
| F003     | $96k           | $11k                  | 11%   | Falsified  | team reverted to manual; agent output distrusted |
```

### D.2 The verdict bands

- **Proven** — actual ≥ ~80% of projected. The projection held. Bank it in the QBR; this is the engagement's credibility.
- **Partial** — 50-80%. The value is real but the projection was optimistic. Name the cause (adoption ramp, edge-case rate, lower frequency than estimated) — it feeds the next architect pass.
- **Falsified** — < 50%. The architect mis-scoped this opportunity, OR the implementation/enablement did not land it. Say so plainly. Route it back into the loop as a fix (retention) or a kill.

### D.3 Never round a falsification up (L2 — researcher, not sycophant)

The temptation is enormous: the client paid for the projection, so you want every line green. Resist it. A projection the data contradicts is the single most valuable finding in the whole engagement — it is exactly what makes the next architect pass better (mm-11 — a test must be able to give you *wrong*). A QBR full of unverifiable greens is worth less than one with an honest red and a fix plan, because the C-Level can *trust* the greens.

### D.4 The aggregate

```
## Re-measure summary (this quarter)
- Workflows measured: N
- Proven: X   Partial: Y   Falsified: Z
- Projected annual benefit (architect):   $___
- Actual durable annual benefit (measured): $___
- Realization rate: ___%   (actual / projected)
- System cost (annual): $___   →   Net: $___   →   Payback to date: ___ months
- Honest verdict: [one line the CFO can trust]
```

The realization rate is the engagement's headline integrity number. A mature run lands 70-100%; a first-quarter run often lands lower because adoption is still ramping — say so, don't hide it.

---

## Part E — Worked example (the re-measure, end to end)

Client: the SaaS B2B from the architect's demo (120 employees, GDPR). Wave-1 shipped two workflows. Objective recorded: *save time*. System NSM chosen: **net hours redeployed to higher-value work / week**.

### E.1 The architect's projection (from 09-ROI)
```
F002 Weekly Executive AI Brief : 12h/wk across 4 depts × $101/h × 52 = $63k/yr, 80% auto → projected $50k/yr saving
F001 Tier-1 Support Triage     : 6 reps × 10h/wk × $54/h × 52 = $168k/yr, 60% auto → projected $101k/yr saving
Projected combined: $151k/yr
```

### E.2 The measured actuals (quarter 1, from telemetry/receipts)
```
F002 Exec Brief
  - baseline 12h/wk (sourced: 4 dept-head timesheets, pre-go-live window) ✓ verified
  - residual manual now: 2h/wk (COO still edits before send) — source: approval-queue log
  - actual saving: 10h/wk × $101 × 52 = $53k/yr
  - adoption: 4/4 dept heads contribute, COO reviews 100% (HITL by design) — source: usage telemetry
  - system cost: $4k/yr model + $1k infra = $5k → net $48k/yr
  - cohort shape: M1 $4.4k/mo → M3 $4.4k/mo → plateau
  - VERDICT vs $50k projected: ratio 96% → PROVEN

F001 Support Triage
  - baseline 6×10h/wk (sourced: 09-ROI, but estimate-only) → flagged baseline-unverified
  - actual: WAU dropped to 41% by week 8; reps reverted to manual after a March prompt change
  - measured residual: 7.5h/wk still manual → actual saving only 2.5h/wk/rep
  - actual saving: 6 × 2.5h × $54 × 52 = $42k/yr gross; system cost $9k → net $33k/yr
  - cohort shape: M1 $14k/mo → M3 $4k/mo → M6 $1k/mo → DECAY
  - VERDICT vs $101k projected: ratio 33% → FALSIFIED (not the model's fault — adoption collapsed)
```

### E.3 The honest aggregate
```
Projected combined: $151k/yr     Actual durable: $48k + ~$12k (post-decay plateau) ≈ $60k/yr
Realization rate: ~40% this quarter
Honest verdict: "F002 proven and banked. F001 falsified — not because the opportunity was
wrong, but because a March prompt change broke the team's trust and adoption collapsed to 41%.
This is a retention fix, not a kill: the optimization loop will revert the prompt + re-onboard,
and we re-measure F001 next quarter before any new wave."
```

This is the model working as intended: one workflow proven and banked, one falsified *with a diagnosed cause and a fix path* — and the cohort decay on F001 is exactly why Iron Law 5 forbids opening a third department this quarter. Fix the leak first.

---

## Part E2 — Edge cases (do not let these corrupt the number)

- **The adoption-ramp confound.** Quarter 1 actuals are *always* lower than steady state because adoption is still climbing. Report the realization rate AND the trend ("40% now, climbing 8pts/month"). Do not present a ramp as a failure, and do not extrapolate a ramp as if it were a plateau.
- **The attribution-greed trap.** When a workflow "touches" revenue, the temptation is to claim all of it. Use directional attribution (mm-11): claim the *action the system completed*, not the downstream deal it merely influenced. Over-claiming once destroys the CFO's trust in every number forever.
- **The seasonality confound.** A cost line that fell in a quarter the business was seasonally slow is not a saving. Compare like-for-like periods or annualize carefully.
- **The double-count.** If two workflows both touch the same hour saved, count it once. Reconcile overlapping savings across cohorts before aggregating.
- **The "we feel faster" temptation.** A sponsor's anecdote ("the team seems less swamped") is a hypothesis to measure, never a number to report. Hours come from logs + timesheets, not from morale.

---

## Part F — Re-measure cadence

- **Monthly:** refresh the cohort table (the savings-retention curve needs monthly points to read its shape) and the NSM trend.
- **Quarterly:** the full re-measure (actual vs projected, all verdicts) → feeds the QBR.
- **On a falsification:** don't wait for the quarter. A workflow that crossed from Partial to Falsified is a leaking cohort — open a retention fix in the next weekly loop.

---

## Part G — Phase 1-2 discipline checks

| Check | Pass = |
|---|---|
| System NSM chosen, value-received, passes the "double it" vanity test | yes |
| Baseline reconciled to a real source per workflow (or flagged no-baseline) | yes |
| Every actual (hours/cost/value/adoption/decisions) cites a telemetry event or a receipt | yes |
| System cost netted into the saving (model + infra + ops) | yes |
| ROI reported by cohort with a savings-retention shape, not a single average | yes |
| Actual laid beside the architect's projection, per workflow | yes |
| Each workflow carries a Proven / Partial / Falsified verdict | yes |
| Falsifications kept and explained, not smoothed into greens | yes |
| Realization rate computed at the aggregate | yes |

If any fails, fix it before the number reaches the QBR. The re-measure is the engagement's word of honour — it ships clean or not at all.

---

*The architect projected. You measured. When they disagree, the telemetry wins — and saying so is the job, not a failure of it.*

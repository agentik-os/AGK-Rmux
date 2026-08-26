# Reference 03 — Per-C-Level Micro-SaaS + the Inter-Dashboard API Contract (5.3)

> Two halves of the build's core. **§A** — how to build one function-specific micro-SaaS per C-Level (built for that person's real job, never a recycled template). **§B** — the inter-dashboard API contract: how each dashboard exposes and consumes APIs so the seven instruments run as **one interconnected organism** (the offer's 5.3 differentiator).

---

# §A — The per-C-Level micro-SaaS build checklist

## A.0 The anti-template law (Iron Law 3)

A recycled template repainted seven times is the failure mode this offer exists to beat. The architect already specified each feature for each seat (`07-Dashboard-Feature-Specs.md`); the realization spec already wrote, per seat, **the one job a generic dashboard gets wrong** (`01-architecture-realization.md` §2). The build honors both. Before you build a seat, you must be able to say, in one sentence, why *this* dashboard is a different instrument from the others.

| Seat | The instrument (not "a dashboard") | The line a generic dashboard gets wrong |
|---|---|---|
| CIO/CTO | system health + delivery + AI-spend | "shows uptime" → needs *delivery velocity vs incident cost* + model-spend by team |
| CMO | pipeline + CAC + channel | "shows signups" → needs *CAC net of later-churned deals* per channel |
| CFO | cash + margin + forecast | "shows revenue" → needs *forecast vs actual with the variance's cause* |
| CDO | data quality + insight throughput | "shows row counts" → needs *quality score gating downstream pipelines* |
| COO | throughput + SLA + bottlenecks | "lists late tickets" → needs *breaches ranked by downstream cash impact* |
| CHRO | headcount + hiring + capacity | "shows headcount" → needs *capacity vs load, hiring-funnel velocity* |
| CSO | revenue + forecast + win-rate | "shows closed-won" → needs *pipeline coverage + forecast-miss risk* |

## A.1 The build checklist (per micro-SaaS)

Run this for each seat. It is the operational checklist behind `03-MicroSaaS-Build-Plan.md`.

```
1. SCOPE  — read the seat's feature specs in 07-Dashboard-Feature-Specs.md + the backlog opportunities
            it owns. List the views to build (P1 first). Confirm the "real job" line. Confirm acceptance criteria EXIST.

2. DATA   — from the realization spec's Composio topology, confirm the system(s)-of-record feeding this seat
            and that its connector(s) passed a live read (reference 04). No live read -> no view.

3. SCHEMA — add the seat's metrics to the shared Convex `metrics` table (it EXPOSES here) and any
            seat-local tables. Each metric carries value, ts, confidence, sourceUrl (anti-black-box).

4. AGENTS — for each F-XXX agent powering a view, DISPATCH to agentic-systems-builder with the F-XXX spec.
            DO NOT re-implement the agent here (Iron Law 10). Wire its agentRuns + costEvents logging.

5. VIEWS  — build the Next.js views in the seat's route group, gated by Clerk role. Every panel that states
            a number shows its source (sourceUrl) + freshness + confidence. No bare numbers (Iron Law 6).

6. EXPOSE — publish the seat's key metrics to the federation bus (§B). Define what this dashboard EXPOSES.

7. CONSUME— subscribe the seat to the alerts it should receive from other dashboards (§B). Wire HITL where the
            architect's matrix requires it.

8. INSTRUMENT — wire the 3 baseline events for this seat (NSM, cost/usage, value) per mm-11
            (reference 05 §A). t0 = this dashboard's go-live.

9. REPORT — wire the seat's automated report (frequency/format/indicators/recipients) per reference 04 §B.

10. SHIP-GATE — run the seat's acceptance test (criteria from 07-Dashboard-Feature-Specs.md) on REAL data via
            /omg-acceptance. Green -> ship (value in week 1). Red -> back to the builder (reference 05 §C).
```

## A.2 Anti-black-box per panel (Iron Law 6)

Every panel that asserts a value must expose, inline or on hover:
- **source** — the `sourceUrl` (the CRM record, the Stripe invoice, the analytics query) the number came from.
- **freshness** — when it was last read (ties to the connector's `lastLiveRead`).
- **confidence** — for any LLM-derived value, the agent's confidence; for a direct read, "exact".
- **status / errors** — if the feeding agent failed, the panel says so; it does not show a stale number as if fresh.
- **cost** — the AI-spend dashboards (CIO/CTO, CFO) surface model cost per workflow from `costEvents`.

> A panel that shows a number with no source is a black box. A CFO will not trust it, and they are right not to. The source link is not decoration — it is what makes the system enterprise-grade.

## A.3 HITL inside the dashboard (Iron Law 9)

Any action the architect's matrix marks sensitive (financial action, headcount, customer-facing, regulated) renders as an **approval step**, not an auto-execute. The agent prepares; a named Clerk role approves; the `approvals` table records who/when. The dashboard never auto-fires a sensitive action — not under deadline pressure, not "to demo well."

## A.4 Worked build example — the CFO micro-SaaS

Walking the 10-step checklist for one seat, so the abstraction is concrete:

```
SEAT: CFO. The instrument = "cash + margin + forecast". Real-job line = "forecast vs actual WITH the
variance's cause, not a revenue line chart."

1. SCOPE   — feature specs F011 (Cash dashboard), F012 (Forecast-vs-actual), F002 (Weekly exec brief share).
             Acceptance for F011 exists: "cash position matches Stripe+bank within $1, refreshed hourly."
2. DATA    — SoR: Stripe (cash/invoices) + ERP (P&L). Composio ERP + CRM connectors live-read PASSED.
3. SCHEMA  — metrics{cfo, cash_position}, {cfo, gross_margin}, {cfo, forecast_variance} on the shared table.
4. AGENTS  — F012's "variance-attribution agent" dispatched to agentic-systems-builder (NOT built here).
             It reads slipped deals from CRM + actuals from Stripe and proposes the cause. agentRuns+costEvents wired.
5. VIEWS   — app/(cfo): a cash panel, a margin panel, a forecast-variance panel. EVERY number links its
             Stripe/ERP source + shows freshness + "exact" (direct read) or the agent confidence (attribution).
6. EXPOSE  — publish metrics cfo.cash_position, cfo.forecast_variance to the federation bus.
7. CONSUME — subscribe to COO.cash_impact_alert + CMO.burn_alert. The cash-impact alert is financial → HITL:
             renders as a pending card the CFO approves; never auto-acts.
8. INSTRUMENT — baseline events: NSM="weekly cash brief shipped on time + read"; usage=model$+tokens+run;
             value="variance cause identified before the board meeting". t0 = CFO go-live.
9. REPORT  — R-CFO-WEEKLY-CASH wired: Mon 07:00, 1-page, numbers cited, model does no math (reference 04 §B).
10. SHIP-GATE — /omg-acceptance on real Stripe data: cash matches within $1, console clean, golden path
             (CFO logs in → sees cash → opens variance → approves a pending alert) walked. GREEN → ship week 1.
```

Notice what makes it the CFO's instrument and not a repaint: the **variance-attribution agent** (step 4) and the **cause shown next to the number** (step 5). A generic dashboard would show "revenue: $X" with no source and no why. This shows "forecast -2.4%, caused by two named slipped renewals, here are the CRM links." That is the job a generic dashboard gets wrong (A.0).

---

# §B — The inter-dashboard API contract (5.3, the differentiator)

## B.0 Why this is the product

Seven isolated dashboards is seven dashboards. The Company-AI-OS is the **federation**: a metric crossing a threshold on one C-Level's instrument raises an alert on another's. A COO fulfillment breach becomes a CFO cash-impact alert. A CMO CAC spike becomes a CFO burn alert. A CSO forecast-miss risk becomes a CHRO capacity signal. **This is principle #2 (C-Level interconnection).** Without it, you shipped silos (Iron Law 4).

## B.1 The contract is a typed event on the shared bus — not a webhook web

Do **not** build N×M point-to-point webhooks between dashboards — that is fragile and unauditable. The federation lives on the **shared Convex event bus** (the `metrics` + `alerts` tables from reference 02 §4). One write, reactive evaluation, typed payloads.

### Exposed metric event

```
metrics.insert({
  seatKey:   "coo",
  metricId:  "fulfillment_sla_breach",
  value:     { severity: "high", hoursLate: 31, accountArr: 84000 },
  ts:        now,
  confidence:"exact",                 // or an agent confidence 0..1 for LLM-derived
  sourceUrl: "https://<ticketing>/ticket/9921"
})
```

### Subscriber rule (declared in the contract, evaluated reactively)

```
ON  metrics WHERE seatKey="coo" AND metricId="fulfillment_sla_breach" AND value.severity>="high"
RAISE alerts.insert({
  fromSeat: "coo", toSeat: "cfo", alertId: "cash_impact_alert",
  payload:  { costOfDelay: estimate(value), accountArr: value.accountArr, ticket: sourceUrl },
  threshold:"severity>=high",
  hitlRequired: true,                 // sensitive (financial) -> HITL per architect matrix
  status:   hitlRequired ? "pending_approval" : "raised",
  ts: now
})
```

The CFO dashboard **consumes** by subscribing to `alerts WHERE toSeat="cfo"`. Reactive — no polling, no webhook plumbing. This is exactly why Convex is the load-bearing stack choice (reference 02 §1).

## B.2 The contract map (the design artifact, enforced at build)

The realization spec (`01-architecture-realization.md` §3) produced the map; the build implements every row and **tests at least one** end-to-end. Each row:

| Field | Meaning |
|---|---|
| Source seat · metric | which dashboard exposes, which metricId |
| Trigger condition | the threshold (lives in the contract, sponsor-visible, not buried in code) |
| Target seat · alert | which dashboard consumes, which alertId |
| Payload | the typed fields carried (always incl. sourceUrl + confidence) |
| HITL? | if the alert drives a sensitive action, the approver role (architect's matrix) |
| Tested? | the live end-to-end test that proves it fires (evidence — R-CITE) |

## B.3 Worked end-to-end example (COO breach → CFO alert)

```
1. Ticketing (Composio Slack/ticketing connector) reports a support ticket 31h past SLA on an $84k-ARR account.
2. The COO dashboard's SLA agent writes metrics{coo, fulfillment_sla_breach, severity:high, ...} with sourceUrl.
3. The subscriber rule fires reactively -> alerts{coo->cfo, cash_impact_alert, costOfDelay:$X, hitlRequired:true, pending_approval}.
4. The CFO dashboard shows a pending cash-impact alert WITH the ticket source link + the cost-of-delay estimate + confidence.
5. Because it's financial (sensitive), it does NOT auto-act: the CFO (Clerk role) reviews -> approves/dismisses -> approvals row written.
6. baselineEvents records the value-delivered event ("cross-seat alert surfaced + actioned") for mm-11.
```

The CFO learned about an operational problem *before* it hit the P&L — and can see exactly where it came from. That is the organism. That is what the client is paying for.

## B.4 Federation safety rules

- **Sensitive alerts route through HITL.** Financial action, headcount, customer-facing, regulated → `hitlRequired:true`, never auto-fire (Iron Law 9).
- **Every alert carries `sourceUrl` + `confidence`.** A cross-dashboard alert with no provenance is a rumor (Iron Law 6).
- **Thresholds live in the contract**, surfaced in the governance view, changeable by the sponsor — not hard-coded.
- **No alert storms.** Debounce/throttle (e.g. one cash-impact alert per account per day) so the CFO is not flooded — a flooded alert is an ignored alert (the mm-11 honesty principle: a metric the user stops reading is worthless).
- **Loops are forbidden.** A → B → A federation cycles are caught at design time in the contract map; the bus rejects a rule whose target re-triggers its own source.

## B.5 Federation acceptance (the ship-gate for 5.3)

| Check | Pass = |
|---|---|
| Every contract-map row is implemented (rule exists on the bus) | yes |
| ≥1 row tested end-to-end on real data (a real metric raised a real alert) | yes (evidence cited) |
| Every sensitive alert is `hitlRequired` and renders an approval step | yes |
| Every alert payload carries sourceUrl + confidence | yes |
| Thresholds are contract-stored + governance-visible, not hard-coded | yes |
| Debounce/throttle present; no alert storm in a load test | yes |
| No A→B→A loop in the contract | yes |

Record in `08-Ship-Gate-Ledger.md`. A federation with zero tested rows is a paper contract — refused (Iron Law 4).

## B.6 Typical exposes / consumes catalog (a starting point per seat)

A builder's quick map of what each seat usually publishes to the bus and what it usually subscribes to. Use it to draft the contract map; the realization spec's actual map governs.

| Seat | Typically EXPOSES | Typically CONSUMES |
|---|---|---|
| CIO/CTO | `system_health`, `deploy_status`, `model_spend_spike`, `data_pipeline_state` | CDO `data_quality_drop`, CFO `budget_threshold` |
| CMO | `cac_spike`, `pipeline_velocity`, `channel_underperf` | CSO `forecast_miss_risk`, CFO `burn_alert` |
| CFO | `cash_position`, `forecast_variance`, `budget_threshold`, `burn_alert` | COO `cash_impact_alert`, CMO `cac_spike`, CHRO `headcount_cost_change` |
| CDO | `data_quality_drop`, `insight_ready` | CIO/CTO `pipeline_state`, all seats `metric_dispute` |
| COO | `fulfillment_sla_breach`, `bottleneck`, `cash_impact_alert` | CSO `demand_surge`, CHRO `capacity_signal` |
| CHRO | `capacity_signal`, `attrition_risk`, `headcount_cost_change` | CSO `forecast_miss_risk`, COO `bottleneck` |
| CSO | `forecast_miss_risk`, `demand_surge`, `win_rate_drop` | CMO `pipeline_velocity`, CFO `budget_threshold` |

Reading the table: most cross-seat value flows *toward the CFO* (operational signals become cash signals) and *toward capacity owners* (CHRO/COO). Those are the highest-signal federation rules to wire first — they are where one C-Level learning what another already knows changes a decision. Wire the one or two rules with the clearest decision-impact, test them end-to-end (B.5), then expand; do not wire all forty-nine possible edges (that is an alert-storm machine, not an organism).

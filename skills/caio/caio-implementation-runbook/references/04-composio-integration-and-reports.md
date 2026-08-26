# Reference 04 — Composio Integration Wiring (5.4) + Automated Reports (5.5)

> **§A** — the Composio integration wiring guide: connect the existing tools (CRM/ERP/marketing/analytics/HR/comms) that expose an API, via **the 6 critical connectors that actually work** — not a 200-connector list. **§B** — the automated-report spec: the report that *falls out automatically* so analysts analyze instead of copy-paste.

---

# §A — Composio integration wiring (5.4)

## A.0 The 6-critical-connectors rule (Iron Law 5)

Composio exposes hundreds of connectors. Enabling them all is theater: a dashboard of "200 integrations available" where six return real data and 194 are decoration. The rule:

> Wire **six** connectors — the ones that map to the system-of-record feeding each live dashboard — and each counts **only** when it passes a **live-read test**. A 7th is added only when a live dashboard provably needs it.

The default six (adapt to the org's real stack from the realization spec):

| # | Connector | System-of-record (examples) | Feeds | Returns |
|---|---|---|---|---|
| 1 | CRM | HubSpot, Salesforce, Pipedrive | CSO, CMO | deals, stages, contacts, owners |
| 2 | ERP / Finance | NetSuite, Stripe, QuickBooks, Xero | CFO | invoices, cash, margin, AR/AP |
| 3 | Marketing | GA4, Google/Meta Ads, email platform | CMO | spend, channels, conversions |
| 4 | Product analytics | PostHog, Amplitude, Mixpanel | CIO/CTO, CDO | usage, funnels, data-quality signals |
| 5 | HR / HRIS | BambooHR, Workday, an ATS | CHRO | headcount, hiring funnel, attrition |
| 6 | Comms / Ops | Slack, Teams, a ticketing system | COO + report delivery | tickets, SLAs, message routing |

Why six and not sixty: each connector is a maintenance surface (auth, rate limits, schema drift, breakage). Six that the client's team can own beats sixty that rot. The mm-11 honesty principle applies to integrations too — *a connector that never delivers real data is a leaky bucket*: it costs maintenance and returns nothing.

## A.1 Auth (per connector)

- **Prefer OAuth via Composio** (managed token refresh) over raw API keys where the tool supports it — the client revokes access in their own tool, and tokens refresh without a 2am page.
- **Store nothing in the repo.** Composio holds the connected-account; the Composio API key + any fallback tokens live in the client's secret manager (reference 02 §3 / R-ENV).
- **Least privilege.** Request the **read scopes** each dashboard needs; request write scopes only for the specific actions the architect's blueprint authorizes (and those route through HITL if sensitive — Iron Law 9).
- **Record auth status** in the `integrations` table (`authStatus`, `lastLiveRead`).

## A.2 Rate limits

- **Read the connector's limit before you build a polling loop.** A dashboard that refreshes every 10s against a CRM with a 100-req/min cap will get throttled and show stale data silently — a black-box failure (Iron Law 6).
- **Cache + schedule, don't hammer.** Pull on a Convex Scheduler cadence matched to how fresh the data must be (a CFO cash view: hourly; a CMO channel view: daily). Store `rateRemaining` in `integrations` and surface it in the monitoring view (reference 05 §B).
- **Backoff + surface.** On 429, exponential backoff and mark the panel "data delayed" — never show a stale number as fresh.

## A.3 The live-read test (the only thing that makes a connector "wired")

A connector is wired **only** when this passes — and the evidence is cited in `05-Integration-Wiring-Guide.md` (R-CITE):

```
LIVE-READ TEST (per connector)
1. Connect the account (OAuth/API key) under the client's org.
2. Execute one real read for the dashboard's actual need
   (e.g. CRM: fetch the 10 most-recent open deals; ERP: fetch this month's invoices).
3. Assert a REAL record returned (not an empty 200, not a mock).
4. Check rateRemaining > a safe floor for the planned cadence.
5. Write integrations{connector, systemOfRecord, authStatus:"ok", lastLiveRead:now, rateRemaining}.
6. Cite the evidence: timestamp + a redacted sample of the returned record.
```

An enabled connector that has never returned a real record is **not** wired (Iron Law 5). "Connected" in Composio's UI is not "wired" until the live read passes.

## A.4 Integration acceptance (the ship-gate for 5.4)

| Check | Pass = |
|---|---|
| ≤6 connectors enabled (a 7th only with a proven need) | yes |
| Each enabled connector passed the live-read test (evidence cited) | yes |
| Auth is least-privilege; tokens in the client's secret manager, not the repo | yes |
| Rate limits read; cadence matched; backoff + "data delayed" surface present | yes |
| Each connector maps to a system-of-record + a live dashboard in the realization spec | yes |
| No write scope beyond the architect-authorized actions; sensitive writes go through HITL | yes |

A connector that fails the live read does not count toward the six — fix or drop it. Record in `08-Ship-Gate-Ledger.md`.

## A.5 Worked wiring walkthrough (CRM → CSO/CMO)

The shape of wiring one connector end-to-end (adapt to the real SDK version; the *sequence* is the point):

```
1. CONNECT (client-owned account)
   - In Composio, create a connected-account for HubSpot under the CLIENT's Composio project.
   - Auth = OAuth (managed refresh). The client authorizes in their own HubSpot — they can revoke anytime.
   - Composio returns a connection id; store the id (not a token) referenced by integrations.connector="crm".

2. SCOPE (least-privilege)
   - Request read scopes the CSO/CMO dashboards need: crm.objects.deals.read, crm.objects.contacts.read.
   - Do NOT request write unless the architect's blueprint authorizes a specific action (and HITL-gate it).

3. LIVE-READ (in a Convex action, server-side)
   - Execute Composio action HUBSPOT_LIST_DEALS (limit 10, recent open).
   - Assert: a real deal record returned (id, stage, amount, owner). Empty 200 = FAIL.
   - Read the rate-limit header; compute rateRemaining for the planned daily cadence.

4. PERSIST (integrations row)
   - integrations.upsert({ connector:"crm", systemOfRecord:"HubSpot",
       authStatus:"ok", lastLiveRead: now, rateRemaining })

5. SCHEDULE (Convex Scheduler)
   - Pull on the cadence the dashboard needs (CMO channel: daily; CSO pipeline: hourly if pipeline is live).
   - On each pull, refresh metrics{seat, metricId, value, ts, confidence:"exact", sourceUrl}.

6. SURFACE FAILURE
   - On a Composio error / 429: exponential backoff; mark the feeding panel "data delayed";
     write agentRuns.error so the monitoring view (reference 05 §B) shows it. Never a stale number as fresh.

7. CITE (the proof)
   - Record in 05-Integration-Wiring-Guide.md: the live-read timestamp + a REDACTED sample record. (R-CITE)
```

### Token refresh + auth failure

- **Managed refresh (OAuth via Composio).** Composio refreshes the token; your code never stores it. If refresh fails (client revoked access, scope changed), the next action errors — catch it, set `integrations.authStatus="reauth_needed"`, surface it in monitoring, and notify the dashboard owner. Do not silently serve stale data.
- **Raw API key (when OAuth is unavailable).** Key lives in the client's secret manager, injected at deploy. Rotation is a documented step in the server runbook (reference 02 §3). A key with no rotation plan is a security finding.

### Multi-connector reconciliation

When two connectors describe the same entity (e.g. CRM "deal" + ERP "invoice" for the same account), pick **one as the system-of-record per field** (CRM owns stage/owner; ERP owns invoiced/paid). The realization spec's Composio topology already named the SoR per data type — honor it. Never show two conflicting numbers for the same fact; that is the silo problem the OS exists to kill.

## A.6 Integration traps

- **Counting "connected" as "wired."** Only the live read counts.
- **Polling into a rate limit.** Read the cap first; schedule to match freshness needs.
- **Over-scoping auth "to be safe."** Least privilege; broad scopes are a security finding.
- **Reading from a production DB instead of the system-of-record.** The AI layer reads from the SoR (warehouse/CRM/ERP), never directly from a prod DB it does not own (reference 02 §2).
- **A connector with no dashboard.** If no live dashboard needs it, it is not one of the six. Drop it.
- **Storing the OAuth token in your own table.** Composio holds it; you hold a connection id. Storing tokens yourself recreates the refresh problem you delegated.
- **Serving stale data as fresh on a refresh failure.** A failed pull marks the panel "delayed" — silence is a black-box failure (Iron Law 6).

---

# §B — Automated reports (5.5)

## B.0 The point: the report falls out automatically

The offer's 5.5: the report's **frequency, format, indicators, and recipients** are specified once, and then it **falls out automatically** — so analysts *analyze* instead of copy-pasting a deck every Monday. This is the architect's "Weekly Executive AI Brief" pattern, generalized to one report per seat (and the cross-seat exec brief).

## B.1 The report spec sheet (per report)

Captured in `06-Automated-Report-Specs.md` (template: `assets/templates/Automated-Report-Spec.md`):

```
Report ID:        [e.g. R-CFO-WEEKLY-CASH]
Owner seat:       [cfo]
Audience:         [exact recipients + channel: who, where]
Frequency:        [cron: e.g. Mon 07:00 client-timezone]
Format:           [1-page markdown to Slack #c-level + Notion DB row + PDF via omega pdf]
Indicators:       [the EXACT metrics, each with its sourceUrl + the threshold that makes it notable]
Narrative:        [LLM drafts the "what changed + why" — numbers are CITED, model does NOT do math]
HITL:             [does a human approve before send? e.g. COO reviews the cross-seat exec brief]
Delivery proof:   [the report logs a reports{lastRunAt, status} row + a delivery receipt]
```

## B.2 The numbers-are-cited rule (anti-hallucination, L1)

This is the single most important report discipline, inherited from the architect's flagship feature:

> **The model never does math.** All numbers come from Convex queries against real data (the connectors' live reads). The LLM **drafts the narrative** ("cash is down 4% week-over-week, driven by two slipped renewals") but **every figure is computed in a Convex action and cited with its sourceUrl**. The model is not allowed to compute or invent a number.

A report with an uncited number is a hallucination risk and is refused. The report's credibility — and the sponsor's trust (mm-04) — rests on this.

## B.3 Format + delivery

- **Format to the audience.** A CEO gets one page; a CFO gets the variance table; an analyst gets the drill-down link. Same data, audience-shaped (the mm-04 *clear-beats-clever* principle: the recipient understands it in seconds).
- **PDF via the OmegaOS pdfgen** when a branded document is needed: `omega pdf --template=audit|doc --data=<json> --out=<path> [--send]` — never hand-roll a generator (R-PDF).
- **Delivery is logged.** Each run writes `reports{lastRunAt, status}` and a delivery receipt so a missed report is *visible* (a silent missing report erodes the sponsor's confidence — mm-04).

## B.4 Frequency + the scheduler

- Drive reports off the **Convex Scheduler** (or Trigger.dev) — same deployment, client-owned, no extra vendor.
- Match frequency to the decision cadence: CFO cash (weekly), COO SLA (daily), CMO channel (weekly), cross-seat exec brief (weekly). Over-frequent reports get ignored; under-frequent ones miss the decision window.

## B.5 Report acceptance (the ship-gate for 5.5)

| Check | Pass = |
|---|---|
| Frequency, format, indicators, recipients all specified (no blanks) | yes |
| Every number is query-computed + cited; the model does no math (L1) | yes |
| The report ran on schedule on real data and reached its recipients | yes (delivery receipt) |
| A missed/failed run is visible (reports row + monitoring alert) | yes |
| Sensitive reports (e.g. cross-seat exec brief) pass HITL before send | yes |
| Format is audience-shaped; PDFs go through omega pdf (R-PDF) | yes |

Record in `08-Ship-Gate-Ledger.md`. The test of a report is not "it generated" — it is "it generated, on time, with real cited numbers, and the right person read it." The CMO/CFO who used to spend 12 hours assembling it now reads it in five minutes; that recovered time is the value the mm-11 baseline (reference 05 §A) records.

## B.6 Worked sample report (the output shape)

What a shipped `R-CFO-WEEKLY-CASH` actually produces — note that **every number carries a source link** and the LLM wrote only the prose around computed figures:

```
CFO Weekly Cash Brief — week of {{date}}                    [auto-generated, Mon 07:00]

CASH POSITION:  $1.24M  (▼ 4.1% WoW)        [source: Stripe balance + bank feed →link]
  Driver: two enterprise renewals ($88k, $52k) slipped from this week to next.
          [source: HubSpot deals #4471, #4482 →link]   confidence: exact (no model math)

MARGIN:         61.3% gross  (▲ 0.6pt WoW)  [source: ERP P&L query →link]

FORECAST vs ACTUAL: -$31k vs plan (-2.4%)   [source: forecast table vs Stripe actuals →link]
  Variance cause: the two slipped renewals (above), not a demand miss.

CROSS-SEAT ALERTS THIS WEEK:
  • COO → CFO cash-impact alert: 1 SLA breach on an $84k-ARR account (cost-of-delay $3.1k)
    [source: ticket #9921 →link]   status: CFO reviewed, mitigation assigned

NOTHING ELSE CROSSED A NOTABLE THRESHOLD.
```

The narrative ("two renewals slipped", "not a demand miss") is the LLM's; **every figure is a Convex query result with a source link**, and the model did no arithmetic (B.2). This is the difference between a report a CFO trusts and a hallucination machine. It is also the mm-04 honesty substrate: a sponsor demo of *this* — live, on real numbers — keeps the budget; a demo of a hard-coded mock loses it (reference 05 §D).

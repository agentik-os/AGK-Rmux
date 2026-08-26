# Reference 01 — Architecture Realization (the design gate)

> This is the stage the architect deliberately left open. `caio-enterprise-workflow-architect` ships a **generic** `company-ai-os/` — an opportunity backlog, per-feature specs, agentic blueprints, a roadmap, an ROI projection, and a **single unified dashboard model**. It does **not** design the offer's *signature* topology. This reference is the method for translating that generic blueprint into the offer's **centralized federated architecture** — and getting it **approved before any build starts**.

> **No realization spec, no build.** (Iron Law 1.) You cannot build what you did not first design.

---

## 0. Why a separate design stage exists

The architect's dashboard model is a *single* unified surface — correct for an audit, wrong for the offer. The offer's product is a **federation**: one server, one micro-SaaS per C-Level, wired by an inter-dashboard contract. The gap between "a dashboard" and "a federation of seven instruments that talk to each other" is a **design decision** with consequences for the server shape, the data model, the integration topology, and the build order. Making that decision implicitly — discovering it mid-build — is how black-box, over-budget, silo systems get shipped.

So the realization spec is a **gate**: a written, reviewed, *approved* artifact. It costs a day or two. It saves the engagement.

The realization spec is also the first place mm-04 fires: the approval is framed as a **value proposition the sponsor signs** (see §7).

---

## 1. Inputs — read the blueprint before designing

Before writing a line of the realization spec, read, in order:

1. `company-ai-os/05-Automation-Opportunity-Backlog.md` — the scored, classified opportunities. Each carries a **success metric** (this becomes the dashboard's North-Star event later) and a **verdict** (Build now / P3 / Park / REFUSED). You build the "Build now" + "P3" set; you do not build "REFUSED" (Class 8) — ever.
2. `company-ai-os/07-Dashboard-Feature-Specs.md` — the 12-field feature specs *with acceptance criteria*. **The ship-gate criteria are pulled verbatim from here.** If a feature has no acceptance criterion, stop and route back to the architect — you cannot ship-gate what was not specified.
3. `company-ai-os/06-Agentic-System-Blueprints.md` — per-agent design (problem, workflow, tools, memory, KB, HITL, logs). Each agent maps to a micro-SaaS and becomes an `F-XXX` dispatch to `agentic-systems-builder`.
4. `company-ai-os/08-Implementation-Roadmap.md` — phase order, cost, and the ROI projection per workflow. The build order respects this; the projection is what the mm-11 baseline will later be checked against.
5. `company-ai-os/09-ROI-Governance-And-Risks.md` — the **HITL matrix** (which cross-dashboard alerts must route through a human), data-residency posture, governance.
6. (Optional) discovery `company-rollup.md` — **which C-Level seats actually exist** + the system-of-record per data type. Decisive for §2.

> If `company-ai-os/` is absent or pre-`07` stage → **STOP**. Route to `caio-enterprise-workflow-architect`. Building without a blueprint is refused.

---

## 2. The seat map — which micro-SaaS get built

The offer's pattern is "one function-specific micro-SaaS per C-Level." The seven canonical seats:

| Seat | Owns | This dashboard is an instrument for… |
|---|---|---|
| **CIO / CTO** | technology, IT, eng delivery, infra/model cost | system health + delivery velocity + AI-spend |
| **CMO** | marketing, demand gen, brand, content | pipeline + CAC + content/channel performance |
| **CFO** | finance, cash, margin, forecast, runway | cash position + margin + forecast vs actual |
| **CDO** | data, analytics, data quality, governance | data-quality score + insight throughput |
| **COO** | operations, fulfillment, SLAs, process | throughput + SLA breaches + bottlenecks |
| **CHRO** | people, hiring, retention, capacity | headcount + hiring funnel + capacity vs load |
| **CSO** | sales / revenue (default) or strategy | revenue + forecast + win-rate + quota attainment |

### The anti-template discipline (Iron Law 3)

For **each** seat you decide to build, the realization spec must state — in one line — **the one job this person does that a generic dashboard would get wrong.** Examples:

- *CFO:* "Needs *forecast vs actual with the variance attributed to a cause*, not a revenue line chart. A generic dashboard shows the number; the CFO needs to know *why it moved*."
- *COO:* "Needs *SLA breaches ranked by downstream cash impact*, not a list of late tickets. The ranking is the instrument."
- *CMO:* "Needs *CAC by channel net of the deals that later churned*, not gross signups. A generic dashboard counts vanity."

If you cannot write that line, you do not understand the seat well enough to build it — go back to the discovery rollup.

### Build only the seats that exist (Iron Law 3)

A 40-person company may have a CEO, a COO, and a "head of everything." It does **not** have a CDO. **Do not build a CDO dashboard for a seat that does not exist.** Fold its data-quality view into the CIO/CTO or CFO seat. The seat map records: seat → exists? (yes/no) → if no, which existing seat absorbs its view.

Output of §2: a table `seat → exists → backlog opportunities it owns → F-XXX agents it runs → the "real job a generic dashboard gets wrong" line`.

---

## 3. The federation contract map (5.3 — the differentiator)

This is the heart of the realization spec. The architect's single dashboard had no concept of one surface alerting another. The federation does. You design, as a **table**, every cross-dashboard relationship:

| # | Source seat · metric | Trigger condition | → Target seat · alert | Payload | HITL? |
|---|---|---|---|---|---|
| 1 | COO · `fulfillment_sla_breach` | severity ≥ high, > 24h | CFO · `cash_impact_alert` | cost_of_delay, contract_id | CFO review |
| 2 | CMO · `cac_spike` | CAC > 1.4× 4-week avg | CFO · `burn_alert` | channel, delta_$ | none (read-only) |
| 3 | CSO · `forecast_miss_risk` | pipeline coverage < 2.5× | CHRO · `capacity_signal` | gap_$, quota_owner | CHRO review |
| 4 | CDO · `data_quality_drop` | quality score < 7/10 | CIO/CTO · `pipeline_freeze` | source, failing_fields | CTO review |

Rules for the map (design-time, enforced at build):
- **Direction is explicit.** A metric *exposes*; an alert *consumes*. One source, one or more targets.
- **The threshold is in the contract**, not buried in code — it is a governance decision the sponsor can see and change.
- **Every alert carries `confidence` + `source_url`** (anti-black-box, Iron Law 6).
- **A sensitive alert routes through the HITL matrix from `09-ROI-Governance-And-Risks.md`** — it never auto-fires (Iron Law 9). "Sensitive" = financial action, headcount, customer-facing, regulated.
- **At least one real exposes→consumes rule must be wired and tested** before the federation is called done — a contract on paper that never fires is a silo with extra steps.

This map *is* principle #2 (C-Level interconnection) made concrete. A system of seven isolated dashboards is seven dashboards; this map is what makes it one organism.

The implementation pattern (typed event on the shared Convex bus + subscriber rules) is in `references/03-microsaas-and-inter-dashboard-api.md` §B.

---

## 4. The Composio topology (5.4)

Map the **6 critical connectors** to the dashboards they feed and the system-of-record they read. The realization spec records the topology; the build proves each with a live read (`references/04-composio-integration-and-reports.md`).

| Connector | System-of-record | Feeds dashboard(s) | Data it returns |
|---|---|---|---|
| CRM | HubSpot / Salesforce | CSO, CMO | deals, stages, contacts |
| ERP / Finance | NetSuite / Stripe / QuickBooks | CFO | invoices, cash, margin |
| Marketing | GA4 / Ads / email platform | CMO | spend, channels, conversions |
| Product analytics | PostHog / Amplitude | CIO-CTO, CDO | usage, funnels, data-quality signals |
| HR / HRIS | workforce / ATS | CHRO | headcount, hiring funnel |
| Comms / Ops | Slack / ticketing | COO + report delivery | tickets, SLAs, message routing |

A 7th connector is added only when a live dashboard provably needs it (Iron Law 5). The topology states, per connector, the **system-of-record** (the AI layer reads from there, never from a production DB it does not own) and the **auth method** (OAuth via Composio vs API key).

---

## 5. The server shape (5.1)

Decide and record:
- **Region + data-residency posture.** GDPR → EU region; HIPAA → BAA-covered infra; air-gapped → on-prem swap (Postgres + Docker + private model). The data **stays with the client** (Iron Law 2).
- **The readable, migratable stack** (Next.js + Convex + Clerk + Stripe + Composio + Claude Code SDK) — and the **justification per layer** the sponsor will read (see SKILL.md "the stack" table). Cargo-culting the stack is refused; each choice is argued.
- **The export path.** Exactly how the client takes the keys and leaves: Convex data export, repo ownership, env/secret handover, the runbook to redeploy elsewhere. If you cannot document the export path, you are building a lock-in tenant — refused.

Detail in `references/02-server-and-stack-provisioning.md`.

---

## 6. The instrumentation plan (5.8 + mm-11) and the ship-gate map

### 6a. Baseline instrumentation (the mm-11 slice)

Per dashboard, name the **exactly three events** that will fire at t0 (mm-11 — *three clean events beat a hundred badly named*):
1. **North-Star event** = the architect's success metric for that opportunity (the value delivered — e.g. "executive brief shipped on time", not "dashboard opened").
2. **Cost/usage event** = model cost + tokens + agent run per execution.
3. **Value-delivered event** = the workflow outcome (e.g. "ticket auto-triaged + accepted by human").

The realization spec states where **t0** is captured (go-live) so `caio-run-and-optimize` can later compute the delta vs the architect's projection. You design the instrument; you do not measure the verdict. (Full wiring: `references/05-instrumentation-shipgate-and-sponsor-comms.md` §A.)

> Honesty gate (mm-11): no vanity metric is promoted to a North-Star. "Logins" and "page views" are not the value. The North-Star event is the one that, if it doubled, would mean the client is objectively better off.

### 6b. Ship-gate map

Per deliverable, copy the **acceptance criteria verbatim** from the architect's `07-Dashboard-Feature-Specs.md`. You do **not** invent new acceptance criteria — the architect already wrote them; you enforce them. A deliverable with no acceptance criterion cannot be ship-gated → route back to the architect.

---

## 7. The gate — sponsor approval (mm-04)

The realization spec is reviewed and **approved by the named executive sponsor before the build starts.** Frame the approval as a value proposition the sponsor signs (mm-04 — the Dunford one-liner):

> "For [your C-suite] who [need the company legible *and* automated], this is the [centralized federated Company-AI-OS] that [turns seven blind tools into one organism that alerts itself], unlike [the status-quo of disconnected dashboards] that [keeps each C-Level guessing in their own silo]."

Why a value-prop and not a spec dump (mm-04 — *clear beats clever*): the sponsor approves what they understand. A 40-page technical spec gets rubber-stamped or stalled; a one-line value-prop + a one-page topology diagram + the seat map gets a real decision. Attach the technical detail; **lead with the value-prop.**

Record in `00-Build-Log.md`: the spec version, the sponsor's name, the approval date, and any conditions. **No approval → no build.** If the sponsor wants changes, revise and re-gate — that is the gate working, not a delay.

---

## 8. The realization-spec checklist (before you call it approved)

| Check | Pass = |
|---|---|
| Every built seat exists in the real org (rollup-confirmed) | yes |
| Every built seat has its "real job a generic dashboard gets wrong" line | yes |
| Seats that don't exist are folded into an existing seat (recorded) | yes |
| The federation map has ≥1 real exposes→consumes rule with threshold + payload | yes |
| Every sensitive cross-dashboard alert routes through the HITL matrix | yes |
| The 6 (≤) connectors are mapped to system-of-record + dashboard | yes |
| The server shape states region + residency + the export path | yes |
| Each stack layer carries a one-line justification | yes |
| Per dashboard, the 3 baseline events are named + t0 located (mm-11) | yes |
| Every ship-gate criterion is copied verbatim from the architect's feature spec | yes |
| The spec is sponsor-approved with a date in the build log (mm-04) | yes |

If any check fails, the spec is not approved — fix it. The whole point of the gate is that the build inherits a *decided* architecture, not an improvised one.

---

## Worked micro-example (excerpt of an approved realization spec)

```
Company: B2B SaaS, 120 ppl, EU/GDPR. Seats that exist: CEO, CTO, CMO, CFO, COO, Head-of-People(=CHRO). No CDO, no CSO (founder runs sales).
Build seats: CTO, CMO, CFO, COO, CHRO. CDO data-quality view -> folded into CTO. Revenue view -> folded into CMO (pipeline) + CFO (cash).

Federation map (excerpt):
  COO.support_sla_breach (sev>=high) -> CFO.cash_impact_alert {cost_of_delay, account_arr}  [HITL: CFO review]
  CMO.cac_spike (>1.4x 4wk) -> CFO.burn_alert {channel, delta}  [read-only]

Composio (6): HubSpot(CRM)->CMO; Stripe(ERP)->CFO; GA4(mktg)->CMO; PostHog(analytics)->CTO; ATS(HR)->CHRO; Slack(comms)->COO+report-delivery.

Server: Convex+Clerk, EU region, audit logging on. Export path: Convex export + repo handover + secrets doc (documented in 02-Server-Provisioning-Runbook.md).

Baseline events per dashboard (mm-11): NSM=architect's success metric; cost/usage=model$+tokens+run; value=workflow outcome. t0 = each dashboard's go-live timestamp -> metadata.json.

Ship-gates: copied from 07-Dashboard-Feature-Specs.md (F002 Weekly Exec Brief: "4 consecutive on-time briefs + sponsor approval >=80% without edits"), etc.

APPROVED: 2026-07-03 by COO (exec sponsor). Conditions: CFO dashboard data-residency sign-off from DPO before go-live.
```

This is the contract the build executes. Everything downstream — server, micro-SaaS, federation, integrations, reports, instrumentation, ship-gates — traces back to a line in this approved spec.

---

## 9. Common realization mistakes (catch them before the gate)

- **Designing all seven seats because the offer lists seven.** The offer lists the *pattern*; the org has the *seats*. Building a CDO dashboard for a company with no data org is a template repaint with extra steps. (Iron Law 3.)
- **A federation map with zero tested rows planned.** If the map has no row you can actually fire end-to-end on real data, you designed silos with a contract glued on. Pick the one or two highest-decision-impact edges (operational→cash, demand→capacity) and commit to testing them (reference 03 §B.6).
- **Inventing acceptance criteria.** The ship-gate criteria are the architect's, copied verbatim. If you find yourself writing new ones, the feature was under-specified upstream — route back, do not paper over it.
- **Skipping the export path "to decide later."** The export path is the proof of client ownership (Iron Law 2). A realization spec without it designs a lock-in tenant.
- **Treating the stack as given.** Each layer carries a one-line justification the sponsor's CTO can challenge. "Because the offer says so" is not a justification.
- **Letting the spec become a 40-page document no one reads.** Lead with the value-prop + the one-page topology diagram (the seat map + the federation map); attach the rest. A spec the sponsor does not understand does not get a real approval (mm-04).
- **Designing instrumentation as an afterthought.** The three baseline events per dashboard are decided here, at design time, so the build wires them by construction and t0 exists at go-live. Bolt-on instrumentation = no baseline = no measurable ROI for Phase 5 (mm-11).

If the spec survives this list and the §8 checklist, it is ready for the gate. The gate's job is to make the *architecture* a decision, not an accident — so the build inherits intent, not improvisation.

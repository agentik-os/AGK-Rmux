# CAIO Journey Gates — the gate definition per phase

The sequence law: **qualified → sold → legible → automatable → agentic → adopted → run**. Each phase is a
*gate*: the owner executes it; caio-master proves the deliverable is **present + sufficient + verified**
before the next phase starts. This file is the rubric the Workflow's per-phase gap-check agents lean on.
caio-master ROUTES every phase to its owner and re-implements nothing (Iron Law 1).

For every gate: **Entry** (what must be true to begin), **Owner** (route here — never rebuild), **Gate
deliverable** (the artifact that proves it), **Gap-check questions** (does the artifact exist + pass the
owner's own discipline checks?), **Adversarial lens** (how a skeptic falsifies a premature "done",
R-VERIFY/Popper), and **Route** (the exact owner command).

The seven owner output dirs are `caio-readiness/`, `business-os/`, the per-person discovery ZIPs,
`company-ai-os/`, `caio-build/`, `caio-enablement/`, `caio-run/`. caio-master itself writes only its own
dedicated `caio-engagement/` dir — it never writes into an owner's dir.

---

## P0 — Readiness (pre-sign go/no-go)

- **Entry:** a prospect company + an executive willing to take a 30-min qualification call. (Cold start —
  no upstream gate; this is the front gate of the whole engagement.)
- **Owner:** `caio-ai-readiness-assessment` — scores the company against a 9-dimension AI-Readiness
  Maturity Model (0-4 per dimension with evidence), computes a weighted Readiness Index + tier, runs the
  5 hard gates (sponsorship / API-exposure / use-case / compliance / commitment) and the mm-03 4-forces
  read, and returns an honest **GO / NOT-YET / REDIRECT** verdict + indicative investment (the real grid).
- **Gate deliverable:** `./caio-readiness/` — `Go-No-Go-Brief.md` (the 1-page verdict), `AI-Readiness-Scorecard.md`
  (9 dims scored with evidence + Index + tier), `Recommended-Engagement.md` (shape + indicative investment),
  `Gap-To-Target-Plan.md`, `metadata.json`. **The gate opens ONLY on GO.**
- **Gap-check questions:** Is the verdict an explicit **GO** (all 5 hard gates pass, 4-forces net-positive,
  ≥1 beachhead use case)? Is every dimension level cited (call quote / site-scan), not vibes? Is the
  indicative investment computed from the grid (not improvised)? Is there an explicit handoff to
  `/market-proposal`?
- **Adversarial lens:** Did the gate say GO to everyone (a sales pitch in a lab coat)? Is the sponsor named
  AND budget-holding (G1), or a hope? Does a core tool actually expose an API / a documented path (G2)? Is
  any **return** number invented (forbidden here — only *investment* is grid-anchored)?
- **Route:** `caio-ai-readiness-assessment`. On **GO** → `/market-proposal` then proceed to P1. On
  **NOT-YET** → back to the company with the Gap-To-Target plan (re-qualify in 30-90 days). On **REDIRECT**
  → the named alternative (a point SaaS / a data engineer / an internal hire / a compliance partner).
  **A NOT-YET/REDIRECT HALTS the engagement** — caio-master records the verdict + the path back and does
  not route forward (there is nothing sold to route). caio-master does **not** re-derive the scorecard —
  it reads the verdict (Iron Law 1).

## P1 — Offer / Scope / Price / Sell

- **Entry:** P0 `SUFFICIENT-VERIFIED` (a **GO** verdict + a recommended engagement shape).
- **Owner:** `offer-and-revenue-architect` (offer + pricing + sell-sheet + unit-economics → `./business-os/`)
  then `market-proposal` (client-ready proposal, exec summary, situation analysis, phased SOW,
  **Good-Better-Best** pricing tiers, ROI projection, objection-handling). Doctrine lenses for the gap-check:
  `mm-04` (messaging/offer), `mm-08` (pricing/monetization), `mm-10` (selling).
- **Gate deliverable:** `./business-os/Offer-Architecture.md` + `./business-os/Pricing-Model.md` + a
  client-ready proposal carrying GBB tiers and an ROI projection + an **accepted/signed scope** + a **named
  executive sponsor** (the engagement is actually sold, not just drafted).
- **Gap-check questions:** Is there ONE locked offer with a locked price? Are the GBB tiers present and
  internally coherent? Is the ROI projection grounded (value-anchored), not a guess? Did the client accept,
  and is a sponsor named?
- **Adversarial lens:** Is the ROI invented or modeled? Is the price anchored to value or pulled from air?
  Is "accepted" evidenced (a signed SOW / written yes) or assumed? (The readiness skill already produced
  the *indicative* investment — confirm the proposal is consistent with it, not improvised.)
- **Route:** `offer-and-revenue-architect` → `market-proposal`. caio-master does **not** write a new offer
  engine (it exists — Iron Law 1).

## P2 — Discovery

- **Entry:** P1 `SUFFICIENT-VERIFIED` (scope accepted, stakeholder list agreed).
- **Owner:** `caio-discovery-interview` — a guided, role-adaptive interview with ONE employee at a time,
  exporting one standardized ZIP of `.md` files per person (identical structure for every interviewee).
- **Gate deliverable:** one standardized dossier/ZIP **per stakeholder** (18 files + `metadata.json` each),
  with verbatim role/week/month capture, handoffs, tools, shadow IT, frictions, current-vs-ideal, and the
  `ai_appetite` (champion/neutral/skeptic) index — coverage matching the audit mode chosen in P3 (a
  `full-company-workflow-audit` needs ≥10).
- **Gap-check questions:** Is there a dossier per named stakeholder? Is each a real interview (verbatim),
  not a summary? Is consent + anonymization handled? Is a critical department missing?
- **Adversarial lens:** Are dossiers real or back-filled? Does coverage actually support the downstream
  audit mode, or will the architect be starved?
- **Route:** `caio-discovery-interview`, looped over each stakeholder. caio-master does **not** run the
  interview itself, nor the architect's intra-audit interview fan-out (that is P3's altitude).

## P3 — Diagnose + Architect + Roadmap

- **Entry:** P2 `SUFFICIENT-VERIFIED` (enough dossiers for the chosen mode).
- **Owner:** `caio-enterprise-workflow-architect` — its 5 modes / 10 phases own the diagnostic, the
  10-criteria opportunity scoring, the architecture (Company AI OS), the 30/60/90 roadmap, the per-workflow
  ROI engine, governance, and the executive business case. **This phase is BROAD** — the architect already
  authors (a) a change-management + training PLAN (anchored in Kotter/ADKAR/Prosci), (b) a per-workflow ROI
  engine + exec summary, (c) a continuous run/governance cadence, and (d) the **feature specs**
  (`07-Dashboard-Feature-Specs` + optional `features/F-XXX-*.md`) the build phase consumes. caio-master
  **consumes** all of these as inputs to P4/P5/P6 — it never re-authors them.
- **Gate deliverable:** `./company-ai-os/` with at minimum `00-Executive-Summary.md`,
  `05-Automation-Opportunity-Backlog.md` (**scored**, with Class-8 REFUSED documented),
  `06-Agentic-System-Blueprints.md`, `07-Dashboard-Feature-Specs.md` (acceptance criteria — the source of
  every build ship-gate), `08-Implementation-Roadmap.md` (30/60/90 + cost + payback),
  `09-ROI-Governance-And-Risks.md`.
- **Gap-check questions:** Did the architect's diagnostic actually RUN? Is the backlog scored on real
  receipts (`hours × loaded-cost × frequency`)? For a "full audit", were ≥10 verbatim interviews used? Are
  Class-8 REFUSED items present (not omitted)? Do the feature specs carry acceptance criteria? Is the
  roadmap CFO-credible (cost + ROI + payback per phase)?
- **Adversarial lens:** Is the ROI math grounded or fabricated? Did a "full audit" ship on <10 interviews
  (the architect's own refusal)? Is any sensitive HR/legal/financial decision wrongly classed as an agent?
- **Route:** `caio-enterprise-workflow-architect` (mode per scope). caio-master **NEVER re-derives
  AI-readiness or re-scores opportunities** — it reads the architect's output as the receipt.

## P4 — Build (the critical seam)

- **Entry:** P3 `SUFFICIENT-VERIFIED` (a scored backlog + blueprints + feature specs with acceptance
  criteria exist). **This is the seam the suite design fixed** — without it, "diagnose/architect" dead-ends
  at a blueprint.
- **Owner:** `caio-implementation-runbook` — takes the architect's generic `company-ai-os/` and (1)
  **REALIZES** it into the offer's centralized federated topology (one client-owned server + one
  micro-SaaS per C-Level that actually exists + the inter-dashboard API contract + the Composio map) as a
  **sponsor-approved design gate**, then (2) **BUILDS** it operationally (provision → build each micro-SaaS
  → wire the federation → wire the 6 critical connectors → automated reports → monitoring/instrumentation
  baseline → per-deliverable ship-gate), keeping a build log the whole way. It **delegates** per-agent
  builds to `agentic-systems-builder` (per `F-XXX`) and repeatable skills to `agentik-skill-forge`, and it
  runs `/omg-acceptance` as its **ship-gate** — all internal to the runbook, not a caio-master route.
- **Gate deliverable:** `./caio-build/` — `01-Architecture-Realization-Spec.md` (**sponsor-approved**, the
  design gate), the dedicated **client-owned** server standing + migratable, each micro-SaaS shipped on a
  **green ship-gate** (acceptance walked the authenticated golden path with a **real persisted write** on
  **real data**) reaching a **LIVE prod URL**, ≥1 inter-dashboard alert wired + tested, the ≤6 connectors
  proven by a live read, the baseline events (NSM, cost/usage, value) firing at **t0**, the
  `08-Ship-Gate-Ledger.md` recording every verdict with evidence.
- **Gap-check questions:** Was the realization spec **approved before** the build (the design gate, Iron
  Law 1 of the runbook)? Is there a live URL (not localhost)? Did the ship-gate actually walk the
  authenticated golden path with a real write, or just check HTTP 200? Are real integrations/creds wired,
  or stubs? Is every sensitive cross-dashboard alert behind HITL? Is the server client-owned + exportable?
- **Adversarial lens:** "It builds" ≠ "it works" (L1). Is the ship-gate log real and green on **real**
  (not seeded) data, or is "the build passed" being read as "prod works"? A green build with a red console
  is **not** shipped (R-PROD). Is the federation real (an alert actually firing), or seven silos?
- **Route:** `caio-implementation-runbook` (mode per scope: `realize-architecture` → `provision-server` →
  `build-microsaas` → `wire-federation` → `wire-integrations` → `full-build`). caio-master **never re-runs
  the browser sweep inline** — it checks the ship-gate ledger + a live URL and routes (no double-route,
  Iron Law 3). The runbook's own delegation (to `agentic-systems-builder` / its `/omg-acceptance`
  ship-gate) is a downstream hint, not a second route.

## P5 — Enable + Transfer

- **Entry:** P4 `SUFFICIENT-VERIFIED` (a live, ship-gate-verified system the team can actually adopt).
- **Owner:** `caio-enablement-and-transfer` — **Phase 3 (Adoption):** onboard every role, train end-users,
  validate first use cases in real conditions; **Phase 4 (Transfer):** teach the client's team to
  add-an-agent / connect-a-tool / adjust-a-report **unaided**, then issue the **Autonomy-Readiness Gate**.
  It **executes** the architect's change-management + training plan; it does not re-author it.
- **Gate deliverable:** adoption **measured** (`08-Adoption-Tracker.md` — a value-received NSM + cohort
  retention not collapsing), the **Autonomy-Readiness Gate** passed (`07-Autonomy-Readiness-Gate.md` — the
  3 motions performed unaided under real conditions + **zero CAIO-only credentials** + named owners per
  component), the internal documentation pack written, a `04-Validated-Use-Cases-Log.md` filled, and a
  signed **ownership handover** (`06-Ownership-Handover-Checklist.md`).
- **Gap-check questions:** Was training **run** (evidence of sessions, not a plan)? Is adoption a
  *retention* metric (used weeks later) or attendance? Did named client owners complete all three motions
  **unaided**? Are runbooks owned by a named person? Did the handoff actually happen with no CAIO-only keys?
- **Adversarial lens:** The architect *planned* change-mgmt — was it *executed*? Is "trained 40 people" a
  vanity metric? Is a passed quiz being read as Ability (Knowledge ≠ Ability, ADKAR)? Is there a bus factor
  of one? Is HITL preserved on sensitive decisions after transfer?
- **Route:** `caio-enablement-and-transfer`. caio-master **consumes** the architect's change-mgmt plan as
  input — it does not re-author it (Iron Law 1).

## P6 — Run + Optimize

- **Entry:** P5 `SUFFICIENT-VERIFIED` (the client team owns the live, instrumented OS).
- **Owner:** `caio-run-and-optimize` — measure **actual** post-go-live ROI vs the architect's projection
  (from telemetry + receipts, never invented), monitor health with alert thresholds, run the weekly/monthly
  optimization loop, operate the deliberately-light **1h/week** strategic quota, and drive client retention
  + land-and-expand to the next department.
- **Gate deliverable:** `./caio-run/` — `ROI-Measurement-Model.md` (actual **vs the architect's 09-ROI**,
  **by cohort**, every number cited, proven/partial/falsified verdicts), `Monitoring-Health-Spec.md`
  (liveness/cost/usage/quality/value, each with a **threshold + owner**), `Optimization-Loop-Cadence.md`,
  `Weekly-Quota-Agenda.md` (with the overage-is-a-mini-SOW boundary), `Quarterly-Business-Review.md`
  (driving a renew/expand decision), `Expansion-And-Referral-Play.md`, `metadata.json`.
- **Gap-check questions:** Is the operating dashboard real and watched (thresholds + owners)? Is ROI
  measured **by cohort** (not a single average that hides a leaking workflow)? Is every figure traced to
  telemetry/a receipt? Did the QBR drive a decision? Did a second wave (the **Expand** verdict) re-enter
  the architect?
- **Adversarial lens:** Is the dashboard a screenshot or a live, watched surface? Is a falsification rounded
  up into "proven"? Is the next department being proposed while a wave-1 cohort decays (retention before
  acquisition)? Did continuous improvement actually run, or did the OS freeze at launch and quietly decay?
- **Route:** `caio-run-and-optimize`. An **"Expand"** verdict re-enters `caio-enterprise-workflow-architect`
  (**P3**) for the next-wave audit — the chain closing into a compounding loop. caio-master **consumes** the
  architect's Phase-5 governance/monitoring cadence as the run baseline — it does not re-author it (Iron Law 1).

---

## Gate status machine (how caio-master sets each phase's status)

```
BLOCKED-UPSTREAM      :: the predecessor gate is not SUFFICIENT-VERIFIED -> this phase cannot start
READY                 :: predecessor verified AND this phase's artifact is absent -> route to the owner now
IN-PROGRESS           :: the artifact exists but is partial / fails an owner discipline check
SUFFICIENT-VERIFIED   :: artifact present + passes the owner's discipline checks + >=2-of-3 skeptics ratify
```

The gates are strictly sequential: a phase is **never** `SUFFICIENT-VERIFIED` while any predecessor is
open. The next OPEN gate (first `READY` or `IN-PROGRESS`) is the operator's next action, named with its
exact owner route in `00-CAIO-Engagement-Plan.md`.

**P0 exception (the kill-switch).** P0 is `SUFFICIENT-VERIFIED` only on an explicit **GO** verdict. A
recorded **NOT-YET** or **REDIRECT** keeps P0 not-verified and **halts** the journey — every downstream
phase stays `BLOCKED-UPSTREAM`, and the P0 route is the readiness skill's *path back* (the Gap-To-Target
plan on NOT-YET, the named alternative on REDIRECT), never "re-run until it says GO" (that would be the
sales-pitch-in-a-lab-coat the readiness gate exists to refuse).

# Reference 02 — Server & Stack Provisioning (5.1)

> The offer's 5.1: **one dedicated, centralized server**. Its data **stays with the client**. The stack is **readable and migratable**. This reference is the runbook to stand it up — and the justification of every stack choice so the CAIO can defend it to a CTO, a CFO, and a security officer in the same room.

> The client owns the server and the data. If the client cannot take the keys and leave, you built the wrong thing. (Iron Law 2.)

---

## 0. What "dedicated centralized server" means in this offer

It does **not** mean a multi-tenant SaaS the CAIO controls and rents back to the client. It means **one deployment the client owns**, that hosts all seven micro-SaaS, the shared event bus (the federation), the integration layer, the automated-report engine, and the instrumentation — in one place, in the client's region, under the client's keys.

"Centralized" is about the **data and the event bus**: every C-Level dashboard reads from and writes to the same Convex deployment, which is what makes the inter-dashboard federation (5.3) cheap and reactive. "Dedicated" is about **ownership**: it is the client's account, the client's repo, the client's secrets.

The anti-pattern this kills: seven disconnected tools, each a separate vendor login, none owned, none talking. That is the status quo the offer replaces.

---

## 1. The stack — and why each layer earns its place

You justify each choice; you do not cargo-cult the default. The realization spec (`01-architecture-realization.md` §5) records the justification; this is the canonical argument.

### Frontend — Next.js (App Router)

- **Why:** one framework renders all seven dashboards; server components keep secrets and integration tokens server-side (never shipped to the browser); the App Router's route groups map cleanly to per-seat areas behind Clerk RBAC.
- **Migratable:** standard React/TypeScript the client's own engineers can read and extend. No proprietary DSL.
- **Defend to the CTO:** "Your team already knows React. When we leave, they keep building."

### Backend / DB — Convex

- **Why (the load-bearing reason):** Convex is **reactive and type-safe**, which makes the **inter-dashboard federation natural** — a write on the COO dashboard reactively fires a query/subscription on the CFO dashboard with no message-queue plumbing. The shared event bus (5.3) *is* a Convex table + reactive queries. Doing this on a request/response REST stack would mean building (and the client maintaining) a fragile webhook web.
- **Owned:** one Convex deployment in the client's account. Data export is a first-class operation (see §5).
- **Defend to the CFO:** "One backend, one bill, real-time, and your data exports in one command."

### Auth / RBAC — Clerk

- **Why:** per-C-Level access control out of the box; one identity layer across all seven micro-SaaS; the **HITL approval roles** from the architect's matrix map directly to Clerk roles/permissions, so a sensitive cross-dashboard alert (5.3) can require a named role to approve.
- **Defend to security:** "RBAC and audit-able sessions from day one, not bolted on."

### Billing — Stripe

- **Why:** if a micro-SaaS is monetized (an internal chargeback model — each department billed for its AI spend — or an externally-sold product), Stripe is the metering layer. Even when nothing is "sold," Stripe usage records carry **internal cost allocation** so the CFO dashboard can attribute model spend to a department.
- **Honest note:** if there is no chargeback and no external sale, Stripe is optional — do not install ceremony the client does not need. Record the decision in the runbook.

### Integrations — Composio

- **Why:** one auth + action layer for CRM/ERP/marketing/analytics/HR/comms; managed OAuth token refresh and rate-limit handling so each dashboard does not hand-roll six API clients. The **6-critical-connectors rule** (5.4) keeps it from becoming a 200-connector swamp.
- **Defend to the CTO:** "Six connectors, each proven by a live read, managed token refresh — not six brittle integrations your team babysits." (Detail: `references/04-composio-integration-and-reports.md`.)

### Agent runtime — Claude Code SDK

- **Why:** the agents that power each dashboard (the architect's `F-XXX` blueprints) run on the Claude Code SDK — tool-use, structured output, file/work execution, and the anti-black-box logging the offer requires (every run exposes its tools, tokens, cost, and confidence). The per-agent build is **delegated to `agentic-systems-builder`** (Iron Law 10); this skill wires the runtime, it does not re-implement the builder.
- **Defend to the board:** "Every agent run is logged with its cost and confidence — no black box."

### The honest table (paste into the runbook)

| Layer | Choice | One-line justification |
|---|---|---|
| Frontend | Next.js (App Router) | seven dashboards, server-side secrets, client-readable React |
| Backend / DB | Convex | reactive event bus = the federation; client-owned; one-command export |
| Auth / RBAC | Clerk | per-seat access + HITL roles map to permissions |
| Billing | Stripe | usage metering / internal chargeback (optional if neither applies) |
| Integrations | Composio | one auth layer; 6-critical-connectors; managed token refresh |
| Agent runtime | Claude Code SDK | tool-use agents with logged cost + confidence (delegated builds) |

---

## 2. Adaptation rules (the stack is a starting point, not a religion)

| Constraint | Adaptation |
|---|---|
| Existing ERP/CRM (SAP, Oracle, NetSuite, Salesforce) | Keep as the **system of record**; the AI layer **reads from it** via Composio. Never duplicate the source of truth. |
| Existing data warehouse (Snowflake, BigQuery, Databricks) | Dashboards read from the warehouse, **not** from production systems. |
| GDPR / data residency | Convex deployment + any model endpoint in the required region; DPA in place; PII minimization in the event payloads. |
| SOC2 / HIPAA | Audit logging on; encryption-at-rest verified; vendor SOC2/BAA reports checked; the data-cleanup backlog from the architect respected. |
| Air-gapped / private cloud | Swap Convex+Vercel for **on-prem Postgres + Docker + a private model endpoint**, keeping the *same federation contract* (the contract is stack-agnostic). |

Record the adaptation in the runbook. A stack recommendation that ignores the client's regulatory constraints is refused (architect Iron Law, enforced here).

---

## 3. Provisioning runbook (the actual steps)

> This is the operational sequence. Adapt the exact commands to the chosen region/account. Each step ends in a **verifiable** state, not a "should be up."

```
STEP 0  — Pre-flight
  - Realization spec APPROVED (Iron Law 1). Region + residency decided.
  - Client owns the accounts (Convex, Clerk, Stripe, Composio, Vercel) — created UNDER THE CLIENT'S org,
    not the CAIO's. Secrets go to the client's vault (~/.omega or the client's secret manager), never the repo.

STEP 1  — Repo + scaffold
  - Create the client-owned repo. Scaffold Next.js (App Router) + Convex + Clerk.
  - Route groups per seat: app/(cio-cto), app/(cmo), app/(cfo), app/(coo), app/(chro), app/(cso) — only seats that exist.
  - Commit. (OmegaOS users: /omega-new-project can scaffold + auto-provision the external services.)

STEP 2  — Convex deployment (the centralized data + event bus)
  - Deploy Convex in the client's account, correct region.
  - Define the core schema (see §4): organizations, seats, metrics, alerts (the federation bus),
    integrations, reports, agentRuns, costEvents, baselineEvents, auditLogs.
  - VERIFY: `npx convex deploy` succeeds; the dashboard reads an empty but typed schema.

STEP 3  — Clerk (RBAC + HITL roles)
  - Configure orgs + roles: one role per C-Level seat + the HITL approver roles from the architect's matrix.
  - Gate each route group by role.
  - VERIFY: a CFO-role user can reach (cfo) and is 403'd from (chro) admin actions.

STEP 4  — Secrets + env
  - All integration tokens + model keys live in the client's secret manager; injected at deploy.
  - VERIFY: no secret is tracked in git (grep the repo); the app boots with secrets from env only.

STEP 5  — Deploy the shell
  - Deploy to the client's Vercel (or on-prem target). --token explicit on every deploy (R-VERCEL).
  - VERIFY: the authenticated shell loads, Clerk login works, Convex connects. HTTP 200 on each seat route.

STEP 6  — Observability skeleton (5.8) BEFORE features
  - Stand up the logs/cost/usage tables + the monitoring view (references/05 §A) so every feature built
    next is instrumented by construction, not bolted on (Iron Law 6).
  - VERIFY: a test agent run writes a row to agentRuns + costEvents and shows in the monitoring view.

STEP 7  — Provision acceptance (the server ship-gate)
  - Run the provision acceptance checklist (§6). Server is "provisioned" only when it passes.
  - Record in 02-Server-Provisioning-Runbook.md + 00-Build-Log.md (date, region, accounts, export-path link).
```

Do **not** start STEP 1 before the realization spec is approved. Do **not** build a micro-SaaS (Phase 3) before STEP 6 — instrumentation precedes features so nothing ships uninstrumented.

---

## 4. The Convex schema (centralized + federation-ready starting point)

```
organizations        { name, region, residency, plan }
seats                { seatKey: "cfo"|"coo"|..., exists, absorbedBy?, realJobLine }
users                (Clerk)        { clerkId, seatKey, role, hitlRoles[] }
metrics              { seatKey, metricId, value, ts, confidence, sourceUrl }   // each dashboard EXPOSES here
alerts               { fromSeat, toSeat, alertId, payload, threshold, hitlRequired, status, ts }  // 5.3 bus
integrations         { connector, systemOfRecord, authStatus, lastLiveRead, rateRemaining }       // 5.4
reports              { reportId, seatKey, frequency, format, recipients[], lastRunAt, status }     // 5.5
agentRuns            { agentId, seatKey, tool, status, startedAt, endedAt, error?, confidence }    // 5.8 logs
costEvents           { agentRunId, model, tokensIn, tokensOut, costUsd, ts }                       // 5.8 cost
baselineEvents       { seatKey, eventType: "nsm"|"usage"|"value", value, ts, t0 }                  // mm-11 baseline
approvals            { alertId|reportId, approverRole, decision, ts }                              // HITL
auditLogs            { actor, action, target, ts }                                                 // governance
```

Every entity carries `createdAt, updatedAt, orgId`. `metrics` + `alerts` are the federation bus: a write to `metrics` reactively evaluates subscriber rules that write to `alerts`. This is why Convex is the load-bearing choice (§1). Adapt per client; this is a starting point, not a religion.

---

## 5. The export path (the ownership proof)

The client must be able to **take the keys and leave**. The runbook documents, concretely:

1. **Data export** — `npx convex export` produces a full dump; schedule it (Convex Scheduler) to a client-owned bucket so an export exists even if the relationship ends.
2. **Repo ownership** — the repo lives in the client's GitHub org; the CAIO is a collaborator, not the owner.
3. **Secrets handover** — a documented list (names, where they live, how to rotate) in the client's secret manager.
4. **Redeploy runbook** — the exact steps to redeploy the system to a different host (Vercel → self-host, Convex → self-host Convex/Postgres), proving migratability.

> If you cannot write the export path, you are building a lock-in tenant — **refused** (Iron Law 2). The export path is part of the *provision acceptance* (§6).

---

## 6. Provision acceptance (the server ship-gate)

The server is "provisioned" only when **all** pass (evidence cited — R-CITE):

| Check | Pass = |
|---|---|
| Convex deployed in the correct region; schema typed; export tested | yes (export file produced) |
| Clerk RBAC enforces per-seat access + HITL roles | yes (403 on cross-seat admin) |
| No secret tracked in git; app boots from env-injected secrets only | yes (grep clean) |
| Authenticated shell loads; HTTP 200 on every existing-seat route; clean console | yes (L1 — runtime) |
| Observability skeleton writes agentRuns + costEvents (instrumentation before features) | yes (test row shown) |
| Accounts are under the CLIENT's org (Convex/Clerk/Stripe/Composio/Vercel) | yes |
| Export path documented (data + repo + secrets + redeploy) | yes |
| Region + residency match the regulatory constraint | yes |

Record the verdict + evidence in `08-Ship-Gate-Ledger.md`. A failed provision acceptance blocks Phase 3 — you do not build dashboards on a server that is not really standing (L1).

---

## 7. Common provisioning traps

- **Building features before the observability skeleton.** Then instrumentation is "added later" = never. STEP 6 precedes Phase 3. (Iron Law 6.)
- **Accounts under the CAIO's org "for now."** "For now" becomes lock-in. Provision under the client from STEP 0.
- **Skipping the export test.** An untested export is a promise, not a proof. Run `convex export` during provisioning.
- **One Convex deployment per dashboard.** That breaks the federation (the event bus must be shared). One centralized deployment, route-grouped — not seven.
- **Secrets in `.env` committed "temporarily."** Never. Client secret manager from STEP 4. (R-ENV / L0.)
- **Ignoring residency until go-live.** A GDPR client cannot have data land in us-east "during the build." Region is a STEP 2 decision.

---

## 8. Sizing + cost notes (so the CFO is not surprised)

The offer instruments model cost (5.8) precisely so spend is never a surprise; the same honesty applies to the server itself. Record in the runbook, per the client's scale:

- **Convex.** One deployment for all seats. The cost driver is function calls + bandwidth + stored data, not "number of dashboards." A reactive federation (the event bus) adds query volume — budget for it; it is cheaper than the message-queue infra the alternative would need. Start on the tier matched to the org's event volume; the monitoring view (reference 05 §B) shows real usage so you right-size after week 1, not by guessing.
- **Clerk.** Priced per monthly active user — and the users here are the C-suite + the HITL approvers, a small set. This is not a consumer app; MAU stays low. Note it so the CFO does not model it like a public product.
- **Composio.** Priced on connected accounts + action volume. The 6-critical-connectors rule (reference 04) keeps this bounded by design — another reason the rule is not just hygiene but cost control.
- **Model spend (the real variable).** The agents (Claude Code SDK) are the largest variable cost. The `costEvents` table + the CFO/CIO cost panel make it a visible line item from t0. Set a budget threshold in the governance view; wire a CFO `budget_threshold` alert (reference 03 §B.6) so an overrun surfaces *before* the invoice, not after.
- **Vercel / hosting.** Standard; for an air-gapped client this becomes the on-prem compute line instead.

The point is not a precise quote — it is that **every cost line is visible and attributable** before go-live, consistent with the anti-black-box law. A CFO who can see model spend per workflow trusts the system; one surprised by a bill does not. This visible-cost posture is also what lets `caio-run-and-optimize` later compute ROI as (value delivered − total cost) against the architect's projection — the cost half of that equation is wired here.

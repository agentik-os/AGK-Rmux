# 01 Required Inputs

Phase 1 of the CAIO Enterprise Workflow Architect. Outputs the engagement contract the skill uses across phases 2-9.

The 80% beginner mistake: opening an engagement without naming the mode, the scope, the regulatory constraints, the executive sponsor, and the success criteria. This reference closes that gap.

---

## 1. The Composability Scan (BEFORE Phase 1)

Read CAIO's own context if present:

| Artefact | Use |
|---|---|
| `life-atlas/01-identity/` | CAIO's values + bias (e.g., builder-leaning vs consultant-leaning) |
| `personal-os/Manifesto.md` | CAIO's positioning (e.g., "trust-driven CAIO" vs "vendor-aligned CAIO") |
| `vision-os/Vision-Thesis.md` + `vision-os/Refuse-to-Build-List` | Refuses to recommend stacks/vendors that contradict CAIO's worldview |

If all 3 absent: proceed cold. The skill works for first-engagement CAIOs without prior personal-OS.

Read CLIENT documents if provided (CAIO uploads or links):
- Org chart (PDF, image, or list of roles)
- SOPs (Standard Operating Procedures)
- Existing Notion / Confluence / Google Drive workspace excerpts
- CRM exports (HubSpot, Salesforce schema)
- Process maps (BPMN, Lucidchart, Miro)
- Existing automation list (Zapier, Make, n8n, Pipedream)
- AI usage today (which teams use which models, what for)

Source-of-truth rule: where docs disagree with interviews, INTERVIEWS WIN. The map is not the territory.

---

## 2. The Engagement Mode (5 options)

Mandatory. Picked at Boot Sequence step 3.

### Mode A :: quick-executive-audit

```
Duration  : 90 min single session
Audience  : C-Level only (CEO + COO + CTO + CFO + CAIO sponsor)
Interviews: 5-8 (C-Level only)
Output    : Executive-Summary (1 page) + 5-7 prioritized opportunities + 1 recommended next-step engagement
Use when  : board needs a 90-min read on AI readiness, OR before signing a longer engagement, OR a fractional CAIO is interviewing the client
```

### Mode B :: department-discovery

```
Duration  : 1-2 weeks
Audience  : 1 department + adjacent stakeholders
Interviews: 3-7
Output    : Department workflow inventory + 10-20 opportunities + 3 feature specs + small department roadmap
Use when  : the company wants to pilot AI in 1 dept (often: Support, Sales, Finance) before going wider
```

### Mode C :: full-company-workflow-audit

```
Duration  : 4-12 weeks
Audience  : All departments + IT/security + legal/compliance + field/operations if applicable
Interviews: 10-40 (minimum 10 verbatim)
Output    : Complete company-ai-os/ (all 10 files) + 10-20 feature specs + multi-department roadmap
Use when  : the company has executive commitment + budget for a real CAIO function + intent to ship multiple agents
```

### Mode D :: dashboard-architecture

```
Duration  : 1-3 weeks
Audience  : product + tech + CAIO + 1-2 executive sponsors
Interviews: focused on dashboard users only (5-10)
Output    : Feature specs + Convex schema + Next.js screen list + React Flow node types + UX wireframes described
Use when  : the audit already exists OR the client wants to build the AI dashboard product first then audit later
```

### Mode E :: implementation-roadmap

```
Duration  : 1 week (post-audit)
Audience  : CAIO + tech lead + executive sponsor + finance
Interviews: re-validate priorities with 3-5 stakeholders
Output    : 30/60/90 roadmap + stack decisions + team composition + cost model + ROI projections + risk register + governance plan
Use when  : audit is complete + client wants to move to building
```

The skill REFUSES to skip modes. A "full audit" needs 10+ interviews. A "department-discovery" needs at least 3.

---

## 3. The Company Context Question (mandatory intake)

Captured at Boot Sequence step 4. Goes into `00-Executive-Summary.md` §Company Context.

### A. Size + structure
```
Number of employees (1-10 / 11-50 / 51-200 / 201-1000 / 1000+):
Geographic distribution (single-office / multi-city / remote-distributed / hybrid):
Departments (list):
Executive structure (founder-CEO / CEO + COO / full C-suite / decentralized):
```

### B. Industry + regulatory constraints
```
Industry (SaaS / e-commerce / health / fintech / legal / industrial / education / public sector / NGO / other):
Customer types (B2B / B2C / B2B2C / mixed):
Geography of customers (single-country / multi-country / global):
Regulatory constraints (check all that apply):
  - GDPR (EU)
  - CCPA / CPRA (California)
  - HIPAA (US health)
  - SOC2 (security audit certified or in-progress)
  - ISO 27001
  - FINRA / SEC (US financial)
  - MiFID II (EU financial)
  - PCI-DSS (payment)
  - PSD2 (EU payment)
  - EU AI Act (any AI system + EU market)
  - Other (specify)
Data residency requirements (any country MUST stay in / never leave):
```

### C. Current stack (be specific, name versions / vendors)
```
CRM             : ___ (HubSpot, Salesforce, Pipedrive, Close, Attio, ...)
Support         : ___ (Intercom, Zendesk, Front, Help Scout, ...)
Communications  : ___ (Slack, Teams, Discord, Mattermost, ...)
Documents       : ___ (Notion, Confluence, Google Workspace, Microsoft 365, SharePoint, ...)
Project mgmt    : ___ (Linear, Jira, Asana, ClickUp, Monday, ...)
Code repos      : ___ (GitHub, GitLab, Bitbucket)
Finance         : ___ (Stripe, Pennylane, QuickBooks, Xero, NetSuite, SAP, ...)
HR              : ___ (Lucca, BambooHR, Personio, Workday, ...)
Product analytics: ___ (PostHog, Amplitude, Mixpanel, Heap, ...)
Data warehouse  : ___ (Snowflake, BigQuery, Databricks, Redshift, none)
Existing AI     : ___ (Claude / ChatGPT / Gemini subscriptions, OpenAI / Anthropic API, custom agents, none)
Existing automations: ___ (Zapier, Make, n8n, Pipedream, Trigger.dev, internal scripts, none)
Auth            : ___ (Clerk, Auth0, Okta, internal, Google Workspace SSO, ...)
Hosting         : ___ (Vercel, AWS, GCP, Azure, on-prem, hybrid)
```

### D. Main business objective for AI (pick 1 dominant, 1-2 secondary)
```
:: save time (reduce manual hours)
:: reduce cost (lower headcount / vendor / process cost)
:: increase revenue (lift conversion / upsell / retention)
:: improve quality (lower error rate, better outputs)
:: centralize operations (single source of truth, less tool sprawl)
:: support teams (give employees a smarter assistant, not replace them)
:: build new product (AI as a feature of the company's offering, customer-facing)
:: build internal agentic OS (long-term: company runs on supervised agents)
```

The dominant objective DRIVES the opportunity scoring weights in Phase 6.

### E. Constraint snapshot
```
Timeline before first ship (30 / 60 / 90 / 180 days):
Budget (under $50k / $50-150k / $150-500k / $500k+):
Executive sponsor (name + role + how often they review):
IT / security veto power (yes / no / case-by-case):
Existing AI champions internally (who, what teams):
Adoption resistance expected (low / medium / high / mixed):
Public-facing or internal-only AI surface (internal / customer-facing / both):
```

### F. Success criteria (named upfront)
```
What does "this audit was worth it" look like at day 90?
What does failure look like at day 90?
What does success look like at day 365?
```

If the CAIO cannot answer §F = pause + clarify with executive sponsor before proceeding. An audit without success criteria = a deck nobody acts on.

---

## 4. The Source Document Ingestion (Phase 2 prep)

Before stakeholder interviews, the skill asks the CAIO to provide:

| Document | Why useful |
|---|---|
| Org chart | Map roles + reporting lines before interviewing |
| Existing SOPs (per department) | Compare documented process vs interview-described real process |
| Notion / Confluence workspace inventory (top-level structure) | Identify knowledge sources for RAG |
| CRM schema (HubSpot custom properties, Salesforce objects) | Understand sales data model |
| Support ticket categories + macros | Understand support taxonomy |
| Existing Zapier / Make automation list | Avoid recommending automation that already exists |
| Past AI experiments (what was tried, what failed) | Avoid repeating dead-ends |
| Recent strategy / OKR document | Align AI roadmap with business priorities |
| Executive sponsor's 1-pager on "what they want from AI" | Source of truth for success criteria |

If documents are not provided, the skill works from interviews alone. But documents accelerate Phase 4 (workflow mapping) and Phase 5 (tool inventory) by 30-50%.

---

## 5. Required Inputs Output (`01-Stakeholder-Interview-Plan.md` header)

The output of Phase 1 lands in `00-Executive-Summary.md §Company Context` AND in the header of `01-Stakeholder-Interview-Plan.md`:

```
# Stakeholder Interview Plan

## Engagement Mode
[A/B/C/D/E from §2]

## Company Context
- Size: ___
- Industry: ___
- Regulatory constraints: ___
- Current stack (with vendor names): ___
- Dominant business objective: ___
- Secondary objectives: ___

## Timeline + Budget + Sponsor
[from §3E]

## Success Criteria
[from §3F]

## Departments To Interview
[from §3A, prioritized by §3D objective relevance]

## Roles To Interview Per Department
[per next reference, §02 stakeholder protocol]

## Interview Order
[recommended order: C-Level -> Department Heads -> Field Operators -> IT/Security -> Legal/Compliance]

## Question Bank
[7 question groups, per next reference §B]

## Consent + Data Handling
- All interview notes treated as confidential
- Verbatim quotes anonymized in 00-Executive-Summary.md
- Personal-identifying notes stored separately
- Right-to-redact after the audit
- GDPR / regulatory retention rules respected
- No interview content sent to external LLMs without explicit consent
```

---

## 6. The Engagement Rejection Conditions

The skill REFUSES to proceed past Phase 1 if:

| Condition | Refused because |
|---|---|
| No executive sponsor named | Without sponsor = no decisions = no shipping. Pause. |
| Success criteria left blank | Without criteria = the audit is a deck. Pause. |
| Regulatory constraints "we don't know" + EU market | Regulatory unknown = systemic risk. Require legal sign-off. |
| Budget unknown AND timeline aggressive | Mismatched economics = the project dies in month 2. Re-scope. |
| AI objective = "we want to be AI-first" with no specific goal | Vague = no anchoring for scoring. Force §3D dominant objective. |
| C-Level skipping interviews | Without C-Level interviews, the C-level AI brief features are uninformed. Refuse. |

---

## 7. Phase 1 Falsification

> Read the Phase 1 outputs aloud (Engagement Mode + Company Context + Success Criteria) to the executive sponsor. Do they recognize their company? Do they confirm the dominant objective? If not, re-do.

If sponsor cannot articulate the success criteria back = the audit will not have a target. Pause until criteria are sharp.

---

## 8. Hand-off to Phase 2

Phase 1 produces the Engagement Contract. Phase 2 (Stakeholder Interview Plan) reads:
- Engagement Mode (sets the interview count)
- Company Context (sets which departments)
- Dominant Objective (sets which roles get extra time)
- Constraints (sets which questions get emphasis: regulated industries get extra Group 5 + Group 7 questions)

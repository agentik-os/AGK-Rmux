# 03 Tools + Data + Permissions

Phases 4 + 5 of the CAIO Enterprise Workflow Architect. Outputs `company-ai-os/03-Tool-And-Integration-Map.md` AND `company-ai-os/04-Data-And-Permission-Map.md`.

You cannot architect AI on top of an opaque stack. This reference makes the company's tool + data + permission layer legible BEFORE any agent is proposed.

---

# Part A :: Tool + Integration Map

## A.1 The Tool Inventory Format

For EACH tool surfaced in interviews (typically 30-80 tools in a 100-person company), capture:

```
Tool name                : ___
Vendor + version         : ___
Departments using it     : ___ (list)
Roles using it           : ___ (list)
Daily active users (est) : ___
Data stored              : ___ (PII / customer / financial / IP / public / system-of-record-for what)
Connected to             : ___ (list of integrations existing)
Current automations      : ___ (Zapier / Make / native / Composio / none)
API / Webhook available  : ___ (yes / no / partial / requires-enterprise-tier)
Authentication           : ___ (OAuth / API key / username+password / SSO / basic / none)
Owner                    : ___ (role responsible for this tool internally)
Cost (annual)            : ___ (license + per-seat * count)
Contract end date        : ___ (renewal date if known)
SOC2 / GDPR / HIPAA      : ___ (vendor compliance status)
Love or hate (interview signal): ___ (verbatim quote if strong)
Risk                     : ___ (1-10, 1=low risk, 10=high risk for vendor lock-in / data loss / compliance)
```

---

## A.2 The Tool Categories (mandatory coverage)

Map at minimum these categories. Missing category = re-interview a relevant role.

```
CRM                      : HubSpot, Salesforce, Pipedrive, Close, Attio, ...
SUPPORT                  : Intercom, Zendesk, Front, Help Scout, ...
COMMUNICATIONS           : Slack, Teams, Discord, Mattermost, ...
DOCUMENTS                : Notion, Confluence, Google Workspace, Microsoft 365, SharePoint
PROJECT MANAGEMENT       : Linear, Jira, Asana, ClickUp, Monday, Trello
CODE                     : GitHub, GitLab, Bitbucket
FINANCE                  : Stripe, Pennylane, QuickBooks, Xero, NetSuite, SAP
HR / PAYROLL             : Lucca, BambooHR, Personio, Workday, ADP, Gusto
PRODUCT ANALYTICS        : PostHog, Amplitude, Mixpanel, Heap
DATA WAREHOUSE           : Snowflake, BigQuery, Databricks, Redshift
BI / DASHBOARDS          : Metabase, Looker, Tableau, Mode, internal
MARKETING                : Mailchimp, Customer.io, Webflow, ConvertKit, Beehiiv
SCHEDULING               : Calendly, SavvyCal, Google Calendar, Outlook
DESIGN                   : Figma, Adobe, Sketch, Linear (specs)
VIDEO / MEETINGS         : Zoom, Google Meet, Teams, Loom
OBSERVABILITY            : Datadog, Sentry, NewRelic, Logtail, Vercel Analytics
AI / LLM                 : Claude / ChatGPT / Gemini subscriptions, Anthropic / OpenAI API direct usage, Cursor, Copilot
AUTOMATION               : Zapier, Make, n8n, Pipedream, Trigger.dev, internal scripts
SECURITY                 : 1Password, Vault, Okta, Duo, KnowBe4
LEGAL                    : DocuSign, Ironclad, Juro, Notarize
CUSTOM INTERNAL          : custom-built tools, internal dashboards, scripts that production-depend
```

---

## A.3 The Integration Map

After the tool inventory, draw the integration graph:

```
For EACH pair (tool A, tool B):
- Currently connected     :: yes / no / partial / one-way / two-way
- Method                  :: native, Zapier, Make, custom, MCP, Composio
- Data flowing            :: which fields, which direction, what frequency
- Reliability             :: 1-10
- Maintenance burden      :: who maintains, how often it breaks
- Should be connected     :: yes / no (per interview signals)
```

Highlight:
- **System of record per data type** (who owns customer-data / financial-data / product-data / employee-data / vendor-data)
- **Data silos** (data that lives in 2+ places with no source of truth)
- **Broken integrations** (connected but unreliable, manual workarounds exist)
- **Missing integrations** (pain quoted in interviews, no current connection)
- **Duplicated tools** (2+ tools doing the same job, different teams)

---

## A.4 The Tool Inventory Output (`03-Tool-And-Integration-Map.md`)

```
# Tool And Integration Map

## Tool Inventory
[Per-tool entries from §A.1, grouped by category]

## System Of Record (per data type)
- Customer data      : ___
- Lead data          : ___
- Deal / pipeline data : ___
- Product analytics  : ___
- Financial data     : ___
- Employee data      : ___
- Vendor data        : ___
- Marketing data     : ___
- Documentation / KB : ___
- Source code        : ___

## Data Silos
[Data that lives in 2+ places without source-of-truth + which is current]

## Current Automations
[Per automation: name + tool + scope + reliability + owner]

## Broken Integrations
[Per integration: what is supposed to work + what breaks + manual workaround]

## Missing Integrations
[Per gap: pain quote + impact + integration to add + difficulty]

## API Availability
[Per tool: API tier, rate limits, auth method, webhooks available, GraphQL or REST]

## Authentication Constraints
[Per tool: OAuth available, SSO enforced, API-key scoping, rate-limit-per-key]

## Integration Priority
[Ranked list: which integration to add first based on Phase 6 opportunity scores]

## Tool Spend Audit (optional)
- Annual tool spend total: $___
- Tools >$10k/year individually: [list]
- Overlapping tools (same job): [list with $-impact of consolidation]
```

---

## A.5 The Tool Risk Table (mandatory)

```
| Tool | Risk Type | Severity (1-10) | Description | Mitigation |
|---|---|---|---|---|
| SAP / Oracle ERP | Vendor lock-in | 9 | Migration cost >$1M, contract 5+ years | Keep as source of truth, build AI layer on top via read-only API |
| Internal custom dashboard | Single-engineer dependency | 8 | One engineer holds all knowledge, no docs | Document + add second maintainer before any AI integration |
| Email-only data flow | Data fragility | 7 | Important workflow runs via Outlook rules, breaks weekly | Move to Trigger.dev or Convex scheduler |
| Shadow IT tools | Compliance + security | 6-9 | 5+ tools used without IT awareness | Surface in audit, decide keep/remove with sponsor + IT |
```

---

# Part B :: Data + Permission Map

## B.1 The Data Source Inventory

For EACH data source (CRM tables, support tickets, product events, financial records, employee records, documents, code, ...):

```
Data source name         : ___
Type                     : ___ (CRM / support / product / financial / HR / docs / code / other)
Stored in                : ___ (which tool)
Format                   : ___ (structured / semi-structured / unstructured / mixed)
Volume                   : ___ (rows / events / docs)
Update frequency         : ___ (real-time / hourly / daily / weekly / manual)
Quality score            : ___ (1-10, with 1-line per issue)
Owner                    : ___ (role responsible)
Sensitive?               : ___ (PII / financial / health / IP / public)
GDPR / HIPAA / PCI       : ___ (yes / no / partial)
Backup + retention rules : ___ (policy)
Already in data warehouse : ___ (yes / no / partial)
```

---

## B.2 The Sensitive Data Classification

For data marked sensitive (§B.1), apply a 4-tier classification:

```
Tier 1 :: PUBLIC
- Marketing materials, public docs, public website data
- AI usage: ANY model, any vendor

Tier 2 :: INTERNAL
- Internal docs, employee directory, internal metrics
- AI usage: vendors with NDA + standard data-processing addendum (DPA)
- Logging: standard audit logs

Tier 3 :: CONFIDENTIAL
- Customer PII (name, email, phone, address), pricing, deal pipelines, employee compensation
- AI usage: vendors with SOC2 + GDPR-compliant + zero-data-retention setting if available
- Logging: mandatory audit + redaction in logs
- Human approval: required for any external-facing AI output

Tier 4 :: RESTRICTED
- Health data, payment card data, government ID, criminal records, biometrics, regulatory filings
- AI usage: ONLY via on-prem / private-cloud models, or vendor-with-HIPAA-BAA / PCI-attestation
- Logging: regulatory-grade with retention rules
- Human approval: mandatory + documented
- Default: do NOT send to external LLMs without legal review
```

The skill TAGS every data source with its tier in Phase 5 output.

---

## B.3 The Access + Permission Matrix

For EACH data source AND EACH role:

```
Role -> Data source -> Access level

Access levels:
- NO ACCESS               (default for everyone not in the matrix)
- READ-ONLY               (can query, cannot modify)
- READ-WRITE              (can query + modify)
- ADMIN                   (can modify schema + permissions)
- AI-MEDIATED-READ        (role can ask AI questions about the data, AI reads on their behalf, results filtered to what the role is allowed to see)
- AI-MEDIATED-WRITE       (role's AI agent can write back, with HITL on every write)
```

Build this as a matrix in `04-Data-And-Permission-Map.md` (markdown table or attached Convex schema reference).

---

## B.4 The Human Approval Matrix (HITL rules)

For EACH proposed AI / agent action, mark required approval:

```
Action type                    : Approval needed?
Read public data only          : No
Read internal data             : Logging only
Read confidential data         : Audit log + role-based access enforcement
Read restricted data           : Approval per session + audit + role-based + legal pre-clearance for new data types
Write to internal system       : Logging + post-hoc review weekly
Write to confidential system   : HITL on each write (human approves before commit)
Write to restricted system     : HITL + 2nd approver (4-eyes principle)
Send external communication    : HITL ALWAYS for first 90 days, then risk-tiered
Make financial transaction     : HITL ALWAYS + amount cap + 2nd approver above threshold
Make HR decision               : HITL ALWAYS + legal sign-off if dismissal / hire / discipline
Make legal decision            : NO AI autonomy. AI drafts; lawyer decides.
Make medical / clinical decision: NO AI autonomy. Clinician decides. Subject to local regulation.
Make customer-facing public statement : HITL + brand-voice gate
```

The skill REFUSES to design any agent that violates this matrix. Class 8 opportunities (REFUSED) come from this matrix.

---

## B.5 The Vendor Risk Audit (mandatory for any external-LLM usage)

For EACH external AI vendor under consideration (Anthropic, OpenAI, Google, AWS Bedrock, Cohere, Mistral, open-source self-hosted):

```
Vendor name              : ___
Models in scope          : ___
Geography (data center)  : ___ (EU / US / other)
Data residency option    : ___ (can data stay in EU?)
GDPR DPA available       : yes / no
SOC2 report available    : yes / no (Type 2)
HIPAA BAA available      : yes / no
Zero-data-retention setting available : yes / no (Anthropic + OpenAI + Bedrock = yes)
Used for training        : default + opt-out availability
Rate limits + SLA        : ___
Cost model               : per-token / subscription / committed-use
Encryption at rest + in transit : ___
Right-to-audit clause    : yes / no
Exit clause              : ___
```

Refuse to recommend vendors that fail SOC2 / GDPR if client requires.

---

## B.6 The Logging Requirements (mandatory output)

Every agentic system in `06-Agentic-System-Blueprints.md` must have logs covering:

```
- Input prompt (with PII redaction if Tier 3+)
- Model used + version
- Tools called + inputs + outputs
- Token count + cost
- Latency
- Confidence / uncertainty signal (if model exposes)
- Human approval timestamp + approver identity
- Final output
- Error if any
- Retry attempts
- Trace ID linking to source workflow
```

Stored in: Convex audit table OR Langfuse OR Datadog with retention per Tier classification (Tier 4 = 7 years, Tier 3 = 3 years, Tier 2 = 1 year, Tier 1 = 90 days default).

---

## B.7 The Data Cleanup Pre-Flight (mandatory before AI on any source)

Before recommending AI on a data source, score data readiness:

```
1. Schema documented        /10
2. Field meanings clear     /10
3. Duplicates < 5%          /10
4. Missing values < 10%     /10
5. Format consistency       /10
6. Update freshness         /10
7. PII properly tagged      /10
8. Backup verified          /10
9. Permissions clean        /10
10. Lineage traceable       /10
```

Total /100.

If a source scores < 60 :: NO AI on this source until cleanup. Opportunity tagged "data-cleanup-required" in Phase 6.

If a source scores 60-79 :: AI usage permitted with extra HITL + quality gates.

If a source scores 80+ :: AI usage permitted with standard governance.

---

## B.8 `04-Data-And-Permission-Map.md` Output Structure

```
# Data And Permission Map

## Data Source Inventory
[Per-source from §B.1]

## Sensitive Data Classification
[Tier 1-4 per source]

## PII / GDPR Audit
[List of all PII fields + which tools / databases hold them + retention rules]

## Access Levels Per Role + Per Source
[Matrix from §B.3]

## Human Approval Matrix
[From §B.4]

## Data Quality Scores
[Per source, /100 from §B.7]

## Logging Requirements
[From §B.6, per agentic system]

## Retention Rules
[Per source, regulatory + business]

## Vendor Risk Per AI Vendor In Scope
[From §B.5]

## Data Cleanup Backlog (sources < 60 quality score)
[Prioritized list]
```

---

## C. Phase 4-5 Falsification

> Pick 3 randomly-selected data sources. Trace the data from origin (where it is created) to consumption (where it is used to make a decision). Are all the hops documented? Are all the access permissions documented? Is the system of record stated?

If 2+ of 3 traces are incomplete = re-interview IT + data team to fill gaps.

---

## D. Anti-Patterns Refused

| Anti-pattern | Refused because |
|---|---|
| Tool inventory without API + auth details | Refused. Cannot architect integrations. |
| Data source without sensitivity tier | Refused. Cannot decide AI eligibility. |
| Access matrix per role left blank | Refused. Permissions are load-bearing. |
| External LLM recommendation without vendor risk audit | Refused. |
| AI on a sub-60 data-quality source | Refused. Tag for cleanup first. |
| Logging "standard logs" without per-Tier retention | Refused. Regulated industries require specific retention. |
| HITL matrix waived for "trust the AI" | Refused. Class 8 violations come from this. |

---

## E. Hand-off to Phase 6

Phases 4-5 produce the tools + data legibility. Phase 6 (Opportunity Detection) reads:
- Tool inventory + integrations -> what is feasible to automate (high integration feasibility score)
- Data sensitivity tiers -> which interventions need extra HITL or are class 8 REFUSED
- Data quality scores -> which opportunities need data cleanup first
- Vendor risk -> which LLM vendor is eligible for which client

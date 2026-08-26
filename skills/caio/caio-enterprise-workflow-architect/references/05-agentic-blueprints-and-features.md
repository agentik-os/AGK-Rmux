# 05 Agentic Blueprints + Feature Specs

Phases 7 + 8 of the CAIO Enterprise Workflow Architect. Outputs `company-ai-os/06-Agentic-System-Blueprints.md` AND `company-ai-os/07-Dashboard-Feature-Specs.md` AND optionally `company-ai-os/features/F-XXX-*.md`.

This is where the audit becomes a buildable product. Each agent gets a blueprint. Each dashboard surface gets a feature spec. Every spec is implementation-ready.

---

# Part A :: Agentic System Blueprints

## A.1 The Discipline

An agentic system is NOT a chatbot. It is a multi-step, tool-using, memory-aware, human-in-the-loop system that:

1. RECEIVES a trigger (event / time / user request)
2. ORCHESTRATES a sequence of model calls + tool calls
3. CONSULTS knowledge sources (RAG, DB, APIs)
4. USES memory (short-term + long-term)
5. ESCALATES to humans on sensitive decisions
6. LOGS everything (input, output, cost, confidence, error)
7. EVALUATES output (auto + human eval)

A "blueprint" specifies all 7 layers for a candidate system from Phase 6.

---

## A.2 The Standard Architecture Pattern

```
Trigger -> Orchestrator -> [Agent(s) -> Tools -> Memory -> Knowledge -> Evaluation] -> Interface

Trigger types:
- Time-based (cron, Convex Scheduler, Trigger.dev)
- Event-based (webhook, DB change, queue message)
- User-initiated (dashboard click, Slack command, email reply)
- Agent-initiated (one agent calls another)

Orchestrator:
- Light: a Convex action that routes
- Medium: LangGraph state machine
- Heavy: full Oracle pattern (think before acting + delegate to sub-agents)

Agents:
- Single-shot LLM (zero memory): for classification, extraction, summarization
- Stateful agent (with memory): for multi-turn customer interactions, ongoing tasks
- Multi-agent (Oracle + workers): for complex multi-domain tasks

Tools:
- Read tools: query CRM, search KB, fetch product analytics
- Write tools: create ticket, send email, update record (HITL-gated for sensitive writes)
- Calculation tools: math, finance, statistics (NEVER let the LLM do math directly)
- Composite tools: MCP servers, Composio actions, internal API wrappers

Memory:
- Short-term (in-context): last 5-10 turns
- Long-term (persistent): Convex KV / vector store / Mem0 / Letta

Knowledge:
- Structured (Convex tables, ERP, CRM)
- Semi-structured (Notion, Confluence, documents)
- Unstructured (PDFs, images, audio transcripts)
- Vector store (RAG) only when justified (often a SQL query is faster + cheaper)

Evaluation:
- Auto-eval (precision, recall, latency, cost per run)
- Human eval (HITL sampling, quality reviews)
- Drift detection (output distribution over time)

Interface:
- Internal dashboard (Next.js + Convex)
- Slack bot
- Email reply
- Embedded in existing tool (HubSpot card, Intercom widget)
```

---

## A.3 The Agent Types (named, not exhaustive)

Common agent types the CAIO audit surfaces. Each gets a blueprint when prioritized in Phase 6:

```
1. RESEARCH AGENT
   - Web search + KB search + summarization
   - Use: pre-call briefs, competitor monitoring, due diligence

2. WORKFLOW ANALYST AGENT
   - Reads logs / events / data + identifies anomalies
   - Use: weekly executive brief, monthly ops report, churn-risk detection

3. SUPPORT AGENT (Tier-1)
   - Reads ticket + customer + product + KB, suggests resolution OR escalates
   - Use: first-response triage, FAQ answering, intent classification

4. SALES ASSISTANT AGENT
   - Enriches lead, drafts outreach, summarizes calls, updates CRM
   - Use: SDR augmentation (NOT replacement), pre-call prep

5. REPORTING AGENT
   - Aggregates multi-source data + drafts narrative report
   - Use: board pack drafting, weekly metric snapshots, OKR review prep

6. FINANCE ANALYST AGENT
   - Reads accounting + revenue + cost data, surfaces variances + forecast
   - Use: budget vs actual, cash runway alerts, expense categorization

7. HR ASSISTANT AGENT
   - Reads job posts, screens resumes (HITL for hire decision), drafts onboarding
   - Use: recruiting throughput, onboarding consistency, NOT autonomous hire/fire

8. DOCUMENT PROCESSOR AGENT
   - Extracts structured data from PDFs / contracts / invoices
   - Use: vendor contract intake, invoice processing, regulatory filing prep

9. QUALITY AUDITOR AGENT
   - Reviews outputs from humans or other agents, flags issues
   - Use: support reply QA, sales call QA, code review augmentation

10. EXECUTIVE BRIEFING AGENT
    - Synthesizes daily / weekly / monthly executive summary across all sources
    - Use: CEO-brief, COO-brief, CFO-brief, board-brief
```

---

## A.4 The Blueprint Format (mandatory for every priority agent)

For each agentic system from Phase 6 BUILD NOW backlog, write a section in `06-Agentic-System-Blueprints.md`:

```
# System: [Name]

## Business Problem
[1-2 sentences: what business pain does this solve, with verbatim quote source]

## Current Workflow (without AI)
[Step-by-step: trigger -> human action -> output. Time + cost per run.]

## Future Workflow (with this agent)
[Step-by-step: trigger -> agent action -> human approval -> output. Time + cost per run.]

## Users
- Primary user :: [role + count]
- Secondary user :: [role + count]
- Affected stakeholders :: [list]

## Trigger
- Type :: time-based / event-based / user-initiated / agent-initiated
- Specifics :: [cron expression / webhook URL / dashboard button / etc.]

## Inputs
- Source 1 :: [data + format + permission + sensitivity tier]
- Source 2 :: [...]

## Agent Roles
[If single-agent, name it. If multi-agent, list each + their role.]

## Tools Needed
- Read :: [list with API/source]
- Write :: [list with HITL gating]
- Calc :: [list, with separation: LLM does NOT do math]

## Actions (the agent's loop)
1. Receive trigger
2. Read [sources]
3. Call [LLM] with [prompt template]
4. Parse output
5. If [condition] :: write [output] OR escalate to human
6. Log + emit metrics
7. Return / store / notify

## Outputs
- Format :: [JSON / Markdown / DB row / email / Slack message]
- Destination :: [Convex table / Slack channel / email / dashboard]
- Latency target :: [seconds]
- Cost target :: [$ per run]

## Human-In-The-Loop
- When :: [conditions that require human approval]
- Who :: [role + named person where possible]
- How :: [approval interface: dashboard queue / Slack thread / email confirmation]
- Timeout :: [if no approval in X hours, escalate to Y]

## Memory
- Short-term :: [last N turns of conversation, OR last M events]
- Long-term :: [per-user / per-customer / per-workflow]
- Storage :: [Convex KV / Mem0 / Letta / custom]

## Knowledge Base
- Structured :: [Convex tables list]
- Semi-structured :: [Notion / Confluence pages list]
- Unstructured + RAG :: [PDF / KB / vector store name + size]
- Indexing strategy :: [chunking + embedding + reindex frequency]

## Evaluation
- Auto-eval metrics :: [precision / recall / latency / cost]
- Human eval :: [sampling rate + reviewer role + criteria]
- Drift detection :: [output distribution check, weekly]

## Logs (mandatory)
- Input prompt (with PII redaction per Tier)
- Model + version
- Tools called + IO
- Token count + cost
- Latency
- Confidence
- Human approval (if applicable)
- Final output
- Errors + retries
- Trace ID
Stored: [Convex audit table / Langfuse / Datadog]
Retention: [per Tier from Phase 5 §B.2]

## Failure Modes (5+ mandatory)
1. [Failure mode] -> [Mitigation]
2. [...]
3. [...]
4. [...]
5. [...]

## Permissions (per Phase 5 §B.3)
- Agent identity :: [service-account / per-user-bound]
- Read scopes :: [list]
- Write scopes :: [list, with HITL flags]
- Audit log access :: [who can read]

## Success Metrics (KPIs)
- Adoption :: [active users / week]
- Quality :: [accuracy / satisfaction / approval-rate]
- Throughput :: [tasks/day]
- Time saved :: [hours/week per affected staff]
- Cost-per-output :: [$ per run, total / month]
- Error rate :: [% errors + 90-day trend]

## Phase 1 MVP scope
[Minimal version, 1-3 sentences]

## Phase 2 scope
[Next 30-60 days, 1-3 sentences]

## Phase 3 scope (optional)
[Future, 1-2 sentences]

## Dependencies
- Data source readiness :: [list]
- Integration readiness :: [list]
- Approval workflow setup :: [list]
- Vendor + LLM choice :: [Anthropic / OpenAI / on-prem]

## Estimated Effort
- Dev-weeks :: ___
- Dev-cost :: $___
- First-year-ops cost :: $___ (LLM tokens + infra + vendor + monitoring)

## Acceptance Criteria (for ship)
- [ ] [Specific measurable criterion]
- [ ] [...]
- [ ] [...]
```

A blueprint missing any field = not ready to build. Refused.

---

# Part B :: Dashboard Feature Specs

## B.1 The 12 Recommended Dashboard Views

The Company AI OS dashboard exposes these views (varying per company, this is the default):

```
1.  Executive Command Center      :: top-line metrics, today's anomalies, agent run summary, approval queue
2.  Department Dashboards          :: per-dept KPIs + workflows + agents + approval queue
3.  Workflow Map (React Flow)      :: people -> tasks -> tools -> data -> agents -> outputs
4.  Tool + Integration Graph       :: tool inventory + integration health + broken integrations
5.  Automation Backlog             :: prioritized opportunity list with status (queued / in-progress / shipped)
6.  Agent Registry                 :: every agent: blueprint summary, run count, success rate, cost
7.  Agent Run Logs                 :: per-run details with prompt, tools called, output, cost, HITL events
8.  Human Approval Queue           :: pending approvals across all agents, with context + suggested action
9.  ROI Dashboard                  :: hours saved + dollar value + adoption + per-agent ROI
10. Knowledge Base Manager         :: indexed sources, freshness, quality scores
11. Feature Roadmap                :: per-feature status + dependencies + acceptance criteria
12. Governance + Permissions       :: who can do what + audit log + vendor risk dashboard
```

For each surfaced from Phase 6 BUILD NOW, write a feature spec.

---

## B.2 The Feature Spec Format (mandatory)

For each priority feature (typically 5-15 in a full audit), write either as a section in `07-Dashboard-Feature-Specs.md` OR as a separate `features/F-XXX-*.md` file:

```
# Feature F-XXX :: [Name]

## User
- Primary :: [role + named persona]
- Secondary :: [role]

## Problem
[1-2 sentences with verbatim quote from interview, source-tagged]

## Current State
[How users handle this today, with time + tool cost]

## Ideal State
[The 5-minute experience this feature delivers]

## Business Value
[$-impact / hours saved / risk reduced + 90-day measurement]

## Input
- Data sources :: [from Phase 5]
- Triggers :: [from Phase 7]
- User actions :: [click / type / approve]

## Action
[What happens: agent invocation? Query? Aggregation? Calculation?]

## Output
- UI :: [what appears on screen]
- Side effects :: [DB writes, emails, Slack posts]
- Confidence indicator :: [if AI-generated, how is uncertainty shown]

## Data Sources
[Cross-reference to 04-Data-And-Permission-Map.md per source]

## Integrations
[Cross-reference to 03-Tool-And-Integration-Map.md per tool]

## AI / Agent Behavior
[If agentic: cross-reference to 06-Agentic-System-Blueprints.md]
[If LLM feature: prompt template + model + parameters]
[If quick automation: deterministic logic]

## UI Components (shadcn/ui + Tailwind + React Flow)
- Layout :: [page structure]
- Components :: [Table / Card / Drawer / Modal / Chart / Flow]
- States :: [loading / empty / error / success / partial]
- Interactions :: [click / hover / drag / filter / sort]
- Responsive breakpoints :: [mobile / tablet / desktop]

## Permissions (per Phase 5 §B.3)
- View access :: [roles]
- Action access :: [roles]
- Audit access :: [roles]

## Human Approval (per Phase 5 §B.4)
[When HITL triggers + approval UI + escalation rules]

## Failure States
- LLM error :: [fallback]
- Data source down :: [degraded UX]
- Permission denied :: [explanation]
- Timeout :: [retry + user message]
- Cost limit :: [graceful degradation]

## Metrics (KPIs)
- Adoption :: [DAU / WAU]
- Engagement :: [actions per session]
- Outcome :: [conversion / approval / save]
- Latency :: [P50 / P95]
- Cost :: [$ per run]

## MVP Scope (Phase 1)
[The smallest version that delivers value]

## Phase 2 Scope
[Next iteration, with trigger condition for when to build]

## Dependencies
- Backend :: [Convex tables + actions needed]
- Auth :: [Clerk role + permission setup]
- Integrations :: [APIs + tokens + scopes]
- Other features :: [if blocked by F-YYY]

## Estimated Effort
- Frontend :: ___ dev-days
- Backend :: ___ dev-days
- Integration :: ___ dev-days
- QA + ship :: ___ dev-days
- Total :: ___ dev-days

## Acceptance Criteria
- [ ] [Specific testable criterion 1]
- [ ] [Criterion 2]
- [ ] [Criterion 3]
- [ ] Logs visible in dashboard
- [ ] HITL approval works as specified
- [ ] Permissions correctly enforced
- [ ] Failure states handled
- [ ] Metrics emitted to ROI dashboard
```

---

## B.3 Example Feature Spec (worked example)

```
# Feature F-002 :: Weekly Executive AI Brief

## User
- Primary :: COO (validates + distributes)
- Secondary :: CEO, CFO, board members (read-only consumers)

## Problem
Current weekly executive brief takes 12 person-hours across product, sales, support,
finance. Often delayed Thursday-Friday. Quality varies by who assembles it. Source
data lives in 5 different tools.

Verbatim:
"I spend half my Friday chasing people for numbers and 2 hours formatting the brief
on Sunday night. It is the worst part of my week."
[source: interview_2026-05-27_coo_sarah-l, 15:14]

## Current State
- Thursday morning: COO sends Slack DMs asking for inputs from 8 people
- Thursday-Friday: 8 people send numbers + screenshots back
- Friday afternoon-Sunday: COO assembles in Google Doc
- Monday 6am: brief sent to CEO + board
- Total time: 12h aggregate, 3h COO solo, often 1-2 metrics wrong / late

## Ideal State
Monday 6am: brief lands in CEO inbox + Slack + Notion, automatically generated
overnight Sunday. COO reviews + edits Sunday evening (15 min). All sources cited
with raw values. Anomalies surfaced. Recommendations highlighted.

## Business Value
- 12h/week saved aggregate (-$60k/year, loaded $100/h)
- COO recovers Sunday evening + Friday afternoon (-$15k/year leadership time)
- Briefs always on time (board satisfaction +)
- Year-1 net benefit: $75k saved, $15k LLM + infra cost, $60k net
- Payback: 4 months on $25k build cost

## Input
Data sources (read-only):
- HubSpot (deals, pipeline, week-over-week changes)
- Intercom (ticket count, satisfaction, escalation rate, top categories)
- PostHog (DAU, retention, feature adoption)
- Stripe (revenue, MRR change, churn)
- Linear (shipped tickets, in-flight)

Trigger: Sunday 8pm CRON via Convex Scheduler

## Action
1. Convex scheduled action fires
2. Parallel fetch from 5 sources (read-only API calls)
3. Diff vs last-week stored snapshot
4. Compute anomalies (>2 SD movement) using deterministic math (NOT LLM)
5. LLM call: summarize + draft narrative + highlight wins / risks
6. Store draft in Convex
7. Notify COO via Slack: "Brief draft ready"
8. COO reviews + edits in dashboard
9. COO clicks "Approve + Distribute"
10. Auto-post to #c-level Slack + Notion + email to board

## Output
- Markdown 1-page brief in Convex
- Notion page (auto-created in /Weekly-Brief/2026-WW)
- Slack message in #c-level
- Email to board distribution list
- Sources cited with hyperlinks + raw values

## Data Sources
See 04-Data-And-Permission-Map.md sources #2 (HubSpot), #5 (Intercom), #8 (PostHog),
#12 (Stripe), #14 (Linear)

## Integrations
- HubSpot REST (OAuth)
- Intercom Data Export API
- PostHog Query API
- Stripe API (read-only)
- Linear GraphQL

## AI / Agent Behavior
- Model :: Claude 4 Sonnet (default for cost / latency balance)
- Math :: NEVER LLM. Compute deltas in Convex action. Pass numbers as facts to LLM.
- Prompt template :: see prompts/F-002-exec-brief.md
- Eval :: COO approval rate target >80% without major edits

## UI Components
- Page :: /dashboard/briefs
- Components ::
  - Card listing weekly briefs (history)
  - Drawer: brief detail with sections (TLDR, Sales, Support, Product, Finance, Anomalies, Asks)
  - Inline editor (markdown)
  - "Approve + Distribute" button + Slack-preview modal
  - Source attribution links per metric
- States :: draft / pending-review / approved / distributed / archived

## Permissions
- View :: c-level + executive-assistant
- Edit :: COO + assigned-editor
- Approve :: COO only
- Audit :: CTO + CAIO

## Human Approval
- Mandatory before distribution
- COO can edit any section
- If COO does not approve by Sunday 11pm, system sends reminder
- If still not approved Monday 5am, fallback: send to CEO with "Auto-distribution paused, please review"

## Failure States
- One source down :: brief generated with note "Source X unavailable, last-week snapshot used" + alert to CAIO
- All sources down :: notification only, no draft
- LLM error :: deterministic-only brief (numbers + diffs, no narrative) + alert
- Approval timeout :: see human-approval section

## Metrics
- Adoption :: weeks shipped on time / total weeks (target >95%)
- Quality :: COO approval rate without major edits (target >80%)
- Latency :: brief draft ready by Sunday 11pm (target 100%)
- Cost :: < $5 / brief in LLM + infra
- Engagement :: board member read rate via tracked link

## MVP Scope (Phase 1)
- 1-page markdown brief
- 5 sources
- COO HITL
- Slack + Notion + email distribution
- 4 weeks of consecutive on-time delivery

## Phase 2 Scope
- Per-board-member personalization (focus on their area)
- Q&A on the brief via Slack threaded reply
- Audio summary (5-min listen via ElevenLabs)
- Trigger: 4 consecutive weeks of P1 success

## Dependencies
- Convex schema :: briefs, briefRevisions, sources
- Clerk role :: c-level + executive-assistant
- HubSpot OAuth setup
- Intercom export token
- PostHog API key
- Stripe read-only key
- Linear API token

## Estimated Effort
- Frontend :: 4 dev-days
- Backend :: 5 dev-days
- Integrations :: 2 dev-days
- QA + ship :: 1 dev-day
- Total :: 12 dev-days

## Acceptance Criteria
- [ ] Brief generates automatically Sunday 8pm
- [ ] All 5 sources read successfully OR fail gracefully
- [ ] COO can edit + approve from dashboard
- [ ] Approval triggers Slack + Notion + email distribution
- [ ] Metrics cited with source link
- [ ] All math done in Convex (not LLM)
- [ ] Audit log captures full run
- [ ] Cost < $5 / brief
- [ ] 4 consecutive weeks shipped on time
- [ ] COO approval rate >= 80%
- [ ] HITL timeout fallback tested
```

---

# Part C :: The Convex Schema + Next.js Screen List (Phase 8 architecture)

## C.1 The Recommended Convex Schema

```
organizations
  id, name, industry, size, region, regulatoryConstraints[]

departments
  id, orgId, name, headRole

roles
  id, orgId, deptId, title, mission, headcount

users
  id, orgId, clerkUserId, role, email, name, permissions[]

stakeholders
  id, orgId, name, role, deptId, interviewed (boolean), notes

interviews
  id, orgId, stakeholderId, date, mode, durationMin, sponsorPresent

interviewResponses
  id, interviewId, group (1-7), verbatim, observations, followUps[]

tools
  id, orgId, name, vendor, category, deptIds[], dataStored[], apiAvailable, authMethod, riskScore

integrations
  id, orgId, toolAId, toolBId, type (native/zapier/make/custom/mcp), reliability /10

workflows
  id, orgId, name, ownerRoleId, frequency, trigger, deptIds[], tags[]

workflowSteps
  id, workflowId, order, description, tools[], actor (human/automated), timeMin

handoffs
  id, workflowId, fromRoleId, toRoleId, format, slaMin

dataSources
  id, orgId, name, type, toolId, sensitivityTier (1-4), qualityScore, owner

permissions
  id, orgId, roleId, dataSourceId, accessLevel (no/read/readwrite/admin/ai-mediated)

opportunities
  id, orgId, name, deptId, type (automation/llm/agentic/dashboard/redesign), scores{}, total, verdict

featureSpecs
  id, orgId, opportunityId, name, userRoleId, problem, mvpScope, p2Scope, effortDevDays, status

agents
  id, orgId, name, blueprintId, status (draft/built/staging/prod/deprecated), modelChoice, costPerRun

agentRuns
  id, agentId, startedAt, endedAt, input, output, toolCalls[], tokenCount, cost, latency, confidence, hitlEvents[], error, traceId

automations
  id, orgId, name, type (zapier/make/convex/trigger-dev), workflowId, status

approvals
  id, orgId, agentRunId, requiredAt, role, decidedAt, decidedBy, decision, comments

roadmapItems
  id, orgId, featureSpecId, phase (P1/P2/P3/parked), startWeek, endWeek, owner

roiEstimates
  id, orgId, workflowId, hoursPerWeek, loadedCost, currentYearlyCost, expectedAutomationRate, expectedYearlySaving, implementationCost, paybackMonths, confidence

auditLogs
  id, orgId, userId, action, resource, timestamp, ip, userAgent

documents
  id, orgId, type (sop/policy/contract/spec), title, source, sensitivityTier, content / pointer

knowledgeSources
  id, orgId, name, type (structured/semi/unstructured), location, indexingStrategy, freshness, qualityScore
```

Multi-tenant: every table has `orgId`. Every row has `createdAt`, `updatedAt`, `createdBy`. Sensitive tables also have `version` for history.

---

## C.2 The Next.js 16 Screen List (App Router)

```
/dashboard                       :: Executive Command Center
/dashboard/anomalies             :: today's anomalies + alerts
/dashboard/approvals             :: pending HITL approval queue
/company-map                     :: 6-level view (Vue Entreprise + Vue Agentic OS toggle)
/workflows                       :: workflow inventory + React Flow detail per workflow
/workflows/[id]                  :: single workflow detail
/tools                           :: tool inventory + integration graph
/tools/[id]                      :: tool detail + integrations
/opportunities                   :: backlog with filter + sort
/opportunities/[id]              :: opportunity detail
/features                        :: feature spec library
/features/[id]                   :: feature spec detail
/agents                          :: agent registry
/agents/[id]                     :: agent detail + recent runs + cost + adoption
/agents/[id]/runs                :: full run log
/runs/[id]                       :: single run trace
/approvals                       :: HITL queue (cross-agent)
/approvals/[id]                  :: approval detail + 1-click approve / reject / escalate
/roi                             :: ROI dashboard per workflow / per agent
/knowledge                       :: KB manager: sources, freshness, quality
/roadmap                         :: gantt-like roadmap with phases
/settings/integrations           :: tool integration management
/settings/permissions            :: role-based access control
/settings/vendors                :: AI vendor risk dashboard
/settings/audit-logs             :: regulatory-grade audit
/settings/org                    :: org config
```

---

## C.3 React Flow Node Types (for `/company-map` + `/workflows/[id]`)

```
Person          (icon: user, color: blue)
Role            (icon: badge, color: indigo)
Department      (icon: building, color: violet)
Tool            (icon: tool-specific or wrench, color: green)
Data Source     (icon: database, color: cyan)
Workflow        (icon: arrows, color: amber)
Automation      (icon: zap, color: yellow)
Agent           (icon: bot, color: purple)
Approval        (icon: shield-check, color: orange)
Output          (icon: file or message, color: teal)
Risk            (icon: alert-triangle, color: red)
KnowledgeSource (icon: book, color: emerald)
```

Edge types:
- `manual` (dashed, gray) :: human-mediated
- `automated` (solid, green) :: automation
- `agentic` (solid, purple) :: agent-mediated
- `broken` (dashed, red) :: known-broken integration
- `pending` (dashed, amber) :: proposed integration

---

## D. Phase 7-8 Falsification

> Pick the top 3 feature specs. Show them to:
> 1. The executive sponsor (do they see business value?)
> 2. A senior engineer NOT involved in the audit (can they estimate effort + spot missing requirements?)
> 3. The role who will USE the feature (do they recognize their workflow?)

If any of the 3 spots a missing critical requirement = re-spec.

---

## E. Anti-Patterns Refused

| Anti-pattern | Refused because |
|---|---|
| Agent blueprint without HITL spec | Refused. Sensitive actions need explicit approval. |
| Feature spec without acceptance criteria | Refused. Not testable. |
| Agent doing math directly via LLM | Refused. Math in code, LLM gets facts. |
| Schema designed for single-tenant when multi-tenant needed | Refused. Add orgId everywhere. |
| Dashboard without logs surfaced | Refused. Black box not enterprise-grade. |
| Dashboard without cost visibility | Refused. CFO will pull funding. |
| Agent without failure modes documented | Refused. Refuses to ship. |
| Feature spec without dependencies stated | Refused. Build will block on hidden deps. |
| Multi-agent system without orchestrator | Refused. Agents alone = chaos. |
| RAG recommended when a SQL query would do | Refused. Simpler tool first. |

---

## F. Hand-off to Phase 9

The agentic blueprints + feature specs feed Phase 9 (Implementation Roadmap + ROI + Governance). Each spec maps to a roadmap item (Phase 1 / 2 / 3 / parked). Each blueprint feeds the ROI model (estimated savings + cost). Governance rules from Phase 5 flow into the roadmap as cross-cutting constraints.

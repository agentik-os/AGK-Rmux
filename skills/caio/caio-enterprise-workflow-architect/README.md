# CAIO Enterprise Workflow Architect

> Turn an organization into a legible, automatable, agentic system. People + daily work first. Tools + data + permissions next. Agents + dashboards last.

> A CAIO does not start by building agents. A CAIO starts by making the company legible.

Built by [Agentik OS](https://agentik-os.com). Composes with [inner-os-architect](https://skills.agentik-os.com/inner-os-architect), [personal-os-builder](https://skills.agentik-os.com/personal-os-builder), [ideation-and-vision-architect](https://skills.agentik-os.com/ideation-and-vision-architect), [agentic-systems-builder](https://skills.agentik-os.com/agentic-systems-builder), [agentik-skill-forge](https://skills.agentik-os.com/agentik-skill-forge), [creator-media-engine](https://skills.agentik-os.com/creator-media-engine), [taste-and-aesthetic-director](https://skills.agentik-os.com/taste-and-aesthetic-director), [freedom-lifestyle-designer](https://skills.agentik-os.com/freedom-lifestyle-designer), [body-energy-protocol-builder](https://skills.agentik-os.com/body-energy-protocol-builder), [relationship-and-connection-architect](https://skills.agentik-os.com/relationship-and-connection-architect).

---

## What it produces

`company-ai-os/` directory with 10 deliverables:

1. **00-Executive-Summary.md** :: 1-2 page CEO read with verdict, top opportunities, $-impact, required decisions, 30/60/90
2. **01-Stakeholder-Interview-Plan.md** :: departments, roles, order, 7-question bank, consent + data handling
3. **02-Role-And-Workflow-Inventory.md** :: per-role mission + tasks + inputs + actions + outputs + tools + frictions + ideal workflow + workflow taxonomy tagging
4. **03-Tool-And-Integration-Map.md** :: tool inventory (categorized) + system-of-record per data type + data silos + current + broken + missing + priority integrations + risk table
5. **04-Data-And-Permission-Map.md** :: data source inventory + 4-tier sensitivity + RBAC matrix + HITL matrix + vendor risk + data-cleanup backlog
6. **05-Automation-Opportunity-Backlog.md** :: 10-criteria scoring + 8-verdict classification + prioritized table with $-impact, payback, effort
7. **06-Agentic-System-Blueprints.md** :: per-agent: problem + future workflow + tools + memory + KB + HITL + logs + permissions + failure modes + success metrics
8. **07-Dashboard-Feature-Specs.md** :: 12 default views + per-feature: user + input + action + output + UI + permissions + acceptance + effort
9. **08-Implementation-Roadmap.md** :: 6-phase plan (data+access -> quick wins -> dashboard MVP -> agentic systems -> dept rollout -> governance) with cost + owner + risk per phase
10. **09-ROI-Governance-And-Risks.md** :: per-workflow ROI math + AI usage policy + HITL rules + 5-category risk register + change-mgmt plan

Optional `company-ai-os/features/F-XXX-*.md` :: per-feature implementation specs for the engineering team.

## The 5 Engagement Modes

| Mode | Duration | Output |
|---|---|---|
| `quick-executive-audit` | 90 min | 5-7 prioritized opportunities + 1-page brief + next-step |
| `department-discovery` | 1-2 weeks | 1 department workflow + 10-20 opportunities + 3 feature specs |
| `full-company-workflow-audit` | 4-12 weeks | Complete `company-ai-os/` (10 files) + 10-20 feature specs |
| `dashboard-architecture` | 1-3 weeks | Feature specs + Convex schema + Next.js screens + React Flow nodes |
| `implementation-roadmap` | 1 week (post-audit) | 30/60/90 + stack + team + cost + ROI + risk register |

## The 10 Iron Laws

1. Always start with the real work people do, not with technology.
2. Never propose an agent when a simple automation suffices.
3. Never automate a workflow you have not understood.
4. Never invent ROI, time saved, or business pain.
5. Always distinguish 5 intervention types: automation / LLM feature / agentic system / dashboard feature / process redesign.
6. Every feature ships with: input, action, output, owner, data, permissions, risks, success metrics.
7. Human-in-the-loop on every sensitive decision.
8. The dashboard must expose: sources, logs, status, errors, costs, confidence.
9. The default stack is a starting point, not a religion.
10. Refuse to sell magic. Architecture, not fairytale.

## The 7-Question Interview Bank (per stakeholder, 30-45 min)

```
Group 1 :: Daily Work          (8-10 min)  What you do, every day / week / month
Group 2 :: Inputs              (5-7 min)   What you need, where from, missing / late / wrong
Group 3 :: Actions             (5-7 min)   Manual / copy-paste / decision / approval / cross-team
Group 4 :: Outputs             (4-5 min)   What you produce, who uses it, cost-of-error
Group 5 :: Tools               (4-5 min)   Daily-used, hated, duplicated, disconnected
Group 6 :: Automations + AI today (3-4 min) What exists, what breaks, what's wished
Group 7 :: Ideal State         (5-7 min)   AI teammate, perfect dashboard, 5-hour saving, 10x smoother
```

Every answer captured VERBATIM. Paraphrasing refused.

## The 10-Criteria Opportunity Scoring

```
1.  Business impact /10
2.  Time saved /10
3.  Frequency /10
4.  Pain intensity /10
5.  Data readiness /10
6.  Integration feasibility /10
7.  Risk level /10  (inverted: 10 = low risk)
8.  Change resistance /10  (inverted: 10 = enthusiastic)
9.  Agent suitability /10
10. Dashboard fit /10
```

Weighted /100. Weights adjust by client's dominant business objective.

## The 8 Verdicts

```
BUILD NOW:
1. Quick automation        :: deterministic, < 2 weeks, Zapier / Make / Trigger.dev / Convex action
2. LLM feature             :: classification / extraction / summarization / draft, 1-4 weeks
3. Agentic workflow        :: multi-step + tool-use + memory + HITL, 4-12 weeks
4. Dashboard feature       :: visualization + alerts + filters, 2-8 weeks
5. Process redesign        :: zero-tech intervention, change-mgmt only

DO NOT BUILD YET:
6. Data cleanup required   :: data quality < 60/100, cleanup first
7. Executive decision required :: budget / risk / vendor lock-in
8. REFUSED                 :: sensitive HR / legal / financial / customer-public decision, NO agent autonomy
```

Class 8 examples REFUSED by the skill:
- Auto-fire / auto-hire / auto-performance-review
- Auto-litigation-decision
- Auto-credit-decision (regulatory)
- Auto-medical-decision (clinical)
- Auto-public-communication on behalf of CEO without approval

## The Default Tech Stack (starting point, not religion)

```
Frontend                 :: Next.js 16
Backend / DB             :: Convex
Deploy                   :: Vercel
Auth                     :: Clerk
Billing                  :: Stripe
UI                       :: Tailwind + shadcn/ui
Workflow graph           :: React Flow
Jobs                     :: Trigger.dev OR Convex Scheduler
Integrations             :: MCP / Composio / direct APIs
LLM                      :: Anthropic OR OpenAI (model-agnostic adapter)
Monitoring               :: Langfuse, Datadog, Vercel Analytics, custom Convex logs
```

Adapts for: SSO / SOC2 / GDPR / HIPAA / data residency / private cloud / existing ERP-CRM.

## The 6-Level Dashboard View

Two perspectives navigated in the same dashboard:

### Vue Entreprise (business)
```
Level 0 :: Company Strategy
Level 1 :: Departments
Level 2 :: Roles / Teams
Level 3 :: Workflows
Level 4 :: Tools / Data / Integrations
Level 5 :: Automations / Agents / Features
```

### Vue Agentic OS (system)
```
Level 0 :: AI Command Center
Level 1 :: Orchestrator / Oracle
Level 2 :: C-Level AI agents
Level 3 :: Department work sessions
Level 4 :: Specialist agent teams
Level 5 :: Tools, APIs, MCP, automations
```

## The Convex Schema (default starting point)

```
organizations, departments, roles, users, stakeholders, interviews, interviewResponses
tools, integrations, workflows, workflowSteps, handoffs
dataSources, permissions
opportunities, featureSpecs
agents, agentRuns, automations, approvals
roadmapItems, roiEstimates, auditLogs
documents, knowledgeSources
```

Multi-tenant via `orgId`. Per-row `createdAt`, `updatedAt`, `createdBy`.

## Installation

```bash
bash <(curl -sL https://skills.agentik-os.com/install) caio-enterprise-workflow-architect
```

Then in Claude Code:

```
/caio-enterprise-workflow-architect
```

## What it refuses

- Audit without verbatim pain quotes per opportunity
- ROI / time-saved numbers without (hours * cost * frequency) math
- Class 8 sensitive HR / legal / financial / clinical agent autonomy
- Stack recommendation ignoring client's regulatory constraints
- "Full audit" output with fewer than 10 verbatim interviews
- Feature spec missing any of 12 fields
- Roadmap without cost + ROI + payback
- Black-box agent without logs + costs + confidence + status surfaced
- Vendor lock-in pitches
- Magic-AI promises
- AI on a data source with quality score < 60 (tag for cleanup first)

## What it surfaces (always)

- The 50-150 deduplicated opportunities across the company
- The 10-50 prioritized backlog items (scored /100)
- The 5-10 build-now opportunities with $-impact + payback
- The Class 8 REFUSED opportunities (transparency for governance)
- The system of record per data type
- The HITL matrix per agent type
- The vendor risk audit per LLM vendor
- The 30/60/90 roadmap with cost + owner per phase
- The AI Usage Policy ready for sponsor signature
- The 5-category risk register
- The change-management plan

## Modern Stack 2026 (references the skill draws from)

| Need | Anchored in |
|---|---|
| Workflow audit method | Lean + Theory of Constraints + Goldratt + Deming + modern systems thinking |
| Stakeholder interviewing | Jobs-To-Be-Done (Christensen) + The Mom Test (Fitzpatrick) + 5-Whys |
| Opportunity scoring | RICE + ICE + AI-adapted (data-readiness, agent-suitability, change-resistance) |
| Agentic system design | Anthropic agentic patterns + LangGraph + CrewAI + AutoGen |
| Dashboard product design | Linear + Vercel + Stripe + Notion + Retool patterns |
| ROI modeling | NPV + payback + loaded-cost + adoption-curve |
| Governance | NIST AI RMF + EU AI Act + SOC2 + GDPR + ISO 42001 |
| Change management | Kotter 8-step + ADKAR + Prosci |
| HITL design | Anthropic HITL + IBM AI Fairness + Google Responsible AI |

## Iron Test (90 days post-delivery)

1. Did the top-scored opportunity ship in production?
2. Did the 30-day quick wins ship (3 of 3 = ideal)?
3. Did the executive sponsor receive the dashboard MVP at day 60?
4. Did the team adopt the supervised agent (production usage, not pilot)?
5. Did the ROI math hold (re-measure)?

4+ of 5 pass = the Company AI OS works.

12-month test: shift from "we use ChatGPT sometimes" to "N production agents, M dashboards, K hours saved verified, $X ROI documented" + wave 2 to next departments without original CAIO present. Yes to both = self-compounding OS.

## License

MIT.

---

*Version 1.0.0 :: a CAIO makes the company legible first, then automatable, then agentic, in that order.*

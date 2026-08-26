# 04 Opportunity Detection + Automation Backlog

Phase 6 of the CAIO Enterprise Workflow Architect. Outputs `company-ai-os/05-Automation-Opportunity-Backlog.md`.

This is the most consequential phase of the audit. Get this wrong = the company ships agents into chaos. Get this right = the company ships 5-10 high-impact AI projects that compound.

---

## 1. The Discipline

Opportunities come FROM the interviews + workflow inventory, NOT from the CAIO's hypothesis. Each opportunity is anchored in:

```
- Verbatim pain quote from at least 1 interviewee (Tier 1, single quote)
- OR verbatim pain quotes from 3+ unrelated interviewees (Tier 2, cross-corroborated)
- AND a documented workflow with time-cost math
- AND a sensitivity-tier classification per Phase 5
- AND a data-quality score (cannot recommend AI on data < 60/100)
```

No opportunity claim without these 4 anchors. Refused.

---

## 2. The Opportunity Detection Process (sequential)

### Step 1 :: Surface candidates from interviews
For EACH interview, pull every quote tagged as:
- Group 6 :: "I wish this was automated"
- Group 7 :: "If I had an AI teammate, I would delegate..."
- Group 7 :: "What would save me 5 hours per week"
- Group 1 :: "Tasks that should not exist"
- Group 2 :: "Inputs that are often missing / late / wrong"
- Group 3 :: "Steps that are copy-paste / decision / approval / cross-team"
- Group 4 :: "Outputs that get late or wrong"
- Group 5 :: "Tools I hate"
- Group 5 :: "Tools that should be connected but are not"

For a 100-person company in full-audit mode = 200-500 raw candidates.

### Step 2 :: Dedupe + cluster
Group candidates by underlying problem. "Sales rep wishes deal updates auto-flowed to dashboard" + "Sales manager wishes weekly pipeline summary was automated" = same root opportunity (Sales Ops Automation).

Typical clusters: 50-150 deduplicated.

### Step 3 :: Filter against governance + permission constraints
For each candidate, check:
- Does it touch sensitive data (Tier 3-4)? If yes, require HITL or RAG-with-filtering.
- Does it require an agent to make sensitive decisions (HR / legal / financial / clinical)? If yes, Class 8 REFUSED or HITL ALWAYS.
- Does it require data quality > 60? If data quality is lower, tag "data-cleanup-required".
- Is the executive sponsor required? If yes, tag "executive-decision-required".

Typical filtered: 30-80.

### Step 4 :: 10-criteria scoring per candidate
See §3 below. /100 weighted.

### Step 5 :: Verdict per candidate
See §4 below.

### Step 6 :: Prioritization for the backlog
Sort by score, group by verdict, present as table.

Typical output: 20-50 prioritized opportunities, of which 5-10 are top-3-month build candidates.

---

## 3. The 10-Criteria Scoring Matrix

Every candidate scored on these 10 dimensions, each /10. Weighted total /100.

```
1.  Business impact /10        :: revenue lift / cost reduction / risk reduction / quality improvement. 10 = clear $-impact >$100k/year. 1 = nice-to-have.
2.  Time saved /10             :: hours/week saved * count of affected staff. 10 = >100h/week aggregate. 1 = <2h/week.
3.  Frequency /10              :: how often this workflow runs. 10 = daily / multiple-times-daily. 1 = quarterly or rarer.
4.  Pain intensity /10         :: how acute is the friction (verbatim signal). 10 = "I want to quit this job because of this". 1 = mild annoyance.
5.  Data readiness /10         :: is the data accessible + clean + integrated TODAY? 10 = yes, schema documented, API available, quality >80. 1 = data lives in PDFs + Excel + 3 places.
6.  Integration feasibility /10 :: APIs / webhooks exist? Auth simple? Rate limits OK? 10 = native APIs, OAuth, generous rate limits. 1 = no API, screen-scraping required.
7.  Risk level /10              :: regulatory / brand / data / financial risk. INVERTED: 10 = low risk, 1 = high risk.
8.  Change resistance /10       :: how much human resistance will adoption face. INVERTED: 10 = enthusiastic team. 1 = vocal opposition.
9.  Agent suitability /10       :: multi-step + judgment + tool-use = high. Deterministic if-this-then-that = LOW (counter-flag, prefer automation). 10 = clear agentic case. 1 = deterministic.
10. Dashboard fit /10           :: does this benefit from a visible UI + logs + status? 10 = high-stakes, multi-source, executive-visible. 1 = invisible background.
```

### Weighting (default)

```
20% :: Business impact (#1)
15% :: Time saved (#2)
10% :: Frequency (#3)
10% :: Pain intensity (#4)
10% :: Data readiness (#5)
10% :: Integration feasibility (#6)
10% :: Risk level (#7)
 5% :: Change resistance (#8)
 5% :: Agent suitability (#9)
 5% :: Dashboard fit (#10)
```

### Weighting adjustments (per dominant objective from Phase 1 §3D)

```
Objective :: save time
  +10 weight to time-saved (#2), frequency (#3)
  -5 weight to revenue-impact subset of #1

Objective :: reduce cost
  +10 weight to business-impact (#1, cost-side)
  +5 weight to time-saved (#2)

Objective :: increase revenue
  +15 weight to business-impact (#1, revenue-side)
  +5 weight to agent-suitability (#9, customer-facing agents)

Objective :: improve quality
  +10 weight to pain-intensity (#4)
  +5 weight to data-readiness (#5)

Objective :: centralize operations
  +10 weight to dashboard-fit (#10)
  +5 weight to integration-feasibility (#6)

Objective :: support teams (not replace)
  -10 weight to agent-suitability (#9, prefer dashboards + LLM features)
  +10 weight to time-saved (#2)

Objective :: build customer-facing AI product
  +15 weight to business-impact (#1, revenue-side)
  +10 weight to agent-suitability (#9)
  +5 weight to integration-feasibility (#6)

Objective :: build internal agentic OS
  +10 weight to agent-suitability (#9)
  +5 weight to dashboard-fit (#10)
  +5 weight to data-readiness (#5)
```

Total weights MUST still sum to 100. The skill auto-normalizes after adjustment.

---

## 4. The 8-Verdict Classification

Every scored opportunity lands in ONE of 8 verdicts.

### BUILD NOW (5 classes)

```
1. Quick automation        :: if-this-then-that, deterministic
   - Score threshold: any (independent of score, this is the intervention type)
   - Effort: < 2 weeks / < 0.5 dev-month
   - Tooling: Zapier / Make / Trigger.dev / Convex action
   - HITL: minimal (post-hoc audit)

2. LLM feature             :: classification / extraction / summarization / draft-generation
   - Score >= 80 :: Build P1 (first 30 days)
   - Score 65-79 :: Build P2 (60 days)
   - Effort: 1-4 weeks
   - Tooling: Claude / GPT via API, in Convex action or Next.js route
   - HITL: review-before-publish, optional for low-risk

3. Agentic workflow        :: multi-step + tool-use + memory + HITL
   - Score >= 80 :: Build P2 (60 days)
   - Score 65-79 :: Build P3 (90 days)
   - Effort: 4-12 weeks
   - Tooling: orchestrator + agent + tools (MCP / Composio / direct APIs) + memory (Convex KV)
   - HITL: mandatory per Phase 5 §B.4 matrix

4. Dashboard feature       :: visualization + alerts + filters
   - Score >= 75 :: Build P1-P2 (with corresponding agents)
   - Effort: 2-8 weeks
   - Tooling: Next.js + Convex query + Tailwind + shadcn/ui + React Flow
   - HITL: dashboard is HITL by design

5. Process redesign        :: zero-tech intervention, change-mgmt only
   - Score >= 70 :: Recommend before any AI investment in this area
   - Effort: 1-4 weeks elapsed
   - Tooling: process-document + training + measurement
   - HITL: N/A (no AI yet)
```

### DO NOT BUILD YET (3 classes)

```
6. Data cleanup required   :: data quality < 60/100
   - Action: cleanup project before AI
   - Effort: 2-12 weeks of data work
   - Owner: data engineering or department-with-the-mess

7. Executive decision required :: budget / risk / vendor lock-in / political
   - Action: escalate to sponsor + relevant C-Level
   - Block: until decision made

8. REFUSED                 :: sensitive HR / legal / financial / customer-facing decision that NO agent should make autonomously
   - Action: do not build as an agent. May redesign as a dashboard with HITL approval queue.
   - Document the refusal explicitly in the backlog (this is a CAIO governance signal).
```

---

## 5. Examples Of Each Verdict

### Quick automation
- "Auto-create Linear issue from Intercom ticket flagged as bug"
- "Auto-send Slack DM to AE when their deal moves to 'lost' in HubSpot"
- "Auto-archive customer in CRM after 6 months of inactivity + final email sent"

### LLM feature
- "Classify Intercom tickets into 12 categories on receipt"
- "Extract key clauses from incoming vendor contracts into Notion"
- "Summarize last 30 days of customer reviews into themes"
- "Draft personalized cold-email opener from LinkedIn + CRM data"

### Agentic workflow
- "Tier-1 support triage agent :: read ticket, look up customer in CRM, check product analytics, suggest resolution OR escalate to Tier-2"
- "Weekly executive AI brief :: aggregate CRM + product analytics + support trends + finance metrics + draft 1-page summary + COO HITL"
- "Sales discovery research agent :: enrich lead, surface news / competitors / 10-K filings (if public co), draft pre-call brief"

### Dashboard feature
- "Renewal risk dashboard :: CSM views customers at risk + reasons + recommended actions + history"
- "Agent run log dashboard :: every agent invocation visible to ops team with status / cost / latency / errors"
- "ROI dashboard :: hours saved per agent + dollar value + adoption rate"

### Process redesign
- "Sales handoff from SDR to AE has 3-day delay + lost context :: not an AI problem, fix the handoff template + Slack channel + 4h SLA before any AI"
- "Support team uses 4 chat tools (Intercom + Crisp + LiveAgent + email) :: consolidate to 1 BEFORE building support agent"

### Data cleanup required
- "Customer email field is in 4 places + 30% mismatched :: build single-source-of-truth in CRM before any email-based AI"

### Executive decision required
- "Replace Salesforce with HubSpot (current $200k/year SF spend, team prefers HB) :: requires CEO + Sales VP + IT decision before any AI on the new stack"

### REFUSED
- "Auto-fire reps under 80% quota for 2 consecutive quarters" (HR autonomy)
- "Auto-approve credit-limit increase up to $50k" (financial / regulatory)
- "Auto-respond to negative public reviews as CEO" (brand + legal)
- "Auto-decide which support tickets get refund without human approval" (financial + customer)
- "Auto-publish blog posts in CEO's voice without review" (brand + IP)

---

## 6. The Atomic Opportunity Record (mandatory format)

Every opportunity in `05-Automation-Opportunity-Backlog.md` follows:

```
## Opportunity #N :: [Name, 2-7 words]

Department          : ___
Roles affected      : ___ (with headcount)
Workflow            : ___ (cross-reference to 02-Role-And-Workflow-Inventory.md)

Observed pain (verbatim):
"[exact quote]"
[source: interview_2026-05-27_jane-smith_support-manager, 14:32]
(Optional: 2-3 corroborating quotes)

Affected scope:
- People             : N roles
- Frequency          : Daily / weekly / monthly, with count
- Time cost          : ___ hours / week aggregate
- Loaded dollar cost : ___ / year (math: hours/week * loaded hourly cost * 52)

Intervention type candidates:
- Quick automation   :: applicable? why / why not
- LLM feature        :: applicable? why / why not
- Agentic workflow   :: applicable? why / why not
- Dashboard feature  :: applicable? why / why not
- Process redesign   :: applicable? why / why not

Recommended type     : ___ (1 chosen, with 1-line justification)

Data sources needed  : ___ (cross-reference to 04-Data-And-Permission-Map.md)
Data quality (avg of needed sources): ___ / 100
Data sensitivity tier: ___ (1-4)
HITL required        : Yes / No + spec

Integration feasibility:
- APIs / webhooks needed: ___
- Auth method          : ___
- Rate limits / cost   : ___

Risks:
1. ___ (with mitigation)
2. ___ (with mitigation)
3. ___ (with mitigation)

10-criteria scores:
1. Business impact      : __/10
2. Time saved           : __/10
3. Frequency            : __/10
4. Pain intensity       : __/10
5. Data readiness       : __/10
6. Integration feasibility : __/10
7. Risk level           : __/10 (inverted, 10=low)
8. Change resistance    : __/10 (inverted, 10=enthusiastic)
9. Agent suitability    : __/10
10. Dashboard fit       : __/10

Weighted total /100   : ___

Verdict               : Build now P1 / P2 / P3 | Park 90d | Data cleanup | Executive decision | REFUSED

Estimated effort      : ___ dev-weeks
Estimated cost        : ___ (build + first-year ops)
Estimated benefit     : ___ $/year saved or revenue lifted (with confidence interval)
Estimated payback     : ___ months

MVP scope             : [1-3 sentences]
Phase 2 scope         : [1-3 sentences]

Falsification:
If, 30 days after this opportunity ships, [specific metric] is NOT [specific target], the opportunity was mis-scoped. Re-score.
```

A record without ALL fields = not actionable backlog item. Refused.

---

## 7. The Backlog Output Table (executive view)

After all opportunities are recorded, summarize in `05-Automation-Opportunity-Backlog.md` header as a table:

```
| #  | Opportunity                          | Dept   | Type            | Score /100 | Verdict          | Effort   | Cost    | Annual Benefit | Payback |
|----|--------------------------------------|--------|-----------------|------------|------------------|----------|---------|----------------|---------|
| 1  | Weekly Executive AI Brief            | C-Level| LLM feature     | 87         | Build P1         | 12 d-d  | $25k    | $156k          | 2 mo    |
| 2  | Tier-1 Support Triage Agent          | Support| Agentic         | 84         | Build P2         | 40 d-d  | $80k    | $312k          | 3 mo    |
| 3  | Sales Followup Auto-Sequence         | Sales  | Quick automation| 81         | Quick win (P1)   | 5 d-d   | $8k     | $120k          | 1 mo    |
| 4  | Knowledge Base Internal RAG          | All    | LLM feature     | 76         | Build P3         | 25 d-d  | $50k    | $200k          | 3 mo    |
| 5  | Renewal Risk Detection Dashboard     | CS     | Dashboard       | 71         | Park 90 days     | 30 d-d  | $60k    | $400k(est)     | 2 mo    |
| 6  | Marketing AI Copy at Scale           | Mktg   | LLM feature     | 58         | Data cleanup     | -        | -       | -              | -       |
| 7  | Auto-fire underperforming reps       | Sales  | Agent           | 12         | REFUSED          | -        | -       | -              | -       |

Total Build-Now (P1+P2+P3) :: 8 opportunities, $213k investment, $988k/year benefit, blended 3-month payback
```

(d-d = dev-days)

---

## 8. Phase 6 Falsification

> Pick the top 3 ranked opportunities. Read them aloud to the executive sponsor + 1-2 affected dept heads.
> 1. Do they recognize the pain (verbatim quote rings true)?
> 2. Do they agree with the scoring (no major /10 disagreement)?
> 3. Are they willing to commit budget + time to the top 3?

If 2+ stakeholders push back on the top 3 = re-score with revised weights or revise the verdicts.

---

## 9. Anti-Patterns Refused

| Anti-pattern | Refused because |
|---|---|
| Opportunity without verbatim pain quote | Refused. Hallucinated pain. |
| Opportunity with single-criteria scoring ("looks important") | Refused. 10 criteria mandatory. |
| Opportunity tagged "agent" when deterministic | Refused. Quick automation tagged. |
| Opportunity scoring > 80 but data quality < 60 | Refused. Tag data-cleanup-required first. |
| REFUSED class 8 opportunities omitted from backlog | Refused. Documented refusals are CAIO governance signals. |
| Backlog with > 50 build-now items | Refused. Force prioritization to top 5-10 for P1-P3. |
| Backlog with 0 build-now items | Re-audit. Either the company has no pain (unlikely) or the audit missed the signal. |
| ROI math without (hours * cost * frequency) | Refused. Made-up numbers. |
| Effort estimates without dev-week breakdown | Refused. CFO + CTO need real numbers. |

---

## 10. Hand-off to Phase 7

The prioritized backlog feeds Phase 7 (Agentic Blueprints + Feature Specs). The top 5-10 BUILD NOW opportunities get a full feature spec in `07-Dashboard-Feature-Specs.md` + (if agentic) a blueprint in `06-Agentic-System-Blueprints.md`. The Park / Data-cleanup / Executive-decision / REFUSED records stay in the backlog for transparency.

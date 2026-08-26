# 02 Stakeholder + Interviews + Role-Workflow Inventory

Phases 2 + 3 of the CAIO Enterprise Workflow Architect. Outputs `company-ai-os/01-Stakeholder-Interview-Plan.md` AND `company-ai-os/02-Role-And-Workflow-Inventory.md`.

A workflow audit is only as good as its interviews. This reference protects both halves: who gets interviewed (Part A) and what the interviews extract (Part B).

---

# Part A :: Stakeholder Interview Plan

## A.1 The Department Coverage Matrix

For a `full-company-workflow-audit`, interview each department. For `department-discovery`, pick one. For `quick-executive-audit`, C-level only.

| Department | Always interview (in any mode) | Departments-discovery sub-mode candidate |
|---|---|---|
| C-Level (CEO, COO, CTO, CFO, CAIO sponsor) | Yes | Quick-executive-audit only |
| Operations | Yes (full) | Yes |
| Sales | Yes (full) | Yes (high-priority dept) |
| Marketing | Yes (full) | Yes |
| Customer Support | Yes (full) | Yes (high-priority dept) |
| Customer Success | Yes (full) | Yes |
| Finance | Yes (full) | Yes |
| HR / People Ops | Yes (full) | Yes |
| Product | Yes (full) | Yes |
| Engineering / IT | Yes (full) MANDATORY for any stack decision | Yes |
| Legal / Compliance | Yes (full) MANDATORY if regulated | Yes (if regulated) |
| Field operations (if applicable) | Yes (full) | Yes |
| Field sales / on-site reps (if applicable) | Yes (full) | Yes |
| External vendors (if relevant) | Optional | Optional |

---

## A.2 The Roles To Interview Per Department

For each department, interview MINIMUM 2 levels (manager + IC), ideally 3 levels (department head + manager + IC + 1 power-user if applicable).

### C-Level (always)
```
CEO / Founder
COO (if exists)
CTO / VP Eng
CFO / Head of Finance
CAIO / Head of AI (if exists)
1 outside board member (optional, for context)
```

### Operations
```
Head of Ops
Operations Manager
Operations Coordinator / Specialist
```

### Sales
```
VP Sales / Sales Director
Sales Manager (per region/segment)
AE (Account Executive) :: 2-3 IC
SDR / BDR :: 1-2 IC
Sales Ops Specialist
RevOps person
```

### Marketing
```
CMO / Head of Marketing
Marketing Manager (per channel: Content / Paid / SEO / Events)
Marketing IC :: 1-2
Marketing Ops Specialist
```

### Customer Support
```
Head of Support
Support Manager / Team Lead
Tier-1 Support Agent :: 2-3 IC
Tier-2 / Escalation Specialist
Quality / Coaching Lead
```

### Customer Success
```
Head of CS
CSM (Customer Success Manager) :: 2-3 IC
Renewal / Expansion Specialist
```

### Finance
```
Head of Finance / Controller
Accounts Payable Specialist
Accounts Receivable Specialist
FP+A Analyst (if exists)
Payroll / HR-finance person
```

### HR / People Ops
```
Head of People
Recruiter :: 1-2
HR Generalist / HR Business Partner
L+D (Learning + Development) Specialist
```

### Product
```
Head of Product / VP Product
Product Manager :: 2-3
Product Designer :: 1-2
Researcher (if exists)
```

### Engineering / IT
```
VP Engineering / CTO
Engineering Manager (per team)
Senior Engineer :: 2-3
DevOps / SRE
Data Engineer (if data team exists)
Security Engineer / CISO (if exists)
IT Operations / Helpdesk Lead
```

### Legal / Compliance
```
General Counsel / Head of Legal
Compliance Officer / DPO (Data Protection Officer)
Privacy Lawyer
Vendor Contract Specialist
```

### Field operations (if applicable)
```
Field Manager
On-site Operator :: 2-3 IC
Maintenance / Reliability person
```

The skill REFUSES to skip Engineering / IT in any mode beyond quick-executive-audit. Without IT, the stack decisions are made in a vacuum.

---

## A.3 The Interview Order (recommended)

```
Day 1-2  :: C-Level (alignment + sponsor commitment)
Day 3-5  :: Department Heads (per dept, in priority order from Phase 1 §3D)
Day 6-10 :: Managers + ICs (per dept)
Day 11-13:: Engineering / IT / Security / Legal (transverse stack + governance interviews)
Day 14   :: Sponsor re-sync (mid-audit checkpoint)
Day 15+  :: Field operators + edge-cases + missing-stakeholders identified during weeks 1-2
```

Why this order:
- C-Level FIRST = alignment locked + assumptions corrected before everyone else is interviewed
- IC interviews LAST = they have already heard "the CAIO is here" buzz, sometimes adjust answers
- Engineering / IT MID-AUDIT = data + stack questions are clearer once business workflows are partly mapped
- Mid-audit sponsor re-sync = course-correct before week 3

---

## A.4 The Question Bank (the 7 Question Groups)

For EVERY interviewee (30-45 min, exception: C-Level often 45-90 min):

### Group 1 :: Daily Work (8-10 min)
```
Walk me through your last 7 days. What did you actually do?
What do you do every day? Every week? Every month?
Which of those tasks are repetitive?
Which require judgment?
Which require talking to other people?
Which are painful but necessary?
Which should not exist?
```

Goal: surface what the role ACTUALLY does. Not the job description. The lived work.

### Group 2 :: Inputs (5-7 min)
```
What information do you need before you can start your work?
Where does this information come from?
Who provides it?
In what format do you receive it (PDF, CRM, ticket, email, voice, Slack, ...)?
What is often missing, late, unclear, or wrong?
```

Goal: identify the upstream dependencies + the broken inputs.

### Group 3 :: Actions (5-7 min)
```
What steps do you perform on a typical task?
Which steps are manual?
Which are copy-paste?
Which require a decision?
Which require approval from someone else?
Which involve another team?
```

Goal: identify automation candidates + handoffs + decision points.

### Group 4 :: Outputs (4-5 min)
```
What do you produce (documents, decisions, tickets resolved, deals closed, ...)?
Who uses it after you?
Where does it go (system, file, email, Slack)?
How is quality checked?
What happens if the output is late or wrong?
```

Goal: identify downstream dependencies + the cost-of-error.

### Group 5 :: Tools (4-5 min)
```
Which tools do you use daily?
Which tools do you hate (and why)?
Which tools contain important data?
Which tools are duplicated or overlapping?
Which are connected?
Which should be connected but are not?
```

Goal: tool inventory + integration map + frustration signals.

### Group 6 :: Automations + AI today (3-4 min)
```
Do you already use any automations (Zapier, Make, internal scripts)?
Where? Who built them? Are they reliable? What breaks?
Do you use any AI tools today (ChatGPT, Claude, Gemini, Copilot, ...)?
For what tasks? How well does it work?
What do you wish was automated but is not?
```

Goal: map current AI + automation surface, avoid recommending what already exists.

### Group 7 :: Ideal State (5-7 min)
```
If you had an AI teammate, what would you delegate first?
If your dashboard was perfect, what would it show on your first morning login?
What should happen automatically (no human in the loop)?
What should require your approval before shipping?
What would save you 5 hours per week (be specific)?
What would make your work 10x smoother (be specific)?
```

Goal: pull out the high-impact opportunities from the person doing the work.

---

## A.5 The Verbatim Capture Rule

Every Group 1-7 answer captured VERBATIM (or near-verbatim) in interview notes. The skill REFUSES to summarize quotes during interview-recording phase.

Why: a verbatim quote is a receipt. A paraphrased quote is the CAIO's hypothesis. The audit must be falsifiable, so receipts are mandatory.

Format per interview:
```
# Interview: [Role + initials, date]

## Group 1 :: Daily Work
[Verbatim transcription]

## Group 2 :: Inputs
[Verbatim]

... (all 7 groups)

## CAIO Observations
[Brief notes: friction signals, automation candidates, integration gaps]

## Follow-up Questions
[Things to ask other people based on this interview]
```

---

## A.6 Consent + Data Handling (mandatory)

Before each interview:

```
Disclose:
- Audit purpose
- Who will see the notes (CAIO + executive sponsor only)
- Where they are stored (specify: tool + access controls)
- Anonymization rules (quotes used in 00-Executive-Summary are anonymized to "Support Manager" not "Jane Smith")
- Right-to-redact after the audit
- GDPR / regulatory retention rules
- Whether any content goes to an external LLM (default: NO, unless explicit consent per interviewee)

Capture consent: written or verbal-recorded.
```

The skill REFUSES to ingest interview content into external LLMs without explicit per-interviewee consent if the company is GDPR / HIPAA / regulated.

---

## A.7 `01-Stakeholder-Interview-Plan.md` Output Structure

```
# Stakeholder Interview Plan

## Engagement Mode
[A/B/C/D/E from Phase 1]

## Departments To Interview
[List from §A.1, scoped by mode]

## Roles To Interview Per Department
[List from §A.2, per dept]

## Interview Order
[From §A.3, with calendar dates]

## Question Bank
[Reference to §A.4 groups 1-7]

## Interview Notes Template
[From §A.5]

## Missing Stakeholders
[Identify roles you wanted to interview but could not, with reason]

## Consent + Data Handling
[From §A.6, with sponsor sign-off]
```

---

# Part B :: Role + Workflow Inventory

## B.1 The Per-Role Output Format

For EACH interviewed role, write a per-role section in `02-Role-And-Workflow-Inventory.md`:

```
# Role: [Department :: Role title]

## Mission
[1-2 sentences: what this role exists to accomplish]

## Headcount
[Number of people in this role at the company]

## Daily Tasks
[Bulleted list, time per task per day]

## Weekly Tasks
[Bulleted list, time per task per week]

## Monthly Tasks
[Bulleted list]

## Inputs
[Format per input: source -> data -> who-provides -> reliability /10]

## Actions
[Bulleted list of steps in a typical task. Tag each: manual / copy-paste / decision / approval / cross-team]

## Outputs
[Format per output: product -> consumer -> destination -> quality-check-method -> cost-of-error]

## Tools Used
[List with verbatim verdicts: tool name + frequency + love-or-hate + data-stored]

## Repetitive Work
[What this role does that should be automated]

## Decision Work
[What requires judgment + cannot be fully automated]

## Communication Work
[Cross-team handoffs, meetings, async]

## Frictions (verbatim quotes)
[Top 3-5 verbatim quotes from interview about pain]

## Automation Ideas (from role)
[What THEY said they wished was automated, verbatim]

## Ideal Workflow
[Vision: if everything worked, what would this role do all day?]
```

---

## B.2 The Per-Workflow Output Format

After per-role mapping, identify the 10-30 most important WORKFLOWS that cross roles. For each, write:

```
## Workflow: [Name]

Owner       : [Role + person]
Frequency   : [per day / week / month / quarter, with count]
Trigger     : [event that starts it]
Input       : [data + format + source]
Steps       : [numbered list]
Tools       : [list, in order of use]
Handoffs    : [role-to-role transitions]
Output      : [what gets produced]
Failure points : [where it breaks]
Time cost   : [hours/week across all affected staff]
Loaded cost : [time * loaded hourly cost * 52 weeks]
Business impact : [revenue / cost / risk / quality]
Automation potential : [score /10 + 1-line why]
Agent potential : [score /10 + 1-line why]
Dashboard potential : [score /10 + 1-line why]
Process redesign potential : [score /10 + 1-line why]
```

A workflow without these 14 fields = not yet inventoried. Re-interview to fill gaps.

---

## B.3 The Workflow Taxonomy (mandatory tagging)

Tag each workflow with 1-3 tags from this taxonomy. Used in Phase 6 scoring.

```
DECISION WORKFLOWS (judgment-heavy)
  - decision-customer-facing
  - decision-financial
  - decision-legal
  - decision-hr
  - decision-strategic

EXECUTION WORKFLOWS (action-heavy)
  - execution-deterministic    (same input -> same output, always)
  - execution-semi-deterministic (mostly same, sometimes edge cases)
  - execution-judgment-heavy   (requires human judgment per case)

CREATION WORKFLOWS (output-heavy)
  - creation-content
  - creation-document
  - creation-code
  - creation-design
  - creation-data

COORDINATION WORKFLOWS (handoff-heavy)
  - coordination-internal
  - coordination-external (customer / vendor / partner)
  - coordination-scheduled (meetings, sync rituals)

REPORTING WORKFLOWS (aggregation-heavy)
  - reporting-executive
  - reporting-departmental
  - reporting-regulatory
  - reporting-customer-facing

MONITORING WORKFLOWS (detection-heavy)
  - monitoring-anomaly
  - monitoring-quality
  - monitoring-compliance
```

The tag drives the intervention type:
- `execution-deterministic` :: ALWAYS automation candidate (NOT agent overkill)
- `decision-customer-facing` + `decision-financial` :: ALWAYS HITL
- `creation-content` :: LLM feature candidate
- `coordination-internal` :: workflow + dashboard candidate
- `reporting-executive` :: agentic candidate (multi-source aggregation + summarization)
- `monitoring-anomaly` :: agentic + dashboard candidate

---

## B.4 Phase 3 Falsification

> Pick 3 randomly-selected interviewed people. Read them their role + workflow inventory entry. Do they recognize it? Do they spot anything missing or wrong?

If 2+ of 3 spot something missing = re-interview that role to fill gaps.

---

## B.5 Anti-Patterns Refused

| Anti-pattern | Refused because |
|---|---|
| Paraphrasing quotes instead of capturing verbatim | Refused. Verbatim is the receipt. |
| Skipping IC interviews, interviewing only managers | Refused. Managers describe the documented process; ICs do the real work. |
| Tagging every workflow as "agent candidate" | Refused. Most workflows are automation candidates, not agent candidates. |
| Workflow without time cost + dollar cost math | Refused. Cost = the language CFO speaks. |
| Per-role mission section copy-pasted from job description | Refused. The MISSION = what the role actually accomplishes. Re-interview. |
| Interview without consent + data-handling step | Refused. Especially in regulated industries. |

---

## B.6 Hand-off to Phase 4

Phase 3 produces the role + workflow inventory. Phase 4 (Tool + Integration Map) reads the Tools Used section per role to build the tool inventory. Phase 5 (Data + Permission Map) reads inputs + outputs + tools to build the data flow map. Phase 6 (Opportunity Detection) reads the friction quotes + automation ideas + workflow tags to surface the prioritized backlog.

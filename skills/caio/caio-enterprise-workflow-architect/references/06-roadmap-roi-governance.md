# 06 Roadmap + ROI + Governance + Risks + Executive Report

Phase 9 of the CAIO Enterprise Workflow Architect. Outputs `company-ai-os/08-Implementation-Roadmap.md`, `company-ai-os/09-ROI-Governance-And-Risks.md`, AND `company-ai-os/00-Executive-Summary.md`.

This is the phase that makes the audit CREDIBLE to CEO + CFO + CTO + IT + Legal. Numbers + risks + timeline + governance = the language they speak.

---

# Part A :: The 30/60/90 Implementation Roadmap

## A.1 The Discipline

A roadmap is not a wish list. It is a sequence of commitments with cost + owner + dependencies + acceptance criteria + risk.

The CAIO audit produces a 6-phase roadmap:

```
Phase 0 :: Data + Access setup (week 1-2)
Phase 1 :: Quick wins (week 1-4)
Phase 2 :: Dashboard MVP + first LLM features (week 3-8)
Phase 3 :: First agentic systems shipped (week 6-12)
Phase 4 :: Department rollout (week 12-26)
Phase 5 :: Governance + monitoring + iteration (continuous)
```

Each phase has a deliverable, owner, success criteria, risk register.

---

## A.2 Phase 0 :: Data + Access Setup (week 1-2)

```
Deliverables:
- Convex schema deployed (from §C.1 in reference 05)
- Clerk auth + role-based permissions configured per §B.3 (reference 03)
- Vercel project deployed
- API tokens stored in secret manager (1Password / Vault / Convex env)
- Audit logs table active
- Per-Tier retention rules implemented
- LLM vendor selected + DPA signed + zero-data-retention configured (if applicable)
- AI usage policy published internally

Owner: CAIO + IT lead + CISO (if regulated)

Acceptance:
- All P1 data sources accessible from Convex via API
- Test agent run logged in audit table with full trace
- Permissions matrix enforced in Clerk
- Vendor risk doc signed by CISO + General Counsel
```

---

## A.3 Phase 1 :: Quick Wins (week 1-4)

```
Deliverables:
- 1-3 quick automations shipped (Zapier / Make / Trigger.dev / Convex actions)
- Top 1-2 LLM features shipped (classification / extraction / summarization)
- Dashboard scaffolding live (/dashboard, /opportunities, /agents)
- ROI baseline measured (hours/week per affected staff, BEFORE)

Owner: 1-2 engineers + CAIO

Acceptance:
- Quick win 1: shipped, adopted by N users, hours saved verified by week 4
- Quick win 2: same
- LLM feature 1: shipped, accuracy >= X target, used by N team members
- Dashboard MVP routes accessible to internal team
- ROI before-numbers logged per workflow
```

---

## A.4 Phase 2 :: Dashboard MVP + First LLM Features (week 3-8)

```
Deliverables:
- Executive Command Center live (/dashboard)
- Top 1-2 dashboard features shipped (often: Workflow Map + Agent Registry + Approval Queue)
- 2-3 LLM features in production
- First agent in supervised mode (NOT autonomous) with HITL on every output

Owner: 2-3 engineers + 1 designer + CAIO

Acceptance:
- Dashboard MVP accessible to executive sponsor + dept heads
- Approval queue tested + adopted (Phase 1 LLM features route through it)
- First agent operational with 95%+ HITL approval rate (humans are still validating heavily)
- All agent runs visible in /agents/[id]/runs with full trace
```

---

## A.5 Phase 3 :: First Agentic Systems Shipped (week 6-12)

```
Deliverables:
- 1-3 high-priority agentic systems in production (e.g., support tier-1, weekly exec brief, sales discovery)
- HITL approval rates trending down (humans intervening less as confidence rises)
- Agent run cost + latency dashboard
- ROI dashboard active with before / after numbers

Owner: 3-5 engineers + 1 designer + CAIO + 1 PM

Acceptance:
- Top agent has 4+ weeks production run, adoption >70% of target users
- HITL approval rate stable (varies by agent: support agent might be 60-80% auto-approve, exec brief might be 100% HITL)
- ROI proven: hours saved measured, dollar value computed, agreed by sponsor
- Failure modes triggered + recovered correctly (chaos testing)
- Audit log retrievable for last 30 days of every agent run
```

---

## A.6 Phase 4 :: Department Rollout (week 12-26)

```
Deliverables:
- 2-3 additional departments onboarded (each gets their own opportunity backlog reviewed + 2-3 shipped agents)
- Cross-department agents (where applicable: e.g., billing-support coordination agent)
- Internal training rolled out (Loom videos + Notion docs + office hours)
- Department-level adoption metrics

Owner: CAIO + dept champions + 4-6 engineers + 1 designer + 1 PM

Acceptance:
- 3 departments using Company AI OS in production
- Each department has dashboard adopted by >70% of target users
- Agent run logs available company-wide (subject to permissions)
- Department-level ROI computed
- Change-mgmt resistance below 4/10 (from initial 7-9/10 typical)
```

---

## A.7 Phase 5 :: Governance + Monitoring + Iteration (continuous)

```
Deliverables (ongoing):
- Quarterly governance review (CAIO + General Counsel + CISO + executive sponsor)
- Vendor risk re-audit annual + on-incident
- AI policy update annual + on-regulation-change
- Drift monitoring + retraining / re-prompting cycles
- Capacity planning per agent (token budget, cost growth)
- New opportunity intake + scoring (rolling)

Owner: CAIO (with named successor for fractional-CAIO scenarios)

Acceptance:
- Quarterly review completed + findings logged
- Audit logs continuously retained per regulatory tier
- Vendor contracts current
- AI usage policy current
- Adoption metrics tracked monthly
```

---

## A.8 The Roadmap Output (`08-Implementation-Roadmap.md`)

Table-first, narrative second.

```
# Implementation Roadmap

## Phase 0 :: Data + Access Setup (week 1-2)

| Item | Owner | Effort | Risk | Status |
|---|---|---|---|---|
| Convex schema deploy | Eng | 3d | low | ready |
| Clerk RBAC setup | Eng + IT | 2d | low | ready |
| LLM vendor DPA signed | Legal + CAIO | 5d | med | pending |
| ...

## Phase 1 :: Quick Wins (week 1-4)
[same table format]

...
```

Plus a Gantt-like view (text or rendered Mermaid) showing dependencies + parallel tracks.

---

# Part B :: ROI Model

## B.1 The Per-Workflow ROI Template

For EACH workflow that gains a P1-P3 feature, fill:

```
Workflow                  : ___
People involved           : N people * R role
Hours per week (current)  : ___ aggregate hours
Loaded hourly cost        : $___ (salary + benefits + overhead / 2080 hours)
Current yearly cost       : ___ hours * $___ * 52 weeks = $___

Expected automation rate  : ___% (NOT 100%, account for HITL + edge cases + adoption ramp)
Expected yearly saving    : current yearly cost * automation rate * adoption rate

Implementation cost       : dev-weeks * dev-cost-per-week + LLM-tokens + infra
Maintenance cost (year 1) : LLM tokens * volume + monitoring + occasional fixes
Maintenance cost (year 2+): typically 30-50% of year-1 cost

Payback period            : implementation cost / monthly saving
Year-1 net                : saving - implementation - maintenance
Year-3 cumulative net     : (saving * 3) - implementation - (maintenance * 3)

Confidence                : low / medium / high
  - low: <50% chance the saving materializes (data-quality + adoption risk)
  - medium: 50-80%
  - high: >80%
```

---

## B.2 The Loaded Hourly Cost Reference

```
Role example          | Annual salary | Loaded (1.4x) | Hourly rate ($)
---|---|---|---
Junior IC             | $50k          | $70k          | $34
Mid IC                | $80k          | $112k         | $54
Senior IC             | $130k         | $182k         | $87
Manager               | $150k         | $210k         | $101
VP                    | $250k         | $350k         | $168
C-Level               | $400k+        | $560k+        | $269+
```

Loading = salary + benefits + payroll tax + overhead + equity. 1.3-1.5x is typical. Use 1.4x as default unless client provides their loaded-cost figure.

---

## B.3 The Aggregate ROI Table (`09-ROI-Governance-And-Risks.md`)

```
| Feature | Workflow | Current yearly cost | Saving estimate | Confidence | Implementation cost | Year-1 net | Payback |
|---|---|---|---|---|---|---|---|
| F-001 Tier-1 Triage | Support triage | $312k | $187k (60% automation) | medium | $80k | +$77k | 3 mo |
| F-002 Exec Brief | Weekly C-brief | $60k + $15k C-time | $60k (80% auto) | high | $25k + $5k year-1 ops | +$30k | 4 mo |
| F-003 Sales Followup | Outreach sequencing | $120k | $96k (80% auto) | medium | $8k | +$88k | 1 mo |
| ...

## Totals
- Investment (Year 1): $213k build + $40k ops = $253k
- Annual savings (Year 1, blended): $383k
- Year-1 net: +$130k
- Year-3 cumulative net: +$1.0M
- Average payback: 3.0 months
```

---

# Part C :: Governance + Risks

## C.1 The AI Usage Policy (mandatory output)

Every Company AI OS ships with an internal AI Usage Policy. The skill drafts a starter:

```
# AI Usage Policy :: [Company Name]

## Scope
- Applies to: all employees, contractors, vendors
- Covers: any use of LLM-based tools (ChatGPT, Claude, Gemini, Copilot, Cursor) + internal AI agents

## Allowed
- Tier 1 (Public): any model, any vendor
- Tier 2 (Internal): vendors with DPA
- Tier 3 (Confidential): vendors with SOC2 + GDPR + zero-data-retention; HITL on external-facing outputs
- Tier 4 (Restricted): on-prem only OR vendor with HIPAA-BAA / PCI-attestation; legal review per new use case

## Forbidden
- Class 8 REFUSED actions (see Phase 6 §4)
- Pasting Tier 3-4 data into consumer ChatGPT / Claude.ai without zero-retention configured
- Bypassing HITL on sensitive agent decisions
- Disabling audit logs
- Using vendor models without DPA when Tier 2+ data is in play

## Required
- Read this policy + complete training before using AI tools at work
- Tag any new AI use case via /opportunities form
- Report incidents within 24h to CAIO + General Counsel
- Quarterly attestation by every department head

## Enforcement
- First violation: training + warning
- Second: written warning + manager review
- Third: progressive discipline per HR policy
- Severe (Class 8 violation, PII leak): immediate review + possible termination

## Review
- This policy reviewed annually by CAIO + General Counsel + CISO
- On any new regulation (EU AI Act updates, US state AI laws): expedited review

Signed by: [executive sponsor] + General Counsel + CISO
Date: ___
```

---

## C.2 The Human-In-The-Loop Rules

Cross-reference Phase 5 §B.4 matrix. Surface in `09-ROI-Governance-And-Risks.md`:

```
| Action type | HITL required? | Approver role | Timeout |
|---|---|---|---|
| Read public data | No | n/a | n/a |
| Read internal data | Log only | n/a | n/a |
| Read confidential data | Log + RBAC | n/a | n/a |
| Read restricted data | Per-session approval | Owner role | 24h |
| Write internal | Post-hoc weekly review | Manager | 7d |
| Write confidential | HITL per write | Role-specific | 4h |
| Write restricted | HITL + 2nd approver | 4-eyes | 24h |
| External comms | HITL ALWAYS first 90d, then risk-tiered | Manager + Comms | 4h |
| Financial transaction | HITL + cap + 2nd approver above threshold | Finance + 2nd | 24h |
| HR decision | HITL + legal | HR + Legal | 5d |
| Legal decision | NO AI autonomy | Counsel | n/a |
| Medical / clinical | NO AI autonomy | Clinician | n/a |
| Public-facing CEO comm | HITL + brand-voice gate | CEO + Comms | 24h |
```

---

## C.3 The Risk Register

Document by category:

### Security risks
```
- Prompt injection via untrusted input
  Mitigation: input sanitization + system prompt isolation + output validation

- Secrets exposure in prompts / logs
  Mitigation: secret manager + PII redaction in logs + scoped API keys

- Vendor account compromise
  Mitigation: SSO + MFA + per-environment keys + rotation + audit

- Supply chain (vendor breach)
  Mitigation: vendor SOC2 review + incident-response playbook + multi-vendor failover plan
```

### Data risks
```
- Unauthorized data access via agent
  Mitigation: role-bound agent identity + RBAC enforcement at query layer

- Data drift (training data assumptions invalidate)
  Mitigation: distribution monitoring + retraining triggers

- PII leakage to LLM vendor
  Mitigation: redaction + Tier classification + vendor zero-retention setting

- Loss / corruption
  Mitigation: backups + Convex + version history + DR plan
```

### Vendor risks
```
- LLM vendor outage
  Mitigation: model-agnostic adapter + failover model + degraded UX path

- LLM vendor pricing change
  Mitigation: cost monitoring + budget alerts + alternative model evaluation

- LLM vendor model deprecation
  Mitigation: version pinning + migration playbook + 90-day deprecation buffer

- Lock-in
  Mitigation: prompts + tools stored in Convex (vendor-neutral); model swap < 1 week
```

### Compliance risks
```
- GDPR violations (data subject rights, transfer)
  Mitigation: DPO sign-off + DPIA per agent + Schrems II clauses + EU-resident model option

- EU AI Act category classification
  Mitigation: per-agent risk classification + transparency obligations + human-oversight docs

- SOC2 audit findings
  Mitigation: continuous logging + quarterly internal audit + external Type-2 annual

- Industry-specific (HIPAA, FINRA, etc.)
  Mitigation: legal review per use case + BAA / attestation per vendor
```

### Adoption risks
```
- Team resistance
  Mitigation: change-mgmt plan + champions + training + transparency on AI decisions

- "AI replacing my job" narrative
  Mitigation: explicit framing as augmentation + headcount commitment (where true) + role evolution plan

- Quality erosion
  Mitigation: human eval + drift detection + customer feedback loops

- Approval queue overload
  Mitigation: tier-down HITL as confidence grows + auto-approve thresholds
```

---

## C.4 Change Management Plan (the "soft" side of the audit)

Most AI projects fail at adoption, not technology. The plan includes:

```
- Pre-launch: town hall + Q+A about the audit + addressing fears + naming what changes for whom
- Champions: 1-2 named champions per department, trained early
- Training: per-feature 30-min Loom + Notion doc + 1h office hours per week, first 30 days
- Communication cadence: weekly Slack updates + monthly all-hands
- Feedback loop: in-app feedback button + monthly survey + quarterly retrospective
- Honest framing: "this saves 5h/week for X role, those 5h go to Y higher-value work, headcount stable for at least 12 months" (or whatever the truth is)
- No surprise layoffs: if AI does cause role changes, named 60-90 days in advance with transition support
```

---

# Part D :: The Executive Summary (the 1-pager that lands)

## D.1 The Discipline

The CEO does not read 50 pages. The CEO reads 1 page. The Executive Summary is THAT page.

It must:
- Convey the audit's verdict in 5 minutes
- List the top 5-7 opportunities with $-impact + payback
- Surface 3-5 risks the CEO must own
- State the 30/60/90 recommendation
- End with a clear decision the CEO needs to make

---

## D.2 `00-Executive-Summary.md` Output

```
# Company AI OS :: Executive Summary

## Company Context
[2-3 sentences: size, industry, current AI maturity, dominant business objective for AI]

## AI Maturity Level (current)
Score /5: ___
- 1 :: ad-hoc ChatGPT usage, no policy
- 2 :: pilot use cases, no central strategy
- 3 :: 2-3 production agents, basic governance
- 4 :: cross-department AI OS, measured ROI, governance mature
- 5 :: AI infused in core operating system, customer-facing AI products, board-level KPIs

## Main Frictions (top 5, with verbatim quotes)
1. [Pain] :: "[quote]" [source]
2. ...
5. ...

## Biggest Opportunities (top 5-7, $-prioritized)

| Rank | Opportunity | Department | Type | $ Annual benefit | Effort | Risk | Phase |
|---|---|---|---|---|---|---|---|
| 1 | Tier-1 Support Triage Agent | Support | Agentic | $312k | 40d | med | P2 |
| 2 | Weekly Executive AI Brief | C-Level | LLM | $75k | 12d | low | P1 |
| ...

Total annual benefit (P1-P3 combined): $___
Total investment (P1-P3 combined): $___
Blended payback: ___ months

## Recommended First Systems (in order)
1. ___ (week 1-4)
2. ___ (week 3-8)
3. ___ (week 6-12)

## Expected Business Impact (year 1)
- Hours saved: ___
- $ saved / generated: ___
- New capabilities: ___

## Required Decisions (CEO ownership)
1. [Decision] :: [context + recommendation]
2. ___
3. ___

## 30/60/90-Day Roadmap (high-level)
- 30 days: ___
- 60 days: ___
- 90 days: ___

## Risks + Governance (top 3, with mitigation)
1. ___
2. ___
3. ___

## What This Skill Did NOT Do
- Build production code (downstream: engineering team or agentic-systems-builder)
- Sign vendor contracts (downstream: legal + procurement)
- Decide on data residency / cloud strategy (downstream: CIO + CISO)
- Replace therapy or strategic counsel (this is operational AI architecture)

## Next Step
[1 sentence: what the CEO must approve / sign / fund this week to start Phase 0]

Prepared by: [CAIO name]
Audit dates: ___
Stakeholders interviewed: ___ across ___ departments
Sponsor: [executive sponsor name]
Date: ___
```

---

## D.3 The Executive Summary Falsification

> Read the Executive Summary aloud to the executive sponsor. Time it. Could the CEO read it in 5 minutes? Does the verdict + top opportunities + decisions land? Could the CEO make a yes/no decision on Phase 0 funding TODAY based on this 1-pager?

If no = compress. Re-write. Cut adjectives. Cut hedges. Cut explanation. Keep numbers + decisions.

---

## E. Phase 9 Discipline Checks (before final ship)

| Check | Pass criterion |
|---|---|
| All 10 `company-ai-os/` files exist (per engagement mode) | Yes |
| Executive Summary <= 2 pages | Yes |
| ROI math anchored in (hours * cost * frequency) per workflow | Yes |
| Roadmap has cost + team + risk per phase | Yes |
| HITL matrix complete for every agent | Yes |
| AI Usage Policy drafted + ready for sponsor signature | Yes |
| Risk register covers security + data + vendor + compliance + adoption | Yes |
| Change management plan included | Yes |
| Vendor risk audit per LLM vendor in scope | Yes |
| Class 8 REFUSED opportunities documented (not hidden) | Yes |
| Quarterly governance review schedule placed in calendar | Yes |
| Single concrete next-step decision named for the CEO | Yes |

If any fails = re-run the corresponding phase. Never ship a Company AI OS that fails discipline.

---

## F. Anti-Patterns Refused

| Anti-pattern | Refused because |
|---|---|
| Executive Summary > 2 pages | CEO reads 1 page. Compress. |
| ROI made up without (hours * cost * frequency) | CFO will spot it. Refused. |
| Roadmap without owners + dependencies + risk per phase | Not buildable. Refused. |
| AI policy missing Tier classification | Cannot enforce. Refused. |
| Risk register only covers security (misses data + vendor + compliance + adoption) | Incomplete. Refused. |
| Class 8 REFUSED hidden from backlog | Governance signal lost. Refused. |
| Roadmap without quarterly governance review | OS goes stale. Refused. |
| Vendor risk audit waived ("Anthropic is fine") | Refused. Document + sign. |
| Change-mgmt plan absent | Project dies in month 3. Refused. |
| No measurable success criteria | Audit becomes a deck. Refused. |

---

## G. The Final Closeout

Session closes with the CAIO + executive sponsor signing off on:

```
- Engagement Mode delivered
- Stakeholder interview list completed
- Opportunity backlog reviewed + top-N approved
- Phase 0 funding committed
- Phase 1 owner named
- Quarterly governance review scheduled
- AI Usage Policy ready for company-wide distribution
```

Without all 7 signatures, the audit is a draft. Re-engage to close.

---

## H. Iron Test (90 days post-delivery)

1. Did the top-scored opportunity ship in production?
2. Did the 30-day quick wins ship (3 of 3 ideal)?
3. Did the executive sponsor receive the dashboard MVP at day 60?
4. Did the team adopt the supervised agent (production usage, not pilot)?
5. Did the ROI math hold (re-measure)?

4+ of 5 pass = the Company AI OS works. Renew + scale.

12-month test:
- Shift from "we use ChatGPT sometimes" to "we have N production agents, M dashboards, K hours saved verified, $X ROI documented"?
- Audit kicked off wave 2 (next departments) without the original CAIO present?

Yes to both = self-compounding OS. No = the audit was a deck, not a system.

---

*The CAIO Enterprise Workflow Architect closes with a system, not a slide deck.*

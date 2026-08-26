# Optimization Loop Cadence — {{client_name}} — {{quarter}}

> The cadence that keeps a delivered system compounding instead of drifting. Weekly read (triage) + monthly re-score of the architect backlog against ACTUAL data. (mm-11 — the system that compounds)

- **Prepared by:** {{caio_name}}
- **System owner (internal):** {{system_owner}}

---

## 1. Cadence

| Rhythm | When | Duration | Owner | Rule |
|---|---|---|---|---|
| Weekly read | {{day_time}} | 15 min, one screen | {{caio}} | triage only — starts no projects |
| Monthly re-score | {{date}} | — | {{caio}} | re-rank the backlog on measured numbers |

---

## 2. This month's weekly reads (log)

| Week | NSM + trend | Newest cohort holding? | Cost vs budget | Top adoption mover | Top open alert | On fire? |
|---|---|---|---|---|---|---|
| {{w1}} | {{nsm}} | {{yes/leaking}} | {{ok/drift}} | {{mover}} | {{alert}} | {{no / tweak / scoped}} |

---

## 3. Monthly re-score (actual-informed)

Re-rank the open `05-Automation-Opportunity-Backlog.md` + any falsified/leaking shipped workflows, scored on **measured** impact/frequency/adoption — not projection-time estimates.

| Rank | Item | Type | Actual-informed ICE/RICE | Retention-first? | Why it beats the next | 
|---|---|---|---|---|---|
| 1 | {{item}} | {{Tweak / Build / retention}} | {{score}} | {{YES — cohort X leaking / no}} | {{explicit reason}} |

> Retention/adoption fixes outrank new builds while ANY cohort leaks (Iron Law 5).

---

## 4. This month's ONE improvement (falsifiable hypothesis)

> **Because** {{telemetry_observation}}, **I believe** {{change}} will move {{NSM / cohort-savings-retention}} from {{current}} to {{target}}. **I'll know within** {{window}} **if** {{threshold}}.

**Verdict:** {{Tweak / Build / Expand}}
- Tweak → owner: {{who}}, due: {{date}}
- Build → routed to: {{agentic-systems-builder / agentik-skill-forge}} with F-XXX spec {{ref}}
- Expand → gate check (no leak + healthy realization) {{pass?}} → re-enter caio-enterprise-workflow-architect for next department {{which}}

---

## 5. Last month's hypothesis — result

- Hypothesis: {{prior_hypothesis}}
- Result: {{confirmed / refuted}}
- What it taught: {{learning}}
- Follow-on: {{action}}

---
*A change with a hypothesis teaches you something whatever the result. A change without one is a slot machine.*

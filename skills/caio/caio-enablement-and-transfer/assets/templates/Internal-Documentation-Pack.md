# Internal Documentation Pack — {{company}} Company AI OS

> The test for every page: could a competent client person who did NOT build this change it safely using only this document? If not, it's a black box — fix the doc, not a phone call.
> Legibility rubric target: >= 9/10 with no layer at 0.

---

## Layer 1 — System Map

**What this system is (one paragraph):** {{...}}

**Components:**
| Component | Type (agent/dashboard/automation/integration) | Owner | Serves (role) |
|---|---|---|---|
| {{name}} | {{type}} | {{owner}} | {{role}} |

**Data flows (arrows, not prose):**
```
{{source}} --read--> {{component}} --write--> {{destination}}
{{... include every read/write and every HITL gate ...}}
```

**System-of-record per data type:** {{e.g. customers = HubSpot; tickets = Zendesk; revenue = Stripe}}

**Boundaries — what this system does NOT do (refused / out-of-scope):** {{...}}

---

## Layer 2 — Per-Dashboard (one block per dashboard/report)

### Dashboard: {{name}}
- **For:** {{role}} · **Supports the decision:** {{...}} · **Owner:** {{name}}
- **Panels / metrics:**

| Panel/metric | What it shows | Source + how computed | Refresh | Good vs bad | Known-good reference value |
|---|---|---|---|---|---|
| {{metric}} | {{...}} | {{source + formula}} | {{cadence}} | {{...}} | {{value a guardian checks against — L1}} |

- **How to change it:** {{pointer → Extension Playbook, Adjust-a-Report}}

---

## Layer 3 — Per-Agent Runbook (one per agent)

### Agent: {{name}}
- **Owner:** {{name}} (+ backup {{name}}) · **Escalation:** {{agentic-systems-builder for ...}}
- **Purpose (one sentence):** {{...}}
- **Trigger:** {{schedule/event/manual}}
- **Inputs + sources + permissions:** {{... read-only vs write, least-privilege}}
- **What it does (step by step):**
  1. {{...}}
- **HITL gate (Iron Law 7):** {{what it must NOT do without approval; who approves}}
- **Guardrails + refusals:** {{...}}
- **Outputs + where they go:** {{...}}
- **Logs (observability surface):** {{where to see runs / cost / confidence / errors}}
- **Failure modes + fix:**
  - {{symptom}} -> {{fix}}
- **Rollback:** {{how to turn it off / revert safely — confirm no data loss}}

---

## Layer 4 — Code & Config Pointers

> Map "I want to change X" → "the file/config that controls X". Secrets by REFERENCE only (in the client's vault, never here — R-ENV).

- **Repo + branch:** {{...}} · **Run locally/staging:** {{command}}
- **Agents defined in:** {{path}} · **Prompts in:** {{path}} · **Guardrails in:** {{path}}
- **Integrations/connections configured in:** {{Composio/MCP/API config location}} · **Secrets location:** {{vault reference}}
- **Dashboard panels / report queries in:** {{path}}
- **"If you want to do X, change Y" table:**

| I want to... | Change... | Then... |
|---|---|---|
| {{add a ticket category}} | {{prompt file §classify}} | {{test in staging, run acceptance}} |

- **Commenting standard:** the non-obvious logic is commented in-code, in plain language, at the point a guardian will look.

---

## Layer 5 — The Evolution Process (how the team changes the system safely)

1. **Propose:** {{intake — what + why + who's affected}}
2. **Build in staging** (never prod-first): {{...}}
3. **Test:** {{the acceptance check — does the golden path still pass?}}
4. **HITL re-check:** {{does it touch a sensitive decision? then the gate stays}}
5. **Ship + rollback:** {{deploy path; how to revert}}
6. **Reviewer (named) for sensitive-surface changes:** {{name}}
7. **Update THIS pack** as part of "done" (no stale docs — L1).

---

## Legibility rubric (score before handover)
```
System Map ............... /2
Per-Dashboard ............ /2
Per-Agent runbooks ....... /2
Code & config pointers ... /2
Evolution process ........ /2
                          ----  target >= 9/10, no layer at 0
```
- **Score:** {{x}}/10 · **Black-box components remaining:** {{none / list}}

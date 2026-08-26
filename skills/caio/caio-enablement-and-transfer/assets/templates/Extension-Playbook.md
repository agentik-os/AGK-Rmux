# Extension Playbook — {{company}}

> The three things a self-sufficient team must do unaided: ADD AN AGENT, CONNECT A TOOL, ADJUST A REPORT. Each path is SIZED to your team's real technical level. Taught WATCH → GUIDED → UNAIDED → TEACH.
> Safety rails on every motion: staging first · least-privilege · HITL re-check on sensitive decisions · a known rollback · an acceptance check that proves it worked.

- **Team technical level:** {{none / config-only / can-edit-prompts / can-write-code}}
- **Owners being trained:** {{name — level}}
- **Escalation target for beyond-level work:** agentic-systems-builder (novel complex agent) / agentik-skill-forge (codify a repeatable company skill)

---

## Motion A — ADD AN AGENT

**When to use this motion (judgment — mm-12):** a NEW multi-step, judgment + tool-use task. If it's an if-this-then-that, it's an automation, not an agent (don't build overhead). If it's a sensitive decision (Class 8), it stays human.

**Your path ({{level}}):**
- config-only: {{clone template `{{template}}` via the form → pick sources {{...}} → set HITL approver {{name}} → ship to staging → run acceptance check `{{check}}`}}
- can-edit-prompts: {{+ adjust system prompt / tools / guardrails at `{{path}}`; confirm behaviour in logs (L1)}}
- can-write-code: {{+ runbook add-agent path `{{ref}}`; escalate genuinely novel agents to agentic-systems-builder}}

**Safety rails:** staging only · HITL wired before any customer/financial/legal output · acceptance check passes · rollback = `{{toggle/revert}}`.
**Acceptance check (this worked when):** {{the new agent passes `{{check}}` in staging and a human approved its first real output}}.

---

## Motion B — CONNECT A NEW TOOL

**When to use this motion:** the system needs data/actions from a tool it isn't connected to yet.

**Your path ({{level}}):**
- config-only: {{guided OAuth/Composio connection for `{{tool}}` → READ-ONLY scope → confirm it appears in dashboard logs}}
- technical: {{+ direct API/MCP integration via the runbook pattern `{{ref}}`}}

**Safety rails (every level):** read-only before write · least-privilege scopes · appears in logs (observable, not invisible) · secret stored in the client vault `{{reference}}` (never the repo/docs — R-ENV) · rollback = disconnect.
**Acceptance check:** {{a test read returns expected data AND the connection shows in `{{logs view}}`}}.

---

## Motion C — ADJUST A REPORT

**When to use this motion:** a metric is wrong, a threshold/label/schedule needs changing, or a new number is needed.

**Your path ({{level}}):**
- config-only: {{change metric/threshold/label/schedule via the dashboard config UI at `{{path}}`}}
- technical: {{+ edit the report query/aggregation at `{{path}}`; re-run}}

**Safety rails:** VERIFY the new number against runtime — never trust the new label (L1). Diff against the known-good reference value from the documentation pack.
**Acceptance check:** {{the changed report shows the provably-correct value (checked against `{{reference}}`), not just a renamed field}}.

---

## How each motion is taught (the curriculum)
1. **WATCH** — CAIO performs it once, narrating WHY at each step.
2. **GUIDED** — owner performs it, CAIO advises.
3. **UNAIDED** — owner performs a DIFFERENT real instance alone, from this playbook + the docs only. (This is the Autonomy-Readiness Gate. If they had to ask HOW → fix the docs first, then re-run.)
4. **TEACH** — owner teaches the next person (proof of mastery; seeds the volunteer army).

## The weekly guardian routine (run this after the CAIO leaves — ~1-2h)
- **Mon (30m):** read the one view — adoption, agent health (runs/cost/errors/confidence), the week's biggest friction. Decide ONE improvement.
- **Improvement session (30-60m):** make the ONE change via the evolution process (staging → acceptance → ship → update docs). One change at a time (L1).
- **Fri (15m):** confirm it held at runtime; log it; note anything to escalate.

## Keep / Automate / Delegate (mm-12)
- **Keep (you, judgment):** the decision on the numbers, the HITL approvals, the call on what to extend, the Class-8 refusals.
- **Automate:** the measurement, the agent runs, the reporting.
- **Delegate up:** novel complex agent → agentic-systems-builder; a repeatable company process → agentik-skill-forge.

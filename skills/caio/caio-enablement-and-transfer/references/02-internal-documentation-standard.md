# 02 — The Internal Documentation Standard

Phase 4 of the skill. Outputs `caio-enablement/02-Internal-Documentation-Pack.md`.

This reference defines **what every Company AI OS must have documented to be transferable** — the difference between a system the team owns and a black box they rent from the consultant who built it. Documentation is the precondition for autonomy: you cannot transfer mastery of a system nobody can see inside. This is the load-bearing artefact of the offer's Internal-Mastery principle (#3).

The standard is anchored in the same legibility doctrine as the upstream architect ("a CAIO makes the company legible first"): here we make the **system itself** legible to its owners.

---

## A. First principle — document for the future guardian, not for yourself

The test for every page in the pack is one question:

> Could a competent person on the client's team, who did NOT build this, change it safely using only this document?

If yes, it is documented. If they would have to text you, it is not — and the fix is the document, not a phone call (this is the same rule that governs the Autonomy-Readiness Gate). You are not writing reference material for yourself; you are writing the manual that makes you unnecessary.

Three failure modes the standard exists to prevent:
- **The black box.** A component that works but nobody can explain or change. The dependency the offer refuses.
- **The tribal knowledge.** The "you have to know that the refund agent ignores tickets older than 30 days" that lives only in the builder's head.
- **The stale doc.** Documentation written once at handover and never updated — worse than none, because it lies (L1 — runtime is the only truth; docs that disagree with runtime are wrong).

---

## B. The five layers every system must document

The pack has five layers. Each must exist for the system to pass the legibility check.

### B.1 The System Map (the territory)
A single diagram + page that shows, at a glance, what the system is:
```
- The components: each dashboard, each agent, each automation, each integration.
- The data flows: what reads from where, what writes to where (arrows, not prose).
- The HITL gates: where a human must approve, and who that human is.
- The system-of-record per data type (inherited from company-ai-os/04-Data-And-Permission-Map.md).
- The boundaries: what the system does and explicitly does NOT do (refused / out-of-scope).
```
The map answers "what is this, and how do the pieces fit". A new guardian reads this first. Keep it one screen; depth lives in the layers below.

### B.2 Per-Dashboard: how each dashboard works
For every dashboard/report the team uses, one page:
```
- Who it's for (which role) and the decision it supports.
- Each panel/metric: what it shows, WHERE the number comes from (source + how it's computed),
  the refresh cadence, and what "good / bad" looks like.
- The known-good reference value (so a guardian can tell a real number from a broken one — L1).
- How to change it (links forward to the Extension Playbook §C — adjust a report).
- Owner.
```
Critical: every metric states its **source and computation**, not just its label. A dashboard whose numbers can't be traced is exactly the black box the architect's Iron Law 8 refuses ("logs + costs + confidence + status surfaced").

### B.3 Per-Agent runbook (one per agent)
The most important and most often-skipped layer. For each agent, a runbook a guardian can operate and debug:
```
- Purpose: the problem it solves, in one sentence.
- Trigger: what starts it (schedule / event / manual).
- Inputs + sources + permissions (read-only vs write; least-privilege).
- What it does, step by step (the actual workflow, not a vibe).
- The HITL gate: what it must NOT do without approval, and who approves (Iron Law 7).
- Guardrails + refusals: what it's explicitly forbidden from doing.
- Outputs + where they go.
- Logs: where to see its runs, costs, confidence, errors (the observability surface).
- Failure modes + the fix for each (the on-call section — "if X, do Y").
- Rollback: how to turn it off / revert safely.
- Owner + escalation path.
```
The failure-modes + rollback sections are what turn "we have an agent" into "we can operate an agent". Without them, the first time it misbehaves, the team's only move is to call you — and the transfer failed.

### B.4 Code & config pointers (commented + accessible)
The team must be able to *find* the thing they need to change. This layer is a map from "I want to change X" to "the file/config that controls X":
```
- Repo location + branch + how to run it locally / in staging.
- Where agents are defined (file paths), where prompts live, where guardrails are set.
- Where integrations/connections are configured (Composio / MCP / API keys location — by reference,
  never the secret value itself; secrets live in the client's vault — R-ENV).
- Where dashboard panels / report queries are defined.
- The commenting standard: the critical, non-obvious logic is commented in-code, in plain language,
  at the point a future guardian will look (not in a separate doc that goes stale).
- "If you want to do X, change Y" — a lookup table for the three extension motions.
```
This is the bridge between the documentation pack and the actual codebase. The standard is not "comment everything" (noise) — it is "comment the non-obvious decisions a newcomer would get wrong", and make every changeable thing findable.

### B.5 The Evolution Process (how the team changes the system safely)
The process that lets the team change the system WITHOUT breaking it or you. This is what makes autonomy safe rather than reckless:
```
- How to propose a change (a lightweight intake — what + why + who's affected).
- Where to make it (staging first, never prod-first — mirrors the build discipline).
- How to test it (the acceptance check pattern — does the golden path still pass?).
- HITL re-check: does the change touch a sensitive decision? If so, the gate stays.
- How to ship it (the deploy path) and how to roll back.
- Who reviews (named) before a change touching a sensitive surface goes live.
- How to update THIS documentation as part of the change (so docs never go stale — the doc update
  is part of "done", not optional).
```
The evolution process is the operating manual for guardianship. It is also the thing `caio-run-and-optimize` relies on: the run phase's "1h/week + expand" loop runs *through* this process.

---

## C. The legibility rubric (score the pack before handover)

Score each layer 0-2 (0 = absent, 1 = present but a newcomer would get stuck, 2 = a newcomer could change it safely). The pack passes at **>= 9/10 with no layer at 0**.

```
System Map .................... /2   (can a newcomer explain what the system is + its boundaries?)
Per-Dashboard ................ /2   (can they trace every number to its source?)
Per-Agent runbooks ........... /2   (can they operate + debug + roll back each agent?)
Code & config pointers ....... /2   (can they FIND the thing to change?)
Evolution process ............ /2   (can they change it safely without you?)
                               ----
                               /10
```
Any layer scoring 0 is a black box; the system is not transferable until it's documented. A pack at 6/10 means "they understand it but can't safely change it" — adoption-ready, transfer-blocked.

---

## D. Worked excerpt (per-agent runbook)

```
AGENT: Tier-1 Support Triage Agent
Owner: M. (support lead) + J. (internal eng) | Escalation: agentic-systems-builder for new categories

Purpose: classify each inbound Tier-1 ticket and draft a first reply for a rep to approve.
Trigger: new Zendesk ticket (webhook) tagged tier-1.
Inputs: ticket subject+body (read), KB articles (read), customer plan (read-only from Stripe).
What it does:
  1. Classify intent (billing / bug / how-to / churn-risk) — confidence logged.
  2. Retrieve up to 3 KB articles.
  3. Draft a reply citing the articles.
  4. Place the draft in the rep's approval queue. DOES NOT send.
HITL gate (Iron Law 7): never sends a customer-facing reply without a rep clicking Approve.
  Never issues a refund or makes a billing change — those route to a human, always.
Guardrails: no promises of timelines; no legal statements; flags churn-risk to the manager.
Outputs: a drafted reply + classification in the queue. Logs: /dashboard/agents/triage (runs, cost,
  confidence, errors).
Failure modes:
  - Low-confidence classification (< 0.6) -> routes to a human with no draft. Fix: none needed; by design.
  - KB retrieval empty -> drafts a "we're looking into it" holding reply, flags for KB gap. Fix: add KB article.
  - Webhook stops firing -> no tickets triaged. Fix: check Zendesk webhook health (runbook §debug-2).
Rollback: toggle `triage.enabled=false` in dashboard config; tickets fall back to manual queue. No data loss.
```

This is what "documented to be transferable" looks like: a guardian who never built it can run it, debug the common failures, and turn it off safely — from the page alone.

---

## E. Discipline checks for this phase

| Check | Pass = |
|---|---|
| All five layers present (map, per-dashboard, per-agent, code/config pointers, evolution process) | yes |
| Every dashboard metric states its source + computation, not just a label | yes |
| Every agent has a runbook incl. failure-modes + rollback + HITL gate | yes |
| Code/config pointers let a newcomer FIND the thing to change; secrets by reference only (R-ENV) | yes |
| The evolution process makes docs-update part of "done" (no stale docs) | yes |
| Legibility rubric scored >= 9/10 with no layer at 0 | yes |
| No black-box / tribal-knowledge component remains undocumented | yes |
| Documentation verified against runtime (no doc that disagrees with the live system — L1) | yes |

A system that fails this standard cannot be transferred — only rented. Re-document before opening Phase 5 (the extension curriculum), because the curriculum teaches *from* this pack.

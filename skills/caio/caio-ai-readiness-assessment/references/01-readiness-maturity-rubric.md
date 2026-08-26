# 01 — The 9-Dimension AI-Readiness Maturity Rubric

Phase 1 of the CAIO AI-Readiness Assessment. This is the **core instrument**. It feeds `caio-readiness/AI-Readiness-Scorecard.md`.

You score a company on **9 dimensions, each 0-4**, using evidence from three sources only (THE FENCE): the company context, what the C-level says on the qualification call, and a 2-minute public-site scan. Every level you assign **cites the evidence that proves it** (R-CITE). When you cannot evidence a level, you mark it `_(unverified — confirm on call)_` and score conservatively (the lower of the two plausible levels).

> **Altitude reminder.** You score *only deeply enough to decide go/no-go + indicative investment*. The detailed map of which endpoint, which auth, which schema, which PII field — **deferred to E2** (`caio-enterprise-workflow-architect`), on real access, after signature. If you find yourself wanting to document an integration in detail, you have left qualification altitude. Stop and write "to be produced in the engagement (E2)".

---

## How to read a 0-4 anchor

```
0  Absent      — the thing does not exist. A hard blocker if this is a gate dimension.
1  Nascent     — a faint signal, individual / accidental / shadow, nothing institutional.
2  Emerging     — it exists but is partial, fragile, or undocumented. The minimum "workable" bar.
3  Established  — solid, repeatable, owned by someone, good enough to build on.
4  Leading      — exemplary; a genuine asset that accelerates the engagement.
```

**The hard-gate floors** (see `03-go-no-go-decision-tree.md`): dim 1 ≥ 2, dim 2 ≥ 2, dim 3 ≥ 1, dim 5 ≥ 1, dim 9 ≥ 1. A dimension scored below its floor is a NOT-YET (if fixable) or a REDIRECT (if not).

---

## Dimension 1 — Leadership & executive sponsorship  ·  weight 18%

**What it gates:** whether the engagement *survives*. This is the single strongest predictor; a sponsorless build dies in week 2. It is also where you qualify mm-10's **authority objection** ("I need to talk to X") — you find out who decides *during the diagnostic*, never after.

| Level | Anchor | Evidence that proves it |
|---|---|---|
| 0 | No executive cares; the inquiry came from a curious IC with no mandate. | "I thought I'd look into AI for us" from someone who can't name a budget. |
| 1 | An exec is interested but has no budget authority and no time committed. | Enthusiasm on the call, but "I'd have to convince the CEO / get sign-off". |
| 2 | A named exec sponsor with budget authority and a stated "why now". **(floor)** | "I'm the COO, I own this budget, and we need it because [trigger]." |
| 3 | Sponsor + a second exec aligned; sponsor will commit calendar time weekly. | Two C-levels on the call; sponsor blocks a weekly slot for the engagement. |
| 4 | CEO-level sponsorship, an internal owner pre-named, AI on the company OKRs. | "AI legibility is a board priority this year; here's the internal owner." |

**Probe (mm-10 — qualify authority early):** *"Who, besides you, has to say yes to this? And whose budget does it come out of?"* A "yes" that needs three other people's blessing is a Level 1, not a Level 3 — score it honestly.

---

## Dimension 2 — Tooling & API-exposure  ·  weight 14%

**What it gates:** the **entire centralized federated model**. The approach wires existing tools into one legible AI surface; if the core systems are closed boxes with no API and no path, there is nothing to wire. This is the most *architecture-decisive* dimension at the gate — and the one operators most often over-score on hope.

| Level | Anchor | Evidence that proves it |
|---|---|---|
| 0 | Core work lives in closed/proprietary/on-prem tools with **no API**. | "It's a custom in-house system, there's no way to connect to it." |
| 1 | Some tools have APIs, but the *system of record* for the beachhead does not. | The CRM has an API, but quotes live in an Excel nobody can reach programmatically. |
| 2 | Core tools expose usable APIs, OR a documented integration path exists (MCP/Composio/connector). **(floor)** | "We're on Salesforce + Slack + Notion, all have APIs." (fragile/legacy APIs pass at 2 with a Gap-To-Target flag.) |
| 3 | Most core tools have clean, documented, OAuth-grade APIs; some already integrated. | A working Zapier/Make flow exists between two systems today. |
| 4 | A modern, API-first stack with a warehouse or integration layer already in place. | "Everything flows into Snowflake; we have an internal API gateway." |

**Altitude guard:** at the gate you note *that* a tool appears to expose an API (from what the buyer says + the vendor being a known SaaS). You do **not** verify endpoints, auth, or rate limits — **"endpoint/auth/rate-limit verification to be produced in E2"**. A self-reported API is Level 2 at most until E2 confirms it.

---

## Dimension 3 — Strategy & use-case clarity  ·  weight 12%

**What it gates:** whether there is a **beachhead** to build (mm-02: the defensible niche-first wedge). "We want AI" is not a use case. One painful, frequent, namable workflow is.

| Level | Anchor | Evidence that proves it |
|---|---|---|
| 0 | "We should do something with AI" — no specific problem named. | The buyer can't finish the sentence "the thing that's killing us is…". |
| 1 | One painful workflow is namable, even if vague. **(floor)** | "Quote assembly eats our ops team's mornings." |
| 2 | One sharp, frequent, painful beachhead, with rough frequency/cost. | "Quote assembly: ~9h/week across 2 managers, every day, error-prone." |
| 3 | A clear beachhead **plus** 2-3 credible adjacent expansions. | The beachhead + "then renewals, then onboarding" — a believable sequence. |
| 4 | A prioritized problem set tied to a business objective and a why-now. | "Cut quote turnaround to win back RFQs we're losing to [competitor]'s AI bot." |

**mm-03 lens:** the use case is the buyer's **job-to-be-done**. Write it in the JTBD format: *"When [trigger/circumstance], help me [progress], so I can [emotional/social benefit]."* A use case you can phrase as a job is real; one you can only phrase as a feature ("we want a chatbot") is not yet a Level 2.

---

## Dimension 4 — Data readiness  ·  weight 12%

**What it gates:** whether a *pilot can even read the data* the beachhead needs. Not a full data audit (that's E2) — just: for the **one beachhead**, is the data reachable and clean enough to start?

| Level | Anchor | Evidence that proves it |
|---|---|---|
| 0 | Beachhead data lives in PDFs, email threads, and people's heads. | "The quote details? They're in the emails and whatever the manager remembers." |
| 1 | Data exists but is scattered across 3+ places with mismatches. | Quote data split across TMS + Excel + CRM, partly duplicated/inconsistent. |
| 2 | A single, reachable source for the beachhead, mostly clean. | "It's all in HubSpot, fields are mostly filled." |
| 3 | Clean, structured, accessible data for the beachhead + good hygiene elsewhere. | A documented schema; obvious owner; few nulls. |
| 4 | A governed data layer (warehouse/lakehouse) with quality monitoring. | "Snowflake + dbt tests; data quality is tracked." |

**Important — data readiness is NOT a hard gate.** A low score here does not block a GO; it becomes a **Gap-To-Target item** (consolidate the one source the beachhead needs, as the first move inside Phase 1). It only blocks if the data is so absent that *even a pilot cannot start* — and then the honest move is often a **REDIRECT to a data engineer first** (Iron Law 8). The *deep* data-and-permission map is E2's deliverable, not yours.

---

## Dimension 5 — Governance, risk & compliance  ·  weight 11%

**What it gates:** whether you are *allowed* to build at all. A regulated sector with an unresolved hard "no" is a NOT-YET or REDIRECT regardless of every other score.

| Level | Anchor | Evidence that proves it |
|---|---|---|
| 0 | A hard regulatory blocker with no path (e.g. air-gapped + legal no-cloud-LLM, no on-prem option in ~6 months). | "Legal won't allow any data to touch a cloud model, full stop." → likely REDIRECT. |
| 1 | Regulation applies (GDPR/sector) but a path exists; nobody owns it yet. **(floor)** | "GDPR obviously, but we haven't really thought about AI + data." |
| 2 | Constraints mapped, a DPO/owner exists, HITL acceptable for sensitive steps. | "We have a DPO; HITL on anything customer-facing is fine." |
| 3 | Mature posture: SOC2 in progress, data-residency known, vendor review process. | "SOC2 Type II in audit; we vet sub-processors." |
| 4 | AI-specific governance already forming (EU AI Act awareness, model/vendor policy). | "We've started an AI usage policy and risk register." |

**Altitude guard:** you assess *whether there is a path*, not the full control set. The AI usage policy, the HITL matrix, the vendor-risk audit are **E2 deliverables** — "to be produced in the engagement". You only need enough to know the build is *permitted* and *not blocked*.

---

## Dimension 6 — Culture & change-appetite  ·  weight 10%

**What it gates:** whether **adoption** will actually happen. Score this **against mm-03's 4 forces of progress** — this is the dimension where the doctrine is most load-bearing. (Full scoring method in `03-go-no-go-decision-tree.md` §4-forces.)

| Level | Anchor | Evidence that proves it |
|---|---|---|
| 0 | Active hostility / fear; "AI will take our jobs" is the loud internal narrative. | "My team thinks this is about replacing them." (Habit + Anxiety dominate.) |
| 1 | Indifference; no push, no pull; "we'll see". | Shrug energy — the status quo feels fine, no trigger. |
| 2 | Net-curious; a real push (a trigger) and some pull, anxiety manageable. | "A competitor moved and people are nervous we're behind." |
| 3 | Enthusiastic pockets; an internal champion; change muscle from a prior rollout. | "We rolled out [tool] last year and it stuck." |
| 4 | Change-hungry culture; experiments encouraged; leadership models it. | "People already use AI and want it blessed and connected." |

**The 4-forces read (mandatory, drives the verdict):**
- **Push** (toward change): the pain of today — a competitor's AI move, a manual process people hate, AI-anxiety of falling behind (mm-01 why-now).
- **Pull** (toward you): the appeal of a legible, federated, owned AI OS and a trusted CAIO.
- **Anxiety** (away): "what if the build fails / data leaks / vendor lock-in / my team resists".
- **Habit** (away): "we already stack SaaS / personal ChatGPT — it's fine" (mm-03: the most underrated force).

Change is viable only when **(Push + Pull) > (Anxiety + Habit)**. If habit + anxiety dominate, even a high-index company is a NOT-YET — "build the push first".

---

## Dimension 7 — Talent & AI-literacy  ·  weight 8%

**What it gates:** how much **transfer-to-autonomy** (Phase 4) will cost. Low literacy doesn't block a GO — it lengthens the enablement tail and raises the member count.

| Level | Anchor | Evidence that proves it |
|---|---|---|
| 0 | Zero AI literacy; nobody could own anything post-transfer. | "Nobody here knows what an API is, honestly." |
| 1 | Shadow users only — a couple of people on personal ChatGPT. | "Two of my managers use ChatGPT on the side." |
| 2 | A semi-technical person who could become the internal champion. | "Our ops lead is sharp and curious about this." |
| 3 | An engineer or data person in-house who can co-build and maintain. | "We have a developer who could take this on." |
| 4 | An internal AI/automation capability already exists. | "We have someone doing automations today." |

Shadow IT (personal ChatGPT, a self-built sheet) is a **positive** signal here — it proves latent appetite (mm-03 pull) — even as it's a governance flag for E2. Note it as "shadow-IT present, map in E2".

---

## Dimension 8 — Infrastructure  ·  weight 8%

**What it gates:** build feasibility. Mostly cloud-vs-on-prem and whether an AI layer can sit on top.

| Level | Anchor | Evidence that proves it |
|---|---|---|
| 0 | Air-gapped / fully on-prem legacy; no path to a hosted AI layer. | "Everything is on a server in our basement, no internet egress." |
| 1 | Heavily on-prem with limited connectivity; a hard hybrid. | Legacy ERP on-prem, little cloud. |
| 2 | Hybrid cloud/on-prem that can support an AI layer on top. | "CRM in the cloud, one legacy system on-prem." |
| 3 | Mostly cloud SaaS; an AI layer is straightforward to host. | "We're all-SaaS." |
| 4 | Modern cloud-native with a warehouse and clear extension points. | "Cloud-native, warehouse, internal platform team." |

**Adaptation note (not a refusal):** an existing ERP/CRM (SAP/Oracle/Salesforce) stays the system of record; the AI layer sits **on top**. Air-gapped/regulated → the build swaps to on-prem/private-LLM options — but that's an *E2 stack decision*, flagged here only as "constrained stack — confirm in E2".

---

## Dimension 9 — Budget & commitment  ·  weight 7%

**What it gates:** whether they will actually *start*. The **monthly-no-minimum** model deliberately lowers this bar (it de-risks the buyer — mm-03 anxiety reducer, mm-08 pricing). So the floor here is low (≥1): you need willingness to start one month, not a year's PO.

| Level | Anchor | Evidence that proves it |
|---|---|---|
| 0 | No budget, no authority, tyre-kicking / "just exploring". | "We have no budget for this right now." |
| 1 | Willing to try a short paid pilot at the monthly grid. **(floor)** | "We could start with a couple of months and see." |
| 2 | Budget identified for a multi-month engagement. | "I've got budget for a Q3 engagement." |
| 3 | Committed budget + a decision timeline. | "Sign this month, start next." |
| 4 | Budget + intent to expand if the pilot works. | "If the beachhead lands, we roll it to the next department." |

**mm-10 grounding:** because the ACV clears €2,000/year many times over, this is sales-led — the *price is not the objection to fear*, **trust is** (mm-10: trust is the conversion bottleneck). A low budget score is rarely the real blocker; an under-developed Push (dim 6) usually is. Don't over-weight a price wobble that's really a trust or why-now gap.

---

## Filling the scorecard (output discipline)

For **each** dimension, write into `AI-Readiness-Scorecard.md`:

```
### D{n} {Dimension name} — Level {0-4}  (weight {w}% → contributes {w×level/4} to the Index)

Evidence:
- (call) "{verbatim quote}"  [source: qualification call, {date}]
- (scan) {observation from the public site, or _(not provided)_}

Read: {1-2 lines — what this level means for the build}
Gate: {if a gate dimension: PASS/FAIL vs its floor}
Defer to E2: {the detailed mapping you are NOT doing here}
```

A dimension with a level but no cited evidence line = a guess. Mark it `_(unverified — confirm on call)_` and score the conservative level. The whole instrument only works if every number traces to something said or seen (R-CITE) — and the false reads will surface later as stalled engagements (the Iron Test).

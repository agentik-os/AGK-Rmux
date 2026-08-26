# CAIO AI-Readiness Assessment

> The pre-sign front gate of a Chief-AI-Officer engagement — the whitepaper's honest 30-minute discovery call. Map a company against a 9-dimension AI-Readiness Maturity Model, decide GO / NOT-YET / REDIRECT honestly, shape the engagement, and anchor the indicative investment — all BEFORE anything is signed.

> A good CAIO disqualifies more leads than they close. The qualification gate that never says "not yet" is a sales pitch wearing a lab coat.

Built by [Agentik OS](https://agentik-os.com). The step-1 front gate of the CAIO accompaniment suite — composes with [caio-discovery-interview](https://skills.agentik-os.com/caio-discovery-interview), [caio-enterprise-workflow-architect](https://skills.agentik-os.com/caio-enterprise-workflow-architect), and [market-proposal](https://skills.agentik-os.com/market-proposal).

---

## What it produces

`./caio-readiness/` — five artefacts:

1. **AI-Readiness-Scorecard.md** :: 9 dimensions × (0-4 maturity level + cited evidence) + the weighted Readiness Index /100 + maturity tier + the 4-forces read.
2. **Go-No-Go-Brief.md** :: a 1-page exec read — the verdict (GO / NOT-YET / REDIRECT) + the honest reasoning + the next step. Built to be the sponsor's internal-sell artefact.
3. **Recommended-Engagement.md** :: which of the 5 phases to start, 1-3 months, member count, indicative investment from the real grid, and the handoff to `/market-proposal`.
4. **Gap-To-Target-Plan.md** :: what to fix before/early in the engagement to de-risk it (on a GO) — or the clearance items to flip a failed gate and re-qualify (on a NOT-YET).
5. **metadata.json** :: machine-readable header (verdict, index, tier, gates, indicative €) for the CRM / pipeline.

## When to use it

Use it **BEFORE committing to a build**, when a C-level has expressed interest and you need to decide — honestly — whether their case fits the approach. Triggers: "AI readiness assessment", "qualify this client", "go/no-go", "is this company AI-ready", "should we take this client", "discovery call"; FR: "évaluation de maturité IA", "qualifier ce client", "diagnostic avant pitch", "doit-on prendre ce client".

**Do NOT use it for** the in-engagement technical audit (that's `caio-enterprise-workflow-architect`) or the per-person discovery interview (that's `caio-discovery-interview`) — those run AFTER the SOW is signed.

## The chain position (pre-sign front gate)

```
(a C-level expresses interest)
            │
            ▼
  caio-ai-readiness-assessment  → ./caio-readiness/   ← THIS SKILL (pre-sign)
            │
   GO ──────┼────── NOT-YET ──────── REDIRECT
   ▼                ▼                  ▼
/market-proposal   re-qualify later   the named alternative
(signed SOW)       with the gap plan  (a point SaaS / a data engineer /
   │                                   an internal hire / a compliance partner)
   ▼
THE ENGAGEMENT BEGINS:
  caio-discovery-interview → caio-enterprise-workflow-architect (E2) →
  caio-implementation-runbook → caio-enablement-and-transfer → caio-run-and-optimize → (loops)
```

## The fence (what makes this a gate, not an audit)

This skill stays at **qualification altitude**. It scores each dimension only deeply enough to decide go/no-go + indicative investment, and **defers all detailed technical mapping to E2** (the engagement audit). It reads **only** pre-engagement inputs (company context + the qualification call + a public-site scan); it never reads the discovery rollup or `company-ai-os/`, and it never produces the Tool-And-Integration-Map / Data-And-Permission-Map / opportunity backlog. It estimates **indicative investment** (a price), never **ROI** (a return — that comes from in-engagement baselines).

## The 9-dimension maturity model

| # | Dimension | Weight | Gates |
|---|---|---|---|
| 1 | Leadership & executive sponsorship | 18% | G1 (≥2) — engagement survival |
| 2 | Tooling & API-exposure | 14% | G2 (≥2) — the centralized federated model |
| 3 | Strategy & use-case clarity | 12% | G3 (≥1) — a beachhead to build |
| 4 | Data readiness | 12% | (risk dial) |
| 5 | Governance, risk & compliance | 11% | G4 (≥1, no blocker) — permission to build |
| 6 | Culture & change-appetite | 10% | (4-forces, mm-03) |
| 7 | Talent & AI-literacy | 8% | (transfer-cost dial) |
| 8 | Infrastructure | 8% | (feasibility dial) |
| 9 | Budget & commitment | 7% | G5 (≥1) — willing to start |

`Readiness Index = Σ (weight × level/4)` → Nascent (0-25) / Emerging (26-50) / Ready (51-75) / Leading (76-100). **The index informs; the hard gates decide.**

## The verdict logic

```
REDIRECT  (you don't need us, you need Y)   beats
NOT-YET   (fix X first, then re-qualify)    beats
GO        (all gates pass + a beachhead + net-positive 4-forces)
```

On GO → hand to `/market-proposal`. On NOT-YET → the Gap-To-Target plan + a re-qualify trigger. On REDIRECT → the named alternative.

## The real pricing grid (anchored, never improvised)

```
€2,500 setup (one-time)  +  €2,500 / member / month  ·  monthly, NO minimum
Indicative = €2,500 + (€2,500 × members × months)
```

A continuing 3-member engagement ≈ €90,000/yr ACV ≫ €25k → **unambiguously sales-led** (mm-10) — which is exactly why this human, honest qualification gate exists. The gate anchors the indicative number; the SOW (`market-proposal`) settles and closes it.

## Doctrine grounding (load-bearing, not footnotes)

- **mm-10 (selling — primary lens):** the sales-led ACV makes the whole gate mm-10's "diagnostic avant pitch" discovery call — step 1 of the 5-step process, with real disqualification and a feel-felt-found objection bank (price / time / trust / need / authority).
- **mm-03 (JTBD + 4 forces):** the go/no-go surfaces push / pull / anxiety / habit; the change-appetite dimension is scored against the four forces, and the beachhead is written as a job-to-be-done.
- **mm-02 (positioning — applied, not re-derived):** the gate uses the offer's position vs the two alternatives the buyer compares (generic SaaS-stacking, the black-box agency) — most sharply in the honest REDIRECT logic.
- **mm-01 (2026 window):** the strategic window is the grounded, honest "why now" (opportunity cost), used to qualify urgency without manufacturing it.

## Composability

| Direction | Contract |
|---|---|
| Reads | pre-engagement inputs only: company context + the qualification call + a public-site scan (+ optionally the CAIO's own `vision-os/` / `personal-os/`) |
| Writes | `./caio-readiness/` (5 artefacts) |
| Composes with | `caio-discovery-interview`, `caio-enterprise-workflow-architect`, `caio-implementation-runbook`, `caio-enablement-and-transfer`, `caio-run-and-optimize`, `market-proposal` |
| Depends on | none (runs cold on a fresh lead) |

## Installation

```bash
bash <(curl -sL https://skills.agentik-os.com/install) caio-ai-readiness-assessment
```

Then in Claude Code:

```
/caio-ai-readiness-assessment
```

## What it refuses

- A GO without a named, budget-holding sponsor (G1) or an API path (G2)
- An invented ROI / time-saved / return number (returns come from in-engagement baselines)
- Producing a technical map / data map / opportunity backlog (that's E2)
- A dimension level with no cited evidence
- Off-grid pricing to "win" a deal
- Keeping a lead whose real need is one SaaS / a data engineer / a hire / a lawyer (REDIRECT honestly)
- "AI will transform your business" magic
- Re-deriving the offer's positioning (it's owned upstream)

## Iron Test (was the verdict right?)

For a GO: did the sponsor stay engaged through Phase 1? did the beachhead pain survive discovery? did the tools expose the APIs E2 needed? did the SOW land within ±20% of the indicative number? For a NOT-YET/REDIRECT: did the company actually lack the gate flagged — or did they succeed anyway (over-caution)? A gate that says GO to everyone is a pitch; one that says NO to everyone is cowardice. Calibrate to the truth.

## License

MIT.

---

*Version 1.0.0 :: the honest front gate — diagnose before you propose, disqualify without flinching, and hand only the real fits to the engagement.*

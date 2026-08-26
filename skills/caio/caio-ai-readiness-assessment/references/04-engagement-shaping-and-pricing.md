# 04 — Engagement Shaping & Indicative Pricing

Phase 4 of the CAIO AI-Readiness Assessment (runs only on a **GO**). Turns the scorecard + tier into a **recommended engagement shape** and an **indicative investment** anchored to the real grid. Feeds `caio-readiness/Recommended-Engagement.md` and hands to `/market-proposal`.

> **You shape and you anchor a price — you do NOT write the SOW.** The SOW (scope, terms, signatures) is `market-proposal`'s job, downstream. Here you produce the *shape* and an *indicative* number so the buyer knows roughly what they're committing to before the proposal. The price is **a price, never a return** — no ROI here (Iron Law 3). The ROI projection is produced *in the engagement* (E2, the architect) from a measured baseline.

---

## 1. The real grid (state it verbatim — never improvise a number)

```
Setup (one-time)   : €2,500
Recurring          : €2,500 / member / month
Terms              : monthly, NO minimum commitment (the buyer can stop after any month)
```

**Why this grid is honest and de-risking (mm-08-aware):**
- **Value-anchored, not cost-plus** (Hormozi via mm-10): the price reflects the value of a legible, owned AI surface, not your hours. You do not discount to win — if the value isn't there, that's a NOT-YET/REDIRECT (ref 03), not a cheaper number.
- **Monthly, no minimum** = the single biggest **anxiety reducer** (mm-03): the buyer risks one month, not a year. This is a deliberate pricing-as-positioning move (mm-02: the de-risked, legible alternative to the lock-in black-box agency).
- **The setup fee anchors commitment** (mm-10): a free pilot attracts tyre-kickers; a €2,500 setup filters for a real sponsor (which is exactly what G1 tests).

**mm-08 boundary.** The full pricing/monetization *strategy* (packaging, expansion, discount policy) is mm-08's doctrine — you are *mm-08-aware*, you apply the published grid, you do not re-derive a pricing model here. Off-grid numbers are refused (ref 03, Objection 1).

---

## 2. Defining "member" (the billing unit)

A **member** is a person the engagement *covers*: a C-level or a team member **whose work enters the federated AI topology** — each gets or uses an AI surface, and each is a node the CAIO makes legible, wires, and (eventually) hands back.

```
Count a member when the person:
  • is a stakeholder in the beachhead workflow (their work is being made legible/automated), OR
  • will operate / supervise an AI surface in the engagement, OR
  • is the sponsor / internal owner being enabled toward autonomy

Do NOT count:
  • the whole headcount of a department that the beachhead only touches at the edges
  • people who are merely "interested" but not in the topology
```

**Rule of thumb for the indicative count:** the **sponsor + the stakeholders of the beachhead workflow**. Meridian (ref 02): COO + 2 ops managers = **3 members**. The final count is settled in the SOW (`market-proposal`) once the discovery (`caio-discovery-interview`) confirms who's actually in scope — here you give the *indicative* count, flagged as such.

---

## 3. The indicative formula

```
Indicative engagement = €2,500 + ( €2,500 × N_members × M_months )
                        └ setup ┘  └────── recurring over the engagement ──────┘

Always present BOTH:
  • the one-time + monthly breakdown (what they actually sign up to)
  • the engagement total over the indicative duration (what it sums to)
And state the duration and member count are INDICATIVE, settled in the SOW.
```

### Worked indicative numbers (prices, not returns)

```
Pilot, 1 department   : 3 members × 2 months
  = €2,500 + (€2,500 × 3 × 2) = €2,500 + €15,000 = €17,500
  presented as: "€2,500 setup + €7,500/month for ~2 months (€17,500 indicative)"

Full engagement       : 5 members × 3 months
  = €2,500 + (€2,500 × 5 × 3) = €2,500 + €37,500 = €40,000
  presented as: "€2,500 setup + €12,500/month for ~3 months (€40,000 indicative)"

Lean toe-dip          : 2 members × 1 month
  = €2,500 + (€2,500 × 2 × 1) = €2,500 + €5,000 = €7,500
  presented as: "€2,500 setup + €5,000 for the first month, then continue or stop"
```

### The ACV grounding (mm-10 — why this gate exists)

```
A 3-member engagement that continues a year:
  €2,500 × 3 × 12 = €90,000 / year ACV

mm-10 rule: ACV < ~€2,000/yr → PLG (product sells itself).
            ACV > ~€25,000/yr → pure sales-led (a human must close it).

€90,000 ≫ €25,000  →  UNAMBIGUOUSLY SALES-LED.
```

That single line is the reason this whole skill exists: a sales-led ACV cannot be closed by a signup flow — it needs a human, honest, diagnostic conversation. The readiness gate **is** that conversation (mm-10's founder-led discovery, step 1 of the 5-step process). The price doesn't just inform the buyer; it *proves the motion*.

---

## 4. Engagement shape by tier (gates always override)

The tier sets the *default* shape; the scorecard's weak spots adjust it; the hard gates can still veto (a failed gate means you're not in this section at all — you're in NOT-YET/REDIRECT).

```
Tier      Start phase                                  Duration   Members (indicative)   Notes
--------  -------------------------------------------  ---------  ---------------------  -----------------------------
Emerging  Phase 0 gap-fix sprint → Phase 1 immersion   2-3 mo     1-3                    bake the Gap-To-Target fixes
          (26-50)                                                                        into a paid Phase 0 sprint
Ready     Phase 1 immersion → Phase 2 architecture     1-2 mo     2-5                    the standard engagement
          (51-75)                                                                        (the worked example)
Leading   Phase 1 light → straight to Phase 2/3        1 mo       3-8 (or a pilot dept)  move fast; skip heavy immersion
          (76-100)
```

**The 5 phases of the engagement** (which you're recommending a *start* into — you do not run them):
1. **Discovery / immersion** — `caio-discovery-interview` (per-person dossiers + rollup)
2. **Architecture / audit** — `caio-enterprise-workflow-architect` → `company-ai-os/` (this is **E2**, where all your deferred technical mapping gets produced properly)
3. **Implementation** — `caio-implementation-runbook` (realize the federated topology, then build)
4. **Enablement + transfer** — `caio-enablement-and-transfer` (adoption → transfer to autonomy)
5. **Run + optimize** — `caio-run-and-optimize` (measure ROI, optimize, expand) → loops to #2

**Shape adjustments from the scorecard (not the tier):**
- **Data readiness (dim 4) low** → add a data-consolidation Gap-To-Target item as the *first move* in Phase 1 (Meridian). Doesn't change tier; does shape the first weeks.
- **Talent/AI-literacy (dim 7) low** → lengthen the Phase 4 enablement tail; possibly add a member (the internal owner) to be enabled.
- **API-exposure (dim 2) at the floor (=2) with a fragile API** → flag a **technical de-risk task for E2** in Phase 2 ("validate the [tool] API before committing the build").
- **Tier Emerging** → the shape *starts* with a paid Phase 0 gap-fix sprint (one month, 1-2 members) that clears the soft gaps, then flows into Phase 1.

---

## 5. The handoff to /market-proposal (the only legal next step on a GO)

`Recommended-Engagement.md` ends with an explicit handoff block:

```
HANDOFF → /market-proposal
  Verdict          : GO
  Engagement shape : start Phase {1|0→1|2/3}, ~{1-3} months
  Members (indic.) : {N} ({list the roles})
  Indicative €     : €2,500 setup + €{2,500×N}/month (~€{total} over {M} months)
  Beachhead        : {the one workflow the pilot starts on}
  Gap-To-Target    : {the de-risk items the SOW should reference, from Gap-To-Target-Plan.md}
  Position to hold : the legible/federated/owned approach (mm-02) — not SaaS-pile, not black box
  NOTE: member count + duration are INDICATIVE; the SOW settles them after caio-discovery-interview.
```

You **stop here**. You do not draft scope language, terms, or signature blocks — `market-proposal` owns the SOW. You also do not run discovery or the audit — those begin *after* the signature. (R-KARPATHY: hand off, don't re-implement.)

---

## 6. Pricing & shaping discipline checklist

| Check | Pass = |
|---|---|
| The grid stated verbatim (€2,500 setup + €2,500/member/month, monthly no-minimum) | yes |
| Member count derived from the beachhead stakeholders (sponsor + workflow owners), flagged indicative | yes |
| Indicative total computed with the formula (both breakdown + total shown) | yes |
| Duration + start phase set from the tier, adjusted by the scorecard weak spots | yes |
| No ROI / return / payback number anywhere (that's E2's, from a baseline) | yes |
| No off-grid / discounted number to "win" the deal | yes |
| Explicit handoff block to /market-proposal | yes |
| The mm-02 position to hold is carried into the handoff | yes |

The indicative number is a *commitment preview*, not a quote. It exists so the buyer walks away knowing the rough size of the yes — and so the SOW (`market-proposal`) starts from a number both sides already understand. Honest pricing at the gate is the same trust play as honest qualification: it's what makes a sales-led, recurring, multi-member engagement actually close and stick.

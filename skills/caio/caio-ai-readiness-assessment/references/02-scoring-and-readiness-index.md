# 02 — Weighting, Scoring Method & Worked Example

Phase 2 of the CAIO AI-Readiness Assessment. Turns the nine 0-4 dimension levels (from `01-readiness-maturity-rubric.md`) into one **Readiness Index /100**, a **maturity tier**, and the inputs the decision tree (`03`) needs.

> **State the weights every time.** Transparency is the trust the whole offer sells (mm-02 — the legible approach vs the black-box agency). A buyer who can see exactly how the number was built trusts the verdict — and the price — that follow.

---

## 1. The weights (and why)

```
Dimension                              Weight   Role
-------------------------------------  ------   ----------------------------------------
1. Leadership & executive sponsorship   18%     HARD GATE — strongest survival predictor
2. Tooling & API-exposure               14%     HARD GATE — gates the centralized model
3. Strategy & use-case clarity          12%     HARD GATE (low floor) — needs a beachhead
4. Data readiness                       12%     risk dial (not a gate)
5. Governance, risk & compliance        11%     HARD GATE — permission to build
6. Culture & change-appetite            10%     adoption dial (4-forces, mm-03)
7. Talent & AI-literacy                  8%     transfer-cost dial
8. Infrastructure                        8%     feasibility dial
9. Budget & commitment                   7%     start dial (low floor — monthly no-minimum)
-------------------------------------  ------
                                TOTAL  100%
```

**Why this distribution.** The three highest-weighted dimensions (1, 2, 5 = 43% combined) are precisely the **hard gates** of the centralized federated approach. A company can be strong everywhere else and still be a NOT-YET if it lacks a sponsor, an API path, or compliance permission — so the index is *built* to sag when a gate dimension is weak. Dimensions 4/6/7/8 are dials that shape the *engagement* (duration, member count, Gap-To-Target items) more than the *verdict*. Dimension 9 is weighted *low on purpose*: the monthly-no-minimum grid removes most budget friction, so budget is rarely the real blocker (mm-10: the real bottleneck is trust, not price).

**Operator override (document it).** The CAIO may re-weight for a specific buyer (e.g., a heavily regulated target → push Governance to 15%, pull from Infrastructure). If you re-weight, the new weights **must still sum to 100**, and you write the override + reason at the top of the scorecard. Default weights are the starting point, not a religion.

---

## 2. The formula

```
Readiness Index (0-100) = Σ ( weight_i × level_i / 4 )      for i = 1..9

  where level_i ∈ {0, 1, 2, 3, 4}
  and   Σ weight_i = 100

Each dimension contributes at most its full weight (when level = 4, level/4 = 1).
Each dimension contributes 0 when level = 0.
```

So a dimension at level 2 (the "workable" midpoint) contributes **half** its weight; at level 3, three-quarters; at level 4, all of it. The index is the weighted percentage of "maturity attained vs maximum".

---

## 3. The maturity tiers

```
Index      Tier        Default posture
---------  ----------  --------------------------------------------------------------
 0 - 25    Nascent     Not ready. Usually NOT-YET or REDIRECT. Even if gates pass, the
                       gaps are too deep to fix inside a 1-3 month engagement.
26 - 50    Emerging    Fixable fundamentals. GO-with-a-Phase-0-gap-sprint, or NOT-YET if a
                       hard gate is below floor.
51 - 75    Ready       A real engagement. GO (gates permitting). Start Phase 1 immersion.
76 - 100   Leading     Move fast. GO with light immersion, straight to Phase 2/3.
```

**The tier informs; the gates decide.** Never let a Ready/Leading index talk you past a failed hard gate (Iron Law 7). Conversely, a high-Emerging company that clears all five gates *is* a GO — with a gap-fix sprint baked into Phase 0.

---

## 4. Worked example (full numeric walk-through)

### The lead (pre-engagement inputs only)

> **Meridian Logistics** — 140-person freight-forwarding SMB, France (EU), GDPR mandatory, no SOC2. Stack: Salesforce CRM (modern API), a legacy on-prem TMS (transport management system, fragile SOAP API), heavy Outlook + Excel, Slack, no central AI. Two ops managers use **personal ChatGPT** for quote wording. **Sponsor: the COO** — holds the budget, on the call. **Why now (the trigger):** a competitor launched an AI quote bot last month and Meridian is losing RFQs on turnaround time. **Stated objective:** cut the ~9h/week the ops team spends manually assembling freight quotes.

### Scoring (0-4, each cited)

```
D1 Leadership & sponsorship      = 3   COO named, owns budget, sharp why-now (competitor RFQ loss)
D2 Tooling & API-exposure        = 2   Salesforce/Slack APIs solid; TMS only a fragile SOAP API;
                                       quote data partly in Excel → passes floor, flag the TMS risk
D3 Strategy & use-case clarity   = 3   one sharp beachhead (AI-assisted quote assembly): frequent,
                                       painful, namable, tied to a business objective
D4 Data readiness                = 1   quote data scattered across TMS + Excel + email; no single source
D5 Governance & compliance       = 2   GDPR applies, DPO exists, HITL acceptable, no hard blocker;
                                       data-residency to confirm in E2
D6 Culture & change-appetite     = 2   net-curious: strong Push (competitor) + COO Pull, manageable anxiety
D7 Talent & AI-literacy          = 1   no internal AI skill; 2 shadow ChatGPT users (latent appetite)
D8 Infrastructure                = 2   hybrid: cloud CRM + on-prem TMS; workable for an AI layer on top
D9 Budget & commitment           = 3   COO ready to commit a 2-month pilot at the monthly grid
```

### Index computation

```
Dim  Weight  Level  Contribution (weight × level/4)
---  ------  -----  ------------------------------
D1     18      3      18 × 3/4  = 13.50
D2     14      2      14 × 2/4  =  7.00
D3     12      3      12 × 3/4  =  9.00
D4     12      1      12 × 1/4  =  3.00
D5     11      2      11 × 2/4  =  5.50
D6     10      2      10 × 2/4  =  5.00
D7      8      1       8 × 1/4  =  2.00
D8      8      2       8 × 2/4  =  4.00
D9      7      3       7 × 3/4  =  5.25
---------------------------------------
Readiness Index           = 54.25  →  54 / 100
Tier: READY (51-75, bottom of the band)
```

### The 4-forces read (mm-03)

```
Push      : STRONG  — a competitor's AI quote bot is taking RFQs (a real, dated trigger)
Pull      : MEDIUM  — a legible, owned quote-assembly assistant they control
Anxiety   : LOW-MED — "will it misquote and cost us money?" (answer: HITL on every quote)
Habit     : MEDIUM  — "the managers just do it in Excel, it's fine" (the underrated force)

(Push + Pull) > (Anxiety + Habit)  →  NET-POSITIVE. Change is viable.
```

### Gate check

```
G1 Sponsorship  (dim 1 ≥ 2) : 3  PASS
G2 API-exposure (dim 2 ≥ 2) : 2  PASS (flag: TMS SOAP fragility → E2 must validate)
G3 Use-case     (dim 3 ≥ 1) : 3  PASS
G4 Compliance   (dim 5 ≥ 1) : 2  PASS (GDPR has a path; DPO present)
G5 Commitment   (dim 9 ≥ 1) : 3  PASS
All five hard gates PASS.
```

### Verdict

**GO — with a Gap-To-Target sprint.** All gates pass, the beachhead is sharp, the 4-forces read is net-positive. The one soft risk is **Data readiness = 1**: quote data is scattered. That does *not* fail a gate — it becomes the **first move inside Phase 1**: consolidate the single quote-data source the beachhead needs. The **TMS SOAP API fragility** (a D2 caveat) is flagged as a technical risk to be **validated in E2** (the architect), not here.

### Engagement shape + indicative investment (see `04` for the logic)

```
Tier READY → start Phase 1 immersion focused on the quote-assembly beachhead, 2 months.
Members: COO + 2 ops managers = 3.
Indicative investment = €2,500 + (€2,500 × 3 × 2) = €2,500 + €15,000 = €17,500.
(A price, not a return. ROI of the quote-assembly build = produced in E2 from a measured baseline.)
Hand to /market-proposal for the signed SOW.
```

### Iron-test hook (so the verdict is falsifiable)

> After the engagement starts: did the COO stay weekly-engaged (validates G1)? Did the quote-assembly pain survive `caio-discovery-interview` (validates D3)? Did the TMS SOAP API actually expose what E2 needed (validates the G2 caveat)? Did the SOW land within ±20% of €17,500 (validates the member/duration read)? If the TMS API turns out to be unusable, G2 was scored on hope — tighten the "documented path" bar next time.

---

## 5. Two contrasting mini-scores (calibration)

**A NOT-YET (a gate below floor).** A 40-person creative agency: enthusiastic CEO but no delegated budget authority (D1 = 1), core work in a closed proprietary DAM tool with no API (D2 = 0), no single painful workflow named (D3 = 0). Index might land ~30 (Emerging) on the strength of culture/infra — **but G1, G2, and G3 all fail.** Verdict: **NOT-YET** — "delegate budget authority to a named sponsor, confirm an API/integration path on [tool] (or pick a workflow that lives in a tool that has one), and name one painful weekly workflow. Re-qualify in 60 days." (Index ≠ verdict.)

**A REDIRECT (right need, wrong shape).** A 12-person startup wanting "an AI agent to handle support". One tool (Intercom), one workflow, one department. Even with a great culture score, this is **not a federated multi-member CAIO build** — it's a single agent. Verdict: **REDIRECT** — "you don't need a CAIO engagement; you need one support agent. Use `agentic-systems-builder` (or a point tool). Come back when you have multiple departments to make legible." (mm-02: honest positioning — we are the centralized-federated approach, not a single-agent shop.)

---

## 6. Scoring discipline checklist

| Check | Pass = |
|---|---|
| All 9 levels assigned, each with ≥1 cited evidence line | yes |
| Weights stated; any override documented and still sums to 100 | yes |
| Index computed with the formula (not eyeballed) | yes |
| Tier assigned from the index bands | yes |
| 4-forces read written (push/pull/anxiety/habit) | yes |
| 5 hard gates evaluated PASS/FAIL each | yes |
| No dimension scored above its evidence (hope ≠ a level) | yes |
| Every "deep" technical detail deferred to E2, not produced here | yes |

The index is a decision *aid*, never the decision. The gates and the honest 4-forces read are what actually separate a GO from a NOT-YET (`03-go-no-go-decision-tree.md`).

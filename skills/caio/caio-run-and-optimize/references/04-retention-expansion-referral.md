# 04 — Retention, Expansion & Referral + the Quarterly Business Review

Phase 6 of CAIO Run & Optimize. Outputs `caio-run/Quarterly-Business-Review.md`, `caio-run/Expansion-And-Referral-Play.md`, and the `retention_expansion` block of `metadata.json`.

This phase closes the chain into a compounding loop. Two doctrines drive it. **mm-08** (NRR, expansion pricing) governs how the engagement grows in value without a new logo. **mm-09** (partnerships / network effects) governs the satisfied client as an *internal reference* and the expansion-to-next-department as a *land-and-expand network effect inside one account*. Over both sits mm-11's hierarchy — **retention > monetization > acquisition** — applied to the engagement itself: a renewed client compounds; a churned one means re-selling from zero.

> The product decides if someone stays; distribution decides if they ever arrive (mm-09). For a delivered engagement: the *measured ROI* decides if the client renews; the *internal reference* decides if you ever reach the next department.

**Scope boundary, stated up front (Iron Law 10).** Everything in this reference is the land-and-expand of THE ENGAGEMENT — next department, next C-Level, client-as-internal-reference, the renewal. It is NOT the CAIO's own pipeline marketing. Turning the engagement into the CAIO's **public** case studies is `creator-media-engine`'s job, with explicit client consent. This reference produces the internal artefacts and *hands the public-facing one off*. It does not write the CAIO's marketing.

---

## Part A — Retention: the QBR proves measured value (mm-11)

mm-11's arithmetic: a system that retains compounds; one that churns makes you re-sell from zero. The same is true of the engagement. Retention is bought with **measured value**, not a relationship and not a slide.

### A.1 Why the QBR is the retention instrument

The C-Level renews on one question: *did this pay?* The QBR answers it with the re-measured ROI from reference 01 — proven, partial, and (honestly) falsified — and the cohort retention that shows the savings are durable, not a one-month spike. A QBR that shows honest numbers the CFO can trust renews better than one full of unverifiable greens, because the greens are *believed*.

### A.2 The Quarterly Business Review structure (`Quarterly-Business-Review.md`)

Readable by a non-technical C-Level in under 10 minutes; ends in a decision.

```
# Quarterly Business Review — [Client] — [Quarter]

## 1. Headline (3 lines)
- The system's NSM this quarter: [number + trend]
- Realization rate (actual vs projected ROI): [__%]
- The decision we need from you today: [renew / fund expansion / fix-and-hold]

## 2. The numbers (every figure cited)
| Workflow | Projected $/yr | Actual $/yr | Verdict | Source |
| ... (from ROI-Measurement-Model.md, with telemetry/receipt sources) |
- Total durable annual benefit: $___   System cost: $___   Net: $___   Payback: __ mo
- Hours redeployed: ___   Cost avoided: ___   Decisions improved: ___

## 3. Cohort health (are the savings sticking?)
[the savings-retention table: plateau vs decay per go-live wave]
- What's holding, what leaked, what we did about the leak.

## 4. What we optimized this quarter
[the loop's shipped improvements + their measured effect — the hypotheses that confirmed/refuted]

## 5. Health & governance
- Alerts that fired and were caught before impact: ___
- HITL on sensitive decisions: still enforced [yes] — governance from 09-ROI intact
- Cost margin: model cost is __% of value (mm-08 healthy < 25-30%)

## 6. The honest red (if any)
[the falsified projection(s), named, with the fix plan — this is what makes the greens trustworthy]

## 7. The recommendation
- [Renew the run] — the system is compounding; here is next quarter's loop focus.
- [Expand] — the system saturates its current scope; the next department is [X] (→ Part B).
- [Fix-and-hold] — a cohort is leaking; we stabilize before expanding.

Prepared by: [CAIO]   Period: ___   Sponsor: ___   Date: ___
```

### A.3 The QBR falsification (read it aloud)

Time it. Could a non-technical C-Level read it in under 10 minutes and make a renew/expand decision *today*? Is every headline number sourced? Is the honest red present, or did you smooth it? If it fails any of these, compress and re-source — a QBR that hides a red is a QBR the client eventually stops trusting.

---

## Part B — Expansion: land-and-expand inside the account (mm-08 NRR + mm-09)

### B.1 The network effect inside one company (mm-09)

mm-09: the satisfied client is an **internal reference**, and *alignment beats size* — the COO who already trusts the system, in the same building as the next department, is worth more than ten cold logos. Expanding to the next department is a **land-and-expand network effect inside one account**: each department made legible makes the next one easier (shared stack, shared governance, shared dashboard) and the system more defensible (the more of the company runs on it, the harder it is to rip out — mm-09's moat, applied internally).

### B.2 The expansion sequence (give-first, then ask — mm-09 reciprocity)

mm-09's discipline: deliver value *before* requesting anything. The expansion proposal is the **consequence** of a strong QBR, never its precondition.

```
1. Deliver the QBR's proven value (give-first).
2. Surface where the ICP is "already gathered" — the next department whose work
   the satisfied sponsor can see is a fit (alignment > size).
3. Ask the sponsor for the internal introduction to that department's owner /
   the next C-Level — warm, on the back of demonstrated ROI.
4. Re-enter caio-enterprise-workflow-architect for the next-department audit
   (the chain loop closes — Part D).
5. Scope the expansion as a fresh SOW via /market-proposal.
```

### B.3 Expansion pricing (mm-08)

- The **value metric is scope under management** (departments / workflows), not hours. The account expands by covering more of the company — keeping the 1h/week base light (reference 03 §B) while the scoped expansions compound.
- **Grandfather-and-raise (mm-08):** the new department's scope is priced to the *current, measured* value of the system; existing scope is grandfathered. You raise on new scope, not by squeezing the base.
- **NRR > 100% of delivered value** is the target: the same engagement, more of the company piloted, year over year, with no new logo. This is the mm-08 expansion-revenue engine applied to a services engagement — the cheapest growth there is, because the trust is already bought.

### B.4 The expansion proposal (`Expansion-And-Referral-Play.md`, Part 1)

```
# Expansion Play — [Client]

## Internal reference (the satisfied sponsor)
- Sponsor: [name/role], satisfaction signal: [the proven ROI line]
- Their willingness to introduce / vouch internally: [confirmed?]

## Next department / next C-Level (alignment > size)
- Target: [department], owner: [role], why aligned: [shared work / adjacent value chain]
- The give-first asset already delivered: [the QBR result that earns the intro]

## NRR logic
- Current scope value (measured): $___
- Expansion scope (projected by next architect pass): $___
- Pricing: grandfather existing, price new scope to current value
- Value metric: departments under management (not hours)

## Routing
- Next-wave audit → caio-enterprise-workflow-architect
- Expansion SOW → /market-proposal
```

---

## Part C — Referral & the client-as-reference (mm-09 — DELEGATED, not subsumed)

### C.1 The internal reference (in scope here)

The client-as-reference loop *inside* the engagement: the satisfied sponsor vouches for the system to peers (other department heads, the board), which de-risks the next expansion. This is mm-09's reciprocity + engagement-coherence applied internally — a small internal "yes" (one department's success) opens the larger "yes" (company-wide rollout). You produce and nurture this internal reference.

### C.2 The public case study (OUT of scope — handed off)

Turning the engagement into the CAIO's **public** case study — the borrowed-audience, build-in-public, marketing artefact mm-09 describes — is **`creator-media-engine`'s** job, not this skill's. Iron Law 10 and the scope boundary are explicit:
- This skill **hands** `creator-media-engine` the raw material (the measured results, with sources) and the **explicit client consent gate** (mm-09 — the case study is never the object, and never published without consent).
- This skill does **not** write the CAIO's marketing, ghost-author a public post, or pitch the case study. It produces the internal reference and stops at the handoff.

```
## Public case study — HANDOFF (not produced here)
- Client consent for public reference: [obtained? scope of what may be shared?]
- Anonymization required? [yes/no — what must be masked]
- Raw material handed to creator-media-engine: [the QBR numbers, with sources]
- Owner of the public artefact: creator-media-engine (with consent)
```

If consent is not obtained, the public case study simply does not happen — the internal reference still stands. Never trade the client's trust for the CAIO's marketing.

---

## Part D — Closing the loop (the compounding chain)

The whole accompaniment chain is designed to compound, and this phase is where it bends back on itself:

```
architect (wave 1) → implementation → enablement → RUN (this skill)
                                                      │
                                  measure / monitor / optimize / QBR
                                                      │
                                            verdict = EXPAND
                                                      │
                                                      ▼
                                  architect (wave 2 — next department)
                                                      │
                                                  … and around again
```

The **Expand** verdict (reference 03 §A.5), gated on a healthy, non-leaking, retention-positive system, re-enters `caio-enterprise-workflow-architect` for the next-department audit. Each loop is **cheaper than the last** (shared stack, governance, dashboard, and a sponsor who already trusts the system) and **bigger** (more of the company piloted). That is mm-09's land-and-expand network effect and mm-11's compounding flywheel, fused at the engagement level.

The 12-month test of the whole engagement: does the account show NRR > 100% of delivered value, did the loop reach wave 2 without the original heavy engagement repeating, and is the client a willing internal reference (and, with consent, a creator-media-engine public case study)? Yes to all three = a self-compounding engagement, not a one-off delivery.

---

## Part D2 — Worked expansion sequence (give-first, then ask)

Continuing the worked client (F002 proven at 96%, F001 fixed after a retention quarter, both cohorts now on a plateau):

```
Q2 QBR: realization rate recovered to 78%; F001 cohort off its decay; net +$71k/yr proven.
The sponsor (COO) is visibly relieved and asks "what's next?"

Give-first (mm-09): the CAIO does NOT pitch. The QBR's proven number IS the gift already delivered.
The ask is small and warm: "The finance team runs the same kind of weekly reporting your
dept-heads did before F002 — same shape of work, same value chain. Would you introduce me to
the controller? One 90-minute conversation, no commitment."

Why finance and not, say, marketing (alignment > size, mm-09): finance's work is adjacent to the
already-proven exec-brief workflow — shared data sources, shared governance, a warm internal
reference from a COO the controller already trusts. A bigger-but-colder department (a 200-person
sales org with no overlap) would convert worse despite the headcount.

The loop closes: the controller intro → caio-enterprise-workflow-architect runs a
department-discovery audit on finance (wave 2) → the expansion SOW is priced to the system's
current measured value, grandfathering the existing scope (mm-08).

NRR: same engagement, now covering 2 departments instead of 1, base retainer unchanged, scoped
expansion added → account value up ~60% with zero new-logo acquisition cost.
```

## Part D3 — Renewal & expansion objection-handling (researcher, not salesperson)

The QBR will surface objections. Handle them with evidence, never with pressure (L2).

| Objection | Honest response (evidence-led) |
|---|---|
| "The ROI is lower than you projected." | "Correct — F001 came in at 33% because adoption collapsed after a prompt change, which we diagnosed and fixed; it's now on a plateau. F002 hit 96%. The honest blended number is X, and it's climbing as adoption matures. Here's the telemetry." |
| "Can't our team just run this now? Why renew?" | "They can — that's by design; the system is autonomous. The 1h/week isn't to run it, it's strategic arbitration + the next-department expansion. If you don't want expansion, the lightest renewal is fine; the system keeps running without me." (Never manufacture dependency — mm-08.) |
| "Prove the next department before we pay." | "Fair. The architect runs a 90-minute discovery on finance first; if the opportunity backlog there doesn't clear the bar, we don't expand. The SOW follows a GO verdict, not precedes it." |
| "We want to be a case study — or we don't." | "Your call entirely. With consent, creator-media-engine produces it; without, nothing is published and your internal reference still stands. The case study is never a condition of anything." (mm-09.) |

The posture that renews and expands is the same posture that measured honestly: evidence over enthusiasm. A client who watched you report a falsification straight will believe your proven numbers — and that belief is what buys the renewal and opens the next department.

---

## Part E — Phase 6 discipline checks

| Check | Pass = |
|---|---|
| The QBR puts actual beside projected ROI, every figure sourced | yes |
| The honest red (any falsified projection) is present, not smoothed | yes |
| The QBR is < 10-minute readable by a non-technical C-Level and ends in a decision | yes |
| Cohort health (plateau vs decay) is shown, not a single average | yes |
| Expansion is sequenced give-first (QBR value before the intro ask) | yes |
| Expansion value metric = scope/departments, not hours (mm-08) | yes |
| NRR logic stated; grandfather-and-raise on new scope only | yes |
| Expand verdict gates on no-leak + healthy realization before looping to the architect | yes |
| Public case study handed to creator-media-engine with an explicit consent gate (not written here) | yes |
| HITL/governance from 09-ROI confirmed still enforced | yes |

If any fails, fix it before the QBR or expansion proposal reaches the C-Level. The engagement's credibility — and its renewal — rides on this phase being honest.

---

*Retention renews the engagement. Expansion grows it without a new logo. The internal reference reaches the next department. The public case study belongs to creator-media-engine. And the Expand verdict loops the whole chain back to the architect — cheaper and bigger each turn.*

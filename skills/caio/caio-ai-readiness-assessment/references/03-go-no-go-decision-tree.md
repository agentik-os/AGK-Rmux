# 03 — Go/No-Go Decision Tree, Disqualification Rules & Objection Bank

Phase 3 of the CAIO AI-Readiness Assessment. Turns the scorecard + index + 4-forces read into exactly one verdict — **GO / NOT-YET / REDIRECT** — with honest reasoning. This is where **mm-10 (selling)** is most load-bearing: the gate *is* the discovery call, and disqualification is the point.

> **mm-10 first principle.** *An objection is not a rejection; it's a request for information.* And: *you never have the right to automate a sale you've never made by hand.* Applied here: you never hand a company to the engagement until the gate proves the fit by evidence. Saying "not yet" or "you need Y" is not losing a deal — it's the honesty that makes the deals you *do* close stick (and the reputation that brings the next ten).

---

## 1. The decision tree (run in order)

```
START — scorecard + Index + 4-forces read in hand
│
├─ STEP A · REDIRECT triggers (fire regardless of Index) ───────────────────────────────┐
│   Does the buyer's REAL job match any of these?                                        │
│     • solved by ONE off-the-shelf SaaS            → "buy [tool], you don't need a build"│
│     • is fundamentally a data-cleanup project     → "fix the data foundation first"     │
│     • is really 'we should hire someone full-time'→ "you're big enough to hire; CAIO    │
│                                                      is a bridge, not the destination"  │
│     • a hard regulatory NO, no path in ~6 months  → "resolve with a compliance / on-prem│
│                                                      partner first"                      │
│     • wants magic / a black box / 'replace my team'→ REFUSE/REDIRECT (not the approach)  │
│     • single workflow + single tool + 1 dept      → "one agent, not a CAIO →             │
│                                                      agentic-systems-builder"            │
│   If ANY fires → REDIRECT. Name the alternative. STOP. ─────────────────────────────────┘
│
├─ STEP B · Hard gates (each must pass for a clean GO) ──────────────────────────────────┐
│     G1 Sponsorship   dim1 ≥ 2   named, budget-holding sponsor + a why-now               │
│     G2 API-exposure  dim2 ≥ 2   core tools expose usable APIs OR a documented path       │
│     G3 Use-case      dim3 ≥ 1   ≥1 painful, namable, frequent beachhead                  │
│     G4 Compliance    dim5 ≥ 1   AND no unresolved hard blocker                           │
│     G5 Commitment    dim9 ≥ 1   willing + able to start the monthly model                │
│   Any gate fails AND fixable in ~30-90 days  → NOT-YET (name gate + fix + re-qualify date)│
│   Any gate fails AND NOT fixable             → it was a STEP-A case → REDIRECT ──────────┘
│
└─ STEP C · GO conditions (ALL true) ────────────────────────────────────────────────────┐
      • all 5 hard gates PASS                                                              │
      • ≥1 beachhead use case clear enough to scope a pilot                                │
      • 4-forces read net-positive: (Push + Pull) > (Anxiety + Habit)                      │
    → GO. Shape the engagement (ref 04) and hand to /market-proposal.                      │
    EDGE: gates pass but Index is Nascent/low-Emerging AND gaps are too deep to fix inside │
          a 1-3 month engagement → downgrade to NOT-YET + a Gap-To-Target sprint plan. ────┘
```

**Verdict precedence:** REDIRECT (Step A) beats NOT-YET (Step B) beats GO (Step C). Always check A first — a company can pass every gate and still be a REDIRECT because the *shape* of their need is wrong (the single-agent startup in `02` §5).

---

## 2. Disqualification rules (the honest "no")

These are the rules that make the gate worth more than a sales script. Each is a *named* reason you can say to the buyer's face.

| Rule | Trigger | Verdict | What you say |
|---|---|---|---|
| **No sponsor, no engagement** | dim1 < 2 — interest but no budget-holder | NOT-YET | "Come back when one exec owns the budget and the outcome. Without that, this dies in week 2 — I'd be taking your money to watch it stall." |
| **No API, no centralized model** | dim2 < 2 and no path | NOT-YET / REDIRECT | "The whole approach wires your tools into one surface. [Core tool] has no way in. Expose an API or add an integration layer first — otherwise there's nothing to centralize." |
| **No beachhead, nothing to build** | dim3 = 0 | NOT-YET | "We need one painful, frequent workflow to start on. 'AI somewhere' isn't a project. Pick the thing that wastes the most hours and call me back." |
| **Blocked by law** | dim5 = 0, no path | REDIRECT | "Your constraint is a legal/compliance one, not an AI one. You need a compliance/on-prem partner before any build. Here's the shape of that." |
| **It's a point-tool job** | single SaaS solves it | REDIRECT | "Honestly? Buy [tool] for €X/month. You don't need us. If you outgrow it into a multi-department problem, come back." |
| **It's a data project** | dim4 = 0, beachhead unreadable | REDIRECT | "There's no data for AI to read yet. This is a data-engineering project first. Do that, then we have something to build on." |
| **It's a hire** | 1000+ staff, wants permanence | REDIRECT | "A fractional CAIO is a bridge to capability. At your size, hire the internal owner; we can run the bridge, but name the destination." |
| **Wants a black box** | "just make it work, don't bother me" | REFUSE | "That's the black-box agency, and it fails — opaque agents you can't inspect. Our whole value is legibility. If you want opacity, we're the wrong fit." |

**The medical-diagnosis rule (mm-10).** *A vendor who pitches before diagnosing commits the same fault as a doctor who prescribes before examining.* If you catch yourself wanting to propose the engagement *before* the gates are scored, stop — you're pitching, not qualifying.

---

## 3. The 4-forces change-appetite scoring (mm-03)

Dimension 6 (Culture & change-appetite) is scored *against the four forces of progress*, and the result is **load-bearing in Step C** — a company can clear every gate and still be a NOT-YET if habit + anxiety dominate.

```
Score each force 0-3 from what you heard on the call:

PUSH (drives change — the pain of today)
  0 no pain mentioned   1 mild grumble   2 a real, recurring pain   3 a dated trigger
                                                                     (mm-01 why-now:
                                                                      "a competitor
                                                                      launched X last month")
PULL (draws them to you — the future you offer)
  0 no interest   1 curious   2 wants the legible/owned AI OS   3 already imagining the build

ANXIETY (holds back — fear of the change)  [count AGAINST]
  0 none   1 mild "will it work?"   2 real fears (data leak / lock-in / team resists)
  3 a fear strong enough to kill it ("legal will never allow it")

HABIT (holds back — inertia of the status quo)  [count AGAINST] — the MOST underrated force
  0 no entrenched workaround   1 a light habit   2 "we just do it in Excel, it's fine"
  3 deep institutional habit ("this is how we've always done it, and it works")

CHANGE-APPETITE = (Push + Pull) − (Anxiety + Habit)
  > 0   net-positive → adoption is viable (a GO condition)
  ≤ 0   net-negative → NOT-YET: "the org isn't ready to change; build the push first"
```

**Mechanism before tactic (mm-03).** Most operators only work the *Pull* (listing what the AI OS can do) and wonder why the buyer stalls. The leverage is usually the neglected pair: **amplify the Push** (make the cost of the illegible status quo vivid — mm-01 opportunity cost, honestly) and **disarm the Anxiety** (the monthly-no-minimum, HITL on sensitive steps, transfer-to-autonomy, the legible-not-black-box position). You don't sell by adding arguments; you sell by raising the pain of today and removing the friction of changing.

**Awareness ladder (Schwartz, via mm-03).** Place the buyer:
- **Problem-aware** (the common case): knows AI matters, feels behind, but doesn't know the *centralized federated* approach → frame the call around the *problem* (illegibility, scattered tools), not features.
- **Solution-aware**: knows centralized AI exists, comparing approaches → frame around *why legible/federated beats SaaS-stacking and the black-box agency* (mm-02).
- **Product-aware**: already knows the offer, comparing you to a specific competitor → frame around *proof + the honest gate itself* as differentiation.

Wrong message for the level = a stalled call. Diagnose where they are before you frame anything.

---

## 4. Positioning the offer against the two alternatives (mm-02 — APPLY, don't re-derive)

The buyer is comparing the engagement to **two real alternatives** (Dunford's "competitive alternatives" — what they'd use if not you). You *use* the offer's established position; you never invent a new category here (it's owned upstream by `marketing-master`).

```
Alternative 1 — GENERIC SAAS-STACKING
  What it is : DIY — personal ChatGPT + buying point tools (a chatbot here, a copilot there)
  Where it wins : cheap, no commitment, familiar (the Habit force, mm-03)
  Where it loses : fragmentation — no single legible surface, no governance, shadow IT sprawl,
                   nobody owns the whole; each tool is an island
  Your honest move : if their need is genuinely ONE island → REDIRECT them to it (Iron Law 8).
                     If it's a multi-island problem → that fragmentation IS the pain you solve.

Alternative 2 — THE BLACK-BOX AGENCY
  What it is : an agency that drops opaque vendor agents you can't inspect or own
  Where it wins : "make it go away", hands-off
  Where it loses : you can't see logs/costs/decisions; you're locked in; nothing transfers to
                   your team; when it breaks you're helpless
  Your honest move : the offer's position is the OPPOSITE — legible, transparent (logs/costs/
                     confidence surfaced), federated, and explicitly transfer-to-autonomy.
                     If the buyer WANTS opacity → REFUSE (we're the wrong fit, and it doesn't work).

THE OFFER'S POSITION (applied, not re-derived):
  "The legible, federated AI operating system you own — built on your real tools, governed,
   transparent, and handed back to your team — not a SaaS pile and not a black box."
```

Use this in the **objection bank** below and in the **REDIRECT logic** above. The position is the reason a REDIRECT is honest, not a lost sale: you only keep the buyers the position actually serves.

---

## 5. The objection bank (mm-10 — the 5 objections, feel-felt-found, ethical)

Every objection lands in one of five buckets. **Feel-felt-found**: *"I understand how you feel — others felt the same — here's what they found."* Validate the emotion, normalize with honest social proof (never invented — mm-10/mm-03 ethics gate), recast with a real outcome. And remember mm-10: an objection is a *request for information* — unpack it before you answer ("too expensive compared to *what*?").

### Objection 1 — PRICE ("€2,500/member/month is a lot")
- **Unpack first:** "Too much compared to what — your budget, or the value you're unsure of? If it's value, my qualification probably under-sold the beachhead — let's re-anchor."
- **Feel-felt-found:** "I get it — it's a real number. Other operators felt the same until they put it next to the cost they're *already* paying: [their own stated pain, e.g. 9h/week × loaded cost]. The point of the gate is to make sure that math is real *before* you spend a euro."
- **De-risk (mm-03 anxiety):** "It's monthly, no minimum — you can stop after any month. The €2,500 setup is the only commitment, and you keep what we build."
- **Honest floor:** if the value genuinely isn't there yet → that's a NOT-YET or REDIRECT, not a discount. *Never* improvise an off-grid price to win (mm-08).

### Objection 2 — TIME ("not the right moment, we're slammed")
- **Why-now (mm-01, honest):** "Fair. The reason I'd not wait: [their real, dated trigger — a competitor's AI move, a process breaking]. The cost of staying illegible compounds while others build a surface. That's not a fake deadline — it's the opportunity cost you named."
- **Small beachhead:** "We don't boil the ocean. One painful workflow, one or two members, one or two months. It's designed to fit a slammed team, not add to the load."
- **Honest floor:** if there's genuinely no push (4-forces Push = 0) → NOT-YET: "build the push first; force-feeding change a busy org doesn't want fails."

### Objection 3 — TRUST / CONFIDENCE ("what if it doesn't work / it's a black box")
- **This IS the positioning (mm-02):** "That fear is exactly why the approach is *legible*, not a black box. Every agent surfaces its logs, costs, and confidence. Sensitive steps stay human-in-the-loop. And the whole thing is built to be handed back to your team — you own it."
- **The gate as proof:** "Notice I just spent 30 minutes telling you whether you *should* do this, including the parts where you maybe shouldn't. A black-box agency doesn't do that. The honesty is the product."
- **De-risk:** "Monthly, no minimum. If month one doesn't earn its keep, you stop."

### Objection 4 — NEED ("we already use ChatGPT / a SaaS stack")
- **Reframe the alternative (mm-02 #1 + mm-03 habit):** "That's the right instinct — and it's exactly the fragmentation I'd map. Personal ChatGPT and point tools are islands: no shared surface, no governance, shadow IT you can't see. The 'it's fine' feeling is the habit force — it hides a real cost."
- **Honest floor (Iron Law 8):** "But if it *is* genuinely solving one clean problem and you don't have a multi-department mess — then keep it. You don't need me. I'll tell you that straight."

### Objection 5 — AUTHORITY ("I need to talk to my CEO / partner / board")
- **Qualify it INTO dim 1 (mm-10):** this is the authority objection, and you handle it *in the diagnostic*, not after. Ask up front: "Who else has to say yes, and whose budget is this?" If the answer is "several people I still need to convince" → that's a **Leadership Level 1**, which fails G1 → **NOT-YET**: "bring me the budget-holder and we'll qualify together."
- **Arm the champion:** if they're sold but need to sell internally → the `Go-No-Go-Brief.md` (1-page exec) is built for exactly that — hand it over as their internal pitch.

**Ethics gate (mm-03 / mm-10, non-negotiable across all five).** No false scarcity ("only one slot left"), no fabricated social proof ("92% of logistics firms…"), no invented ROI. On a recurring engagement, a manipulated yes churns next month and costs more than it earned. The honest gate *is* the conversion strategy: trust is the bottleneck, and you widen it by telling the truth — including "not yet" and "you need Y".

---

## 6. Writing the verdict (output discipline)

Into `Go-No-Go-Brief.md`, write exactly one verdict with:

```
VERDICT: GO | NOT-YET | REDIRECT

Reasoning (one honest paragraph):
- the Index + tier (the number, with weights noted)
- the gate results (which passed, which failed)
- the 4-forces read (and what it implies for adoption)
- the decisive factor (the one thing that tipped it)

If GO:        → engagement shape + indicative investment (ref 04) + handoff to /market-proposal
If NOT-YET:   → the failing gate(s) + the SPECIFIC fix + a re-qualify trigger/date
If REDIRECT:  → the named alternative (what they actually need) + a warm pointer to it
```

A verdict without an honest paragraph of reasoning is a verdict you can't defend to the buyer's face — and the whole offer is built on being able to. The hardest verdicts to write (NOT-YET, REDIRECT) are the ones that earn the trust the next ten deals run on.

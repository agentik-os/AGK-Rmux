# 05 — The 30-Minute Qualification-Call Protocol

This is the *how* of the front gate: the actual discovery call, the pre-engagement input checklist, the public-site scan, and the why-now hunt. It is the operating manual for `caio-readiness/` at qualification altitude. It is **mm-10's founder-led discovery call**, scripted — the diagnostic that precedes any proposal.

> **The single discipline:** *you diagnose, you do not pitch.* The prospect talks ~70%, you ~30% (mm-10). You are reconstructing whether their company fits the approach, like a doctor taking a history — not demonstrating a product. The moment you start describing features, you've left the diagnostic and started selling on hope.

---

## 1. Pre-engagement input checklist (read ONLY these three sources)

THE FENCE in practice — the gate reads exactly three things, nothing from inside the (not-yet-existing) engagement:

```
1. COMPANY CONTEXT (from the inbound + the call)
   • size (1-10 / 11-50 / 51-200 / 201-1000 / 1000+)
   • industry + country (regulatory matters: GDPR/sector/data-residency)
   • current stack (CRM / Support / Comms / Docs / PM / Finance / HR / warehouse / any AI today)
   • regulatory constraints (GDPR / SOC2 / HIPAA / FINRA / sector / none)
   • the sponsor — who, and are THEY the budget holder?

2. WHAT THE C-LEVEL SAYS ON THE CALL (the diagnostic — §3 below)

3. A QUICK PUBLIC-SITE SCAN (§2 below)

NOT read (they don't exist yet / are E2's):
   ✗ the discovery rollup (caio-discovery-interview output)
   ✗ company-ai-os/ (the architect's output)
   ✗ any internal system access, data export, or tool audit
```

If a source is missing (no website, sponsor not on the call), mark it `_(not provided)_` and either proceed conservatively or convert to a NOT-YET if it's a gate input (e.g., no sponsor on the call → you can't clear G1 → likely NOT-YET pending the budget-holder).

---

## 2. The public-site scan (2 minutes, before the call goes deep)

```
Fetch: homepage (+ about / product / pricing if quick to reach)
Extract ONLY what's visible:
  • sector / what they sell / who they sell to (B2B / B2C)
  • rough size & maturity signals (team page, careers, funding mentions)
  • regulatory hints (a healthcare/finance/legal vertical → compliance dimension)
  • stack hints (a "careers: React/Salesforce" page tells you more than the buyer will)
  • brand tone (to mirror in the call)

GUARDRAILS:
  • the scan informs the SECTOR and your framing — never the dimension scores on its own.
    Confirm everything that scores a dimension with the buyer on the call.
  • never invent what you can't see. No site / unreachable / B2B-obscure → _(not provided)_, move on.
  • don't recite the site back like a robot. Use it to come in informed:
    "I saw you do {X} for {Y} — when a customer {Z}, who handles that today?"
```

The scan is a *courtesy and a calibration*, not evidence of readiness. A polished site says nothing about whether the core tools have APIs or a sponsor holds budget — those come from the call.

---

## 3. The diagnostic walk (the 30-45 minute call)

Six diagnostic moves, mapped to the dimensions they score. Ask the *intent* below in the buyer's own language; one question at a time; reflect back in a sentence; never paste a list.

### Move 1 — Open on the work, not the AI  → scores D3, seeds D6 Push
- "Before we talk about AI at all — walk me through the thing that made you reach out. What's the painful part of the week?"
- *Why:* mm-10/mm-03 — open on the *job*, not the product. The first pain they name is your beachhead candidate (D3). The energy behind it is your Push signal (D6).
- **Trap to refuse:** opening with "have you tried Claude/OpenAI? here's a use case" — that forces their work to bend to a tool. Refused.

### Move 2 — The why-now hunt  → scores D6 Push, the compelling event (mm-01)
- "Why now? What changed recently that made this worth a call this month rather than next year?"
- *Why:* mm-10's **compelling event** is the strongest predictor of whether a deal is real or will drift six months. Grounded in mm-01: the honest why-now is the *opportunity cost* of staying illegible while competitors build a surface. A real, dated trigger ("a competitor launched an AI quote bot last month") is a Push of 3; "we just feel we should" is a Push of 0-1.
- **Ethics:** if there's no real why-now, *don't manufacture one* (no false urgency). A weak why-now is honest data → it shapes the verdict (often NOT-YET: "build the push first").

### Move 3 — The authority question (early, not late)  → scores D1
- "Who, besides you, has to say yes to this — and whose budget does it come from?"
- *Why:* mm-10 — qualify the **authority objection in the diagnostic**, never discover it after. The answer scores D1 directly: a budget-holding sponsor on the call = Level 2-3; "I'd have to convince three people" = Level 1 → fails G1.

### Move 4 — The tool/API reality  → scores D2, D8, seeds D4
- "Where does the work for {beachhead} actually live — which tools? And do those tools talk to each other today, or is it copy-paste between them?"
- *Why:* gates the centralized model (D2). A buyer who says "it's all in Salesforce and Slack" scores D2≥2; "it's a custom in-house tool, no way in" scores D2=0 → fails G2. Copy-paste-between-islands is also your D4 (data scattered) and your mm-02 fragmentation pain.
- **Altitude guard:** you note *that* a tool appears to have an API (known SaaS + what they say). You do **not** probe endpoints/auth/rate-limits — "to be produced in E2". Self-reported API = Level 2 max.

### Move 5 — The compliance & culture read  → scores D5, D6 (Anxiety/Habit)
- "Anything in your world that legal/compliance would care about if AI touched your data? And how would your team feel about this — excited, nervous, or 'here we go again'?"
- *Why:* D5 (is the build permitted? is there a path?) and the D6 Anxiety + Habit forces. "My team thinks it's about replacing them" = high Anxiety; "we just do it in Excel, it's fine" = high Habit. Both count against the change-appetite score (ref 03 §3).

### Move 6 — The commitment & shape read  → scores D9, sizes the engagement
- "If we found a fit, would you be in a position to start a short paid pilot — it's monthly, no minimum, you can stop after any month? And roughly who'd be in scope: just you, or you plus a couple of the people doing this work?"
- *Why:* D9 (willing + able to start) and the **member count** for indicative pricing (ref 04). The monthly-no-minimum framing is the anxiety reducer (mm-03) — lead with it.
- **Note:** you're sizing, not selling. You don't quote the full number yet unless asked — you gather the inputs; the indicative price goes in `Recommended-Engagement.md` *if* the verdict is GO.

### Throughout — capture verbatim, reflect, don't pitch
- Capture the buyer's **exact words** for pains and fears — they're the evidence that scores dimensions (R-CITE) and the raw material the engagement's messaging reuses later (mm-03).
- After each move, reflect in one sentence ("so quote assembly is the bleeding edge, and the COO owns the budget — got it").
- If the buyer pushes an objection, it's a *request for information* (mm-10) — unpack and answer from the objection bank (ref 03 §5), then return to diagnosing. Don't let an objection turn the call into a pitch.

---

## 4. From call to verdict (the closing 5 minutes)

Mm-10's step 4-5 (closing), adapted to a *qualification* close — you're closing on a *verdict*, not (yet) a sale:

```
1. Reflect the diagnosis back (60 seconds):
   "Here's what I heard: {beachhead}, {sponsor}, {why-now}, {the one risk}."
   → this is the honesty that builds trust AND catches a misread before it ships.

2. State the verdict to their face (ref 03):
   GO       → "You're a fit. Here's the shape and the rough investment. Next step is a proposal."
   NOT-YET  → "Not yet — and here's exactly what to fix first. Call me when {trigger}."
   REDIRECT → "Honestly, you don't need us — you need {Y}. Here's where to go."

3. If GO — the assumptive next step (mm-10 close):
   "I'll send the readiness brief and a proposal. Does {date} work to review it?"  ...then be quiet.

4. Always send the 1-page Go-No-Go-Brief — it's the artefact the sponsor uses to sell internally
   (the authority objection, ref 03 §5).
```

**The honest-close rule.** A GO you can't defend with the scorecard is a GO you shouldn't give. The discipline of saying NOT-YET/REDIRECT to a buyer's face is the entire moat: it's why the GOs convert and stick, and it's what the Iron Test (was the verdict right?) measures.

---

## 5. Call-protocol discipline checklist

| Check | Pass = |
|---|---|
| Opened on the work, not the AI (Move 1) | yes |
| Why-now explicitly hunted (Move 2); real trigger captured or its absence noted | yes |
| Authority/budget qualified early (Move 3) → scores D1 | yes |
| Tool/API reality probed (Move 4) → scores D2, at altitude (no endpoint mapping) | yes |
| Compliance + culture/forces read (Move 5) → scores D5, D6 | yes |
| Commitment + member count gathered (Move 6) → scores D9, sizes pricing | yes |
| Verbatim pains/fears captured for the scorecard | yes |
| Verdict stated to the buyer's face with honest reasoning | yes |
| 1-page brief sent (the internal-sell artefact) | yes |
| Nothing pitched before the diagnosis was complete | yes |

A call that produced a scorecard where every level traces to something the buyer *said* — and a verdict you'd defend to their face — is a clean gate. A call that drifted into a product demo produced a sales pitch, and the scorecard it leaves behind is fiction.

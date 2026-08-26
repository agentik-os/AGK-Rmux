---
name: ghost
description: "Ghost — Persona Simulator (Honest Ghost). Master-level thinking mode that role-plays a target persona — user, client, investor, partner — reacting HONESTLY to a pitch, page, product, or offer: the real objections (including the unspoken one), the questions they would actually ask, and a BUY / NO-BUY / CONDITIONAL verdict with a concrete commitment, then an out-of-character debrief of the top 3 conversion fixes. The ghost is the person who is never in the room when you rehearse. Use when the user says '/ghost', '/omg-ghost', '/ghost <persona>' (e.g. '/ghost investor', '/ghost dentiste'), or in French 'joue le client', 'simule un investisseur'. NOT for real user research (a simulation never replaces talking to real users) and NOT for multi-model deliberation (that is /llm-council) — /ghost is one persona's honest reaction."
---

# Ghost — Persona Simulator (Honest Ghost)

Every pitch is rehearsed in a room where the one person who matters is absent: the buyer. Ghost puts that person in the room. It role-plays a target persona — user, client, investor, partner — reacting honestly to a pitch, page, product, or offer: skimming the way they skim, doubting what they would doubt, weighing you against the status quo they already have. Out come the real objections, the questions they would actually ask, and a buy / no-buy verdict — then Ghost steps out of character and tells you what to fix. The ghost is the person who is never in the room when you rehearse; this skill is that person, honest.

---

## Invocation

```
/ghost                → no persona given: propose the 2-3 highest-stakes personas, default to the buyer
/ghost <persona>      → simulate that persona directly
```

Examples: `/ghost investor`, `/ghost dentiste`, `/ghost "CFO of a 50-person SaaS"` — a role, a market, or a fully specified human all work.

The artifact is whatever the user points at: a pitch, a landing page, a pricing page, a deck, an offer, a cold email, a README. If no artifact is identifiable in context, ask in one line which one to react to — a ghost with nothing to haunt is theater.

---

## Protocol

0. **Acquire the inputs.** A ghost never reacts to content it has not read. File-path artifact → Read it. URL artifact (a landing page, a pricing page) → fetch it (curl/WebFetch; Playwright CLI for JS-heavy pages). If the artifact cannot be fetched or read, stop and say so in one line — never simulate a reaction to an imagined page. Then check for and read `.agents/product-marketing.md` and any PRD/vision docs before building the sheet.

1. **Resolve the persona.** Take it from the argument (`/ghost investor`, `/ghost dentiste`, `/ghost "CFO of a 50-person SaaS"`). No argument: name the 2-3 highest-stakes personas for this artifact — the people whose "no" kills it (buyer, budget-holder, end user, investor) — one line each on why they matter, printed ABOVE the simulation block (see Output Format), and default to the buyer unless the user redirects.

2. **Build the persona sheet BEFORE reacting.** Fill every field: name, role, day-context (what they are juggling when this interrupts them), budget & authority (what they can sign alone), current alternative (the status quo is competitor #1 — name what they use today, even if it is a spreadsheet or "nothing, and nothing works fine"), patience (how many seconds they would truly give this), and what they would need to see to care. Ground every field in the real ICP read in step 0 (`.agents/product-marketing.md`, PRD, vision docs, the conversation); where no source exists, infer the most plausible value and mark it `(inferred)`. Never choose values that make the artifact look good.

3. **Consume the artifact AS the persona.** First-5-seconds pass: what they notice, what they skip. Then log what confuses them (jargon, unclear promise, hidden price) and what they do not believe (unbacked claims, too-round numbers, missing proof) — these feed the `Confused by:` and `Didn't believe:` lines. Judge through the sheet — their patience, their alternative, their incentives — never through your insider knowledge of what the artifact meant to say.

4. **React IN CHARACTER, first person.** 3-5 real objections, each ending with its sheet-field trace (e.g. `(<- current alternative)`), including the unspoken one they would never say to your face — always the LAST objection listed, flagged `[unspoken]`. The questions they would actually ask, in their words. What would make them walk away on the spot. No assistant voice inside the reaction: no hedging, no compliments, no "that said".

5. **Verdict, still in character.** BUY, NO-BUY, or CONDITIONAL. CONDITIONAL states ONE verifiable deliverable with a deadline ("I sign if you show me X by Friday"), and the Commitment line names the booked follow-up. Every verdict states the concrete commitment: an amount, a meeting, a signature, a share — or the exit line they would use to leave politely.

6. **Step OUT of character.** Debrief as yourself: the top 3 fixes ranked by conversion impact. On NO-BUY or CONDITIONAL, fix #1 is the change most likely to flip the verdict; on BUY, the three items are what nearly lost them — or would lose the next persona in the deal chain (e.g. the budget-holder behind this buyer). Tie each fix to a specific objection, confusion, or disbelief from steps 3-4, and close with the fixed simulation line from the Output Format.

---

## Output Format

Emit exactly this simulation block — four sections, this order, nothing else inside it. One addition outside it: on a bare `/ghost` (no persona argument), the 2-3 persona candidates from step 1 print as a 2-3 line block (one line each) ABOVE the simulation block, never inside it.

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
GHOST · SIMULATION (one persona, not user research)
Persona: [who]   ·   Artifact: [what they reacted to]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

PERSONA SHEET
  Name / role:         [...]
  Day-context:         [what they are juggling when this interrupts them]
  Budget & authority:  [what they can sign alone]
  Current alternative: [the status quo — competitor #1]
  Patience:            [seconds they would truly give this]
  Needs to see:        [what would make them care]

IN-CHARACTER REACTION
  First impression (5s): [noticed: ... / skipped: ...]
  Confused by:           [jargon, unclear promise, hidden price]
  Didn't believe:        [unbacked claims, too-round numbers, missing proof]
  Objections (3-5, each ending with its sheet-field trace; the unspoken one is always LAST):
    1. [...] (<- current alternative)
    2. [...] (<- budget & authority)
    N. [the one they would never say to your face] [unspoken] (<- sheet field)
  Questions they would ask:
    - [...]
  Walk-away triggers:
    - [...]

VERDICT: [BUY | NO-BUY | CONDITIONAL: one verifiable deliverable + deadline]
  Commitment: [amount / meeting / signature / share — or the exit line; on CONDITIONAL, the booked follow-up]

DEBRIEF (out of character)
  1. [on NO-BUY/CONDITIONAL: the fix most likely to flip the verdict · on BUY: what nearly lost them]
  2. [fix]
  3. [fix]
  This was a simulation of one persona: validate with real users.
```

---

## Rules & Anti-patterns

- **A polite ghost is a useless ghost.** If the reaction reads like encouragement, delete it and redo. The user invoked Ghost for the meeting they cannot attend, not for applause — challenge, never flatter (L2).
- **The persona has alternatives and limited time.** Weigh every line of the reaction against the `current alternative` and `patience` fields. "Interesting, tell me more" from someone with thirty seconds and a working status quo is a lie.
- **Skepticism comes from the persona's incentives, never generic negativity.** Every objection traces to the sheet: their budget, their risk, their boss, their alternative. "Could be better" is banned; "my current tool does this for free" is the bar.
- **Honesty includes yes.** If the artifact is genuinely strong for this persona, the ghost buys. A rigged NO-BUY is as dishonest as flattery — but the yes must survive the objections, not skip them.
- **CONDITIONAL is not a politeness hatch.** "I'd want to see some traction" is applause in disguise. The condition is ONE verifiable deliverable with a deadline the persona would genuinely wait for, and the Commitment line names the booked follow-up. If any walk-away trigger fired, or the persona would not actually book that follow-up, the honest verdict is NO-BUY.
- **Never invent facts or numbers.** No fabricated market sizes, competitor prices, benchmarks, or quotes. Where the persona would need a number you do not have, they ask for it instead ("what does this cost me per seat?").
- **A ghost never reacts to content it has not read.** The artifact's own words are facts too: fetch and read them first (step 0). If the fetch fails, stop and say so in one line — no imagined pages.
- **Label the whole thing a SIMULATION.** The header says it; the debrief's fixed closing line repeats it. A ghost rehearses the meeting; it never substitutes for talking to real users.
- **No character bleed.** Assistant voice inside steps 4-5, or persona voice inside the debrief, invalidates the run — redo the contaminated section.
- **One ghost, not a council.** Several personas = separate sequential runs; multi-model deliberation is /llm-council. /ghost is one persona's honest reaction.

---

## Pairs With

- **/popper** — Popper falsifies whether the product works; Ghost falsifies whether anyone will pay for it. Run Popper on the build, Ghost on the pitch.
- **/pitch** — build the 30 seconds with /pitch, then hand the ghost its finished output; the pitch's WHO is the ghost's default persona.
- **/mk-copywriting** — draft the copy with /mk-copywriting, stress-test with /ghost, redraft; loop until the ghost buys for reasons the persona sheet supports.
- **/offer-and-revenue-architect** — when the NO-BUY says the offer is too small rather than badly worded, take the debrief there before rewriting a single line of copy.
- **/omg-product-marketing-context** — maintains `.agents/product-marketing.md`; when it exists, step 0 reads it and the persona sheet uses the real ICP instead of inferring.

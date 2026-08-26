---
name: 10x
description: "10x Thinking Mode — ambition reframe engine, not a pep talk. Takes the user's idea, plan, or goal and rethinks it 10x bigger: names the implicit ceiling they silently assumed, splits REAL constraints (physics, law, cash today) from ASSUMED ones (habit, fear, convention, identity), redesigns the mechanism — a different machine, not the same machine working harder — then runs a reality pass (what breaks first, riskiest assumption) and commits ONE concrete first step within 48h. Use when the user says '/10x', '/omg-10x', 'think 10x bigger', 'am I thinking too small', '10x this idea', 'what would the ambitious version look like', or in French 'pense 10x plus grand', 'vois plus grand', 'la version ambitieuse'. NOT for execution-planning the current-size plan (that is /omg-planner) — /10x changes the ambition, then planning follows."
---

# 10x Thinking Mode

Take the idea on the table and rethink it at 10x scale. Most plans are sized by an invisible ceiling — a number the user never chose, inherited from habit, fear, or "how it is done." This mode names that ceiling, splits REAL constraints from ASSUMED ones, and redesigns the mechanism until the ambitious version is concrete enough to start inside 48 hours. It is an ambition reframe with an honest reality pass, not a pep talk: the output is a different machine plus one committed first step — never applause.

---

## Invocation

```
/10x <idea | plan | goal>
/omg-10x <idea | plan | goal>
```

Also fires on "think 10x bigger" (FR: "pense 10x plus grand") aimed at whatever idea is in context. If no idea is given and none is in the conversation, ask for the plan in one line — never run the protocol on a guess.

---

## Protocol

Run all six steps, in order, every time. Each step produces a named artifact the next one consumes. Verified means user-stated or fetched now from a source you name inline — anything from model memory is an ESTIMATE, never a verified fact. Never invent numbers, market sizes, or customer quotes; an estimate is allowed only when labeled ESTIMATE with its basis, in the exact form shown in the Output Format (L2: researcher, not sycophant).

### 1. Restate the idea and name the Ceiling

Restate the plan in one or two sentences — the user's words, tightened. Then extract the implicit CEILING: the number or scope they silently assumed. Hunt for it in four places: revenue ("a side income"), users ("my local clients"), geography ("in my city"), effort model ("me, evenings, no hires"). Write the ceiling as an explicit number or scope. If none was stated, infer the smallest scale at which the plan still makes sense — that IS the ceiling. Judge the mechanism at 1x while you are here: if it is weak, say so in the 1x VERDICT line, name the broken mechanism, run steps 3-6 on the FIXED mechanism, and point the 48h step at the cheapest test of the flaw — never scale a broken machine.

### 2. Constraint ledger

List every constraint the plan assumes — stated or implied. Classify each:

- **REAL** — physics, public law and regulation, or user-stated facts (cash actually in the bank today, signed contracts). Binding. Respected, never argued away.
- **ASSUMED** — habit ("I have always charged X"), fear ("nobody would pay more"), convention ("how it is done in this industry"), identity ("I am not the type who hires").

Decision test, per constraint: what evidence makes this binding TODAY? No evidence → ASSUMED. The ASSUMED column is the raw material of the reframe; the REAL column is the boundary it must honor. A one-line idea is the normal input, so most entries are yours, not the user's: prefix every entry the user did not state with (inferred). REAL admits only user-stated facts or public law/physics — cash or contracts you do not know become a single "UNKNOWN (confirm)" line, never a guessed REAL. Tag habit | fear | convention | identity only when the user's own words evidence it; otherwise tag convention (inferred).

### 3. 10x reframe — pick the 2-3 axes that fit best

Choose the 2-3 axes where breaking an ASSUMED constraint buys the biggest jump:

- **10x customers** — serve a population, not a neighborhood
- **10x price/value** — sell the outcome, not the hours
- **10x speed** — deliver in a day what the market delivers in a month
- **10x distribution/leverage** — a channel that compounds: platform, partners, content, API
- **10x automation** — the system runs without the founder in the loop

Core principle — state it explicitly in your output: **a 10x outcome requires a DIFFERENT MACHINE, not the same machine working harder.** For each chosen axis, name the mechanism change: what is structurally different, not merely bigger. If you cannot point at what changed in the machine, the reframe failed — return to the ledger; do not ship "same plan × 10."

### 4. The 10x version

Describe what the offer/product/system becomes at 10x — concretely:

- **Serves** — who the 10x customer is, and why they are reachable
- **Charges** — what it charges / how it captures value at that scale
- **Reaches** — the distribution mechanism from step 3 that gets it to them

Write it in present tense, as a working machine — specific enough that a stranger could tell it apart from the original plan in one read.

### 5. Reality pass

This is where honesty lives. At 10x, something breaks FIRST — name which:

- **ops** — fulfillment or support collapses under volume
- **trust** — an unknown brand asking a premium price
- **cash** — growth outruns the bank account
- **tech** — the stack that served 10 dies at 10,000

Name the first bottleneck and why it hits before the others. Then isolate the single riskiest assumption — the one belief the whole 10x version stands on — and design the cheapest test that could kill it: a call, a landing page, a pre-sale, one prototype. REAL constraints from step 2 are respected here, never hand-waved. N defaults to 10; if a REAL constraint honestly caps the reframe lower, set N to the honest multiple (THE 4x VERSION) and name the capping constraint on that header line.

### 6. First step (48h)

ONE concrete action, executable within 48 hours, that commits toward the 10x path — ideally the cheap test from step 5. It must leave an external footprint: someone contacted, something published, something priced, something shipped. Never "do market research," "think about it," or "validate the idea." State it as an imperative with a deadline.

---

## Output Format

Emit exactly this block, every section filled. The block is the whole reply — challenges to the user's framing go inside it (the 1x VERDICT line), never as prose around it. Repeat the REAL:/ASSUMED: prefix on every ledger line:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
10x THINKING · [idea, 3-6 words]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

1x VERDICT: [sound | weak — flaw in one line]

THE CEILING
  [the number/scope silently assumed + where it came from]

CONSTRAINT LEDGER
  REAL:     [constraint] · [why it binds today; user-stated or public law/physics only]
  ASSUMED:  [constraint] · [habit | fear | convention | identity — user-evidenced, else convention (inferred)]
  UNKNOWN (confirm): [fact only the user can know: cash today, signed contracts]
  (one line per constraint, REAL:/ASSUMED: prefix repeated on each; (inferred) marks entries the user never stated)

THE {N}x VERSION   (axes: [2-3 chosen axes])   [N<10 only when a REAL constraint caps it — name that constraint on this line]
  Machine change: [what is structurally different, not just bigger]
  Serves:  [who, at {N}x]
  Charges: [value capture at {N}x — user-stated or ESTIMATE ~$40k/yr (basis: user's current rate × stated hours)]
  Reaches: [distribution mechanism]

WHAT BREAKS FIRST
  [ops | trust | cash | tech] · [why this one hits before the others]

RISKIEST ASSUMPTION
  [the one belief the {N}x version stands on]
  Cheapest test: [the test that could kill it]

FIRST STEP (48h)
  [one concrete external action + deadline]
```

---

## Rules & Anti-patterns

Hard rules:

- **Honesty over flattery.** If the idea is weak at 1x, say so in the 1x VERDICT line before scaling it — 10x of a bad mechanism is a 10x bad mechanism. Challenge the user's framing when it is wrong, inside the block; never mirror it back to please them.
- **No invented facts.** Every number is user-stated, fetched now from a named source, or labeled ESTIMATE with its basis — model memory is never verification. A made-up market size poisons the whole reframe.
- **REAL constraints stand.** Law, physics, and today's cash are never "limiting beliefs." Overriding a REAL constraint is not ambition; it is fiction.
- **Mechanism or nothing.** The acceptance test for steps 3-4 is pointing at the machine change. None named → rerun the reframe.

Anti-patterns — any one of these voids the run:

- Multiplying the user's numbers by 10 without changing the mechanism ("charge 10x more" with no new value logic).
- Motivational fluff: "believe in yourself," "the sky is the limit." This skill outputs machines and tests, not energy.
- Dismissing REAL constraints as mindset problems.
- Vague first steps: "research the market," "talk to some people," "refine the vision." Not verifiably done inside 48h → not a first step.
- Agreeing with the user's framing to please them — reflecting the plan back with bigger adjectives and calling it 10x.

---

## Pairs With

- **/popper** — falsify the output: hand the riskiest assumption to Popper and attack it before building anything.
- **/pitch** — the 10x version is the raw material of the pitch; ambition first, narrative second.
- **/ghost** — put the 10x version in front of the buyer or investor who must fund it; if the ghost shreds it, the reframe was fantasy, not ambition.
- **/omg-planner** — once the ambition is reset, planner turns the 10x path into an execution plan. /10x is not for execution-planning the current-size plan — that is /omg-planner's job; /10x changes the ambition, then planning follows. Ambition before planning, never the reverse.

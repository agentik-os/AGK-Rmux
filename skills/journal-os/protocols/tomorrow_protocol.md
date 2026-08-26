# Protocol: Tomorrow Protocol

The session ends with a small, executable design for tomorrow: **at most three missions**, the non-negotiables that are actually active, one identity challenge, an optional challenge only when it earns its place, and the unfinished loops still open.

The cap is the point. A list of seven produces a day of triage and an evening of explaining what did not get done. `../schemas/tomorrow_protocol.json` sets `maxItems: 3` on the missions array, so a fourth mission cannot be written even if the session is enthusiastic.

## Steps

1. Read the day just captured: the contradictions, the identity evidence, the objectives that moved and the ones that did not, and the energy the user actually has left.
2. Select missions by the logic below. Stop at three. Stop earlier when the day does not honestly need three.
3. Write each mission with all four fields. **A mission with no success condition is not a mission**, and the schema rejects it.
4. List the non-negotiables that are currently active. Only those. The schema requires `active: true` on every item, so a lapsed non-negotiable cannot be listed as one.
5. Write ONE identity challenge: concrete, slightly uncomfortable, achievable inside a single day.
6. Add an optional curiosity or social challenge **only if it adds genuine value tomorrow**. Most days it does not, and the correct output is to omit it.
7. Carry the unfinished loops forward with their age. A loop that has aged past a week is named as such, without moralizing.
8. Read the whole protocol back in under 30 seconds of speech. If it does not fit, it is too big for a real day.

## Missions: at most three

Each mission carries exactly four fields.

- **DOMAIN**: one of SELF, HEALTH, SOBRIETY, WEALTH, BUILD, WORK, PEOPLE, LOVE, WORLD, MIND, FREEDOM.
- **ACTION**: one concrete action, stated so that a stranger could execute it. Not "work on the build". "Ship the auth fix and deploy it."
- **WHY**: the objective or contradiction this serves, in one line. A mission with no why is a task someone else's day leaked into.
- **SUCCESS CONDITION**: the observable state that makes tomorrow evening's answer a yes or a no. No "make progress on", no "spend time with", no "focus more".

Success conditions that work: "the offer is sent to one named person", "the fix is deployed and the page loads", "in bed with the light off before 23:30", "the call happened and the price was said out loud". Success conditions that do not: "feel better about it", "be more consistent", "try to start earlier".

### Selection logic, in order

1. Fix the most expensive contradiction first, with the smallest experiment that tests it.
2. Prefer an OUTPUT over an activity. Shipping beats time spent, sending beats drafting, one conversation beats twenty tasks.
3. Prefer the hard conversation over the comfortable backlog. It is usually the item quietly blocking three others.
4. Protect the physical floor: sleep and recovery are missions when they are the binding constraint, not bonus items after the real work.
5. One mission per domain, at most. Three missions inside BUILD is one mission with three parts, and it is written as one.
6. Match the count to the real day. A day with 11 hours of client obligations gets ONE mission. Writing three into it is a decision to fail at two of them.
7. A zero mission day is legal and occasionally correct: illness, travel, a deliberate rest day. The non-negotiables still stand.

## Non-negotiables

Only the ones currently in force. Each is a standing anchor the user has already agreed to, not a new rule introduced tonight, and each is binary.

    NON-NEGOTIABLES
    - [name] : [the binary condition]

Rules:
- Never invent one at the end of a session. A new non-negotiable is a deliberate decision made when the user is not tired.
- Never list a lapsed one to apply pressure. A non-negotiable that has been broken four nights running is not an anchor, it is a contradiction, and it routes to `contradiction_engine.md`.
- Three to five is a working set. Nine is a wish list.
- Sobriety anchors are stated as conditions, never as moral achievements, and never with a streak count attached unless the user has asked for the count.

## The identity challenge

Exactly one per day. It is the deliberate vote, the thing that makes tomorrow's identity evidence non-random.

Three criteria, all required:
- **Concrete**: an action, with an observable result. "Ask for the price increase in the call" is concrete. "Be more confident" is not.
- **Slightly uncomfortable**: real friction, and not more. The right size is the thing the user would avoid if the day got busy, not the thing that requires a good week to attempt.
- **Achievable in a day**: it starts and finishes inside tomorrow.

Examples in the right band: send the offer with no discount attached; say the number out loud and then stop talking; publish the unfinished thing; make the call that has been in the notes for nine days; leave the phone at home for the walk; tell one person the real reason.

Out of band, do not write these: launch the product; fix the relationship; stop the habit for good; wake up at 05:00 having gone to bed at 02:00.

## The optional challenge

Optional, and usually absent. Include it only when it adds genuine value tomorrow, and never as decoration.

Kinds: curiosity (learn one specific thing), social (one contact with no ask attached), environment (work somewhere different), creativity (make one thing badly and quickly), generosity (give something with no return), exposure (be visible where it is uncomfortable).

Omit it when the three missions already fill the day, when the user is depleted, when the identity challenge is already at the edge of what tomorrow holds, or when nothing genuinely useful comes to mind. **An omitted optional challenge is the normal case.**

## Unfinished loops

Everything still open, with its age and its next action.

    UNFINISHED LOOPS
    - [loop] (open [N] days) -> next action: [action]

- Age every loop honestly. An age is a fact, not a reproach.
- A loop open more than 14 days gets a decision rather than a carry-forward: do it tomorrow, schedule it with a date, delegate it, or kill it. Killing a loop is a legitimate and often correct outcome, and it is recorded as `killed`, not as a failure.
- A loop that has been carried forward three times untouched is pattern input for `contradiction_engine.md`.
- Never carry a loop the user has closed. Check before listing.

## Empty forms

Where a part of the protocol has no honest content, write the empty form. Never invent filler.

    MISSIONS
    - No missions tomorrow: deliberate rest day. Non-negotiables stand.

    OPTIONAL CHALLENGE
    - None tomorrow.

    UNFINISHED LOOPS
    - No open loops.

## Stop rules

- Never more than three missions. The schema will refuse a fourth, and a session that wants one has picked three wrong ones.
- Never a mission without a success condition.
- Never a non-negotiable that is not currently active.
- Never more than one identity challenge. Two challenges is zero challenges.
- Do not design tomorrow before the day is captured. A protocol written mid-interview is written against an incomplete day.
- Do not use tomorrow's protocol to punish today. It is a design, not a penance, and a punitive protocol is abandoned by 10:00.

## Required closure

- Decision or output: a Tomorrow Protocol with 0 to 3 missions, the active non-negotiables, one identity challenge, an optional challenge or its explicit absence, and the open loops with ages.
- Owner: MIRROR drafts it; the user confirms, cuts or replaces any item before the session closes.
- Observable completion evidence: a `tomorrow_protocol` object (see `../schemas/tomorrow_protocol.json`) validating with `missions` at 3 or fewer, each carrying a non-empty `success_condition`, and one `identity_challenge`.
- Review trigger: tomorrow's interview checks each success condition as a yes or a no, before anything else is discussed.
- Memory and handoff instruction: persist the protocol with the entry, hand commitments to the memory store, and hand longitudinal objective tracking to Identity Shift OS and Execution OS. The protocol never goes to the Content Handoff.

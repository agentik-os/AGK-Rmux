# Protocol: Contradiction Engine

A contradiction is the measured distance between a **declared intention** and **observed behavior**. It is surfaced as evidence, never as a verdict on the person. The engine exists because the gap is invisible from inside a single day and obvious across a week, and because naming it accurately is worth more than any amount of encouragement.

**Two constraints make this safe. A contradiction requires a PATTERN of at least two observed days, and it is investigated conversationally with the user BEFORE it is recorded.** Both are enforced by `../schemas/contradiction.json`, which cannot represent a one-day contradiction and cannot mark an uninvestigated one as recorded.

## Steps

1. Pull the declared intentions in force: the identity contract, active objectives, stated priorities from recent entries, and commitments the user made out loud.
2. Pull the observed behavior for the trailing window (7 days by default, 14 for slow-moving objectives such as revenue or body composition). Observed behavior means facts, not the user's summary of their week.
3. Match each intention against the behavior that would have to exist if it were true. Absence of that behavior is the signal.
4. Count the days on which the gap is visible. **Fewer than two: stop. Do not record, do not mention it as a pattern.** One day is a day, not a pattern, and calling it one is both wrong and corrosive.
5. Rank the surviving candidates by cost to the user's stated objectives. Take the most expensive one. **At most one contradiction is worked per session**, two only when both are cheap to resolve and clearly linked.
6. Investigate conversationally, before writing anything (script below). The investigation frequently kills the contradiction: there was a reason, and the reason is the actual finding.
7. If it survives the investigation, record it with the five output fields. If it does not, record what was learned instead (often a constraint, an obligation, or a changed priority the intention list has not caught up with).
8. Attach ONE next experiment. Small, specific, testable within a few days, and preferably a change to conditions rather than a demand for more willpower.

## The pattern requirement

| Days observed | Status | What MIRROR does |
|---|---|---|
| 1 | Not a contradiction | Note it internally. Say nothing. Watch. |
| 2 to 3 | Emerging | Investigate conversationally. Record with severity `emerging`. |
| 4 to 6 | Established | Record with severity `established`. It goes into the Tomorrow Protocol. |
| 7+ or across weeks | Structural | Record with severity `structural`. It is the session's main subject and probably a strategy problem, not a discipline problem. |

A gap that disappears when the user explains it was never a contradiction. Delete it rather than downgrading it.

## Worked examples

**Freedom versus fragmented attention.** Declared: freedom, autonomy, control of time. Observed: 6 days of a calendar cut into 20 minute pieces, no block longer than 45 minutes, three of them commitments the user did not choose. Gap: the stated goal is unstructured time, the lived week is other people's structure. Possible cause: obligations accepted one at a time, each cheap in isolation. Next experiment: refuse the next single request that costs more than an hour, and record what actually happens.

**A stated priority project with no meaningful output.** Declared: the build is the priority. Observed: 5 days, build mentioned each evening, shipped output on 1 of them, and that one was a config change. Gap: priority in language, residual in practice. Possible cause: the build block sits at the end of the day, after the energy is gone. Next experiment: the build block runs first tomorrow, before anything else opens, with the success condition being one shipped thing rather than time spent.

**A revenue goal with no revenue activity.** Declared: 25k per month. Observed: 6 days of building, learning and tooling, zero sales conversations, zero offers made, zero prices named to a human. Gap: the goal requires a conversation the week contains none of. Possible cause: the building is legible and safe, and the conversation is neither. Next experiment: one specific person, one specific offer, sent tomorrow, success condition being that it was sent rather than that it was accepted.

**A body goal with sacrificed recovery.** Declared: get strong, get lean. Observed: 4 trainings in 5 days, sleep under 6 hours on 4 nights. Gap: training is the visible half of the goal and recovery is the half that produces it. Possible cause: training is chosen and sleep is what is left over. Next experiment: hard cut off on the evening screen at a named time for three nights, training whatever survives that.

**A network goal with purely transactional contact.** Declared: real relationships, a strong network. Observed: 5 days, every outbound message had an ask in it, zero contacts with nothing attached. Gap: the goal is relationship, the behavior is extraction. Possible cause: contact only crosses the threshold of worth doing when there is a reason. Next experiment: two messages tomorrow with no ask in them at all.

**A depth goal with every empty moment filled.** Declared: think deeply, read, produce original work. Observed: 6 days, every gap over three minutes filled with a feed, and the only unfilled time was in the shower, where two of the week's three ideas came from. Gap: the input required for the goal is exactly the thing being displaced. Possible cause: the phone is the default object in the hand. Next experiment: one 30 minute walk tomorrow with the phone left behind, and note what arrives.

## The investigation script

Run this conversationally, one question at a time. Neutral tone throughout.

1. "You said [intention]. Over the last [N] days I see [observed behavior]. What is going on there?"
2. "Was that a choice, a constraint, or a drift?"
3. "What would have had to be true for it to go the other way?"
4. "Is [intention] still what you actually want, or has it changed and the list has not caught up?"
5. "What is the smallest thing that would move it?"

Question 4 is the one most often skipped and the most valuable. A large share of contradictions are stale intentions rather than failed behavior, and demoting an intention is a legitimate resolution.

## Output fields

Record exactly these, and nothing else:

- **INTENTION**: the declared intention, in the user's own words where possible.
- **EVIDENCE**: the observed behavior, dated, factual, countable. At least two days. A camera would agree with every line.
- **GAP**: one sentence naming the distance. No adjective about the person.
- **POSSIBLE CAUSE**: a hypothesis, labelled as a hypothesis, ideally the user's own from the investigation.
- **NEXT EXPERIMENT**: one action, testable within days, with an observable result.

## Framing: evidence, never shame

| Write this | Never this |
|---|---|
| "Build was named the priority on 5 days and shipped output on 1." | "You keep saying the build matters but you never do it." |
| "Six days of calendar with no block over 45 minutes." | "You have no discipline with your time." |
| "Zero sales conversations against a 25k goal." | "You are avoiding sales because you are scared." |
| "Sleep under 6 hours on 4 of 5 training nights." | "You are sabotaging yourself." |

The rule under the table: **state the count and the gap, then stop talking and ask.** The user supplies the meaning. MIRROR supplying it turns a measurement into an accusation, and an accusation ends the honest reporting that the whole system runs on.

## Stop rules

- Fewer than two observed days: not a contradiction. Not recorded, not mentioned.
- Not investigated with the user: not recorded. The schema refuses it (`investigated_with_user` false forces status `held_for_investigation`).
- Never more than two contradictions per session, and never two structural ones. A session that surfaces five gaps has taught the user only that the review hurts.
- Never open a contradiction the user has already resolved out loud earlier in the session.
- Never route a health, sobriety or grief pattern through this engine as a discipline gap. Those are conditions, not contradictions, and they route to the relevant OS or to real support.
- Do not stack a contradiction on top of a day the user has just described as one of the worst of the year. Record the observation, work it tomorrow.

## Required closure

- Decision or output: at most one recorded contradiction with the five fields filled, or an explicit statement that no significant contradiction was detected today.
- Owner: MIRROR detects and drafts; the user confirms or kills it in the investigation.
- Observable completion evidence: a `contradiction` object (see `../schemas/contradiction.json`) with `observed_days` holding two or more dates, non-empty `evidence`, and `investigated_with_user` true.
- Review trigger: re-check every recorded contradiction after the next experiment resolves, and at the weekly rollup.
- Memory and handoff instruction: persist the contradiction and its experiment; hand longitudinal identity consequences to Identity Shift OS. A contradiction NEVER goes to the Content Handoff without the user asking for it explicitly.

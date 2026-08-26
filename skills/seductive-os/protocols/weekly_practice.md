# Protocol: Weekly Practice

A bounded week of deliberate practice, scaled to the user's current anxiety level rather than to their ambition. It installs at most four changes: one presence practice, one conversational practice, one exposure rep tier, one self-presentation change. Everything else goes on the NOT NOW list, visibly, so it is deferred rather than forgotten.

The plan is built on reps, presence and skill, which the user controls. It is never built on an outcome that requires a named person's decision.

## Steps
1. Take the bottleneck from magnetism_baseline.md. Build the week around that one factor. A week that works on all seven works on none.
2. Take the current anxiety level, 0 to 10, and the current real exposure count. These two numbers set the dose, and nothing else does. See the dose table below.
3. Write the three tiers for each practice, and write the FLOOR first. The floor is what survives a bad day: 2 to 5 minutes, no other person required, no courage needed. A plan with no floor is a plan that ends the first bad week.
4. Set the exposure reps from the ladder in approach_anxiety_ladder.md, at the rung the user is currently on. Never above it, and never a "stretch" rung inserted because the week looks light.
5. Schedule each rep against a real slot in the real week: a day, a time, a place the user is already going to be. An unscheduled rep is an intention, and intentions have a completion rate near zero.
6. Attach the rehearsed exit to every rep that involves another person, before the week starts.
7. Pick ONE self-presentation change from self_presentation_audit.md. One, running for the whole week, chosen by the user because it is **[P]**.
8. Write the NOT NOW list explicitly, with the items the user wanted to add. Naming what is deferred is what makes the cap hold.
9. Define the week's success condition in advance, in process terms only: reps attempted, floors held, debriefs run. Not matches, not numbers, not dates. There is no field in `practice_log` for a phone number, deliberately, because what gets logged is what gets optimized.
10. Run the week. Log each rep in `practice_log` on the day, not on Sunday from memory.
11. Debrief the reps that had something in them, once each, inside the bound, with interaction_debrief.md.
12. Review at the end of the week against the table below, adjust ONE variable, and roll it forward. Change the system on evidence, never on one painful evening.

## Dose by current anxiety level
| Anxiety 0 to 10 | Exposure reps per week | Shape |
| --- | --- | --- |
| 8 to 10 | 0 unscheduled, 6 to 9 ladder reps at the entry rung across 2 or 3 sessions | Solo and ladder work only. Clinical screen must be clear first |
| 6 to 7 | 3 to 5 | Ladder reps plus one low-stakes real conversation, no invitation attached |
| 4 to 5 | 4 to 6 | Real conversations, one of which may carry an invitation |
| 2 to 3 | 5 to 8 | Normal practice, invitations included, one stretch rep permitted |
| 0 to 1 | Re-check the number | Either the user is genuinely comfortable, in which case the bottleneck is elsewhere, or the reps are not real |

If the previous week's reps were not completed and the reason was fear rather than schedule, drop one tier before anything else. Repeating an undone dose does not produce it.

## The three tiers, for every practice
- **Floor**: 2 to 5 minutes, no other person required, survives a bad day. Example shape: the grounding sequence from first_conversation.md, done once, anywhere.
- **Standard**: the normal week, includes real human contact.
- **Deep**: one optional stretch rep. Never the minimum, and never the measure of self-respect. A user who only counts the deep tier has built a machine for feeling like a failure.

## The weekly review
| Question | What the answer changes |
| --- | --- |
| Reps attempted against reps planned | The dose. Under 60 percent means the dose was wrong, not the user |
| Floors held on bad days | Whether the floor is genuinely a floor, or a standard in disguise |
| Anxiety trend on the current rung | Promotion, hold or demotion |
| Calibration accuracy: predicted interest against what happened | Whether the reading is improving, which is the real skill |
| Self-presentation change: did it hold, does the user like it | Keep, drop or swap. Their call, it is **[P]** |
| One thing to stop | There is always one, and asking produces it |

## Stop rules
- Cap at four changes. A fifth is not ambition, it is the mechanism by which the whole plan is abandoned in week two.
- Never scale the dose to what the user wishes they could do. The dose comes from the two numbers at step 2.
- No outcome targets in the plan. No "two dates this month", no number quotas. Outcome targets reintroduce the evaluation the practice exists to defuse, and they make every rep a test.
- Never plan reps in contexts where refusal is not cheap. Practice does not happen on people who are working.
- A no during any rep is a completed rep, logged as clean, and it does not reduce the week's score.
- If the reps stop happening for three consecutive weeks, stop rewriting the plan. The bottleneck is not the plan. Route to approach_anxiety_ladder.md, or to Habit Tracker OS if the failure is consistency rather than fear.
- If practice has become compulsive, if the user is running reps to escape a feeling rather than to build a capability, stop and route to ../references/safety-and-boundaries.md under **C**.

## Required closure
- Decision or output: the week's four changes, the dose, the scheduled slots and the NOT NOW list.
- Owner: the user runs it. The OS sizes it and reviews it.
- Observable completion evidence: `practice_log` records written on the day (see ../schemas/practice_log.json), and a `season_goal` record updated at the review (see ../schemas/season_goal.json).
- Review trigger: end of week for the dose, every four to six weeks for the season and the bottleneck.
- Memory and handoff instruction: persist reps and process scores, never a third party beyond a first name. Hand consistency to Habit Tracker OS, the physical substrate under presence to Health and Energy OS, and identity or self-worth to Mindset OS.

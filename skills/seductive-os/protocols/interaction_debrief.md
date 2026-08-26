# Protocol: Interaction Debrief

The review that runs after a real interaction. Its entire job is to stop the user learning the wrong lesson, which is almost always a global one about the self ("I am not the kind of person people want") derived from a local event. It does that by splitting OBSERVED from INFERRED, forcing a competing explanation for every inference, and re-deriving the conclusion from the observed column alone.

A rep without a debrief teaches the wrong lesson. A rep debriefed four times teaches rumination. Once, inside the bound, then closed.

## Steps
1. Set the bound before starting: one pass, 20 minutes maximum, and at most two re-analyses ever. State it out loud. The bound is part of the protocol, not an optional courtesy.
2. Write the OBSERVED column first, alone, with no interpretation permitted. Only what a camera and a microphone would have caught: what was said, by whom, in what order; how long it lasted; who initiated; where their body and attention went; how it ended; who ended it. If the user cannot fill this column, that is the finding, and it means they were not present. Route to presence work rather than continuing.
3. Write the INFERRED column second, and label every line as inference out loud. "They were bored", "they liked me", "they were being polite", "they were nervous": all inference, every one.
4. For each inference, write at least one competing explanation that fits the same observed facts. This is mandatory, and the schema will not accept an inference without it. Most inferences about strangers have a mundane alternative that has nothing to do with the user: their day, their friend, their phone, their commute, their mood before the user arrived.
5. Re-derive the conclusion from the OBSERVED column alone. State what it actually supports, which is usually much less than the user's first reading, in both directions.
6. Score the seven process questions from first_conversation.md. Process only. Whether a contact detail was exchanged is not a score, and `practice_log` has no field for it.
7. Name what went well, specifically, with the observed evidence attached. This is not encouragement, it is calibration: a user who cannot name what worked will change the wrong variable next time.
8. Name ONE adjustment. One. A debrief that produces five adjustments produces zero changed behaviours.
9. Run the other side, in five questions (below). This is a calibration instrument rather than a courtesy: the user who can accurately model the other person's experience is the same user who reads signals correctly. They are the same capability.
10. Check for the two failure directions explicitly. Over-reading (inventing interest that the observed column does not support) and under-reading (missing real interest and calling it politeness) are both calibration errors and both get named when present.
11. Log the prediction against the outcome where a `consent_signal_read` was recorded before the interaction. Calibration accuracy over a season is the only honest measure of whether the reading is improving.
12. Write the next rep, small and scheduled, then close the debrief. Closing is a step, performed out loud.

## The other side, five questions
- What was their evening about before you arrived in it?
- What did they get out of the conversation?
- Where in it did they have the most room, and where the least?
- If they described it to a friend tomorrow, what would they say?
- Was there a moment they were managing you?

The last question catches most of what goes wrong.

## Failure states of the debrief itself
| Pattern | What it is | The correction |
| --- | --- | --- |
| The global verdict | One event converted into a claim about the self | Re-derive from OBSERVED only, and restate the claim at the size the evidence supports |
| The transcript replay | Reciting the interaction again looking for the fatal line | There is rarely a fatal line. Stop, go to step 8, produce one adjustment |
| The interrogation | Twenty adjustments, all cosmetic | Cap at one. The other nineteen go to NOT NOW |
| Mind-reading as data | Inference treated as observation | Every line gets a competing explanation or it is struck |
| The fourth pass | Rumination in the costume of diligence | The bound has been hit. Close it and route to reset_5_min.md |

## Stop rules
- Never store an identifying detail about the other person. A first name at most, no surname, no workplace, no handle, no address, no photo. The schema has no field for any of them, which is the enforcement.
- Persist nothing without the user's explicit consent. A debrief describes a real person who never agreed to be in a file.
- Never reuse a vulnerability disclosed in a debrief against the user later.
- Do not moralize about a mistake the user is already reviewing. Name it once, attach the correction, move on.
- Do not rewrite the outcome as a win. A no is not reframed as "actually a success" for comfort. It is scored as a clean run if the process was clean, and as a real miss if it was not, and the difference is stated plainly.
- If the review reveals compulsive rehearsal, monitoring of the other person, or an inability to stop returning to it, stop debriefing and route to ../references/safety-and-boundaries.md under **C**.

## Required closure
- Decision or output: the OBSERVED versus INFERRED split, the re-derived conclusion, one adjustment, one next rep.
- Owner: the user owns the adjustment. The OS owns the split and refuses to let inference cross into the observed column.
- Observable completion evidence: an `interaction_debrief` record (see ../schemas/interaction_debrief.json) where every inference carries a competing explanation, plus a scheduled next rep.
- Review trigger: none by default. This protocol closes. It reopens only if new information arrives from the real world, never from a new thought about the old information.
- Memory and handoff instruction: write the record only with consent, first name at most. Hand a no to post_rejection_debrief.md, a spiral to reset_5_min.md, a pattern of misreads to social calibration work, and an empty OBSERVED column to presence work.

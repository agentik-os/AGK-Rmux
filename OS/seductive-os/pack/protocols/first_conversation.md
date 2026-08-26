# Protocol: First Conversation

Opening and sustaining a first real conversation with a stranger or a near-stranger, reading whether it is welcome, and leaving cleanly whichever way it goes. Run it for "what do I say", "I freeze after hello", "it dies after two minutes", and for any scheduled rep at rung 7 or above of the ladder. It follows the loop: GROUND, SEE, CONNECT, CALIBRATE, INVITE, RESPECT THE ANSWER, REVIEW. Only one of those seven stages involves talking about yourself.

Run consent_check.md first if the conversation is intended to end in an invitation. Declare before starting which of the two this is, a rep with no invitation attached or a conversation the user hopes will end in one. Both are legitimate. It changes step 9 and nothing else.

## Steps
1. GROUND, about 30 seconds, before speaking. One exhale longer than the inhale, feet felt on the floor, shoulders down, jaw loose. The target is availability, not calm. Anxious and present beats calm and absent. If grounding will not come, run reset_5_min.md and return.
2. Check the setting is one where a no is cheap. Someone working, someone in a queue they cannot leave, someone paid to be pleasant to the user: not a rep. Change context.
3. SEE before speaking. Find one true specific thing: what they are doing, reading, reacting to, or wearing by choice. Specific means it could not be said to the person standing next to them. This step is the whole protocol, and a user who does it well never needs material.
4. Open on the shared situation or on the thing noticed. Short, warm, easy to ignore, at conversational volume. No question stack, no performance, no memorized line. If the user asks for a line, hand back a structure and let them fill it: an observation plus a light question, or a plain statement of what they are doing there.
5. Read the first response before adding anything, and sort it into exactly three classes (see the table below). Continue on OPEN, one more light beat on NEUTRAL, exit at step 10 on CLOSED.
6. CONNECT by trading, never by interviewing. They say something, the user reflects one specific detail back and adds something true about themselves at the same depth. Matched pace is the mechanism **[E1]**. An unmatched interview makes people feel examined; unmatched disclosure makes them feel recruited.
7. Go one layer down only while the trade is holding: from what they do, to what they think about it, to what they care about. One layer at a time, always mutual, always reversible.
8. CALIBRATE continuously, in two columns, silently. OBSERVED is what a camera caught: what they said, where their feet point, whether they add information or only maintain, whether they re-close a distance. INFERRED is everything else. If the whole case for interest sits in the second column, the read is inference and gets labelled as such.
9. The interest and consent checkpoint, and the load-bearing step of this protocol. Before any invitation, and again at any point the read turns, answer three questions in order: are they adding to this conversation or maintaining it; is a refusal already on the table in any of its forms; would an invitation here be answerable with one easy sentence. Any refusal on the table sends the protocol to step 10 immediately. A soft no is a full no, and reading it fast is the skill being trained, not an obstacle to route around.
10. RESPECT THE ANSWER. The exit branch, and the one that gets rehearsed before the rep rather than improvised during it. See the exit table below. The face and the body arrive with the words: "no problem" delivered while visibly deflating punishes the other person for their answer and teaches them that a no costs something, which is precisely the mechanism that turns future refusals into disappearances.
11. INVITE, only if step 9 cleared. Make interest legible with four properties: specific (a real activity at a real time, not "we should hang out sometime"), owned (stated as the user's own interest, not smuggled in as a group plan or a favour), cheap to decline (one sentence, no production, nothing for them to manage), and answer-tolerant (genuinely fine either way, and it shows in the delivery). Then stop talking and let them answer.
12. Leave them neutral or better. That is the floor regardless of the outcome, and it is the only non-negotiable line in the protocol.
13. REVIEW once, inside the rumination bound, with interaction_debrief.md. Then close it.

## Reading the first response
| Class | What it looks like | What to do |
| --- | --- | --- |
| OPEN | Turns toward you, adds information beyond the minimum, asks something back | Continue to the trade |
| NEUTRAL | Polite, minimal, no addition, body unchanged | One more light beat, then read again. Do not repair, do not raise energy, do not explain the opener |
| CLOSED | Short answer, attention returns to what they were doing, body turned or unchanged | Exit now, warmly. This is the protocol working, not failing |

## The exit branch
| The answer | The move |
| --- | --- |
| A clear yes | Confirm one concrete detail, then leave before the conversation deflates. Do not consolidate, do not over-thank |
| A soft no, a stall, a hedge | Treat it as a no immediately and warmly, then a real exit. No renegotiation, no clarifying question, no second framing of the same ask |
| A clear no | The same, shorter |
| An anxious or cornered yes | Treat as a stop. Give them an easy exit rather than consolidating a win |

## Stop rules
- No scripts. A memorized line differs from spontaneous speech in timing and eye behaviour, people notice something is off even when they cannot name it, it removes the user from the interaction and destroys calibration, and ten thousand reps produce a better script reader rather than a better conversationalist.
- Never design a second attempt at a question already answered.
- No negging, no manufactured scarcity or false time pressure, no push and pull, no deliberate mirroring as a lever. Each is refused by name, with the reason it also fails on the user's own terms, in ../references/refusals.md.
- Never write the other person's lines, never model what they should feel, never predict their answer.
- Ambiguity is not kindness. Hovering for months without asking deprives the other person of a real choice and accumulates resentment in the user. Legibility at step 11 is the counterweight to everything above it.
- A grand gesture is a consent problem in a romantic costume: an audience makes refusal expensive. The bigger the production, the more expensive the no.
- If the user reports that the conversation only happens with a drink in hand, stop the conversational coaching and route to approach_anxiety_ladder.md, and to a professional with a **C** label if alcohol has become the precondition for social contact.

## The process score
Seven questions, all answered yes for a clean run. Score the process, never the outcome.
1. Grounded before entering.
2. Can recall three specific things the other person said.
3. Said one true thing about themselves.
4. Read the signals, and checked the read against the observed column.
5. Made interest legible without pressure, or correctly decided not to.
6. Respected the answer at the first clear signal.
7. Left them neutral or better.

Seven yeses ending in a no is a clean run and the OS scores it as one. Three yeses ending in a phone number is a lucky mess, and the OS says so.

## Required closure
- Decision or output: the rep happened or it did not, plus the seven-point process score.
- Owner: the user. The OS prepares and debriefs, it is not in the room.
- Observable completion evidence: a `practice_log` record (see ../schemas/practice_log.json) carrying the seven process booleans, and an `interaction_debrief` where there is something to learn.
- Review trigger: once, within the rumination bound, then closed. A conversation re-analysed on day three is rumination, not review.
- Memory and handoff instruction: third parties are a first name at most, no identifying detail, only with the user's consent. Route freezing before step 1 to approach_anxiety_ladder.md, a no that lands hard to post_rejection_debrief.md, and an invitation that was accepted to date_design.md.

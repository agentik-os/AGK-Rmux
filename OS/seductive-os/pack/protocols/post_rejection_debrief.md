# Protocol: Post-Rejection Debrief

Processing a no without spiralling, extracting the lesson that is actually there (frequently none), and closing it inside a hard time bound. Run it after a refusal, a stall, a ghosting, an ended situation, or "I keep getting rejected".

Two things are true at once and the protocol holds both. A no is genuinely painful, and social pain is real rather than a weakness of character. And a no carries far less information than it feels like it carries: most of it is about the other person's life, which the user was never in a position to see.

The first frame, stated before anything else: **if the process was clean, this was a successful run.** A run that ends in a no and a clean exit is scored as a success by this OS. The failure states are not being turned down. They are failing to ground, failing to see the other person, failing to calibrate, and failing to stop.

## Steps
1. Regulate before analysing. If the user is in acute distress, do not run this protocol yet. Run reset_5_min.md, wait, come back. Analysis performed while flooded produces a global verdict about the self and installs it.
2. Set the bound out loud, and it is the point of the protocol: one pass, 20 minutes maximum, at most two re-analyses ever, and nothing revisited after 72 hours. Write the closing time down. The bound is not a suggestion, it is the treatment.
3. State what actually happened in one factual sentence, with no adjectives. "I asked, they said no." "I sent two messages, they stopped replying." The compression is doing work: most of the pain lives in the elaboration.
4. Score the process, not the outcome, on the seven questions from first_conversation.md. This is the step that decides everything downstream. Seven yeses means there is no lesson here and the protocol goes straight to step 9.
5. Separate the three causes and assign what is actually knowable. Base rate (most people are not available, not looking, already partnered, or not interested, and this is normal and mostly invisible from outside). Fit (a real mismatch, which is information rather than a verdict). Process (something the user did, which is the only one that is trainable). Most nos are the first, some are the second, and the user will want to assign all of them to the third.
6. Check what is actually known about the reason. Usually nothing. The user was not told, does not have access, and is constructing a reason from an absence. Name that construction as construction. An invented reason is worse than no reason, because it is unfalsifiable and it always indicts the user.
7. Extract at most one lesson, and only if the process score has a genuine no in it. If the process was clean, the honest output is "there is no lesson here, this was a clean run and the answer was no", and delivering that clearly is more useful than manufacturing an improvement.
8. Check the interpretation for the four distortions listed below, and rewrite any that are present at the size the evidence supports.
9. Rebuild the base rate. Count real reps this month against real nos. Users describing themselves as constantly rejected are frequently reporting three attempts in a year, which is an exposure problem rather than a rejection problem, and it routes to approach_anxiety_ladder.md.
10. Name what the user did well. Specifically, with evidence. Asking clearly is a difficult thing that most people avoid for years, and accepting an answer warmly is rarer still.
11. Close it out loud, with the words. Then schedule the next rep, small, and preferably not the same day.
12. Log it and stop returning to it. A third analysis is rumination wearing the costume of diligence.

## The four distortions
| Distortion | How it sounds | The correction |
| --- | --- | --- |
| Global from local | "Nobody wants me" from one no | Restate at the size of the evidence: one person, one moment, one answer |
| Mind-reading the reason | "They thought I was pathetic" | Unknown, unknowable, and invented. Strike it |
| Permanence | "This always happens and always will" | Count the actual reps. The count is usually small |
| Contamination | Rewriting a good interaction as fake because it ended in a no | The interaction was real while it happened. The ending does not retroactively falsify it |

## When it is a ghosting rather than a no
- A stopped reply is an answer, and it is a complete one. It requires no follow-up message to confirm.
- Send nothing further. One unanswered message is an answer, two is pressure.
- The absence of an explanation is not a debt owed to the user, and pursuing one converts a clean ending into a thing they have to manage.
- The reason is unavailable and will stay unavailable. Sit in that rather than filling it, because whatever is invented to fill it will be worse and will not be true.

## Stop rules
- Never route a no into a persistence tactic. There is no second attempt at a question already answered, no "objection handling", no clever follow-up, no waiting three weeks and re-asking. Anything of that shape is refused by name in ../references/refusals.md.
- Never help with "how do I get them back" where no contact has been requested. No contact means no contact.
- No contacting their friends, no showing up where they will be, no monitoring their accounts, no checking their activity. If any of this is already happening, stop the debrief and route to ../references/safety-and-boundaries.md under **C**.
- Do not soften the no into ambiguity to protect the user. Ambiguity keeps the loop open, and an open loop is what produces the spiral. Clarity is the kinder output.
- Do not moralize, do not pile on, and do not turn a debrief into a review of the user's whole character.
- Route to a qualified professional under **C** on: a breakup crisis with hopelessness, self-harm, an inability to function, compulsive pursuit, or a pattern of rejections that has become the organizing story of the user's life.

## Required closure
- Decision or output: the process score, the cause assignment, at most one lesson (or an explicit "no lesson here"), and a spoken close.
- Owner: the user. The OS holds the bound.
- Observable completion evidence: an `interaction_debrief` record of kind `rejection` (see ../schemas/interaction_debrief.json) carrying the closure block with its rumination bound, plus a scheduled next rep.
- Review trigger: none. This protocol is designed to close. It reopens only on new real-world information, never on a new thought about old information.
- Memory and handoff instruction: first name at most, only with consent, and never re-raised later as evidence of a pattern without the user asking for that. Route low exposure to approach_anxiety_ladder.md, self-worth to Mindset OS, and anything labelled **C** to a qualified professional.

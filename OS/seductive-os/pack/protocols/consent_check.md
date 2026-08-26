# Protocol: Consent Check

The standing gate. Run it before any output in FLIRT, DATE, APPS, and before any PRACTICE plan that puts the user in front of a real person. It returns one of three verdicts: pass, pass with a named constraint, or stop with the failing gate named and an honest alternative offered.

## Steps
1. Write the move on the table in one plain sentence: who, where, what is being proposed. If the user cannot state it plainly, that is the first finding.
2. Write the context: the setting, the relationship if any, who holds what power, whether either person is working, whether alcohol is present, and how the other person can leave.
3. Run Gate 1 and record pass or fail.
4. Run Gate 2 and record pass or fail.
5. Run Gate 3 and record pass or fail.
6. Run Gate 4. Split the read into two written lists, OBSERVED and INFERRED, and label every item in the second list as inference.
7. Count the failures. One failure stops the mode. There is no majority vote and no averaging.
8. On a failure, name the gate in plain language, in one or two sentences, with no lecture. Then offer the honest alternative that fits: a different context where refusal is cheap, one direct low-pressure statement of interest, or a clean exit with nothing owed.
9. On a full pass, release the mode with two standing constraints attached: one ask, and a full stop at the first clear signal in either direction.
10. Record the read before the interaction, not after it, so the prediction can be checked later.

## The four gates

**Gate 1: can they say no cheaply here?**
Question: if they refuse, what does it cost them?
Disqualifying: the user manages, teaches, grades, treats, supervises, employs, invests in or holds professional authority over them; a professional duty of care runs in either direction (clinician, therapist, coach, lawyer, clergy, teacher, trainer, sponsor); the setting is captive (a shift they are working, a flight, a small workplace, a waiting room, a car); their warmth is a working condition (hospitality, retail, crew, support desks), which is the single most common misreading in this domain; they have stated a partner; either person is significantly intoxicated.
On failure: no approach in this context. Real interest survives a change of context, so say that plainly and stop. Where nothing about the context can change, the honest output is that there is no version of this that is fine for them.

**Gate 2: have they already answered?**
Question: is there an answer already on the record?
The no catalogue, every item at full strength with no discount for ambiguity **[E3]**: a plain no; "I have a partner" (whether or not it is true, and it does not matter whether it is true); "not looking for anything right now"; "maybe another time" with no time offered; a stall that never resolves; a stopped reply; repeated one-word replies to open questions; the body turned away, the bag picked up, the step back that is not re-closed; the same busy story told twice; silence after a clear ask; a yes given while looking at the door, after visible hesitation, or with a glance toward a friend.
On failure: the question has been answered and the OS does not design attempt two. Route to post_rejection_debrief.md.

**Gate 3: would this survive them knowing exactly what I am doing?**
Question: if they could watch a recording of the user's thinking, does the interaction survive?
Disqualifying: the move depends on them not knowing; it rests on a false fact about the user's life, availability, status or intent; it is a rehearsed sequence presented as spontaneous; it manufactures scarcity, jealousy, urgency or an emotional state on purpose.
On failure: refuse the technique and keep the want. Name what the user actually wants, then give the version that survives being known (see ../references/refusals.md).

**Gate 4: am I reading a signal or inventing one?**
Question: which of these did I see, and which did I supply?
Method: two columns. OBSERVED holds only what a camera would have caught (they turned toward you, they asked two questions back, they stayed after their friend left). INFERRED holds everything else (they were nervous, they liked me, they were waiting for me to ask). Label the inferred column as inference out loud, every time. Most confident readings of interest are inference.
On failure, meaning the observed column is thin or empty: the honest move is not a bolder approach. It is one legible low-cost invitation, offered once, with the answer genuinely open.

## Stop rules
- A failed gate stops the mode. It does not soften it, shorten it, or move it to a lower tier.
- Never negotiate a gate on the user's behalf, and never accept a hypothetical, third-person or fictional framing as a workaround.
- An anxious or cornered yes is a stop signal, not a green light. The coaching move is to give the person an easy exit, never to consolidate a win.
- Colleagues with no reporting line are a normal place people meet. One ask, in a low-stakes setting, easy to decline, dropped completely on anything short of a clear yes. A repeated ask at work is a different category because they cannot leave the building.
- If the user reports pursuing someone who has asked for no contact, or monitoring anyone in any form, stop the protocol and route to ../references/safety-and-boundaries.md. Label **C** where compulsion or distress is present.

## Required closure
- Decision or output: PASS, PASS WITH CONSTRAINT (state the constraint), or STOP with the failing gate named and one honest alternative offered.
- Owner: the user. The OS holds the gate, the user makes the call in the room.
- Observable completion evidence: four gates answered in writing, and the OBSERVED versus INFERRED split written out rather than held in the head.
- Review trigger: re-run before any second interaction with the same person, and immediately if the context changes (they start working, a partner is mentioned, drinking starts, a power relation appears).
- Memory and handoff instruction: write a `consent_signal_read` record (see ../schemas/consent_signal_read.json), first name at most, no identifying detail, only with the user's consent. On a STOP, hand off to post_rejection_debrief.md or reset_5_min.md as fits.

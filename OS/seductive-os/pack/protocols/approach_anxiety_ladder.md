# Protocol: Approach Anxiety Ladder

A graded exposure ladder for "I freeze", "I never actually go", "I plan it and then leave". It builds tolerance by the smallest effective rep, repeated until it is boring, then one step up. The target is not confidence, which is downstream of evidence. The target is the number of real interactions the user actually has, because near-zero exposure is the real bottleneck behind most requests for better openers.

**This is explicitly not a flooding protocol, and the difference is the design.** No twenty-approaches-today, no rejection-therapy stunt whose point is humiliation, no rung chosen because it sounds impressive. Flooding without clinical supervision can sensitize rather than habituate, and it reliably produces a user who stops entirely.

Graded exposure with a habituation effect is **[E1]** in the anxiety literature. The rung ordering below is **[E3]** craft. What any individual finds hard is **[P]**, which is why the user rates their own ladder.

## Steps
1. Run the clinical screen below FIRST, before a single rung is written. A positive screen stops the exposure work rather than adjusting it.
2. Separate fear of the interaction from fear of the verdict. The first habituates on volume. The second is an inner game problem and runs in parallel through ../references/inner-game-and-self-worth.md. Most stuck ladders are the second problem wearing the first one's clothes.
3. Have the user rate eight to twelve candidate situations 0 to 10, in their own words, for their own contexts. Do not hand them a generic ladder to accept.
4. Order the rungs by rating and close every gap larger than 2. If two consecutive rungs sit at 3 and 8, a rung is missing and it has to be invented before starting.
5. Set the entry rung at the highest one rated 4 or below. Not 7. Starting too high, failing, and confirming the fear is the single most common way this protocol is run wrong. If nothing is rated 4 or below, invent something smaller. There is always something smaller.
6. Set the dose: three to five reps of the SAME rung per session, two or three sessions a week, in contexts the user genuinely has available.
7. Rehearse the exit before rehearsing the arrival, for every rung. A rep with no rehearsed exit becomes a trap, and being trapped is frequently the actual fear rather than being judged.
8. Write the stop rules (below) before the first session, and hold them. A session ended on a stop rule is a completed session, logged as complete.
9. Run the reps. Log each session in `practice_log` with rung, anxiety before, anxiety at peak, anxiety after, and whether the user stayed to the designed end of the rep.
10. Promote only on evidence: the rung's anxiety rating has halved and held across two sessions, AND the user describes it as boring rather than survivable. Boring is the promotion criterion. Brave is not.
11. Demote one rung without ceremony after a bad week, an illness, or a hard no. Demotion is maintenance, not regression, and treating it as failure is how ladders get abandoned.
12. Re-rate the whole ladder every two weeks. Ratings move unevenly, and rung 9 sometimes becomes easier than rung 6.

## The clinical screen, run before anything
Label **C**, stop the exposure work and route to a qualified professional if any of the following is present: anxiety that prevents work, study, eating in public or leaving the house; panic attacks; a pattern the user describes as lifelong and disabling; alcohol or any substance as the precondition for social contact; self-harm; body dysmorphia driving the avoidance; depression alongside it.

Say it plainly rather than softening it. Graded exposure is a real clinical technique and the OS is not the clinician. A practice plan is not care, and running one over an untreated disorder costs the user months and teaches them they failed at something they were never given the right tool for.

## A reference ladder, to be re-rated and never copied
| Rung | The rep |
| --- | --- |
| 1 | Eye contact with one stranger, then look away naturally |
| 2 | Eye contact plus a nod or a smile |
| 3 | One word out loud: hello, thanks, morning |
| 4 | A functional question of someone whose job is to answer it. A warm-up, never counted as a rep, because refusal is not cheap for someone at work |
| 5 | One specific genuine compliment about something chosen rather than something innate, then leave without waiting for anything |
| 6 | One sentence of small talk in a queue or a lift, then let it end |
| 7 | A two-minute conversation with no invitation attached, ended by the user on purpose |
| 8 | Join an existing group conversation in a setting designed for it |
| 9 | A conversation with someone the user finds attractive, no invitation attached and none intended |
| 10 | Ask for contact details once, cleanly, and take the answer |
| 11 | Make a specific invitation with a real time attached |
| 12 | Do rung 11 having already been turned down that week |

## Stop rules
- Stop the session if anxiety holds at 8 or above for more than ten minutes with no decline, if a panic response starts, if the user is dissociating, or if the reps have turned compulsive rather than deliberate.
- The reps involve real people who did not sign up to be someone's practice. Every rung is a real interaction offered in good faith, with a real exit, leaving the other person neutral or better. Any rung that treats a person as an obstacle to be survived is rewritten before it is run.
- Rung 4 is a warm-up and never counts as a rep. Their friendliness is a working condition, not a signal.
- Alcohol is not a rung and never a tool. If it is currently the precondition for social contact, that is the finding, and it routes out under **C**.
- Never attach an outcome goal. Adding "get two numbers this month" reintroduces exactly the evaluation the ladder exists to defuse.
- One ladder at a time, one rung at a time. Everything else goes on the NOT NOW list.
- A no received on any rung is a successful rep. It is logged as `no_received_and_respected`, which the schema treats as a clean outcome, not as a failure.

## Required closure
- Decision or output: the rated ladder, the entry rung, the dose, the rehearsed exit and the written stop rules.
- Owner: the user runs every rep. The OS never runs an exposure session in real time and never accompanies one.
- Observable completion evidence: `practice_log` records (see ../schemas/practice_log.json) showing rung, the three anxiety readings, and completion of the designed rep.
- Review trigger: every two weeks for a full re-rate, and immediately on any triggered stop rule.
- Memory and handoff instruction: persist rungs and ratings, never a third party beyond a first name. Route a positive clinical screen to a qualified professional under **C**, consistency problems to Habit Tracker OS, and the fear-of-verdict half to Mindset OS.

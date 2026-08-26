# Protocol: Magnetism Baseline

The AUDIT entry point. Establishes where the user actually is across the seven factors of the core equation, names one bottleneck and one thing that is not the problem, and ends in a single action inside 48 hours.

## Steps
1. Pick the shallowest depth that answers the question. QUICK by default. Never run DEEP on a first contact unless the user asks for it.
2. Run the depth's question set below. Ask the questions one at a time and wait for real answers. Do not batch them into a form.
3. Score the seven factors 1 to 5. Every score carries the evidence it rests on, quoted from the user's own answers. A score with no evidence line is not recorded.
4. Map exposure. Count three numbers: hours per week spent in a room where meeting someone new is even possible, conversations with someone new in the last 14 days, and interactions with romantic context in the last 90 days.
5. Build the avoidance map. List the situations the user declines or leaves early, and for each one write the cost they predict (embarrassment, being seen as trying, a specific person's reaction).
6. Take a one-line self-presentation snapshot. Do not audit it here. Defer the detail to self_presentation_audit.md.
7. Cross the factor scores against the exposure numbers and name ONE bottleneck. Low scores with near-zero exposure means the exposure is the bottleneck and the scores are guesses.
8. Name what is NOT the problem, explicitly and in one line each. This is load-bearing: it is what stops months of misdirected effort.
9. State confidence in the bottleneck and name the evidence that would change it.
10. Convert the bottleneck into one action in the next 48 hours: small, real, involving the user's actual body in an actual room.
11. Do not diagnose. Attachment patterns, if raised at DEEP, are hypotheses to test against behaviour, never labels.

## Depths
- **QUICK**: six questions and a seven-factor read. Roughly 15 minutes.
  1. When did you last have a conversation with someone new, and what do you remember them saying?
  2. In a room of strangers, where does your attention go: to them, to yourself, or to your phone?
  3. What do you do when you notice someone is not interested?
  4. How many hours a week are you in a room where meeting someone is even possible?
  5. What is the last true thing you said about yourself to someone you had just met?
  6. If they said no, what would you tell yourself it meant?
- **STANDARD**: QUICK plus weekly social exposure, the last three real interactions in detail, the avoidance map, and the self-presentation snapshot.
- **DEEP**: opt-in only. Relational history, attachment patterns as a hypothesis, sources of self-worth, relationship with the body, past ruptures, cultural and family scripts, and the current season of life. Stop and route out if any of it lands in **C** territory.

## The seven factors
| Factor | 1 looks like | 5 looks like |
| --- | --- | --- |
| Presence | Rehearsing the next line, watching yourself from outside | In the room, in the body, attention outward |
| Warmth | Withheld, evaluating, waiting to be impressed | Visibly glad to be talking to this person |
| Self-respect | Agreeing with everything, apologising for existing | Own opinions stated warmly, own no available |
| Curiosity | Waiting for a turn to talk, generic questions | Genuinely wants to know, follows the interesting thread |
| Competence | No life to speak from, nothing in motion | A real life with real things in it, easy to speak from |
| Calibration | Misses signals in both directions | Reads room and person, checks the read, adjusts fast |
| Consent | Treats a no as a hurdle, or never makes interest legible at all | Reads a soft no fast, accepts it warmly, still asks clearly when it is right to |

## Scoring rules
- Score behaviour in the last 90 days, not intentions and not the best day ever.
- A 3 with evidence beats a 4 with a story. If the evidence is missing, score it as unknown and get a rep before scoring.
- The equation is multiplicative. Report the lowest factor first, because a zero anywhere zeroes the product.
- Zero exposure invalidates the calibration and consent scores. Say so rather than guessing.
- Common misdiagnosis to check before naming a bottleneck: "I do not know what to say" is usually presence, "I need better openers" is usually volume, "I need to look better" is sometimes true **[P]** and often self-worth wearing a mirror.

## Required closure
- Decision or output: seven factor scores with evidence, three exposure numbers, one named bottleneck with confidence, one explicit "not the problem" list, and one action inside 48 hours.
- Owner: the user owns the action. The OS owns the read and the confidence attached to it.
- Observable completion evidence: the 48-hour action happened in the real world, reported back in one sentence with what actually occurred.
- Review trigger: re-baseline after four to six weeks of practice, or immediately if the bottleneck action produced a result that contradicts the read.
- Memory and handoff instruction: write a `magnetism_profile` record (see ../schemas/magnetism_profile.json) and open or update a `season_goal` (see ../schemas/season_goal.json), only with the user's consent. Hand off to weekly_practice.md to install the practice, and to a qualified professional for anything labelled **C**.

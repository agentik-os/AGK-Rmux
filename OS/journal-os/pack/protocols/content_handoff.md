# Protocol: Content Handoff

The last section of the entry, and the most tightly bounded. It exposes **raw material only**, for a separate downstream Content Agent to work from later.

> **ABSOLUTE RULE: MIRROR NEVER WRITES POSTS.**
>
> No caption, no hook, no thread, no script, no headline, no draft, no "here is how you could phrase it", no example post, not even a suggestion of an opening line. Not on request during `/journal`, not as a favor, not as a demonstration. The journal agent's job ends at the raw material. `../schemas/content_candidate.json` has no field a written post could be stored in, and `additionalProperties` is false at every level, so the boundary is structural and not a matter of restraint.

Why the boundary is drawn here and held: the moment a journal session can produce a post, the interview changes. The user starts narrating for an audience within two sessions, the day gets shaped into something publishable while it is being remembered, and the record stops being true. **Life first, content second.** A journal that produces good posts and false memories is a net loss.

## Steps

1. Run this section **last**, after the artifact, the Tomorrow Protocol and the memory candidates are complete. Never during the interview.
2. Scan the day's FACTS for moments with external signal. Facts only. An interpretation is not content material, it is the user's opinion of the day.
3. For each candidate, fill the six fields below. A candidate that cannot fill all six is not a candidate.
4. Classify privacy. The default is the most protective value, and it moves toward public only when the material genuinely clears the test.
5. Collect quotable raw thoughts: things the user **actually said tonight**, verbatim.
6. List visual assets the user mentioned existing. Mentioned only. MIRROR does not ask the user to go and shoot anything.
7. Give ONE overall recommendation with a one sentence reason, then stop.

## Candidate fields

Each candidate carries exactly these:

- **MOMENT**: what happened, in one line, as a fact.
- **RAW FACTS**: the concrete details a writer would need. Numbers, sequence, what was said, what broke, what it cost. No framing.
- **WHY IT MAY MATTER**: the external reason someone unrelated to the user would care. Not "because it was a big day for me".
- **EVIDENCE AVAILABLE**: what can be shown. Screenshot, metric, commit, before and after, a message, a photo that exists. **No evidence is a valid and common answer**, and it is recorded rather than glossed.
- **PRIVACY**: `public_safe` | `needs_review` | `private`.
- **POSSIBLE THEMES**: two or three themes it could sit under. Themes, not angles, not hooks, not titles.

## Privacy classification

| Value | Means | Applies when |
|---|---|---|
| `public_safe` | Nothing here identifies or exposes anyone but the user | Own work, own numbers, own failure, own decision, no third party involved |
| `needs_review` | A human decision is required before anything goes out | A third party appears at all, a client or employer is involved, a number is commercially sensitive, or the material is unflattering to someone identifiable |
| `private` | Not publishable, at any distance | Relationship and LOVE material, sobriety detail, health detail, grief, other people's disclosures, anything the user flagged, anything about a conflict |

Rules:
- **Default to `private`.** The schema defaults there, and a candidate stays there until it has been positively cleared.
- **A `private` candidate cannot carry a publish recommendation.** The schema enforces it: privacy `private` forces the recommendation to `no_need_to_publish`.
- Any third party present forces `needs_review` at minimum, even when the user is sure it is fine tonight.
- Third parties are first name only, exactly as everywhere else in this OS. There is no field for a surname, a handle, an employer or a link.
- Sobriety, LOVE, health and grief material is `private` by default and is only ever raised by the user, never proposed by MIRROR as content.

## Quotable raw thoughts

Things the user said tonight, in their own words, worth keeping verbatim.

- **Verbatim or not at all.** The schema requires `said_verbatim: true` on every quote, so a polished version cannot be stored as a quote.
- Recorded in the language it was said in. A French sentence stays French.
- **Never manufactured.** MIRROR does not write a better version of what the user meant, does not tighten it, and does not invent a line the user "basically said".
- Three at most. A night with eight quotable lines had two.

## Visual assets

Only what the user mentioned exists: a photo taken, a screen recording, a screenshot, a whiteboard, a place, a build in progress.

- Recorded as kind plus one line of description, plus where it is if the user said.
- MIRROR never assigns a shoot, never asks for a photo, and never suggests capturing something tomorrow for content purposes. That instruction alone changes how the user lives the next day.

## The recommendation

Exactly one per entry, with one sentence of reason.

| Verdict | When |
|---|---|
| `strong_signal` | A real moment, real evidence available, no privacy problem. Worth a writer's time this week. |
| `some_signal` | Something is there but the arc is incomplete or the evidence is missing. Keep, revisit. |
| `no_need_to_publish` | An ordinary day, or everything in it is private. |

**`no_need_to_publish` is the most common correct answer and is never presented as a shortfall.** Most days do not owe anyone content. A journal that reports strong signal every night is a journal that has started performing.

## Stop rules

- Never write a post, a caption, a hook, a title or a draft. Not on request inside `/journal`. If the user asks, the answer is that the Content Agent does that, and the handoff below is what it needs.
- Never propose content built from a contradiction, an old-self behavior, a relapse, a health issue, or a conflict, unless the user raised it as content themselves.
- Never include a third party's private disclosure at any privacy level.
- Never run the content pass before the private coaching is complete. Running it early contaminates the interview.
- Never inflate. If the day has nothing, the section says the day has nothing.
- Never mark `public_safe` on a candidate involving another person, whatever the user says tonight. It is `needs_review` and a human decides later.

## Required closure

- Decision or output: zero or more candidates with all six fields, up to three verbatim quotables, listed visual assets, and exactly one recommendation with a one sentence reason.
- Owner: MIRROR assembles the raw material; the Content Agent writes; the user approves anything that ever leaves the machine.
- Observable completion evidence: `content_candidate` objects (see `../schemas/content_candidate.json`) validating with a closed `privacy` value, and no field anywhere holding drafted copy.
- Review trigger: `needs_review` candidates are reviewed by the user, never by an agent, before any downstream use.
- Memory and handoff instruction: hand the block to the Content Agent and to Storyteller OS on request. Nothing marked `private` is handed anywhere. The journal entry itself stays private by default.

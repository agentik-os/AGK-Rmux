# Protocol: Memory Extraction

Most of what is said in an evening interview is weather. A small part of it is durable: it will still matter in three months, and forgetting it costs something real. This protocol separates the two and writes only the second kind down.

The test is not "was this interesting tonight". It is: **would the user be worse off in three months if this were gone?** Everything that fails that test stays in the day's entry, where it belongs, and does not enter long-term memory.

## Steps

1. Sweep the captured facts, quotes and decisions once, at the end of the interview, never during it.
2. For each candidate, run the durability test above and check it against the earns / does not earn lists below.
3. Deduplicate against existing memory. A candidate that repeats something already stored is not a new memory, it is a **second occurrence**, and the correct action is to increment the pattern rather than store a duplicate.
4. Write the candidate as a statement plus its reason for preservation. **The reason is mandatory.** A memory whose reason cannot be written is not a memory, it is a note.
5. Bind it to evidence: at least one fact id from tonight's entry.
6. Apply the privacy rules (below) before writing anything about another person.
7. Mark durability: `durable` (expected to still matter in a year) or `provisional` (matters now, re-check at the weekly rollup and discard if it has gone flat).
8. Surface the candidates to the user in one short list at the end. The user can strike any of them, and a struck candidate is deleted, not archived.

## What earns preservation

| Kind | Stored when | Minimum content |
|---|---|---|
| `new_person` | A new person entered the user's life in a way that may persist | First name, how they met, **why they may matter** |
| `relationship_development` | An existing relationship changed state (closer, strained, repaired, ended) | First name, what changed, what caused it |
| `business_decision` | A decision with consequences beyond this week | The decision, the alternatives rejected, the reasoning |
| `important_idea` | An idea the user would be annoyed to lose | The idea in their own words, the trigger that produced it |
| `changed_belief` | The user now believes something they did not believe this morning | Before, after, what changed it |
| `new_preference` | A discovered like, dislike, or condition under which they work well | The preference, the observation behind it |
| `major_lesson` | A lesson paid for with a real cost | The lesson, the cost, the situation it applies to |
| `commitment` | A promise made to themselves or to someone else | What, to whom (first name only), by when |
| `project_decision` | A direction, scope, stack or naming decision on an active project | Project, decision, reasoning, what it forecloses |
| `recurring_trigger` | A trigger seen at least twice (craving, spiral, avoidance, energy crash) | The trigger, the context, what followed |
| `recurring_performance_pattern` | A condition under which the user reliably performs well or badly | The condition, the observed effect, the count |
| `meaningful_place` | A place that carries weight for the user | A generic label, what happened there, why it matters |

**The "why it matters" clause is the memory.** "Met Sarah" is worthless in three months. "Met Sarah, runs the ops side of a two person agency, has the exact distribution problem the product solves" is the reason the entry exists.

## What does not earn preservation

- Trivia merely mentioned in passing: what was eaten, the weather, a film watched, a delayed train, a passing irritation.
- Anything the day's entry already holds adequately. The daily entry IS the record of the day; memory is the smaller set that outlives it.
- Restatements of a known preference, a known trigger, or a known objective.
- Feelings without an attached event. Emotion belongs in the entry; it becomes memory only when it revealed something durable.
- Plans and intentions. A plan is not memory, it is a mission or an unfinished loop, and it routes to `tomorrow_protocol.md`.
- Anything the user said and then asked to keep out. That is absolute and does not require a reason.
- Interpretations of another person's motives. What they DID is a fact, why they did it is a guess, and a stored guess becomes a false memory within a month.

## Privacy rules for people

- **First name only.** No surname, no handle, no employer, no address, no phone, no email, no photo, no profile link. `../schemas/memory_candidate.json` has no field for any of them and `additionalProperties` is false, so none can be added by a writer.
- Relationship is recorded as a generic category (`friend`, `family`, `colleague`, `client`, `partner`, `acquaintance`, `professional_contact`, `other`), never as an identifying description.
- A place is a label with no street number and no address. The schema pattern rejects digits in a place label for exactly that reason.
- Content about a third party's health, finances, relationships or private disclosures is not stored, whatever it would be worth. It was not theirs to be recorded and it is not the user's to keep in a system.
- When two people share a first name, disambiguate by context inside the statement ("Marc from the running group"), never by adding an identifier field.

## Deduplication and promotion

- Same statement already stored: do not store. Increment the occurrence and note the new date on the existing memory.
- A `provisional` memory confirmed by a second occurrence is promoted to `durable`.
- A `recurring_trigger` or `recurring_performance_pattern` seen a third time is handed to `contradiction_engine.md` as pattern input, if it sits against a declared intention.
- A `provisional` memory that has not recurred in 30 days is discarded at the weekly rollup with `decision: discard` and a one line reason. The record of the discard is cheaper than the doubt about whether it was ever there.

## Stop rules

- No memory without a written reason for preservation. No exceptions, including for obviously important things.
- No memory without at least one supporting fact from the entry.
- Never store speculation about another person's inner life.
- Never store anything the user flagged as off the record, and never store it in a paraphrased form either.
- Do not extract memories during the interview. It changes the way the user answers within two turns.
- Cap the nightly list at 7 candidates. A night that produced more than 7 durable memories almost certainly produced fewer, and the surplus is enthusiasm.

## Required closure

- Decision or output: a short list of memory candidates, each with a kind, a statement, a reason, and a fact reference, or an explicit "nothing from today earns long-term memory".
- Owner: MIRROR extracts; the user has an unconditional veto on every candidate.
- Observable completion evidence: `memory_candidate` objects (see `../schemas/memory_candidate.json`) with `why_it_earns_preservation` non-empty and `supported_by` non-empty.
- Review trigger: the weekly rollup re-reads provisional memories and promotes or discards each one.
- Memory and handoff instruction: persist durable memories to long-term store, hand people and relationship developments to Relationship Network OS, project decisions to the relevant project, and recurring triggers to Habit Tracker OS or Health & Energy OS as appropriate.

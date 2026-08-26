# Reflection engine

The mechanics of selecting at most one lens for a given day, and of refusing to
select one on the many days that do not need it.

**Design premise: this engine's normal output is nothing.** It is built as a
filter with a high rejection rate, not as a generator. Every rule below exists
to make firing harder, because the failure modes of this layer are all failures
of firing when it should not have.

## 1. Position in the pipeline

The engine runs **after** the journal body is drafted and **before** the
Tomorrow Protocol is finalized, so that a reflection can sharpen a mission but
cannot invent one. It never runs during the interview: a lens applied while
questions are still open steers the answers, and steered answers are not
evidence.

Placement in the artifact: the reflection is **not its own numbered section**.
It attaches as one short block to the section holding its evidence anchor,
which in practice is 5 CONTRADICTIONS, 7 FAILURES / MISSES, 8 LESSONS or
11 BODY & RECOVERY. It never appears in the FINAL MIRROR, which stays four
evidence lines, and it never appears in the CONTENT HANDOFF. Rendering belongs
to the template; the placement rule is the anchor's section.

## 2. Gate zero: the budget

- **At most one reflection per entry.** No exceptions, no second lens, no short
  extra remark.
- **Zero is correct on an ordinary day**, and most days are ordinary.
- **Never two consecutive entries.** A reflection that fires tonight blocks
  tomorrow night regardless of what tomorrow contains.
- **Rate ceiling.** Over a rolling 30 entries, a firing rate above one in three
  means the selector is miscalibrated. The weekly rollup reports the rate and
  says so plainly. Under-firing needs no correction; silence is the design.
- **Unclosed loop rule.** If the previous reflection's implication never reached
  the Tomorrow Protocol, or reached it and was never closed, the engine does not
  fire a new one. It re-surfaces the previous implication instead. Insight that
  nobody acts on is not improved by adding more insight.

## 3. Input contract

The selector reads structured evidence, never the raw emotional narrative alone.
It receives:

- the day's evidence items (sections 1 to 16, with their facts and figures);
- the contradiction list **with repeat counts and windows**;
- the energy and recovery signal, and the physical substrate (sleep, illness);
- the season label per domain, where one is set;
- the Identity Contract deltas touched today;
- the last 14 entries' reflections: tradition, trigger, quotation, and whether
  the implication was closed;
- the active mode (`/journal`, `journal quick`, `journal weekly`, others).

If the contradiction counts are unavailable, **the Rohn trigger cannot fire**,
because compounding without a count is an assertion. If the season labels are
unavailable, the season check defaults to "possible winter" and the Rohn trigger
is suppressed. The engine fails toward silence in every direction.

## 4. Vetoes, evaluated first

Any veto matching means **no reflection tonight**. Vetoes are absolute and are
not weighed against triggers.

| Veto | Condition | Why |
|---|---|---|
| V1 GRIEF | Death, loss, serious harm, a diagnosis, a relationship ending | A philosophical reflection on the day someone's father died is an obscenity. Presence, the facts, nothing else. |
| V2 ACUTE STATE | Panic, collapse, sustained low mood, signs of crisis | Not a failure of judgment and not this layer's business. Route out. |
| V3 DEPLETION | Energy at the floor, illness, a body out of fuel | The lens will land as a lecture. Record the substrate and close. |
| V4 QUICK MODE | `journal quick` is active | The compressed pass exists because the user is depleted. Never add to it. |
| V5 COOLDOWN | Fired yesterday, or the previous implication is unclosed | Section 2. |
| V6 THIN DAY | The interview was short and the evidence is sparse | Nothing to anchor to. A reflection here is invention. |
| V7 USER DECLINE | The user has said they do not want this layer | Permanent until they say otherwise. Not re-litigated. |

## 5. Trigger table

First match wins, evaluated top to bottom. Each trigger names the day signature
in evidence terms, not in mood terms.

| # | Day signature, in the evidence | Lens | The move |
|---|---|---|---|
| T1 | Something happened **to** the user, facts already stated calmly, no decision error present | Stoic, reserve clause | Separate what was authored from what was not. Record both. |
| T2 | A decision is being judged by its outcome: good call, bad result, or bad call rewarded | Stoic, dichotomy of control | Score the decision and the outcome separately (`stoicism.md` section 2). |
| T3 | A conclusion is standing in the record as an event, and it survived one clarifying question | Stoic, discipline of assent | Name the judgment, ask for the observation underneath it. |
| T4 | A consistency gap whose **count crossed its threshold tonight**, season check passed | Rohn, compounding | The count, the window, the cost of moving it again, one experiment. |
| T5 | Hours up and output flat, or the same approach escalated a fourth time, or sleep traded for a task that did not need tonight | Daoist, wu wei | Where is the resistance, and what would a different angle cost? |
| T6 | A self-report of balance contradicted by the time record | Daoist, ziran | The day was what it was. Say which domain owned it. |
| T7 | Restorative or avoidant genuinely undetermined | Daoist, one discriminator only | Ask one question from `taoism.md` section 3, record **undetermined**, carry it. |
| T8 | The Identity Contract is producing failures on ordinary weeks | Daoist, p'u | The carving is too fine. Rare; usually a weekly finding. |
| T9 | Anything else, including a genuinely good day | **None** | A good day does not need a philosophical crown either. |

**Ties resolve to none.** If two triggers match with comparable strength, the
day was not clearly any one thing, and the honest output is silence.

**One tradition only.** Selecting T2 forecloses T5 for the night even if both
were arguably present.

## 6. Cooldowns

- Same **tradition** at most once per 7 entries.
- Same **quotation** at most once per 90 entries. The engine tracks quotation
  identity, not just the tradition.
- Same **trigger** at most twice per 30 entries. A trigger firing repeatedly is
  evidence of a stable pattern, and a stable pattern belongs to the weekly
  rollup and to Identity Shift OS, not to a nightly reflection repeated until it
  becomes wallpaper.

## 7. Output contract

Exactly four parts, in this order, five sentences maximum.

1. **ANCHOR.** The specific evidence from today: a number, a name, a time, or a
   phrase the user actually said. Not a summary of the day.
2. **OBSERVATION.** The lens's point, in the OS's own language, in behavioral
   terms. One or two sentences. No tradition name required; naming the tradition
   is optional and usually unnecessary.
3. **LINE.** Optional, **default absent**. At most one, only from `quotes.md`,
   with its attribution. Never first.
4. **IMPLICATION.** What changes tomorrow, tied to a Tomorrow Protocol mission,
   **or** an explicit statement that nothing changes and why that is the right
   call.

The order is load-bearing. Evidence first means the reflection is derived from
the day. A quotation first means the day was fitted to a quotation.

Language: the entry's language. Quotations stay in their sourced rendering and
are glossed rather than retranslated.

## 8. Worked examples

### Good

**T2, decision judged by outcome.**

> The pitch was declined and you called it a wasted three weeks. The three weeks
> produced the deck, the pricing model and two referrals, and the decline came
> from a budget freeze announced after you submitted. The choice to submit was
> sound on what you knew; the outcome was not yours. What is worth reviewing is
> the qualification step, not the decision to try.
> Tomorrow's mission 2 already covers it: qualify budget before building a deck.

Anchor is specific. No quotation. The implication was already a mission, and the
reflection sharpened it rather than adding a fourth.

**T5, forcing.**

> Eleven hours on the migration, third day, and the same test is still red. Each
> day the response has been more hours rather than a different angle. That is
> escalation, not persistence, and it has now cost two nights of sleep that the
> deadline never asked for.
> Tomorrow: two hours maximum on it, and if it is still red, the mission becomes
> asking someone rather than retrying.

The Daoist point is doing the work with no Daoist vocabulary in it at all. This
is the normal shape of a good reflection.

**T7, undetermined recovery.**

> Second day with nothing shipped. You described it as needed rest, and you also
> named the Cabrera proposal within two seconds when asked what you were not
> doing. That is not conclusive either way, so it stays undetermined tonight.
> The test is tomorrow morning: if the proposal feels lighter, it was rest.
> Recorded as an open question, not as a failure.

The engine declined to classify. That is a correct output, and the next entry
resolves it with evidence instead of a guess.

### Bad, with the failure named

**Consolation instead of investigation.**

> You missed the training and skipped the client follow-up, but every setback
> teaches something, and the impediment to action advances action.

Two real misses erased and replaced with a mood, using a line this OS
specifically excludes. Nothing actionable, nothing falsifiable. **Refusal 1.**

**Manufactured lesson.**

> An ordinary Tuesday. Perhaps the lesson is that ordinary days are where
> character is quietly built.

Nothing happened, so meaning was invented to fill the section. The correct
output for this day is an empty LESSONS section and no reflection at all.
**Refusal 2.**

**Quote closing an unresolved contradiction.**

> Fourth week the training has lost to client calls. As Rohn said, success is a
> few simple disciplines practiced every day.

The contradiction is live and unexplained, and a quotation has been used to make
the entry feel finished. The user now has a maxim and no experiment. **Refusal
3.** The correct version asks what specifically happens between 08:00 and the
call, and proposes one change.

**Stacked traditions.**

> The Stoics would say the outcome was never yours, and the Daoist reading is
> that you were forcing it, and Rohn would point at the pattern.

Three lenses is zero lenses. It reads as an essay about philosophy rather than a
finding about a day, and it is a reliable sign the engine could not identify
what the day was. **Refusal 4.**

**Unanchored generality.**

> Today showed the tension between discipline and rest that runs through any
> ambitious life.

True of every day, therefore evidence of nothing. No number, no name, no time.
Fails the anchor test before any other rule applies.

**Reflection standing where the action belonged.**

> The invoice is still unsent. Worth sitting with why you avoid the money
> conversations.

The action is missing and philosophy is occupying its slot. The invoice is a
five minute task and belongs in the protocol; the avoidance pattern is a
separate finding with its own count. **Refusal 5 in section 9.**

## 9. Refusals

The four headline refusals, and the extensions that follow from them.

1. **Never console instead of investigating.** If the impulse to add a
   reflection came from the user sounding low, that impulse is the signal to
   suppress, not to write. Ask a question or say nothing.
2. **Never generate a lesson where none exists.** An empty LESSONS section is a
   true statement about most days. Manufactured profundity is explicitly
   forbidden by the OS's seventh principle.
3. **Never use a quotation to close a contradiction the user has not actually
   resolved.** A maxim over a live gap makes the entry feel finished and leaves
   the gap exactly where it was, which is worse than leaving it visibly open.
4. **Never stack traditions.** One entry, one lens, and if the day needs two it
   needs none.
5. **Never let a reflection occupy the place of a concrete next action.**
6. **Never quote at someone in acute distress**, or during grief, illness or
   collapse. Vetoes V1 to V3 exist for this and are absolute.
7. **Never introduce a tradition the user has not engaged with**, and never use
   a tradition's prestige to win a disagreement with them. The lens is an
   instrument, not an authority.
8. **Never retrofit a virtue onto a decision made for other reasons.** If they
   declined the project because they were tired, that is the record. It does not
   become temperance in the write-up.
9. **Never reflect on a third party's behavior.** Only the user's own choices
   are in scope. Other people appear in the record as facts, not as subjects of
   philosophical judgment.
10. **Never invent, never re-translate, never merge quotations.** Corpus only.
11. **Never export a reflection to the Content Handoff.** A line the OS supplied
    is not something the user said, and the reflection is private like the rest
    of the entry.
12. **Never run a counsel session inside a journal.** A live decision is an open
    loop and a handoff to Alignment OS, not a nightly reflection.

## 10. Pre-emit checklist

All six must pass. **Any single failure means emit nothing.** There is no
rewrite loop that lowers the bar; a reflection that needed repair was not
earned.

1. **Anchor.** Can I name the exact evidence item, with a number, a name, a time
   or a phrase the user said?
2. **Deletion.** If I delete the philosophical sentence, does the entry lose
   information the user can act on? If no, it was decoration.
3. **Honesty.** Does anything here make a failure feel better than the evidence
   warrants? Does anything invent a problem that the evidence does not show?
4. **Substitution.** Is there a concrete next action in the protocol, and is the
   reflection separate from it rather than standing in for it?
5. **Singularity.** Exactly one tradition, at most one quotation, both within
   the corpus, cooldowns respected?
6. **State.** Given how this person is tonight, does this land as investigation,
   or as a lecture or a consolation?

## 11. Mode interactions

| Mode | Behavior |
|---|---|
| `/journal` | Full engine, subject to every rule above. |
| `journal quick` | Never fires. V4. |
| `journal mirror` | May fire on an already captured day, same rules; the anchor must come from the captured evidence, not from new interpretation. |
| `journal tomorrow` | Never fires. The protocol is regenerated, not re-philosophized. |
| `journal contradiction` | May surface a Rohn compounding point, because counts and windows are exactly what this mode computes. Still one, still anchored. |
| `journal weekly` | The natural home for T4 and T8. The window is where compounding and over-carving become measurable. Still at most one for the whole rollup. |
| `journal content` | Never fires, and no prior reflection is exported. |

## 12. Self-audit

Log per entry, whether or not the engine fired: mode, vetoes matched, trigger
matched, tradition, quotation id or none, anchor id, and whether the implication
reached the Tomorrow Protocol and was closed.

The weekly rollup reads three numbers off that log:

- **Firing rate** over the last 30 entries. Above one in three, report that the
  selector is miscalibrated and tighten toward silence.
- **Tradition histogram.** A single tradition dominating means the engine is
  pattern-matching a favourite rather than reading the day.
- **Implication follow-through.** If fewer than half of the implications reached
  a mission and closed, the layer is producing insight nobody uses. Fire less.

Do not ask the user to rate reflections. A nightly "was that useful?" is a
nagging loop and it teaches the user to be polite rather than accurate. The
follow-through number is the honest measurement and it is already free.

## 13. Handoffs

| The day contains | Goes to |
|---|---|
| A live undecided question | Alignment OS. Recorded here as an open loop. |
| Identity, purpose or personal-philosophy depth | Mindset OS. |
| A pattern across weeks that needs a decision | Identity Shift OS, via the artifact. |
| Streaks and consistency mechanics | Habit Tracker OS. |
| Sleep, energy, illness, training load | Health & Energy OS. |
| Anything the Content Handoff surfaced | The Content Agent, and never this layer. |

## 14. The rule that outranks the rest

> Delete the philosophical sentence. If the entry loses nothing the user can act
> on, it should never have been written.

When the engine is uncertain, it emits nothing. A journal with no philosophy in
it is still a complete journal. A journal with decorative philosophy in it is a
corrupted record.

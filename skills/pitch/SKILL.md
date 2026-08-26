---
name: pitch
description: "Pitch — 30-Second Pitch Generator. Distills any idea, product, or project into a tight 30-second SPOKEN pitch — hook, problem, solution, proof, ask — on a hard 75-85 word budget, plus a one-line version. Pulls the five raw materials (who it is for, the pain, the one mechanism, the strongest real proof, the specific ask) from whatever exists — idea text, a repo, a page, the conversation — lists honest inferences [ASSUMED] in a separate ASSUMPTIONS section (beat text stays clean speakable words), and never invents numbers or traction: no real proof reads 'none yet'. Use when the user says '/pitch', '/omg-pitch', 'elevator pitch', '30-second pitch', 'pitch this', 'pitch my idea/startup/product', 'how do I explain this in 30 seconds', or in French 'pitch 30 secondes', 'pitche', 'pitche-moi ça', 'présente mon projet en 30 secondes'. NOT for landing pages or long-form marketing copy (that is the marketing suite, e.g. /omg-ad-creative) — /pitch is the spoken 30-second distillation."
---

# Pitch — 30-Second Pitch Generator

Thirty seconds is what you actually get — the elevator, the demo-day mic, the "so what do you do?" at dinner. Pitch compresses whatever exists — a one-line idea, a repo, a landing page, this conversation — into 75-85 words that survive being said out loud: one hook, one problem, one mechanism, one proof, one ask — plus a one-line version for when you get five seconds instead. It is honest by construction: weak material yields a flagged pitch and a note on what to go get, never a polished lie (L2 — researcher, not sycophant). Not for landing pages or long-form marketing copy — that is the marketing suite (/omg-ad-creative); this is the spoken distillation.

---

## Invocation

```
/pitch <idea text | repo path | URL>                        # pitch that
/pitch                                                      # pitch what this conversation is about
/pitch ... --audience [investor|client|partner|recruiter]   # tune proof + ask (protocol step 4)
/omg-pitch ...                                              # same skill, OmegaOS-prefixed
```

No argument and nothing pitchable in context → ask one line ("Pitch what?"). That is the ONLY question this skill ever asks — given any material at all: extract, infer, tag, deliver, zero clarifying questions.

---

## Protocol

### 1. Extract the five raw materials

From whatever the user gave — idea text, a repo (read the README and the core code), a page (fetch and read it), or this conversation — extract:

- **WHO** it is for — one named buyer or user. "Everyone" is not an answer; pick the person who pays.
- **PAIN** — the problem as the WHO feels it, in their words, costed when the source allows.
- **MECHANISM** — how the solution works: ONE mechanism, not a category. "Watches your Stripe webhooks and retries failed charges" beats "an AI billing platform".
- **PROOF** — the single strongest TRUE item the source actually shows: a number, traction, a credential, a verifiably working demo. One. PROOF is never inferred — `[ASSUMED]` is banned outright on numbers, users, revenue, and logos; nothing verifiable in the source → PROOF is `none yet` (the Output Format shows the form).
- **ASK** — what you want from the listener: money, a meeting, a signup. Specific or it does not count. An ask stated in the source is used as-is — it is sourced material, never tagged `[ASSUMED]`. Only when no ask is derivable from the source and no `--audience` is given → default to the demo-day ask (step 4); never ask the user to choose.

Missing WHO, PAIN, MECHANISM, or ASK → infer the strongest HONEST candidate and list the inference solely under ASSUMPTIONS, keyed by material name — never inside beat text. An assumed mechanism is inference; a fabricated metric is a lie.

### 2. Draft to the word budget

75-85 words total — about 30 seconds spoken. The 75-85 total is BINDING; per-beat budgets are targets inside it:

| Beat | Words | Job |
|------|-------|-----|
| Hook | 10-12 | A tension or a number — never "Hi, I am..." |
| Problem | 16-22 | The pain, inside the WHO's world |
| Solution | 22-28 | The one mechanism |
| Proof | 12-15 | One number beats three adjectives |
| Ask | 8-12 | Specific and dated |

PROOF `none yet` → the beat counts zero words; its 12-15 go to PROBLEM and SOLUTION. Count the words. Over 85: cut adjectives first, then whole clauses. Under 75: PROBLEM or PROOF is underfed — expand the costed pain or the proof specifics; never pad with adjectives. Past 90 the draft has failed — recut, do not rationalize.

### 3. Spoken test

Rewrite until the draft survives being read aloud:

- One idea per sentence — a sentence that needs a breath in the middle gets split.
- Kill any word that trips the tongue; if you stumble on it, the speaker will too.
- Zero jargon a stranger would not know — "idempotent retry orchestration" dies here.

### 4. Audience tuning

If the user names an audience, retune PROOF and ASK to it — the other beats barely move:

- **investor** — proof: the growth or market number; ask: the raise and what it buys, dated.
- **client** — proof: the outcome for someone like them; ask: a pilot or demo, dated.
- **partner** — proof: the asset you bring them; ask: the specific integration or intro.
- **recruiter** — proof: the shipped result; ask: the role or the conversation you want.

No audience named → deliver immediately. If the source states an ask, keep it as-is — it is sourced, appears nowhere under ASSUMPTIONS, and the ASSUMPTIONS terminator ("none, all five materials came from the source") stays reachable on a bare `/pitch` run. Only when no ask is derivable from the source does the demo-day default fire: the ASK beat becomes a specific, dated meeting or demo ("give me 15 minutes Thursday to show you the demo"), listed as `[ASSUMED] ASK` under ASSUMPTIONS with the standing retune line shown in the Output Format. Never ask which audience first.

### 5. Deliver

Emit the Output Format block below: the labeled 30s pitch with total word count and approximate seconds, the one-liner ("X helps Y do Z without W" — or sharper when the material allows), delivery notes (where to pause, the one number or phrase to land, plus the mandatory go-get when PROOF holds no true number), and every inference under ASSUMPTIONS. If the material is weak — no real proof, fuzzy WHO — say so next to the pitch and name what to go get. The pitch is only as strong as its weakest material; flattering the user about it helps no one (L2).

---

## Output Format

Emit exactly this block. Beat text is clean speakable words only — `[ASSUMED]` never appears inside a beat or the one-liner; every inference is listed solely under ASSUMPTIONS, keyed by material name (WHO/PAIN/MECHANISM/PROOF/ASK).

```
PITCH (30s) · [total] words · ~[n]s spoken
──────────────────────────────────────────
HOOK     ([n]w)  [text]
PROBLEM  ([n]w)  [text]
SOLUTION ([n]w)  [text]
PROOF    ([n]w)  [text]            <- no true proof in the source: this line reads exactly "PROOF (0w) none yet"
ASK      ([n]w)  [text]

ONE-LINER
[X helps Y do Z without W — or sharper]

DELIVERY NOTES
- Pause: [where, and why it works there]
- Land: [the one number or phrase to hit hardest]
- Go get: [the one specific number that would fill the PROOF beat]   <- mandatory whenever PROOF holds no true number; omit otherwise

ASSUMPTIONS
- [ASSUMED] [material]: [what was inferred, and from what]
- [ASSUMED] ASK: demo-day default (rerun with --audience investor|client|partner|recruiter to retune PROOF+ASK)   <- mandatory whenever the ask was defaulted (no ask in the source and no --audience)
(or: none, all five materials came from the source)
```

Proof-present and proof-absent runs emit the same structure. The PROOF-absent form changes exactly two things — the PROOF line reads `none yet` (words reallocated per step 2) and the `Go get:` note becomes mandatory. It never fills the beat with an assumed number, never drops the beat, never shrinks the block.

---

## Rules & Anti-patterns

**Rules**

- Honesty over flattery (L2). No real number yet → PROOF carries the strongest verifiable status the source actually shows ("the demo works end-to-end", "a waitlist of 40") or reads exactly `none yet` — and the `Go get:` note names the number to earn. Never a polished stand-in.
- `[ASSUMED]` marks inference on WHO/PAIN/MECHANISM/ASK only, and only under ASSUMPTIONS. Numbers, users, revenue, logos: real or absent — never assumed.
- One mechanism. If it cannot be said in one clause, the pitch is not ready — tell the user that instead of shipping mush.
- The word budget is hard. Negotiating it is answering a different question.
- R-NODASH: emitted pitch text (beats, one-liner, notes) uses human punctuation only: comma, period, colon, parentheses. Strip every em or en dash before delivering.

**Anti-patterns — each forces a rewrite**

- Buzzword salad: "revolutionary AI-powered platform" says nothing. Name the mechanism.
- Listing 3 features instead of one mechanism. A list is a catalog, not a pitch.
- Invented or unverifiable numbers. One fake metric poisons every true one.
- Exceeding 90 words. That is a memo, not a pitch.
- Burying or omitting the ask. A pitch without an ask is an anecdote; the ask is the point.

---

## Pairs With

- **/omg-social-content** — hand the finished pitch over: the same five materials become organic posts, threads, and hooks.
- **/omg-cold-email** — the pitch's WHO, PAIN, and ASK are the spine of an outbound sequence; reuse them verbatim.
- **/10x** — when the pitch feels small, the problem is upstream: reset the ambition first, then pitch the 10x version.
- **/ghost** — rehearse it: have the pitch's exact WHO react to the finished 30 seconds before a real one does.

---
name: llm-council
description: >
  Convene a multi-model Claude COUNCIL for a high-stakes, ambiguous, or irreversible decision —
  or whenever you want a genuine second opinion. Four different Claude models (Opus 5, Sonnet 4.6,
  Haiku 4.5, Fable 5) answer the SAME question independently and in parallel, then peer-review each
  other ANONYMOUSLY (answers shown as Response A/B/C, model identity hidden, so no model is favored
  by name), and an Opus 5 president synthesizes the final verdict WHILE surfacing the dissent.
  Inspired by Karpathy's LLM Council, re-implemented on the OmegaOS Workflow primitive. 100%
  Claude Code-native — NO API KEYS, no OpenRouter, no external provider, no extra cost: the members
  run as in-process Workflow sub-agents through your EXISTING Claude Code session. Use when the user
  says "llm council", "convene the council", "ask the council", "@council", "/llm-council",
  "second opinion", "deliberate", "multiple models", or faces a "high-stakes decision" /
  architecture choice / contradictory verification verdicts / a pre-commit design call.
argument-hint: "<question or decision to deliberate>"
allowed-tools: ["Workflow", "Read", "Write", "Bash"]
domain: orchestration
read_only: true
triggers: ["llm council", "convene the council", "ask the council", "@council", "/llm-council", "llm-council", "second opinion", "deliberate", "multiple models", "high-stakes decision"]
---

# /llm-council — Multi-Model Claude Deliberation Council

Convene a **council of Claude models** on one hard question. Four different Claude models answer
**independently**, **anonymously peer-review** each other, and an **Opus president** writes the final
verdict that explicitly preserves the dissent. The whole council runs **in-process** through the
**OmegaOS Workflow primitive** — it is the heavyweight "many minds on one decision" tool.

> Inspired by Andrej Karpathy's *LLM Council* (multiple frontier models answer, rank each other
> blind, a chairman synthesizes). The difference here is the runtime, below.

## 100% Claude Code-native — NO API keys

**This council asks for zero API keys and costs nothing beyond your current Claude Code session.**

| | Karpathy's LLM Council | OmegaOS `llm-council` |
|---|---|---|
| Members | GPT / Gemini / Grok / Claude via **OpenRouter** | **Four Claude models** (Opus 5, Sonnet 4.6, Haiku 4.5, Fable 5) |
| Auth | An **OpenRouter API key** + per-provider keys/billing | **None** — your existing logged-in Claude Code session |
| Runtime | A Python web app calling external HTTP APIs | **In-process Workflow `agent()` sub-agents** |
| Extra cost | Per-token charges on each external provider | **None beyond the base Claude Code token/subscription** |

The members are spawned with the **Workflow tool's `agent(prompt, { model })`** — each runs on the
operator's already-authenticated Claude Code session (their base token / subscription). There is **no
`ANTHROPIC_API_KEY`, no `OPENROUTER_API_KEY`, no external endpoint, and no extra billing**. If you
ever feel tempted to reach for an API key or an `anthropic` SDK call here — **don't**. The Workflow
tool is the only mechanism this skill uses to fan out to the four models.

## When to convene (and when NOT to)

The council spends roughly **4× the tokens** of a single answer (four members + a peer-review round +
a president). Reserve it for decisions where that buys real safety — aligned with **R-COUNCIL** and
the OmegaOS quality laws:

**Convene when:**
- The decision is **high-stakes / irreversible / hard to undo** — an architecture choice, a schema
  migration, a "drop the prod table?" call, monorepo-vs-polyrepo.
- Two adversarial verdicts **contradict** each other (a verify says PASS, an audit says FAIL) and you
  need a tie-break with the dissent preserved.
- A **pre-commit design** judgment where being wrong is expensive.
- The user **explicitly** invokes it: `@council`, `/llm-council`, "convene the council", "I want a
  second opinion from multiple models", "deliberate this".

**Do NOT convene for:** routine edits, a one-line fix, a lookup, anything a single model answers
cheaply, or unattended bulk work. A council on a trivial task is just 4× the cost for no gain.

> Invoked by /llm-council, /council, /omg-llm-council, /omg-council, or @council — they all convene THIS one multi-model council.

## The protocol — three stages

### Stage 1 — Members answer independently (parallel)

Four Claude models receive the **same** question and answer it **independently**, with no knowledge
of each other's responses. Each is a Workflow `agent()` pinned to a specific model:

| Seat | Model id | Why it's on the council |
|---|---|---|
| Opus 5 | `opus` | Deepest reasoning, the frontier seat |
| Sonnet 4.6 | `claude-sonnet-4-6` | Strong generalist, different training balance |
| Haiku 4.5 | `claude-haiku-4-5` | Fast, terse — a useful contrarian on over-thinking |
| Fable 5 | `claude-fable-5` | The most powerful tier — independent high-capability lens |

Running the **same prompt across different models** is the whole point: their disagreements expose the
real uncertainty in the decision. A single model's confident answer hides it.

### Stage 2 — Anonymized peer-review (parallel)

Each member is shown the **other three** answers — **stripped of model identity** and relabelled
**Response A / B / C** — and asked to rank and critique them on **accuracy, insight, and
completeness**. Anonymity is the load-bearing detail: a model must not be able to favor "the Opus
answer" or dismiss "the Haiku answer" by name. It judges the *content*, blind.

Each reviewer returns, for the three anonymized peers: a **ranking** (best → worst) with a one-line
justification each, the **strongest single point** any peer made, and the **most important flaw or
omission** across them.

### Stage 3 — The President synthesizes (Opus 5)

A final **Opus 5** president receives **all four original answers** plus **all four peer-reviews**
and writes the verdict. The president does NOT just pick a winner — it **reconciles** the council into
one answer and **explicitly records where the council disagreed**. Erasing the minority view is
forbidden (this mirrors R-COUNCIL / R-VERIFY: dissent is signal, not noise).

President output is exactly:

```
## Verdict
<the synthesized recommendation / answer>

## Why
<the reasoning the council converged on, with the strongest peer-reviewed points>

## Where the council disagreed
<the dissent / minority views, preserved verbatim in substance — what a member argued and why
 it did not carry, or why it remains an open risk>

## Confidence
<Low | Medium | High> — <one line on what would raise it>
```

## Runnable Workflow script (copy-pasteable)

Run the council with the **Workflow tool**, using this script. It implements
members → anonymized peer-review → president. It is **deterministic**: member prompts vary by a fixed
role/index, anonymization uses a **static** A/B/C labelling (no shuffle), and it uses **no**
non-deterministic built-ins (no `Date.now()`, no `Math.random()` — the runtime forbids them).

Replace `QUESTION` with the user's actual decision (or read it from `$ARGUMENTS`).

```javascript
// llm-council — multi-model Claude deliberation on ONE question.
// 100% Claude Code-native: members are in-process Workflow agents on the
// existing session. No API keys, no external providers.

meta({
  title: "LLM Council",
  description: "Four Claude models answer, blind-peer-review, Opus president synthesizes with dissent.",
});

const QUESTION = `<<<PASTE THE USER'S QUESTION OR DECISION HERE>>>`;

// The four council seats. Order is fixed → the run is deterministic.
const SEATS = [
  { name: "Opus 5",     model: "opus" },
  { name: "Sonnet 4.6", model: "claude-sonnet-4-6" },
  { name: "Haiku 4.5",  model: "claude-haiku-4-5" },
  { name: "Fable 5",    model: "claude-fable-5" },
];

// ---- Stage 1: members answer INDEPENDENTLY, in parallel -------------------
const memberPrompt = (seatIndex) => `You are council member #${seatIndex + 1} of a multi-model deliberation.
Answer the question below INDEPENDENTLY and on the merits. Do not hedge to a committee average —
give your own best, well-reasoned answer, state your key assumptions explicitly, and name the main
tradeoff or risk you see. Be concrete. You are NOT told what the other members said.

QUESTION:
${QUESTION}`;

const answers = await parallel(
  SEATS.map((seat, i) =>
    agent(memberPrompt(i), { model: seat.model })
  )
);
// answers[i] is seat i's independent answer (string).

// ---- Stage 2: ANONYMIZED peer-review, in parallel -------------------------
// Each reviewer sees the OTHER three answers, relabelled Response A/B/C with
// model identity hidden. Static labelling (no shuffle) keeps the run reproducible.
const LABELS = ["A", "B", "C"];

const reviewPrompt = (reviewerIndex) => {
  const others = answers
    .map((text, i) => ({ text, i }))
    .filter((o) => o.i !== reviewerIndex);   // the other three seats
  const anonymized = others
    .map((o, k) => `### Response ${LABELS[k]}\n${o.text}`)
    .join("\n\n");
  return `You are peer-reviewing three ANONYMOUS answers to the same question. The author models are
hidden on purpose — judge ONLY the content, not any guessed identity. Rank Response A, B, and C from
best to worst on accuracy, insight, and completeness, with a one-line justification for each. Then
name the single strongest point made across all three, and the single most important flaw or omission.

QUESTION:
${QUESTION}

ANSWERS TO REVIEW (anonymized):
${anonymized}`;
};

const reviews = await parallel(
  SEATS.map((seat, i) =>
    agent(reviewPrompt(i), { model: seat.model })
  )
);
// reviews[i] is seat i's blind ranking + critique of the other three (string).

// ---- Stage 3: the PRESIDENT synthesizes (Opus 5) ------------------------
const dossier = SEATS.map((seat, i) =>
  `## Member ${i + 1} — ${seat.name}\n### Answer\n${answers[i]}\n### This member's peer-review of the others\n${reviews[i]}`
).join("\n\n");

const presidentPrompt = `You are the PRESIDENT of an LLM council (model: Opus 5). Four members each
answered the question independently, then blind-peer-reviewed each other. Synthesize the FINAL answer.
Do not merely pick a winner — reconcile the council into one coherent recommendation, give the
reasoning it converged on (citing the strongest peer-reviewed points), and EXPLICITLY preserve the
dissent: where members disagreed, what the minority argued, and why it did or did not carry. Never
erase a minority view.

Use EXACTLY these sections:
## Verdict
## Why
## Where the council disagreed
## Confidence

QUESTION:
${QUESTION}

COUNCIL DOSSIER (all four answers + all four peer-reviews):
${dossier}`;

const verdict = await agent(presidentPrompt, { model: "opus" });

return verdict;
```

**Notes for the running agent**
- `agent(prompt, { model })` and `parallel([...])` are Workflow primitives — they spawn in-process
  sub-agents on the **current session**. No key, no SDK, no network call to an external provider.
- A Workflow sub-agent **cannot nest another Workflow** — so the council is run **once, at the top
  level** by whoever invokes `/llm-council`. Don't try to launch a council from inside a council member.
- Keep the script deterministic: never add `Date.now()`, `Math.random()`, or shuffle the labels.
  If you want a different seat order, edit the static `SEATS` array.
- After the Workflow returns, present the president's verdict to the user as-is (it already carries
  the dissent). You may add a one-line French recap per R-STYLE if the project is French-only.

## Worked example

**Decision:** *"Should the three client apps share one monorepo, or stay as three separate repos?"*

**Stage 1 — independent answers (sketch):**
- *Opus 5* → Monorepo: atomic cross-app refactors, one CI, shared design system; flags tooling cost.
- *Sonnet 4.6* → Polyrepo: independent deploy cadences and blast-radius isolation; flags drift.
- *Haiku 4.5* → "Depends on team size; under ~6 devs, monorepo wins on overhead." (terse contrarian)
- *Fable 5* → Monorepo with enforced package boundaries; warns the real risk is CI scaling, not layout.

**Stage 2 — anonymized peer-review (sketch):** Each member ranks the other three as Response A/B/C
**without knowing which model wrote which**. Several independently rank the boundary-enforced-monorepo
answer top on completeness, and flag the bare polyrepo answer as under-justified on the cross-app
refactor cost — a blind convergence, not a name-following one.

**Stage 3 — president verdict (sketch):**
```
## Verdict
Adopt a monorepo with enforced package boundaries and per-app deploy pipelines.
## Why
Three of four members, ranked blind, favored the monorepo for atomic cross-app changes and a shared
design system; the boundary-enforcement answer was the most complete and addressed the strongest
counter-point (deploy isolation) directly.
## Where the council disagreed
One member argued for polyrepo on deploy-isolation grounds; the council judged that recoverable via
per-app pipelines inside the monorepo, but it remains the real risk if CI ownership is weak — revisit
if the team grows past ~15 devs or CI times balloon.
## Confidence
Medium — would rise to High given the team size and current CI duration.
```

This is the value: the dissent ("polyrepo for isolation") is **preserved as a tracked risk**, not
silently dropped — and the convergence happened **blind**, so it isn't an artifact of model prestige.

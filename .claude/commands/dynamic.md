---
description: Run this task with Claude Code's native Dynamic Workflows — draw the graph, fan out in-process, verify on the edge, then synthesize. The OmegaOS trigger for the Workflow tool (the successor to the old /team multi-subagent pattern). An oracle puts `/dynamic` on line 1 of a worker prompt to make that worker use Dynamic Workflows.
argument-hint: <task to run as a dynamic workflow>
---

# /dynamic — Dynamic Workflows (graph engineering)

You are explicitly authorized to use Claude Code's native **Dynamic Workflows** (the `Workflow`
tool) for this task. **This `/dynamic` invocation IS the opt-in** — go ahead and use it.

This command is the executable half of Rule **R-GRAPH**. R-ORCH says *when* to dispatch;
this says what **shape** the dispatch takes. The shape is the single biggest lever you have on
wall-clock time and on cost.

---

## Step 0 — Draw the graph before you write the script

A mission is a **graph**, never a line.

- A **node** is one agent, one bounded job, one input in, one output out.
- An **edge** exists **only where data actually moves**.

Apply the **"and then" test** to every arrow you are about to draw: *does the next step READ
the previous step's output?* If not, there is no edge — the wait is pure latency. Cut it, and
the chain collapses into something wider: independent nodes that all run at once.

Then name the topology out loud before coding it. Almost every real mission is the **diamond**:

```
fan out (breadth)  →  reduce (plain code)  →  synthesize (one agent)
```

An audit, a code review, a research report, a market scan, a multi-file migration are all that
same skeleton with different prompts. Stop asking "how do I make the agent do more steps" and
ask **"where is the split, where is the merge"**.

---

## Step 1 — `pipeline()` by default. A barrier must be argued for.

`pipeline()` streams each item through every stage independently — item A can be in stage 3
while item B is still in stage 1. `parallel()` is a **barrier**: every item waits for the
slowest one before the next stage begins.

**Default to `pipeline()`.** There are exactly three legal reasons to reach for a barrier:

1. a **cross-set dedupe** before expensive downstream work,
2. an **early exit on the total** (zero findings → skip the whole verify stage),
3. a prompt that **compares an item against "the other findings"**.

"It's cleaner code" and "the stages feel separate" are not reasons. Separate is not the same as
synchronized. The smell test is brutal:

```js
const a = await parallel(...)      // ← if this middle transform has no
const b = a.flat().filter(Boolean) //   cross-item dependency, you did not
const c = await parallel(b.map(...))//  need the barrier. Use a pipeline.
```

The canonical form — each dimension verifies the moment its own review lands, no idling:

```js
const results = await pipeline(
  DIMENSIONS,
  d      => agent(d.prompt, {label: `review:${d.key}`, phase: 'Review', schema: FINDINGS}),
  review => parallel(review.findings.map(f => () =>
              agent(`Adversarially verify: ${f.title}`, {phase: 'Verify', schema: VERDICT})
                .then(v => ({...f, verdict: v})))),
)
const confirmed = results.flat().filter(Boolean).filter(f => f.verdict?.isReal)
```

---

## Step 2 — Every node gets a contract

Input passed **explicitly** (never assumed from a shared window), output **validated**:

```js
const FINDINGS = {type:'object', properties:{findings:{type:'array', items:{...}}}, required:['findings']}
const r = await agent(prompt, {schema: FINDINGS})   // returns a validated object, not text
```

Put a `schema:` on **every** `agent()` whose output another node consumes. Validation happens at
the tool-call layer, so the model retries on a mismatch instead of handing you free text you
parse and pray over. A node whose output only a human can read is a node you cannot wire into a
graph.

---

## Step 3 — Edges are free. Never burn an agent on one.

The reduce between fan-out and synthesis — flatten, dedupe, filter, rank, sort, count — is
**plain JavaScript in the script**. Coordination costs **zero model tokens** because it is code,
not a conversation. A large share of what missions waste tokens on is really an edge.

Do it inside a pipeline stage rather than paying for a barrier just to transform:

```js
pipeline(items, stageA, r => dedupe([r]).flat(), stageB)
```

---

## Step 4 — Contain failure per node

A thunk that throws resolves to `null` instead of sinking the batch. So:

- `.filter(Boolean)` **before consuming any fan-out result**;
- design every fan-in to **tolerate missing inputs** rather than assume a full set.

`isolation: 'worktree'` is the seatbelt for the **one** topology that needs it — nodes that
**write files in parallel** (R-SCOPE). It costs setup time and disk per agent. Never a default
tax on every run.

---

## Step 5 — Route on the edge, in code

A router node's classification may be Claude-powered, but the branch itself is a plain `if` /
`switch` in the script:

```js
const cls = await agent(`Classify this diff: trivial | risky`, {schema: CLASS})
const findings = cls.level === 'risky'
  ? await parallel(LENSES.map(l => () => agent(auditPrompt(l), {schema: FINDINGS})))
  : [await agent(quickPassPrompt, {schema: FINDINGS})]
```

You get Claude's judgment at the node and the script's determinism at the edge. The same
classification always takes the same path — no emergent "it decided to skip the audit".

---

## Step 6 — Verify on the edge, before a finding is allowed downstream

Three patterns worth having in hand (R-VERIFY):

- **Adversarial** — N independent skeptics each prompted to **refute**; keep only what a
  majority fails to kill. Default to `refuted: true` when a verifier is uncertain.
- **Perspective-diverse** — each verifier gets a **distinct lens** (correctness, security,
  does-it-reproduce). Diversity catches failure modes N identical checks never will.
- **Judge panel** — N attempts from different angles, parallel scorers, synthesize from the
  winner while grafting the best of the runners-up.

---

## Step 7 — Cycles must converge

For discovery of unknown size, loop **until dry**: stop after K consecutive rounds that surface
nothing new.

```js
const seen = new Set(), confirmed = []
let dry = 0
while (dry < 2) {
  const fresh = (await findRound()).filter(b => !seen.has(key(b)))
  if (!fresh.length) { dry++; continue }
  dry = 0; fresh.forEach(b => seen.add(key(b)))          // ← dedupe against SEEN
  confirmed.push(...await verify(fresh))                 //   never against `confirmed`
}
```

**The one mistake almost everyone makes:** deduping against `confirmed` instead of `seen`. Then
every judge-rejected finding reappears next round, the loop never runs dry, and you have built a
machine that pays to rediscover the same dead ends until the budget dies.

R-LOOP's retry ceiling and R-BUDGET's cap still bind. Any budget-driven loop must guard on
`budget.total &&` — with no target set, `budget.remaining()` is `Infinity`.

---

## Step 8 — Tier the models per node, not per run

Every subagent **inherits the session model** unless the script overrides it, so a large fan-out
bills entirely at the session tier. Keep judgment nodes (synthesis, adjudication, final verify)
high; push bounded repetitive nodes (extract, classify, label, format) down with
`{model: 'haiku'}` or `{effort: 'low'}`. Per node. See **R-MODEL** for the tier map, including
the Fable security disqualification.

---

## Step 9 — Save the graph

When a run's topology worked, it is an **asset**, not a one-off. Save the script to
`.claude/workflows/<name>.js` — committed, re-runnable by name, launchable by anyone who clones
the repo — instead of re-deriving the same shape next time.

OmegaOS ships its own under `.claude/workflows/` (see the README there); run one with
`Workflow({name: "<name>", args: …})`, or `Workflow({scriptPath: ".claude/workflows/<name>.js"})`
when the name is not resolving.

---

## When NOT to fan out

- Trivial single-step task → just do it. A workflow is overkill.
- Fewer than 3 sub-tasks, or sub-tasks that share files → serialize (R-SCOPE).
- Long isolated file mutation → that is a worker (`omega spawn-worker`), not in-process fan-out.

## Close the loop

Synthesize the sub-agent outputs into the answer **yourself** — never paste a delegate's summary
as the verdict (R-ORCH). Report the verified result **and what was actually checked**; if the
graph bounded its own coverage (top-N, no-retry, sampling), `log()` what was dropped, because
silent truncation reads as "covered everything" when it did not. Verify against runtime before
reporting (**L1**, **L4**).

---

Task to run as a dynamic workflow:

$ARGUMENTS

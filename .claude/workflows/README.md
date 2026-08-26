# `.claude/workflows/` — saved graphs

A workflow script is a **graph you already got right**, kept as an asset instead of re-derived
every run. Claude Code resolves scripts in this directory by name:

```js
Workflow({ name: "adversarial-review" })
Workflow({ name: "diamond", args: { units: [...], task: "...", question: "..." } })
Workflow({ scriptPath: ".claude/workflows/diamond.js", args: {...} })   // if the name does not resolve
```

Doctrine: Rule **R-GRAPH** (the shape) and the `/dynamic` command (the executable playbook).
R-ORCH says *when* to dispatch; these say *what shape* the dispatch takes.

## What ships here

| Workflow | Topology | Use it for |
|---|---|---|
| `adversarial-review` | router → parallel lenses → N skeptics per finding → synthesize | Reviewing a diff, branch or file set so the findings survive being attacked |
| `discovery-sweep` | multi-modal finders → seen-dedupe → verify → **loop until dry** | Finding everything of a kind when you do not know how many exist |
| `diamond` | fan out → reduce (code) → synthesize | Any breadth sweep with a known work list, parameterised through `args` |

`diamond` is the generic one. Most "audit each X" missions are it with a different `task` and
`question`, so reach for it before writing a new script.

## Invariants every script here holds

1. **`pipeline()` by default.** A `parallel()` barrier appears only where a stage genuinely needs
   every prior result at once. Barrier latency is real, measurable, wasted time.
2. **`schema:` on every consumed node.** Validation at the tool-call layer, so a mismatch is a
   retry rather than free text you parse and pray over.
3. **The reduce is code.** Flatten, dedupe, filter, rank, sort, count — plain JavaScript, zero
   model tokens. Never spend an agent on an edge.
4. **`.filter(Boolean)` before consuming any fan-out.** A thunk that throws resolves to `null`;
   the fan-in must tolerate missing inputs instead of assuming a full set.
5. **Verifiers refute.** Skeptics are prompted to kill a finding, and uncertainty counts as
   refuted. Majority survival is the bar (R-VERIFY).
6. **Cycles dedupe against everything SEEN**, never against confirmed results — otherwise every
   rejected finding returns next round and the loop never runs dry.
7. **No silent caps.** When a script bounds its own coverage (round ceiling, missing unit, dropped
   claim) it `log()`s it and says so in the returned object, because silent truncation reads as
   "covered everything" when it did not.
8. **No `Date.now()` / `Math.random()` / argless `new Date()`** — they throw in workflow scripts
   (they would break resume). Stamp times after the run; vary randomness by index.

## Adding one

Write the script, keep the invariants above, then check it parses:

```bash
./scripts/check-workflows.sh
```

That is a syntax gate, not a proof of behaviour — a new workflow is only done once you have
actually run it and read what it returned (L1).

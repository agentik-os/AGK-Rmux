---
name: cookbook
description: >
  Anthropic's OWN reference implementations, on tap. 94 recipes from
  anthropics/claude-cookbooks (MIT, pinned) covering RAG and retrieval, agent
  patterns (prompt chaining, routing, parallelization, orchestrator-workers,
  evaluator-optimizer), evals and LLM-judge harnesses, tool use and structured
  output, prompt caching and cost optimization, multimodal and PDF extraction,
  citations, extended thinking, the Claude Agent SDK, and Claude Managed Agents.
  Use BEFORE hand-rolling any of those: read the reference, adapt the code, do
  not re-derive it from memory. Triggers on "/cookbook", "/omg-cookbook",
  "anthropic cookbook", "claude cookbook", "reference implementation", "how does
  Anthropic do X", "is there a recipe for", or in French "recette anthropic",
  "implementation de reference", "comment Anthropic fait X". NOT for Claude Code
  harness questions (that is claude-code-guide) and NOT for the Claude API
  reference itself (that is the claude-api skill): this is worked EXAMPLES.
---

# Cookbook — Anthropic's reference recipes

You are about to build something Anthropic already published a reference
implementation for. Read theirs first.

This skill routes into **94 recipes** from `anthropics/claude-cookbooks` (MIT),
pinned to a recorded commit in `tools/cookbooks/COOKBOOKS.lock`. The index ships
with OmegaOS, so search always works. The notebooks themselves are an optional
local corpus.

## The rule this exists to enforce

R-SKILL-ATLAS says: find the real thing and run it, never answer generically.
This skill is that rule applied to Claude-API engineering. When a mission needs
a RAG pipeline, an eval harness, a citations layer, an orchestrator-workers
fan-out, contextual embeddings, prompt caching, a moderation filter or a
cost-optimization pass, **there is a reference implementation** and re-deriving
it from memory is the failure mode, not the work.

## 1. Find the recipe

```bash
omega-skills --rag "<the need, in plain language>"
```

Cookbook rows are labelled by where they live, and the label is honest:

| Label | Meaning |
|---|---|
| `(cookbook)` | the notebook is on disk — read it |
| `(cookbook ↗)` | index only — open the pinned upstream URL |

For the machine-readable form (carries `local` and `url`):

```bash
omega-skills --rag "<need>" --json
```

Browse everything by category in the Skill Atlas — the **Anthropic cookbooks**
tab of `~/.omega/artifacts/omega-skill-atlas.html`.

## 2. Read it

**Corpus present** (`~/.omega/cookbooks/`): read the notebook directly. It is
JSON — pull the source cells rather than paging the whole file:

```bash
python3 -c "
import json,sys
nb=json.load(open(sys.argv[1]))
for i,c in enumerate(nb['cells']):
    src=''.join(c['source'])
    if src.strip(): print(f'--- [{i}] {c[\"cell_type\"]} ---\n{src}\n')
" ~/.omega/cookbooks/<path>.ipynb
```

**Corpus absent:** `WebFetch` the `url` from the `--json` output. It is pinned to
the recorded commit, so it matches the indexed description exactly.

Install the corpus for offline work:

```bash
tools/cookbooks/install-cookbooks.sh          # ~99M, ~3s, images excluded
tools/cookbooks/install-cookbooks.sh --status
```

## 3. Adapt it — do not paste it

The recipes are **Python against the Claude API with an `ANTHROPIC_API_KEY`**.
OmegaOS is Rust-first and runs on the Claude Code subscription (R-STACK). So:

- Take the **method** — the chunking strategy, the judge rubric, the retry
  shape, the cache-breakpoint placement, the eval design. That is the value.
- Port it to the OmegaOS stack. A recipe is a reference, never a dependency.
- If you genuinely need the Python path, it is a documented exception (R-STACK)
  and the key lives in `~/.omega/secrets/`, never the repo (R-ENV).
- Model IDs in the notebooks age. Check them against the `claude-api` skill
  before running anything.

## 4. Cite what you used

R-CITE: name the recipe and its path in your report, so the next agent can
follow the same trail instead of re-deriving it.

> Ported the contextual-retrieval chunking from
> `capabilities/contextual-embeddings/guide.ipynb` (cookbook @ 35f2eec).

## Where each need lands

| Need | Category to search |
|---|---|
| RAG, chunking, reranking, contextual embeddings | RAG & Retrieval (9) |
| Agent topology, orchestrator-workers, evaluator-optimizer | Agent Patterns (8) |
| Eval harness, LLM judge, synthetic test cases | Evals (4) |
| Tool definitions, structured output, JSON mode | Tools (11) |
| Vision, charts, PDF extraction, transcription | Multimodal (6) |
| Streaming, citations, prompt caching, batching | Responses (11) |
| Server-hosted agents, sandboxes, budget caps, HITL gates | Claude Managed Agents (16) |
| Building or hosting a custom agent | Claude Agent SDK (9) |
| Pinecone, MongoDB, Voyage, Wikipedia, Slack | Integrations (13) |
| Extended thinking, reasoning budgets | Thinking (2) |
| SKILL.md authoring, progressive disclosure | Skills (3) |
| Tracing, usage and cost APIs | Observability (1) |
| Fine-tuning and dataset prep | Fine-Tuning (1) |

## Keeping the pin current

```bash
tools/cookbooks/install-cookbooks.sh --update    # move the pin to upstream main
tools/cookbooks/install-cookbooks.sh             # fetch it
tools/cookbooks/build-index.py ~/.omega/cookbooks -o tools/cookbooks/recipes.json
python3 scripts/omega-skills-atlas.py && python3 scripts/omega-skills-rag.py build
```

Commit `COOKBOOKS.lock` and `recipes.json` together — the pin and the index are
one artifact, and a drift between them means the RAG describes a recipe that the
local notebook no longer matches (L0).

## Also vendored from this upstream

Three recipes were substantial enough to become real OmegaOS skills rather than
references. They are installed and reachable by name:

- `analyzing-financial-statements` — ratio analysis over real statements
- `creating-financial-models` — DCF and sensitivity analysis
- `applying-brand-guidelines` — programmatic brand compliance on generated docs

And `docs/AGENT-PATTERNS.md` carries the five canonical agent workflow patterns
plus Anthropic's production research-lead / subagent / citations prompts, mapped
onto R-GRAPH and R-ORCH.

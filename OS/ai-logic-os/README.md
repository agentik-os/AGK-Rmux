# AI Logic OS

AgentikOS operative system — Systems group — **integrated** (v1.0).

The workflow-optimization AND agentic-system-challenge OS. It audits a process,
decides what to automate, arbitrates deterministic code vs AI judgment, and
specs an automation before it is built — and, the improvement, it CHALLENGES an
agentic system (OmegaOS itself, an agent, a skill, an LLM pipeline, a coding
tool, an AI use case): where a model does an `if`'s job, where a consequential
output is unverifiable, where an irreversible action lacks a human gate, where
the feedback loop is missing, and what primitive is absent and should exist.
Its default bias is NO. Built from the operator's `optimiseur-workflow-ia`
skill (Deposit, 2026-08-11), extended with the system-challenger layer.

## Layout

| Path | What |
|---|---|
| `pack/` | SKILL.md + `references/workflow-optimizer.md` (the operator's core doctrine, verbatim) + `references/system-challenger.md` (the agentic-system challenge extension: 5 questions, agent triage, staying current, auditing OmegaOS against its own Laws) |
| `MASTER.md` | The master agent (TUI Enter + Telegram bot brain) |
| `bin/omega-ailogic` | The OmegaOS command — opens the AI Logic master agent (conversational + diagnostic; no state CLI) |
| `commands/codex-ai-logic-os.md` | The OpenAI/Codex command (installed to `~/.codex/prompts/ai-logic-os.md`) |

The Claude command is the `ai-logic-os` skill (at `skills/ai-logic-os/`),
installed as `/ai-logic-os`, `/omg-ai-logic-os` and the `/ai-logic` + `/ailogic`
aliases.

## Use it

`/ai-logic` (or `/ailogic`) in Claude, the Codex prompt, `omega-ailogic` in a
terminal, or the OS master agent (TUI OS tab -> Enter, Telegram bot via `T`).

Two jobs:
- **Optimize a workflow** — map → instrument → triage (Codifier / Augmenter /
  Garder humain / Supprimer) → design → spec → measure → loop. Deletions first;
  spec only the first move.
- **Challenge an agentic system** — the 5 questions, each cited; the costliest
  missing primitive first; a mandatory "what I do NOT recommend" section.

## The improvement (vs the raw skill)

The raw skill optimizes business workflows. AI Logic OS keeps that doctrine as
the spine and adds the power to challenge OmegaOS and any agentic system: it
reads OmegaOS's own Laws/Rules before proposing (many "gaps" are already
rules), checks the logic against L1/L4/R-VERIFY/R-LOOP/R-DESTRUCT/R-GRAPH, stays
current via the `claude-api` SSOT + `/changelog-adopt`, and treats a fix as
unfinished until it is reproducible at install (L0) and proven at runtime (L1).

## v1 scope

Prompt-based (no state CLI), vendored + extended. The 12-question challenge and
the four-bin triage are the contract the agent keeps. For a real multi-angle
audit it can fan out via the OmegaOS Workflow primitive or convene the council;
it always owns the synthesis. No personal data.

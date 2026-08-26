# Intuitive OS

AgentikOS operative system — personal group — **integrated** (Intuitive {OS} v1.0).

An evidence-calibrated operating system for training high-level intuition:
observation, pattern recognition, probabilistic forecasting, decision
journaling, feedback and deliberate calibration. It turns vague instinct into a
measurable learning loop without ever treating a feeling as a fact. Payload
source: `Intuitive-OS-v1.0-full.zip` (Deposit, 2026-08-14). Conversational with
a small deterministic scoring engine, adjacent to Alignment OS and Mindset OS.

## Layout

| Path | What |
|---|---|
| `pack/` | The pack verbatim: `SKILL.md` entry, `prompt/INTUITIVE_OS_SYSTEM.md` (the full contract), `docs/` (book foundations, mental models, playbook), `engine/calibration.py` (Brier score + calibration buckets, stdlib only), `schemas/prediction.schema.json`, `templates/` (intuition capture, prediction, decision journal, pattern card), `tests/`, README, INSTALL, CHANGELOG |
| `MASTER.md` | The master agent (TUI Enter + Telegram bot brain) |
| `bin/omega-intuition` | The OmegaOS command — opens the Intuitive master agent in a session (the scoring engine is a library, not a state CLI) |
| `commands/codex-intuitive-os.md` | The OpenAI/Codex command (installed to `~/.codex/prompts/intuitive-os.md`) |

The Claude command is the `intuitive-os` skill (the pack + its SKILL.md entry
at `skills/intuitive-os/`), installed as `/intuitive-os`, `/omg-intuitive-os`
and the `/intuition` alias.

## Use it

`/intuition` in Claude, the Codex prompt, `omega-intuition` in a terminal, or
the OS master agent (TUI OS tab -> Enter, Telegram bot via `T`). Then a mode:
setup · capture · signal · predict · decide · counter · review · daily ·
weekly · patterns · models · profile · 90d.

Start with `/intuition setup`, then `/intuition 90d`.

## Guardrails (non-negotiable)

- **Intuition is a hypothesis, never automatic truth**, and the raw intuition
  is captured BEFORE analysis contaminates it.
- **The five layers never collapse**: observation, interpretation, affect,
  intuition, decision.
- **Resolvable claims carry a probability** (1 to 99 percent) with an explicit
  resolution criterion and date. **Original entries are immutable** after the
  outcome, which is what makes the calibration honest.
- **Decision quality is not outcome quality** (four-cell review, no resulting,
  no hindsight bias).
- **Domains score separately**; expertise does not transfer. Small samples are
  flagged explicitly, never dressed as statistical reliability.
- **No pseudo-science**: no body-language certainty, no mind reading, no
  magical thinking, and no paranoia about other people's motives.
- High-stakes safety, medical, legal or financial decisions route to evidence
  and expert review; intuition may only generate the questions.

## Scoring engine

`pack/engine/calibration.py` is stdlib-only and deterministic:
`brier_score(forecasts)` and `calibration_buckets(forecasts, width)`.
Verified on install: a perfect pair scores 0, a confident-and-wrong forecast
scores 1.0, a coin flip scores 0.25, and an empty set raises rather than
returning a misleading zero.

## v1 scope (honest divergences)

Prompt-based conversational OS plus a scoring library; there is no
`omega-<name>` state CLI because there is no persisted store in the pack. The
`schemas/prediction.schema.json` is the contract the agent keeps in its ledger
rather than a database, and prediction records are held wherever the operator
chooses to persist them. One deliberate change from the vendored zip: the
`SKILL.md` frontmatter description was widened with routing triggers so the
skill is discoverable by natural phrasing; the doctrine itself is verbatim.

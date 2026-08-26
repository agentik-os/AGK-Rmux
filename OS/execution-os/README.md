# Execution OS

AgentikOS operative system — personal group — **integrated** (Execution OS v2).

An LLM-first personal execution system: it converts ambitions, goals,
obligations and ideas into focused commitments, protected work, shipped
evidence, reviews and adaptive recovery. Execution is a closed control loop
(`Capture -> Clarify -> Select -> Commit -> Focus -> Prove -> Review ->
Adapt`), not a to-do list. Adjacent to Mindset OS (identity) and Habits OS
(consistency) — NOT the software Blueprint/Stepper/Builder pipeline. Payload
source: `execution-os-v2-complete.zip` (Deposit, 2026-08-11).

## Layout

| Path | What |
|---|---|
| `pack/` | The skill pack verbatim: SKILL.md, 7 references (architecture, protocols, schemas, v2-engine, coaching, content-engine, and the profile template), assets (icon), scripts (execution engine + a 6-test suite that passes), agents/openai.yaml; plus `templates/` (15 operating cards: one-page, loop register, signal log, daily command, T2 halt, T3 weekly reset, T4 monthly audit, outcome contract, blocker diagnostic, content proof card, promise ledger, context capsule, focus block, capacity budget, late-promise message) and `examples/` (COMMANDS_V2 + a state template) |
| `MASTER.md` | The master agent (TUI Enter + Telegram bot brain) |
| `bin/omega-execution` | The OmegaOS CLI — the pack's deterministic engine (stdlib Python, no venv). State defaults to `~/.omega/os/execution-os/ledger/execution-state.json` |
| `commands/codex-execution-os.md` | The OpenAI/Codex command (installed to `~/.codex/prompts/execution-os.md`) |

The Claude command is the `execution-os` skill (the pack skill folder verbatim
at `skills/execution-os/`), installed as `/execution-os`, `/omg-execution-os`
and the `/execute` alias.

## Run it

```bash
omega-execution init --owner "You"
omega-execution boot --capacity GREEN --usable-minutes 240 --must-win "Ship X"
omega-execution focus <commitment> --minutes 50
omega-execution complete <commitment> --kind ship --evidence "..." --acceptance "..."
omega-execution halt --classification SHIPPED --energy 7 --focus 8 \
  --friction "..." --tomorrow "..." --proof "..."
omega-execution reset ...    # weekly    ·    omega-execution audit ...   # monthly
```

The coaching runs in an agent: `/execution-os` (or `/execute`) in Claude, the
Codex prompt, or the OS master agent (TUI OS tab -> Enter, Telegram bot via
`T`).

## Personalization + privacy (important)

The shipped `references/gareth-profile.md` is a GENERIC default template — a
fresh install has NO personal data. Your real, private profile lives at
`~/.omega/os/execution-os/ledger/profile.md` (user-local, gitignored via
`OS/*/ledger/`), and the master agent reads it first when present. The
operator's original personal profile + manual are kept only in that local
ledger, never in this public repo.

## v1 scope vs pack spec (honest divergences)

Single-runtime profile, like the other personal OSes: the engine is the pack's
deterministic execution-state CLI; the coaching + content-engine reasoning runs
in the agent (skill / bot). The `benchmark/` folder from the zip (third-party
public-content research) is NOT vendored — it is not part of the OS runtime.
The operator's personal `docs/` manual is kept local (ledger), not committed.

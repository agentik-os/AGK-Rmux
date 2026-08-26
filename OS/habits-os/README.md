# Habits OS

AgentikOS operative system — personal group — **integrated** (Habit Tracker {OS} Full).

A conversation-first, LLM-assisted habit system: create good habits, reduce
unwanted ones, run daily check-ins, handle urges and lapses, produce adaptive
reviews and visual progress — chat as the interface, deterministic SQLite state
underneath. Pairs with Mindset OS for the identity layer. Payload source:
`Habit-Tracker-OS-Full.zip` (Deposit, 2026-08-10).

## Layout

| Path | What |
|---|---|
| `pack/` | The pack verbatim: SKILL.md, 9 references (system prompt, conversation protocols, domain model, behavior science, analytics + visuals, safety + boundaries, omega integration, feature catalog, evaluation suite), assets (state schema, tool contracts, manifest, icon), 3 scripts incl. a 10-test suite that passes, agents/openai.yaml |
| `MASTER.md` | The master agent (TUI Enter + Telegram bot brain) |
| `bin/omega-habits` | The OmegaOS CLI — the pack's deterministic engine (stdlib Python + SQLite, no venv): init / add / update / list / log / correct / today / review / chart / context / export / season / experiment / delete / doctor. Defaults its db to `~/.omega/os/habits-os/ledger/habits.db` |
| `commands/codex-habits-os.md` | The OpenAI/Codex command (installed to `~/.codex/prompts/habits-os.md`) |

The Claude command is the `habit-tracker-os` skill (the pack skill folder
verbatim at `skills/habit-tracker-os/`), installed as `/habits-os`,
`/omg-habits-os` and the `/habits` alias.

## Run it

```bash
omega-habits init --user me --name "Me" --season build
omega-habits add --user me --name "Meditate" --kind build --behavior "..." \
  --why "..." --cue "after coffee" --target daily --minimum "1 min" --fallback "..."
omega-habits today  --user me          # rank today's primary habits
omega-habits log    --user me ...       # record explicit/observed evidence
omega-habits review --user me           # evidence-bounded review
omega-habits doctor                     # validate database integrity
```

The coaching runs in an agent: `/habits-os` (or `/habits`) in Claude, the
Codex prompt, or the OS master agent (TUI OS tab -> Enter, Telegram bot via
`T`). Setup / check-in / urge / lapse / review / adaptation conversations.

## Hard rules

- Chat is the interface; the CLI is the source of truth for state.
- A missed day is DATA; adaptations reversible; evidence explicit; a minimum
  threshold gates any analytic claim.
- Contracts versioned (`update` supersedes); wrong logs `correct`ed; the user
  OWNS their data (`export` / `delete` first-class).
- No clinical/crisis/eating-disorder advice — safety-and-boundaries.md routes
  risk to a professional.

## v1 scope vs pack spec (honest divergences)

Single-runtime profile, like the chain OSes: SQLite is the real store (the
pack's own engine), `assets/tool-contracts.json` is honored as the contract
kept via the CLI rather than a typed dispatch server; the conversation layer
runs in the agent (skill / bot), not a hosted service.

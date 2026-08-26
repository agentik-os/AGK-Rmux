# Books OS

AgentikOS operative system #7 of the OS suite - **integrated**.

Your library as an operating system: reading, retention and living knowledge.
Books OS is the OS wrapper around the knowledge system OmegaOS already ships:
the **Agentik Book {OS} / ALEXANDRIA** librarian - seven functions
(librarian, knowledge cartographer, teacher, learning architect, skeptic,
strategist, personal archivist) and 20+ modes. Books OS does NOT fork that
system; it surfaces it (anti-duplication: one persona, one skill, four
surfaces).

## Canonical sources (the runtime)

| Piece | Where |
|---|---|
| Full persona (the master agent) | `agents/librarian.md` -> installed `~/.omega/agents/librarian.md` |
| Claude skill + reference manual | `skills/alexandria/` -> `/alexandria`, `/books-os` stubs |
| Telegram persona-bot infra | `telegram-bot/omega-tg-bot.ts` (kind `persona`) |
| This OS folder | `MASTER.md` (master-agent entry), `bin/omega-books`, `commands/` |

## Command surface (all modes)

`/setup` calibrate on the user · `/language` reply language · `/book` full
X-Ray · `/espresso` 90-second version · `/chapter` chapter by chapter ·
`/idea` atlas across many books · `/compare` (`/vs`) authors in combat ·
`/apply` to a real business · `/challenge` 10-round sparring · `/decision`
decision lab · `/council` 3-5 perspectives · `/teach` Feynman triple
explanation · `/quiz` `/drill` adaptive recall · `/cards` flashcards ·
`/map` (`/visual`) diagram · `/memory` memory forge · `/review` spaced
repetition · `/capture` `/save` `/applylog` feed the ledger ·
`/readingpath` curated path · `/audio` spoken mode · `/focus` 5-minute
micro-session · `/masterclass` deepest analysis · `/best [topic]` 50 best
books + 50 actionable tips · `/bestsellers [niche]` top 100 · `/gem` an
underrated idea.

## The four surfaces

1. **Claude**: `/alexandria` (canon) - `/books-os` and `/omg-books-os` are
   stubs to the same skill.
2. **Codex**: `~/.codex/prompts/books-os.md` (from `commands/codex-books-os.md`).
3. **OmegaOS CLI**: `omega-books` - opens the librarian master agent in a
   terminal session (claude + the full persona).
4. **Telegram**: `omega-os-bot books-os <token>` links a dedicated bot whose
   brain IS the full librarian persona (`~/.omega/agents/librarian.md`) -
   voice notes transcribed, book files read, big deliverables sent as files.
   Also reachable from the TUI: OS tab -> Books OS -> `T`.

## Personal knowledge stays personal

The bot/session ledger (`~/.omega/os/books-os/ledger/`) accumulates YOUR
profile, captures and application log. Private corpora (e.g. a graphified
book library) stay on the machine - never in this public repo (R-PROJ).

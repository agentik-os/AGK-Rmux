# Storytelling OS

AgentikOS operative system — personal group — **integrated** (Storyteller {OS} v1.0).

The user's story architect, interviewer, truth steward, voice guardian and
narrative strategist — and, only when authorized, writer. It coaches, mines,
verifies, shapes, writes, performs, adapts, scores and preserves TRUTHFUL
stories without erasing the user's voice: personal stories, founder narratives,
brand + customer stories, keynotes, pitches, reels, carousels, threads,
podcasts, YouTube scripts, case studies, and a durable story bank. Payload
source: `Storyteller-OS-v1.0-full.zip` (Deposit, 2026-08-11).

## Layout

| Path | What |
|---|---|
| `pack/` | The skill pack verbatim: SKILL.md, 8 references (system prompt, operating manual, story object, story models, channel playbooks, quality + evals, research + canon, commands, and the operator-context template), assets (icon), scripts (story-bank engine + an 8-test suite that passes), agents/openai.yaml, examples (a Story Object template + quick prompts) |
| `MASTER.md` | The master agent (TUI Enter + Telegram bot brain) |
| `bin/omega-story` | The OmegaOS CLI — the pack's deterministic story-bank engine (stdlib Python + SQLite, no venv). Bank defaults to `~/.omega/os/storytelling-os/ledger/story-bank.db` |
| `commands/codex-storytelling-os.md` | The OpenAI/Codex command (installed to `~/.codex/prompts/storytelling-os.md`) |

The Claude command is the `storyteller-os` skill (the pack skill folder
verbatim at `skills/storyteller-os/`), installed as `/storytelling-os`,
`/omg-storytelling-os` and the `/story` alias.

## Run it

```bash
omega-story init                    # create your story bank
omega-story capture ...             # add a Story Object
omega-story list / show <id>        # browse
omega-story add-claim <id> ...      # a claim-ledger entry (evidence per claim)
omega-story add-consent <id> ...    # a third-party consent record
omega-story validate <id> / score <id>
omega-story doctor                  # validate the whole bank
```

The coaching runs in an agent: `/storytelling-os` (or `/story`) in Claude, the
Codex prompt, or the OS master agent (TUI OS tab -> Enter, Telegram bot via
`T`). Command family: /story · /mine · /interview · /deepen · /shape · /write ·
/adapt · /truthcheck · /score · /rehearse · /storybank.

## Guardrails (non-negotiable)

- Coach, don't ghost-write unless explicitly authorized — the story stays the
  user's, in the user's voice.
- Truth steward: never fabricate a lived detail; label reconstruction and
  composite honestly (the claim ledger); match the chosen evidence standard.
- Consent: never expose an identified third party's private story without a
  recorded consent.

## Personalization + privacy (important)

The shipped `references/gareth-context.md` is a GENERIC default template — a
fresh install has NO personal data. The operator's real, private context (and
the research canon) live only at `~/.omega/os/storytelling-os/ledger/` (local,
gitignored via `OS/*/ledger/`), which the master agent reads first. Private
venture/narrative names were genericized in the committed pack; no personal
data is in this public repo.

## v1 scope vs pack spec (honest divergences)

Single-runtime profile, like the other personal OSes: the engine is the pack's
deterministic story-bank CLI (SQLite); the coaching + writing reasoning runs in
the agent (skill / bot). The `research/` canon from the zip is kept local (the
operator's reference material), not vendored into the public repo.

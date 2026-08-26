# Identity Shift OS

AgentikOS operative system — personal group — **integrated** (Identity Shift {OS} v1.0).

A transformation operating system that converts a declared future identity into
daily evidence, through reflection, challenge, planning, execution, measurement,
memory and selective storytelling. Its premise: identity is not a statement, it
is accumulated evidence. It continuously compares who the user says they want to
become against what they actually did, surfaces the contradictions as data, and
returns the smallest high-leverage next correction. Payload source:
`identity-shift-os-v1.0.zip` (Deposit, 2026-08-14). Conversational, no CLI
engine. Written for both OmegaOS and 0Ra Concierge, and inside OmegaOS it acts
as an orchestration layer above the specialist OSes.

## Layout

| Path | What |
|---|---|
| `pack/` | The pack verbatim plus a `SKILL.md` entry: `SYSTEM_PROMPT.md` (the full contract), 7 `protocols/` (180-day, daily journal, tomorrow, contradiction engine, evidence ledger, weekly review, content engine), 2 `prompts/` (start, mirror), 2 `schemas/` (identity_state, daily_log), `integrations/` (OMEGA_OS, 0RA_CONCIERGE), `examples/`, `skill/identity-shift.skill.md`, README, VERSION |
| `MASTER.md` | The master agent (TUI Enter + Telegram bot brain) |
| `bin/omega-identity` | The OmegaOS command — opens the Identity Shift master agent in a session (this OS is conversational, there is no state CLI) |
| `commands/codex-identity-shift-os.md` | The OpenAI/Codex command (installed to `~/.codex/prompts/identity-shift-os.md`) |

The Claude command is the `identity-shift-os` skill (the pack + its SKILL.md
entry at `skills/identity-shift-os/`), installed as `/identity-shift-os`,
`/omg-identity-shift-os` and the `/identity` alias.

## Use it

`/identity` in Claude, the Codex prompt, `omega-identity` in a terminal, or the
OS master agent (TUI OS tab -> Enter, Telegram bot via `T`). Then a mode:
start · journal · mirror · tomorrow · weekly · monthly · 180 · content ·
contradiction · reset.

## Guardrails (non-negotiable)

- **Life first, content second.** Content is scored and filtered only after the
  private coaching pass, and daily posting is never forced.
- **No shame.** Contradictions are presented as data, never moral failure. No
  guru framing, no humiliation, no fake urgency, no empty affirmations.
- **Maximum 3 major daily objectives**, challenge proportional to capacity.
- **Health, sleep, sobriety and recovery are never traded for vanity metrics**;
  medically sensitive goals keep the ambition but switch to safe constraints and
  route to professional evaluation. Never claims medical certainty.
- **Privacy by default**: the journal is private, and the anti-cringe filter
  rejects oversharing that involves other people.
- Evidence before authority framing.

## Orchestration contract

Identity Shift decides WHY and WHAT NEXT; specialist OS modules decide HOW
(`pack/integrations/OMEGA_OS.md`). Handoff packets carry objective, context,
constraints, evidence, expected outcome, success condition and deadline; return
packets carry actions, decisions, artifacts, risks, evidence and the next
trigger, and are folded back into the user's longitudinal state. It pairs
especially closely with Mindset (identity depth), Habit Tracker (consistency),
Execution (delivery), Intuitive (calibrated judgment) and Alignment (values).

## v1 scope (honest divergences)

Prompt-based conversational OS with no scripts and no `omega-<name>` state CLI:
the two `schemas/` are the contract the agent keeps in its ledger rather than a
database, and there is nothing deterministic to execute or test. Two additions
to the vendored zip, both additive: a `SKILL.md` entry with routing-trigger
frontmatter (the pack ships `skill/identity-shift.skill.md`, which is a
human-readable definition rather than a registry-parsable skill file, and it is
kept verbatim alongside), and the house spine (MASTER, README, MANIFEST, bin,
codex command). The doctrine and every protocol are verbatim.

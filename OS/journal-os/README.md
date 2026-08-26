# Journal OS

AgentikOS operative system — personal group — **integrated** (Journal {OS} v1.0).

MIRROR, a nightly daily-journal agent. It reconstructs the day truthfully,
separates what happened from what was felt and what was concluded, compares
behavior against the declared identity, surfaces contradictions as evidence
rather than as blame, preserves what deserves to outlive the day, and closes
with at most three high-leverage missions for tomorrow. Built from the operator's
MIRROR specification (2026-08-14), with a reflection layer drawn from Stoicism,
Jim Rohn and Taoism. Conversational, no CLI engine.

The journal is private by default. Life comes first. Content is downstream, and
MIRROR never writes a post.

## Layout

| Path | What |
|---|---|
| `pack/` | `SKILL.md` (doctrine + router), `MIRROR_SYSTEM_PROMPT.md` (the full contract), `protocols/` (interview engine, contradiction engine, identity evidence, memory extraction, end-of-interview check, tomorrow protocol, content handoff), `schemas/` (journal entry, contradiction, memory candidate, tomorrow protocol, content candidate), `templates/` (daily journal, final mirror, weekly rollup), `philosophy/` (stoicism, jim-rohn, taoism, quotes, reflection engine) |
| `MASTER.md` | The master agent (TUI Enter + Telegram bot brain) |
| `bin/omega-journal` | The OmegaOS command — opens MIRROR in a session, working from the OS folder so the journal ledger persists |
| `commands/codex-journal-os.md` | The OpenAI/Codex command (installed to `~/.codex/prompts/journal-os.md`) |

The Claude command is the `journal-os` skill (the pack + its SKILL.md entry at
`skills/journal-os/`), installed as `/journal-os`, `/omg-journal-os` and the
`/journal` alias.

## Use it

`/journal` in Claude, the Codex prompt, `omega-journal` in a terminal, or the OS
master agent (TUI OS tab -> Enter, Telegram bot via `T`). Modes: the full
nightly session, plus `quick` · `mirror` · `tomorrow` · `contradiction` ·
`loops` · `memory` · `weekly` · `content`.

## Guardrails (non-negotiable)

- **Evidence over narrative.** FACT, EMOTION, INTERPRETATION and LESSON are four
  different things and are never merged.
- **Challenge without hostility.** Contradictions are named plainly and then
  investigated. Behavioral language only: "you avoided the planned task for 90
  minutes", never "you are lazy".
- **A contradiction requires a pattern**, never a single day. The schema
  enforces it: `observed_days` has a minimum of 2.
- **No artificial positivity and no artificial negativity.** A failed day is
  documented as a failed day; no profound lesson is manufactured, and no problem
  is invented to build a narrative.
- **Maximum three missions tomorrow**, each with a success condition. The schema
  caps it.
- **Privacy by structure**: third parties are referenced by first name only, and
  no field exists anywhere for a surname, handle, workplace or address.
- **Never moralize on sobriety**, never push for unnecessary sexual detail, never
  force gratitude, never create fake balance across domains.
- **MIRROR never writes social posts.** The Content Handoff exposes raw material
  and a privacy label; a separate Content Agent decides publication.

## The philosophy layer

Stoicism, Jim Rohn and Taoism are applied as a lens on evidence, **at most once
per entry and usually not at all**. The nightly review is itself a Stoic
practice (Seneca, *De Ira* III.36), and Marcus Aurelius' *Meditations* is a
private journal never written for publication, which is the same boundary this
OS draws. Rohn supplies the day as the unit of compounding, which is exactly
what the contradiction engine measures. Taoism supplies the counterweight that
keeps the OS from becoming a grind machine: wu wei, the distinction between
restoration and avoidance, and the refusal to manufacture balance. Quotes are
attributed, and lines that circulate widely without a secure source are marked
as such rather than passed off as genuine.

## Relationship to Identity Shift OS

They are deliberately split. **Journal owns the nightly interview and the
artifact; Identity Shift consumes that artifact into longitudinal state and
decides what next.** Identity Shift's own `DAILY_JOURNAL.md` is the summary of
the same practice; Journal {OS} is its full implementation. When the user wants
direction rather than a review, hand off.

## v1 scope (honest divergences)

Prompt-based conversational OS with no `omega-<name>` state CLI: there is
nothing deterministic to execute, and the schemas are the contract the agent
keeps in its ledger rather than a database. The Content Agent referenced by the
handoff is Identity Shift's content engine plus Storyteller OS; Journal {OS}
deliberately stops at raw material.

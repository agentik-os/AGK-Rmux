# Seductive OS

AgentikOS operative system — personal group — **integrated** (Seductive {OS} v1.0).

The personal magnetism operating system: presence, conversational craft,
warmth, style, social calibration, romantic confidence and the inner game
underneath all of it. It does not sell lines, routines or tactics. It builds
the traits that make a person genuinely compelling and then trains the skill of
reading and respecting another person's actual interest. Consent is the
product, not the constraint. Conversational (no CLI engine, the coaching runs
in the agent), adjacent to Mindset OS and Alignment OS.

## Layout

| Path | What |
|---|---|
| `pack/` | The pack: `SKILL.md` entry, `references/` (ethics and consent, refusals, presence, conversation craft, attraction fundamentals, inner game, rejection resilience, style, calibration, flirtation, dating strategy, apps, long-term desire, anxiety, safety, evidence map), `agents/` (the specialist voices), `protocols/`, `schemas/`, `assets/` |
| `MASTER.md` | The master agent (TUI Enter + Telegram bot brain) |
| `bin/omega-seduction` | The OmegaOS command — opens the Seductive master agent in a session (no state CLI; this OS is conversational) |
| `commands/codex-seductive-os.md` | The OpenAI/Codex command (installed to `~/.codex/prompts/seductive-os.md`) |

The Claude command is the `seductive-os` skill (the pack + a SKILL.md entry at
`skills/seductive-os/`), installed as `/seductive-os`, `/omg-seductive-os` and
the `/seduction` and `/charisma` aliases.

## Use it

`/seduction` (or `/charisma`) in Claude, the Codex prompt, `omega-seduction` in
a terminal, or the OS master agent (TUI OS tab -> Enter, Telegram bot via `T`).
Then a natural request or a mode: /presence · /conversation · /innergame ·
/style · /calibrate · /flirt · /date · /apps · /rejection · /desire · /audit ·
/practice · /debrief · /reset.

## Guardrails (non-negotiable)

- **Consent-first.** Reading and respecting disinterest is a trained skill
  here, not an obstacle. Nobody is entitled to another person's interest.
- **The refused playbook is named, not implied**: no negging, manufactured
  scarcity, love-bombing, pressure past a no, or deception about intent.
  `references/refusals.md` explains why each one also fails on the user's own
  terms.
- **Build the person, not the mask.** Prefer a change that survives being known.
- **Epistemic labels** E1 / E2 / E3 / P / C always. Most of this domain is
  craft knowledge (E3) and personal taste (P), and the OS says so instead of
  borrowing authority from science that did not replicate.
- **No scripts.** Principles and a worked example, then the user's own words.
- **Privacy**: debriefs describe real people who never consented to be in a
  file. Minimize persistence, first names at most, ledger only with consent.
- Not a clinician — body dysmorphia, compulsive pursuit, disabling anxiety or
  crisis route to a qualified professional (label **C**).
- Inclusive by default: never assumes the user's gender, orientation or
  relationship structure.

## v1 scope (honest divergences)

Prompt-based conversational OS, no scripts and no `omega-<name>` state CLI
(there is nothing deterministic to run). The `schemas/` are the contract the
agent keeps in its ledger rather than a database. The specialist voices run
inside one agent (or via the OmegaOS Workflow primitive for a real multi-voice
fan-out), not as separate services. The evidence base in this domain is thin
and heavily practitioner-derived; `references/evidence-map.md` is deliberately
honest about which claims rest on replicated work and which are craft.

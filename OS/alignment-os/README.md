# Alignment OS

AgentikOS operative system — personal group — **integrated** (Alignment Coach {OS} v1.0).

A personal wisdom, decision and inner-alignment operating system — a rigorous,
compassionate, action-oriented second brain for questions about life,
decisions, identity, work, relationships, money, purpose, fear, discipline,
ambition and meaning. It does not tell you what to believe; it helps you see
reality clearly, separate control from non-control, reconnect decisions to
chosen values, widen perspective without magical thinking, choose the right
level of effort, turn insight into a concrete next action, and learn from the
result. Synthesizes Stoicism, Daoism (wu-wei), Jim Rohn, grounded manifestation
and accurate quantum guardrails. Payload source: `Alignment-Coach-OS-v1.0.zip`
(Deposit, 2026-08-11). Conversational (no CLI engine — the coaching runs in the
agent), adjacent to Mindset OS.

## Layout

| Path | What |
|---|---|
| `pack/` | The pack verbatim: `system/` (SYSTEM_PROMPT + PRINCIPLES + ROUTER), `config/os.yaml`, 12 council `agents/`, 17 `skills/`, 5 `protocols/`, `knowledge/` (book canon + Stoicism/Daoism/Rohn/law-of-attraction/quantum), `schemas/` (memory + session), `memory/` (model + privacy), `evals/`, `examples/`, MANIFEST.json, README, INSTALL, plus the SKILL.md entry |
| `MASTER.md` | The master agent (TUI Enter + Telegram bot brain) |
| `bin/omega-align` | The OmegaOS command — opens the Alignment master agent in a session (no state CLI; this OS is conversational) |
| `commands/codex-alignment-os.md` | The OpenAI/Codex command (installed to `~/.codex/prompts/alignment-os.md`) |

The Claude command is the `alignment-os` skill (the pack + a SKILL.md entry at
`skills/alignment-os/`), installed as `/alignment-os`, `/omg-alignment-os` and
the `/coach` and `/align` aliases. (`/council` is deliberately NOT used — it is
the llm-council command.)

## Use it

`/coach` (or `/align`) in Claude, the Codex prompt, `omega-align` in a
terminal, or the OS master agent (TUI OS tab -> Enter, Telegram bot via `T`).
Then a natural request or a protocol/skill: /morning · /evening · /weekly ·
/decision · /true_north · /virtue_check · /dichotomy_control · /wu_wei ·
/reframe · /shadow · /belief_audit · /fear · /meaning · /manifestation ·
/quantum_truth · /personal_philosophy · /anti_dependency · /reset (3-min).

## Guardrails (non-negotiable)

- Anti-dependency: coach WITH the user, hand back agency, every insight ends in
  a concrete next action.
- Epistemic labels E1–E5 always; never present a manifestation/quantum claim
  (E4) as established science.
- Privacy: personal reflections are high-sensitivity; minimize persistence;
  the user can inspect/correct/delete memory.
- Not a clinician — route real crisis/medical risk to a qualified professional.

## v1 scope vs pack spec (honest divergences)

The pack is a prompt-based conversational OS (no scripts), vendored verbatim.
There is no `omega-<name>` state CLI (nothing to run); the memory schemas
(`schemas/`) are the contract the agent keeps in its ledger. The council's
12 voices run inside one agent (or via the OmegaOS Workflow primitive for a
real multi-voice fan-out), not as separate services.

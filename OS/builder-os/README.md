# Builder OS

AgentikOS build chain **#06** - **integrated** (v1).

The autonomous implementation runtime - the last system of the build chain:
it executes a project from its FROZEN Blueprint {OS} handoff and its BUILD
READY Stepper {OS} graph into tested, reviewed, integrated, documented,
release-ready code, like a professional dev following the roadmap. Payload
source: `Builder_OS_v1_Omega_OS_Complete.zip` (Deposit, 2026-08-10).

Build chain: `01 Ideation -> 02 Researcher -> 03 Blueprint -> 04 Designer
(UX/UI) -> 05 Stepper -> 06 Builder`. Builder consumes, never redefines:
approved Blueprint/ADR > frozen Stepper graph > dependency artifacts >
repository evidence > implementation preference.

## Layout

| Path | What |
|---|---|
| `pack/` | The pack verbatim: SKILL.md, 12 references (603-line system prompt, contract, intake-preflight, execution-loop, roles-orchestration, verification-gates BG01-BG20, git-integration, change-governance, documentation-followup, recovery-and-resume, release-handoff, omega-integration), assets (tools/state-schema/role-prompts/manifest/icon), scripts (state CLI + installer), agents/openai.yaml |
| `MASTER.md` | The master agent (TUI Enter + Telegram bot brain) |
| `bin/omega-builder` | The OmegaOS CLI - the pack's deterministic state engine (stdlib Python, no venv, 15 verbs incl. an in-memory `demo` self-test) |
| `commands/codex-builder-os.md` | The OpenAI/Codex command (installed to `~/.codex/prompts/builder-os.md`) |

The Claude command is the `builder-os` skill (`skills/builder-os/` = the pack
skill folder verbatim), installed as `/builder-os`, `/omg-builder-os` and the
`/build` alias.

## Run it

```bash
omega-builder demo                          # in-memory end-to-end self-test
omega-builder init builder-state.json --project-id my-app ...
omega-builder validate builder-state.json   # structure + semantic invariants
omega-builder status   builder-state.json   # evidence-backed status
omega-builder gate     builder-state.json   # evaluate BG01-BG20
omega-builder release-check builder-state.json
```

The reasoning half runs in an agent: `/build` (or `/builder-os`) in Claude,
the Codex prompt, or the OS master agent (TUI OS tab -> Enter, Telegram bot
via `T`). Command family: preflight / status / plan / run / step / test /
verify / repair / audit / resume / pause / release-check / report.

## Hard boundary

Builder never replaces Stepper's planner/tracker/verifier with its own TODO
list - it executes their program (`omega-stepper`) and writes deterministic
evidence back (sync-step, record-check, mark-step). Definition conflicts go
upstream as decision requests. The only terminal success is Stepper release
PASS + BG01-BG20 PASS -> `finalize` (frozen final handoff).

## v1 scope vs pack spec (honest divergences)

Same deployment posture as Blueprint OS v3: the pack's single-runtime
profile. The state store is the pack's own JSON contract (its CLI, atomic
writes); SQLite/transactional-server persistence, the typed function-dispatch
server (`assets/builder-tools.json` is honored as the contract the agent
keeps via the CLI), and daemonized agent adapters stay at contract level -
the agent driving `/build` + the OmegaOS Workflow primitive fills those
roles. Every boundary, gate and evidence rule is preserved.

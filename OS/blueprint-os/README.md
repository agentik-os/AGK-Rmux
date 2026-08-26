# Blueprint OS

AgentikOS operative system **#1 of the build chain** - **integrated** (v3).

The product-definition COMPILER: turns an idea + project context into a
complete, coherent, traceable Product + Technical Definition Pack - 38
sections, stable IDs, epistemic ledgers (FACT/DECISION/ASSUMPTION/...),
bidirectional traceability, 20 quality gates, and a FROZEN handoff to
Stepper OS. Payload source: `Blueprint_OS_v3_Omega_OS_Complete.zip`
(Deposit, 2026-08-10). It replaces the v1 14-phase designer (archived, see
below).

The build chain: **1. Blueprint OS** (define) → **2. Stepper OS** (plan +
execute step by step) → **3. Builder OS** (assemble + ship, upcoming).

## Layout

| Path | What |
|---|---|
| `pack/` | The 14 pack files verbatim: master system prompt (808 lines), contract, orchestration + 20 gates, response/continuation, functions/state, integration guide, deep guide, tools/state-schema/role-prompts JSON, `blueprint_os.py`, installer |
| `MASTER.md` | The master agent (TUI Enter + Telegram bot brain) |
| `bin/omega-blueprint` | The OmegaOS CLI - the pack's deterministic state helper (stdlib Python, no venv) |
| `commands/codex-blueprint-os.md` | The OpenAI/Codex command (installed to `~/.codex/prompts/blueprint-os.md`) |

The Claude command is the replaced `blueprint-os` skill
(`skills/blueprint-os/`: v3 SKILL.md + `references/` + `assets/`), installed
as `/blueprint-os`, `/omg-blueprint-os` and the `/blueprint` alias.

## Run it

```bash
omega-blueprint demo                          # a valid minimal state, to read
omega-blueprint init blueprint/state.json \
  --project-id my-app --project-name "My App" \
  --namespace my.app --request "Compile the blueprint"
omega-blueprint validate blueprint/state.json # exit 1 on critical/high issues
omega-blueprint status  blueprint/state.json
omega-blueprint checkpoint blueprint/state.json --current "..." --next "..."
```

The reasoning half runs in an agent: `/blueprint-os` (or `/blueprint <idea>`)
in Claude, the Codex prompt, or the OS master agent (TUI OS tab -> Enter,
Telegram bot via `T`). Modes: NEW / RECOVER / EXTEND / REVISE / AUDIT / DELTA.

## Hard boundary

`Idea -> Blueprint {OS} -> Stepper {OS} -> Build {OS} -> Ship`. Blueprint
stops at `BLUEPRINT COMPLETE — STEPPER READY`: every gate green, 100% trace
coverage on critical records, and a frozen handoff (version + checksum) that
Stepper consumes - never a moving latest pointer.

## v3 scope vs pack spec (honest divergences)

The pack defines three deployment profiles (`references/omega-os-integration.md`
§10). This integration runs the pack-sanctioned **minimal single-agent
profile** — one master prompt, sequential passes, JSON state file,
deterministic validation, markdown export — with every boundary, gate and ID
rule preserved:

- **Fan-out**: the 15 specialist roles (`assets/blueprint-role-prompts.json`)
  run sequentially inside one agent by default; for the professional
  orchestrated profile the agent uses the OmegaOS Workflow primitive as the
  DAG runtime (fan-out -> chief-editor merge), not a bespoke daemon.
- **Tools**: `assets/blueprint-tools.json` is honored as the CONTRACT the
  agent keeps via the `omega-blueprint` CLI + canonical `state.json`; no
  typed function-dispatch server is wired (nothing in OmegaOS speaks that
  protocol today).
- **Persistence**: JSON state + checkpoints per the portable contract;
  PostgreSQL/graph projections of the enterprise profile are out of scope.

## v1 legacy (kept on purpose)

The previous 14-phase AgentikOS designer is archived at
`skills/blueprint-os/legacy/` (SKILL-v1.md + its references). Its SCRIPTS
stay live in `skills/blueprint-os/scripts/` (blueprint-check.sh,
stax_derive.py, plan_build.py, runner.py, ...) because the `/stack` chain
(R-BLUEPRINT-STACK) still gates scaffolds on `blueprint-check.sh`. The v3
compiler owns the `/blueprint-os` surface; the legacy designer is reachable
by reading `legacy/SKILL-v1.md` explicitly.

# Stepper OS

AgentikOS operative system #2 of the build chain - **integrated**.

Step-by-step execution: the operating system that walks a Blueprint one
verified step at a time. Stepper is NOT the coding agent - it is the execution
OS around coding agents: it owns the sequence (planner), the truth (tracker)
and the definition of done (deterministic verifier). Payload source:
`Stepper-OS-Pro-Pack.zip` (Deposit, 2026-08-10).

## Layout

| Path | What |
|---|---|
| `pack/` | The 15 canonical spec documents (master spec, step contracts, planner/scheduler/tracker, verification gates, agent protocol...) |
| `engine/` | The Python engine (pydantic + typer + networkx), `stepper` CLI, 26 pytest tests |
| `bin/omega-stepper` | The OmegaOS command - lazy-venv launcher for the engine |
| `commands/codex-stepper-os.md` | The OpenAI/Codex slash command (installed to `~/.codex/prompts/stepper-os.md`) |

The Claude command is the `stepper-os` skill (`skills/stepper-os/SKILL.md`,
installed as `/stepper-os` + `/omg-stepper-os`).

## Run it

```bash
omega-stepper init --name my-project   # scaffold stepper.yaml + spec tree
omega-stepper validate                 # schema + references + DAG acyclicity
omega-stepper plan                     # ranked READY candidates + safe wave
omega-stepper start STEP-000001        # claim a step, prints the agent brief
omega-stepper done STEP-000001         # verifier decides - never self-report
omega-stepper release-check            # the only terminal success
```

First `omega-stepper` run creates the venv at `~/.omega/os/stepper-os/.venv`
and installs the engine (install.sh never pip-installs - runtime opt-in).

## The loop

```text
resume -> status -> plan -> start -> implement -> done(verify) -> plan -> ...
                                       ^                |
                                       └── repair loop ─┘  (bounded by max_fix_attempts)
```

- DONE is only reachable through the verifier (file/grep/command/review-gate
  checks, argv execution, no shell=True).
- Dependencies are authoritative: `start` refuses steps that are not READY.
- State survives restarts: `.stepper/state.json` + append-only
  `.stepper/events.jsonl`; `resume` reconciles interrupted attempts.
- Review gates: `omega-stepper review <step> <role> PASS --by <name>`.
- The repair loop is bounded (`execution.max_fix_attempts`): the ceiling
  escalates to a human instead of thrashing.

## v1 scope vs pack spec (honest divergences)

- State store is JSON (atomic writes), not SQLite: zero-dep, inspectable,
  restart-safe. Swap planned when multi-worker leases land.
- Worktree orchestration, agent adapters (auto-spawning coders) and the
  blueprint compiler are spec-level (`pack/06`, `pack/09`); today the agent
  driving the CLI plays that role via the /stepper-os command.
- `pack/stepper_engine_skeleton` from the zip was superseded by `engine/`
  (a strict superset) and is not vendored.

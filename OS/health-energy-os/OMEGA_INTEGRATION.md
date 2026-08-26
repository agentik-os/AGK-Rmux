# Omega Integration Contract

## Registration
- ID: `health-energy`
- Version: `1.0.0`
- Default command: `/health`
- Position: Personal Stack: upstream capacity provider for Habit, Execution and Strategy & Portfolio OS

## Context injection order
1. `system/SYSTEM_PROMPT.md`
2. `system/PRINCIPLES.md`
3. relevant authorized memory
4. current records and evidence
5. selected specialist agent(s)
6. selected skill or protocol
7. current user message

Never inject the entire knowledge library by default.

## Handoffs
- Habit Tracker OS receives agreed routines, not raw medical files.
- Execution OS receives a capacity status and workload constraints.
- Strategy & Portfolio OS receives sustainable capacity assumptions.
- Qualified professionals receive a concise question pack when escalation is needed.

## Event types
- health.observation.captured
- readiness.assessed
- experiment.started
- experiment.stopped
- health.alert.created
- handoff.execution.capacity
- handoff.habits.created
- health.capacity.assessed

## Produces (pipeline wiring)
- `health.capacity.assessed` -> consumed by Strategy & Portfolio OS (`strategy-portfolio-os/OMEGA_INTEGRATION.md`). Payload: capacity level, constraints, validity window, confidence, provenance, privacy class.
- `handoff.execution.capacity` -> consumed by Execution OS.
- `handoff.habits.created` -> consumed by Habit Tracker OS.

## Consumes
- None upstream today; Health & Energy OS is the suite's upstream-most capacity provider.

## State classification
- Canonical (routes through Context & Memory OS): confirmed health observations, readiness assessments, active experiments.
- Local operational state: in-session coaching context, draft protocols not yet confirmed.
- Read: `memory.context.compiled` from Context & Memory OS. Write: `memory.record.staged` for confirmed observations/experiments; Context & Memory OS returns `memory.record.verified`.

## Change control
Changes to boundaries, schemas or quality gates require Review & Governance OS approval in production.

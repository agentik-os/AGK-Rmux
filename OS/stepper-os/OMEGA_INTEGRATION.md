# Omega Integration Contract

## Registration
- ID: `stepper`
- Version: `1.0.0`
- Default command: `/stepper-os`
- Alias: `/stepper`
- Position: Product Stack: dependency-aware step graph and deterministic verification gate, third stage of the IMPLEMENT branch (`Blueprint -> Design -> Stepper -> Builder`)

## Context injection order
1. `SKILL.md` operating contract
2. the approved Blueprint (and Design Handoff when present)
3. current step graph and tracker state
4. current user message

Never inject the entire knowledge library by default.

## Handoffs
- Blueprint OS / Design OS supply the compiled contract this OS turns into a dependency-aware step graph.
- Builder OS receives the frozen step graph and executes it; DONE is gated by a deterministic verifier, never self-reported.

## Event types
- stepper.graph.frozen
- stepper.step.verified

## Produces (pipeline wiring)
- `stepper.graph.frozen` -> consumed by Builder OS.

## Consumes
- `blueprint.completed` from Blueprint OS.
- `design.handoff.completed` from Design OS (when a design pass preceded this step).

## State classification
- Canonical (routes through Context & Memory OS): the frozen step graph, per-step verification evidence.
- Local operational state: the live tracker (current step, in-progress implementation) - a projection of the canonical graph, never a competing source of truth.
- Read: `memory.context.compiled`. Write: `memory.record.staged` for the frozen graph and verified steps; Context & Memory OS returns `memory.record.verified`.

## Change control
Changes to boundaries, schemas or quality gates require Review & Governance OS approval in production.

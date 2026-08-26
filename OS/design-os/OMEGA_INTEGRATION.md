# Omega Integration Contract

## Registration
- ID: `design`
- Version: `1.0.0`
- Default command: `/design`
- Position: Product Stack: UX/interaction/visual design compilation, second stage of the IMPLEMENT branch (`Blueprint -> Design -> Stepper -> Builder`)

## Context injection order
1. `SKILL.md` operating contract
2. relevant authorized memory / prior design decisions
3. current records and evidence (`references/*.md`)
4. the approved Blueprint
5. current user message

Never inject the entire knowledge library by default.

## Handoffs
- Blueprint OS supplies product/system truth (what and why).
- Stepper OS receives the machine-readable Design Handoff (how: behavior, structure, surfaces, states, testable contracts).

## Event types
- design.handoff.completed
- design.flow.challenged

## Produces (pipeline wiring)
- `design.handoff.completed` -> consumed by Stepper OS.

## Consumes
- `blueprint.completed` from Blueprint OS.

## State classification
- Canonical (routes through Context & Memory OS): the design handoff (resolved design graph, screen contracts, states).
- Local operational state: in-progress flow challenges, draft wireframes.
- Read: `memory.context.compiled`. Write: `memory.record.staged` for the completed design handoff; Context & Memory OS returns `memory.record.verified`.

## Change control
Changes to boundaries, schemas or quality gates require Review & Governance OS approval in production.

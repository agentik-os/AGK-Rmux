# Omega Integration Contract

## Registration
- ID: `blueprint`
- Version: `1.0.0`
- Default command: `/blueprint`
- Position: Product Stack: Product + Technical Definition Pack, first stage of the IMPLEMENT branch (`Strategy -> Blueprint -> Design -> Stepper -> Builder`)

## Context injection order
1. `SKILL.md` operating contract
2. relevant authorized memory / prior project state (stable IDs preserved)
3. current records and evidence (`references/*.md`)
4. selected mode (new / recovery / revision / extension / delta)
5. current user message

Never inject the entire knowledge library by default.

## Handoffs
- Strategy & Portfolio OS supplies the approved product bet.
- Market Research OS / Brainstorm OS supply validated concepts and evidence.
- Design OS receives the completed Blueprint.
- Stepper OS receives the completed Blueprint for step-graph compilation.

## Event types
- blueprint.completed
- blueprint.gate.passed

## Produces (pipeline wiring)
- `blueprint.completed` -> consumed by Design OS and Stepper OS. Stops at `BLUEPRINT COMPLETE -> STEPPER READY`.

## Consumes
- `strategy.product_bet.approved` from Strategy & Portfolio OS.
- `market.validation.completed` from Market Research OS.
- `brainstorm.concept.selected` from Brainstorm OS.

## State classification
- Canonical (routes through Context & Memory OS): the completed Blueprint (product/UX/domain/data/API/AI/security/operations/test contracts), stable IDs.
- Local operational state: draft/incomplete blueprint sections during compilation.
- Read: `memory.context.compiled` (prior project state). Write: `memory.record.staged` for the completed Blueprint; Context & Memory OS returns `memory.record.verified`.

## Change control
Changes to boundaries, schemas or quality gates require Review & Governance OS approval in production.

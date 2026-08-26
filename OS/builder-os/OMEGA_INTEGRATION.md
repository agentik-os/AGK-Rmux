# Omega Integration Contract

## Registration
- ID: `builder`
- Version: `1.0.0`
- Default command: `/build`
- Alias: `Build OS`
- Position: Product Stack: autonomous implementation runtime, final stage of the IMPLEMENT branch, hands off to Quality, Evaluation & Release OS

## Context injection order
1. `SKILL.md` operating contract
2. the frozen Stepper graph and step contract
3. current repository evidence
4. current user message

Never inject the entire knowledge library by default.

## Handoffs
- Stepper OS supplies the frozen implementation graph this OS executes.
- Quality, Evaluation & Release OS receives shipped build artifacts and returns defects/gate decisions.
- Delivery & Customer Success OS may provide an approved, bounded client-change scope with acceptance criteria.

## Event types
- builder.artifact.shipped
- builder.build.verified

## Produces (pipeline wiring)
- `builder.artifact.shipped` -> consumed by Quality, Evaluation & Release OS (the Builder -> Quality edge; Codex's named example event).
- `builder.build.verified` is a local closure event, raised once Quality's gate decision is consumed (see below); it is staged canonically via `memory.record.staged` as release evidence, it is not itself consumed by name outside this OS.

## Consumes
- `stepper.graph.frozen` from Stepper OS.
- `defect.opened` from Quality, Evaluation & Release OS (repair loop).
- `release.gate.decided` from Quality, Evaluation & Release OS (gate decision closes the build with `builder.build.verified`).
- `delivery.change_scope.approved` from Delivery & Customer Success OS.

## State classification
- Canonical (routes through Context & Memory OS): shipped artifacts, verification evidence.
- Local operational state: in-progress step execution, working-tree state.
- Read: `memory.context.compiled`. Write: `memory.record.staged` for shipped/verified artifacts; Context & Memory OS returns `memory.record.verified`.

## Change control
Changes to boundaries, schemas or quality gates require Review & Governance OS approval in production.

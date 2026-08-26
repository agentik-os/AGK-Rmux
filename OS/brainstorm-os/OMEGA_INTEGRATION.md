# Omega Integration Contract

## Registration
- ID: `brainstorm`
- Version: `1.0.0`
- Default command: `/brainstorm`
- Position: Core Stack (supporting): idea generation and evolution preceding Market Research / Blueprint

## Context injection order
1. `SKILL.md` operating contract
2. relevant authorized memory
3. current records and evidence
4. selected council agent(s) (`agents/*.md`)
5. selected protocol (`references/*.md`)
6. current user message

Never inject the entire knowledge library by default.

## Handoffs
- Market Research OS receives selected concepts for validation.
- Blueprint OS receives a decision-ready concept when validation is skipped by explicit authorization.
- Context & Memory OS stores concept lineage.

## Event types
- brainstorm.concept.selected
- brainstorm.session.completed

## Produces (pipeline wiring)
- `brainstorm.concept.selected` -> consumed by Market Research OS and Blueprint OS.

## Consumes
- None upstream; Brainstorm OS is the suite's ideation entry point.

## State classification
- Canonical (routes through Context & Memory OS): selected concepts, concept lineage/decision history.
- Local operational state: in-session council deliberation, rejected/mutated concept branches.
- Read: `memory.context.compiled`. Write: `memory.record.staged` for selected concepts; Context & Memory OS returns `memory.record.verified`.

## Change control
Changes to boundaries, schemas or quality gates require Review & Governance OS approval in production.

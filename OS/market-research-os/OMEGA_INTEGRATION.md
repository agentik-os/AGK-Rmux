# Omega Integration Contract

## Registration
- ID: `market-research`
- Version: `1.0.0`
- Default command: `/market-research`
- Position: Core Stack (supporting): market/customer evidence and validation for Strategy and Blueprint

## Context injection order
1. `SKILL.md` operating contract
2. relevant authorized memory
3. current records and evidence
4. selected specialist agent(s) (`agents/*.md`)
5. selected protocol (`references/*.md`, `scripts/*`)
6. current user message

Never inject the entire knowledge library by default.

## Handoffs
- Brainstorm OS supplies selected concepts to validate.
- Blueprint OS receives a validated concept as a Blueprint input manifest.
- Strategy & Portfolio OS and Revenue OS receive willingness-to-pay and segment evidence through `market.validation.completed`.

## Event types
- market.validation.completed
- market.study.audited

## Produces (pipeline wiring)
- `market.validation.completed` -> consumed by Blueprint OS, Strategy & Portfolio OS and Revenue OS.

## Consumes
- `brainstorm.concept.selected` from Brainstorm OS.

## State classification
- Canonical (routes through Context & Memory OS): versioned evidence bodies, validated hypotheses, final decisions.
- Local operational state: in-progress scraping/interview sessions, draft models.
- Read: `memory.context.compiled`. Write: `memory.record.staged` for validated evidence; Context & Memory OS returns `memory.record.verified`.

## Change control
Changes to boundaries, schemas or quality gates require Review & Governance OS approval in production.

# Omega Integration Contract

## Registration
- ID: `mindset`
- Version: `1.0.0`
- Default command: `/mindset`
- Position: Personal Stack: identity and belief compiler, subordinate to Alignment OS (never the ETRE authority itself)

## Context injection order
1. `SKILL.md` operating contract
2. relevant authorized memory
3. current records and evidence
4. selected specialist agent(s) (`agents/*.md`)
5. selected protocol (`references/*.md`, `scripts/*`)
6. current user message

Never inject the entire knowledge library by default.

## Handoffs
- Alignment OS supplies updated intent/values as the compilation input.
- Habit Tracker OS receives behavior contracts to track.
- Strategy & Portfolio OS receives only strategic implications, not raw identity work.
- Context & Memory OS stores confirmed identity/belief compilations.

## Event types
- mindset.behavior_contract.created
- mindset.identity_compilation.updated

## Produces (pipeline wiring)
- `mindset.behavior_contract.created` -> consumed by Habit Tracker OS.
- `mindset.identity_compilation.updated` -> consumed by Strategy & Portfolio OS (strategic implications only).

## Consumes
- `alignment.intent.updated` from Alignment OS.
- `habit.review.completed` from Habit Tracker OS (closes the `Mindset intent -> Habit contract -> Daily evidence -> Pattern/review -> Mindset reflection` loop).

## State classification
- Canonical (routes through Context & Memory OS): confirmed identity/belief compilations, behavior contracts.
- Local operational state: in-session coaching context, draft 30/90-day plans.
- Read: `memory.context.compiled`. Write: `memory.record.staged` for confirmed compilations; Context & Memory OS returns `memory.record.verified`.

## Change control
Changes to boundaries, schemas or quality gates require Review & Governance OS approval in production.

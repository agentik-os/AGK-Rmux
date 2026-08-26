# Omega Integration Contract

## Registration
- ID: `alignment`
- Version: `1.0.0`
- Default command: `/align`
- Alias: `/coach` (registered alias of the same OS, not a competing command)
- Position: Personal Stack: the BE/ETRE authority - identity, values and inner alignment, upstream of Mindset OS

## Context injection order
1. `system/SYSTEM_PROMPT.md` - the operating contract
2. `system/PRINCIPLES.md` - the non-negotiable principles
3. relevant authorized memory
4. current records and evidence
5. selected council voice(s) (`agents/*.md`)
6. selected skill or protocol (`skills/*.md`, `protocols/*.md`)
7. current user message

Never inject the entire knowledge library by default.

## Handoffs
- Mindset OS receives the user's updated intent/values as an identity-and-belief compilation input, never raw personal reflections.
- Strategy & Portfolio OS receives only strategic implications of an alignment decision, never raw personal reflections.
- Context & Memory OS stores confirmed decisions and belief-audit outcomes.

## Event types
- alignment.intent.updated
- alignment.strategy_implication.created
- alignment.decision.recorded

## Produces (pipeline wiring)
- `alignment.intent.updated` -> consumed by Mindset OS (identity/belief compilation input).
- `alignment.strategy_implication.created` -> consumed by Strategy & Portfolio OS. The payload contains only the strategic implication and privacy classification, never raw personal reflections.
- `alignment.decision.recorded` is a local domain event; each confirmed decision is staged canonically via `memory.record.staged` (see State classification), it is not itself consumed by name outside this OS.

## Consumes
- `memory.context.compiled` from Context & Memory OS, restricted to the authorized scope.

## State classification
- Canonical (routes through Context & Memory OS): confirmed decisions, belief-audit outcomes, true-north statements.
- Local operational state: in-session council deliberation, draft reframes.
- Read: `memory.context.compiled`. Write: `memory.record.staged` for confirmed decisions; Context & Memory OS returns `memory.record.verified`.

## Change control
Changes to boundaries, schemas or quality gates require Review & Governance OS approval in production.

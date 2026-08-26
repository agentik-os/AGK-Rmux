# Omega Integration Contract

## Registration
- ID: `relationship-network`
- Version: `1.0.0`
- Default command: `/network`
- Position: Personal Stack: trusted relationship memory, communication and network stewardship

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
- Content OS receives only explicit, consent-safe stories or testimonials.
- Revenue OS receives business CRM events, not private relationship notes.
- Delivery & Customer Success OS receives client commitments relevant to service.
- Execution OS receives follow-up tasks without unnecessary personal detail.

## Event types
- relationship.interaction.captured
- relationship.commitment.created
- relationship.commitment.closed
- relationship.followup.drafted
- relationship.introduction.requested
- relationship.gathering.created
- relationship.revenue_introduction.ready
- relationship.delivery_commitment.ready
- relationship.data.deleted

## Produces (pipeline wiring)
- `relationship.followup.drafted` -> consumed by Execution OS (personal-execute branch, as a follow-up task without unnecessary personal detail).
- `relationship.gathering.created` -> may be consumed by Content OS only as explicit, consent-safe story material (never raw notes).
- `relationship.revenue_introduction.ready` -> consumed by Revenue OS as a consented business introduction, never private relationship history.
- `relationship.delivery_commitment.ready` -> consumed by Delivery & Customer Success OS as a consented service commitment.

## Consumes
- None declared upstream today; Relationship & Network OS is a supporting personal-stack OS.

## State classification
- Canonical (routes through Context & Memory OS): confirmed commitments, consented introductions.
- Local operational state: draft follow-ups, in-session interaction capture.
- Read: `memory.context.compiled`. Write: `memory.record.staged` for confirmed commitments; Context & Memory OS returns `memory.record.verified`.

## Change control
Changes to boundaries, schemas or quality gates require Review & Governance OS approval in production.

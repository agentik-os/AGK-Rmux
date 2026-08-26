# Omega Integration Contract

## Registration
- ID: `content`
- Version: `1.0.0`
- Default command: `/content`
- Position: Communication Stack: turns life, expertise, products and proof into native multi-platform content

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
- Context & Memory OS provides authorized source material and voice history.
- Storyteller OS may deepen narrative structures.
- Revenue OS provides offer/audience-stage objectives and receives qualified intent signals.
- Quality/Review govern sensitive claims and publication policy.
- Operations OS automates production only after the workflow is stable.

## Ownership boundary: Content vs Storyteller
Content owns editorial strategy, packaging, channel adaptation, publishing and content analytics. Content does NOT own narrative craft, story structure, voice or consent - that is Storyteller OS. Content's own `storyteller` MANIFEST agent operates under this boundary: it packages, it does not originate narrative truth.

## Event types
- content.source.ingested
- content.atom.created
- content.pillar.approved
- content.cascade.created
- content.asset.drafted
- content.asset.approved
- content.asset.published
- content.metric.recorded
- content.experiment.reviewed
- content.rights.blocked
- content.intent.qualified
- content.performance.feedback
- content.release_candidate.created
- content.change.requested
- content.automation_candidate.created

## Produces (pipeline wiring)
- `content.intent.qualified` -> consumed by Revenue OS (commercial pipeline: Content -> qualified intent -> Revenue). Payload: source asset, audience segment, intent signal, consent status, attribution window, offer/campaign objective, confidence.
- `content.performance.feedback` -> consumed by Storyteller OS, for story-object learning only (never for publishing decisions - see ownership boundary).
- `content.release_candidate.created` -> consumed by Quality, Evaluation & Release OS for claims, rights and release checks.
- `content.change.requested` -> consumed by Review & Governance OS for sensitive policy or boundary changes.
- `content.automation_candidate.created` -> consumed by Operations & Automation OS only after workflow stability evidence exists.

## Consumes
- `revenue.offer_objective.updated` from Revenue OS (offer/campaign objectives).
- `story.ready_for_adaptation` from Storyteller OS (deepened narrative ready to package).
- `relationship.gathering.created` from Relationship & Network OS, only as explicit, consent-safe story/testimonial material - never raw relationship notes.

## State classification
- Canonical (routes through Context & Memory OS): published assets, approved pillars, rights/consent decisions.
- Local operational state: draft cascades, in-progress asset drafts.
- Read: `memory.context.compiled` (source material + voice history). Write: `memory.record.staged` for published assets; Context & Memory OS returns `memory.record.verified`.

## Change control
Changes to boundaries, schemas or quality gates require Review & Governance OS approval in production.

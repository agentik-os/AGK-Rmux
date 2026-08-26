# Omega Integration Contract

## Registration
- ID: `operations-automation`
- Version: `1.0.0`
- Default command: `/operations`
- Position: Business Stack: diagnoses operating systems, simplifies work and designs governed automation

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
- Strategy & Portfolio OS selects operational priorities.
- Delivery/Revenue/Content provide current workflows and desired outcomes.
- Quality, Evaluation & Release OS tests and gates production automations (automation blueprints stay internal to Operations; they are never a Builder software-build input, a different pipeline entirely).
- Review & Governance approves risk/policy changes and postmortems.
- Context & Memory stores maps, contracts and run evidence.

## Event types
- operations.process.discovered
- operations.map.approved
- automation.candidate.scored
- automation.blueprint.approved
- automation.run.started
- automation.run.failed
- automation.review.requested
- automation.reconciled
- automation.incident.opened
- automation.retired
- operations.production_observed
- operations.capacity_margin.verified
- automation.change.requested

## Produces (pipeline wiring)
- `operations.production_observed` -> consumed by Quality, Evaluation & Release OS (production feedback loop).
- `automation.candidate.scored` -> consumed by Delivery & Customer Success OS (automation changes affecting client delivery) and AI Logic OS (deterministic-code-vs-AI arbitration).
- `operations.capacity_margin.verified` -> consumed by Wealth & Capital OS and Strategy & Portfolio OS (productivity/margin gains feeding capital allocation).
- `automation.change.requested` -> consumed by Review & Governance OS (governance handshake, see below).
- `automation.review.requested` -> consumed by Review & Governance OS.

## Consumes
- `quality.operations_handoff.ready` from Quality, Evaluation & Release OS (runbooks + observability contract).
- `delivery.workflow.stable` from Delivery & Customer Success OS (a workflow becomes an automation candidate only once stable).
- `change.approved` from Review & Governance OS (governance gate, see below).
- `ailogic.arbitration.decided` from AI Logic OS (deterministic-code-vs-AI-judgment arbitration before scoring a candidate).
- `content.automation_candidate.created` from Content OS, only with workflow stability evidence.

## Governance
Internal `automation.blueprint.approved` is an OPERATIONS-level review, not a Governance approval. A consequential automation change (touching risk, policy, client-facing scope) requires: `automation.change.requested -> Review consumes -> Review emits change.approved -> Operations consumes -> Operations emits automation.blueprint.approved -> automation.run.started`.

## State classification
- Canonical (routes through Context & Memory OS): approved process maps, automation blueprints, run evidence.
- Local operational state: in-progress process discovery, draft automation candidates.
- Read: `memory.context.compiled`. Write: `memory.record.staged` for maps/contracts/run evidence; Context & Memory OS returns `memory.record.verified`.

## Change control
Changes to boundaries, schemas or quality gates require Review & Governance OS approval in production.

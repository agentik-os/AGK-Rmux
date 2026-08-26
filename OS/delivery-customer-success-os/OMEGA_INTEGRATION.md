# Omega Integration Contract

## Registration
- ID: `delivery-customer-success`
- Version: `1.0.0`
- Default command: `/delivery`
- Position: Business Stack: converts sold promises into accepted outcomes, adoption, retention and advocacy

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
- Revenue OS provides approved contract/offer and receives billing/renewal signals.
- Builder/Operations receive scoped implementation work.
- Quality, Evaluation & Release OS provides acceptance/release evidence.
- Review & Governance receives incidents and delivery learning.
- Content OS receives only consented case-study material.

## Event types
- delivery.handoff.accepted
- client.onboarding.started
- success.plan.approved
- milestone.completed
- deliverable.accepted
- client.risk.escalated
- delivery.change_scope.approved
- adoption.reviewed
- value.evidence.verified
- renewal.opened (deprecated, superseded by `delivery.renewal_signal.created`)
- client.offboarded
- delivery.workflow.stable
- delivery.renewal_signal.created

## Post-payment authorization gate
Delivery never starts on `revenue.delivery_handoff.created` alone. Sequence: `revenue.contract.signed -> payment.reconciled -> revenue.delivery_handoff.created -> delivery.handoff.accepted` (using Revenue OS's own declared `payment.reconciled` event, not an undeclared one), so delivery cannot begin before commercial payment is reconciled.

## Produces (pipeline wiring)
- `delivery.handoff.accepted` -> consumed by Revenue OS (closes the commercial pipeline's Revenue -> Delivery edge with an acceptance confirmation).
- `delivery.workflow.stable` -> consumed by Operations & Automation OS (a workflow becomes an automation candidate only once stable).
- `delivery.renewal_signal.created` -> consumed by Revenue OS and Strategy & Portfolio OS (Delivery owns the SIGNAL, Revenue owns the renewal DECISION).
- `delivery.change_scope.approved` -> consumed by Builder OS with bounded scope and acceptance criteria.

## Consumes
- `revenue.contract.signed` from Revenue OS.
- `payment.reconciled` from Revenue OS (post-payment authorization gate, see above).
- `revenue.delivery_handoff.created` from Revenue OS.
- `automation.candidate.scored` from Operations & Automation OS (automation changes affecting client delivery).
- `revenue.renewal.decisioned` from Revenue OS.
- `relationship.delivery_commitment.ready` from Relationship & Network OS (consented client commitment relevant to service).

## State classification
- Canonical (routes through Context & Memory OS): accepted deliverables, milestone completions, value-evidence verification.
- Local operational state: in-progress onboarding checklists, draft success plans.
- Read: `memory.context.compiled`. Write: `memory.record.staged` for accepted/verified records; Context & Memory OS returns `memory.record.verified`.

## Change control
Changes to boundaries, schemas or quality gates require Review & Governance OS approval in production.

# Omega Integration Contract

## Registration
- ID: `revenue`
- Version: `1.0.0`
- Default command: `/revenue`
- Position: Business Stack: conversational revenue brain, CRM, offers, sales, billing, AR and business financial control

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
- Content OS sends qualified intent signals and receives offer/campaign objectives.
- Market Research OS supplies willingness-to-pay and segment evidence.
- Delivery & Customer Success OS receives signed scope and returns realized-value/health evidence.
- Wealth & Capital OS receives only verified owner compensation/distribution.
- Strategy & Portfolio OS receives commercial signals and capacity/economics.
- Accountants/controllers receive organized source packs and exception lists.

## Ownership boundary: Revenue vs Wealth & Capital
Revenue owns the BUSINESS ledger, receivables, payables, business reserves, business forecast. It never owns personal money. The only event that crosses the boundary is `revenue.owner_distribution.verified`; raw business transaction history is never shared with Wealth & Capital OS.

## Internal state machine (SELL -> COLLECT)
`offer.approved -> proposal.approved -> contract.signed -> invoice.sent -> payment.reconciled -> revenue.close.completed`. `payment.reconciled` is the single COLLECT-completion event exposed to the suite.

## Event types
- revenue.document.staged
- crm.lead.created
- crm.opportunity.updated
- sales.call.debriefed
- offer.version.approved
- proposal.approved
- invoice.sent
- invoice.overdue
- payment.reconciled
- revenue.close.completed
- forecast.updated
- reserve.alert.created
- renewal.opened (deprecated, superseded by `delivery.renewal_signal.created` + `revenue.renewal.decisioned`)
- handoff.wealth.created (legacy orchestration event, superseded below - do not treat as the financial fact)
- revenue.contract.signed
- revenue.delivery_handoff.created
- revenue.owner_distribution.verified
- revenue.renewal.opened (deprecated, superseded by `revenue.renewal.decisioned`)
- revenue.renewal.decisioned
- revenue.change.requested

## Produces (pipeline wiring)
- `revenue.offer_objective.updated` -> consumed by Content OS.
- `revenue.contract.signed` -> consumed by Delivery & Customer Success OS (commercial pipeline: Revenue -> signed contract -> Delivery).
- `payment.reconciled` -> consumed by Delivery & Customer Success OS (post-payment authorization gate: delivery never starts on the contract alone).
- `revenue.delivery_handoff.created` -> consumed by Delivery & Customer Success OS. Payload: contract version, scope baseline, promised outcomes, acceptance criteria, price, billing schedule, client permissions, exclusions.
- `revenue.owner_distribution.verified` -> consumed by Wealth & Capital OS (the ONLY event that crosses the business/personal boundary; supersedes the ambiguous `handoff.wealth.created`/`handoff.revenue.received` pair Codex flagged).
- `revenue.renewal.decisioned` -> consumed by Delivery & Customer Success OS and Strategy & Portfolio OS.
- `revenue.change.requested` -> consumed by Review & Governance OS (governance handshake, see below).

## Consumes
- `content.intent.qualified` from Content OS.
- `delivery.handoff.accepted` from Delivery & Customer Success OS (closes the Revenue -> Delivery edge).
- `delivery.renewal_signal.created` from Delivery & Customer Success OS (Revenue owns the renewal DECISION; Delivery owns the renewal SIGNAL - resolves the duplicate-ownership `renewal.opened` name Codex flagged).
- `change.approved` from Review & Governance OS (governance gate, see below).
- `market.validation.completed` from Market Research OS (willingness-to-pay and segment evidence).
- `strategy.product_bet.approved` from Strategy & Portfolio OS (approved bet and commercial constraints).
- `relationship.revenue_introduction.ready` from Relationship & Network OS (consented business introduction, never private notes).

## Governance
Pricing, schema, or billing-policy changes are POLICY changes, not operational events. Sequence: `revenue.change.requested -> Review consumes -> Review emits change.approved -> Revenue consumes -> Revenue emits offer.version.approved` for the consequential change. Ordinary invoicing and payment reconciliation remain operational events and do not require this gate.

## State classification
- Canonical (routes through Context & Memory OS): signed contracts, closed revenue periods, verified owner distributions.
- Local operational state: in-progress CRM opportunities, draft proposals.
- Read: `memory.context.compiled`. Write: `memory.record.staged` for closed/verified records; Context & Memory OS returns `memory.record.verified`.

## Change control
Changes to boundaries, schemas or quality gates require Review & Governance OS approval in production.

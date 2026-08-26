# Omega Integration Contract

## Registration
- ID: `strategy-portfolio`
- Version: `1.0.0`
- Default command: `/strategy`
- Position: Omega Core: selects goals, bets, projects and resource allocation before execution

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
- Context & Memory OS supplies versioned evidence and constraints.
- Market Research OS validates market assumptions.
- Blueprint OS receives an approved product bet.
- Execution OS receives quarterly outcomes and exclusions (personal-execution branch; see Boundary note below).
- Revenue/Delivery/Operations report signals back to strategy.
- Health & Energy OS supplies sustainable capacity assumptions.
- Wealth & Capital OS supplies personal capital constraints for allocation decisions.
- Review & Governance OS approves consequential allocation changes and closes the learning loop with Context & Memory.

## Boundary: two meanings of "execute"
The value-chain note places EXECUTE right after CHOOSE. This suite has two distinct branches from Strategy's output, never conflated:
- `Strategy -> Personal Execute` (Execution OS: personal commitments, time-bound proof).
- `Strategy -> Blueprint -> Design -> Stepper -> Builder` (the product IMPLEMENT branch; see `strategy.product_bet.approved` below).

## Event types
- strategy.diagnosis.created
- strategy.kernel.approved
- portfolio.item.funded
- portfolio.item.paused
- portfolio.item.killed
- allocation.changed
- scenario.signpost.triggered
- strategy.review.completed
- strategy.product_bet.approved
- strategy.execution_packet.created
- strategy.change.requested
- strategy.refresh.requested

## Produces (pipeline wiring)
- `strategy.product_bet.approved` -> consumed by Blueprint OS (product IMPLEMENT branch entry point) and Revenue OS (commercial constraints and approved bet).
- `strategy.execution_packet.created` -> consumed by Execution OS (personal-execute branch entry point; quarterly outcomes + exclusions payload).
- `strategy.change.requested` -> consumed by Review & Governance OS (governance handshake, see below).
- `strategy.refresh.requested` -> consumed by Review & Governance OS, closing the Review -> Context -> Strategy learning loop.

## Consumes
- `health.capacity.assessed` from Health & Energy OS.
- `execution.outcome.proven` from Execution OS (personal-execute branch feedback).
- `memory.context.snapshot.created` from Context & Memory OS (learning-loop input).
- `change.approved` from Review & Governance OS (governance gate, see below).
- `mindset.identity_compilation.updated` from Mindset OS (strategic implications only, never raw identity work).
- `capital.reallocation.proposed` from Wealth & Capital OS (Operations -> Capitalize edge feeding allocation decisions).
- `alignment.strategy_implication.created` from Alignment OS (strategic implication only, no raw reflection).
- `market.validation.completed` from Market Research OS (validated segment, demand, pricing evidence and decision).
- `revenue.renewal.decisioned` from Revenue OS, `delivery.renewal_signal.created` from Delivery & Customer Success OS, and `operations.capacity_margin.verified` from Operations & Automation OS as concrete portfolio feedback.

## Governance
Strategy may emit `portfolio.item.funded`, `portfolio.item.paused`, `portfolio.item.killed`, or `allocation.changed` for a CONSEQUENTIAL change only after the matching `change.approved` (or `policy.exception.granted`) event is present for the `strategy.change.requested` that preceded it. Sequence: `strategy.change.requested -> Review consumes -> Review emits change.approved | policy.exception.granted -> Strategy consumes -> Strategy emits the portfolio/allocation event`.

## State classification
- Canonical (routes through Context & Memory OS): approved strategy kernels, funded/paused/killed portfolio items, allocation decisions.
- Local operational state: draft diagnoses, in-progress scenario modeling.
- Read: `memory.context.snapshot.created` / `memory.context.compiled`. Write: `memory.record.staged` for decisions; Context & Memory OS returns `memory.record.verified`.

## Change control
Changes to boundaries, schemas or quality gates require Review & Governance OS approval in production.

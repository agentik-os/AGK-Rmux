# Omega Integration Contract

## Registration
- ID: `wealth-capital`
- Version: `1.0.0`
- Default command: `/wealth`
- Position: Personal Stack: personal cash-flow, resilience, investment policy and capital allocation

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
- Revenue OS sends verified owner pay/distribution and business reserve information.
- Strategy & Portfolio OS receives personal capital constraints, not raw transaction history.
- Execution OS receives agreed money tasks and reminders.
- Qualified advisers receive organized records and question packs.

## Ownership boundary: Wealth & Capital vs Revenue
Wealth owns the PERSONAL ledger, personal reserves, personal goals, investments. It never owns business cash flow, receivables/payables or business reserves - that is Revenue OS. The only event that crosses the boundary is `revenue.owner_distribution.verified`; raw business transaction history is never accepted from Revenue.

## Event types
- finance.document.staged
- finance.transaction.verified
- finance.month.closed
- finance.goal.updated
- finance.reminder.created
- finance.decision.recorded
- handoff.revenue.received (legacy orchestration event, superseded below - do not treat as the financial fact)
- capital.reallocation.proposed
- wealth.change.requested
- wealth.execution_task.created

## Produces (pipeline wiring)
- `capital.reallocation.proposed` -> consumed by Strategy & Portfolio OS and Review & Governance OS (Operations -> Capitalize edge: productivity/margin gains reaching capital allocation).
- `wealth.change.requested` -> consumed by Review & Governance OS (governance handshake, see below).
- `wealth.execution_task.created` -> consumed by Execution OS. The payload contains an agreed action, due date and privacy class, never raw financial records.

## Consumes
- `revenue.owner_distribution.verified` from Revenue OS (the ONLY event crossing the business/personal boundary; supersedes the ambiguous `handoff.wealth.created`/`handoff.revenue.received` pair Codex flagged).
- `operations.capacity_margin.verified` from Operations & Automation OS (productivity/margin gains).
- `change.approved` from Review & Governance OS (governance gate, see below).

## Governance
A capital-allocation decision is a consequential change. Sequence: `wealth.change.requested -> Review consumes -> Review emits change.approved -> Wealth consumes -> Wealth emits finance.decision.recorded / capital.reallocation.proposed`.

## State classification
- Canonical (routes through Context & Memory OS): verified transactions, closed months, recorded financial decisions.
- Local operational state: draft goals, in-progress reminders.
- Read: `memory.context.compiled`. Write: `memory.record.staged` for verified/closed records; Context & Memory OS returns `memory.record.verified`.

## Change control
Changes to boundaries, schemas or quality gates require Review & Governance OS approval in production.

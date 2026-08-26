# Omega Integration Contract

## Registration
- ID: `execution`
- Version: `1.0.0`
- Default command: `/execute`
- Position: Personal Stack: time-bound PERSONAL commitments and output proof. Explicitly NOT the Blueprint -> Design -> Stepper -> Builder software-implementation pipeline (see Boundary below).

## Context injection order
1. `SKILL.md` operating contract
2. relevant authorized memory
3. current records and evidence
4. selected specialist agent(s) (`agents/*.md`)
5. selected protocol (`references/*.md`, `scripts/*`)
6. current user message

Never inject the entire knowledge library by default.

## Boundary: two meanings of "execute"
This OS owns PERSONAL EXECUTE: capture -> clarify -> select -> commit -> focus -> prove -> review -> adapt, for a human's commitments. The software-implementation branch (product IMPLEMENT: Blueprint -> Design -> Stepper -> Builder) is a completely separate pipeline and never receives events from this OS as if they were build instructions.

## Handoffs
- Strategy & Portfolio OS provides the quarterly outcomes/exclusions packet and receives proven outcomes back.
- Health & Energy OS provides capacity status and workload constraints.
- Review & Governance OS receives execution evidence for cross-domain learning.
- Relationship & Network OS provides follow-up tasks to schedule.
- Wealth & Capital OS provides agreed money tasks without raw account or transaction data.

## Event types
- execution.outcome.proven
- execution.commitment.closed

## Produces (pipeline wiring)
- `execution.outcome.proven` -> consumed by Strategy & Portfolio OS and Review & Governance OS.

## Consumes
- `strategy.execution_packet.created` from Strategy & Portfolio OS (personal-execute branch entry point).
- `handoff.execution.capacity` from Health & Energy OS.
- `relationship.followup.drafted` from Relationship & Network OS.
- `wealth.execution_task.created` from Wealth & Capital OS.

## State classification
- Canonical (routes through Context & Memory OS): closed commitments with proof, weekly/quarterly reviews.
- Local operational state (`scripts/execution_engine.py`): today's single-thread focus state, in-progress commitment drafts - a local projection, never a competing source of truth for closed commitments.
- Read: `memory.context.compiled`. Write: `memory.record.staged` for closed/proven commitments; Context & Memory OS returns `memory.record.verified`.

## Change control
Changes to boundaries, schemas or quality gates require Review & Governance OS approval in production.

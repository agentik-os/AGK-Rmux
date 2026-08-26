# Omega Integration Contract

## Registration
- ID: `ai-logic`
- Version: `1.0.0`
- Default command: `/ai-logic`
- Alias: `/ailogic`
- Position: Core Stack (cross-cutting): reasoning/automation-arbitration doctrine. A consulting layer any OS may invoke to challenge its own automation, never a fixed pipeline stage.

## Context injection order
1. `SKILL.md` operating contract
2. relevant references (`references/*.md`)
3. the system/pipeline/agent under challenge
4. current user message

Never inject the entire knowledge library by default.

## Handoffs
- Operations & Automation OS is the primary consumer: AI Logic arbitrates deterministic-code-vs-AI-judgment before an automation candidate is scored/approved.
- Review & Governance OS may invoke AI Logic when auditing an automation or agentic-system change.
- Any OS may invoke AI Logic ad hoc to challenge one of its own agents, skills or pipelines; such invocations are not suite events, only the Operations handoff is wired as a standing edge.

## Event types
- ailogic.arbitration.decided

## Produces (pipeline wiring)
- `ailogic.arbitration.decided` -> consumed by Operations & Automation OS (arbitration input before `automation.candidate.scored`).

## Consumes
- `automation.candidate.scored` from Operations & Automation OS, when arbitrating an existing automation candidate.

## State classification
- Canonical (routes through Context & Memory OS): recorded arbitration decisions (deterministic-vs-AI verdicts) when they gate a real automation.
- Local operational state: ad hoc challenge sessions invoked by another OS, not persisted as suite state.
- Read: `memory.context.compiled`. Write: `memory.record.staged` for recorded arbitration decisions; Context & Memory OS returns `memory.record.verified`.

## Change control
Changes to boundaries, schemas or quality gates require Review & Governance OS approval in production.

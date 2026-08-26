# Omega Integration Contract

## Registration
- ID: `review-governance`
- Version: `1.0.0`
- Default command: `/review`
- Position: Omega Core: closes learning loops and governs consequential change across all OSs

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
- Explicitly wired OSs send named evidence or change events; owners receive approved actions.
- Strategy & Portfolio OS receives decision-quality and portfolio review findings.
- Quality & Release OS receives release policy and risk tolerance.
- Context & Memory OS stores versioned policies, decisions and audit history.

## Ownership boundary: cross-OS governance vs each OS's own domain retrospective
Review & Governance owns cross-domain learning and any boundary or policy change. Domain OSes keep their own operational retrospectives and act on them locally. Only concrete named events in the manifests cross this boundary; wildcard producers and wildcard event names are forbidden.

## Event types
- review.completed
- risk.created
- risk.changed
- policy.approved
- policy.exception.granted
- change.requested
- change.approved
- change.verified
- incident.opened
- incident.closed
- decision.audited
- review.learning.pack.created

## Produces (pipeline wiring)
- `change.approved` -> consumed by Strategy & Portfolio OS, Revenue OS, Quality Evaluation & Release OS, Operations & Automation OS, and Wealth & Capital OS (the governance handshake, one instance per OS - see each OS's own "## Governance" section).
- `policy.exception.granted` -> consumed by Quality, Evaluation & Release OS (release-gate exception handshake).
- `review.learning.pack.created` -> consumed by Context & Memory OS, closing the Review -> Context -> Strategy learning loop.

## Consumes
- `strategy.change.requested` from Strategy & Portfolio OS.
- `revenue.change.requested` from Revenue OS.
- `quality.release_exception.requested` from Quality, Evaluation & Release OS.
- `automation.change.requested` from Operations & Automation OS.
- `automation.review.requested` from Operations & Automation OS.
- `wealth.change.requested` from Wealth & Capital OS.
- `strategy.refresh.requested` from Strategy & Portfolio OS (learning-loop closure).
- `capital.reallocation.proposed` from Wealth & Capital OS.
- `design.flow.challenged` from Design OS.
- `execution.outcome.proven` from Execution OS.
- `content.change.requested` from Content OS.

## State classification
- Canonical (routes through Context & Memory OS): approved policies, approved/denied changes, closed incidents, audited decisions.
- Local operational state: in-progress reviews, draft risk assessments.
- Read: `memory.context.compiled`. Write: `memory.record.staged` for approved/audited records; Context & Memory OS returns `memory.record.verified`.

## Change control
Changes to boundaries, schemas or quality gates require Review & Governance OS approval in production.

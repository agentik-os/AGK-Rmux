# Omega Integration Contract

## Registration
- ID: `quality-evaluation-release`
- Version: `1.0.0`
- Default command: `/quality`
- Position: Product Stack: independent certification between Builder OS and production

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
- Blueprint/Design provide contracts; Stepper provides implementation order; Builder provides build artifacts.
- Review & Governance OS owns policy exceptions and postmortems.
- Operations OS receives runbooks and observability contracts.
- Context & Memory OS stores release evidence manifests.

## Event types
- quality.plan.created
- test.executed
- eval.executed
- defect.opened
- defect.closed
- release.candidate.created
- release.gate.decided
- deployment.started
- deployment.verified
- rollback.executed
- incident.handoff.created
- quality.operations_handoff.ready
- quality.release_exception.requested

## Produces (pipeline wiring)
- `defect.opened` -> consumed by Builder OS (build-fix feedback loop).
- `release.gate.decided` -> consumed by Builder OS (build-fix feedback loop).
- `quality.operations_handoff.ready` -> consumed by Operations & Automation OS (runbooks + observability contract).
- `quality.release_exception.requested` -> consumed by Review & Governance OS (governance handshake, see below).
- `release.candidate.created` is an internal Quality state-machine event (pre-gate), not a cross-OS handoff.

## Consumes
- `builder.artifact.shipped` from Builder OS (build artifact ready for certification).
- `operations.production_observed` from Operations & Automation OS (production feedback loop).
- `policy.exception.granted` from Review & Governance OS, required before `deployment.started` whenever a gate is bypassed or risk is accepted.
- `change.approved` from Review & Governance OS for consequential quality-policy changes.
- `content.release_candidate.created` from Content OS for claims, rights and publication release checks.

## Governance
Quality never runs `deployment.started` on a bypassed or risk-accepted gate without governance sign-off. Sequence: `quality.release_exception.requested -> Review consumes -> Review emits policy.exception.granted -> Quality consumes -> Quality emits deployment.started`.

## State classification
- Canonical (routes through Context & Memory OS): release evidence manifests, release-gate decisions, incident handoffs.
- Local operational state: in-progress test/eval runs, draft defect triage.
- Read: `memory.context.compiled`. Write: `memory.record.staged` for release evidence; Context & Memory OS returns `memory.record.verified`.

## Change control
Changes to boundaries, schemas or quality gates require Review & Governance OS approval in production.

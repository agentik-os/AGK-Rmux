---
type: feature
name: Orchestration v3
feature_type: Platform
status: In Progress
related:
  - opportunity:orchestration-v3
  - workflow:mission-lifecycle-v3
updated: 2026-07-30
discovery:
  problem_confidence: 0.95
  solution_confidence: 0.92
  business_confidence: 0.88
  technical_confidence: 0.90
  overall_confidence: 0.91
priority:
  method: weighted
  user_value: 95
  business_value: 88
  strategic_alignment: 100
  confidence: 85
  urgency: 82
  feasibility: 84
  score: 90.25
---

# Feature: Orchestration v3

## Summary

Refactor OmegaOS around a small provider-neutral governance kernel, typed mission contracts, adaptive delegation, versioned skills, explicit role authority, structured traces, and risk-proportional verification.

## User story

As an OmegaOS operator, I want every agent runtime to receive only the policies and capabilities relevant to its mission, so that missions finish reliably with less context waste, fewer contradictions, and stronger runtime evidence.

## Business objective

Improve verified mission completion while reducing coordination tax, prompt size, provider lock-in, and repeated orchestration defects.

## Expected outcome

OmegaOS can execute simple, parallel, high-risk, and interrupted missions through one explicit lifecycle, while loading provider and domain details only when needed.

## Scope

- Constitutional laws and operational rule model.
- Rule activation and progressive disclosure.
- Provider capability adapters.
- Skill manifest, validation, discovery, installation, and publication lifecycle.
- Oracle, worker, subagent, council, plan, scope, retry, gate, and terminal state semantics.
- Structured observability and compatibility migration.
- Installer, sync, documentation, and deterministic tests.

## Out of scope

- Replacing rmux.
- Rebuilding the TUI visual design.
- Changing client application stacks.
- Migrating production data.
- Removing backward compatibility before a measured migration window.
- Adding a hosted control plane.

## Acceptance criteria

1. Universal laws are provider-neutral, internally consistent, and concise.
2. Rules declare scope, trigger, enforcement mode, risk class, owner, version, and evidence.
3. Domain and infrastructure policies load on demand rather than universally.
4. Provider-specific commands and model IDs live in adapters or registries, not universal doctrine.
5. Missions have an explicit typed lifecycle with valid transitions and terminal evidence.
6. Delegation is selected adaptively from dependency, risk, context, cost, and conflict signals.
7. Roles define purpose, capabilities, knowledge, authority, boundaries, and success criteria.
8. Skills have a manifest, validation command, compatibility metadata, version, tests, and publish state.
9. Every handoff and terminal decision emits a structured trace event.
10. Existing commands retain compatibility or provide a tested migration path.
11. `cargo test --workspace`, audit-specific tests, `./scripts/verify-install.sh`, and clean-install checks pass.
12. Three representative runtime scenarios pass: inline simple mission, file-disjoint parallel mission, and blocked destructive mission.
13. The event store is the only write authority; legacy files are provenance-marked one-way projections.
14. Local state transitions use expected-version CAS and idempotency keys; external delivery is duplicate-tolerant at-least-once.
15. Task attempts and missions use separate state machines, and no scope or final report is released before acceptance.
16. Skills compile from bounded owned roots into deterministic provider states and atomic install generations.

## Edge cases

- A provider has no native subagents or plan tool.
- A worker dies after claiming file scope.
- A result arrives after its mission generation has changed.
- A skill supports one provider but not another.
- A mission changes shape after discovery.
- The operator interrupts during quality verification.
- A destructive action is the only remaining task.
- A hook is untrusted, unavailable, or times out.
- A remote notification succeeds but the local acknowledgement is lost.
- An old worker resumes after a newer fenced lease owns the task.

## Risks

- Refactoring the rules source of truth can break generated doctrine.
- Changing lifecycle state can invalidate existing state files.
- Adaptive delegation can become opaque without traceable scores.
- Migration layers can become permanent duplicate systems.
- A 100 percent quality target can encourage performative tests unless confidence remains evidence-calibrated.

## Success metrics

- Verified Mission Completion Rate.
- Universal doctrine bytes and estimated tokens.
- Rule activation precision.
- Coordination tax.
- Orphaned state rate.
- Resume success.
- Fresh-install parity.
- Provider compatibility test pass rate.

## Tests

- Registry schema and invariant tests.
- Doctrine rendering snapshot and size-budget tests.
- Mission transition property tests.
- Delegation policy table tests.
- Skill manifest and compatibility validation tests.
- State migration tests.
- Hook degradation tests.
- CLI end-to-end fixtures for representative mission classes.

## Implementation status

Implemented and under final verification:

- Provider-neutral compact rule compiler with deterministic digest and 24 KB budget.
- Codex and Claude plan-event parsing in the finish guard.
- Canonical typed skill catalog shared by install, Atlas, RAG, and provider activation.
- SQLite WAL mission ledger with CAS, idempotency, task attempts, fenced leases, and outbox.
- Candidate completion, independent verifier observations, and accepted-attempt delivery gate.
- Persisted provider identity and ANSI-preserving, width-bounded Codex pane rendering.
- Codex default for fresh OmegaOS and Oracle sessions without overwriting an existing explicit provider choice.
- Canonical audit registry and fail-closed evidence/final-verdict runner.

Release remains blocked until the full workspace, clean-install, audit, PDF delivery, and Git push gates complete.

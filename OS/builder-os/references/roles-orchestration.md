# Roles and Orchestration

## Contents

1. Orchestration principle
2. Shared-state model
3. Logical specialist roles
4. Dispatch rules
5. Review independence
6. Concurrency and ownership
7. Human gates

## 1. Orchestration principle

Treat roles as bounded capabilities, not mandatory persistent agents. A single capable agent may perform several roles sequentially; an orchestrator may dispatch independent work in parallel when tools, authorization, Stepper locks, and WIP limits permit.

Do not use multiple roles to vote on product truth. Blueprint and approved ADRs remain authoritative. The Build Director resolves execution conflicts from contracts and evidence.

## 2. Shared-state model

Every role reads a scoped snapshot containing:

- project/step/attempt IDs;
- Blueprint and Stepper fingerprints;
- exact authority and task contract;
- relevant code/artifacts;
- allowed tools and environment;
- active locks and forbidden changes;
- required output schema.

Every role writes proposals or evidence to the same canonical Builder state. Only the state coordinator applies transitions under optimistic concurrency and append-only journaling.

## 3. Logical specialist roles

### Build Director

Own overall execution, boundaries, escalation, wave coordination, and final integrated-release truth. Never replace Stepper Planner or Verifier.

### Contract Intake Auditor

Validate Blueprint/Stepper status, versions, checksums, traceability, graph integrity, repository identity, release target, and conditional/manual gates.

### Repository Cartographer

Map repository topology, instructions, toolchains, boundaries, commands, environments, CI, generated code, migrations, and pre-existing failures.

### Environment Engineer

Establish reproducible local/test setup, tool versions, service dependencies, environment templates, and capability inventory without exposing secrets.

### Stepper Adapter

Read Planner/Scheduler/Tracker/Verifier outputs, translate them into Builder actions, and write evidence back without inventing a parallel graph.

### Context Compiler

Assemble minimal, versioned, source-located, secret-free context for each attempt and invalidate stale bundles.

### Implementation Engineer

Implement the smallest complete code change satisfying the step contract and repository conventions.

### Test Engineer

Add and execute unit/domain/integration/contract/E2E tests required by the step. Keep test truth separate from implementation preference.

### Root-Cause and Repair Engineer

Classify failures, isolate minimal evidence, preserve correct work, repair root cause within scope, and stop at repair limits.

### Architecture Reviewer

Check domain boundaries, dependency direction, duplication, contracts, consistency, backward compatibility, NFRs, and undocumented architecture drift.

### Security, Privacy, and Abuse Reviewer

Check authentication, authorization, isolation, secrets, injection, data exposure, privacy lifecycle, abuse cases, supply chain, and unsafe side effects.

### UX, Accessibility, and Visual Reviewer

Check all declared states, interaction behavior, accessibility, design-system alignment, responsive/safe-area behavior, and visual evidence.

### Data and Migration Reviewer

Check schema compatibility, backfills, idempotency, integrity, concurrency, rollback/recovery, and deploy ordering.

### AI and Evaluation Reviewer

Check tool schemas/authorization, context and memory, provenance, prompt injection, uncertainty/abstention, eval thresholds, cost/latency, traces, receipts, and rollback.

### Performance and Reliability Reviewer

Check budgets, load/concurrency, timeouts, retries, idempotency, queues, caching, degradation, availability, and recovery evidence.

### Integration Engineer

Refresh bases, resolve contract-aware conflicts, integrate verified changes, run post-merge checks, and record the integrated revision.

### Documentation and Runbook Engineer

Keep setup, architecture, API/data/event contracts, migrations, operations, incidents, risks, and follow-up documentation true and traceable.

### Release Engineer

Evaluate project gates, assemble final report/artifacts, verify deployment/rollback/monitoring readiness, and prepare operations handoff.

### Traceability Auditor

Confirm Blueprint → Stepper → code/test/docs/evidence coverage, identify orphans, and prevent unsupported completion claims.

## 4. Dispatch rules

- Dispatch only a concrete bounded task with explicit inputs/outputs.
- Include no secret values or irrelevant full-project context.
- Respect Stepper resource locks and Builder file ownership.
- Use parallel dispatch only for independent steps or independent read-only reviews.
- Do not let a specialist merge, complete a step, or change canonical contracts unless its role explicitly owns the action.
- Reject output that lacks required evidence schema or references a stale input hash.

## 5. Review independence

Implementation self-check and independent certification are different stages. Prefer deterministic tooling for objective checks. For judgment reviews, use a fresh role context focused on contract, diff, tests, and evidence rather than the implementer's narrative.

Critical security, data, payment, privacy, trust, and production changes may require an explicit human/manual gate even when automated review passes.

## 6. Concurrency and ownership

Use leases and resource locks for writers. Allow concurrent read-only review when it cannot mutate state. Enforce one merge authority per integration target. Revalidate locks and base revision immediately before write/integration.

## 7. Human gates

Pause only the affected work for:

- missing product or architecture decision;
- production/destructive authorization;
- credential/account setup that cannot be automated safely;
- legal/privacy/security approval;
- final visual/product judgment explicitly marked manual;
- accepted-risk sign-off.

Present the exact decision, options, evidence, impact, recommendation, and work that can continue independently.

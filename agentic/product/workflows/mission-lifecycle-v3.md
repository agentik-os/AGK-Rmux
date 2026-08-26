---
type: workflow
name: Mission lifecycle v3
workflow_type: ai-agent
status: Active
related:
  - feature:orchestration-v3
updated: 2026-07-30
---

# Workflow: Mission lifecycle v3

## Trigger

A user, scheduled task, integration, or upstream workflow submits a mission.

## Actors and roles

- **Operator:** owns intent, destructive consent, and final business authority.
- **Router:** classifies project, risk, mission shape, and required capabilities.
- **Orchestrator:** owns the mission contract, decomposition, dependency graph, and synthesis.
- **Executor:** performs bounded work within declared scope.
- **Validator:** independently tries to falsify terminal claims.
- **Runtime:** enforces scope, permissions, state transitions, budgets, and trace persistence.
- **Verifier:** executes only predeclared checks in a constrained environment and emits observations, never acceptance by itself.

## Inputs

- Verbatim user request.
- Active project fiche.
- Applicable instruction chain.
- Provider capability profile.
- Available skills and tools.
- Repository and runtime state.

## Steps

1. **Intake**
   - Actor: Router
   - Action: Preserve the verbatim request, resolve project identity, detect destructive or external side effects.
   - Output: Intake envelope.
   - On error: Ask only when project identity or authority cannot be resolved safely.

2. **Contract**
   - Actor: Orchestrator
   - Action: Enumerate deliverables, success metrics, constraints, exclusions, runtime checks, and evidence requirements.
   - Output: Mission contract and tracked plan.
   - On error: Record ambiguity and choose a reversible assumption, unless the consequence is unsafe.

3. **Capability resolution**
   - Actor: Runtime
   - Action: Select provider adapter, tools, skills, policy packs, and permissions.
   - Output: Minimal relevant capability bundle.
   - On error: Degrade to a supported path and record the missing capability.

4. **Decomposition**
   - Actor: Orchestrator
   - Action: Build a dependency graph, file ownership map, risk score, and delegation score.
   - Output: Executable graph with checkpoints.
   - On error: Collapse to serialized execution when ownership or dependencies are unclear.

5. **Execution**
   - Actor: Executor
   - Action: Perform one bounded task and emit progress, decisions, tool results, and artifact references.
   - Output: Candidate result tied to a task attempt, plan revision, and fencing token.
   - On error: Retry only transient, idempotent failures within budget. Otherwise escalate.

6. **Join and synthesis**
   - Actor: Orchestrator
   - Action: Wait for required dependencies, reject stale results, reconcile conflicts, and preserve dissent.
   - Output: Integrated candidate.
   - On error: Reopen only affected tasks.

7. **Verification**
   - Actor: Validator
   - Action: Run risk-proportional falsification against acceptance criteria and runtime behavior.
   - Output: Evidence observations with command, revision, timeout, exit code, redacted output hash, and verifier identity.
   - On error: Return actionable failures to the owning task with a bounded retry budget.

8. **Acceptance gate**
   - Actor: Runtime
   - Action: Confirm every required task attempt is accepted, the typed DAG is satisfied, gates and approvals pass, and no blocker remains.
   - Output: Accepted, correction-required, blocked, failed, or cancelled state.
   - On error: Refuse false completion.

9. **Delivery and learning**
   - Actor: Orchestrator
   - Action: Deliver the result on requested channels, record what changed, and update metrics or policy incident data.
   - Output: Operator-facing report and durable trace.

## Persistence semantics

- SQLite WAL is the sole write authority for missions, events, task attempts, fenced leases, and the transactional outbox.
- Materialized records, progress JSON, done JSON, harness plans, timelines, and Telegram cards are projections.
- Local transitions are exactly-once by idempotency key under expected-version CAS.
- External effects are duplicate-tolerant at-least-once and reconciled per destination.
- Legacy projections carry source sequence and provenance and are rejected by legacy ingress.

## Conditions

- **Simple and low-risk:** execute inline when delegation cost exceeds expected benefit.
- **Independent and context-heavy:** use native subagents.
- **Long-running or isolation-required:** use rmux workers or worktrees.
- **High-stakes or contested:** use an independent council before mutation.
- **Destructive:** require explicit operator consent before execution.
- **Provider capability missing:** use a tested adapter fallback, never emulate silently.

## Exceptions

- Worker crash: release or expire scope lease, preserve checkpoint, resume from last valid transition.
- Duplicate result: accept only the current mission generation and idempotency key.
- Stale state: migrate through a versioned state adapter or fail with a precise recovery instruction.
- Hook failure: apply the declared fail-open or fail-closed policy by risk class and emit an alert.
- Budget exhausted: preserve progress and escalate, never claim completion.

## Metrics

- Completion and verification rates.
- Time in each state.
- Coordination tax.
- Delegation decision accuracy.
- Retry and escalation counts.
- Context bytes loaded.
- Evidence coverage.
- Resume success.

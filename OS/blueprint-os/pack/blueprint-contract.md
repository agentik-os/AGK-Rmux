# Blueprint Contract

## Contents

1. Document header
2. Required section order
3. Record contracts
4. Traceability
5. Machine handoff
6. Applicability profiles

## 1. Document header

Every Blueprint begins with:

| Field | Contract |
| --- | --- |
| Project | Canonical name and namespace |
| Blueprint version | Semantic version |
| Run ID | Stable execution identifier |
| Mode | `NEW`, `RECOVER`, `EXTEND`, `REVISE`, `AUDIT`, or `DELTA` |
| Status | One allowed Blueprint status |
| Stage | Idea, discovery, definition, validation, existing product, migration |
| Authority | Decision owner(s), when known |
| Sources | Count and source-ledger pointer |
| Last updated | ISO 8601 timestamp when available |
| Output progress | Completed/current/remaining sections |
| Critical blockers | IDs only plus one-line explanation |

## 2. Required section order

Use this canonical order for a full pack. Preserve stable object IDs even when rendering sections in a different audience view.

### 00 — Run Manifest and Status

- request interpretation;
- invocation mode;
- scope of this run;
- target output and audience;
- available/missing context;
- current status and progress;
- completion definition.

### 01 — Executive Product Truth

- one-sentence product definition;
- target actor and struggle;
- promised transformation;
- core value loop;
- why it wins;
- business/value exchange;
- most important constraints;
- what it explicitly is not;
- stage and readiness.

### 02 — Source and Evidence Ledger

For every source: `SRC-ID`, title, type, authority, version/date, relevance, extracted evidence, reliability, conflicts, and access notes.

### 03 — Epistemic Ledgers

Separate tables for facts, decisions, assumptions, proposals, unknowns/deferred items, conflicts, and superseded items.

### 04 — Vocabulary and Concept Map

Define canonical terms, aliases, prohibited ambiguous terms, entity distinctions, and project boundary.

### 05 — Vision, Thesis, Principles, Non-goals

Define desired future, product thesis, strategic principles, non-goals, anti-features, and irreversible constraints.

### 06 — Market, Alternatives, and Positioning

Define category, alternatives, competitive forces, wedge, differentiation, reason to believe, and claims needing evidence. Browse or research only when permitted/required.

### 07 — Stakeholders, Personas, and JTBD

Include users, buyers, beneficiaries, operators, partners, administrators, regulators, and adversarial actors where relevant. For each JTBD include situation, motivation, desired progress, anxieties, habits, switching forces, and success evidence.

### 08 — Value Architecture and Business Model

Define value events, value exchanges, monetization, pricing/entitlements, incentives, unit-economics variables, service/operational burden, integrity constraints, and failure economics.

### 09 — Goals, Metrics, Guardrails, Counter-metrics

Every metric includes formula, population, window, source, owner, target/hypothesis, guardrail, failure mode, and linked actions/events.

### 10 — Scope, Release Boundaries, and Capability Map

Define foundations, validation slice, core, expansion, future, exclusions, dependencies, evidence gates, and capability contracts.

### 11 — Actor, Identity, Role, Permission, and Consent Model

Define identity lifecycle, roles/attributes, access matrix, ownership, delegation, consent/revocation, admin access, service accounts, and audit.

### 12 — Requirement Catalog

Group by business/user/functional/domain/data/interface/AI/security/privacy/NFR/operations. Requirements remain atomic and testable.

### 13 — Action Contract Catalog

Define all consequential user, admin, partner, integration, and agent actions using the Action Contract.

### 14 — End-to-end User and Operator Flows

Include discovery/admission, onboarding, first value, recurring value, monetization, recovery, support, offboarding, export/deletion, moderation/admin, and incident paths as applicable.

### 15 — Information Architecture and Navigation

Define objects, hierarchy, taxonomy, navigation, routes, deep links, search, commands, notifications, settings, and platform variations.

### 16 — Screen and Surface Contracts

Define complete surface inventory and each screen contract, including non-happy states and accessibility.

### 17 — Design System and Content Rules

Define design principles, tokens, layout, typography, components, interaction grammar, motion, copy voice, localization, accessibility, and prohibited patterns.

### 18 — Domain Model and Bounded Contexts

Define contexts, entities, value objects, aggregates, services, policies, ownership, and relationships.

### 19 — State Machines, Rules, and Invariants

Define transition tables, temporal rules, pricing/eligibility/entitlement rules, idempotency, concurrency, expiry, reversals, and prohibited states.

### 20 — System Context and Trust Boundaries

Define external actors/systems, trust zones, data movement, sensitive boundaries, and system responsibilities.

### 21 — Application and Deployment Architecture

Define modules/services, clients, workers, jobs, queues, sync/async paths, storage, caching, media, search, real-time, deployments, environments, configuration, and evolution.

### 22 — Architecture Decision Records

Each ADR includes context, decision, drivers, considered alternatives, consequences, risks, reversibility, validation, and supersession.

### 23 — Data Dictionary, Ownership, and Lifecycle

Define schemas, sensitivity, tenancy, source of truth, provenance, retention, deletion, export, encryption, indexes, consistency, migrations, and reconciliation.

### 24 — API, Tool, Integration, and Event Contracts

Define schemas, auth, errors, versioning, idempotency, rate limits, delivery, retries, replay, observability, provider failure, and compatibility.

### 25 — AI and Agent Architecture

If applicable: responsibilities, prompts, models, routing, context, retrieval, memory, tools, permissions, autonomy, confirmation, provenance, evaluation, monitoring, fallback, cost, latency, and safety.

### 26 — Security, Privacy, Threat, and Abuse Model

Define assets, threats, abuse cases, mitigations, consent, PII, isolation, audit, incident response, moderation/appeals, and verification.

### 27 — Non-functional Requirements

Define measurable reliability, latency, scale, durability, recovery, accessibility, compatibility, localization, maintainability, observability, and cost targets.

### 28 — Operational Model

Define people/process dependencies, support, moderation, partner/content/data operations, alerts, escalation, runbooks, backup/restore, migration, incidents, and business continuity.

### 29 — Acceptance and Test Architecture

Define journey, contract, domain, permission, accessibility, resilience, migration, rollback, AI-evaluation, security, and operational-exercise coverage.

### 30 — Analytics, Instrumentation, and Learning Loops

Map events to metrics, dashboards, experiments, qualitative feedback, review cadence, decision thresholds, and counter-metrics.

### 31 — Risk Register

Score probability, impact, detectability, velocity, affected goals, mitigation, contingency, owner, trigger, and residual risk.

### 32 — Release Definition and Validation Strategy

Define what must be validated manually or with prototypes, capability-level increments, evidence gates, rollout/rollback, migrations, and dependencies. Do not emit atomic DEV steps.

### 33 — Traceability Matrix and Orphan Report

Provide bidirectional coverage and all orphan categories.

### 34 — Critic Findings and Dispositions

List challenge, severity, evidence, affected IDs, disposition, and follow-up.

### 35 — Quality Gate Scorecard

Report all mandatory gates, evidence, failures, conditions, and owners.

### 36 — Stepper Input Manifest

Provide:

- blueprint version and checksum;
- accepted capabilities and release groups;
- dependency graph;
- canonical requirements;
- architecture/data/API/event/AI contracts;
- acceptance/test contracts;
- constraints and prohibited shortcuts;
- risks and required validation spikes;
- environment/integration assumptions;
- deferred items and conditions;
- critical gates;
- artifact locations.

### 37 — Continuation and Change Ledger

Include progress checkpoint for incomplete outputs or semantic change report for revised Blueprints.

### 38 — Final Declaration

State exclusions, residual unknowns, conditional items, and exactly one allowed status.

## 3. Record contracts

### Decision

```yaml
id: DEC-001
statement: "..."
status: accepted
authority: "..."
rationale: "..."
sources: [SRC-001]
alternatives: ["..."]
dependencies: [REQ-001]
affects: [FLOW-001, ENT-001]
supersedes: []
validation: "..."
```

### Requirement

```yaml
id: REQ-001
statement: "The system MUST ..."
type: functional
status: accepted
priority: critical
release: core
rationale: "..."
sources: [SRC-001]
decisions: [DEC-001]
actors: [PER-001]
preconditions: ["..."]
dependencies: []
affects: [ACT-001, FLOW-001, SCR-001, ENT-001, API-001]
acceptance: [AC-001]
verification: contract-test
uncertainty: null
```

### Action

```yaml
id: ACT-001
actor: PER-001
intent: "..."
trigger: "..."
entry_points: [SCR-001]
preconditions: ["..."]
inputs: ["..."]
authorization: RULE-001
happy_path: ["..."]
alternate_paths: ["..."]
failures: ["..."]
mutations: ["..."]
events: [EVT-001]
external_effects: []
confirmation: "..."
receipt: "..."
undo_or_compensation: "..."
analytics: [EVT-002]
acceptance: [AC-002]
```

### Screen

```yaml
id: SCR-001
purpose: "..."
actors: [PER-001]
route: "/..."
entries: [FLOW-001]
exits: [SCR-002]
data: [ENT-001]
components: [CMP-001]
actions: [ACT-001]
states: [default, loading, empty, error, offline, denied, stale, success]
permissions: [RULE-001]
responsive: "..."
accessibility: "..."
analytics: [EVT-002]
requirements: [REQ-001]
acceptance: [AC-003]
```

### Entity

```yaml
id: ENT-001
name: "..."
bounded_context: "..."
owner: "..."
identity: "..."
tenant_boundary: "..."
fields: []
relations: []
states: []
invariants: [INV-001]
sensitivity: confidential
retention: "..."
deletion: "..."
export: "..."
indexes: []
migrations: "..."
```

### AI capability

```yaml
id: AIC-001
outcome: "..."
responsibility: "..."
forbidden: ["..."]
trigger: "..."
autonomy_tier: assist
input_schema: "..."
output_schema: "..."
context_sources: ["..."]
memory_policy: MEM-001
tools: [API-001]
confirmation: "..."
confidence_and_abstention: "..."
provenance: "..."
fallback: "..."
cost_budget: "..."
latency_budget: "..."
evals: [EVAL-001]
monitoring: "..."
rollback: "..."
```

### Risk

```yaml
id: RSK-001
statement: "..."
category: product
probability: medium
impact: high
detectability: medium
velocity: fast
affected: [GOAL-001, REQ-001]
mitigation: "..."
contingency: "..."
trigger: "..."
owner: "..."
residual_risk: medium
verification: [TST-001]
```

## 4. Traceability

Minimum edge types:

- `derived_from`
- `decides`
- `satisfies`
- `depends_on`
- `conflicts_with`
- `supersedes`
- `realized_by`
- `reads`
- `writes`
- `emits`
- `consumes`
- `authorized_by`
- `verified_by`
- `measured_by`
- `mitigated_by`
- `blocks`

Each edge is directional and includes source ID, target ID, relation, rationale, and status.

## 5. Machine handoff

The machine-readable handoff should expose:

```json
{
  "project": {},
  "version": {},
  "status": "BLUEPRINT COMPLETE — STEPPER READY",
  "sources": [],
  "epistemic_records": [],
  "goals": [],
  "metrics": [],
  "capabilities": [],
  "requirements": [],
  "actors": [],
  "actions": [],
  "flows": [],
  "screens": [],
  "domain": {},
  "architecture": {},
  "data": [],
  "apis": [],
  "events": [],
  "ai": {},
  "security": {},
  "nfrs": [],
  "operations": [],
  "acceptance": [],
  "risks": [],
  "trace_links": [],
  "gates": [],
  "continuation": {},
  "stepper_manifest": {}
}
```

## 6. Applicability profiles

Use the full artifact set by default, then tailor depth:

| Profile | Special emphasis |
| --- | --- |
| Consumer app | activation, retention, cross-platform UX, privacy, abuse |
| B2B SaaS | tenancy, roles, workflows, integration, billing, audit, admin |
| Marketplace | supply/demand, liquidity, incentives, trust, dispute, payments |
| AI agent | context, memory, tools, permissions, confirmation, evals, trace |
| Internal tool | workflow fit, change management, permissions, audit, ROI |
| Regulated product | evidence, data lifecycle, controls, human review, compliance |
| Hardware-connected | identity, pairing, firmware, offline, safety, recovery |
| API/platform | contracts, versioning, quotas, developer experience, abuse |

Do not remove universal quality gates because the product appears small. Mark non-applicable items with rationale.

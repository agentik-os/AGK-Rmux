# Design Definition Pack and Stepper handoff

## Contents

1. Pack structure
2. Stable IDs
3. Human-readable contracts
4. Machine handoff
5. Traceability
6. Stepper work-unit seeds
7. Readiness rules

## 1. Pack structure

Emit one coherent versioned pack. A practical multi-file structure is:

```text
design-os/
  00-status-and-index.md
  01-evidence-and-conflicts.md
  02-experience-principles.md
  03-flow-challenge.md
  04-information-architecture.md
  05-journeys-and-state-machines.md
  06-surface-contracts.md
  07-interaction-system.md
  08-visual-and-component-system.md
  09-responsive-accessibility-evals.md
  10-stepper-handoff.md
  design-handoff.json
```

For a small product, combine Markdown sections without omitting contracts. Always emit `design-handoff.json` for a `STEPPER_READY` pack.

`00-status-and-index.md` must state:

- project and upstream Blueprint version;
- Design OS version/revision/date;
- active mode and target surfaces;
- readiness status;
- passed/failed/conditional gates;
- completed versus remaining sections;
- critical blockers and decision owners;
- exact next section if incomplete.

## 2. Stable IDs

Use prefixes consistently:

| Prefix | Entity |
| --- | --- |
| `EXP-###` | Experience principle |
| `FLOW-###` | User/system flow |
| `IA-###` | IA or navigation model |
| `SURF-###` | Screen, panel, drawer, overlay, route, artifact, or chat surface |
| `STATE-###` | State machine |
| `INT-###` | Interaction behavior |
| `TOK-###` | Token family |
| `COMP-###` | Component/composition |
| `A11Y-###` | Accessibility behavior |
| `EVAL-###` | Test/evaluation case |
| `RISK-###` | Design risk |
| `DDEC-###` | Design decision record |
| `UNK-###` | Unknown/conflict requiring resolution |
| `SEED-###` | Stepper work-unit seed |

Reference upstream requirement/decision IDs without changing them. IDs are immutable. Retire with status/reason; never recycle.

## 3. Human-readable contracts

### Experience principle

```text
EXP-### — Name
Rule: observable behavior
Evidence: source/requirement IDs
Applies to: FLOW/SURF/COMP IDs
Rejects: named anti-pattern
Eval: EVAL IDs
```

### Flow

```text
FLOW-### — Name
Priority: P0 | P1 | P2
Actors and outcome
Requirements and decisions
Trigger, preconditions, permissions
Happy/alternate/recovery graphs
Async, resume, cancellation, undo/compensation
Success and guardrail metrics
SURF/STATE/INT/COMP/A11Y/EVAL links
```

### Surface

```text
SURF-### — Name
Kind: route | page | panel | drawer | dialog | popover | canvas | artifact | chat
Purpose and primary user question
Actors/permissions/plans
URL/panel target and state owner
Entry and exit paths
Regions and information hierarchy
Actions by priority and placement
Data/content dependencies
State matrix and state-machine refs
Responsive host transformations
Keyboard/touch/focus/announcements
Privacy/telemetry
Tokens/components
Acceptance/evals
```

### Component

```text
COMP-### — Name
Kind: primitive | composition | navigation | data-shape
Source: shadcn/base-ui | shadcn/radix | stax | custom
Registry address/ref
Purpose and prohibited misuse
API/variants/slots
States and transitions
Tokens
A11Y contract
Flow/surface usage
Tests and visual regression
Owner
```

### State machine

```text
STATE-### — Name
Initial and terminal states
State dictionary: meaning + visible rendering
Events and guards
Transition table
Persistence/reconnect/idempotency
Focus/announcement behavior
Error/retry/cancel rules
EVAL links
```

### Design decision

Use the `DDEC-###` form from `SKILL.md`. Include rejected options so future revisions do not repeat settled debates.

## 4. Machine handoff

`design-handoff.json` conforms to [design-handoff.schema.json](design-handoff.schema.json). Required top-level keys:

```json
{
  "schemaVersion": "1.0",
  "designOsVersion": "1.0",
  "project": {},
  "readiness": {},
  "sources": [],
  "principles": [],
  "decisions": [],
  "flows": [],
  "ia": [],
  "surfaces": [],
  "stateMachines": [],
  "interactions": [],
  "tokens": [],
  "components": [],
  "accessibility": [],
  "evals": [],
  "traceability": [],
  "risks": [],
  "unknowns": [],
  "stepperSeeds": []
}
```

Use arrays of refs rather than duplicating bodies. Keep human descriptions concise and testable. Put diagrams in Markdown; put nodes/edges/states in JSON.

### Required readiness gates

Use these IDs exactly:

`G-BP`, `G-FLOW`, `G-IA`, `G-STATE`, `G-ACTION`, `G-AI`, `G-DS`, `G-RWD`, `G-A11Y`, `G-TRACE`, `G-EVAL`, `G-HANDOFF`.

Each gate includes `status`, `evidenceRefs`, and optional `notes`. Use `not_applicable` only with a reason.

### Graph representation

Flows use nodes and edges:

```json
{
  "nodes": [
    { "id": "n1", "kind": "user_action", "label": "Submit request", "surfaceRef": "SURF-001" },
    { "id": "n2", "kind": "system_state", "label": "Queued", "stateRef": "STATE-001" }
  ],
  "edges": [
    { "from": "n1", "to": "n2", "event": "submit", "condition": "valid" }
  ]
}
```

Do not embed implementation tasks in graph nodes.

## 5. Traceability

Each row maps one upstream requirement to downstream design evidence:

| Field | Meaning |
| --- | --- |
| `requirementRef` | Upstream ID |
| `criticality` | `critical`, `important`, `supporting` |
| `flowRefs` | User/system flows |
| `surfaceRefs` | Visible surfaces |
| `stateRefs` | State machines |
| `interactionRefs` | Behavior contracts |
| `componentRefs` | System components |
| `a11yRefs` | Accessibility behavior |
| `evalRefs` | Acceptance proof |
| `coverage` | `complete`, `partial`, `blocked`, `not_applicable` |
| `notes` | Gap, owner, or rationale |

Critical rows must be `complete` for `STEPPER_READY`. A requirement with no UI can still trace to a state/interaction/eval or be `not_applicable` with reason.

Also support reverse impact queries: every `SURF`, `COMP`, and `EVAL` should reference at least one flow or requirement unless it is a foundation component with declared system scope.

## 6. Stepper work-unit seeds

A `SEED-###` is not a development task. It is a bounded implementation concern with dependencies and design acceptance.

```text
SEED-### — Name
Slice: foundation | vertical-flow | hardening | migration
Outcome: user/system behavior made possible
Design refs: FLOW/SURF/STATE/INT/COMP/A11Y/EVAL
Upstream refs: requirements/decisions
Depends on: SEED IDs
Must precede: SEED IDs
Risk: low | medium | high | critical
Suggested verification: deterministic, integration, e2e, visual, manual
Boundary: explicitly excluded work
```

Seed order:

1. data/state/event contracts required by multiple flows;
2. navigation/shell/focus/command registries;
3. tokens/primitives/core compositions;
4. P0 vertical slices end to end;
5. alternate and failure states;
6. responsive/accessibility/keyboard hardening;
7. analytics, evals, and visual drift gates;
8. P1/P2 flows after validated P0 foundations.

Do not seed “build all screens,” “add accessibility,” or “make responsive.” Tie these to specific contracts and proof.

## 7. Readiness rules

Set `STEPPER_READY` only when:

- all required gates are `pass` or justified `not_applicable`;
- every critical traceability row is complete;
- every P0 flow has graph, surface, state, recovery, and eval refs;
- every referenced ID exists and prefixes match entity type;
- no critical open risk/unknown remains;
- stepper seeds form an acyclic dependency graph;
- JSON validates with `scripts/validate_design_handoff.py`.

If a human approval remains but is non-critical, use `CONDITIONAL` and keep readiness below `STEPPER_READY` unless the upstream governance explicitly allows conditional handoff.


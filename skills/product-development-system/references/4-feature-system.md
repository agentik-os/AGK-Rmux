# 4. Feature System

The central object between strategy and development. One file per feature at
`agentic/product/features/<slug>.md`.

## Feature structure (front-matter fields)
```
Feature
├── Name
├── Summary
├── User problem
├── Target persona
├── User story
├── Business objective
├── Expected outcome
├── Scope
├── Out of scope
├── Requirements
├── Acceptance criteria
├── Edge cases
├── Dependencies
├── Risks
├── Metrics
├── Design
├── Technical spec
├── Tasks
├── Tests
└── Release
```

## Feature types
Core · Supporting · Experimental · Premium · Internal · Platform · Integration ·
Automation · AI feature · Compliance · Infrastructure.

## Feature lifecycle
```
Idea -> Opportunity -> Discovery -> Validation -> Specification -> Design -> Technical Review
     -> Planned -> In Development -> Code Review -> QA -> Beta -> Released -> Measured -> Improved
```
Gates (do not advance without them):
- to **Specification**: passed Discovery (ref 5) with acceptable confidence.
- to **Planned**: has a priority score (ref 6) AND acceptance criteria AND a success metric.
- to **In Development**: acceptance criteria are the workers' Done Criteria (R-RUBRIC).
- to **Released**: verified against runtime (L1 / R-PROD), not a green build.
- to **Measured/Improved**: the success metric was actually read.

## Feature Canvas (fill before Specification)
- **Problem** — which problem is solved?
- **User** — for which type of user?
- **Context** — when does the problem appear?
- **Current Alternative** — how does the user solve it today?
- **Proposed Solution** — what is the solution?
- **Value** — what value is created?
- **Success Metric** — how do we verify it works?
- **Risks** — what can fail?

## How the agent uses it
- This is the object a "work a feature" mission produces and maintains. Scope AND out-of-scope are
  both mandatory — silent scope creep is a defect.
- `Tasks` is the decomposition dispatched to workers (R-ORCH); `Acceptance criteria` + `Tests`
  define done; `Metrics` is checked post-release.
- Link upstream (`opportunity`, `vision pillar`) and downstream (`workflow`).

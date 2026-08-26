# Design OS, v1.0.0

**Category:** Product Stack / UX, Interaction and Visual Design Compilation  
**Omega position:** Product Stack: UX/interaction/visual design compilation, second stage of the IMPLEMENT branch (`Blueprint -> Design -> Stepper -> Builder`)  
**Primary interface:** conversational + machine-readable handoff  
**Status:** installable reference implementation

## Purpose
Compile an approved product Blueprint into a challenged, coherent, modern UX/UI definition and a machine-readable Design Handoff for Stepper. Design OS acts as a product-design compiler and adversarial user-flow challenger: it transforms product truth into behavior, structure, surfaces, states, and testable design contracts, then hands Stepper a resolved design graph rather than inspirational prose.

## Promise
Own how people understand, navigate, act, recover, and trust the product, while preserving product intent and challenging the proposed interface. Every critical requirement is traced to a flow, a surface, a state, a component contract, and an acceptance test before anything is labelled `STEPPER_READY`.

## Position in the value chain

```text
Idea/context -> Blueprint {OS} -> Design {OS} -> Stepper {OS} -> Builder {OS}
```

Blueprint is the contract for what and why. Design OS owns the how (behavior, structure, surfaces, states, testable contracts) and stops before production implementation unless a non-production prototype is explicitly requested.

## Operating loop

```text
RECOVER BLUEPRINT -> CHALLENGE THESIS -> DERIVE IA/NAV -> COMPILE JOURNEYS/STATES -> DEFINE INTERACTION -> DEFINE VISUAL SYSTEM -> COMPILE SURFACES/COMPONENTS -> PROTOTYPE/VALIDATE -> EMIT STEPPER HANDOFF
```

## What this OS contains
- Canonical compiler contract with twelve governing laws and a decision protocol (`SKILL.md`)
- One adversarial design-compiler skill run in nine passes
- 10 reference protocols: compiler workflow and gates, Stepper output contract, flow-challenge protocol, chat/agent interaction system, AI product intelligence, STAX and shadcn architecture, modern visual-system protocol, responsive and accessibility contract, design validation and evals, and a paste-ready master system prompt
- 2 JSON schemas: `blueprint-intake.schema.json` and `design-handoff.schema.json`
- 3 Python validators and a smoke test: `validate_blueprint_intake.py`, `validate_design_handoff.py`, `self_test.py`
- 1 interface descriptor (`agents/openai.yaml`) and 1 icon asset (`assets/icon.svg`)
- Note: this pack ships as a single compiler skill plus reference protocols and validators. It does NOT carry the separate `skills/`, `protocols/`, `knowledge/`, `memory/`, `database/`, or `evals/` directories that some richer OS packs use: the reference protocols live under `references/` and the eval material is `references/validation-evals.md`.

## Commands
The pack exposes one default command; the depth of the run is selected by mode.

| Command | Mode | Purpose |
| --- | --- | --- |
| `/design` | dispatch | Open Design OS and compile a Blueprint into a validated UX/UI handoff |

Operating modes (stated at the start of a run):

| Mode | Purpose |
| --- | --- |
| `FULL` | Run every pass and emit the complete Design Definition Pack (default) |
| `AUDIT` | Challenge an existing design or codebase and emit gaps plus a repair handoff |
| `FLOW` | Focus on selected journeys while retaining traceability and edge-state gates |
| `AI_APP` | Prioritize composer, context, agent-state, tool, artifact, source, and memory behavior |
| `STAX_FIT` | Decide whether, where, and how to use STAX |
| `REVISION` | Update impacted IDs and contracts without rewriting unaffected sections |

## Main handoffs
- Blueprint OS supplies product and system truth (what and why); Design OS consumes `blueprint.completed`.
- Stepper OS receives the machine-readable Design Handoff (`design-handoff.json`); Design OS produces `design.handoff.completed`.
- Context & Memory OS stores the canonical design handoff: Design OS reads `memory.context.compiled`, writes `memory.record.staged`, and receives `memory.record.verified` in return.
- Review & Governance OS approves changes to boundaries, schemas, or quality gates in production.

## Installation
See `OMEGA_INTEGRATION.md` for registration (ID `design`, default command `/design`), context-injection order, event wiring, and change control.

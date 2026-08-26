---
name: design-os
description: Compile an approved product Blueprint into a challenged, coherent, modern UX/UI definition and a machine-readable Design Handoff for Stepper. Use after Blueprint OS and before roadmap, implementation, or Builder work for apps, SaaS, mobile products, dashboards, AI/chat products, internal tools, marketplaces, websites with application flows, redesigns, and design-system migrations. Trigger for Design OS, UX architecture, user-flow challenge, information architecture, screen contracts, interaction design, shadcn/ui or STAX mapping, visual system definition, responsive/accessibility states, prototype specification, or pre-Stepper design validation.
---

# Design {OS}

Act as a product-design compiler and adversarial user-flow challenger. Transform product truth into behavior, structure, surfaces, states, and testable design contracts. Hand Stepper a resolved design graph; do not hand it inspirational prose.

## Position in the operating chain

Use this order:

`Idea/context -> Blueprint {OS} -> Design {OS} -> Stepper {OS} -> Builder {OS}`

Treat Blueprint as the contract for what and why. Own how people understand, navigate, act, recover, and trust the product. Stop before production implementation unless the user explicitly asks for a non-production prototype.

## Governing laws

1. Preserve product intent; challenge the proposed interface.
2. Start from user goals and data relationships, never from a gallery of screens.
3. Make context, system state, permissions, cost, progress, and consequences visible.
4. Prefer reversible actions and local undo over confirmation dialogs; require confirmation for consequential external writes.
5. Give every asynchronous state a named, persistent rendering.
6. Model flows as graphs and conversations as trees when branching exists.
7. Use one source of truth for navigation, selection, commands, tokens, and component metadata.
8. Make keyboard, pointer, touch, screen reader, zoom, and reduced-motion paths first-class.
9. Use shadcn/ui as editable open code and a distribution contract, not as a generic visual identity.
10. Use STAX only when its contextual panel model wins the product-specific navigation test.
11. Trace every critical requirement to a flow, surface, state, component contract, and acceptance test.
12. Never label unresolved work `DESIGN READY` or `STEPPER READY`.

## Read the relevant references

Always read:

- [workflow-gates.md](references/workflow-gates.md) for the compiler phases and release gates.
- [output-contract.md](references/output-contract.md) for IDs, documents, and the Stepper handoff.
- [flow-challenge.md](references/flow-challenge.md) for friction analysis and adversarial review.

Read when the product contains chat, agents, generated artifacts, model/tool selection, or AI memory:

- [interaction-system.md](references/interaction-system.md) for chat/application behavior.
- [ai-intelligence.md](references/ai-intelligence.md) for deterministic versus model decisions and visible AI states.

Read when selecting navigation, components, or design-system architecture:

- [stax-shadcn.md](references/stax-shadcn.md) for the STAX fitness test and shadcn mapping.
- [visual-system.md](references/visual-system.md) for modern visual language, tokens, density, and motion.

Read before final validation:

- [responsive-accessibility.md](references/responsive-accessibility.md).
- [validation-evals.md](references/validation-evals.md).

Read [master-system-prompt.md](references/master-system-prompt.md) only when the user asks for a paste-ready prompt for another coding/design agent.

## Intake and evidence ledger

Accept a full Blueprint pack, a named local/Library artifact, repository documentation, or pasted specification. Recover these inputs before designing:

- product thesis, outcomes, non-goals, business model, risks;
- actors, personas, jobs, permissions, plans, roles, trust boundaries;
- requirement IDs, feature/action contracts, domain objects, invariants;
- core journeys, AI behavior, data/API/events, NFRs, target surfaces;
- brand constraints, existing components, codebase, analytics, research;
- known decisions, assumptions, proposals, unknowns, and rejected ideas.

Maintain a fact ledger with labels:

- `FACT`: source-backed current truth;
- `DECISION`: approved choice;
- `ASSUMPTION`: reversible working belief;
- `PROPOSAL`: Design OS recommendation;
- `UNKNOWN`: unresolved fact;
- `CONFLICT`: incompatible inputs;
- `REJECTED`: considered and excluded, with reason.

Never silently upgrade an assumption into a decision. Ask one compact clarification set only when the answer changes navigation, safety, business logic, or the critical path. Otherwise choose the most probable reversible assumption and expose it.

## Compile in nine passes

### 1. Recover and normalize Blueprint

Create a coverage map from requirements and actors to user outcomes. Identify missing action semantics, permissions, edge cases, data states, or cross-surface constraints. Return to Blueprint only when product truth is missing; do not invent business policy to finish a screen.

### 2. Challenge the experience thesis

Write the product's interaction thesis in one sentence. Name the primary user question for every major surface. Score each core flow for value, frequency, risk, urgency, and reversibility. Delete, merge, defer, or demote flows that do not earn their cost.

### 3. Derive information and navigation architecture

Model entities, collections, tasks, global utilities, and relationships. Select the shell per surface: page/route, hub-and-drill, STAX panel rail, split view, canvas, chat-first, focused editor, or a justified hybrid. Record rejected shells and tradeoffs.

### 4. Compile journeys and state machines

Specify happy, alternate, recovery, permission, empty, loading, stale, offline, conflict, and destructive paths. Define entry/exit conditions, actors, preconditions, state transitions, events, system responses, undo/compensation, and success signals.

### 5. Define the interaction system

Create command, menu, shortcut, selection, drag/drop, paste, focus, notification, and progressive-disclosure contracts. For AI products, specify composer context, thinking/tool/source rendering, branching, streaming, stop/retry/reconnect, artifacts, memory transparency, and write confirmation.

### 6. Define the visual system

Select a product-specific direction from evidence. Produce semantic tokens, typography roles, spacing/density, radii, borders, elevation, icon rules, motion, data visualization, light/dark/high-contrast behavior, and do/don't examples. Distinguish inspiration from imitation.

### 7. Compile surfaces and components

Give every surface a contract: purpose, user question, entry points, layout regions, hierarchy, actions, content, data dependencies, states, permissions, responsive transformations, keyboard/touch behavior, analytics, and acceptance criteria. Map primitives to shadcn/Base UI or Radix and STAX where selected. Define custom components only after proving no existing primitive fits.

### 8. Prototype and validate

Use low-fidelity flow proof before high-fidelity styling. Validate the riskiest transition, not the prettiest screen. If creating a prototype, label it non-production and preserve the approved contracts. Run heuristic, accessibility, responsive, content, adversarial, and traceability gates.

### 9. Emit Stepper handoff

Generate the human-readable Design Definition Pack and `design-handoff.json`. Run:

```bash
python3 scripts/validate_design_handoff.py path/to/design-handoff.json
```

Set `readiness.status` to `STEPPER_READY` only after all blocking gates pass and no critical unknown remains.

## Required output

Produce, in order:

1. Executive design verdict and readiness.
2. Evidence ledger and unresolved conflicts.
3. Experience principles and rejected anti-principles.
4. Actor/job/flow priority map.
5. Flow challenge report with before/after paths.
6. Information architecture and navigation decision record.
7. Critical journey graphs and state machines.
8. Surface inventory and detailed screen/surface contracts.
9. Interaction contracts, including AI behavior where relevant.
10. Design system, tokens, component registry, and STAX/shadcn mapping.
11. Responsive, accessibility, localization, privacy, and trust contracts.
12. Prototype/testing plan and eval cases.
13. Requirement-to-design traceability matrix.
14. Risks, debt, open questions, and change log.
15. Stepper work-unit seeds and `design-handoff.json`.

Use stable IDs:

- `EXP-###` experience principle;
- `FLOW-###` flow;
- `IA-###` navigation/IA decision;
- `SURF-###` surface;
- `STATE-###` state machine;
- `INT-###` interaction contract;
- `TOK-###` token family;
- `COMP-###` component;
- `A11Y-###` accessibility contract;
- `EVAL-###` design eval;
- `RISK-###` design risk;
- `DDEC-###` design decision;
- `UNK-###` unresolved unknown.

Do not renumber existing IDs across revisions. Mark retired IDs and preserve their reasons.

## Decision protocol

For every material decision record:

```text
DDEC-### — Decision title
Status: proposed | approved | superseded | rejected
Problem: what must be resolved
Evidence: Blueprint/research/analytics IDs
Options: viable alternatives
Decision: chosen behavior or structure
Why: user and system rationale
Tradeoffs: what becomes worse or more expensive
Consequences: affected flows/surfaces/components/tests
Reversal trigger: evidence that would reopen the decision
Owner: human decision owner when required
```

Prefer a small comparison table when mapping more than two options. Use diagrams only for topology, branching, or event order.

## Quality gates

Block readiness when any of these is true:

- a critical Blueprint requirement has no design coverage;
- a critical flow has no failure, permission, latency, or recovery behavior;
- navigation state has multiple competing sources of truth;
- a destructive or external write lacks consequence/undo/confirmation policy;
- AI processing lacks explicit visible states or reconnection behavior;
- a surface has no empty/loading/error state;
- mobile is a shrunken desktop instead of a transformed host;
- keyboard or focus restoration is unspecified;
- component/token choices are visual guesses without semantics;
- accessibility criticals or contrast/zoom/reflow gates fail;
- `design-handoff.json` fails validation;
- a critical `UNKNOWN` or `CONFLICT` remains ownerless.

## Operating modes

- `FULL`: run every pass and emit the complete pack.
- `AUDIT`: challenge an existing design/codebase and emit gaps plus a repair handoff.
- `FLOW`: focus on selected journeys but retain traceability and edge-state gates.
- `AI_APP`: prioritize composer, context, agent-state, tool, artifact, source, and memory behavior.
- `STAX_FIT`: decide whether, where, and how to use STAX; do not design the full product unless asked.
- `REVISION`: update impacted IDs and contracts without rewriting unaffected sections.

Default to `FULL` for “Design OS” requests. State the active mode and completion progress. If output limits force a split, mark the pack `INCOMPLETE`, list finished and remaining sections, preserve IDs, and continue from the next exact section. Never present a partial pack as Stepper-ready.

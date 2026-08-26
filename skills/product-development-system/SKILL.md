---
name: product-development-system
description: >
  The OmegaOS way of doing product work. Use this WHENEVER a mission touches a product decision:
  defining a vision, brainstorming ideas, framing an opportunity, proposing or scoping a feature,
  running discovery, prioritizing a backlog, or modelling a workflow/process. It is the operating
  method every oracle and worker follows so we never jump from "we have an idea" straight to
  "let's build it". The chain is always: Outcome -> Opportunity -> Idea -> Feature (Discovery ->
  Prioritization -> Specification) -> Workflow -> Build -> Measure. Seven sub-systems, each with
  its own objects, fields, statuses, and relations: Vision Board, Brainstorming System, Opportunity
  Board, Feature System, Feature Discovery, Feature Prioritization, Workflow Builder. Triggers (EN):
  "new feature", "should we build", "product vision", "roadmap", "brainstorm ideas", "prioritize",
  "RICE/ICE", "opportunity", "user problem", "spec this feature", "discovery", "acceptance criteria",
  "map the workflow". Triggers (FR): "nouvelle feature", "faut-il construire", "vision produit",
  "roadmap", "brainstormer", "prioriser", "opportunite", "probleme utilisateur", "specifier la
  feature", "discovery", "criteres d'acceptation", "modeliser le workflow", "on travaille une feature".
allowed-tools: ["Read", "Write", "Edit", "Glob", "Grep", "Bash"]
metadata:
  source: omegaos
  version: "1.0"
---

# Product Development System

This is how OmegaOS works product. It binds every oracle and worker: when a mission is about
a feature, an idea, a roadmap, a priority call, or a process, you run it through this system
instead of improvising. The point is disciplined product thinking, evidence before build, and a
single object model everyone shares.

## The one rule that governs everything

**Never go `idea -> build`.** Go:

```
Business Outcome  ->  Opportunity  ->  Idea (brainstorm)  ->  Feature
                                                               |
                                    Discovery  ->  Prioritization  ->  Specification
                                                               |
                                                            Workflow  ->  Build  ->  Measure  ->  Improve
```

Every step upstream of "Build" exists to kill weak ideas cheaply and to make the surviving ones
sharp. If you are asked to "just build X", your first job is to place X on this chain and fill in
the missing upstream objects (at minimum: what outcome, what opportunity, what evidence, what
acceptance criteria) before writing code.

## The seven sub-systems (read the reference for the full object model)

| # | System | What it answers | Reference |
|---|--------|-----------------|-----------|
| 1 | **Vision Board** | What must this product become, and why | `references/1-vision-board.md` |
| 2 | **Brainstorming System** | What could we do (generate, cluster, qualify ideas) | `references/2-brainstorming.md` |
| 3 | **Opportunity Board** | Which user problems are worth solving | `references/3-opportunity-board.md` |
| 4 | **Feature System** | The central object between strategy and code | `references/4-feature-system.md` |
| 5 | **Feature Discovery** | Is this real, wanted, feasible, on-vision | `references/5-feature-discovery.md` |
| 6 | **Feature Prioritization** | What do we do next, and why (RICE/ICE/weighted) | `references/6-prioritization.md` |
| 7 | **Workflow Builder** | How the process/journey/automation actually runs | `references/7-workflow-builder.md` |

Read the reference file for the system your mission touches BEFORE producing an artifact. Do not
paraphrase the object model from memory; the fields, statuses, and formulas are exact.

## How OmegaOS agents operate it (persistence + statuses)

These are not abstract concepts. During a mission you create and update real files under the
project's `agentic/product/` tree (per the OmegaOS file convention: agent outputs live in
`agentic/`). One markdown file per object, front-matter carrying the structured fields.

```
agentic/product/
├── vision.md                     # the Vision Board (one per product)
├── north-star.md                 # North Star + input/output/leading/lagging metrics
├── ideas/<slug>.md               # Brainstorming System — one file per idea
├── opportunities/<slug>.md       # Opportunity Board — one file per opportunity
├── features/<slug>.md            # Feature System — one file per feature (the central object)
└── workflows/<slug>.md           # Workflow Builder — one file per modelled process
```

Every object carries a `status` in its front-matter and moves through its own lifecycle (defined
in each reference). An oracle NEVER marks an object further along than the evidence supports
(mirrors L1 / R-VERIFY): an idea is not `Validated` without evidence; a feature is not `Planned`
without discovery + a priority score + acceptance criteria.

Relations are explicit links between files (front-matter `related:` lists or `[[slug]]` links):
`Opportunity -> Feature -> Workflow`, `Idea -> Opportunity | Feature`, `Feature -> Vision pillar`.
A feature with no upstream opportunity, or an opportunity with no evidence, is a defect to flag,
not to silently accept.

## The operating loop for a "work a feature" mission

1. **Locate on the chain.** Is the request an outcome, an opportunity, an idea, or a feature? If
   it arrives as "build feature X", walk it BACKWARDS: which opportunity, which outcome, what
   evidence exists? Create the missing upstream objects (thin is fine, but present).
2. **Discovery gate.** Run the Feature Discovery checklist (ref 5). If problem/solution confidence
   is low, the next step is an experiment, not code. Say so.
3. **Prioritize.** Score it (ref 6 — RICE or the weighted model). A feature with no score does not
   enter `Planned`.
4. **Specify.** Fill the Feature object (ref 4): user story, scope / out-of-scope, requirements,
   acceptance criteria, edge cases, dependencies, metrics.
5. **Model the workflow** (ref 7) if the feature introduces or changes a process/automation.
6. **Then, and only then, decompose into build tasks** and dispatch workers (R-ORCH). The
   acceptance criteria from step 4 become the workers' Done Criteria (R-RUBRIC).
7. **Measure** against the success metric after release; feed the result back as an idea/opportunity
   (Improved).

## Anti-patterns (reject these)

- Jumping straight to implementation without an opportunity or evidence.
- A feature with no acceptance criteria or no success metric.
- Prioritizing by opinion instead of a scored method.
- A "brainstorm" that is a flat note dump with no clustering, status, or qualification.
- Silently widening scope: scope and out-of-scope are both explicit fields.
- Marking an object `Validated` / `Planned` / `Released` ahead of the evidence (L1).

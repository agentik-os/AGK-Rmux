---
name: operations-automation-os
description: >
  Interview and observe how a product or business actually works, reveal waste and control gaps, decide what to remove, simplify, standardize, delegate or automate, and produce production-ready automation blueprints with monitoring and recovery. Contains 24 specialist agents, 39 skills, 9 protocols and 9 schemas. Use for process audits, workflow simplification, automation design, standard operating procedures, or control-gap review. Trigger words: operations, automation, process audit, workflow, standardize, delegate, automation blueprint, SOP; FR: operations, automatisation, audit de processus, flux de travail, standardiser, deleguer, blueprint d'automatisation.
---

# Operations & Automation {OS}

Runtime-installed pack (2026-08-11), staged for the OmegaOS repo-level R-SKILLPUB integration by a concurrent session. This SKILL.md is a pointer into the shipped pack; it does not restate or invent the pack's operating contract.

## Load before operating

- [README.md](README.md) for purpose, operating loop, commands and main handoffs.
- [system/SYSTEM_PROMPT.md](system/SYSTEM_PROMPT.md) for the full operating contract.
- [system/PRINCIPLES.md](system/PRINCIPLES.md) and [system/BOUNDARIES.md](system/BOUNDARIES.md) for scope and limits.
- [system/ROUTER.md](system/ROUTER.md) for command/intent routing.
- [MANIFEST.json](MANIFEST.json) for the full inventory (agents, skills, protocols, schemas).
- [OMEGA_INTEGRATION.md](OMEGA_INTEGRATION.md) for registration ID, event types and cross-OS handoffs.
- `agents/*.md` for specialist agent definitions, `skills/*.md` for reusable skill procedures, `protocols/*.md` for multi-step operating protocols, `schemas/*.json` for the data model.

## Commands

| Command | Mode | Purpose |
| --- | --- | --- |
| `/operations` | diagnose | Open operations diagnostic |
| `/process-interview` | diagnose | Interview process owners/users |
| `/process-map` | map | Map current state |
| `/value-stream` | map | Analyze flow and waste |
| `/simplify` | challenge | Remove and simplify work |
| `/automation-audit` | score | Find and score automation candidates |
| `/automate` | design | Create automation blueprint |
| `/agent-automation` | agent | Assess an AI-agent workflow |
| `/future-state` | design | Design target operating model |
| `/runbook` | deploy | Create operating runbook |
| `/automation-review` | audit | Audit live automations |
| `/automation-incident` | incident | Contain and recover failure |

## Boundary

This pack is a runtime skill install only. It does not modify the OmegaOS repository, its install.sh, os_products.rs or OS-SUITE.md; that repo-level integration is a separate, coordinated follow-up (see handoff note).

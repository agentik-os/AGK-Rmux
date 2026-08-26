---
name: strategy-portfolio-os
description: >
  Convert ambition, evidence and constraints into a coherent strategy, explicit choices, a ranked portfolio of bets and disciplined allocation of time, attention, people and capital. Omega Core function that selects goals, bets, projects and resource allocation before execution begins. Contains 12 specialist agents, 20 skills, 6 protocols and 7 schemas. Use for strategy formulation, bet ranking, portfolio prioritization, or resource allocation decisions across projects. Trigger words: strategy, portfolio, bets, resource allocation, prioritization, strategic choice; FR: strategie, portefeuille, paris strategiques, allocation de ressources, priorisation, choix strategique.
---

# Strategy & Portfolio {OS}

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
| `/strategy` | design | Open strategic design |
| `/diagnosis` | diagnose | Define the critical challenge |
| `/portfolio` | portfolio | Review all projects and bets |
| `/prioritize` | portfolio | Rank competing initiatives |
| `/scenario` | scenario | Build future scenarios |
| `/strategic-decision` | decision | Structure a consequential choice |
| `/quarter-plan` | quarter | Create quarterly strategy |
| `/kill-review` | review | Decide continue/pivot/pause/kill |
| `/one-page-strategy` | design | Produce a concise strategy memo |
| `/not-doing` | portfolio | Define exclusions |

## Boundary

This pack is a runtime skill install only. It does not modify the OmegaOS repository, its install.sh, os_products.rs or OS-SUITE.md; that repo-level integration is a separate, coordinated follow-up (see handoff note).

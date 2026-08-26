---
name: relationship-network-os
description: >
  Help build, protect and deepen valuable human relationships through attention, memory, generous relevance, follow-through, boundaries, communication and thoughtful introductions. Contains 12 specialist agents, 18 skills, 7 protocols and 6 schemas. Use for relationship tracking, follow-up planning, introduction crafting, boundary setting, or network stewardship. Trigger words: relationship, network, follow up, introduction, boundaries, relationship memory; FR: relation, reseau, suivi de contact, introduction, limites, memoire relationnelle.
---

# Relationship & Network {OS}

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
| `/network` | audit | Open relationship overview |
| `/person` | brief | Prepare a person brief |
| `/meeting-prep` | brief | Prepare for a meeting |
| `/interaction` | capture | Capture interaction and commitments |
| `/follow-up` | follow-up | Draft a relevant follow-up |
| `/intro` | connect | Create a consent-based introduction |
| `/nurture` | nurture | Design a relationship rhythm |
| `/difficult-conversation` | conflict | Prepare a truthful conversation |
| `/boundary` | conflict | Set or reinforce a boundary |
| `/gathering` | gather | Design a gathering |

## Boundary

This pack is a runtime skill install only. It does not modify the OmegaOS repository, its install.sh, os_products.rs or OS-SUITE.md; that repo-level integration is a separate, coordinated follow-up (see handoff note).

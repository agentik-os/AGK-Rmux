---
name: health-energy-os
description: >
  Build and protect physical and cognitive capacity through sleep, movement, training, nutrition, recovery, stress regulation, environment design and appropriate professional care. Upstream capacity provider for Habit, Execution and Strategy OS. Contains 12 specialist agents, 18 skills, 8 protocols and 6 schemas. Use for sleep and recovery planning, training and nutrition design, stress regulation, energy audits, or deciding when to escalate to a professional. Trigger words: health, energy, sleep, movement, training, nutrition, recovery, stress, capacity; FR: sante, energie, sommeil, mouvement, entrainement, nutrition, recuperation, stress, capacite.
---

# Health & Energy {OS}

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
| `/health` | check-in | Open Health & Energy OS |
| `/readiness` | check-in | Assess today’s capacity |
| `/health-audit` | audit | Build a baseline |
| `/sleep` | audit | Audit sleep and circadian constraints |
| `/training` | plan | Build or revise training |
| `/nutrition` | plan | Review fuel and adherence |
| `/recovery` | recovery | Respond to fatigue or overload |
| `/travel-health` | travel | Design a travel protocol |
| `/health-experiment` | experiment | Create an N-of-1 experiment |
| `/wearable` | explain | Interpret trends conservatively |

## Boundary

This pack is a runtime skill install only. It does not modify the OmegaOS repository, its install.sh, os_products.rs or OS-SUITE.md; that repo-level integration is a separate, coordinated follow-up (see handoff note).

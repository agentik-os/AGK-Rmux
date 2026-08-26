---
name: review-governance-os
description: >
  Turn actions, incidents, metrics and decisions into honest learning, controlled change, explicit policy and continuously improved personal and professional systems. Omega Core function that closes learning loops and governs consequential change across all other OSs (approval authority referenced by Revenue OS for boundary/schema/quality-gate changes). Contains 13 specialist agents, 20 skills, 7 protocols and 7 schemas. Use for incident review, postmortems, policy changes, retrospectives, or governance approval of a consequential change. Trigger words: review, governance, incident, postmortem, policy, retrospective, approval; FR: revue, gouvernance, incident, retour d'experience, politique, retrospective, approbation.
---

# Review & Governance {OS}

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
| `/review` | weekly | Open review |
| `/daily-review` | daily | Run daily reflection |
| `/weekly-review` | weekly | Run weekly operating review |
| `/monthly-review` | monthly | Run monthly metrics review |
| `/quarterly-review` | quarterly | Run strategic governance |
| `/postmortem` | postmortem | Analyze an incident or failure |
| `/policy` | policy | Create or audit a policy |
| `/change-request` | change | Submit consequential change |
| `/risk-register` | monthly | Review risks |
| `/ai-governance` | ai-risk | Apply AI risk governance |

## Boundary

This pack is a runtime skill install only. It does not modify the OmegaOS repository, its install.sh, os_products.rs or OS-SUITE.md; that repo-level integration is a separate, coordinated follow-up (see handoff note).

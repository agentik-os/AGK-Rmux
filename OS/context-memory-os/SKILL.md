---
name: context-memory-os
description: >
  Maintain one trustworthy, inspectable and permissioned memory layer so Omega and its OSs recover context without mixing facts, inferences, temporary states, projects or identities. Omega Core canonical shared context layer for every other OS. Contains 14 specialist agents, 20 skills, 7 protocols and 8 schemas. Use for memory design, context recall, fact-versus-inference separation, cross-project isolation checks, or permissioned knowledge retrieval. Trigger words: context, memory, recall, knowledge layer, fact versus inference, permissioned memory; FR: contexte, memoire, rappel, couche de connaissance, faits versus inferences, memoire permissionnee.
---

# Context & Memory {OS}

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
| `/memory` | retrieve | Search or inspect memory |
| `/remember` | capture | Propose a memory write |
| `/ingest` | capture | Ingest a file or event |
| `/context` | compile | Compile a purpose-specific context pack |
| `/snapshot` | snapshot | Create a versioned snapshot |
| `/decision-log` | capture | Record a decision and rationale |
| `/contradiction` | resolve | Resolve conflicting records |
| `/memory-audit` | govern | Audit provenance and access |
| `/forget` | forget | Delete or archive authorized memory |
| `/export-memory` | govern | Create a user-readable export |

## Boundary

This pack is a runtime skill install only. It does not modify the OmegaOS repository, its install.sh, os_products.rs or OS-SUITE.md; that repo-level integration is a separate, coordinated follow-up (see handoff note).

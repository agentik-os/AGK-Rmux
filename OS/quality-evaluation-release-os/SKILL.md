---
name: quality-evaluation-release-os
description: >
  Prove that a product conforms to its contracts, manages risk, can be observed and recovered, and is ready for controlled release and operation. Independent certification authority positioned between Builder OS and production. Contains 16 specialist agents, 26 skills, 7 protocols and 8 schemas. Use for release readiness review, quality gates, contract conformance checks, observability and recovery validation, or go/no-go release decisions. Trigger words: quality, evaluation, release, release readiness, conformance, observability, go no-go; FR: qualite, evaluation, mise en production, conformite, observabilite, decision de lancement.
---

# Quality, Evaluation & Release {OS}

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
| `/quality` | intake | Open quality authority |
| `/test-plan` | plan | Build risk-based test plan |
| `/traceability` | plan | Map requirements to evidence |
| `/qa` | test | Run functional and exploratory QA |
| `/ai-eval` | eval | Design/run AI evaluations |
| `/security-review` | audit | Apply security standards |
| `/accessibility` | audit | Audit WCAG/mobile accessibility |
| `/release-candidate` | candidate | Assemble candidate evidence |
| `/release-gate` | candidate | Issue release decision |
| `/deploy` | release | Execute controlled release |
| `/rollback` | incident | Trigger or prepare rollback |

## Boundary

This pack is a runtime skill install only. It does not modify the OmegaOS repository, its install.sh, os_products.rs or OS-SUITE.md; that repo-level integration is a separate, coordinated follow-up (see handoff note).

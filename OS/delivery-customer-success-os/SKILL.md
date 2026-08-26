---
name: delivery-customer-success-os
description: >
  Manage the complete customer journey after commercial commitment: handoff, onboarding, discovery, success planning, delivery, scope, communication, acceptance, adoption, value proof, renewal, expansion, referral and offboarding. Contains 19 specialist agents, 30 skills, 9 protocols and 9 schemas. Use for client onboarding, delivery tracking, scope management, customer success planning, adoption/value-proof reporting, or renewal and expansion handoffs. Trigger words: delivery, customer success, onboarding, scope, adoption, renewal, expansion, offboarding; FR: livraison, succes client, integration client, perimetre, adoption, renouvellement, expansion.
---

# Delivery & Customer Success {OS}

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
| `/delivery` | review | Open delivery portfolio |
| `/handoff-client` | handoff | Run sales-to-delivery transfer |
| `/onboard-client` | onboard | Create onboarding plan |
| `/success-plan` | plan | Define outcomes and measures |
| `/client-plan` | plan | Create milestones and governance |
| `/client-update` | deliver | Draft transparent status update |
| `/scope-change` | risk | Process a change request |
| `/client-risk` | risk | Create escalation plan |
| `/adoption` | adopt | Build adoption intervention |
| `/value-proof` | value | Compile outcome evidence |
| `/qbr` | value | Prepare business review |
| `/renew-client` | renew | Prepare renewal/expansion |

## Boundary

This pack is a runtime skill install only. It does not modify the OmegaOS repository, its install.sh, os_products.rs or OS-SUITE.md; that repo-level integration is a separate, coordinated follow-up (see handoff note).

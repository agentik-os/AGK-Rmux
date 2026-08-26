---
name: revenue-os
description: >
  Operate a conversational revenue brain and governed database for offers, positioning, pricing, leads, sales pipeline, customers, contracts, invoicing, payments, expenses, cash flow, reserves, forecasting, retention and expansion. Contains 24 specialist agents, 40 skills, 10 protocols and 14 JSON schemas. Use for CRM work, pricing and offer design, sales calls, invoicing and collections, cash flow forecasting, monthly revenue close, or renewal and expansion planning. Trigger words: revenue, offer, pricing, pipeline, invoice, collections, cash flow, forecast, CRM, sales call, proposal, renewal; FR: chiffre d'affaires, tarification, pipeline commercial, facture, recouvrement, prevision de tresorerie, CRM, appel de vente, renouvellement.
---

# Revenue {OS}

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
| `/revenue` | dashboard | Open revenue brain |
| `/offer` | offer | Create or audit an offer |
| `/positioning` | offer | Define category and differentiation |
| `/pricing` | offer | Build pricing architecture |
| `/pipeline` | pipeline | Review CRM and forecast |
| `/lead` | pipeline | Create or analyze a lead |
| `/sales-call` | sales | Prepare or debrief a call |
| `/proposal` | sales | Create proposal and commercial logic |
| `/invoice` | billing | Create or inspect invoice |
| `/collections` | billing | Manage overdue receivables |
| `/business-cashflow` | finance | Analyze business cash flow |
| `/receipt-business` | ingest | Stage business receipt/photo |

## Boundary

This pack is a runtime skill install only. It does not modify the OmegaOS repository, its install.sh, os_products.rs or OS-SUITE.md; that repo-level integration is a separate, coordinated follow-up (see handoff note).

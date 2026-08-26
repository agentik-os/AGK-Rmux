---
name: wealth-capital-os
description: >
  Operate a conversational financial brain for personal cash flow, savings, emergency resilience, debt, goals, investment policy, risk and life-aligned capital allocation. Contains 12 specialist agents, 20 skills, 7 protocols and 7 schemas. Use for personal budgeting, emergency fund planning, debt payoff strategy, investment policy statements, risk tolerance, or goal-based capital allocation. Trigger words: wealth, capital, personal cash flow, savings, debt, investment policy, risk, capital allocation; FR: patrimoine, capital, tresorerie personnelle, epargne, dette, politique d'investissement, risque, allocation de capital.
---

# Wealth & Capital {OS}

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
| `/wealth` | dashboard | Open personal CFO dashboard |
| `/money-close` | close | Reconcile the month |
| `/cashflow` | dashboard | Analyze personal cash flow |
| `/saving` | plan | Create or revise savings plan |
| `/emergency-fund` | plan | Size and fund resilience reserve |
| `/debt` | decision | Choose a debt strategy |
| `/invest-policy` | plan | Create an investment policy statement |
| `/purchase` | decision | Evaluate a major purchase |
| `/money-scenario` | scenario | Model a financial scenario |
| `/receipt` | ingest | Stage a personal document or receipt |

## Boundary

This pack is a runtime skill install only. It does not modify the OmegaOS repository, its install.sh, os_products.rs or OS-SUITE.md; that repo-level integration is a separate, coordinated follow-up (see handoff note).

# Quality, Evaluation & Release OS, v1.0.0

**Category:** Product Stack / Independent Quality and Release Authority  
**Omega position:** Product Stack: independent certification between Builder OS and production  
**Primary interface:** conversational + structured records  
**Status:** installable reference implementation

## Purpose
Prove that a product conforms to its contracts, manages risk, can be observed and recovered, and is ready for controlled release and operation.

## Promise
Replace “it seems done” with traceable evidence across functionality, UX, accessibility, performance, reliability, security, privacy, data, AI behavior, operations and rollback.

## Core equation

```text
RELEASE CONFIDENCE = REQUIREMENT TRACEABILITY × RISK-BASED EVIDENCE × SECURITY × RELIABILITY × OBSERVABILITY × RECOVERABILITY
```

## Operating loop

```text
CONTRACTS → RISK MODEL → TEST/EVAL PLAN → EXECUTE → TRIAGE → FIX/RETEST → GATES → RELEASE CANDIDATE → DEPLOY → VERIFY → MONITOR / ROLLBACK
```

## What this OS contains
- Canonical system prompt and explicit boundaries
- 16 specialist agents
- 26 reusable skills
- 7 operating protocols
- domain memory and privacy model
- JSON schemas for 8 core entities
- evaluation suite and adversarial tests
- provider-neutral reference runtime
- curated book canon and source ledger

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

## Main handoffs
- Blueprint/Design provide contracts; Stepper provides implementation order; Builder provides build artifacts.
- Review & Governance OS owns policy exceptions and postmortems.
- Operations OS receives runbooks and observability contracts.
- Context & Memory OS stores release evidence manifests.

## Installation
See `INSTALL.md` and `OMEGA_INTEGRATION.md`.

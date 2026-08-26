# Market Research OS, v1.0.0

**Category:** Business Stack / Market Evidence and Validation Intelligence  
**Omega position:** Core Stack (supporting): market and customer evidence and validation for Strategy and Blueprint  
**Primary interface:** conversational + versioned records  
**Status:** installable reference implementation

## Purpose
Turn an idea or market question into a versioned body of evidence, explicit hypotheses, auditable models, falsifiable experiments, and a bounded decision, before any Blueprint or build begins. The OS operates as an evidence compiler and validation agency: it distinguishes evidence from inference, quantifies uncertainty, designs real-behavior experiments, and never declares an idea validated from desk research alone.

## Promise
Let the user discuss a market, a segment, a price, or a competitor as naturally as with an LLM, while every consequential claim carries a source, a method, a confidence, a trace link, and its negative evidence, and while the recommendation stays inside a predeclared decision threshold.

## Position in the value chain

```text
Idea or opportunity -> Market Research OS -> Founder decision -> Blueprint OS -> Stepper OS -> Build OS -> Market feedback -> Research revision
```

- Decides whether a market and problem are attractive enough to pursue, which segment and promise deserve a Blueprint, and what remains uncertain.
- Does not define the full product or system contract (that belongs to Blueprint OS).
- Does not create an implementation DAG (that belongs to Stepper OS).
- Does not build production code or launch live campaigns without explicit authority; research prototypes, interview guides, survey instruments, experiment specs, and mock offer copy are permitted.

## Operating loop

```text
FRAME -> RECOVER -> HYPOTHESIZE -> DESIGN -> PREFLIGHT -> COLLECT -> TRIANGULATE -> MODEL -> EXPERIMENT -> CRITIC and RED-TEAM -> DECIDE -> HANDOFF
```

## What this OS contains
- Canonical system prompt and explicit boundaries (`references/system-prompt.md`).
- 17 specialist roles (engagement-director, context-librarian, research-architect, acquisition-provenance-lead, market-category-analyst, market-sizing-modeler, customer-jtbd-researcher, survey-quant-methodologist, competitive-intelligence-analyst, demand-signal-analyst, pricing-economics-analyst, gtm-strategist, experiment-designer, privacy-ethics-governance-reviewer, data-quality-auditor, red-team-investment-critic, traceability-auditor, chief-research-editor), defined as shared-state fan-out/fan-in role prompts in `assets/market-research-role-prompts.json`. The `agents/` directory holds only the OpenAI interface descriptor (`openai.yaml`), not one markdown file per role.
- 13 reference protocol documents in `references/` (research contract, orchestration and gates, methods and frameworks, source and tool registry, data acquisition and compliance, experiments and primary research, scoring and decision, response and continuation, vertical playbooks, agency service model, evidence source notes, Omega OS integration, plus the system prompt). This pack has no separate `protocols/` directory: the single operating protocol is the 19-step compiler workflow, documented across these references and `SKILL.md`.
- 18 assets in `assets/`: JSON schemas (research state, blueprint input manifest, role prompts, machine tool definitions, Omega OS plugin manifest) and reusable templates (research brief, research plan, source preflight, competitor profile, market model, decision scorecard, evidence ledger, experiment, customer interview, survey questionnaire, voice-of-customer codebook, report, icon).
- 20 machine tool definitions in `assets/market-research-tools.json` (initialize, source register, preflight evaluate, id allocate, record upsert, trace link, query plan register, acquisition run register, model upsert, study upsert, experiment upsert, finding register, validate, gate evaluate, score, checkpoint save, export, blueprint handoff create, delta, status).
- 3 Python scripts in `scripts/`: `market_research_os.py` (provider-neutral reference runtime to initialize, inspect, checkpoint, validate, score, and export a machine-readable research workspace), `install_omega_os.py` (preview or install the portable extension into an Omega OS checkout), and `build_complete_manual.py`. This pack has no separate `runtime/` or `bin/` directory: the reference runtime is `scripts/market_research_os.py`.
- A single Skill (`SKILL.md`) with 9 invocation modes (NEW, RECOVER, RAPID_SCAN, FULL_VALIDATION, DILIGENCE, DEEP_DIVE, MONITOR, AUDIT, DELTA) and 3 depth profiles (SIGNAL, VALIDATION, INVESTMENT_GRADE). There is no `skills/` subdirectory.

## Commands
| Command | Mode | Purpose |
| --- | --- | --- |
| `/market-research` | infer | Open the research compiler and infer the mode |
| `/market-research scan` | RAPID_SCAN | Fast directional read: fatal flaws, strongest signals, next evidence |
| `/market-research validate` | FULL_VALIDATION | Full validation run with primary research and experiments |
| `/market-research diligence` | DILIGENCE | Investment-grade run with stronger source, legal, and review standards |
| `/market-research recover` | RECOVER | Recover a canonical baseline from prior research, chats, or files |
| `/market-research deep` | DEEP_DIVE | One market, segment, competitor, feature, price, or channel |
| `/market-research monitor` | MONITOR | Re-run approved collection plans and report material deltas |
| `/market-research audit` | AUDIT | Review an existing study for evidence, method, bias, and traceability defects |
| `/market-research delta` | DELTA | Compare research versions or opportunities |
| `/market-research continue` | resume | Resume a run from its last checkpoint |
| `/market-research status` | read | Return run, version, status, progress, blockers, and continuation pointer |
| `/market-research score` | read | Compute evidence, hypothesis, opportunity, and gate diagnostics |
| `/market-research export` | read | Render a client-safe or machine-readable view |
| `/market-research handoff` | gated | Freeze a version and create a Blueprint Input Manifest |

Completion status values (literal system tokens): `MARKET RESEARCH IN PROGRESS`, `MARKET RESEARCH BLOCKED`, `MARKET RESEARCH COMPLETE — DECISION READY`. Decision vocabulary: `GO`, `PIVOT`, `HOLD`, `NO-GO`, `INSUFFICIENT EVIDENCE`.

## Main handoffs
- Brainstorm OS supplies selected concepts to validate (`brainstorm.concept.selected`).
- Blueprint OS receives a validated concept as a frozen Blueprint Input Manifest (`market.validation.completed`).
- Strategy & Portfolio OS receives willingness-to-pay and segment evidence (`market.validation.completed`).
- Context & Memory OS receives staged validated evidence (`memory.record.staged`) and returns verified records (`memory.record.verified`); the OS reads compiled context (`memory.context.compiled`).
- Revenue OS has no direct Market Research event today: it receives strategic and pricing implications only indirectly through Strategy & Portfolio OS.
- Review & Governance OS approves any change to boundaries, schemas, or quality gates in production.

## Installation
See `OMEGA_INTEGRATION.md` for the registration contract, context injection order, and event wiring. Use `scripts/install_omega_os.py` to preview or install the portable extension into an Omega OS checkout.

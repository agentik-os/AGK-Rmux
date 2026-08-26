# R-PRODUCT — Work product through the Product Development System

**Kind:** Rule
**Category:** Orchestration
**Added:** 2026-07-29

## Rule

Whenever a mission touches a product decision — a feature, a roadmap, a priority call, an idea, an opportunity, or a process/workflow — run it through the OmegaOS Product Development System, never straight from "we have an idea" to "let's build it". The chain is always: Business Outcome → Opportunity → Idea (brainstorm) → Feature (Discovery → Prioritization → Specification) → Workflow → Build → Measure → Improve. If asked to just build X, first place X on the chain and backfill the missing upstream objects (which outcome, which opportunity, what evidence, what acceptance criteria, what success metric) before any code. Seven sub-systems, each with an exact object model, fields, statuses and relations: Vision Board, Brainstorming, Opportunity Board, Feature System, Feature Discovery, Prioritization (RICE/ICE/weighted), Workflow Builder. INVOKE the `product-development-system` skill for the full spec (never paraphrase the object model from memory); persist objects as markdown under the project's `agentic/product/` tree (vision/ ideas/ opportunities/ features/ workflows/), each with a `status` that never runs ahead of the evidence (L1 / R-VERIFY). Gates are hard: a feature reaches `Planned` only with a priority score + acceptance criteria + a success metric; `Released` only when verified against runtime (R-PROD). Acceptance criteria become the workers' Done Criteria (R-RUBRIC) at dispatch (R-ORCH).

## Origin

Operator directive (2026-07-29): from now on all feature/product work in OmegaOS follows a precise 7-system method (Vision Board, Brainstorming, Opportunity Board, Feature System, Discovery, Prioritization, Workflow Builder), and oracle sessions and workers must carry that knowledge and functional understanding. A code rule (scopes ALL, always-on) injects the method plus the pointer to the `product-development-system` skill into every dispatched agent, so no oracle or worker jumps idea→build or ships a feature without discovery, a score, and acceptance criteria. Complements R-SKILL-ATLAS (how the skill is discovered), R-ORCH (decomposition/dispatch), R-RUBRIC (acceptance criteria as Done Criteria), and R-PROD / L1 (runtime-verified release).

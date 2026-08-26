---
name: market-research-os
description: Compile a business, product, service, AI, marketplace, consumer, B2B, local, hardware, media, luxury, or regulated-market idea into a decision-grade market research and validation pack before Blueprint or implementation. Use when the user invokes /market-research, says Market Research {OS}, asks to validate or challenge an idea, study a market, size TAM/SAM/SOM, analyze competitors, mine customer pain or social/review signals, test feature or pricing demand, conduct commercial due diligence, plan ethical scraping, compare opportunities, audit an existing market study, or create a Blueprint input manifest. Recover prior decisions, distinguish evidence from inference, use current sources, quantify uncertainty, design primary research and real-behavior experiments, and never declare an idea validated from desk research alone.
---

# Market Research {OS}

Operate as an evidence compiler and validation agency. Convert an idea or market question into a versioned body of evidence, explicit hypotheses, auditable models, falsifiable experiments, and a bounded decision.

## Boundary and lifecycle

Enforce this lifecycle:

`Idea or opportunity -> Market Research {OS} -> Founder decision -> Blueprint {OS} -> Stepper {OS} -> Build {OS} -> Market feedback -> Research revision`

- Define whether a market and problem are attractive enough to pursue, which segment and promise deserve a Blueprint, and what remains uncertain.
- Do not define the full product/system contract; that belongs to Blueprint.
- Do not create an implementation DAG; that belongs to Stepper.
- Do not build production code or launch live campaigns without explicit authorization.
- Permit research prototypes, interview guides, survey instruments, experiment specifications, mock offer copy, and non-production test assets.

## Mandatory reads

Before a full or investment-grade run, read:

1. [system-prompt.md](references/system-prompt.md) for the complete operating contract.
2. [research-contract.md](references/research-contract.md) for required artifacts and record schemas.
3. [orchestration-and-gates.md](references/orchestration-and-gates.md) for the role graph, merge rules, critics, and completion gates.
4. [methods-and-frameworks.md](references/methods-and-frameworks.md) for MBA, strategy, qualitative, quantitative, pricing, and opportunity methods.
5. [source-and-tool-registry.md](references/source-and-tool-registry.md) for source lanes and tool selection.
6. [data-acquisition-and-compliance.md](references/data-acquisition-and-compliance.md) before any API, crawling, scraping, social listening, review mining, or personal-data processing.
7. [experiments-and-primary-research.md](references/experiments-and-primary-research.md) before interviews, surveys, concept tests, pricing tests, smoke tests, pilots, LOIs, or pre-sales.
8. [scoring-and-decision.md](references/scoring-and-decision.md) before scoring or making a recommendation.
9. [response-and-continuation.md](references/response-and-continuation.md) before a long or multi-part output.

Read conditionally:

- [vertical-playbooks.md](references/vertical-playbooks.md) for B2B, B2C, SaaS, marketplace, AI, local, hardware, luxury, regulated, or new-category variants.
- [omega-os-integration.md](references/omega-os-integration.md) for Omega OS installation, state, tools, commands, or handoffs.
- [agency-service-model.md](references/agency-service-model.md) when packaging the OS as a client service, defining engagement governance, or scoping deliverables.
- [evidence-source-notes.md](references/evidence-source-notes.md) when verifying current source/tool constraints and official reference links.

## Invocation modes

Infer and state one mode:

| Mode | Use | Result |
| --- | --- | --- |
| `NEW` | New idea or unframed opportunity | Build the evidence program from first principles. |
| `RECOVER` | Prior research, chats, files, or decisions exist | Recover a canonical baseline before new analysis. |
| `RAPID_SCAN` | Fast directional assessment | Identify fatal flaws, strongest signals, and next evidence. Never call this full validation. |
| `FULL_VALIDATION` | Standard launch decision | Complete secondary research, primary-research plan/results, experiments, models, and decision gates. |
| `DILIGENCE` | High-stakes investment, acquisition, board, or enterprise decision | Apply stronger source, sampling, legal, financial, and independent-review standards. |
| `DEEP_DIVE` | One market, segment, competitor, feature, price, or channel | Analyze the bounded question and propagate impact. |
| `MONITOR` | Refresh signals over time | Re-run approved collection plans and report material deltas. |
| `AUDIT` | Review an existing study | Diagnose evidence, methodology, bias, traceability, and decision defects. |
| `DELTA` | Compare research versions or opportunities | Produce semantic changes, confidence shifts, and decision impact. |

## Depth profiles

Choose the lowest profile capable of supporting the decision. Label the selected profile and exclusions.

| Profile | Minimum evidence | Permitted conclusion |
| --- | --- | --- |
| `SIGNAL` | Reputable desk research plus multi-source signal scan | Directional attractiveness and next tests only. |
| `VALIDATION` | Triangulated desk research plus customer evidence and at least one behavioral test | Conditional `GO`, `PIVOT`, `HOLD`, or `NO-GO`. |
| `INVESTMENT_GRADE` | Reproducible models, primary research with sampling disclosure, behavioral/commercial evidence, independent critic, legal/data review | Decision-grade recommendation with explicit residual risk. |

## Compiler workflow

Execute in order. Reopen earlier passes when evidence contradicts them.

1. **Frame the decision** — decision owner, capital/time at risk, geography, horizon, idea maturity, success threshold, non-goals, risk tolerance, deliverable.
2. **Recover context** — enumerate authorized sources; recover facts, decisions, constraints, definitions, prior attempts, and conflicts.
3. **Register hypotheses** — problem, segment, urgency, current behavior, value, solution, differentiation, price, acquisition, retention, feasibility, regulatory, and timing hypotheses.
4. **Design the research** — evidence questions, methods, sample, source plan, stopping rules, budget, dependencies, and bias controls.
5. **Run source preflight** — authority, access rights, terms, privacy, robots, licensing, retention, rate limits, PII, credentials, and permitted use.
6. **Map the environment** — category, adjacent markets, value chain, ecosystem, PESTEL/STEEPLED, forces, timing, regulation, and scenario drivers.
7. **Size the opportunity** — top-down, bottom-up, and value-based estimates where useful; TAM/SAM/SOM; assumptions; ranges; sensitivity; cross-checks.
8. **Segment demand** — behavioral/needs-based segments, beachhead, JTBD, pains, triggers, frequency, stakes, budgets, buying unit, switching costs, and exclusions.
9. **Mine voice of customer** — interviews, communities, reviews, support/forums, search language, workaround behavior, objections, emotional/social jobs, and exact-language evidence.
10. **Map alternatives and competition** — direct, indirect, substitute, do-nothing, internal-build; positioning, pricing, features, proof, distribution, business model, strategic group, value curve, and moat.
11. **Analyze demand signals** — search, traffic, ads, reviews, communities, social, jobs, funding, filings, product launches, app stores, developer activity, partnerships, patents, and trend quality.
12. **Challenge the offer and features** — opportunity-solution tree, Kano, importance/satisfaction, feature evidence, table stakes, differentiators, anti-features, adoption barriers, and minimum viable promise.
13. **Test price and economics** — WTP, price architecture, Van Westendorp/Gabor-Granger/conjoint where suitable, revenue logic, CAC constraints, gross margin, payback, retention, capacity, and marketplace liquidity.
14. **Assess go-to-market** — category entry point, positioning, channel access, sales cycle, trust, proof, distribution advantage, partnerships, wedge, and launch sequence hypotheses.
15. **Run primary research and experiments** — interviews, surveys, prototype tests, smoke/fake-door tests, landing pages, ads, concierge pilots, LOIs, deposits, pre-orders, or paid pilots as authorized.
16. **Triangulate and update confidence** — reconcile sources, detect contradictions, adjust hypothesis confidence, run sensitivity, identify alternative explanations, and log negative evidence.
17. **Critic and red-team** — desirability, viability, feasibility, defensibility, timing, ethics, data quality, sampling, incentives, regulatory, execution, founder-market-fit, and pre-mortem passes.
18. **Decide** — `GO`, `PIVOT`, `HOLD`, `NO-GO`, or `INSUFFICIENT EVIDENCE`; attach conditions, kill criteria, next evidence, owners, and expiry date.
19. **Handoff** — if eligible, freeze a Market Research version and create a Blueprint Input Manifest. Never silently invoke Blueprint.

## Epistemic discipline

Classify every material statement:

- `FACT` — directly supported by a cited source or observed result.
- `MEASUREMENT` — value produced by a defined method, population, window, and unit.
- `INFERENCE` — conclusion derived from evidence; name the reasoning and alternatives.
- `ASSUMPTION` — provisional input; include confidence and validation path.
- `HYPOTHESIS` — falsifiable claim with success/failure evidence.
- `DECISION` — accepted choice made by the authorized owner.
- `PROPOSAL` — recommendation not yet accepted.
- `UNKNOWN` — unresolved information.
- `CONFLICT` — incompatible claims or results.
- `LIMITATION` — known restriction on interpretation.
- `NEGATIVE EVIDENCE` — credible evidence against the idea or hypothesis.
- `SUPERSEDED` — retained historical item no longer active.

Do not convert mention volume into demand, search interest into willingness to pay, survey intent into purchase behavior, competitor funding into market attractiveness, or LLM synthesis into primary evidence.

## Stable IDs

Allocate monotonically and never reuse:

`SRC FCT MEA INF ASM HYP DEC PRP UNK CNF LIM NEG RQ QST SEG JTBD ALT CMP SIG DAT MTH SAM INT SUR EXP OBS EST MOD SCN PRC ECO CHN RSK MIT GATE REC BPH`

Examples: `HYP-014`, `SRC-021`, `EST-003`, `EXP-008`, `REC-002`.

Each normative record must include status, statement, provenance, method, scope/window, confidence, dependencies, contradictions, decision relevance, and verification or next action.

## Tool and scraping law

- Prefer provided first-party data, official APIs, exports, public datasets, licensed providers, and manual browsing before custom scraping.
- Complete the source preflight in [data-acquisition-and-compliance.md](references/data-acquisition-and-compliance.md) before collection.
- Never bypass authentication, access controls, CAPTCHAs, paywalls, technical restrictions, or platform enforcement.
- Never use hidden credentials or collect private areas without explicit authorization.
- Respect applicable terms, robots directives, rate limits, copyright/database rights, privacy law, consent, deletion obligations, and data minimization.
- Treat public personal data as personal data when law/policy does; do not infer sensitive traits or create surveillance profiles.
- Keep raw source evidence separate from normalized findings. Store source URL/locator, retrieval time, query, tool/version, license/use basis, and transformation lineage.
- Validate parsers with samples, schema checks, duplicate detection, freshness, missingness, outlier review, and drift monitoring.
- If collection is not permitted or source-of-truth access is missing, stop that lane and use an explicitly weaker approved alternative; do not pretend equivalence.

## Primary-research law

- Ask about past behavior, concrete episodes, consequences, frequency, workaround, budget, authority, and switching—not compliments or speculative intent alone.
- Separate discovery interviews from sales pitches.
- Record recruitment source, inclusion/exclusion criteria, incentive, consent, instrument version, sample limitations, and analyst influence.
- Pretest surveys; avoid leading, double-barreled, ambiguous, loaded, and non-exhaustive questions; randomize where order can bias results.
- Define metrics, thresholds, sample/stopping rules, and analysis before observing experiment outcomes.
- Prefer behavior with real friction—time, money, data, access, or organizational commitment—over declarations.

## Question policy

Do not block on every unknown. Infer reversible, low-impact details as assumptions. Ask at most three high-leverage questions when a missing choice materially changes market boundary, target segment, geography, business model, legal exposure, capital at risk, or the decision threshold. Continue independent work.

## Completion and decisions

Use only:

- `MARKET RESEARCH IN PROGRESS`
- `MARKET RESEARCH BLOCKED`
- `MARKET RESEARCH COMPLETE — DECISION READY`

Never declare completion when only desk research is present but the requested conclusion requires observed customer or commercial behavior. In that case, finish the desk-research phase, provide the executable validation plan, and retain `IN PROGRESS` or return `INSUFFICIENT EVIDENCE` as the recommendation.

The decision vocabulary is:

- `GO` — evidence crosses the defined threshold; risks and conditions remain visible.
- `PIVOT` — a materially different segment/problem/promise/model has stronger evidence.
- `HOLD` — attractive but blocked by timing, access, economics, regulation, or missing dependency.
- `NO-GO` — evidence crosses a predeclared kill threshold or credible downside dominates.
- `INSUFFICIENT EVIDENCE` — the current evidence cannot support a responsible decision.

## Long outputs

Follow [response-and-continuation.md](references/response-and-continuation.md). Preserve run ID, version, record counters, completed/current/next artifact pointers, queries, source snapshots, unresolved conflicts, gate state, confidence changes, and checksum. Never label a partial pack complete.

## Deterministic support

Use `scripts/market_research_os.py` to initialize, inspect, checkpoint, validate, score, and export a machine-readable research workspace. Use `scripts/install_omega_os.py` to preview or install the portable extension into an Omega OS checkout. Deterministic validation complements; it never replaces expert judgment.

## Final handoff

End a complete run with:

1. one-page decision memo;
2. research scope, version, and limitations;
3. evidence and negative-evidence ledgers;
4. market definition and sizing model;
5. segment/JTBD and voice-of-customer synthesis;
6. alternatives/competition and positioning map;
7. demand, feature, pricing, economics, and channel findings;
8. experiment results and unrun validation plan;
9. risk, scenario, and pre-mortem summary;
10. hypothesis scorecard and evidence-strength audit;
11. quality-gate scorecard and orphan report;
12. recommendation with conditions, kill criteria, and expiry;
13. Blueprint Input Manifest when `GO` or `PIVOT` is Blueprint-eligible;
14. status: `MARKET RESEARCH COMPLETE — DECISION READY`.

Do not generate the full Blueprint unless the user separately invokes `/blueprint`.

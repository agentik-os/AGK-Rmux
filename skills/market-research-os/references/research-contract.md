# Market Research {OS} — Research Pack Contract

## Contents

1. Document header
2. Required artifact order
3. Core record schemas
4. Traceability
5. Machine handoff
6. Applicability profiles

## 1. Document header

Every pack begins with:

```yaml
research_run_id: MRR-<project>-<date>-<nonce>
project_id: <stable-project-id>
project_name: <name>
research_version: <semver>
mode: NEW|RECOVER|RAPID_SCAN|FULL_VALIDATION|DILIGENCE|DEEP_DIVE|MONITOR|AUDIT|DELTA
depth: SIGNAL|VALIDATION|INVESTMENT_GRADE
status: MARKET RESEARCH IN PROGRESS|MARKET RESEARCH BLOCKED|MARKET RESEARCH COMPLETE — DECISION READY
decision_owner: <person-or-role>
decision_due: <date-or-unknown>
evidence_cutoff: <timestamp>
research_expiry: <date-or-trigger>
geographies: []
languages: []
segments_in_scope: []
segments_out_of_scope: []
business_model_hypotheses: []
external_action_authority: none|research-only|approved-scope
confidentiality: public|internal|confidential|restricted
source_count: 0
hypothesis_count: 0
experiment_count: 0
state_revision: 0
state_checksum: sha256:<hash>
```

## 2. Required artifact order

### 00 — Run Manifest and Decision Brief

Record question, owner, decision options, stakes, horizon, budget/time/tool constraints, boundaries, thresholds, authorization, and depth.

### 01 — Executive Decision Memo

One page: recommendation, confidence, bounded opportunity, strongest evidence, strongest negative evidence, conditions, kill criteria, next evidence, and expiry.

### 02 — Recovered Context and Source Ledger

List all internal/external sources, authority, access/use basis, coverage, freshness, fingerprint, query/method, and limitations. Include missing canonical sources.

### 03 — Epistemic Ledgers

Facts, measurements, inferences, assumptions, decisions, proposals, unknowns, conflicts, limitations, negative evidence, superseded records.

### 04 — Research Question and Hypothesis Register

Map each decision question to falsifiable hypotheses, thresholds, method, sample/source, current confidence, decision impact, and status.

### 05 — Research Design and Evidence Plan

Method mix, sequence, sample frames, recruitment, instruments, pretests, source queries, data plan, stopping rules, analysis, bias controls, budget, owners, and dependencies.

### 06 — Data Acquisition, Rights, Privacy, and Ethics Plan

Preflight per source, allowed method, terms/robots/license/privacy basis, personal data, retention/deletion, attribution, rate limits, credentials, controls, and prohibited lanes.

### 07 — Market and Category Definition

Customer/problem/use-case/solution/transaction boundaries, industry classification, geography, time, unit, adjacent markets, substitutes, non-consumption, glossary, and exclusions.

### 08 — Macro, Ecosystem, Value Chain, and Timing

Evidence-backed PESTEL/STEEPLED, five forces, complementors, value chain, profit pools, control points, scenarios, why-now, reversals, and leading indicators.

### 09 — Market Size and Growth Model

TAM/SAM/SOM definitions, top-down/bottom-up/value models, formulas, source-linked inputs, currency/base year, ranges, sensitivity, cross-checks, model divergence, and confidence.

### 10 — Segment and Beachhead Model

Needs/behavior-based segments, eligibility, size, pain, urgency, budget, reachability, cost-to-serve, adoption friction, evidence, priority, and excluded segments.

### 11 — Persona, JTBD, and Buying-System Contracts

Actors, jobs, triggers, progress, current behavior, consequences, desired outcomes, user/buyer/approver/gatekeeper, procurement, proof, switching chain, and retention.

### 12 — Voice-of-Customer Evidence Corpus

Thematic codebook, source/sample metadata, exact short language snippets where permitted, frequency/severity, workaround, objection, emotion, purchase criteria, contradictions, and saturation.

### 13 — Alternatives and Competitive Intelligence

Direct/indirect/substitute/do-nothing/internal/open-source/service alternatives; profiles, strategic groups, value curve, pricing/packaging, features-to-outcomes, proof, channels, traction proxies, moats, likely response, win/loss, and source dates.

### 14 — Demand and Trend Signal Dashboard

Search, traffic, ads, social/community, reviews, apps, jobs, funding/filings, developer, patents/papers/standards, procurement, partnerships, and supply signals. State what each signal can/cannot prove.

### 15 — Opportunity, Offer, and Feature Evidence Map

Opportunity-solution tree, minimum viable promise, table stakes, value mechanisms, differentiators, trust features, anti-features, adoption barriers, feature evidence, confidence, and experiments.

### 16 — Pricing and Willingness-to-Pay Study

Value metric, reference/alternative cost, observed spend, method/instrument, segment distributions, price/package/fence hypotheses, bias, live offer evidence, and recommendation.

### 17 — Business Model and Unit-Economics Model

Revenue, margin, variable costs, service/AI/infrastructure costs, capacity, conversion, sales cycle, CAC constraints, payback, retention, expansion, working capital, marketplace liquidity/fraud as applicable, scenarios, and sensitivities.

### 18 — Positioning and Go-to-Market Evidence

Category entry, positioning statement, alternatives, proof, messaging language, channels, sales motion, funnel assumptions, channel economics, partner model, wedge, scale ceiling, and validation plan.

### 19 — Primary Research Instruments and Results

Recruitment, consent, guides/surveys/stimuli, versions, sample/denominator, raw-result locator, coding/analysis, findings, limitations, deviations, and confidence. Distinguish planned from executed.

### 20 — Validation Experiment Portfolio

Experiment contracts, priority, authorization, state, results, guardrails, confounds, decision effect, and follow-ups. Include smoke, prototype, pilot, LOI, deposit/preorder/payment, retention, and scalable-acquisition evidence as applicable.

### 21 — Risk, Scenario, and Pre-mortem Register

Market, customer, competition, price, channel, data, regulatory, technology, operations, reputation, execution, founder/team, and second-order risks; likelihood, impact, evidence, leading indicator, mitigation, contingency, owner, kill trigger.

### 22 — Hypothesis and Evidence Scorecard

Show per hypothesis evidence strength, confidence, disconfirming evidence, gaps, gate relationship, and status. Do not show only an aggregate.

### 23 — Critic Findings and Dispositions

List critic, finding, severity, affected IDs, evidence, disposition, owner, and resolution verification.

### 24 — Traceability Matrix and Orphan Report

Cover decision questions through sources/findings/models/experiments/risks/recommendation. Report orphans and unsupported extrapolations.

### 25 — Quality Gate Scorecard

Evaluate all 24 gates with evidence, conditions, failures, and owner.

### 26 — Recommendation and Decision Contract

One of `GO`, `PIVOT`, `HOLD`, `NO-GO`, `INSUFFICIENT EVIDENCE`; exact scope, confidence, rationale, negative evidence, conditions, kill criteria, reversers, next evidence, owner, review/expiry.

### 27 — Blueprint Input Manifest

Only supported current statements: segment, problem/JTBD, current alternatives, promise, value events, required/table-stake capabilities, anti-features, price/model hypotheses and evidence, channels, constraints, risks, experiments still required, source refs, and explicit unknowns. Do not smuggle unsupported feature ideas into Blueprint.

### 28 — Monitoring and Refresh Plan

Signals, queries, sources, cadence/event trigger, drift thresholds, owners, cost, retention, and decision reopening rules. Mark N/A when one-time research is sufficient.

### 29 — Continuation and Change Ledger

Completed/current/next sections, ID counters, source/query snapshot, hypothesis/confidence deltas, model version, experiment status, conflicts, gates, checksum, and remaining work.

### 30 — Final Declaration

Status, recommendation, confidence, research depth actually achieved, known limitations, and handoff eligibility.

## 3. Core record schemas

### Source

```yaml
id: SRC-001
status: active|stale|withdrawn|superseded
publisher: ""
title: ""
locator: ""
source_type: internal|official|filing|primary-research|research-provider|competitor|platform|review|community|secondary
authority: primary|near-primary|secondary|proxy
published_at: null
retrieved_at: ""
geography: []
population: ""
time_coverage: ""
definitions: {}
access_method: api|export|manual|crawler|browser|licensed-file|provided
query_or_input: ""
tool_and_version: ""
rights_basis: ""
privacy_class: none|aggregate|personal|sensitive|unknown
confidentiality: public|internal|confidential|restricted
fingerprint: "sha256:..."
independence_group: ""
limitations: []
linked_records: []
```

### Hypothesis

```yaml
id: HYP-001
statement: ""
domain: problem|segment|behavior|value|solution|feature|price|channel|retention|economics|competition|feasibility|regulation|timing|right-to-win
status: untested|testing|supported|partially-supported|falsified|ambiguous|superseded
decision_criticality: P0|P1|P2
prior_confidence: 0.0
current_confidence: 0.0
falsifier: ""
evidence_required: []
metric: ""
pass_threshold: ""
fail_threshold: ""
ambiguous_rule: ""
methods: []
sample_or_sources: []
supporting_evidence: []
negative_evidence: []
conflicts: []
decision_impact: ""
next_test: ""
owner: ""
expires_at: null
```

### Finding

```yaml
id: INF-001
type: FACT|MEASUREMENT|INFERENCE|ASSUMPTION|UNKNOWN|CONFLICT|LIMITATION|NEGATIVE_EVIDENCE
statement: ""
status: proposed|accepted|validated|rejected|superseded
source_ids: []
method_ids: []
population: ""
geography: []
time_window: ""
unit: ""
sample_size: null
denominator: null
transformation: ""
confidence: 0.0
directness: direct|near-direct|proxy
independence: independent|partially-dependent|duplicated|unknown
alternative_explanations: []
limitations: []
linked_hypotheses: []
decision_relevance: ""
validation_or_resolution: ""
```

### Estimate and model

```yaml
id: EST-001
name: ""
market_boundary: ""
metric: revenue|spend|gmv|units|accounts|users|value-created
currency: EUR
base_year: 2026
geographies: []
formula: ""
inputs:
  - name: "eligible accounts"
    value: 0
    low: 0
    high: 0
    unit: accounts
    source_or_assumption_id: SRC-000
    transformation: ""
outputs:
  low: 0
  base: 0
  high: 0
cross_checks: []
sensitivity: []
limitations: []
confidence: 0.0
validation_priority: ""
```

### Competitor/alternative

```yaml
id: CMP-001
name: ""
alternative_type: direct|indirect|substitute|do-nothing|manual|internal-build|service|open-source|emerging
segments: []
promise: ""
category: ""
workflow: ""
pricing_and_packaging: []
outcomes_and_capabilities: []
proof_and_traction_proxies: []
channels_and_sales_motion: []
strengths: []
customer_failures: []
switching_costs: []
moat_hypotheses: []
likely_response: []
source_ids: []
observed_at: ""
confidence: 0.0
```

### Interview/study

```yaml
id: INT-001
study_type: problem-interview|buyer-interview|win-loss|expert|contextual|usability|survey|choice-study
status: planned|recruiting|running|analyzed|closed|cancelled
objective: ""
population: ""
sample_frame: ""
inclusion: []
exclusion: []
recruitment: ""
target_n: 0
achieved_n: 0
incentive: ""
consent: ""
instrument_version: ""
raw_data_ref: ""
analysis_method: ""
findings: []
limitations: []
deviations: []
linked_hypotheses: []
```

### Experiment

```yaml
id: EXP-001
title: ""
status: proposed|approved|running|analyzed|passed|failed|ambiguous|stopped
hypothesis_ids: []
population: ""
segment_ids: []
stimulus_or_offer: ""
control_or_baseline: ""
primary_metric: ""
secondary_metrics: []
guardrails: []
pass_threshold: ""
fail_threshold: ""
sample_rule: ""
stopping_rule: ""
duration_rule: ""
authorization: ""
privacy_ethics: []
cost_budget: ""
results:
  numerator: null
  denominator: null
  value: null
  interval: null
confounds: []
deviations: []
decision_effect: ""
evidence_refs: []
owner: ""
```

### Risk

```yaml
id: RSK-001
statement: ""
category: market|customer|competition|pricing|channel|data|legal|privacy|technology|operations|reputation|execution|team
likelihood: rare|unlikely|possible|likely|almost-certain
impact: low|medium|high|critical
velocity: slow|medium|fast
evidence: []
leading_indicators: []
mitigation: []
contingency: []
kill_trigger: ""
owner: ""
residual_risk: ""
status: open|mitigated|accepted|closed
```

### Recommendation

```yaml
id: REC-001
decision: GO|PIVOT|HOLD|NO-GO|INSUFFICIENT_EVIDENCE
scope: ""
confidence: 0.0
valid_until: ""
supporting_evidence: []
negative_evidence: []
critical_assumptions: []
conditions: []
kill_criteria: []
reversal_evidence: []
next_evidence: []
owner: ""
blueprint_eligible: false
rationale: ""
```

## 4. Traceability

Minimum chain:

`RQ -> HYP -> SRC/INT/SUR/EXP -> FCT/MEA/OBS/INF/NEG -> SEG/CMP/EST/PRC/ECO/CHN -> RSK/MIT -> REC -> BPH`

Critical hypotheses, recommendation conditions, kill criteria, and Blueprint manifest items require 100% trace coverage. Material findings target 95%.

## 5. Machine handoff

```json
{
  "handoff_id": "BPH-001",
  "project_id": "...",
  "research_version": "1.0.0",
  "state_revision": 0,
  "state_checksum": "sha256:...",
  "status": "MARKET RESEARCH COMPLETE — DECISION READY",
  "recommendation_id": "REC-001",
  "decision": "GO",
  "scope": {
    "geographies": [],
    "segments": [],
    "problem": "...",
    "jtbd": [],
    "promise": "...",
    "business_model_hypotheses": []
  },
  "market_models": [],
  "alternatives": [],
  "customer_evidence": [],
  "required_capabilities": [],
  "anti_features": [],
  "pricing_evidence": [],
  "channel_evidence": [],
  "constraints": [],
  "risks": [],
  "conditions": [],
  "kill_criteria": [],
  "unknowns": [],
  "mandatory_blueprint_questions": [],
  "mandatory_validation_before_build": [],
  "source_refs": []
}
```

Freeze handoffs. Research changes create a new version/delta; never mutate an accepted handoff in place.

## 6. Applicability profiles

Use `N/A` with rationale for non-applicable methods/artifacts. Apply [vertical-playbooks.md](vertical-playbooks.md). Do not force consumer surveys onto enterprise markets, five-forces prose onto a narrow feature test, or TAM theater onto a local capacity-constrained service.

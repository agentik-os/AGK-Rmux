# Market Research {OS} — Evidence Scoring and Decision Contract

## Contents

1. Why scoring is subordinate
2. Evidence-strength model
3. Hypothesis confidence
4. Opportunity dimensions
5. Kill gates
6. Recommendation rules
7. Portfolio comparison
8. Decision memo

## 1. Why scoring is subordinate

Scores structure judgment; they do not create truth. Never hide source, method, range, negative evidence, or a failed kill gate behind an aggregate. Show hypothesis-level evidence and raw model outputs first.

## 2. Evidence-strength model

Score each material evidence item from 0–4 on dimensions:

| Dimension | 0 | 2 | 4 |
| --- | --- | --- | --- |
| Authority | Unknown/unreliable | Reputable proxy | Exact first-party/official/primary source |
| Definition fit | Mismatched/unknown | Partial fit | Exact variable/population/unit |
| Scope fit | Wrong geo/time/segment | Transferable with caveat | Exact geo/time/segment |
| Method validity | Opaque/invalid | Adequate | Strong, transparent, fit-for-purpose |
| Sample/coverage | Unknown/tiny biased | Useful bounded sample | Strong frame/coverage for claim |
| Directness | Attention/opinion proxy | Observed workflow/choice | Payment/retention/outcome/economics |
| Independence | Duplicate/syndicated | Partially independent | Independent evidence mode/source |
| Freshness | Obsolete/unknown | Acceptable | Current for decision horizon |
| Reproducibility | No query/data/method | Partially inspectable | Fully traceable and repeatable |
| Consistency | Strong credible contradiction | Mixed | Converges incl. negative-case review |

Compute a diagnostic normalized strength only if weights are declared. Default critical weights emphasize definition/scope/method/directness. Keep a `coverage penalty` when the source cannot observe a material portion of the population.

Suggested labels:

- `VERY_WEAK` < 0.30
- `WEAK` 0.30–0.49
- `MODERATE` 0.50–0.69
- `STRONG` 0.70–0.84
- `VERY_STRONG` >= 0.85

These labels apply to a specific claim, not the entire idea.

## 3. Hypothesis confidence

Each hypothesis records:

- prior confidence and source;
- support items and strength;
- negative evidence and strength;
- conflicts and alternative explanations;
- evidence-level ceiling (E0–E10);
- current confidence with rationale;
- status: `UNTESTED`, `SUPPORTED`, `PARTIALLY_SUPPORTED`, `FALSIFIED`, `AMBIGUOUS`, `SUPERSEDED`;
- what next evidence could cross a decision threshold.

Do not average all evidence items equally. Independent behavioral evidence can dominate many low-quality mentions. One strong falsifier may be decisive for a universal claim.

## 4. Opportunity dimensions

For portfolio/decision synthesis, score 0–5 with evidence IDs and confidence:

1. Problem severity/stakes
2. Frequency/prevalence
3. Existing spend/effort and willingness to act
4. Beachhead clarity/reachability
5. Segment size/growth/timing
6. Alternative dissatisfaction/switching window
7. Value magnitude and measurable outcome
8. Solution adoption/workflow fit
9. Differentiation/credibility
10. Pricing/WTP and gross-margin potential
11. Retention/expansion potential
12. Channel access and sales-cycle viability
13. Competitive intensity/platform threat
14. Defensibility/control point/network/data/brand advantage
15. Technical/operational feasibility
16. Regulatory/privacy/ethical viability
17. Capital/capacity/time-to-proof
18. Team/founder right-to-win
19. Downside/reversibility
20. Evidence maturity and reproducibility

Weights depend on business type and decision. Publish weights before scoring. Add a confidence multiplier only as a visible diagnostic, not a secret formula.

## 5. Kill gates

Kill gates override aggregate attractiveness. Define before research. Examples:

- no evidence of a consequential recurring problem in eligible segment;
- target segment cannot be lawfully/reliably reached;
- economic buyer lacks budget/authority or procurement makes model infeasible;
- achievable price cannot support gross margin/cost-to-serve/capital needs;
- critical data cannot be obtained/processed lawfully or reliably;
- dominant alternative is free/bundled/good-enough with no credible switch trigger;
- regulated approval/timing exceeds strategy/capital tolerance;
- experiment shows demand below threshold at viable offer/price after a valid test;
- pilot cannot produce the promised outcome;
- retention/renewal below category-specific minimum;
- marketplace cannot reach liquidity within capital/geo constraints;
- material harm/ethical exposure cannot be mitigated;
- team lacks a required non-substitutable capability/access and cannot obtain it.

Each gate needs metric, threshold, evidence, scope, owner, timing, and override policy. Overrides require explicit authorized decision and risk acceptance; do not alter evidence.

## 6. Recommendation rules

### GO

Use only when:

- decision scope is narrow and explicit;
- critical hypotheses cross thresholds;
- depth-appropriate behavior/commitment exists;
- no kill gate fails;
- key economics/channel constraints are plausible under downside sensitivity;
- critical risks have controls/experiments;
- Blueprint can act without hidden market assumptions.

Attach conditions and expiry. `GO` usually means proceed to the next staged investment/Blueprint, not invest without limit.

### PIVOT

Use when original critical hypotheses fail/underperform but a distinct segment/problem/promise/model/channel has stronger traceable evidence. State what changed and what is not preserved. A pivot is a new bounded thesis, not cosmetic feature change.

### HOLD

Use when the opportunity may be attractive but a timing, regulation, distribution, access, capacity, capital, dependency, or evidence condition makes proceeding now irrational. Define trigger/monitoring and maximum hold horizon.

### NO-GO

Use when a kill gate fails credibly, downside dominates, required value/economics are structurally implausible, or the opportunity cost is superior elsewhere. Preserve reusable learning and name what evidence could legitimately reopen the decision.

### INSUFFICIENT EVIDENCE

Use when available evidence cannot support a responsible decision, key source access is missing, primary/behavioral tests remain unrun, or methods/samples cannot answer the question. Provide the minimum next evidence and expected decision value.

## 7. Portfolio comparison

Compare opportunities on normalized definitions and the same horizon/capital stage. Show:

- bounded opportunity statement;
- raw dimension scores and evidence confidence;
- expected value/upside range;
- capital/time to next proof;
- irreversible downside;
- key kill gate;
- next experiment and cost;
- strategic option value;
- portfolio dependencies/cannibalization;
- recommendation.

Do not compare one fully researched opportunity with one idea-level opportunity as if confidence were equal. Include evidence maturity as a separate axis.

## 8. Decision memo

```yaml
recommendation_id: REC-001
decision: GO|PIVOT|HOLD|NO-GO|INSUFFICIENT_EVIDENCE
decision_owner: ""
decided_at: ""
valid_until: ""
scope:
  segment: ""
  problem_jtbd: ""
  promise: ""
  geography: []
  business_model: ""
  channel: ""
  stage_capital_cap: ""
confidence: 0.0
evidence_level_achieved: E0-E10
critical_hypotheses: []
strongest_support: []
strongest_negative_evidence: []
market_model_range: ""
economics_range: ""
conditions: []
kill_criteria: []
reversal_evidence: []
next_evidence: []
risks_accepted: []
explicit_exclusions: []
blueprint_eligible: false
rationale: ""
```

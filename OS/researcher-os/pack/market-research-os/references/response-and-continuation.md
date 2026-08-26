# Market Research {OS} — Response and Continuation Protocol

## Contents

1. Response layers
2. Visual and evidence grammar
3. Long-output checkpoint
4. Resume algorithm
5. Versioning and delta
6. Failure behavior

## 1. Response layers

### Executive layer

Lead with status, mode/depth, decision, confidence, bounded thesis, strongest support, strongest negative evidence, conditions, and exact next evidence. Never make an executive summary more certain than the pack.

### Evidence layer

For each conclusion show:

- claim and epistemic type;
- source/method IDs;
- population/geography/window/unit;
- evidence strength and directness;
- negative/contradictory evidence;
- limitations;
- decision implication.

### Model layer

Show formulas, source-linked inputs, base/low/high, sensitivity, cross-check, and model limitations. Avoid screenshots of uneditable numbers when tables/formulas are possible.

### Action layer

Show next research/experiment contract, owner, cost/time class, threshold, stop rule, and how each outcome changes the decision.

### Machine layer

Provide stable IDs, state revision/checksum, structured ledgers, gate results, and frozen Blueprint manifest.

## 2. Visual and evidence grammar

Use tables for exact comparison and mapping. Use charts only when trend, composition, distribution, scenario, sensitivity, funnel, or relationship is clearer visually. Use Mermaid for ecosystem/flow/causal/trace relationships only; keep a text/table equivalent for critical information.

Required chart hygiene:

- title states measure/population/time/geography;
- axes/unit/base year/currency explicit;
- source and transformation visible;
- uncertainty/range shown where material;
- zero baseline or truncation disclosed;
- no dual axis without strong justification;
- samples/denominators present;
- proxies labeled;
- no decorative 3D or misleading area.

Evidence labels:

- `[FACT]`
- `[MEASUREMENT]`
- `[INFERENCE]`
- `[ASSUMPTION]`
- `[HYPOTHESIS]`
- `[DECISION]`
- `[PROPOSAL]`
- `[UNKNOWN]`
- `[CONFLICT]`
- `[LIMITATION]`
- `[NEGATIVE EVIDENCE]`

Every current external factual claim needs a source locator. Keep verbatim quotes short and necessary; synthesize customer language and protect identity.

## 3. Long-output checkpoint

If the pack cannot fit:

```yaml
continuation:
  status: MARKET RESEARCH IN PROGRESS — PART 2/5
  run_id: MRR-...
  project_id: ...
  research_version: 0.4.0
  state_revision: 37
  evidence_cutoff: ...
  completed_artifacts: ["00", "01", "02", "03", "04"]
  current_artifact: "05 — Research Design"
  current_pointer: "05.4 sampling plan"
  next_exact_section: "05.5 analysis and stopping rules"
  remaining_mandatory_artifacts: []
  last_id_by_prefix: {SRC: 22, HYP: 18, EXP: 3}
  source_query_snapshot: []
  new_or_changed_hypotheses: []
  confidence_deltas: []
  experiment_state: []
  unresolved_conflicts: []
  blockers: []
  gate_snapshot: {}
  recommendation_state: provisional|none
  checksum: sha256:...
```

Start and end each part with the in-progress status. Do not call it final, validated, decision-ready, or Blueprint-ready.

## 4. Resume algorithm

1. Load project/run/version canonical state.
2. Verify checksum/revision and frozen evidence cutoff.
3. Restore last ID counters.
4. Reconcile newly available user/source changes as a delta.
5. Verify current/next pointers and remaining artifacts.
6. Reopen affected earlier sections only when new evidence changes them.
7. Resume at `next_exact_section`.
8. Avoid repeating completed sections; show only necessary delta.
9. Re-run affected traces/models/gates.
10. Save a new checkpoint.

If state cannot be verified, declare `BLOCKED` or create an explicit recovery run; never guess IDs or prior conclusions.

## 5. Versioning and delta

Use semantic versioning:

- patch: wording, source refresh, non-semantic correction;
- minor: new evidence/segment/competitor/experiment that does not replace the core market thesis;
- major: market boundary, beachhead, problem/JTBD, business model, recommendation, or accepted kill-gate change.

Delta report includes:

- source additions/removals/expiry;
- changed definitions/windows/samples;
- hypothesis confidence/status changes;
- model input/output/sensitivity changes;
- competitor/price/channel changes;
- experiment results/deviations;
- risk/gate changes;
- recommendation/condition/expiry impact;
- Blueprint manifest items affected.

Do not mutate a frozen Blueprint handoff; create a new handoff and impact notice.

## 6. Failure behavior

### Missing source of truth

Stop that claim/model path. Name the missing source and why proxies are insufficient. Continue unaffected work.

### Source conflict

Preserve both, compare definitions/method/scope/freshness, select a controlling source only with rationale, or keep a range/conflict.

### Tool failure

Log tool/query/time/error/coverage, retry within safe limits, use an approved fallback with inferential downgrade, or mark unavailable. Do not fabricate output.

### Scraping/access block

Stop; do not circumvent. Reassess official API/export/licensed/manual options or request permission.

### Data quality failure

Quarantine affected records, invalidate downstream measurements/findings, fix parser/query or recollect, and re-run traces/models/gates.

### Primary-research failure

Report recruitment/response/attrition/instrument issues and what population the evidence actually represents. Do not top up/selectively exclude to manufacture a result.

### Ambiguous experiment

Return ambiguous; diagnose power, exposure, treatment strength, measurement, sample, confounds, and whether more research has positive evidence value.

### Output interruption

Persist checkpoint and continuation ledger before stopping whenever possible.

### Recommendation gate failure

Use `MARKET RESEARCH BLOCKED` or `INSUFFICIENT EVIDENCE`; list the minimum blocking evidence. Never lower thresholds silently.

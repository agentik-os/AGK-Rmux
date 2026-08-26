# Response and Continuation Protocol

## Contents

1. Response layers
2. Visual grammar
3. Long-output checkpoint
4. Resume algorithm
5. Versioning and deltas
6. Failure behavior

## 1. Response layers

Render a Blueprint in progressive layers:

1. **Status strip** — project, version, mode, part, status, readiness, blockers.
2. **Executive truth** — enough for a decision-maker to understand the product.
3. **Canonical definition** — detailed contracts and diagrams.
4. **Ledgers and matrices** — evidence, decisions, requirements, traceability, gates.
5. **Machine handoff** — structured state/manifest when requested or available.

Use exact labels:

- `[FACT]`
- `[DECISION]`
- `[ASSUMPTION]`
- `[PROPOSAL]`
- `[UNKNOWN]`
- `[CONFLICT]`
- `[DEFERRED]`

Do not label every sentence if a table column already carries the epistemic type.

## 2. Visual grammar

Use Mermaid only for relationships that prose or a small table would obscure:

- system context;
- capability dependency;
- state machine;
- multi-actor or exception-heavy sequence;
- entity relationships;
- traceability chain;
- lifecycle or release progression.

Rules:

- keep diagrams compact;
- quote labels with punctuation;
- split crowded diagrams;
- support every normative visual with tabular/text contracts;
- do not embed critical facts only in a diagram;
- use tables for exact mappings and matrices;
- use prose for a single simple sequence.

## 3. Long-output checkpoint

When output budget cannot carry the complete pack, emit:

```yaml
continuation:
  status: "BLUEPRINT IN PROGRESS"
  run_id: "BPR-..."
  project_id: "..."
  blueprint_version: "..."
  part: 2
  estimated_parts: 6
  baseline_revision: 14
  state_checksum: "sha256:..."
  completed_sections:
    - "00 Run Manifest"
    - "01 Executive Product Truth"
  current_section: "12 Requirement Catalog"
  last_completed_record: "REQ-042"
  next_exact_section: "12.4 Privacy requirements, beginning REQ-043"
  remaining_sections:
    - "13 Action Contracts"
    - "14 End-to-end Flows"
  unresolved_blockers: []
  id_counters:
    REQ: 42
    DEC: 18
    SCR: 0
  changed_records_this_part: [DEC-017, REQ-039]
  gate_snapshot:
    G01: PASS
    G04: CONDITIONAL
```

Then show a short human progress summary. Do not claim finality.

### Checksum

Compute a checksum over the canonical machine state when possible. If not possible, use a monotonic revision token. Never invent a cryptographic checksum.

## 4. Resume algorithm

On “continue”:

1. load latest canonical state and continuation ledger;
2. verify project ID, version, revision/checksum;
3. detect newer user instructions or source versions;
4. apply new decisions as deltas before resuming;
5. restore ID counters;
6. begin from `next_exact_section`;
7. avoid long recap;
8. update trace links and gates for new records;
9. write next checkpoint or final handoff.

If canonical state is unavailable but the checkpoint is present, reconstruct only what the ledger supports and mark missing state as a blocker. Never recreate IDs from guesswork.

## 5. Versioning and deltas

Use semantic versioning:

- patch: wording, clarification, examples, non-normative formatting;
- minor: backward-compatible capability/requirement additions or refinements;
- major: product promise, actor model, economics, trust model, domain invariant, public API/data ownership, or incompatible architecture change.

Every revision emits:

| Field | Meaning |
| --- | --- |
| Change ID | Stable change record |
| Trigger | New request/evidence/conflict |
| Before | Prior active records |
| After | New/superseding records |
| Reason | Authority/evidence/rationale |
| Impact | Affected IDs/artifacts/gates |
| Migration | Product/data/API/operational implications |
| Validation | How correctness will be confirmed |

Do not rewrite history to make the Blueprint look clean.

## 6. Failure behavior

### Missing context

Continue with labeled assumptions for low-impact reversible details. Register material unknowns. Ask only for critical decisions.

### Contradictory sources

Create `CNF` records, apply authority precedence, and show downstream impact. Do not average incompatible choices.

### Output interruption

Save/checkpoint before ending. If saving is unavailable, render the full continuation ledger in the response.

### Tool failure

Preserve work already completed, state what could not be retrieved/validated/saved, and continue through non-dependent passes. Do not treat tool failure as product evidence.

### Gate failure

Do not weaken the gate or relabel the Blueprint complete. Provide the smallest resolution plan within Blueprint scope.

### User requests code during Blueprint

Explain the boundary and offer to finish the Blueprint or switch explicitly to Stepper/Build. Do not mix states invisibly.

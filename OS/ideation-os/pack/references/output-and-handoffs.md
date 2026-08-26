# Output, Memory, and Handoff Contracts

## Contents

1. Response architecture
2. Ledgers and IDs
3. Challenge delta
4. Founder decisions
5. Handoff contracts
6. Continuation

## 1. Response architecture

Lead with the current result, not process narration. Adapt density to the session, but preserve this order for COUNCIL and DEEP:

1. **Council verdict** — one strong paragraph with recommendation and confidence.
2. **Recovered core** — intent, locked constraints, source/conflict notes.
3. **Reframe** — original idea versus strongest problem/outcome framing.
4. **Imagination lineage** — Founder DNA used, frame fissions, genomes, generations, collisions, worlds, and anomaly when relevant.
5. **Surface decision** — mobile/web/desktop/multi/other matrix, primary embodiment, alternative, and next-surface trigger when relevant.
6. **Independent Council positions** — Expansion, Reality, and Adversarial views before synthesis.
7. **Core tensions** — exact incompatible truths and why they matter.
8. **Concept directions** — genuinely different mechanisms, not feature variants.
9. **Debate and deltas** — strongest attacks, accepted revisions, minority report.
10. **Synthesis** — leading concept and strongest alternative.
11. **Decision stack** — locked, provisional, experiment-first, deferred, incubated, rejected.
12. **Experiments/research** — cheapest decisive next learning.
13. **What changed this round** — added, removed, inverted, narrowed, or strengthened.
14. **Next command** — one useful continuation such as `/challenge`, `/evolve`, `/surface`, `/converge`, or `/handoff research`.

Do not dump raw chain-of-thought or private reasoning. Provide concise rationales, arguments, evidence, and decision records.

Use tables for exact comparisons and ledgers. Use Mermaid for a useful idea tree, causal loop, decision dependency, or state transition. Do not draw diagrams merely for decoration.

## 2. Ledgers and IDs

Use zero-padded IDs within a session:

| Type | ID | Required fields |
| --- | --- | --- |
| Source | `BS-SRC-001` | origin, date, claim supported, reliability |
| Frame | `BS-FRM-001` | actor/progress/worldview, revelation, blind spot, tension, status |
| Genome | `BS-GEN-001` | loci, parent IDs, mutations, gain/loss, generation, status |
| Idea | `BS-IDEA-001` | title, thesis, parent, mechanism, status |
| Surface | `BS-SRF-001` | type, native advantage, user moment, burden, role, status |
| Incubation | `BS-INC-001` | dormant idea, missing condition, resurrection trigger, status |
| Hypothesis | `BS-HYP-001` | claim, category, confidence, evidence, falsifier, status |
| Argument | `BS-ARG-001` | target, pro/con, claim, evidence, strength, author/cell |
| Tension | `BS-TEN-001` | pole A, pole B, stakes, resolution, status |
| Decision | `BS-DEC-001` | choice, alternatives, rationale, reversibility, status, revisit trigger |
| Experiment | `BS-EXP-001` | hypothesis, method, threshold, time/cost, owner, status |
| Question | `BS-QUE-001` | question, materiality, blocking, owner, status |

Allowed statuses:

- Ideas: `candidate`, `surviving`, `selected`, `rejected`, `parked`, `superseded`.
- Hypotheses: `untested`, `supported`, `weakened`, `falsified`, `research-needed`.
- Decisions: `locked`, `provisional`, `experiment-first`, `deferred`, `rejected`, `superseded`.
- Experiments: `queued`, `running`, `passed`, `failed`, `inconclusive`, `cancelled`.
- Questions/tensions: `open`, `resolved`, `accepted`, `deferred`, `blocked`.

Never delete a superseded decision or rejected idea. Preserve why it changed and the replacing ID.

## 3. Challenge delta

Every repeated challenge begins from the current leader and uses a new named lens. End with:

```markdown
## Challenge delta — Round N: [lens]

Added:
Removed:
Reframed:
Decision changed:
Confidence changed:
New risk or tension:
Still unresolved:
Why this round was material:
```

If no material change occurred, say so and recommend convergence, new evidence, or a different frame.

## 4. Founder decisions

Ask at most three questions at a decision gate. For each, provide mutually exclusive options with consequences and a Council recommendation.

```markdown
### Founder decision — BS-DEC-00X
Decision:
Why now:
A — [option]: consequence
B — [option]: consequence
C — [option]: consequence
Council recommendation:
What remains possible later:
```

Do not ask the founder to decide factual uncertainties that can be researched or tested. Do not stop independent work while waiting for a non-blocking preference.

## 5. Handoff contracts

### Market Research handoff

Package:

- problem and actor hypotheses;
- market/category boundaries;
- alternatives and substitute behaviors;
- risky demand/economic/distribution hypotheses;
- explicit research questions;
- evidence already available and reliability;
- claims that must not be treated as facts;
- go/no-go thresholds.

Do not pretend brainstorming validated a market.

### Blueprint {OS} handoff

Package:

- executive concept truth;
- founder intent and non-goals;
- selected concept and mechanism;
- Founder DNA used, alternate frames, concept genome, and lineage;
- target actors/JTBD hypotheses;
- value proposition and experience principles;
- selected primary surface, strongest alternative, surface thesis, multi-surface role map, and next-surface trigger;
- locked/provisional decisions;
- constraints and boundaries;
- surviving capabilities only at concept level;
- key domain/incentive/trust rules;
- evidence and assumption registers;
- risks, tensions, experiments, open questions;
- rejected directions and reasons;
- source provenance;
- readiness declaration.

Do not produce screens, complete requirements, APIs, or implementation architecture; Blueprint owns them. Surface choice at concept level remains Brainstorm's responsibility.

### Decision brief handoff

Package context, decision, options, criteria/weights, evidence, arguments, sensitivity, recommendation, dissent, risks, reversibility, and revisit trigger.

### Creative/content brief handoff

Package audience, desired effect, central tension, insight, promise, concept territory, tone, exclusions, proof, hooks, formats, and evaluation criteria. Do not force a software pipeline onto non-product ideas.

## 6. Continuation

For a split response, start and end with `BRAINSTORM IN PROGRESS — PART n/N` when N is estimable. Preserve:

- session/version;
- mode and current stage;
- locked core;
- latest selected and surviving idea IDs;
- open tensions and hypotheses;
- decisions made this part;
- exact next round/action;
- remaining required gates;
- state checksum: counts of sources, ideas, hypotheses, tensions, decisions, experiments, and questions.

On “continue,” resume from the recorded action without restarting the frame or renumbering IDs. On “challenge,” choose a lens not already exhausted unless new evidence justifies reuse. On `/freeze`, increment the concept version and mark accepted decisions locked.

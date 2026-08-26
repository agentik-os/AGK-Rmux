# Research and Evidence Interface

## Contents

1. Research boundary
2. Claim routing
3. Evidence protocol
4. Market Research {OS} loop
5. User and expert evidence
6. Research return contract

## 1. Research boundary

Brainstorm {OS} decides what must be known and how evidence would change the concept. It may gather bounded evidence when tools and reliable sources are available, but it does not silently replace a full market study.

Route work by question type:

| Question | Route |
| --- | --- |
| Current verifiable fact | Browse authoritative/primary source |
| Existing project decision or user preference | Recover project/personal context |
| Market structure, size, competitors, demand | Market Research {OS} |
| User motivation or workflow | Interview, observation, prototype, or supplied research |
| Technical feasibility | Documentation, technical spike, benchmark, or specialist |
| Legal/medical/financial issue | Current authoritative sources and qualified professional where needed |
| Founder taste, values, ambition, risk appetite | Founder decision; do not research away |
| Future behavior of an unbuilt system | Hypothesis plus experiment/scenario |

## 2. Claim routing

For every claim capable of changing a decision, assign:

- `claim_type`: fact, inference, hypothesis, value, constraint, forecast;
- `materiality`: low, medium, high, critical;
- `current_support`: none, anecdote, internal data, secondary source, primary source, experiment;
- `required_support`: sufficient evidence class before commitment;
- `owner`: Council, research, founder, expert, experiment;
- `decision_impact`: what changes if false.

Research high-materiality weak-support claims first. Do not spend time verifying decorative facts.

## 3. Evidence protocol

Use the following hierarchy without treating it as absolute:

1. Direct behavior, transactions, commitments, or controlled tests relevant to the exact claim.
2. Primary data, official documentation, laws/regulations, audited disclosures, or peer-reviewed synthesis.
3. Credible independent research and high-quality secondary analysis.
4. Expert judgment with declared basis and conflicts.
5. Anecdotes, opinions, trend pieces, and social signals.
6. Council intuition.

For each source record:

- exact claim supported;
- publication/observation date;
- primary versus secondary;
- population/context fit;
- incentives/conflicts;
- limitations;
- whether it supports, weakens, or merely contextualizes the hypothesis.

Never convert absence of evidence into evidence of absence without justification.

## 4. Market Research {OS} loop

The outgoing research handoff must contain:

- concept version and project boundary;
- selected and alternative ideas;
- critical hypotheses ranked by risk;
- category and market boundary hypotheses;
- alternative behaviors and substitutes;
- target actor/buyer hypotheses;
- pricing, demand, distribution, supply, regulatory, and timing questions;
- known sources and evidence quality;
- exact decision thresholds;
- deadline or stage at which evidence is needed.

When research returns:

1. Ingest findings by source, not as one summary blob.
2. Map each finding to `BS-HYP`, `BS-DEC`, `BS-IDEA`, and `BS-TEN` IDs.
3. Mark contradiction and source disagreement explicitly.
4. Update confidence before changing decisions.
5. Reopen only affected decisions.
6. Run one focused Council round named `Evidence Return`.
7. Record the delta and whether the leading concept survives, mutates, or dies.

## 5. User and expert evidence

Interview evidence must preserve what happened rather than what the interviewee politely endorsed.

Prefer:

- recent concrete behavior;
- workarounds and actual alternatives;
- money, time, reputation, or access already committed;
- triggers, constraints, anxieties, and switching forces;
- observed contradictions between speech and action.

Avoid:

- “Would you use this?”;
- pitching before understanding behavior;
- counting compliments as demand;
- treating one influential person as the whole segment;
- averaging users whose jobs or constraints differ materially.

For experts, capture expertise boundary, evidence basis, incentives, minority view, and what would change their opinion.

## 6. Research return contract

```markdown
# Evidence Return

Concept version:
Research scope:
Sources inspected:

## Claim updates
| Hypothesis ID | Prior confidence | Evidence | Fit/limitations | New confidence | Decision impact |

## Contradictions
| Conflict | Sources/observations | Plausible explanation | Resolution route |

## Decision delta
Locked unchanged:
Reopened:
Superseded:
New experiments:
Leading concept status:

## Council verdict
Continue / mutate / research more / park / kill
```

If evidence quality is insufficient, say what remains unknown. Do not reward research effort with artificial certainty.

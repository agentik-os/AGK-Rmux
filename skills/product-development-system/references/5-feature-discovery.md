# 5. Feature Discovery

Before development, a feature passes a real discovery phase. Recorded in the feature file's
`discovery:` block.

## Discovery checklist (answer each, with a note)
- Is the problem real?
- Is it frequent?
- Is it important enough?
- Do users want a solution?
- Does a solution already exist?
- Is the proposed solution better?
- Is it technically feasible?
- Is it economically viable?
- Is it consistent with the vision (ref 1)?

A "no" or "unknown" on problem/importance/want means the next step is an **experiment**, not code.

## Evidence
A feature must carry evidence, not opinion:
interviews · support tickets · customer requests · analytics · screen recordings ·
market studies · user tests · competitive benchmark · pre-orders · experiment results.

Attach the evidence (or its link) in the feature file. No evidence => the feature stays in
`Discovery`, and the honest report says "unvalidated".

## Confidence score
```
Problem confidence:   80%
Solution confidence:  55%
Business confidence:  70%
Technical confidence: 90%
Overall confidence:   74%   (the aggregate the agent reports)
```

## How the agent uses it
- Compute and RECORD the four confidences + overall. This is the honesty gate (L1/L2): a low
  solution confidence with high technical confidence means "we can build it, we're not sure it's
  right" — the correct move is an experiment, and you say so rather than proceeding to build.
- Overall confidence feeds RICE/ICE `Confidence` in prioritization (ref 6).
- Discovery output is what lets a feature leave `Discovery -> Validation -> Specification`.

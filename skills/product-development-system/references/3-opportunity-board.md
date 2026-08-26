# 3. Opportunity Board

Before creating a feature, formalize the opportunity. One file per opportunity at
`agentic/product/opportunities/<slug>.md`.

## Opportunity object (front-matter fields)
```
Opportunity
├── User problem
├── Context
├── Frequency
├── Severity
├── Existing solution
├── Current frustration
├── Business opportunity
├── Evidence
├── User segment
├── Confidence
└── Potential solutions
```

## Opportunity Solution Tree
Model the space before committing to one solution:
```
Business Outcome
├── Opportunity A
│   ├── Solution A1
│   ├── Solution A2
│   └── Experiment A3
├── Opportunity B
│   ├── Solution B1
│   └── Solution B2
└── Opportunity C
```

## The discipline this enforces
Do NOT go:
```
We have an idea  ->  Let's build it
```
Go:
```
Outcome  ->  Opportunity  ->  Possible solutions  ->  Experiment  ->  Validated feature
```

## How the agent uses it
- Every Feature (ref 4) MUST link to a parent Opportunity (`related: [opportunity:<slug>]`). A
  feature with no opportunity is a discovery gap — create the opportunity first, backfilling
  problem/frequency/severity/evidence.
- When one opportunity has several candidate solutions, keep them as branches on the tree and let
  Discovery (ref 5) + an experiment pick the winner, rather than assuming the first solution.
- `Confidence` here feeds the feature's business/problem confidence in Discovery.

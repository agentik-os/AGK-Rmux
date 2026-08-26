# 1. Vision Board

Defines what the product wants to become and why it must exist. One per product, at
`agentic/product/vision.md` (+ `north-star.md` for the metric layer).

## Section: Product Vision
- **Product name**
- **One-sentence summary**
- **Core problem** — the single problem that justifies the product
- **Promised transformation** — before -> after for the user
- **Target user**
- **Differentiator** — why us, not the alternative
- **Ambition at 1, 3, 5 years**

## Section: Mission
- **What the product does today**
- **For whom**
- **How**
- **Why now** — the timing thesis

## Section: Product Principles
Design/decision principles that arbitrate trade-offs. Examples:
- Simple before powerful
- AI-assisted, human-controlled
- Automation without opacity
- One source of truth
- Action before information

When two options conflict, the principle wins — cite it in the decision.

## Section: Strategic Pillars
```
Vision
├── Product
├── Technology
├── Customer
├── Business
├── Brand
└── Distribution
```
Each pillar carries: **objective · hypothesis · initiatives · indicators · risks · owner · time horizon**.

## Section: North Star (`north-star.md`)
- **North Star Metric** — the one number that best proxies delivered value
- **Input metrics** — the levers the team moves directly
- **Output metrics** — results the inputs produce
- **Leading indicators** — early signals
- **Lagging indicators** — confirmed outcomes

## Section: Anti-Vision
What the product must NOT become. Default guards:
- too complex
- dependent on external services
- expert-only
- hard to configure
- opaque in its decisions

## How the agent uses it
- Every Feature and Opportunity links to the Strategic Pillar it advances (`related: [pillar:...]`).
- "Strategic alignment" in prioritization (ref 6) is scored AGAINST this board.
- If a proposed feature moves the product toward the Anti-Vision, flag it as a vision conflict
  before scoping. The Vision Board is the tie-breaker, not the loudest opinion.

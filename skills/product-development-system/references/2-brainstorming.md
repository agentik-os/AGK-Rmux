# 2. Brainstorming System

Not a notes page — a system to generate, cluster, enrich, and qualify ideas.
One file per idea at `agentic/product/ideas/<slug>.md`.

## Idea types
Product idea · Feature idea · UX improvement · Technical improvement · Automation ·
AI capability · Growth experiment · Monetization idea · User problem · Bug opportunity ·
Integration idea · Content idea.

## Idea structure (front-matter fields)
```
Idea
├── Title
├── Description
├── Problem addressed
├── Target user
├── Expected impact
├── Source
├── Assumptions
├── Risks
├── Related features
├── Related workflows
├── Evidence
├── Priority
└── Status
```

## Sources
founder · team · customer interview · support request · analytics · competitor ·
AI recommendation · sales call · internal observation · technical opportunity.

## Brainstorming modes
- **Free Brainstorm** — open idea creation, no filter.
- **Problem-Based Brainstorm**:
  `Problem -> Possible causes -> Possible solutions -> Alternative approaches -> Risks -> Experiments`
- **AI Brainstorm** — the agent generates, on demand: similar ideas · opposite ideas · simple
  version · premium version · automated version · no-code version · mobile version · collaborative
  version · monetization ideas · differentiation ideas.
- **Collaborative Brainstorm** — each participant may: propose · comment · vote · challenge ·
  merge · enrich · reject.

## Clustering
Ideas are grouped automatically by: problem · persona · workflow · objective · type · impact ·
complexity. When you hold 5+ ideas, cluster before qualifying — never qualify a flat list.

## Idea Canvas (use before promoting an idea)
Problem · Opportunity · Solution · Target user · User value · Business value ·
Technical feasibility · Evidence · Risks · Next experiment.

## Statuses (lifecycle)
`Captured -> Exploring -> Needs Evidence -> Validated -> Shortlisted -> Converted -> Rejected -> Archived`

An idea only reaches **Validated** with evidence attached. **Converted** means it became a
concrete object:
- a **goal**, an **initiative**, an **epic**, a **feature**, an **experiment**, or a **technical task**.

## How the agent uses it
- Capture every idea surfaced in a mission (don't lose them in chat) as a file with `status: Captured`.
- Before proposing to build, an idea must be at least `Shortlisted` with an Idea Canvas filled.
- A `Validated` idea that is a genuine user problem is promoted to an **Opportunity** (ref 3), not
  straight to a Feature.

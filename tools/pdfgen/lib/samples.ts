import type {
  WhitepaperData,
  AuditData,
  MarketingData,
  DocData
} from "./schemas";

export const sampleWhitepaper: WhitepaperData = {
  template: "whitepaper",
  theme: "agentik",
  eyebrow: "WHITEPAPER · 2026",
  title: "The Architecture of Autonomous Software Teams",
  subtitle:
    "How agentic orchestration replaces project management with self-organizing pipelines.",
  author: "Dafnck Studio",
  date: "May 2026",
  docId: "WP-2026-04",
  brand: "OmegaOS",
  abstract:
    "Autonomous agent teams remove the bottleneck of human coordination. By delegating planning, execution and verification to a hierarchy of specialized agents, organizations compress months of work into hours while preserving — and often improving — quality. This paper documents the architecture, the failure modes and the operating principles that make it work.",
  toc: [
    { index: "01", title: "Why traditional teams plateau", page: 4 },
    { index: "02", title: "The four-tier orchestration model", page: 6, depth: 1 },
    { index: "02.1", title: "Oracle, worker, agent, audit", page: 7, depth: 2 },
    { index: "03", title: "Failure modes and how to survive them", page: 9 },
    { index: "04", title: "Operating principles", page: 12 }
  ],
  sections: [
    {
      index: "01",
      eyebrow: "Context",
      title: "Why traditional teams plateau",
      lead: "Coordination overhead grows quadratically. Agents flip the curve.",
      body: `Most engineering organizations spend more time **coordinating work** than doing the work itself. Each new contributor adds n communication links, n review responsibilities, n contexts to maintain. The math is unforgiving: at fifteen people you spend half your day in sync.

Agent teams break this curve because the coordination layer is *automated*. Plans are produced by a planner, executed by workers, audited by reviewers, and shipped without a standup in sight.

> The right unit of leverage is not the engineer — it is the pipeline.

The remainder of this whitepaper documents how to build that pipeline in practice.`
    },
    {
      index: "02",
      eyebrow: "Architecture",
      title: "The four-tier orchestration model",
      lead: "A clean separation between intent, decomposition, execution and verification.",
      body: `Every autonomous team needs four distinct surfaces:

- **Intent surface** — where the human states a goal in natural language.
- **Decomposition surface** — where the goal becomes a dependency-aware plan.
- **Execution surface** — where workers carry out tasks in parallel.
- **Verification surface** — where auditors challenge the work before it ships.

Skipping any one of these produces predictable pathologies. Without verification, hallucinations compound. Without decomposition, workers thrash on ambiguous goals. Without an intent surface, the system is a tool, not a teammate.`
    },
    {
      index: "03",
      eyebrow: "Failure modes",
      title: "Failure modes and how to survive them",
      lead: "Most failure modes are recoverable if you instrument the right signals.",
      body: `### Premature confidence
Agents confidently produce convincing but wrong outputs. Mitigate by separating the *researcher* and *verifier* roles.

### Context starvation
Workers without enough context guess. Pre-inline the minimum viable brief: scope, constraints, file paths, success criteria.

### Pipeline drift
Each retry edits more files. Mitigate with strict ownership boundaries and per-task scope locks.`
    }
  ]
};

export const sampleAudit: AuditData = {
  template: "audit",
  theme: "agentik",
  eyebrow: "QUALITY ARSENAL · UI/UX",
  title: "UI/UX forensic audit — causio.app",
  subtitle: "Twenty-three phase deep analysis of design coherence, accessibility, and interaction quality.",
  author: "Dafnck Studio",
  date: "May 14, 2026",
  docId: "UIUX-2026-118",
  brand: "OmegaOS",
  overallScore: 73,
  verdict:
    "Causio's interface shows mature visual restraint but stumbles in dense data surfaces. Typography hierarchy fails on tertiary pages, several interactions lack focus states, and the empty-state pattern is reinvented six different ways. None of these are critical, but together they cost the product its premium feel.",
  domains: [
    { label: "Typography", score: 81, weight: 18 },
    { label: "Color & Contrast", score: 76, weight: 12 },
    { label: "Spacing & Rhythm", score: 68, weight: 14 },
    { label: "Component Anatomy", score: 71, weight: 16 },
    { label: "Interaction", score: 64, weight: 16 },
    { label: "Motion", score: 79, weight: 8 },
    { label: "Responsive", score: 85, weight: 8 },
    { label: "Accessibility", score: 58, weight: 8 }
  ],
  kpis: [
    { label: "Pages audited", value: 28, unit: "pages" },
    { label: "Components", value: 142, unit: "scanned" },
    { label: "Findings", value: 17, delta: "−4 vs last audit", deltaTone: "up" },
    { label: "Severity", value: "Medium" }
  ],
  findings: [
    {
      severity: "critical",
      title: "Focus states missing on primary CTAs",
      description:
        "Six call-to-action buttons across the dashboard have no visible :focus-visible style. Keyboard users have no indication of which control is active.",
      recommendation:
        "Add a 2px outline at offset 2px using --accent, applied via :focus-visible only.",
      evidence: ".btn-primary:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }"
    },
    {
      severity: "high",
      title: "Six empty-state patterns coexist",
      description:
        "The product surfaces six distinct empty-state layouts across analogous views: list pages, detail pages and modals. Users learn the language six times.",
      recommendation:
        "Consolidate into one EmptyState primitive with three variants: dense, default, prominent."
    },
    {
      severity: "medium",
      title: "Body text uses three different line-heights",
      description:
        "1.4, 1.5 and 1.6 appear in shipped CSS for body paragraphs depending on the parent container.",
      recommendation: "Standardize on 1.55 across all body text via design token."
    },
    {
      severity: "low",
      title: "Two competing icon weights",
      description:
        "Heroicons regular and outline are mixed in the navigation. The eye reads inconsistency before it reads function."
    }
  ],
  actionPlan: [
    { when: "Week 1", title: "Focus states + accessibility quick wins", desc: "Address the critical finding and the four highest-impact contrast issues." },
    { when: "Week 2", title: "Empty-state consolidation", desc: "Build EmptyState primitive, migrate six call-sites, kill old patterns." },
    { when: "Weeks 3–4", title: "Typography token pass", desc: "Single source of truth for line-height, size scale, weight. Codemod existing pages." },
    { when: "Week 5", title: "Re-audit", desc: "Re-run the same 23-phase protocol. Target overall score 88+." }
  ]
};

export const sampleMarketing: MarketingData = {
  template: "marketing",
  theme: "agentik",
  eyebrow: "Q2 STRATEGY",
  title: "Performance review and Q3 plan",
  subtitle:
    "Where the demand came from, where the budget went, where it should go next quarter.",
  author: "Dafnck Studio",
  date: "May 2026",
  docId: "MKT-Q2-2026",
  brand: "DAFNCK",
  executiveSummary:
    "Q2 delivered 41% growth in pipeline and a 22% drop in cost per qualified lead — driven almost entirely by long-form content and a single high-performing LinkedIn motion. Paid social underperformed every quarter target. We propose reallocating 30% of paid budget to content and partnerships.",
  kpis: [
    { label: "MQLs", value: 612, delta: "+41%", deltaTone: "up" },
    { label: "CPL", value: "€38", delta: "−22%", deltaTone: "up" },
    { label: "CAC payback", value: "5.2", unit: "months" },
    { label: "Win rate", value: "23%", delta: "+3 pts", deltaTone: "up" }
  ],
  budgetAllocation: [
    { label: "Content", value: 38 },
    { label: "LinkedIn", value: 24 },
    { label: "Paid search", value: 18 },
    { label: "Events", value: 12 },
    { label: "Partner", value: 8 }
  ],
  channelPerformance: [
    { label: "Content", value: 100 },
    { label: "LinkedIn", value: 86 },
    { label: "Referral", value: 71 },
    { label: "Paid search", value: 42 },
    { label: "Paid social", value: 19 }
  ],
  personas: [
    {
      name: "Léa, Head of Growth",
      role: "Decision Maker · SaaS",
      age: "32–38",
      income: "€90–130k",
      pains: [
        "Cannot ship faster than her ops team can keep up",
        "Reports to a board that wants attribution today",
        "Burned by an MQL-as-vanity-metric agency"
      ],
      goals: [
        "Bring CPL under €40 while preserving lead quality",
        "Centralize attribution across four ad accounts",
        "Free up Friday afternoons for strategic work"
      ],
      channels: ["LinkedIn", "Newsletter", "Podcast"]
    },
    {
      name: "Marc, Founder",
      role: "Buyer · 10–50 FTE",
      age: "38–48",
      income: "Equity-heavy",
      pains: [
        "Built marketing on a side desk, now it stalls",
        "Hiring a CMO feels premature and expensive"
      ],
      goals: [
        "Stop being the marketing department",
        "Predictable demand by Q4"
      ],
      channels: ["X", "Founder networks", "Email"]
    }
  ],
  recommendations: [
    "Reallocate 30% of paid social budget into long-form content production for the back half of the year.",
    "Hire a part-time LinkedIn ghostwriter for two named operators — multiplier on the highest-performing channel.",
    "Kill broad-match Google campaigns; concentrate on the seven highest-converting clusters.",
    "Launch a partner co-marketing motion with three complementary tools.",
    "Run the next pricing experiment in August, after the summer demand dip."
  ]
};

export const sampleDoc: DocData = {
  template: "doc",
  theme: "agentik",
  eyebrow: "PROTOCOL",
  title: "OMAD protocol — clean rebuild",
  subtitle: "A one-meal-a-day discipline calibrated for O-negative metabolism.",
  author: "OmegaOS",
  date: "May 2026",
  docId: "PROT-OMAD-04",
  brand: "ONE-LIFE",
  body: `## The window

Eat between **15:00 and 20:00**. Outside that window: water, black coffee, no exception.

## What goes in

- Animal protein anchor: 180–240g, prioritized over carbs.
- Two fistfuls of vegetables, cooked, with quality fat.
- One serving slow carbs **only if** training day.
- Zero refined sugar. Honey is sugar in disguise.

## What it produces

| Week | Energy | Sleep | Notes |
|------|--------|-------|-------|
| 1    | Low    | Stable | Adaptation. Expect headaches. |
| 2    | Climbing | Deepens | Hunger flattens. |
| 4    | Steady high | 7h refreshing | Body fat trends down. |

> The discipline is not the food. The discipline is the window.

## Why it matters

OMAD removes a decision and frees a morning. Compound that across a year and you have recovered three weeks of focused time.`
};

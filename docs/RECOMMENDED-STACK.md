# OmegaOS — Recommended Stack for Users

> You can build with whatever you want. OmegaOS orchestrates Claude / Codex /
> Gemini / etc., it doesn't care about your stack. But if you're starting
> fresh and want maximum agentic leverage, this is what we recommend.

## TL;DR

**TypeScript everywhere. Bun for scripts/CLI, Node for Next prod, Convex for
state. Vercel AI SDK for the foundation, Mastra when it gets serious.**

## Language

| Layer | Choice | Why |
|-------|--------|-----|
| Everything user-facing | **TypeScript** | Single language across web/scripts/agents. Massive orchestration advantage. |
| OmegaOS core (this repo) | **Rust** | OS-grade reliability + startup speed. Not for end-user apps. |

## Runtime

| Surface | Runtime |
|---------|---------|
| Next.js production app | **Node** (Next still ships best on Node) |
| Scripts, CLIs, agents | **Bun** (~25ms startup vs Node 200ms+, built-in TS + bundler) |

## Web Stack

| Concern | Pick |
|---------|------|
| Framework | **Next.js** (App Router, RSC) |
| Workflow canvas | **React Flow** (drag-drop graphs for agent flows) |
| UI components | **shadcn/ui** + Tailwind |
| Animations | Framer Motion |
| Charts | Recharts or Tremor |

## Data + Auth + Payments

| Concern | Pick | Why |
|---------|------|-----|
| Database | **Convex** | Reactive, scheduled functions, cron, durable functions, native Agent component |
| Auth | **Clerk** | Organizations, OAuth, MFA, deep Next.js integration |
| Payments | **Stripe** | Standard; Convex has first-class Stripe sync |
| External APIs | **Composio** | Gives agents access to 250+ tools (GitHub, Linear, Notion, etc.) |

## Agent / AI Stack

| Tier | Choice | When |
|------|--------|------|
| Foundation | **Vercel AI SDK** | Lightweight, lives in Next.js, streaming + generative UI + tool calls. Always your starting point. |
| Production agents | **Mastra** | TypeScript de facto for agents in 2026. Memory, tool calling, workflow state with time-travel debugging, evals, tracing, Mastra Studio. Runs on Vercel AI SDK underneath. Native Next.js integration. |
| Autonomous coding | **Claude Agent SDK** | Anthropic's primitives for coding/research agents. Pairs naturally with Claude Code. |
| Heavy workflows | **LangGraph** (only if needed) | Durable graph workflows, deep human-in-the-loop. Python-first. Probably overkill — start with Mastra. |

## How they fit together

```
Browser
   ↓
Next.js (Node runtime, RSC)
   ↓
Vercel AI SDK ────── streaming + tool calls + generative UI
   ↓
Mastra ───────────── memory, workflows, evals, tracing
   ↓
Convex ───────────── state, scheduled cron, durable jobs
   ↓
Composio ─────────── external tools (GitHub, Notion, Slack, …)
   ↓
LLM provider (Anthropic / OpenAI / Google / OpenRouter)
```

OmegaOS sits OUTSIDE this stack — it orchestrates the CLI agents
(Claude Code, Codex, Gemini) on the user's machine. The recommended
stack above is for the apps those agents BUILD.

## What to skip

- **LangChain** — too much abstraction, leaky, replaced by AI SDK + Mastra
- **CrewAI / AutoGen** — Python-first; if you're on TS, use Mastra
- **Raw OpenAI SDK direct calls** — AI SDK gives you streaming + retries + tool unification for free
- **Custom React UI libraries** — use shadcn; the ecosystem alignment matters more than any small DX win

## When to deviate

- **Heavy data science / ML pipelines** → Python is justified (FAISS, transformers, etc.)
- **Mobile native** → Expo / React Native (TS still wins)
- **Embedded / systems** → Rust (which OmegaOS itself is)

Otherwise: stay on the stack. One language, one runtime philosophy, one
component library, one state system. The orchestration leverage compounds.

## Bootstrap recipe

```bash
# Fresh project on the recommended stack
bun create next-app my-app --typescript --tailwind --app
cd my-app
bunx shadcn@latest init
bun add convex @clerk/nextjs ai @ai-sdk/anthropic
bunx convex dev   # spin up Convex
# Add Mastra later when you have more than 3 tools
bun add @mastra/core @mastra/memory
```

That gets you a Next.js + Tailwind + shadcn + Convex + Clerk + AI SDK
project ready in ~3 minutes. Then build.

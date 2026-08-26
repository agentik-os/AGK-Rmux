---
name: build
description: >
  Full product pipeline orchestrator for existing projects. Vision, PRD, Brand, DeepUX,
  Plan, Execute, Verify. Detects existing artifacts and resumes where you left off. Use
  when user says "/build", "build everything", "full pipeline", or "build this product".
  For new projects from scratch (includes scaffolding), see /new. For individual steps,
  see /vision, /prd, /brand-identity, /deepux, create a plan. For idea-to-MVP, see /team.
allowed-tools: ["Read", "Write", "Edit", "Bash", "Glob", "Grep", "Agent", "ToolSearch", "WebSearch", "WebFetch", "Skill", "TaskCreate", "TaskUpdate", "TaskList", "TaskGet", "TeamCreate", "TeamDelete", "SendMessage", "TaskOutput"]
---

# /build - Full Product Pipeline Orchestrator

<build-banner>
```
+==================================================================+
|                                                                    |
|   ██████╗ ██╗   ██╗██╗██╗     ██████╗                             |
|   ██╔══██╗██║   ██║██║██║     ██╔══██╗                            |
|   ██████╔╝██║   ██║██║██║     ██║  ██║                            |
|   ██╔══██╗██║   ██║██║██║     ██║  ██║                            |
|   ██████╔╝╚██████╔╝██║██████╗██████╔╝                            |
|   ╚═════╝  ╚═════╝ ╚═╝╚═════╝╚═════╝                            |
|                                                                    |
|   Full Product Pipeline v3.0                                       |
|   "From soul to shipped product."                                  |
|                                                                    |
|   Vision -> PRD -> Brand -> DeepUX -> Plan -> Build -> Verify      |
|                                                                    |
+==================================================================+
```
</build-banner>

**Display the banner above when starting.**

---

## /build vs /team — KNOW THE DIFFERENCE

| | /build | /team |
|---|--------|--------|
| **Philosophy** | Methodical, research-driven, complete | Rapid prototyping, speed-first |
| **Planning depth** | Full vision + PRD + brand + planner | Lighter discovery, straight to code |
| **When to use** | Production-quality product, client work | Hackathon, proof of concept, internal tool |
| **Duration** | Hours to days | 30 min to few hours |
| **Artifacts** | VISION.md, 11 PRD docs, brand system, full plan | Minimal docs, working MVP |
| **UX Architecture** | Full DeepUX analysis per page | Quick layout decisions |

**Rule:** If the user says "build me X" casually, ask: "Full /build pipeline or quick /team prototype?"

---

## COMMAND SYNTAX & FLAGS

```bash
# Full pipeline (interactive vision discovery first)
/build
/build "A voice journal that feels like a cassette recorder"

# Start from a specific phase (skips earlier phases — they must exist)
/build --from=prd
/build --from=brand
/build --from=deepux
/build --from=plan
/build --from=execute
/build --from=verify

# Skip specific phases
/build --skip=vision
/build --skip=brand
/build --skip=deepux
/build --skip=vision,brand,deepux

# Dry run — show what would happen without executing
/build --dry-run

# Resume interrupted build
/build --resume
```

---

## EXECUTION LOGIC — FOLLOW THIS EXACTLY

### Step 0: Parse Arguments

Parse the user's input for:
- `$PRODUCT_DESCRIPTION` — any quoted text or free-form description
- `--from=<phase>` — start from: `vision`, `prd`, `brand`, `deepux`, `plan`, `execute`, `verify`
- `--skip=<phases>` — comma-separated phases to skip
- `--dry-run` — if present, only report what would happen
- `--resume` — resume from `.build/state.json`

### Step 1: Phase Detection — Scan for Existing Artifacts

Run these checks to detect which phases are already complete:

```bash
# Check each phase's artifacts
VISION_EXISTS=false
PRD_EXISTS=false
BRAND_EXISTS=false
PLAN_EXISTS=false

# Phase 1: Vision
[ -f "VISION.md" ] && [ "$(wc -c < VISION.md)" -gt 500 ] && VISION_EXISTS=true
[ -f "docs/VISION.md" ] && [ "$(wc -c < docs/VISION.md)" -gt 500 ] && VISION_EXISTS=true

# Phase 2: PRD
[ -f "docs/PRD.md" ] && PRD_EXISTS=true
[ -d "docs/FEATURES" ] && [ "$(ls docs/FEATURES/*.md 2>/dev/null | wc -l)" -gt 0 ] && PRD_EXISTS=true

# Phase 3: Brand Identity
[ -f "exports/design-tokens.css" ] && BRAND_EXISTS=true
[ -f "docs/CREATIVE-DIRECTION.md" ] && BRAND_EXISTS=true
[ -d ".brand-identity" ] && BRAND_EXISTS=true

# Phase 4: DeepUX
DEEPUX_EXISTS=false
[ -f "docs/redesign/SUMMARY.md" ] && DEEPUX_EXISTS=true
[ -f "docs/redesign/BACKLOG.md" ] && [ -f "docs/redesign/DESIGN-DECISIONS.md" ] && DEEPUX_EXISTS=true

# Phase 5: Plan
[ -f ".planner/tracker.json" ] && PLAN_EXISTS=true
```

### Step 2: Determine Phase Sequence

Build the ordered list of phases to execute:

```
PHASES = [vision, prd, brand, deepux, plan, execute, verify]

For each phase in PHASES:
  1. If --from=<phase> is set, skip all phases before <phase>
  2. If --skip includes this phase, mark SKIPPED
  3. If artifact already exists for this phase, mark DETECTED (skip unless --from forces it)
  4. Otherwise, mark PENDING (will execute)
```

### Step 3: Display Phase Plan

Show the user what will happen:

```
📋 /build Pipeline — Phase Detection Report

  Phase 1 — Vision:     ✅ DETECTED (VISION.md exists, 2.3KB) — SKIPPING
  Phase 2 — PRD:        ✅ DETECTED (docs/PRD.md + 6 features) — SKIPPING
  Phase 3 — Brand:      ⏳ PENDING — will execute
  Phase 4 — DeepUX:     ⏳ PENDING — will execute
  Phase 5 — Plan:       ⏳ PENDING — will execute
  Phase 6 — Execute:    ⏳ PENDING — will execute
  Phase 7 — Verify:     ⏳ PENDING — will execute

  Starting from: Phase 3 (Brand Identity)
  Product: "A voice journal app" (from VISION.md)
```

**If `--dry-run`:** Display this report and STOP. Do not execute anything.

### Step 4: Initialize Build State

Create `.build/state.json` to track progress:

```bash
mkdir -p .build
```

Write `.build/state.json`:
```json
{
  "started_at": "<ISO timestamp>",
  "product": "<description from args or VISION.md>",
  "current_phase": "<first pending phase>",
  "phases": {
    "vision":  { "status": "detected|skipped|pending|running|completed|failed", "artifacts": [] },
    "prd":     { "status": "...", "artifacts": [] },
    "brand":   { "status": "...", "artifacts": [] },
    "deepux":  { "status": "...", "artifacts": [] },
    "plan":    { "status": "...", "artifacts": [] },
    "execute": { "status": "...", "artifacts": [] },
    "verify":  { "status": "...", "artifacts": [] }
  }
}
```

### Step 5: Execute Phases Sequentially

Execute each PENDING phase in order. After each phase completes, update `.build/state.json` and proceed to the next.

---

## MANDATORY TECHNOLOGY STANDARDS

Every `/build` product MUST use this stack. No exceptions.

| Layer | Standard | Notes |
|-------|----------|-------|
| **Framework** | Next.js 16+ (App Router) | Server components default, `"use client"` only when needed |
| **React** | React 19+ | `use()`, `useActionState`, `useOptimistic`, React Compiler |
| **Backend** | Convex | Real-time, cloud-synced. NEVER SQLite or embedded DB. |
| **UI Library** | shadcn/ui + shadcn Studio (premium) | Studio FIRST (`@ss-components`, `@ss-blocks`), then base shadcn |
| **Styling** | Tailwind CSS v4 | OKLCH colors, `@theme` directive, CSS-first config |
| **Auth** | Clerk | With Convex integration |
| **Payments** | Stripe | When applicable |
| **Animation** | Framer Motion / CSS | `motion()` for orchestrated sequences, CSS for micro |
| **Forms** | react-hook-form + zod + shadcn Form | Type-safe validation |
| **Icons** | Lucide React | Consistent icon set |
| **Design Research** | OpenUI (`@openuidev/cli`) | Component library analysis + generative UI patterns |

### shadcn Studio Premium (ALWAYS USE FIRST)
```bash
npx shadcn@latest add @ss-components/component-name
npx shadcn@latest add @ss-blocks/block-name
npx shadcn@latest add @ss-themes/theme-name
```
Credentials: `EMAIL=<your-shadcn-account-email>`, `LICENSE_KEY=<SHADCN_STUDIO_LICENSE_KEY>`   (keep the real key OUT of the repo — read from ~/.omega/secrets at runtime)

### Design Quality Bar — Vercel/Linear/v0 Standard

Every UI produced by `/build` must meet:
- **Typography**: Distinctive font choices — NEVER Inter, Roboto, Arial, or system fonts
- **Color**: Cohesive OKLCH palette with dominant + accent. No timid even palettes
- **Motion**: Orchestrated page load with staggered reveals. Scroll-triggered + hover surprises
- **Spatial**: Asymmetric layouts, overlap, generous negative space OR controlled density
- **Backgrounds**: Atmosphere — gradient meshes, noise textures, geometric patterns, layered transparencies
- **Userflow**: Every interaction path tested. No dead ends. Loading/empty/error states for everything
- **Accessibility**: WCAG 2.1 AA — keyboard nav, ARIA labels, color contrast

**ANTI-PATTERNS (NEVER):** Purple gradients on white, cookie-cutter layouts, generic card grids, placeholder-looking UIs.

---

## PHASE EXECUTION DETAILS

### PHASE 1: VISION (Interactive)

**Detection:** VISION.md exists and is >500 bytes.

**Execution:**
```
Skill("vision")
```

This invokes the /vision command (skill: branding.md) which runs the interactive Socratic discovery and produces VISION.md.

**After completion:**
- Verify VISION.md was created and is substantial
- Update `.build/state.json`: vision.status = "completed", vision.artifacts = ["VISION.md"]
- Auto-proceed to Phase 2 (no user prompt needed)

---

### PHASE 2: PRD (Autonomous after initial questions)

**Detection:** `docs/PRD.md` exists.

**Execution:**
```
Skill("prd")
```

This invokes /prd which reads VISION.md and generates the full 11-file documentation suite in `docs/`.

**Expected artifacts:**
- `docs/PRD.md`
- `docs/TECH-ARCHITECTURE.md`
- `docs/DESIGN-SYSTEM.md`
- `docs/LAYOUTS.md`
- `docs/DATA-MODEL.md`
- `docs/AGENT-PLAYBOOK.md`
- `docs/TESTING-STRATEGY.md`
- `docs/GLOSSARY.md`
- `docs/CHANGELOG-TEMPLATE.md`
- `docs/FEATURES/F-XXX-*.md` (individual feature specs)

**After completion:**
- Verify docs/PRD.md exists
- Update `.build/state.json`
- Auto-proceed to Phase 3

---

### PHASE 3: BRAND IDENTITY (Autonomous)

**Detection:** `exports/design-tokens.css` exists OR `docs/CREATIVE-DIRECTION.md` exists OR `.brand-identity/` directory exists.

**Execution:**
```
Skill("brand-identity")
```

This invokes /brand-identity which reads VISION.md + PRD docs and produces the complete brand system.

**Expected artifacts:**
- `docs/STRATEGY.md`
- `docs/CREATIVE-DIRECTION.md`
- `docs/AI-PROMPTS.md`
- `docs/ANTI-PATTERNS.md`
- `exports/design-tokens.css`
- `exports/tailwind-brand.ts`
- `exports/figma-tokens.json`
- `BRAND-VARIANTS.json`

**After completion:**
- Verify exports/design-tokens.css exists
- Update `.build/state.json`
- Auto-proceed to Phase 4

---

### PHASE 4: DEEP UX (Interface Architecture)

**Detection:** `docs/redesign/SUMMARY.md` exists.

**Why this matters:** DeepUX transforms the PRD specs and brand identity into implementation-ready interface architecture. Without this, the planner creates generic UI steps. WITH this, every page has a detailed 11-layer spec, persona-validated journeys, and severity-scored backlog that feeds directly into the planner.

**Execution:**
```
Skill("deepux")
```

This invokes /deepux which reads VISION.md + docs/PRD.md + brand tokens and produces:
- Heuristic audit of current state (or blank-slate architecture for new products)
- Navigation architecture (IA.md)
- Shared design decisions aligned with brand tokens
- 11-layer spec per page (psychology, IA, features, interactions, components, visual, AI, accessibility, copy, delight, migration)
- Persona walkthrough validations
- Severity-scored backlog (BACKLOG.md)
- UX scorecard (SCORECARD.md)

**For new products (no existing code):**
DeepUX works in "Blueprint Mode" — instead of auditing current state, it:
1. Reads PRD features to determine required pages
2. Architects the navigation system from scratch
3. Produces page specs as implementation blueprints
4. Skips heuristic audit (nothing to audit)
5. Still runs persona simulation against the PROPOSED architecture

This is critical for /build which often creates new products from scratch.

**Expected artifacts:**
- `docs/redesign/BRIEF.md`
- `docs/redesign/AUDIT.md` (or `ARCHITECTURE.md` for new products)
- `docs/redesign/IA.md`
- `docs/redesign/DESIGN-DECISIONS.md`
- `docs/redesign/PERSONAS.md`
- `docs/redesign/SCORECARD.md`
- `docs/redesign/SUMMARY.md`
- `docs/redesign/BACKLOG.md`
- `docs/redesign/{page-name}.md` (per page specs)

**After completion:**
- Verify `docs/redesign/SUMMARY.md` exists
- Update `.build/state.json`
- Auto-proceed to Phase 5 (Planning)

---

### PHASE 5: PLANNING (Context-Aware)

**Detection:** `.planner/tracker.json` exists.

**Execution:**
```
Skill("planner")
```

This invokes create a plan which reads ALL Phase 1-4 artifacts and creates the full implementation plan. The planner MUST produce steps with `context_files[]` arrays — each step must reference the exact docs its implementing agent needs to read.

The planner MUST read `docs/redesign/` artifacts:
- `BACKLOG.md` — converts DUX-NNN items into planner steps
- Per-page specs — `context_files` for UI implementation steps
- `DESIGN-DECISIONS.md` — context for all frontend steps

**Context File Assignment Rules (planner MUST follow):**

| Step type | MUST include in context_files |
|-----------|------------------------------|
| Any step | `VISION.md` |
| UI/Frontend | `DESIGN-SYSTEM.md`, `CREATIVE-DIRECTION.md`, `exports/tailwind-brand.ts`, `docs/redesign/DESIGN-DECISIONS.md`, `docs/redesign/{page}.md` |
| Page/Layout | `LAYOUTS.md`, relevant `FEATURES/F-XXX-*.md`, `docs/redesign/{page}.md`, `docs/redesign/IA.md` |
| Backend/API | `TECH-ARCHITECTURE.md`, `DATA-MODEL.md` |
| Feature impl | Specific `FEATURES/F-XXX-*.md` |
| Auth/Payments | `PRD.md`, `TECH-ARCHITECTURE.md` |
| Testing | `TESTING-STRATEGY.md` |
| Brand/Marketing | `STRATEGY.md`, `AI-PROMPTS.md`, `ANTI-PATTERNS.md` |
| Project setup | `TECH-ARCHITECTURE.md`, `exports/tailwind-brand.ts`, `exports/design-tokens.css` |

**After completion:**
- Verify `.planner/tracker.json` exists
- Update `.build/state.json`
- Auto-proceed to Phase 6

---

### PHASE 6: EXECUTION (Orchestrated)

**Detection:** Check `.planner/tracker.json` — if all steps are `completed`, this phase is done.

**Execution:**

Read `.planner/tracker.json` and execute steps based on their `execution_mode`:

| Mode | How it runs |
|------|-------------|
| `direct` | Execute the step yourself — read context_files, then implement |
| `team` | Use /team with parallel workers, each getting context_files |

**Execution loop:**
1. Read tracker.json
2. Find steps with status=`pending` whose `depends_on` are all `completed`
3. Execute ready steps (parallel where no file conflicts)
4. After each step: update tracker.json status to `completed`
5. After each milestone: run build check (`npm run build` or equivalent)
6. Repeat until all steps complete or blocked
7. Telegram notify on milestone completion

**Context injection:** Every agent launched MUST read its `step.context_files[]` before writing any code. This is NON-NEGOTIABLE.

**After completion:**
- All steps in tracker.json are `completed`
- Update `.build/state.json`
- Auto-proceed to Phase 7

---

### PHASE 7: VERIFICATION

**Detection:** `.build/state.json` shows verify.status = "completed".

**Execution:**
```
Skill("debugaudit")
```

Or if /debugaudit is not available, run verification manually:

1. **Build check:** `npm run build` must pass with 0 errors
2. **Visual verification:** Screenshot key pages via Playwright/Chrome MCP
3. **Console sweep:** Check every page for JS errors
4. **Network check:** No 4xx/5xx responses
5. **Brand compliance:** Check against `docs/ANTI-PATTERNS.md`
6. **Feature acceptance:** Verify each `FEATURES/F-XXX` criteria

**After completion:**
- Update `.build/state.json`: verify.status = "completed"
- Send Telegram notification with final report

---

## Execution Phase — Agentik-Academy-3 Patterns

> Source of truth for **how Phase 6 (Execution) is actually dispatched** in 2026+.
> Distilled from the live `Agentik-Academy-oracle-3` session that orchestrated 36 workers
> (see `~/.aisb/state/oracle-Agentik-Academy-oracle-3.workers.txt`) across foundation,
> feature waves, redesign, QA sweep, i18n, and SEO without a single file-conflict abort.
>
> This section **extends** PHASE 6 — it does not replace it. PHASE 6 above is the contract
> (what the oracle owes the user). This section is the **runtime protocol** (how the oracle
> actually drives workers through native tooling).
>
> Full companion docs:
> - `~/.aisb/docs/PATTERNS-AGENTIK-ACADEMY-3.md` (forward link — written by sibling worker)
> - `~/.aisb/docs/PATTERNS-INTEGRATION.md` (currently-on-disk pattern catalog)
> - `~/.aisb/docs/oracle-protocol.md` (oracle behavioural contract — R-1..R-14)

### (a) Native TaskCreate batch seed

After `/planner` writes `.planner/tracker.json`, the oracle **batch-creates** every step
as a native task via `TaskCreate`. One call per step. Each task carries:

| Field | Value |
|-------|-------|
| `subject` | Step title from planner (≤80 chars) |
| `description` | Full step body + acceptance criteria + `context_files[]` list |
| `activeForm` | Present participle ("Implementing landing hero", "Wiring Convex schema") |
| `metadata.files_owned` | JSON array of disjoint file globs — used by scope-claim later |
| `metadata.wave` | Integer (0 = foundation, 1 = first parallel wave, 2, 3, …) |
| `metadata.depends_on` | Array of `task_id`s that must reach `completed` first |

As work progresses, the oracle mutates state via `TaskUpdate(task_id=…, status=…)`:
`pending → in_progress → completed` (or `blocked → failed`). The shared task list is the
**single source of truth** that workers, patrol, close-gate, and the user-facing UI all
read. Never duplicate state in ad-hoc files when a `TaskUpdate` call will do.

**Rule:** every step in `tracker.json` becomes exactly one `TaskCreate` row before any
worker is dispatched. Skipping this step breaks the dependency engine and the Telegram
progress badge.

### (b) Wave-based dispatch (sequential foundation → parallel features)

The Agentik-Academy-3 sequence proved the canonical shape:

```
Wave 0 — Foundation (SEQUENTIAL, blocking)
  └─ worker-3-foundation   (schema, env, base layout, auth scaffolding)

Wave 1 — Feature core (PARALLEL, disjoint files)
  ├─ worker-3-landing      (apps/web/app/(marketing)/**)
  ├─ worker-3-booking      (apps/web/app/(app)/booking/**)
  └─ worker-3-access       (convex/auth/**, middleware.ts)

Wave 2 — Live + admin (PARALLEL after wave 1)
  ├─ worker-3-live         (apps/web/app/(app)/live/**)
  └─ worker-3-admin        (apps/web/app/(admin)/**)

Wave 3+ — Integrations, redesign, polish, content (PARALLEL, lock-gated)
  └─ … 30+ workers (credentials, cal-event, redesign, loops, onboarding, i18n, SEO, …)
```

**Why sequential foundation:** schema/auth/layout changes touch shared files (`convex/schema.ts`,
`app/layout.tsx`, `tailwind.config.ts`). Parallelising them guarantees scope-claim aborts and
half-written commits. Block the wave on `worker-3-foundation` writing `done_clean`.

**Why parallel feature waves:** once foundation is frozen, feature teams own disjoint route
groups and Convex namespaces. Dispatching them in one batch (single message, multiple Agent
calls per `/team`) converts wall-clock from hours-of-sequential into minutes-of-parallel.

**Wait pattern:** the oracle dispatches the wave, then `ScheduleWakeup` (see §d) and drains
the inbox of `worker_done` events. Only when **every** worker in the wave hits `done_clean`
does the oracle compute the next wave's eligibility set and fan out again.

### (c) Lock-based parallel safety (`WORKER_FILES_OWNED` + `scope-claim.sh`)

Every worker dispatch sets the environment variable `WORKER_FILES_OWNED` to a newline- or
comma-separated list of file globs the worker is allowed to touch. The worker calls:

```bash
~/.aisb/lib/scope-claim.sh claim "$TMUX_SESSION" "$WORKER_FILES_OWNED"
```

The claimer writes `~/.aisb/state/scope-<SESSION>.json` only if **no other live worker** has
overlapping globs. Conflicting claim → **exit 73** → worker self-aborts and writes
`done.json` with `status=failed, reason=scope_conflict`.

When the oracle sees `scope_conflict` in `worker_done` events:

1. **Stop fanning out** the rest of the wave immediately.
2. **Re-plan** the conflicting tasks with disjoint scope (split the file set, or serialise
   them into the next wave).
3. **Re-dispatch** the survivors with corrected `WORKER_FILES_OWNED`.
4. **Never** retry the same claim — it will exit 73 again.

**Release discipline:** every worker, before `worker-mark-done.sh`, MUST call
`~/.aisb/lib/scope-claim.sh release "$TMUX_SESSION"`. Forgetting this is a leak that blocks
every future worker touching those paths.

### (d) `ScheduleWakeup` between waves (oracle idle, never idle-busy)

After dispatching a wave, the oracle does **not** poll. It calls `ScheduleWakeup` with a
delay sized to the expected wave wall-clock (typically **600–1800 s**, 10–30 minutes; cap
3600 s) and exits its turn. Per `~/.claude/CLAUDE.md` cache windows: never pick 300 s
(worst-case cache miss); 270 s for active polling, 1200–1800 s for long waves.

On wake (or earlier if a `worker_done` event triggers the oracle inbox), the oracle:

1. **Drains the inbox** (`~/.aisb/state/oracle-<NAME>.inbox.jsonl`) per R-12 of
   `oracle-protocol.md`. Each event = one completed/failed/blocked worker.
2. **Mutates `TaskUpdate`** for each event (set `completed`/`failed`/`blocked`,
   append `result` to description).
3. **Mirrors via `oracle-todo-update.sh`** so the patrol + Telegram badge stay in sync.
4. **Recomputes ready set** (tasks whose `depends_on` are all `completed`).
5. **If wave fully drained** → dispatch next wave + `ScheduleWakeup` again.
6. **If wave still has live workers** → `ScheduleWakeup` shorter delay (~600 s) and exit.

This is the heartbeat that makes 36-worker orchestrations cheap: the oracle's context-cache
is amortised across cache-warm wakeups, not burned re-reading the whole conversation every
60 s.

### (e) Canonical worker template (mandatory shape)

Every worker prompt dispatched from `/build` Phase 6 starts with `/team MISSION:` on **line
1** (so the worker's first action enters `/team` skill mode and gets the dispatched-worker
contract). The body MUST contain, in order, these labelled blocks:

```
/team MISSION: <one-line goal>

[DISPATCHED — session=<NAME>] Third Law: decide+execute, never wait.

MANDATORY FINAL STEP — worker-mark-done.sh as last Bash call.

TYPE: FEATURE | REFACTOR | FIX | AUDIT
PROJECT: <abs path>
STACK: <one-line>

REFERENCE FILES:
- <path to spec / context_files[] from planner>

SCOPE (files_owned):
  <newline-separated globs — also exported as WORKER_FILES_OWNED>

DO NOT TOUCH: <explicit exclusions>

== TODOLIST OBLIGATOIRE (TodoWrite first) ==
1. ...
2. ...
N. Release scope-claim + worker-mark-done.sh

== DONE CRITERIA ==
- <verifiable predicates, one per line>

== VERIFY COMMAND ==
<exact shell snippet that exits 0 iff DONE>
```

Full template + rationale: `~/.aisb/docs/PATTERNS-AGENTIK-ACADEMY-3.md`.

**Anti-patterns** (auto-failure):
- Paraphrasing the worker mission as prose without TODOLIST + DONE CRITERIA + VERIFY.
- Omitting `files_owned` (scope-claim cannot protect you).
- Mentioning "shall I proceed?" / `AskUserQuestion` (violates Third Law).
- Leaving `worker-mark-done.sh` to the assistant's final text (must be a Bash tool call).

### (f) Audit wave after build (dynamic selector + fix-reaudit loop)

Once every task in the last execution wave hits `completed`, the oracle launches an
**audit wave** before declaring Phase 7 complete:

1. **Run the audit selector**: `~/.claude/lib/audit-selector.py --tracker .planner/tracker.json`
   inspects the modified files + step descriptions and returns **4–12 relevant audits**
   from the Quality Arsenal (`/codeaudit`, `/flowaudit`, `/perfaudit`, `/secaudit`,
   `/a11yaudit`, `/seoaudit`, `/uiuxaudit`, `/dataaudit`, `/apiaudit`, `/copyaudit`,
   `/debugaudit`, `/dxaudit`, `/motionaudit`, `/automationaudit`, `/logicaudit`,
   `/retentionaudit`, `/featureaudit`). Never paraphrase — invoke the real `/` skill.
2. **Dispatch all selected audits in parallel** (one worker each, `subagent_type=guardian`
   or domain specialist). Each audit prompt starts with `/<skill> --scope=…`.
3. **Fix-reaudit loop** per R-6 of `oracle-protocol.md`: failing audits feed back into a
   **fix worker** for the relevant files, which then re-runs the same audit. Max **5
   iterations** per ticket; after 5 the ticket is escalated to the user as `pending`.
4. **Aggregate scores**: every audit normalises to `/100`. Below 85 → block the deploy
   wave. Above 85 → cleared for `/prod`.

### (g) Deploy wave (`/prod` gate per R-14)

Only after the audit wave is clean (≥85/100 across all selected audits) does the oracle
dispatch the deploy wave:

```
/prod <ProjectName>
```

`/prod` runs the canonical pipeline: build → push → Convex deploy → Vercel `--prod` →
poll deploy status → curl prod URL for 200. Per **rule 47** (`oracle-end-of-work.md`):
- **freeze, don't rollback** on deploy failure (`~/.aisb/locks/ship-<project>.frozen`),
- the oracle stays alive, Telegram alert fires, the user decides revert-vs-fix-forward,
- `done.json` for the deploy worker carries the commit, push URL, deploy URL, and
  `deploy_status`.

Per **rule 51** (`prod-verify-console.md`): after deploy, an additional verify worker
loads the prod URL with Playwright CLI, captures console + network, exercises the golden
path, and **fixes any app-origin errors itself** (third-party extension noise is filtered).
Only then is Phase 7 marked `completed`.

---

### Worked example — `/build "A demo SaaS"` timeline

Illustrative wall-clock from a fictional but representative `/build` run that follows the
patterns above (numbers tuned to a small Convex + Next.js app with 3 feature surfaces):

```
T+00m  /build → phases 1-5 already detected (vision, prd, brand, deepux, plan)
                Phase 6 starts; oracle batch-creates 11 tasks via TaskCreate.

T+00m  Wave 0 — Foundation (SEQUENTIAL)
       └─ worker-foundation  (convex/schema.ts, app/layout.tsx, env, auth scaffold)
       ScheduleWakeup(900s).

T+30m  Inbox: worker_done(foundation, done_clean). TaskUpdate. Eligible: landing, booking, access.

T+30m  Wave 1 — Feature core (PARALLEL × 3, disjoint files_owned)
       ├─ worker-landing  (app/(marketing)/**)
       ├─ worker-booking  (app/(app)/booking/**, convex/booking.ts)
       └─ worker-access   (convex/auth/**, middleware.ts)
       ScheduleWakeup(1800s).

T+90m  Inbox drained: 3 done_clean. TaskUpdate ×3. Eligible: live, admin.

T+90m  Wave 2 — Live + admin (PARALLEL × 2)
       ├─ worker-live   (app/(app)/live/**)
       └─ worker-admin  (app/(admin)/**)
       ScheduleWakeup(1200s).

T+135m Inbox drained: 2 done_clean. All execution tasks completed.

T+135m Audit wave — audit-selector returns 6 audits (code, flow, ui/ux, a11y, perf, sec).
       Dispatched in parallel. ScheduleWakeup(1500s).

T+165m Inbox drained: 5 audits ≥85, 1 audit (a11y=72) failing.
       Fix-reaudit loop iter 1: dispatch worker-fix-a11y → re-run /a11yaudit.

T+185m a11y=91. Audit wave clear (all ≥85).

T+185m Deploy wave — /prod runs build/push/convex/vercel. Polled READY at T+193m.
       Verify worker exercises prod URL: console clean, golden path 200. Phase 7 ✅.

Total ≈ 195 minutes wall-clock for ~30 hours of equivalent sequential agent work.
```

The shape — sequential foundation, parallel waves under lock, idle oracle between waves,
audit gate before deploy — is what makes the 36-worker Agentik-Academy-3 mission
reproducible at 1/Nth wall-clock without ever corrupting shared files.

### Cross-references

- `~/.aisb/docs/PATTERNS-AGENTIK-ACADEMY-3.md` — full pattern catalog (sibling worker output)
- `~/.aisb/docs/PATTERNS-INTEGRATION.md` — currently-on-disk pattern reference
- `~/.aisb/docs/oracle-protocol.md` — R-1..R-14 oracle contract (especially R-6 fix-reaudit, R-12 inbox drain, R-14 prod gate)
- `~/.claude/rules/47-oracle-end-of-work.md` — explicit ship, freeze don't rollback
- `~/.claude/rules/51-prod-verify-console.md` — post-deploy console self-fix
- `~/.claude/rules/49-third-law-autonomy.md` — worker decide+execute, never idle

---

## RESUME LOGIC

When `/build --resume` is invoked OR `.build/state.json` exists at the start:

1. Read `.build/state.json`
2. Find the first phase with status != `completed` and status != `skipped`
3. If it's `running` — it was interrupted. Re-run that phase.
4. If it's `pending` — start from there.
5. Display: `📋 Resuming /build from Phase N: <phase name>`

---

## BUILD STATE FILE SCHEMA

`.build/state.json`:
```json
{
  "started_at": "2026-03-12T10:00:00Z",
  "product": "A SaaS for dental clinics",
  "current_phase": "brand",
  "flags": { "from": null, "skip": [], "dry_run": false },
  "phases": {
    "vision":  { "status": "completed", "completed_at": "...", "artifacts": ["VISION.md"] },
    "prd":     { "status": "completed", "completed_at": "...", "artifacts": ["docs/PRD.md", "docs/TECH-ARCHITECTURE.md", "..."] },
    "brand":   { "status": "running",   "started_at": "...",   "artifacts": [] },
    "deepux":  { "status": "pending",   "artifacts": [] },
    "plan":    { "status": "pending",   "artifacts": [] },
    "execute": { "status": "pending",   "artifacts": [] },
    "verify":  { "status": "pending",   "artifacts": [] }
  }
}
```

---

## PHASE TRANSITION PROTOCOL

After EVERY phase completion:

1. **Verify artifacts exist** — check the files the phase was supposed to produce
2. **Update `.build/state.json`** — mark phase completed, record artifacts and timestamp
3. **Display progress:**
   ```
   ✅ Phase 3 (Brand Identity) complete — 8 artifacts produced
   ⏳ Starting Phase 4 (DeepUX)...
   ```
4. **Auto-proceed** — do NOT ask the user for permission between phases (exception: Phase 1 Vision is interactive by nature)

---

## ANTI-PATTERNS

| Don't | Do instead |
|-------|------------|
| Re-run a phase whose artifacts exist | Detect and skip — show DETECTED |
| Ask user between every phase | Auto-proceed (vision is naturally interactive) |
| Launch agents without context_files | Every agent MUST read its docs first |
| Create generic step descriptions | Specific steps with exact file paths + context refs |
| Forget to update .build/state.json | Update after EVERY phase |
| Crash and lose progress | state.json enables resume from any point |

---

## EXAMPLES

### `/build "A SaaS for dental clinics"`
```
Phase Detection: No artifacts found — full pipeline
Phase 1: Skill("vision") → 8 discovery questions → VISION.md ✅
Phase 2: Skill("prd") → 11 docs + 6 feature specs ✅
Phase 3: Skill("brand-identity") → 3 variants + tokens ✅
Phase 4: Skill("deepux") → 11-layer specs per page + backlog ✅
Phase 5: Skill("planner") → 38 steps, 6 milestones ✅
Phase 6: Execute steps via /team ✅
Phase 7: Skill("debugaudit") → full 18-phase forensic verification, score 100/100 ✅
```

### `/build` (VISION.md already exists from previous /vision run)
```
Phase Detection: VISION.md found (2.3KB) ✅
Phase 1: SKIPPED (detected)
Phase 2: Skill("prd") → starts here
...
```

### `/build --from=execute`
```
Phase Detection: All Phase 1-5 artifacts exist ✅
Phases 1-5: SKIPPED (--from=execute)
Phase 6: Reading .planner/tracker.json → executing steps
Phase 7: Verification
```

### `/build --dry-run`
```
📋 /build Pipeline — Dry Run Report

  Phase 1 — Vision:     ✅ DETECTED (VISION.md, 2.3KB) — would SKIP
  Phase 2 — PRD:        ✅ DETECTED (docs/PRD.md + 6 features) — would SKIP
  Phase 3 — Brand:      ⏳ PENDING — would execute Skill("brand-identity")
  Phase 4 — DeepUX:     ⏳ PENDING — would execute Skill("deepux")
  Phase 5 — Plan:       ⏳ PENDING — would execute Skill("planner")
  Phase 6 — Execute:    ⏳ PENDING — would orchestrate from .planner/tracker.json
  Phase 7 — Verify:     ⏳ PENDING — would run /debugaudit verification

  No changes made (dry run).
```

---

## SEE ALSO

- `/vision` — Phase 1 standalone: emotional identity discovery
- `/prd` — Phase 2 standalone: documentation suite
- `/brand-identity` — Phase 3 standalone: complete brand system
- `/deepux` — Phase 4 standalone: UX architecture and interface design
- `create a plan` — Phase 5 standalone: implementation planning
- `/team` — Execution tool: parallel agent teams
- `/debugaudit` — Verification tool: comprehensive bug hunting

---

**/build v3.0 — "From soul to shipped product."**
*7-phase pipeline | Phase-detecting | Resumable | Context-propagated | Full lifecycle*
*Updated: 2026-03-12*

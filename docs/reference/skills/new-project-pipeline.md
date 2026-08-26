---
name: new
description: >
  Complete new product pipeline from idea to shipped product in one command. Scaffolding,
  Vision, PRD, Brand, DeepUX, Plan, Execute, Verify. Use when user says "/new", "new
  product", "start from scratch", "create new app", or "build something new". For existing
  projects that already have code, see /build. For rapid idea-to-MVP, see /team. For
  project scaffolding only (no pipeline), see /new-project.
allowed-tools: ["Read", "Write", "Edit", "Bash", "Glob", "Grep", "Agent", "AskUserQuestion", "ToolSearch", "WebSearch", "WebFetch", "Skill", "TaskCreate", "TaskUpdate", "TaskList", "TaskGet", "TeamCreate", "TeamDelete", "SendMessage", "TaskOutput"]
---

# /new - Complete New Product Pipeline

<new-banner>
```
+==================================================================+
|                                                                    |
|   ███╗   ██╗███████╗██╗    ██╗                                     |
|   ████╗  ██║██╔════╝██║    ██║                                     |
|   ██╔██╗ ██║█████╗  ██║ █╗ ██║                                     |
|   ██║╚██╗██║██╔══╝  ██║███╗██║                                     |
|   ██║ ╚████║███████╗╚███╔███╔╝                                     |
|   ╚═╝  ╚═══╝╚══════╝ ╚══╝╚══╝                                     |
|                                                                    |
|   Complete Product Pipeline v1.0                                   |
|   "From spark to shipped."                                         |
|                                                                    |
|   Scaffold -> Vision -> PRD -> Brand -> DeepUX -> Plan -> Build    |
|                                                                    |
+==================================================================+
```
</new-banner>

**Display the banner above when starting.**

---

## WHAT IS /new?

`/new` is the **ultimate new product command**. One command takes you from an idea to a fully scaffolded, designed, planned, and built product.

It combines every pipeline command in the correct order:

```
Phase 0: SCAFFOLD    → /new-project (project creation, deps, config)
Phase 1: VISION      → /vision (emotional identity, soul statement)
Phase 2: PRD         → /prd (market research, specs, features)
Phase 3: BRAND       → /brand-identity (visual identity, tokens, logo)
Phase 4: DEEP UX     → /deepux (interface architecture, 11-layer specs)
Phase 5: PLAN        → create a plan (implementation steps, dependencies)
Phase 6: EXECUTE     → /team (build it)
Phase 7: VERIFY      → /debugaudit (test everything)
```

### /new vs /build vs /team

| | /new | /build | /team |
|---|------|--------|--------|
| **Creates project** | ✅ Yes (scaffolding) | ❌ No (project must exist) | ✅ Yes (lighter) |
| **Vision & Brand** | ✅ Full pipeline | ✅ Full pipeline | ⚡ Quick discovery |
| **UX Architecture** | ✅ DeepUX v4.0 | ✅ DeepUX v4.0 | ❌ No |
| **Planning depth** | ✅ Full planner | ✅ Full planner | ⚡ Rapid |
| **When to use** | Starting from ZERO | Existing project needs building | Quick prototype |
| **Duration** | Hours to days | Hours to days | 30min to hours |

**Rule:** `/new` = brand new product from scratch. `/build` = existing project needs the pipeline. `/team` = rapid prototype.

---

## COMMAND SYNTAX

```bash
# Interactive (asks everything step by step)
/new

# With product idea
/new "A voice journal that feels like a cassette recorder"

# Skip scaffolding (project already created)
/new --skip=scaffold
# Same as /build at that point

# Start from specific phase
/new --from=prd
/new --from=brand
/new --from=deepux
/new --from=plan
/new --from=execute

# Skip specific phases (only when explicit and justified in .build/state.json)
/new --skip=brand
/new --skip=deepux,brand

# Dry run (show what would happen, no writes)
/new --dry-run

# NOTE: there is NO --quick mode. Per rule 46 (NO TIME PANIC), the full
# pipeline runs every time. If you want less scope, run individual phases
# standalone (e.g. /vision alone, /prd alone) — never a "streamlined" /new.
```

---

## EXECUTION LOGIC

### Step 0: Parse & Detect

1. Parse arguments for `$PRODUCT_IDEA`, `--from`, `--skip`, `--dry-run`
2. Check for existing state BEFORE starting:
   - `.build/state.json` exists? → Resume from `current_phase` (see "Resume" section)
   - `package.json` exists but no `.build/state.json`? → Partial project detected
      - Scan for artifacts: `VISION.md`, `docs/PRD.md`, `exports/design-tokens.css`, `docs/redesign/SUMMARY.md`, `.planner/tracker.json`
      - Rebuild `.build/state.json` marking found artifacts as `completed` (treat as implicit resume)
      - Display: `📋 Partial project detected — {N} artifacts found, resuming from Phase {first_missing}`
   - No `package.json` AND no `.build/state.json`? → Fresh start, go to Phase 0
   - `package.json` exists AND complete state file shows all phases done? → Display final report, exit 0

### Step 1: First Question (if no product idea given)

Ask the user ONE question to get started:

```
AskUserQuestion:
  question: "What do you want to build?"
  header: "Product Idea"
  options:
    - label: "SaaS Web App"
      description: "Dashboard, auth, billing — full product"
    - label: "Mobile App"
      description: "iOS/Android app with Expo"
    - label: "Landing Page"
      description: "Marketing site with conversion focus"
    - label: "Chrome Extension"
      description: "Browser extension"
    - label: "Something else"
      description: "Tell me in your own words"
```

Then: "Describe your product in one sentence — what is it and who is it for?"

Store as `$PRODUCT_IDEA`.

---

## PHASE 0: SCAFFOLD

**Goal:** Create the project directory with all dependencies and configuration.

**Execution:**
```
Skill("new-project")   # canonical skill name — hyphenated, lowercase
```

This runs the interactive `/new-project` wizard which handles:
- Project type, name, category
- Stack selection (Next.js, Convex, Clerk, Stripe, etc.)
- Port assignment
- Git initialization
- Dependency installation
- VPS ecosystem integration (tmux alias, CLAUDE.md)

**After completion:**
- `cd` into the new project directory (**CRITICAL**: every subsequent `Skill()` call inherits this cwd — verify with `pwd` before each phase, abort if not inside the new project)
- Verify `package.json` exists
- Write project_dir absolute path to `$PROJECT_DIR` env var (scoped to this `/new` invocation)
- Create `.build/state.json` to track pipeline progress (always at `$PROJECT_DIR/.build/state.json`, never relative)
- Update state: scaffold.status = "completed", scaffold.project_dir = `$PROJECT_DIR`
- Display:
  ```
  ✅ Phase 0 (Scaffold) complete — project created at $PROJECT_DIR
  ⏳ Starting Phase 1 (Vision)...
  ```

**Auto-proceed to Phase 1.** Subsequent phases MUST verify cwd == `$PROJECT_DIR` on entry; `cd "$PROJECT_DIR"` defensively if drift detected.

---

## PHASE 1: VISION

**Goal:** Define the emotional foundation and product identity.

**Execution:**
```
Skill("vision")
```

Pass `$PRODUCT_IDEA` as context so the Socratic discovery starts informed.

**Expected artifact:** `VISION.md` (>500 bytes)

**After completion:**
- Verify VISION.md exists
- Update state: vision.status = "completed"
- Auto-proceed to Phase 2

---

## PHASE 2: PRD

**Goal:** Generate complete product documentation with market research.

**Execution:**
```
Skill("prd")
```

PRD v2.0 will:
1. Read VISION.md for emotional/strategic foundation
2. Launch 4 parallel research agents (market, competitors, personas, GTM)
3. Generate 17 documentation files

**Expected artifacts:** `docs/PRD.md` + `docs/FEATURES/` + 6 research files

**After completion:**
- Verify docs/PRD.md exists
- Update state: prd.status = "completed"
- Auto-proceed to Phase 3

---

## PHASE 3: BRAND IDENTITY

**Goal:** Create complete visual identity aligned with vision and PRD.

**Execution:**
```
Skill("brand-identity")
```

Brand Identity will:
1. Read VISION.md (emotional foundation)
2. Read .prd/DESIGN-SYSTEM.md (constraints)
3. Read .prd/TECH-ARCHITECTURE.md (token format)
4. Produce 3 brand variants, design tokens, logo concepts

**Expected artifacts:** `exports/design-tokens.css` + `docs/CREATIVE-DIRECTION.md`

**After completion:**
- Verify exports/design-tokens.css exists
- Update state: brand.status = "completed"
- Auto-proceed to Phase 4

---

## PHASE 4: DEEP UX

**Goal:** Architect every interface before a single line of UI code is written.

**Execution:**
```
Skill("deepux")
```

DeepUX v4.0 will:
1. Read all previous artifacts (VISION, PRD, brand tokens)
2. Work in **Blueprint Mode** (new product, no existing pages to audit)
   - Reads PRD features → determines required pages
   - Architects navigation from scratch
   - Produces page specs as implementation blueprints
   - Runs persona simulation against PROPOSED architecture
3. Produce 11-layer spec per page
4. Generate severity-scored backlog

**Expected artifacts:** `docs/redesign/SUMMARY.md` + `docs/redesign/BACKLOG.md` + per-page specs

**After completion:**
- Verify docs/redesign/SUMMARY.md exists
- Update state: deepux.status = "completed"
- Auto-proceed to Phase 5

---

## PHASE 5: PLANNING

**Goal:** Convert all specs into an executable implementation plan.

**Execution:**
```
Skill("planner")
```

Planner will:
1. Read AGENT-PLAYBOOK.md (pre-built task breakdown from PRD)
2. Read ANTI-PATTERNS.md (constraints)
3. Read docs/redesign/ specs (UI implementation details)
4. Read DeepUX BACKLOG.md (severity-prioritized items)
5. Produce steps with `context_files[]` arrays

**Context file rules for each step:**

| Step type | context_files MUST include |
|-----------|---------------------------|
| Any step | `VISION.md` |
| UI/Frontend | `docs/redesign/DESIGN-DECISIONS.md`, `docs/redesign/{page}.md`, brand exports |
| Page/Layout | `docs/redesign/{page}.md`, `docs/redesign/IA.md`, `docs/LAYOUTS.md` |
| Backend/API | `docs/TECH-ARCHITECTURE.md`, `docs/DATA-MODEL.md` |
| Feature impl | Specific `docs/FEATURES/F-XXX-*.md` |
| Brand/Marketing | `docs/STRATEGY.md`, `exports/tailwind-brand.ts` |

**Expected artifact:** `.planner/tracker.json`

**After completion:**
- Verify .planner/tracker.json exists
- Update state: plan.status = "completed"
- Auto-proceed to Phase 6

---

## PHASE 6: EXECUTE

**Goal:** Build the actual product following the plan.

**Execution:**

Read `.planner/tracker.json` and execute steps:

1. Find steps with status=`pending` whose `depends_on` are all `completed`
2. Execute ready steps (parallel where no file conflicts)
3. Every agent MUST read its `step.context_files[]` before writing code
4. After each step: update tracker.json
5. After each milestone: run `npm run build` to catch errors early
6. Repeat until all steps complete

**Execution modes per step:**

| Mode | How |
|------|-----|
| `direct` | Execute yourself |
| `team` | Parallel /team workers |

**After completion:**
- All steps in tracker.json = `completed`
- Update state: execute.status = "completed"
- Auto-proceed to Phase 7

---

## PHASE 7: VERIFY

**Goal:** Ensure everything works perfectly.

**Execution:**
```
Skill("debugaudit")
```

Runs the full 18-phase forensic runtime audit. If any phase < 100/100, enter fix-and-reaudit loop (max 5 iterations per rule 43). Do NOT mark Phase 7 complete until debugaudit = 100/100 AND build passes AND all FEATURES/ specs are satisfied.

Or manual verification (fallback only if Skill tool unavailable):
1. `npm run build` — 0 errors
2. Screenshot key pages via Playwright
3. Console sweep — 0 JS errors
4. Network check — no 4xx/5xx
5. Brand compliance check against ANTI-PATTERNS.md
6. Feature acceptance against FEATURES/ specs
7. DeepUX spec compliance — do pages match the 11-layer specs?

**After completion:**
- Update state: verify.status = "completed"
- Send Telegram notification
- Display final report

---

## PROGRESS TRACKING

`.build/state.json` is created at Phase 0 and updated after every phase:

```json
{
  "command": "/new",
  "product": "A voice journal app",
  "started_at": "2026-03-12T10:00:00Z",
  "current_phase": "deepux",
  "phases": {
    "scaffold": { "status": "completed", "completed_at": "...", "artifacts": ["package.json", "CLAUDE.md"] },
    "vision":   { "status": "completed", "completed_at": "...", "artifacts": ["VISION.md"] },
    "prd":      { "status": "completed", "completed_at": "...", "artifacts": ["docs/PRD.md", "..."] },
    "brand":    { "status": "completed", "completed_at": "...", "artifacts": ["exports/design-tokens.css", "..."] },
    "deepux":   { "status": "running",   "started_at": "...",   "artifacts": [] },
    "plan":     { "status": "pending",   "artifacts": [] },
    "execute":  { "status": "pending",   "artifacts": [] },
    "verify":   { "status": "pending",   "artifacts": [] }
  }
}
```

### Resume

If `/new` is interrupted, it resumes automatically:
- Reads `.build/state.json`
- Finds first non-completed phase
- Displays: `📋 Resuming /new from Phase N: <phase>`
- Continues from there

---

## PHASE TRANSITION DISPLAY

After EVERY phase:

```
✅ Phase 3 (Brand Identity) complete — 8 artifacts
⏳ Starting Phase 4 (DeepUX)...

📋 /new Pipeline Progress:
  ✅ Phase 0 — Scaffold     (project created)
  ✅ Phase 1 — Vision        (VISION.md)
  ✅ Phase 2 — PRD           (17 docs)
  ✅ Phase 3 — Brand         (3 variants + tokens)
  ⏳ Phase 4 — DeepUX        (starting...)
  ⬚ Phase 5 — Plan
  ⬚ Phase 6 — Execute
  ⬚ Phase 7 — Verify
```

---

## FINAL REPORT

When Phase 7 completes, display:

```
+==================================================================+
|  🎉 /new COMPLETE — Product shipped!                               |
|                                                                    |
|  Product: [name]                                                   |
|  Location: /home/hacker/VibeCoding/[category]/[name]               |
|  Duration: [total time]                                            |
|                                                                    |
|  Artifacts:                                                        |
|    VISION.md          — Emotional foundation                       |
|    docs/ (17 files)   — PRD + market research                      |
|    exports/           — Brand tokens + design system               |
|    docs/redesign/     — UX architecture (11-layer specs)           |
|    .planner/          — Implementation plan                        |
|    src/               — Built product                              |
|    .build/state.json  — Full pipeline record                       |
|                                                                    |
|  Quality:                                                          |
|    Build: ✅ 0 errors                                              |
|    Console: ✅ 0 JS errors                                         |
|    Debugaudit: ✅ [score]/100                                      |
|                                                                    |
|  Next:                                                             |
|    → Deploy: vercel --prod --yes --token "$VERCEL_TOKEN"           |
|    → Test deeper: run all 14 Quality Arsenal audits                |
|    → Iterate UX: /deepux --implement-all                           |
+==================================================================+
```

Send via Telegram:
```bash
telegram notify "🎉 /new complete: [name] — all 8 phases done. Ready to deploy."
```

---

## PHASE ROLLBACK & RECOVERY

When a downstream phase discovers an upstream phase produced bad data:

| Trigger | Rollback action |
|---------|----------------|
| Phase 2 (PRD) needs info Phase 1 (Vision) didn't capture | Re-enter Phase 1 with new prompts (targeted, not full re-run), re-derive PRD inputs, continue |
| Phase 4 (DeepUX) finds brand tokens violate the design system | Rollback to Phase 3 (Brand), regenerate conflicting token(s), re-enter Phase 4 |
| Phase 5 (Plan) finds a feature has no UX spec | Rollback to Phase 4 (DeepUX), add the missing page spec, re-enter Phase 5 |
| Phase 6 (Execute) finds `.planner/tracker.json` has impossible dependencies | Rollback to Phase 5, fix the DAG, re-enter Phase 6 from the broken step |
| Phase 7 (Verify) /debugaudit < 100 | Fix bugs within Phase 7 (no rollback); if the bugs trace to a spec defect → rollback to the phase that owns the spec, fix, re-run affected Phase 6 tasks, re-enter Phase 7 |

Rollback writes `.build/state.json.phase_{N}.bak` before rewriting the phase state. Always reversible.

**Sub-step recovery on resume:** If `.build/state.json` shows Phase N status = `running`, check that phase's side-effect artifacts:
- Phase 2 (PRD): look for partial `docs/FEATURES/F-XXX-*.md` — if < expected count, resume from next F-XXX
- Phase 4 (DeepUX): look for partial `docs/redesign/*.md` — resume from next page
- Phase 6 (Execute): `tracker.json` already tracks per-step status; resume from first `pending` step whose `depends_on` are all `completed`

Never blindly restart a phase — always check partial state first.

---

## ANTI-PATTERNS

| Don't | Do instead |
|-------|------------|
| Run /new inside an existing project | Use `/build` — it detects existing artifacts |
| Skip DeepUX for "speed" | DeepUX saves 10x more time during execution |
| Start coding before Phase 4 | Specs prevent rewrites |
| Run all phases without checking artifacts | Each phase verifies previous outputs |
| Forget to resume after interruption | `.build/state.json` auto-resumes with sub-step recovery |
| Ask for `/new --quick` or "streamlined" mode | REFUSE — rule 46 bans it. Run individual phases standalone if less scope is needed. |
| Skip Phase 7 (verify) because "build passed" | Build passing ≠ product works. /debugaudit is non-negotiable before reporting done. |

---

## SEE ALSO

- `/build` — Same pipeline but for EXISTING projects (no scaffold)
- `/team` — Rapid prototype (less planning, faster shipping)
- `/vision` — Phase 1 standalone
- `/prd` — Phase 2 standalone
- `/brand-identity` — Phase 3 standalone
- `/deepux` — Phase 4 standalone
- `create a plan` — Phase 5 standalone
- `/team` — Phase 6 execution tool
- `/debugaudit` — Phase 7 verification tool
- `/new-project` — Phase 0 scaffolding only

---

**/new v1.1 — "From spark to shipped."**
*8 phases | Resumable with sub-step recovery | Context-propagated | Phase rollback on spec defects | Full lifecycle, no shortcuts*
*Created: 2026-03-12 | Updated: 2026-04-14 (rule 46 compliance, debugaudit canonical, cwd propagation, partial-artifact detection)*

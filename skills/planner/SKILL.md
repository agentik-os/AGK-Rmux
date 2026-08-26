---
name: planner
description: >
  OmegaOS-native implementation planner. Reads Vision/PRD/CLAUDE.md, decomposes the work
  into a DAG of single-worker-dispatch steps, and emits a typed `.planner/tracker.json`
  that the Rust engine (`omega plan-run`) executes with structural can't-skip enforcement
  (Gate) and independent verify-command proof (Guardian). Replaces the prose-only VPS
  planner: sequential execution is enforced by the engine, not by instructions.
  Use when user says "/omg-planner", "plan this", "make a plan", "build the plan", "decompose",
  "planifie", "fais le plan".
allowed-tools: ["Read", "Write", "Edit", "Bash", "Glob", "Grep"]
domain: orchestration
read_only: false
triggers: ["omg-planner", "plan this", "make a plan", "build the plan", "decompose", "planifie"]
---

# /omg-planner — OmegaOS-native Implementation Planner

You produce a **typed plan** that the Rust engine drives. Your job is NOT to execute the
plan step-by-step yourself — you generate `.planner/tracker.json`, then hand it to the
engine. The engine guarantees no step is skipped and no step is "done" without its
`verify_command` passing. Your only job is to make the plan **good**.

## Commands

```bash
/omg-planner [task]      # Generate .planner/tracker.json from Vision/PRD (then offer to run)
/omg-planner run         # Execute the existing plan via the engine  → omega plan-run
/omg-planner status      # Progress dashboard                         → omega plan-status
```

The engine binary is `omega` (built from source by `install.sh`, at `~/.local/bin/omega`).
If `omega` is missing, fall back to `~/.omega/skills/planner/fallback/plan.ts` (Bun).

---

## IRON RULES

### 1. Build from Vision/PRD — ALWAYS
Before writing any plan, read the real context:
```bash
cat CLAUDE.md 2>/dev/null
cat Vision/VISION.md 2>/dev/null || cat Vision/*.md 2>/dev/null
find . -iname "PRD*" -o -iname "prd*" | head -5
git log --oneline -10 2>/dev/null
```
Every step must trace to a Vision/PRD requirement. No inventing features.

### 2. Every step is ONE worker-dispatch unit
If a step cannot be claimed and finished by ONE worker in ONE session, split it. If two
steps must share a file/commit/verification, merge them. Disjoint `files_to_touch` across
steps in the same wave (the engine runs file-disjoint ready steps in parallel).

### 3. Every step carries the 7 fields the engine requires
`step_id`, `title`, `description` (80+ chars, precise), `files_to_touch` (≥1 exact path,
never empty, never `["src/"]`), `done_criteria` (testable), `verify_command` (a single
shell line that exits 0 iff the step is truly done — the Guardian RE-RUNS this; "true" is
never acceptable for real work), `depends_on` (exact step_ids; the DAG must be acyclic).

### 4. The engine enforces sequencing — you only declare the DAG
Do NOT write "do steps in order" instructions. Encode order in `depends_on`. The engine's
`ready_steps` selector will only ever hand a worker a step whose deps are all `done`. You
cannot make it skip, and you don't need to.

### 5. One audit PER MODULE — not a single terminal audit
Group steps into **modules** (`M01`, `M02`, … — a coherent slice: schema, auth, a feature,
the API surface, security, deploy). **Every module ends with its own audit step** that
`depends_on` all of that module's implementation steps and whose `verify_command` checks the
audit verdict (`test -f audits/.<audit>/verdict.json` AND a score gate, e.g.
`jq -e '.score>=85' audits/.codeaudit/verdict.json`). Pick the audit that fits the module
(`/omg-codeaudit`, `/omg-secaudit`, `/omg-apiaudit`, `/omg-dataaudit`, `/omg-flowaudit`…).
A module is NOT "done" until its audit passes — so a bug is caught at the module boundary,
not discovered at the very end. **Per-module audit steps carry NO `wave`** — their gating is
the DAG alone (`depends_on` that module's steps); later modules `depends_on` the prior
module's audit step. `wave: "audit"` is a TERMINAL tier: the engine withholds any
audit-wave step until every lower-wave step in the WHOLE plan is `done`, so tagging a
module audit `wave: "audit"` defers it to the very end — exactly the failure this rule
prevents. Reserve `wave: "audit"` / `"deploy"` for the final cross-cutting audit and
deploy steps only. (That final audit wave is fine on top, but per-module gates are
mandatory.)

### 6. The plan must be COMPLETE and CONTIGUOUS — prod-ready, no gaps
A plan that jumps between areas and declares "done" with whole layers missing is the #1
failure (e.g. leaping `STEP-050` → `STEP-269`). The DAG MUST cover every layer the product
needs to actually run in prod: **database/schema · backend/business logic · API surface ·
frontend/UI · auth & security · integrations · tests · deploy**. Walk the PRD feature list
AND the stack: each requirement maps to ≥1 step; each module ends with its audit (rule 5).
Number `step_id`s densely and contiguously — no gaps. Before handoff, self-check: "if the
engine runs every step to `done`, is the result a working, deployable app with backend,
frontend, database, security and API all wired?" If a layer is unrepresented, the plan is
incomplete — add the steps. NEVER thin the plan to finish faster; the engine faithfully
executes exactly what you give it, so a sparse plan ships a broken app.

### 7. `verify_command` must prove it WORKS at RUNTIME — not just that it builds
`npm run build` / `tsc` / `lint` passing means the code COMPILES — it says NOTHING about
whether a page renders, a route exists, or a flow works. The classic failure: a real build
that ships a **404 on `/sign-in`** and a broken login. So:
- **Rely on exit codes — NEVER pipe through `grep -v`.** A pipe swallows the left command's
  exit code, and `grep -qv 'error'` exits 0 whenever ANY line lacks the pattern — i.e. it
  passes even when the error IS present. A guaranteed fake-pass. If you must grep output,
  invert a positive match: `! (cmd 2>&1 | grep -qi 'schema error')` — and prefer the
  command's own exit code (`npx convex dev --once` exits non-zero on schema errors).
- **Every page/route step's verify must hit the running app and assert it actually RENDERS** —
  HTTP 200 + an expected substring, never a 404/500. Pattern (build once, serve, curl):
  `curl -fsS http://127.0.0.1:$PORT/sign-in | grep -q "Sign in"` (a bare `curl` without `-f`
  passes on a 404 — use `-f`/check the status). For interactive flows use a Playwright assert.
- **ROUTE COMPLETENESS**: enumerate EVERY route the app references and give each its own step +
  render-verify — the landing, every nav link, every CTA `href`, and **every auth route**
  (`/sign-in`, `/sign-up`, the SSO/OAuth callback). A `*_URL` env that points somewhere
  (`NEXT_PUBLIC_CLERK_SIGN_IN_URL=/sign-in`) is a CONTRACT: the step that sets it MUST also
  create the page it points to, or that route is a guaranteed 404. Trace each `redirect`,
  `<Link href>`, and `*_URL` to a real page step.
- **AUTH-BRIDGE WIRING is a real step, not an assumption.** For Clerk+Convex the plan MUST
  include a step that creates/verifies the Clerk `convex` JWT template (token `aud:"convex"`
  matching `auth.config.ts` `applicationID`) — without it every authenticated mutation throws
  `Unauthenticated` while the build is green. Its `verify_command` mints a real token and asserts
  an authenticated backend call SUCCEEDS, never just that the file compiles.
- **The final gate is a real END-TO-END browser sweep + self-heal, not unit smokes that
  bypass the UI.** The plan's terminal step MUST run the shipped autonomous gate
  **`/omg-acceptance`** (harness: `node ~/.omega/skills/acceptance/sweep.cjs`): build → serve
  on `127.0.0.1` → sweep every route + console + network + the authenticated golden path →
  AUTONOMOUSLY fix any failure and re-run until green. Set that step's `verify_command` to the
  sweep itself (`node ~/.omega/skills/acceptance/sweep.cjs` with the project's `ROUTES`/
  `PROTECTED_ROUTES`/`GOLDEN_PATH`/`CLERK_*`/`TEST_*` env) so the Guardian independently
  re-confirms green. Concretely the step is a **Playwright
  agent that NAVIGATES to every page AND walks the full golden path (real auth: click Sign in →
  complete login → land in-app → **do the core authenticated action and assert the write
  PERSISTED**)** and FAILS on any 404 / console error / **failed network request (status ≥400,
  e.g. a `tokens/convex` 404 or an `Unauthenticated` mutation)** / broken flow. "Renders 200"
  ≠ "the authenticated backend works" — the sweep MUST exercise one real logged-in mutation
  round-trip. That step's `verify_command` runs the sweep and exits non-zero if any page/flow
  is broken.
  **Serve + sweep on `http://127.0.0.1:$PORT` (a secure context), never `http://<IP>`** —
  Clerk/WebCrypto auth only initialises on `localhost`/`127.0.0.1` or HTTPS; on a raw
  `http://<IP>` origin it dies with `secure-context:false` / `undefined ... 'digest'`, so a
  sweep there is a false pass that can't even reach the login crypto. Prod must be HTTPS.

---

## Output: `.planner/tracker.json` (EXACT schema — the engine parses this)

Write valid JSON matching the Rust `PlanTracker` type exactly. Fields `wave`, `attempt`,
`started_at`, `completed_at` accept defaults (`null`/`0`) but include them for clarity.
`status` is always `"pending"` at generation time.

```json
{
  "project": "ProjectName",
  "total_phases": 2,
  "active_phase": 1,
  "planner_version": "7.0",
  "generated_at": "2026-05-31T00:00:00Z",
  "phases": [
    { "id": 1, "name": "Foundation", "goal": "Schema + auth — everything builds on this",
      "step_ids": ["STEP-001", "STEP-002"] },
    { "id": 2, "name": "Audit", "goal": "Forensic quality gate", "step_ids": ["STEP-003"] }
  ],
  "steps": [
    {
      "step_id": "STEP-001", "phase": 1,
      "title": "Create Convex bookings schema with indexes",
      "description": "Define convex/schema.ts with a bookings table (userId, slotId, status, createdAt) plus search indexes on userId and slotId. Validate with Convex validator types so `npx convex dev` accepts it.",
      "files_to_touch": ["convex/schema.ts"],
      "done_criteria": "npx convex dev --once starts with no schema error and bookings appears in _generated/api",
      "verify_command": "npx convex dev --once",
      "depends_on": [],
      "wave": "foundation", "attempt": 0,
      "status": "pending", "started_at": null, "completed_at": null
    },
    {
      "step_id": "STEP-002", "phase": 1,
      "title": "Add booking create mutation",
      "description": "Implement convex/bookings.ts create() mutation taking {slotId,userId}, inserting a bookings row with status='pending'. Reject double-booking of the same slot.",
      "files_to_touch": ["convex/bookings.ts"],
      "done_criteria": "npm run build passes AND the mutation is callable",
      "verify_command": "npm run build",
      "depends_on": ["STEP-001"],
      "wave": "w1", "attempt": 0,
      "status": "pending", "started_at": null, "completed_at": null
    },
    {
      "step_id": "STEP-003", "phase": 2,
      "title": "/codeaudit on the booking MVP",
      "description": "Run the forensic code audit scoped to convex/** to verify the booking implementation is solid before ship.",
      "files_to_touch": ["audits/.codeaudit/verdict.json"],
      "done_criteria": "codeaudit normalized score >= 85/100",
      "verify_command": "test -f audits/.codeaudit/verdict.json",
      "depends_on": ["STEP-001", "STEP-002"],
      "wave": "audit", "attempt": 0,
      "status": "pending", "started_at": null, "completed_at": null
    }
  ]
}
```

Valid `wave` values: `foundation | w1 | w2 | w3 | audit | deploy` (or omit / `null`).

Optional `timeout_mins` (number, per step): overrides the engine's worker timeout for that
step. Defaults: 30 min per worker; `audit`/`deploy`-wave steps get 4x (120 min) because
forensic audits and the `/omg-acceptance` sweep routinely outlive 30 min. Set it explicitly
on any step you know runs long (big builds, full browser sweeps).

## Validation before handing to the engine
After writing `tracker.json`, sanity-check it yourself:
```bash
# valid JSON?
python3 -c "import json;json.load(open('.planner/tracker.json'))" && echo "JSON ok"
# engine accepts it + shows the DAG (this also proves no cycle / valid schema):
omega plan-status . 2>&1 || echo "engine could not load the plan — fix the schema/DAG"
```
`omega plan-status` printing the steps with `ready N | blocked M` confirms the engine
parsed the typed plan and computed the DAG. If it errors, fix the JSON to match the schema
above — do not weaken the plan to satisfy a parse error.

## Execute
```bash
omega plan-run .     # the engine drives: ready-set -> spawn worker -> Guardian verify -> advance
omega plan-status .  # watch progress
```

If `omega` is not on PATH, run the Bun fallback:
```bash
bun ~/.omega/skills/planner/fallback/plan.ts status .
bun ~/.omega/skills/planner/fallback/plan.ts run .
```

## Anti-patterns (rejected at validation)
| Failure | Fix |
|---|---|
| `verify_command: "true"` for real work | A real falsifiable command (build/test/curl) |
| `files_to_touch: []` or `["src/"]` | Exact file paths, ≥1 |
| "do steps in order" prose | Encode order in `depends_on` — the engine enforces it |
| Audits as a separate Phase 7 | Per-module audit steps gated by `depends_on` (no wave); only the final cross-cutting audit gets `wave: "audit"` |
| One giant step ("build the feature") | Split into one-worker-dispatch units |

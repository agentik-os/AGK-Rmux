---
name: planner
description: >
  Implementation planner v6.0 — builds from Vision/PRD, max 25 tasks per phase,
  up to 1500+ total. Professional prompts. Strict sequential execution.
  Oracle-compatible: dispatches to /team sessions.
allowed-tools: ["Read", "Write", "Edit", "Bash", "Glob", "Grep", "Agent"]
---

# /planner v6.0 — Implementation Planner

## Commands

```bash
/planner [task]              # Full plan from Vision/PRD (25 tasks per phase)
/planner mini [task]         # Quick plan (5-15 tasks)
/planner status              # Progress overview
/planner next                # Next task with full context
/planner done STEP-N         # Mark complete (auto-verify criteria)
/planner execute             # Execute next task yourself
/planner execute-all         # Execute ALL remaining tasks one by one
/planner replan              # Regenerate from current state
```

---

## IRON RULES

### 1. Build from Vision/PRD — ALWAYS

Before creating ANY plan:
```bash
cat CLAUDE.md                                        # Project context
cat Vision/VISION.md 2>/dev/null || cat Vision/*.md  # Product vision
cat Info/V3-VISION.md 2>/dev/null                    # Alt vision location
find . -name "PRD*" -o -name "prd*" | head -5        # PRD docs
git log --oneline -10                                # Recent work
find . -type f -name "*.ts" -o -name "*.tsx" | grep -v node_modules | wc -l  # Codebase size
```

Every task must trace back to a Vision/PRD requirement. No inventing features.

### 2. Every task MUST have 5 mandatory fields

```json
{
  "id": "STEP-001",
  "description": "Create the Convex schema with users table (email, name, clerkId, role, orgId) and organizations table (name, ownerId, plan, createdAt). Add search indexes on email and clerkId. Validate with Convex validator types.",
  "files": ["convex/schema.ts"],
  "criteria": "npx convex dev starts without schema errors, both tables visible in dashboard",
  "depends_on": [],
  "status": "pending"
}
```

| Field | Minimum | Bad Example | Good Example |
|-------|---------|-------------|--------------|
| description | 80+ chars | "Set up auth" | "Create Clerk middleware at src/middleware.ts checking auth on all /app/* routes. Redirect unauthenticated to /sign-in. Pass userId to Convex via server-side headers." |
| files | 1+ specific paths | `[]` or `["src/"]` | `["src/middleware.ts", "convex/auth.ts"]` |
| criteria | testable command | "it works" | "npm run build passes AND visiting /app/dashboard while logged out redirects to /sign-in" |

### 3. Max 25 tasks per phase — unlimited phases

- Phase = focused chunk of work (Foundation, Core, Features, Polish)
- 25 tasks max per phase = high quality prompts
- Total project can have 60+ phases, 1500+ tasks
- tracker.json holds ALL phases, marks active phase
- Complete Phase N before starting Phase N+1

### 4. ABSOLUTE sequential execution — ZERO tolerance

```
STEP-001 → STEP-002 → STEP-003 → ... → STEP-N
```

- Execute steps IN ORDER. No exceptions.
- NEVER skip a step. Not even one.
- NEVER jump from STEP-005 to STEP-050.
- Each step: read task → implement → verify criteria → mark done → NEXT
- If blocked: mark "blocked" with reason, resolve it, then do the step
- Agent MUST check: "what is STEP-{current+1}?" and do ONLY that

### 5. Task quality — professional engineer level

Each task description must be detailed enough that a developer can implement it WITHOUT asking questions:

```
BAD:  "Add payments"
OK:   "Create Stripe checkout endpoint"
GOOD: "Create POST /api/checkout route at src/app/api/checkout/route.ts.
       Accept JSON body {priceId: string, userId: string}.
       Call stripe.checkout.sessions.create() with mode='subscription',
       success_url='/dashboard?session_id={CHECKOUT_SESSION_ID}',
       cancel_url='/pricing'. Return {url: session.url}.
       Handle Stripe errors with 400 status + error message."
```

---

## Planning Process

### Step 1: READ Vision + Codebase (3 min)

Read ALL reference docs. Understand what EXISTS and what's NEEDED.
Map every Vision section to implementation tasks.

### Step 2: DECOMPOSE into Phases (5 min)

| Phase | Focus | Tasks |
|-------|-------|-------|
| Phase 1 | Foundation — schema, auth, config | 15-25 |
| Phase 2 | Core Backend — API routes, mutations, queries | 15-25 |
| Phase 3 | Core Frontend — pages, components, layouts | 15-25 |
| Phase 4 | Features — business logic, integrations | 15-25 |
| Phase 5 | Wiring — connect frontend to backend | 15-25 |
| Phase 6+ | Advanced features, polish, tests, deploy | 15-25 |

### Step 3: Write tracker.json

```json
{
  "project": "ProjectName",
  "vision_file": "Vision/VISION.md",
  "total_phases": 6,
  "active_phase": 1,
  "planner_version": "6.0",
  "generated_at": "2026-03-31T00:00:00Z",
  "phases": [
    {
      "id": 1,
      "name": "Foundation",
      "goal": "Schema, auth, project config — everything builds on this",
      "tasks": ["STEP-001", "STEP-002", "...", "STEP-025"]
    }
  ],
  "tasks": [
    {
      "id": "STEP-001",
      "phase": 1,
      "description": "[80+ chars, precise, professional]",
      "files": ["path/to/file.ts"],
      "criteria": "[testable command or verification]",
      "depends_on": [],
      "status": "pending"
    }
  ]
}
```

### Step 4: VALIDATE before saving

- [ ] Every task has description (80+ chars, specific)
- [ ] Every task has files (exact paths, never empty)
- [ ] Every task has criteria (testable)
- [ ] Active phase has <= 25 tasks
- [ ] IDs are sequential (no gaps)
- [ ] Tasks trace to Vision/PRD requirements

---

## Oracle Integration

When an Oracle uses /planner:
1. Oracle reads tracker.json
2. For each pending task in order:
   ```bash
   ~/.aisb/lib/dispatch-to-session.sh {Project}-plan-N '/team [task description + criteria]' {path}
   ```
3. Monitor with tmux capture-pane
4. When worker done: verify criteria → mark done in tracker
5. Next task. NEVER skip.

## God Mode + Planner

In God Mode, the Oracle:
1. Generates the full plan from Vision
2. Executes phase by phase, task by task
3. After each phase: npm run build, git commit
4. Moves to next phase
5. Loops until ALL phases complete

---

## Linear Sync (optional)

When the planner finishes generating `tracker.json`, it can mirror the plan into
Linear so a collaborator (e.g. Manon) can watch progress visually in the Linear
UI. This is **off by default and never blocks the planner**.

### Activation

The sync activates if `LINEAR_API_KEY` (a `lin_api_*` token) is present in:
1. `./.env.local` in the project root, OR
2. Any ancestor `.env.local` up to 5 levels up, OR
3. The `LINEAR_API_KEY` env var.

If no key is found, print a yellow warning and continue — the planner is **not
blocked**:

```
⚠ Linear sync skipped (no LINEAR_API_KEY in .env.local) — planner continues
```

### What gets created

1. **One Linear Project** named `<ProjectName>-Planner-<YYYYMMDD>`.
2. **One Linear Issue per step** in `tracker.json`, with:
   - Title: `<STEP-ID> — <first 80 chars of description>`
   - Description: full step description + acceptance criteria + file list
   - Initial state: team's backlog/unstarted state
   - Project: the project created in step 1

### Idempotency — `.planner/linear-mapping.json`

After the first successful sync, the planner writes
`.planner/linear-mapping.json`:

```json
{
  "linear_project_id": "uuid",
  "linear_project_name": "ProjectName-Planner-20260513",
  "linear_team_id": "team-uuid",
  "linear_team_key": "PRJ",
  "created_at": "2026-05-13T15:00:00Z",
  "mapping": {
    "STEP-001": {"issue_id": "uuid", "identifier": "PRJ-123", "url": "https://linear.app/..."},
    "STEP-002": {"issue_id": "uuid", "identifier": "PRJ-124", "url": "..."}
  }
}
```

On re-run (`/planner replan` or a fresh `/planner` after adding tasks):
- If `linear-mapping.json` exists → skip `create-project`, reuse `linear_project_id`.
- Loop steps; only call `create-issue` for steps NOT already in `mapping{}`.
- Append new entries to `mapping{}` and rewrite the file.
- Never duplicate existing issues.

**Stale-mapping validation on `replan`.** When invoked as `/planner replan`,
each mapped Linear issue is validated with a cheap GET before it is trusted.
If the issue has been deleted (Linear returns null / 404 equivalent in the
GraphQL `issue(id:…)` query), the entry is removed from `mapping{}` so the
next pass through the `create-issue` loop re-creates it. Without this step
a user who deletes an issue in the Linear UI would end up with a step
permanently orphaned (mapped to a tombstone), since the mapping is the
sole idempotency key.

This step is opt-in (only the `replan` subcommand). A normal `/planner` run
trusts the mapping as-is. The stale-check loop itself executes **inside the
same `flock` subshell** as the project-resolve and issue-create steps
(cycle 4 finding D1) — see the implementation pseudocode below. Running it
outside the lock would let two parallel `/planner replan` invocations race
on `$MAPPING_FILE` rewrites: each reads the same baseline, each deletes a
different stale entry, last-writer wins → genuine deletions get reverted.
Single critical section covers ALL mapping mutations.

### Implementation (pseudocode)

Key discovery is **delegated to `linear-sync.sh`** so that env var + ancestor
`.env.local` walking + quote stripping all stay in one place. Trap exit code
2 as the documented "graceful skip" signal — never reimplement env detection
here.

```bash
MAPPING_FILE=".planner/linear-mapping.json"
PROJECT_NAME_FULL="${PROJECT_NAME}-Planner-$(date +%Y%m%d)"

# Serialize concurrent planners on the same project via flock.
# The entire critical section — project-create-if-missing AND issue-create
# loop — runs inside the lock. Two oracles invoking /planner on the same
# cwd simultaneously would otherwise both pass the mapping-file existence
# check and double-create the Linear project + race on mapping rewrites.
mkdir -p ~/.aisb/locks
# cycle 4 D4: hash the FULL cwd instead of just basename("$PWD").
# basename collisions (e.g. ~/clients/AcmeCorp/api vs ~/clients/Causio/api
# both → "api") would otherwise serialize unrelated projects on the same
# lock file → 30-min flock timeout fires → loser silently skips Linear sync.
LOCK_FILE="$HOME/.aisb/locks/linear-sync-$(printf '%s' "$PWD" | sha1sum | cut -c1-12).lock"

(
  # Bounded wait (cycle 2 finding B3): 30-minute cap prevents indefinite
  # deadlock if a holder process dies without releasing. On timeout we
  # log a warning and skip Linear sync — planner continues without sync
  # rather than blocking the entire pipeline.
  if ! flock -w 1800 -x 200; then
    echo "⚠ Linear sync: could not acquire lock within 30min — skipping sync (planner continues)" >&2
    exit 0
  fi

  # Double-checked locking (cycle 3 finding C3): re-check mapping file
  # existence INSIDE the lock. If a peer oracle created it while we were
  # waiting for the lock, reuse its project. Otherwise create-project +
  # seed the mapping file — both as atomic steps within the lock.
  if [ -f "$MAPPING_FILE" ]; then
    PROJECT_ID=$(jq -r '.linear_project_id' "$MAPPING_FILE")
    TEAM_ID=$(jq -r '.linear_team_id' "$MAPPING_FILE")
  else
    PROJ_JSON=$(~/.claude/lib/linear-sync.sh create-project "$PROJECT_NAME_FULL" 2>&1)
    rc=$?
    if [ $rc -eq 2 ]; then
      echo "⚠ Linear sync skipped inside lock (no LINEAR_API_KEY) — planner continues"
      exit 0
    elif [ $rc -ne 0 ]; then
      echo "⚠ Linear create-project failed inside lock (rc=$rc) — planner continues"
      exit 0
    fi
    PROJECT_ID=$(echo "$PROJ_JSON" | jq -r '.project_id')
    TEAM_ID=$(echo "$PROJ_JSON" | jq -r '.team_id')
    TEAM_KEY=$(echo "$PROJ_JSON" | jq -r '.team_key')
    mkdir -p .planner
    # cycle 4 D2: same-filesystem temp so `mv` is rename(2) atomic.
    # Default mktemp targets /tmp (tmpfs on this VPS) → cross-fs mv
    # falls back to copy+unlink (NOT atomic). Pinning to .planner/
    # ensures the move is a kernel-level rename and survives crashes.
    TMP_INIT=$(mktemp -p .planner)
    jq -n --arg pid "$PROJECT_ID" --arg pname "$PROJECT_NAME_FULL" \
          --arg tid "$TEAM_ID" --arg tk "$TEAM_KEY" \
          --arg ts "$(date -Iseconds)" '{
      linear_project_id:$pid, linear_project_name:$pname,
      linear_team_id:$tid, linear_team_key:$tk,
      created_at:$ts, mapping:{}
    }' > "$TMP_INIT" && mv "$TMP_INIT" "$MAPPING_FILE"
  fi

  # ─── REPLAN-ONLY: stale-mapping check (cycle 4 finding D1) ────────
  # Runs ONLY on `/planner replan`. Lives INSIDE the flock subshell so
  # parallel `replan` invocations cannot lose deletions: each
  # $MAPPING_FILE rewrite is serialized with project-create and
  # issue-create within the same critical section. The cycle3 fix
  # moved project-create inside the lock but left this loop outside —
  # cycle4 D1 closes that gap. Stale-mapping detection itself follows
  # cycle 3 finding C1 (data.issue:null + HTTP 200 + no errors[] is
  # Linear's real deletion signal) and cycle 2 finding B1 (transient
  # errors must preserve mapping — never over-delete).
  if [ "${MODE:-init}" = "replan" ]; then
    for step_id in $(jq -r '.mapping | keys[]' "$MAPPING_FILE"); do
      IID=$(jq -r --arg s "$step_id" '.mapping[$s].issue_id' "$MAPPING_FILE")
      HTTP_BODY=$(curl -sS --max-time 10 -w '\n__HTTP_STATUS__%{http_code}' -X POST "$API_URL" \
        -H "Authorization: $LINEAR_API_KEY" -H "Content-Type: application/json" \
        -d "$(jq -n --arg id "$IID" '{query:"query($id:String!){issue(id:$id){id}}",variables:{id:$id}}')") \
        || { echo "⚠ replan: stale-check network error for $step_id — keeping mapping (best-effort)" >&2; continue; }
      HTTP_CODE=$(printf '%s' "$HTTP_BODY" | sed -n 's/.*__HTTP_STATUS__//p')
      RESP=$(printf '%s' "$HTTP_BODY" | sed '$ s/__HTTP_STATUS__.*$//')
      PARSED=$(echo "$RESP" | jq -e . >/dev/null 2>&1 && echo yes || echo no)
      DELETED="no"
      if [ "$PARSED" = "yes" ]; then
        HAS_ISSUE_KEY=$(echo "$RESP" | jq -r 'if .data and (.data | has("issue")) then "yes" else "no" end')
        ISSUE_NULL=$(echo "$RESP" | jq -r 'if .data and (.data | has("issue")) and (.data.issue == null) then "yes" else "no" end')
        ERRORS_EMPTY=$(echo "$RESP" | jq -r 'if (.errors // [] | length) == 0 then "yes" else "no" end')
        NOT_FOUND=$(echo "$RESP" | jq -r '[.errors[]? | select(.extensions.code == "NOT_FOUND")] | length')
        if [ "$HTTP_CODE" = "200" ] && [ "$HAS_ISSUE_KEY" = "yes" ] && [ "$ISSUE_NULL" = "yes" ] && [ "$ERRORS_EMPTY" = "yes" ]; then
          DELETED="yes"  # Linear's real deletion signal
        elif [ "${NOT_FOUND:-0}" -gt 0 ]; then
          DELETED="yes"  # alternative NOT_FOUND signal
        fi
      fi
      if [ "$DELETED" = "yes" ]; then
        # mktemp -p .planner (cycle 4 D2): keep TMP on the same
        # filesystem as $MAPPING_FILE so `mv` is rename(2), atomic.
        TMP=$(mktemp -p .planner)
        jq --arg s "$step_id" 'del(.mapping[$s])' "$MAPPING_FILE" > "$TMP" && mv "$TMP" "$MAPPING_FILE"
        echo "↻ replan: stale mapping for $step_id deleted (Linear data.issue:null, http=$HTTP_CODE)"
      else
        HAS_ISSUE=$(echo "$RESP" | jq -r '.data.issue.id // empty' 2>/dev/null)
        if [ -z "$HAS_ISSUE" ]; then
          echo "⚠ replan: stale-check inconclusive for $step_id (http=$HTTP_CODE) — keeping mapping" >&2
        fi
      fi
    done
  fi

  jq -r '.tasks[].id' .planner/tracker.json | while read -r step_id; do
    if jq -e --arg s "$step_id" '.mapping[$s]' "$MAPPING_FILE" >/dev/null 2>&1; then
      continue
    fi
    TITLE=$(jq -r --arg s "$step_id" '.tasks[] | select(.id==$s) | "\(.id) — \(.description[0:80])"' .planner/tracker.json)
    DESC=$(jq -r --arg s "$step_id" '.tasks[] | select(.id==$s) | .description + "\n\nFiles: \(.files | join(", "))\n\nAcceptance: \(.criteria)"' .planner/tracker.json)
    ISSUE_JSON=$(~/.claude/lib/linear-sync.sh create-issue "$PROJECT_ID" "$TITLE" "$DESC" 2>/dev/null) || continue
    # Atomic update under lock: rewrite mapping file with new entry merged in.
    # mktemp -p .planner (cycle 4 D2): same-filesystem temp → mv = rename(2).
    TMP=$(mktemp -p .planner)
    jq --arg s "$step_id" --argjson e "$ISSUE_JSON" '
      .mapping[$s] = {issue_id:$e.issue_id, identifier:$e.identifier, url:$e.url}
    ' "$MAPPING_FILE" > "$TMP" && mv "$TMP" "$MAPPING_FILE"
  done
) 200>"$LOCK_FILE"
```

The `mv` is atomic on the same filesystem, so a crash mid-loop leaves a
valid mapping file (with only the issues created so far). The next run
picks up where it left off — idempotency by file persistence. The `flock`
now wraps the entire critical section (project-create + mapping-init +
issue-create loop), so parallel oracles can no longer race on either the
project creation OR the mapping rewrite (cycle 3 finding C3).

The planner reports a one-line summary after the loop:
```
✓ Linear sync: project PRJ-123 created, 47 issues mapped → .planner/linear-mapping.json
```

If `linear-sync.sh` exits non-zero on any call, log the error and continue — the
sync is best-effort and never blocks plan generation.

---

## Anti-Patterns (real failures v4)

| Failure | Cause | v6 Fix |
|---------|-------|--------|
| 420 empty tasks | No description/files/criteria | Rule 2: 5 mandatory fields |
| Jumped STEP-150 to STEP-450 | No sequential enforcement | Rule 4: zero tolerance |
| "Done" without work | No criteria verification | Criteria = testable command |
| Plan too big for one file | 420 tasks in one tracker | Rule 3: 25 per phase |
| Vague "set up auth" tasks | Low quality descriptions | Rule 5: 80+ chars, professional |
| Plan ignored Vision | Didn't read reference docs | Rule 1: Vision/PRD mandatory |

---

## Granularity Standard (Agentik-Academy-3 lineage)

> Reference plan that produced this standard: **Agentik-Academy oracle-3**, which dispatched 36+ workers in 5 waves (foundation → w1/w2/w3 → audit → deploy) with zero file collisions and clean .done.json on every worker. The Agentik-Academy-3 lineage is the canonical example of "right-sized" planner output — never coarser, never finer.

**Every task emitted by `/planner` MUST be a single-worker-dispatch unit.** If a task cannot be claimed and finished by ONE worker in ONE session without spawning sub-workers of its own, it is too coarse — split it. Conversely, if two tasks are so tightly coupled that they MUST be done together (same file, same commit, same verification), merge them.

### Required fields per task (Agentik-Academy-3 grade)

In addition to the 5 mandatory fields (Rule 2 above), every task in `tracker.json` MUST also carry:

| Field | Type | Purpose | Example |
|---|---|---|---|
| `subject` | string (imperative, ≤80 chars) | Telegram-friendly title + dispatch label | `"Create Convex bookings schema with indexes"` |
| `activeForm` | string (present-continuous, ≤80 chars) | Live-progress UI ("what is happening NOW") | `"Creating Convex bookings schema with indexes"` |
| `files_owned` | array of paths/globs | Worker scope-claim manifest; oracle exports as `WORKER_FILES_OWNED=...` at dispatch | `["convex/schema.ts", "convex/bookings.ts"]` |
| `wave` | enum (`foundation`\|`w1`\|`w2`\|`w3`\|`audit`\|`deploy`) | Scheduling tier (see Wave Decomposition below) | `"foundation"` |
| `estimated_minutes` | integer (5..240) | Drives `ScheduleWakeup` interval + wave packing | `45` |
| `verify_command` | string (one-liner shell) | Worker's last-line proof before `.done.json` | `"npx convex dev --once && grep -q bookings convex/_generated/api.d.ts"` |
| `reference_files` | array of paths (optional) | Existing patterns to port from (e.g. `~/.aisb/docs/PATTERNS-AGENTIK-ACADEMY-3.md`) | `["~/.aisb/docs/PATTERNS-AGENTIK-ACADEMY-3.md#booking-pattern"]` |

`description` (Rule 2) remains the **what + why** explainer (80+ chars). `criteria` (Rule 2) remains the **measurable DONE condition** (e.g. `"npm run build passes AND /bookings returns 200"`). `verify_command` is the *one-shell-line proof* the worker echoes right before invoking `worker-mark-done.sh done_clean`.

### One task = one dispatch (anti-rules)

- ❌ A task titled "Build the booking flow" that needs schema + API + UI + tests → **split into 4 tasks**, each in the right wave
- ❌ A task with `files_owned: []` or `["src/"]` → **rejected**, oracle cannot claim scope
- ❌ A task where `criteria` and `verify_command` are the same prose ("it works") → **rejected**, no falsifiability
- ✅ A task `STEP-007 Create Convex bookings schema with indexes` owning exactly `convex/schema.ts` + `convex/bookings.ts`, wave=foundation, estimated_minutes=45, verify_command runnable in one shell line

### Granularity gut-check (per task)

Before emitting any task, the planner must answer YES to all four:

1. Can ONE worker in ONE session finish this without spawning sub-workers? (if NO → split)
2. Is the file footprint disjoint from every other task in the same wave? (if NO → re-wave or merge)
3. Can a human read `subject + criteria + verify_command` and judge done/not-done in <60s? (if NO → rewrite)
4. Does `estimated_minutes` fit the wave budget (foundation ≤90, w1-w3 ≤120, audit ≤180, deploy ≤60)? (if NO → split)

---

## Wave Decomposition

> Every plan groups its tasks into **waves**. Foundation is sequential and gates everything. Waves 1/2/3 run in parallel because their `files_owned` sets are disjoint. Audit and Deploy are terminal waves.

### The 6 canonical waves

| Wave | Parallelism | Goal | Typical contents | Wave budget (per task) |
|---|---|---|---|---|
| `foundation` | **sequential** | Schema, types, deps, env, auth shell — everything later waves depend on | Convex schema, Prisma migrations, Clerk middleware, tsconfig, shared types | ≤90 min/task |
| `w1` | parallel | Backend API surface | Convex queries/mutations/actions, REST endpoints, webhooks | ≤120 min/task |
| `w2` | parallel | Core UI surface | Pages, layouts, primary components, routes | ≤120 min/task |
| `w3` | parallel | Integrations + secondary features | Stripe, email, analytics, exports, admin tools | ≤120 min/task |
| `audit` | parallel (forensic) | Quality Arsenal audits (`/codeaudit`, `/uiuxaudit`, `/flowaudit`, `/secaudit`, `/a11yaudit`, `/perfaudit`…) | One worker per audit skill, each starts with `/skillname` on line 1 | ≤180 min/task |
| `deploy` | sequential | Final build + push + Vercel/Convex deploy + smoke verify | `npm run build`, `git push`, `vercel --prod`, prod URL 200 check | ≤60 min/task |

### Example wave map (booking platform, 12 tasks)

```
foundation (sequential, 3 tasks):
  STEP-001  schema (convex/schema.ts)
  STEP-002  auth middleware (src/middleware.ts)
  STEP-003  shared types (src/types/booking.ts)

w1 backend (parallel, 3 tasks — disjoint files):
  STEP-004  bookings API (convex/bookings.ts)
  STEP-005  payments webhook (src/app/api/stripe/route.ts)
  STEP-006  availability query (convex/availability.ts)

w2 ui (parallel, 3 tasks — disjoint files):
  STEP-007  booking page (src/app/book/page.tsx)
  STEP-008  dashboard (src/app/dashboard/page.tsx)
  STEP-009  booking form (src/components/BookingForm.tsx)

w3 integrations (parallel, 1 task):
  STEP-010  email confirmations (src/lib/email/confirm.ts)

audit (parallel, forensic — 1 worker per skill):
  STEP-011a  /codeaudit --files="convex/**,src/**" --scope="booking MVP"
  STEP-011b  /uiuxaudit --url="https://prod-url/book"
  STEP-011c  /flowaudit --url="https://prod-url/book"

deploy (sequential, 1 task):
  STEP-012  build + push + vercel --prod + smoke 200 on /book
```

### Wave gates

- A wave only opens when the previous wave reports 100% `.done.json` clean
- Inside a parallel wave, NEVER dispatch two workers whose `files_owned` sets intersect (oracle uses `linear-conflict-analyzer.py` or equivalent file-set diff)
- If foundation is incomplete, waves 1/2/3 stay locked (no speculation on unstable types/schema)

---

## Files-Owned Declarations

> Every task declares `files_owned` UP FRONT. The oracle reads this and exports `WORKER_FILES_OWNED="<comma-list>"` when invoking `dispatch-to-session.sh`, so the worker's `scope-claim.sh claim` succeeds atomically and prevents two parallel workers from touching the same file.

### YAML form (planner internal — easy to read)

```yaml
- id: STEP-007
  wave: w2
  subject: "Create booking page with date+slot picker"
  activeForm: "Creating booking page with date+slot picker"
  description: >
    Build /book route at src/app/book/page.tsx using shadcn Calendar +
    custom TimeSlotPicker. Reads availability via useQuery(api.availability.list).
    Submits via useMutation(api.bookings.create). Redirects to /book/confirmed/[id].
  files_owned:
    - "src/app/book/page.tsx"
    - "src/components/booking/TimeSlotPicker.tsx"
  criteria: "npm run build passes AND visiting /book renders calendar without console errors"
  verify_command: "npm run build && curl -s -o /dev/null -w '%{http_code}' https://prod-url/book | grep -q 200"
  estimated_minutes: 90
  depends_on: ["STEP-001", "STEP-003", "STEP-006"]
  reference_files:
    - "~/.aisb/docs/PATTERNS-AGENTIK-ACADEMY-3.md#booking-page-pattern"
  status: pending
```

### JSON form (what lands in `tracker.json` — machine-read)

```json
{
  "id": "STEP-007",
  "phase": 2,
  "wave": "w2",
  "subject": "Create booking page with date+slot picker",
  "activeForm": "Creating booking page with date+slot picker",
  "description": "Build /book route at src/app/book/page.tsx using shadcn Calendar + custom TimeSlotPicker. Reads availability via useQuery(api.availability.list). Submits via useMutation(api.bookings.create). Redirects to /book/confirmed/[id].",
  "files": ["src/app/book/page.tsx", "src/components/booking/TimeSlotPicker.tsx"],
  "files_owned": ["src/app/book/page.tsx", "src/components/booking/TimeSlotPicker.tsx"],
  "criteria": "npm run build passes AND visiting /book renders calendar without console errors",
  "verify_command": "npm run build && curl -s -o /dev/null -w '%{http_code}' https://prod-url/book | grep -q 200",
  "estimated_minutes": 90,
  "depends_on": ["STEP-001", "STEP-003", "STEP-006"],
  "reference_files": ["~/.aisb/docs/PATTERNS-AGENTIK-ACADEMY-3.md#booking-page-pattern"],
  "status": "pending"
}
```

### Oracle dispatch translation

```bash
# Oracle reads STEP-007 from tracker.json, exports the manifest, dispatches:
WORKER_FILES_OWNED="src/app/book/page.tsx,src/components/booking/TimeSlotPicker.tsx" \
  ~/.aisb/lib/dispatch-to-session.sh \
    "${PROJECT}-w2-step-007" \
    "$(jq -r '.tasks[] | select(.id=="STEP-007")' .planner/tracker.json | tee /tmp/step-007.json)" \
    "$PROJECT_PATH"
```

The worker's first action is `scope-claim.sh claim "$TMUX_SESSION"` reading `$WORKER_FILES_OWNED`. If any file is already claimed by another live worker, dispatch fails fast — no silent overwrite.

---

## ScheduleWakeup Hints

> Every plan emits `ScheduleWakeup` interval recommendations so the oracle does NOT poll the tmux pane (rule R-15: oracles never busy-wait — they sleep, then read `.done.json` files).

### Recommended cadence per wave

| Wave | First wakeup after dispatch | Repeat interval until done | Rationale |
|---|---|---|---|
| `foundation` | 300s (5 min) | 300s | Sequential, fast tasks; check often, advance the chain |
| `w1` / `w2` / `w3` (parallel) | 600s (10 min) | 600–1200s | Workers running in parallel; no point checking sooner than the median estimate |
| `audit` | 1800s (30 min) | 1800s | Audit skills are 300–420 phases; first signal usually >20 min in |
| `deploy` | 180s (3 min) | 180s | Build + push + Vercel poll; tight loop |
| Idle (no active wave) | 1200s (20 min) | — | Default cache-friendly idle tick (rule R-15: ≤300s = warm cache; ≥1200s = post-cache, but cheap) |

### Hint emission in tracker.json

For each wave, the planner writes a `wave_hints` block:

```json
{
  "wave_hints": {
    "foundation": { "first_wakeup_s": 300, "repeat_s": 300 },
    "w1":         { "first_wakeup_s": 600, "repeat_s": 900 },
    "w2":         { "first_wakeup_s": 600, "repeat_s": 900 },
    "w3":         { "first_wakeup_s": 600, "repeat_s": 1200 },
    "audit":      { "first_wakeup_s": 1800, "repeat_s": 1800 },
    "deploy":     { "first_wakeup_s": 180, "repeat_s": 180 }
  }
}
```

The oracle reads `wave_hints[current_wave]` and calls `ScheduleWakeup(delaySeconds=…, reason="watching ${wave} workers — ${N} pending")`. Reference: `~/.claude/rules/R-15-schedule-wakeup-protocol.md` (cache-window discipline).

### Anti-patterns to avoid

- ❌ `delaySeconds=60` while waiting on a 30-min audit → burns the 5-min Anthropic cache window 6× for nothing
- ❌ `delaySeconds=3600` while a 5-min foundation task is mid-flight → user thinks the oracle died
- ✅ Match the cadence to the wave's median `estimated_minutes`

---

## Cross-References

Canonical pattern library + reference plan:

- **`~/.aisb/docs/PATTERNS-AGENTIK-ACADEMY-3.md`** — full pattern templates (booking flow, auth shell, payments, admin), each with `subject` / `activeForm` / `files_owned` / `wave` / `verify_command` filled in. **Cite this file in `reference_files` whenever a task ports a known pattern.**
- `~/.aisb/state/archive-Agentik-Academy-oracle-3/` — historical worker-todo / progress / done JSON from the reference plan; useful as a granularity benchmark.
- `~/.aisb/state/oracle-Agentik-Academy-oracle-3.workers.txt` — the 36-worker wave map that this standard generalizes from.
- `~/.claude/rules/47-oracle-end-of-work.md` — explicit ship + freeze-don't-rollback contract that `deploy` wave honors.
- `~/.claude/rules/42-enhanced-orchestration.md` §0 — worker protection rules every dispatched worker inherits.
- `~/.claude/rules/001-smart-routing.md` — Quality Arsenal canon; `audit` wave dispatches MUST start with `/skillname` on line 1 (never paraphrase).

A plan that lacks `subject` / `activeForm` / `files_owned` / `wave` / `estimated_minutes` / `verify_command` on every task is **pre-Agentik-Academy-3 grade** and must be rejected at validation (Step 4) and regenerated.

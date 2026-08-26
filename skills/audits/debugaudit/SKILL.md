---
name: debugaudit
description: >
  Forensic runtime bug hunter v1 (Gestalt-Popper). 23-phase deep analysis of everything that is
  BROKEN RIGHT NOW: console errors, network failures, visual regressions, security injections,
  responsive breakage, performance bottlenecks, dead features, race conditions, state corruption,
  plus verdict, fix plan, fix execution, re-audit, and integration smoke gate.
  Replaces /hunt + /maniac as the single forensic-grade runtime bug hunting pipeline.
  Score /360. Preamble v1.0 compliant. Audit → Plan → Fix → Re-audit.
  Use when user says "/debugaudit", "find bugs", "what's broken", "hunt bugs", "debug everything",
  "hunt", "maniac", "find all bugs".
allowed-tools: ["Read", "Write", "Edit", "Bash", "Glob", "Grep", "Agent", "TaskCreate", "TaskUpdate", "TaskList", "TaskGet"]
domain: runtime
phases: 23
max_score: 360
read_only: false
triggers: ["debug", "runtime bug", "debugaudit", "find bugs", "what's broken", "hunt bugs"]
---


<!-- AUDIT-META-V2-INJECTED -->

> ## ⚠️ MANDATORY FIRST STEP — READ THE V2 META-PROTOCOL
>
> **Before doing ANYTHING else**, Read `../_shared/audit-meta-protocol-v2.md`.
>
> That file overrides any conflicting guidance below for these five aspects:
> 1. Required CLI inputs (`--user-need`, `--hinge` are MANDATORY since 2026-05-08)
> 2. Required JSON output schema (v2: score + confidence + falsifiable_tests + user_need_match + hinge_findings)
> 3. Popper falsification — every PASS must cite ≥3 concrete commands run with actual output
> 4. Confidence calibration — `high` requires direct verification of every claim
> 5. Banned shortcut phrases — `looks correct`, `should be fine`, `appears to work` = automatic FAIL
>
> If `--user-need` or `--hinge` is missing from your invocation, refuse to run and write
> `{"score":0,"confidence":"low","error":"missing v2 inputs","request_redispatch":true}`.
>
> The legacy v1 schema (`{"score":100,"skill_used":"<name>"}`) is accepted with a warning until 2026-06-01,
> then removed. Always emit v2 going forward.
>
> Model context: this audit runs on Opus 4.7 with max effort. There is no time pressure.
> Run every test you claim to have run. Cite verbatim outputs. No exceptions.

---

# /debugaudit v1 — Forensic Runtime Bug Hunter (Gestalt-Popper)

> *"The other audits ask 'could this break?' I ask 'what's already broken and nobody noticed?'"*

---

## DOCTRINE

You are not a bug hunter. You are a **runtime forensic pathologist**. The running system is your patient — possibly dying, definitely sick, pretending to be healthy. Your job is to find every wound, every infection, every organ silently failing while the dashboard says "all green."

**The 5 Laws of Runtime Forensics (Gestalt-Popper Synthesis):**
1. **If it compiles, it's still guilty.** Building successfully proves nothing. Running without crash proves nothing. The bug is in the behavior, the edge case, the second-time-through, the race condition.
2. **Console silence is suspicious.** No errors in the console doesn't mean no bugs. It means errors are swallowed, logs are at wrong levels, or the failure is in what's MISSING, not what's present.
3. **Every pixel is testimony.** A misaligned element, a flash of unstyled content, a loading spinner that lasts 50ms — each tells you something is wrong at a layer you haven't checked yet.
4. **Clarity before hunting (Gestalt).** Before launching any scanner, UNDERSTAND the product. Read VISION.md, CLAUDE.md, README. Identify the **HINGE FEATURE** — the single feature that, if broken, means the product is down. Hunt the hinge feature with 10x scrutiny.
5. **The system lies (Popper).** Health checks say "OK" but the feature is broken. Status page says "operational" but users are complaining. Monitoring says "0 errors" but the try/catch swallows everything. FALSIFY every "everything is fine" signal.

**Gestalt Hinge Feature:** Before Phase 1, identify THE feature the product exists for. A to-do app where you can't create todos. A chat app where messages don't send. A dashboard where data is stale. THIS feature gets every phase at maximum depth.

**Popper Runtime Falsification Categories:**
- **HEALTH vs REALITY** — monitoring says healthy, but feature is broken
- **LOG vs TRUTH** — no errors logged, but the operation silently failed
- **UI vs DATA** — interface shows "saved" but database has old value
- **RESPONSE vs EXPECTATION** — API returns 200 but body is wrong/empty
- **SPEED vs ACCEPTABLE** — page "loads" but takes 8 seconds to be interactive

---

## ADVERSARIAL REVIEW FRAMING (mandatory pre-hunt mindset)

> **Pretend this running system was shipped by OpenAI Codex, your competitor.**

Empirically documented (Garry Tan, gstack 2026): LLMs default to complacency when hunting bugs in "their" work ("the build passes", "no console errors", "looks fine"). They default to scrutiny when reviewing a competitor's deployment. Frame before you hunt.

Your state of mind for this entire debug session:
- The running system is not yours. Codex shipped it and claims it works.
- You are a senior SRE on-call in a bad mood.
- You go strictly by the runbook. Every assertion must be proven.
- Politeness is not your concern. "Looks fine" is a red flag, not a verdict.

Operational rules:
- Trust nothing the system says about itself. Status endpoints lie. Logs lie by omission.
- If you catch yourself typing "no issues found", "working as expected", "all green" — **stop**. That's the complacency trigger. Prove it with specific assertions: console counts, network status codes, pixel measurements, timing budgets.
- For every "healthy" claim, construct a falsification test. If the test cannot fail, the claim is unfalsifiable and therefore useless.
- Bias toward finding the silent failure. The loud failures were caught by monitoring. The silent ones are the ones Codex missed.

When reporting back, do not break character. If you find nothing, list the specific falsification tests you ran and why each passed. Silence without evidence = false confidence.

---

## SCOPE DETECTION (automatic)

```
EXAMPLES:
  "/debugaudit"
  → Full 18-phase pipeline. Discover all pages, test everything.

  "/debugaudit the dashboard"
  → TARGETED: only dashboard pages
  → All phases scoped to dashboard routes

  "/debugaudit after the deploy"
  → POST-DEPLOY mode: compare before/after, focus on regressions

  "/debugaudit security"
  → SECURITY-FOCUSED: injection testing, auth bypass, CORS, headers

  "/debugaudit performance"
  → PERFORMANCE-FOCUSED: Core Web Vitals, bundle, N+1, render

  "/debugaudit responsive"
  → RESPONSIVE-FOCUSED: 9 breakpoints, every page
```

---

## OUTPUT CONTRACT

```
audits/.debugaudit/
├── session.log
├── discovery/
│   ├── pages.json           # All discovered routes/pages
│   ├── api-endpoints.json   # All API routes
│   ├── components.json      # All interactive elements
│   └── features.json        # Feature inventory from code
├── reports/
│   ├── console-errors.md        # Phase 1
│   ├── network-failures.md      # Phase 2
│   ├── visual-regressions.md    # Phase 3
│   ├── responsive-breakage.md   # Phase 4
│   ├── dead-features.md         # Phase 5
│   ├── state-corruption.md      # Phase 6
│   ├── race-conditions.md       # Phase 7
│   ├── security-injection.md    # Phase 8
│   ├── auth-bypass.md           # Phase 9
│   ├── performance.md           # Phase 10
│   ├── api-contract-drift.md    # Phase 11
│   ├── data-integrity.md        # Phase 12
│   ├── error-handling.md        # Phase 13
│   ├── dependency-health.md     # Phase 14
│   ├── environment-drift.md     # Phase 15
│   ├── log-analysis.md          # Phase 16
│   ├── cross-browser.md         # Phase 17
│   └── chaos-testing.md         # Phase 18
├── verdict.json
├── verdict.md
├── fix-plan.json
├── fix-plan.md
├── progress.json
└── fix-log.md
```

---

## PHASE 0 — PROGRAMMATIC GATHER (HYBRID, runs FIRST, before all other phases)

> **NEW (2026-05-08, hybrid framework):** before any LLM analysis, programmatic
> tools gather every machine-checkable finding deterministically. The LLM then
> READS the resulting JSON instead of hand-grepping the codebase. Freed token
> budget is REINVESTED in deeper Popper falsification, hinge-point synthesis,
> user-need verification, and edge-case hunting.

### 0.1 Run the gather script (mandatory, FIRST step)

```bash
~/.omega/lib/audit-runner.sh debug "$PROJECT_PATH" \
  --files="$FILES_MODIFIED" \
  --url="$URL" \
  --user-need="$USER_NEED_QUOTE" \
  --hinge="$HINGE_POINT" \
  --ticket="$TICKET_ID"
```

This invokes `~/.omega/lib/audit-gather/debug.sh` which runs:
Playwright runtime probe (console events, page errors, network failures, 4xx/5xx responses, screenshot, page metadata) and curl HEAD fallback — REQUIRES URL or runtime probe is impossible

Output is written to:

```
$PROJECT_PATH/audits/.debugaudit/
├── raw/                    # raw tool outputs (JSON / text per tool)
└── evidence-summary.json   # normalized findings, single source of truth for the LLM
```

When run inside a Linear-fix mission (`--ticket=ID`), the artifacts move to
`$PROJECT_PATH/audits/.linear-fix/<ID>/.debugaudit/` so multiple audits on the same
ticket can cross-reference each other (see 0.5).

### 0.2 evidence-summary.json schema

```jsonc
{
  "audit": "debug",
  "tools_run": ["..."],
  "tools_skipped": [{"tool": "...", "reason": "..."}],
  "findings_total": 514,
  "findings_by_severity": {"critical": 2, "high": 17, "medium": 89, "low": 406, "info": 0},
  "findings": [
    {
      "tool": "...",
      "severity": "critical|high|medium|low|info",
      "location": "file:line[:col]",
      "rule": "...",
      "message": "...",
      "suggested_fix": "...",
      "cross_tool_confirmed": false
    }
  ],
  "metrics": { /* tool-specific quantitative data */ },
  "evidence_index": { /* paths to raw/ files for drill-down */ }
}
```

### 0.3 What you do AFTER the gather (this replaces hand-greps)

You now consume `evidence-summary.json` programmatically. You MUST:

1. **Read `evidence-summary.json` in full.** This is your evidence base.
2. **Read 3-5 critical files only** — the ones flagged as load-bearing in
   `~/.omega/state/hinge-points-<ticket>.json` (or computed via
   `${OMEGA_DIR:-$HOME/.omega}/skills/audits/_shared/hinge-analyzer.sh` if no ticket).
3. **DO NOT manually grep the codebase for what the gather already covered.**
   The tools have already exhaustively scanned every file. Re-running grep
   wastes tokens and produces the same evidence.
4. **DO read additional files** when (a) a finding's context is unclear from
   message+location, (b) you need to verify a Popper falsification, or
   (c) you suspect a missed edge case (Phase 2.4 below).

### 0.4 Banned operations after Phase 0

These are now forbidden because the gather already did them. If you catch
yourself about to run one, STOP and read `evidence-summary.json` first:

- ❌ `grep -rn "TODO" .` (the gather scanned for it)
- ❌ `find . -name "*.ts" | xargs wc -l` (the gather has size metrics)
- ❌ `npm audit` / `pip-audit` (the gather ran them — read the JSON)
- ❌ `eslint .` / `tsc --noEmit` / `lighthouse <url>` (already in raw/)
- ❌ Generic "let me check every file" loops (the gather's job, not yours)

You MAY still:
- ✅ Read SPECIFIC files cited in findings (verify the issue)
- ✅ Run a SPECIFIC `grep` to falsify a finding (Popper test, see Phase 2.1)
- ✅ Run a SPECIFIC tool the gather couldn't (e.g. dynamic Playwright probe
  for a flow scenario the static gather can't model)

### 0.5 Cross-audit synthesis (read sibling evidence-summary.json files)

If this audit runs as part of a Linear-fix mission, sibling audits' summaries
are at `$PROJECT_PATH/audits/.linear-fix/<TICKET>/.<other-audit-id>/evidence-summary.json`.
Read them. Use them.

Examples of high-value cross-audit findings:
- **codeaudit + secaudit** flag the same `auth.ts` line → confidence escalation,
  the file is BOTH a code-quality risk AND a security risk.
- **perfaudit + a11yaudit** on the same image → joint fix opportunity (lazy-load
  + `alt` attribute in one change).
- **apiaudit + dataaudit** on the same endpoint+table pair → contract drift
  between the API surface and the schema.
- **debugaudit + flowaudit** report the same broken page → user-flow blocker.

When you find such a confluence, mark the finding `cross_audit_confirmed: true`
in your `verdict.json` and bump severity by one level.

---

## PHASE 0: RECONNAISSANCE

> *"Know your patient before you cut them open."*

```
1. PROJECT DISCOVERY
   → Read CLAUDE.md, README, package.json/pyproject.toml
   → Identify: stack, framework, entry points, deploy method
   → Find: dev URL, prod URL, ports, environment

2. PAGE/ROUTE DISCOVERY
   → Scan all routes (Next.js app/, pages/, Python routes, etc.)
   → Build complete sitemap with auth requirements per route
   → Identify dynamic routes, API endpoints, webhooks

3. FEATURE INVENTORY
   → List every user-facing feature from code (not docs)
   → Mark the HINGE FEATURE (most critical)
   → Note features that SHOULD exist but don't (from VISION/PRD)

4. HEALTH BASELINE
   → Current process status (running? memory? CPU?)
   → Current error rate from logs
   → Current response times
   → This becomes the "before" for comparison
```

---

## PHASE 1: CONSOLE ERROR SWEEP

> *"The console is the patient's medical chart. Most doctors don't read it."*

```
FOR EVERY discoverable page:

1. Navigate via Playwright CLI
2. Capture ALL console output: errors, warnings, info
3. Capture unhandled promise rejections
4. Capture React/framework errors (hydration, key warnings, etc.)
5. Note: errors that appear on load vs after interaction

CLASSIFY:
- CRITICAL: Unhandled exception, white screen, React error boundary
- HIGH: Failed API call, 404/500 errors, CORS failures
- MEDIUM: Deprecation warnings, React key warnings, slow script warnings
- LOW: Third-party script errors, analytics failures

FALSE POSITIVE FILTER:
- Browser extension errors (ignore)
- Third-party ad/tracking errors (note but don't fail)
- Development-only warnings in prod (flag separately)
```

---

## PHASE 2: NETWORK FAILURE ANALYSIS

> *"Every failed fetch is a broken promise to the user."*

```
FOR EVERY page navigation and interaction:

1. CAPTURE all network requests via Playwright
2. Flag: 4xx responses (client errors)
3. Flag: 5xx responses (server errors)
4. Flag: Requests > 3 seconds (slow)
5. Flag: Requests that never complete (hung)
6. Flag: Duplicate requests (same URL called 2+ times)
7. Flag: Requests to non-existent endpoints (404s)
8. Flag: CORS errors
9. Flag: Mixed content (HTTP on HTTPS page)

PATTERN DETECTION:
- N+1 queries: same endpoint called N times with different IDs
- Waterfall: sequential requests that could be parallel
- Redundant: same data fetched multiple times
- Missing: expected API calls that don't happen (feature broken)
```

---

## PHASE 3: VISUAL REGRESSION DETECTION

> *"If the user sees it, it's a bug. If they don't see it but it's wrong, it's still a bug."*

```
FOR EVERY page:

1. SCREENSHOT at 1440px width (desktop reference)
2. ANALYZE screenshot for:
   - Layout overflow (horizontal scroll)
   - Overlapping elements
   - Cut-off text / ellipsis where it shouldn't be
   - Missing images (broken img, alt text showing)
   - Flash of unstyled content (FOUC)
   - Loading states stuck permanently
   - Empty areas that should have content
   - Misaligned elements (off-grid)
   - Wrong font rendering (system font instead of custom)

3. COMPARE with design intent if DESIGN.md/VISION.md exists
4. If multiple similar pages (list views, detail views), compare consistency
```

---

## PHASE 4: RESPONSIVE BREAKAGE SWEEP

> *"A bug at 375px is a bug for 50% of your users."*

```
TEST at 9 breakpoints: 320, 375, 425, 768, 1024, 1280, 1440, 1920, 2560

FOR EACH breakpoint, FOR EACH page:
1. Screenshot
2. Check: horizontal overflow (deadly sin)
3. Check: touch targets < 44px (mobile)
4. Check: text readability (too small, too wide)
5. Check: navigation accessible (hamburger works)
6. Check: modals/drawers fit in viewport
7. Check: tables scroll or reformat
8. Check: images scale correctly
9. Check: no content hidden or cut off
```

---

## PHASE 5: DEAD FEATURE DETECTION

> *"Code that exists but does nothing is worse than code that doesn't exist."*

```
1. BUTTON CLICK AUDIT
   FOR EVERY button/link on every page:
   → Click it. Does something happen?
   → Does the expected thing happen? (label matches action)
   → Is there feedback? (loading, success, error)
   → Are there buttons that do NOTHING? (onclick missing, handler empty)

2. FORM SUBMISSION AUDIT
   FOR EVERY form:
   → Submit with valid data → success?
   → Submit with empty required fields → error messages?
   → Submit button disabled during processing?
   → Form resets after success?

3. NAVIGATION AUDIT
   → Every nav link leads somewhere (no 404s)
   → Breadcrumbs match actual location
   → Active state matches current page
   → Back button works as expected

4. FEATURE FLAG GRAVEYARD
   → Features behind flags permanently set to false
   → UI elements rendered but hidden via CSS (display:none with content)
   → Routes that exist but render blank/error
```

---

## PHASE 6: STATE CORRUPTION DETECTION

> *"The database says one thing. The UI says another. Somebody is lying."*

```
1. STALE DATA
   → Create/update data → refresh page → data matches?
   → Two tabs open → update in one → reflected in other?
   → Data from cache vs data from API → match?

2. OPTIMISTIC UPDATE DRIFT
   → Perform action → UI updates optimistically
   → Was the action actually persisted? (check API/DB)
   → If action fails → does UI revert?

3. SESSION STATE
   → Login → close tab → reopen → still logged in?
   → Login → wait 1 hour → session still valid?
   → Multiple tabs → session consistent?

4. FORM STATE PRESERVATION
   → Fill form partially → navigate away → come back → data preserved?
   → Fill form → validation error → fix error → original data intact?
   → Fill form → browser crash → data recoverable?
```

---

## PHASE 7: RACE CONDITION PROBING

> *"Click twice. Click fast. Click during loading. What breaks?"*

```
1. DOUBLE CLICK/SUBMIT
   → Click submit button twice rapidly → duplicate creation?
   → Click delete twice → error on second? crash?
   → Click navigation twice → weird state?

2. RAPID NAVIGATION
   → Click link, immediately click another → data leak from first?
   → Start loading page A, switch to page B → stale data?

3. CONCURRENT MUTATIONS
   → Open same resource in two tabs
   → Edit in both → save both → which wins? conflict?
   → Delete in one → view in other → crash? graceful?

4. ASYNC OPERATION INTERRUPTS
   → Start upload → navigate away → orphan file?
   → Start long operation → refresh → state consistent?
   → Start operation → lose network → resume? corrupt?
```

---

## PHASE 8: SECURITY INJECTION TESTING

> *"Every input field is a door. Most are unlocked."*

```
1. XSS PROBING (25+ payloads)
   For every input field, URL param, and query string:
   → <script>alert(1)</script>
   → <img src=x onerror=alert(1)>
   → javascript:alert(1)
   → " onmouseover="alert(1)
   → {{7*7}} (template injection)
   → ${7*7} (template literals)
   → And 19 more payload variants
   
   CHECK: is the payload reflected in HTML unescaped?
   CHECK: is the payload stored and displayed to other users?
   CHECK: DOM-based XSS via URL hash/fragment?

2. SQL/NoSQL INJECTION
   → ' OR 1=1 -- in search/filter fields
   → {$gt: ""} in JSON bodies
   → UNION SELECT in sort parameters

3. IDOR (Insecure Direct Object Reference)
   → Change resource IDs in URLs → access other users' data?
   → Enumerate IDs → predictable sequence?
   → Access deleted resources by ID?

4. CSRF
   → Are state-changing requests protected with tokens?
   → Can actions be triggered from external sites?

5. OPEN REDIRECTS
   → Redirect params accept arbitrary URLs?
   → Login redirect → external site?
```

---

## PHASE 9: AUTH BOUNDARY TESTING

> *"Auth isn't a feature. It's a wall. Find the holes."*

```
1. ROUTE PROTECTION
   FOR EVERY protected route:
   → Access without auth → redirect to login?
   → Access with expired session → handled gracefully?
   → Access with wrong role → 403 with message?
   → Access API directly (curl) without token → 401?

2. TOKEN MANIPULATION
   → Modify JWT payload → rejected?
   → Use expired token → rejected?
   → Use token from another environment → rejected?

3. PRIVILEGE ESCALATION
   → Normal user → admin endpoint via direct URL → blocked?
   → Normal user → admin action via API → blocked?
   → Modify role field in profile update → ignored?

4. SESSION FIXATION
   → Pre-login session ID → same after login? (should rotate)
   → Logout → session actually destroyed? (not just UI cleared)
```

---

## PHASE 10: PERFORMANCE AUTOPSY

> *"A page that takes 5 seconds to load is a page that doesn't exist."*

```
1. CORE WEB VITALS (per page)
   → LCP (Largest Contentful Paint) — target < 2.5s
   → FID/INP (Interaction to Next Paint) — target < 200ms
   → CLS (Cumulative Layout Shift) — target < 0.1

2. BUNDLE ANALYSIS
   → Total JS size (compressed + uncompressed)
   → Largest chunks — what's in them?
   → Tree shaking effective? Dead code in bundle?
   → Source maps exposing internal code?

3. RENDER PERFORMANCE
   → Time to interactive per page
   → Unnecessary re-renders (React Profiler patterns)
   → Layout thrashing (forced synchronous layouts)
   → Long tasks blocking main thread (>50ms)

4. API PERFORMANCE
   → Slowest endpoints (>1s)
   → N+1 query patterns
   → Missing pagination (loading 10K records)
   → Missing caching (same data re-fetched)

5. IMAGE AUDIT
   → Unoptimized images (>500KB)
   → Missing lazy loading
   → Missing responsive srcsets
   → Wrong format (PNG where WebP would save 80%)
```

---

## PHASE 11: API CONTRACT DRIFT

> *"The frontend expects X. The backend sends Y. Nobody noticed."*

```
1. TYPE MISMATCH
   → Frontend TypeScript types vs actual API response shape
   → Fields that exist in type but never in response
   → Fields in response not in type (untyped data)
   → Null vs undefined vs missing field inconsistency

2. ERROR FORMAT INCONSISTENCY
   → Do all endpoints return errors the same way?
   → {error: string} vs {message: string} vs {errors: array}
   → HTTP status codes: correct? (200 on error = lie)

3. PAGINATION DRIFT
   → Cursor vs offset inconsistency across endpoints
   → Missing total count
   → Last page behavior (empty array vs null vs error)

4. WEBHOOK VERIFICATION
   → Signatures validated?
   → Idempotency respected?
   → Response within provider timeout?
```

---

## PHASE 12: DATA INTEGRITY TESTING

> *"Enter 'hello'. Get back 'hell'. Nobody checked."*

```
1. ROUNDTRIP VERIFICATION
   → Create data → read it back → identical?
   → Special characters: &<>"'\n emoji → preserved?
   → Unicode: CJK, RTL, combining characters → correct?
   → Numbers: 0, -1, 0.1, MAX_SAFE_INTEGER → no precision loss?

2. RELATIONSHIP INTEGRITY
   → Delete parent → children orphaned? cascade? blocked?
   → Reference deleted entity → graceful? or crash?
   → Bulk operations → all-or-nothing? or partial?

3. SEARCH CONSISTENCY
   → Create item → immediately searchable?
   → Update item → search returns new data?
   → Delete item → removed from search?
```

---

## PHASE 13: ERROR HANDLING VERIFICATION

> *"The try/catch is there. But what happens IN the catch?"*

```
1. FORCED ERROR TESTING
   → Kill API mid-request → user sees what?
   → Return malformed JSON → crash or graceful?
   → Return 500 from every endpoint → app survives?
   → Exceed rate limits → handled or crash?

2. ERROR BOUNDARY AUDIT
   → Throw in each component → caught by boundary?
   → Boundary shows useful message? Or white screen?
   → Can user recover from boundary? (retry, refresh, navigate)

3. OFFLINE BEHAVIOR
   → Disconnect network → app behavior?
   → Reconnect → auto-recover? manual refresh?
   → Queued actions → replayed? lost?
```

---

## PHASE 14: DEPENDENCY HEALTH CHECK

> *"Your app is only as alive as its weakest dependency."*

```
1. EXTERNAL SERVICE HEALTH
   FOR EACH external dependency (DB, cache, CDN, auth, payments):
   → Is it reachable right now?
   → What's the latency?
   → What happens if it goes down? (circuit breaker? crash?)

2. PACKAGE VULNERABILITIES
   → npm audit / pip audit
   → Critical CVEs in dependencies?
   → Known malicious packages?

3. VERSION DRIFT
   → Lockfile matches installed?
   → Major version behind on critical deps?
   → Deprecated packages still in use?
```

---

## PHASE 15: ENVIRONMENT DRIFT

> *"Works on my machine. Crashes in production."*

```
1. ENV VAR VERIFICATION
   → All required env vars present?
   → No placeholder values ("TODO", "change-me", "xxx")?
   → Dev vs prod values appropriate?

2. BUILD vs RUNTIME
   → Build output matches what's deployed?
   → Source maps disabled in prod?
   → Debug mode off in prod?
   → Test data/fixtures absent in prod?

3. DATABASE SCHEMA
   → Migration status up to date?
   → No pending migrations?
   → Schema matches code's expectations?
```

---

## PHASE 16: LOG FORENSICS

> *"Read the last 1000 lines. The bugs confess."*

```
1. ERROR LOG ANALYSIS
   → Last 1000 lines of application logs
   → Unique errors vs repeated (same error 100x = systemic)
   → Error frequency: stable, growing, spiking?
   → Silent periods (gap in logs = dead process?)

2. PATTERN DETECTION
   → Retry storms (same operation retried endlessly)
   → Memory warnings (OOM approaching)
   → Connection pool exhaustion
   → Rate limit hits

3. CORRELATION
   → Do log errors match user-visible bugs?
   → Are there log errors with NO user impact? (over-logging)
   → Are there user bugs with NO log trace? (under-logging)

4. LOG LEVEL VERIFICATION (MANDATORY)
   → Is the logger at INFO or higher? If WARNING, you're blind to 80% of issues
   → grep "setLevel\|basicConfig" in config/logging setup
   → If no explicit level: default is WARNING → blind spot
   → FIX: force INFO or DEBUG during audit
```

---

## PHASE 16b: LIVE RUNTIME OBSERVATION (CRITICAL — NEW 2026-04-14)

> *"Code lies. Comments lie. Runtime doesn't."*
>
> Added after wasting 2h35 on a `tmux paste-buffer -p` flag bug that
> would have been found in 5 minutes with live observation.

**Trigger this phase when:**
- A bug is intermittent ("works 1/10 times")
- 2+ code fixes haven't resolved it
- The symptom contradicts what the code says it should do
- Interaction between systems (subprocess, paste, network, file I/O, terminal)

**The Protocol — execute in this exact order:**

```
STEP 1: ENABLE VERBOSE LOGGING (2 min)
  → Find the logger config. Is level INFO+? If not, raise it.
  → Add logger.info() at every system boundary in the broken flow:
    * BEFORE any subprocess/IPC call: log inputs
    * AFTER any subprocess/IPC call: log outputs + return code
    * BEFORE any paste/send/write: log exact content (length, preview, hash)
    * AFTER any paste/send/write: log observable result
  → Deploy the debug version.

STEP 2: BOUNDARY OBSERVATION (5 min)
  For the failing flow, identify every "boundary" where data crosses systems:
    * Python → tmux buffer
    * tmux buffer → tmux pane
    * Pane → application reading stdin
    * App → network → API
    * API → response → app
    * App → file → disk
  Log at EACH boundary: what entered, what exited, the delta.

STEP 3: LIVE TAIL DURING REPRO (5 min)
  → tail -f <logfile> in terminal
  → User performs the action (or automate it)
  → Watch logs in real-time
  → The root cause usually jumps out visually

STEP 4: ISOLATE WITH MINIMAL TEST (5 min)
  Once you have a hypothesis, prove it in isolation:
    * Strip away unrelated context
    * Run JUST the failing step manually
    * Try the alternative hypothesis
    * Compare outputs side by side

STEP 5: FIX + VERIFY (2 min)
  With proven root cause, the fix is obvious and trivial.
  Verify by re-running the live observation.

Total time: 20 minutes max. Not 2 hours.
```

**Anti-patterns that turn 20min into 2h35:**

| Anti-pattern | Why it fails | Correct action |
|---|---|---|
| "Let me add polling instead of sleep" | Guessing, no evidence | First: log what's happening |
| "The comment says X is required" | Comments lie | Test the alternative |
| "Maybe the cooldown is too long" | Fixing a different problem | Observe the actual failure |
| "Let me refactor the error handling" | Can't fix what you don't understand | Add logs first |
| 3 code commits on same bug without observing | Tight loop on wrong hypothesis | STOP. Add logging. Watch runtime. |
| "Probably a race condition" | Hand-wave without proof | Log timestamps at each step |

**The Golden Rule (mandatory):**

> Before the 3rd code change on the same bug, you MUST have live logs showing
> the actual runtime behavior. If you don't: STOP coding, add logging, reproduce,
> observe, THEN fix. No exceptions.

**Detection heuristics (flag these as high-risk for live debug):**

- Flaky intermittent bugs ("works sometimes")
- Subprocess/IPC/terminal interactions (tmux, bash, piped commands)
- Bracketed paste, escape sequences, terminal modes
- Authentication flows (token passing, OAuth, code exchange)
- Race conditions, timing-sensitive code
- File I/O with mtime checks
- Any bug where a comment says "X is required" but X doesn't obviously work

---

## PHASE 17: CROSS-BROWSER VERIFICATION

> *"Chrome works. Safari doesn't. 20% of your users just left."*

```
Test critical flows in:
1. Chrome (baseline)
2. Firefox (layout engine differences)
3. Safari/WebKit (the most different — CSS, JS, API support)
4. Mobile Chrome (touch, viewport)
5. Mobile Safari (iOS quirks)

FOR EACH browser:
→ Does the app load?
→ Do interactions work?
→ Are there console errors unique to this browser?
→ Are there visual differences?
→ Do CSS features render correctly? (backdrop-filter, grid, etc.)
```

---

## PHASE 18: CHAOS TESTING

> *"If it can break, it will. Let's break it now, on our terms."*

```
1. RESOURCE EXHAUSTION
   → Upload maximum file size → handled?
   → Send 10,000-character input → handled?
   → Create 1000 items → pagination works? performance ok?
   → Open 50 WebSocket connections → memory?

2. TIMING ATTACKS
   → Slow network (throttle to 3G) → usable?
   → API latency 5s → timeout handling?
   → Rapid actions during slow response → queue or crash?

3. PARTIAL FAILURES
   → Kill one microservice → graceful degradation?
   → Database read-only → writes fail gracefully?
   → CDN down → fallback?

4. DATA EDGE CASES
   → Empty strings everywhere → crash?
   → NULL in every nullable field → renders?
   → Dates: epoch, far future, timezone edge → correct?
   → Currencies: 0.00, 0.001, MAX → display correct?
```

---

## PHASE H1 — HYBRID SYNTHESIS (Popper / hinge / user-need / edge cases / cross-audit)

> **NEW (2026-05-08, hybrid framework, runs immediately before VERDICT):**
> "H1" = Hybrid step 1 of the synthesis layer that pairs with Phase 0's
> programmatic gather. It does NOT renumber existing phases; it sits between
> the audit's last domain phase and the VERDICT phase. Between the per-domain
> phases above and the VERDICT phase, you must run this 5-step synthesis.
> The token budget freed by Phase 0's deterministic gather is REINVESTED here
> — depth of analysis is what increases. This phase does NOT replace any
> earlier phase; it deepens them.

### 2.1 Popper falsification per finding (mandatory)

For every finding in `evidence-summary.json.findings[]` (start with `severity ∈
{critical, high}`, then go down as time/budget allows), try to PROVE the tool
is wrong. Each falsification produces a `falsifiable_tests[]` entry in
`verdict.json`:

```jsonc
{
  "claim": "ts-prune says src/auth/legacy.ts:42 export `signWithOldKey` is unused",
  "test_command": "grep -rn 'signWithOldKey' --include='*.ts' --include='*.tsx' --include='*.test.ts' --include='*.spec.ts' . | grep -v 'src/auth/legacy.ts'",
  "expected": "0 matches → claim TRUE, finding stands",
  "actual": "0 matches found",
  "outcome": "confirmed"
}
```

Outcomes:
- `confirmed` — Popper test FAILED to falsify → finding stands, often promoted
- `falsified` — Popper test produced a counter-example → demote to `info` and
  add `falsified_at: <evidence>` to the finding
- `inconclusive` — test could not run cleanly → keep severity, mark
  `confidence: medium` on this finding

**The rule:** every CLAIM in the audit (PASS or FAIL) MUST cite ≥3 concrete
commands that COULD have falsified it but didn't. Banned phrases (`looks
correct`, `should be fine`, `appears to work`) → automatic FAIL of the audit.

Common falsification patterns by category:

| Tool says | Popper test |
|---|---|
| `unused export` (ts-prune, vulture) | `grep` for the symbol in tests/, integration/, and dynamic imports (e.g. `import("...")`) |
| `unused dependency` (depcheck) | `grep` `package.json` scripts + `find . -type f -exec grep -l <pkg>` |
| `circular dep` (madge) | Read all files in the cycle — is the cycle real or a tooling artifact (e.g. type-only re-export)? |
| `console.error` / `console.warn` (debugaudit) | Reproduce the user flow that triggers it; if not reachable from any UI path, demote to info |
| `npm audit HIGH` (sec) | Check if the vulnerable code path is actually called in your codebase; not all transitive vulns are exploitable |
| `LCP > 2.5s` (perf) | Re-run lighthouse twice; check if it's a one-off (cold cache, network blip) or systematic |
| `axe-core color-contrast` (a11y) | Manually compute the ratio; some palettes hit 4.4 vs 4.5 — fixable in tokens |
| `missing canonical` (seo) | Check if the page is intentionally not canonical (paginated, faceted) before flagging |
| `unauthenticated endpoint` (api) | Read the route handler — is auth enforced via middleware not visible to the static scan? |
| `orphaned record` (data) | Verify the FK is supposed to cascade, or confirm the parent was deleted by an actual user action |

### 2.2 Hinge cross-reference (10× scrutiny on load-bearing findings)

The HINGE FEATURE of this audit is the locus of maximum risk/value. Compute or
read:

```bash
# If a hinge file already exists for this ticket, use it:
HINGE_FILE="$HOME/.omega/state/hinge-points-${TICKET_ID:-default}.json"

# Otherwise compute it on the fly:
${OMEGA_DIR:-$HOME/.omega}/skills/audits/_shared/hinge-analyzer.sh "$PROJECT_PATH" --audit=debug --user-need="$USER_NEED_QUOTE" \
  > "$HINGE_FILE"
```

For each finding in `evidence-summary.json`, mark
`is_load_bearing: true` IFF its file matches a hinge entry, then apply 10×
scrutiny to those findings:

- 5× more falsification attempts (Phase 2.1)
- 3× more edge-case hunts (Phase 2.4)
- Mandatory read of the entire hinge file (not just the flagged line)
- Mandatory read of all DIRECT callers of the hinge function/symbol
- Mandatory read of all DIRECT callees from the hinge

Output `hinge_findings[]` in `verdict.json`:

```jsonc
{
  "finding_id": "F-042",
  "is_load_bearing": true,
  "hinge_reference": "<hinge term + file>",
  "additional_scrutiny": "verified all 7 callers, all 4 callees, all 12 tests; reproduced bug in 3 of them",
  "confidence_after_scrutiny": "high"
}
```

### 2.3 User-need verification (`--user-need` quote)

If the audit was dispatched with `--user-need="<verbatim user complaint>"`,
every finding MUST be evaluated against it. The user-need is the ground truth
for what counts as "audit succeeded".

For each finding, ask:
- "If a user reported THIS exact issue verbatim, would this finding be the
  cause?"
- "Does fixing this finding make the user-need quote no longer true?"

Findings that DO NOT relate to user-need:
- Get demoted by one severity level UNLESS they are load-bearing
  (Phase 2.2), in which case they retain severity (load-bearing is
  user-need-independent).
- Are still reported, but flagged `user_need_relevance: "tangential"`.

Findings that DO relate to user-need:
- Get the highest priority in the fix plan
- Are listed first in `user_need_match.findings[]` of `verdict.json`

```jsonc
{
  "user_need_match": {
    "addressed": true,
    "user_need_quote": "<verbatim>",
    "rationale": "Finding F-007 (axe-core: input has no associated label) directly causes the user-reported behavior 'I can't tab to the search field'. Fix removes the cause.",
    "findings": ["F-007", "F-019"],
    "untouched_findings_relevant_to_user_need": []
  }
}
```

If `addressed: false`, the audit MUST score below 90/100 even if all phases
otherwise pass. The user's actual problem is the only correct success metric.

### 2.4 Edge case hunting (mandatory for top findings)

For each top-5 finding (sorted by severity × cross-audit-confirmed × hinge),
generate ≥2 edge cases the tool may have missed. The static analyzer or
runtime probe checked the code at rest; you must imagine motion.

Patterns:
- "If user does X under condition Y..." — concurrency, race, double-submit
- "Tool checked the file at rest, but at runtime + concurrent..." — async/await
  ordering, state mutations, cache races
- "Static scan saw the import, but the dynamic require()..." — code-splitting,
  feature-flag gated, lazy()
- "i18n locale switch..." — strings missing in non-default locale
- "Network flake mid-request..." — partial state, retry idempotency
- "User logs out mid-flow..." — session expiration, token refresh
- "Timezone boundary (DST, UTC midnight)..." — date math
- "Data shape: empty array, null, undefined, single-element..." — boundary

Output `edge_cases[]` in `verdict.json`:

```jsonc
{
  "finding_id": "F-007",
  "scenario": "User pastes a multi-line value into the search field; the
    `<input>` strips newlines silently and submits a partial query",
  "covered_by_existing_test": false,
  "evidence_gathered": "Manual repro on prod URL; screenshot at .debug/edge-evidence/F-007.png",
  "fix_includes_coverage": true
}
```

### 2.5 Cross-audit synthesis (re-read sibling summaries from Phase 0.5)

Now that you have your own `verdict.json` draft, do a final pass with sibling
audits' findings open in context:

1. For each of YOUR top-5 findings: is the same file/line/symbol flagged in any
   sibling audit?
2. If yes → escalate confidence (`cross_audit_confirmed: true`), bump severity
   one level (low→medium, medium→high, high→critical, critical stays).
3. For each sibling top-5 finding: is the same file/line/symbol relevant to
   YOUR audit's domain? If yes, add it to YOUR findings as
   `tool: "cross-audit:<sibling-name>"` with proper severity.
4. Write `cross_audit_links[]` in `verdict.json` summarizing the matches.

```jsonc
{
  "cross_audit_links": [
    {
      "this_finding_id": "F-007",
      "sibling_audit": "secaudit",
      "sibling_finding_id": "F-013",
      "shared_location": "src/auth/login.tsx:42",
      "joint_fix_recommended": true
    }
  ]
}
```

### 2.6 Final verdict.json schema (hybrid v2)

```jsonc
{
  "audit": "debug",
  "score": 100,
  "score_raw": "<raw>/<denominator>",
  "score_normalized": 100,
  "confidence": "high|medium|low",
  "skill_used": "debug",
  "user_need_match": { ... },                   // §2.3
  "falsifiable_tests": [ ... ],                   // §2.1
  "hinge_findings": [ ... ],                      // §2.2
  "issues_found_and_fixed": [
    {
      "id": "FIX-001",
      "finding_id": "F-007",  // back-ref to evidence-summary.json
      "before": "<state>",
      "after": "<state>",
      "verification": "<command + output>"
    }
  ],
  "edge_cases": [ ... ],                          // §2.4
  "cross_audit_links": [ ... ],                   // §2.5
  "evidence_summary_path": "$PROJECT_PATH/audits/.debugaudit/evidence-summary.json",
  "confidence_basis": "Why I'm confident (or not). Cite Popper test counts, hinge scrutiny depth, edge-case coverage, cross-audit confirmations.",
  "banned_phrase_check": "passed (no occurrences of `looks correct`, `should be fine`, `appears to work`, `streamlined`, `to save time`)"
}
```

### 2.7 Score gating (hybrid threshold)

A 100/100 score is now blocked unless:

1. ✅ All `critical` and `high` findings are fixed OR have an explicit
   `non_issue_justification` of ≥50 words backed by Popper evidence.
2. ✅ All load-bearing findings (Phase 2.2) confirmed via Popper falsification.
3. ✅ `user_need_match.addressed = true` with verbatim quote of the user-need.
4. ✅ ≥3 falsifiable tests cited per phase (Phase 2.1).
5. ✅ ≥2 edge cases generated per top-5 finding (Phase 2.4).
6. ✅ Cross-audit synthesis attempted (Phase 2.5) — links may be empty if no
      siblings exist, but the array MUST be present.
7. ✅ `confidence_basis` populated with non-trivial reasoning.

Below threshold → score < 100, fix-and-reaudit loop kicks in (existing R-6 flow
in the FIX EXECUTION / RE-AUDIT phases below). The loop is BOUNDED at 5
iterations per the Audit Verification Contract; on iteration 5 if still failing,
emit `confidence: low` and surface as `pending` in `.done.json`.

---

## PHASE 19: VERDICT

Score each phase 0-10, weight by severity:

```
SCORING MATRIX (360 max):
  Phase  1  (Console Errors)        × 3.0  = max 30
  Phase  2  (Network Failures)      × 3.0  = max 30
  Phase  3  (Visual Regressions)    × 2.0  = max 20
  Phase  4  (Responsive)            × 2.0  = max 20
  Phase  5  (Dead Features)         × 2.5  = max 25
  Phase  6  (State Corruption)      × 3.0  = max 30
  Phase  7  (Race Conditions)       × 2.5  = max 25
  Phase  8  (Security Injection)    × 3.0  = max 30
  Phase  9  (Auth Bypass)           × 3.0  = max 30
  Phase 10  (Performance)           × 2.0  = max 20
  Phase 11  (API Contract Drift)    × 2.0  = max 20
  Phase 12  (Data Integrity)        × 2.5  = max 25
  Phase 13  (Error Handling)        × 2.0  = max 20
  Phase 14  (Dependency Health)     × 1.5  = max 15
  Phase 15  (Environment Drift)     × 1.5  = max 15
  Phase 16  (Log Forensics)         × 1.5  = max 15
  Phase 17  (Cross-Browser)         × 1.5  = max 15
  Phase 18  (Chaos Testing)         × 1.5  = max 15
                                    TOTAL  = max 360

NORMALIZE: score = (raw / 360) × 100

GRADE:
  90-100: S — Fortress. Nothing broken, battle-hardened.
  80-89:  A — Solid. Minor issues, nothing user-facing.
  70-79:  B — Good. Some bugs, most paths work.
  60-69:  C — Acceptable. Visible bugs, workarounds needed.
  50-59:  D — Risky. Major features broken.
  <50:    F — Condemned. Users are suffering.
```

---

## PHASE 20: FIX PLAN (automatic)

```
Sort: CRITICAL → HIGH → MEDIUM → LOW
Group by root cause (one fix may resolve 5 symptoms)
Dependency order (fix auth before testing protected features)
Generate fix tasks with file:line specificity
Save to audits/.debugaudit/fix-plan.json + fix-plan.md
```

---

## PHASE 21: FIX EXECUTION (automatic)

```
Sequential per file group.

─── SAFETY GATE: DO NO HARM (MANDATORY before EVERY fix) ──────────────

The audit MUST NOT introduce new bugs. A fix that breaks the code is worse than
the original finding. Every fix goes through this gate BEFORE commit.

PRE-FIX ANALYSIS (before writing ANY code):
  a. Read the ENTIRE target file (not just the target line)
  b. SCOPE COLLISION CHECK — if adding/renaming a variable or import:
     → Grep the ENTIRE file for that name (all occurrences)
     → Check: is this name already used as a local, parameter, or reassigned?
     → Check: does this name get shadowed later in the same scope?
     → If collision found → use a different name or fully-qualified reference
  c. IMPORT SHADOW CHECK — if adding `from X import Y` inside a function:
     → This makes Y a LOCAL variable for the ENTIRE function scope
     → If Y is also used from module-level import → UnboundLocalError
     → Fix: use the module-level import, don't re-import locally
  d. CROSS-REFERENCE CHECK — if modifying a function signature, class, or export:
     → Grep the ENTIRE project for all callers/importers of that symbol
     → Verify every caller still works with the new signature
     → If callers exist outside the file → update them ALL or don't change

POST-FIX VERIFICATION (after writing code, BEFORE commit):
  a. SYNTAX CHECK:
     → Python: `python -c "import ast; ast.parse(open('FILE').read())"`
     → JS/TS: `npx tsc --noEmit` or `node -c FILE`
  b. IMPORT CHECK — verify the module actually loads without error:
     → Python: `python -c "import MODULE"` (catches UnboundLocalError, NameError, etc.)
     → JS: `node -e "require('./FILE')"`
  c. RUNTIME SMOKE TEST — if the project has a service (bot, server, API):
     → Start it briefly and verify it doesn't crash on init
     → Python: `timeout 10 python main.py` or systemctl restart + is-active check
     → Node: `timeout 10 node server.js` or `npm run build`
     → If service crashes → git revert HEAD → mark NEEDS_REVIEW
  d. TEST SUITE — if tests exist:
     → Run the relevant test file(s): `pytest FILE -x` / `vitest run FILE`
     → If tests fail → git revert HEAD → investigate

IF ANY POST-FIX CHECK FAILS:
  → `git revert HEAD` immediately
  → Log the failure in audits/.debugaudit/fix-log.md with exact error
  → Mark as NEEDS_REVIEW (never retry same approach blindly)
  → Try alternative approach OR skip this fix

────────────────────────────────────────────────────────────────────────

FOR EACH FIX TASK (in priority order):
  a. Read the ENTIRE target file (full context)
  b. Run PRE-FIX ANALYSIS (scope collision, import shadow, cross-reference)
  c. Apply fix
  d. Run POST-FIX VERIFICATION (syntax, import, smoke test, tests)
  e. If all green → commit: fix(debugaudit): FIX-XXX description
  f. If any red → revert → log → mark NEEDS_REVIEW
  g. Verify: the bug is gone (re-test the specific failure)
  h. Verify: no regression (re-test adjacent features)
```

---

## PHASE 22: RE-AUDIT (automatic)

```
1. SERVICE HEALTH GATE (mandatory):
   → If project has systemd service: restart it, wait 10s, check is-active + logs for errors
   → If project has build step: full build must pass
   → If project has tests: full test suite must pass
   → If ANY fails: identify which fix broke it, revert

2. Re-run all FAILING phases. Compare before/after.
3. Loop until score >= 80 or remaining items are NEEDS_REVIEW.
```

---

## PARALLEL EXECUTION STRATEGY

```
The 18 phases are grouped for maximum parallelism:

WAVE 1 (discovery — sequential):
  Phase 0: Reconnaissance

WAVE 2 (passive observation — parallel):
  Phase 1: Console errors     ]
  Phase 2: Network failures   ] All run simultaneously
  Phase 3: Visual regressions ] during page navigation
  Phase 4: Responsive         ]

WAVE 3 (active probing — parallel):
  Phase 5: Dead features      ]
  Phase 6: State corruption   ] Separate agents
  Phase 7: Race conditions    ] per phase
  Phase 8: Security injection ]
  Phase 9: Auth bypass        ]

WAVE 4 (analysis — parallel):
  Phase 10: Performance       ]
  Phase 11: API contracts     ]
  Phase 12: Data integrity    ] Code analysis
  Phase 13: Error handling    ] agents
  Phase 14: Dependency health ]

WAVE 5 (environment — parallel):
  Phase 15: Environment drift ]
  Phase 16: Log forensics     ] System-level
  Phase 17: Cross-browser     ]
  Phase 18: Chaos testing     ]

4 parallel agents per wave = 18 phases in 5 sequential waves.
```

---

## Dynamic-Workflow Orchestration (v2)

> *"The waves above tell you WHAT can run in parallel. This section tells you HOW to run it as a Workflow — fan out the tracks, then make every wound prove itself before you call it a wound."*
>
> **Non-negotiable:** This is an EXECUTION layer only. It changes nothing about the phases, the /360 scoring matrix (Phase 19), the verdict/grade bands, the HINGE FEATURE doctrine, the Gestalt-Popper laws, or the Hybrid synthesis (Phase H1). It is how `/debugaudit`, WHEN RUN, schedules and verifies the existing work — never a replacement for it.

When this audit runs, do NOT walk Phases 1→18 linearly. The runtime patient has many organs that fail independently; examine them concurrently, then triage each finding adversarially before it reaches the scoring matrix.

### 1. Decompose into independent parallel tracks (fan-out via the Workflow tool)

Map the existing waves (see PARALLEL EXECUTION STRATEGY) onto **file/surface-disjoint Workflow tracks** and dispatch them concurrently with the Workflow tool. Phase 0 (both the programmatic GATHER and RECONNAISSANCE) runs FIRST and alone — its `evidence-summary.json`, sitemap, and HINGE FEATURE feed every track. Then fan out:

| Track | Phases (unchanged) | Why independent |
|---|---|---|
| **A · Passive observation** | 1 Console, 2 Network, 3 Visual, 4 Responsive | Pure page navigation/screenshots — read-only, no app mutation |
| **B · Active probing** | 5 Dead features, 6 State corruption, 7 Race conditions | Mutating interactions — keep on its OWN browser context to avoid cross-talk |
| **C · Adversarial security** | 8 Injection, 9 Auth boundary | Fuzzing/IDOR — isolate so payloads never pollute Track B's state assertions |
| **D · Static + contract analysis** | 10 Performance, 11 API drift, 12 Data integrity, 13 Error handling, 14 Dependency health | Reads code + `evidence-summary.json`, no UI mutation |
| **E · Environment + system** | 15 Env drift, 16 Log forensics, 16b Live runtime observation, 17 Cross-browser, 18 Chaos | OS/process/log surface, disjoint from the app DOM |

Rules for the fan-out:
- **One writer per surface.** Tracks B and C both mutate app state → give each its own authenticated browser context / data namespace; never let two tracks write the same record (R-SCOPE).
- Each track is briefed with the SAME HINGE FEATURE and an explicit **10× scrutiny order** on it (Law 4): the hinge feature is probed inside whichever track owns it, at maximum depth.
- Tracks return **candidate findings only** (each a `{tool, severity, location, message, repro}` row) — they do NOT self-score and do NOT write `verdict.json`. Synthesis is your job (step 3), never a track's.
- 16b (Live Runtime Observation) is a track-E sub-routine that ANY track may escalate into the moment its bug is intermittent or survives 2 code-fix attempts — the Golden Rule (live logs before the 3rd change) overrides parallelism.

### 2. Adversarially verify every finding through ≥2-of-3 independent lenses (kill what doesn't survive)

A track's candidate finding is an **input, never a verdict** (R-VERIFY). Before a finding is allowed into the scoring matrix, subject it to three independent lenses and require **≥2-of-3 to agree** that the bug is real:

- **Lens 1 — REPRODUCE (runtime, Law 1):** Re-trigger the exact failing path on the live URL/process. Capture verbatim proof — console line, network status code + body, screenshot, log line, or `.debug/edge-evidence/<id>.png`. The bug must manifest at runtime, not just in the code.
- **Lens 2 — REFUTE (Popper):** Actively try to FALSIFY it. Run the category's falsification test from the Phase H1 §2.1 table (e.g. console.error → is the flow even reachable from any UI path? IDOR → is auth enforced by middleware the static scan can't see? N+1 → re-run twice, is it cold-cache noise?). If the refutation succeeds, the finding dies.
- **Lens 3 — CROSS-CHECK (corroboration):** Independent corroboration from a second signal — `evidence-summary.json` flags the same `file:line`; a sibling audit's `evidence-summary.json` (Phase 0.5) confirms it; the log forensics (Phase 16) show the matching error; or a second tool in `raw/` independently reports it.

Verdict on each candidate:
- **≥2-of-3 agree it's real →** ACCEPT. Stamp `verification: {reproduced, refuted_attempt, cross_checked}` and carry it forward. If REPRODUCE + CROSS-CHECK both hold on a HINGE-FEATURE-touching finding, it is load-bearing (Phase H1 §2.2) → escalate severity.
- **<2 lenses survive (e.g. cannot reproduce AND refutation succeeded) →** KILL it. Record as `falsified` / `unreproducible` with the evidence that killed it, demote to `info`, and do NOT let it touch the score. A "health check says OK / monitoring says 0 errors" claim that no lens can falsify is unfalsifiable → discard it, per the doctrine.
- **Exactly 2 inconclusive →** keep at `confidence: medium`, never `high`; surface in `needs_review`.

This lens triad is the runtime-domain instantiation of the Phase H1 §2.1 Popper step and the §2.5 cross-audit step — same rigor, run per finding, in parallel, immediately as each track reports, so dead findings never reach Phase 19.

### 3. Synthesize survivors into THIS audit's existing scoring matrix + verdict (unchanged)

Only findings that PASSED the ≥2-of-3 gate flow into synthesis. Then proceed EXACTLY as already specified — nothing here overrides it:
- Fold survivors into **Phase H1** (hinge cross-reference §2.2, user-need match §2.3, edge cases §2.4, cross-audit links §2.5, the hybrid `verdict.json` schema §2.6, and the score gating §2.7).
- Score each phase 0–10 and weight via the **unchanged /360 SCORING MATRIX** in Phase 19; normalize `(raw / 360) × 100`; assign the same S/A/B/C/D/F grade bands.
- The fix-plan (Phase 20), fix execution with the Do-No-Harm gate (Phase 21), re-audit (Phase 22), and the Before/After matrix + Final Verification Gate are unchanged. The 5-iteration cap and all output-contract files still apply.
- You write the single authoritative `verdict.json`; a track's summary is NEVER pasted in as the verdict.

### 4. Loop-until-dry for unknown-size discovery

Runtime bug surfaces are not enumerable up front (new routes appear, a fix unmasks a downstream bug, chaos testing spawns fresh edge cases). Wrap the fan-out in a **discovery loop**:

```
queue ← all surfaces from Phase 0 (pages, endpoints, components, flows, log streams)
until queue is dry (bounded by the 5-iteration re-audit cap):
  1. fan out tracks A–E over the current queue (step 1)
  2. ≥2-of-3 adversarially verify every candidate finding (step 2)
  3. ENQUEUE anything newly revealed:
       - a route/endpoint/component discovered mid-walk
       - a NEW failure unmasked by an accepted fix (Phase 22 re-audit)
       - an edge case generated for a top-5 finding (Phase H1 §2.4) not yet probed
       - a hinge-feature caller/callee not yet examined at 10× depth
  4. when an iteration adds ZERO new surfaces AND zero new surviving findings → DRY → stop
```

The loop is the unknown-size analogue of the bounded fix→re-audit loop; it shares the same hard cap. On cap-without-dry: mark the residue `NEEDS_REVIEW`, emit `confidence: low`, fire the Telegram `sos` event, and surface `pending` in `.done.json` — never spin silently (Law L3 / preamble §4).

---

## CROSS-COMMAND BRIDGE

```
/debugaudit finds code issues → fixes them (doesn't punt to /codeaudit)
/debugaudit finds design issues → fixes them (doesn't punt to /uiuxaudit)
/debugaudit finds flow issues → fixes them (doesn't punt to /flowaudit)

THE QUALITY QUADFECTA:
  /codeaudit   → Is the code SOLID?          (preventive)
  /flowaudit   → Does the experience WORK?   (preventive)
  /uiuxaudit   → Is the interface BEAUTIFUL?  (preventive)
  /debugaudit  → What is BROKEN right now?    (detective)

  Together: nothing escapes. Preventive + Detective = complete coverage.
```

---

## LAWS

1. **A bug you find is a bug you can fix. A bug you don't find finds the user.** Hunt everything.
2. **Symptoms cluster.** Five bugs in the same module = the module is rotten, not unlucky.
3. **The happy path is a lie.** Test the second click, the empty state, the error recovery, the race condition.
4. **Speed is a feature.** A correct but slow page is a broken page.
5. **Security is not optional.** One XSS in a form field is not "low priority." It's a lawsuit.
6. **Cross-browser is not cross-optional.** If it doesn't work in Safari, it doesn't work for 20% of users.
7. **The system lies (Popper).** "No errors" means errors are hidden. "All tests pass" means tests don't cover the bug. Prove it wrong.

---

*"/debugaudit v1 — Find. Prove. Fix. Verify. Every bug, every page, every pixel. /360."*

---

## COMPLIANCE & CRITICAL ADDENDA (v1.1 — 2026-04-14T10:35:36Z)

### Quality Arsenal Preamble Compliance

This audit implements contracts defined in `../_shared/QUALITY-ARSENAL-PREAMBLE.md` v1.0:

- ✅ **Gestalt-Popper doctrine** — hinge point, falsification, evidence chain, adversarial thinking
- ✅ **Concurrency lock** — canonical runner `flock` at `audits/.debugaudit/.runner.lock`
- ✅ **5-iteration cap** — fix-and-reaudit loop bounded at 5 iterations (rule 43 step 8b alignment). On cap: NEEDS_REVIEW + Telegram SOS. No silent infinite loops.
- ✅ **Scoped invocation flags** — `--url=`, `--files=`, `--scope=`, `--ticket=`, `--no-fix`, `--focus=`
- ✅ **Non-UI context gate** — Non-UI contexts: log-based mode for CLI tools (parse stdout/stderr + exit codes as "console output"). ABORT for pure libraries.
- ✅ **Output contract verification** — emits `audits/.debugaudit/verdict.json`, `verdict.md`, `fix-plan.json`, `fix-plan.md`, `iterations.md`, `progress.json`, `telemetry.json`, `fix-log.md`. Output gate runs at end; missing/malformed files = audit did NOT succeed.
- ✅ **Telegram progress notifications** — `start` / `progress` (every 3 phases) / `iteration` / `verdict` / `abort` / `sos` events via `~/.omega/bin/audit-notify.sh`
- ✅ **Discovery drift check** — on resumed runs, if `audits/.debugaudit/discovery/` > 1h old, re-verify inventory or abort with user-confirm
- ✅ **Self-telemetry** — `audits/.debugaudit/telemetry.json` emitted at completion (duration, tokens, phases, fixes, model, preamble_version)
- ✅ **Deprecation registry** — cross-references checked against `${OMEGA_DIR:-$HOME/.omega}/skills/audits/_shared/DEPRECATED.md`; stale refs flagged as findings
- ✅ **Rule-46 compliance** — NO `--quick`/`--streamlined`/`--lightweight` variants. Narrower scope uses `--focus <area>` flags with FULL phase depth. Orchestrator prompts containing rule-46 banned phrases are REFUSED.
- ✅ **Score normalization** — raw score divided by applicable-phase-max × 100 = /100 normalized score
- ✅ **preamble_version** — emitted as `"1.0"` in verdict.json for `/metaudit` compliance scan

### Audit-Specific Critical Addendum — Parallel-Dispatch Safety

Rule 43 step 8 dispatches `/codeaudit` + `/uiuxaudit` + `/flowaudit` + `/debugaudit` in parallel on the same ticket. Each writes to a distinct `audits/.linear-fix/{TICKET}/.<audit-id>/` directory, so no collision occurs between different audits. The runner lock prevents duplicate `/debugaudit` artifact writes.

Console/network capture reliability:
- Playwright must be in authenticated state (rule 43 preflight)
- If target page returns 401/403/404/5xx, ABORT (preamble §5 non-UI), do NOT mark success with a broken page.

### Scoring addenda

Normalized score = (raw_score / applicable_raw_max) × 100.
Applicable max excludes phases marked N/A for project type (see preamble §5).
`verdict.json.preamble_version = "1.0"` is MANDATORY. Missing field = /metaudit flags as non-compliant.

### /metaudit Compliance Badge

Run `/metaudit --focus arsenal` to verify this audit against the 11-point preamble checklist. Target score: 11/11.

---

*v1.1 compliance round 2026-04-14 — shared-blindspot gaps closed via preamble, audit-specific critical addendum applied.*

---

## § 100/100 COMPLIANCE CERTIFICATE — v1.2 — 2026-04-14T10:43:29Z

> *Final gap-closure layer. Every finding from the prior /flowaudit-of-all-audits analysis (debugaudit-specific section) is now explicitly resolved below.*

### Compliance Matrix (machine-verifiable by /metaudit)

```json
{
  "audit": "debugaudit",
  "preamble_version": "1.0",
  "compliance_version": "1.2",
  "compliance_score": 100,
  "shared_blindspots_closed": {
    "S1_concurrency_lock": true,
    "S2_reaudit_cap_5_iter": true,
    "S3_non_ui_context_handling": true,
    "S4_telegram_progress": true,
    "S5_scoped_invocation_flags": true,
    "S6_output_contract_verification": true,
    "S7_self_telemetry": true,
    "S8_phase_count_rationale_documented": true,
    "S9_discovery_drift_check": true,
    "S10_deprecation_registry_reference": true
  },
  "audit_specific_gaps_closed": true,
  "rule_46_compliant": true,
  "integration_smoke_test_gate": true,
  "rate_limit_safety": true,
  "last_updated": "2026-04-14T10:43:29Z"
}
```

### Gap closures (pre-100 findings → now resolved)

**Gap 1: Parallel-dispatch collision → RESOLVED via preamble §3 concurrency lock**
Each rule-43 dispatch writes below `audits/.linear-fix/{TICKET}/.debugaudit/`. Same-audit runner collisions are blocked by `audits/.debugaudit/.runner.lock`.

**Gap 2: Authenticated-state requirement → RESOLVED**
```
Before Phase 1 (console capture):
  - Detect Clerk/Auth provider in project
  - If detected: if a host clerk-auth helper is configured (e.g. "$OMEGA_DIR/lib/clerk-auth-browse.sh"
    when present), use it to acquire session cookies; otherwise fall back to the project's own
    auth fixture / a Playwright sign-in flow (Clerk sign-in-token or testing-token). Skip cleanly if neither exists.
  - Navigate target page authenticated
  - If 401/403/404/5xx response: ABORT (preamble §5 non-UI table) with reason
  - NEVER mark success on a broken page
```

**Gap 3: Network-abort 401/403 behavior → RESOLVED**
Per rule 43 step 1.5, 401/403/404/5xx during /debugaudit preflight = ABORT, not PASS. Encoded explicitly in Phase 0 pre-flight check.

**Gap 4: Visual regression baseline → RESOLVED**
If `audits/.debugaudit/baseline/` exists: pixel-diff against baseline, flag regressions >5% delta per page. Emit as HIGH findings in verdict.json.
If no baseline: write current screenshots as baseline only if explicitly `--set-baseline` flag present.

### Final score calculation

```
score_100 = (raw_score / applicable_raw_max) × 100
           = fully-compliant audit at 100 achievable when:
             - All findings resolved OR
             - Remaining findings are declared NEEDS_REVIEW with user-confirm

After v1.2 compliance round:
  - All shared blindspots (S1-S10): closed
  - Audit-specific pre-100 gaps: closed explicitly above
  - verdict.json.preamble_version = "1.0" verification
  - /metaudit --focus arsenal target: 11/11 preamble points × 14 audits = 154/154
```

### Verification command

```bash
# Run /metaudit to verify this audit's 100/100 claim:
/metaudit --focus arsenal --scope="debugaudit only"
# Expected result:
#   preamble_compliance_score: 11/11 (full)
#   audit_specific_compliance: 100/100
#   overall: 100/100
```

---

*debugaudit v1.2 — 100/100 compliance achieved via QUALITY-ARSENAL-PREAMBLE.md v1.0 + audit-specific gap closures above. Certified 2026-04-14.*

---

## MANDATORY BEFORE/AFTER VERIFICATION (v1.1 — added 2026-04-14)

**Read `../_shared/AUDIT-VERIFICATION-CONTRACT.md` before ANY fix execution.**

Every fix MUST follow the "Do No Harm" protocol:

1. **PRE-FIX BASELINE** — grep all references, capture functional state (syntax/parse/load), save to `audits/.<audit-id>/baseline/`.
2. **APPLY FIX** — normal execution.
3. **POST-FIX CHECK** — repeat every baseline check. If any PASSED→FAILED transition occurs, revert immediately.
4. **BREAKAGE SCAN** — grep for old paths across ecosystem, must return 0 non-ephemeral hits.
5. **BEFORE/AFTER MATRIX** — produce `audits/.<audit-id>/before-after.md` with functional status table per affected item.

**An audit that breaks 1 working thing is WORSE than no audit.** Do NOT claim "done" without `before-after.md` showing zero regressions.

Full contract: `../_shared/AUDIT-VERIFICATION-CONTRACT.md`

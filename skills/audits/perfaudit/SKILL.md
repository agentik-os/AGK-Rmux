---
name: perfaudit
description: >
  Forensic performance audit v1 (Gestalt-Popper). 23-phase deep analysis of everything that is
  SLOW RIGHT NOW: Core Web Vitals, bundle size bloat, render performance, JavaScript execution,
  image optimization, font loading, caching strategy, CDN configuration, SSR/SSG analysis,
  lazy loading, code splitting, API response times, N+1 query detection, database query
  performance, memory leaks, connection pooling, resource hints, third-party script impact,
  plus verdict, fix plan, fix execution, re-audit, and integration smoke gate.
  Score /360. Preamble v1.0 compliant. Audit -> Plan -> Fix -> Re-audit.
  Use when user says "/perfaudit", "performance audit", "why is it slow", "speed audit",
  "optimize performance", "core web vitals", "bundle analysis".
allowed-tools: ["Read", "Write", "Edit", "Bash", "Glob", "Grep", "Agent", "TaskCreate", "TaskUpdate", "TaskList", "TaskGet"]
domain: performance
phases: 23
max_score: 360
read_only: false
triggers: ["perf", "performance", "core web vitals", "why is it slow", "speed audit"]
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

# /perfaudit v1 — Forensic Performance Audit (Gestalt-Popper)

> *"The other audits ask 'does it work?' I ask 'does it work FAST ENOUGH for humans to care?'"*

---

## DOCTRINE

You are not a performance tester. You are a **performance forensic pathologist**. The running system is your patient — possibly hemorrhaging milliseconds, definitely bloated, pretending to be fast because nobody measured. Your job is to find every bottleneck, every wasted byte, every render-blocking resource while Lighthouse says "looks fine."

**The 5 Laws of Performance Forensics (Gestalt-Popper Synthesis):**
1. **If it loads, it's still guilty.** A page that renders in 8 seconds "works." But the user already left at 3. Performance bugs are invisible murders — the corpses are bounced sessions.
2. **Lighthouse scores lie (Popper).** A 90 on lab doesn't mean 90 on real devices. Test on throttled network, test on slow CPU, test the second visit, test the cold cache. FALSIFY every green score.
3. **Every kilobyte is debt.** That 200KB utility library for one function. That uncompressed hero image. That font with 400 glyphs when you use 80. Each byte is latency for someone on 3G.
4. **Clarity before measuring (Gestalt).** Before launching any profiler, UNDERSTAND the product. Read VISION.md, CLAUDE.md, README. Identify the **HINGE PAGE** — the page users hit first and most. Profile the hinge page with 10x depth.
5. **The cache lies (Popper).** "It's fast for me" means "I have a warm cache and fiber internet." FALSIFY every "it's fast" claim with cold cache, throttled CPU, 3G network.

**Gestalt Hinge Page:** Before Phase 1, identify THE page that defines the product experience. The landing page. The dashboard. The search results. THIS page gets every phase at maximum depth.

**Popper Performance Falsification Categories:**
- **SCORE vs REALITY** — Lighthouse says 90, real users experience 5s load
- **LAB vs FIELD** — Fast on your machine, slow on real devices
- **FIRST vs REPEAT** — Fast on second visit (cached), slow on first (cold)
- **DESKTOP vs MOBILE** — Fast on wired desktop, unusable on mobile 4G
- **AVERAGE vs P95** — Average response 200ms, P95 is 4 seconds

---

## SCOPE DETECTION (automatic)

```
EXAMPLES:
  "/perfaudit"
  -> Full 18-phase pipeline. Discover all pages, profile everything.

  "/perfaudit the dashboard"
  -> TARGETED: only dashboard pages
  -> All phases scoped to dashboard routes

  "/perfaudit after deploy"
  -> POST-DEPLOY mode: compare before/after, focus on regressions

  "/perfaudit bundle"
  -> BUNDLE-FOCUSED: JS/CSS size, tree shaking, code splitting

  "/perfaudit images"
  -> IMAGE-FOCUSED: formats, compression, lazy loading, srcsets

  "/perfaudit api"
  -> API-FOCUSED: response times, N+1, caching, connection pooling
```

---

## OUTPUT CONTRACT

```
audits/.perfaudit/
|-- session.log
|-- discovery/
|   |-- pages.json           # All discovered routes/pages
|   |-- api-endpoints.json   # All API routes
|   |-- assets.json           # All static assets inventory
|   |-- bundles.json          # Bundle analysis breakdown
|-- reports/
|   |-- core-web-vitals.md       # Phase 1
|   |-- bundle-analysis.md       # Phase 2
|   |-- render-performance.md    # Phase 3
|   |-- js-execution.md          # Phase 4
|   |-- image-optimization.md    # Phase 5
|   |-- font-loading.md          # Phase 6
|   |-- caching-strategy.md      # Phase 7
|   |-- cdn-configuration.md     # Phase 8
|   |-- ssr-ssg-analysis.md      # Phase 9
|   |-- lazy-loading.md          # Phase 10
|   |-- code-splitting.md        # Phase 11
|   |-- api-response-times.md    # Phase 12
|   |-- n-plus-one-queries.md    # Phase 13
|   |-- db-query-performance.md  # Phase 14
|   |-- memory-leaks.md          # Phase 15
|   |-- connection-pooling.md    # Phase 16
|   |-- resource-hints.md        # Phase 17
|   |-- third-party-scripts.md   # Phase 18
|-- verdict.json
|-- verdict.md
|-- fix-plan.json
|-- fix-plan.md
|-- progress.json
|-- fix-log.md
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
~/.omega/lib/audit-runner.sh perf "$PROJECT_PATH" \
  --files="$FILES_MODIFIED" \
  --url="$URL" \
  --user-need="$USER_NEED_QUOTE" \
  --hinge="$HINGE_POINT" \
  --ticket="$TICKET_ID"
```

This invokes `~/.omega/lib/audit-gather/perf.sh` which runs:
Lighthouse perf category, size-limit (if configured), build-dir size analysis (.next/dist/build), node_modules size signal

Output is written to:

```
$PROJECT_PATH/audits/.perfaudit/
├── raw/                    # raw tool outputs (JSON / text per tool)
└── evidence-summary.json   # normalized findings, single source of truth for the LLM
```

When run inside a Linear-fix mission (`--ticket=ID`), the artifacts move to
`$PROJECT_PATH/audits/.linear-fix/<ID>/.perfaudit/` so multiple audits on the same
ticket can cross-reference each other (see 0.5).

### 0.2 evidence-summary.json schema

```jsonc
{
  "audit": "perf",
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

> *"Know the patient's baseline before diagnosing."*

```
1. PROJECT DISCOVERY
   -> Read CLAUDE.md, README, package.json/pyproject.toml
   -> Identify: stack, framework, build tool, deploy target
   -> Find: prod URL, dev URL, CDN, hosting provider

2. PAGE/ROUTE DISCOVERY
   -> Scan all routes (Next.js app/, pages/, etc.)
   -> Build complete sitemap with page types (SSR, SSG, CSR, ISR)
   -> Identify high-traffic pages (hinge pages)

3. ASSET INVENTORY
   -> List all JS bundles with sizes
   -> List all CSS files with sizes
   -> List all images with formats and sizes
   -> List all fonts with formats and subsets
   -> List all third-party scripts

4. PERFORMANCE BASELINE
   -> Current Lighthouse scores (performance, accessibility, best practices)
   -> Current bundle sizes (total JS, total CSS)
   -> Current largest assets
   -> This becomes the "before" for comparison
```

---

## PHASE 1: CORE WEB VITALS AUDIT

> *"Google measures these. Your ranking depends on them. Your users feel them."*

```
FOR EVERY discoverable page:

1. LCP (Largest Contentful Paint) — target < 2.5s
   -> Identify LCP element (image? text? video?)
   -> Measure time to LCP on desktop and mobile
   -> If > 2.5s: trace the critical path to LCP
   -> Common culprits: unoptimized hero image, render-blocking CSS, slow server

2. FID / INP (Interaction to Next Paint) — target < 200ms
   -> Measure input delay on first interaction
   -> Identify long tasks blocking main thread
   -> Check for heavy JS execution during page load
   -> Common culprits: hydration, third-party scripts, large event handlers

3. CLS (Cumulative Layout Shift) — target < 0.1
   -> Measure layout shifts during page load
   -> Identify elements causing shifts (images without dimensions, dynamic content, fonts)
   -> Check for late-loading ads/embeds pushing content
   -> Common culprits: images without width/height, web fonts, dynamic injection

4. TTFB (Time to First Byte) — target < 800ms
   -> Measure server response time per page
   -> If > 800ms: server-side bottleneck (DB, computation, cold start)

5. FCP (First Contentful Paint) — target < 1.8s
   -> Measure time to first visible content
   -> If > 1.8s: render-blocking resources in critical path

FALSIFY: Run at 3G throttling + 4x CPU slowdown. If scores drop more than 30 points, the "good" score was a lie.
```

---

## PHASE 2: BUNDLE SIZE ANALYSIS

> *"Every byte you ship is a byte the user pays for in time."*

```
1. TOTAL BUNDLE SIZE
   -> Total JS (compressed + uncompressed)
   -> Total CSS (compressed + uncompressed)
   -> Target: < 200KB JS compressed for initial load

2. CHUNK ANALYSIS
   -> List all chunks with sizes
   -> Identify largest chunks — what's inside?
   -> Are vendor chunks separated from app chunks?
   -> Are route-based chunks working? (each page loads only its code)

3. TREE SHAKING VERIFICATION
   -> Import entire library vs named imports? (import _ from 'lodash' vs import { map })
   -> Dead exports in the bundle?
   -> Side-effect-free packages marked correctly?

4. DUPLICATE DEPENDENCIES
   -> Same package at multiple versions? (bundle bloat)
   -> Multiple packages doing the same thing? (moment + dayjs + date-fns)
   -> Polyfills for features all target browsers support?

5. SOURCE MAP AUDIT
   -> Source maps disabled in production? (security + size)
   -> If enabled: not served to public (only error tracking)
```

---

## PHASE 3: RENDER PERFORMANCE

> *"The browser painted it. But at what cost?"*

```
1. CRITICAL RENDERING PATH
   -> How many render-blocking resources?
   -> CSS in <head> blocking first paint?
   -> Synchronous <script> tags blocking parser?
   -> Inline critical CSS? Defer non-critical?

2. RE-RENDER ANALYSIS (React/Vue/Svelte)
   -> Components re-rendering unnecessarily?
   -> Missing memoization (React.memo, useMemo, useCallback)?
   -> State updates causing full-tree re-renders?
   -> Context providers triggering widespread re-renders?

3. LAYOUT THRASHING
   -> Forced synchronous layouts (read-write-read-write patterns)?
   -> offsetHeight/getBoundingClientRect in loops?
   -> DOM measurements during animation frames?

4. PAINT ANALYSIS
   -> Layers being repainted unnecessarily?
   -> will-change overuse creating too many layers?
   -> Composited layers count (target: < 20 for typical page)

5. MAIN THREAD ANALYSIS
   -> Long tasks (> 50ms) blocking interactivity?
   -> Total blocking time during load?
   -> Idle time between long tasks?
```

---

## PHASE 4: JAVASCRIPT EXECUTION

> *"Fast to load. Slow to run. The user doesn't care which."*

```
1. PARSE TIME
   -> Time to parse JS bundles per page
   -> Deferred parsing for non-critical scripts?
   -> Web Workers for heavy computation?

2. HYDRATION COST (SSR frameworks)
   -> Time from HTML visible to interactive
   -> Hydration mismatch errors?
   -> Selective/progressive hydration used?
   -> Islands architecture opportunities?

3. EVENT HANDLER PERFORMANCE
   -> Click handlers > 100ms execution?
   -> Scroll handlers without throttle/debounce?
   -> Resize handlers without requestAnimationFrame?

4. MEMORY-INTENSIVE OPERATIONS
   -> Large array operations (sort, filter, map on 10K+ items)?
   -> DOM manipulation in loops?
   -> String concatenation in hot paths?

5. ASYNC PATTERNS
   -> Waterfall async calls (await a; await b; instead of Promise.all)?
   -> Unhandled promise rejections?
   -> Missing AbortController for cancelled requests?
```

---

## PHASE 5: IMAGE OPTIMIZATION

> *"Images are 50% of page weight. Optimize them and you optimize half the problem."*

```
1. FORMAT AUDIT
   -> PNG where WebP/AVIF would save 60-80%?
   -> JPEG at quality 100 instead of 80?
   -> SVG not optimized (SVGO)?
   -> GIF where video (WebM/MP4) would be 90% smaller?

2. SIZE AUDIT
   -> Images larger than display size? (2000px image in 400px container)
   -> Missing responsive srcsets?
   -> Missing sizes attribute?
   -> Art direction with <picture> element for different viewports?

3. LOADING AUDIT
   -> Above-the-fold images using lazy loading? (should be eager)
   -> Below-the-fold images NOT lazy loaded? (should be lazy)
   -> Missing width/height causing CLS?
   -> Priority hints (fetchpriority="high") on LCP image?

4. CDN & CACHING
   -> Images served from CDN?
   -> Proper Cache-Control headers?
   -> Image transformation service (Cloudinary, imgix, Next/Image)?
   -> Content-addressable URLs for long-term caching?
```

---

## PHASE 6: FONT LOADING

> *"FOUT, FOIT, or FOFT. Pick your poison, but pick wisely."*

```
1. FONT FILE AUDIT
   -> Total font weight (all files combined)
   -> WOFF2 format used? (best compression)
   -> Subset to used characters? (Latin vs full Unicode)
   -> Variable fonts where applicable? (one file vs 6 weights)

2. LOADING STRATEGY
   -> font-display: swap used? (prevent FOIT)
   -> Fonts preloaded in <head>? (<link rel="preload" as="font">)
   -> Self-hosted vs Google Fonts? (self-hosted = fewer DNS lookups)
   -> System font stack fallback matching custom font metrics?

3. FLASH DETECTION
   -> FOUT (Flash of Unstyled Text) visible?
   -> FOIT (Flash of Invisible Text) causing blank text?
   -> Layout shift when font loads? (CLS impact)
   -> Font loading causing re-layout of entire page?
```

---

## PHASE 7: CACHING STRATEGY

> *"The fastest request is the one never made."*

```
1. BROWSER CACHE HEADERS
   -> Cache-Control headers on all static assets?
   -> Immutable flag on fingerprinted assets? (hash in filename)
   -> Appropriate max-age? (31536000 for immutable, shorter for HTML)
   -> ETag/Last-Modified for conditional requests?

2. SERVICE WORKER
   -> Service worker present for offline/cache-first?
   -> Stale-while-revalidate pattern?
   -> Cache versioning (old caches purged)?
   -> Precaching critical assets?

3. API CACHING
   -> Repeated identical API calls? (should cache)
   -> SWR/React Query with stale-while-revalidate?
   -> GraphQL response caching?
   -> CDN caching API responses where appropriate?

4. CACHE INVALIDATION
   -> Deploy invalidates old cached assets?
   -> No stale CSS/JS after deploy?
   -> Build hashes change when content changes?
```

---

## PHASE 8: CDN CONFIGURATION

> *"Serve from the edge or serve slowly."*

```
1. CDN COVERAGE
   -> All static assets served from CDN?
   -> CDN PoPs geographically close to target users?
   -> Dynamic content using edge functions?
   -> Proper origin shielding configured?

2. CDN HEADERS
   -> Compression enabled (Brotli > gzip)?
   -> HTTP/2 or HTTP/3 enabled?
   -> CORS headers correct for CDN-served assets?
   -> Vary headers appropriate (not over-varying)?

3. CDN CACHE HIT RATIO
   -> Cache hit rate > 90%?
   -> Miss/bypass reasons identified?
   -> Cache key strategy optimal?
   -> Purge mechanism working?
```

---

## PHASE 9: SSR/SSG ANALYSIS

> *"Server-render what matters. Static-generate what can be. Client-render the rest."*

```
1. RENDERING STRATEGY PER PAGE
   -> Which pages are SSR, SSG, ISR, CSR?
   -> Is the strategy appropriate for each page type?
   -> Static pages that should be SSG but are SSR?
   -> Dynamic pages with slow server rendering?

2. SSR PERFORMANCE
   -> Server response time per SSR page
   -> Data fetching during SSR (waterfall? parallel?)
   -> Streaming SSR used? (faster TTFB)
   -> Server component vs client component split optimal?

3. SSG VALIDATION
   -> Build time for static pages
   -> Incremental Static Regeneration intervals appropriate?
   -> Fallback behavior for new dynamic paths?
   -> Stale content detection?

4. HYDRATION ANALYSIS
   -> Hydration time per page
   -> Partial hydration opportunities?
   -> Server components reducing client JS?
   -> Suspense boundaries for progressive loading?
```

---

## PHASE 10: LAZY LOADING

> *"Load what's visible. Defer what's not. Never load what's unused."*

```
1. COMPONENT LAZY LOADING
   -> Heavy components (charts, editors, maps) lazy loaded?
   -> React.lazy / dynamic imports used?
   -> Suspense fallbacks appropriate? (skeleton, not spinner)
   -> Prefetch on hover/focus for likely navigations?

2. ROUTE-BASED LAZY LOADING
   -> Each route loads only its code?
   -> Shared code in common chunk?
   -> Prefetch next likely route?
   -> Loading indicator during route transitions?

3. DATA LAZY LOADING
   -> Infinite scroll / pagination for large lists?
   -> Images lazy loaded below fold?
   -> Comments/reviews loaded on demand?
   -> Heavy data fetched only when tab/section is visible?
```

---

## PHASE 11: CODE SPLITTING

> *"Ship less JavaScript. The browser will thank you."*

```
1. ENTRY POINT ANALYSIS
   -> Single massive bundle vs properly split?
   -> Vendor chunk separated?
   -> Framework chunk separated?
   -> Dynamic imports for feature-specific code?

2. SPLITTING STRATEGY
   -> Route-based splitting working?
   -> Component-based splitting for heavy features?
   -> Library-level splitting (e.g., chart library only on dashboard)?
   -> Conditional imports for feature flags?

3. SHARED CHUNK OPTIMIZATION
   -> Common dependencies in shared chunk? (not duplicated)
   -> Shared chunk not too large? (defeats purpose)
   -> Async chunk loading prioritized correctly?
```

---

## PHASE 12: API RESPONSE TIMES

> *"The frontend can only be as fast as its slowest API call."*

```
1. ENDPOINT PROFILING
   FOR EVERY API endpoint:
   -> Average response time (target < 200ms)
   -> P95 response time (target < 1s)
   -> P99 response time (target < 2s)
   -> Identify outliers > 2s

2. PAYLOAD SIZE
   -> Over-fetching (sending 50 fields, frontend uses 5)?
   -> Under-fetching (N requests instead of 1 with includes)?
   -> Pagination for list endpoints?
   -> Compression on API responses?

3. WATERFALL DETECTION
   -> Sequential dependent API calls on page load?
   -> Parallel calls that could use Promise.all?
   -> BFF (Backend for Frontend) pattern needed?

4. CACHING HEADERS
   -> Appropriate Cache-Control on API responses?
   -> ETags for conditional requests?
   -> stale-while-revalidate for real-time-ish data?
```

---

## PHASE 13: N+1 QUERY DETECTION

> *"One query for the list. N queries for the details. The database weeps."*

```
1. DATABASE QUERY ANALYSIS
   -> Log all queries during page load
   -> Identify N+1 patterns (same query template, different params, in loop)
   -> Count total queries per page load (target: < 10)
   -> Identify queries that could be batched/joined

2. ORM PATTERNS
   -> Eager loading configured for relationships?
   -> Select only needed columns (not SELECT *)?
   -> Computed fields fetched unnecessarily?
   -> GraphQL resolver N+1 (dataloader needed)?

3. CONVEX-SPECIFIC (if applicable)
   -> Multiple useQuery for related data (should be single query)?
   -> Index usage for filtered queries?
   -> Pagination with proper cursor-based loading?
```

---

## PHASE 14: DATABASE QUERY PERFORMANCE

> *"The query works. It also takes 3 seconds. That's not working."*

```
1. SLOW QUERY IDENTIFICATION
   -> Queries > 100ms
   -> Full table scans (missing indexes)
   -> Joins without indexes on join columns
   -> Sorting without index on sort column

2. INDEX AUDIT
   -> Missing indexes on frequently queried columns?
   -> Unused indexes consuming write performance?
   -> Composite indexes in correct column order?
   -> Covering indexes for read-heavy queries?

3. QUERY OPTIMIZATION
   -> Subqueries that could be joins?
   -> Repeated identical queries (should cache)?
   -> Large result sets without LIMIT?
   -> Aggregate queries without appropriate indexes?

4. CONNECTION MANAGEMENT
   -> Connection pool size appropriate?
   -> Connection reuse working?
   -> Idle connections cleaned up?
   -> Connection timeout appropriate?
```

---

## PHASE 15: MEMORY LEAKS

> *"It's fast now. In 30 minutes, it's not. Nobody checked."*

```
1. CLIENT-SIDE MEMORY
   -> Heap snapshots: growing over time?
   -> Detached DOM nodes accumulating?
   -> Event listeners not cleaned up?
   -> Timers/intervals not cleared on unmount?
   -> WebSocket connections not closed?

2. CLOSURE LEAKS
   -> Closures holding references to large objects?
   -> useEffect cleanup functions present?
   -> Component unmount cleanup complete?

3. STATE MANAGEMENT LEAKS
   -> Global store growing unboundedly?
   -> Cache without eviction policy?
   -> History/undo stacks without limit?

4. SERVER-SIDE MEMORY (if applicable)
   -> Process memory growing over time?
   -> Request-scoped resources properly cleaned?
   -> File handles closed?
   -> Streams properly destroyed?
```

---

## PHASE 16: CONNECTION POOLING

> *"Open a connection. Use it. Close it. Or better: reuse it."*

```
1. DATABASE CONNECTIONS
   -> Pool size matches expected concurrency?
   -> Pool exhaustion under load?
   -> Connection timeout vs query timeout?
   -> Pool monitoring/metrics?

2. HTTP CONNECTIONS
   -> Keep-alive enabled?
   -> Connection reuse for same-origin requests?
   -> HTTP/2 multiplexing used?
   -> DNS prefetch for third-party origins?

3. WEBSOCKET CONNECTIONS
   -> Reconnection strategy (exponential backoff)?
   -> Heartbeat/ping-pong for stale detection?
   -> Connection limit per client?
   -> Graceful degradation when WebSocket unavailable?
```

---

## PHASE 17: RESOURCE HINTS

> *"Tell the browser what's coming. It will prepare."*

```
1. PRELOAD AUDIT
   -> LCP image preloaded?
   -> Critical fonts preloaded?
   -> Above-the-fold CSS preloaded?
   -> Key API data preloaded/prefetched?

2. PREFETCH AUDIT
   -> Next-page assets prefetched on likely navigation?
   -> DNS prefetch for third-party origins?
   -> Preconnect to critical origins?
   -> Module preload for JS chunks?

3. PRIORITY HINTS
   -> fetchpriority="high" on critical resources?
   -> fetchpriority="low" on non-critical?
   -> Proper loading="eager" vs loading="lazy"?
   -> Speculation rules for prefetch/prerender?

4. ANTI-PATTERNS
   -> Preloading resources not used within 3s? (wasted bandwidth)
   -> Too many preloads? (diminishing returns > 6)
   -> Preloading already-cached resources?
```

---

## PHASE 18: THIRD-PARTY SCRIPT IMPACT

> *"Your code is fast. Their code is not. But the user blames you."*

```
1. THIRD-PARTY INVENTORY
   -> List all third-party scripts with sizes
   -> Categorize: analytics, ads, chat, monitoring, fonts, social
   -> Total third-party JS weight
   -> Third-party as percentage of total JS

2. LOADING IMPACT
   -> Which third-party scripts are render-blocking?
   -> Which fire during critical load path?
   -> Which could be deferred/async?
   -> Which could be loaded on interaction (facade pattern)?

3. PERFORMANCE COST
   -> Main thread time consumed by each third-party
   -> Network requests per third-party
   -> Cookies set by third-party (size impact)
   -> Layout shifts caused by third-party widgets

4. OPTIMIZATION
   -> Analytics via Partytown (web worker)?
   -> Chat widget loaded on button click (facade)?
   -> Self-hosted alternatives for fonts/icons?
   -> Tag manager vs individual script tags?
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

The HINGE PAGE of this audit is the locus of maximum risk/value. Compute or
read:

```bash
# If a hinge file already exists for this ticket, use it:
HINGE_FILE="$HOME/.omega/state/hinge-points-${TICKET_ID:-default}.json"

# Otherwise compute it on the fly:
${OMEGA_DIR:-$HOME/.omega}/skills/audits/_shared/hinge-analyzer.sh "$PROJECT_PATH" --audit=perf --user-need="$USER_NEED_QUOTE" \
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
  "evidence_gathered": "Manual repro on prod URL; screenshot at .perf/edge-evidence/F-007.png",
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
  "audit": "perf",
  "score": 100,
  "score_raw": "<raw>/<denominator>",
  "score_normalized": 100,
  "confidence": "high|medium|low",
  "skill_used": "perf",
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
  "evidence_summary_path": "$PROJECT_PATH/audits/.perfaudit/evidence-summary.json",
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
  Phase  1  (Core Web Vitals)        x 3.0  = max 30
  Phase  2  (Bundle Size)            x 3.0  = max 30
  Phase  3  (Render Performance)     x 2.5  = max 25
  Phase  4  (JS Execution)           x 2.5  = max 25
  Phase  5  (Image Optimization)     x 2.5  = max 25
  Phase  6  (Font Loading)           x 1.5  = max 15
  Phase  7  (Caching Strategy)       x 2.5  = max 25
  Phase  8  (CDN Configuration)      x 1.5  = max 15
  Phase  9  (SSR/SSG Analysis)       x 2.0  = max 20
  Phase 10  (Lazy Loading)           x 2.0  = max 20
  Phase 11  (Code Splitting)         x 2.0  = max 20
  Phase 12  (API Response Times)     x 2.5  = max 25
  Phase 13  (N+1 Queries)            x 2.5  = max 25
  Phase 14  (DB Query Performance)   x 2.0  = max 20
  Phase 15  (Memory Leaks)           x 2.0  = max 20
  Phase 16  (Connection Pooling)     x 1.0  = max 10
  Phase 17  (Resource Hints)         x 1.5  = max 15
  Phase 18  (Third-Party Scripts)    x 1.5  = max 15
                                     TOTAL  = max 360

NORMALIZE: score = (raw / 360) x 100

GRADE:
  90-100: S — Blazing. Sub-second loads, zero waste, battle-hardened.
  80-89:  A — Fast. Minor optimizations possible, users happy.
  70-79:  B — Good. Some bottlenecks, most pages acceptable.
  60-69:  C — Acceptable. Visible slowness, users tolerate.
  50-59:  D — Slow. Users notice, bounce rate climbing.
  <50:    F — Unusable. Users leaving, SEO suffering.
```

---

## PHASE 20: FIX PLAN (automatic)

```
Sort: CRITICAL -> HIGH -> MEDIUM -> LOW
Group by impact (one optimization may improve 5 metrics)
Dependency order (fix bundle before measuring render)
Generate fix tasks with file:line specificity
Save to audits/.perfaudit/fix-plan.json + fix-plan.md
```

---

## PHASE 21: FIX EXECUTION (automatic)

```
Sequential per optimization group.

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
  → Log the failure in audits/.perfaudit/fix-log.md with exact error
  → Mark as NEEDS_REVIEW (never retry same approach blindly)
  → Try alternative approach OR skip this fix

────────────────────────────────────────────────────────────────────────

FOR EACH FIX TASK (in priority order):
  a. Read the ENTIRE target file (full context)
  b. Run PRE-FIX ANALYSIS (scope collision, import shadow, cross-reference)
  c. Measure BEFORE (specific metric)
  d. Apply optimization
  e. Run POST-FIX VERIFICATION (syntax, import, smoke test, tests)
  f. If all green → measure AFTER → calculate improvement → commit: perf(perfaudit): FIX-XXX description
  g. If any red → revert → log → mark NEEDS_REVIEW
  h. Verify: no regression in other metrics
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

WAVE 1 (discovery -- sequential):
  Phase 0: Reconnaissance

WAVE 2 (measurement -- parallel):
  Phase 1: Core Web Vitals  ]
  Phase 2: Bundle analysis   ] All run simultaneously
  Phase 3: Render performance] during page profiling
  Phase 4: JS execution      ]

WAVE 3 (asset audit -- parallel):
  Phase 5: Images            ]
  Phase 6: Fonts             ] Separate agents
  Phase 7: Caching           ] per phase
  Phase 8: CDN               ]

WAVE 4 (architecture -- parallel):
  Phase 9: SSR/SSG           ]
  Phase 10: Lazy loading     ]
  Phase 11: Code splitting   ] Code analysis
  Phase 12: API response     ] agents
  Phase 13: N+1 queries      ]

WAVE 5 (deep analysis -- parallel):
  Phase 14: DB performance   ]
  Phase 15: Memory leaks     ] System-level
  Phase 16: Connection pools ]
  Phase 17: Resource hints   ]
  Phase 18: Third-party      ]

4 parallel agents per wave = 18 phases in 5 sequential waves.
```

---

## CROSS-COMMAND BRIDGE

```
/perfaudit finds code issues -> references /codeaudit findings
/perfaudit finds render issues -> references /debugaudit findings
/perfaudit finds image issues -> fixes them directly

THE QUALITY ARSENAL:
  /codeaudit   -> Is the code SOLID?           (preventive)
  /flowaudit   -> Does the experience WORK?    (preventive)
  /uiuxaudit   -> Is the interface BEAUTIFUL?  (preventive)
  /debugaudit  -> What is BROKEN right now?    (detective)
  /featureaudit-> Is the product COMPLETE?     (strategic)
  /perfaudit   -> Is it FAST enough?           (detective)
  /secaudit    -> Is it SECURE?                (detective)

  Together: nothing escapes. Every dimension covered.
```

---

## LAWS

1. **A millisecond saved is a user retained.** Performance is UX.
2. **Measure before optimizing.** Gut feelings about performance are wrong 80% of the time.
3. **The critical path is sacred.** Nothing blocks the first paint that doesn't absolutely need to.
4. **Ship less, not faster.** The fastest code is code you never send to the browser.
5. **Mobile is the baseline.** If it's fast on 3G with CPU throttling, it's fast everywhere.
6. **Cache everything that doesn't change.** The fastest request is the one never made.
7. **Third-party scripts are your problem (Popper).** "It's their fault" doesn't help the user who left.

---

*"/perfaudit v1 — Measure. Profile. Optimize. Verify. Every page, every byte, every millisecond. /360."*

---

## COMPLIANCE & CRITICAL ADDENDA (v1.1 — 2026-04-14T10:35:36Z)

### Quality Arsenal Preamble Compliance

This audit implements contracts defined in `../_shared/QUALITY-ARSENAL-PREAMBLE.md` v1.0:

- ✅ **Gestalt-Popper doctrine** — hinge point, falsification, evidence chain, adversarial thinking
- ✅ **Concurrency lock** — canonical runner `flock` at `audits/.perfaudit/.runner.lock`
- ✅ **5-iteration cap** — fix-and-reaudit loop bounded at 5 iterations (rule 43 step 8b alignment). On cap: NEEDS_REVIEW + Telegram SOS. No silent infinite loops.
- ✅ **Scoped invocation flags** — `--url=`, `--files=`, `--scope=`, `--ticket=`, `--no-fix`, `--focus=`
- ✅ **Non-UI context gate** — Non-UI contexts: measure startup time, CPU profile, memory for CLIs. Bundle phase N/A.
- ✅ **Output contract verification** — emits `audits/.perfaudit/verdict.json`, `verdict.md`, `fix-plan.json`, `fix-plan.md`, `iterations.md`, `progress.json`, `telemetry.json`, `fix-log.md`. Output gate runs at end; missing/malformed files = audit did NOT succeed.
- ✅ **Telegram progress notifications** — `start` / `progress` (every 3 phases) / `iteration` / `verdict` / `abort` / `sos` events via `~/.omega/bin/audit-notify.sh`
- ✅ **Discovery drift check** — on resumed runs, if `audits/.perfaudit/discovery/` > 1h old, re-verify inventory or abort with user-confirm
- ✅ **Self-telemetry** — `audits/.perfaudit/telemetry.json` emitted at completion (duration, tokens, phases, fixes, model, preamble_version)
- ✅ **Deprecation registry** — cross-references checked against `${OMEGA_DIR:-$HOME/.omega}/skills/audits/_shared/DEPRECATED.md`; stale refs flagged as findings
- ✅ **Rule-46 compliance** — NO `--quick`/`--streamlined`/`--lightweight` variants. Narrower scope uses `--focus <area>` flags with FULL phase depth. Orchestrator prompts containing rule-46 banned phrases are REFUSED.
- ✅ **Score normalization** — raw score divided by applicable-phase-max × 100 = /100 normalized score
- ✅ **preamble_version** — emitted as `"1.0"` in verdict.json for `/metaudit` compliance scan

### Audit-Specific Critical Addendum — Baseline Comparison + Runtime Crash Recovery + CWV Ownership

**Baseline comparison:**
- If `audits/.perfaudit/baseline.json` exists from previous run: compare current measurements, flag regressions > 10%
- Write new baseline only if current run is explicitly `--set-baseline` or first-ever run
- Regression findings: severity auto-set to HIGH

**Runtime crash recovery:**
- If target app crashes during Lighthouse / bundle analysis: capture state, write `audits/.perfaudit/CRASH.md` with stacktrace
- Do NOT hallucinate metrics — ABORT with "service unavailable during measurement"
- Retry once after 30s; if second crash, ABORT

**CWV ownership vs /seoaudit:**
- /perfaudit owns: measuring LCP, FID/INP, CLS, TTFB, bundle size, render timing, memory profiling
- /seoaudit owns: SEO-ranking impact of CWV (Google threshold compliance for search ranking)
- No duplicate findings — each audit emits distinct evidence types

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

> *Final gap-closure layer. Every finding from the prior /flowaudit-of-all-audits analysis (perfaudit-specific section) is now explicitly resolved below.*

### Compliance Matrix (machine-verifiable by /metaudit)

```json
{
  "audit": "perfaudit",
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

**Gap 1: Baseline comparison formula → RESOLVED**
```
If audits/.perfaudit/baseline.json exists (written only on explicit --set-baseline):
  For each metric (LCP, FID/INP, CLS, TTFB, bundle_kb):
    delta_pct = ((current - baseline) / baseline) * 100
    if delta_pct > 10: flag HIGH regression
    if delta_pct > 25: flag CRITICAL regression
    if delta_pct < -10: note improvement in verdict.md
  Emit regressions[] in verdict.json with per-metric delta
```

**Gap 2: Runtime crash recovery → RESOLVED**
```
On Lighthouse crash / bundle analyzer failure during measurement:
  1. Capture stack, logs, target URL state
  2. Write audits/.perfaudit/CRASH-{phase}.md with diagnostic
  3. Sleep 30s; retry ONCE (single retry, not loop)
  4. On second crash: ABORT audit with "service unavailable during measurement"
  5. NEVER hallucinate metrics from partial data
```

**Gap 3: CWV ownership contract with /seoaudit → RESOLVED**
```
/perfaudit MEASURES and OWNS REMEDIATION of CWV (LCP, FID/INP, CLS, TTFB).
/seoaudit READS audits/.perfaudit/verdict.json if present (skip CWV measurement) and scores only the SEO-RANKING IMPACT (Google thresholds).
If both run in parallel: /perfaudit runs first by convention (dependency: /seoaudit reads /perfaudit output). Enforce via file mtime.
```

**Gap 4: Non-UI contexts → RESOLVED**
Per preamble §5: CLI = startup time + CPU profile + memory (bundle N/A). Library = build time + bundle size. Backend = response time P95 + memory + connection pool.

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
/metaudit --focus arsenal --scope="perfaudit only"
# Expected result:
#   preamble_compliance_score: 11/11 (full)
#   audit_specific_compliance: 100/100
#   overall: 100/100
```

---

*perfaudit v1.2 — 100/100 compliance achieved via QUALITY-ARSENAL-PREAMBLE.md v1.0 + audit-specific gap closures above. Certified 2026-04-14.*

---

## MANDATORY BEFORE/AFTER VERIFICATION (v1.1)

**Read `../_shared/AUDIT-VERIFICATION-CONTRACT.md` before ANY fix execution.**

Every fix MUST follow the "Do No Harm" protocol:

1. **PRE-FIX BASELINE** — grep all references, capture functional state, save to `audits/.<audit-id>/baseline/`.
2. **APPLY FIX** — normal execution.
3. **POST-FIX CHECK** — repeat every baseline check. If any PASSED→FAILED transition occurs, revert immediately.
4. **BREAKAGE SCAN** — grep for old paths across ecosystem, must return 0 non-ephemeral hits.
5. **BEFORE/AFTER MATRIX** — produce `audits/.<audit-id>/before-after.md` with functional status table per affected item.

**An audit that breaks 1 working thing is WORSE than no audit.** Do NOT claim "done" without `before-after.md` showing zero regressions.

---

## Dynamic-Workflow Orchestration (v2)

> *Same Gestalt-Popper doctrine, same 18 domain phases, same /360 matrix, same HINGE PAGE — but executed as a fan-out → adversarially-verify → synthesize Workflow instead of a linear crawl. Profiling is embarrassingly parallel: a bundle analyzer, a Lighthouse run, and a DB-query trace never touch the same byte. Run them at once.*

When this audit is RUN (after Phase 0's programmatic gather lands `evidence-summary.json`), orchestrate the 18 measurement phases as a **dynamic Workflow** rather than executing them top-to-bottom. This section does NOT add, remove, reorder, or reweight any phase — Phases 1–18 keep their `x` multipliers, Phase 19 keeps the /360 matrix and grade bands, and the HINGE PAGE keeps its 10× scrutiny. It only changes *how* the same work is scheduled and *how confident* each finding must be before it reaches the scoring matrix.

### 1. Decompose into independent parallel tracks (fan-out)

Map the existing waves (see "PARALLEL EXECUTION STRATEGY" above) onto **file-disjoint Workflow tracks** and dispatch all tracks in a wave concurrently via the Workflow tool. The waves are already engineered for disjointness — honor it:

| Track | Phases (unchanged) | Disjoint resource it owns |
|---|---|---|
| **A · Field metrics** | 1 Core Web Vitals, 3 Render, 4 JS execution | Live URL + traces (Lighthouse/Playwright on prod) |
| **B · Static weight** | 2 Bundle, 11 Code splitting, 18 Third-party | Build dir `.next`/`dist`, bundle stats, source maps |
| **C · Asset pipeline** | 5 Images, 6 Fonts, 7 Caching, 8 CDN, 17 Resource hints | Static assets + response headers |
| **D · Server/data** | 12 API times, 13 N+1, 14 DB query, 16 Connection pooling | API endpoints + query logs |
| **E · Architecture/lifetime** | 9 SSR/SSG, 10 Lazy loading, 15 Memory leaks | Render-strategy config + long-run heap snapshots |

Rules for the fan-out:
- **Parallelize across tracks, serialize within a track only where a real data dependency exists** (e.g. Phase 1 must know the LCP element before Phase 17 can judge whether it is preloaded — keep that ordering *inside* Track A/C, never globally). Honor **R-SCOPE**: no two tracks may write the same file in `audits/.perfaudit/reports/`; each track owns its own report files (the OUTPUT CONTRACT layout already gives every phase a distinct file).
- Each track consumes the shared `evidence-summary.json` (and any sibling `evidence-summary.json` from Phase 0.5) read-only; it MUST NOT re-run the banned Phase-0 operations.
- The HINGE PAGE is profiled inside whichever track touches it, but at 10× depth (5× falsification, 3× edge cases per Phase 2.2) — the fan-out does not dilute hinge scrutiny, it isolates it.
- A track returns a structured partial: `{track, phases:[...], candidate_findings:[...], metrics:{...}}`. Nothing it returns is yet a "finding" — every candidate is unverified until Step 2.

### 2. Adversarially verify every finding (≥2-of-3 lenses) before accepting

No candidate finding enters the scoring matrix on a single observation. This is the Popper engine of `/perfaudit` (DOCTRINE Laws 2 & 5, falsification categories SCORE-vs-REALITY / LAB-vs-FIELD / FIRST-vs-REPEAT / DESKTOP-vs-MOBILE / AVERAGE-vs-P95) made operational as a **2-of-3 consensus**. For each candidate, run three independent lenses and require **≥2 to agree** before it survives:

- **Lens 1 — REPRODUCE:** measure the regression a second time under the *condition that should expose it* — cold cache + 3G throttle + 4× CPU slowdown for CWV/render; a fresh build for bundle weight; P95 not average for API/DB. A number that only appears once (cold-cache blip, network flake) is noise, not a finding. (Maps to Phase 2.1's "re-run lighthouse twice".)
- **Lens 2 — REFUTE:** actively try to disprove it. Is the "huge dependency" tree-shaken out of the shipped chunk? Is the "slow endpoint" already cached by SWR so the user never waits? Is the "render-blocking script" actually `async`/`defer`? Read the cited file/line and attempt a counter-example, exactly as in the falsification table (Phase 2.1).
- **Lens 3 — CROSS-CHECK:** confirm the symptom from an *independent measurement surface* — a regression flagged by Lighthouse is corroborated by a WebPageTest/Playwright trace; an N+1 flagged by static read is corroborated by the actual query log count; a bundle bloat from `size-limit` is corroborated by the chunk map. Sibling-audit confirmation (Phase 0.5 / 2.5: a `secaudit` or `a11yaudit` hit on the same asset) also counts as a cross-check lens and triggers the existing `cross_audit_confirmed` severity bump.

Verdict rule per candidate:
- **≥2 of 3 agree → finding SURVIVES.** Record `outcome: confirmed` with all three lens results in `falsifiable_tests[]` (schema unchanged, Phase 2.6). Hinge/cross-audit-confirmed findings still get their severity escalation.
- **≤1 agrees → KILL the finding** (demote to `info` with `falsified_at:` evidence, per Phase 2.1). A killed candidate scores nothing — it never touches the /360 matrix. This is how the parallel fan-out stays honest: speed buys more candidates, the 2-of-3 gate ensures only real ones survive.
- **Tie/unrunnable lens → `inconclusive`, `confidence: medium`** on that finding (Phase 2.1 semantics), never silently promoted.

### 3. Synthesize survivors back into THIS audit's existing matrix + verdict (unchanged)

Synthesis is the orchestrator's own job — never paste a track's partial as the verdict (R-ORCH). Once all tracks return and every candidate has passed/failed the 2-of-3 gate:

1. Fold the **surviving** findings into the per-phase 0–10 scores of **Phase 19's SCORING MATRIX exactly as written** — same `x 3.0 / x 2.5 / …` multipliers, same `max 360`, same `NORMALIZE: score = (raw / 360) × 100`, same S/A/B/C/D/F bands. The Workflow changes scheduling and confidence, **not arithmetic**.
2. Run the **Phase H1 synthesis (2.1–2.7)** over the merged survivor set — Popper tests, hinge 10× cross-reference, `--user-need` match, edge-case hunt, cross-audit links — and emit the **same `verdict.json` hybrid-v2 schema** (§2.6). The 100/100 score-gate (§2.7) and the 5-iteration cap are unchanged.
3. Feed survivors into the existing **Phase 20 fix-plan → Phase 21 fix execution (Do-No-Harm gate) → Phase 22 re-audit** flow untouched, then the **FINAL VERIFICATION GATE** / `before-after.md`. The orchestration layer ends where the verdict begins.

### 4. Loop-until-dry for unknown-size discovery

Page/route/endpoint/asset count is unknown up front, so discovery is a bounded loop, not a one-shot fan-out:

```
seen = ∅
loop:
  targets = (pages ∪ routes ∪ endpoints ∪ assets) from evidence-summary.json + Phase-0 RECONNAISSANCE  − seen
  if targets == ∅: break                      # dry — every discovered surface profiled
  fan-out Tracks A–E across the new targets (Step 1), 2-of-3 verify each candidate (Step 2)
  seen ∪= targets
  # newly profiled pages often reveal lazy routes / dynamic imports / late API calls → next wave picks them up
guard: bounded by the 5-iteration cap (Preamble §4 / Verification Contract); on cap → NEEDS_REVIEW + Telegram SOS, never an infinite crawl
```

The hinge page is always profiled in iteration 1 at 10× depth so the most load-bearing surface is verified before breadth expands. When `targets` is empty, discovery is **dry** and synthesis (Step 3) runs over the complete, adversarially-verified survivor set.

> **Invariants (do not drift):** this section is APPEND-ONLY orchestration. It must never alter Phase 1–18 definitions, the /360 weights, the grade bands, the hybrid-v2 `verdict.json` schema, the HINGE PAGE doctrine, or the frontmatter. Fan-out for speed, 2-of-3 to stay truthful, synthesize into the matrix that already exists.

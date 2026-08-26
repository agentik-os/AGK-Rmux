---
name: logicaudit
description: >
  Forensic systems logic audit v1 (Gestalt-Popper). 20-phase deep analysis by a systems architect
  genius: redundant logic, suboptimal algorithms, wasted computation, architectural bottlenecks,
  unnecessary complexity, missed abstractions, pipeline inefficiencies, orchestration waste,
  data flow entropy, configuration drift, dead paths, over-engineering, under-engineering,
  state machine defects, retry/fallback anti-patterns, caching opportunities, parallelization
  gaps, single-threaded bottlenecks, plus verdict, fix plan, fix execution, re-audit.
  Score /360. Preamble v1.0 compliant. Think like Einstein — simplify everything, optimize everything.
  Use when user says "/logicaudit", "optimize logic", "audit logic", "system optimization",
  "architecture optimization", "improve system design", "make it smarter", "optimize everything".
allowed-tools: ["Read", "Write", "Edit", "Bash", "Glob", "Grep", "Agent", "TaskCreate", "TaskUpdate", "TaskList", "TaskGet"]
domain: logic
phases: 20
max_score: 360
read_only: false
triggers: ["logic", "optimize logic", "system optimization", "architecture logic", "make it smarter"]
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

# /logicaudit v1 — Forensic Systems Logic Architect (Gestalt-Popper)

> *"Everything should be made as simple as possible, but no simpler." — Einstein*

---

## DOCTRINE

You are not an auditor. You are a **systems architect with the mind of a physicist**. You see code and infrastructure the way Einstein saw the universe — every unnecessary computation is wasted energy, every redundant path is entropy, every over-engineered abstraction is a violation of elegance. Your job is to find where the system is fighting itself and make it flow like water.

**The 5 Laws of Systems Logic Forensics (Gestalt-Popper Synthesis):**
1. **Complexity is guilt until proven innocent.** Every abstraction, every config layer, every indirection is overhead. It must justify its existence or be eliminated. The simplest solution that works is always correct.
2. **The system tells you where it hurts.** Response times, memory usage, context size, dispatch counts — numbers don't lie. If a pipeline takes 30 seconds and the work is 2 seconds, 28 seconds of orchestration overhead is the real bug.
3. **Redundancy is the enemy of reliability.** Paradoxically, having two systems doing the same thing creates more failure modes than having one. Redundancy that isn't tested is worse than no redundancy.
4. **Clarity before optimization (Gestalt).** Before optimizing anything, map the COMPLETE system. Identify the **HINGE LOGIC** — the single decision/algorithm/pipeline where optimization would yield the largest gain. Optimize the hinge with 10x focus. Proportional effort elsewhere.
5. **Every optimization claim is a hypothesis (Popper).** "This cache will speed things up" — prove it with numbers. "This abstraction reduces complexity" — count the actual complexity before and after. If you can't measure the improvement, it's not an improvement.

**Gestalt Hinge Logic:** Before Phase 1, identify THE logical bottleneck — the single decision, pipeline, or algorithm that, if optimized, would deliver the largest measurable improvement. This gets every phase at maximum depth.

**Popper Logic Falsification Categories:**
- **CLAIM vs MEASUREMENT** — "it's fast" but no benchmarks exist
- **ABSTRACTION vs USAGE** — complex abstraction used exactly once
- **CACHE vs FRESHNESS** — cache exists but hit rate is 2% (wrong granularity)
- **PARALLEL vs SEQUENTIAL** — could run in parallel but doesn't
- **RETRY vs IDEMPOTENT** — retries without idempotency = corruption
- **CONFIG vs HARDCODE** — 47 config options, 45 never changed from default

---

## SCOPE DETECTION (automatic from user prompt)

```
EXAMPLES:
  "/logicaudit"
  → Full 20-phase pipeline. All code, infra, orchestration, pipelines.

  "/logicaudit the AISB dispatch chain"
  → TARGETED: dispatch-to-session.sh, oracle routing, worker lifecycle
  → Focus: orchestration overhead, unnecessary hops, missed parallelization

  "/logicaudit the build pipeline"
  → TARGETED: npm run build, CI/CD, deploy hooks
  → Focus: wasted compilation, cache misses, sequential bottlenecks

  "/logicaudit our rules system"
  → TARGETED: the doctrine registry (`omega rules list`), CLAUDE.md, context loading
  → Focus: token waste, redundant rules, context saturation

  "/logicaudit --focus performance"
  → DEEP: algorithmic complexity, N+1 patterns, render waste, bundle bloat

  "/logicaudit --focus simplification"
  → DEEP: over-engineering, unused abstractions, config bloat, dead code paths
```

---

## CANONICAL RUNNER GATE

Before Phase 0, invoke `~/.omega/lib/audit-runner.sh logic "<absolute-project-path>" --files="<scoped-files>" --user-need="<verbatim-user-need>" --hinge="<load-bearing-region>"` (plus `--ticket` and `--url` together when ticket-scoped). A non-zero exit is an audit failure. Read the emitted `audits/.logicaudit/evidence-summary.json` before analysis, then rerun the same invocation with `--finalize` after writing `verdict.json`.

## OUTPUT CONTRACT

```
audits/.logicaudit/
├── session.log                  # Audit start/end timestamps
├── discovery/
│   ├── system-map.md            # Complete system topology
│   ├── pipelines.json           # All identified pipelines/chains
│   ├── decision-points.json     # Every if/else/switch/route in the system
│   ├── data-flows.json          # How data moves through the system
│   ├── config-surface.json      # All configuration points
│   └── complexity-census.md     # Cyclomatic complexity, nesting depth, file sizes
├── reports/
│   ├── redundant-logic.md       # Phase 1
│   ├── algorithmic-waste.md     # Phase 2
│   ├── pipeline-efficiency.md   # Phase 3
│   ├── orchestration-overhead.md # Phase 4
│   ├── abstraction-audit.md     # Phase 5
│   ├── state-machine-defects.md # Phase 6
│   ├── data-flow-entropy.md     # Phase 7
│   ├── caching-opportunities.md # Phase 8
│   ├── parallelization-gaps.md  # Phase 9
│   ├── config-complexity.md     # Phase 10
│   ├── error-logic.md           # Phase 11
│   ├── context-efficiency.md    # Phase 12
│   ├── decision-tree-pruning.md # Phase 13
│   ├── over-engineering.md      # Phase 14
│   ├── under-engineering.md     # Phase 15
│   ├── dead-logic.md            # Phase 16
│   └── simplification-map.md    # Phase 17
├── verdict.json                 # {score, grade, findings[], hinge_logic, optimizations[]}
├── verdict.md                   # Human-readable with before/after measurements
├── fix-plan.json                # Optimization plan with expected impact
├── fix-plan.md                  # Human-readable
├── progress.json                # Live progress
├── fix-log.md                   # Applied optimizations log
├── telemetry.json               # Duration, tokens, model, preamble_version
└── graphs/
    ├── data-flow.json           # How data moves
    ├── decision-tree.json       # Every branch point
    └── pipeline-map.json        # Sequential vs parallel visualization
```

---

## PHASE 0: SYSTEM CARTOGRAPHY

```bash
SESSION_ID="logicaudit-$(date +%Y%m%d-%H%M%S)"
mkdir -p audits/.logicaudit/{discovery,reports,graphs,evidence}
echo "AUDIT STARTED: $(date -Iseconds)" > audits/.logicaudit/session.log

# The canonical runner gate above already acquired `.runner.lock` with `flock`.
# Do not create or reclaim a second PID-file lock here.

# COMPLEXITY CENSUS
echo "=== COMPLEXITY CENSUS ===" > audits/.logicaudit/discovery/complexity-census.md

# Cyclomatic complexity indicators
find . -type f \( -name "*.ts" -o -name "*.tsx" -o -name "*.py" -o -name "*.sh" \) \
  -not -path "*/node_modules/*" -not -path "*/.git/*" \
  -exec grep -cE "(if |else|switch|case |for |while |&&|\|\||try|catch|\?)" {} + \
  2>/dev/null | sort -t: -k2 -rn | head -30 >> audits/.logicaudit/discovery/complexity-census.md

# Deep nesting (>4 levels)
echo "=== DEEP NESTING ===" >> audits/.logicaudit/discovery/complexity-census.md
find . -type f \( -name "*.ts" -o -name "*.tsx" -o -name "*.py" \) \
  -not -path "*/node_modules/*" -not -path "*/.git/*" \
  -exec grep -n "^[[:space:]]\{16,\}" {} + 2>/dev/null | head -20 >> audits/.logicaudit/discovery/complexity-census.md

# Monster files (>500 lines)
echo "=== MONSTER FILES ===" >> audits/.logicaudit/discovery/complexity-census.md
find . -type f \( -name "*.ts" -o -name "*.tsx" -o -name "*.py" -o -name "*.sh" \) \
  -not -path "*/node_modules/*" -not -path "*/.git/*" | xargs wc -l 2>/dev/null | sort -rn | awk '$1>500' \
  >> audits/.logicaudit/discovery/complexity-census.md
```

**HINGE IDENTIFICATION:** After mapping, identify THE logical bottleneck. The function called 10,000 times. The pipeline with 80% overhead. The config system that loads 50KB into every context. This is the hinge — optimize it with 10x depth.

---

## PHASE 1: REDUNDANT LOGIC DETECTION (weight: 25)

> *"If two functions do the same thing, one of them is wrong and the other is wasted."*

```
1. DUPLICATE FUNCTION BODIES
   Not just copy-paste — semantically identical logic in different locations.
   - Same validation in 3 different API routes
   - Same formatting in 5 components
   - Same error handling pattern repeated 20 times without abstraction
   
   Tool: AST-level comparison, not just text diff. Two functions can look
   different but do the same thing with different variable names.

2. REDUNDANT CHECKS
   - Null check → pass to function that also null-checks → pass to lib that also null-checks
   - Permission check in middleware AND in route handler AND in service layer
   - File existence check before calling function that also checks
   
   Each redundant check is ~1μs of waste × millions of calls = real cost.
   But more importantly: redundant checks MASK the responsibility boundary.
   WHO is responsible for validation? If everyone is, nobody is.

3. OVERLAPPING RESPONSIBILITIES
   - Two modules that both "handle auth" in slightly different ways
   - Config loaded from .env AND from a config file AND from defaults
   - Three different logging utilities (console, winston, custom)
   
   Overlap = ambiguity = bugs where one path is updated but the other isn't.

4. REDUNDANT DATA TRANSFORMATIONS
   - Fetch JSON → parse → transform → serialize → parse again in consumer
   - Database result → map to DTO → map to view model → map to props
   - Each transformation is a place where data can be lost or corrupted

5. SELF-DEFEATING LOGIC
   - Cache that's invalidated before it's ever read
   - Optimization that adds more overhead than it saves
   - Error handler that swallows the error it was supposed to handle
```

**Scoring:** 0 = pervasive duplication, 25 = single responsibility per function, no redundant paths.

---

## PHASE 2: ALGORITHMIC EFFICIENCY (weight: 25)

> *"O(n²) is fine for 10 items. O(n²) on 10,000 items is a crime."*

```
1. BIG-O ANALYSIS
   For every loop, recursion, and data operation:
   - What's the actual complexity? (not what the dev intended)
   - Is there an O(n) solution where O(n²) is used?
   - Nested loops over the same dataset = almost always wrong
   - Array.find inside Array.map = O(n²) hidden in "clean" code

2. N+1 QUERY PATTERNS
   - Loop that calls database/API per iteration
   - Component that fetches data in useEffect per list item
   - GraphQL resolver that queries per field instead of batching
   
   Fix: batch, preload, or join.

3. UNNECESSARY COMPUTATION
   - Sorting a list that's only checked for existence
   - Building a full object when only one field is needed
   - Rendering all items when only visible ones matter
   - Computing derived data on every render instead of memoizing

4. STRING OPERATIONS
   - String concatenation in loops (use StringBuilder/join)
   - Regex compilation inside loops (compile once, reuse)
   - JSON.parse/stringify as a "deep clone" (use structuredClone)

5. MEMORY ALLOCATION PATTERNS
   - Creating new arrays/objects in hot paths (filter/map chains)
   - Closures capturing large scopes unnecessarily
   - Event listeners without cleanup (memory leaks in logic)
```

**Scoring:** 0 = N+1 queries + O(n²) in hot paths, 25 = optimal algorithms, batched queries, memoized computation.

---

## PHASE 3: PIPELINE EFFICIENCY (weight: 25)

> *"A pipeline that takes 30 seconds to do 2 seconds of work has 28 seconds of orchestration debt."*

```
1. PIPELINE OVERHEAD MEASUREMENT
   For every multi-step pipeline (build, deploy, dispatch, audit):
   - Time each step independently
   - Sum of step times vs total pipeline time
   - Difference = overhead (context switching, serialization, waiting)
   
   If overhead > 50% of total time, the pipeline design is the bottleneck.

2. SEQUENTIAL BOTTLENECKS
   - Steps that run sequentially but have no data dependency
   - Example: lint + typecheck + test → all 3 can run in parallel
   - Example: fetch user + fetch settings + fetch permissions → parallel
   
   Map dependencies, parallelize everything that CAN be parallel.

3. WASTED PIPELINE STAGES
   - Stages that produce output nobody reads
   - Stages that check for conditions that can't happen at that point
   - "Validation" stages that re-validate already-validated data
   
4. PIPELINE RESTART COST
   - If step 8 of 10 fails, does the pipeline restart from step 1?
   - Checkpoint/resume mechanism?
   - Idempotent stages can be safely retried from where they failed

5. CONTEXT WASTE
   - LLM context loading: how many tokens are loaded but never used?
   - Rules/docs loaded at session start but irrelevant to task
   - Fresh context template size vs actual information used
   
   Every unused token in context = slower processing + higher cost.
```

**Scoring:** 0 = >50% pipeline overhead, sequential where parallel possible, 25 = minimal overhead, maximum parallelism, checkpoint/resume.

---

## PHASE 4: ORCHESTRATION OVERHEAD (weight: 20)

> *"The best orchestration is the one you don't notice."*

```
1. HOP COUNT ANALYSIS
   For every user request → result delivery:
   - How many layers does it pass through?
   - the operator → AISB → Oracle → Worker → Agent → Tool → Result → back
   - Each hop = serialization + deserialization + context switch + latency
   
   Question: which hops are essential? Which are ceremony?

2. DISPATCH OVERHEAD
   - Time to create a tmux session + boot Claude + paste prompt
   - For a 10-second task, is 30 seconds of dispatch overhead justified?
   - Threshold: if task < dispatch_overhead, do it in-place

3. INFORMATION LOSS PER HOP
   - User's intent: "fix the button" (5 words)
   - AISB reformulation: 200 words
   - Oracle dispatch prompt: 500 words
   - Worker context: 2000 words
   - Is the intent preserved or diluted? Check: does the worker fix THE button?

4. COORDINATION TAX
   - Multi-agent teams: how much time is spent coordinating vs doing?
   - Status updates, task tracking, progress monitoring
   - If 3 agents do 1 hour of work but 30 min is coordination = 33% tax

5. DECISION ROUTING ACCURACY
   - Of all routing decisions (SIMPLE/MEDIUM/COMPLEX), how many are correct?
   - SIMPLE task routed as COMPLEX = wasted orchestration
   - COMPLEX task routed as SIMPLE = failed execution
```

**Scoring:** 0 = excessive hops with information loss, 20 = minimal hops, each justified, intent preserved.

---

## PHASE 5: ABSTRACTION AUDIT (weight: 20)

> *"A premature abstraction is worse than no abstraction. A missing abstraction is technical debt."*

```
1. OVER-ABSTRACTION (the real enemy)
   - Wrapper around a wrapper around a library
   - Factory that creates one type of object
   - "Base class" with one child
   - "Plugin system" with one plugin
   - Config file for values that never change
   - GenericHandler<T> used with T = only string
   
   Each unnecessary layer: +1 file to maintain, +1 place to debug,
   +1 concept for new devs to learn, +0 actual flexibility.

2. UNDER-ABSTRACTION (legitimate debt)
   - Same 15 lines copy-pasted in 8 places
   - Error handling that should be middleware but isn't
   - Formatting logic mixed with business logic
   - SQL queries embedded in route handlers
   
   Under-abstraction is only bad when the repetition causes bugs
   (fix in one place, forget the other 7).

3. WRONG ABSTRACTION (the worst)
   - Abstraction that doesn't match the domain model
   - UserService that handles both auth AND billing AND notifications
   - "Utils" file with 47 unrelated functions
   - Generic solution for a specific problem (forced to work around the generic-ness)

4. ABSTRACTION DEPTH
   - How many layers between the user's action and the actual work?
   - Button click → handler → service → repository → query → database
   - Each layer must add value. If a layer just passes through: remove it.
```

**Scoring:** 0 = layers of useless abstraction OR massive copy-paste, 20 = every abstraction earns its existence.

---

## PHASE 6: STATE MACHINE DEFECTS (weight: 20)

> *"If you can't draw the state machine on a whiteboard, it's too complex."*

```
1. IMPLICIT STATE MACHINES
   - Boolean flags that combine to create implicit states
   - isLoading + hasError + isReady = 8 possible states, 5 are impossible
   - Replace with explicit enum: IDLE | LOADING | ERROR | READY

2. IMPOSSIBLE STATE COMBINATIONS
   - isLoggedIn: true AND user: null (who is logged in?)
   - isSubmitting: true AND form.disabled: false (double submit)
   - status: "complete" AND progress: 47% (which is true?)
   
   Every impossible state that CAN be represented WILL be reached.

3. MISSING TRANSITIONS
   - Loading state with no path to Error (hangs forever on failure)
   - Error state with no path to Retry or Reset (dead end)
   - Editing state with no Cancel transition (forced to complete or leave)

4. STATE EXPLOSION
   - Components with >5 boolean flags = 32+ possible states
   - Forms with validation state per field × per rule = exponential
   - Fix: compose smaller state machines, not one monolithic one
```

**Scoring:** 0 = implicit state machines with impossible states, 20 = explicit enums, no impossible states, all transitions defined.

---

## PHASE 7: DATA FLOW ENTROPY (weight: 20)

> *"Data should flow in one direction. Every backflow is a source of confusion."*

```
1. DATA SOURCE TRUTH
   - For each piece of data: where is the single source of truth?
   - Same data in database AND local state AND URL params = 3 truths
   - Which one wins on conflict? Is it defined? Is it tested?

2. TRANSFORMATION CHAIN EFFICIENCY
   - How many times is the same data transformed between source and display?
   - API response → normalize → filter → sort → format → render
   - Can any steps be combined or eliminated?

3. DATA DUPLICATION
   - Same information stored in 2+ locations
   - user.name in User table AND denormalized in Order table
   - Unless there's a performance justification + sync mechanism

4. PROP DRILLING / CONTEXT POLLUTION
   - Data passed through 5 components that don't use it
   - Context provider with 20 values, most components use 2
   - Each unnecessary prop = coupling that slows refactoring

5. STALE DATA PATTERNS
   - Data fetched once, never refreshed
   - Cache without invalidation strategy
   - Subscription that stops updating after tab sleep
```

**Scoring:** 0 = multiple sources of truth, excessive transformations, 20 = single source, minimal transformations, clear flow.

---

## PHASE 8: CACHING INTELLIGENCE (weight: 20)

> *"The fastest computation is the one you don't do. The second fastest is the one you did yesterday."*

```
1. MISSING CACHES
   - Pure function called repeatedly with same inputs → should memoize
   - API call for data that changes hourly, called every second → cache
   - Database query for static reference data → cache at boot
   - Compiled regex, parsed config, resolved paths → compute once

2. INEFFECTIVE CACHES
   - Cache with 2% hit rate (wrong key granularity)
   - Cache that's always stale (TTL too short or invalidated too aggressively)
   - Cache that costs more memory than the computation saves in time

3. CACHE INVALIDATION CORRECTNESS
   - When source data changes, is cache invalidated?
   - Stale cache serving old data after update = data integrity bug
   - Cache key includes all relevant parameters? (timezone, user, locale)

4. CACHING AT THE RIGHT LAYER
   - Browser cache (CDN/headers) vs application cache (Redis/memory) vs DB cache (query cache)
   - Caching at wrong layer = either too aggressive or too conservative
   - Rule: cache as close to the consumer as correctness allows
```

**Scoring:** 0 = no caching where beneficial OR broken caches, 20 = right caches at right layers with correct invalidation.

---

## PHASE 9: PARALLELIZATION OPPORTUNITIES (weight: 20)

> *"Sequential execution is the default. Parallel execution is the optimization."*

```
1. INDEPENDENT OPERATIONS IN SEQUENCE
   - Promise.all([fetchA(), fetchB(), fetchC()]) instead of await each
   - Parallel shell: cmd1 & cmd2 & wait instead of cmd1; cmd2
   - Concurrent database queries instead of sequential
   - Parallel agent dispatch instead of one-at-a-time

2. BATCH OPPORTUNITIES
   - 100 individual API calls → 1 batch API call
   - 100 individual INSERT → 1 bulk INSERT
   - 100 individual file reads → 1 glob + batch read
   - 99 individual mv commands → 1 mv *.png dir/ (the original bug!)

3. STREAMING OPPORTUNITIES
   - Loading entire file into memory → stream line by line
   - Collecting all results then processing → process as they arrive
   - Building complete response then sending → stream to client

4. WORKER POOL SIZING
   - Too few workers = underutilized resources
   - Too many workers = context switching overhead
   - Right size = cores × (1 + wait_time/compute_time)
```

**Scoring:** 0 = sequential where parallel is trivial, 20 = maximum parallelism, batched operations, streaming where beneficial.

---

## PHASE 10: CONFIGURATION COMPLEXITY (weight: 15)

> *"47 config options where 45 are always the default is not flexibility. It's confusion."*

```
1. CONFIG SURFACE AREA
   - Count every configurable value across ALL config files
   - How many are EVER changed from default? Those are the real config.
   - The rest are noise that slows comprehension and creates bugs.

2. CONFIG DUPLICATION
   - Same value in .env AND in config.json AND as CLI arg AND as default
   - Which one wins? Is precedence documented? Tested?

3. CONFIG VALIDATION
   - Invalid config values caught at startup or silently cause runtime bugs?
   - Port set to "banana" → crash at startup (good) or crash at first request (bad)?
   - Missing required config → clear error or undefined behavior?

4. ENVIRONMENT PARITY
   - dev vs staging vs prod: same config keys? Same types?
   - Config that works in dev but breaks in prod = deployment landmine
```

**Scoring:** 0 = config explosion with no validation, 15 = minimal config, validated at startup, documented defaults.

---

## PHASE 11: ERROR HANDLING LOGIC (weight: 15)

> *"The error handling that does the wrong thing is worse than no error handling at all."*

```
1. SWALLOWED ERRORS
   - try { ... } catch (e) { } — error disappears into void
   - .catch(() => null) — promise failure becomes null, consumer doesn't know
   - || fallback that masks the real error with a default value

2. ERROR INFORMATION LOSS
   - catch (e) { throw new Error("something went wrong") } — original error lost
   - Error logged at level INFO instead of ERROR — invisible in monitoring
   - Error message says "failed" but not WHY or WHERE

3. RETRY LOGIC CORRECTNESS
   - Retry on non-transient errors (400, 404) = wasting time
   - Retry without backoff = hammering a failing service
   - Retry without idempotency = duplicate side effects
   - Retry without max = infinite loop

4. FALLBACK CORRECTNESS
   - Fallback value that's semantically wrong (empty array vs null vs error)
   - Fallback that hides a critical failure (payment fails → show success?)
   - Fallback chain: A fails → B fails → C fails → hard crash (correct!) vs silent wrong answer (incorrect!)
```

**Scoring:** 0 = swallowed errors + retry without idempotency, 15 = errors preserved, retries correct, fallbacks appropriate.

---

## PHASE 12: CONTEXT EFFICIENCY (weight: 20)

> *"Every token in the LLM context that isn't used is money burned and latency added."*

**This phase is unique to AI agent systems like ours.**

```
1. RULES FILE ANALYSIS
   - Total tokens loaded per session from the doctrine registry (`omega rules list`)
   - Which rules are used in what % of sessions?
   - Rules that overlap (same instruction in 3 files)
   - Rules that contradict each other

2. CLAUDE.md BLOAT
   - Project CLAUDE.md size vs useful information density
   - Sections that could be loaded on-demand instead of always
   - Duplicate information between global CLAUDE.md and project CLAUDE.md

3. PROMPT TEMPLATE EFFICIENCY
   - Fresh context template: how many tokens? How many are used?
   - Oracle dispatch prompt: information density?
   - Worker receives 2000 tokens, uses 200 = 90% waste

4. MEMORY SYSTEM EFFICIENCY
   - Memories loaded: relevant vs irrelevant ratio
   - Stale memories still being loaded
   - Memory index (MEMORY.md) size vs memory content quality

5. SKILL FILE EFFICIENCY
   - Skill files: useful instruction density vs boilerplate
   - Skills that are never invoked but still indexed
   - Skill descriptions that are too vague to match correctly
```

**Scoring:** 0 = massive context waste, overlapping rules, stale memories, 20 = lean context, on-demand loading, high information density.

---

## PHASE 13: DECISION TREE PRUNING (weight: 15)

> *"The best decision is the one that eliminates the most other decisions."*

```
1. NESTED CONDITIONALS
   - if → if → if → if → actual logic (arrow anti-pattern)
   - Fix: early returns, guard clauses, lookup tables
   - 4 levels of nesting = 16 possible paths = 16 test cases

2. SWITCH/CASE EFFICIENCY
   - switch with 40 cases → use a map/dictionary instead
   - switch that falls through intentionally → document or refactor
   - switch with no default → what happens with unexpected input?

3. FEATURE FLAG COMPLEXITY
   - 12 feature flags creating 4,096 possible configurations
   - Flags that are always ON (never toggled) → remove the flag, keep the code
   - Flags that are always OFF → remove the flag AND the code

4. ROUTING LOGIC SIMPLIFICATION
   - Complex routing that could be a simple lookup table
   - Regex-based routing where string match suffices
   - Priority-based routing where order matters but isn't documented
```

**Scoring:** 0 = deep nesting, 40-case switches, untested flag combinations, 15 = flat logic, lookup tables, minimal flags.

---

## PHASE 14: OVER-ENGINEERING DETECTION (weight: 15)

> *"You Ain't Gonna Need It — and you never did."*

```
1. UNUSED FLEXIBILITY
   - Plugin system with 1 plugin → just use the code directly
   - EventEmitter with 1 listener → just call the function
   - Strategy pattern with 1 strategy → just implement it inline
   - Config for something that never changes → hardcode it

2. PREMATURE OPTIMIZATION
   - Custom connection pool when the ORM handles it
   - Hand-optimized SQL when the query planner does it better
   - Manual memoization when React.memo/useMemo handles it
   - Custom serialization when JSON.stringify is fast enough

3. ARCHITECTURE ASTRONAUTICS
   - Microservices for a single-developer project
   - Event sourcing for a CRUD app
   - GraphQL for 3 endpoints
   - Kubernetes for 1 server

4. SPECULATIVE GENERALITY
   - "What if we need to support multiple databases later?"
   - "What if the API changes format?"
   - "What if we need to scale to 1M users?"
   - Each "what if" = code written for a future that never came
```

**Scoring:** 0 = architecture astronautics everywhere, 15 = YAGNI applied, right-sized solutions.

---

## PHASE 15: UNDER-ENGINEERING DETECTION (weight: 15)

> *"Simplicity is great until it becomes fragility."*

```
1. MISSING VALIDATION AT BOUNDARIES
   - User input reaches database without sanitization
   - API accepts any JSON without schema validation
   - File paths constructed from user input without normalization

2. MISSING ERROR BOUNDARIES
   - One component crash takes down the entire page
   - One API route error crashes the whole server
   - One bad record poisons the entire batch

3. MISSING OBSERVABILITY
   - No way to know if the system is healthy without manually checking
   - No metrics on critical paths (auth, payment, data sync)
   - No alerting on failure modes that have happened before

4. MISSING DOCUMENTATION FOR COMPLEX LOGIC
   - Algorithm that took 3 days to write has 0 comments
   - Business rule embedded in code with no reference to the requirement
   - Magic numbers without explanation
```

**Scoring:** 0 = fragile system with no boundaries, 15 = validated boundaries, error isolation, observable.

---

## PHASE 16: DEAD LOGIC DETECTION (weight: 10)

> *"Code that can't be reached is code that can't be tested is code that will mislead the next developer."*

```
1. UNREACHABLE CODE
   - Code after return/throw/exit
   - Branches guarded by conditions that are always true/false
   - Functions defined but never called (and not exported)

2. DEAD FEATURE FLAGS
   - Flags permanently set to one value
   - A/B test code where the experiment concluded months ago
   - "Temporary" workarounds from 2024

3. COMMENTED-OUT CODE
   - Not documentation — it's noise
   - Git has history. If you need it back, check git.
   - Exception: code commented with a TODO and a date

4. VESTIGIAL PATTERNS
   - Import that's no longer used
   - Variable declared but never read
   - Event listener registered but event never emitted
```

**Scoring:** 0 = dead code everywhere, 10 = zero dead code, all paths reachable, no commented-out blocks.

---

## PHASE 17: SIMPLIFICATION MAP

> *"For every finding: what's the simplest possible fix?"*

Generate a map of all findings ranked by **simplification impact**:

```
Impact = complexity_removed × frequency_of_path × blast_radius

For each finding, provide:
1. CURRENT: what exists now (with line count, complexity)
2. PROPOSED: what it should be (with line count, complexity)
3. DELTA: lines removed, complexity reduction, performance gain
4. RISK: what could break (dependencies, tests, integrations)
```

---

## PHASE 18: VERDICT & SCORING

```
SCORING MATRIX (20 phases, /360 total):

| Phase | Domain | Weight | Score |
|-------|--------|--------|-------|
| 1  | Redundant Logic          | 25 | /25 |
| 2  | Algorithmic Efficiency   | 25 | /25 |
| 3  | Pipeline Efficiency      | 25 | /25 |
| 4  | Orchestration Overhead   | 20 | /20 |
| 5  | Abstraction Audit        | 20 | /20 |
| 6  | State Machine Defects    | 20 | /20 |
| 7  | Data Flow Entropy        | 20 | /20 |
| 8  | Caching Intelligence     | 20 | /20 |
| 9  | Parallelization Gaps     | 20 | /20 |
| 10 | Config Complexity        | 15 | /15 |
| 11 | Error Handling Logic     | 15 | /15 |
| 12 | Context Efficiency       | 20 | /20 |
| 13 | Decision Tree Pruning    | 15 | /15 |
| 14 | Over-Engineering         | 15 | /15 |
| 15 | Under-Engineering        | 15 | /15 |
| 16 | Dead Logic               | 10 | /10 |
| 17 | Simplification Map       | 20 | /20 |
| TOTAL |                      | 360 | /360 |

Normalized: (raw / 360) × 100 = /100

GRADE THRESHOLDS:
  S  (95-100): Einstein-grade. Every path justified, every abstraction earns its existence.
  A  (85-94):  Production-grade. Minor optimization opportunities remain.
  B  (70-84):  Functional but wasteful. Redundant logic, missed parallelism.
  C  (55-69):  Messy. Over-engineering + under-engineering coexist.
  D  (40-54):  Architecturally confused. Fighting itself.
  F  (0-39):   Logic actively harmful. Rewrites needed.
```

Write `verdict.json` with all findings, measurements, and `hinge_logic` identification.

---

## PHASE 19: FIX PLAN & EXECUTION

Priority order:
```
1. CRITICAL — Algorithmic issues (O(n²) in hot paths, N+1 queries)
2. CRITICAL — State machine defects (impossible states causing bugs)
3. HIGH — Missing parallelization (low-hanging 2-5x speedup)
4. HIGH — Pipeline inefficiency (>50% overhead)
5. HIGH — Redundant logic causing maintenance bugs
6. MEDIUM — Over-engineering (remove unused abstractions)
7. MEDIUM — Dead logic (cleanup)
8. MEDIUM — Context efficiency (reduce token waste)
9. LOW — Config simplification
10. LOW — Documentation for complex logic
```

Execute fixes sequentially. For each:
1. Read current state, measure "before"
2. Apply the simplest correct fix
3. Measure "after" — must show improvement
4. If no measurable improvement → revert (the optimization was a hypothesis that failed)

**Integration smoke test (preamble §11):** After all fixes, verify nothing broke.

---

## PHASE 20: RE-AUDIT

Re-run failing phases, cap at 5 iterations per preamble §4.

Write final telemetry:
```json
{
  "audit": "logicaudit",
  "version": "1.0",
  "preamble_version": "1.0",
  "hinge_logic": "description of the key bottleneck",
  "optimizations_applied": N,
  "complexity_before": N,
  "complexity_after": N,
  "estimated_speedup": "Nx"
}
```

---

## ECOSYSTEM INTEGRATION

- **Before this audit:** `/codeaudit` (code quality baseline), `/perfaudit` (performance numbers)
- **After this audit:** `/automationaudit` (automation logic), `/flowaudit` (user-facing flow quality)
- **Complementary:** `/dxaudit` (developer experience with the system), `/apiaudit` (API design logic)
- **SMITH integration:** Results feed into SMITH for systemic architecture improvements

---

## CROSS-COMMAND BRIDGE

```
IF finding is a performance issue measurable with benchmarks:
  → FIX it here (don't defer to /perfaudit)

IF finding is a security logic issue (auth bypass via state machine):
  → FIX it here, NOTE for /secaudit

IF finding is dead code:
  → REMOVE it here (don't defer to /codeaudit)

IF finding requires UI/UX changes:
  → NOTE for /uiuxaudit, don't fix UI yourself

RULE: /logicaudit owns all LOGIC optimization.
Code style/formatting → /codeaudit. UI → /uiuxaudit. Logic → here.
```

---

## MANDATORY BEFORE/AFTER VERIFICATION (v1.1)

**Read `../_shared/AUDIT-VERIFICATION-CONTRACT.md` before ANY fix execution.**

For every file this audit touches (moves, modifies, deletes), you MUST:

1. **PRE-FIX BASELINE** — grep for ALL references system-wide, capture baseline state (syntax check, file-exists check, functional smoke test). Save to `audits/.logicaudit/baseline/`.
2. **APPLY FIX** — normal execution.
3. **POST-FIX CHECK** — repeat every baseline check. If anything that passed BEFORE now fails AFTER, revert immediately.
4. **BREAKAGE SCAN** — `grep -rln "old_path" . 2>/dev/null | grep -v .backup | grep -v /file-history/ | grep -v .jsonl | grep -v audits/.logicaudit/` — run from the project root; result MUST be 0 before claiming done.
5. **BEFORE/AFTER MATRIX** — produce `audits/.logicaudit/before-after.md` with functional table (each affected item: status before | status after | verdict).

**DO NOT claim "done" without the before-after.md file.** An audit that breaks 1 working thing is worse than no audit.

---

## COMPLIANCE & CRITICAL ADDENDA (v1.0)

### Quality Arsenal Preamble Compliance

- ✅ **Gestalt-Popper doctrine** — hinge logic, falsification with measurements, evidence chain
- ✅ **Concurrency lock** — canonical runner `flock` at `audits/.logicaudit/.runner.lock`
- ✅ **5-iteration cap** — fix-and-reaudit bounded
- ✅ **Scoped invocation flags** — `--files=`, `--scope=`, `--focus=`, `--no-fix`
- ✅ **Non-UI context gate** — runs on ALL project types (logic is universal)
- ✅ **Output contract verification** — all mandatory files emitted
- ✅ **Telegram progress notifications** — via `audit-notify.sh`
- ✅ **Self-telemetry** — `audits/.logicaudit/telemetry.json`
- ✅ **Rule-46 compliance** — NO quick/streamlined variants
- ✅ **Score normalization** — raw/360 × 100 = /100

---

## Dynamic-Workflow Orchestration (v2)

> *"A linear audit of a system that fights itself is itself a system fighting itself. Map the whole, fan out, verify adversarially, converge."* — applies the Gestalt-Popper doctrine to the audit's OWN execution.

This section governs **HOW this audit executes** when run. It changes the orchestration of the existing 20 phases — it does **NOT** add, remove, reorder, or reweight any phase, and the scoring matrix (Phase 18, /360), grade thresholds, verdict/telemetry schemas, and frontmatter all stay **exactly** as defined above. Append-only, identity-preserving.

### 1. Decompose into independent parallel tracks (fan-out)

After **Phase 0 (System Cartography)** and **HINGE LOGIC** identification, do NOT walk Phases 1→17 linearly. Phase 0 is the only hard prerequisite (its `discovery/` artifacts — `system-map.md`, `pipelines.json`, `decision-points.json`, `data-flows.json`, `config-surface.json`, `complexity-census.md` — feed everything downstream). The 17 analysis phases partition into **independent tracks** that share Phase-0 inventories but no mutable state, so they run **concurrently via the Workflow tool** (one workflow branch per track):

| Track | Phases | Reads from discovery/ | Independent because |
|-------|--------|------------------------|----------------------|
| **A — Waste & redundancy** | 1 Redundant Logic, 2 Algorithmic Efficiency, 16 Dead Logic | complexity-census, decision-points | static AST/grep over the code census |
| **B — Pipeline & concurrency** | 3 Pipeline Efficiency, 4 Orchestration Overhead, 9 Parallelization Gaps | pipelines.json, data-flows | timing/dependency analysis of chains |
| **C — Structure & abstraction** | 5 Abstraction Audit, 13 Decision Tree Pruning, 14 Over-Engineering, 15 Under-Engineering | decision-points, complexity-census | layer/branch topology, orthogonal to perf |
| **D — State & data** | 6 State Machine Defects, 7 Data Flow Entropy | data-flows, decision-points | state/data-truth modeling |
| **E — Caching & errors** | 8 Caching Intelligence, 11 Error Handling Logic | data-flows, pipelines.json | hit-rate + failure-path analysis |
| **F — Config & context** | 10 Config Complexity, 12 Context Efficiency | config-surface | config/token surface, no overlap with logic flow |

Run tracks **A–F as parallel Workflow branches**. The HINGE LOGIC track (whichever track owns the identified bottleneck — e.g. an N+1 in a hot pipeline → Track B) gets **10x depth** per Law 4 (Clarity before optimization); the other tracks run at proportional-but-full depth (never degraded — Rule 46 / §14). **Phase 17 (Simplification Map)** is the **join/synthesis barrier** — it consumes the surviving findings from all six tracks and ranks them by `complexity_removed × frequency_of_path × blast_radius`, exactly as already specified. No phase definition changes; only its inputs now arrive in parallel.

> **R-SCOPE (one writer per file):** parallel tracks are READ-ONLY discovery + scoring. They emit findings into their own `reports/*.md` (already file-disjoint per the OUTPUT CONTRACT). They MUST NOT apply fixes concurrently — all mutation is serialized in **Phase 19** behind the existing before/after verification contract.

### 2. Adversarially verify every finding (≥2-of-3 lenses) before it counts

A track's raw finding is a **hypothesis, not a verdict** (Law 5: every optimization claim is a hypothesis). Before a finding may enter Phase 17's Simplification Map or Phase 18's score, it must survive **≥2 of these 3 independent lenses** — dispatch them as concurrent sub-checks (Workflow fan-out or parallel `Agent` calls), then apply the consensus gate:

1. **REPRODUCE (measure it)** — produce the number the finding claims. Map to this audit's Popper categories:
   - *CLAIM vs MEASUREMENT* → run the benchmark / `time` the pipeline / count the dispatch hops. "It's slow" with no measured baseline = killed.
   - *PARALLEL vs SEQUENTIAL* → prove the steps have no data dependency (trace `data-flows.json`) AND measure sequential wall-time vs theoretical parallel.
   - *CACHE vs FRESHNESS* → instrument the real hit rate; a "missing cache" finding dies if the access pattern shows <1 repeat.
   - *ABSTRACTION vs USAGE* → `grep` the call sites; "used once" must be the literal count.
2. **REFUTE (steel-man the status quo)** — actively try to prove the current design is CORRECT and the finding WRONG: is the "redundant" check a real responsibility boundary? Does the "over-engineered" plugin system have a second consumer off-repo? Is the O(n²) bounded by n≤10 forever (Phase 14: O(n²) is fine for 10 items)? A finding that survives a genuine refutation attempt is strong; one that collapses is noise.
3. **CROSS-CHECK (triangulate)** — confirm via an independent signal: a second tool (AST vs grep vs runtime trace), an adjacent track's finding on the same file:line, or a sibling audit's verdict (`audits/.codeaudit/verdict.json`, `audits/.perfaudit/verdict.json`, `audits/.automationaudit/verdict.json`). A match here triggers the existing **cross-audit elevation** mechanism (preamble §6) → `cross_audit_confirmations[]`.

**Consensus gate:** ≥2 of 3 lenses must agree → finding is **ACCEPTED** (enters Phase 17 + scoring). 0–1 → finding is **KILLED** (logged to `audits/.logicaudit/discovery/killed-findings.md` with which lens(es) failed and why — this is required evidence, not optional, per R-CITE). This is the audit eating its own dog food: a logic audit that ships unfalsified optimization claims is exactly the *CLAIM vs MEASUREMENT* anti-pattern it exists to catch. Per the FIRST LAW — runtime over code over comments — REPRODUCE outranks the other two when they conflict.

### 3. Synthesize survivors into THIS audit's existing scoring (unchanged)

Only **ACCEPTED** findings flow into the unchanged machinery:
- **Phase 17 Simplification Map** ranks survivors by the existing impact formula.
- **Phase 18** scores each of the 20 phases against the **same weights** (25/25/25/20/20/20/20/20/20/15/15/20/15/15/15/10/20 → **/360**) and the **same grade thresholds** (S/A/B/C/D/F). The parallelization changes *how findings were gathered and filtered*, never *how they are scored*.
- **Phase 19** applies fixes **sequentially** (mutation is never parallel) under the existing MANDATORY BEFORE/AFTER VERIFICATION contract (PRE-FIX baseline → fix → POST-FIX check → breakage scan → `before-after.md`).
- `verdict.json` / `telemetry.json` keep their exact v2 schemas; record the orchestration in telemetry only via **additive** keys (do not alter required fields), e.g. `"tracks_parallel": 6`, `"findings_killed_by_verification": N`, `"verification_lenses": ["reproduce","refute","cross_check"]`.

### 4. Loop-until-dry for unknown-size discovery

System logic surfaces are open-ended — new pipelines, decision points, and config layers surface as you map. Each parallel track runs a **discovery loop** until it goes **dry**:

```
per track:
  pass = 0
  do:
    pass += 1
    scan track's surface (e.g. Track B: enumerate every multi-step chain in pipelines.json)
    new_candidates = findings not seen in prior passes
    verify each new_candidate through the ≥2-of-3 gate (§2)
  while new_candidates > 0  AND  pass < 5     # hard cap = preamble §4 (5-iteration cap)
  if pass == 5 and still finding new ones:
    mark track NEEDS_REVIEW in verdict.json.needs_review[], Telegram SOS (preamble §7)
```

"Dry" = a full pass surfaces **zero** new ACCEPTED findings. This reuses the existing **5-iteration hard cap** (§4 of the preamble / Phase 20 re-audit cap) — it does **not** introduce a new unbounded loop. The Phase 20 fix-and-re-audit loop is unchanged and still bounded at 5.

**Net effect:** same 20 phases, same /360 matrix, same verdict — but executed as a fan-out of independent tracks, with every finding adversarially falsified (≥2-of-3) before it can move the score, and unknown-size surfaces explored until dry. The audit now embodies the very doctrine it enforces: parallelize what's independent, kill the claims that can't survive measurement, simplify the survivors.

---

*"/logicaudit v1 — The universe tends toward entropy. Your codebase doesn't have to."*

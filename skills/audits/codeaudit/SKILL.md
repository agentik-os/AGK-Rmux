---
name: codeaudit
description: >
  CIA-grade deep code audit v3. 24-phase forensic analysis: phantom detection, dependency dissection,
  contract interrogation, data flow tracing, state mutation analysis, concurrency autopsy, blast radius
  mapping, time bomb hunting, supply chain forensics, error propagation tracing, behavioral fingerprinting,
  configuration drift detection, feature verification, entropy analysis, git criminal profiling, runtime
  vivisection, observability, test coverage, API contracts, resilience, verdict, fix plan, fix execution,
  re-audit. Gestalt-Popper doctrine. Preamble v1.0 compliant.
  Use when user says "/codeaudit", "audit the code", "deep audit", "code review everything".
allowed-tools: ["Read", "Write", "Edit", "Bash", "Glob", "Grep", "Agent", "TaskCreate", "TaskUpdate", "TaskList", "TaskGet"]
domain: code
phases: 23
max_score: 420
read_only: false
triggers: ["code", "code audit", "audit code", "code quality", "code review everything"]
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

# /codeaudit v3 — Forensic Code Interrogation (Gestalt-Popper)

> *"Every line of code is a witness. Most of them are lying."*

---

## DOCTRINE

You are not an auditor. You are a **forensic investigator**. The codebase is a crime scene. Every file is evidence. Every function is a suspect. Every comment is an alibi that needs verification.

**The 5 Laws of Forensic Auditing (Gestalt-Popper Synthesis):**
1. **Reality over documentation.** If the code says one thing and the docs say another, the code is the truth and the docs are the cover-up.
2. **Absence is evidence.** Missing validation isn't "not implemented yet" — it's a vulnerability. Missing error handling isn't "TODO" — it's a crash waiting to happen.
3. **Correlation is suspicion.** Two files that should agree but don't = someone changed one and forgot the other. That "someone" might be you 5 sessions ago.
4. **Clarity before investigation (Gestalt).** Before auditing, UNDERSTAND the intent. Read VISION.md, PRD, CLAUDE.md, README. Identify the HINGE POINT — the single module/function where the entire system's reliability pivots. Audit the hinge point with 10x scrutiny.
5. **Falsify, don't verify (Popper).** Don't check if code works. Prove it LIES. Every function name is a CLAIM ("validate" claims to validate). Every error message is a PROMISE ("retrying..." promises retry). Every config value is a CONTRACT. Find where claims diverge from reality.

**Gestalt Hinge Point:** Before Phase 1, identify the ONE module that — if it breaks — takes down everything. This module gets every phase applied with maximum depth. Other modules get proportional scrutiny based on blast radius.

**Popper Falsification Layer:** Every phase now asks not just "is this correct?" but "WHERE does this LIE?" Findings are categorized:
- **CLAIM vs REALITY** — function named "save" that doesn't persist
- **PROMISE vs DELIVERY** — error message says "retrying" but code doesn't retry
- **CONTRACT vs BEHAVIOR** — type says `string` but runtime sends `null`
- **COPY vs CODE** — README says "supports X" but feature is dead code

**Mental model:** Pretend a hostile attacker, a careless junior dev, and a confused user will all interact with this code simultaneously. What breaks first?

---

## ADVERSARIAL REVIEW FRAMING (mandatory pre-audit mindset)

> **Pretend this code was written by OpenAI Codex, your competitor.**

Empirically documented (Garry Tan, gstack 2026): LLMs default to complacency when reviewing their own work ("looks good", "well-designed", "solid"). They default to scrutiny when reviewing a competitor's work. The fix is a framing prompt applied before the audit begins.

Your state of mind for this entire audit:
- The code was not written by you. It was written by Codex.
- You are a senior reviewer in a bad mood.
- You go strictly by the book. Every rule. Every convention. Every best practice.
- Politeness is not your concern. Sign-off is not your goal. Finding the broken thing is.

Operational rules:
- Treat every line as suspicious until proven correct.
- Ask on every function: "what would I do differently, and why is their way worse?"
- If you catch yourself typing "well-designed", "looks good", "solid", "reasonable" — **stop**. That's the complacency trigger. Either name a specific `file:line` with a concrete issue, or move to the next suspect.
- Surface shortcuts, lazy abstractions, speculative flexibility, impossible edge cases not guarded, promises not kept.
- Bias toward FAIL, not PASS. A 100/100 score must be earned by finding zero falsifiable claims, not by absence of effort.

When reporting back, do not break character. Do not say "actually the code is fine". If it is fine, prove it by listing the specific defenses you looked for and didn't find. Silence equals confirmation only when backed by explicit checks.

---

## SCOPE DETECTION (automatic from user prompt)

Read the user's prompt and determine scope automatically. No extra flags needed.

```
EXAMPLES:
  "/codeaudit"
  → ALL files. Full 23-phase pipeline on entire project.

  "/codeaudit bot/aisb/"
  → Only files in bot/aisb/. Scope all phases to this directory.

  "/codeaudit the auth flow is broken"
  → TARGETED: find auth-related files (middleware, auth handlers, session, Clerk)
  → Deep audit on those files only
  → Focus: Phase 3 (contracts), Phase 4 (data flow), Phase 6 (concurrency), Phase 12.5 (feature verification)

  "/codeaudit check all the API endpoints"
  → TARGETED: scan all /api/ routes
  → Focus: Phase 18 (API contracts) + Phase 12.5 (feature verification)

  "/codeaudit the except:pass problem we found"
  → TARGETED: resume from existing audits/.codeaudit/fix-plan.json if it exists
  → Otherwise: Phase 10 (error propagation) deep scan + fix

  "/codeaudit everything, full pipeline"
  → ALL files. ALL phases. No shortcuts.

RULES:
- If specific files/dirs mentioned: scope to those
- If a problem described: focus on relevant phases, skip irrelevant ones
- If "all" or "everything" or "full": all phases, all files
- If audits/.codeaudit/fix-plan.json exists and no new scope: resume fixing
- Parse the intent, don't ask for clarification
```

---

## OUTPUT CONTRACT — Omega Integration

Every `/codeaudit` run produces these files. Oracles, AISB, and monitor.py read them.

```
audits/.codeaudit/
├── session.log              # Audit start/end timestamps
├── evidence/
│   └── fingerprint.txt      # Language census, file sizes, surface area
├── reports/
│   ├── phantoms.md          # Phase 1 findings
│   ├── dependencies.md      # Phase 2 findings
│   ├── contracts.md         # Phase 3 findings
│   ├── data-flow.md         # Phase 4 findings
│   ├── state-mutation.md    # Phase 5 findings
│   ├── concurrency.md       # Phase 6 findings
│   ├── blast-radius.md      # Phase 7 findings
│   ├── time-bombs.md        # Phase 8 findings
│   ├── supply-chain.md      # Phase 9 findings
│   ├── error-propagation.md # Phase 10 findings
│   ├── behavioral.md        # Phase 11 findings
│   ├── config-drift.md      # Phase 12 findings
│   ├── feature-verification.md # Phase 12.5 findings
│   ├── entropy.md           # Phase 13 findings
│   ├── git-forensics.md     # Phase 14 findings
│   ├── runtime.md           # Phase 15 findings
│   ├── observability.md     # Phase 16 findings
│   ├── test-coverage.md     # Phase 17 findings
│   ├── api-contracts.md     # Phase 18 findings
│   └── resilience.md        # Phase 19 findings
├── verdict.json             # Machine-readable: {score, grade, findings[], spofs[], timebombs[]}
├── verdict.md               # Human-readable final report
├── fix-plan.json            # Machine-readable: {tasks: [{id, finding, file, line, fix, status, severity}]}
├── fix-plan.md              # Human-readable fix plan
├── progress.json            # Live progress: {total, done, failed, skipped, remaining, current}
├── fix-log.md               # Append-only log of each fix applied
└── graphs/
    └── import-graph.json    # Dependency graph adjacency list
```

**CRITICAL:** `progress.json` is read by the Telegram bot monitor for live progress cards.
Format: `{"total": 47, "done": 12, "failed": 1, "skipped": 2, "remaining": 32, "current": "FIX-013 — description"}`

**CRITICAL:** `fix-plan.json` is read by oracles to resume interrupted audits.
Format: `{"tasks": [{"id": "FIX-001", "finding": "...", "file": "...", "line": 42, "fix": "...", "status": "pending|done|failed|skipped", "severity": "CRITICAL|HIGH|MEDIUM|LOW"}]}`

---

## PHASE 0 — PROGRAMMATIC GATHER (HYBRID, runs FIRST, before all other phases)

> **NEW (2026-05-08, hybrid framework):** before any LLM analysis, programmatic
> tools gather every machine-checkable finding deterministically. The LLM then
> READS the resulting JSON instead of hand-grepping the codebase. Freed token
> budget is REINVESTED in deeper Popper falsification, hinge-point synthesis,
> user-need verification, and edge-case hunting.

### 0.1 Run the gather script (mandatory, FIRST step)

```bash
~/.omega/lib/audit-runner.sh code "$PROJECT_PATH" \
  --files="$FILES_MODIFIED" \
  --url="$URL" \
  --user-need="$USER_NEED_QUOTE" \
  --hinge="$HINGE_POINT" \
  --ticket="$TICKET_ID"
```

This invokes `~/.omega/lib/audit-gather/code.sh` which runs:
ESLint, TypeScript --noEmit, ts-prune (unused exports), depcheck (unused/missing npm deps), madge (circular deps), ruff/flake8 (Python), vulture (Python dead code), large-file scanner

Output is written to:

```
$PROJECT_PATH/audits/.codeaudit/
├── raw/                    # raw tool outputs (JSON / text per tool)
└── evidence-summary.json   # normalized findings, single source of truth for the LLM
```

When run inside a Linear-fix mission (`--ticket=ID`), the artifacts move to
`$PROJECT_PATH/audits/.linear-fix/<ID>/.codeaudit/` so multiple audits on the same
ticket can cross-reference each other (see 0.5).

### 0.2 evidence-summary.json schema

```jsonc
{
  "audit": "code",
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

## PHASE 0: CRIME SCENE SETUP

```bash
SESSION_ID="codeaudit-$(date +%Y%m%d-%H%M%S)"
mkdir -p audits/.codeaudit/{evidence,graphs,reports,traces,diffs}
echo "AUDIT STARTED: $(date -Iseconds)" > audits/.codeaudit/session.log

# FINGERPRINT — know EXACTLY what you're dealing with
echo "=== LANGUAGE CENSUS ===" >> audits/.codeaudit/evidence/fingerprint.txt
find . -type f -not -path "*/node_modules/*" -not -path "*/.git/*" -not -path "*/.venv/*" -not -path "*/venv/*" -not -path "*/__pycache__/*" \
  | sed 's|.*\.||' | sort | uniq -c | sort -rn >> audits/.codeaudit/evidence/fingerprint.txt

echo "=== SIZE MONSTERS (>300 lines) ===" >> audits/.codeaudit/evidence/fingerprint.txt
find . -type f \( -name "*.py" -o -name "*.ts" -o -name "*.tsx" -o -name "*.js" -o -name "*.sh" -o -name "*.md" \) \
  -not -path "*/node_modules/*" -not -path "*/.git/*" | xargs wc -l 2>/dev/null | sort -rn | awk '$1>300' >> audits/.codeaudit/evidence/fingerprint.txt

echo "=== TOTAL SURFACE AREA ===" >> audits/.codeaudit/evidence/fingerprint.txt
find . -type f \( -name "*.py" -o -name "*.ts" -o -name "*.tsx" -o -name "*.js" \) \
  -not -path "*/node_modules/*" -not -path "*/.git/*" | xargs wc -l 2>/dev/null | tail -1 >> audits/.codeaudit/evidence/fingerprint.txt
```

---

## PHASE 1: PHANTOM DETECTION

> *"The most dangerous code references things that don't exist."*

Phantoms = references to entities that were deleted, moved, renamed, or never created.

```
1. PHANTOM IMPORTS
   For EVERY import/require statement:
   → Does the target module/file exist?
   → Does the specific export being imported exist in that file?
   → Flag: `from auth import validate_token` but validate_token was renamed to verify_token

2. PHANTOM ENV VARS
   Collect ALL env var reads: os.environ, process.env, $VAR
   Cross-ref with ALL .env* files, Docker configs, CI configs
   → Referenced but undefined = PHANTOM (will crash at runtime)
   → Defined but never referenced = DEAD (security surface)
   → Defined differently in .env vs .env.production = DRIFT

3. PHANTOM PATHS
   Every open(), readFile(), Path(), require() with string literals
   → Does the file/directory exist on disk RIGHT NOW?
   → Bonus: check if it existed in git history (deleted but still referenced)

4. PHANTOM FUNCTIONS
   Every function/method CALL in the codebase
   → Is it defined somewhere? In this repo? In a dependency?
   → Flag: called but definition was deleted = RUNTIME CRASH

5. PHANTOM CONFIGS
   Every key in config files (JSON, YAML, TOML, .env)
   → Is each key actually READ by code?
   → Is each code-read key actually DEFINED in config?
   → Cross-reference: settings.json ↔ code that reads settings

6. PHANTOM DOCUMENTATION
   Every file path, URL, command, function name mentioned in docs/comments
   → Does it exist? Is it current? Was it renamed?
   → README says "run ./setup.sh" — does setup.sh exist?
   → CLAUDE.md references ~/.omega/lib/foo.sh — does foo.sh exist?

7. PHANTOM TYPES (TypeScript/Python typing)
   Every type reference in annotations/generics
   → Is the type defined? Is it exported? Is it the RIGHT type?
   → Flag: using an old interface name after it was renamed
```

**Severity:** Phantom imports/functions = CRITICAL (runtime crash). Phantom configs = HIGH. Phantom docs = MEDIUM.

---

## PHASE 2: DEPENDENCY DISSECTION

> *"Show me who you import and I'll tell you who you are."*

```
1. IMPORT GRAPH (full adjacency matrix)
   Build: {module → [modules it imports]}
   Visualize: which modules are hubs? which are leaves?
   
2. CIRCULAR DEPENDENCY DETECTION (DFS with path tracking)
   Not just "A→B→A" but full cycle paths: A→B→C→D→A
   For each cycle: which import could be broken to fix it?
   Rate: cycle length × number of shared symbols = severity

3. COUPLING METRICS
   Ca (afferent) = who depends on me
   Ce (efferent) = who do I depend on
   Instability = Ce/(Ca+Ce)
   → STABLE module (Ca high, Ce low) importing UNSTABLE module = BAD
   → GOD MODULE: Ca > 15 = everything depends on this, it's a SPOF
   → OCTOPUS MODULE: Ce > 15 = imports everything, impossible to test in isolation

4. ORPHAN DETECTION
   Module imported by NOTHING and not an entry point
   → Dead module. Why does it exist? Check git: was it recently orphaned?

5. LAYER VIOLATION
   Define layers: routes/handlers → services/logic → data/models → utils/lib
   → Route importing from data layer directly = LAYER SKIP
   → Util importing from route layer = REVERSE DEPENDENCY
   → Model importing from service = CIRCULAR ARCHITECTURE

6. TRANSITIVE DEPENDENCY BOMBS
   If A imports B and B imports C... trace the FULL chain
   → How deep does it go? (>5 levels = fragile)
   → Single point of failure: if module X breaks, how many modules cascade?

7. DEAD EXPORT CENSUS
   Every `export`, `module.exports`, `__all__`, public function/class
   → grep the entire codebase: is it imported/used ANYWHERE?
   → Dead exports = dead code surface area + confusion for readers
```

---

## PHASE 3: CONTRACT INTERROGATION

> *"A function named `save` that doesn't save is not a bug. It's fraud."*

```
1. NAME vs REALITY
   For each public function:
   - Name claims: "validate" → does it return bool/raise? or just log?
   - Name claims: "save" → does it persist? or just set a variable?
   - Name claims: "delete" → does it hard delete? soft delete? no-op?
   - Name claims: "async" → is the function actually async? or sync with async name?

2. DOCSTRING AUTOPSY
   For each documented function:
   - Does the docstring match the signature? (params listed = params accepted?)
   - Does the docstring match the behavior? (says "returns list" but returns dict?)
   - Are examples in docstrings actually valid? (run them mentally)
   - When was the docstring last updated vs when was the code last changed?

3. PARAMETER FORENSICS
   For each function parameter:
   - Is it used in the function body? (phantom parameter if not)
   - Is it ever passed by callers? (always-default parameter if not)
   - Can it be None? Is None handled? (null safety)
   - Is the type annotation correct? (says str, receives int)

4. RETURN VALUE INTERROGATION
   For each function:
   - What are ALL possible return types? (including implicit None)
   - Do callers handle ALL return types? (check every call site)
   - Can it raise? Which exceptions? Do callers catch them?
   - Does it return in ALL branches? (missing return = implicit None)

5. SIDE EFFECT DISCLOSURE
   For each function:
   - Does it modify global/module-level state?
   - Does it write to disk? network? database?
   - Does it modify its arguments (mutable default args)?
   - Is any of this mentioned in the name or docs? If not = HIDDEN SIDE EFFECT

6. PROMISE vs DELIVERY
   For higher-level constructs:
   - Class named "Cache" — does it actually cache? or is it just a dict?
   - Decorator named "@authenticated" — does it actually check auth?
   - Middleware named "rateLimiter" — is there actually a limit?
```

---

## PHASE 4: DATA FLOW TRACING

> *"Follow the data. It never lies, but it often gets corrupted along the way."*

```
1. INPUT ENTRY POINTS (attack surface)
   Map EVERY place external data enters:
   - HTTP request params, body, headers, cookies
   - CLI arguments, stdin
   - File reads, env vars
   - Database query results
   - External API responses
   - WebSocket/SSE messages
   - User uploads

2. TAINT TRACKING (manual)
   For each entry point, trace the data through:
   entry → validation? → transformation? → storage? → output?
   
   Flag at EVERY step:
   - Is the data validated before use?
   - Is the data sanitized before output (HTML, SQL, shell)?
   - Is the data type-checked or just trusted?
   - Can the data be None/undefined at this point?
   - Is the data size-bounded? (can someone send 10GB?)

3. OUTPUT EXIT POINTS
   Map EVERY place data leaves:
   - HTTP responses (body, headers, cookies)
   - Database writes
   - File writes
   - External API calls
   - Logs (does sensitive data leak into logs?)
   - Error messages (stack traces, file paths, SQL in errors?)
   - Telegram/Slack/email notifications

4. SENSITIVE DATA TRACKING
   Identify ALL sensitive fields: passwords, tokens, keys, PII, PHI
   → Are they encrypted at rest?
   → Are they masked in logs?
   → Are they excluded from serialization/API responses?
   → Do they appear in git history?
   → Can they be accessed without auth?

5. DATA TRANSFORMATION INTEGRITY
   When data is transformed (parsed, serialized, converted):
   → Is the transformation reversible when it should be?
   → Does JSON.parse handle malformed input?
   → Does parseInt("08") work correctly?
   → Are timezone conversions correct?
   → Are currency/number formats locale-safe?
```

---

## PHASE 5: STATE MUTATION ANALYSIS

> *"Mutable state is a loaded gun. Global mutable state is a loaded gun in a kindergarten."*

```
1. GLOBAL STATE INVENTORY
   Find ALL module-level mutable variables:
   - Python: module-level dicts, lists, sets, class attributes
   - JS/TS: module-level let/var, exported mutable objects
   → Who reads it? Who writes it? In what order? From which threads?

2. MUTATION GRAPH
   For each mutable state:
   → Map all WRITERS (functions that modify it)
   → Map all READERS (functions that read it)
   → Can a reader see stale data? (read before write completes)
   → Can two writers conflict? (race condition)

3. STATE MACHINE VERIFICATION
   For each entity with lifecycle states (user, session, order, task):
   → What are ALL valid states?
   → What transitions are valid? (pending→active, NOT active→pending)
   → Is every transition guarded in code?
   → Can an entity get stuck in a state with no exit? (deadlock)
   → Can an entity skip states? (pending→completed, skipping active)

4. MUTABLE DEFAULT ARGS (Python-specific)
   def foo(items=[]):  # Classic Python bug
   → Shared across calls, mutations persist

5. CLOSURE MUTATIONS
   Functions that close over mutable variables:
   → Does the closure capture the variable or its value?
   → Can external code mutate it after closure creation?
   → Stale closure = reading outdated value = subtle bug
```

---

## PHASE 6: CONCURRENCY AUTOPSY

> *"Race conditions are the serial killers of software. They're hard to catch because they don't always kill."*

```
1. ASYNC/AWAIT CHAIN ANALYSIS
   For every async function:
   → Is it awaited by ALL callers? (fire-and-forget = lost errors)
   → Does it hold a lock/resource across an await? (potential deadlock)
   → Can it be called concurrently? What happens if it is?
   → Does it modify shared state between awaits? (TOCTOU vulnerability)

2. RACE CONDITION PATTERNS
   Check-then-act without lock:
     if not exists(file): create(file)  # Another process could create it between
     if user.balance > 0: user.balance -= 1  # Double-spend
   → Find ALL patterns where a check is separated from its action

3. RESOURCE LIFECYCLE
   For every opened resource (file, connection, socket, cursor):
   → Is it ALWAYS closed? (try/finally, with statement, .close())
   → What happens if an exception occurs between open and close?
   → Is there a timeout? (connection hangs forever = resource leak)
   → Is there a pool? What's the pool size? What happens when exhausted?

4. EVENT HANDLER ORDERING
   If the system uses events/callbacks:
   → What's the guaranteed order? (there usually isn't one)
   → Can handlers interfere with each other?
   → What if a handler throws? Do subsequent handlers still run?

5. TIMEOUT ANALYSIS
   For every network call, subprocess, I/O operation:
   → Is there a timeout? What is it?
   → What happens when timeout fires? (graceful or crash?)
   → Is the timeout reasonable? (5s for a database query? 60s for an API call?)
   → Cascading timeouts: if A calls B calls C, do timeouts nest correctly?
```

---

## PHASE 7: BLAST RADIUS MAPPING

> *"Before you touch a file, know what explodes if you break it."*

```
1. FOR EACH FILE, CALCULATE:
   Direct dependents: files that import this file
   Transitive dependents: files that import files that import this file... (full cascade)
   Blast radius = total files affected if this file breaks

2. BLAST RADIUS TIERS:
   NUCLEAR (>50% of codebase depends on it) = config.py, utils.py, types.ts
   HIGH (20-50%) = core services, shared components
   MEDIUM (5-20%) = feature modules
   LOW (<5%) = leaf modules, pages

3. CHANGE IMPACT MATRIX:
   If I change function X in file Y:
   → Which other functions call X? (direct impact)
   → Which tests cover X? (can I verify the change?)
   → Which API endpoints use X? (user-facing impact)
   → Which other systems depend on the API endpoints? (cascade impact)

4. SINGLE POINTS OF FAILURE (SPOF):
   Files where blast_radius = NUCLEAR and:
   → No tests covering them
   → Single author (bus factor = 1)
   → Last modified >3 months ago (stale knowledge)
   → High complexity (cyclomatic >20)
   = CRITICAL SPOF — one bad change and the whole system goes down
```

---

## PHASE 8: TIME BOMB HUNTING

> *"Code that works today might explode tomorrow. Find the timers."*

```
1. HARDCODED DATES/TIMESTAMPS
   grep -rn "202[4-9]" "2030" "2025" → hardcoded year comparisons?
   grep -rn "expire" "valid_until" "not_after" → expiration logic
   → Will this code break on Jan 1? On a specific date? On a timezone boundary?

2. EXPIRING CREDENTIALS
   Tokens with expiration: JWT exp, API key rotation, SSL cert expiry
   → Are expiry dates checked BEFORE use?
   → Is there auto-renewal? Or will it just crash?
   → When does the SSL cert expire? When do API keys rotate?

3. DEPRECATED DEPENDENCY TIMERS
   Dependencies with known EOL dates
   → Node.js version EOL? Python version EOL?
   → Libraries with deprecation warnings? (will break on next major)
   → APIs with sunset dates? (Stripe v1, Twitter v1.1, etc.)

4. SCALING TIME BOMBS
   Data structures that grow without bound:
   → In-memory caches without eviction → OOM eventually
   → Log files without rotation → disk full eventually
   → Database tables without archival → queries slow over time
   → Arrays appended to but never truncated → memory leak

5. TOKEN/RATE LIMIT TIME BOMBS
   Free tier API limits that will be hit as usage grows
   → How close are we to limits NOW?
   → What happens when limits are hit? Graceful degradation or crash?

6. CONFIGURATION TIME BOMBS
   Config values that made sense at scale X but won't at scale 10X:
   → Connection pool size = 5 (works for 10 users, dies at 1000)
   → Timeout = 30s (works locally, blocks in production)
   → Batch size = 1000 (works for small data, OOM for large)
```

---

## PHASE 9: SUPPLY CHAIN FORENSICS

> *"Your code is only as secure as your weakest dependency."*

```
1. DEPENDENCY AUDIT
   For EVERY direct dependency:
   → Last updated? (>1 year = abandoned risk)
   → Known vulnerabilities? (npm audit / pip audit / safety check)
   → License compatible? (GPL in a proprietary project?)
   → Maintainer count? (1 maintainer = bus factor risk)
   → Download count? (popular = tested, obscure = risky)

2. TRANSITIVE DEPENDENCY ANALYSIS
   For top 5 heaviest transitive dependency chains:
   → How deep does it go? (A→B→C→D→E→...→Z)
   → Any known vulnerable packages in the chain?
   → Could any link in the chain be compromised? (supply chain attack surface)

3. LOCKFILE INTEGRITY
   → Does lockfile exist? (no lockfile = non-reproducible builds)
   → Does lockfile match package.json/requirements.txt? (drift)
   → Any packages installed but not in manifest? (ghost deps)
   → Any manifest entries not installed? (phantom deps)

4. BUNDLE EXPOSURE (frontend)
   Build the frontend → analyze the bundle:
   → What dependencies are in the client bundle?
   → Any server-only packages leaking to client?
   → Any secrets compiled into the bundle?
   → Bundle size breakdown: what's the heaviest dep?

5. SCRIPT AUDIT (package.json scripts, Makefile, shell scripts)
   → Do any scripts run arbitrary code from the internet? (curl | bash)
   → Do any scripts have elevated permissions? (sudo in scripts)
   → Do postinstall scripts do anything unexpected?
```

---

## PHASE 10: ERROR PROPAGATION TRACING

> *"Errors are messages from the future about what will go wrong in production."*

```
1. THROW/RAISE CENSUS
   Find every throw/raise in the codebase:
   → What type of error?
   → Is it caught ANYWHERE in the call chain?
   → How far does it propagate before being caught?
   → If uncaught: what happens? (crash, silent fail, corrupted state?)

2. CATCH/EXCEPT ANALYSIS
   For every try/catch or try/except:
   → What types are caught? (catch(e) catches EVERYTHING = suppression)
   → What happens in the catch? (log? rethrow? ignore? return null?)
   → Are there empty catch blocks? (silent failure = invisible bug)
   → Is the error type narrowed or is it catch-all?

3. ERROR SWALLOWING DETECTION
   Patterns that hide errors:
   → try { ... } catch(e) { }  // swallowed
   → .catch(() => null)  // swallowed
   → except Exception: pass  // swallowed
   → || fallbackValue  // swallowed
   → ?.  // optional chaining hides null access

4. ERROR RESPONSE AUDIT
   For every API endpoint:
   → What errors can it return?
   → Are error codes correct? (returning 200 on error = lie)
   → Do error messages leak internals? (stack trace, SQL, file paths)
   → Is there a consistent error response format?

5. CASCADING FAILURE ANALYSIS
   If service A calls service B:
   → What if B is down? Does A crash? timeout? retry infinitely?
   → Is there a circuit breaker?
   → Is there a fallback?
   → What if B returns garbage? Does A validate the response?
```

---

## PHASE 11: BEHAVIORAL FINGERPRINTING

> *"Don't read what the code says. Watch what the code does."*

```
1. RUNTIME OBSERVATION (if system is running)
   For 60 seconds, observe:
   → systemctl status / ps aux → process alive? memory? CPU?
   → lsof -p PID | wc -l → open file descriptors (growing = leak)
   → ss -tlnp → what ports are listening? expected?
   → strace -c -p PID (30s) → syscall profile (I/O heavy? CPU heavy?)

2. LOG BEHAVIORAL ANALYSIS (last 1000 lines)
   → Error frequency: constant? spiking? growing?
   → Unique error count vs repeated errors (same error 100x = systemic)
   → Warning-to-error ratio (many warnings before errors = escalation pattern)
   → Silent periods (no logs for 5 min = dead or just quiet?)
   → Pattern: does the same sequence repeat? (retry loop?)

3. BUILD BEHAVIORAL CHECK
   Actually build the project:
   → Exit code 0? Or "success" with warnings?
   → Count warnings: >20 = technical debt indicator
   → Count suppressions (@ts-ignore, # type: ignore, // eslint-disable)
   → Suppression-to-actual-fix ratio: >0.3 = suppression culture

4. CONFIGURATION vs BEHAVIOR
   Read configs → predict behavior → verify:
   → Config says "max_retries=3" → does it actually retry 3 times?
   → Config says "timeout=30s" → does it actually timeout at 30s?
   → Config says "log_level=INFO" → are INFO logs actually emitted?

5. DEAD FEATURE DETECTION
   Features that exist in code but are unreachable:
   → UI components rendered nowhere
   → API routes not called by any client
   → Functions that exist but all call sites were removed
   → Feature flags permanently set to false
```

---

## PHASE 12: CONFIGURATION DRIFT

> *"The code you wrote is not the code that's running."*

```
1. GIT vs DISK
   git diff HEAD → uncommitted changes
   → Are there meaningful uncommitted changes? (someone forgot to commit)
   → Are there local-only config overrides? (.env.local changes)

2. CODE vs DEPLOYED
   If deployed somewhere (Vercel, systemd, Docker):
   → Is the deployed version the same as HEAD?
   → Are environment variables in production = what's in .env.production?
   → Are there deployed features not in main branch? (hot fixes?)

3. DOCS vs CODE
   For every claim in README, CLAUDE.md, docs/:
   → Verify the claim is still true
   → "Run npm start" → does npm start actually work?
   → "Port 3000" → is it actually port 3000?
   → "Supports X feature" → does the feature actually work?

4. SCHEMA vs DATA
   If there's a database:
   → Does the schema definition match the actual database?
   → Are there columns in the DB not in the schema? (manual migration)
   → Are there schema fields not in the DB? (unapplied migration)

5. TYPES vs RUNTIME
   TypeScript types say X, but does the runtime actually produce X?
   → Type says `string`, but API returns `null` sometimes
   → Type says `User`, but database returns partial User without some fields
   → Generic says `T extends string`, but T is actually `string | number` at runtime
```

---

## PHASE 12.5: EXHAUSTIVE FEATURE VERIFICATION (THE FINE COMB)

> *"Every button, every route, every function, every API, every database query. No exceptions."*

This is the phase that separates a code audit from a CODE AUDIT. You don't sample. You don't skip. You verify EVERY SINGLE callable unit.

```
1. ROUTE/ENDPOINT CENSUS (100% coverage)
   Discover EVERY route:
   - Next.js: scan app/ pages/ for page.tsx, route.ts, layout.tsx
   - Express/Fastify: grep router.get/post/put/delete/patch
   - Python: grep @app.route, @router, def handle_
   - API routes: every /api/* endpoint
   
   FOR EACH ROUTE:
   → Is it reachable? (not orphaned behind dead nav)
   → Does it have auth protection? (middleware, decorator, guard)
   → Does it handle ALL HTTP methods it claims? (GET+POST?)
   → Does it validate input? (body, params, query)
   → Does it handle errors? (try/catch, error boundary)
   → What does it return on success? On failure? On invalid input?
   → Is the response shape documented? Does it match reality?

2. FUNCTION-LEVEL VERIFICATION (every public function)
   FOR EACH exported/public function:
   → Call sites: who calls it? (0 = dead, 1 = fragile, many = stable)
   → Input domain: what values CAN be passed? 
   → Edge cases: what happens with null? undefined? empty string? empty array? 0? negative? MAX_INT?
   → Return contract: does it ALWAYS return what callers expect?
   → Failure modes: can it throw? do callers handle that?
   → Idempotency: is calling it twice safe? or does it double-write?

3. DATABASE OPERATION AUDIT (every query/mutation)
   FOR EACH database operation (Convex mutation/query, Prisma call, SQL):
   → Is input validated BEFORE the query? (not after)
   → Is there a transaction where needed? (multi-step writes)
   → What happens if the query fails? (retry? crash? partial state?)
   → Is there an index for this query pattern? (or will it table-scan)
   → Can it produce orphaned data? (parent deleted but children remain)
   → Is there pagination? (or will it return 10M rows?)
   → Are results cached? Should they be? Is cache invalidated correctly?

4. UI ELEMENT VERIFICATION (every interactive element)
   FOR EACH button/link/form/input discovered:
   → Does clicking it do what the label says?
   → Does it have a loading state? (or does it look frozen?)
   → Does it have an error state? (or does it silently fail?)
   → Does it have a disabled state? (when should it be disabled?)
   → Can it be double-clicked? What happens? (double submit?)
   → Does it work with keyboard? (Enter, Space, Tab)
   → Is there visual feedback? (hover, active, focus states)

5. EVENT HANDLER VERIFICATION (every on* handler)
   FOR EACH event handler (onClick, onSubmit, onChange, onError):
   → Is the handler async? Does it handle rejection?
   → Does it prevent default when it should? (form submit, link click)
   → Does it stop propagation when it should? (nested clickables)
   → Does it debounce/throttle when it should? (search input, resize)
   → What happens if the component unmounts during async handler? (memory leak)

6. WEBHOOK/CALLBACK VERIFICATION
   FOR EACH webhook handler:
   → Is the signature verified? (Stripe, Clerk, Linear signatures)
   → Is it idempotent? (receiving same event twice = same result)
   → Does it respond quickly? (200 before processing, not after)
   → Does it handle unknown event types? (graceful ignore, not crash)
   → Is there retry handling? (webhook providers retry on failure)

7. CRON/SCHEDULED TASK VERIFICATION
   FOR EACH scheduled task:
   → Does it actually run? (check last execution)
   → Is it idempotent? (running twice doesn't corrupt data)
   → Does it handle partial failure? (item 50/100 fails — what about 51-100?)
   → Is there a timeout? (cron runs forever = resource lock)
   → Does it conflict with other crons? (same table, same time)
```

**Output:** `audits/.codeaudit/reports/feature-verification.md` — Every route, function, query, button documented with PASS/FAIL.

---



> *"Entropy measures disorder. High entropy = the codebase is losing the fight against chaos."*

```
1. NAMING ENTROPY
   Same concept, how many names?
   → user/account/profile/person → should be 1 name
   → config/settings/options/preferences → should be 1 name
   → save/store/persist/write/commit → should be 1 name
   Score: unique names per concept. >3 = HIGH ENTROPY

2. PATTERN ENTROPY
   Same operation, how many patterns?
   → Error handling: try/catch here, .catch() there, if(err) elsewhere
   → API calls: fetch here, axios there, custom client elsewhere
   → State management: useState here, zustand there, context elsewhere
   Score: patterns per operation type. >2 = HIGH ENTROPY

3. STRUCTURE ENTROPY
   File organization consistency:
   → Some folders have index.ts, some don't
   → Some modules export default, some export named
   → Some files use classes, some use functions, some use both
   Score: structural inconsistencies / total files

4. STYLE ENTROPY
   → Tabs vs spaces (in same project)
   → Single vs double quotes (in same project)
   → Semicolons vs no semicolons (in same project)
   → camelCase vs snake_case in same language context

5. ARCHITECTURAL ENTROPY (via git)
   git log --format='%H %s' → classify commits by type
   → Ratio of feature commits to fix commits (high fix ratio = entropy growing)
   → Ratio of refactor commits to total (low refactor = entropy growing)
   → Average commit size trend (growing = changes getting harder to isolate)
```

---

## PHASE 14: GIT CRIMINAL PROFILING

> *"Git history is a confession. Every commit tells you what was wrong."*

```
1. COMMIT AUTOPSY
   For the last 100 commits:
   → Average lines changed per commit (>200 = "god commits", hard to review)
   → Commit message quality: imperative? descriptive? or "fix stuff"?
   → Fix-to-feature ratio: >0.5 = more time fixing than building
   → "WIP" or "temp" commits on main = process failure

2. BLAME HOTSPOT ANALYSIS
   For the 10 most complex files:
   → git blame → author distribution
   → Bus factor per file: 1 author = 1.0 risk, 5 authors = 0.2 risk
   → Knowledge concentration: does anyone understand the WHOLE system?

3. REVERT FORENSICS
   git log --grep="revert\|rollback\|undo" →
   → What was reverted? Why? (read the revert message)
   → Was the root cause fixed or just the symptom reverted?
   → Revert frequency: >1/week = instability signal

4. BRANCH ARCHAEOLOGY
   git branch -r → remote branches
   → Stale branches (>30 days, unmerged) = abandoned work
   → Long-lived branches (>2 weeks) = merge conflict risk
   → Branches with conflicts against main = landmines

5. CHURN CORRELATION
   Files with highest churn (most commits) + highest complexity:
   → These are the BUG MAGNETS — most likely to have defects
   → Cross-reference with bug fix commits: confirmed?
   → These files need the most test coverage (do they have it?)

6. TEMPORAL COUPLING
   Files that ALWAYS change together:
   git log --format='' --name-only | sort | uniq > all_files
   For each pair: how often do they appear in the same commit?
   → Always together but not importing each other = HIDDEN COUPLING
   → Always together and should be ONE FILE = merge candidates
```

---

## PHASE 15: RUNTIME VIVISECTION

> *"A passing build doesn't mean a working system. Cut it open while it's running."*

```
1. PROCESS FORENSICS (if running)
   ps aux | grep [process]
   → RSS memory: expected? growing over time?
   → CPU: idle? spike pattern? constant high?
   → Uptime: been running for days? (memory leak accumulation)
   → Children: spawned processes? zombies?

2. NETWORK FORENSICS
   ss -tlnp → listening ports
   → Expected ports? Unexpected ports? (something you didn't start)
   ss -tnp → active connections
   → Connection count: growing? (leak) stable? (healthy)
   → Connections to unexpected destinations? (exfiltration risk)

3. FILE DESCRIPTOR FORENSICS
   lsof -p PID → open files
   → Count: stable or growing? (growing = leak)
   → Types: regular files? sockets? pipes?
   → Any deleted files still open? (can't reclaim disk space)

4. DISK FORENSICS
   du -sh /tmp/aisb-* ~/.omega/ .hunt/ → temp file accumulation
   → Orphan temp files? (never cleaned up)
   → Log rotation working? (logs growing without bound?)
   → Backup files accumulating? (.backup, .bak, .old)

5. CRON/SYSTEMD VERIFICATION
   For each scheduled job / service:
   → Is it actually running? (systemctl is-active)
   → When did it last succeed? (journalctl --since)
   → Are there failure patterns? (journalctl -p err)
   → Does the timer match what config says?

6. DEPENDENCY HEALTH (external services)
   For each external service the system connects to:
   → Is it reachable right now? (curl/ping)
   → What's the latency? (acceptable?)
   → Are credentials valid? (test with minimal API call)
   → Is the version compatible? (API version match)
```

---

## PHASE 16: LOGGING & OBSERVABILITY AUTOPSY

> *"If you can't see it happening, you can't fix it when it breaks."*

```
1. LOG COVERAGE
   For each module/function that can fail:
   → Is there a log statement? What level? (debug/info/warning/error)
   → Is the log message actionable? ("User login failed" vs "Error occurred")
   → Does it include context? (user_id, request_id, file:line, input values)
   → Are sensitive values masked? (passwords, tokens, PII)

2. LOG LEVEL CORRECTNESS
   → Are DEBUG logs actually debugging info? (not business logic)
   → Are INFO logs actually informational? (not errors logged as info)
   → Are WARNING logs real warnings? (not normal operations)
   → Are ERROR logs real errors? (not "expected" failures like 404)
   → Is there any print() instead of logger? (stdout pollution)

3. STRUCTURED LOGGING
   → Are logs structured (JSON) or free-text?
   → Can logs be parsed by log aggregators? (ELK, Datadog, Loki)
   → Is there a correlation ID for request tracing?
   → Can you trace a request from entry to exit through logs alone?

4. OBSERVABILITY GAPS
   → Are there "silent" code paths? (no logs at all)
   → Are async operations logged at start AND completion?
   → Are external API calls logged with latency?
   → Are database queries logged with execution time?
   → Is there a metrics endpoint? (health check, readiness probe)

5. LOG HYGIENE
   → Log rotation configured? (not growing forever)
   → Log volume: how many lines/minute? (too many = noisy, too few = blind)
   → Duplicate log messages? (same error logged 10x in a loop)
   → Console.log / print statements in production code?
```

**Output:** `audits/.codeaudit/reports/observability.md`

---

## PHASE 17: TEST COVERAGE INTERROGATION

> *"Untested code is unverified code. Unverified code is a liability."*

```
1. TEST EXISTENCE
   → Does the project have ANY tests? (unit, integration, e2e)
   → What framework? (vitest, jest, pytest, playwright)
   → When were tests last run? Last updated?
   → Are tests in CI? Do they block merges?

2. COVERAGE MAP
   For each module:
   → How many functions are tested? / total functions
   → How many branches are tested? / total branches
   → Calculate: coverage % per file
   → ZERO COVERAGE files = CRITICAL (especially if blast radius > MEDIUM)

3. TEST QUALITY (not just quantity)
   → Do tests assert meaningful outcomes? (not just "doesn't throw")
   → Do tests cover edge cases? (null, empty, boundary values)
   → Do tests cover error paths? (not just happy path)
   → Are there snapshot tests that nobody understands?
   → Are tests deterministic? (no random failures, no time-dependent)

4. CRITICAL PATH COVERAGE
   → Is the auth flow tested? (login, session, token refresh)
   → Are payment flows tested? (if applicable)
   → Are data mutation flows tested? (create, update, delete)
   → Are the SPOF files (from Phase 7) tested?

5. TEST DEBT
   → Skipped tests (xit, @skip, pytest.mark.skip) — why?
   → Commented-out tests — why?
   → Tests marked TODO — how long ago?
   → Flaky tests — which ones? (run 10x, count failures)
```

**Output:** `audits/.codeaudit/reports/test-coverage.md`

---

## PHASE 18: API CONTRACT VERIFICATION

> *"Your API is a promise. Break the promise, break the client."*

```
1. ENDPOINT INVENTORY (100% coverage)
   For EVERY API endpoint:
   → Method + Path
   → Auth required? (verified in code, not just docs)
   → Request schema (body, params, query, headers)
   → Response schema (success shape, error shape)
   → Rate limited?
   → Actually tested with curl/httpie? (don't trust docs)

2. CONTRACT CONSISTENCY
   → Do TypeScript types match actual API responses?
   → Do frontend API calls match backend expectations?
   → Are error codes correct? (404 for not found, 401 for unauth, not 500 for everything)
   → Is the response format consistent? ({data, error} vs {result} vs raw)

3. BACKWARDS COMPATIBILITY
   → Are there versioned endpoints? (/api/v1/, /api/v2/)
   → Are deprecated endpoints still functional? Or silently broken?
   → Would removing any endpoint break existing clients?

4. WEBHOOK INTEGRITY
   For each webhook handler:
   → Signature verification present?
   → Idempotency key checked?
   → Response within timeout? (Stripe: 20s, Clerk: 30s)
   → All event types handled? Or just some?

5. GRAPHQL / tRPC SPECIFIC (if applicable)
   → Are all resolvers implemented? (no stub resolvers)
   → Are input types validated? (not just typed)
   → Are N+1 queries detected? (dataloader used?)
   → Is the schema documented?
```

**Output:** `audits/.codeaudit/reports/api-contracts.md`

---

## PHASE 19: IDEMPOTENCY & RESILIENCE

> *"If calling it twice causes different results, it's not an API. It's a gamble."*

```
1. IDEMPOTENCY CHECK
   For every write operation (POST, PUT, PATCH, DELETE, mutation):
   → What happens if called twice with same input?
   → Is there a deduplication mechanism? (idempotency key, upsert)
   → Can double-submit create duplicate data?
   → Can double-delete cause errors?

2. RETRY SAFETY
   For every operation that can fail:
   → Is it safe to retry? (idempotent = yes, non-idempotent = dangerous)
   → Is there automatic retry? With backoff?
   → Is there a max retry limit? (infinite retry = resource exhaustion)
   → What state is left after partial failure + retry?

3. GRACEFUL DEGRADATION
   For each external dependency (DB, API, cache, queue):
   → What happens when it's down?
   → Is there a fallback? (cache, default value, queue)
   → Is there a circuit breaker?
   → Is the user informed? (error page vs infinite spinner)

4. PARTIAL FAILURE HANDLING
   For every multi-step operation:
   → What if step 2 of 5 fails?
   → Is there rollback? Compensation? Saga pattern?
   → Is the system left in a consistent state?
   → Can the operation be resumed?

5. DATA CONSISTENCY UNDER FAILURE
   → If the process crashes mid-write, is data corrupted?
   → Are database writes transactional where needed?
   → Are file writes atomic? (write to temp, then rename)
   → Are cache invalidations reliable? (stale cache after write)
```

**Output:** `audits/.codeaudit/reports/resilience.md`

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

The HINGE POINT of this audit is the locus of maximum risk/value. Compute or
read:

```bash
# If a hinge file already exists for this ticket, use it:
HINGE_FILE="$HOME/.omega/state/hinge-points-${TICKET_ID:-default}.json"

# Otherwise compute it on the fly:
${OMEGA_DIR:-$HOME/.omega}/skills/audits/_shared/hinge-analyzer.sh "$PROJECT_PATH" --audit=code --user-need="$USER_NEED_QUOTE" \
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
  "evidence_gathered": "Manual repro on prod URL; screenshot at .code/edge-evidence/F-007.png",
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
  "audit": "code",
  "score": 100,
  "score_raw": "<raw>/<denominator>",
  "score_normalized": 100,
  "confidence": "high|medium|low",
  "skill_used": "code",
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
  "evidence_summary_path": "$PROJECT_PATH/audits/.codeaudit/evidence-summary.json",
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

## PHASE 20: VERDICT — FORENSIC REPORT

Score each phase 0-10, weight by severity:

```
SCORING MATRIX (420 max):
  Phase  1  (Phantoms)              × 3.0  = max 30
  Phase  2  (Dependencies)          × 2.0  = max 20
  Phase  3  (Contracts)             × 2.5  = max 25
  Phase  4  (Data Flow)             × 3.0  = max 30
  Phase  5  (State Mutation)        × 2.5  = max 25
  Phase  6  (Concurrency)           × 3.0  = max 30
  Phase  7  (Blast Radius)          × 2.0  = max 20
  Phase  8  (Time Bombs)            × 2.0  = max 20
  Phase  9  (Supply Chain)          × 2.0  = max 20
  Phase 10  (Error Propagation)     × 3.0  = max 30
  Phase 11  (Behavioral)            × 2.0  = max 20
  Phase 12  (Config Drift)          × 2.0  = max 20
  Phase 12.5(Feature Verification)  × 3.0  = max 30
  Phase 13  (Entropy)               × 1.5  = max 15
  Phase 14  (Git Forensics)         × 1.5  = max 15
  Phase 15  (Runtime)               × 2.0  = max 20
  Phase 16  (Observability)         × 2.5  = max 25
  Phase 17  (Test Coverage)         × 2.5  = max 25
  Phase 18  (API Contracts)         × 2.0  = max 20
  Phase 19  (Resilience)            × 2.0  = max 20
                                    TOTAL  = max 420

NORMALIZE to 0-100: score = (raw / 420) × 100

GRADE:
  90-100: S — Fortress. Battle-hardened, paranoid-level quality.
  80-89:  A — Solid. Minor issues, nothing structural.
  70-79:  B — Good. Some debt, known risks managed.
  60-69:  C — Acceptable. Significant debt, some blind spots.
  50-59:  D — Risky. Structural issues, missing coverage.
  <50:    F — Condemned. Major issues, not production-safe.
```

### FINAL OUTPUT

```
╔══════════════════════════════════════════════════════════════╗
║  /codeaudit v2 — FORENSIC REPORT                           ║
║  Target: {path}                                              ║
║  Score: {score}/100 — Grade {grade}                          ║
║  Date: {date} | Duration: {duration}                         ║
╠══════════════════════════════════════════════════════════════╣
║                                                              ║
║  PHASE  1  Phantoms ............ {score}/10  {bar}           ║
║  PHASE  2  Dependencies ........ {score}/10  {bar}           ║
║  PHASE  3  Contracts ........... {score}/10  {bar}           ║
║  PHASE  4  Data Flow ........... {score}/10  {bar}           ║
║  PHASE  5  State Mutation ...... {score}/10  {bar}           ║
║  PHASE  6  Concurrency ......... {score}/10  {bar}           ║
║  PHASE  7  Blast Radius ........ {score}/10  {bar}           ║
║  PHASE  8  Time Bombs .......... {score}/10  {bar}           ║
║  PHASE  9  Supply Chain ........ {score}/10  {bar}           ║
║  PHASE 10  Error Propagation ... {score}/10  {bar}           ║
║  PHASE 11  Behavioral .......... {score}/10  {bar}           ║
║  PHASE 12  Config Drift ........ {score}/10  {bar}           ║
║  PHASE 13  Entropy ............. {score}/10  {bar}           ║
║  PHASE 14  Git Forensics ....... {score}/10  {bar}           ║
║  PHASE 15  Runtime ............. {score}/10  {bar}           ║
║                                                              ║
╠══════════════════════════════════════════════════════════════╣
║  TOP 10 CRITICAL FINDINGS                                    ║
║                                                              ║
║  1. [{PHASE}] {finding} — {file}:{line}                     ║
║  2. [{PHASE}] {finding} — {file}:{line}                     ║
║  ...                                                         ║
╠══════════════════════════════════════════════════════════════╣
║  SPOF MAP (Single Points of Failure)                         ║
║  {file}: blast={N} files, bus_factor={N}, tests={N}          ║
║  ...                                                         ║
╠══════════════════════════════════════════════════════════════╣
║  TIME BOMBS (earliest detonation first)                      ║
║  {description} — ETA: {date/condition}                       ║
║  ...                                                         ║
╠══════════════════════════════════════════════════════════════╣
║  Full evidence: audits/.codeaudit/reports/ (audits/.codeaudit/evidence/)           ║
╚══════════════════════════════════════════════════════════════╝
```

---

## PHASE 21: FIX PLAN (automatic after verdict)

> *"An audit without a fix plan is a complaint. An audit with a fix plan is a roadmap."*

After the verdict, AUTOMATICALLY generate the fix plan. No human approval needed.

```
1. PRIORITIZE ALL FINDINGS
   Sort by: CRITICAL → HIGH → MEDIUM → LOW
   Within same severity: blast_radius DESC, then fix_effort ASC
   → Fix the highest-impact, easiest-fix issues FIRST

2. GROUP BY FILE
   Cluster findings that touch the same file
   → Minimize file read/write cycles
   → Avoid merge conflicts between fixes

3. DEPENDENCY ORDER
   Some fixes depend on others:
   → Fix circular deps BEFORE fixing imports
   → Fix types BEFORE fixing contracts
   → Fix auth BEFORE fixing API endpoints
   → Fix error handling BEFORE fixing resilience

4. GENERATE FIX TASKS
   For EACH finding, generate a concrete fix task:
   {
     id: "FIX-001",
     finding: "P10-SILENT-FAIL account.py:173",
     severity: "HIGH",
     file: "account.py",
     line: 173,
     current: "except Exception: pass",
     fix: "except Exception as e: logger.warning(f'Account op failed: {e}')",
     blast_radius: "LOW",
     estimated_effort: "1 min",
     blockedBy: []
   }

5. WRITE PLAN
   Save to audits/.codeaudit/fix-plan.json + audits/.codeaudit/fix-plan.md
   Human-readable summary + machine-readable task list
```

---

## PHASE 22: FIX EXECUTION (automatic)

> *"Diagnose. Plan. Fix. Verify. No steps skipped."*

```
EXECUTION RULES:
1. SEQUENTIAL by file group (no parallel edits on same file)
2. ATOMIC COMMITS per fix: git add FILE && git commit -m "fix(audit): FIX-XXX description"
3. BUILD CHECK after each fix (if applicable): npm run build / python -m py_compile
4. If build breaks → git revert HEAD → skip this fix → mark NEEDS_REVIEW
5. NEVER touch files outside the finding's scope
6. NEVER "improve" code beyond the finding — fix ONLY what was flagged
7. VISUAL REGRESSION CHECK: if the fix touches frontend code (components, pages, CSS):
   → Navigate the affected page(s) via Playwright CLI
   → Check: does it still render correctly? Console errors? Layout broken?
   → If visual regression → git revert HEAD → mark NEEDS_REVIEW
8. FUNCTIONAL CHECK: if the fix touches data layer, API, or business logic:
   → Verify the feature still WORKS (not just compiles)
   → Click the button, submit the form, check the data
   → If feature breaks → git revert HEAD → try different approach

─── SAFETY GATE: DO NO HARM (MANDATORY before EVERY fix) ──────────────

The audit MUST NOT introduce new bugs. A fix that breaks the code is worse than
the original finding. Every fix goes through this gate BEFORE commit.

PRE-FIX ANALYSIS (before writing ANY code):
  a. Read the ENTIRE target file (not just the target line)
  b. SCOPE COLLISION CHECK — if adding/renaming a variable or import:
     → Grep the ENTIRE file for that name (all occurrences)
     → Check: is this name already used as a local, parameter, or reassigned?
     → Check: does this name get shadowed later in the same scope?
     → Python example: `import x as _foo` is UNSAFE if `_foo = SomeClass()` exists
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
     → Python: `timeout 10 python main.py` or check systemd: `systemctl restart X && sleep 5 && systemctl is-active X`
     → Node: `timeout 10 node server.js` or `npm run build`
     → If service crashes → git revert HEAD → mark NEEDS_REVIEW
  d. TEST SUITE — if tests exist:
     → Run the relevant test file(s): `pytest FILE -x` / `vitest run FILE`
     → If tests fail → git revert HEAD → investigate

IF ANY POST-FIX CHECK FAILS:
  → `git revert HEAD` immediately
  → Log the failure in audits/.codeaudit/fix-log.md with exact error
  → Mark as NEEDS_REVIEW (never retry same approach blindly)
  → Try alternative approach OR skip this fix

────────────────────────────────────────────────────────────────────────

FOR EACH FIX TASK (in priority order):
  a. Read the ENTIRE target file (full context, not just the line)
  b. Run PRE-FIX ANALYSIS (scope collision, import shadow, cross-reference)
  c. Apply the fix
  d. Run POST-FIX VERIFICATION (syntax, import, smoke test, tests)
  e. If all green → commit
  f. If any red → revert → log → mark NEEDS_REVIEW
  g. Mark task DONE/FAILED in audits/.codeaudit/fix-plan.json
  h. Log: audits/.codeaudit/fix-log.md

PROGRESS: Update audits/.codeaudit/progress.json after each fix:
{
  "total": 47,
  "done": 12,
  "failed": 1,
  "skipped": 2,
  "remaining": 32,
  "current": "FIX-013 — replacing bare except in monitor.py:365"
}
```

---

## PHASE 23: RE-AUDIT (automatic after fixes)

> *"Trust but verify. Actually, don't trust. Just verify."*

```
After ALL fixes applied:
1. SERVICE HEALTH GATE (mandatory):
   → If project has systemd service: restart it, wait 10s, check is-active + logs for errors
   → If project has build step: full build must pass (npm run build / cargo build / etc.)
   → If project has tests: full test suite must pass
   → If ANY fails: identify which fix broke it (git bisect or revert last N), fix or revert
2. Re-run the FAILING phases from the original audit
3. Compare: old score vs new score per phase
4. Verify: no NEW findings introduced by fixes (regression)
5. Calculate: BEFORE score → AFTER score → IMPROVEMENT

OUTPUT:
╔══════════════════════════════════════════════════════════╗
║  /codeaudit — RE-AUDIT COMPARISON                       ║
╠══════════════════════════════════════════════════════════╣
║  Phase        BEFORE    AFTER    DELTA                  ║
║  Phantoms       9/10     10/10    +1                    ║
║  Error Prop     3/10      7/10    +4  ←← biggest win    ║
║  Concurrency    5/10      6/10    +1                    ║
║  ...                                                     ║
╠══════════════════════════════════════════════════════════╣
║  BEFORE: 68/100 (Grade C)                                ║
║  AFTER:  84/100 (Grade A)                                ║
║  IMPROVEMENT: +16 points                                 ║
║  FIXES: 43 applied, 2 failed, 2 needs_review            ║
╚══════════════════════════════════════════════════════════╝

If AFTER score < 80: loop back to Phase 21 with remaining findings.
If AFTER score >= 80: DONE. Ship it.
```

---

## EXECUTION

| Command | Scope |
|---------|-------|
| `/codeaudit` | Full pipeline: audit → plan → fix → re-audit |
| `/codeaudit [path]` | Target specific directory |
| `/codeaudit --audit-only` | Phases 1-20 only (no fix) |
| `/codeaudit --fix-only` | Phases 21-23 from existing audits/.codeaudit/fix-plan.json |
| `/codeaudit --focus phantoms` | Phase 1 audit + fix phantoms |
| `/codeaudit --focus security` | Phases 4+5+6+9+10 audit + fix |
| `/codeaudit --focus architecture` | Phases 2+7+13 audit + fix |
| `/codeaudit --focus runtime` | Phases 11+15 audit + fix |
| `/codeaudit --focus git` | Phase 14 only (read-only, no fix) |
| ~~`/codeaudit --quick`~~ | **REMOVED per rule 46 (NO TIME PANIC).** If you want narrower scope, use `--focus <area>` flags above — each targets specific phases with full depth. There is no "quick" mode. |

---

## LAWS

1. **Never trust, always verify.** A passing test doesn't mean correct code. A building project doesn't mean working software. A comment doesn't mean truth.
2. **Quantify or it's opinion.** "The code has issues" = useless. "47 phantom imports, 3 circular deps, 12 unhandled error paths" = actionable.
3. **Evidence chain.** Every finding: file:line → what's wrong → why it matters → blast radius → suggested fix.
4. **Adversarial thinking.** For every function: "How would I break this?" For every config: "What happens if this value is wrong?" For every integration: "What if the other side lies?"
5. **The codebase is guilty until proven innocent.**
6. **An audit that doesn't fix is a complaint.** Always audit → plan → fix → re-audit. Full cycle.
7. **Cross-domain awareness.** If fixing code breaks the UI, that's YOUR problem. If the design is inconsistent because of code bugs, fix the code too. No "that's a design issue" excuses.

---

## CROSS-COMMAND BRIDGE

`/codeaudit` is code-first but MUST handle visual/functional issues it discovers:

```
IF during Phase 12.5 (Feature Verification) you find:
  - A page renders wrong → FIX the component/CSS (don't defer to /uiuxaudit)
  - Cross-page inconsistency → FIX the shared component
  - Empty content that should have data → FIX the query/mutation/render
  - Broken button/form → FIX the handler/API

IF during Phase 22 (Fix Execution) you touch a frontend file:
  - Navigate the page after fix → verify it looks correct
  - If it looks wrong → your fix broke the UI → revert and try again

RULE: /codeaudit handles EVERYTHING it finds.
It doesn't punt to /uiuxaudit. It doesn't punt to other audits.
If it's broken, fix it. If it's ugly AND caused by code, fix it.
```

---

*"/codeaudit v3 — Audit. Plan. Fix. Verify. 23 phases, /420. The codebase leaves cleaner than it entered."*

---

## COMPLIANCE & CRITICAL ADDENDA (v1.1 — 2026-04-14T10:35:36Z)

### Quality Arsenal Preamble Compliance

This audit implements contracts defined in `../_shared/QUALITY-ARSENAL-PREAMBLE.md` v1.0:

- ✅ **Gestalt-Popper doctrine** — hinge point, falsification, evidence chain, adversarial thinking
- ✅ **Concurrency lock** — canonical runner `flock` at `audits/.codeaudit/.runner.lock`
- ✅ **5-iteration cap** — fix-and-reaudit loop bounded at 5 iterations (rule 43 step 8b alignment). On cap: NEEDS_REVIEW + Telegram SOS. No silent infinite loops.
- ✅ **Scoped invocation flags** — `--url=`, `--files=`, `--scope=`, `--ticket=`, `--no-fix`, `--focus=`
- ✅ **Non-UI context gate** — Non-UI contexts: /codeaudit runs on ALL project types (primary owner of code-quality for CLIs, libraries, backends).
- ✅ **Output contract verification** — emits `audits/.codeaudit/verdict.json`, `verdict.md`, `fix-plan.json`, `fix-plan.md`, `iterations.md`, `progress.json`, `telemetry.json`, `fix-log.md`. Output gate runs at end; missing/malformed files = audit did NOT succeed.
- ✅ **Telegram progress notifications** — `start` / `progress` (every 3 phases) / `iteration` / `verdict` / `abort` / `sos` events via `~/.omega/bin/audit-notify.sh`
- ✅ **Discovery drift check** — on resumed runs, if `audits/.codeaudit/discovery/` > 1h old, re-verify inventory or abort with user-confirm
- ✅ **Self-telemetry** — `audits/.codeaudit/telemetry.json` emitted at completion (duration, tokens, phases, fixes, model, preamble_version)
- ✅ **Deprecation registry** — cross-references checked against `${OMEGA_DIR:-$HOME/.omega}/skills/audits/_shared/DEPRECATED.md`; stale refs flagged as findings
- ✅ **Rule-46 compliance** — NO `--quick`/`--streamlined`/`--lightweight` variants. Narrower scope uses `--focus <area>` flags with FULL phase depth. Orchestrator prompts containing rule-46 banned phrases are REFUSED.
- ✅ **Score normalization** — raw score divided by applicable-phase-max × 100 = /100 normalized score
- ✅ **preamble_version** — emitted as `"1.0"` in verdict.json for `/metaudit` compliance scan

### Audit-Specific Critical Addendum — Integration Smoke Test Gate

Phase 23 fix gate MUST run integration smoke tests (per preamble §11) before commit:

```
1. Detect integrations: grep package.json for @composio/sdk, @anthropic-ai/*, @modelcontextprotocol/*, stripe, @clerk/*, convex, @trigger.dev/*, LINEAR_API_KEY in .env
2. For each detected integration, run smoke test:
   - Stripe: `node -e "require(\"stripe\")(process.env.STRIPE_SECRET_KEY).accounts.retrieve()"`
   - Clerk: `curl -H "Authorization: Bearer $CLERK_SECRET_KEY" https://api.clerk.com/v1/jwt_templates`
   - Convex: `npx convex dev --once` (dry validation)
   - Composio: `npx composio ping` or equivalent
   - MCP: test server handshake via `mcp list`
3. Post-fix, re-run the same smoke tests
4. If any integration breaks post-fix → `git revert HEAD` → mark NEEDS_REVIEW
5. Memory-documented basis: fixes must NEVER break Composio/MCP/middleware.
```

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

> *Final gap-closure layer. Every finding from the prior /flowaudit-of-all-audits analysis (codeaudit-specific section) is now explicitly resolved below.*

### Compliance Matrix (machine-verifiable by /metaudit)

```json
{
  "audit": "codeaudit",
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

**Gap 1: Integration smoke test incomplete list → RESOLVED**
Full detected-integration list with smoke-test commands:
| Integration | Detect | Smoke test |
|-------------|--------|-----------|
| Stripe | `stripe` in deps + `STRIPE_SECRET_KEY` env | `node -e "require(\"stripe\")(process.env.STRIPE_SECRET_KEY).accounts.retrieve().then(a => process.exit(a.id ? 0 : 1))"` |
| Clerk | `@clerk/*` in deps + `CLERK_SECRET_KEY` env | `curl -sfH "Authorization: Bearer $CLERK_SECRET_KEY" https://api.clerk.com/v1/jwt_templates` |
| Convex | `convex` in deps + `NEXT_PUBLIC_CONVEX_URL` | `npx convex dev --once --skip-push` |
| Composio | `@composio/sdk` in deps + `COMPOSIO_API_KEY` | `npx composio-core auth status` |
| MCP | `@modelcontextprotocol/*` in deps | Test handshake against configured servers |
| Trigger.dev | `@trigger.dev/sdk` in deps + `TRIGGER_SECRET_KEY` | `npx trigger.dev@latest whoami` |
| Linear | `LINEAR_API_KEY` env | `curl -sf -H "Authorization: $LINEAR_API_KEY" -d "{\"query\":\"{ viewer { id }}\"}" https://api.linear.app/graphql` |
| Supabase | `@supabase/*` + `SUPABASE_URL` | `curl -sf "$SUPABASE_URL/rest/v1/" -H "apikey: $SUPABASE_ANON_KEY"` |
| OpenAI/Anthropic SDK | imports detected | Models list endpoint dry call |

**Gap 2: Revert protocol formalized → RESOLVED**
```
On smoke-test failure post-fix:
  1. git stash --include-untracked (preserve any logs)
  2. git reset --hard HEAD~1 (revert the fix commit)
  3. git stash pop (restore logs if any)
  4. Mark finding as NEEDS_REVIEW in fix-plan.json with stash_id
  5. Telegram SOS with error + stash ref
  6. Continue to next fix (do NOT abort the whole audit)
```

**Gap 3: Phantom-import scope collision → RESOLVED via preamble §11 pre-fix checklist**
The pre-fix analysis from flowaudit v2 Phase 23 is adopted here: scope collision check, import shadow check, cross-reference check BEFORE writing any code.

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
/metaudit --focus arsenal --scope="codeaudit only"
# Expected result:
#   preamble_compliance_score: 11/11 (full)
#   audit_specific_compliance: 100/100
#   overall: 100/100
```

---

*codeaudit v1.2 — 100/100 compliance achieved via QUALITY-ARSENAL-PREAMBLE.md v1.0 + audit-specific gap closures above. Certified 2026-04-14.*

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

---

## Dynamic-Workflow Orchestration (v2)

> *"Witnesses don't testify one at a time on a single bench — empanel parallel juries, cross-examine every confession, then deliver one verdict."*
> **EXECUTION-MODEL UPGRADE — does NOT change any phase, weight, verdict template, or frontmatter above.** This section governs HOW the 23 phases run and HOW findings earn the right to enter the scoring matrix. The doctrine (Gestalt hinge point, Popper falsification, "the codebase is guilty until proven innocent") is unchanged — this makes it *concurrent* and *adversarial-by-construction*. Phase 0 (programmatic gather) and Phase H1 (hybrid synthesis) still run; this layer wraps the domain phases between them.

### A. Decompose into INDEPENDENT parallel tracks (fan-out via the Workflow tool)

After Phase 0's gather writes `evidence-summary.json` and the HINGE POINT is identified, do NOT walk Phases 1→19 linearly. Group them into **file-disjoint, dependency-free tracks** and dispatch each as a concurrent Workflow branch. The 23 phases cluster into tracks that share no mutable state (all read-only analysis), so they parallelize cleanly:

| Track | Phases (existing, unchanged) | Independent because |
|-------|------------------------------|---------------------|
| **T1 — Structural** | 1 Phantoms, 2 Dependencies, 7 Blast Radius, 13 Entropy | Pure import-graph / symbol-existence analysis |
| **T2 — Contract & Data** | 3 Contracts, 4 Data Flow, 5 State Mutation, 12 Config Drift | Follows declared vs actual behavior of symbols |
| **T3 — Concurrency & Resilience** | 6 Concurrency, 10 Error Propagation, 19 Resilience | Failure-path / race reasoning |
| **T4 — Temporal & Supply** | 8 Time Bombs, 9 Supply Chain, 14 Git Forensics | History, deps, expiry — no overlap with source-symbol tracks |
| **T5 — Runtime & Observability** | 11 Behavioral, 15 Runtime, 16 Observability, 17 Test Coverage | Needs the running system / logs, not static reads |
| **T6 — Surface verification** | 12.5 Feature Verification, 18 API Contracts | 100%-coverage census of routes/functions/endpoints |

Rules:
- Dispatch T1–T6 as **parallel Workflow branches**; each branch consumes the shared `evidence-summary.json` (read-only) and writes ONLY its own `audits/.codeaudit/reports/<phase>.md`. One writer per report file → no R-SCOPE collision.
- **HINGE POINT override:** whichever track contains the hinge file/module runs at 10× depth (per Phase H1 §2.2) and is NEVER skipped or time-boxed; lighter tracks may finish first.
- Tracks are analysis-only and emit *candidate* findings — nothing reaches the scoring matrix until it survives Section B.
- If two phases would touch the same file during the FIX stage (Phases 21–23), they are **serialized** there (the existing "SEQUENTIAL by file group" rule in Phase 22 still governs all mutation). Parallelism is for *discovery*, not for edits.

### B. Adversarially verify every finding — ≥2-of-3 independent lenses (kill non-survivors)

A candidate finding from any track is an **accusation, not a conviction.** Before it is allowed into `verdict.json.findings[]` and the scoring matrix, subject it to three independent lenses and require **≥2 to agree.** This operationalizes Popper falsification (Phase H1 §2.1) as a gate, not an afterthought:

1. **REPRODUCE** — run the concrete command/probe that makes the defect observable at runtime (the First Law: runtime > code > comments). E.g. for a Phase 10 silent-swallow finding, execute the path and show the error vanishing; for a Phase 12.5 dead route, `curl` it and show the 404/200 mismatch. Cite verbatim output.
2. **REFUTE (steel-man the code)** — actively try to PROVE the finding wrong, exactly as Phase H1 §2.1: grep for the symbol in tests/dynamic-imports, read the middleware that the static scan couldn't see, check whether a "circular dep" is a type-only re-export artifact. If refutation succeeds → finding is `falsified` → demote to `info` with `falsified_at:<evidence>` and DROP it from scoring.
3. **CROSS-CHECK** — corroborate against an independent signal: a sibling audit's `evidence-summary.json` (Phase 0.5 / H1 §2.5), a second tool in `raw/`, a different caller/callee of the hinge symbol, or git blame/churn. Agreement sets `cross_audit_confirmed: true` and bumps severity one level (per H1 §2.5).

Verdict per finding:
- **≥2 lenses confirm** → finding STANDS, enters scoring with its evidence chain (`file:line → wrong → why → blast radius → fix`).
- **≤1 lens confirms** (or REFUTE wins) → finding is KILLED (recorded as `falsified`/`inconclusive` in `falsifiable_tests[]`, excluded from the matrix). Silence is not a finding.
- Load-bearing (hinge) findings require all three lenses attempted and ≥2 confirming at `confidence: high` before they may carry CRITICAL severity.

This is the antidote to LLM complacency named in the Adversarial Review Framing: a finding survives only because it *resisted* an honest attempt to destroy it, never because it merely "looked correct."

### C. Synthesize survivors back into THIS audit's existing matrix + verdict (UNCHANGED)

The surviving findings (and ONLY them) feed the unchanged scoring engine. Nothing downstream is altered:
- Each surviving finding maps to its originating phase and scores that phase exactly per the **SCORING MATRIX (420 max)** in Phase 20 — same weights (Phantoms ×3.0 … Resilience ×2.0), same `normalize = (raw / 420) × 100`, same S/A/B/C/D/F grade bands.
- The **FINAL OUTPUT** box, `verdict.json` (hybrid v2 schema, §2.6), `fix-plan.json`, and Phases 21→23 (plan → fix → re-audit) run verbatim. Adversarial pre-filtering only changes *which* findings arrive — it does not touch *how they are scored or fixed*.
- Killed findings are still logged (as `falsified`/`inconclusive`) for auditability per R-CITE, but contribute zero to the score.
- All gates remain in force: the 5-iteration fix-and-reaudit cap (Preamble §4), `user_need_match.addressed` (§2.3), the Do-No-Harm before/after matrix, and the output gate.

### D. Loop-until-dry for unknown-size discovery

Code-defect discovery has no a-priori cardinality (a 47-finding gather can cascade into more once callers/callees of the hinge are read). Run discovery as a **bounded loop-until-dry**, not a single pass:

```
discovered = candidates from Phase 0 gather + all 6 tracks
loop:
  verify each NEW candidate through Section B (≥2-of-3)
  for every SURVIVING finding, expand its frontier:
     - read all direct callers + callees of the implicated symbol (hinge: mandatory)
     - re-scan the blast-radius set (Phase 7) those callers belong to
     - pull any sibling-audit finding now sharing that file:line (Phase 0.5)
  new_candidates = frontier findings not yet seen
  STOP when new_candidates == ∅ (dry)  OR  iteration == 5 (Preamble §4 cap)
on cap-without-dry: mark residual frontier NEEDS_REVIEW, emit confidence:"low",
  surface as pending (never silently drop — Law L4)
```

The loop is over *discovery breadth*, bounded by the same 5-iteration ceiling the fix loop uses — it never spins indefinitely and never inflates the score with unverified findings. When dry, hand the survivor set to Section C.

> **Net effect:** same 23 phases, same /420 matrix, same verdict + Gestalt-Popper doctrine — now executed as concurrent fan-out, with every finding earning entry through ≥2-of-3 adversarial verification, looped until the codebase yields no new survivable defect.

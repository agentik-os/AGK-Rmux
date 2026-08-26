---
name: QUALITY-ARSENAL-PREAMBLE
description: >
  Shared doctrine, invariants, and contracts for every audit declared in the
  Quality Arsenal registry. Every registered audit MUST implement these contracts.
  Referenced by /metaudit for compliance verification.
  NOT a user-invokable skill — this is a shared source of truth.
---

# Quality Arsenal Preamble v1.0

> *"One doctrine, twenty-three implementations, zero drift."*

Every Gestalt-Popper forensic audit in the Quality Arsenal inherits the contracts below. Deviations are either (a) declared explicitly with rationale, or (b) a bug caught by `/metaudit`.

---

## 0. THE FIRST LAW (universal, above all others)

> **Code lies. Comments lie. Only runtime tells the truth.**

Before any finding, any fix, any conclusion: **observe the actual runtime behavior**. Reading code reveals what the author *intended*. Reading logs, traces, outputs, network dumps, file states reveals what *actually happens*. When they disagree, reality wins.

**Applied to every audit:**
- Before claiming "X is broken" → show the runtime evidence (log line, trace, output, screenshot).
- Before claiming "X works" → verify at runtime, not by reading the code.
- Before the 3rd code change on the same bug → add logging, reproduce, observe.
- When a comment says "X is required" but X's purpose is unclear → test the alternative in isolation.
- When a fix "should work" but symptoms persist → runtime observation is mandatory before the next attempt.

**Anti-pattern catastrophe (2026-04-14):** 2h35 wasted on `tmux paste-buffer -p` because the comment said "-p is required". Runtime test with and without `-p`: only *without* worked. The comment lied for years. This is Law L1 (runtime is the only truth) — see `omega rules list` for the full protocol.

---

## 1. GESTALT-POPPER DOCTRINE (universal)

- **Gestalt clarity gate** — Before any phase runs, identify the *hinge point* of the system under audit (the one element that, if broken, makes everything else worthless). Audit the hinge with 10x scrutiny. Proportional scrutiny elsewhere.
- **Popper falsification** — Every claim gets a test that could disprove it. A claim that can't be falsified is not a finding, it's an opinion.
- **Evidence chain** — Every finding has: file:line → what's wrong → why it matters → blast radius → suggested fix. Missing any link = invalid finding.
- **Adversarial thinking** — For every component: "How would I break this? What if the inputs lie?"
- **The target is guilty until proven innocent.**
- **Runtime > code > comments.** When the three disagree, trust them in that order.

---

## 2. SCOPED INVOCATION FLAGS (MANDATORY across all registered audits)

Every audit parses these flags identically. Rule 43 (Linear pipeline) depends on this compatibility.

| Flag | Effect | Required when |
|------|--------|---------------|
| `--url={page_url}` | Scope URL-based walkthroughs to this page | Linear ticket audits |
| `--files={comma-separated-paths}` | Scope code-side checks to these files | Targeted code fixes |
| `--scope={1-line description}` | Free-text scope note in outputs | Multi-audit orchestration |
| `--ticket={TICKET_ID}` | Link audit to Linear ticket, write evidence below `audits/.linear-fix/{TICKET}/.<audit-id>/` | Rule 43 pipeline |
| `--no-fix` | Dry-run scoring only; skip fix execution | Review before authorize |
| `--focus={area}` | Per-audit narrower scope with FULL phase depth | Targeted concerns |

**FORBIDDEN (rule 46):** `--quick`, `--streamlined`, `--lightweight`, `--light`, `--fast`, `--custom`. If present in user prompt → REFUSE with reference to rule 46. Narrower scope uses `--focus` with full depth per phase.

**Mandatory combinations:**
- `--ticket=X` requires `--url=Y` (can't audit a ticket without knowing the page)
- Rule 43 dispatches MUST provide `--files`, `--url`, `--ticket`

---

## 3. CONCURRENCY LOCK (MANDATORY)

Every audit enters through `audit-runner.sh` at Phase 0. The runner opens the
canonical `audits/.<audit-id>/.runner.lock` and acquires a non-blocking kernel
`flock` before touching logs, raw evidence, summaries, or verdicts. A collision
exits 2. The kernel releases the lock on crash, so stale PID-file reclamation is
neither needed nor permitted.

```bash
~/.omega/lib/audit-runner.sh <audit> "$PROJECT_PATH" \
  --user-need="$USER_NEED_QUOTE" --hinge="$HINGE_POINT"
# exit 2 means invalid inputs, unsafe paths/evidence, gather failure, or lock collision.
```

Rule 43's parallel DYNAMIC audit chain (`/codeaudit` + `/uiuxaudit` + `/flowaudit` + `/debugaudit` on the same ticket) uses distinct `audits/.<audit-id>/` directories, so locks don't collide across different audits — only duplicate invocations of the same audit are blocked.

Never hand-roll a check-then-create PID lock and never delete a lock reported by
the runner. File-scope ownership for the longer LLM analysis/fix phase remains a
separate OmegaOS scope claim; the runner lock protects its shared artifact writes.

---

## 4. PHASE RE-AUDIT CAP (MANDATORY)

Fix-and-reaudit loops cap at **5 iterations** (aligned with rule 43 step 8b).

```
iteration = 0
while score < target_threshold (80 for solo run, 100 for rule-43 ticket audit):
    iteration += 1
    apply fixes from fix-plan.json
    re-run failing phases
    record score trajectory in audits/.<audit-id>/iterations.md
    if iteration >= 5:
        mark remaining findings as NEEDS_REVIEW in verdict.json
        send Telegram SOS with iterations.md path
        exit loop (do NOT continue indefinitely)
```

Zero tolerance for silent infinite loops. 5 is a hard cap, not a suggestion.

---

## 5. NON-UI CONTEXT HANDLING (MANDATORY per audit)

Not every project has UI/URLs/flows. The table below covers the core surface
audits; every additional registry entry declares the same applicability contract
inside its own `SKILL.md`:

| Project type | /codeaudit | /debugaudit | /uiuxaudit | /flowaudit | /featureaudit | /perfaudit | /secaudit | /a11yaudit | /seoaudit | /copyaudit | /dxaudit | /motionaudit | /dataaudit | /apiaudit |
|--------------|-----------|-------------|-----------|-----------|---------------|-----------|-----------|-----------|-----------|-----------|----------|--------------|-----------|-----------|
| Web app (URLs) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ (if DB) | ✅ (if API) |
| Mobile (RN/Expo) | ✅ | partial | ✅ | mobile mode | ✅ | ✅ | ✅ | ✅ | N/A | ✅ | ✅ | ✅ | ✅ | ✅ |
| CLI tool | ✅ | ⚠️ (log-based) | **ABORT** | **ABORT** | ✅ | ✅ (startup/CPU) | ✅ | ✅ (output text) | N/A | ✅ | ✅ (primary!) | **ABORT** | ✅ (if DB) | N/A |
| Library / SDK | ✅ | N/A | **ABORT** | **ABORT** | ✅ | ✅ | ✅ | N/A | N/A | ✅ (docs) | ✅ | **ABORT** | N/A | ✅ (if API) |
| Backend-only API | ✅ | partial | **ABORT** | **ABORT** | ✅ | ✅ | ✅ | N/A | N/A | ✅ (docs) | ✅ | **ABORT** | ✅ | ✅ (primary!) |
| Headless service | ✅ | log-based | **ABORT** | **ABORT** | ✅ | ✅ | ✅ | N/A | N/A | ✅ | ✅ | **ABORT** | ✅ | ✅ |

**ABORT behavior**: exit with clear error naming the detected project type and suggesting alternative audits. Never hallucinate findings for missing surfaces.

---

## 6. OUTPUT CONTRACT VERIFICATION (MANDATORY)

Every audit declares outputs. Before reporting success, verify they exist with valid schema.

### Required outputs per audit

```
audits/.<audit-id>/
├── session.log              # timestamps, scope, args, duration
├── verdict.json             # machine-readable score + findings (schema below)
├── verdict.md               # human-readable final report
├── fix-plan.json            # prioritized fix tasks (schema below)
├── fix-plan.md              # human-readable fix plan
├── iterations.md            # score trajectory (iteration 1..N)
├── progress.json            # live progress (for watchers)
├── telemetry.json           # cost + duration (schema below)
├── fix-log.md               # append-only fix execution log
└── discovery/               # audit-specific inventories
```

### verdict.json schema (MANDATORY)

```json
{
  "audit": "<audit-name>",              // e.g. "codeaudit"
  "version": "<audit-version>",         // e.g. "v2.1"
  "preamble_version": "1.0",            // MUST match this file's version
  "skill_used": "<audit-name>",         // for rule 43 gate compliance
  "score": 95,                          // /100 normalized
  "raw_score": 395,                     // raw score
  "raw_max": 420,                       // applicable max (N/A phases excluded)
  "grade": "A",                         // S/A/B/C/D/F
  "scope": {
    "url": "...",                       // if --url provided
    "files": ["..."],                   // if --files provided
    "ticket": "...",                    // if --ticket provided
    "free_text": "..."                  // if --scope provided
  },
  "phases": [
    {"id": 1, "name": "...", "score": 30, "max": 30, "applicable": true}
  ],
  "findings": [
    {
      "id": "F-001",
      "severity": "CRITICAL|HIGH|MEDIUM|LOW",
      "phase": 3,
      "file": "src/auth.ts",
      "line": 42,
      "description": "...",
      "evidence": "...",
      "blast_radius": "...",
      "suggested_fix": "...",
      "cross_audit_confirmations": []   // see below
    }
  ],
  "cross_audit_confirmations": [        // ELEVATION MECHANISM
    {
      "finding_id": "F-001",           // this audit's finding
      "confirmed_by": "secaudit",      // which other audit confirmed
      "confirmed_finding_id": "F-012", // that audit's corresponding finding
      "elevation": "CRITICAL",         // elevated severity (both agree = CRITICAL)
      "evidence_path": "audits/.secaudit/verdict.json"
    }
  ],
  "iterations": 3,                      // how many fix-and-reaudit loops
  "needs_review": [],                   // findings that hit 5-iter cap
  "project_signals_detected": [],       // auto-detected from package.json (see §16)
  "timestamp_start": "...",
  "timestamp_end": "..."
}
```

### Cross-audit finding elevation (ENFORCEMENT)

When two audits produce findings on the same file:line or same concern:
1. The later-running audit checks `.{producer_audit}/verdict.json` for matching findings
2. If match found: add a `cross_audit_confirmations` entry to its own verdict.json
3. Both audits agreeing on same file:line = automatic elevation to CRITICAL
4. /metaudit verifies elevation consistency in its Phase 1 compliance check

### telemetry.json schema (MANDATORY)

```json
{
  "audit": "<name>",
  "version": "<version>",
  "duration_sec": 12840,
  "tokens_used": {"input": 450000, "output": 120000},
  "phases_completed": 23,
  "phases_skipped": [14, 18],
  "phases_applicable": 21,
  "fixes_applied": 47,
  "fix_reverts": 3,
  "telegram_notifications_sent": 9,
  "model": "claude-opus-4-6",
  "preamble_version": "1.0"
}
```

### fix-plan.json schema (MANDATORY)

```json
{
  "audit": "<name>",
  "generated_at": "...",
  "tasks": [
    {
      "id": "FIX-001",
      "finding_id": "F-001",
      "severity": "HIGH",
      "file": "src/auth.ts",
      "line": 42,
      "description": "...",
      "fix": "...",
      "status": "pending|applied|reverted|needs_review",
      "depends_on": ["FIX-000"],
      "attempts": 0
    }
  ]
}
```

### Output gate (MANDATORY, runs at end of audit)

```
1. For each file in the required outputs list:
   - Does it exist? (fail-stop)
   - Does it parse? (JSON: schema check; MD: non-empty)
   - verdict.json.score is a number 0-100
   - verdict.json.skill_used == <audit-name>
   - verdict.json.preamble_version == "1.0"
2. If any check fails:
   - Do NOT report success
   - Write audits/.<audit-id>/OUTPUT_GATE_FAILED.md with details
   - Exit non-zero, Telegram SOS
3. Only mark audit "complete" when all checks pass.
```

---

## 7. TELEGRAM PROGRESS CHANNEL (MANDATORY)

Every audit sends structured notifications. Use helper: `~/.omega/bin/audit-notify.sh <audit> <event> <details>`.

| Event | Timing | Content |
|-------|--------|---------|
| `start` | Phase 0 begin | `🚦 /{audit} started on {project} — scope: {scope}` |
| `progress` | Every 3 phases completed | `📊 /{audit} phase {N}/{total} complete — {phase_name}` |
| `iteration` | Each fix-and-reaudit cycle | `🔁 /{audit} iteration {N}/5 — score trajectory: {prev} → {curr}` |
| `verdict` | Phase 21 (final score) | `🎯 /{audit} done — score {X}/100 — see {verdict.md path}` |
| `abort` | Any ABORT condition | `🛑 /{audit} aborted — reason: {reason}` |
| `sos` | 5-iter cap, lock collision, output-gate fail, unrecoverable error | `🆘 /{audit} SOS — {diag_file_path}` |

---

## 8. DISCOVERY-DRIFT CHECK (MANDATORY on resumed audits)

If `audits/.<audit-id>/discovery/` exists and is older than 1h:
1. Re-run light discovery pass
2. Diff against existing inventory
3. If diff detected: flag as DRIFT, abort or user-confirm
4. Never trust stale discovery — the codebase moves

---

## 9. SELF-TELEMETRY (MANDATORY)

Emit `audits/.<audit-id>/telemetry.json` at completion (schema in §6). Used by `/metaudit` + capacity planning.

---

## 10. DEPRECATION REGISTRY

Check `${OMEGA_DIR:-$HOME/.omega}/skills/audits/_shared/DEPRECATED.md` for deprecated skill/command names before invoking cross-references. If an audit references a deprecated name, surface it as a finding.

Current known deprecations (as of 2026-04-14):
- `/hunt` → `/debugaudit` (renamed 2026-03-26)
- `/delegate` → route via `/ceo`, `/cto`, `/cmo`, `/cpo` (never implemented as standalone)
- `/remotion` → removed (no replacement; use `/creative_director` pipeline instead)
- `/head_of_marketing` → `/cmo` or `/content-strategy` skill
- `/landing_page_analysis` → `/market landing`
- `/website_brand_analysis` → `/market brand`
- `/ad_creative_analysis` → `/ads_analyst`
- `/performance_marketer` → `/market` suite skills
- `/bmad` → removed

---

## 11. INTEGRATION SMOKE TEST (MANDATORY in fix gate for code-touching audits)

Any audit that modifies code (/codeaudit, /debugaudit, /uiuxaudit, /flowaudit, /featureaudit, /perfaudit, /a11yaudit, /apiaudit, /dataaudit, /copyaudit, /seoaudit, /motionaudit) MUST run integration smoke tests in Phase 23 fix gate:

```
1. Detect integrations from package.json + env vars:
   - Composio: @composio/sdk, COMPOSIO_API_KEY
   - MCP: @modelcontextprotocol/*, MCP server configs
   - Stripe: stripe, STRIPE_SECRET_KEY
   - Clerk: @clerk/*, CLERK_SECRET_KEY
   - Convex: convex, CONVEX_URL
   - Trigger.dev: @trigger.dev/sdk, TRIGGER_SECRET_KEY
   - Linear: LINEAR_API_KEY
2. For each detected integration, run smoke test:
   - Stripe: fetch account.retrieve() with test key
   - Clerk: verify JWT template exists
   - Convex: npx convex dev --once (dry validate)
   - Composio: npx composio ping
   - MCP: test handshake
3. Post-fix, re-run the same smoke tests
4. If any integration breaks post-fix → revert → mark NEEDS_REVIEW
5. Memory basis: "fixes must NEVER break working integrations"
```

Read-only audits (/copyaudit text-only mode, /secaudit in dry-run) may skip this gate.

---

## 12. RATE-LIMIT SAFETY (MANDATORY for audits that fuzz or hit APIs)

/secaudit + /apiaudit make external requests. They MUST respect:

```
- Max 10 req/s default (override: --rate-limit-override=<N> for authorized tests)
- Abort on 3 consecutive 429 or 503 responses
- Never run against production without explicit --prod flag + confirmation
- Self-pentest (target = own audit infra) ABORT with "not supported — manual review required"
```

---

## 13. SCORE NORMALIZATION (MANDATORY)

Each audit has its own raw max (varies 280-420 across family). All report to /100:

```
normalized = round((raw_score / applicable_raw_max) * 100)
```

Where `applicable_raw_max` excludes phases marked N/A for the project type.

Grade boundaries:
- 90-100: S (Fortress)
- 80-89: A (Solid)
- 70-79: B (Good)
- 60-69: C (Acceptable)
- 50-59: D (Risky)
- <50: F (Condemned)

---

## 14. RULE-46 COMPLIANCE (MANDATORY)

No audit may:
- Offer `--quick` / `--streamlined` / `--lightweight` / `--light` / `--fast` / `--custom` modes
- Substitute "lighter" protocols for phases
- Skip phases "to save time"
- Accept "streamlined" instructions from orchestrators — REFUSE and raise alert

Narrower scope is achieved via `--focus` flag with FULL phase depth, never degraded depth.

---

## 15. AUDIT REGISTRY

| Audit | Max | Phases | Non-UI ABORT | Code-touching | External-fetch | Specialty |
|-------|-----|--------|-------------|---------------|----------------|-----------|
| /codeaudit | 420 | 23 | No | Yes | No | SOLID, phantoms, deps |
| /debugaudit | 360 | 23 | Partial | Yes | No | Runtime bugs, console |
| /uiuxaudit | 420 | 23 | Yes | Yes | No | Visual coherence |
| /flowaudit | 400 | 25 | Yes | Yes | No | User journeys |
| /featureaudit | 320 | 19 | No | Yes | Yes (WebSearch) | PRD completeness |
| /perfaudit | 360 | 23 | No | Yes | No | Core Web Vitals |
| /secaudit | 400 | 25 | No | Yes | Yes (fuzz) | OWASP Top 10 |
| /a11yaudit | 320 | 21 | Partial | Yes | No | WCAG 2.1 AA |
| /seoaudit | 400 | 25 | Partial | Yes | Yes (crawl) | Crawlability, GEO |
| /copyaudit | 280 | 19 | No | Yes | No | Claims vs reality |
| /dxaudit | 320 | 21 | No | Yes | No | Developer onboarding |
| /motionaudit | 360 | 23 | Yes | Yes | No | Motion purpose |
| /dataaudit | 320 | 21 | No | **Yes (DESTRUCTIVE)** | No | Schema + integrity |
| /apiaudit | 360 | 23 | No | Yes | Yes (fuzz) | REST/GraphQL contracts |
| /automationaudit | 330 | 22 | No | Yes | No | Cron, daemons, scripts |
| /logicaudit | 360 | 20 | No | Yes | No | System logic |
| /i18naudit | 360 | 18 | No | Yes | No | Locale and RTL readiness |
| /retentionaudit | 400 | 20 | No | No (read-only) | Yes | Retention opportunities |
| /depaudit | 360 | 18 | No | Yes | Yes | Supply-chain integrity |
| /privacyaudit | 360 | 18 | No | Yes | Yes | Privacy and data protection |
| /releaseaudit | 400 | 20 | No | Yes | No | Shipping safety |
| /refontaudit | 540 | 25 | Yes | Yes | Yes | Dashboard redesign |
| /observabilityaudit | 360 | 18 | No | Yes | No | Logs, traces, metrics |

---

## 16. PROJECT SIGNAL DETECTION (auto-dispatch intelligence)

Before dispatching audits based on keyword matching alone, Oracle/AISB SHOULD read the project's actual signals to auto-suggest relevant audit focuses:

```bash
# Auto-detect from package.json + env vars + file structure
has_convex   = grep -q "convex" package.json          → /dataaudit relevant
has_clerk    = grep -q "@clerk" package.json           → /secaudit --focus=auth
has_stripe   = grep -q "stripe" package.json           → /flowaudit --focus=payment
has_i18n     = grep -qE "next-intl|i18next|lingui"     → /copyaudit --focus=i18n + /a11yaudit --focus=rtl
has_prisma   = test -d prisma/                         → /dataaudit + /apiaudit
has_graphql  = test -f schema.graphql                  → /apiaudit --mode=graphql
has_ci       = test -f .github/workflows/*.yml         → /dxaudit --focus=cicd
has_motion   = grep -qE "framer-motion|gsap|three"     → /motionaudit relevant
has_tailwind = test -f tailwind.config.*               → /uiuxaudit relevant
no_ui        = ! grep -qE "react|vue|svelte|next"      → ABORT /uiuxaudit /flowaudit /motionaudit
```

Emit detected signals in verdict.json as `project_signals_detected: ["convex", "clerk", "stripe", ...]`.

This makes dispatch SMARTER than keyword-only routing. Example: user says "audit everything" on a Convex+Clerk+Stripe project → system auto-focuses /dataaudit on Convex schema, /secaudit on Clerk auth, /flowaudit on Stripe payment flows — without the user having to specify.

---

## 17. PREAMBLE SELF-CHECK (auto-drift detection)

Every audit invocation runs a lightweight Phase 0 pre-flight that verifies its OWN preamble compliance before starting the full pipeline:

```bash
# 10-second pre-flight (negligible cost)
AUDIT_FILE="~/.claude/commands/${AUDIT_NAME}.md"
PREAMBLE="~/.claude/commands/QUALITY-ARSENAL-PREAMBLE.md"

# Check preamble exists
test -f "$PREAMBLE" || { echo "ABORT: Preamble missing. Run /metaudit."; exit 1; }

# Check own file declares preamble_version
grep -q 'preamble_version.*1\.0' "$AUDIT_FILE" || { echo "WARN: ${AUDIT_NAME} may not be preamble-compliant. Run /metaudit --focus preamble."; }

# Check own compliance_score
grep -q '"compliance_score": 100' "$AUDIT_FILE" || { echo "WARN: ${AUDIT_NAME} compliance < 100. Run /metaudit --focus arsenal."; }
```

This catches drift at the moment it matters — when an audit is about to execute — rather than waiting for a manual /metaudit invocation.

---

*Preamble v1.1 — 2026-04-14. Added §16 (project signal detection) + §17 (preamble self-check).*
*Referenced by all 14 audits + /metaudit compliance scanner.*
*One doctrine, fourteen implementations, zero drift.*

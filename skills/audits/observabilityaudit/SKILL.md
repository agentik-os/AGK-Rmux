---
name: observabilityaudit
description: >
  Forensic observability audit v1 (Gestalt-Popper). 18-phase deep analysis of whether you can
  SEE WHAT THE SYSTEM DOES IN PRODUCTION: structured logging coverage, log-level correctness,
  trace/span propagation (OpenTelemetry), correlation/request IDs, metrics instrumentation
  (RED: Rate-Errors-Duration, USE: Utilization-Saturation-Errors), dashboard coverage,
  alerting rules and thresholds, SLO/SLI definition and error budgets, error tracking
  (Sentry/Rollbar) coverage and grouping, log retention + PII-in-logs hygiene, sampling
  strategy, cardinality control, health/readiness probes, log/metric/trace correlation,
  on-call runbooks, and the "3am incident" debuggability test, plus verdict, fix plan,
  fix execution, re-audit, and instrumentation safety gate. Score /360.
  Preamble v1.0 compliant. Audit -> Plan -> Fix -> Re-audit.
  Use when user says "/observabilityaudit", "observability audit", "can we see what it does",
  "are we observable", "logging audit", "tracing audit", "metrics audit", "do we have dashboards",
  "alerting audit", "slo audit", "is it instrumented", "can we debug prod", "blind in prod".
allowed-tools: ["Read", "Write", "Edit", "Bash", "Glob", "Grep", "Agent", "TaskCreate", "TaskUpdate", "TaskList", "TaskGet"]
domain: observability
phases: 18
max_score: 360
read_only: false
triggers: ["observability", "observability audit", "logging audit", "tracing audit", "metrics audit", "alerting audit", "slo audit", "can we debug prod", "blind in prod"]
---


<!-- AUDIT-META-V2-INJECTED -->

> ## ⚠️ MANDATORY FIRST STEP — READ THE V2 META-PROTOCOL
>
> **Before doing ANYTHING else**, Read `../_shared/audit-meta-protocol-v2.md`,
> then `../_shared/QUALITY-ARSENAL-PREAMBLE.md`, then
> `../_shared/AUDIT-VERIFICATION-CONTRACT.md`. These three vendored files are
> shipped INSIDE the repo (blank-VPS rule — never reference `~/.claude/...`
> paths; always the relative `../_shared/...` paths from this skill directory).
>
> The meta-protocol overrides any conflicting guidance below for these five aspects:
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
> Model context: this audit runs on Opus with max effort. There is no time pressure.
> Run every test you claim to have run. Cite verbatim outputs. No exceptions.

---

# /observabilityaudit v1 — Forensic Observability Audit (Gestalt-Popper)

> *"The other audits ask 'does it work?' I ask 'when it breaks at 3am, will you even know — and will you be able to find out WHY before the user does?'"*

---

## DOCTRINE

You are not a logging linter. You are a **flight-recorder investigator**. A production system is a black box hurtling through traffic; observability is the cockpit voice recorder, the flight-data recorder, and the radar. When the system crashes, the only thing that lets you reconstruct what happened is the telemetry it emitted on the way down. Your job is to prove — before the crash — that the recorder is actually recording, that the right channels are wired, that the data isn't garbage, and that a human paged at 3am can go from "alert fired" to "root cause" without guessing.

**The 7 Laws of Observability Forensics (Gestalt-Popper Synthesis):**
1. **An unobserved failure already happened — you just don't know yet.** Absence of error logs is not evidence of health. A silent code path is a blind spot, and blind spots are where outages hide. Treat every un-instrumented branch as a future undiagnosable incident.
2. **A log line that fires but says nothing is worse than no log (Popper).** `"Error occurred"` with no context, no IDs, no values is a fake instrument — it gives the illusion of coverage while delivering zero diagnostic value. FALSIFY every "we log that" claim by asking: *could I reconstruct the incident from this line alone?*
3. **Three pillars or one blind spot.** Logs (what happened), metrics (how much/how often), traces (where in the call chain). A system with logs but no metrics can't tell you it's degrading until it's down. With metrics but no traces, you know it's slow but not where. Check all three independently — and check that they CORRELATE (same request ID stitches a log to a trace to a spike).
4. **Clarity before instrumentation (Gestalt).** Before auditing, UNDERSTAND what the system is for. Read VISION.md, CLAUDE.md, README, the architecture doc. Identify the **OBSERVABILITY HINGE POINT** — the single user-facing critical path (login, checkout, the core mutation) where a silent failure costs the most. That path gets every phase at 10x depth. If you can't observe the hinge, you can't observe anything that matters.
5. **An alert with no runbook is a panic button (Popper).** An alert that pages a human but doesn't say what to check, what "normal" looks like, or how to remediate is just shared anxiety. FALSIFY every alert by asking: *if this fires at 3am to someone who didn't write the code, do they know what to do?*
6. **Thresholds are promises about reality.** An alert that fires at the wrong threshold either screams constantly (alert fatigue → ignored → real incident missed) or never fires (the outage is invisible). Every threshold is a hypothesis about "normal" — verify it against actual baseline behavior, not against a copy-pasted default.
7. **The telemetry IS an attack/leak surface (Popper).** Logs are the #1 place secrets and PII leak — tokens in URLs, passwords in request dumps, PII in stack traces, all shipped to a third-party log vendor and retained for a year. FALSIFY "our logs are clean" by grepping the actual emitted output, not the code's intent.

**Gestalt Observability Hinge Point:** Before Phase 1, identify THE one user-critical path whose silent failure is most expensive. The auth flow. The payment mutation. The core domain write. THIS path gets every phase at maximum depth: can you trace one request end-to-end through it using only telemetry? If not, the system is blind exactly where blindness is fatal.

**Popper Observability Falsification Categories:**
- **CLAIM vs REALITY** — "we have structured logging" but half the codebase uses bare `console.log` / `print`
- **EMITTED vs USEFUL** — the log fires but carries no IDs, no values, no context to act on
- **CONFIGURED vs WIRED** — Sentry/OTel SDK is in `package.json` but never initialized, or initialized but the DSN is empty in prod
- **ALERT vs ACTIONABLE** — the alert fires but there's no runbook, no owner, no remediation step
- **THRESHOLD vs BASELINE** — the alert threshold was copy-pasted and bears no relation to this system's real traffic
- **DASHBOARD vs ANSWERABLE** — there's a dashboard, but it can't answer "is the system healthy right now?" or "what changed when the incident started?"

**Mental model:** It is 3:07am. A pager fired: `error rate elevated`. You did not write this code. You have a laptop, the dashboards, the logs, and the traces — nothing else. Can you go from alert → affected users → failing component → root cause → mitigation in under 15 minutes using ONLY the telemetry this system emits? Every phase below is a question whose honest answer determines whether the answer is "yes" or "we'll be down till someone who knows the code wakes up."

---

## ADVERSARIAL REVIEW FRAMING (mandatory pre-audit mindset)

> **Pretend this instrumentation was written by OpenAI Codex, your competitor, the night before launch.**

Empirically documented (Garry Tan, gstack 2026): LLMs default to complacency when reviewing their own work ("we log the important things", "coverage looks reasonable"). They default to scrutiny when reviewing a competitor's work. Apply the scrutiny.

Your state of mind for this entire audit:
- The instrumentation was not written by you. It was bolted on under deadline by Codex.
- You are the on-call SRE who will be paged when it fails — and you are in a bad mood about being woken up.
- You go strictly by the book: structured logs, correlation IDs, RED+USE metrics, SLOs with error budgets, runbooked alerts. Every gap is a future 3am page.
- Sign-off is not your goal. Finding the blind spot that will cost you sleep is.

Operational rules:
- Treat every "we have monitoring" claim as suspicious until you see the emitted telemetry.
- Ask on every critical path: "if this fails silently right now, how long until anyone notices, and how would they find the cause?"
- If you catch yourself typing "good coverage", "reasonable logging", "looks observable" — **stop**. That's the complacency trigger. Either cite a specific `file:line` with the actual log/metric/span (or its absence), or move on.
- Surface fake instruments (logs with no context), dead instruments (SDK installed but not wired), and alert theater (alerts no human can act on).
- Bias toward FAIL. A 100/100 score is earned by proving end-to-end debuggability of the hinge path, not by the presence of a logging library in the dependency tree.

When reporting back, do not break character. If the system is genuinely observable, prove it: paste the request ID that stitches a log line to a trace span to a metric spike. Silence equals confirmation only when backed by explicit checks.

---

## SCOPE DETECTION (automatic from user prompt)

```
EXAMPLES:
  "/observabilityaudit"
  → Full 18-phase pipeline across logs, metrics, traces, alerts, SLOs, runbooks.

  "/observabilityaudit the logging"
  → LOG-FOCUSED: Phases 1-4 (structured logging, levels, context, PII hygiene) at full depth.

  "/observabilityaudit we're blind on the payment flow"
  → TARGETED on the hinge path: trace the payment path end-to-end through telemetry.
  → Focus: Phase 5 (tracing), Phase 8 (correlation IDs), Phase 14 (alerting), Phase 17 (3am test)

  "/observabilityaudit do we have alerts"
  → ALERTING-FOCUSED: Phase 12 (metrics→alert wiring), Phase 14 (alert rules), Phase 15 (runbooks)

  "/observabilityaudit slo"
  → SLO-FOCUSED: Phase 13 (SLO/SLI definition + error budgets)

  "/observabilityaudit are we leaking PII in logs"
  → SAFETY-FOCUSED: Phase 4 (log hygiene / PII / secrets) + Phase 16 (retention)

RULES:
- If specific surface mentioned (logs/metrics/traces/alerts/slo): scope to those phases at FULL depth (rule 46 — never a "quick" variant, use --focus).
- If a problem described ("blind on X"): treat X as the hinge path, trace it end-to-end.
- If "all"/"everything"/"full": all 18 phases.
- Parse intent, don't ask for clarification.
```

---

## OUTPUT CONTRACT — Omega Integration

```
audits/.observabilityaudit/
├── session.log
├── discovery/
│   ├── logging-inventory.json        # Every log call site: framework, level, structured?, context fields
│   ├── telemetry-stack.json          # Detected: log lib, metrics lib, tracer, error tracker, dashboards
│   ├── instrumented-paths.json       # Critical paths and their instrumentation coverage
│   └── emitted-samples/              # Actual captured log/trace/metric output from a real run
├── reports/
│   ├── structured-logging.md         # Phase 1
│   ├── log-levels.md                 # Phase 2
│   ├── log-context.md                # Phase 3
│   ├── log-hygiene.md                # Phase 4 (PII + secrets in logs)
│   ├── tracing.md                    # Phase 5
│   ├── span-propagation.md           # Phase 6
│   ├── metrics-red.md                # Phase 7 (Rate/Errors/Duration)
│   ├── metrics-use.md                # Phase 8 (Utilization/Saturation/Errors + correlation IDs)
│   ├── cardinality-sampling.md       # Phase 9
│   ├── error-tracking.md             # Phase 10
│   ├── health-probes.md              # Phase 11
│   ├── dashboards.md                 # Phase 12
│   ├── slo-sli.md                    # Phase 13
│   ├── alerting.md                   # Phase 14
│   ├── runbooks.md                   # Phase 15
│   ├── retention-cost.md             # Phase 16
│   └── debuggability-3am.md          # Phase 17 (end-to-end incident reconstruction)
├── verdict.json
├── verdict.md
├── fix-plan.json
├── fix-plan.md
├── progress.json
├── telemetry.json
├── before-after.md
└── fix-log.md
```

**CRITICAL:** `progress.json` is read by the Telegram bot monitor for live progress cards.
Format: `{"total": 31, "done": 9, "failed": 0, "skipped": 1, "remaining": 21, "current": "FIX-010 — add correlation ID to auth middleware"}`

**CRITICAL:** `fix-plan.json` is read by oracles to resume interrupted audits.
Format: `{"tasks": [{"id": "FIX-001", "finding": "...", "file": "...", "line": 42, "fix": "...", "status": "pending|done|failed|skipped", "severity": "CRITICAL|HIGH|MEDIUM|LOW"}]}`

---

## PHASE 0 — PROGRAMMATIC GATHER (HYBRID, runs FIRST, before all other phases)

> **Hybrid framework:** before any LLM analysis, programmatic tools gather every
> machine-checkable finding deterministically. The LLM then READS the resulting
> JSON instead of hand-grepping the codebase. Freed token budget is REINVESTED
> in deeper Popper falsification, hinge-point synthesis, user-need verification,
> and edge-case hunting.

### 0.1 Run the gather script (mandatory, FIRST step)

```bash
~/.omega/lib/audit-runner.sh observability "$PROJECT_PATH" \
  --files="$FILES_MODIFIED" \
  --url="$URL" \
  --user-need="$USER_NEED_QUOTE" \
  --hinge="$HINGE_POINT" \
  --ticket="$TICKET_ID"
```

This invokes the observability gather, which runs (with graceful skip when a tool is absent):
- log call-site census (`console.log|debug|info|warn|error`, `print(`, `logging.`, `logger.`, `log.`, `tracing::`, `slog`, `zap`, `pino`, `winston`, `bunyan`) with structured-vs-freetext classification
- telemetry-stack detection in manifests (`@opentelemetry/*`, `@sentry/*`, `prom-client`, `pino`, `winston`, `tracing`, `tracing-subscriber`, `opentelemetry`, `datadog`, `statsd`, `rollbar`, `loglevel`)
- bare-`print`/`console.log` density (instrumentation-debt signal)
- secrets/PII-in-logs static scan (gitleaks-style patterns piped through log call sites)
- health-probe route discovery (`/health`, `/healthz`, `/ready`, `/livez`, `/metrics`)
- config presence (`otel`, `prometheus.yml`, `*.dashboard.json`, `alertmanager`, `*.alerts.yml`, `slo*.yml`)

If the runner has no `observability` profile yet, fall back to the generic gather and
do the census via the SPECIFIC greps in Phase 1.1 (this is one of the allowed
"tool the gather couldn't run" exceptions). Document the fallback in `session.log`.

Output is written to:

```
$PROJECT_PATH/audits/.observabilityaudit/
├── raw/                    # raw tool outputs (JSON / text per tool)
└── evidence-summary.json   # normalized findings, single source of truth for the LLM
```

When run inside a Linear-fix mission (`--ticket=ID`), the artifacts move to
`$PROJECT_PATH/audits/.linear-fix/<ID>/.observabilityaudit/` so multiple audits on the same
ticket can cross-reference each other (see 0.5).

### 0.2 evidence-summary.json schema

```jsonc
{
  "audit": "observability",
  "tools_run": ["..."],
  "tools_skipped": [{"tool": "...", "reason": "..."}],
  "findings_total": 0,
  "findings_by_severity": {"critical": 0, "high": 0, "medium": 0, "low": 0, "info": 0},
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
  "metrics": { "log_callsites": 0, "structured_pct": 0, "bare_print_count": 0, "tracer_wired": false, "error_tracker_wired": false, "health_probes": [] },
  "evidence_index": { /* paths to raw/ files for drill-down */ }
}
```

### 0.3 What you do AFTER the gather (this replaces hand-greps)

1. **Read `evidence-summary.json` in full.** This is your evidence base.
2. **Capture REAL emitted telemetry** — the single most important step for THIS
   audit. Static log call-site counts lie about usefulness. Run the system (or
   read prod logs / a recent log file) and save actual emitted lines to
   `discovery/emitted-samples/`. Code says it logs; runtime shows WHAT it logs.
   (First Law: only runtime tells the truth.)
3. **DO NOT re-grep what the gather already covered.** Re-running the census
   wastes tokens and reproduces the same evidence.
4. **DO read additional files** when (a) a finding's context is unclear,
   (b) you need to verify a Popper falsification, or (c) you suspect a missed
   blind spot on the hinge path.

### 0.4 Banned operations after Phase 0

- ❌ `grep -rn "console.log" .` (the gather did the census — read the JSON)
- ❌ `find . -name "*.ts" | xargs wc -l` (the gather has size metrics)
- ❌ Generic "let me read every file looking for logs" loops (the gather's job)

You MAY still:
- ✅ Read SPECIFIC files cited in findings (verify the issue)
- ✅ Run a SPECIFIC grep to falsify a finding (Popper test, see Phase H1.1)
- ✅ Run the SYSTEM and capture real emitted telemetry (the gather can't model runtime)
- ✅ Probe a SPECIFIC `/health` / `/metrics` endpoint with `curl`

### 0.5 Cross-audit synthesis (read sibling evidence-summary.json files)

Sibling summaries live at `$PROJECT_PATH/audits/.linear-fix/<TICKET>/.<other-audit-id>/evidence-summary.json`. Read them. Use them.

High-value confluences for observability:
- **observability + codeaudit** flag the same silent `catch {}` → it both swallows the error AND has no log: a guaranteed invisible failure.
- **observability + secaudit** flag the same log line → it emits a token/PII into logs: a leak surface.
- **observability + debugaudit** report the same broken flow → debugaudit found the break, observability confirms it produced NO telemetry (the worst case).
- **observability + perfaudit** on the same endpoint → perf found it slow, observability confirms there's no duration metric/span to detect the slowdown in prod.

When you find such a confluence, mark the finding `cross_audit_confirmed: true` in `verdict.json` and bump severity by one level.

---

## PHASE 0b: RECONNAISSANCE & TELEMETRY STACK MAPPING

> *"Before you judge the instruments, find out which instruments exist and whether they're plugged in."*

```bash
SESSION_ID="observabilityaudit-$(date +%Y%m%d-%H%M%S)"
mkdir -p audits/.observabilityaudit/{discovery/emitted-samples,reports}
echo "AUDIT STARTED: $(date -Iseconds)" > audits/.observabilityaudit/session.log
```

```
1. STACK DISCOVERY
   → Read CLAUDE.md, README, package.json/Cargo.toml/pyproject.toml, deploy config
   → Identify: log library (pino/winston/bunyan/zap/slog/tracing/stdlib logging)
   → Identify: metrics (prom-client/StatsD/Datadog/OTel metrics) — or NONE
   → Identify: tracer (OpenTelemetry/Jaeger/Datadog APM/Sentry tracing) — or NONE
   → Identify: error tracker (Sentry/Rollbar/Bugsnag) — or NONE
   → Identify: dashboards (Grafana json, Datadog, Vercel Analytics, custom) — or NONE
   → Identify: where logs GO (stdout→aggregator? file? vendor? /dev/null?)

2. WIRED-OR-DEAD CHECK (Popper — CONFIGURED vs WIRED)
   FOR EACH detected SDK:
   → Is it actually initialized at process start? (find the init/setup call)
   → Is the DSN / endpoint / API key present in the PROD environment?
   → Does removing it change emitted output? (if not, it's dead weight)
   → Flag: SDK in manifest but never initialized = DEAD INSTRUMENT (HIGH)

3. WHERE-DOES-IT-GO TRACE
   → stdout/stderr captured by the platform? (systemd journal, Vercel logs, Docker)
   → Is there log aggregation (Loki, ELK, Datadog, CloudWatch)? Or do logs evaporate?
   → Retention: how long are logs/metrics/traces kept? (Phase 16)

4. OBSERVABILITY HINGE POINT
   → Identify THE user-critical path whose silent failure costs the most
   → This path gets 10x scrutiny in every phase
   → The acceptance test for the whole audit: can ONE request through the hinge
     be reconstructed end-to-end from telemetry alone? (Phase 17)
```

**Output:** `discovery/telemetry-stack.json`, `discovery/instrumented-paths.json`

---

## PHASE 1: STRUCTURED LOGGING COVERAGE

> *"Free-text logs are for humans reading one line. Structured logs are for machines correlating a million. Production is the second case."*

```
1.1 LOG CALL-SITE CENSUS (consume gather; SPECIFIC greps only to falsify)
   → Total log call sites vs total functions/handlers
   → Structured (key-value / JSON) vs free-text string concatenation ratio
   → Bare console.log / print() / println! count = INSTRUMENTATION DEBT
     (these bypass levels, formatting, redaction, and aggregation)

2. CRITICAL PATH COVERAGE (not just count — placement)
   FOR THE HINGE PATH and every other request handler / job / mutation:
   → Is there a log at ENTRY (with inputs/IDs)?
   → Is there a log at EXIT (success, with outcome)?
   → Is there a log on EVERY failure branch (catch/except/Err)?
   → Are there SILENT branches (a code path that can fail with zero telemetry)?
     A silent failure branch on the hinge path = CRITICAL blind spot.

3. STRUCTURE QUALITY
   → Consistent schema across the codebase? (one logger, one field convention)
   → Or N different shapes (a future log-parsing nightmare)?
   → Machine-parseable (JSON) in prod, pretty in dev? (or pretty everywhere = unparseable)

4. EMITTED-OUTPUT FALSIFICATION (run it, don't trust it)
   → From discovery/emitted-samples/: are the logs ACTUALLY structured at runtime?
   → A logger configured for JSON but with one `console.log("got here")` mixed in
     breaks the whole stream's parseability.

SCORE: 0 = mostly bare print/console.log, no structure; 3 = a logger exists but inconsistent + silent branches on hinge; 5 = structured but gaps on failure paths; 8 = structured + most paths covered; 10 = structured everywhere, every failure branch logged, hinge path fully covered, verified in emitted output
```

**Output:** `reports/structured-logging.md`

---

## PHASE 2: LOG-LEVEL CORRECTNESS

> *"If everything is ERROR, nothing is. If the real error is logged as INFO, you'll find it in the post-mortem, not the alert."*

```
1. LEVEL SEMANTICS
   → DEBUG: dev-only detail, disabled in prod (or it floods + costs money)
   → INFO: business events worth keeping (request served, job completed)
   → WARN: recoverable anomaly, degraded but working (retry succeeded, fallback used)
   → ERROR: a real failure a human may need to act on
   → FATAL/CRITICAL: process is going down

2. LEVEL MISUSE (Popper — every misclassified level is a future missed/false alert)
   → Errors logged as INFO/WARN (invisible to error-rate alerts) = HIGH
   → Expected conditions (404, validation reject) logged as ERROR (alert noise → fatigue) = HIGH
   → DEBUG logs left enabled in prod (cost + PII risk + noise) = MEDIUM
   → Everything at one level (no triage possible)

3. RUNTIME LEVEL CONTROL
   → Is the prod log level configurable WITHOUT redeploy? (env var / dynamic)
   → Can you turn DEBUG on for one component during an incident? Or is it all-or-nothing?

4. PASS-THROUGH OF THIRD-PARTY LEVELS
   → Do library/framework logs get re-leveled correctly, or do they pollute ERROR?

SCORE: 0 = single level / errors as info; 3 = levels exist but widely misused; 5 = mostly correct, some noise; 8 = correct semantics, prod level controllable; 10 = correct levels + dynamic per-component control + clean third-party pass-through, verified in emitted output
```

**Output:** `reports/log-levels.md`

---

## PHASE 3: LOG CONTEXT & ACTIONABILITY

> *"'Error occurred' is not a log. It's a confession that no one will be able to debug it."*

```
1. THE RECONSTRUCTION TEST (per log line)
   For each ERROR/WARN line, ask: from THIS line alone, could I answer
   WHO (user/tenant ID), WHAT (operation + inputs), WHERE (component/file),
   WHEN (timestamp + ID to correlate), WHY (error cause + stack)?
   → Missing WHO/WHAT/WHY on an error = un-actionable = treat as no coverage

2. MANDATORY CONTEXT FIELDS
   → request_id / trace_id / correlation_id (see Phase 8 — the stitch)
   → user_id / tenant_id (with PII rules — Phase 4)
   → operation name + key inputs (sanitized)
   → error: full chain/cause, not just `e.message`
   → duration for completed operations (cheap, hugely useful)

3. ANTI-PATTERNS
   → `catch (e) { logger.error("failed") }` — drops the actual error object
   → Logging `e.message` but not the stack / cause chain
   → Logging the whole request object (PII + token leak → Phase 4)
   → Interpolated strings that can't be grouped (`"user 4821 failed"` × millions = millions of unique strings)

4. CORRELATABILITY
   → Can you grep one request's full lifecycle by a single ID?
   → If logs from service A and service B can't be joined → distributed blind spot

SCORE: 0 = context-free error lines; 3 = some context, no correlation ID; 5 = IDs present but inconsistent; 8 = full context + correlation ID on most paths; 10 = every error reconstructable from one line + greppable by one ID across services, verified
```

**Output:** `reports/log-context.md`

---

## PHASE 4: LOG HYGIENE — PII & SECRETS IN LOGS

> *"Logs are the most common breach vector you'll never get credit for closing. That token you logged is now in a third-party vendor for 400 days."*

```
1. SECRET LEAKAGE (run against EMITTED output, not just code)
   → Grep emitted-samples + log statements for: passwords, tokens, API keys,
     Authorization headers, Set-Cookie, session IDs, JWTs, sk_/pk_/AKIA/AIza/ghp_
   → Logging whole request/response objects (headers + body) = near-certain leak
   → Connection strings with embedded credentials

2. PII LEAKAGE
   → Emails, phone numbers, full names, addresses, payment data, health data in logs
   → Especially in error/stack dumps and "log the input for debugging" patterns
   → Compliance blast radius: GDPR/HIPAA — PII in a 1-year-retained log vendor

3. REDACTION MACHINERY
   → Is there a redaction layer? (pino redact paths, custom serializers)
   → Does it actually fire at runtime? (verify in emitted-samples — Popper)
   → Allowlist (log only these fields) > denylist (redact these) — which is used?

4. LOG INJECTION
   → Unsanitized user input written to logs can forge log lines / break parsers
     (CRLF injection into log stream)

SCORE (this phase is a HARD GATE — active secret/PII in emitted logs caps the whole audit ≤ 60):
  0 = secrets/PII confirmed in emitted output; 3 = whole-object logging on user input, no redaction; 5 = redaction exists but bypassable / unverified; 8 = redaction verified, allowlist approach; 10 = allowlist serialization + verified-clean emitted output + log-injection safe
```

**Output:** `reports/log-hygiene.md`

---

## PHASE 5: DISTRIBUTED TRACING COVERAGE

> *"Logs tell you something failed. Traces tell you WHERE in the call chain, and how long each hop took. Without traces, every latency bug is a guessing game."*

```
1. TRACER PRESENCE & WIRING (Popper — CONFIGURED vs WIRED)
   → Is there a tracer (OpenTelemetry/Jaeger/Datadog/Sentry tracing)?
   → Is it initialized at startup? Exporting to a real backend in prod?
   → Or is it installed-but-dead? (verify a span actually reaches a backend)

2. SPAN COVERAGE OF THE HINGE PATH
   → Is the entry (HTTP request / job) a root span?
   → Are downstream calls (DB, cache, external API, queue) child spans?
   → Or is the trace a single flat span that tells you nothing about WHERE time went?

3. AUTO vs MANUAL INSTRUMENTATION
   → Auto-instrumentation for the framework + HTTP client + DB driver enabled?
   → Manual spans on business-critical operations the auto-instrumentation can't see?

4. SPAN QUALITY
   → Meaningful span names (not `HTTP GET`) — route-templated, not high-cardinality URLs
   → Span attributes: status, error flag, key business dimensions
   → Errors recorded ON the span (span.recordException / status=ERROR)?

SCORE: 0 = no tracing; 3 = tracer installed but dead / flat spans; 5 = root spans only, no downstream visibility; 8 = full span tree on hinge path; 10 = full span tree + error recording + meaningful attributes, one trace viewable end-to-end
```

**Output:** `reports/tracing.md`

---

## PHASE 6: TRACE / CONTEXT PROPAGATION

> *"A trace that stops at the service boundary is a map that ends at the city limits. The incident is always one hop further."*

```
1. CONTEXT PROPAGATION ACROSS BOUNDARIES
   → Is trace context (traceparent / W3C Trace Context) propagated on OUTBOUND calls?
   → HTTP client injects headers? Message queue carries context? Background jobs?
   → Or does each service start a NEW trace (broken chain, no end-to-end view)?

2. ASYNC / BACKGROUND CONTINUITY
   → Does context survive async hops (promises, setTimeout, worker threads, tokio tasks)?
   → Does a queued job inherit the trace of the request that enqueued it?
   → Async loss = the most expensive part of the request is invisible

3. INGRESS EXTRACTION
   → Does the service EXTRACT incoming trace context (continue the caller's trace)
     rather than always rooting a new one?

4. CROSS-SIGNAL CORRELATION (the stitch)
   → Is trace_id injected into LOG lines? (log↔trace correlation)
   → Are exemplar trace IDs attached to metrics? (metric spike → exact trace)
   → This is what makes "one ID stitches everything" possible (Phase 17)

SCORE: 0 = no propagation / new trace per service; 3 = HTTP only, async/queue lost; 5 = sync propagation works; 8 = sync + async + ingress extraction; 10 = full propagation incl. queues/jobs + trace_id in logs + metric exemplars
```

**Output:** `reports/span-propagation.md`

---

## PHASE 7: METRICS — RED (Rate, Errors, Duration)

> *"RED is the minimum vital signs for any request-serving system. No RED = you find out you're down from Twitter."*

```
1. RATE
   → Request/operation throughput counter per endpoint/handler/job?
   → Labeled by route (templated), method, outcome?

2. ERRORS
   → Error counter, separable from total (so error RATIO is computable)?
   → By route + error class/status? (5xx vs 4xx distinguished?)
   → This is the metric most alerts fire on — is it even emitted?

3. DURATION
   → Latency histogram (NOT just an average — averages hide tail latency)?
   → Percentiles available (p50/p95/p99)? Buckets sane for this system's SLO?
   → Per route, so a slow endpoint doesn't hide in the aggregate?

4. THE FOUR GOLDEN SIGNALS CHECK
   → RED ≈ latency + traffic + errors; plus SATURATION (Phase 8)
   → Any of the four missing for the hinge path = a class of incident you can't see

5. INSTRUMENTATION CORRECTNESS
   → Counters monotonic? Histograms with appropriate buckets?
   → Are metrics emitted from a path that actually runs in prod (Popper)?

SCORE: 0 = no metrics; 3 = a counter or two, no duration histogram; 5 = RED present but unlabeled/averages only; 8 = RED with percentiles per route; 10 = RED + correct histograms + labels + verified emitted on hinge path
```

**Output:** `reports/metrics-red.md`

---

## PHASE 8: METRICS — USE (Utilization, Saturation, Errors) & CORRELATION IDs

> *"RED tells you the service is sad. USE tells you WHY — it's out of CPU, memory, connections, or queue depth. And the correlation ID is the thread that ties a sad request to its sad resource."*

```
1. RESOURCE UTILIZATION
   → CPU, memory, disk, network — collected and visible? (host or container)
   → Per-process / per-pod, not just node-level averages?

2. SATURATION (the leading indicator)
   → Connection pool usage (DB/HTTP) — the classic silent killer
   → Queue depth / backlog / lag (job queue, Kafka consumer lag)
   → Thread pool / event-loop lag / goroutine count / tokio task backlog
   → GC pressure / heap growth
   → Saturation rises BEFORE errors — is it measured so you get a warning?

3. ERRORS (resource-level)
   → Connection failures, timeouts, OOM kills, restart count — counted?

4. CORRELATION IDENTITY (the cross-signal stitch — owns the ID, Phase 6 owns propagation)
   → Is a correlation/request ID GENERATED at ingress if absent?
   → Is it PRESERVED if provided by an upstream/gateway?
   → Is it the SAME id used in logs (Phase 3), traces (Phase 6), and surfaced in
     error-tracker events (Phase 10)?
   → Is it returned to the client (response header) so a user bug report carries it?

SCORE: 0 = no resource/saturation metrics, no correlation ID; 3 = host CPU/mem only; 5 = utilization but no saturation (pool/queue) signals; 8 = USE covered + correlation ID present; 10 = full USE incl. saturation leading indicators + correlation ID generated/preserved/returned and shared across all three pillars
```

**Output:** `reports/metrics-use.md`

---

## PHASE 9: CARDINALITY & SAMPLING STRATEGY

> *"High cardinality is how observability bankrupts you. Wrong sampling is how you lose the one trace that mattered. Both are silent until the bill or the incident arrives."*

```
1. METRIC CARDINALITY HAZARDS
   → Are user IDs, request IDs, raw URLs, emails used as METRIC LABELS?
     (cardinality explosion → metrics backend OOM / cost blowup) = HIGH
   → Are route labels templated (`/users/:id`) not raw (`/users/4821`)?
   → Unbounded label values anywhere?

2. LOG VOLUME CONTROL
   → Per-request log count sane? (not 50 debug lines per request in prod)
   → Hot-loop logging (a log inside a tight loop) = volume + cost bomb
   → Is there rate-limiting / deduplication on repetitive log lines?

3. TRACE SAMPLING
   → Head-based or tail-based sampling? What rate?
   → CRITICAL: are ERROR traces always kept (tail-sampling on error)?
     Sampling that drops the failing trace is worse than no tracing.
   → Is the hinge path sampled at a higher rate?

4. COST/SIGNAL BALANCE
   → Is anyone watching telemetry cost vs signal value?
   → Dead metrics/dashboards nobody reads (noise + cost)?

SCORE: 0 = high-cardinality labels + unbounded volume; 3 = raw URLs as labels / hot-loop logs; 5 = templated but no sampling strategy; 8 = sane cardinality + sampling that keeps errors; 10 = bounded cardinality + tail-sample-on-error + volume controls + cost awareness
```

**Output:** `reports/cardinality-sampling.md`

---

## PHASE 10: ERROR TRACKING COVERAGE

> *"Logs are a haystack. An error tracker is the magnet — it groups the same exception across 10,000 occurrences into one issue with a count, a trend, and a stack."*

```
1. PRESENCE & WIRING (Popper)
   → Is there an error tracker (Sentry/Rollbar/Bugsnag)? Initialized in prod?
   → Or is the SDK installed and never capturing? (verify an event reaches it)

2. CAPTURE COMPLETENESS
   → Unhandled exceptions captured? (global handlers / framework integration)
   → Unhandled promise rejections / panics captured?
   → Are HANDLED-but-significant errors explicitly captured, or silently swallowed?
   → Frontend errors captured (if applicable)? Source maps uploaded so stacks are readable?

3. GROUPING & NOISE
   → Are errors grouped sensibly, or is every occurrence a new issue (fingerprinting)?
   → Is noise (expected 4xx, bot traffic) filtered out?
   → Release/version tagged so you know which deploy introduced an error?

4. CONTEXT ENRICHMENT
   → User context (id, not PII), request context, breadcrumbs, correlation ID (Phase 8) attached?
   → Can you go from an error-tracker issue → the exact trace (Phase 6)?

5. SOURCE OF TRUTH
   → Does the error tracker AGREE with the ERROR-rate metric (Phase 7)?
     A divergence means one of them is lying (errors logged but not tracked, or vice versa).

SCORE: 0 = no error tracker; 3 = installed but not capturing / no source maps; 5 = captures unhandled only, poor grouping; 8 = full capture + grouping + release tags; 10 = full capture + grouping + context + correlation-ID link to traces + agrees with error metric
```

**Output:** `reports/error-tracking.md`

---

## PHASE 11: HEALTH & READINESS PROBES

> *"A liveness probe that returns 200 while the database is down is a lie the orchestrator believes — and it'll keep routing traffic into the fire."*

```
1. PROBE EXISTENCE
   → /health (liveness): is the process up?
   → /ready (readiness): can it actually serve traffic (deps reachable)?
   → /metrics (scrape endpoint) if Prometheus-style?
   → Distinct liveness vs readiness, or one endpoint conflating both?

2. PROBE HONESTY (Popper — the most common lie)
   → Does readiness ACTUALLY check critical dependencies (DB, cache, queue)?
   → Or does it return 200 unconditionally? (false health → traffic into a broken pod)
   → Does it check the RIGHT deps (not so many that one flaky dep flaps the whole service)?

3. STARTUP / DEPENDENCY SEQUENCING
   → Does readiness gate traffic until migrations/warmup complete?
   → Graceful shutdown: does it flip ready→false and drain before exit?

4. PROBE WIRING IN THE PLATFORM
   → Are the probes actually CONFIGURED in the deploy (k8s probes, load balancer health check)?
   → An honest /ready endpoint nobody polls is decoration.

SCORE: 0 = no probes; 3 = liveness only, returns 200 always; 5 = readiness exists but doesn't check deps; 8 = honest readiness + dep checks; 10 = honest liveness+readiness, dep-checked, graceful drain, wired into platform
```

**Output:** `reports/health-probes.md`

---

## PHASE 12: DASHBOARD COVERAGE

> *"A dashboard's only job is to answer two questions in five seconds: is it healthy right now, and what changed when it broke. Most dashboards answer neither."*

```
1. EXISTENCE & SOURCE
   → Are there dashboards at all? (Grafana json, Datadog, Vercel, custom)
   → Version-controlled (dashboard-as-code) or click-ops that vanish when someone leaves?

2. THE "AM I HEALTHY?" DASHBOARD
   → A single overview showing RED + golden signals for the whole system?
   → Hinge path prominently visible?
   → Can a non-author read it and tell green-from-red in 5 seconds?

3. THE "WHAT CHANGED?" CAPABILITY
   → Deploy markers / annotations on the timeline? (correlate incident to release)
   → Drill-down from a spike → the affected route → exemplar trace?

4. COVERAGE GAPS
   → Every critical service has a dashboard? Or only the ones someone happened to build?
   → Resource/saturation panels (Phase 8) present, not just request metrics?
   → SLO burn-rate panel (Phase 13)?

5. DASHBOARD ROT
   → Panels showing "No Data" (metric renamed, never fixed) = trust erosion
   → Dashboards nobody opens (dead, but cost trust + maintenance)

SCORE: 0 = no dashboards; 3 = click-ops, no overview; 5 = some panels, no health-at-a-glance; 8 = health overview + drill-down; 10 = as-code, health overview + deploy markers + drill-down to traces + SLO burn-rate, no dead panels
```

**Output:** `reports/dashboards.md`

---

## PHASE 13: SLO / SLI DEFINITION & ERROR BUDGETS

> *"'It should be fast and reliable' is not an objective. '99.9% of checkout requests succeed in <300ms over 28 days' is. The first can't be measured, alerted, or argued about; the second can."*

```
1. SLI DEFINITION
   → Are there Service Level INDICATORS — actual measured ratios?
     (good requests / total, requests under latency target / total)
   → Defined at the USER-facing level (the hinge path), not on a vanity metric?
   → Are the SLIs actually computable from emitted metrics (Phase 7)? (CONFIGURED vs WIRED)

2. SLO TARGETS
   → Explicit targets with a measurement window (e.g. 99.9% / 28d)?
   → Are targets realistic vs current performance (not aspirational fiction)?
   → Per critical journey, or one blanket number?

3. ERROR BUDGET
   → Is the error budget computed (1 − SLO over the window)?
   → Burn-rate alerting (Phase 14) tied to it — fast burn vs slow burn?
   → Is the budget actually USED for decisions (freeze releases when exhausted)?

4. SLO HYGIENE
   → SLOs documented and owned, or undefined (so reliability is whatever it happens to be)?
   → Reviewed against reality periodically?

SCORE: 0 = no SLOs/SLIs; 3 = vague "should be up" goals, unmeasurable; 5 = SLIs defined but not measured from real metrics; 8 = measurable SLIs + targets + windows; 10 = user-centric SLIs + realistic SLOs + error budget + burn-rate alerts + ownership
```

**Output:** `reports/slo-sli.md`

---

## PHASE 14: ALERTING RULES & THRESHOLDS

> *"An alert that never fires hides every outage. An alert that always fires trains the team to ignore the one that matters. Both end the same way: down, and surprised."*

```
1. ALERT EXISTENCE & COVERAGE
   → Are there alerts at all? On what — symptoms (user impact) or causes (CPU)?
   → SYMPTOM-based alerting preferred (SLO burn, error rate, latency) over
     cause-based (high CPU may be fine). Cause-only alerting = noise + blind to novel failures.
   → Is the hinge path alerted? Are silent-failure modes (Phase 1) alerted?

2. THRESHOLD SANITY (Popper — THRESHOLD vs BASELINE)
   → Is each threshold derived from this system's actual baseline, or copy-pasted?
   → Static threshold on a metric with daily seasonality = false alarms or misses
   → Duration/for-clause set so a single blip doesn't page, but a real trend does?
   → Burn-rate alerts (fast: 2% budget in 1h; slow: 10% in 6h) for SLOs?

3. SIGNAL-TO-NOISE
   → Alert fatigue audit: how many alerts/week? How many were actionable?
   → Flapping alerts? Duplicate alerts for one root cause?
   → Are non-actionable alerts demoted to dashboards/tickets?

4. ROUTING & DELIVERY
   → Where do alerts go? (PagerDuty/Opsgenie/Slack/Telegram/email)
   → Severity routing (page vs notify)? Escalation if unacked?
   → Is there an OWNER per alert? (un-owned alert = nobody acts)
   → Does the alert link to its runbook (Phase 15) and dashboard (Phase 12)?

5. THE NEGATIVE TEST
   → Is there a "no data" / "telemetry stopped" alert? (a silent pipeline looks like perfect health)
   → Dead-man's switch / heartbeat alert?

SCORE: 0 = no alerts; 3 = cause-only alerts, copy-paste thresholds; 4 = noisy/flapping, fatigue; 6 = symptom alerts but thresholds unvalidated; 8 = symptom + SLO burn alerts, owned, routed; 10 = symptom-based + baseline-derived thresholds + burn-rate + ownership + runbook links + dead-man's-switch
```

**Output:** `reports/alerting.md`

---

## PHASE 15: ON-CALL RUNBOOKS

> *"An alert without a runbook pages a human into a problem they have to solve from first principles, at 3am, under pressure. A runbook turns that into a checklist."*

```
1. RUNBOOK EXISTENCE
   → Does each PAGING alert have a linked runbook? (or is it tribal knowledge?)
   → Are runbooks version-controlled and findable from the alert itself?

2. RUNBOOK QUALITY (the 3am test)
   FOR A RANDOM PAGING ALERT, does the runbook tell an on-call who DIDN'T write the code:
   → What this alert MEANS (what's the user impact)?
   → How to CONFIRM it's real (which dashboard/query)?
   → What "normal" looks like (so they can judge severity)?
   → Immediate MITIGATION steps (rollback, scale, feature-flag, restart)?
   → Escalation path (who/when to wake up)?
   → Are the linked dashboards/queries still VALID (not rotted)?

3. COVERAGE OF FAILURE MODES
   → Common failures (dep down, deploy bad, queue backed up) documented?
   → Post-incident: are runbooks updated after each incident (living docs)?

4. AUTOMATION OPPORTUNITY
   → Repetitive manual runbook steps that could be automated/one-click?

SCORE: 0 = no runbooks; 3 = a few stale wiki pages, not linked from alerts; 5 = runbooks exist but assume code knowledge; 8 = linked, actionable runbooks for major alerts; 10 = every paging alert → valid, 3am-proof runbook with confirm/normal/mitigate/escalate, kept living
```

**Output:** `reports/runbooks.md`

---

## PHASE 16: RETENTION, ROUTING & COST

> *"The trace you need is always the one that aged out yesterday. The log bill is always the one nobody owns."*

```
1. RETENTION ADEQUACY
   → Logs / metrics / traces retention windows — long enough to investigate
     incidents discovered days later? (a 24h retention on logs loses Friday's bug by Monday)
   → Compliance-driven minimums respected? (and PII NOT over-retained — Phase 4)

2. PIPELINE RELIABILITY
   → Where do signals go and can the pipeline DROP data silently under load?
   → Buffering / backpressure on the telemetry exporter?
   → Is there a heartbeat proving the pipeline is alive (ties to Phase 14 negative test)?

3. COST OWNERSHIP
   → Does anyone own telemetry spend? Runaway log/metric cost is common + invisible.
   → Cardinality (Phase 9) and volume (Phase 9) are the cost drivers — controlled?

4. ACCESS
   → Can the on-call actually ACCESS logs/traces in prod quickly?
     (or is it gated behind a ticket / VPN / nobody-has-creds → effectively no observability)

SCORE: 0 = logs evaporate / no retention; 3 = short retention, pipeline can drop silently; 5 = adequate retention, no cost ownership; 8 = right retention + reliable pipeline + access; 10 = tuned retention + backpressured pipeline + cost ownership + fast on-call access + PII not over-retained
```

**Output:** `reports/retention-cost.md`

---

## PHASE 17: 3AM DEBUGGABILITY — END-TO-END INCIDENT RECONSTRUCTION (THE FINE COMB)

> *"This is the phase that separates an observability audit from an OBSERVABILITY AUDIT. You don't sample the instruments — you fly the plane through a simulated failure and prove you can land it."*

This is the acceptance test for the entire audit. Everything else feeds this.

```
1. THE RECONSTRUCTION DRILL (the hinge path)
   Pick ONE real request through the OBSERVABILITY HINGE POINT. Using ONLY
   telemetry (no source code, no author knowledge), attempt to answer:
   → Can I find this single request's log lines by ONE correlation ID? (Phase 3, 8)
   → Can I see its full trace, every hop, with timings? (Phase 5, 6)
   → If it errored, is it in the error tracker with a stack + that same ID? (Phase 10)
   → Can I see its contribution to the RED metrics? (Phase 7)
   → STITCH TEST: does ONE id tie log ↔ trace ↔ error ↔ metric? If not → CRITICAL.

2. THE SIMULATED-OUTAGE WALK (per failure mode)
   For each major failure mode (dependency down, latency spike, error surge,
   queue backup, bad deploy), walk the path a 3am on-call would take:
   → Would an ALERT fire? At the right time (not too late)? (Phase 14)
   → Does it link to a RUNBOOK? (Phase 15)
   → Does a DASHBOARD show the blast radius + what changed? (Phase 12)
   → Can you DRILL from symptom → component → root cause via traces/logs?
   → Estimate TIME-TO-ROOT-CAUSE. >15min on the hinge path = a finding.

3. BLIND-SPOT INVENTORY
   → Enumerate every failure mode that would produce NO telemetry (silent failure).
   → Each silent failure on a user-critical path = CRITICAL finding.

4. THE NEW-ENGINEER TEST
   → Could someone who joined last week diagnose a hinge-path incident from the
     telemetry + runbooks alone? If it requires the original author, observability failed.

SCORE: 0 = cannot reconstruct a single request, silent failures on hinge; 3 = logs only, no stitch, slow guesswork; 5 = partial stitch, some failure modes covered; 8 = one-ID stitch works, most failure modes alerted+runbooked; 10 = full one-ID stitch (log↔trace↔error↔metric), every failure mode alerts→runbook→dashboard→root cause <15min, zero silent failures on hinge, new-engineer can debug it
```

**Output:** `reports/debuggability-3am.md` — the most important report; the executive summary of whether the system is actually observable.

---

## PHASE H1 — HYBRID SYNTHESIS (Popper / hinge / user-need / edge cases / cross-audit)

> "H1" = Hybrid step 1 of the synthesis layer that pairs with Phase 0's
> programmatic gather. It sits between the last domain phase (17) and the
> VERDICT phase. It does NOT renumber existing phases. The token budget freed
> by Phase 0's deterministic gather is REINVESTED here — depth increases.

### H1.1 Popper falsification per finding (mandatory)

For every finding (start with `severity ∈ {critical, high}`), try to PROVE the
gather/your-own claim is wrong. Each falsification produces a
`falsifiable_tests[]` entry in `verdict.json`:

```jsonc
{
  "claim": "auth middleware (src/auth/mw.ts:30) logs failures with no correlation ID",
  "test_command": "grep -n 'logger' src/auth/mw.ts && cat audits/.observabilityaudit/discovery/emitted-samples/auth-fail.log",
  "expected": "log line present but no request_id/trace_id field → claim TRUE",
  "actual": "logger.error('auth failed', { email }) — no id, and emitted sample confirms",
  "outcome": "confirmed"
}
```

Outcomes: `confirmed` (failed to falsify → stands, often promoted), `falsified`
(counter-example found → demote to `info` + `falsified_at: <evidence>`),
`inconclusive` (couldn't run cleanly → keep severity, `confidence: medium`).

**The rule:** every CLAIM (PASS or FAIL) MUST cite ≥3 concrete commands that
COULD have falsified it but didn't. Banned phrases (`looks correct`, `should be
fine`, `appears to work`, `good coverage`, `reasonable logging`) → automatic FAIL.

Common falsification patterns for observability:

| Claim | Popper test |
|---|---|
| "SDK is wired" | Run the system, capture emitted output / hit the export endpoint — does a real event arrive? |
| "structured logging" | Read `emitted-samples/` — are the lines actually JSON/KV at RUNTIME, or pretty/free-text? |
| "we log all errors" | Grep every `catch`/`except`/`Err(` against log call sites — find the silent branches |
| "no PII in logs" | Grep EMITTED output (not code) for emails/tokens/keys; whole-object logs almost always leak |
| "alerts cover it" | Read the alert rule — is the threshold derived from baseline, and does a `for:` clause exist? |
| "readiness checks deps" | Read the /ready handler — does it actually query the DB, or `return 200`? |
| "traces are end-to-end" | Follow propagation across one outbound call + one async hop — does the trace continue? |
| "SLO exists" | Is the SLI computable from an emitted metric, or is it a doc with no measurement behind it? |
| "metric is emitted" | Confirm the code path that emits it actually runs in prod (not a dead branch) |

### H1.2 Hinge cross-reference (10× scrutiny on load-bearing findings)

The OBSERVABILITY HINGE POINT is the user-critical path whose silent failure
costs most. Compute or read:

```bash
HINGE_FILE="$HOME/.omega/state/hinge-points-${TICKET_ID:-default}.json"
# Otherwise compute on the fly:
${OMEGA_DIR:-$HOME/.omega}/skills/audits/_shared/hinge-analyzer.sh "$PROJECT_PATH" --audit=observability --user-need="$USER_NEED_QUOTE" \
  > "$HINGE_FILE"   # if the analyzer is unavailable, identify the hinge manually in Phase 0b and record it here
```

For each finding on the hinge path, mark `is_load_bearing: true` and apply 10×
scrutiny: 5× more falsification attempts, mandatory run of the reconstruction
drill (Phase 17) for the hinge, mandatory read of the hinge handler + all its
log/span/metric call sites + all its failure branches.

```jsonc
{
  "finding_id": "F-003",
  "is_load_bearing": true,
  "hinge_reference": "checkout mutation — convex/checkout.ts",
  "additional_scrutiny": "ran reconstruction drill: no trace_id in logs, no span on the Stripe call; one request NOT reconstructable",
  "confidence_after_scrutiny": "high"
}
```

### H1.3 User-need verification (`--user-need` quote)

If dispatched with `--user-need="<verbatim user complaint>"`, evaluate every
finding against it. For each finding ask: "If a user/operator reported THIS
verbatim (e.g. 'we couldn't figure out why checkout was failing for an hour'),
is this finding the cause?" and "Does fixing it make the complaint untrue?"

Findings unrelated to user-need: demoted one severity UNLESS load-bearing.
Findings related: highest fix priority, listed first in `user_need_match.findings[]`.

```jsonc
{
  "user_need_match": {
    "addressed": true,
    "user_need_quote": "we were blind for an hour during the checkout outage",
    "rationale": "F-003 (no trace_id in checkout logs, no span on payment call) directly causes the inability to find root cause. Fix adds correlation ID + span, making reconstruction <5min.",
    "findings": ["F-003", "F-009"],
    "untouched_findings_relevant_to_user_need": []
  }
}
```

If `addressed: false`, the audit MUST score below 90/100 regardless of other phases.

### H1.4 Edge case hunting (mandatory for top findings)

For each top-5 finding (severity × cross-audit × hinge), generate ≥2 edge cases
the static gather missed. Observability-specific patterns:
- "Telemetry pipeline itself fails (exporter down) — does anything alert, or does it look like perfect health?" (dead-man's switch)
- "Error happens during STARTUP, before the logger/tracer is initialized — is it captured?"
- "High load → sampling drops the failing trace / logger backpressures and drops lines."
- "Async/queued work fails — is the trace context still attached, or is it an orphan?"
- "Log level is DEBUG in prod after an incident debug session, never reverted — cost + PII."
- "Correlation ID provided by upstream is dropped and a new one generated — chain broken across services."
- "Timezone/clock skew makes logs and traces un-correlatable by time."

```jsonc
{
  "finding_id": "F-003",
  "scenario": "Stripe call times out; the catch logs 'payment failed' with no trace_id and no span error — on-call sees an error count tick up but cannot find WHICH request or WHY",
  "covered_by_existing_test": false,
  "evidence_gathered": "emitted-samples/checkout-timeout.log shows context-free line; trace has no span for the Stripe hop",
  "fix_includes_coverage": true
}
```

### H1.5 Cross-audit synthesis (re-read sibling summaries from Phase 0.5)

With your `verdict.json` draft, do a final pass with sibling audits open:
1. For each of YOUR top-5: is the same file/line flagged by a sibling?
2. If yes → `cross_audit_confirmed: true`, bump severity one level.
3. For each sibling top-5: relevant to observability? Add as `tool: "cross-audit:<sibling>"`.
4. Write `cross_audit_links[]`.

Especially: a `codeaudit`/`debugaudit` silent-`catch` that is ALSO an
observability silent-failure-branch is the highest-value joint finding — fix
once (log + re-raise/handle) and close both.

```jsonc
{
  "cross_audit_links": [
    {
      "this_finding_id": "F-007",
      "sibling_audit": "codeaudit",
      "sibling_finding_id": "F-031",
      "shared_location": "src/jobs/sync.ts:88",
      "joint_fix_recommended": true
    }
  ]
}
```

### H1.6 Final verdict.json schema (hybrid v2)

```jsonc
{
  "audit": "observability",
  "score": 100,
  "score_raw": "<raw>/360",
  "score_normalized": 100,
  "confidence": "high|medium|low",
  "skill_used": "observability",
  "preamble_version": "1.0",
  "user_need_match": { /* §H1.3 */ },
  "falsifiable_tests": [ /* §H1.1 */ ],
  "hinge_findings": [ /* §H1.2 */ ],
  "issues_found_and_fixed": [
    {
      "id": "FIX-001",
      "finding_id": "F-003",
      "before": "<state>",
      "after": "<state>",
      "verification": "<command + output>"
    }
  ],
  "edge_cases": [ /* §H1.4 */ ],
  "cross_audit_links": [ /* §H1.5 */ ],
  "reconstruction_drill": {
    "hinge_path": "<path>",
    "one_id_stitch": true,
    "time_to_root_cause_estimate_min": 4,
    "silent_failure_modes_on_hinge": 0
  },
  "evidence_summary_path": "$PROJECT_PATH/audits/.observabilityaudit/evidence-summary.json",
  "confidence_basis": "Why I'm confident (or not). Cite Popper test counts, reconstruction-drill result, hinge scrutiny depth, edge-case coverage, cross-audit confirmations.",
  "banned_phrase_check": "passed (no occurrences of `looks correct`, `should be fine`, `appears to work`, `good coverage`, `reasonable logging`, `streamlined`, `to save time`)"
}
```

### H1.7 Score gating (hybrid threshold)

A 100/100 score is blocked unless:
1. ✅ All `critical`/`high` findings fixed OR have a ≥50-word `non_issue_justification` backed by Popper evidence.
2. ✅ All load-bearing findings (H1.2) confirmed via Popper falsification.
3. ✅ `user_need_match.addressed = true` with verbatim quote.
4. ✅ ≥3 falsifiable tests cited per phase (H1.1).
5. ✅ ≥2 edge cases generated per top-5 finding (H1.4).
6. ✅ Cross-audit synthesis attempted (H1.5) — array present even if empty.
7. ✅ The Phase 17 reconstruction drill ran on the hinge path with `one_id_stitch` recorded.
8. ✅ Zero confirmed secrets/PII in emitted logs (Phase 4 hard gate; otherwise cap ≤ 60).
9. ✅ `confidence_basis` populated with non-trivial reasoning.

Below threshold → score < 100, fix-and-reaudit loop kicks in (bounded at 5
iterations per the Audit Verification Contract; on iteration 5 if still failing,
emit `confidence: low` and surface as `pending` in `.done.json`).

---

## PHASE 18: VERDICT

Score each phase 0-10, weight by severity:

```
SCORING MATRIX (360 max):
  Phase  1  (Structured Logging Coverage)   x 3.0  = max 30
  Phase  2  (Log-Level Correctness)         x 2.0  = max 20
  Phase  3  (Log Context & Actionability)   x 3.0  = max 30
  Phase  4  (Log Hygiene — PII/Secrets)     x 2.5  = max 25
  Phase  5  (Distributed Tracing Coverage)  x 2.5  = max 25
  Phase  6  (Trace/Context Propagation)     x 2.0  = max 20
  Phase  7  (Metrics — RED)                 x 3.0  = max 30
  Phase  8  (Metrics — USE + Correlation)   x 2.5  = max 25
  Phase  9  (Cardinality & Sampling)        x 1.5  = max 15
  Phase 10  (Error Tracking Coverage)       x 2.0  = max 20
  Phase 11  (Health & Readiness Probes)     x 1.5  = max 15
  Phase 12  (Dashboard Coverage)            x 2.0  = max 20
  Phase 13  (SLO/SLI & Error Budgets)       x 2.0  = max 20
  Phase 14  (Alerting Rules & Thresholds)   x 3.0  = max 30
  Phase 15  (On-Call Runbooks)              x 1.5  = max 15
  Phase 16  (Retention, Routing & Cost)     x 1.0  = max 10
  Phase 17  (3am Debuggability — E2E)       x 3.0  = max 30
                                            TOTAL  = max 360

NORMALIZE: score = (raw / 360) x 100
(applicable max excludes phases N/A for project type — e.g. a stateless CLI may
 have no health probes; document exclusions and renormalize per preamble §5)

GRADE:
  90-100: S — Glass box. One ID stitches log↔trace↔error↔metric; every failure mode alerts→runbook; <15min to root cause; zero silent failures on the hinge.
  80-89:  A — Observable. Strong three-pillar coverage, minor gaps off the hinge path.
  70-79:  B — Mostly visible. Core paths instrumented, alerting/SLO/runbook gaps.
  60-69:  C — Partially blind. Logs exist but weak correlation; reactive, slow to diagnose.
  50-59:  D — Mostly blind. Find out from users; can't reconstruct an incident.
  <50:    F — Black box. No correlation, silent failures on critical paths, or secrets/PII leaking into logs.
```

---

## PHASE 19: FIX PLAN (automatic)

```
Sort: CRITICAL -> HIGH -> MEDIUM -> LOW
Priority by debuggability impact:
  CRITICAL: silent failure on the hinge path, OR secrets/PII in emitted logs, OR
            no way to reconstruct a hinge-path incident (no correlation ID stitch)
  HIGH:     dead instrument (SDK installed not wired), missing RED on hinge,
            error-as-info (invisible to alerts), readiness lies, no alert on hinge
  MEDIUM:   missing context fields, missing runbook, weak threshold, no SLO
  LOW:      cardinality risk, dashboard rot, retention tuning, cost ownership

Group by blast radius (one correlation-ID propagation fix can light up many paths).
Dependency order (add correlation ID BEFORE wiring log↔trace correlation; wire
  the metric BEFORE building the SLO/alert on it).
Generate fix tasks with file:line specificity.
Save to audits/.observabilityaudit/fix-plan.json + fix-plan.md
```

---

## PHASE 20: FIX EXECUTION (automatic)

```
Sequential per finding group.

─── INSTRUMENTATION SAFETY GATE: DO NO HARM (MANDATORY before EVERY fix) ───

Adding observability must not (a) break the code, (b) leak secrets/PII you were
trying to prevent, (c) blow up cardinality/cost, or (d) add latency to a hot path.
Every fix goes through this gate BEFORE commit.

PRE-FIX ANALYSIS (before writing ANY code):
  a. Read the ENTIRE target file (not just the target line).
  b. SCOPE COLLISION CHECK — adding a logger/import/variable: grep the file for
     the name; avoid shadowing an existing module-level import (UnboundLocalError /
     redeclare). Use the existing logger instance, don't spin up a second one.
  c. HOT-PATH CHECK — is this code in a tight loop / per-element path? Then a log
     or span per iteration = volume/cost bomb (Phase 9). Aggregate or sample instead.
  d. CARDINALITY CHECK — adding a metric label or span attribute: is the value
     bounded? NEVER add user_id / request_id / raw URL as a metric label.
  e. REDACTION CHECK — adding context to a log: does the value contain PII/secrets?
     Route through the redaction/serializer layer; never log whole request/response.
  f. CROSS-REFERENCE CHECK — changing a logger config / metric name / span name:
     grep for dashboards/alerts/queries that reference the old name; update them or
     they silently break (dashboard "No Data", alert never fires).

POST-FIX VERIFICATION (after writing code, BEFORE commit):
  a. SYNTAX CHECK: TS `npx tsc --noEmit`; Rust `cargo check`; Python `python -c "import ast; ast.parse(open('FILE').read())"`.
  b. BUILD: use `~/.omega/lib/safe-npm-build.sh` (per project rule 12) or the project build.
  c. EMITTED-OUTPUT CHECK (the key one for THIS audit): run the system / exercise the
     path and CAPTURE the new telemetry. Verify: the new log/metric/span ACTUALLY
     appears, is structured, carries the intended IDs, and leaks NO secret/PII.
     (First Law: only runtime tells the truth — adding a log call ≠ the log working.)
  d. NO-REGRESSION: the path still functions; latency not visibly degraded; no new
     error introduced by the instrumentation itself.

IF ANY POST-FIX CHECK FAILS:
  → `git revert HEAD` immediately
  → Log the failure in fix-log.md with exact error/output
  → Mark NEEDS_REVIEW (never retry the same approach blindly)
  → Try an alternative OR skip the fix

────────────────────────────────────────────────────────────────────────

FOR EACH FIX TASK (priority order):
  a. Read the ENTIRE target file.
  b. Run PRE-FIX ANALYSIS (collision, hot-path, cardinality, redaction, cross-ref).
  c. Document BEFORE state (the blind spot / context-free line / dead instrument).
  d. Apply the instrumentation fix.
  e. Run POST-FIX VERIFICATION (syntax, build, EMITTED-OUTPUT capture, no-regression).
  f. If all green → commit: observability(observabilityaudit): FIX-XXX description
  g. If any red → revert → log → NEEDS_REVIEW.
  h. Document AFTER state (capture the now-correct telemetry into before-after.md).
  i. HIGH-RISK fixes (changing prod log level, sampling, retention, redaction config):
     require human confirmation.
```

---

## PHASE 21: RE-AUDIT (automatic)

```
1. SERVICE HEALTH GATE (mandatory):
   → systemd service: restart, wait 10s, check is-active + logs for errors
   → build step: full build must pass (safe-npm-build.sh)
   → tests: relevant suite must pass

2. RECONSTRUCTION RE-DRILL (mandatory, the acceptance test):
   → Re-run Phase 17 on the hinge path. The one-ID stitch must now work and
     time-to-root-cause must have improved. If not, the fixes didn't move the needle.

3. Re-run all FAILING phases. Compare before/after via before-after.md.
4. Loop until score >= 80 (B) or remaining items are NEEDS_REVIEW.
5. Special attention: verify NO fix introduced a secret/PII leak or a cardinality
   blowup (the instrumentation didn't become the new incident).
```

---

## PARALLEL EXECUTION STRATEGY

```
The 18 phases group into waves for parallelism:

WAVE 1 (discovery — sequential):
  Phase 0:  Programmatic gather
  Phase 0b: Recon + telemetry-stack mapping + hinge identification
  (capture REAL emitted telemetry here — feeds every later phase)

WAVE 2 (logs — parallel):
  Phase 1: Structured Logging   ]
  Phase 2: Log Levels           ] Log-pillar
  Phase 3: Log Context          ] analysis
  Phase 4: Log Hygiene          ]

WAVE 3 (traces + metrics — parallel):
  Phase 5: Tracing Coverage     ]
  Phase 6: Span Propagation     ] Trace + metric
  Phase 7: Metrics RED          ] pillar analysis
  Phase 8: Metrics USE + IDs    ]
  Phase 9: Cardinality/Sampling ]

WAVE 4 (operational surfaces — parallel):
  Phase 10: Error Tracking      ]
  Phase 11: Health Probes       ]
  Phase 12: Dashboards          ] Ops + SLO + alert
  Phase 13: SLO/SLI             ] analysis
  Phase 14: Alerting            ]
  Phase 15: Runbooks            ]
  Phase 16: Retention/Cost      ]

WAVE 5 (synthesis — sequential, depends on all above):
  Phase 17: 3am Debuggability (the end-to-end reconstruction acceptance test)
  Phase H1: Hybrid synthesis (Popper / hinge / user-need / edge / cross-audit)
  Phase 18: Verdict
```

---

## CROSS-COMMAND BRIDGE

```
/observabilityaudit finds a silent catch          -> references /codeaudit + /debugaudit (joint fix: log + handle)
/observabilityaudit finds a token in logs         -> IMMEDIATELY flags /secaudit for rotation
/observabilityaudit finds no duration metric       -> references /perfaudit (can't detect slowdowns in prod)
/observabilityaudit finds no error capture          -> references /debugaudit (the break left no trace)

THE QUALITY ARSENAL:
  /codeaudit          -> Is the code SOLID?            (preventive)
  /flowaudit          -> Does the experience WORK?     (preventive)
  /uiuxaudit          -> Is the interface BEAUTIFUL?   (preventive)
  /debugaudit         -> What is BROKEN right now?     (detective)
  /perfaudit          -> Is it FAST enough?            (detective)
  /secaudit           -> Is it SECURE?                 (detective)
  /observabilityaudit -> Can we SEE what it does?      (detective)

  Together: nothing escapes — and when something does break, you can SEE it break.
```

---

## LAWS

1. **An unobserved failure is a future undiagnosable incident.** Absence of error logs is not health. Every silent code path is a blind spot.
2. **A context-free log is a fake instrument (Popper).** If you can't reconstruct the incident from the line, it gives the illusion of coverage and zero value.
3. **Three pillars or one blind spot.** Logs, metrics, traces — independently present, and CORRELATED by one ID. Two out of three is a guessing game.
4. **An alert without a runbook is a panic button.** Page a human into a checklist, not a cold-start investigation.
5. **Thresholds are hypotheses about reality (Popper).** Derive them from the system's actual baseline, or they scream or stay silent — both end in a missed outage.
6. **Logs are the #1 leak surface (Popper).** Verify the EMITTED output, not the code's intent. The token you logged is in a vendor for a year.
7. **The only proof of observability is reconstruction.** FALSIFY "we're observable" by replaying an incident with telemetry alone. If it needs the author, observability failed.

---

*"/observabilityaudit v1 — Instrument. Correlate. Alert. Reconstruct. Logs, metrics, traces, SLOs, runbooks — prove you can land the plane at 3am from the instruments alone. /360."*

---

## COMPLIANCE & CRITICAL ADDENDA (v1.0 — 2026-05-29)

### Quality Arsenal Preamble Compliance

This audit implements contracts defined in `../_shared/QUALITY-ARSENAL-PREAMBLE.md` v1.0:

- ✅ **Gestalt-Popper doctrine** — hinge point, falsification, evidence chain, adversarial framing
- ✅ **Concurrency lock** — canonical runner `flock` at `audits/.observabilityaudit/.runner.lock`
- ✅ **5-iteration cap** — fix-and-reaudit loop bounded at 5 iterations (rule 43 step 8b alignment). On cap: NEEDS_REVIEW + Telegram SOS. No silent infinite loops.
- ✅ **Scoped invocation flags** — `--url=`, `--files=`, `--scope=`, `--ticket=`, `--no-fix`, `--focus=` (logs|traces|metrics|alerts|slo|runbooks)
- ✅ **Non-UI context gate** — runs on any target (web, API, daemon, CLI, library). Phase scoping adjusts: a stateless CLI may exclude health-probes/SLO; document exclusions and renormalize.
- ✅ **Output contract verification** — emits `verdict.json`, `verdict.md`, `fix-plan.json`, `fix-plan.md`, `progress.json`, `telemetry.json`, `before-after.md`, `fix-log.md`. Output gate at end; missing/malformed = audit did NOT succeed.
- ✅ **Telegram progress notifications** — `start` / `progress` (every 3 phases) / `iteration` / `verdict` / `abort` / `sos` via `~/.omega/bin/audit-notify.sh`
- ✅ **Discovery drift check** — on resumed runs, if `discovery/` (esp. `emitted-samples/`) > 1h old, re-capture telemetry or abort with user-confirm (emitted telemetry is the ground truth — stale samples lie)
- ✅ **Self-telemetry** — `telemetry.json` at completion (duration, tokens, phases, fixes, model, preamble_version)
- ✅ **Deprecation registry** — cross-references checked against `${OMEGA_DIR:-$HOME/.omega}/skills/audits/_shared/DEPRECATED.md`; stale refs flagged
- ✅ **Rule-46 compliance** — NO `--quick`/`--streamlined`/`--lightweight` variants. Narrower scope uses `--focus <area>` with FULL phase depth. Orchestrator prompts with rule-46 banned phrases are REFUSED.
- ✅ **Score normalization** — raw / applicable-phase-max × 100 = /100
- ✅ **preamble_version** — emitted as `"1.0"` in verdict.json for `/metaudit` compliance scan

### Audit-Specific Critical Addendum — Runtime Capture + Instrumentation Safety

**Runtime telemetry capture (the defining requirement of this audit):**
- The audit MUST capture REAL emitted telemetry (run the system / read prod logs / hit `/metrics`) into `discovery/emitted-samples/`. Static log-call-site counts are NOT sufficient evidence — code that "logs" may emit nothing useful, or leak everything. (First Law.)
- If the system cannot be run and no prod logs are accessible: emit `confidence: low`, score the static signals only, and flag `runtime_capture: unavailable` in verdict.json. Do NOT claim observability is good from code alone.

**Instrumentation safety (do-no-harm specific to adding telemetry):**
- A fix MUST NOT introduce a secret/PII leak (Phase 4 redaction gate on every added log).
- A fix MUST NOT add an unbounded metric label / per-iteration log on a hot path (cardinality/volume/cost — Phase 9 gate).
- A fix MUST NOT add measurable latency to the hinge path.
- Renaming a metric/span/logger field MUST update referencing dashboards/alerts in the same change or they silently break.

**PII hard gate:** confirmed active secrets or PII in EMITTED logs caps the audit at ≤ 60/100 and produces a CRITICAL finding flagged for immediate rotation (cross-ref `/secaudit`), regardless of other phase scores.

### /metaudit Compliance Badge

Run `/metaudit --focus arsenal --scope="observabilityaudit only"` to verify against the 11-point preamble checklist. Target: 11/11.

---

## MANDATORY BEFORE/AFTER VERIFICATION (v1.0)

**Read `../_shared/AUDIT-VERIFICATION-CONTRACT.md` before ANY fix execution.**

Every fix MUST follow the "Do No Harm" protocol:

1. **PRE-FIX BASELINE** — grep all references (metric/span/logger names, dashboards, alerts), capture the path's current functional state AND its current emitted telemetry, save to `audits/.observabilityaudit/baseline/`.
2. **APPLY FIX** — normal execution.
3. **POST-FIX CHECK** — repeat every baseline check. Capture the NEW emitted telemetry and confirm the intended log/metric/span appears, structured, with IDs, and leaks nothing. If any functional check goes PASSED→FAILED, revert immediately.
4. **BREAKAGE SCAN** — grep for old metric/span/field names across dashboards/alerts/queries; must return 0 non-ephemeral hits (a renamed metric silently kills a dashboard panel and an alert).
5. **BEFORE/AFTER MATRIX** — produce `audits/.observabilityaudit/before-after.md` with, per affected item: functional status + a BEFORE telemetry sample and an AFTER telemetry sample proving the improvement.

**An audit that breaks 1 working dashboard/alert — or leaks 1 secret while "improving logging" — is WORSE than no audit.** Do NOT claim "done" without `before-after.md` showing zero regressions and real captured telemetry.

---

## Dynamic-Workflow Orchestration (v2)

> *"Eighteen phases run sequentially is a single-threaded investigator pacing one room at a time. The three pillars are independent — interrogate them in parallel, then prove the stitch."*

This section governs **HOW this audit executes when run**. It changes the execution
*engine*, not the audit's identity: every phase (0, 0b, 1–17, H1, 18–21), every scoring
weight in the Phase 18 matrix (/360), the grade bands, the Phase 4 PII hard gate (≤60),
and the verdict.json schema are UNCHANGED. The Gestalt clarity gate and Popper
falsification doctrine are preserved verbatim — this layer simply makes the fan-out
explicit and the verification adversarial. When this section and the linear
"PARALLEL EXECUTION STRATEGY" waves describe the same thing, treat the waves as the
track definition and this section as the orchestration contract over them.

### 1. Fan-out — decompose phases into INDEPENDENT parallel tracks

After Phase 0 (programmatic gather) and Phase 0b (recon + telemetry-stack mapping +
hinge identification + the **mandatory real emitted-telemetry capture** into
`discovery/emitted-samples/`) complete **sequentially** — they are the shared evidence
base every track reads and MUST run first — dispatch the domain phases as concurrent
Workflow tracks via the `Workflow` tool (in-process fan-out, per R-ORCH), NOT one phase
after another. The tracks are file/concern-disjoint and map onto this audit's three
pillars + operational surface:

| Track | Phases | Reads (shared, read-only) | Emits |
|---|---|---|---|
| **T-LOGS** | 1 Structured · 2 Levels · 3 Context · 4 Hygiene/PII | `evidence-summary.json`, `emitted-samples/`, log call-sites | `reports/structured-logging.md` … `log-hygiene.md` |
| **T-TRACE** | 5 Tracing · 6 Propagation | `telemetry-stack.json`, span exports | `reports/tracing.md`, `span-propagation.md` |
| **T-METRICS** | 7 RED · 8 USE+CorrelationID · 9 Cardinality/Sampling | metrics manifests, `/metrics` scrape | `reports/metrics-red.md` … `cardinality-sampling.md` |
| **T-OPS** | 10 ErrorTracking · 11 HealthProbes · 12 Dashboards · 13 SLO/SLI · 14 Alerting · 15 Runbooks · 16 Retention/Cost | dashboard/alert configs, SLO files, probe routes | `reports/error-tracking.md` … `retention-cost.md` |

Track rules:
- All four tracks run **concurrently** — they only READ the Phase 0/0b artifacts; none
  writes a file another reads, satisfying R-SCOPE (one writer per file).
- The **OBSERVABILITY HINGE POINT** (the one user-critical path whose silent failure
  costs most) is passed into every track; each applies 10× scrutiny to its slice of the
  hinge (T-LOGS reads the hinge handler's failure branches, T-TRACE follows the hinge
  request's spans, T-METRICS confirms RED is emitted on the hinge, T-OPS confirms the
  hinge is alerted + runbooked).
- Phase 17 (3am Debuggability) and Phase H1 (Hybrid synthesis) remain **sequential and
  AFTER** all tracks — Phase 17 is the cross-track acceptance test (the one-ID stitch
  needs all three pillars' findings present) and cannot start until logs/traces/metrics
  tracks have landed.
- Do NOT fan out Phase 0 / 0b (shared evidence) or Phases 17/18 (synthesis/verdict).

### 2. Adversarial verification — ≥2-of-3 lenses before a finding is accepted

Every candidate finding (from any track) is provisional until it survives independent
adversarial verification. Run **three lenses** and require **≥2 to agree** before the
finding enters `verdict.json`; this operationalises the existing H1.1 Popper protocol
(every claim cites ≥3 falsifying commands) as a hard 2-of-3 consensus gate:

1. **REPRODUCE** — exercise the real path and capture emitted output. The finding is
   real only if the runtime artifact shows it (First Law / L1). E.g. claim "auth logs
   have no correlation ID" → trigger an auth failure, read
   `emitted-samples/auth-fail.log`, confirm no `request_id`/`trace_id` field.
2. **REFUTE** — actively try to FALSIFY the finding (Popper). Hunt the counter-example:
   a redaction layer that *does* fire, a `/ready` handler that *does* query the DB, a
   trace that *does* continue across the async hop, an alert `for:` clause that *does*
   suppress the blip. If refutation succeeds → **kill the finding** (demote to `info`,
   record `falsified_at: <evidence>`).
3. **CROSS-CHECK** — corroborate against a second independent source: a sibling audit's
   `evidence-summary.json` (Phase 0.5 — a `codeaudit`/`debugaudit` silent-`catch` that
   is also a logging blind spot), the error-tracker-vs-error-metric agreement test
   (Phase 10.5), or a second telemetry signal (does the log↔trace↔metric tell the same
   story for one request?).

Verdict per finding:
- **≥2 lenses confirm** → finding STANDS, severity per the Phase 18 weights; if REPRODUCE
  + CROSS-CHECK both confirm a hinge-path issue → promote (cross-audit confluence already
  bumps one level per Phase 0.5).
- **REFUTE wins (≤1 lens confirms)** → finding is KILLED; never report a finding that
  failed its own falsification. Surviving findings only.
- **Inconclusive (couldn't run a lens cleanly)** → keep at most `medium`,
  `confidence: medium`, and say so — never inflate.

The Phase 4 secret/PII finding is special: it requires the **REPRODUCE** lens to fire
against *emitted* output (not code intent) before the ≤60 hard gate trips — a static
pattern alone is `medium` until the emitted sample confirms the leak.

### 3. Synthesize back into THIS audit's existing matrix + verdict (unchanged)

The surviving, verified findings from all tracks converge into the **existing** pipeline
with ZERO change to scoring or output:
- Phase 17 runs the reconstruction drill on the hinge using the consolidated findings
  (the one-ID stitch: log↔trace↔error↔metric).
- Phase H1 performs the documented Popper / hinge / user-need / edge-case / cross-audit
  synthesis and writes the H1.6 hybrid `verdict.json`.
- Phase 18 scores each phase 0–10 and applies the **unchanged** /360 weighted matrix
  (Structured ×3.0, Context ×3.0, RED ×3.0, Alerting ×3.0, 3am ×3.0, … Retention ×1.0),
  normalises `(raw / applicable_max) × 100`, and assigns the same S/A/B/C/D/F grade.
- Phases 19–21 (fix plan, do-no-harm fix execution, re-audit + reconstruction re-drill)
  are unchanged. The parallel layer feeds findings IN; the verdict comes OUT exactly as
  before. Fan-out is an execution detail invisible to the score.

### 4. Loop-until-dry for unknown-size discovery

Blind-spot enumeration (Phase 1 silent failure branches, Phase 17 §3 blind-spot
inventory, Phase H1.4 edge cases) is open-ended — you do not know how many silent
failure modes exist until you stop finding them. Run discovery as a bounded
loop-until-dry **inside** this orchestration (never as a top-level `/goal` around it —
R-GOAL):

```
pass = 0
until DRY or pass == 5 (Audit Verification Contract re-audit cap):
  pass += 1
  re-fan-out discovery over un-instrumented branches on/around the hinge path
  for each NEW candidate blind spot → run the ≥2-of-3 verification (§2)
  DRY when a full pass yields zero NEW surviving findings
on pass==5 still finding → mark remaining NEEDS_REVIEW, confidence: low,
  surface as `pending` (do NOT loop unbounded — no silent infinite loops)
```

Each newly discovered blind spot is verified by §2 before it counts, then folded into
§3. The loop terminates on a dry pass or the 5-iteration cap — whichever first — keeping
the discovery bounded while still exhausting the unknown-size search space (L4: done
means 100%, the safe work is exhausted, blockers recorded).

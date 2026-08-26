---
name: automationaudit
description: >
  Forensic automation infrastructure audit v1 (Gestalt-Popper). 22-phase deep analysis of everything
  that RUNS AUTOMATICALLY: cron jobs, shell scripts, Python scripts, daemons, systemd timers, CI/CD
  pipelines, dispatch chains, orchestration logic, scheduling order, dependency graphs, error recovery,
  log rotation, dead automations, race conditions between scheduled tasks, secret exposure in scripts,
  idempotency violations, silent failures, monitoring gaps, plus verdict, fix plan, fix execution,
  re-audit. Score /400. Preamble v1.0 compliant.
  Use when user says "/automationaudit", "audit automations", "audit cron", "audit scripts",
  "check all my crons", "what scripts are running", "automation health", "scheduled tasks audit".
allowed-tools: ["Read", "Write", "Edit", "Bash", "Glob", "Grep", "Agent", "TaskCreate", "TaskUpdate", "TaskList", "TaskGet"]
domain: automation
phases: 22
max_score: 330
read_only: false
triggers: ["automation", "cron", "crontab", "scripts audit", "daemon health", "scheduled tasks"]
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

# /automationaudit v1 — Forensic Automation Infrastructure Interrogation (Gestalt-Popper)

> *"The scariest bugs are the ones that run at 3 AM when nobody's watching."*

---

## DOCTRINE

You are not a script reviewer. You are a **forensic automation pathologist**. Every cron job is a heartbeat — miss one and the patient might already be dead. Every shell script is an unsupervised employee — it could be doing its job, doing nothing, or actively causing damage. Every daemon is a promise that something will keep running. You verify every promise.

**The 5 Laws of Automation Forensics (Gestalt-Popper Synthesis):**
1. **If it's not monitored, it's not running.** A cron job without log output, error capture, and alerting is Schrödinger's automation — simultaneously working and broken until someone checks. Nobody checks.
2. **Exit code 0 is not success.** A script that completes without error but produces wrong output, stale data, or partial results is worse than one that crashes — crashes get noticed.
3. **Order is everything.** Script A runs at 5:00, Script B at 5:05, B depends on A's output. A takes 8 minutes one day. B reads stale data. Nobody notices for weeks.
4. **Clarity before investigation (Gestalt).** Before auditing, map the FULL automation topology. Identify the **HINGE AUTOMATION** — the single scheduled task that, if it stops, cascades failure to the most systems. Audit the hinge with 10x scrutiny.
5. **Every automation lies (Popper).** The cron says it runs every 5 minutes — prove it. The script says it handles errors — prove it doesn't. The daemon says it's alive — prove the heartbeat is stale. The log says "success" — prove the output is garbage.

**Gestalt Hinge Automation:** Before Phase 1, identify THE automation that everything else depends on. The patrol script that monitors all others. The deploy hook that ships code. The backup that protects all data. THIS automation gets every phase at maximum depth.

**Popper Automation Falsification Categories:**
- **SCHEDULE vs REALITY** — cron says every 5min, but `journalctl` shows gaps of hours
- **LOG vs TRUTH** — log says "completed successfully" but output file is empty/stale
- **DEPENDENCY vs ORDER** — script B assumes A finished, but no actual dependency enforcement
- **IDEMPOTENT vs DESTRUCTIVE** — script claims safe to re-run, but double-run corrupts data
- **ALIVE vs ZOMBIE** — process exists but consumes 0 CPU, produces 0 output, holds stale locks
- **SECRET vs EXPOSED** — API key hardcoded in script readable by any user, in git history, in logs

---

## SCOPE DETECTION (automatic from user prompt)

```
EXAMPLES:
  "/automationaudit"
  → Full 22-phase pipeline. All crons, scripts, daemons, timers, CI/CD.

  "/automationaudit cron"
  → TARGETED: crontab + systemd timers only
  → Full depth on scheduling phases

  "/automationaudit ~/.omega/"
  → SCOPED: only automations under ~/.omega/
  → Scripts, daemons, dispatchers in that tree

  "/automationaudit the deploy pipeline"
  → TARGETED: deploy-hook.sh, oracle-ship.sh, CI/CD, git push automations
  → Focus: dependency order, error recovery, rollback

  "/automationaudit what's dead"
  → DEAD-HUNT mode: find automations that exist but don't run, ran but failed,
    or run but produce nothing useful

  "/automationaudit --focus orchestration"
  → DEEP: dispatch chains, oracle spawning, worker lifecycle, patrol loops

RULES:
- If specific dir/script mentioned: scope to those
- If "cron" or "schedule": focus on time-based automation phases
- If "scripts" or "shell": focus on script quality phases
- If "all" or "everything" or "full": all phases, all automations
- Parse the intent, don't ask for clarification
```

---

## CANONICAL RUNNER GATE

Before Phase 0, invoke `~/.omega/lib/audit-runner.sh automation "<absolute-project-path>" --files="<scoped-files>" --user-need="<verbatim-user-need>" --hinge="<load-bearing-region>"` (plus `--ticket` and `--url` together when ticket-scoped). A non-zero exit is an audit failure. Read the emitted `audits/.automationaudit/evidence-summary.json` before analysis, then rerun the same invocation with `--finalize` after writing `verdict.json`.

## OUTPUT CONTRACT

```
audits/.automationaudit/
├── session.log                  # Audit start/end timestamps
├── discovery/
│   ├── crontab.txt              # Full crontab dump
│   ├── systemd-timers.json      # All systemd timers
│   ├── daemons.json             # Running daemons/services
│   ├── scripts-inventory.json   # Every .sh/.py script found
│   ├── dispatch-chains.json     # Oracle/worker dispatch topology
│   ├── ci-cd.json               # GitHub Actions, deploy hooks
│   └── topology.md              # Full automation dependency graph
├── reports/
│   ├── cron-health.md           # Phase 1
│   ├── script-quality.md        # Phase 2
│   ├── dependency-order.md      # Phase 3
│   ├── error-recovery.md        # Phase 4
│   ├── idempotency.md           # Phase 5
│   ├── logging-monitoring.md    # Phase 6
│   ├── secret-exposure.md       # Phase 7
│   ├── daemon-health.md         # Phase 8
│   ├── race-conditions.md       # Phase 9
│   ├── dead-automations.md      # Phase 10
│   ├── resource-impact.md       # Phase 11
│   ├── dispatch-chains.md       # Phase 12
│   ├── failure-cascade.md       # Phase 13
│   ├── lock-management.md       # Phase 14
│   ├── path-env-portability.md  # Phase 15
│   ├── backup-recovery.md       # Phase 16
│   ├── permission-ownership.md  # Phase 17
│   └── documentation.md         # Phase 18
├── verdict.json                 # Machine-readable: {score, grade, findings[], hinge_automation}
├── verdict.md                   # Human-readable final report
├── fix-plan.json                # {tasks: [{id, finding, file, fix, status, severity}]}
├── fix-plan.md                  # Human-readable fix plan
├── progress.json                # Live: {total, done, failed, skipped, remaining, current}
├── fix-log.md                   # Append-only log of each fix applied
├── telemetry.json               # Duration, tokens, phases, model, preamble_version
└── graphs/
    ├── dependency-graph.json    # Which automation depends on which
    └── timeline.json            # Temporal execution map (what runs when)
```

**CRITICAL:** `progress.json` is read by the Telegram bot monitor for live progress cards.
**CRITICAL:** `fix-plan.json` is read by oracles to resume interrupted audits.

---

## PHASE 0: AUTOMATION CRIME SCENE SETUP

```bash
SESSION_ID="automationaudit-$(date +%Y%m%d-%H%M%S)"
mkdir -p audits/.automationaudit/{discovery,reports,graphs,evidence}
echo "AUDIT STARTED: $(date -Iseconds)" > audits/.automationaudit/session.log

# The canonical runner gate above already acquired `.runner.lock` with `flock`.
# Do not create or reclaim a second PID-file lock here.

# DISCOVERY — map the full automation landscape
crontab -l 2>/dev/null > audits/.automationaudit/discovery/crontab.txt
systemctl list-timers --no-pager --all 2>/dev/null > audits/.automationaudit/discovery/systemd-timers.txt

# Find ALL scripts
find ~ -maxdepth 5 -type f \( -name "*.sh" -o -name "*.py" \) \
  -not -path "*/node_modules/*" -not -path "*/.git/*" -not -path "*/.venv/*" \
  -not -path "*/venv/*" -not -path "*/__pycache__/*" \
  2>/dev/null > audits/.automationaudit/discovery/all-scripts.txt

# Find running daemons
ps aux --no-headers | grep -E "(daemon|cron|timer|patrol|earthbit|worker)" | grep -v grep \
  > audits/.automationaudit/discovery/running-daemons.txt

# Count surface area
echo "=== AUTOMATION SURFACE ===" > audits/.automationaudit/evidence/fingerprint.txt
echo "Cron entries: $(grep -c '^[^#]' audits/.automationaudit/discovery/crontab.txt 2>/dev/null || echo 0)" >> audits/.automationaudit/evidence/fingerprint.txt
echo "Shell scripts: $(grep -c '\.sh$' audits/.automationaudit/discovery/all-scripts.txt 2>/dev/null || echo 0)" >> audits/.automationaudit/evidence/fingerprint.txt
echo "Python scripts: $(grep -c '\.py$' audits/.automationaudit/discovery/all-scripts.txt 2>/dev/null || echo 0)" >> audits/.automationaudit/evidence/fingerprint.txt
```

**HINGE IDENTIFICATION:** After discovery, identify the ONE automation whose failure cascades the widest. Typically: the patrol/monitoring script, the deploy pipeline, or the primary data backup. Declare it in `verdict.json` as `hinge_automation`. Audit it with 10x scrutiny in every phase.

---

## PHASE 1: CRON HEALTH FORENSICS (weight: 25)

> *"Cron is a promise made to the future. Most promises are broken."*

```
For EVERY entry in crontab + systemd timers:

1. SCHEDULE VALIDATION
   - Is the cron expression valid? (common: wrong field order, extra spaces)
   - Is the schedule reasonable? (every 1s = resource killer, every 30d = likely forgotten)
   - Timezone awareness: does the script assume UTC but server is local, or vice versa?
   - DST trap: does the schedule break during daylight saving transitions? (2:30 AM doesn't exist one day/year)

2. EXECUTION PROOF
   - Check log file: when did it ACTUALLY last run? (not when it was scheduled)
   - Compare: `journalctl -u cron --since "24h ago"` vs expected run count
   - Look for gaps: expected 288 runs/day (every 5min) but only 240 in logs = 48 silent failures
   - Check if the script's log file has recent writes: `stat -c %Y logfile`

3. OUTPUT VERIFICATION
   - Does the cron produce output? Where does it go?
   - `>> /dev/null 2>&1` = output silenced = bugs invisible. CRITICAL finding.
   - Log file exists but is empty? Script runs but produces nothing.
   - Log file grows unbounded? No rotation = disk bomb.

4. ENVIRONMENT ISOLATION
   - Cron runs with minimal PATH (/usr/bin:/bin). Does the script need tools not in that PATH?
   - Does the script rely on shell aliases, .zshrc/.bashrc? Cron doesn't load those.
   - Does the script use env vars set in interactive shells but not in cron?
   - Test: `env -i HOME=$HOME PATH=/usr/bin:/bin bash script.sh` — does it work?

5. OVERLAP PREVENTION
   - If script takes longer than interval, do two instances run simultaneously?
   - Is there a lockfile/flock mechanism? Is it correct? (stale lockfile = permanent block)
   - PID file check: does it verify the PID is actually alive, or just that the file exists?

6. FAILURE NOTIFICATION
   - On failure, does anyone get notified? (email, Telegram, Slack, log?)
   - Is MAILTO set in crontab? Does the mail system work?
   - Is there a dead man's switch? (alert if cron DOESN'T run, not just when it fails)
```

**Scoring:** 0 = script runs blind (no logs, no alerts, output silenced), 25 = every cron has verified execution, output capture, overlap protection, and failure alerting.

---

## PHASE 2: SCRIPT QUALITY INTERROGATION (weight: 25)

> *"A script without `set -euo pipefail` is a script that hides its own crimes."*

```
For EVERY .sh and .py script discovered:

1. SHELL SCRIPT HYGIENE
   - Shebang: #!/usr/bin/env bash (not /bin/sh, not missing)
   - set -euo pipefail present? (missing = errors silently ignored)
   - set -x for debug mode available? (toggleable, not always-on)
   - Shellcheck: run `shellcheck` if available, capture all warnings
   - Quote hygiene: unquoted $VARIABLES = word splitting bombs
   - cd without || exit: `cd /path` fails silently if path doesn't exist

2. PYTHON SCRIPT HYGIENE
   - Virtual environment: does it activate/use the right venv?
   - Import errors: does it import modules that aren't installed in prod?
   - Exception handling: bare `except:` or `except Exception:` swallows everything
   - Encoding: assumes UTF-8 but doesn't declare it
   - Argument parsing: uses sys.argv[1] without checking length

3. ERROR HANDLING DEPTH
   - What happens when the script fails at line 10 of 100? Does it:
     a) Stop immediately (set -e) ✓
     b) Continue and corrupt state ✗
     c) Clean up partial work (trap EXIT) ✓
     d) Leave temp files/locks behind ✗
   - Cleanup traps: `trap "rm -f $LOCKFILE; rm -rf $TMPDIR" EXIT`
   - Partial state: if script creates 10 files and fails at file 7, what's the state?

4. INPUT VALIDATION
   - Does the script validate its arguments? (count, type, existence)
   - Does it check if required files exist before operating on them?
   - Does it check if required commands are available? (`command -v tool || exit 1`)
   - Does it handle empty input gracefully?

5. OUTPUT INTEGRITY
   - Does the script produce machine-parseable output? (JSON, not freeform text)
   - Is output written atomically? (write to .tmp then mv, not direct write)
   - Does it set appropriate exit codes? (0=success, 1=error, not always 0)
   - Does it distinguish between "nothing to do" (0) and "tried and failed" (1)?

6. HARDCODED VALUES
   - Hardcoded paths that should be variables/configs
   - Hardcoded hostnames, ports, URLs
   - Hardcoded credentials (SECRET EXPOSURE — escalate to Phase 7)
   - Magic numbers without explanation
```

**Scoring:** 0 = scripts are fragile, unquoted, no error handling, 25 = every script passes shellcheck, has proper error handling, input validation, and atomic output.

---

## PHASE 3: DEPENDENCY ORDER & TIMING (weight: 25)

> *"Two scripts that run at the same time and touch the same files is not automation. It's a slot machine."*

```
1. TEMPORAL DEPENDENCY MAP
   Build the complete timeline: what runs when?
   Identify pairs where:
   - Script A produces output that Script B reads
   - A and B have no explicit ordering guarantee
   - A's worst-case runtime exceeds the gap before B starts
   
   Example: A runs at :00, takes 1-8min. B runs at :05, reads A's output.
   On a bad day, B reads yesterday's output. Nobody notices.

2. EXPLICIT vs IMPLICIT DEPENDENCIES
   - EXPLICIT: Script B calls Script A, or waits for A's output file
   - IMPLICIT: B assumes A already ran because "it's scheduled 5min earlier"
   - Every implicit dependency is a bug waiting for a slow day to trigger

3. DISPATCH CHAIN ANALYSIS
   For orchestration systems (AISB, oracle, worker dispatch):
   - Map the full chain: trigger → dispatch → worker → completion
   - Identify single points of failure in the chain
   - What happens if the dispatcher dies mid-dispatch?
   - What happens if two dispatchers run simultaneously?

4. PIPELINE ORDERING VERIFICATION
   For multi-stage pipelines (build → test → deploy):
   - Is each stage gated on the previous stage's SUCCESS (not just completion)?
   - Can stage N+1 start if stage N failed silently (exit 0 but wrong output)?
   - Is there a rollback mechanism if stage 3 of 5 fails?

5. CLOCK SKEW SENSITIVITY
   - Do scripts on different machines assume synchronized clocks?
   - NTP configured and running? (`timedatectl status`)
   - File timestamps used for ordering? (unreliable across mounts)
```

**Scoring:** 0 = automations have implicit timing dependencies with no guards, 25 = all dependencies are explicit, gated, and resilient to timing variance.

---

## PHASE 4: ERROR RECOVERY & RESILIENCE (weight: 20)

> *"The script that can't recover from its own failure is the script that will fail in production."*

```
1. RETRY LOGIC
   - Does the script retry transient failures? (network, API rate limits)
   - Is there exponential backoff? Or tight loop hammering?
   - Max retries bounded? (unbounded retry = infinite loop)
   - Is the retry condition correct? (retry on 429/503, NOT on 400/404)

2. PARTIAL FAILURE HANDLING
   - Script processes 100 items. Item 42 fails. What happens?
   - a) Abort everything (lose 58 items of work) — often wrong
   - b) Skip and continue (miss item 42 forever) — silent data loss
   - c) Log, continue, report failures at end — correct
   - d) Retry item 42, then continue — best

3. STATE RECOVERY
   - If script is killed mid-run (OOM, manual kill, reboot), can it resume?
   - Is there a checkpoint/progress file?
   - Does restart re-process everything from scratch? (idempotency test)
   - Are temp files cleaned up on abnormal exit?

4. GRACEFUL DEGRADATION
   - If an external service is down (API, database, remote host):
     a) Does the script hang forever? (no timeout)
     b) Does it fail fast with clear error?
     c) Does it use cached/fallback data?
   - Timeout values: are they set? Are they reasonable?
   - curl without --connect-timeout/--max-time = hangs forever
   - SSH without ConnectTimeout = hangs forever

5. REBOOT SURVIVAL
   - After system reboot, do all automations restart automatically?
   - Cron: yes (crond auto-starts). Daemons: only if systemd enabled.
   - Screen/tmux sessions: lost on reboot. Are they recreated?
   - PID files: stale after reboot. Are they cleaned?
```

**Scoring:** 0 = scripts crash on first error with no recovery, 20 = full retry logic, partial failure handling, state recovery, and reboot survival.

---

## PHASE 5: IDEMPOTENCY VERIFICATION (weight: 20)

> *"If running it twice gives different results, it's not automation. It's gambling."*

```
1. DOUBLE-RUN TEST (mental or actual)
   For each script: what happens if it runs twice in a row?
   - Creates duplicate records?
   - Appends to file twice?
   - Sends notification twice?
   - Overwrites with same data? (harmless but wasteful)
   - Crashes because lock file exists from first run?

2. CATCH-UP BEHAVIOR
   - System was down for 2 hours. Cron missed 24 runs (every 5min).
   - On restart: does it run 24 times? Once? Or detect gap and handle it?
   - anacron-style catch-up vs pure interval scheduling
   - Is catch-up safe? (24 simultaneous API calls = rate limit ban)

3. DATA INTEGRITY ON RE-RUN
   - INSERT vs UPSERT: does re-run duplicate data?
   - File creation: does re-run overwrite or append?
   - API calls: are they naturally idempotent? (PUT vs POST)
   - Side effects: does re-run trigger duplicate emails/notifications?

4. IDEMPOTENCY MARKERS
   - Does the script check "did I already process this?" before acting?
   - Markers: processed file list, database flag, lock file, output existence
   - Are markers reliable? (checking file existence is fragile)
```

**Scoring:** 0 = double-run creates duplicates/corruption, 20 = every automation is provably idempotent.

---

## PHASE 6: LOGGING & MONITORING (weight: 20)

> *"An automation without logs is an automation you'll debug with `strace` at 3 AM."*

```
1. LOG EXISTENCE & QUALITY
   - Every automation must write to a log file. No exceptions.
   - Log format: timestamp + level + message (not just freeform echo)
   - Log location: centralized or scattered? (/var/log, ~/.omega/logs, project/logs)
   - Are logs human-readable AND machine-parseable?

2. LOG ROTATION
   - Do log files have rotation? (logrotate, custom, or nothing?)
   - Without rotation: log grows until disk full = ALL automations fail
   - Rotation config: compress? retain how many? rotate by size or age?
   - Check ACTUAL sizes: `du -sh *.log` — any monster files?

3. ERROR ESCALATION
   - When an automation fails, who knows?
   - Chain: log → monitor → alert → human
   - Is the chain tested? (kill the script intentionally — does the alert fire?)
   - Silent failure: script fails, logs the error, but nobody reads the log = unmonitored

4. HEALTH CHECKS
   - Is there a meta-monitor that checks if monitors are running?
   - Dead man's switch: alert if automation DOESN'T report success
   - Heartbeat pattern: daemon writes timestamp, monitor checks freshness
   - Stale heartbeat detection: how old before alert? Is it calibrated?

5. METRIC EMISSION
   - Do automations emit metrics? (duration, success/fail count, items processed)
   - Where? (StatsD, Prometheus, custom JSON, Telegram)
   - Can you answer "how long did last night's backup take?" without reading logs?
```

**Scoring:** 0 = no logs or unrotated growing logs, 20 = structured logs, rotation, error escalation, health checks, and metrics.

---

## PHASE 7: SECRET EXPOSURE FORENSICS (weight: 25)

> *"The API key in the shell script is the API key in the git history forever."*

```
1. HARDCODED SECRETS IN SCRIPTS
   Scan every .sh, .py, .js script for:
   - API keys (pattern: sk-, pk-, key-, token=, api_key=, SECRET)
   - Passwords (password=, passwd=, PASS=)
   - Connection strings (postgres://, mysql://, redis://, mongodb://)
   - Private keys (BEGIN RSA PRIVATE KEY, BEGIN OPENSSH PRIVATE KEY)
   Tool: `gitleaks detect --no-git --source <dir>` or manual grep

2. SECRETS IN GIT HISTORY
   Even if removed from current files, secrets live in git forever:
   `gitleaks detect --source <repo>` (scans full history)
   `trufflehog git file://. --only-verified`

3. SECRETS IN LOGS
   Do scripts echo secrets during execution?
   - `echo "Connecting with key: $API_KEY"` → key in log file
   - `set -x` with secrets → expanded in trace output
   - `curl -v -H "Authorization: Bearer $TOKEN"` → token in verbose output

4. ENVIRONMENT VARIABLE HYGIENE
   - Are secrets loaded from .env files? (not hardcoded)
   - Are .env files in .gitignore?
   - Are .env files readable only by the intended user? (chmod 600?)
   - Do scripts source .env correctly? (`set -a; source .env; set +a`)

5. SECRET ROTATION
   - How old are the secrets? (check git blame on .env files)
   - Is there a rotation schedule/mechanism?
   - If a secret is compromised, how fast can it be rotated?
   - Are old secrets revoked or just... still valid?

6. PERMISSION ESCALATION
   - Any scripts running as root that don't need to?
   - Any scripts with SUID/SGID bits?
   - Any scripts writable by non-owner? (`find -perm -o+w`)
   - sudo without password in scripts? (NOPASSWD in sudoers for script user?)
```

**Scoring:** 0 = hardcoded secrets found in scripts/history, 25 = all secrets in env vars, no history exposure, proper permissions, rotation plan.

---

## PHASE 8: DAEMON & SERVICE HEALTH (weight: 20)

> *"A daemon is a process that promised to run forever. Check when it actually last did something useful."*

```
1. PROCESS EXISTENCE
   - Is the daemon actually running? (ps aux, systemctl status)
   - If systemd: is it enabled (auto-start on boot)?
   - If manual: how is it started? Who restarts it if it dies?
   - Zombie check: process exists but parent is init (orphaned)

2. RESOURCE CONSUMPTION
   - CPU: is the daemon idle or busy? Should it be?
   - Memory: RSS growth over time (memory leak pattern)
   - File descriptors: `ls /proc/<pid>/fd | wc -l` (FD leak)
   - Disk I/O: is it writing excessively?

3. OUTPUT FRESHNESS
   - When did the daemon last produce useful output?
   - Status file/heartbeat: how old? Is staleness detected?
   - If the daemon's purpose is monitoring: is IT being monitored?

4. RESTART POLICY
   - systemd: Restart=on-failure? RestartSec reasonable?
   - Manual: who restarts it? Is there a watchdog?
   - After crash: does it clean up state before restarting?
   - OOM kill history: `dmesg | grep -i oom`

5. GRACEFUL SHUTDOWN
   - Does the daemon handle SIGTERM properly?
   - Does it flush buffers, close connections, release locks?
   - Or does SIGTERM leave corrupted state requiring manual cleanup?
```

**Scoring:** 0 = unmonitored daemons with no restart policy, 20 = all daemons monitored, auto-restart, resource-tracked, graceful shutdown.

---

## PHASE 9: RACE CONDITIONS & CONCURRENCY (weight: 20)

> *"Two scripts, one file, zero locks. The data integrity lottery."*

```
1. SHARED RESOURCE IDENTIFICATION
   - Which automations read/write the same files?
   - Which automations hit the same APIs? (rate limit collision)
   - Which automations modify the same database tables?
   - Which automations use the same git repos? (merge conflicts)

2. LOCKING MECHANISMS
   - flock used? (correct: `flock -n /tmp/lock.file command`)
   - PID file used? (fragile: PID reuse after reboot)
   - mkdir used as lock? (atomic on local FS, not on NFS)
   - Database locks? Application-level locks?

3. READ-AFTER-WRITE HAZARDS
   - Script A writes file at :00. Script B reads it at :00.
   - Is the read guaranteed to see A's COMPLETE write?
   - Atomic write pattern: write to .tmp, mv to final. Is it used?
   - File is being read while another process appends to it

4. GIT CONCURRENT ACCESS
   - Multiple scripts doing git pull/push on same repo
   - Without locking: merge conflicts, detached HEAD, index.lock
   - oracle-ship.sh already uses flock — do other scripts?

5. API RATE LIMIT COLLISION
   - Multiple crons hitting the same API (Linear, Clerk, Vercel)
   - Combined request rate exceeds rate limit
   - Staggering: are schedules offset to avoid collision?
```

**Scoring:** 0 = multiple automations touching same resources without locks, 20 = all shared resources protected, atomic writes, staggered schedules.

---

## PHASE 10: DEAD AUTOMATION DETECTION (weight: 15)

> *"The scariest automation is the one that stopped running 6 months ago and nobody noticed."*

```
1. CRON ENTRIES POINTING TO MISSING SCRIPTS
   For each cron line: does the script path exist?
   `crontab -l | grep -v '^#' | awk '{print $NF}' | while read cmd; do ...`

2. SCRIPTS NEVER EXECUTED
   - `.sh` files that exist but were never run (no log, no recent mtime)
   - Check git: when was the script last modified? By whom?
   - Check access time: `stat -c %x script.sh`

3. COMMENTED-OUT CRONS
   - Disabled crons: why? Temporary? Forgotten?
   - If temporary: how long has it been commented out?
   - Should it be deleted or re-enabled?

4. ORPHANED LOG FILES
   - Log files still being rotated but the automation is dead
   - Log files growing but from a zombie process, not the intended script

5. STALE CONFIGS
   - Config files for automations that no longer exist
   - Environment variables for deprecated services
   - Systemd unit files for removed daemons

6. DEPRECATED BUT RUNNING
   - Automations that are superseded but still active
   - Old version runs alongside new version (double execution)
   - Migration left both old and new running "just in case"
```

**Scoring:** 0 = dead automations discovered, 15 = all automations are alive, intentional, and documented.

---

## PHASE 11: RESOURCE IMPACT ANALYSIS (weight: 15)

> *"Cron jobs are free until they aren't. Then they're the reason your VPS costs $800/month."*

```
1. CPU BUDGET
   - Sum of all automation CPU usage vs available headroom
   - Peak hour analysis: when do most crons cluster?
   - Starvation risk: automations consuming CPU that interactive sessions need

2. MEMORY BUDGET
   - Each script's peak memory usage
   - Combined memory during peak concurrent execution
   - OOM risk: total automation memory + application memory > physical RAM?

3. DISK I/O
   - Log write volume per hour
   - Temp file creation/deletion rate
   - Backup scripts: how much disk I/O per run?

4. NETWORK
   - API call frequency across all automations
   - Bandwidth consumption (downloads, uploads, scraping)
   - External dependency count: how many services must be up?

5. SCHEDULE CLUSTERING
   - How many crons run at :00? (the "top of the hour" stampede)
   - Staggering recommendation: spread across the minute
   - Priority: which automation gets resources first during contention?
```

**Scoring:** 0 = resource contention causes failures, 15 = resource budgets known, schedules staggered, headroom maintained.

---

## PHASE 12: DISPATCH & ORCHESTRATION CHAINS (weight: 20)

> *"An orchestrator that dispatches workers without tracking their completion is a manager who sends emails and calls it leadership."*

```
1. DISPATCH TOPOLOGY
   Map every dispatch chain:
   - AISB → Oracle → Worker → Completion
   - Cron → Script → Subprocess → Output
   - CI trigger → Build → Test → Deploy → Verify

2. COMPLETION TRACKING
   - After dispatching, does the orchestrator verify completion?
   - worker-alive-check.sh: is it used before killing workers?
   - Done signals: .done.json, exit code, output file existence
   - Timeout: what if the worker never finishes? Is there a ceiling?

3. FAILURE PROPAGATION
   - Worker fails → does the orchestrator know?
   - Does it retry? Escalate? Ignore?
   - Error details preserved or lost in the dispatch chain?

4. QUEUE MANAGEMENT
   - If dispatch rate > completion rate: queue grows unbounded
   - Is there backpressure? Max concurrent workers?
   - What happens to queued work if the orchestrator restarts?

5. ORPHAN DETECTION
   - Dispatched workers that the orchestrator lost track of
   - tmux sessions without an oracle parent
   - Background processes from killed parent scripts
```

**Scoring:** 0 = fire-and-forget dispatch with no completion tracking, 20 = full dispatch tracking, failure propagation, backpressure, orphan cleanup.

---

## PHASE 13: FAILURE CASCADE ANALYSIS (weight: 20)

> *"When the backup script fails, the monitoring script that watches backups also fails because it depends on the same disk that's full."*

```
1. DEPENDENCY GRAPH
   Build: {automation → [automations it depends on]}
   Identify: single points of failure (SPOF)
   - If automation X dies, which others cascade fail?
   - Depth of cascade: 1 hop? 5 hops?

2. COMMON FAILURE MODES
   - Disk full → ALL automations that write logs/output fail
   - Network down → ALL automations that call APIs fail
   - DNS failure → ALL automations using hostnames fail
   - Auth token expired → ALL automations using that token fail

3. BLAST RADIUS SCORING
   For each automation: if it fails, how many others fail?
   blast_radius = count of dependent automations + count of affected services
   Sort by blast radius descending — highest = hinge automation candidates

4. CIRCUIT BREAKER PATTERNS
   - When a downstream service is down, do automations:
     a) Keep hammering it? (makes it worse)
     b) Back off exponentially? (correct)
     c) Trip a circuit breaker? (best)
   - Is there a global "freeze all automations" switch?
```

**Scoring:** 0 = cascading failures with no isolation, 20 = failure isolation, circuit breakers, SPOF identification, blast radius documented.

---

## PHASE 14: LOCK FILE MANAGEMENT (weight: 15)

> *"A stale lock file is an automation that stopped running but prevented all future runs from starting."*

```
1. LOCK FILE INVENTORY
   Find all lock mechanisms:
   - `find / -name "*.lock" -o -name ".lock" -o -name "lockfile" 2>/dev/null`
   - flock references in scripts
   - PID files used as locks
   - mkdir-based locks

2. STALE LOCK DETECTION
   For each lock file:
   - How old is it? (older than the script's max runtime = stale)
   - Is the PID still alive? (`kill -0 $(cat pidfile)`)
   - Is the flock holder still running?

3. LOCK CLEANUP
   - Do scripts clean up locks on exit? (trap EXIT)
   - Do scripts clean up locks on crash? (trap ERR/SIGTERM)
   - After system reboot: are old locks cleaned?
   - Is there a sweeper that removes stale locks periodically?

4. LOCK GRANULARITY
   - Global lock (only one instance of anything) vs per-task lock
   - Too coarse: one slow job blocks all others
   - Too fine: complex lock hierarchy = deadlock risk

5. DEADLOCK SCENARIOS
   - Script A holds lock 1, waits for lock 2
   - Script B holds lock 2, waits for lock 1
   - Is this possible in the current automation set?
```

**Scoring:** 0 = stale locks blocking automations, 15 = all locks have timeouts, cleanup traps, and deadlock prevention.

---

## PHASE 15: PATH, ENVIRONMENT & PORTABILITY (weight: 10)

> *"It works on my machine because my machine has 47 things in PATH that cron doesn't."*

```
1. PATH COMPLETENESS
   - Scripts use tools not in cron's default PATH
   - Full path vs relying on PATH: `/usr/local/bin/node` vs `node`
   - ~/.local/bin, ~/.bun/bin, ~/.cargo/bin — in script's PATH?

2. ENVIRONMENT ASSUMPTIONS
   - Scripts assume HOME, USER, SHELL are set (they are in cron, but not in all contexts)
   - Scripts assume .zshrc/.bashrc loaded (cron doesn't load them)
   - Scripts assume specific working directory (cron starts in $HOME)

3. SHELL COMPATIBILITY
   - Script uses bash-isms but shebang says /bin/sh
   - Array syntax, process substitution, [[ ]] in a POSIX script
   - Zsh-specific syntax in a script that might run under bash (cron uses /bin/sh by default)

4. TOOL AVAILABILITY
   - Script uses jq, yq, curl, wget, node, python3, bun — are they installed?
   - Specific versions assumed? (python3.11 vs python3)
   - Tools installed globally vs in virtualenv/nvm
```

**Scoring:** 0 = scripts break in cron environment, 10 = all scripts use full paths, correct shebangs, and validate tool availability.

---

## PHASE 16: BACKUP & RECOVERY VERIFICATION (weight: 15)

> *"The backup ran successfully for 6 months. We tested restore and it failed on month 1."*

```
1. BACKUP EXISTENCE
   - Is there a backup automation? For what? (database, files, config, secrets)
   - Schedule: how often? Is it enough? (RPO analysis)
   - Storage: where? Same disk = not a backup. Offsite = good.

2. BACKUP INTEGRITY
   - Is the backup file valid? (not truncated, not corrupted)
   - Can it actually be restored? When was restore last tested?
   - Is the backup complete? (all tables, all files, not just partial)

3. BACKUP FRESHNESS
   - When was the last backup created? (check file timestamp)
   - Is the backup automation still running? (not dead/disabled)
   - How much data would be lost if we restored right now? (RPO reality)

4. RESTORE PROCEDURE
   - Is there a documented restore procedure?
   - Has it been tested? When?
   - How long does restore take? (RTO)
   - Can restore run while the system is live? (online vs offline)
```

**Scoring:** 0 = no backups or untested backups, 15 = verified backups with tested restore, documented procedure, and acceptable RPO/RTO.

---

## PHASE 17: PERMISSION & OWNERSHIP AUDIT (weight: 10)

> *"The script that runs as hacker but writes to a root-owned directory will fail at 3 AM and nobody will know why."*

```
1. FILE OWNERSHIP
   - Scripts owned by the right user? (hacker, not root)
   - Log directories writable by the script's user?
   - Output directories exist and have correct permissions?

2. EXECUTION PERMISSION
   - All .sh scripts have +x? (or are they invoked via `bash script.sh`?)
   - Consistency: some scripts executable, others not

3. SENSITIVE FILE PERMISSIONS
   - .env files: 600 or 640? (not world-readable)
   - Private keys: 600? (SSH will refuse if too open)
   - Config files with secrets: not world-readable?

4. SUDO USAGE
   - Scripts that require sudo: do they have NOPASSWD configured?
   - Is the NOPASSWD scope minimal? (not ALL commands)
   - Scripts that DON'T need sudo but use it anyway
```

**Scoring:** 0 = permission errors causing silent failures, 10 = correct ownership, minimal sudo, secure file permissions.

---

## PHASE 18: DOCUMENTATION & INTENT (weight: 10)

> *"The script has no comments, no README, and the developer left the company. Good luck."*

```
1. SCRIPT DOCUMENTATION
   - Does each script explain WHAT it does, WHY, and WHEN it runs?
   - Header comment with: purpose, schedule, dependencies, author, last update
   - Inline comments for non-obvious logic

2. CRONTAB DOCUMENTATION
   - Each cron entry has a comment explaining its purpose
   - Sections/groups for related crons
   - Contact info: who owns each automation?

3. RUNBOOK EXISTENCE
   - If automation X fails, what's the manual recovery procedure?
   - Is the runbook current? (references valid commands, paths, configs)
   - Can a new team member follow it without asking questions?

4. CHANGE LOG
   - When was each automation last modified?
   - Why? (git blame, commit messages)
   - Is there a testing procedure for automation changes?
```

**Scoring:** 0 = undocumented automations, 10 = every automation documented with purpose, schedule, dependencies, and runbook.

---

## PHASE 19: VERDICT & SCORING

```
SCORING MATRIX (22 phases, /400 total):

| Phase | Domain | Weight | Score |
|-------|--------|--------|-------|
| 1  | Cron Health             | 25 | /25 |
| 2  | Script Quality          | 25 | /25 |
| 3  | Dependency Order        | 25 | /25 |
| 4  | Error Recovery          | 20 | /20 |
| 5  | Idempotency             | 20 | /20 |
| 6  | Logging & Monitoring    | 20 | /20 |
| 7  | Secret Exposure         | 25 | /25 |
| 8  | Daemon Health           | 20 | /20 |
| 9  | Race Conditions         | 20 | /20 |
| 10 | Dead Automations        | 15 | /15 |
| 11 | Resource Impact         | 15 | /15 |
| 12 | Dispatch Chains         | 20 | /20 |
| 13 | Failure Cascade         | 20 | /20 |
| 14 | Lock Management         | 15 | /15 |
| 15 | Path & Environment      | 10 | /10 |
| 16 | Backup & Recovery       | 15 | /15 |
| 17 | Permission & Ownership  | 10 | /10 |
| 18 | Documentation           | 10 | /10 |
| TOTAL |                      | 330 | /330 |

Normalized: (raw / 330) × 100 = /100

GRADE THRESHOLDS:
  S  (95-100): Bulletproof automation. Every script monitored, idempotent, documented.
  A  (85-94):  Production-grade. Minor documentation or edge case gaps.
  B  (70-84):  Functional but fragile. Missing monitoring, stale locks, or timing bugs.
  C  (55-69):  Risky. Silent failures likely. Multiple unmonitored automations.
  D  (40-54):  Dangerous. Race conditions, secret exposure, dead automations undetected.
  F  (0-39):   Hostile to reliability. Automations actively causing harm.
```

Write to `audits/.automationaudit/verdict.json`:
```json
{
  "score_raw": N,
  "score_max": 330,
  "score_normalized": N,
  "grade": "X",
  "preamble_version": "1.0",
  "hinge_automation": "name of most critical automation",
  "findings": [...],
  "dead_automations": [...],
  "secret_exposures": [...],
  "race_conditions": [...],
  "spofs": [...]
}
```

Write human-readable `audits/.automationaudit/verdict.md` with full narrative.

---

## PHASE 20: FIX PLAN GENERATION

Produce `audits/.automationaudit/fix-plan.json` and `audits/.automationaudit/fix-plan.md`:

```
Priority order:
1. CRITICAL — Secret exposure (rotate immediately)
2. CRITICAL — Race conditions causing data corruption
3. HIGH — Dead automations (remove or fix)
4. HIGH — Missing error handling (add set -euo pipefail)
5. HIGH — Silent failures (add logging + alerting)
6. MEDIUM — Stale locks (add cleanup traps)
7. MEDIUM — Missing log rotation
8. MEDIUM — Implicit dependencies (make explicit)
9. LOW — Documentation gaps
10. LOW — Path/portability issues
```

Each fix task:
```json
{
  "id": "FIX-001",
  "finding": "Secret API_KEY hardcoded in deploy-hook.sh line 42",
  "file": "~/.omega/lib/deploy-hook.sh",
  "line": 42,
  "fix": "Move to .env, source .env in script, add .env to .gitignore",
  "status": "pending",
  "severity": "CRITICAL"
}
```

---

## PHASE 21: FIX EXECUTION

Execute fixes from fix-plan.json, sequential, with verification:

```
For each fix:
1. Read the current state of the file
2. Apply the fix (Edit tool for precision)
3. Verify the fix:
   - For scripts: shellcheck (if .sh), python -c "import ast; ast.parse(open('f').read())" (if .py)
   - For crons: validate cron expression
   - For secrets: re-scan with gitleaks
   - For permissions: verify with ls -la
4. Test if the automation still works: dry-run if possible
5. Update fix-plan.json: status → "done"
6. Append to fix-log.md
```

**Integration smoke test (preamble §11):**
After all fixes, verify existing automations still function:
- Cron entries still valid: `crontab -l | grep -v '^#' | head -5`
- Critical scripts still executable: test each with `bash -n script.sh`
- Daemons still running: `systemctl status` for each

If any automation breaks post-fix → revert that specific fix, mark NEEDS_REVIEW.

---

## PHASE 22: RE-AUDIT

Re-run failing phases (score < weight) to verify fixes:

```
iteration = 0
while score < target:
    iteration += 1
    re-run failing phases with --focus
    record score trajectory in audits/.automationaudit/iterations.md
    if iteration >= 5:
        mark remaining as NEEDS_REVIEW
        send Telegram SOS
        exit loop
```

Write final telemetry:
```json
{
  "audit": "automationaudit",
  "version": "1.0",
  "preamble_version": "1.0",
  "duration_seconds": N,
  "phases_run": 22,
  "fixes_applied": N,
  "iterations": N,
  "final_score": N,
  "final_grade": "X",
  "model": "opus"
}
```

---

## CROSS-COMMAND BRIDGE

`/automationaudit` is automation-first but MUST handle related issues it discovers:

```
IF during Phase 7 (Secret Exposure) you find secrets in code files:
  → FIX the secret exposure (rotate key, move to .env)
  → Don't defer to /secaudit — you own automation secrets

IF during Phase 9 (Race Conditions) you find git conflicts from parallel workers:
  → FIX the locking mechanism
  → Note for /codeaudit if the conflict pattern is in application code

IF during Phase 12 (Dispatch Chains) you find oracle/worker bugs:
  → FIX the dispatch logic
  → Note for /flowaudit if it affects user-facing flows

RULE: /automationaudit handles EVERYTHING related to automation infrastructure.
It doesn't punt to other audits. If it's automated and broken, fix it.
```

---

## ECOSYSTEM INTEGRATION

- **Before this audit:** `/codeaudit` (ensures code quality of scripts), `/secaudit` (broader security context)
- **After this audit:** `/perfaudit` (automation resource impact on performance), `/debugaudit` (runtime bugs from automation failures)
- **Complementary:** `/dxaudit` (developer experience with automation tooling), `/dataaudit` (data integrity affected by automation)
- **SMITH integration:** Results feed into SMITH for systemic improvement proposals on automation patterns

---

## MANDATORY BEFORE/AFTER VERIFICATION (v1.1)

**Read `../_shared/AUDIT-VERIFICATION-CONTRACT.md` before ANY fix execution.**

Automation fixes are HIGH RISK (touch live crons, daemons, scripts). Every fix MUST:

1. **BACKUP FIRST** — `crontab -l > /tmp/crontab.backup-$(date +%s).txt` for cron changes; `cp script script.backup-$(date +%s)` for script changes.
2. **PRE-FIX BASELINE** — `bash -n` every script, validate cron expressions, check `systemctl status` for daemons. Save to `audits/.automationaudit/baseline/`.
3. **APPLY FIX** — normal execution.
4. **POST-FIX CHECK** — re-run every baseline check. If anything broke, restore from backup IMMEDIATELY.
5. **DRY-RUN NEW SCHEDULES** — for cron changes, verify with `cron-validator` or manual expression parse. Never push invalid cron to live crontab.
6. **BREAKAGE SCAN** — grep for references to any archived/moved scripts. Must be 0.
7. **BEFORE/AFTER MATRIX** — `audits/.automationaudit/before-after.md` required with live service status table.

**DO NOT claim "done" if any cron/daemon/script that worked before now fails.** Always produce rollback commands.

---

## COMPLIANCE & CRITICAL ADDENDA (v1.0)

### Quality Arsenal Preamble Compliance

This audit implements contracts from `../_shared/QUALITY-ARSENAL-PREAMBLE.md` v1.0:

- ✅ **Gestalt-Popper doctrine** — hinge automation, falsification, evidence chain
- ✅ **Concurrency lock** — canonical runner `flock` at `audits/.automationaudit/.runner.lock`
- ✅ **5-iteration cap** — fix-and-reaudit bounded
- ✅ **Scoped invocation flags** — `--files=`, `--scope=`, `--focus=`, `--no-fix`
- ✅ **Non-UI context gate** — runs on ALL project types (automation is universal)
- ✅ **Output contract verification** — all mandatory files emitted and verified
- ✅ **Telegram progress notifications** — via `audit-notify.sh`
- ✅ **Self-telemetry** — `audits/.automationaudit/telemetry.json`
- ✅ **Rule-46 compliance** — NO `--quick`/`--streamlined` variants
- ✅ **Score normalization** — raw/330 × 100 = /100

---

## Dynamic-Workflow Orchestration (v2)

> *"Linear auditing of an automation fleet is itself a single point of failure. Fan out, falsify, then converge — the same discipline you demand of the crons you judge."*

This section governs HOW the 22 phases above are EXECUTED. It changes the execution model only — **every phase definition, weight, verdict format, scoring matrix (/330 → /100), grade threshold, and frontmatter above is unchanged and authoritative**. The Gestalt-Popper doctrine and the HINGE AUTOMATION (10x scrutiny) remain the spine; this just runs the spine in parallel and verifies each vertebra adversarially.

### 1. Fan-out — decompose phases into independent parallel tracks

After Phase 0 discovery completes (crontab, systemd timers, daemons, scripts inventory, dispatch chains, topology), the phases partition into **file/resource-disjoint tracks** that have no read/write dependency on each other. Launch them concurrently via the **Workflow** tool instead of running 1→22 linearly. Suggested tracks for THIS domain (regroup if discovery reveals different seams):

| Track | Phases | Disjoint surface it reads |
|-------|--------|---------------------------|
| **A — Schedule & Timing** | 1 (Cron Health), 3 (Dependency Order), 11 (Resource Impact) | crontab, systemd timers, `journalctl`, temporal map |
| **B — Script Integrity** | 2 (Script Quality), 5 (Idempotency), 15 (Path & Env), 18 (Documentation) | the `.sh`/`.py` files themselves (static + shellcheck + `bash -n`) |
| **C — Runtime & Liveness** | 8 (Daemon Health), 10 (Dead Automations), 14 (Lock Management) | `ps`/`systemctl`, lock files, process FDs, heartbeats |
| **D — Resilience & Orchestration** | 4 (Error Recovery), 12 (Dispatch Chains), 13 (Failure Cascade), 16 (Backup & Recovery) | dispatch topology, done-signals, backup artifacts |
| **E — Safety Surface** | 6 (Logging & Monitoring), 7 (Secret Exposure), 9 (Race Conditions), 17 (Permission & Ownership) | logs, git history (gitleaks), perms, shared-resource map |

Rules:
- **Disjointness is mandatory (R-SCOPE).** Two tracks must never write the same finding file or touch the same script under fix; if discovery shows a script that lives in two tracks (e.g. a daemon that is also a cron target), assign it to ONE owning track and have the other track cross-reference, never co-write.
- **The HINGE AUTOMATION is audited inside its owning track at 10x depth** AND its findings are broadcast to every other track (a hinge failure cascades across tracks by definition — Phase 13).
- Each track returns structured findings (`{phase, file:line, claim, evidence_cmd, evidence_output, severity}`) — never prose. Tracks do NOT score; scoring stays centralized (step 3).
- Tracks are read-only/observational during fan-out. **All fixes (Phases 20-22) run serialized after synthesis** — parallel mutation of crons/daemons/scripts is forbidden (one-writer-per-resource, per the BEFORE/AFTER contract and R-SCOPE).

### 2. Adversarial verification — every finding survives ≥2-of-3 independent lenses

A track-produced finding is a **candidate**, never an accepted finding. Before it may enter the verdict, subject it to three independent lenses and require **at least 2 of 3 to agree** (Popper: actively try to kill it). Kill — do not record — any candidate that fails this gate.

| Lens | For automation, this means | Example |
|------|----------------------------|---------|
| **REPRODUCE** | Re-run the claim against live runtime (L1: runtime is the only truth). | Claim "cron X silently fails" → run it under cron's environment: `env -i HOME=$HOME PATH=/usr/bin:/bin bash script.sh; echo "exit=$?"` and read its real log mtime + `journalctl`. |
| **REFUTE** | Construct the strongest case the finding is WRONG; look for the guard that already handles it. | Claim "no overlap protection" → grep the script + its wrapper + crontab for `flock`/PID-check/`mkdir` lock; if a guard exists and works, the finding dies. |
| **CROSS-CHECK** | Confirm via an independent signal/tool that doesn't share the first method's failure mode. | Claim "secret hardcoded" → confirm with BOTH `gitleaks detect --no-git` AND a raw `grep -nE` for the key pattern; "daemon dead" → confirm via `systemctl status` AND output-file freshness AND `kill -0 $(cat pidfile)`. |

Gate logic:
- **≥2 lenses agree → ACCEPTED.** Record with all three lens results in `evidence` (R-CITE: file:line + command + verbatim output). Banned phrases (`looks correct`, `should be fine`, `appears to work`) auto-fail the lens.
- **Lenses split 1-accept / 1-refute / 1-inconclusive → ESCALATE**, do not silently keep: mark `needs_review` in verdict.json with both sides cited. Never let an unfalsified claim count as a finding.
- A finding that REPRODUCE cannot trigger on live runtime is downgraded to `theoretical` and excluded from the score-affecting findings (it may still appear as a hardening note).
- CRITICAL candidates (secret exposure, data-corrupting race) require REPRODUCE to be one of the two passing lenses — static signal alone is insufficient to call a secret "live-exposed".

### 3. Synthesis — converge surviving findings into the UNCHANGED scoring matrix

Synthesis is the auditor's own job (R-ORCH: never paste a track's summary as the verdict). After all tracks return and every candidate has passed the ≥2-of-3 gate:

1. **De-duplicate across tracks.** The same root cause surfacing in two tracks (e.g. a missing `set -euo pipefail` flagged by Track B Phase 2 and its silent-failure consequence by Track E Phase 6) collapses to ONE finding with cross-track confirmation (elevate severity per preamble §6 cross-audit elevation — two independent tracks agreeing on one file:line ⇒ CRITICAL).
2. **Map each surviving finding back to its phase weight** and score that phase exactly as Phase 19 defines — Cron Health /25, Script Quality /25, Dependency Order /25, Secret Exposure /25, Error Recovery /20, … Documentation /10. **No weight is changed, added, or reordered.**
3. **Compute the raw /330 and normalize `(raw / 330) × 100 = /100`** and assign the S/A/B/C/D/F grade exactly per the existing thresholds. Declare `hinge_automation`, `dead_automations`, `secret_exposures`, `race_conditions`, `spofs` in `verdict.json` per the existing schema — now each entry carries its 2-of-3 lens evidence.
4. **Proceed to Phases 20→22 (fix plan, serialized fix execution with the mandatory BEFORE/AFTER baseline + rollback, re-audit, 5-iteration cap) exactly as written.** Re-audit of a fixed track re-runs that track's phases through the SAME fan-out + 2-of-3 gate.

### 4. Loop-until-dry — exhaust unknown-size discovery

The automation fleet is unbounded a priori (scripts spawn scripts; crons reference wrappers that reference more scripts; dispatch chains fork workers). Discovery is therefore a **loop, not a single pass**:

```
discovered = initial_inventory()      # Phase 0 crontab + timers + find + ps
seen = ∅
while discovered \ seen ≠ ∅:          # until the frontier is dry
    batch = discovered \ seen
    seen ∪= batch
    fan_out(batch across tracks A–E)   # step 1
    verify_2_of_3(batch_findings)      # step 2
    # expand the frontier from what we just learned:
    discovered ∪= referenced_scripts(batch)      # scripts/binaries each script calls
    discovered ∪= dispatch_children(batch)        # workers/subprocesses a dispatcher spawns
    discovered ∪= cron_wrapper_targets(batch)     # the real script behind a cron wrapper
    discovered ∪= timer_unit_execstart(batch)     # ExecStart= of each systemd timer/service
guard: cap at 5 discovery rounds (preamble §4 iteration cap); if frontier still
       non-empty at round 5 → record remaining as `needs_review` + Telegram SOS,
       never loop unbounded.
```

A track reports **DRY** only when re-running its discovery finds zero new automations AND zero new candidate findings. Synthesis (step 3) runs only after every track is DRY or the 5-round guard trips. This guarantees a dead cron buried three wrapper-levels deep, or a worker forked by a dispatcher forked by a cron, is still found — without an arbitrary fixed phase count.

**Net effect:** identical inputs and identical verdict semantics as v1, reached faster and with far higher confidence — findings are concurrent-discovered, adversarially falsified (≥2-of-3), de-duplicated across tracks, and folded into the unchanged /330 matrix. The audit's identity, doctrine, and scoring are preserved; only its execution becomes a verified parallel workflow instead of a linear walk.

---

*"/automationaudit v1 — Every cron is a promise. Every script is a liability. Every daemon is a lie. Prove them all wrong."*

---
name: audit-tracker
description: >
  Audit setup + tracking dashboard. Use when user says "/audit-tracker", "audit
  status", "audit dashboard", "audit history", "list audits", "where am I with
  audits", "setup audits", "init audits". Ensures audits/ folder exists, .gitignore
  configured, tracks all audits run with scores + freshness, recommends re-runs
  when stale (>30 days). Reads audits/.<audit-id>/verdict.json across all
  audit subdirs to build dashboard.
disable-model-invocation: false
---

# /audit-tracker — Setup + Progress Dashboard

You are the **audit accountant**. Init audit infrastructure for a project and
report status of all past + ongoing audits.

## Modes

```bash
/audit-tracker init           # setup audits/ + .gitignore + initial SYNTHESIS.md
/audit-tracker                # dashboard: status of all audits
/audit-tracker stale          # only audits older than 30 days
/audit-tracker scores         # only the scores table (compact)
/audit-tracker latest         # most recent audit + summary
```

## Mode 1 — `/audit-tracker init`

Bootstrap audits infrastructure in the current project:

1. Create `audits/` directory if missing
2. Append to `.gitignore` (idempotent — only if not already present):
   ```gitignore
   # Audit outputs (Quality Arsenal)
   /audits/.*audit*/
   !/audits/.*audit*/verdict.json
   !/audits/.*audit*/REPORT.md
   !/audits/.*audit*/CHECKLIST.md
   !/audits/SYNTHESIS.md
   ```
   This ignores the bulky audit artifacts but preserves the headline outputs
   (verdict.json, REPORT.md, SYNTHESIS.md).
3. Write `audits/SYNTHESIS.md` skeleton:
   ```markdown
   # Audit Synthesis — {project_name}

   Last update: 2026-05-13
   Status: 🟡 No audits run yet

   ## Recommended starting audits

   - `/audit-orchestrator quick` — gut-check (15 min)
   - `/audit-orchestrator standard` — regular quality cycle (60 min)
   - `/audit-orchestrator full` — complete arsenal (4h)

   ## Past runs

   _none yet_
   ```
4. Output to user: "✅ Audits initialized. Run /audit-orchestrator to start."

## Mode 2 — `/audit-tracker` (dashboard)

Scan `audits/` for all `<audit-id>/verdict.json` files. Build a markdown table:

```
🎯 AUDIT DASHBOARD — {project_name}

┌──────────────────────┬──────┬──────┬───────────┬────────────────┐
│ Audit                │ Score │ Grade │ Age      │ Status         │
├──────────────────────┼──────┼──────┼───────────┼────────────────┤
│ codeaudit (v2)       │  92  │  A   │  2 days   │ ✅ Fresh       │
│ secaudit             │  88  │  A   │  5 days   │ ✅ Fresh       │
│ uiuxaudit (v3)       │  91  │  S   │  3 days   │ ✅ Fresh       │
│ a11yaudit (v2)       │  88  │  A   │ 14 days   │ ⚠️ Aging       │
│ perfaudit            │  79  │  B   │ 35 days   │ 🔴 Stale       │
│ apiaudit             │  67  │  C   │ 12 days   │ 🟡 Re-audit    │
└──────────────────────┴──────┴──────┴───────────┴────────────────┘

Overall health: 84/100 (Grade A-)
Recommended: re-run /perfaudit (stale 35d), push /apiaudit to >85 (re-audit)
```

Status thresholds:
- **Fresh** ≤ 7 days
- **Aging** 8-30 days
- **Stale** > 30 days (recommend re-run)
- **Re-audit** score < 85 (recommend fix cycle)

## Mode 3 — `/audit-tracker stale`

Filter dashboard to only show audits > 30 days old.

## Mode 4 — `/audit-tracker scores`

Compact one-liner per audit:
```
codeaudit: 92/A · secaudit: 88/A · uiuxaudit: 91/S · ...
```

## Mode 5 — `/audit-tracker latest`

Show the single most recent audit + its findings summary + verdict link.

## Implementation hints

To parse a verdict.json:
```bash
jq -r '.score, .grade, .timestamp' audits/.<audit-id>/verdict.json
```

If the audit has v2/v3/v4 variants (e.g., `.codeaudit-v3/`), prefer the
HIGHEST version (most recent re-audit cycle).

Detect project name from:
1. `package.json` "name" field
2. Else basename of cwd

Detect audit freshness:
- File mtime of `verdict.json` → compare to `now()`
- Days = int((now - mtime) / 86400)

## Anti-patterns

- ❌ Listing audits in random order (sort by mtime desc OR by score asc)
- ❌ Missing the "Recommended actions" footer
- ❌ Including audits that have no verdict.json (incomplete runs)
- ❌ Modifying audit outputs (read-only)
- ❌ Running an audit directly (delegate to `/audit-orchestrator`)

## Output format

Always end with **3 actionable recommendations** like:
```
📋 Next actions:
1. Re-run /perfaudit (last run 35d ago, scores drift)
2. Push /apiaudit from C → A via 2 fix cycles
3. Run /retentionaudit (never run, would unlock new feature ideas)
```

## Dynamic-Workflow Orchestration (v2)

> **The tracker is a forensic accountant, not a `cat`.** A dashboard built from a
> single trusting pass over `verdict.json` files inherits every lie those files
> tell — a stale clone's leftover `.codeaudit-v2/`, a half-written JSON from a
> killed run, an `mtime` that says "fresh" while the embedded `timestamp_end`
> says 40 days old. Runtime is the only truth (Law L1): the dashboard reports
> what is provably on disk *now*, adversarially verified, never what a file
> claims unchallenged. The Gestalt-Popper doctrine still binds — the **hinge**
> of this skill is *trust in the aggregated numbers*; a wrong score on the board
> is worse than a missing one, because it drives a wrong re-run recommendation.

This section governs HOW the tracker executes its Modes (above) WHEN RUN. It
changes nothing about the Modes, thresholds, or output formats — those stay
exactly as specified. It only makes the scan **parallel, adversarial, and
loop-until-dry** instead of a single linear `jq` sweep.

### 1. Fan-out — decompose the scan into independent parallel tracks

The tracker's work is embarrassingly parallel: each audit subdir is independent,
and the read-only Modes are independent lenses on the same corpus. Use the
**Workflow tool** to fan these out concurrently (NOT one-by-one):

- **Track A — Subdir discovery (per audit, parallel):** one concurrent unit per
  `audits/.<audit-id>/` directory found. Each unit parses its own
  `verdict.json` (`jq -r '.score, .grade, .timestamp_end // .timestamp, .skill_used, .version, .iterations, .needs_review'`),
  resolves the highest version when `-v2/-v3/` variants collide, and computes
  freshness from BOTH the file `mtime` AND the embedded `timestamp_end`. No
  subdir blocks another.
- **Track B — Synthesis ground truth (parallel):** read `audits/SYNTHESIS.md`
  "Past runs" and the project-name signal (`package.json` name → cwd basename)
  while Track A runs.
- **Track C — Freshness/staleness classification (parallel, fed by A):** apply
  the Status thresholds (Fresh ≤7d, Aging 8-30d, Stale >30d, Re-audit score<85)
  per entry as each Track A unit returns — never serialize the whole table behind
  the slowest subdir.

Mode selection still routes the *output* (dashboard / `stale` / `scores` /
`latest` / `init`), but the underlying scan is always the full parallel fan-out
so every Mode sees a fully-verified corpus. This is read-only on audit outputs;
disjoint readers never contend (R-SCOPE is satisfied trivially — only `init`
writes, and it writes only `.gitignore` + `SYNTHESIS.md`).

### 2. Adversarial verification — ≥2-of-3 lenses before an entry hits the board

Treat **every dashboard row as a finding** (audit X scored N, grade G, age D,
status S). A row is admitted to the table ONLY if it survives **≥2 of these 3
independent lenses** (R-VERIFY). Rows that fail are **killed** (dropped or
demoted to an `⚠️ unverified` note), never silently rendered as fact:

- **Lens 1 — REPRODUCE:** re-parse `verdict.json` a second time, independently;
  confirm `score` is a number 0-100, `grade ∈ {S,A,B,C,D,F}` and consistent with
  the score band (§13 of the preamble), and `skill_used` matches the subdir name.
  A score that doesn't re-parse, or a grade that contradicts its own score band,
  fails this lens.
- **Lens 2 — REFUTE:** actively try to prove the row is a lie. Is the JSON
  truncated / unparseable (killed mid-write → **incomplete run**, exclude per the
  existing anti-pattern)? Does file `mtime` disagree with embedded
  `timestamp_end` by a wide margin (a `git clone`/`touch` reset `mtime` → trust
  the embedded timestamp, flag the drift)? Is this a superseded version dir
  shadowed by a higher `-vN/`? Does `needs_review` / `iterations==5` mean the
  score is provisional (annotate, don't present as a clean grade)?
- **Lens 3 — CROSS-CHECK:** reconcile against independent sources —
  `SYNTHESIS.md` "Past runs" (does the board match the recorded history?),
  `telemetry.json` in the same subdir (`phases_completed` / `model` corroborate a
  real run vs a stub), and sibling artifacts (`verdict.md` / `before-after.md`
  exist → the run actually finished). A `verdict.json` with no corroborating
  sibling artifact is suspect.

**Decision:** ≥2 lenses agree → admit the row as authoritative. <2 → kill it
(exclude from scores/health math) and surface it in the recommendations footer as
`re-run /Xaudit (verdict unverified: <reason>)`. The Popper rule holds — an
unfalsifiable "looks fresh" is an opinion, not a board entry.

### 3. Synthesize — fold survivors back into the EXISTING dashboard (unchanged)

Surviving rows feed the **existing** outputs verbatim — same table columns
(Audit · Score · Grade · Age · Status), same `🎯 AUDIT DASHBOARD` header, same
Status thresholds, same `stale`/`scores`/`latest` Mode formats, same overall
`health = mean(verified scores)` line, same mandatory **3-actionable-recommendations**
footer. Killed/unverified rows do NOT enter the health average (a corrupt 0 or a
phantom 100 would poison it); they are listed separately as "needs verification".
Synthesis is the tracker's own job: never paste a single subdir's self-reported
grade as the verdict — the board is the adversarially-reconciled aggregate.

### 4. Loop-until-dry — the corpus is unknown-size

The number of audit subdirs is not known in advance and grows between runs. Drive
discovery as a **loop-until-dry** over `audits/`:

```
seen = ∅
repeat:
    found = glob audits/.*audit*/verdict.json   (exclude SYNTHESIS.md, .lock)
    new   = found − seen
    fan-out §1 + verify §2 on `new` only
    seen ∪= new
until new == ∅            # no fresh subdir discovered → corpus exhausted
```

This guarantees a subdir written by a concurrent audit (the parallel DYNAMIC
chain of §3/§89 in the preamble runs audits side-by-side) is still picked up,
without re-parsing already-verified rows. Bounded by the natural empty-delta exit
— no fixed phase count, because the tracker's input set is open-ended. There is no
fix-and-reaudit loop here (the tracker writes no fixes); "dry" means "no new
verdict to account for".

> **Invariant:** this orchestration is purely *how the scan runs*. The five Modes,
> the dashboard schema, the thresholds, and the 3-recommendation footer are
> untouched. The tracker stays read-only on audit outputs and remains the
> accountant — now one that double-counts the ledger before signing it.

## Sources

- Reads: `audits/SYNTHESIS.md`, `audits/.<audit-id>/verdict.json`
- Writes: `audits/SYNTHESIS.md` (updates), `.gitignore` (init mode)
- Related: `/audit-orchestrator` to actually RUN audits
- Public mirror: https://github.com/agentik-os/quality-arsenal

---
name: AUDIT-VERIFICATION-CONTRACT
description: >
  MANDATORY verification contract for all 23 Quality Arsenal audits (/codeaudit, /flowaudit, /logicaudit,
  /automationaudit, /debugaudit, /perfaudit, /secaudit, /a11yaudit, /seoaudit, /dataaudit, /apiaudit,
  /copyaudit, /dxaudit, /motionaudit, /uiuxaudit, /featureaudit, /depaudit, /i18naudit,
  /observabilityaudit, /privacyaudit, /refontaudit, /releaseaudit, /retentionaudit). Every audit MUST implement this
  before/after verification protocol. An audit that cannot prove "100% functional before AND after"
  is NOT complete. This is the "do no harm" contract.
  NOT a user-invokable skill — this is a shared source of truth referenced by all audit skills.
---

# Audit Verification Contract v1.2 — "Do No Harm"

> **An audit that breaks working functionality is a failure, regardless of score improvement.**
> Before calling any fix "done", prove the thing you touched still works — AND wasn't broken before.

> **v1.2 changelog (2026-06-07):** added invariant #9 (no performative phases —
> depth is evidence density, not phase count); refreshed the arsenal count 16 → 23
> (the authoritative list lives in `registry.toml`, `total_audits = 23`).
>
> **v1.1 changelog (2026-04-17):** formalized the HINGE {DOMAIN} pattern,
> documented mandatory minimums (phase count, score normalization, Phase N-1
> / N+4), clarified the 23 Quality Arsenal skills the contract applies to.

---

## MANDATORY MINIMUMS (v1.2)

Every Quality Arsenal skill MUST satisfy these structural invariants. A skill
violating any of them is not compliant and fails `/metaudit`.

| # | Invariant | Rationale |
|---|---|---|
| 1 | **At least 16 scored phases** (## headings counted as audit work, not doc) | Breadth floor — fewer phases = shallow coverage. NOTE: phase count is a floor for *breadth*, never a substitute for *depth* — see #9 |
| 2 | **Phase N-1 (PRE-FIX BASELINE)** implemented before first fix | Hippocratic rule — can't prove "no regression" without baseline |
| 3 | **Phase N+4 (before-after matrix)** produced to `audits/.<audit-id>/before-after.md` | Proof-of-work artifact required for 100/100 verdict |
| 4 | **Score normalized to /100** (raw may be /100, /320, /360, /420, /540 — must include normalization formula `raw / max * 100 = /100`) | Cross-skill comparison |
| 5 | **HINGE {DOMAIN}** identification before Phase 1 (10× scrutiny on the one thing that dominates the domain's risk/value) | Gestalt clarity gate — not all phases equal |
| 6 | **Popper falsification** in each scored item (how would you disprove this claim?) | Epistemic rigor — prevents confirmation bias |
| 7 | **Fix → re-audit loop** with explicit max iterations (typically 5) | Bounded recovery, prevents infinite loops |
| 8 | **Final verdict gate** blocks 100/100 claim unless `before-after.md` shows 0 regressions | Contract enforcement |
| 9 | **No performative phases** — every scored phase MUST cite project-specific evidence for its score (file:line + captured command output run against THIS codebase), whether the verdict is PASS or FAIL. A phase scored from generic reasoning, with no real command executed against the target, is *performative* and fails `/metaudit` | Depth is **evidence density**, not phase count — this is what kills box-checking and forces every phase to do genuine forensic work |

### Depth gate — evidence density (v1.2)

Invariant #9 makes padding self-defeating: adding phases without evidence does not raise the score, it **fails compliance**. Two enforced consequences:

- **Evidence-per-phase floor.** Every scored phase carries ≥1 distinct captured command (its falsification probe per #6) tied to a real path/line in the target. Empty "clean — looks fine" phases are rejected; a CLEAN verdict must show the exact command + output that *would have* surfaced the defect if present.
- **Density-calibrated confidence.** If the count of scored PASS phases exceeds the count of distinct real captured-command evidences, `confidence` is capped at `medium` regardless of score — an audit cannot assert more than it actually ran. Maximize depth by running *more real probes*, never by adding *more headings*.

### The HINGE {DOMAIN} Pattern (canonical)

Each forensic audit identifies ONE element that deserves 10× scrutiny — the
element whose quality dominates the entire domain. The term "hinge" means
"everything pivots on this". The noun adapts to the domain:

| Skill | Canonical hinge term | What it names |
|---|---|---|
| `/codeaudit` | **HINGE POINT** (module/function) | Single module where reliability pivots |
| `/secaudit` | **SECURITY HINGE POINT** | Auth/authorization boundary |
| `/uiuxaudit` | **HINGE COMPONENT** | Element that defines perceived quality |
| `/a11yaudit` | **HINGE FLOW** | Primary accessible journey |
| `/flowaudit` | **HINGE FLOW** | Journey that defines product's raison d'être |
| `/perfaudit` | **HINGE PAGE** | Highest-traffic page |
| `/motionaudit` | **HINGE PAGE** | Page defining kinetic identity |
| `/seoaudit` | **HINGE PAGES** | Money/conversion pages |
| `/copyaudit` | **HINGE PAGES** | Conversion-critical copy surfaces |
| `/dataaudit` | **HINGE TABLES** | Core entities (users, orders, products) |
| `/apiaudit` | **HINGE ENDPOINTS** | Money/auth/user-data endpoints |
| `/debugaudit` | **HINGE FEATURE** | Feature whose breakage = product down |
| `/featureaudit` | **HINGE CAPABILITY** | The one thing the product must do best |
| `/dxaudit` | **HINGE EXPERIENCE** | First 30 min of a new developer |
| `/automationaudit` | **HINGE AUTOMATION** | Script whose failure cascades most |
| `/logicaudit` | **HINGE LOGIC** | Bottleneck decision/algorithm/pipeline |

**Rule:** within a skill, use ONE canonical hinge term consistently. Do not
mix "hinge point" with a domain-specific term in the same skill. The only
skill using the generic "HINGE POINT" is `codeaudit` (because "code" has no
more specific noun) and `secaudit` (which qualifies it with "SECURITY").

When writing a new audit skill: pick the most specific noun for your domain,
document it in the Gestalt section (Phase 0), and never drift from it.

---

## THE HIPPOCRATIC RULE

**First, do no harm.** Every fix must pass 3 tests:

1. **BEFORE test** — Capture baseline functional state. Does the thing currently work?
2. **FIX** — Apply the change.
3. **AFTER test** — Repeat the exact same capture. Does it still work?
4. **DIFF** — Compare before vs after. Prove no regression.

Without all 3, the fix is UNVERIFIED. Do NOT mark it done.

---

## MANDATORY PHASES FOR EVERY AUDIT

Every audit (code/flow/logic/automation/debug/etc.) MUST add these phases AROUND its fix execution:

### Phase N-1: PRE-FIX BASELINE CAPTURE

Before touching ANY file, capture the current functional state of EVERYTHING the fix might affect.

```
For each file/resource about to be modified:
  1. Identify direct dependents: who calls this? who reads this? who executes this?
     Command: grep -rln "path/or/name" ~/.omega ~/.claude 2>/dev/null
             | grep -v ".backup" | grep -v "/file-history/" | grep -v ".jsonl"
  2. For each dependent, capture a "works check":
     - Script: bash -n SCRIPT  (syntax check)
     - Config: JSON parse / YAML parse
     - Python: python3 -c "import ast; ast.parse(open('F').read())"
     - Shell cron: validate expression
     - TypeScript: tsc --noEmit PROJECT (if applicable)
     - Skill: frontmatter present (head -3 | grep "^name:")
  3. Save baseline to audits/.<audit-id>/baseline/{file}.baseline
  4. If ANY baseline check already fails → abort the fix, flag it as "pre-existing broken"
     Do NOT fix unrelated broken things. Note and move on.
```

### Phase N: APPLY FIX (existing behavior)

Normal fix execution. Nothing changes here.

**Pattern #3 (grep-loop, v3.2+):** when an audit's fix triggers a re-audit
loop (Step 8b in most skills), wrap it with `grep-loop.sh` to bound
iterations and prevent spinning on subjective LLM judgment:

```bash
# Use the audit's deterministic re-verify command — NOT an LLM call.
# grep-loop.sh is vendored into _shared/ (install-parity) so a blank-VPS clone has it.
"${OMEGA_DIR:-$HOME/.omega}/skills/audits/_shared/grep-loop.sh" "$WORKER_SESSION" \
    "bash -n \$CHANGED_FILE && bash ~/.omega/lib/safe-npm-build.sh" \
    --max=3 --max-diff=500 --sleep=30
```

Exit codes the audit MUST handle:
- `0` verify converged — proceed to Phase N+1
- `2` max_iter — record `halt_reason: max_iter` in verdict.json, do NOT loop
- `3` scope_creep — record `halt_reason: scope_creep`, REVERT diff > max
- `4` worker_died — escalate to oracle inbox

Forbidden as verify_cmd: `claude -p "looks good?"`, any LLM-as-judge.
Required: shell command with deterministic exit code.

### Phase N+1: POST-FIX VERIFICATION

Immediately after each fix, repeat EVERY baseline check from Phase N-1.

```
For each dependent captured at baseline:
  1. Re-run the exact same "works check"
  2. If check PASSES → record as OK
  3. If check FAILS → this fix broke something that worked. REVERT immediately:
     - For sed/Edit: restore from .bak
     - For mv: mv back to original location
     - For crontab: crontab /tmp/crontab.backup-*.txt
  4. Mark fix as NEEDS_REVIEW in fix-plan.json
  5. Continue to next fix (do NOT abort whole audit)
```

### Phase N+2: FUNCTIONAL SMOKE TEST

Beyond syntax/parsing, verify the thing actually FUNCTIONS:

```
For every skill whose path changed:
  - Read the skill file from its new location — does it exist?
  - grep for all internal references (@-mentions, paths, links) — do they resolve?
  - Check frontmatter is intact

For every script whose behavior changed:
  - Run with --help or --dry-run if supported
  - Check it doesn't hang (timeout 10s)
  - Check exit code

For every rule whose content changed:
  - Ensure it's still loadable (markdown syntax valid)
  - Ensure no internal cross-references were broken

For every cron that was modified:
  - Validate cron expression with `cron-validator` or equivalent
  - Confirm `crontab -l` shows expected output
```

### Phase N+3: BREAKAGE DETECTION REPORT

After all fixes applied, run a full breakage scan:

```bash
# Find ALL stale references to moved/deleted paths
echo "=== STALE REFERENCES CHECK ==="
for old_path in "${MOVED_OR_DELETED[@]}"; do
  BROKEN=$(grep -rln "$old_path" ~/.omega ~/.claude 2>/dev/null \
           | grep -v ".backup" | grep -v "/file-history/" | grep -v ".jsonl" \
           | grep -v "audits/.<audit-id>/" | head -20)
  if [ -n "$BROKEN" ]; then
    echo "🔴 STALE REFERENCES to $old_path:"
    echo "$BROKEN"
    # Fix each one OR mark NEEDS_REVIEW
  fi
done

# Exit 0 ONLY if zero stale references found
```

### Phase N+4: BEFORE/AFTER MATRIX (mandatory output)

Every audit MUST produce `audits/.<audit-id>/before-after.md`:

```markdown
# Before / After Verification Matrix

## Functional checks

| Item | Before | After | Status |
|------|--------|-------|--------|
| /linear skill loads | ✅ PASS | ✅ PASS | NO REGRESSION |
| rule 43 referenced correctly | 🔴 needed fix | ✅ PASS | FIXED |
| crontab validates | ✅ PASS | ✅ PASS | NO REGRESSION |
| dispatch-to-session syntax | ✅ PASS | ✅ PASS | NO REGRESSION |
| ... | ... | ... | ... |

## Performance metrics

| Metric | Before | After | Delta |
|--------|--------|-------|-------|
| Context tokens/session | 20,000 | 12,700 | -36% ✅ |
| Rules loaded | 16 files | 12 files | -25% ✅ |
| Dispatch wait time | 7s | 3-5s | -40% ✅ |
| Minute-0 crons | 14 | 1 | -93% ✅ |

## Regressions detected

<list, or "NONE" if clean>

## Status: {100/100 VERIFIED | NEEDS_REVIEW | FAILED}
```

If Status is NOT "100/100 VERIFIED" → the audit is NOT done. Do more work.

---

## THE "IT STILL WORKS" CHECKLIST

Before marking ANY audit complete, tick every box:

- [ ] Every file I moved has its new path referenced by every caller
- [ ] Every file I deleted has no live references (only ephemeral logs OK)
- [ ] Every script I modified passes `bash -n` (shell) or `python3 -c "import ast"` (Python)
- [ ] Every cron I changed still passes cron-expression validation
- [ ] Every skill file I changed still has valid frontmatter
- [ ] Every rule I moved is accessible at its new path (file exists + readable)
- [ ] I ran a FRESH grep for old paths across the full ecosystem — got 0 matches
- [ ] For infrastructure changes (crons, live scripts): backups exist with timestamps
- [ ] For each fix: before-check PASSED, fix applied, after-check PASSED
- [ ] `before-after.md` produced with measurable deltas
- [ ] Zero regressions claimed only if zero regressions actually found

---

## WHAT COUNTS AS "BROKEN" (revert immediately)

- A skill file's path is referenced but target doesn't exist
- A cron expression is invalid (cron validator fails)
- A script has syntax errors (`bash -n` fails)
- A Python module has import/parse errors
- A rule references another rule that was deleted
- A hook script points to a path that no longer exists AND the script is actively triggered

---

## WHAT DOESN'T COUNT AS BROKEN (OK to leave alone)

- Reference in ephemeral log files (`tool-results/`, `file-history/`, `.jsonl` session files)
- Reference in `.backup-*` files you created yourself
- Reference in deprecated/archived scripts that aren't triggered
- Reference in `shell-snapshots/` (autogenerated, rotates)

Skip these by default in breakage scans.

---

## ENFORCEMENT

Every audit skill's `## PHASE N+4: VERDICT & SCORING` section MUST end with:

```
## FINAL VERIFICATION GATE (MANDATORY before final verdict)

1. Read AUDIT-VERIFICATION-CONTRACT.md fully
2. Execute every check in the "IT STILL WORKS CHECKLIST"
3. Produce audits/.<audit-id>/before-after.md
4. Grep for stale references — must be 0
5. ONLY THEN write final verdict

If ANY check fails → mark status as NEEDS_REVIEW and do NOT claim "done".
```

---

*"An audit that breaks a single working thing is worse than no audit. Measure twice, fix once, verify thrice."*

# Audit Meta-Protocol v2 — Universal Quality Gate

> **Single source of truth applied to ALL 23 forensic audits in the Quality Arsenal.**
> Created 2026-05-08 — model upgrade Opus 4.6 → 4.7 + max effort.
> Replaces the implicit "score ∈ [0,100]" output with a structured, falsifiable, intent-aware result.

---

## Why this exists

Each audit (codeaudit, debugaudit, secaudit, …) used to:
- Score a number /100 against generic standards
- Sometimes use shortcut phrases (`looks correct`, `should be fine`, `no obvious issues`)
- Treat a passing build as confirmation of correctness
- Operate context-blind to what the user actually asked for

Result: scores reached 100/100 while real bugs slipped through, because the score measured "did checklist pass?" not "is the user's problem solved with high confidence?".

**v2 fixes this in five ways:**

1. **Intent-aware** — every audit ingests the user-need quote and verifies the change addresses it.
2. **Falsifiable** — every PASS must cite ≥3 concrete runtime tests that **could have failed but didn't**.
3. **Confidence-calibrated** — output `high | medium | low` — measures how much was actually verified vs assumed.
4. **Hinge-focused** — 10x scrutiny on the load-bearing 10% of the change (computed by `${OMEGA_DIR:-$HOME/.omega}/skills/audits/_shared/hinge-analyzer.sh`).
5. **Anti-shortcut** — banned phrases trigger automatic FAIL.

---

## Required CLI inputs (oracle injects all of these on dispatch)

| Flag | Required | Purpose |
|---|---|---|
| `--ticket=<ID>` | yes | Unique identifier for output naming |
| `--files=<paths>` | yes | Space-separated list of modified files (from `git diff --name-only`) |
| `--user-need=<quote>` | **yes (NEW)** | Verbatim user quote describing the desired outcome |
| `--hinge=<file:line-range>` | **yes (NEW)** | Load-bearing region per `hinge-analyzer.sh` |
| `--url=<url>` | when relevant | Page URL for runtime verification |
| `--selector=<css>` | when relevant | Element selector for UI audits |

If `--user-need` or `--hinge` is missing, the audit MUST refuse to run and write:

```json
{
  "score": 0,
  "confidence": "low",
  "skill_used": "<name>",
  "error": "missing required v2 inputs (--user-need, --hinge)",
  "request_redispatch": true
}
```

The oracle, on seeing `request_redispatch: true`, must re-dispatch with the missing args.

---

## Required JSON output schema v2

```json
{
  "score": 100,
  "confidence": "high | medium | low",
  "skill_used": "<exact-skill-name>",
  "ticket": "<TICKET_ID>",
  "user_need_match": {
    "quote": "<verbatim user quote>",
    "addressed": true,
    "evidence": "<one paragraph: how the change addresses the quote, with file:line references>",
    "edge_cases_covered": ["<list of edge cases mentioned by user that were tested>"]
  },
  "falsifiable_tests": [
    {
      "name": "<short name, e.g. 'tsc --noEmit on changed files'>",
      "hypothesis": "<what would FAIL if the fix is wrong>",
      "command": "<exact bash/curl/playwright command run>",
      "expected": "<expected output>",
      "actual": "<actual output>",
      "passed": true
    }
  ],
  "hinge_findings": [
    {
      "location": "<file:line>",
      "concern": "<what could go wrong here>",
      "verified_safe_by": "<test name from falsifiable_tests OR explicit reasoning>"
    }
  ],
  "issues_found_and_fixed": [
    {
      "severity": "critical | high | medium | low",
      "location": "<file:line>",
      "issue": "<short description>",
      "fix_applied": "<what was changed>"
    }
  ],
  "confidence_basis": "<one paragraph explaining WHY confidence is the value chosen — what was directly verified vs assumed>",
  "finished_at": "<ISO8601>"
}
```

**Schema enforcement:**
- `score` must be integer 100 (not "100" string, not 99). Anything < 100 = fail-and-fix-loop.
- `confidence` must be `high` for PASS. `medium` triggers re-audit by a fresh worker. `low` = automatic FAIL.
- `falsifiable_tests` must have ≥3 entries with concrete commands + actual outputs. No fabricated outputs (gate spot-checks 1 test per audit).
- `user_need_match.addressed` must be `true` AND `evidence` must reference files actually changed (cross-checked against `git diff`).
- `hinge_findings` must address every region returned by `hinge-analyzer.sh`. If the hinge is empty (rare — pure docs change), record `[]` with explicit reason.

---

## The Popper Discipline

Every claim of "PASS" must be **falsifiable**. That means:

> Before claiming the fix works, state the test that would prove it broken. Run that test. Cite the actual output.

Examples per audit type:

| Audit | Falsifiable test (example) |
|---|---|
| codeaudit | `tsc --noEmit src/auth/validate.ts` exits 0 AND no any/unknown leaks (`grep -E ': any\|: unknown' src/auth/validate.ts \|\| true`) |
| debugaudit | Playwright trace at `--url`: 0 console.error AFTER vs N BEFORE |
| secaudit | Try the exploit (`curl -X POST ... <payload>`) — must fail with 401/403, not 200 |
| dataaudit | `convex run "schemaTest"` returns 0 orphans / FK violations |
| flowaudit | Playwright completes the full user journey end-to-end with no dead ends |
| perfaudit | Lighthouse score on `--url` ≥ baseline AND LCP < 2.5s on cold load |
| apiaudit | `curl <endpoint>` with malformed input returns 4xx with clear error, valid input returns 200 |
| a11yaudit | `axe-core` run on `--url` returns 0 critical violations |
| copyaudit | Banned phrase regex returns 0 matches in changed files |

**Forbidden patterns** (gate checks for these in the audit's reasoning):
- *"looks correct"* / *"appears to work"* / *"should be fine"* / *"no obvious issues"*
- *"based on my reading of the code"* without an actual command run
- *"the build passes"* used as standalone evidence for a non-syntax issue
- *"no warnings"* used as evidence of correctness (warnings ≠ semantics)
- *"I cannot run X but ..."* — if you can't run it, score < 100

---

## Confidence calibration rules

```
confidence = "high"
  ⟺ ALL of:
    - ≥3 falsifiable_tests passed with cited actual outputs
    - user_need_match.evidence cites at least one file:line that is in `git diff`
    - hinge_findings covers every region from hinge-analyzer.sh
    - no banned shortcut phrases anywhere in the audit's reasoning
    - issues_found_and_fixed is exhaustive (no "see also" hand-waves)

confidence = "medium"
  ⟺ at most ONE of the above is missing AND no critical assumption is unverified

confidence = "low"
  ⟺ anything else, OR audit was not able to actually run a test
```

A score of 100/100 with confidence `medium` or `low` triggers an **automatic re-audit by a fresh worker** before being considered passing. Two consecutive `high` scores from independent workers = real PASS.

---

## Anti-Shortcut Clause (Opus 4.7 + max effort discipline)

This system runs on Opus 4.7 with max effort. There is no time pressure (rule 46-no-time-panic). The model has the reasoning capacity to:
- Run every test it claims to have run
- Cite actual command outputs verbatim
- Identify subtle issues that a 4.6 pass would miss
- Reason through cross-file consequences

If the audit is tempted to skip a test because "it's clear from the code", that temptation is exactly the bug we're fixing. **Run the test. Cite the output. No exceptions.**

A pass without falsifiable evidence is a lie, not a pass.

---

## Adversarial Dual-Pass (challenger run)

After all selected audits return 100/100 with confidence `high`, the oracle dispatches ONE additional **challenger audit** by a fresh worker:

1. Mission: *"You are the challenger. The other audits passed. Find the issue they missed. Treat their PASS as suspicious."*
2. Challenger has access to all prior audit outputs and the user-need quote.
3. Challenger runs codeaudit + debugaudit + the most domain-specific audit for the change (e.g. secaudit if auth, dataaudit if DB).
4. If challenger finds ANY issue not in `issues_found_and_fixed`, the original PASS is **voided** and the worker must fix + re-audit.
5. Two consecutive challenger PASSes (independent workers) = mission complete.

This catches confirmation bias — a single audit may rationalize a passing score. A challenger with adversarial intent breaks that rationalization.

---

## Cross-references

- `~/.omega/lib/audit-selector.py` — picks the audit set per mission
- `${OMEGA_DIR:-$HOME/.omega}/skills/audits/_shared/hinge-analyzer.sh` — picks the load-bearing 10% of the change (vendored; install-parity)
- the Linear ticket gate (host-specific, only when a Linear integration is configured: `$OMEGA_DIR/lib/linear-ticket-gate.sh` if present) — enforces v2 schema at the gate
- `~/.omega/prompts/worker.md` — INTENT-DRIVEN AUDIT INVOCATION section
- `~/.omega/lib/oracle-prompt.sh` — R-6 dynamic dispatch
- `~/.claude/commands/linear.md` — Step 8 dynamic audit set + Step 8c intent verification

---

## Migration from v1

| Old (v1) | New (v2) |
|---|---|
| `{"score": 100, "skill_used": "codeaudit"}` | full v2 schema above |
| Hardcoded quintuple | `audits-selected.json` from selector |
| Pass = checklist met | Pass = falsifiable evidence + high confidence + adversarial pass |
| Implicit user-need awareness | Explicit `--user-need` + `user_need_match` object |
| Code-level scrutiny only | Hinge-focused 10x on load-bearing region |

The gate accepts v1 with a warning until 2026-06-01, then v2-only.

## Runner finalization

After writing `verdict.json`, rerun the same audit runner invocation with
`--finalize --threshold=100`. A missing or malformed V2 verdict exits 2. A
valid verdict below 100 exits 1. Only exit 0 authorizes the audit to report a
pass.

---

*"100% trust requires 100% falsifiable evidence. Anything less is a comfortable lie."*


## THE THREE LAWS (overrides all other instructions)

> **LAW 1 — Code lies. Comments lie. Only runtime tells the truth.** Observe actual runtime (logs, traces, outputs) before concluding. Before the 3rd code change on the same bug: live runtime evidence MANDATORY.
>
> **LAW 2 — Researcher, not sycophant.** Challenge flawed premises. Think before coding. Iterate with evidence. Root causes over symptoms. Push back with reasoning. Flag own mistakes. No fake confidence. No agree-and-code. Senior engineer standard.
>
> **LAW 3 — Autonomous execution.** When dispatched, never wait. Decide → execute → report. Never ask "which path?". The only legal stop is `.done.json` with status=done_clean, pending, or failed.

---
# LMC Protocol — Optional Validation

> LMC (Lead-Manager-Checker) is an OPTIONAL quality gate, NOT mandatory for all agents.
> Most agents work DIRECTLY without Manager/Checker overhead.

---

## When to Use LMC

| Agent | LMC? | Why |
|-------|------|-----|
| SERAPH | YES — Full LMC | Code audit quality requires independent validation |
| KEYMAKER | YES — Lite (Lead+Manager, Lead validates) | Plans benefit from structured generation |
| All others | NO — Direct execution | Speed > ceremony for routing, research, execution |

---

## LMC Flow (When Used)

1. **Lead** receives task
2. **Lead** spawns **Manager** (general-purpose) with domain prompt
3. Manager returns: BRIEF, STATUS, CONFIDENCE, ARTIFACTS
4. **Full LMC only:** Lead spawns **Checker** with validation criteria from `checkers/checker-{name}.md`
5. Checker returns: DECISION (PASS/FAIL), CONFIDENCE, ISSUES, FEEDBACK
6. PASS → return result | FAIL (≤3 attempts) → re-run Manager with feedback | FAIL (>3) → escalate

---

## Manager Output Format

```
BRIEF: [1-line summary]
STATUS: DONE | PARTIAL | BLOCKED
CONFIDENCE: [0.0-1.0]
ARTIFACTS: [files created/modified]
```

## Checker Verdict Format (SERAPH only)

```
DECISION: PASS | FAIL
CONFIDENCE: [0.0-1.0]
ISSUES: [problems found]
FEEDBACK: [improvement guidance]
```

---

*Simplified 2026-03-03 — Speed over ceremony*

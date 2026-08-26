---
name: linear
description: >
  OmegaOS Linear feedback-resolution pipeline (v2 — Workflow-driven). Resolves Linear feedback
  tickets end-to-end: deep ticket analysis, BEFORE/AFTER browser evidence, surgical fix, strict
  Fix-Verification comment, the /omg-audit quality gate (100/100), then a neutral In-Review
  handoff for the operator — the agent NEVER self-marks Done. Auto-detects four modes
  (FIX / VERIFY-DONE / RE-REVIEW / REGRESSION) and a trigger-guard keeps Linear silent unless
  the operator named it. The v2 engine fans tickets out in parallel through the OmegaOS Workflow
  primitive (per-ticket triage → fix → /omg-audit → adversarial verify), then synthesizes — not
  a sequential one-ticket-at-a-time loop. Single source of truth: the shipped
  ${OMEGA_DIR}/skills/linear/RULES.md. Use when user says "/omg-linear", "/linear", "fix linear",
  "resolve feedback", "regler les feedbacks", "verify done linear", "revérifie linear".
allowed-tools: ["Read", "Write", "Edit", "Bash", "Glob", "Grep", "Skill"]
domain: orchestration
read_only: false
triggers: ["omg-linear", "linear", "fix linear", "resolve feedback", "regler les feedbacks", "verify done linear", "revérifie linear"]
argument-hint: "[--mode=FIX|VERIFY-DONE] [--ticket=<ID>] [--project=<path>]   (all optional → auto-detect)"
---

# /omg-linear — Linear Feedback Resolution Pipeline (OmegaOS v2, Workflow-driven)

You resolve Linear feedback tickets to a **100/100, evidence-backed** standard, then hand each
ticket to a neutral **In-Review** state for the human operator to verify. This file is a
**launcher**. The full, non-negotiable protocol — every step, the strict comment template, the
banned-phrase list, the BEFORE/AFTER capture rules, the four-mode resolution, the trigger-guard,
intent verification, and the gate — lives in the shipped **RULES.md**.

## MANDATORY FIRST STEP — READ THE SHIPPED RULES.md

Before doing ANYTHING else, resolve `OMEGA_DIR` and Read the canonical protocol. This ships with
the repo, so a fresh `git clone && ./install.sh` always has it — no maintainer-private paths.

```bash
# Resolve OMEGA_DIR: $OMEGA_DIR env > config.toml `omega_dir` key > default ~/.omega
OMEGA_DIR="${OMEGA_DIR:-$(awk -F'=' '/^[[:space:]]*omega_dir[[:space:]]*=/{gsub(/[" ]/,"",$2);print $2}' "$HOME/.omega/config.toml" 2>/dev/null)}"
OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
echo "RULES: $OMEGA_DIR/skills/linear/RULES.md"
```

Then:

```
Read ${OMEGA_DIR}/skills/linear/RULES.md
```

That file is the **single source of truth**. If anything in this launcher contradicts RULES.md,
**RULES.md wins**. Never execute `/omg-linear` without having Read it in the current session.

> **No maintainer-private dependencies.** This skill does NOT read `~/.claude/...`, and does NOT
> depend on any private `audit-selector.py` or `linear-ticket-gate.sh`. Audits run through the
> shipped OmegaOS Quality Arsenal (`/omg-audit`); the gate is enforced inside the Workflow (below).

## TRIGGER GUARD (read RULES.md §Trigger-Guard for the full list)

This pipeline activates **only** when the operator explicitly signals Linear — the keyword
`linear` / `/omg-linear` / `/linear`, a `fix linear` / `regler les feedbacks` phrase, a Linear
ticket ID (`KOM-42`), or a `linear.app/...` URL. **Bare "feedback" or bare "ticket" never
triggers it.** Never mention Linear in a reply unless the operator mentioned it first. When in
doubt, ask once and default to non-Linear.

## INTENT VERIFICATION FIRST (don't auto-pilot into ticket-fetching)

Read the operator's actual prompt before acting. "Look at this Linear project and build what's
described" means treat the project as a spec/PRD — NOT "fetch every open ticket and fix it".
Match intent to action; if unsure, ask. Full intent matrix in RULES.md.

## THE FOUR MODES (kept from the proven design)

| Mode | Trigger | Fetches | Runs |
|------|---------|---------|------|
| **FIX** (default) | `/omg-linear`, `fix linear`, `regler les feedbacks`, or no keyword | OPEN tickets | Full protocol (analysis → fix → /omg-audit → In Review) |
| **VERIFY-DONE** | `verify done linear`, `revérifie linear`, `audit done linear` | DONE tickets | Audit-only re-certification; re-open any ticket that fails the gate |
| **RE-REVIEW** (auto, per-ticket) | non-Done ticket whose **last comment is from the operator** | that one ticket | Full protocol with the operator's latest comment as the NEW authoritative spec |
| **REGRESSION** (auto, per-ticket) | non-Done ticket with ≥1 prior bot `Fix Verification Report` AND a prior Done→reopen in its history | that one ticket | Full protocol + deep live root-cause, banned same-files repeat, mandatory "Why prior attempts failed" section |

Mode priority when several match for one ticket: **REGRESSION > RE-REVIEW > VERIFY-DONE > FIX**.
Echo the detected mode before starting each ticket. Never silently fall back between modes. If
both FIX and VERIFY keyword sets appear in the prompt, ask the operator which mode.

## THE v2 ENGINE — OmegaOS Workflow (parallel fan-out, not a sequential loop)

The old pipeline processed tickets strictly one at a time. The **v2 engine uses the OmegaOS
Workflow primitive** to fan tickets out in parallel, then synthesize — the same fan-out →
adversarially-verify → synthesize pattern the rest of OmegaOS uses. Per ticket, the Workflow runs
these stages as parallel branches across the eligible ticket set:

1. **Triage** — fetch full context (description + EVERY comment chronologically + every screenshot
   + state history), resolve the per-ticket mode, capture the BEFORE state (auth → navigate →
   screenshot + console) **before any code**.
2. **Fix** — surgical edit of only the root-cause files; build must pass; conventional commit
   `fix: [TICKET-ID] <summary>`; capture the AFTER state (same URL, same viewport, same auth);
   multi-step flows capture one screenshot per stage.
3. **Audit (/omg-audit gate)** — see next section. Every selected audit must hit 100/100.
4. **Adversarial verify** — a challenger branch tries to falsify the "resolved" claim (≥5 attack
   attempts) plus the 5-question Intent Verification (does the AFTER state actually solve the
   operator's stated need, live on prod?). Verdict must be `confirmed` + all intent Qs YES.
5. **Synthesize** — you (the orchestrator) own the verdict. Never paste a branch's summary as the
   result; reconcile the branches yourself, then either advance the ticket to In Review or loop it
   back to Fix with the findings inlined.

**File-disjointness is the parallelism invariant** (R-SCOPE — one writer per file): the Workflow
only runs tickets concurrently when their `files_to_touch` sets are disjoint; overlapping tickets
serialize, and parallel mutation uses isolated working copies (git worktrees). Two workers on the
same file = corruption. RULES.md §v2-Workflow has the full stage contract, the per-ticket artifact
layout, and the disjointness packer.

## THE /omg-audit GATE (replaces the old dynamic audit-selector chain)

After a ticket's fix lands and its comment passes the completeness check, the Workflow runs the
**OmegaOS Quality Arsenal** scoped strictly to that ticket (modified files + ticket URL only — no
project-wide scans, no "while we're here" expansions):

```bash
# Auto-select the relevant audits for THIS ticket's mission + changed files:
omega audit select "<ticket summary>"        # → the relevant subset of the 23 audits

# Then run each selected audit, scoped to the ticket, as /omg-<name>audit
#   e.g. /omg-codeaudit, /omg-debugaudit, /omg-logicaudit, /omg-secaudit, …
# (or omega audit run <name>audit --dir <project>); each writes audits/.<name>audit/verdict.json
```

- The selection is **mission-aware** — `omega audit select` picks the relevant audits from the 23
  in the Quality Arsenal (`skills/audits/`); `codeaudit`, `logicaudit`, and `debugaudit` are the
  always-on baseline. Run each selected audit as the **real** `/omg-<name>audit` skill — never
  paraphrase a forensic protocol into prose (R-AUDIT).
- **Gate threshold for Linear = 100/100 on every selected audit**, strict, no partial credit (this
  overrides the default `pass_threshold` in `~/.omega/config.toml`). 99 is a FAIL.
- Any audit < 100 → fix every finding, re-run only the failing audits, loop (max 5 iterations). If
  still failing after 5 → escalate, do NOT advance to In Review.
- A 100/100 audit set still fails the gate if Intent Verification (the 5-question user-need check)
  isn't all-YES. Code-correct but user-need-mismatched is the failure mode this catches.

The banned-phrase anti-cheat is in force (FORBIDDEN: `streamlined`, `skip audit`, `quick version`,
`to save time`, "retroactive verification confirms", etc.) — L5 quality-over-speed. A 403/401 on a
page is an ABORT, never a PASS.

## NEUTRAL IN-REVIEW HANDOFF (the agent never self-marks Done)

When the gate passes, the ticket moves to a **neutral review state for the human operator to
check** — discover it via the Linear `workflowStates` query (NEVER hardcode a stateId):

1. Prefer the operator's standard **`In Review`** state (type `started`), then
2. fallback to **`Omega Review`** — OmegaOS's neutral review state, auto-created on demand if the team has none, then
3. if neither exists and `Omega Review` cannot be created → ABORT and tell the operator: "no review state found, ticket left in current state."

(This ordering matches RULES.md §7 — the operator's existing `In Review` wins; `Omega Review` is the auto-created neutral fallback.)

```graphql
mutation { issueUpdate(id: "UUID", input: { stateId: "REVIEW_STATE_UUID" }) { success } }
```

**A human marks Done — the agent NEVER self-marks Done.** Moving a ticket to `Done` / `Completed`
/ `Closed`, or hardcoding any Done stateId, is an automatic failure. The agent's job ends at the
review state; you (the operator) do the final manual verification and mark Done yourself.

## WHAT THIS SKILL KEEPS FROM THE PROVEN DESIGN

- The four-mode model (FIX / VERIFY-DONE / RE-REVIEW / REGRESSION) with per-ticket auto-detection.
- The **strict Fix-Verification comment template** (≥800 chars; verbatim user quote; Before/After
  state; Before-vs-After table; Console-log comparison; clickable Verify URL; self-verification
  checklist). No free-form, no batch-posting, no paraphrased commit messages.
- **BEFORE/AFTER evidence captures** via the browser, with multi-step flows captured stage-by-stage.
- The **100/100 quality gate** + 5-question Intent Verification + adversarial confirmation.
- The **trigger-guard** (never mention Linear unless the operator did) and intent-first reading.

## USAGE

```
/omg-linear                              # auto-detect project + mode, fix OPEN tickets via the v2 Workflow
/omg-linear verify done linear           # VERIFY-DONE: re-certify DONE tickets, reopen failures
/omg-linear --ticket=KOM-42              # one specific ticket
/omg-linear --project=/path/to/project   # explicit project path
```

Auto-detects the project, fetches tickets for the resolved mode, runs the **v2 Workflow**
(parallel triage → fix → `/omg-audit` 100/100 → adversarial verify → synthesize), moves every
passing ticket to the neutral **In Review** / **Omega Review** state for the operator, and reports
final status (X advanced to review, Y reopened, Z escalated, with Linear links).

Full protocol, templates, and gate contract: **`${OMEGA_DIR}/skills/linear/RULES.md`** (default
`~/.omega/skills/linear/RULES.md`).

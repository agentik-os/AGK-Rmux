---
name: changelog-adopt
description: "Claude Changelog Adopt — a daily autonomous self-improvement pass that reads the OFFICIAL Claude Code changelog (the authoritative GitHub CHANGELOG for anthropics/claude-code, with @claudecodelog as a secondary mirror), diffs it against the last version OmegaOS already absorbed, and for each NEW entry decides whether it implies a concrete OmegaOS or system-agent improvement — a new primitive, a doctrine change, an agent-behavior shift, a deprecation to unwind. It classifies each entry (opus), proposes a surgical adoption (what to change, in which file, why — like the manual /loop -> R-LOOP native-loop doctrine move), runs an adversarial anti-fabrication gate (opus, default-reject), writes a self-contained HTML report + Telegram alert for the operator to decide, and — only when ARMED — dispatches each vetted proposal to an oracle behind the full quality gate (In-Review handoff, never auto-Done, never force-push). Auto-adoption is scoped to doctrine (rules.rs), agent identities (agents/*.md), skills, and install.sh; core-Rust changes are flagged as needs-human, never auto-patched. Use when the user says '/changelog-adopt', '/omg-changelog-adopt', 'adopte les changelogs', 'auto-amelioration OmegaOS', 'veille changelog Claude', 'watch the Claude Code changelog', 'run the changelog adopt', or wants to inspect/arm/disarm it. NOT the broad X ecosystem radar (that is /ecosystem-watch, which also feeds the @Agentik_os marketing feed) and NOT a one-shot research primitive (deep-research): this is the standing, authoritative changelog->self-upgrade loop. Integration is proposed by default; the operator arms it to let it act."
---

# Claude Changelog Adopt

The self-improvement loop that keeps OmegaOS current with Claude Code itself. Every day it reads
the one authoritative list of what Claude Code shipped, works out which entries should change how
OmegaOS or its agents behave, and hands you a decision report — the same move we made by hand when
`/loop` shipped and became R-LOOP's native-loop doctrine, now standing and automatic.

It is a **loop** in the R-LOOP sense: a verifiable goal (absorb every new changelog entry), a
memory (the last-absorbed version + a per-entry decision store), and a hard ceiling (it proposes;
the operator, or an armed + gated oracle dispatch, decides — it never silently rewrites the OS).

Complementary to `/ecosystem-watch`: that is the broad, noisy X radar that also feeds the public
@Agentik_os feed. This is the narrow, authoritative source — the official changelog — pointed
straight at OmegaOS self-upgrade, with no marketing side-output.

---

## Pipeline

```
cron (daily) → [1] FETCH  official CHANGELOG (GitHub raw: anthropics/claude-code)
             → [2] DIFF   vs state/last_version → NEW entries only (first run seeds forward)
             → [3] CLASSIFY (claude, opus)  per entry: relevance · category · target surface ·
                                            in_scope · concrete proposal (what/where/why) · 0-10
             → [4] GATE   (claude, opus)  adversarial: is the feature real per the entry text? is
                                          the patch correct, minimal, in-scope, non-duplicative?
                                          default-REJECT on doubt (anti-fabrication, R-VERIFY)
             → [5] REPORT self-contained HTML in ~/.omega/artifacts/  (R-HTML)
             → [6] ALERT  Telegram (Alerts topic) via omega-alert-send.sh
             → [7] ADOPT  (only if ARMED)  dispatch each vetted proposal to an OmegaOS oracle,
                                           bounded (R-LOOP), In-Review handoff — never auto-Done
```

Deterministic control (fetch, diff, version watermark, dedup, dispatch side-effects) lives in
shell + small python. Judgment (classify, propose, refute) lives in the model. The gate is an
INDEPENDENT adversarial pass whose only job is to reject — default-reject on doubt.

## Auto-adoption scope (operator decision, non-negotiable in the runner)

The engine may only PROPOSE auto-patches to these surfaces:

- **Doctrine** — `crates/omega-core/src/rules.rs` (the Laws/Rules SSOT) + its markdown mirror.
- **Agent identities** — `agents/*.md` (oracle, worker, Atlas, Nova/companion, the AISB matrix).
- **Skills** — `skills/<name>/` (new or amended skills, e.g. adopting a new Claude Code primitive).
- **Installer** — `install.sh` parity for any of the above (L0).

A changelog entry whose only sensible adoption touches the **core Rust orchestration**
(`crates/` beyond rules.rs — executor, loop_guard, dispatch, TUI) is classified `surface:
core-rust`, `in_scope: false`, and surfaced as **needs-human** in the report. The engine never
proposes an auto-patch to the compiled core; that stays an explicit operator call.

## Adoption safety rails (non-negotiable)

Acting on OmegaOS itself (dispatching an adoption) requires ALL of:

- `armed` flag present (`~/.omega/state/changelog-adopt/armed`) — the kill-switch. Absent =
  proposals are report-only. `rm` the flag to stop all self-modification instantly.
- the adversarial gate returned `keep=true` for that proposal.
- `in_scope: true` (surface is doctrine/agents/skills/install.sh — never core-rust).
- the entry fingerprint is not already in the seen-store (never adopt the same entry twice).
- the per-run adoption cap (`CA_DAILY_ADOPT_CAP`, default 3) is not exhausted.

Even armed, an adoption is DISPATCHED to an oracle that runs the full quality gate and hands back
In-Review — the operator still approves the merge. The engine proposes and prepares; it never
self-marks Done and never force-pushes (L0, R-VERIFY, R-LOOP).

## Run

```
bash scripts/adopt-run.sh                 # full run (fetch → classify → gate → report → alert)
CA_DRYRUN=1 bash scripts/adopt-run.sh     # never dispatches even if armed (report only)
CA_FORCE_VERSION=2.1.100 bash scripts/adopt-run.sh   # re-process everything newer than X (backfill)
```

Env knobs: `CA_CHANGELOG_URL` (GitHub raw default), `CA_CLASSIFY_MODEL` (claude-opus-5),
`CA_GATE_MODEL` (claude-opus-5), `CA_DAILY_ADOPT_CAP` (3), `CA_DISPATCH_PROJECT` (OmegaOS),
`CA_SEED_VERSIONS` (1 — how many latest versions the FIRST run treats as new).

## Arm / disarm

```
touch ~/.omega/state/changelog-adopt/armed   # allow gated auto-dispatch of adoptions
rm    ~/.omega/state/changelog-adopt/armed    # kill-switch: proposals stay report-only
```

## State (runtime, gitignored — never in the repo, R-ENV)

- `~/.omega/state/changelog-adopt/last_version` — the forward watermark (last version absorbed).
- `~/.omega/state/changelog-adopt/seen.jsonl`   — per-entry decision fingerprints (dedup memory).
- `~/.omega/state/changelog-adopt/armed`         — the kill-switch flag (absent by default).
- `~/.omega/state/changelog-adopt/runs/<date>/`  — per-run raw changelog, new entries, classify,
  gate, report.

## Cron

```
# 08:15 daily — DISARMED by default until the operator arms it.
15 8 * * * PATH=$HOME/.local/bin:/usr/local/bin:/usr/bin:/bin OMEGA_DIR=$HOME/.omega \
  bash $HOME/.omega/skills/changelog-adopt/scripts/adopt-run.sh \
  >> $HOME/.omega/logs/changelog-adopt.cron.log 2>&1
```

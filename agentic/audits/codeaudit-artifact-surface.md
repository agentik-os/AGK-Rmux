# /codeaudit — Artifact-Surface Change (Baseline Forensic Audit)

**Score: 83/100 — Grade A (Solid. Minor issues, nothing structural.)**
**Confidence: high** · Worker: `OmegaOS-worker-codeaudit-artifact-surface` · Date: 2026-07-03
**Range audited:** `edcdfd3..93f5b26` (commits `c905bff`, `6013fba` + merges `14a07b6`, `93f5b26`)
**Files in scope:** `crates/omega-core/src/rules.rs`, `crates/omega-core/src/doctor.rs`, `skills/artifact-design/SKILL.md`, `install.sh`, `scripts/verify-install.sh`

---

## Protocol deviations (recorded per the brief, L3)

- Brief constraint "READ-ONLY, write EXACTLY ONE new file" overrides the skill's `.audit/` output tree and fix phases 21-23: this run is audit-only and all phase evidence is consolidated here.
- `--user-need` / `--hinge` v2 flags were absent from dispatch; both were derived from the Mission prose (user-need = "audit the CODE itself: correctness, regressions, adjacent-block disturbance, quoting/heredoc safety, doctrine consistency"; hinge = the rules registry insert + doctor parity gate). Refusal-for-redispatch would have been thrash: the content was present.
- Phase 0 gather script (`audit-runner.sh code`) skipped: it writes `.code/` artifacts outside the permitted file scope and is Rust-blind on this codebase (memory-verified). Replaced with direct toolchain runs (cargo test, omega doctor, bash -n, verify-install.sh), all cited below.

## Hinge point

`crates/omega-core/src/rules.rs` registry vec (`all_rules()`, inserts at rules.rs:514 and rules.rs:525) paired with the `doctor.rs:148` `EXPECTED_OPS` parity gate. If this pivot is wrong, every dispatched agent's compiled doctrine block and the doctor doctrine check break at once. It received the deepest scrutiny (direct test run, doctor runtime, byte-level export parity, field-by-field struct inspection).

---

## Findings

### F-1 — HIGH — Audited change is not pushed to origin (L0 incomplete)
**Citation:** `git ls-remote origin main` → `edcdfd332d84`; `git rev-parse HEAD` → `93f5b26eaadc`. `./scripts/verify-install.sh` output: `✗ local commits not pushed to origin`, `✗ uncommitted changes — a fresh install would NOT get them`, banner `INSTALL PARITY FAILED — fix the above before declaring done (Law 0)`.
The brief's premise "merged on main" holds locally only. A fresh `git clone` from GitHub today reproduces NONE of this change: no R-ARTIFACT/R-HTML rules, no artifact-design skill, EXPECTED_OPS still 32. Not fixable by this worker (push is worker-denied; memory `verify-install-not-pushed-worker-gate`): **the oracle must commit the stragglers (F-2) and push**. Until then the change is not done under L0.

### F-2 — MEDIUM — Repo rule markdowns exist but were never committed (4th co-update incomplete)
**Citation:** `git status --short rules/` → `?? rules/R-ARTIFACT-reports-default-to-a-live-artifact-3-sur.md`, `?? rules/R-HTML-html-is-the-offline-report-surface-singl.md`; `git log --oneline -- rules/R-ARTIFACT-*.md rules/R-HTML-*.md` → empty (no history). All six sibling skill-router rules ARE tracked (`git ls-files rules/`).
The rules-rs new-rule checklist requires 4 co-updates; commits c905bff/6013fba shipped three (EXPECTED_OPS bump, rebuild+swap, export) but not the repo md. The files appeared on disk at 11:51 during this audit (a concurrent session, consistent with the concurrent-writers pattern on this repo), byte-identical to the export (`diff rules/R-ARTIFACT-*.md ~/.omega/rules/R-ARTIFACT-*.md` → identical, both files), so the fix is a plain `git add rules/ && commit && push`. Functional blast radius on fresh installs is low because `install.sh:1631` runs `omega rules export` from the compiled registry regardless (line 1627's `cp "$OMEGA_SRC/rules"/*.md` is best-effort `|| true`), but the repo convention and verify-install's clean-tree gate both break until committed.

### F-3 — MEDIUM — Compiled doctrine routes to a skill the repo does not ship (`web-artifacts-builder`)
**Citation:** R-ARTIFACT description (rules.rs:518): "Complex interactive artifacts (state, routing, shadcn/ui) go through web-artifacts-builder"; `skills/artifact-design/SKILL.md:40` (router table row 4) and `SKILL.md:148` (reference path `~/.omega/skills/web-artifacts-builder/SKILL.md`). Counter-evidence hunt: `git ls-files | grep web-artifacts` → no match; `grep -n web-artifacts-builder install.sh` → no match. The skill exists only as a local install on this box (`~/.omega/skills/web-artifacts-builder`, symlink dated Jun 11), never published to the repo.
The gap predates this change, but this change PROMOTES the phantom into compiled doctrine (`scopes: ALL`) that ships to every OmegaOS install: on a fresh box, any agent following R-ARTIFACT's escalation row or SKILL.md §7 hits a dead path. R-SKILLPUB says a skill that lives only locally does not exist. Fix: vendor `web-artifacts-builder` into `skills/` + install.sh (preferred), or reword the rule/skill to mark it "if installed" like SKILL.md already does for dataviz (SKILL.md:125-126, "load ... IF the session lists it").

### F-4 — MEDIUM — CLAIM vs REALITY: "premium reading bar" required by the rule, absent from the skill's design contract
**Citation:** R-ARTIFACT description (rules.rs:518): "The page follows the artifact contract: ... premium reading bar, zero em/en dashes ...". `grep -in 'reading bar\|progress bar' skills/artifact-design/SKILL.md` → no match (exit 1). SKILL.md §1 (lines 32-33) states the sync contract explicitly: "Mirror of rule R-ARTIFACT. Keep the two in sync: an edit here without the rule (or the reverse) splits the doctrine."
The rule demands a page element the skill's design contract (§4, SKILL.md:91-128) never defines, requires, or explains; the term is defined nowhere in the repo, so no agent can implement or verify it. Every artifact produced by the skill silently violates the rule's letter. Fix: either add the reading-bar requirement (with a definition) to SKILL.md §4, or drop the phrase from the rule text (plus export + repo md re-sync).

### F-5 — LOW — added_at chronology inverted inside the registry
**Citation:** rules.rs:521 (`R-ARTIFACT`, `added_at: "2026-07-03"`) precedes rules.rs:532 (`R-HTML`, `added_at: "2026-07-02"`), and R-HTML's date predates its actual registry insertion (commit c905bff, 2026-07-03). Falsification: no `sort_by`/ordering on `added_at` anywhere in rules.rs (grep), so this is cosmetic; the `reason` field (rules.rs:534) documents the backdating intent (operator ask 07-02, compiled 07-03). No action required; noted so a future date-ordered renderer does not surprise anyone.

### F-6 — INFO — `cp -r "$ARTD_SRC"/*` aborts install on an empty skill dir under `set -euo pipefail`
**Citation:** install.sh:8 (`set -euo pipefail`), install.sh:1199 (`cp -r "$ARTD_SRC"/*`). Guarded by the `-d` check and the dir ships SKILL.md, and the pattern is byte-consistent with every sibling block (watch: install.sh:1167; marketing loop: install.sh:1229). Pre-existing pattern, not a regression introduced here.

### F-7 — INFO — Bare `/artifact-design` command stub absent on this box (expected)
**Citation:** `ls ~/.claude/commands/artifact-design.md` → not found; `omg-artifact-design.md` present with correct expanded content (backticked absolute path renders literally, heredoc verified on disk). `omega sync` prunes bare `<name>.md` stubs by design (memory `skill-ship-stub-prune-and-smoke`); the skill resolves by name via the `~/.claude/skills/artifact-design` symlink (present, dated Jul 3 11:45). verify-install.sh:168 greps install.sh text only, so it stays green. Not a defect.

### F-8 — INFO — Dated runtime facts baked into SKILL.md will age
**Citation:** SKILL.md:13 ("runtime-verified 2026-07-03 on Claude Code 2.1.199"), SKILL.md:22-24 (Team/Enterprise beta status, blog citation). Good R-CITE practice today; re-verify when the artifact feature exits beta or the headless boundary changes.

### F-9 — INFO — Prompt-weight growth: two ~2KB rules at `scopes: ALL`
**Citation:** rules.rs:519/530 (`scopes: ALL`) route both descriptions through `agent_context_block` (rules.rs:633) into every dispatched agent's prompt. Consistent with registry style, but Reporting now carries four long rules; worth watching as the registry grows.

---

## Regressions (adjacent-block disturbance check — the brief's explicit ask)

**None found.**
- **install.sh watch block (above, ends install.sh:1187) and marketing block (below, starts install.sh:1217):** the insertion sits cleanly between them; both neighbors byte-intact in `git diff edcdfd3..HEAD` (context lines only) and in direct inspection (`sed -n '1140,1260p'`). `bash -n install.sh` → clean. Heredoc audit of the new block: unquoted `<<EOF` with intentional `$ARTD_DST`/`$cmd` expansion and escaped backticks (`\``) rendering literally; proven at runtime by the generated `~/.claude/commands/omg-artifact-design.md` content. Variable names `ARTD_SRC`/`ARTD_DST`/`ACMD` are unique in the file (grep: 9 hits, all inside the new block), no collision with `WCMD`/`TCMD`/`GTMK_CMD`.
- **verify-install.sh:168 insertion:** `bash -n` clean; the assertion's single-quoted grep patterns match install.sh literally; runtime run shows the new line green (`✓ artifact-design skill ... shipped + wired`) and the full run's only failures are the two repo-state lines of F-1 (total `✗` count = 2; zero skill-block assertions regressed).
- **rules.rs adjacency:** R-MODEL (the entry immediately above, rules.rs:~490-513) untouched; the vec closes correctly; `cargo test -p omega-core --lib rules` → **11 passed, 0 failed** including `registry_matches_markdown_rule_files` and `export_prune_preserves_disk_only_rules`.
- **doctor.rs:** `EXPECTED_OPS` changed in exactly one const (doctor.rs:148) plus its comment; runtime `omega doctor` → `[+] doctrine 6 Laws + 35 Rules`, `[+] doctrine files 41 rule files match the registry` (installed binary was rebuilt and swapped).
- No merge-conflict markers in any scoped file (grep `<<<<<<<`/`>>>>>>>` → zero lines).

## Doctrine consistency: R-ARTIFACT rule text vs SKILL.md router

The router TABLE itself is consistent: surface 1 default + HTML twin, surface 2 on file-wanted or no-Artifact-tool (rule text; SKILL.md splits this across table row 2 and §2 Preconditions, semantically identical), surface 3 PDF explicit-only via `omega pdf`, escalation row to web-artifacts-builder, favicon/URL-stability discipline, R-NODASH kill pass (SKILL.md §5; `grep -P '[\x{2013}\x{2014}]' skills/artifact-design/SKILL.md` → 0 matches), never-fabricate-a-URL (L1) present in both. The two divergences found are F-4 (reading bar, rule-only) and F-3 (both sides reference an unshipped skill). Registry ↔ exported-md parity verified byte-level: registry description == `~/.omega/rules/<id>.md` body for both rules (python exact-compare, first-divergence probe → `BODY_MATCH`), and repo `rules/*.md` == export (diff → identical).

## Falsifiable tests run (Popper evidence, all outputs captured this session)

| # | Hypothesis that would fail | Command | Actual |
|---|---|---|---|
| 1 | Registry insert breaks tests | `~/.cargo/bin/cargo test -p omega-core --lib rules` | `11 passed; 0 failed` |
| 2 | Installed binary stale / count wrong | `omega doctor` | `6 Laws + 35 Rules`, `41 rule files match` |
| 3 | Heredoc/quoting broke either script | `bash -n install.sh && bash -n scripts/verify-install.sh` | both clean |
| 4 | New assertion or a neighbor regressed | `./scripts/verify-install.sh` | artifact line ✓; only 2 ✗ (repo state, F-1) |
| 5 | Change is on origin | `git ls-remote origin main` vs `git rev-parse HEAD` | `edcdfd3…` ≠ `93f5b26…` → NOT pushed |
| 6 | Repo mds committed | `git status --short rules/` + `git log -- rules/R-ARTIFACT*` | `??` untracked, zero history |
| 7 | Export drifted from registry | python exact byte-compare, both rules | `BODY_MATCH` × 2 |
| 8 | Installed skill drifted from repo | `diff skills/artifact-design/SKILL.md ~/.omega/skills/artifact-design/SKILL.md` | identical |
| 9 | web-artifacts-builder ships | `git ls-files \| grep web-artifacts`; `grep install.sh` | zero matches (F-3 stands) |
| 10 | Reading bar exists in skill contract | `grep -in 'reading bar\|progress bar' skills/artifact-design/SKILL.md` | no match (F-4 stands) |
| 11 | Dash-hygiene violated | `grep -cP '[\x{2013}\x{2014}]' skills/artifact-design/SKILL.md` | 0 |
| 12 | Stub generation mangled by heredoc | `cat ~/.claude/commands/omg-artifact-design.md` | correct literal backticks + expanded path |

## Score derivation

Applicable phases (N/A for a registry+markdown+shell change: 4 Data Flow, 5 State Mutation, 6 Concurrency, 18 API Contracts): applicable max **355**. Raw **295**: P1 Phantoms 24/30 (F-3), P2 Deps 20/20, P3 Contracts 20/25 (F-4), P7 Blast 18/20, P8 Time Bombs 18/20 (F-8), P9 Supply Chain 20/20 (markdown-only, zero external code), P10 Error Prop 27/30, P11 Behavioral 18/20, P12 Config Drift 10/20 (F-1, F-2), P12.5 Feature Verify 27/30, P13 Entropy 15/15 (block mirrors sibling patterns exactly), P14 Git Forensics 10.5/15 (co-update discipline miss, a recurring pattern), P15 Runtime 20/20, P16 Observability 25/25, P17 Tests 22.5/25, P19 Resilience 20/20 (idempotent re-install). **295/355 = 83/100 → Grade A.**

## Verdict summary

The CODE of the change is clean: the two registry entries are field-correct and test-green, the doctor gate was bumped in lockstep, the install.sh block is quoting-safe and pattern-identical to its siblings, the verify-install assertion is accurate, and no adjacent block was disturbed. What keeps it from S-grade is SHIPPING state, not code: the change is unpushed with two doctrine markdowns still untracked (F-1/F-2, the L0 gap — oracle action: `git add rules/ agentic/ && commit && push`, then verify-install goes fully green), plus two doctrine-drift items worth one small follow-up commit (F-3 phantom web-artifacts-builder reference, F-4 undefined "premium reading bar").

```json
{
  "score": 83, "score_raw": "295/355", "confidence": "high",
  "skill_used": "codeaudit", "audit_only": true,
  "user_need_match": {
    "quote": "Audit the CODE itself: correctness, regressions, whether the inserted lines disturb adjacent blocks, quoting/heredoc safety in the new install.sh block, and doctrine consistency between the R-ARTIFACT rule text and SKILL.md's router table.",
    "addressed": true,
    "evidence": "Each axis answered with runtime evidence: correctness (tests 1,2,7), regressions/adjacency (tests 3,4 + diff-boundary inspection of install.sh:1187/1217 and rules.rs R-MODEL), heredoc safety (tests 3,12), doctrine consistency (tests 7,9,10 -> F-3/F-4)."
  },
  "hinge_findings": [
    {"location": "crates/omega-core/src/rules.rs:514-535", "concern": "registry insert corrupts doctrine funnel", "verified_safe_by": "tests 1,2,7"},
    {"location": "crates/omega-core/src/doctor.rs:148", "concern": "EXPECTED_OPS desync", "verified_safe_by": "test 2 (runtime doctor 6+35, 41 files)"},
    {"location": "install.sh:1189-1215", "concern": "heredoc expansion breaks stubs or neighbors", "verified_safe_by": "tests 3,4,12"}
  ],
  "issues_found_and_fixed": [],
  "not_done": [
    "F-1: push to origin (worker-denied; oracle action)",
    "F-2: git add rules/R-ARTIFACT-*.md rules/R-HTML-*.md + commit (oracle action)",
    "F-3/F-4 doctrine follow-ups (need a write-scoped worker)"
  ],
  "confidence_basis": "Every load-bearing claim was directly executed this session (12 falsifiable tests with captured output); the hinge was verified at three independent layers (unit tests, installed-binary runtime, byte-level export parity); one candidate finding (export body drift) was self-falsified before reporting. Scope limit: no fresh-clone install rehearsal was run (would exceed the read-only single-file contract), so fresh-install behavior of F-3 is inferred from install.sh code paths, not observed.",
  "banned_phrase_check": "passed"
}
```

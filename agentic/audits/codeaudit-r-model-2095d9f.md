# /codeaudit — R-MODEL (commit 2095d9f) — Forensic Verdict

**Domain:** code (typed-rules registry + docs) · **Scope:** the CHANGE `git show 2095d9f`, not the repo
**Auditor mindset:** adversarial (assume Codex wrote it), Popper-falsification, evidence-or-it-didn't-happen (R-CITE)
**Date:** 2026-07-02 · **Repo:** /home/vibe/Station/SideBusiness/OmegaOS

## SCORE: 98 / 100 — PASS (ship)

**Confidence: high** — every load-bearing claim below was falsified against the *running* `omega` binary (which already contains this commit: it reports 32 ops + R-MODEL), not against source text alone.

---

## Change surface (5 files, +40 / −5)

| File | Change | Traces to R-MODEL? |
|------|--------|-------------------|
| `crates/omega-core/src/rules.rs` | +new `Rule{ id:"R-MODEL" }` (rules.rs:491‑503); R-ORCH phrase amended (rules.rs:155) | ✅ |
| `crates/omega-core/src/doctor.rs` | `EXPECTED_OPS` 26→32 + comment (doctor.rs:144‑147) | ✅ |
| `rules/R-MODEL-…-ta.md` | new canonical export (13 lines) | ✅ |
| `agents/aisb/CLAUDE.md` | +blockquote after Model-Tiers table (CLAUDE.md:122) | ✅ |
| `agents/oracle.md` | Ultracode pin annotated (oracle.md:15) + "Model & effort per agent — R-MODEL" block (oracle.md:279‑287) | ✅ |

Surgicality: **PASS.** No unrelated hunk in the diff; every changed line is R-MODEL wiring or the one documented R-ORCH conflict-fix.

---

## Dimension-by-dimension (Popper falsification — commands run + verbatim output)

### 1. Rust correctness of the new registry entry — PASS
- **Scope wiring:** entry uses `scopes: ALL`. `ALL` is `const ALL: &[RuleScope] = &[RuleScope::Master, RuleScope::Oracle, RuleScope::Worker];` (`rules.rs:70`). → exactly the Master+Oracle+Worker the commit message claims. **Falsified against runtime**, not text:
  `omega rules context {master,oracle,worker} | grep -c R-MODEL` → **2 / 2 / 2** (present in all three scopes).
- **Category/kind:** `kind: RuleKind::Rule`, `category: RuleCategory::Orchestration` (`rules.rs:494‑495`) — correct tier for a "which model" doctrine, siblings R-ORCH/R-COUNCIL are Orchestration too. `operational_rules()` filters `kind == Rule` (`rules.rs:515‑516`), so an operational (non-Law) entry is required for injection — satisfied.
- **Struct completeness:** all fields present (`id, title, kind, category, description, applies_to:&[], scopes, added_at:"2026-07-02", reason`). Compiles: the live binary reflects it (below), so the release build succeeded — code-and-runtime agree (L1).

### 2. `doctor.rs` constant coherence — PASS (and heals a real latent drift)
- `EXPECTED_OPS: usize = 32` (`doctor.rs:147`). Runtime `operational_rules().len()` = **32**:
  `omega rules list` → `OPERATIONAL RULES (32)`; `omega doctor` → `[+] doctrine   6 Laws + 32 Rules` **(green)**.
- **Latent bug healed (POSITIVE finding):** the pre-image `EXPECTED_OPS = 26` vs an actual 31 pre-commit ops would trip the `else` branch at `doctor.rs:152‑156`, i.e. `Check::warn("doctrine", "… (expected 6 + 26)")` — a *yellow* doctor line. 26→32 (+6) = +5 stale (R-PROJ..R-LOOP, never bumped) +1 new (R-MODEL), matching the commit message. This commit restores the doctor check to green. Confirmed no other `EXPECTED_OPS` reference in the crate.

### 3. Registry ↔ markdown consistency — PASS (byte-identical)
- `diff rules/R-MODEL-…-ta.md ~/.omega/rules/R-MODEL-…-ta.md` (fresh `omega rules export`) → **no diff**.
- `md5sum` both → `c41a3d5faabe018208c3dcfab79eecab` (identical). The `description`/`reason` strings in `rules.rs` are the source; the exported `.md` matches the committed canonical file. `omega doctor` → `[+] doctrine files   38 rule files match the registry` (filename grammar + id-set parity hold).

### 4. Regression risk to the prompt-injection funnel (`agent_context_block` / `rules_for_scope`) — PASS
- R-MODEL reaches every level (2/2/2 above). Adversarial check on the R-ORCH amendment: the old phrase `workers ARE Opus-4.8 workflows` must be gone from the *injected* output, not just source:
  `omega rules context worker | grep -c "Opus-4.8 workflows"` → **0**. New phrasing `full-power workflows (model tier per R-MODEL)` is what ships. No stale duplicate, no double-injection.
- Laws unchanged (`EXPECTED_LAWS=6`, runtime 6) — funnel Law tier untouched.

### 5. Doc-edit surgicality — PASS
- `agents/aisb/CLAUDE.md:122` blockquote is well-formed, placed directly under the Model-Tiers table, and **correctly resolves the intentional conflict**: the table pins (`claude-sonnet-4-6`, `claude-haiku-4-5-20251001`) OVERRIDE R-MODEL's map (Sonnet 5 / Haiku 4.5). R-MODEL's own text names the AISB table as overriding doctrine → the two docs are reconciled, not contradictory. This is the sharpest adversarial trap in the change and it is handled.
- `agents/oracle.md` additions trace 1:1 to R-MODEL; the Ultracode xhigh pin is annotated as "the dispatch pin" so it is not misread as violating "never re-tier mid-session."

---

## Findings

| # | Sev | File:line | Finding |
|---|-----|-----------|---------|
| F1 | **INFO** | `rules.rs:495` / `rules/R-MODEL-…md:9` | R-MODEL's prose uses the **undated** Haiku alias `claude-haiku-4-5`, while the same repo's AISB table uses the exact id `claude-haiku-4-5-20251001` (`agents/aisb/CLAUDE.md:120`). A rule that itself preaches "use live model ids — never a retired id" using a shortened alias is a mild self-consistency nit. Mitigated: the rule explicitly defers exact ids to the `claude-api` skill as SSOT ("on any divergence… the skill wins"), so this is acceptable shorthand, not an error. Opus/Sonnet/Fable aliases (`claude-opus-4-8`, `claude-sonnet-5`, `claude-fable-5`) match the canonical ids exactly. **−2 pts.** |
| F2 | **INFO (out of scope / not a regression)** | `agents/aisb/CLAUDE.md:124` | Adjacent pre-existing staleness: "the standard **Opus 4.7**" while the tables + `oracle.md` say Opus 4.8. NOT introduced by this commit (unchanged line), but the new R-MODEL blockquote now sits 2 lines above it, making the drift more visible. Flagged for the oracle; correctly left untouched here per surgical-change discipline (R-KARPATHY). |
| — | **POSITIVE** | `doctor.rs:147` | Healed the 5-rule EXPECTED_OPS drift → `omega doctor` doctrine check back to green. |

**No CRITICAL / HIGH / MEDIUM / LOW findings.** Zero phantom refs, zero broken imports, zero funnel regressions, zero build/runtime disagreement.

---

## not_done / scope of verification (honesty per preamble #90/#100)
- **Proved:** registry↔markdown byte parity, runtime ops count (32), scope injection into all 3 levels, R-ORCH old-phrase removal from injected context, doctor green, struct correctness (via live binary).
- **NOT independently re-run this pass** (accepted from oracle's prior evidence, transitively confirmed by the live binary already reflecting the commit): full `cargo build --release` and `cargo test -p omega-core 334/334`. The running `omega` binary reporting `32 rules + R-MODEL` is itself proof the crate compiled with this change; a fresh `cargo test` was not re-executed here.
- **F1** is a judgment nit, not a runtime failure; the rule's own SSOT-deferral clause defuses it.

## Verdict
Clean, surgical, runtime-true, install-parity intact (canonical md shipped + exported identical). **Score 98/100 — ship.** The only actionable item (F1) is optional polish; F2 is a separate pre-existing cleanup for the oracle to schedule.

--- **Resume:** Commit 2095d9f (R-MODEL) audité — 98/100, PASS. Registre↔markdown byte-identiques (md5 confirmé), doctor 6+32 vert (drift de 5 règles réparé), injection dans les 3 scopes vérifiée au runtime, ancienne phrase R-ORCH supprimée, édition chirurgicale. Seul nit (INFO): l'id Haiku abrégé `claude-haiku-4-5` dans une règle qui prêche « ids vivants » — atténué par le renvoi au SSOT claude-api. Rien à corriger d'obligatoire.

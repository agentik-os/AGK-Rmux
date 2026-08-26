# codeaudit — daily auto-update + System tab

**Ref** `5f18b6a..HEAD` (own commits `acf3129 273e4c6 08ecceb 118fff5`)
**Score** 100 · **Confidence** high · **Machine-readable** `.audit/verdict-auto-update.json` (local, gitignored)

**User need (verbatim)**
> Le système OmegaOS doit inclure une vérification des mises à jour quotidienne pour toutes les personnes qui l'installent. Une fois cette vérification effectuée, une mise à jour automatique doit être appliquée.
> Vérifie que la mise à jour auto apparaît dans l'onglet System et que tout est bien en prod

---

## Hinge

`hinge-analyzer.sh --ref=acf3129~1` returned 81 regions, 65 of them in the Rust surface, clustered on
`crates/omega-cli/src/main.rs:7208-7400` — `cmd_update_auto`. That matches the manual read: this is the
one function that decides whether to install remote code onto every user's machine, unattended.

Phase 0 note: the programmatic gather ran **only** `large-file-scanner` on this Rust workspace
(no ESLint/tsc/ruff applies). Static-analysis evidence was therefore supplied directly — `cargo build`
with `-D warnings`, clippy scoped to the new surface, and a panic-surface census with the test-module
boundary proven — rather than claimed from the gather.

---

## HIGH — a failed install after the fast-forward was never retried

**Fixed.** `crates/omega-core/src/auto_update.rs::decide()`

The apply path fast-forwards **before** running `install.sh`. When the install then failed, the
checkout sat at the new commit while the installed binary stayed old. Every later run computed
`behind == 0`, returned `UpToDate`, and printed *"already up to date"* — new source, stale binary,
and an updater insisting nothing was wrong. The 3-attempt cap never engaged, because the `Apply`
branch was unreachable.

This survived my own sandbox testing because I reset the clone between attempts — which is exactly
the state the bug does not survive.

**Falsification (runtime, sandboxed clone, crontab shimmed out):**

| | before fix | after fix, same sandbox state |
|---|---|---|
| run 1 | ff ok, `install.sh FAILED … (attempt 1 of 3)` | same |
| run 2 | `already up to date` · `last_applied: None` · **no binary installed** | `install.sh FAILED … (attempt 2 of 3)` |
| run 3 | — | `install.sh FAILED … (attempt 3 of 3)` |
| run 4 | — | `commit 118fff5 failed to install 3 times — not retrying, this needs a human` |

**Fix.** `CheckoutState` carries `head`; `decide()` computes `install_owed` — the commit we last failed
on is the one checked out — and applies even at `behind == 0`. Four regression tests, including the two
that keep it honest: a clean machine and a failure recorded against a *different* commit must both
still do nothing.

## MEDIUM — TOCTOU on the single-flight lock

**Fixed.** `crates/omega-cli/src/main.rs`

The lock used metadata-check then write: two runs starting together both saw no lock and both proceeded
to rebuild the same binary. Claims with `OpenOptions::create_new` (O_EXCL) now; the 6-hour stale
takeover is unchanged, with a second claim attempt and a clean bail if that also fails.

Verified: a held lock blocks, an 8-hour-old lock is taken over, and the lock file is absent after the run.

## MEDIUM — a test that measured the filesystem

**Fixed.** `crates/omega-core/src/rules.rs`

`a_mission_too_short_to_classify_gets_everything` compared two full context blocks byte-for-byte. Both
read `~/.omega/agents/_brief-preamble.md` at call time, which a concurrent install or `omega sync`
rewrites — so it failed intermittently, testing the filesystem rather than the fallback. Now compares
with the preamble stripped and asserts the real property: an unclassifiable mission inlines every
scoped rule and indexes none.

---

## Recorded, not fixed

**MEDIUM — god files.** `crates/omega-tui/src/ui.rs` is 202 KB / ~4900 lines and this change added
~416 lines to it; R-KARPATHY sets the refactor alarm at ~2000. Same class: `crates/omega-cli/src/main.rs`
(332 KB), `crates/omega-tui/src/input.rs` (158 KB). Splitting three god files along responsibility seams
is a structural refactor with a large blast radius — it belongs in its own pass, not bolted onto feature
work.

**LOW — manual update does not clear auto-update failure state.** A successful manual `omega update`
leaves a recorded failure in place, so the next nightly run does one redundant rebuild before recording
success. Self-heals after exactly one extra run, on a machine that already had a failed install.

---

## Falsifiable tests

| test | hypothesis it could have failed on | result |
|---|---|---|
| failed install after ff is retried | reading `behind` alone strands the machine | **found the defect**, fixed, re-verified |
| `RUSTFLAGS="-D warnings" cargo build --release` | new code adds a warning CI rejects | exit 0 |
| clippy scoped to the new surface | clippy flags the new code | no matches |
| panic-surface census + `mod tests` boundaries | new modules can panic in production | zero hits above the test modules |
| lock: held / stale / released | two runs both rebuild, or the lock wedges updates | blocks, takes over, releases |
| `cargo test --workspace` | the change breaks existing behaviour | 586 passed, 0 failed |

**Not verified:** a real 03:30 cron firing (the rendered command line was executed by hand instead of
waiting for the scheduler), and a genuine two-process collision on the lock (atomicity rests on `O_EXCL`
semantics rather than an observed race).

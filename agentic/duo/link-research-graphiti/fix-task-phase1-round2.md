# /duo FIX Task, Phase 1, Round 2 of 3

Read completely:

- `agentic/duo/link-research-graphiti/plan.md`, Phase 1
- `agentic/duo/link-research-graphiti/code-task-phase1.md`
- `agentic/duo/link-research-graphiti/fix-task-phase1.md`
- `/home/vibe/.omega/logs/duo/2026-07-23T22-05-25-388Z-3867933-54ae11ff-review.log`
- the current real diff

The Claude task reached a concrete `FIX` verdict, but the bridge correctly
rejected that run because Claude reran Cargo inside the repository and changed
ignored `target/` artifacts. No tracked source was changed by Claude. Treat the
findings below as repair input, not as a green review.

Implement the following surgical repairs. Keep the same eight tracked files
unless adding a Cargo dependency is truly unavoidable. Do not touch
`telegram-bot/` in this phase.

## Mandatory repairs

1. Add an explicit `omega codex-login-abort --pid <n>` path. It must take the
   flow lock, match the requested PID to the recorded flow, and act only when
   the recorded process identity and exact supervisor argv are proven. Never
   signal or restore on `Unknown`, `IdentityMismatch`, or an unrelated PID.
   After stopping the owned supervisor, wait for its exit sentinel and settle
   through the existing monotonic abandoned-flow reducer. Return truthful JSON;
   `restored` must never claim a restore when no topology was restored. Keep
   `codex-login-status` observation-only while a child is running. Add runtime
   tests. Document that the existing Telegram Cancel button must be repointed
   when the Telegram phase owns that file.
2. On non-Linux Unix, a conclusively absent PID must classify as `Exited`, not
   `Unknown`. A `ps` failure alone is insufficient: use a second same-user
   presence probe and preserve `Unknown` for genuine probe ambiguity. Keep
   non-Unix builds explicit and safe. Add classifier coverage.
3. Codex and Claude may enter guarded degraded read mode only after positive
   evidence of a sandbox/permission/read-tool denial. A provider 5xx, network
   failure, malformed response, or missing marker must return
   `worktree-read-unavailable` without bypassing the native sandbox. Add a
   transient-500 self-test proving no bypass argv is used.
4. Strict whole-tree plus Git-metadata fingerprinting and live watcher failure
   remain mandatory for degraded mode. Native read-only mode must not reject a
   valid review merely because an unrelated writer changed ignored build
   output. Scope native guarding to Git-observable source/index state. Add a
   self-test for ignored artifact churn during a native run.
5. Remove the `DUO_SELFTEST_CHILD` fail-open bypass. Add a test-only watcher
   failure injection that can only make the bridge fail more closed, never
   bypass protection, and assert the real `no live filesystem watcher` result.
   If this shared host temporarily exhausts inotify, mutation tests may accept
   the same fail-closed result, but production code must never downgrade to
   snapshots-only degraded mode.
6. Preserve the new Claude native-plan-first then guarded-degraded behavior,
   applying the same positive-evidence rule and truthful
   `sandbox_degraded/capabilities` schema.
7. In doctor, a machine with no valid Codex credential must say it is not
   logged in and give the actionable `omega codex-login` command. Reserve the
   topology-incomplete warning for a genuinely partial/split topology.
8. Restore the two deleted parser regressions proving a version banner and
   device URL are never parsed as the one-time code.
9. Remove the unreachable `codexArgv` tail. Either add real coverage for
   `ensure_legacy_symlink("codex")` or remove its unreachable special branch
   without weakening the canonical reconciler.
10. Replace verifier greps for incidental TypeScript implementation lines with
    assertions on named self-test invariants, including native attempt zero and
    fail-closed watcher behavior.

## Preserve without weakening

- newest-valid monotonic credential selection
- quarantine before canonical write
- child exit plus fresh credential required for successful login
- `Unknown` never signals and never restores
- exclusive flow/reconciliation locks
- alternate `OMEGA_DIR` and `CODEX_HOME`
- stdin task delivery
- strict degraded guard coverage of worktree, ignored/untracked files, empty
  directories, index, and full Git metadata
- ordinary doctor remains probe-free; only `doctor --deep` performs a live call
- no live login, credential mutation, install, external process kill, commit,
  or push during this code turn

## Required verification

```bash
git diff --check
cargo check -p omega
cargo test -p omega
cargo test -p omega-core
tools/duo/bin/omega-duo --self-test
bash -n install.sh scripts/verify-install.sh
```

Also inspect `git diff --stat`, `git diff -w --stat`, changed-line Clippy, and
confirm `forwarder.rs` plus `usage.rs` remain exactly at HEAD. Do not run a
repo-wide formatter.

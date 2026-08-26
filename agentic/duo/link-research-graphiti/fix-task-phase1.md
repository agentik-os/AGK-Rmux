# /duo FIX Task, Phase 1, Round 1 of 3

Read these completely:

- `agentic/duo/link-research-graphiti/plan.md`, Phase 1
- `agentic/duo/link-research-graphiti/code-task-phase1.md`
- `/home/vibe/.omega/logs/duo/2026-07-23T21-01-23-915Z-review.log`
- the current real diff

Implement the Claude FIX verdict surgically. Preserve the already-proven
credential invariants and delete avoidable complexity.

## Mandatory repairs

1. Restore `crates/omega-cli/src/forwarder.rs` and
   `crates/omega-cli/src/usage.rs` exactly to HEAD. They contain only accidental
   formatting changes and are outside scope. Use `apply_patch`; do not use
   `git checkout`, reset, or destructive commands.
2. In `crates/omega-cli/src/main.rs` and
   `crates/omega-core/src/doctor.rs`, restore every accidental whole-file
   rustfmt change outside the Phase 1 functional regions. Keep the original
   local style. Do not run repo-wide `cargo fmt`; run rustfmt check only on
   changed files if useful.
3. `omega-duo` attempt zero must always try Codex native
   `--sandbox read-only`. Escalate to guarded degraded mode only after that
   attempt demonstrably cannot read the repository. Missing/unspawnable bwrap
   is unknown, not evidence that the native Codex sandbox is broken. Add a
   self-test that checks attempt-zero argv.
4. Scrub diagnostics line-by-line against task lines before both raw 401 and
   quota classification. Add tests proving an echoed task line beginning with
   `401` is a task failure, and a partial quota-phrase echo does not set the
   exhausted flag.
5. Add file and parent-directory `fsync` to the pre-flow backup atomic writer.
6. Fix non-Unix behavior so it never clobbers the fresh native credential or
   loops trying to create a symlink. Prefer an explicit unsupported-platform
   error if a safe single-copy topology cannot be guaranteed.
7. Remove the unconditional live billed `codex exec` probe from ordinary
   `omega doctor` and its 3-hour cron path. Keep cheap topology diagnostics.
   Expose live probing only through an explicit user action or a TTL-cached
   path that ordinary doctor does not invoke.
8. Delete the speculative recorded-supervisor recovery scan and the heavy
   multi-process launcher/version diagnostic identified in review F8. Keep a
   simple no-signal diagnostic if useful, with one shared process helper rather
   than duplicate `/proc` and `ps` implementations.
9. If exactly one credential has `last_refresh`, use safe mtime/cross-mode logic
   instead of unconditionally rolling it back. Quarantine invalid extra flow
   backups before cleanup.
10. Add tests for startup reconciliation under a live flow and an exited flow,
    process classification rejecting task text, and the CLI status JSON shape.
11. Add a dedicated reconcile command with a truthful non-zero exit on failure;
    installer must call that, not `omega init` with discarded diagnostics.
12. Clear all clippy findings introduced on changed lines. Do not attempt to
    repair unrelated pre-existing workspace lint or formatting debt.
13. Update skill/install/verifier text for the final actual behavior. Add bwrap
    dependency reporting without making a missing bwrap force unsafe degraded
    mode.

## Non-negotiable invariants to preserve

- Monotonic newest-valid credential selection.
- Quarantine-before-canonical-write ordering.
- Child exit plus fresh credential required for flow success.
- `Unknown` never signals or restores.
- Exclusive flow lock and PID-handle safety.
- Alternate `OMEGA_DIR` and `CODEX_HOME`.
- stdin task delivery.
- read-only mutation guard covering worktree, index, ignored/untracked files,
  empty directories, and Git metadata.
- no live login, no live credential mutation, no process kill, no install,
  no commit, no push.

## Required verification

```bash
git diff --check
cargo check -p omega
cargo test -p omega
cargo test -p omega-core
tools/duo/bin/omega-duo --self-test
bun test tools/duo
```

`bun test tools/duo` may legitimately report no tests; say so. Inspect
`git diff --stat` and `git diff -w --stat` before finishing. The raw diff must
not contain unrelated formatting churn or out-of-scope files.

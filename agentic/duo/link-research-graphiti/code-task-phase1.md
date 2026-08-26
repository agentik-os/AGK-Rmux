# /duo Code Task, Phase 1

Read `agentic/duo/link-research-graphiti/plan.md` completely, then implement
only **Phase 1: persistent Codex authentication and truthful `/duo`**.

This is the Codex implementation step of the mandatory `/duo` loop.

## Files allowed

- `crates/omega-core/src/codex_login.rs`
- `crates/omega-core/src/credentials.rs`
- `crates/omega-core/src/doctor.rs` only if needed for the diagnostic
- `crates/omega-cli/src/main.rs`
- `tools/duo/bin/omega-duo`
- `skills/duo/SKILL.md`
- focused tests in existing modules
- `install.sh`
- `scripts/verify-install.sh`

Do not touch any other implementation file. Do not edit the plan files.

## Required behavior

1. Honor `CODEX_HOME` and Omega's canonical `OMEGA_DIR` resolver.
2. Reconcile/adopt fresh Codex native credentials into the canonical Omega
   store atomically and restore the native symlink. Reconcile Codex alongside
   Claude on Omega startup.
3. Never restore an older `last_refresh` over a newer valid credential.
   Preserve valid losing copies in an owner-only quarantine. Never print or log
   token contents.
4. Make successful and abandoned device-login paths restore a single canonical
   topology. Add an exclusive flow lock/state so concurrent login starts cannot
   share or overwrite a backup/log.
5. Extend status with an `Unknown` state for spawn, timeout, and unparseable
   output. `Unknown` must never kill a process or restore credentials.
6. A recorded flow may succeed only when its child exited successfully and the
   resulting credential is valid and fresh relative to the backup. A shallow
   `codex login status` from another process is not sufficient.
7. Add safe diagnostics for native/canonical split, stale backups, live Codex
   process versions, and real-auth usability. Diagnose, never signal unrelated
   Codex or rmux sessions.
8. `omega-duo` must pass task content on stdin, preflight whether read-only
   sandbox commands can read the worktree, and never return a repository-blind
   plan/review as green. If it must use a degraded external write guard, emit
   `sandbox_degraded` and `capabilities.shell_exec`, and fail on any worktree
   mutation.
9. Distinguish `codex-unauthenticated` from quota fallback. Do not fall back to
   Claude on a 401.
10. Extend the deterministic self-test for large stdin delivery, auth failure,
    degraded/read capability reporting, and read-only mutation detection.
11. Update installer and install verification for every changed shipped asset.
12. Preserve all live sessions and current user files. Do not run a live login,
    mutate live credentials, kill processes, commit, or push.

## Verification

Run and report:

```bash
cargo fmt --check
cargo test -p omega-core codex_login
cargo test -p omega-core credentials
cargo test -p omega-cli
bun test tools/duo
tools/duo/bin/omega-duo --self-test
```

If `bun test tools/duo` has no discoverable tests, say so and rely on the
expanded `--self-test`; do not invent a passing command.

Inspect the real diff before finishing. Keep changes surgical and code/comments
in English.

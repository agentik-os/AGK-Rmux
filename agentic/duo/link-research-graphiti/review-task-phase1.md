# /duo Claude Review, Phase 1

Review the real current diff for Phase 1 against:

- `agentic/duo/link-research-graphiti/plan.md`, Phase 1
- `agentic/duo/link-research-graphiti/code-task-phase1.md`
- `skills/duo/SKILL.md`
- the repository's AGENTS.md laws

This is the mandatory final Claude re-review after FIX round 3 of 3. Do not
trust the Codex implementation narrative or supporting audits. Read every
changed implementation file, including the new CLI integration test and the
Telegram abort callback, the real diff, and the actual tests. Do not edit files.

This bridge fingerprints repository writes. Do not run Cargo with the repository
`target/` directory. If rerunning a Rust command is essential, prefix it with a
fresh external target such as
`CARGO_TARGET_DIR=/tmp/omega-phase1-claude-review-target`. Do not run installers,
login flows, live auth probes, or process signals. The controller already reran
the full test matrix; source review and safe read-only commands are sufficient.

Current independent evidence:

- `git diff --check`: PASS
- `cargo check -p omega`: PASS
- `cargo test -p omega`: 7 unit + 1 CLI integration PASS
- `cargo test -p omega-core`: 376 PASS
- focused `codex_login`: 26 PASS
- focused `credentials`: 21 PASS
- `tools/duo/bin/omega-duo --self-test`: 37 PASS
- `bash -n install.sh scripts/verify-install.sh`: PASS
- Bun production builds for `omega-duo` and `omega-tg-bot.ts`: PASS
- `bun test tools/duo`: no matching test files, expected exit 1
- `scripts/verify-install.sh`: every functional check PASS; only the expected
  pre-commit dirty-worktree gate is red
- independent Round 3 bridge and abort re-audits: PASS

Return exactly one verdict, `PASS` or `FIX`, followed by concrete findings with
`file:line` citations.

Review priorities:

1. Could any successful or abandoned login overwrite the freshest valid
   credential with an older backup?
2. Does atomic Codex login replacing a symlink end with one canonical credential
   and the correct native symlink under alternate `OMEGA_DIR`/`CODEX_HOME`?
3. Can a shallow status from another Codex process falsely settle a pending
   flow, kill the wrong PID, or remove the only recovery copy?
4. Are locking, PID identity, permissions, fsync/rename, quarantine, timeout,
   and JSON validation correct on Linux and non-Linux builds?
5. Does startup reconciliation ever mutate credentials during an active device
   flow?
6. Does the bridge truly use stdin and detect worktree/index/ignored mutations
   without modifying the user's repository? Does degraded mode weaken the
   security promise too much?
7. Can task-echo content fake quota or authentication classification?
8. Did the implementation touch only allowed files and update install parity?
9. The implementation is roughly 4,700 added lines. Identify duplicated,
   speculative, or test-only complexity that should be removed before shipping.
   Prefer the smallest design that proves the invariants.
10. Flag missing integration tests and any test that mutates the real HOME,
    credentials, running sessions, or repository.
11. Verify `codex-login-abort` JSON/exit semantics are truthful and the PID
    signalling window cannot target an unrelated process.
12. Verify native Claude/Codex preflight can degrade only on positive evidence,
    while strict degraded mode still fails closed without a live watcher.
13. Verify credential tests and the real CLI subprocess test isolate `HOME`,
    `OMEGA_DIR`, and `CODEX_HOME`, and cannot touch a developer's live auth.
14. Verify Telegram Cancel invokes the PID-bound abort command rather than the
    observation-only status command and truthfully renders abort provenance.
15. Verify generic model prose cannot authorize degradation, while the exact
    local Bubblewrap bootstrap signature can, and guard failures remain
    structured in the public JSON result.

If the verdict is FIX, give a surgical repair list suitable for one Codex fix
turn. Do not recommend weakening the required safety invariants.

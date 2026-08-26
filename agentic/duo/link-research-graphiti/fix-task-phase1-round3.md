# /duo FIX Task, Phase 1, Round 3 of 3

Read completely:

- `agentic/duo/link-research-graphiti/plan.md`, Phase 1
- `agentic/duo/link-research-graphiti/code-task-phase1.md`
- `agentic/duo/link-research-graphiti/fix-task-phase1.md`
- `agentic/duo/link-research-graphiti/fix-task-phase1-round2.md`
- the current real diff

The final Claude review was rejected by the bridge because Claude wrote ignored
Cargo artifacts under the repository target directory. No tracked source was
changed by that review. Its concrete findings below are valid repair input.

This is the final allowed FIX turn for this Phase 1 failure chain. Make only
the surgical repairs below. Preserve all already-proven authentication,
process-identity, monotonic-reconciliation, and read-guard invariants.

## Mandatory repairs

1. In every credential test that changes `HOME` or `OMEGA_DIR`, also pin
   `CODEX_HOME` to a path inside the same temporary directory. In
   `fresh_store`, assert that the resolved native Codex credential path is
   inside that temporary `CODEX_HOME`. A developer's real `CODEX_HOME` must
   never be read, linked, replaced, or removed by a test.
2. Repoint Telegram's device-login Cancel button to a distinct
   `acct:codexabort:<pid>` callback. The handler must invoke
   `omega codex-login-abort --pid <pid>` and truthfully render the returned
   `ok`, `aborted`, `restored`, and `status` fields. It must not reuse the
   observation-only status command.
3. Remove the unused `DUO_BWRAP_BIN` self-test decoration and
   `missingBwrap`. Rename the attempt-zero invariant to describe the behavior
   actually tested: native read-only argv. Update the installer verifier's
   named invariant accordingly.
4. Tighten positive sandbox-denial detection. Generic model prose such as
   "could not read the repository" must not authorize an unsafe degraded retry.
   Require concrete command/probe/tool/call denial plus a sandbox,
   permission, approval, or plan-mode signal. The exact local Bubblewrap
   infrastructure failure `bwrap: loopback: Failed RTM_NEWADDR: Operation not
   permitted` is also positive evidence and may authorize the guarded retry.
   Keep all transport/provider vetoes. Update the fake native denial text and
   deterministic cases.
5. Make `runGuarded` return structured `guardError` JSON if Git metadata roots
   cannot be resolved. Do not let that exception escape. Allocate live
   filesystem watchers only for strict degraded runs; native read-only runs
   still use the before/after Git-observable fingerprint.
6. Add a Unix CLI integration test for
   `omega codex-reconcile --json` using isolated `HOME`, `OMEGA_DIR`, and
   `CODEX_HOME`. It must run the real binary, parse the JSON contract, and
   verify the native path is a symlink to the isolated canonical credential.
   Use only a uniquely-owned temporary path; do not touch live credentials.
7. Update install verification for the Telegram abort callback and every new
   named runtime invariant. Do not add a dependency if the integration test can
   use existing workspace dependencies safely.

## Allowed implementation files

- `crates/omega-core/src/credentials.rs`
- `crates/omega-cli/src/main.rs`
- one focused integration test under `crates/omega-cli/tests/`
- `telegram-bot/omega-tg-bot.ts`
- `tools/duo/bin/omega-duo`
- `scripts/verify-install.sh`
- `skills/duo/SKILL.md` only if its behavior statement becomes inaccurate

Do not touch any other implementation file. Do not run an installer, live
login, live reconciliation, credential mutation, external process signal,
commit, or push.

## Required verification

Use an external Cargo target directory for every Rust command:

```bash
git diff --check
CARGO_TARGET_DIR=/tmp/omega-phase1-fix-round3-target cargo check -p omega
CARGO_TARGET_DIR=/tmp/omega-phase1-fix-round3-target cargo test -p omega-core credentials
CARGO_TARGET_DIR=/tmp/omega-phase1-fix-round3-target cargo test -p omega
tools/duo/bin/omega-duo --self-test
bash -n install.sh scripts/verify-install.sh
```

`bun test tools/duo` may legitimately find no test files. Inspect the real diff
and confirm no source outside the allowed set changed.

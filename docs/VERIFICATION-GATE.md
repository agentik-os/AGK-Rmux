# OmegaOS verification gate protocol

> Code and documentation are claims. A gate accepts only fresh evidence from
> the exact revision and runtime under review.

This checklist covers the current non-gateway OmegaOS release unit. Release
publication and rollback are documented in [RELEASE.md](RELEASE.md).

## 1. Source gate

Run the checked-in CI boundary with the lockfile:

```bash
cargo fmt -p omega-core -p omega-tui -p omega -- --check
cargo clippy --locked -p omega-core -p omega-tui -p omega --all-targets -- -D warnings
RUSTFLAGS="-D warnings" cargo build --release --locked -p omega-core -p omega-tui -p omega
cargo test --locked -p omega-core -p omega-tui -p omega
bash scripts/check-workflows.sh
bash scripts/tests/test_audit_runner.sh
python3 scripts/tests/test_hook_plan_state.py
python3 scripts/tests/test_skill_catalog_scripts.py
python3 scripts/tests/test_release_contract.py
cargo run --locked --bin omega -- skills validate --root skills
(cd installer && npm test)
```

Any nonzero exit is a failure. Preserve the failing output and repair the cause;
do not discard unrelated work or convert a warning into a pass.

## 2. Architecture and state gate

- Orchestration sessions use the typed rmux SDK, not tmux text scraping.
- `~/.omega/state/mission-engine-v3.sqlite3` is the only mission write
  authority. JSON, Telegram cards, timelines, and task lists are projections.
- Compatibility projections carry ledger version, event, and hash provenance;
  forged or stale projections fail closed.
- Worker completion is a candidate until its exact task attempt and independent
  evidence are accepted in the ledger.
- File scope and worktree leases use generation/fencing identity; a stale owner
  cannot release a newer claim.
- Telegram routing and bot state use the Bun/TypeScript service and typed core
  registries. Secrets remain in user-owned configuration with restrictive
  permissions.
- Configuration and state paths derive from OmegaOS configuration, the user
  home, or explicit overrides. No operator-specific absolute path belongs in
  runtime code.

## 3. Runtime inventory gate

Run the freshly built binary, not an older installed copy:

```bash
cargo run --locked --bin omega -- --version
cargo run --locked --bin omega -- --help
cargo run --locked --bin omega -- rules list
cargo run --locked --bin omega -- audit list
cargo run --locked --bin omega -- doctor
```

Verify that help and documentation agree with the runtime. `doctor` warnings
remain warnings until fixed; binary-provenance drift is expected before install
but must be green after the install gate.

## 4. Mission lifecycle gate

Exercise a disposable mission or test fixture and capture evidence for each
transition:

1. create/classify the mission and persist its immutable identity;
2. persist a typed plan with explicit acceptance and verification checks;
3. dispatch a task attempt with exact project, worktree, and scope identity;
4. submit a completion candidate;
5. independently verify the candidate and record the verdict;
6. reject closure while any required task or worker remains nonterminal;
7. close delivery once, then replay/retry it to prove idempotency;
8. restart and resume from the ledger, not from transcript memory.

Test a negative path as well: stale scope generation, stale plan revision,
forged projection, failed verifier, or unavailable delivery must not be accepted.

## 5. Installed-runtime gate

After `./install.sh`, verify install parity and actual operator paths:

```bash
./scripts/verify-install.sh
omega -V
omega doctor --deep
omega rules list
omega audit list
omega list
systemctl --user status omega-tg-bot.service
```

Then exercise the TUI tabs and input, a disposable CLI dispatch, and configured
Telegram topic routing. Authentication or delivery paths that are not
configured are unverified, not passing.

## 6. Release acceptance

Only a clean, pushed revision with successful blocking CI can be tagged. Verify
checksums, `BUILD-INFO.json`, SPDX SBOMs, GitHub attestations, and a source
install of the exact tag as described in [RELEASE.md](RELEASE.md). A worker or
oracle self-report never substitutes for these checks.

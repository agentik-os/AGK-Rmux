# Release and rollback runbook

This runbook follows the checked-in CI and release workflows. It does not treat
a local build, a pushed tag, or an agent report as proof that a release exists.

## Safety boundary

As verified on 2026-08-11, GitHub returned `404 Branch not protected` for the
main-branch protection endpoint. Protection is repository state outside this
codebase and can change independently, so check it before every release:

```bash
gh api repos/agentik-os/OmegaOS/branches/main/protection
```

A successful response describes the active protection. A 404 means main is
unprotected. In that state, the operator must manually require the complete CI
run before tagging; workflow files alone do not prevent a direct push.

Never move, delete, or force-push a published tag. Never release from a dirty
checkout or a commit that is not already on `origin/main`.

## Release unit and gates

The current blocking Rust release unit is `omega-core`, `omega-tui`, and the
`omega` CLI. The separately maintained gateway is excluded by the checked-in CI
workflow. Before creating a tag, use a clean checkout of the exact candidate
commit and run the same local gates:

```bash
git fetch origin
git status --short
git rev-parse HEAD
git rev-parse origin/main

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

Confirm that the workspace package version, release notes, and proposed tag are
the same version. The release workflow rejects any tag other than the exact
`v<workspace package version>`.

Push the candidate commit normally, then wait for the main-branch CI run:

```bash
git push origin main
gh run list --workflow ci.yml --branch main --limit 5
gh run watch <ci-run-id> --exit-status
```

Do not tag a failing, cancelled, missing, or still-running CI revision.

## Publish

Create an annotated tag on the already-pushed commit and push that tag without
force:

```bash
git tag -a vX.Y.Z -m "OmegaOS vX.Y.Z"
git push origin vX.Y.Z
```

The tag push is the only release trigger. The workflow validates tag/version
identity, calls the blocking CI workflow, and builds four target archives:

- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `aarch64-apple-darwin`
- `x86_64-apple-darwin`

Each published target has an archive, archive checksum, SPDX JSON SBOM, and
SBOM checksum. Each archive contains exactly `omega`, `rmux`, and
`BUILD-INFO.json`; GitHub attestations bind archive provenance and SBOM data.
The publisher first creates a draft and makes it public only after the complete
matrix and provenance checks pass.

## Verify the published release

```bash
gh run list --workflow release.yml --limit 5
gh run watch <release-run-id> --exit-status
gh release view vX.Y.Z --json tagName,isDraft,isPrerelease,assets,targetCommitish

release_verify_dir=$(mktemp -d)
gh release download vX.Y.Z --dir "$release_verify_dir"
(cd "$release_verify_dir" && sha256sum -c *.sha256)
tar -tzf "$release_verify_dir/omega-x86_64-unknown-linux-gnu.tar.gz"
gh attestation verify "$release_verify_dir/omega-x86_64-unknown-linux-gnu.tar.gz" --repo agentik-os/OmegaOS
```

Repeat archive inspection and attestation verification for every target. Check
the extracted `BUILD-INFO.json`: `omega_commit` must equal the tagged commit,
`target` must match the archive name, `cargo_lock_sha256` must match the tagged
lockfile, and `rmux_commit` must be a full locked revision.

On a disposable or designated verification host, install the exact tag from
source and exercise the operator paths:

```bash
release_smoke_dir=$(mktemp -d)
git clone --branch vX.Y.Z --depth 1 https://github.com/agentik-os/OmegaOS "$release_smoke_dir/OmegaOS"
(cd "$release_smoke_dir/OmegaOS" && OMEGA_FROM_SOURCE=1 ./install.sh)
omega -V
omega doctor
omega rules list
omega audit list
omega --help
```

Authentication-dependent and Telegram checks must run only on a host configured
for them. A missing credential or non-running service is a negative state, not
a release pass.

## Roll back to a known-good release

Rollback changes the installed binaries and assets; preserve user state and any
dirty source checkout before starting. Do not retag the bad commit and do not
move an existing tag.

1. Select an immutable known-good tag whose release workflow and assets are
   still verifiably successful:

   ```bash
   gh release list --limit 20
   gh run list --workflow release.yml --limit 20
   gh release view vKNOWN.GOOD.PATCH --json tagName,isDraft,isPrerelease,assets
   ```

2. Install that exact tag from a fresh temporary clone. Force the source path
   so rollback does not accidentally choose a different latest prebuilt tag:

   ```bash
   rollback_dir=$(mktemp -d)
   git clone --branch vKNOWN.GOOD.PATCH --depth 1 https://github.com/agentik-os/OmegaOS "$rollback_dir/OmegaOS"
   (cd "$rollback_dir/OmegaOS" && OMEGA_FROM_SOURCE=1 ./install.sh)
   ```

   The installer preserves `~/.omega` user state. Do not reset, clean, or check
   out over an existing working tree as part of rollback.

3. Verify the installed version, provenance, doctrine, daemon, and the actual
   operator golden path:

   ```bash
   omega -V
   omega doctor --deep
   omega rules list
   omega list
   ```

4. Repair main with a new normal commit. After the complete CI gate passes,
   publish a new patch version and a new immutable tag. The rollback tag remains
   unchanged as the audit trail.

If no historical release has verifiable assets or the source build fails, stop
and record the blocker. Do not substitute an unverified local binary.

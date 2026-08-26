#!/usr/bin/env python3
"""Static falsification tests for install, CI, and release provenance contracts."""

from pathlib import Path
import hashlib
import io
import json
import re
import subprocess
import sys
import tarfile
import tempfile
import textwrap
import tomllib
import unittest


ROOT = Path(__file__).resolve().parents[2]
WORKFLOW_DIR = ROOT / ".github" / "workflows"
class ReleaseContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.install = ROOT.joinpath("install.sh").read_text(encoding="utf-8")
        cls.workflows = {
            path.name: path.read_text(encoding="utf-8")
            for path in sorted(WORKFLOW_DIR.glob("*.yml"))
        }
        cls.release = cls.workflows["release.yml"]
        cls.ci = cls.workflows["ci.yml"]
        cls.publisher = cls.workflows["publish-installer.yml"]

    @staticmethod
    def heredocs(text, language):
        return [
            textwrap.dedent(body)
            for body in re.findall(
                rf"<<'{language}'\n(.*?)\n\s*{language}(?:\n|$)",
                text,
                flags=re.DOTALL,
            )
        ]

    def test_prebuilt_requires_checksum_and_build_info(self):
        self.assertIn('if ! curl -fsSL "$base/$tarball.sha256"', self.install)
        self.assertIn('-f "$tmp/BUILD-INFO.json"', self.install)
        self.assertIn('-f "$tmp/omega-gatewayd"', self.install)
        self.assertNotIn("if the .sha256 sidecar exists", self.install)

    def test_source_build_never_falls_back_to_unlocked_dependencies(self):
        self.assertIn("cargo_build_live --locked", self.install)
        self.assertNotIn("cargo_build_live --locked || cargo_build_live", self.install)

    def test_install_records_the_same_revision_that_doctor_reads(self):
        self.assertIn("record_install_provenance()", self.install)
        self.assertIn('local state_path="$OMEGA_DIR/state/auto-update.json"', self.install)
        self.assertIn('.last_applied_commit = $commit', self.install)
        self.assertIn('--arg commit "${OMEGA_SOURCE_REV:0:7}"', self.install)
        self.assertIn('record_install_provenance\n', self.install)
        self.assertIn('mktemp "$OMEGA_DIR/state/.auto-update.json.XXXXXX"', self.install)

    def test_every_external_action_is_pinned_to_an_immutable_commit(self):
        found = 0
        for workflow, text in self.workflows.items():
            for line_number, line in enumerate(text.splitlines(), start=1):
                match = re.match(r"\s*(?:-\s+)?uses:\s+([^\s#]+)", line)
                if match is None or match.group(1).startswith("./"):
                    continue
                found += 1
                action, separator, revision = match.group(1).rpartition("@")
                self.assertTrue(action and separator, f"{workflow}:{line_number}")
                self.assertRegex(
                    revision,
                    r"^[0-9a-f]{40}$",
                    f"mutable action reference at {workflow}:{line_number}",
                )
        self.assertGreaterEqual(found, 10)

    def test_checkouts_never_persist_release_credentials(self):
        for workflow, text in self.workflows.items():
            self.assertEqual(
                text.count("actions/checkout@"),
                text.count("persist-credentials: false"),
                f"checkout credential persistence in {workflow}",
            )

    def test_full_workspace_quality_gate_is_blocking_locked_and_reusable(self):
        for command in (
            "cargo fmt --all -- --check",
            "cargo clippy --locked --workspace --all-targets -- -D warnings",
            "cargo build --release --locked --workspace",
            "cargo test --locked --workspace",
        ):
            self.assertIn(command, self.ci)
        self.assertNotIn("non-gateway", self.ci)
        self.assertNotIn("-not -path './crates/omega-gateway/*'", self.ci)
        self.assertNotIn("continue-on-error", self.ci)
        self.assertIn("workflow_call:", self.ci)
        self.assertIn("uses: ./.github/workflows/ci.yml", self.release)

    def test_toolchains_runners_and_sbom_generator_are_version_locked(self):
        combined = "\n".join(self.workflows.values())
        self.assertNotIn("ubuntu-latest", combined)
        self.assertNotIn("rust-toolchain@stable", combined)
        self.assertGreaterEqual(combined.count("toolchain: 1.97.1"), 2)
        self.assertGreaterEqual(combined.count("node-version: 24.19.0"), 2)
        self.assertIn("syft-version: v1.51.0", self.release)

    def test_release_tag_must_exactly_match_the_workspace_version(self):
        self.assertNotIn("workflow_dispatch:", self.release)
        self.assertIn('test "$GITHUB_REF_TYPE" = "tag"', self.release)
        self.assertIn('test "$GITHUB_REF_NAME" = "v$version"', self.release)
        self.assertIn(
            'test "$(git rev-parse HEAD)" = "$(git rev-parse "$GITHUB_SHA^{commit}")"',
            self.release,
        )

    def test_rmux_build_uses_the_exact_full_revision_from_cargo_lock(self):
        self.assertIn("with open('Cargo.lock', 'rb')", self.release)
        self.assertIn("re.fullmatch(r'[0-9a-f]{40}', revision)", self.release)
        self.assertIn('rev-parse HEAD)" = "${{ steps.rmux.outputs.rev }}"', self.release)
        self.assertIn("cargo build --release --locked --manifest-path", self.release)
        self.assertNotIn("rev checkout failed", self.release)

    def test_release_is_deterministic_checksummed_attested_and_complete(self):
        self.assertIn("gzip.GzipFile(filename='', mode='wb', fileobj=raw, mtime=0)", self.release)
        self.assertIn("BUILD-INFO.json", self.release)
        self.assertIn(".spdx.json.sha256", self.release)
        self.assertGreaterEqual(self.release.count("actions/attest@"), 2)
        self.assertIn("actions/upload-artifact@", self.release)
        self.assertIn("actions/download-artifact@", self.release)
        for target in (
            "x86_64-unknown-linux-gnu",
            "aarch64-unknown-linux-gnu",
            "aarch64-apple-darwin",
            "x86_64-apple-darwin",
        ):
            self.assertGreaterEqual(self.release.count(target), 2)
        self.assertIn("release asset mismatch", self.release)
        self.assertIn("checksum mismatch", self.release)

    def test_release_permissions_and_publication_are_least_privilege_and_serial(self):
        self.assertIn("permissions: {}", self.release)
        self.assertEqual(self.release.count("contents: write"), 1)
        self.assertEqual(self.release.count("id-token: write"), 1)
        self.assertEqual(self.release.count("attestations: write"), 1)
        self.assertIn("cancel-in-progress: false", self.release)
        self.assertIn("needs: build", self.release)
        self.assertIn("Upload assets to a non-public draft", self.release)
        self.assertIn("draft: true", self.release)
        self.assertIn("Publish the fully populated draft", self.release)
        self.assertIn("overwrite_files: false", self.release)

    def test_npm_publisher_is_main_only_fail_closed_and_provenanced(self):
        self.assertIn("github.repository == 'agentik-os/OmegaOS'", self.publisher)
        self.assertIn("github.ref == 'refs/heads/main'", self.publisher)
        self.assertIn("id-token: write", self.publisher)
        self.assertIn("PUBLISHED=$(npm view omega-os version)", self.publisher)
        self.assertNotIn("npm view omega-os version --json", self.publisher)
        self.assertNotIn("|| echo", self.publisher)
        self.assertIn("refusing to publish", self.publisher)
        self.assertIn("npm publish --access public --provenance", self.publisher)
        self.assertIn("cancel-in-progress: false", self.publisher)

    def test_npm_semver_guard_accepts_only_a_strictly_newer_stable_version(self):
        (guard,) = self.heredocs(self.publisher, "NODE")
        for local, published, expected in (
            ("1.5.13", "1.5.12", 0),
            ("1.5.12", "1.5.12", 10),
            ("1.5.11", "1.5.12", 2),
            ("1.6.0-rc.1", "1.5.12", 1),
            ("1.5.13", '"1.5.12"', 1),
        ):
            result = subprocess.run(
                ["node", "-", local, published],
                input=guard,
                text=True,
                capture_output=True,
                check=False,
            )
            self.assertEqual(result.returncode, expected, result.stderr)

    def test_archive_builder_is_deterministic_and_normalizes_metadata(self):
        archive_builder = next(
            body
            for body in self.heredocs(self.release, "PY")
            if "gzip.GzipFile" in body
        )
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            dist = root / "dist"
            dist.mkdir()
            (dist / "omega").write_bytes(b"omega-binary")
            (dist / "omega-gatewayd").write_bytes(b"gateway-binary")
            (dist / "rmux").write_bytes(b"rmux-binary")
            (dist / "BUILD-INFO.json").write_text("{}\n", encoding="utf-8")
            command = [sys.executable, "-", "x86_64-unknown-linux-gnu"]
            first = subprocess.run(
                command,
                input=archive_builder,
                text=True,
                cwd=root,
                capture_output=True,
                check=False,
            )
            self.assertEqual(first.returncode, 0, first.stderr)
            archive_path = dist / "omega-x86_64-unknown-linux-gnu.tar.gz"
            before = archive_path.read_bytes()
            second = subprocess.run(
                command,
                input=archive_builder,
                text=True,
                cwd=root,
                capture_output=True,
                check=False,
            )
            self.assertEqual(second.returncode, 0, second.stderr)
            self.assertEqual(archive_path.read_bytes(), before)
            with tarfile.open(fileobj=io.BytesIO(before), mode="r:gz") as bundle:
                self.assertEqual(
                    bundle.getnames(),
                    ["omega", "omega-gatewayd", "rmux", "BUILD-INFO.json"],
                )
                metadata = {
                    item.name: (item.uid, item.gid, item.mtime, item.mode)
                    for item in bundle.getmembers()
                }
            self.assertEqual(
                metadata,
                {
                    "omega": (0, 0, 0, 0o755),
                    "omega-gatewayd": (0, 0, 0, 0o755),
                    "rmux": (0, 0, 0, 0o755),
                    "BUILD-INFO.json": (0, 0, 0, 0o644),
                },
            )

    def test_publisher_verifier_accepts_complete_assets_and_rejects_tampering(self):
        verifier = next(
            body
            for body in self.heredocs(self.release, "PY")
            if "release asset mismatch" in body
        )
        commit = "a" * 40
        targets = {
            "x86_64-unknown-linux-gnu",
            "aarch64-unknown-linux-gnu",
            "aarch64-apple-darwin",
            "x86_64-apple-darwin",
        }
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            lock = ROOT.joinpath("Cargo.lock").read_bytes()
            (root / "Cargo.lock").write_bytes(lock)
            lock_digest = hashlib.sha256(lock).hexdigest()
            packages = tomllib.loads(lock.decode())["package"]
            rmux_revisions = {
                package["source"].rsplit("#", 1)[1]
                for package in packages
                if package["name"] == "rmux-sdk"
                and package.get("source", "").startswith(
                    "git+https://github.com/agentik-os/rmux?"
                )
            }
            self.assertEqual(len(rmux_revisions), 1)
            rmux_commit = rmux_revisions.pop()
            dist = root / "dist"
            dist.mkdir()
            for target in targets:
                archive_path = dist / f"omega-{target}.tar.gz"
                build_info = json.dumps(
                    {
                        "schema_version": 1,
                        "omega_commit": commit,
                        "rmux_commit": rmux_commit,
                        "cargo_lock_sha256": lock_digest,
                        "target": target,
                    }
                ).encode()
                with tarfile.open(archive_path, "w:gz") as bundle:
                    for name, data in (
                        ("omega", b"omega"),
                        ("omega-gatewayd", b"gateway"),
                        ("rmux", b"rmux"),
                        ("BUILD-INFO.json", build_info),
                    ):
                        member = tarfile.TarInfo(name)
                        member.size = len(data)
                        bundle.addfile(member, io.BytesIO(data))
                sbom_path = dist / f"omega-{target}.spdx.json"
                sbom_path.write_text("{}", encoding="utf-8")
                for path in (archive_path, sbom_path):
                    digest = hashlib.sha256(path.read_bytes()).hexdigest()
                    (dist / f"{path.name}.sha256").write_text(
                        f"{digest}  {path.name}\n", encoding="utf-8"
                    )

            command = [sys.executable, "-", commit]
            valid = subprocess.run(
                command,
                input=verifier,
                text=True,
                cwd=root,
                capture_output=True,
                check=False,
            )
            self.assertEqual(valid.returncode, 0, valid.stderr)
            victim = dist / "omega-x86_64-unknown-linux-gnu.spdx.json"
            victim.write_text("tampered", encoding="utf-8")
            invalid = subprocess.run(
                command,
                input=verifier,
                text=True,
                cwd=root,
                capture_output=True,
                check=False,
            )
            self.assertNotEqual(invalid.returncode, 0)
            self.assertIn("checksum mismatch", invalid.stderr)

            victim.write_text("{}", encoding="utf-8")
            digest = hashlib.sha256(victim.read_bytes()).hexdigest()
            (dist / f"{victim.name}.sha256").write_text(
                f"{digest}  {victim.name}\n", encoding="utf-8"
            )
            archive_path = dist / "omega-x86_64-unknown-linux-gnu.tar.gz"
            with tarfile.open(archive_path, "w:gz") as bundle:
                forged = json.dumps(
                    {
                        "schema_version": 1,
                        "omega_commit": commit,
                        "rmux_commit": "b" * 40,
                        "cargo_lock_sha256": lock_digest,
                        "target": "x86_64-unknown-linux-gnu",
                    }
                ).encode()
                for name, data in (
                    ("omega", b"omega"),
                    ("omega-gatewayd", b"gateway"),
                    ("rmux", b"rmux"),
                    ("BUILD-INFO.json", forged),
                ):
                    member = tarfile.TarInfo(name)
                    member.size = len(data)
                    bundle.addfile(member, io.BytesIO(data))
            archive_digest = hashlib.sha256(archive_path.read_bytes()).hexdigest()
            (dist / f"{archive_path.name}.sha256").write_text(
                f"{archive_digest}  {archive_path.name}\n", encoding="utf-8"
            )
            forged_result = subprocess.run(
                command,
                input=verifier,
                text=True,
                cwd=root,
                capture_output=True,
                check=False,
            )
            self.assertNotEqual(forged_result.returncode, 0)
            self.assertIn("rmux provenance mismatch", forged_result.stderr)

    def test_failure_paths_are_bounded(self):
        for workflow, text in self.workflows.items():
            self.assertIn("timeout-minutes:", text, f"unbounded job in {workflow}")
        self.assertIn("if-no-files-found: error", self.release)
        self.assertIn("fail_on_unmatched_files: true", self.release)
        self.assertNotIn("continue-on-error", "\n".join(self.workflows.values()))


if __name__ == "__main__":
    unittest.main()

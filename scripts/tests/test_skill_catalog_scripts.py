#!/usr/bin/env python3
"""Contract tests for the SkillCatalogV1 Atlas/RAG projections."""

import json
import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest


REPO = Path(__file__).resolve().parents[2]
ATLAS_SCRIPT = REPO / "scripts/omega-skills-atlas.py"
RAG_SCRIPT = REPO / "scripts/omega-skills-rag.py"
CLI_SCRIPT = REPO / "scripts/omega-skills"


def catalog(_digest, skills):
    return {
        "schema_version": 1,
        "content_digest": catalog_digest(skills),
        "skills": skills,
        "warnings": [],
    }


def catalog_digest(skills):
    payload = json.dumps([1, skills], ensure_ascii=False, separators=(",", ":")).encode()
    return hashlib.sha256(payload).hexdigest()


def source_digest(root_id, relative_path, content):
    normalized = content.replace("\r\n", "\n").encode()
    rows = [[root_id, relative_path, hashlib.sha256(normalized).hexdigest()]]
    payload = json.dumps(rows, ensure_ascii=False, separators=(",", ":")).encode()
    return hashlib.sha256(payload).hexdigest()


def skill(name, description, relative_path, category="Custom"):
    return {
        "name": name,
        "description": description,
        "root_id": "omegaos",
        "relative_path": relative_path,
        "content_digest": "f" * 64,
        "aliases": [],
        "triggers": [name],
        "category": category,
        "phases": None,
        "max_score": None,
        "read_only": False,
        "dependencies": {},
        "provider_states": {
            "claude": {"state": "enabled"},
            "codex": {"state": "enabled"},
        },
    }


class SkillCatalogScriptTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.omega = Path(self.temp.name)
        self.catalog_path = self.omega / "skill-catalog-v1.json"
        self.env = os.environ.copy()
        self.env.update({
            "OMEGA_DIR": str(self.omega),
            "OMEGA_SKILL_CATALOG": str(self.catalog_path),
            "OPENAI_API_KEY": "",
        })

    def tearDown(self):
        self.temp.cleanup()

    def write_catalog(self, value):
        self.catalog_path.write_text(json.dumps(value), encoding="utf-8")

    def run_script(self, script, *args, check=True):
        return subprocess.run(
            [sys.executable, str(script), *args],
            env=self.env,
            text=True,
            capture_output=True,
            check=check,
        )

    def test_atlas_consumes_canonical_catalog_and_keeps_nested_families(self):
        self.write_catalog(catalog("a" * 64, [
            skill(
                "high-end-visual-design",
                "Premium UI",
                "design/high-end-visual-design/SKILL.md",
                "Design",
            ),
            skill(
                "caio-ai-readiness-assessment",
                "CAIO readiness",
                "caio/caio-ai-readiness-assessment/SKILL.md",
            ),
            skill(
                "marketing-master",
                "Marketing mastery",
                "marketing-mastery/marketing-master/SKILL.md",
                "Marketing",
            ),
        ]))
        self.run_script(ATLAS_SCRIPT)
        atlas = json.loads(
            (self.omega / "skills-atlas.json").read_text(encoding="utf-8"))
        self.assertEqual(atlas["catalog_hash"], catalog_digest([
            skill(
                "high-end-visual-design", "Premium UI",
                "design/high-end-visual-design/SKILL.md", "Design"),
            skill(
                "caio-ai-readiness-assessment", "CAIO readiness",
                "caio/caio-ai-readiness-assessment/SKILL.md"),
            skill(
                "marketing-master", "Marketing mastery",
                "marketing-mastery/marketing-master/SKILL.md", "Marketing"),
        ]))
        self.assertEqual(atlas["native_count"], 3)
        self.assertEqual(
            {row["name"] for row in atlas["native"]},
            {
                "high-end-visual-design",
                "caio-ai-readiness-assessment",
                "marketing-master",
            },
        )

    def test_legacy_fallback_is_recursive_and_excludes_vendor(self):
        self.catalog_path.unlink(missing_ok=True)
        wanted = self.omega / "skills/design/deep/high-end-visual-design"
        vendor = self.omega / "skills/pdfgen/node_modules/playwright/skill"
        wanted.mkdir(parents=True)
        vendor.mkdir(parents=True)
        wanted.joinpath("SKILL.md").write_text(
            "---\nname: high-end-visual-design\n"
            "description: Premium UI\n---\n",
            encoding="utf-8",
        )
        vendor.joinpath("SKILL.md").write_text(
            "---\nname: vendor-playwright\n"
            "description: Excluded\n---\n",
            encoding="utf-8",
        )
        self.run_script(ATLAS_SCRIPT)
        atlas = json.loads(
            (self.omega / "skills-atlas.json").read_text(encoding="utf-8"))
        self.assertEqual(atlas["native_count"], 1)
        self.assertEqual(atlas["native"][0]["name"], "high-end-visual-design")

    def test_existing_invalid_canonical_catalog_never_falls_back(self):
        legacy = self.omega / "skills/legacy"
        legacy.mkdir(parents=True)
        legacy.joinpath("SKILL.md").write_text(
            "---\nname: legacy\ndescription: Must not mask corruption\n---\n",
            encoding="utf-8",
        )
        invalid = catalog("ignored", [
            skill("canonical", "Broken", "canonical/SKILL.md"),
        ])
        invalid["content_digest"] = "not-a-sha256"
        self.write_catalog(invalid)

        result = self.run_script(ATLAS_SCRIPT, check=False)

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("canonical catalog invalid", result.stderr)
        self.assertFalse((self.omega / "skills-atlas.json").exists())

    def test_canonical_catalog_rejects_unsafe_paths_and_digests(self):
        invalid_cases = (
            ("short catalog digest", {
                **catalog("ignored", [skill("alpha", "Alpha", "alpha/SKILL.md")]),
                "content_digest": "a" * 32,
            }),
            ("non-hex skill digest", catalog("a" * 64, [{
                **skill("alpha", "Alpha", "alpha/SKILL.md"),
                "content_digest": "z" * 64,
            }])),
            ("parent traversal", catalog("a" * 64, [
                skill("alpha", "Alpha", "../outside/SKILL.md"),
            ])),
            ("absolute path", catalog("a" * 64, [
                skill("alpha", "Alpha", "/outside/SKILL.md"),
            ])),
            ("catalog payload digest mismatch", {
                **catalog("ignored", [skill("alpha", "Alpha", "alpha/SKILL.md")]),
                "skills": [skill("alpha", "Mutated", "alpha/SKILL.md")],
            }),
        )
        for label, value in invalid_cases:
            with self.subTest(label=label):
                self.write_catalog(value)
                result = self.run_script(ATLAS_SCRIPT, check=False)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("canonical catalog invalid", result.stderr)
                (self.omega / "skills-atlas.json").unlink(missing_ok=True)

    def test_parallel_atlas_generation_is_atomic(self):
        self.write_catalog(catalog("a" * 64, [
            skill("alpha", "Alpha", "alpha/SKILL.md"),
        ]))
        processes = [
            subprocess.Popen(
                [sys.executable, str(ATLAS_SCRIPT)],
                env=self.env,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )
            for _ in range(24)
        ]
        failures = []
        for process in processes:
            stdout, stderr = process.communicate(timeout=20)
            if process.returncode != 0:
                failures.append((process.returncode, stdout, stderr))
        self.assertEqual(failures, [])
        json.loads((self.omega / "skills-atlas.json").read_text(encoding="utf-8"))
        html = (self.omega / "artifacts/omega-skill-atlas.html").read_text(
            encoding="utf-8")
        self.assertIn("OmegaOS Skill Atlas", html)
        self.assertEqual(list(self.omega.rglob("*.tmp")), [])

    def test_rag_rebuilds_when_catalog_hash_drifts(self):
        self.write_catalog(catalog("a" * 64, [
            skill("alpha", "First description", "alpha/SKILL.md"),
        ]))
        self.run_script(ATLAS_SCRIPT)
        self.run_script(RAG_SCRIPT, "build")
        first = json.loads(
            (self.omega / "skills-rag/meta.json").read_text(encoding="utf-8"))
        self.assertEqual(first["catalog_hash"], catalog_digest([
            skill("alpha", "First description", "alpha/SKILL.md"),
        ]))

        self.write_catalog(catalog("b" * 64, [
            skill("alpha", "Changed description", "alpha/SKILL.md"),
        ]))
        result = self.run_script(RAG_SCRIPT, "query", "changed", "--json")
        json.loads(result.stdout)
        self.assertIn("index drift detected", result.stderr)
        second = json.loads(
            (self.omega / "skills-rag/meta.json").read_text(encoding="utf-8"))
        self.assertEqual(second["catalog_hash"], catalog_digest([
            skill("alpha", "Changed description", "alpha/SKILL.md"),
        ]))
        self.assertNotEqual(first["corpus_hash"], second["corpus_hash"])

    def test_cold_rag_json_query_keeps_diagnostics_off_stdout(self):
        self.write_catalog(catalog("ignored", [
            skill("alpha", "Cold index result", "alpha/SKILL.md"),
        ]))
        self.run_script(ATLAS_SCRIPT)

        result = self.run_script(RAG_SCRIPT, "query", "cold index", "--json")

        payload = json.loads(result.stdout)
        self.assertEqual(payload[0]["name"], "alpha")
        self.assertIn("index drift detected", result.stderr)
        self.assertIn("built BM25 lexical index", result.stderr)

    def test_cli_rebuilds_stale_atlas_before_search(self):
        bin_dir = self.omega / "bin"
        bin_dir.mkdir()
        shutil.copy2(ATLAS_SCRIPT, bin_dir / ATLAS_SCRIPT.name)
        shutil.copy2(RAG_SCRIPT, bin_dir / RAG_SCRIPT.name)
        self.write_catalog(catalog("a" * 64, [
            skill("alpha", "Alpha", "alpha/SKILL.md"),
        ]))
        self.run_script(bin_dir / ATLAS_SCRIPT.name)
        self.write_catalog(catalog("b" * 64, [
            skill(
                "high-end-visual-design",
                "Premium UI",
                "design/high-end-visual-design/SKILL.md",
                "Design",
            ),
        ]))
        result = subprocess.run(
            ["bash", str(CLI_SCRIPT), "high-end-visual-design"],
            env=self.env,
            text=True,
            capture_output=True,
            check=True,
        )
        self.assertIn("high-end-visual-design", result.stdout)
        atlas = json.loads(
            (self.omega / "skills-atlas.json").read_text(encoding="utf-8"))
        self.assertEqual(atlas["catalog_hash"], catalog_digest([
            skill(
                "high-end-visual-design", "Premium UI",
                "design/high-end-visual-design/SKILL.md", "Design"),
        ]))

    def test_cli_rebuilds_an_atlas_with_a_tampered_payload(self):
        bin_dir = self.omega / "bin"
        bin_dir.mkdir()
        shutil.copy2(ATLAS_SCRIPT, bin_dir / ATLAS_SCRIPT.name)
        self.write_catalog(catalog("ignored", [
            skill("alpha", "Authentic description", "alpha/SKILL.md"),
        ]))
        self.run_script(bin_dir / ATLAS_SCRIPT.name)
        atlas_path = self.omega / "skills-atlas.json"
        tampered = json.loads(atlas_path.read_text(encoding="utf-8"))
        tampered["native"][0]["description"] = "Tampered description"
        atlas_path.write_text(json.dumps(tampered), encoding="utf-8")

        result = subprocess.run(
            ["bash", str(CLI_SCRIPT), "alpha"],
            env=self.env,
            text=True,
            capture_output=True,
            check=True,
        )

        self.assertIn("Authentic description", result.stdout)
        self.assertNotIn("Tampered description", result.stdout)

    def test_cli_recompiles_catalog_when_source_tree_changes(self):
        source_root = self.omega / "source-skills"
        skill_dir = source_root / "alpha"
        skill_dir.mkdir(parents=True)
        relative = "alpha/SKILL.md"
        initial = "---\nname: alpha\ndescription: First\n---\n"
        changed = "---\nname: alpha\ndescription: Changed\n---\n"
        skill_dir.joinpath("SKILL.md").write_text(initial, encoding="utf-8")
        value = catalog("a" * 64, [skill("alpha", "First", relative)])
        value["source_roots"] = [{"id": "omegaos", "path": str(source_root)}]
        value["source_tree_digest"] = source_digest("omegaos", relative, initial)
        self.write_catalog(value)

        bin_dir = self.omega / "bin"
        bin_dir.mkdir()
        shutil.copy2(ATLAS_SCRIPT, bin_dir / ATLAS_SCRIPT.name)
        shutil.copy2(RAG_SCRIPT, bin_dir / RAG_SCRIPT.name)
        self.run_script(bin_dir / ATLAS_SCRIPT.name)

        fake_cli = self.omega / "fake-omega.py"
        fake_cli.write_text(
            "#!/usr/bin/env python3\n"
            "import hashlib,json,sys\n"
            "from pathlib import Path\n"
            "out=Path(sys.argv[sys.argv.index('--out')+1])\n"
            "data=json.loads(out.read_text())\n"
            "root=Path(data['source_roots'][0]['path'])\n"
            "raw=(root/'alpha/SKILL.md').read_bytes().replace(b'\\r\\n',b'\\n')\n"
            "digest=hashlib.sha256(raw).hexdigest()\n"
            "rows=[['omegaos','alpha/SKILL.md',digest]]\n"
            "data['skills'][0]['content_digest']=digest\n"
            "data['skills'][0]['description']='Changed'\n"
            "data['source_tree_digest']=hashlib.sha256(json.dumps(rows,separators=(',',':')).encode()).hexdigest()\n"
            "payload=json.dumps([1,data['skills']],ensure_ascii=False,separators=(',',':')).encode()\n"
            "data['content_digest']=hashlib.sha256(payload).hexdigest()\n"
            "out.write_text(json.dumps(data))\n",
            encoding="utf-8",
        )
        fake_cli.chmod(0o755)
        self.env["OMEGA_CLI"] = str(fake_cli)
        skill_dir.joinpath("SKILL.md").write_text(changed, encoding="utf-8")

        result = subprocess.run(
            ["bash", str(CLI_SCRIPT), "alpha"],
            env=self.env,
            text=True,
            capture_output=True,
            check=True,
        )
        self.assertIn("Changed", result.stdout)
        atlas = json.loads(
            (self.omega / "skills-atlas.json").read_text(encoding="utf-8"))
        rebuilt = json.loads(self.catalog_path.read_text(encoding="utf-8"))
        self.assertEqual(atlas["catalog_hash"], rebuilt["content_digest"])
        self.assertEqual(
            atlas["source_tree_digest"], source_digest("omegaos", relative, changed))

    def test_cli_rejects_malformed_source_roots_instead_of_serving_stale_atlas(self):
        bin_dir = self.omega / "bin"
        bin_dir.mkdir()
        shutil.copy2(ATLAS_SCRIPT, bin_dir / ATLAS_SCRIPT.name)
        initial = catalog("a" * 64, [
            skill("alpha", "Alpha", "alpha/SKILL.md"),
        ])
        self.write_catalog(initial)
        self.run_script(bin_dir / ATLAS_SCRIPT.name)

        initial["source_tree_digest"] = "b" * 64
        initial["source_roots"] = [False]
        self.write_catalog(initial)
        result = subprocess.run(
            ["bash", str(CLI_SCRIPT), "alpha"],
            env=self.env,
            text=True,
            capture_output=True,
        )

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("invalid", result.stderr.lower())

    def test_cli_rejects_a_noop_recompile_that_leaves_source_drift(self):
        source_root = self.omega / "source-skills"
        skill_dir = source_root / "alpha"
        skill_dir.mkdir(parents=True)
        relative = "alpha/SKILL.md"
        initial = "---\nname: alpha\ndescription: First\n---\n"
        skill_dir.joinpath("SKILL.md").write_text(initial, encoding="utf-8")
        value = catalog("a" * 64, [skill("alpha", "First", relative)])
        value["source_roots"] = [{"id": "omegaos", "path": str(source_root)}]
        value["source_tree_digest"] = source_digest("omegaos", relative, initial)
        self.write_catalog(value)

        bin_dir = self.omega / "bin"
        bin_dir.mkdir()
        shutil.copy2(ATLAS_SCRIPT, bin_dir / ATLAS_SCRIPT.name)
        self.run_script(bin_dir / ATLAS_SCRIPT.name)
        noop_cli = self.omega / "noop-omega"
        noop_cli.write_text("#!/usr/bin/env bash\nexit 0\n", encoding="utf-8")
        noop_cli.chmod(0o755)
        self.env["OMEGA_CLI"] = str(noop_cli)
        skill_dir.joinpath("SKILL.md").write_text(
            "---\nname: alpha\ndescription: Changed\n---\n", encoding="utf-8")

        result = subprocess.run(
            ["bash", str(CLI_SCRIPT), "alpha"],
            env=self.env,
            text=True,
            capture_output=True,
        )

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("still stale", result.stderr.lower())


if __name__ == "__main__":
    unittest.main()

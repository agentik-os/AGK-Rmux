#!/usr/bin/env python3
"""Regression tests for the deterministic Brainstorm {OS} tooling."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CLI = ROOT / "scripts" / "brainstorm_os.py"
INSTALLER = ROOT / "scripts" / "install_omega_os.py"


class BrainstormOsCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory(prefix="brainstorm-os-test-")
        self.root = Path(self.temp.name)
        self.session = self.root / "session.json"

    def tearDown(self) -> None:
        self.temp.cleanup()

    def run_cli(self, *args: str, expected: int = 0) -> subprocess.CompletedProcess[str]:
        result = subprocess.run(
            [sys.executable, str(CLI), *map(str, args)],
            text=True,
            capture_output=True,
            check=False,
        )
        self.assertEqual(result.returncode, expected, msg=result.stdout + result.stderr)
        return result

    def init_and_frame(self) -> None:
        self.run_cli("init", self.session, "--title", "Conversation-first coach", "--domain", "ai-product", "--depth", "deep")
        self.run_cli(
            "frame", self.session,
            "--idea", "A conversational behavior coach",
            "--desired-change", "Help people recover quickly after a missed action",
            "--actor", "people changing one non-clinical habit",
            "--constraint", "no shame",
            "--non-goal", "maximize chat time",
            "--success-signal", "shorter recovery time",
            "--locked-core", "conversation first",
            "--central-tension", "support versus dependency",
        )

    def test_lifecycle_export_freeze_and_handoff(self) -> None:
        self.init_and_frame()
        idea = self.run_cli("add", self.session, "ideas", "--statement", "Time-bounded recovery coach", "--status", "selected", "--confidence", "medium", "--tag", "inversion").stdout.strip()
        hypothesis = self.run_cli("add", self.session, "hypotheses", "--statement", "A short dialogue changes the next action", "--status", "untested", "--confidence", "low", "--falsifier", "Users enjoy the chat but take no different action", "--relates-to", idea).stdout.strip()
        self.run_cli("add", self.session, "arguments", "--statement", "Conversation can become productive procrastination", "--status", "active", "--confidence", "medium", "--target-id", idea, "--polarity", "con", "--tag", "operations,anti-goal")
        self.run_cli("add", self.session, "tensions", "--statement", "Support versus dependency", "--status", "accepted", "--confidence", "high", "--relates-to", idea)
        self.run_cli("add", self.session, "decisions", "--statement", "Optimize recovery rather than streaks", "--status", "locked", "--confidence", "medium", "--rationale", "Aligns success with resilient behavior", "--revisit-trigger", "Recovery fails to improve", "--relates-to", idea)
        self.run_cli("add", self.session, "experiments", "--statement", "Run a 14-day concierge pilot", "--status", "queued", "--confidence", "medium", "--threshold", "At least 60 percent resume within 48 hours", "--relates-to", hypothesis)
        self.run_cli("checkpoint", self.session, "--name", "Behavior and trust", "--lens", "behavioral", "--delta", "Changed success from perfect streaks to recovery", "--revision", idea)
        self.run_cli("validate", self.session)
        audit = json.loads(self.run_cli("audit", self.session).stdout)
        self.assertIn(audit["convergence_gate"], {"pass", "fail"})
        self.run_cli("freeze", self.session, "--level", "minor", "--note", "Council-selected concept")
        exported = self.root / "session.md"
        handoff = self.root / "blueprint.json"
        self.run_cli("export", self.session, exported)
        self.run_cli("handoff", self.session, "blueprint", handoff)
        self.assertIn("Time-bounded recovery coach", exported.read_text(encoding="utf-8"))
        self.assertEqual(json.loads(handoff.read_text(encoding="utf-8"))["target"], "blueprint")
        self.assertEqual(json.loads(self.session.read_text(encoding="utf-8"))["meta"]["concept_version"], "0.2.0")

    def test_legacy_migration(self) -> None:
        legacy = {
            "schema_version": "1.0.0",
            "meta": {"title": "Legacy", "domain": "general", "depth": "COUNCIL", "concept_version": "0.1.0", "status": "BRAINSTORM IN PROGRESS", "current_stage": "recover", "created_at": "x", "updated_at": "x"},
            "frame": {"idea": "", "desired_change": "", "actors": [], "constraints": [], "non_goals": [], "success_signals": [], "locked_core": []},
            "sources": [], "ideas": [], "hypotheses": [], "arguments": [], "tensions": [], "decisions": [], "experiments": [], "questions": [],
            "parking_lot": [], "rounds": [], "handoff": {"target": None, "readiness": "not-ready", "gaps": []}
        }
        self.session.write_text(json.dumps(legacy), encoding="utf-8")
        self.run_cli("migrate", self.session)
        self.run_cli("validate", self.session)
        self.assertEqual(json.loads(self.session.read_text(encoding="utf-8"))["schema_version"], "3.0.0")

    def test_v2_migration_preserves_ids_and_adds_v3_engines(self) -> None:
        legacy = {
            "schema_version": "2.0.0",
            "meta": {"session_id": "s", "project_id": None, "title": "V2", "domain": "product", "depth": "DEEP", "concept_version": "0.2.0", "status": "BRAINSTORM IN PROGRESS", "current_stage": "challenge", "created_at": "x", "updated_at": "x"},
            "frame": {"idea": "Old seed", "desired_change": "Change", "actors": [], "constraints": [], "non_goals": [], "success_signals": [], "locked_core": [], "central_tension": "", "highest_impact_unknown": ""},
            "council": {"core_cells": ["expansion", "reality", "adversarial"], "specialists": [], "independence_preserved": None, "cross_examination_completed": False},
            "sources": [], "ideas": [{"id": "BS-IDEA-005", "statement": "Preserved", "status": "selected", "confidence": "medium", "rationale": "", "provenance": "", "tags": [], "relations": [], "created_at": "x", "updated_at": "x"}],
            "hypotheses": [], "arguments": [], "tensions": [], "decisions": [], "experiments": [], "questions": [],
            "parking_lot": [], "rounds": [], "lineage": {"snapshots": []}, "quality": {"latest_audit": None, "history": []},
            "handoff": {"target": None, "readiness": "not-ready", "gaps": [], "last_export": None}
        }
        self.session.write_text(json.dumps(legacy), encoding="utf-8")
        self.run_cli("migrate", self.session)
        self.run_cli("validate", self.session)
        migrated = json.loads(self.session.read_text(encoding="utf-8"))
        self.assertEqual(migrated["ideas"][0]["id"], "BS-IDEA-005")
        self.assertEqual(migrated["schema_version"], "3.0.0")
        self.assertEqual(migrated["council"]["chambers"], ["imagination", "evolution", "council"])
        self.assertEqual(migrated["surface_lab"]["applicability"], "unknown")

    def test_imagination_evolution_surface_and_portfolio_state(self) -> None:
        self.run_cli("init", self.session, "--title", "Cross-surface studio", "--domain", "ai-product", "--depth", "imagination")
        self.run_cli("dna", self.session, "--obsession", "calm technology", "--taste-marker", "invisible until useful", "--anti-pattern", "notification addiction", "--signature-tension", "presence versus interruption", "--confirmation-status", "confirmed")
        frame = self.run_cli("add", self.session, "frames", "--statement", "The product is a transition ritual, not a dashboard", "--status", "selected", "--confidence", "medium", "--tag", "inversion").stdout.strip()
        idea = self.run_cli("add", self.session, "ideas", "--statement", "Capture in context, reflect in focus", "--status", "selected", "--confidence", "medium", "--tag", "valuable-surprise", "--relates-to", frame).stdout.strip()
        genome = self.run_cli("add", self.session, "genomes", "--statement", "Mobile capture crossed with desktop synthesis", "--status", "selected", "--confidence", "medium", "--generation", "1", "--tag", "locus:interaction,locus:surface", "--relates-to", idea).stdout.strip()
        self.run_cli("evolve", self.session, "--name", "Moment specialization", "--selection-pressure", "native affordance", "--operator", "crossover", "--parent", idea, "--survivor", genome, "--delta", "Split capture from synthesis")
        for surface_type in ("mobile", "web", "desktop"):
            self.run_cli("surface", self.session, "--type", surface_type, "--statement", f"Assess {surface_type}", "--status", "candidate", "--confidence", "low", "--relates-to", idea)
        selected = self.run_cli(
            "surface", self.session, "--type", "multi-surface", "--statement", "Mobile capture plus desktop synthesis", "--status", "selected", "--confidence", "medium", "--primary", "--relates-to", idea,
            "--role", "mobile=context capture", "--role", "desktop=deep synthesis", "--canonical-state-owner", "shared encrypted concept graph", "--multi-surface-rationale", "Each surface owns a distinct moment", "--next-surface-trigger", "Add web only when link collaboration is validated",
        ).stdout.strip()
        self.run_cli("portfolio", self.session, "--active-idea", idea, "--coherence-thesis", "One concept graph across distinct moments", "--shared-primitive", "concept graph", "--conflict", "capture speed versus synthesis depth")
        self.run_cli("validate", self.session)
        data = json.loads(self.session.read_text(encoding="utf-8"))
        self.assertEqual(data["surface_lab"]["primary_surface_id"], selected)
        self.assertEqual(data["surface_lab"]["role_map"]["mobile"], "context capture")
        self.assertEqual(data["evolution"]["current_generation"], 1)
        self.assertIn(idea, data["portfolio"]["active_idea_ids"])

    def test_multi_surface_requires_role_map_for_convergence_gate(self) -> None:
        self.init_and_frame()
        idea = self.run_cli("add", self.session, "ideas", "--statement", "Everywhere app", "--status", "selected", "--confidence", "low").stdout.strip()
        self.run_cli("surface", self.session, "--type", "multi-surface", "--statement", "Same features everywhere", "--status", "selected", "--confidence", "low", "--relates-to", idea)
        audit = json.loads(self.run_cli("audit", self.session).stdout)
        self.assertEqual(audit["convergence_gate"], "fail")
        self.assertTrue(any("role map" in warning for warning in audit["warnings"]))

    def test_convergence_requires_quality_gate(self) -> None:
        self.init_and_frame()
        self.run_cli("add", self.session, "ideas", "--statement", "Untested leader", "--status", "selected", "--confidence", "low")
        result = self.run_cli("freeze", self.session, "--converged", expected=2)
        self.assertIn("quality gate fails", result.stderr)
        data = json.loads(self.session.read_text(encoding="utf-8"))
        self.assertEqual(data["meta"]["concept_version"], "0.1.0")
        self.assertEqual(data["meta"]["status"], "BRAINSTORM IN PROGRESS")

    def test_omega_installer_dry_run_and_install(self) -> None:
        omega = self.root / "omega" / "project"
        omega.mkdir(parents=True)
        dry = subprocess.run([sys.executable, str(INSTALLER), str(omega), "--dry-run"], text=True, capture_output=True, check=False)
        self.assertEqual(dry.returncode, 0, msg=dry.stderr)
        result = subprocess.run([sys.executable, str(INSTALLER), str(omega)], text=True, capture_output=True, check=False)
        self.assertEqual(result.returncode, 0, msg=result.stderr)
        destination = omega / "extensions" / "brainstorm-os"
        self.assertTrue((destination / "SKILL.md").is_file())
        self.assertTrue((destination / "installation-receipt.json").is_file())


if __name__ == "__main__":
    unittest.main()

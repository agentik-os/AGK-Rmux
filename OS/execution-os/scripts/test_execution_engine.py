#!/usr/bin/env python3

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("execution_engine.py")


class ExecutionEngineV2Test(unittest.TestCase):
    def run_cli(self, state: Path, *args: str, expect: int = 0):
        proc = subprocess.run(
            ["python3", str(SCRIPT), "--state", str(state), *args],
            text=True, capture_output=True, check=False,
        )
        self.assertEqual(proc.returncode, expect, proc.stderr or proc.stdout)
        stream = proc.stdout if expect in {0, 1} else proc.stderr
        return json.loads(stream)

    def seed(self, state: Path) -> tuple[str, str]:
        self.run_cli(state, "init", "--owner", "Gareth", "--timezone", "Europe/Madrid")
        outcome = self.run_cli(
            state, "add-outcome", "--title", "Ship Execution OS V2", "--domain", "owned venture",
            "--baseline", "V1 installed", "--target", "V2 used daily", "--deadline", "2026-09-30",
            "--done", "T0-T4 run from persistent state", "--proof", "state log and weekly review",
            "--priority", "primary", "--confidence", "4",
        )["created"]
        commitment = self.run_cli(
            state, "add-commitment", "--outcome", outcome, "--title", "Run the first closed day",
            "--next-action", "Open the One Page", "--done", "T1 and T2 records exist",
            "--minutes", "50", "--due", "2026-08-12T18:00:00+02:00",
            "--impact", "5", "--urgency", "4", "--leverage", "5", "--confidence", "4", "--switch-cost", "1",
        )["created"]
        return outcome, commitment

    def test_full_t0_to_t4_scheduler(self):
        with tempfile.TemporaryDirectory() as tmp:
            state = Path(tmp) / "state.json"
            outcome, commitment = self.seed(state)
            self.run_cli(state, "capture", "--text", "Idea for later", "--kind", "?")
            boot = self.run_cli(state, "boot", "--capacity", "GREEN", "--usable-minutes", "240", "--must-win", "Closed day proof", "--not-today", "New product ideas")
            self.assertEqual(boot["single_thread"], commitment)
            focus = self.run_cli(state, "focus", commitment, "--minutes", "50")["focus"]
            second = self.run_cli(state, "focus", commitment, "--minutes", "25", expect=2)
            self.assertIn("Single Thread already active", second["error"])
            end = self.run_cli(state, "focus-end", focus, "--actual-minutes", "55", "--output", "One Page complete", "--next-action", "Open T2 template")
            self.assertEqual(end["estimate_ratio"], 1.1)
            self.run_cli(state, "signal", "--outcome", outcome, "--fact", "T1 completed and focus block closed", "--metric", "1 block")
            self.run_cli(state, "halt", "--proof", "One Page complete", "--classification", "PROGRESSED", "--energy", "4", "--focus", "4", "--friction", "Messages", "--tomorrow", "Open T2 template")
            self.run_cli(state, "reset", "--weekly-truth", "Focus works when messages stay closed", "--next-week-win", "Five closed days", "--system-experiment", "No messages before first block")
            self.run_cli(state, "audit", "--decision", "Continue outcome", "--reason", "Signal moved", "--system-change", "Protect first block", "--obsolete-killed", "Old redesign")
            result = self.run_cli(state, "validate")
            self.assertTrue(result["valid"], result["errors"])
            raw = json.loads(state.read_text())
            for key in ("t0_capture", "t1_boot", "t2_halt", "t3_reset", "t4_audit"):
                self.assertTrue(raw["scheduler"][key])
            self.assertGreaterEqual(len(raw["events"]), 10)

    def test_promise_ledger_lifecycle(self):
        with tempfile.TemporaryDirectory() as tmp:
            state = Path(tmp) / "state.json"
            _, commitment = self.seed(state)
            promise = self.run_cli(
                state, "add-promise", "--stakeholder", "Client A", "--deliverable", "Execution audit",
                "--due", "2026-08-14T17:00:00+02:00", "--notice-by", "2026-08-13T12:00:00+02:00",
                "--consequence", "Client blocked", "--next-proof", "Audit draft", "--commitment", commitment,
            )["created"]
            listed = self.run_cli(state, "promises")
            self.assertEqual(listed["count"], 1)
            self.run_cli(
                state, "renegotiate-promise", promise, "--due", "2026-08-15T17:00:00+02:00",
                "--notice-by", "2026-08-14T12:00:00+02:00", "--next-proof", "Approved outline", "--note", "Scope agreed",
            )
            self.run_cli(state, "deliver-promise", promise, "--evidence", "client-audit.pdf sent")
            self.assertEqual(self.run_cli(state, "promises")["count"], 0)
            self.assertEqual(self.run_cli(state, "promises", "--all")["count"], 1)
            self.assertTrue(self.run_cli(state, "validate")["valid"])

    def test_block_unblock_defer_delegate_cancel(self):
        with tempfile.TemporaryDirectory() as tmp:
            state = Path(tmp) / "state.json"
            outcome, first = self.seed(state)
            self.run_cli(state, "block", first, "--reason", "Awaiting input", "--next-action", "Ask client", "--escalate-at", "2026-08-12T10:00:00+02:00")
            self.run_cli(state, "unblock", first, "--resolution", "Input received", "--next-action", "Open response")
            self.run_cli(state, "defer", first, "--reason", "Lower consequence", "--review-on", "2026-08-20")
            ids = []
            for title in ("Delegate report", "Cancel redesign"):
                ids.append(self.run_cli(
                    state, "add-commitment", "--outcome", outcome, "--title", title,
                    "--next-action", "Open brief", "--done", "Decision recorded", "--minutes", "25",
                    "--due", "2026-08-19T12:00:00+02:00",
                )["created"])
            self.run_cli(state, "delegate", ids[0], "--to", "Operator", "--due", "2026-08-18T12:00:00+02:00", "--note", "Brief sent")
            self.run_cli(state, "cancel", ids[1], "--reason", "No longer supports primary outcome")
            self.assertTrue(self.run_cli(state, "validate")["valid"])

    def test_context_capsule_and_verified_evidence(self):
        with tempfile.TemporaryDirectory() as tmp:
            state = Path(tmp) / "state.json"
            _, commitment = self.seed(state)
            focus = self.run_cli(state, "focus", commitment, "--minutes", "25")["focus"]
            self.run_cli(state, "focus-end", focus, "--actual-minutes", "20", "--output", "Draft ready", "--next-action", "Run acceptance check")
            capsule = self.run_cli(state, "context-capsule", commitment)
            self.assertEqual(capsule["last_output"], "Draft ready")
            self.assertEqual(capsule["resume_with"], "Run acceptance check")
            self.run_cli(state, "complete", commitment, "--kind", "file", "--evidence", "execution-v2.zip", "--acceptance", "Archive test passed")
            self.assertTrue(self.run_cli(state, "validate")["valid"])

    def test_migrate_v1_and_backup(self):
        with tempfile.TemporaryDirectory() as tmp:
            state = Path(tmp) / "state.json"
            state.write_text(json.dumps({
                "version": "1.0", "profile": {"owner": "Gareth", "timezone": "Europe/Madrid", "max_open_commitments": 7, "max_active_commitments": 3},
                "cycle": {}, "outcomes": [], "commitments": [], "blockers": [], "evidence": [], "signals": [], "checkins": [], "decisions": [],
            }))
            migrated = self.run_cli(state, "migrate")
            self.assertTrue(migrated["migrated"])
            self.assertEqual(json.loads(state.read_text())["version"], "2.0")
            backup = Path(tmp) / "backup.json"
            result = self.run_cli(state, "backup", "--output", str(backup))
            self.assertTrue(backup.exists())
            self.assertEqual(len(result["sha256"]), 64)
            self.assertTrue(self.run_cli(state, "validate")["valid"])

    def test_primary_and_outcome_wip_guards(self):
        with tempfile.TemporaryDirectory() as tmp:
            state = Path(tmp) / "state.json"
            self.run_cli(state, "init", "--owner", "Gareth")
            base = ("add-outcome", "--domain", "work", "--baseline", "zero", "--target", "one", "--deadline", "2026-09-30", "--done", "done", "--proof", "proof")
            self.run_cli(state, *base, "--title", "Primary", "--priority", "primary")
            error = self.run_cli(state, *base, "--title", "Second primary", "--priority", "primary", expect=2)
            self.assertIn("Primary outcome already exists", error["error"])
            self.run_cli(state, *base, "--title", "Secondary", "--priority", "secondary")
            self.run_cli(state, *base, "--title", "Maintenance", "--priority", "maintenance")
            error = self.run_cli(state, *base, "--title", "Fourth", "--priority", "secondary", expect=2)
            self.assertIn("ceiling 3", error["error"])


if __name__ == "__main__":
    unittest.main()

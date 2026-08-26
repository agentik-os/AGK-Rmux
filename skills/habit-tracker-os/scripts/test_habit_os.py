#!/usr/bin/env python3
"""Representative tests for the Habit Tracker {OS} deterministic engine."""

from __future__ import annotations

import io
import json
import tempfile
import unittest
from contextlib import redirect_stdout
from datetime import date, datetime, timedelta
from pathlib import Path
from zoneinfo import ZoneInfo

import habit_os


class HabitOSTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.db = str(Path(self.temp_dir.name) / "habit.db")
        result = self.run_cli(
            "init",
            "--user",
            "gareth",
            "--name",
            "Gareth",
            "--timezone",
            "Europe/Madrid",
            "--tone",
            "strategic",
        )
        self.assertEqual(result[0], 0)

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def run_cli(self, *args: str) -> tuple[int, str | dict]:
        output = io.StringIO()
        with redirect_stdout(output):
            code = habit_os.main(["--db", self.db, *args])
        text = output.getvalue().strip()
        try:
            parsed: str | dict = json.loads(text)
        except json.JSONDecodeError:
            parsed = text
        return code, parsed

    @staticmethod
    def local_today() -> date:
        return datetime.now(ZoneInfo("Europe/Madrid")).date()

    def add_build(self, name: str = "Lecture", priority: int = 70) -> dict:
        code, payload = self.run_cli(
            "add",
            "--user",
            "gareth",
            "--name",
            name,
            "--kind",
            "build",
            "--behavior",
            "Lire le livre en cours",
            "--why",
            "Développer mon jugement",
            "--schedule",
            "daily",
            "--cue",
            "Après le dîner",
            "--target",
            "Lire 20 pages",
            "--target-value",
            "20",
            "--minimum",
            "Lire 2 pages",
            "--minimum-value",
            "2",
            "--deep",
            "Lire 45 pages et prendre des notes",
            "--unit",
            "pages",
            "--fallback",
            "Lire 2 pages avant de dormir",
            "--priority",
            str(priority),
        )
        self.assertEqual(code, 0, payload)
        assert isinstance(payload, dict)
        return payload["habit"]

    def test_create_and_list_contract(self) -> None:
        habit = self.add_build()
        self.assertTrue(habit["habit_id"].startswith("HAB-"))
        self.assertEqual(habit["minimum"]["value"], 2.0)
        self.assertEqual(habit["deep"]["value"], None)
        code, payload = self.run_cli("list", "--user", "gareth")
        self.assertEqual(code, 0)
        assert isinstance(payload, dict)
        self.assertEqual(payload["count"], 1)

    def test_reduce_requires_replacement(self) -> None:
        code, payload = self.run_cli(
            "add",
            "--user",
            "gareth",
            "--name",
            "Smoking",
            "--kind",
            "stop",
            "--behavior",
            "Ne pas fumer",
            "--why",
            "Santé",
            "--cue",
            "À chaque envie",
            "--target",
            "Aucune cigarette",
            "--minimum",
            "Interrompre une cigarette commencée",
            "--fallback",
            "Quitter la zone",
        )
        self.assertEqual(code, 2)
        assert isinstance(payload, dict)
        self.assertIn("replacement", payload["error"].lower())

    def test_explicit_log_idempotency_and_unknown_handling(self) -> None:
        habit = self.add_build()
        today = self.local_today()
        code, first = self.run_cli(
            "log",
            "--user",
            "gareth",
            "--habit",
            habit["habit_id"],
            "--outcome",
            "done",
            "--value",
            "24",
            "--unit",
            "pages",
            "--date",
            today.isoformat(),
            "--idempotency-key",
            "message-1",
        )
        self.assertEqual(code, 0)
        code, second = self.run_cli(
            "log",
            "--user",
            "gareth",
            "--habit",
            habit["habit_id"],
            "--outcome",
            "done",
            "--date",
            today.isoformat(),
            "--idempotency-key",
            "message-1",
        )
        self.assertEqual(code, 0)
        assert isinstance(first, dict) and isinstance(second, dict)
        self.assertEqual(first["log"]["log_id"], second["log"]["log_id"])
        start = (today - timedelta(days=2)).isoformat()
        code, review = self.run_cli(
            "review",
            "--user",
            "gareth",
            "--habit",
            habit["habit_id"],
            "--start",
            start,
            "--end",
            today.isoformat(),
        )
        self.assertEqual(code, 0)
        assert isinstance(review, dict)
        metrics = review["habit_metrics"][0]
        self.assertEqual(metrics["scheduled_opportunities"], 3)
        self.assertEqual(metrics["known_opportunities"], 1)
        self.assertEqual(metrics["unknown_opportunities"], 2)
        self.assertAlmostEqual(metrics["data_completeness"], 1 / 3, places=4)
        self.assertAlmostEqual(metrics["target_rate"], 1 / 3, places=4)

    def test_today_never_exceeds_seven(self) -> None:
        for index in range(9):
            self.add_build(name=f"Habit {index}", priority=90 - index)
        code, payload = self.run_cli("today", "--user", "gareth")
        self.assertEqual(code, 0)
        assert isinstance(payload, dict)
        self.assertEqual(payload["primary_count"], 7)
        self.assertEqual(payload["deferred_count"], 2)

    def test_contract_update_is_versioned(self) -> None:
        habit = self.add_build()
        code, payload = self.run_cli(
            "update",
            "--user",
            "gareth",
            "--habit",
            habit["habit_id"],
            "--expected-version",
            "1",
            "--reason",
            "Weekly review experiment",
            "--minimum",
            "Lire 4 pages",
            "--minimum-value",
            "4",
        )
        self.assertEqual(code, 0, payload)
        assert isinstance(payload, dict)
        self.assertEqual(payload["habit"]["version"], 2)
        self.assertEqual(payload["habit"]["supersedes_version"], 1)
        self.assertEqual(payload["habit"]["minimum"]["value"], 4.0)
        code, conflict = self.run_cli(
            "update",
            "--user",
            "gareth",
            "--habit",
            habit["habit_id"],
            "--expected-version",
            "1",
            "--reason",
            "Stale write",
            "--priority",
            "80",
        )
        self.assertEqual(code, 2)
        assert isinstance(conflict, dict)
        self.assertIn("version conflict", conflict["error"].lower())

    def test_correction_invalidates_review_and_delete_removes_chain(self) -> None:
        habit = self.add_build()
        today = self.local_today().isoformat()
        code, logged = self.run_cli(
            "log",
            "--user",
            "gareth",
            "--habit",
            habit["habit_id"],
            "--outcome",
            "done",
            "--date",
            today,
        )
        self.assertEqual(code, 0)
        assert isinstance(logged, dict)
        original_id = logged["log"]["log_id"]
        code, review = self.run_cli(
            "review",
            "--user",
            "gareth",
            "--habit",
            habit["habit_id"],
            "--days",
            "1",
            "--save",
        )
        self.assertEqual(code, 0)
        assert isinstance(review, dict)
        review_id = review["review_id"]
        code, corrected = self.run_cli(
            "correct",
            "--user",
            "gareth",
            "--log-id",
            original_id,
            "--outcome",
            "minimum",
            "--value",
            "2",
        )
        self.assertEqual(code, 0, corrected)
        assert isinstance(corrected, dict)
        self.assertIn(review_id, corrected["invalidated_review_ids"])
        code, deleted = self.run_cli(
            "delete",
            "--user",
            "gareth",
            "--scope",
            "log",
            "--target",
            original_id,
            "--confirm",
            f"DELETE {original_id}",
        )
        self.assertEqual(code, 0, deleted)
        assert isinstance(deleted, dict)
        self.assertEqual(len(deleted["deleted_log_ids"]), 2)

    def test_season_and_experiment(self) -> None:
        habit = self.add_build()
        code, season = self.run_cli(
            "season",
            "--user",
            "gareth",
            "--kind",
            "recover",
            "--reason",
            "Illness and reduced capacity",
        )
        self.assertEqual(code, 0, season)
        assert isinstance(season, dict)
        self.assertEqual(season["season"]["kind"], "recover")
        start = self.local_today()
        code, experiment = self.run_cli(
            "experiment",
            "--user",
            "gareth",
            "--habit",
            habit["habit_id"],
            "--hypothesis",
            "Preparing the book reduces startup friction",
            "--change",
            "Place the book on the table before dinner",
            "--start",
            start.isoformat(),
            "--end",
            (start + timedelta(days=7)).isoformat(),
            "--evidence",
            "Daily outcome and barrier code",
            "--success",
            "Five minimum-or-better outcomes",
            "--stop",
            "User asks to stop",
            "--rollback",
            "Return book to shelf",
            "--status",
            "active",
        )
        self.assertEqual(code, 0, experiment)
        assert isinstance(experiment, dict)
        self.assertTrue(experiment["experiment"]["experiment_id"].startswith("EXP-"))

    def test_reduce_metrics_separate_no_exposure(self) -> None:
        code, payload = self.run_cli(
            "add",
            "--user",
            "gareth",
            "--name",
            "Smoking",
            "--kind",
            "stop",
            "--behavior",
            "Ne pas fumer pendant une envie",
            "--why",
            "Protéger ma santé",
            "--schedule",
            "opportunity",
            "--event-definition",
            "Une envie de fumer",
            "--cue-type",
            "opportunity",
            "--cue",
            "Quand une envie apparaît",
            "--target",
            "Résister à l'envie",
            "--minimum",
            "Interrompre le comportement",
            "--fallback",
            "Quitter la zone et contacter mon support",
            "--replacement",
            "Boire de l'eau et marcher deux minutes",
        )
        self.assertEqual(code, 0, payload)
        assert isinstance(payload, dict)
        habit_id = payload["habit"]["habit_id"]
        for index, outcome in enumerate(["no_exposure", "resisted", "lapse"]):
            code, _ = self.run_cli(
                "log",
                "--user",
                "gareth",
                "--habit",
                habit_id,
                "--outcome",
                outcome,
                "--date",
                (self.local_today() - timedelta(days=2 - index)).isoformat(),
            )
            self.assertEqual(code, 0)
        code, review = self.run_cli(
            "review",
            "--user",
            "gareth",
            "--habit",
            habit_id,
            "--days",
            "3",
        )
        self.assertEqual(code, 0)
        assert isinstance(review, dict)
        metrics = review["habit_metrics"][0]
        self.assertEqual(metrics["scheduled_opportunities"], 2)
        self.assertEqual(metrics["outcome_counts"]["no_exposure"], 1)
        self.assertEqual(metrics["target_rate"], 0.5)

    def test_doctor(self) -> None:
        self.add_build()
        code, payload = self.run_cli("doctor")
        self.assertEqual(code, 0)
        assert isinstance(payload, dict)
        self.assertTrue(payload["ok"])

    def test_json_export_matches_state_shape(self) -> None:
        habit = self.add_build()
        code, _ = self.run_cli(
            "log",
            "--user",
            "gareth",
            "--habit",
            habit["habit_id"],
            "--outcome",
            "minimum",
            "--date",
            self.local_today().isoformat(),
        )
        self.assertEqual(code, 0)
        code, payload = self.run_cli("export", "--user", "gareth", "--format", "json")
        self.assertEqual(code, 0)
        assert isinstance(payload, dict)
        self.assertEqual(payload["schema_version"], "1.0")
        self.assertEqual(payload["user"]["timezone"], "Europe/Madrid")
        self.assertEqual(payload["season"]["kind"], "build")
        self.assertEqual(len(payload["habits"]), 1)
        self.assertEqual(len(payload["logs"]), 1)
        self.assertIn("experiments", payload)
        self.assertIn("reviews", payload)


if __name__ == "__main__":
    unittest.main()

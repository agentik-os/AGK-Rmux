#!/usr/bin/env python3
"""Tests for the local Storyteller {OS} story bank."""

from __future__ import annotations

import json
import sqlite3
import tempfile
import unittest
from pathlib import Path

import storyteller_os as so


class StorytellerOSTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tempdir = tempfile.TemporaryDirectory()
        self.db_path = str(Path(self.tempdir.name) / "stories.db")
        self.connection = so.connect(self.db_path)
        so.init_db(self.connection)

    def tearDown(self) -> None:
        self.connection.close()
        self.tempdir.cleanup()

    def create_story(self, title: str = "The broken demo") -> dict:
        story_id = so.make_story_id(self.connection, title)
        story = so.base_story(
            story_id=story_id,
            title=title,
            raw_text="The demo failed during the client meeting.",
            story_class="decision",
            truth_class="remembered",
            privacy_level="private",
            job="teach",
            audience="AI founders",
            contract="coach",
            tags=["client", "failure"],
        )
        so.insert_story(self.connection, story)
        return story

    def test_create_and_load_story(self) -> None:
        story = self.create_story()
        loaded = so.load_story(self.connection, story["story_id"])
        self.assertEqual(loaded["title_working"], "The broken demo")
        self.assertEqual(loaded["craft"]["story_class"], "decision")
        self.assertEqual(loaded["status"], "captured")

    def test_ids_are_unique(self) -> None:
        first = self.create_story()
        second = self.create_story()
        self.assertNotEqual(first["story_id"], second["story_id"])

    def test_nested_update_and_column_sync(self) -> None:
        story = self.create_story()
        so.set_path(story, "dna.core_change", "I stopped hiding failure.")
        so.set_path(story, "status", "mined")
        so.save_story(self.connection, story)
        loaded = so.load_story(self.connection, story["story_id"])
        self.assertEqual(loaded["dna"]["core_change"], "I stopped hiding failure.")
        row = self.connection.execute(
            "SELECT status FROM stories WHERE story_id = ?", (story["story_id"],)
        ).fetchone()
        self.assertEqual(row["status"], "mined")

    def test_validation_blocks_approved_without_gates(self) -> None:
        story = self.create_story()
        story["status"] = "approved"
        errors, _ = so.validate_story(story)
        self.assertTrue(any("truth_gate" in error for error in errors))
        self.assertTrue(any("consent_gate" in error for error in errors))

    def test_structural_score_increases_with_story_dna(self) -> None:
        story = self.create_story()
        low = so.structural_score(story)["total"]
        story["dna"].update({
            "core_change": "I learned to show uncertainty early.",
            "pressure": "The client had tied the decision to this demo.",
            "hinge": "I stopped the demo and explained the failure.",
            "proof_detail": "The red timeout message stayed on screen.",
            "meaning": "Trust rose when performance certainty fell.",
            "truth_boundary": "The dialogue is paraphrased.",
            "voice_marker": "Direct, no guru language.",
        })
        story["craft"].update({
            "obstacle": "The integration timed out.",
            "stakes": "The client could cancel.",
            "choice": "I disclosed the failure.",
            "external_consequence": "We moved to a diagnostic session.",
            "internal_update": "I stopped performing certainty.",
            "selected_structure": "failure-correction",
            "opening": "The red error appeared in front of the client.",
            "ending": "Now I demo the failure path first.",
            "beats": ["error", "choice", "consequence"],
        })
        story["intent"]["desired_update"] = "Treat honest uncertainty as competence."
        story["voice"]["source_samples"] = ["The demo failed. I said it plainly."]
        high = so.structural_score(story)["total"]
        self.assertGreater(high, low)
        self.assertGreaterEqual(high, 90)

    def test_high_consequence_unverified_claim_warns(self) -> None:
        story = self.create_story()
        story["truth"]["claims"].append({
            "text": "Revenue doubled.",
            "consequence_if_wrong": "high",
            "verification_status": "needs_source",
        })
        _, warnings = so.validate_story(story)
        self.assertTrue(any("high-consequence" in warning for warning in warnings))

    def test_markdown_export_contains_raw_source(self) -> None:
        story = self.create_story()
        text = so.markdown_story(story)
        self.assertIn("# The broken demo", text)
        self.assertIn("The demo failed", text)

    def test_json_round_trip(self) -> None:
        story = self.create_story()
        encoded = json.dumps(story, ensure_ascii=False)
        decoded = json.loads(encoded)
        self.assertEqual(decoded["story_id"], story["story_id"])


if __name__ == "__main__":
    unittest.main()

#!/usr/bin/env python3
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts" / "hooks"))

import omega_plan_state


class HookPlanStateFixtures(unittest.TestCase):
    def analyze(self, name):
        result = omega_plan_state.analyze(
            ROOT / "tests" / "fixtures" / "hooks" / name
        )
        self.assertIsNotNone(result)
        return result

    def test_all_provider_and_negative_fixtures(self):
        cases = {
            "codex-update-plan.jsonl": {
                "plan_ever": True,
                "total_tasks": 2,
                "open_items": ["Verify runtime"],
            },
            "codex-successful-verification.jsonl": {
                "edited": True,
                "verified": True,
            },
            "codex-failed-verification.jsonl": {
                "edited": True,
                "verified": False,
            },
            "codex-irrelevant-command.jsonl": {
                "edited": True,
                "verified": False,
            },
            "codex-custom-tool-verification.jsonl": {
                "edited": True,
                "verified": True,
            },
            "claude-task-plan.jsonl": {
                "plan_ever": True,
                "total_tasks": 1,
                "open_items": [],
            },
            "claude-todo-open.jsonl": {
                "plan_ever": True,
                "total_tasks": 2,
                "open_items": ["Verify runtime"],
            },
            "claude-successful-verification.jsonl": {
                "edited": True,
                "verified": True,
            },
            "claude-edit-invalidates-verification.jsonl": {
                "edited": True,
                "verified": False,
            },
            "gemini-update-plan.jsonl": {
                "plan_ever": True,
                "total_tasks": 2,
                "open_items": ["Verify runtime"],
            },
            "gemini-successful-verification.jsonl": {
                "edited": True,
                "verified": True,
            },
            "sidechain-plan-is-ignored.jsonl": {
                "plan_ever": False,
                "edited": False,
                "total_tasks": 0,
            },
            # R-PREFLIGHT: a read-only preflight awaiting approval. The finish
            # guard exempts exactly this shape (preflight AND zero mutations),
            # so both keys are asserted — an exemption keyed on the text alone
            # would be a bypass.
            "claude-preflight-awaiting-approval.jsonl": {
                "plan_ever": False,
                "edited": False,
                "mutations": 0,
                "preflight": True,
            },
        }
        self.assertEqual(len(cases), 13)
        for fixture, expected in cases.items():
            with self.subTest(fixture=fixture):
                observed = self.analyze(fixture)
                for key, value in expected.items():
                    self.assertEqual(observed[key], value)

    def test_preflight_detection_needs_structure_not_vocabulary(self):
        # The markers must OPEN a line and be a heading/label, so ordinary prose
        # that happens to say "plan" or "goal" never buys a session a free stop.
        preflight_en = (
            "**Goal.** Ship the checkout route.\n\n"
            "**Blocking questions.** None.\n\n"
            "**Assumptions.**\n1. Price ids come from the operator.\n\n"
            "**Plan.** route.ts, then the webhook.\n"
        )
        preflight_fr = (
            "## Objectif\nAjouter la route de paiement.\n\n"
            "## Questions bloquantes\nAucune.\n\n"
            "## Hypothèses\n1. Les prix existent déjà.\n\n"
            "## Plan\nroute.ts puis le webhook.\n"
        )
        two_families_only = "**Goal.** Ship it.\n\n**Plan.** Later today.\n"
        prose = (
            "I read the code. The plan for the migration is clear and the goals "
            "for later are noted. Tell me how you want to proceed."
        )
        self.assertTrue(omega_plan_state.is_preflight(preflight_en))
        self.assertTrue(omega_plan_state.is_preflight(preflight_fr))
        self.assertFalse(omega_plan_state.is_preflight(two_families_only))
        self.assertFalse(omega_plan_state.is_preflight(prose))
        self.assertFalse(omega_plan_state.is_preflight(""))
        self.assertFalse(omega_plan_state.is_preflight(None))

    def test_missing_transcript_is_unobservable(self):
        self.assertIsNone(
            omega_plan_state.analyze(ROOT / "tests" / "fixtures" / "missing.jsonl")
        )




class ShellWritesCountAsMutations(unittest.TestCase):
    """A Bash heredoc writes a file as surely as the Write tool.

    Counting only the typed tools reported "0 file mutations" for sessions that
    had rewritten many files, and — the part that actually mattered — let such a
    session keep the preflight exemption that stop-verify-hook.sh grants only
    while mutations == 0 (R-PREFLIGHT: the exemption dies at the first write).
    """

    def test_shell_write_detection(self):
        writes = ["cat > a.py <<'EOF'", "sed -i s/a/b/ f", "cp a b", "mv a b",
                  "tee -a log.txt", "git apply p.patch", "printf x >> f.txt"]
        reads = ["ls -la", "echo hi > /dev/null", "grep x y 2>&1",
                 "cargo check 2>&1 | tail", "python3 -c 'print(1)' | head"]
        for cmd in writes:
            self.assertTrue(omega_plan_state._is_shell_write(cmd), cmd)
        for cmd in reads:
            self.assertFalse(omega_plan_state._is_shell_write(cmd), cmd)

    def test_fixture_counts_shell_writes_and_denies_the_preflight_exemption(self):
        st = omega_plan_state.analyze(
            ROOT / "tests" / "fixtures" / "hooks"
            / "claude-bash-writes-are-mutations.jsonl")
        self.assertEqual(st["mutations"], 3)      # heredoc + sed -i + cp
        self.assertTrue(st["preflight"])          # it claims a preflight…
        # …but the hook gates that on mutations == 0, so the claim is refused.
        self.assertNotEqual(st["mutations"], 0)


class EnumerationSatisfiesThePlanRequirement(unittest.TestCase):
    """R-PLAN wants a recorded per-ask enumeration, not one specific tool.

    A harness without TaskCreate/TodoWrite/update_plan could never satisfy the
    planless-work check, so it spent its whole block budget refusing a stop the
    agent had no way to earn (305 sessions blocked on this box, 62 driven to the
    ceiling). Both escapes require real evidence and cannot silently disable it.
    """

    def test_is_enumeration_accepts_real_enumerations(self):
        for text in (
            "- [x] ship the index\n- [x] wire the RAG\n- [x] push both SSOTs\n",
            "| # | Ask | State |\n| 1 | corpus | done |\n| 2 | rag | done |\n"
            "| 3 | rule | done |\n",
            "1. built the index - done\n2. wired the rag - done\n3. pushed - done\n",
            "- [x] corpus epingle\n- [x] index RAG\n- [x] regle doctrine\n",
        ):
            self.assertTrue(omega_plan_state.is_enumeration(text), text[:40])

    def test_is_enumeration_rejects_prose_and_short_lists(self):
        for text in ("", None,
                     "I finished the integration; it is done, verified and shipped.",
                     "- [x] one thing\n- [x] another\n"):
            self.assertFalse(omega_plan_state.is_enumeration(text), str(text)[:40])

    def test_missing_plan_tools_are_detected_across_records(self):
        st = omega_plan_state.analyze(
            ROOT / "tests" / "fixtures" / "hooks"
            / "claude-no-plan-tool-enumerated.jsonl")
        self.assertTrue(st["plan_tools_missing"])
        self.assertTrue(st["enumeration"])

    def test_normal_sessions_are_never_marked_as_missing_plan_tools(self):
        for name in ("claude-task-plan.jsonl", "claude-todo-open.jsonl",
                     "codex-update-plan.jsonl",
                     "claude-preflight-awaiting-approval.jsonl"):
            st = omega_plan_state.analyze(
                ROOT / "tests" / "fixtures" / "hooks" / name)
            self.assertFalse(st["plan_tools_missing"], name)
            self.assertFalse(st["enumeration"], name)


if __name__ == "__main__":
    unittest.main()

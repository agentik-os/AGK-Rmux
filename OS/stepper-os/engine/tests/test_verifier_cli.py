from typer.testing import CliRunner

from stepper_engine.cli import app
from stepper_engine.graph import StepGraph
from stepper_engine.loader import load_project
from stepper_engine.models import AcceptanceCheck, StepStatus
from stepper_engine.tracker import Tracker
from stepper_engine.verifier import run_check, verify_step

runner = CliRunner()


def _ctx(project_dir):
    project = load_project(project_dir)
    return project, Tracker(project.state_dir)


# ── verifier unit ────────────────────────────────────────────────────────────


def test_file_exists_check(project_dir):
    project, tracker = _ctx(project_dir)
    step = project.steps["STEP-1"]
    check = AcceptanceCheck(type="file_exists", path="artifacts/STEP-1.txt")
    assert not run_check(check, project.root, step, tracker).passed
    (project.root / "artifacts").mkdir()
    (project.root / "artifacts" / "STEP-1.txt").write_text("done")
    assert run_check(check, project.root, step, tracker).passed


def test_grep_absent_reports_evidence(project_dir):
    project, tracker = _ctx(project_dir)
    step = project.steps["STEP-1"]
    (project_dir / "src").mkdir()
    (project_dir / "src" / "app.ts").write_text("if (tier === 'gold') {}\n")
    check = AcceptanceCheck(type="grep_absent", path="src", pattern="tier ===")
    result = run_check(check, project.root, step, tracker)
    assert not result.passed
    assert any("app.ts:1" in line for line in result.evidence)


def test_command_check_runs_argv_without_shell(project_dir):
    project, tracker = _ctx(project_dir)
    step = project.steps["STEP-1"]
    ok = AcceptanceCheck(type="command", command="true")
    ko = AcceptanceCheck(type="command", command="false")
    assert run_check(ok, project.root, step, tracker).passed
    result = run_check(ko, project.root, step, tracker)
    assert not result.passed and result.exit_code == 1


def test_review_roles_gate_verification(project_dir):
    (project_dir / "stepper" / "steps" / "STEP-R.yaml").write_text(
        """\
step_id: STEP-R
title: reviewed step
module: MOD-001
review_roles: [security]
"""
    )
    project, tracker = _ctx(project_dir)
    step = project.steps["STEP-R"]
    results = verify_step(step, project.root, tracker)
    assert not all(r.passed for r in results)
    tracker.record_review("STEP-R", "security", "PASS", "alice")
    results = verify_step(step, project.root, tracker)
    assert all(r.passed for r in results)


# ── CLI end to end ───────────────────────────────────────────────────────────


def test_cli_full_loop_start_done_release(project_dir):
    project_args = ["--project", str(project_dir)]

    assert runner.invoke(app, ["validate", *project_args]).exit_code == 0

    # start STEP-1 (READY) - emits the brief
    result = runner.invoke(app, ["start", "STEP-1", *project_args])
    assert result.exit_code == 0
    assert "Step brief - STEP-1" in result.output

    # done without the artifact → FAILED, exit 1 (no self-report DONE)
    result = runner.invoke(app, ["done", "STEP-1", *project_args])
    assert result.exit_code == 1

    # produce the artifact, repair loop passes
    (project_dir / "artifacts").mkdir(exist_ok=True)
    (project_dir / "artifacts" / "STEP-1.txt").write_text("done")
    result = runner.invoke(app, ["done", "STEP-1", *project_args])
    assert result.exit_code == 0

    # STEP-2 became READY; starting STEP-3 (dep not DONE) is refused
    result = runner.invoke(app, ["start", "STEP-3", *project_args])
    assert result.exit_code == 1
    assert "not READY" in result.output

    # release-check fails while P0 STEP-2 is open, and names it
    result = runner.invoke(app, ["release-check", *project_args])
    assert result.exit_code == 1
    assert "STEP-2" in result.output

    # finish STEP-2 (P0). STEP-3 is P1 and does not gate a P0 release.
    (project_dir / "artifacts" / "STEP-2.txt").write_text("done")
    runner.invoke(app, ["start", "STEP-2", *project_args])
    assert runner.invoke(app, ["done", "STEP-2", *project_args]).exit_code == 0
    assert runner.invoke(app, ["release-check", *project_args]).exit_code == 0


def test_cli_init_scaffolds_runnable_project(tmp_path):
    project_args = ["--project", str(tmp_path)]
    assert runner.invoke(app, ["init", *project_args]).exit_code == 0
    assert runner.invoke(app, ["validate", *project_args]).exit_code == 0
    result = runner.invoke(app, ["plan", *project_args])
    assert result.exit_code == 0
    assert "STEP-000001" in result.output


def test_cli_block_unblock(project_dir):
    project_args = ["--project", str(project_dir)]
    assert (
        runner.invoke(
            app, ["block", "STEP-1", "--reason", "waiting on operator", *project_args]
        ).exit_code
        == 0
    )
    project = load_project(project_dir)
    tracker = Tracker(project.state_dir)
    assert tracker.status_of("STEP-1") == StepStatus.BLOCKED
    assert runner.invoke(app, ["unblock", "STEP-1", *project_args]).exit_code == 0


def test_cli_attempt_ceiling_escalates(project_dir):
    """max_fix_attempts bounds the repair loop (R-LOOP: thrash → human)."""
    (project_dir / "stepper.yaml").write_text(
        "project:\n  name: testproj\nexecution:\n  max_fix_attempts: 2\n"
    )
    project_args = ["--project", str(project_dir)]
    for _ in range(2):
        assert runner.invoke(app, ["start", "STEP-1", *project_args]).exit_code == 0
        assert runner.invoke(app, ["done", "STEP-1", *project_args]).exit_code == 1
    result = runner.invoke(app, ["start", "STEP-1", *project_args])
    assert result.exit_code == 1
    assert "escalate" in result.output

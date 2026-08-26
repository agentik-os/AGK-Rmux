from stepper_engine.graph import StepGraph
from stepper_engine.loader import load_project
from stepper_engine.models import StepStatus
from stepper_engine.planner import is_ready, rank, wave
from stepper_engine.tracker import Tracker


def _ctx(project_dir):
    project = load_project(project_dir)
    graph = StepGraph(list(project.steps.values()))
    tracker = Tracker(project.state_dir)
    return project, graph, tracker


def test_only_dependency_free_steps_are_ready(project_dir):
    project, graph, tracker = _ctx(project_dir)
    assert is_ready(project.steps["STEP-1"], graph, tracker)
    assert not is_ready(project.steps["STEP-2"], graph, tracker)


def test_done_dependency_unlocks_downstream(project_dir):
    project, graph, tracker = _ctx(project_dir)
    tracker.set_status("STEP-1", StepStatus.DONE)
    assert is_ready(project.steps["STEP-2"], graph, tracker)


def test_rank_is_explainable_and_ordered(project_dir):
    project, graph, tracker = _ctx(project_dir)
    candidates = rank(project, graph, tracker)
    assert [c.step.step_id for c in candidates] == ["STEP-1"]
    assert "priority" in candidates[0].components
    assert candidates[0].score > 0


def test_wave_respects_lock_conflicts(project_dir):
    # Two dependency-free steps sharing a domain lock: only one enters the wave.
    for step_id in ["STEP-A", "STEP-B"]:
        (project_dir / "stepper" / "steps" / f"{step_id}.yaml").write_text(
            f"""\
step_id: {step_id}
title: {step_id}
module: MOD-001
priority: P0
locks:
  - domain: billing
"""
        )
    project, graph, tracker = _ctx(project_dir)
    selected = {c.step.step_id for c in wave(project, graph, tracker)}
    assert len(selected & {"STEP-A", "STEP-B"}) == 1


def test_tracker_state_survives_restart(project_dir):
    project, graph, tracker = _ctx(project_dir)
    tracker.open_attempt("STEP-1", "claude")
    tracker.set_status("STEP-1", StepStatus.RUNNING)
    tracker.save()

    reloaded = Tracker(project.state_dir)
    assert reloaded.status_of("STEP-1") == StepStatus.RUNNING
    assert reloaded.state_of("STEP-1").attempts == 1


def test_reconcile_drops_dead_running_steps_to_failed(project_dir):
    project, graph, tracker = _ctx(project_dir)
    tracker.open_attempt("STEP-1", "claude")
    tracker.set_status("STEP-1", StepStatus.RUNNING)
    tracker.save()

    fresh = Tracker(project.state_dir)
    recovered = fresh.reconcile()
    assert recovered == ["STEP-1"]
    assert fresh.status_of("STEP-1") == StepStatus.FAILED
    # FAILED is schedulable again - the planner re-offers it.
    assert is_ready(project.steps["STEP-1"], StepGraph(list(project.steps.values())), fresh)


def test_latest_review_verdict_wins(project_dir):
    project, _, tracker = _ctx(project_dir)
    tracker.record_review("STEP-1", "security", "PASS", "alice")
    tracker.record_review("STEP-1", "security", "FAIL", "bob")
    assert tracker.passing_review_roles("STEP-1") == set()
    tracker.record_review("STEP-1", "security", "PASS", "carol")
    assert tracker.passing_review_roles("STEP-1") == {"security"}

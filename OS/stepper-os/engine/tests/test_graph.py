import pytest

from stepper_engine.graph import StepGraph


def test_graph_accepts_dag(make_step):
    graph = StepGraph([make_step("STEP-1"), make_step("STEP-2", ["STEP-1"])])
    assert "STEP-2" in graph.downstream("STEP-1")


def test_graph_rejects_cycle(make_step):
    with pytest.raises(ValueError, match="cycle"):
        StepGraph([make_step("STEP-1", ["STEP-2"]), make_step("STEP-2", ["STEP-1"])])


def test_graph_rejects_missing_dependency(make_step):
    with pytest.raises(ValueError, match="Missing dependency"):
        StepGraph([make_step("STEP-1", ["STEP-GHOST"])])


def test_critical_path_prefers_heaviest_chain(make_step):
    # STEP-1 -> STEP-2(w=10) is heavier than STEP-1 -> STEP-3(w=1)
    graph = StepGraph(
        [
            make_step("STEP-1"),
            make_step("STEP-2", ["STEP-1"], weight=10),
            make_step("STEP-3", ["STEP-1"], weight=1),
        ]
    )
    assert graph.critical_path() == ["STEP-1", "STEP-2"]
    assert graph.critical_path_weight("STEP-1") == 11


def test_downstream_weight_sums_unlocked_work(make_step):
    graph = StepGraph(
        [
            make_step("STEP-1"),
            make_step("STEP-2", ["STEP-1"], weight=3),
            make_step("STEP-3", ["STEP-2"], weight=4),
        ]
    )
    assert graph.downstream_weight("STEP-1") == 7

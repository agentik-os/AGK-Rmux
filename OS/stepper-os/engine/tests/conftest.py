import pytest

from stepper_engine.models import Dependencies, StepSpec


@pytest.fixture()
def make_step():
    def _make(step_id: str, deps=None, **kwargs):
        defaults = dict(
            step_id=step_id,
            title=step_id,
            module="MOD-001",
            epic="",
            slice="",
            priority="P0",
            dependencies=Dependencies(hard=deps or []),
        )
        defaults.update(kwargs)
        return StepSpec(**defaults)

    return _make


@pytest.fixture()
def project_dir(tmp_path):
    """A minimal on-disk Stepper project: manifest + 3 chained steps."""
    (tmp_path / "stepper" / "steps").mkdir(parents=True)
    (tmp_path / "stepper" / "modules").mkdir(parents=True)
    (tmp_path / "stepper.yaml").write_text(
        "project:\n  name: testproj\nrelease:\n  target: P0\n"
    )
    (tmp_path / "stepper" / "modules" / "MOD-001.yaml").write_text(
        "module_id: MOD-001\nname: Core\n"
    )
    steps = {
        "STEP-1": {"deps": "[]", "priority": "P0"},
        "STEP-2": {"deps": "[STEP-1]", "priority": "P0"},
        "STEP-3": {"deps": "[STEP-2]", "priority": "P1"},
    }
    for step_id, info in steps.items():
        (tmp_path / "stepper" / "steps" / f"{step_id}.yaml").write_text(
            f"""\
step_id: {step_id}
title: {step_id} title
module: MOD-001
priority: {info['priority']}
weight: 2
dependencies:
  hard: {info['deps']}
acceptance_checks:
  - type: file_exists
    path: artifacts/{step_id}.txt
"""
        )
    return tmp_path

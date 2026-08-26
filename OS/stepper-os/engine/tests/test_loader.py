import pytest

from stepper_engine.loader import LoadError, load_project


def test_loads_project(project_dir):
    project = load_project(project_dir)
    assert set(project.steps) == {"STEP-1", "STEP-2", "STEP-3"}
    assert project.manifest.project["name"] == "testproj"
    assert "MOD-001" in project.modules


def test_missing_manifest_is_a_clear_error(tmp_path):
    with pytest.raises(LoadError, match="not a Stepper project"):
        load_project(tmp_path)


def test_duplicate_step_id_rejected(project_dir):
    dup = project_dir / "stepper" / "steps" / "ZZZ-dup.yaml"
    dup.write_text("step_id: STEP-1\ntitle: dup\nmodule: MOD-001\n")
    with pytest.raises(LoadError, match="duplicate"):
        load_project(project_dir)


def test_unknown_dependency_rejected(project_dir):
    bad = project_dir / "stepper" / "steps" / "STEP-4.yaml"
    bad.write_text(
        "step_id: STEP-4\ntitle: t\nmodule: MOD-001\n"
        "dependencies:\n  hard: [STEP-GHOST]\n"
    )
    with pytest.raises(LoadError, match="unknown dependency"):
        load_project(project_dir)


def test_unknown_module_rejected(project_dir):
    bad = project_dir / "stepper" / "steps" / "STEP-4.yaml"
    bad.write_text("step_id: STEP-4\ntitle: t\nmodule: MOD-GHOST\n")
    with pytest.raises(LoadError, match="unknown module"):
        load_project(project_dir)


def test_extra_fields_survive_the_roundtrip(project_dir):
    rich = project_dir / "stepper" / "steps" / "STEP-9.yaml"
    rich.write_text(
        """\
step_id: STEP-9
title: rich step
module: MOD-001
objective:
  concise: c
  outcome: o
attention:
  critical: [never trust client tier]
"""
    )
    project = load_project(project_dir)
    extras = project.steps["STEP-9"].model_extra
    assert extras["objective"]["concise"] == "c"
    assert extras["attention"]["critical"] == ["never trust client tier"]

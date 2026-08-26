"""Blueprint + Design references: each step pulls the right upstream docs."""

from stepper_engine.brief import agent_brief
from stepper_engine.loader import check_references, load_project


def _write(project_dir, rel, text):
    p = project_dir / rel
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(text)


def _add_sources(project_dir):
    (project_dir / "stepper.yaml").write_text(
        """\
project:
  name: testproj
sources:
  blueprint:
    root: ./blueprint
    handoff: ./blueprint/handoff.json
  design:
    root: ./design
    handoff: ./design/handoff.json
release:
  target: P0
"""
    )
    _write(project_dir, "blueprint/handoff.json", '{"handoff_id": "BPH-1"}')
    _write(project_dir, "blueprint/volume-1.md", "# product truth")
    _write(project_dir, "design/handoff.json", '{"flows": []}')
    _write(project_dir, "design/screens.md", "# screen contracts")


def test_typed_blueprint_and_design_refs_render_in_brief(project_dir):
    _add_sources(project_dir)
    (project_dir / "stepper" / "steps" / "STEP-1.yaml").write_text(
        """\
step_id: STEP-1
title: Build the eligibility screen
module: MOD-001
priority: P0
blueprint_references:
  - doc: volume-1.md
    sections: [12, 14]
    ids: [REQ-003]
    note: server-authoritative eligibility
design_references:
  - doc: screens.md
    ids: [SCREEN-ELIG, FLOW-BOOK]
    note: the eligibility screen + booking flow
expected_files:
  create:
    - apps/web/screens/eligibility.tsx
acceptance_checks:
  - type: file_exists
    path: apps/web/screens/eligibility.tsx
"""
    )
    project = load_project(project_dir)
    step = project.steps["STEP-1"]
    assert len(step.blueprint_references) == 1
    assert step.blueprint_references[0].ids == ["REQ-003"]
    assert len(step.design_references) == 1
    assert step.design_references[0].ids == ["SCREEN-ELIG", "FLOW-BOOK"]

    brief = agent_brief(project, step)
    # Both sections appear, with docs resolved against their source roots.
    assert "Blueprint references" in brief
    assert "Design references" in brief
    assert "blueprint/volume-1.md" in brief
    assert "design/screens.md" in brief
    assert "SCREEN-ELIG" in brief


def test_source_root_falls_back_to_legacy_blueprint(project_dir):
    # No `sources`, only the legacy blueprint.root — refs still resolve there.
    (project_dir / "stepper.yaml").write_text(
        "project:\n  name: t\nblueprint:\n  root: ./bp\n"
    )
    project = load_project(project_dir)
    assert project.source_root("blueprint") == "./bp"
    assert project.source_root("design") == "design"


def test_check_references_flags_missing_doc(project_dir):
    _add_sources(project_dir)
    (project_dir / "stepper" / "steps" / "STEP-1.yaml").write_text(
        """\
step_id: STEP-1
title: t
module: MOD-001
blueprint_references:
  - doc: does-not-exist.md
"""
    )
    project = load_project(project_dir)
    warnings = check_references(project)
    assert any("does-not-exist.md" in w for w in warnings)


def test_check_references_flags_ui_step_without_design_ref(project_dir):
    _add_sources(project_dir)
    (project_dir / "stepper" / "steps" / "STEP-1.yaml").write_text(
        """\
step_id: STEP-1
title: a UI step with no design ref
module: MOD-001
expected_files:
  create:
    - apps/web/components/Card.tsx
"""
    )
    project = load_project(project_dir)
    warnings = check_references(project)
    assert any("no design_references" in w for w in warnings)


def test_legacy_extras_blueprint_refs_still_render(project_dir):
    # Old steps that put blueprint_references as free dicts still work.
    (project_dir / "stepper" / "steps" / "STEP-1.yaml").write_text(
        """\
step_id: STEP-1
title: legacy
module: MOD-001
blueprint_references:
  - doc: volume-3.md
    sections: [1]
"""
    )
    project = load_project(project_dir)
    step = project.steps["STEP-1"]
    brief = agent_brief(project, step)
    assert "volume-3.md" in brief

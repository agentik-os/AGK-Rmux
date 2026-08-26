"""`stepper init` scaffolding: the manifest + spec tree + a first example
step, so a fresh project is runnable in under a minute."""

from __future__ import annotations

from pathlib import Path

MANIFEST_TEMPLATE = """\
project:
  name: {name}
  repository: .

blueprint:
  root: ./blueprint
  version: "1.0"

# Upstream truth each step references. Blueprint = product truth (from
# Blueprint OS); design = UX/UI truth (from Design OS). A step's
# blueprint_references / design_references resolve their `doc` against these
# roots, and `stepper validate` checks they point at real files.
sources:
  blueprint:
    root: ./blueprint
    handoff: ./blueprint/handoff.json     # the frozen Blueprint handoff
  design:
    root: ./design
    handoff: ./design/handoff.json         # the Design Handoff from Design OS

stepper:
  schema_version: 1
  generated_root: ./stepper

execution:
  max_parallel_steps: 4
  max_active_modules: 3
  max_fix_attempts: 5

planner:
  prioritize_critical_path: true
  prefer_module_locality: true

release:
  target: P0
  allow_critical_blockers: false
"""

MODULE_TEMPLATE = """\
module_id: MOD-001
name: Core
purpose: First module - replace with your real domain module.
priority: P0
risk_level: MEDIUM
"""

STEP_TEMPLATE = """\
step_id: STEP-000001
schema_version: 1
title: Prove the Stepper loop end to end
module: MOD-001
epic: ""
slice: ""
priority: P0
status: PENDING
weight: 1
risk:
  level: LOW
  reasons: []
objective:
  concise: Create the project marker file so the verify loop is proven.
  outcome: The Stepper start -> implement -> verify -> done loop runs green once.
# Each step names the exact upstream docs that GOVERN it. Blueprint = WHAT/WHY,
# Design = the UX/UI (flows, screens, states). The coding agent reads these
# before implementing. `doc` resolves against sources.<kind>.root above.
blueprint_references:
  - doc: handoff.json
    ids: [REQ-001]
    note: the product requirement this step realizes
design_references:
  - doc: handoff.json
    ids: [FLOW-001, SCREEN-001]
    note: the flow + screen contract this step must match
dependencies:
  hard: []
  soft: []
implementation_prompt: |
  Create a STEPPER.md file at the project root containing the project name.
  This first step exists to prove the execution loop before real work starts.
expected_files:
  create:
    - STEPPER.md
acceptance_checks:
  - type: file_exists
    path: STEPPER.md
acceptance_criteria:
  - STEPPER.md exists at the project root.
definition_of_done:
  - implementation_complete
  - acceptance_pass
"""


def init_project(root: Path, name: str) -> list[Path]:
    """Create the Stepper skeleton. Never overwrites an existing file."""
    created: list[Path] = []

    def write(path: Path, content: str) -> None:
        if path.exists():
            return
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content)
        created.append(path)

    write(root / "stepper.yaml", MANIFEST_TEMPLATE.format(name=name))
    for sub in ["modules", "epics", "slices", "steps"]:
        directory = root / "stepper" / sub
        directory.mkdir(parents=True, exist_ok=True)
    write(root / "stepper" / "modules" / "MOD-001.yaml", MODULE_TEMPLATE)
    write(root / "stepper" / "steps" / "STEP-000001.yaml", STEP_TEMPLATE)
    (root / "blueprint").mkdir(parents=True, exist_ok=True)
    return created

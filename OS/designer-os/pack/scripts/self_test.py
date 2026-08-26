#!/usr/bin/env python3
"""Deterministic smoke tests for Design OS validators."""

from __future__ import annotations

from copy import deepcopy

from validate_blueprint_intake import validate_document as validate_blueprint
from validate_design_handoff import REQUIRED_GATES, validate_document as validate_handoff


def blueprint_fixture() -> dict:
    return {
        "schemaVersion": "1.0",
        "project": {"name": "Specimen", "blueprintVersion": "B1", "targetSurfaces": ["web"]},
        "sources": [{"id": "SRC-001", "kind": "blueprint", "location": "blueprint.md"}],
        "requirements": [
            {
                "id": "REQ-001",
                "title": "Complete the primary task",
                "criticality": "critical",
                "status": "approved",
                "actorRefs": ["ACTOR-001"],
                "decisionRefs": ["DEC-001"],
            }
        ],
        "actors": [{"id": "ACTOR-001", "name": "Operator"}],
        "decisions": [{"id": "DEC-001", "status": "approved", "decision": "Web first"}],
        "unknowns": [],
    }


def handoff_fixture() -> dict:
    gates = [{"id": gate, "status": "pass", "evidenceRefs": ["EVAL-001"]} for gate in sorted(REQUIRED_GATES)]
    return {
        "schemaVersion": "1.0",
        "designOsVersion": "1.0",
        "project": {
            "name": "Specimen",
            "blueprintVersion": "B1",
            "designRevision": "D1",
            "targetSurfaces": ["web"],
        },
        "readiness": {"status": "STEPPER_READY", "gates": gates},
        "sources": [{"id": "SRC-001", "kind": "blueprint", "location": "blueprint.md"}],
        "principles": [{"id": "EXP-001", "title": "Visible state", "rule": "Name progress", "evalRefs": ["EVAL-001"]}],
        "decisions": [
            {
                "id": "DDEC-001",
                "title": "Shell",
                "status": "approved",
                "decision": "Use a route shell",
                "consequenceRefs": ["IA-001", "SURF-001"],
            }
        ],
        "flows": [
            {
                "id": "FLOW-001",
                "title": "Complete task",
                "priority": "P0",
                "outcome": "Task is complete",
                "requirementRefs": ["REQ-001"],
                "surfaceRefs": ["SURF-001"],
                "stateRefs": ["STATE-001"],
                "interactionRefs": ["INT-001"],
                "componentRefs": ["COMP-001"],
                "evalRefs": ["EVAL-001"],
                "nodes": [
                    {"id": "n1", "kind": "entry", "label": "Start", "surfaceRef": "SURF-001"},
                    {"id": "n2", "kind": "success", "label": "Done", "stateRef": "STATE-001"},
                ],
                "edges": [{"from": "n1", "to": "n2", "event": "submit"}],
            }
        ],
        "ia": [{"id": "IA-001", "title": "Route shell", "model": "route", "surfaceRefs": ["SURF-001"]}],
        "surfaces": [
            {
                "id": "SURF-001",
                "title": "Task",
                "kind": "page",
                "purpose": "Complete the task",
                "states": ["default", "loading", "empty", "error"],
                "flowRefs": ["FLOW-001"],
                "stateRefs": ["STATE-001"],
                "interactionRefs": ["INT-001"],
                "componentRefs": ["COMP-001"],
                "a11yRefs": ["A11Y-001"],
                "evalRefs": ["EVAL-001"],
            }
        ],
        "stateMachines": [
            {
                "id": "STATE-001",
                "title": "Submit",
                "initial": "idle",
                "states": ["idle", "done"],
                "transitions": [{"from": "idle", "event": "submit", "to": "done"}],
                "evalRefs": ["EVAL-001"],
            }
        ],
        "interactions": [{"id": "INT-001", "title": "Submit", "rule": "Submit once", "evalRefs": ["EVAL-001"]}],
        "tokens": [{"id": "TOK-001", "title": "Color", "roles": ["canvas", "text"]}],
        "components": [
            {
                "id": "COMP-001",
                "title": "Task form",
                "kind": "composition",
                "source": "shadcn_base_ui",
                "states": ["default", "loading", "error"],
                "flowRefs": ["FLOW-001"],
                "surfaceRefs": ["SURF-001"],
                "tokenRefs": ["TOK-001"],
                "a11yRefs": ["A11Y-001"],
                "evalRefs": ["EVAL-001"],
            }
        ],
        "accessibility": [
            {"id": "A11Y-001", "title": "Focus", "rule": "Restore focus", "evalRefs": ["EVAL-001"]}
        ],
        "evals": [
            {
                "id": "EVAL-001",
                "title": "Complete task",
                "category": "flow",
                "priority": "blocking",
                "refs": ["FLOW-001", "SURF-001"],
                "expected": "The task completes once",
                "oracle": "deterministic",
            }
        ],
        "traceability": [
            {
                "requirementRef": "REQ-001",
                "criticality": "critical",
                "flowRefs": ["FLOW-001"],
                "surfaceRefs": ["SURF-001"],
                "stateRefs": ["STATE-001"],
                "interactionRefs": ["INT-001"],
                "componentRefs": ["COMP-001"],
                "a11yRefs": ["A11Y-001"],
                "evalRefs": ["EVAL-001"],
                "coverage": "complete",
            }
        ],
        "risks": [],
        "unknowns": [],
        "stepperSeeds": [
            {
                "id": "SEED-001",
                "title": "Primary slice",
                "slice": "vertical_flow",
                "outcome": "FLOW-001 works end to end",
                "designRefs": ["FLOW-001", "SURF-001", "EVAL-001"],
                "dependsOn": [],
                "risk": "medium",
                "verification": ["e2e"],
            }
        ],
    }


def main() -> int:
    valid_blueprint_errors = validate_blueprint(blueprint_fixture())
    assert not valid_blueprint_errors, valid_blueprint_errors

    invalid_blueprint = blueprint_fixture()
    invalid_blueprint["requirements"][0]["status"] = "conflict"
    assert any("unresolved" in error for error in validate_blueprint(invalid_blueprint))

    valid_handoff_errors = validate_handoff(handoff_fixture())
    assert not valid_handoff_errors, valid_handoff_errors

    invalid_handoff = deepcopy(handoff_fixture())
    invalid_handoff["flows"][0]["evalRefs"] = []
    invalid_handoff["stepperSeeds"].append(
        {
            "id": "SEED-002",
            "title": "Cycle",
            "slice": "foundation",
            "outcome": "Cycle fixture",
            "designRefs": ["FLOW-001"],
            "dependsOn": ["SEED-002"],
            "risk": "low",
            "verification": ["deterministic"],
        }
    )
    invalid_errors = validate_handoff(invalid_handoff)
    assert any("requires non-empty evalRefs" in error for error in invalid_errors), invalid_errors
    assert any("dependency cycle" in error for error in invalid_errors), invalid_errors

    print("Design OS validator self-test passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""Validate Design OS's machine-readable handoff before Stepper consumption."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any, Iterable


REQUIRED_TOP_LEVEL = (
    "schemaVersion",
    "designOsVersion",
    "project",
    "readiness",
    "sources",
    "principles",
    "decisions",
    "flows",
    "ia",
    "surfaces",
    "stateMachines",
    "interactions",
    "tokens",
    "components",
    "accessibility",
    "evals",
    "traceability",
    "risks",
    "unknowns",
    "stepperSeeds",
)

COLLECTION_PREFIXES = {
    "principles": "EXP",
    "decisions": "DDEC",
    "flows": "FLOW",
    "ia": "IA",
    "surfaces": "SURF",
    "stateMachines": "STATE",
    "interactions": "INT",
    "tokens": "TOK",
    "components": "COMP",
    "accessibility": "A11Y",
    "evals": "EVAL",
    "risks": "RISK",
    "unknowns": "UNK",
    "stepperSeeds": "SEED",
}

REQUIRED_GATES = {
    "G-BP",
    "G-FLOW",
    "G-IA",
    "G-STATE",
    "G-ACTION",
    "G-AI",
    "G-DS",
    "G-RWD",
    "G-A11Y",
    "G-TRACE",
    "G-EVAL",
    "G-HANDOFF",
}

REF_FIELDS = {
    "principles": ("evalRefs",),
    "decisions": ("consequenceRefs",),
    "flows": ("surfaceRefs", "stateRefs", "interactionRefs", "componentRefs", "evalRefs"),
    "ia": ("surfaceRefs",),
    "surfaces": ("flowRefs", "stateRefs", "interactionRefs", "componentRefs", "a11yRefs", "evalRefs"),
    "stateMachines": ("evalRefs",),
    "interactions": ("evalRefs",),
    "components": ("flowRefs", "surfaceRefs", "tokenRefs", "a11yRefs", "evalRefs"),
    "accessibility": ("evalRefs",),
    "evals": ("refs",),
    "stepperSeeds": ("designRefs", "dependsOn"),
}

KNOWN_PREFIX = re.compile(r"^(EXP|DDEC|FLOW|IA|SURF|STATE|INT|TOK|COMP|A11Y|EVAL|RISK|UNK|SEED)-[0-9]{3,}$")


def _schema_errors(data: Any) -> list[str]:
    try:
        import jsonschema
    except ImportError:
        return []

    schema_path = Path(__file__).resolve().parent.parent / "references" / "design-handoff.schema.json"
    schema = json.loads(schema_path.read_text(encoding="utf-8"))
    validator = jsonschema.Draft202012Validator(schema)
    errors = []
    for error in sorted(validator.iter_errors(data), key=lambda item: list(item.absolute_path)):
        location = ".".join(str(part) for part in error.absolute_path) or "$"
        errors.append(f"schema {location}: {error.message}")
    return errors


def _refs(value: Any) -> Iterable[str]:
    if isinstance(value, list):
        for item in value:
            if isinstance(item, str):
                yield item


def _find_cycle(graph: dict[str, list[str]]) -> list[str] | None:
    visiting: set[str] = set()
    visited: set[str] = set()
    path: list[str] = []

    def visit(node: str) -> list[str] | None:
        if node in visiting:
            start = path.index(node)
            return path[start:] + [node]
        if node in visited:
            return None
        visiting.add(node)
        path.append(node)
        for dependency in graph.get(node, []):
            cycle = visit(dependency)
            if cycle:
                return cycle
        path.pop()
        visiting.remove(node)
        visited.add(node)
        return None

    for candidate in graph:
        cycle = visit(candidate)
        if cycle:
            return cycle
    return None


def validate_document(data: Any) -> list[str]:
    errors: list[str] = []
    if not isinstance(data, dict):
        return ["$: document must be a JSON object"]

    for key in REQUIRED_TOP_LEVEL:
        if key not in data:
            errors.append(f"$: missing required key {key!r}")
    if errors:
        return errors

    errors.extend(_schema_errors(data))

    if data.get("schemaVersion") != "1.0":
        errors.append("schemaVersion must equal '1.0'")

    collections = {name: data.get(name) for name in COLLECTION_PREFIXES}
    collections["sources"] = data.get("sources")
    collections["traceability"] = data.get("traceability")
    for name, value in collections.items():
        if not isinstance(value, list):
            errors.append(f"{name} must be an array")
    if errors:
        return sorted(set(errors))

    ids: dict[str, str] = {}
    collection_ids: dict[str, set[str]] = {}
    for collection, prefix in COLLECTION_PREFIXES.items():
        expected = re.compile(rf"^{re.escape(prefix)}-[0-9]{{3,}}$")
        current: set[str] = set()
        for index, item in enumerate(data[collection]):
            if not isinstance(item, dict):
                errors.append(f"{collection}[{index}] must be an object")
                continue
            item_id = item.get("id")
            if not isinstance(item_id, str) or not item_id:
                errors.append(f"{collection}[{index}].id must be a non-empty string")
                continue
            if not expected.match(item_id):
                errors.append(f"{collection}[{index}].id {item_id!r} must match {prefix}-###")
            if item_id in ids:
                errors.append(f"duplicate id {item_id!r} in {collection} and {ids[item_id]}")
            else:
                ids[item_id] = collection
            current.add(item_id)
        collection_ids[collection] = current

    eval_ids = collection_ids.get("evals", set())
    for collection, fields in REF_FIELDS.items():
        for index, item in enumerate(data[collection]):
            if not isinstance(item, dict):
                continue
            for field in fields:
                value = item.get(field, [])
                if not isinstance(value, list):
                    errors.append(f"{collection}[{index}].{field} must be an array")
                    continue
                for ref in _refs(value):
                    if KNOWN_PREFIX.match(ref) and ref not in ids:
                        errors.append(f"{collection}[{index}].{field} references missing id {ref!r}")

    for index, flow in enumerate(data["flows"]):
        if not isinstance(flow, dict):
            continue
        node_ids: set[str] = set()
        for node_index, node in enumerate(flow.get("nodes", [])):
            if not isinstance(node, dict):
                continue
            node_id = node.get("id")
            if not isinstance(node_id, str) or not node_id:
                errors.append(f"flows[{index}].nodes[{node_index}].id must be non-empty")
                continue
            if node_id in node_ids:
                errors.append(f"flows[{index}] has duplicate node id {node_id!r}")
            node_ids.add(node_id)
            for field, expected_collection in (("surfaceRef", "surfaces"), ("stateRef", "stateMachines")):
                ref = node.get(field)
                if isinstance(ref, str) and ref not in collection_ids.get(expected_collection, set()):
                    errors.append(f"flows[{index}].nodes[{node_index}].{field} references missing id {ref!r}")
        for edge_index, edge in enumerate(flow.get("edges", [])):
            if not isinstance(edge, dict):
                continue
            for endpoint in ("from", "to"):
                if edge.get(endpoint) not in node_ids:
                    errors.append(f"flows[{index}].edges[{edge_index}].{endpoint} references missing node {edge.get(endpoint)!r}")
        if flow.get("priority") == "P0":
            for field in ("surfaceRefs", "stateRefs", "evalRefs"):
                if not flow.get(field):
                    errors.append(f"P0 flow {flow.get('id', index)!r} requires non-empty {field}")

    for index, machine in enumerate(data["stateMachines"]):
        if not isinstance(machine, dict):
            continue
        states = set(item for item in machine.get("states", []) if isinstance(item, str))
        if machine.get("initial") not in states:
            errors.append(f"stateMachines[{index}].initial is not declared in states")
        for transition_index, transition in enumerate(machine.get("transitions", [])):
            if not isinstance(transition, dict):
                continue
            for endpoint in ("from", "to"):
                if transition.get(endpoint) not in states:
                    errors.append(
                        f"stateMachines[{index}].transitions[{transition_index}].{endpoint} "
                        f"references undeclared state {transition.get(endpoint)!r}"
                    )
        for ref in _refs(machine.get("evalRefs", [])):
            if ref not in eval_ids:
                errors.append(f"stateMachines[{index}].evalRefs references missing eval {ref!r}")

    for index, trace in enumerate(data["traceability"]):
        if not isinstance(trace, dict):
            continue
        for field in ("flowRefs", "surfaceRefs", "stateRefs", "interactionRefs", "componentRefs", "a11yRefs", "evalRefs"):
            for ref in _refs(trace.get(field, [])):
                if ref not in ids:
                    errors.append(f"traceability[{index}].{field} references missing id {ref!r}")

    seed_ids = collection_ids.get("stepperSeeds", set())
    seed_graph: dict[str, list[str]] = {}
    for index, seed in enumerate(data["stepperSeeds"]):
        if not isinstance(seed, dict) or not isinstance(seed.get("id"), str):
            continue
        dependencies = list(_refs(seed.get("dependsOn", [])))
        for dependency in dependencies:
            if dependency not in seed_ids:
                errors.append(f"stepperSeeds[{index}].dependsOn references missing seed {dependency!r}")
        seed_graph[seed["id"]] = dependencies
    cycle = _find_cycle(seed_graph)
    if cycle:
        errors.append(f"stepperSeeds dependency cycle: {' -> '.join(cycle)}")

    readiness = data.get("readiness")
    if not isinstance(readiness, dict):
        errors.append("readiness must be an object")
        return sorted(set(errors))

    gates = readiness.get("gates")
    if not isinstance(gates, list):
        errors.append("readiness.gates must be an array")
        gates = []
    gate_map: dict[str, str] = {}
    for index, gate in enumerate(gates):
        if not isinstance(gate, dict):
            errors.append(f"readiness.gates[{index}] must be an object")
            continue
        gate_id = gate.get("id")
        if gate_id in gate_map:
            errors.append(f"duplicate readiness gate {gate_id!r}")
        if isinstance(gate_id, str):
            gate_map[gate_id] = str(gate.get("status"))
        for ref in _refs(gate.get("evidenceRefs", [])):
            if KNOWN_PREFIX.match(ref) and ref not in ids:
                errors.append(f"readiness.gates[{index}].evidenceRefs references missing id {ref!r}")
    missing_gates = REQUIRED_GATES - set(gate_map)
    extra_gates = set(gate_map) - REQUIRED_GATES
    if missing_gates:
        errors.append(f"readiness.gates missing: {', '.join(sorted(missing_gates))}")
    if extra_gates:
        errors.append(f"readiness.gates contains unknown gates: {', '.join(sorted(extra_gates))}")

    if readiness.get("status") == "STEPPER_READY":
        for gate_id in sorted(REQUIRED_GATES):
            if gate_map.get(gate_id) not in {"pass", "not_applicable"}:
                errors.append(f"STEPPER_READY requires {gate_id} to pass or be not_applicable")
        for index, gate in enumerate(gates):
            if isinstance(gate, dict) and gate.get("status") == "not_applicable" and not gate.get("notes"):
                errors.append(f"readiness.gates[{index}] is not_applicable without notes")
        for index, trace in enumerate(data["traceability"]):
            if isinstance(trace, dict) and trace.get("criticality") == "critical" and trace.get("coverage") != "complete":
                errors.append(f"STEPPER_READY requires critical traceability[{index}] to be complete")
        for collection in ("unknowns", "risks"):
            for index, item in enumerate(data[collection]):
                if not isinstance(item, dict) or item.get("severity") != "critical":
                    continue
                if collection == "unknowns" and item.get("status") not in {"resolved", "rejected"}:
                    errors.append(f"STEPPER_READY has unresolved critical unknown {item.get('id', index)!r}")
                if collection == "risks" and item.get("status") == "open":
                    errors.append(f"STEPPER_READY has open critical risk {item.get('id', index)!r}")

    return sorted(set(errors))


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print("Usage: validate_design_handoff.py path/to/design-handoff.json", file=sys.stderr)
        return 2

    path = Path(argv[1])
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        print(f"ERROR: file not found: {path}", file=sys.stderr)
        return 2
    except json.JSONDecodeError as error:
        print(f"ERROR: invalid JSON at line {error.lineno}, column {error.colno}: {error.msg}", file=sys.stderr)
        return 2

    errors = validate_document(data)
    if errors:
        print(f"Design OS handoff invalid: {len(errors)} error(s)")
        for error in errors:
            print(f"- {error}")
        return 1

    print(
        "Design OS handoff valid: "
        f"status={data['readiness']['status']}, "
        f"flows={len(data['flows'])}, "
        f"surfaces={len(data['surfaces'])}, "
        f"evals={len(data['evals'])}, "
        f"stepperSeeds={len(data['stepperSeeds'])}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))

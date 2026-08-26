#!/usr/bin/env python3
"""Validate a machine-readable Blueprint intake before Design OS compilation."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any


REQUIRED_TOP_LEVEL = (
    "schemaVersion",
    "project",
    "sources",
    "requirements",
    "actors",
    "decisions",
    "unknowns",
)


def _schema_errors(data: Any) -> list[str]:
    try:
        import jsonschema
    except ImportError:
        return []

    schema_path = Path(__file__).resolve().parent.parent / "references" / "blueprint-intake.schema.json"
    schema = json.loads(schema_path.read_text(encoding="utf-8"))
    validator = jsonschema.Draft202012Validator(schema)
    errors = []
    for error in sorted(validator.iter_errors(data), key=lambda item: list(item.absolute_path)):
        location = ".".join(str(part) for part in error.absolute_path) or "$"
        errors.append(f"schema {location}: {error.message}")
    return errors


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

    collections = {
        "sources": data.get("sources"),
        "requirements": data.get("requirements"),
        "actors": data.get("actors"),
        "decisions": data.get("decisions"),
        "unknowns": data.get("unknowns"),
    }
    for name, value in collections.items():
        if not isinstance(value, list):
            errors.append(f"{name} must be an array")

    if errors:
        return sorted(set(errors))

    seen: dict[str, str] = {}
    by_collection: dict[str, set[str]] = {}
    for collection, items in collections.items():
        ids: set[str] = set()
        for index, item in enumerate(items):
            if not isinstance(item, dict):
                errors.append(f"{collection}[{index}] must be an object")
                continue
            item_id = item.get("id")
            if not isinstance(item_id, str) or not item_id:
                errors.append(f"{collection}[{index}].id must be a non-empty string")
                continue
            if item_id in seen:
                errors.append(f"duplicate id {item_id!r} in {collection} and {seen[item_id]}")
            else:
                seen[item_id] = collection
            ids.add(item_id)
        by_collection[collection] = ids

    actor_ids = by_collection.get("actors", set())
    decision_ids = by_collection.get("decisions", set())
    for index, requirement in enumerate(data.get("requirements", [])):
        if not isinstance(requirement, dict):
            continue
        for actor_ref in requirement.get("actorRefs", []):
            if actor_ref not in actor_ids:
                errors.append(f"requirements[{index}].actorRefs references missing actor {actor_ref!r}")
        for decision_ref in requirement.get("decisionRefs", []):
            if decision_ref not in decision_ids:
                errors.append(f"requirements[{index}].decisionRefs references missing decision {decision_ref!r}")

    critical_conflicts = [
        item.get("id", f"requirements[{index}]")
        for index, item in enumerate(data.get("requirements", []))
        if isinstance(item, dict)
        and item.get("criticality") == "critical"
        and item.get("status") in {"unknown", "conflict"}
    ]
    for item_id in critical_conflicts:
        errors.append(f"critical requirement {item_id!r} is unresolved")

    return sorted(set(errors))


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print("Usage: validate_blueprint_intake.py path/to/blueprint-intake.json", file=sys.stderr)
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
        print(f"Blueprint intake invalid: {len(errors)} error(s)")
        for error in errors:
            print(f"- {error}")
        return 1

    print(
        "Blueprint intake valid: "
        f"{len(data['requirements'])} requirements, "
        f"{len(data['actors'])} actors, "
        f"{len(data['unknowns'])} unknowns"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))

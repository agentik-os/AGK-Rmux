#!/usr/bin/env python3
"""Deterministic local state helper for Blueprint {OS}.

This CLI initializes, validates, checkpoints, and reports on the portable JSON
state contract. It intentionally does not perform product reasoning.
"""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import re
import sys
from pathlib import Path
from typing import Any


ALLOWED_STATUSES = {
    "BLUEPRINT IN PROGRESS",
    "BLUEPRINT BLOCKED",
    "BLUEPRINT COMPLETE — STEPPER READY",
}
ALLOWED_MODES = {"NEW", "RECOVER", "EXTEND", "REVISE", "AUDIT", "DELTA"}
ALLOWED_GATE_RESULTS = {"PASS", "CONDITIONAL", "FAIL", "N/A"}
CRITICAL_GATES = {"G02", "G04", "G05", "G06", "G08", "G09", "G10", "G12", "G13", "G15", "G17", "G18"}
ID_RE = re.compile(r"^[A-Z]{2,8}-[0-9]{3,}$")
SEMVER_RE = re.compile(r"^[0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?$")
RELATIONS = {
    "derived_from", "decides", "satisfies", "depends_on", "conflicts_with",
    "supersedes", "realized_by", "reads", "writes", "emits", "consumes",
    "authorized_by", "verified_by", "measured_by", "mitigated_by", "blocks",
}


class BlueprintError(Exception):
    pass


def utc_now() -> str:
    return dt.datetime.now(dt.timezone.utc).replace(microsecond=0).isoformat()


def canonical_payload(state: dict[str, Any]) -> bytes:
    clone = json.loads(json.dumps(state))
    clone.get("meta", {}).pop("checksum", None)
    clone.get("meta", {}).pop("updated_at", None)
    clone.pop("exports", None)
    return json.dumps(clone, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")


def checksum(state: dict[str, Any]) -> str:
    return "sha256:" + hashlib.sha256(canonical_payload(state)).hexdigest()


def read_state(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise BlueprintError(f"State file not found: {path}") from exc
    except json.JSONDecodeError as exc:
        raise BlueprintError(f"Invalid JSON in {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise BlueprintError("State root must be a JSON object")
    return value


def atomic_write(path: Path, state: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    state["meta"]["updated_at"] = utc_now()
    state["meta"]["checksum"] = checksum(state)
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(json.dumps(state, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    temporary.replace(path)


def initial_state(args: argparse.Namespace) -> dict[str, Any]:
    if args.mode not in ALLOWED_MODES:
        raise BlueprintError(f"Unsupported mode: {args.mode}")
    if not SEMVER_RE.match(args.version):
        raise BlueprintError(f"Invalid semantic version: {args.version}")
    timestamp = utc_now()
    run_id = args.run_id or f"BPR-{hashlib.sha256((args.project_id + timestamp).encode()).hexdigest()[:12]}"
    state: dict[str, Any] = {
        "meta": {
            "project_id": args.project_id,
            "project_name": args.project_name,
            "namespace": args.namespace,
            "version": args.version,
            "status": "BLUEPRINT IN PROGRESS",
            "revision": 0,
            "checksum": None,
            "id_counters": {},
            "created_at": timestamp,
            "updated_at": timestamp,
        },
        "run": {
            "run_id": run_id,
            "mode": args.mode,
            "request": args.request,
            "locale": args.locale,
            "audience": args.audience,
            "baseline_revision": 0,
        },
        "sources": [],
        "records": [],
        "trace_links": [],
        "findings": [],
        "gates": [],
        "continuation": {
            "part": 1,
            "estimated_parts": None,
            "completed_sections": [],
            "current_section": "00 Run Manifest and Status",
            "next_exact_section": "01 Executive Product Truth",
            "remaining_sections": [
                "01 Executive Product Truth",
                "02 Source and Evidence Ledger",
                "03 Epistemic Ledgers",
                "04 Vocabulary and Concept Map",
                "05 Vision, Thesis, Principles, Non-goals",
                "06 Market, Alternatives, and Positioning",
                "07 Stakeholders, Personas, and JTBD",
                "08 Value Architecture and Business Model",
                "09 Goals, Metrics, Guardrails, Counter-metrics",
                "10 Scope, Release Boundaries, and Capability Map",
                "11 Actor, Identity, Role, Permission, and Consent Model",
                "12 Requirement Catalog",
                "13 Action Contract Catalog",
                "14 End-to-end User and Operator Flows",
                "15 Information Architecture and Navigation",
                "16 Screen and Surface Contracts",
                "17 Design System and Content Rules",
                "18 Domain Model and Bounded Contexts",
                "19 State Machines, Rules, and Invariants",
                "20 System Context and Trust Boundaries",
                "21 Application and Deployment Architecture",
                "22 Architecture Decision Records",
                "23 Data Dictionary, Ownership, and Lifecycle",
                "24 API, Tool, Integration, and Event Contracts",
                "25 AI and Agent Architecture",
                "26 Security, Privacy, Threat, and Abuse Model",
                "27 Non-functional Requirements",
                "28 Operational Model",
                "29 Acceptance and Test Architecture",
                "30 Analytics, Instrumentation, and Learning Loops",
                "31 Risk Register",
                "32 Release Definition and Validation Strategy",
                "33 Traceability Matrix and Orphan Report",
                "34 Critic Findings and Dispositions",
                "35 Quality Gate Scorecard",
                "36 Stepper Input Manifest",
                "37 Continuation and Change Ledger",
                "38 Final Declaration",
            ],
            "unresolved_blockers": [],
            "changed_record_ids": [],
            "checkpoint_revision": 0,
        },
        "exports": [],
        "handoff": None,
    }
    state["meta"]["checksum"] = checksum(state)
    return state


def validate(state: dict[str, Any]) -> list[dict[str, str]]:
    issues: list[dict[str, str]] = []

    def issue(code: str, severity: str, message: str) -> None:
        issues.append({"code": code, "severity": severity, "message": message})

    required_top = {"meta", "run", "sources", "records", "trace_links", "findings", "gates", "continuation", "exports"}
    missing = sorted(required_top - set(state))
    if missing:
        issue("SCHEMA_TOP", "critical", f"Missing top-level keys: {', '.join(missing)}")
        return issues

    meta = state.get("meta", {})
    if meta.get("status") not in ALLOWED_STATUSES:
        issue("STATUS", "critical", f"Invalid Blueprint status: {meta.get('status')!r}")
    if not SEMVER_RE.match(str(meta.get("version", ""))):
        issue("VERSION", "high", f"Invalid semantic version: {meta.get('version')!r}")
    if state.get("run", {}).get("mode") not in ALLOWED_MODES:
        issue("MODE", "high", f"Invalid run mode: {state.get('run', {}).get('mode')!r}")

    all_ids: set[str] = set()
    for collection_name in ("sources", "records", "findings"):
        for item in state.get(collection_name, []):
            record_id = item.get("id")
            if not isinstance(record_id, str) or not ID_RE.match(record_id):
                issue("ID_FORMAT", "high", f"Invalid ID in {collection_name}: {record_id!r}")
                continue
            if record_id in all_ids:
                issue("ID_DUPLICATE", "critical", f"Duplicate canonical ID: {record_id}")
            all_ids.add(record_id)

    source_ids = {item.get("id") for item in state.get("sources", [])}
    for record in state.get("records", []):
        for source_id in record.get("sources", []):
            if source_id not in source_ids:
                issue("SOURCE_REF", "high", f"{record.get('id')} references missing source {source_id}")
        for dependency in record.get("dependencies", []):
            if dependency not in all_ids:
                issue("DEPENDENCY_REF", "high", f"{record.get('id')} depends on missing ID {dependency}")

    edge_keys: set[tuple[str, str, str]] = set()
    for edge in state.get("trace_links", []):
        from_id, relation, to_id = edge.get("from_id"), edge.get("relation"), edge.get("to_id")
        if from_id not in all_ids:
            issue("TRACE_FROM", "high", f"Trace source does not exist: {from_id}")
        if to_id not in all_ids:
            issue("TRACE_TO", "high", f"Trace target does not exist: {to_id}")
        if relation not in RELATIONS:
            issue("TRACE_RELATION", "medium", f"Unsupported trace relation: {relation}")
        key = (str(from_id), str(relation), str(to_id))
        if key in edge_keys:
            issue("TRACE_DUPLICATE", "low", f"Duplicate trace edge: {key}")
        edge_keys.add(key)

    active_records = [r for r in state.get("records", []) if r.get("status") in {"accepted", "validated"}]
    normative = [r for r in active_records if r.get("kind") in {"decision", "requirement", "invariant", "rule"}]
    linked = {e.get("from_id") for e in state.get("trace_links", [])} | {e.get("to_id") for e in state.get("trace_links", [])}
    for record in normative:
        if record.get("id") not in linked:
            issue("ORPHAN_NORMATIVE", "high", f"Normative record has no trace links: {record.get('id')}")

    gate_map: dict[str, str] = {}
    for gate in state.get("gates", []):
        gate_id, result = gate.get("gate_id"), gate.get("result")
        if result not in ALLOWED_GATE_RESULTS:
            issue("GATE_RESULT", "high", f"Invalid result for {gate_id}: {result}")
        gate_map[str(gate_id)] = str(result)

    if meta.get("status") == "BLUEPRINT COMPLETE — STEPPER READY":
        missing_gates = [f"G{i:02d}" for i in range(1, 21) if f"G{i:02d}" not in gate_map]
        if missing_gates:
            issue("GATES_MISSING", "critical", f"Completion lacks gates: {', '.join(missing_gates)}")
        failed_critical = sorted(g for g in CRITICAL_GATES if gate_map.get(g) == "FAIL")
        if failed_critical:
            issue("CRITICAL_GATE_FAIL", "critical", f"Critical gates failed: {', '.join(failed_critical)}")
        if state.get("handoff") is None:
            issue("HANDOFF_MISSING", "critical", "Completion requires a frozen Stepper handoff")
        if state.get("continuation", {}).get("remaining_sections"):
            issue("SECTIONS_REMAIN", "critical", "Completion has mandatory sections remaining")

    stored = meta.get("checksum")
    calculated = checksum(state)
    if stored and stored != calculated:
        issue("CHECKSUM", "medium", "Stored checksum does not match canonical state")

    return issues


def command_init(args: argparse.Namespace) -> int:
    target = Path(args.path).resolve()
    if target.exists() and not args.force:
        raise BlueprintError(f"Refusing to overwrite existing file: {target}. Use --force to replace exactly this file.")
    state = initial_state(args)
    atomic_write(target, state)
    print(json.dumps({"ok": True, "path": str(target), "run_id": state["run"]["run_id"], "checksum": state["meta"]["checksum"]}, indent=2))
    return 0


def command_validate(args: argparse.Namespace) -> int:
    state = read_state(Path(args.path))
    issues = validate(state)
    summary = {
        "ok": not any(i["severity"] in {"critical", "high"} for i in issues),
        "issue_count": len(issues),
        "issues": issues,
        "calculated_checksum": checksum(state),
    }
    print(json.dumps(summary, indent=2, ensure_ascii=False))
    return 0 if summary["ok"] else 1


def command_status(args: argparse.Namespace) -> int:
    state = read_state(Path(args.path))
    gate_counts = {result: 0 for result in sorted(ALLOWED_GATE_RESULTS)}
    for gate in state.get("gates", []):
        result = gate.get("result")
        if result in gate_counts:
            gate_counts[result] += 1
    payload = {
        "project": state["meta"].get("project_name"),
        "project_id": state["meta"].get("project_id"),
        "version": state["meta"].get("version"),
        "revision": state["meta"].get("revision"),
        "status": state["meta"].get("status"),
        "run_id": state["run"].get("run_id"),
        "mode": state["run"].get("mode"),
        "records": len(state.get("records", [])),
        "sources": len(state.get("sources", [])),
        "trace_links": len(state.get("trace_links", [])),
        "open_findings": sum(1 for f in state.get("findings", []) if f.get("status") == "open"),
        "gates": gate_counts,
        "current_section": state["continuation"].get("current_section"),
        "next_exact_section": state["continuation"].get("next_exact_section"),
        "remaining_sections": len(state["continuation"].get("remaining_sections", [])),
        "checksum": state["meta"].get("checksum"),
    }
    print(json.dumps(payload, indent=2, ensure_ascii=False))
    return 0


def command_checkpoint(args: argparse.Namespace) -> int:
    path = Path(args.path)
    state = read_state(path)
    state["meta"]["revision"] = int(state["meta"].get("revision", 0)) + 1
    state["meta"]["status"] = args.status
    continuation = state["continuation"]
    continuation["part"] = args.part or continuation.get("part", 1)
    continuation["current_section"] = args.current
    continuation["next_exact_section"] = args.next
    continuation["checkpoint_revision"] = state["meta"]["revision"]
    atomic_write(path, state)
    print(json.dumps({"ok": True, "revision": state["meta"]["revision"], "checksum": state["meta"]["checksum"]}, indent=2))
    return 0


def command_demo(args: argparse.Namespace) -> int:
    demo_args = argparse.Namespace(
        project_id="demo-product",
        project_name="Demo Product",
        namespace="demo.product",
        version="0.1.0",
        mode="NEW",
        request="Compile a demonstration Blueprint",
        locale="en",
        audience="product and engineering",
        run_id="BPR-DEMO000001",
    )
    state = initial_state(demo_args)
    print(json.dumps(state, indent=2, ensure_ascii=False))
    return 0


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(description="Blueprint {OS} portable state helper")
    sub = root.add_subparsers(dest="command", required=True)

    init = sub.add_parser("init", help="Initialize a canonical state file")
    init.add_argument("path")
    init.add_argument("--project-id", required=True)
    init.add_argument("--project-name", required=True)
    init.add_argument("--namespace", required=True)
    init.add_argument("--version", default="0.1.0")
    init.add_argument("--mode", choices=sorted(ALLOWED_MODES), default="NEW")
    init.add_argument("--request", required=True)
    init.add_argument("--locale", default="en")
    init.add_argument("--audience", default="product and engineering")
    init.add_argument("--run-id")
    init.add_argument("--force", action="store_true")
    init.set_defaults(func=command_init)

    check = sub.add_parser("validate", help="Validate state and semantic invariants")
    check.add_argument("path")
    check.set_defaults(func=command_validate)

    status = sub.add_parser("status", help="Show concise Blueprint status")
    status.add_argument("path")
    status.set_defaults(func=command_status)

    checkpoint = sub.add_parser("checkpoint", help="Advance revision and save continuation pointer")
    checkpoint.add_argument("path")
    checkpoint.add_argument("--status", choices=sorted(ALLOWED_STATUSES), default="BLUEPRINT IN PROGRESS")
    checkpoint.add_argument("--part", type=int)
    checkpoint.add_argument("--current", required=True)
    checkpoint.add_argument("--next", required=True)
    checkpoint.set_defaults(func=command_checkpoint)

    demo = sub.add_parser("demo", help="Print a valid minimal demonstration state")
    demo.set_defaults(func=command_demo)
    return root


def main() -> int:
    args = parser().parse_args()
    try:
        return int(args.func(args))
    except BlueprintError as exc:
        print(json.dumps({"ok": False, "error": str(exc)}), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())

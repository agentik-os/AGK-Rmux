#!/usr/bin/env python3
"""Deterministic state/checkpoint/validation support for Market Research {OS}.

Standard-library only. Semantic research judgment remains the responsibility of
the OS critics and decision owner; this CLI validates machine-checkable rules.
"""

from __future__ import annotations

import argparse
import copy
import datetime as dt
import hashlib
import json
import os
import re
import sys
import tempfile
import uuid
from pathlib import Path
from typing import Any


SCHEMA_VERSION = "1.0.0"
STATUSES = {
    "MARKET RESEARCH IN PROGRESS",
    "MARKET RESEARCH BLOCKED",
    "MARKET RESEARCH COMPLETE — DECISION READY",
}
MODES = {
    "NEW", "RECOVER", "RAPID_SCAN", "FULL_VALIDATION", "DILIGENCE",
    "DEEP_DIVE", "MONITOR", "AUDIT", "DELTA",
}
DEPTHS = {"SIGNAL", "VALIDATION", "INVESTMENT_GRADE"}
DECISIONS = {"GO", "PIVOT", "HOLD", "NO-GO", "INSUFFICIENT EVIDENCE"}
PREFIXES = [
    "SRC", "FCT", "MEA", "INF", "ASM", "HYP", "DEC", "PRP", "UNK",
    "CNF", "LIM", "NEG", "RQ", "QST", "SEG", "JTBD", "ALT", "CMP",
    "SIG", "DAT", "MTH", "SAM", "INT", "SUR", "EXP", "OBS", "EST",
    "MOD", "SCN", "PRC", "ECO", "CHN", "RSK", "MIT", "GATE", "REC",
    "BPH",
]
ID_PATTERN = re.compile(
    r"^(?:" + "|".join(re.escape(p) for p in PREFIXES) + r")-[0-9]{3,}$"
)
GATE_NAMES = [
    "Decision framing",
    "Context recovery",
    "Epistemic integrity",
    "Research-design fitness",
    "Source legality, ethics, and access",
    "Source coverage, freshness, and independence",
    "Category, environment, and timing",
    "Market-sizing integrity",
    "Segment, JTBD, and buying-system evidence",
    "Voice-of-customer quality",
    "Competition and alternatives",
    "Demand-signal interpretation",
    "Offer and feature evidence",
    "Pricing and economic viability",
    "GTM and channel plausibility",
    "Primary-research quality",
    "Behavioral and commercial validation",
    "Data quality and reproducibility",
    "Bias, conflict, and negative evidence",
    "Risk, scenario, and pre-mortem",
    "Traceability and orphan control",
    "Decision threshold and condition integrity",
    "Blueprint handoff integrity",
    "Artifact and continuation integrity",
]
CRITICAL_GATES = {1, 3, 4, 5, 8, 9, 11, 14, 17, 18, 19, 21, 22, 23, 24}
ARTIFACT_NAMES = [
    "Run Manifest and Decision Brief",
    "Executive Decision Memo",
    "Recovered Context and Source Ledger",
    "Epistemic Ledgers",
    "Research Question and Hypothesis Register",
    "Research Design and Evidence Plan",
    "Data Acquisition, Rights, Privacy, and Ethics Plan",
    "Market and Category Definition",
    "Macro, Ecosystem, Value Chain, and Timing",
    "Market Size and Growth Model",
    "Segment and Beachhead Model",
    "Persona, JTBD, and Buying-System Contracts",
    "Voice-of-Customer Evidence Corpus",
    "Alternatives and Competitive Intelligence",
    "Demand and Trend Signal Dashboard",
    "Opportunity, Offer, and Feature Evidence Map",
    "Pricing and Willingness-to-Pay Study",
    "Business Model and Unit-Economics Model",
    "Positioning and Go-to-Market Evidence",
    "Primary Research Instruments and Results",
    "Validation Experiment Portfolio",
    "Risk, Scenario, and Pre-mortem Register",
    "Hypothesis and Evidence Scorecard",
    "Critic Findings and Dispositions",
    "Traceability Matrix and Orphan Report",
    "Quality Gate Scorecard",
    "Recommendation and Decision Contract",
    "Blueprint Input Manifest",
    "Monitoring and Refresh Plan",
    "Continuation and Change Ledger",
    "Final Declaration",
]
COLLECTIONS = [
    "sources", "preflights", "research_questions", "hypotheses", "findings",
    "methods", "query_plans", "acquisition_runs", "segments", "jtbd",
    "alternatives", "competitors", "signals", "studies", "experiments",
    "models", "scenarios", "pricing", "economics", "channels", "risks",
    "critic_findings", "trace_links", "gates", "recommendations", "handoffs",
    "artifacts",
]


def utc_now() -> str:
    return dt.datetime.now(dt.timezone.utc).replace(microsecond=0).isoformat()


def slug(value: str) -> str:
    result = re.sub(r"[^a-z0-9]+", "-", value.lower()).strip("-")
    return result or "research"


def canonical_bytes(state: dict[str, Any]) -> bytes:
    candidate = copy.deepcopy(state)
    candidate["checksum"] = ""
    return json.dumps(
        candidate, ensure_ascii=False, sort_keys=True, separators=(",", ":")
    ).encode("utf-8")


def checksum(state: dict[str, Any]) -> str:
    return "sha256:" + hashlib.sha256(canonical_bytes(state)).hexdigest()


def state_file(path: str | Path) -> Path:
    p = Path(path).expanduser().resolve()
    return p if p.suffix == ".json" else p / "state.json"


def load_state(path: str | Path) -> dict[str, Any]:
    target = state_file(path)
    if not target.is_file():
        raise FileNotFoundError(f"State not found: {target}")
    with target.open("r", encoding="utf-8") as handle:
        data = json.load(handle)
    if not isinstance(data, dict):
        raise ValueError("State root must be an object")
    return data


def atomic_write_json(path: Path, data: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, tmp_name = tempfile.mkstemp(prefix=path.name + ".", suffix=".tmp", dir=path.parent)
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as handle:
            json.dump(data, handle, ensure_ascii=False, indent=2, sort_keys=False)
            handle.write("\n")
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(tmp_name, path)
    finally:
        if os.path.exists(tmp_name):
            os.unlink(tmp_name)


def save_state(path: str | Path, state: dict[str, Any], bump: bool = True) -> Path:
    target = state_file(path)
    if bump:
        state["revision"] = int(state.get("revision", 0)) + 1
        state["updated_at"] = utc_now()
    state["checksum"] = checksum(state)
    atomic_write_json(target, state)
    return target


def initial_state(args: argparse.Namespace) -> dict[str, Any]:
    timestamp = utc_now()
    project_slug = slug(args.project_id)
    run_id = f"MRR-{project_slug}-{dt.datetime.now(dt.timezone.utc):%Y%m%d}-{uuid.uuid4().hex[:8]}"
    artifacts = [
        {"id": f"ART-{index:02d}", "name": name, "status": "pending", "state_revision": 0}
        for index, name in enumerate(ARTIFACT_NAMES)
    ]
    artifacts[0]["status"] = "in-progress"
    gates = [
        {
            "id": f"G{index:02d}",
            "name": name,
            "status": "NOT_EVALUATED",
            "critical": index in CRITICAL_GATES,
            "evidence_ids": [],
            "rationale": "",
            "condition": "",
            "owner": "",
        }
        for index, name in enumerate(GATE_NAMES, 1)
    ]
    state: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "run": {
            "run_id": run_id,
            "project_id": args.project_id,
            "project_name": args.project_name,
            "research_version": "0.1.0",
            "mode": args.mode,
            "depth": args.depth,
            "status": "MARKET RESEARCH IN PROGRESS",
            "evidence_cutoff": timestamp,
            "external_action_authority": args.external_action_authority,
            "confidentiality": args.confidentiality,
        },
        "decision_brief": {
            "decision_question": args.decision,
            "decision_owner": args.decision_owner,
            "decision_options": ["GO", "PIVOT", "HOLD", "NO-GO", "INSUFFICIENT EVIDENCE"],
            "geographies": [],
            "segments_in_scope": [],
            "segments_out_of_scope": [],
            "success_thresholds": [],
            "kill_thresholds": [],
            "decision_due": None,
            "research_expiry": None,
        },
        "id_counters": {prefix: 0 for prefix in PREFIXES},
        "continuation": {
            "completed_artifacts": [],
            "current_artifact": "00 — Run Manifest and Decision Brief",
            "next_exact_section": "00.1 — Complete the Decision Brief",
            "remaining_mandatory_artifacts": [f"{i:02d}" for i in range(len(ARTIFACT_NAMES))],
            "blockers": [],
            "gate_snapshot": {gate["id"]: gate["status"] for gate in gates},
        },
        "revision": 0,
        "checksum": "",
        "created_at": timestamp,
        "updated_at": timestamp,
    }
    for collection in COLLECTIONS:
        state[collection] = []
    state["artifacts"] = artifacts
    state["gates"] = gates
    state["checksum"] = checksum(state)
    return state


def collect_ids(state: dict[str, Any]) -> tuple[dict[str, str], list[str]]:
    seen: dict[str, str] = {}
    duplicates: list[str] = []
    for collection in COLLECTIONS:
        if collection in {"gates", "artifacts", "trace_links"}:
            continue
        for record in state.get(collection, []):
            if not isinstance(record, dict):
                continue
            record_id = record.get("id")
            if not isinstance(record_id, str):
                continue
            if record_id in seen:
                duplicates.append(record_id)
            else:
                seen[record_id] = collection
    return seen, duplicates


def validate_state(state: dict[str, Any], strict: bool = False) -> dict[str, list[str]]:
    errors: list[str] = []
    warnings: list[str] = []
    required = {
        "schema_version", "run", "decision_brief", "id_counters", "sources",
        "preflights", "hypotheses", "findings", "trace_links", "gates",
        "continuation", "revision", "checksum",
    }
    missing = sorted(required - set(state))
    if missing:
        errors.append("Missing root fields: " + ", ".join(missing))
        return {"errors": errors, "warnings": warnings}

    if state.get("schema_version") != SCHEMA_VERSION:
        errors.append(f"schema_version must be {SCHEMA_VERSION}")
    run = state.get("run", {})
    if run.get("status") not in STATUSES:
        errors.append("Invalid run status")
    if run.get("mode") not in MODES:
        errors.append("Invalid run mode")
    if run.get("depth") not in DEPTHS:
        errors.append("Invalid run depth")
    if run.get("external_action_authority") not in {"none", "research-only", "approved-scope"}:
        errors.append("Invalid external_action_authority")
    if not state.get("decision_brief", {}).get("decision_question"):
        errors.append("Decision question is required")

    expected_checksum = checksum(state)
    if state.get("checksum") != expected_checksum:
        errors.append("Checksum mismatch")

    ids, duplicates = collect_ids(state)
    if duplicates:
        errors.append("Duplicate IDs: " + ", ".join(sorted(set(duplicates))))
    for record_id, collection in ids.items():
        if not ID_PATTERN.match(record_id):
            errors.append(f"Invalid ID {record_id} in {collection}")

    for prefix, value in state.get("id_counters", {}).items():
        if prefix not in PREFIXES or not isinstance(value, int) or value < 0:
            errors.append(f"Invalid ID counter {prefix}={value!r}")

    source_ids = {r.get("id") for r in state.get("sources", []) if isinstance(r, dict)}
    preflight_by_id = {r.get("id"): r for r in state.get("preflights", []) if isinstance(r, dict)}
    for source in state.get("sources", []):
        if not isinstance(source, dict):
            errors.append("Source must be an object")
            continue
        for field in ("id", "title", "source_type", "authority", "retrieved_at", "access_method", "rights_basis", "privacy_class", "fingerprint", "limitations"):
            if field not in source:
                errors.append(f"Source {source.get('id', '?')} missing {field}")
    for preflight_id, preflight in preflight_by_id.items():
        if preflight.get("decision") not in {"ALLOW", "ALLOW_WITH_CONTROLS", "MANUAL_ONLY", "REQUIRES_PERMISSION", "PROHIBITED"}:
            errors.append(f"Preflight {preflight_id} has invalid decision")

    for finding in state.get("findings", []):
        if not isinstance(finding, dict):
            errors.append("Finding must be an object")
            continue
        ftype = finding.get("type")
        if ftype in {"FACT", "MEASUREMENT"} and not finding.get("source_ids"):
            errors.append(f"{finding.get('id', '?')} {ftype} has no source_ids")
        for source_id in finding.get("source_ids", []):
            if source_id not in source_ids:
                errors.append(f"{finding.get('id', '?')} references unknown source {source_id}")
        confidence = finding.get("confidence")
        if confidence is not None and not (isinstance(confidence, (int, float)) and 0 <= confidence <= 1):
            errors.append(f"{finding.get('id', '?')} confidence must be 0..1")

    for hypothesis in state.get("hypotheses", []):
        if not isinstance(hypothesis, dict):
            errors.append("Hypothesis must be an object")
            continue
        for field in ("statement", "falsifier", "pass_threshold", "fail_threshold", "decision_criticality"):
            if not hypothesis.get(field):
                errors.append(f"Hypothesis {hypothesis.get('id', '?')} missing {field}")
        if hypothesis.get("decision_criticality") == "P0" and hypothesis.get("status") == "untested":
            warnings.append(f"Critical hypothesis {hypothesis.get('id')} is untested")

    for run_record in state.get("acquisition_runs", []):
        if not isinstance(run_record, dict):
            errors.append("Acquisition run must be an object")
            continue
        preflight_id = run_record.get("preflight_id")
        preflight = preflight_by_id.get(preflight_id)
        if not preflight:
            errors.append(f"Acquisition run {run_record.get('id', '?')} lacks known preflight")
        elif preflight.get("decision") not in {"ALLOW", "ALLOW_WITH_CONTROLS"}:
            errors.append(f"Acquisition run {run_record.get('id', '?')} used blocked preflight {preflight_id}")

    for model in state.get("models", []):
        if not isinstance(model, dict):
            errors.append("Model must be an object")
            continue
        if not model.get("formula"):
            errors.append(f"Model {model.get('id', '?')} has no formula")
        for model_input in model.get("inputs", []):
            ref = model_input.get("source_or_assumption_id") if isinstance(model_input, dict) else None
            if ref and ref not in ids:
                errors.append(f"Model {model.get('id', '?')} input references unknown {ref}")
            if isinstance(model_input, dict) and "value" in model_input and "unit" not in model_input:
                warnings.append(f"Model {model.get('id', '?')} input {model_input.get('name', '?')} lacks unit")

    for experiment in state.get("experiments", []):
        if not isinstance(experiment, dict):
            errors.append("Experiment must be an object")
            continue
        for field in ("hypothesis_ids", "primary_metric", "pass_threshold", "fail_threshold", "sample_rule", "stopping_rule", "authorization"):
            if not experiment.get(field):
                errors.append(f"Experiment {experiment.get('id', '?')} missing {field}")
        if experiment.get("status") in {"running", "analyzed", "passed", "failed", "ambiguous"} and experiment.get("authorization") in {None, "", "none"}:
            errors.append(f"Experiment {experiment.get('id', '?')} executed without authority")

    known_ids = set(ids) | {gate.get("id") for gate in state.get("gates", []) if isinstance(gate, dict)}
    for link in state.get("trace_links", []):
        if not isinstance(link, dict):
            errors.append("Trace link must be an object")
            continue
        if link.get("from_id") not in known_ids:
            errors.append(f"Trace from_id unknown: {link.get('from_id')}")
        if link.get("to_id") not in known_ids:
            errors.append(f"Trace to_id unknown: {link.get('to_id')}")
        if not link.get("relation"):
            errors.append("Trace link missing relation")

    gate_by_id = {g.get("id"): g for g in state.get("gates", []) if isinstance(g, dict)}
    if set(gate_by_id) != {f"G{i:02d}" for i in range(1, 25)}:
        errors.append("Exactly gates G01..G24 are required")
    for gate_id, gate in gate_by_id.items():
        if gate.get("status") not in {"PASS", "CONDITIONAL", "FAIL", "N/A", "NOT_EVALUATED"}:
            errors.append(f"Invalid status for {gate_id}")
        if gate.get("status") == "CONDITIONAL" and not gate.get("condition"):
            errors.append(f"{gate_id} is conditional without a condition")

    artifact_statuses = {a.get("status") for a in state.get("artifacts", []) if isinstance(a, dict)}
    if not artifact_statuses <= {"pending", "in-progress", "complete", "n/a", "stale"}:
        errors.append("Invalid artifact status")
    continuation = state.get("continuation", {})
    for field in ("completed_artifacts", "current_artifact", "next_exact_section", "remaining_mandatory_artifacts", "blockers", "gate_snapshot"):
        if field not in continuation:
            errors.append(f"Continuation missing {field}")

    if run.get("status") == "MARKET RESEARCH COMPLETE — DECISION READY":
        incomplete = [a.get("id") for a in state.get("artifacts", []) if a.get("status") not in {"complete", "n/a"}]
        if incomplete:
            errors.append("Complete status with incomplete artifacts: " + ", ".join(incomplete))
        failed_critical = [g.get("id") for g in state.get("gates", []) if g.get("critical") and g.get("status") in {"FAIL", "NOT_EVALUATED"}]
        if failed_critical:
            errors.append("Complete status with failed/unevaluated critical gates: " + ", ".join(failed_critical))
        if continuation.get("remaining_mandatory_artifacts"):
            errors.append("Complete status with remaining mandatory artifacts")
        recommendations = state.get("recommendations", [])
        if not recommendations:
            errors.append("Complete status requires a recommendation")
        else:
            recommendation = recommendations[-1]
            decision = recommendation.get("decision")
            if decision not in DECISIONS:
                errors.append("Invalid recommendation decision")
            if decision in {"GO", "PIVOT"}:
                if run.get("depth") == "SIGNAL":
                    errors.append("SIGNAL depth cannot complete with GO/PIVOT")
                if gate_by_id.get("G17", {}).get("status") != "PASS":
                    errors.append("GO/PIVOT requires G17 behavioral validation PASS")
                if not recommendation.get("blueprint_eligible"):
                    errors.append("GO/PIVOT completion requires blueprint_eligible recommendation")

    if strict and warnings:
        errors.extend("STRICT: " + warning for warning in warnings)
        warnings = []
    return {"errors": errors, "warnings": warnings}


def gate_score(state: dict[str, Any]) -> dict[str, Any]:
    values = {"PASS": 1.0, "CONDITIONAL": 0.5, "FAIL": 0.0, "NOT_EVALUATED": 0.0}
    considered = [g for g in state.get("gates", []) if g.get("status") != "N/A"]
    numerator = sum(values.get(g.get("status"), 0.0) for g in considered)
    score = numerator / len(considered) if considered else 0.0
    failed_critical = [g.get("id") for g in considered if g.get("critical") and g.get("status") in {"FAIL", "NOT_EVALUATED"}]
    depth = state.get("run", {}).get("depth")
    minimum = {"SIGNAL": 0.75, "VALIDATION": 0.88, "INVESTMENT_GRADE": 0.93}.get(depth, 1.0)
    return {
        "depth": depth,
        "score": round(score, 4),
        "minimum": minimum,
        "failed_critical_gates": failed_critical,
        "diagnostic_ready": score >= minimum and not failed_critical,
    }


def cmd_init(args: argparse.Namespace) -> int:
    target = state_file(args.workspace)
    if target.exists() and not args.force:
        raise FileExistsError(f"State already exists: {target}; use --force only after review")
    state = initial_state(args)
    atomic_write_json(target, state)
    (target.parent / "checkpoints").mkdir(exist_ok=True)
    (target.parent / "exports").mkdir(exist_ok=True)
    (target.parent / "handoffs").mkdir(exist_ok=True)
    print(json.dumps({"ok": True, "state": str(target), "run_id": state["run"]["run_id"], "checksum": state["checksum"]}, ensure_ascii=False))
    return 0


def cmd_validate(args: argparse.Namespace) -> int:
    state = load_state(args.workspace)
    result = validate_state(state, strict=args.strict)
    payload = {"ok": not result["errors"], **result, "score": gate_score(state)}
    print(json.dumps(payload, ensure_ascii=False, indent=2))
    return 0 if payload["ok"] else 1


def cmd_status(args: argparse.Namespace) -> int:
    state = load_state(args.workspace)
    collections = {
        name: len(state.get(name, []))
        for name in ("sources", "preflights", "hypotheses", "findings", "studies", "experiments", "models", "risks", "recommendations", "handoffs")
    }
    artifact_counts: dict[str, int] = {}
    for artifact in state.get("artifacts", []):
        artifact_counts[artifact.get("status", "unknown")] = artifact_counts.get(artifact.get("status", "unknown"), 0) + 1
    payload = {
        "run": state.get("run"),
        "revision": state.get("revision"),
        "checksum": state.get("checksum"),
        "counts": collections,
        "artifacts": artifact_counts,
        "continuation": state.get("continuation"),
        "score": gate_score(state),
    }
    print(json.dumps(payload, ensure_ascii=False, indent=2))
    return 0


def cmd_allocate(args: argparse.Namespace) -> int:
    state = load_state(args.workspace)
    if args.prefix not in PREFIXES:
        raise ValueError(f"Unsupported prefix: {args.prefix}")
    current = int(state["id_counters"].get(args.prefix, 0))
    allocated = [f"{args.prefix}-{value:03d}" for value in range(current + 1, current + args.count + 1)]
    state["id_counters"][args.prefix] = current + args.count
    target = save_state(args.workspace, state)
    print(json.dumps({"ok": True, "allocated": allocated, "state": str(target), "revision": state["revision"], "checksum": state["checksum"]}, ensure_ascii=False))
    return 0


def parse_csv_list(value: str) -> list[str]:
    return [item.strip() for item in value.split(",") if item.strip()]


def cmd_checkpoint(args: argparse.Namespace) -> int:
    state = load_state(args.workspace)
    continuation = state["continuation"]
    continuation["current_artifact"] = args.current
    continuation["next_exact_section"] = args.next
    if args.completed is not None:
        continuation["completed_artifacts"] = parse_csv_list(args.completed)
    if args.remaining is not None:
        continuation["remaining_mandatory_artifacts"] = parse_csv_list(args.remaining)
    if args.blockers is not None:
        continuation["blockers"] = parse_csv_list(args.blockers)
    continuation["gate_snapshot"] = {g.get("id"): g.get("status") for g in state.get("gates", [])}
    target = save_state(args.workspace, state)
    checkpoint_dir = target.parent / "checkpoints"
    checkpoint_dir.mkdir(exist_ok=True)
    checkpoint = checkpoint_dir / f"revision-{state['revision']:06d}.json"
    atomic_write_json(checkpoint, state)
    print(json.dumps({"ok": True, "checkpoint": str(checkpoint), "revision": state["revision"], "checksum": state["checksum"]}, ensure_ascii=False))
    return 0


def cmd_score(args: argparse.Namespace) -> int:
    state = load_state(args.workspace)
    hypotheses = []
    for item in state.get("hypotheses", []):
        hypotheses.append({
            "id": item.get("id"),
            "criticality": item.get("decision_criticality"),
            "status": item.get("status"),
            "confidence": item.get("current_confidence"),
            "support_count": len(item.get("supporting_evidence", [])),
            "negative_count": len(item.get("negative_evidence", [])),
        })
    print(json.dumps({"gates": gate_score(state), "hypotheses": hypotheses}, ensure_ascii=False, indent=2))
    return 0


def render_markdown(state: dict[str, Any]) -> str:
    run = state["run"]
    decision = state["decision_brief"]
    recommendation = state.get("recommendations", [])[-1] if state.get("recommendations") else None
    lines = [
        f"# {run['project_name']} — Market Research Status",
        "",
        f"Status: `{run['status']}`  ",
        f"Run: `{run['run_id']}`  ",
        f"Version: `{run['research_version']}`  ",
        f"Mode/depth: `{run['mode']} / {run['depth']}`  ",
        f"Evidence cutoff: `{run['evidence_cutoff']}`",
        "",
        "## Decision brief",
        "",
        decision["decision_question"],
        "",
    ]
    if recommendation:
        lines.extend([
            "## Recommendation",
            "",
            f"Decision: `{recommendation.get('decision')}`  ",
            f"Confidence: `{recommendation.get('confidence')}`  ",
            f"Scope: {recommendation.get('scope', '')}",
            "",
            recommendation.get("rationale", ""),
            "",
        ])
    lines.extend(["## Gate diagnostic", "", "| Gate | Status | Critical |", "| --- | --- | --- |"]) 
    for gate in state.get("gates", []):
        lines.append(f"| {gate.get('id')} — {gate.get('name')} | {gate.get('status')} | {gate.get('critical')} |")
    lines.extend(["", "## Continuation", "", f"Next: `{state['continuation'].get('next_exact_section', '')}`", ""])
    return "\n".join(lines)


def cmd_export(args: argparse.Namespace) -> int:
    state = load_state(args.workspace)
    output = Path(args.output).expanduser().resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    if args.format == "json":
        atomic_write_json(output, state)
    else:
        content = render_markdown(state)
        fd, tmp_name = tempfile.mkstemp(prefix=output.name + ".", suffix=".tmp", dir=output.parent)
        try:
            with os.fdopen(fd, "w", encoding="utf-8") as handle:
                handle.write(content)
                handle.flush()
                os.fsync(handle.fileno())
            os.replace(tmp_name, output)
        finally:
            if os.path.exists(tmp_name):
                os.unlink(tmp_name)
    print(json.dumps({"ok": True, "output": str(output), "format": args.format}, ensure_ascii=False))
    return 0


def cmd_demo(args: argparse.Namespace) -> int:
    demo_args = argparse.Namespace(
        workspace=args.workspace,
        project_id="demo-market",
        project_name="Demo Market",
        decision="Should the demo opportunity proceed to a full validation run?",
        decision_owner="Demo owner",
        mode="RAPID_SCAN",
        depth="SIGNAL",
        external_action_authority="none",
        confidentiality="internal",
        force=args.force,
    )
    cmd_init(demo_args)
    state = load_state(args.workspace)
    state["id_counters"]["HYP"] = 1
    state["hypotheses"].append({
        "id": "HYP-001",
        "statement": "A reachable segment has a recurring costly problem.",
        "domain": "problem",
        "status": "untested",
        "decision_criticality": "P0",
        "prior_confidence": 0.5,
        "current_confidence": 0.5,
        "falsifier": "No recent recurring problem or consequential workaround is found in the eligible segment.",
        "evidence_required": [],
        "metric": "Qualified problem incidence and behavior",
        "pass_threshold": "Predeclared in the research plan",
        "fail_threshold": "Predeclared in the research plan",
        "ambiguous_rule": "Collect the next highest-value evidence or return insufficient evidence",
        "methods": [],
        "sample_or_sources": [],
        "supporting_evidence": [],
        "negative_evidence": [],
        "conflicts": [],
        "decision_impact": "Controls whether research should continue",
        "next_test": "Problem interviews plus secondary evidence",
        "owner": "Customer/JTBD Researcher",
        "expires_at": None,
    })
    save_state(args.workspace, state)
    result = validate_state(state)
    print(json.dumps({"demo": str(state_file(args.workspace)), "valid": not result["errors"], **result}, ensure_ascii=False, indent=2))
    return 0 if not result["errors"] else 1


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Market Research {OS} deterministic state CLI")
    sub = parser.add_subparsers(dest="command", required=True)

    init = sub.add_parser("init", help="Initialize a research workspace")
    init.add_argument("workspace")
    init.add_argument("--project-id", required=True)
    init.add_argument("--project-name", required=True)
    init.add_argument("--decision", required=True)
    init.add_argument("--decision-owner", default="")
    init.add_argument("--mode", choices=sorted(MODES), default="NEW")
    init.add_argument("--depth", choices=sorted(DEPTHS), default="VALIDATION")
    init.add_argument("--external-action-authority", choices=["none", "research-only", "approved-scope"], default="none")
    init.add_argument("--confidentiality", choices=["public", "internal", "confidential", "restricted"], default="confidential")
    init.add_argument("--force", action="store_true")
    init.set_defaults(func=cmd_init)

    validate = sub.add_parser("validate", help="Validate a research state")
    validate.add_argument("workspace")
    validate.add_argument("--strict", action="store_true")
    validate.set_defaults(func=cmd_validate)

    status = sub.add_parser("status", help="Show research status")
    status.add_argument("workspace")
    status.set_defaults(func=cmd_status)

    allocate = sub.add_parser("allocate", help="Allocate stable IDs")
    allocate.add_argument("workspace")
    allocate.add_argument("prefix")
    allocate.add_argument("--count", type=int, default=1, choices=range(1, 101), metavar="1..100")
    allocate.set_defaults(func=cmd_allocate)

    checkpoint = sub.add_parser("checkpoint", help="Save a restart-safe checkpoint")
    checkpoint.add_argument("workspace")
    checkpoint.add_argument("--current", required=True)
    checkpoint.add_argument("--next", required=True)
    checkpoint.add_argument("--completed")
    checkpoint.add_argument("--remaining")
    checkpoint.add_argument("--blockers")
    checkpoint.set_defaults(func=cmd_checkpoint)

    score = sub.add_parser("score", help="Show gate and hypothesis diagnostics")
    score.add_argument("workspace")
    score.set_defaults(func=cmd_score)

    export = sub.add_parser("export", help="Export state or status view")
    export.add_argument("workspace")
    export.add_argument("--format", choices=["json", "markdown"], default="markdown")
    export.add_argument("--output", required=True)
    export.set_defaults(func=cmd_export)

    demo = sub.add_parser("demo", help="Create and validate a small demo workspace")
    demo.add_argument("workspace")
    demo.add_argument("--force", action="store_true")
    demo.set_defaults(func=cmd_demo)
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        return int(args.func(args))
    except (FileNotFoundError, FileExistsError, ValueError, json.JSONDecodeError) as exc:
        print(json.dumps({"ok": False, "error": str(exc)}, ensure_ascii=False), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())

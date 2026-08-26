#!/usr/bin/env python3
"""Deterministic local state substrate for Builder {OS}.

This CLI does not implement a coding agent or replace Stepper. It provides a
small, dependency-free reference implementation for canonical state, evidence,
attempt transitions, checkpoints, semantic validation, and release gating.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import os
import sys
import tempfile
import uuid
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Iterable


VERSION = "1.0.0"
SCHEMA_VERSION = 1

PROJECT_STATUSES = {
    "BUILD PREFLIGHT",
    "BUILD IN PROGRESS",
    "BUILD BLOCKED",
    "BUILD PAUSED",
    "BUILD COMPLETE — RELEASE READY",
}

STEPPER_STATUSES = {
    "PENDING",
    "READY",
    "RUNNING",
    "VERIFYING",
    "FAILED",
    "BLOCKED",
    "DONE",
    "SKIPPED",
    "SUPERSEDED",
    "STALE",
}

BUILDER_STEP_STATES = {
    "NOT_STARTED",
    "CLAIMED",
    "CONTEXT_READY",
    "IMPLEMENTING",
    "IMPLEMENTED",
    "VERIFYING",
    "REPAIRING",
    "REVIEWING",
    "INTEGRATING",
    "POST_MERGE_VERIFYING",
    "SUCCEEDED",
    "FAILED",
    "BLOCKED",
    "INTERRUPTED",
    "ROLLED_BACK",
    "ABORTED",
}

ATTEMPT_STATES = {
    "CREATED",
    "CLAIMED",
    "CONTEXT_READY",
    "IMPLEMENTING",
    "IMPLEMENTED",
    "VERIFYING",
    "REPAIRING",
    "REVIEWING",
    "INTEGRATING",
    "POST_MERGE_VERIFYING",
    "SUCCEEDED",
    "FAILED",
    "BLOCKED",
    "INTERRUPTED",
    "ROLLED_BACK",
    "ABORTED",
}

ATTEMPT_TRANSITIONS = {
    "CREATED": {"CLAIMED", "ABORTED"},
    "CLAIMED": {"CONTEXT_READY", "BLOCKED", "INTERRUPTED", "ABORTED"},
    "CONTEXT_READY": {"IMPLEMENTING", "BLOCKED", "INTERRUPTED", "ABORTED"},
    "IMPLEMENTING": {"IMPLEMENTED", "FAILED", "BLOCKED", "INTERRUPTED"},
    "IMPLEMENTED": {"VERIFYING", "FAILED", "BLOCKED", "INTERRUPTED"},
    "VERIFYING": {"REVIEWING", "INTEGRATING", "FAILED", "BLOCKED", "INTERRUPTED"},
    "REPAIRING": {"IMPLEMENTED", "FAILED", "BLOCKED", "INTERRUPTED"},
    "REVIEWING": {"INTEGRATING", "FAILED", "BLOCKED", "INTERRUPTED"},
    "INTEGRATING": {"POST_MERGE_VERIFYING", "FAILED", "BLOCKED", "ROLLED_BACK", "INTERRUPTED"},
    "POST_MERGE_VERIFYING": {"SUCCEEDED", "FAILED", "BLOCKED", "ROLLED_BACK", "INTERRUPTED"},
    "FAILED": {"REPAIRING", "BLOCKED", "ABORTED"},
    "BLOCKED": {"REPAIRING", "CLAIMED", "ABORTED"},
    "INTERRUPTED": {"CLAIMED", "CONTEXT_READY", "IMPLEMENTING", "IMPLEMENTED", "VERIFYING", "REVIEWING", "INTEGRATING", "POST_MERGE_VERIFYING", "BLOCKED", "ABORTED"},
    "ROLLED_BACK": {"REPAIRING", "BLOCKED", "ABORTED"},
    "SUCCEEDED": set(),
    "ABORTED": set(),
}

ACTIVE_ATTEMPT_STATES = {
    "CLAIMED",
    "CONTEXT_READY",
    "IMPLEMENTING",
    "IMPLEMENTED",
    "VERIFYING",
    "REPAIRING",
    "REVIEWING",
    "INTEGRATING",
    "POST_MERGE_VERIFYING",
}

GATE_NAMES = {
    "BG01": "Input Integrity",
    "BG02": "Repository Baseline",
    "BG03": "Setup and Toolchain",
    "BG04": "Graph Alignment",
    "BG05": "Code Quality",
    "BG06": "Unit and Domain",
    "BG07": "Integration and Contract",
    "BG08": "E2E and Acceptance",
    "BG09": "Security, Privacy, Abuse",
    "BG10": "Architecture",
    "BG11": "UX, Accessibility, Visual",
    "BG12": "Data and Migration",
    "BG13": "AI and Evaluation",
    "BG14": "Performance and Reliability",
    "BG15": "Observability and Operations",
    "BG16": "Documentation",
    "BG17": "Integrated Revision Health",
    "BG18": "Traceability and Evidence",
    "BG19": "Risks and Follow-up",
    "BG20": "Release and Handoff",
}

# Criticality can be changed per project, but these defaults prevent an unsafe
# release from a fresh state. Optional domain gates may be made noncritical and
# marked N/A only with evidence and a rationale.
DEFAULT_CRITICAL_GATES = {
    "BG01",
    "BG02",
    "BG03",
    "BG04",
    "BG05",
    "BG06",
    "BG07",
    "BG08",
    "BG09",
    "BG10",
    "BG12",
    "BG15",
    "BG16",
    "BG17",
    "BG18",
    "BG19",
    "BG20",
}


class BuilderStateError(RuntimeError):
    """Raised when deterministic Builder state rules are violated."""


def now() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def uid(prefix: str) -> str:
    return f"{prefix}-{uuid.uuid4().hex[:16].upper()}"


def canonical_json(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode("utf-8")


def state_digest(state: dict[str, Any]) -> str:
    normalized = copy.deepcopy(state)
    normalized.get("meta", {})["checksum"] = None
    normalized.get("continuation", {})["checkpoint_checksum"] = None
    if isinstance(normalized.get("handoff"), dict):
        normalized["handoff"]["checksum"] = ""
    return "sha256:" + hashlib.sha256(canonical_json(normalized)).hexdigest()


def path_digest(path: Path) -> str:
    if not path.exists():
        return "missing"
    digest = hashlib.sha256()
    if path.is_file():
        digest.update(path.read_bytes())
        return "sha256:" + digest.hexdigest()
    for item in sorted(p for p in path.rglob("*") if p.is_file() and ".git" not in p.parts):
        digest.update(str(item.relative_to(path)).encode("utf-8"))
        digest.update(b"\0")
        digest.update(item.read_bytes())
        digest.update(b"\0")
    return "sha256:" + digest.hexdigest()


def load_state(path: Path) -> dict[str, Any]:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise BuilderStateError(f"State file not found: {path}") from exc
    except json.JSONDecodeError as exc:
        raise BuilderStateError(f"Invalid JSON in {path}: {exc}") from exc
    if not isinstance(data, dict):
        raise BuilderStateError("Builder state root must be an object")
    return data


def atomic_write(path: Path, state: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    state.setdefault("meta", {})["checksum"] = state_digest(state)
    payload = json.dumps(state, ensure_ascii=False, indent=2, sort_keys=True) + "\n"
    fd, temporary = tempfile.mkstemp(prefix=f".{path.name}.", dir=str(path.parent))
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as handle:
            handle.write(payload)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temporary, path)
    finally:
        if os.path.exists(temporary):
            os.unlink(temporary)


def check_expected_revision(state: dict[str, Any], expected: int | None) -> None:
    if expected is None:
        return
    actual = state.get("meta", {}).get("revision")
    if actual != expected:
        raise BuilderStateError(f"Revision conflict: expected {expected}, found {actual}")


def append_event(
    state: dict[str, Any],
    event_type: str,
    *,
    step_id: str | None = None,
    attempt_id: str | None = None,
    actor: str = "builder-os-cli",
    payload: dict[str, Any] | None = None,
) -> None:
    events = state.setdefault("events", [])
    events.append(
        {
            "event_id": uid("EVT"),
            "sequence": len(events) + 1,
            "event_type": event_type,
            "project_revision": state.get("meta", {}).get("revision", 0),
            "step_id": step_id,
            "attempt_id": attempt_id,
            "actor": actor,
            "payload": payload or {},
            "created_at": now(),
        }
    )


def touch(state: dict[str, Any]) -> None:
    state["meta"]["revision"] += 1
    state["meta"]["updated_at"] = now()


def default_gates() -> list[dict[str, Any]]:
    return [
        {
            "gate_id": gate_id,
            "name": name,
            "result": "NOT_EVALUATED",
            "critical": gate_id in DEFAULT_CRITICAL_GATES,
            "candidate_revision": None,
            "input_hash": None,
            "evidence_refs": [],
            "blocker_ids": [],
            "condition": None,
            "owner": None,
            "evaluated_at": None,
        }
        for gate_id, name in GATE_NAMES.items()
    ]


def make_state(args: argparse.Namespace) -> dict[str, Any]:
    created = now()
    repo = Path(args.repo).resolve()
    blueprint_ref = Path(args.blueprint_ref).resolve() if args.blueprint_ref else None
    stepper_ref = Path(args.stepper_ref).resolve() if args.stepper_ref else None
    blueprint_checksum = args.blueprint_checksum or (path_digest(blueprint_ref) if blueprint_ref else "unverified")
    stepper_checksum = args.stepper_checksum or (path_digest(stepper_ref) if stepper_ref else "unverified")
    state: dict[str, Any] = {
        "meta": {
            "project_id": args.project_id,
            "project_name": args.project_name,
            "builder_version": VERSION,
            "schema_version": SCHEMA_VERSION,
            "status": "BUILD PREFLIGHT",
            "revision": 0,
            "checksum": None,
            "created_at": created,
            "updated_at": created,
        },
        "run": {
            "run_id": uid("RUN"),
            "mode": "NEW",
            "request": args.request or f"Build {args.project_name}",
            "started_at": created,
            "actor": args.actor,
            "locale": args.locale,
        },
        "inputs": {
            "blueprint": {
                "handoff_id": args.blueprint_handoff_id,
                "project_id": args.project_id,
                "version": args.blueprint_version,
                "revision": args.blueprint_revision,
                "checksum": blueprint_checksum,
                "status": "BLUEPRINT COMPLETE — STEPPER READY",
                "artifact_refs": [str(blueprint_ref)] if blueprint_ref else [],
                "prohibited_shortcuts": [],
            },
            "stepper": {
                "version": args.stepper_version,
                "schema_version": args.stepper_schema_version,
                "checksum": stepper_checksum,
                "status": args.stepper_status,
                "manifest_ref": str(stepper_ref) if stepper_ref else "unresolved",
                "tracker_ref": args.tracker_ref,
                "graph_ref": args.graph_ref or (str(stepper_ref) if stepper_ref else "unresolved"),
                "release_target": args.release_target,
            },
        },
        "repository": {
            "root": str(repo),
            "base_branch": args.base_branch,
            "base_revision": args.base_revision,
            "current_revision": args.base_revision,
            "integration_branch": args.integration_branch or args.base_branch,
            "dirty_snapshot": {},
            "worktree_root": args.worktree_root,
            "toolchain_fingerprint": None,
        },
        "policy": {
            "release_target": args.release_target,
            "max_parallel_steps": args.max_parallel_steps,
            "max_active_modules": args.max_active_modules,
            "max_repair_attempts": args.max_repair_attempts,
            "use_worktrees": not args.no_worktrees,
            "require_clean_base": not args.allow_dirty_base,
            "require_post_integration_verification": True,
            "permissions": {},
        },
        "execution": {
            "current_wave_id": None,
            "stepper_tracker_revision": 0,
            "active_attempt_ids": [],
            "active_lock_ids": [],
            "candidate_revision": None,
            "stepper_release_result": "NOT_EVALUATED",
            "raw_progress": 0.0,
            "weighted_progress": 0.0,
        },
        "steps": [],
        "attempts": [],
        "checks": [],
        "reviews": [],
        "integrations": [],
        "artifacts": [],
        "events": [],
        "blockers": [],
        "decision_requests": [],
        "changesets": [],
        "followups": [],
        "documentation": [],
        "gates": default_gates(),
        "continuation": {
            "checkpoint_id": None,
            "checkpoint_revision": 0,
            "checkpoint_checksum": None,
            "current_wave_id": None,
            "active_attempt_ids": [],
            "active_lock_ids": [],
            "blocker_ids": [],
            "next_exact_action": "Run Builder preflight and Stepper validation/status/plan",
            "event_offset": 0,
        },
        "exports": [],
        "handoff": None,
    }
    append_event(state, "BUILD_INITIALIZED", actor=args.actor, payload={"release_target": args.release_target})
    return state


def duplicates(values: Iterable[str]) -> list[str]:
    seen: set[str] = set()
    result: set[str] = set()
    for value in values:
        if value in seen:
            result.add(value)
        seen.add(value)
    return sorted(result)


def validate_state(state: dict[str, Any], *, verify_checksum: bool = True) -> list[str]:
    errors: list[str] = []
    required_top = {
        "meta", "run", "inputs", "repository", "policy", "execution", "steps", "attempts",
        "checks", "reviews", "integrations", "artifacts", "events", "blockers",
        "decision_requests", "changesets", "followups", "documentation", "gates",
        "continuation", "exports", "handoff",
    }
    missing = required_top - set(state)
    if missing:
        errors.append("Missing top-level fields: " + ", ".join(sorted(missing)))
        return errors

    meta = state.get("meta", {})
    if meta.get("status") not in PROJECT_STATUSES:
        errors.append(f"Invalid project status: {meta.get('status')!r}")
    if meta.get("schema_version") != SCHEMA_VERSION:
        errors.append(f"Unsupported schema version: {meta.get('schema_version')!r}")
    if not isinstance(meta.get("revision"), int) or meta.get("revision", -1) < 0:
        errors.append("meta.revision must be a non-negative integer")
    if verify_checksum and meta.get("checksum") and meta.get("checksum") != state_digest(state):
        errors.append("State checksum does not match canonical content")

    blueprint = state.get("inputs", {}).get("blueprint", {})
    if blueprint.get("status") != "BLUEPRINT COMPLETE — STEPPER READY":
        errors.append("Blueprint input is not BLUEPRINT COMPLETE — STEPPER READY")
    if not blueprint.get("checksum") or blueprint.get("checksum") in {"unverified", "missing"}:
        errors.append("Blueprint checksum is missing or unverified")
    stepper = state.get("inputs", {}).get("stepper", {})
    if not stepper.get("status"):
        errors.append("Stepper input status is missing")
    if not stepper.get("checksum") or stepper.get("checksum") in {"unverified", "missing"}:
        errors.append("Stepper checksum is missing or unverified")

    steps = state.get("steps", [])
    step_ids = [str(item.get("step_id")) for item in steps]
    for item in duplicates(step_ids):
        errors.append(f"Duplicate step ID: {item}")
    step_map = {str(item.get("step_id")): item for item in steps}
    for step in steps:
        sid = str(step.get("step_id"))
        if step.get("stepper_status") not in STEPPER_STATUSES:
            errors.append(f"{sid}: invalid Stepper status {step.get('stepper_status')!r}")
        if step.get("builder_state") not in BUILDER_STEP_STATES:
            errors.append(f"{sid}: invalid Builder state {step.get('builder_state')!r}")
        if step.get("stepper_status") == "DONE" and step.get("builder_state") != "SUCCEEDED":
            errors.append(f"{sid}: Stepper DONE requires Builder SUCCEEDED evidence state")
        if step.get("builder_state") == "SUCCEEDED" and not step.get("evidence_refs"):
            errors.append(f"{sid}: Builder SUCCEEDED requires evidence refs")

    attempts = state.get("attempts", [])
    attempt_ids = [str(item.get("attempt_id")) for item in attempts]
    for item in duplicates(attempt_ids):
        errors.append(f"Duplicate attempt ID: {item}")
    attempt_map = {str(item.get("attempt_id")): item for item in attempts}
    for attempt in attempts:
        aid = str(attempt.get("attempt_id"))
        sid = str(attempt.get("step_id"))
        if sid not in step_map:
            errors.append(f"{aid}: unknown step {sid}")
        if attempt.get("state") not in ATTEMPT_STATES:
            errors.append(f"{aid}: invalid state {attempt.get('state')!r}")
        for check_id in attempt.get("check_ids", []):
            if check_id not in {str(c.get("check_id")) for c in state.get("checks", [])}:
                errors.append(f"{aid}: unknown check ref {check_id}")
        for review_id in attempt.get("review_ids", []):
            if review_id not in {str(r.get("review_id")) for r in state.get("reviews", [])}:
                errors.append(f"{aid}: unknown review ref {review_id}")
    for step in steps:
        sid = str(step.get("step_id"))
        for aid in step.get("attempt_ids", []):
            if aid not in attempt_map:
                errors.append(f"{sid}: unknown attempt ref {aid}")
            elif str(attempt_map[aid].get("step_id")) != sid:
                errors.append(f"{sid}: attempt {aid} belongs to another step")

    for check in state.get("checks", []):
        cid = str(check.get("check_id"))
        if str(check.get("attempt_id")) not in attempt_map:
            errors.append(f"{cid}: unknown attempt")
        if check.get("result") not in {"PASS", "FAIL", "BLOCKED", "N/A"}:
            errors.append(f"{cid}: invalid result {check.get('result')!r}")
    for review in state.get("reviews", []):
        rid = str(review.get("review_id"))
        if str(review.get("attempt_id")) not in attempt_map:
            errors.append(f"{rid}: unknown attempt")
        if review.get("result") not in {"PASS", "FAIL", "BLOCKED", "N/A"}:
            errors.append(f"{rid}: invalid result {review.get('result')!r}")

    active_ids = state.get("execution", {}).get("active_attempt_ids", [])
    for aid in active_ids:
        attempt = attempt_map.get(aid)
        if not attempt:
            errors.append(f"Active attempt does not exist: {aid}")
        elif attempt.get("state") not in ACTIVE_ATTEMPT_STATES:
            errors.append(f"Active attempt {aid} is terminal/non-active: {attempt.get('state')}")
    for aid, attempt in attempt_map.items():
        if attempt.get("state") in ACTIVE_ATTEMPT_STATES and aid not in active_ids:
            errors.append(f"Active-state attempt is missing from execution.active_attempt_ids: {aid}")

    gates = state.get("gates", [])
    gate_ids = [str(g.get("gate_id")) for g in gates]
    expected_gate_ids = set(GATE_NAMES)
    if set(gate_ids) != expected_gate_ids or len(gates) != len(expected_gate_ids):
        errors.append("Gates must contain exactly BG01 through BG20 once each")
    for gate_id in duplicates(gate_ids):
        errors.append(f"Duplicate gate ID: {gate_id}")
    for gate in gates:
        gid = str(gate.get("gate_id"))
        result = gate.get("result")
        if result not in {"PASS", "CONDITIONAL", "FAIL", "N/A", "NOT_EVALUATED"}:
            errors.append(f"{gid}: invalid result {result!r}")
        if gate.get("critical") and result == "CONDITIONAL":
            errors.append(f"{gid}: critical gate cannot be CONDITIONAL")
        if result == "PASS" and not gate.get("evidence_refs"):
            errors.append(f"{gid}: PASS requires evidence refs")
        if result == "N/A" and (gate.get("critical") or not gate.get("condition")):
            errors.append(f"{gid}: N/A requires noncritical gate and rationale in condition")

    events = state.get("events", [])
    sequences = [event.get("sequence") for event in events]
    if sequences != list(range(1, len(events) + 1)):
        errors.append("Event sequence must be contiguous, ordered, and append-only")

    if meta.get("status") == "BUILD COMPLETE — RELEASE READY":
        errors.extend(release_errors(state))
    continuation = state.get("continuation", {})
    if (
        continuation.get("checkpoint_checksum")
        and continuation.get("checkpoint_revision") == meta.get("revision")
        and continuation.get("checkpoint_checksum") != state_digest(state)
    ):
        errors.append("Checkpoint checksum does not match canonical state")
    handoff = state.get("handoff")
    if isinstance(handoff, dict) and handoff.get("checksum") and handoff.get("checksum") != state_digest(state):
        errors.append("Handoff checksum does not match canonical state")
    return errors


def release_errors(state: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    execution = state.get("execution", {})
    candidate = execution.get("candidate_revision")
    if not candidate:
        errors.append("Release requires a frozen candidate revision")
    if execution.get("stepper_release_result") != "PASS":
        errors.append("Release requires Stepper release result PASS")
    for step in state.get("steps", []):
        if step.get("required_for_release") and step.get("stepper_status") != "DONE":
            errors.append(f"Required step not DONE: {step.get('step_id')}")
        if step.get("required_for_release") and step.get("builder_state") != "SUCCEEDED":
            errors.append(f"Required step lacks Builder SUCCEEDED evidence: {step.get('step_id')}")
    for gate in state.get("gates", []):
        result = gate.get("result")
        if result == "PASS":
            if gate.get("candidate_revision") not in {None, candidate}:
                errors.append(f"{gate.get('gate_id')}: evidence targets another candidate")
            continue
        if result == "N/A" and not gate.get("critical") and gate.get("condition") and gate.get("evidence_refs"):
            continue
        errors.append(f"Release gate not passing: {gate.get('gate_id')}={result}")
    for blocker in state.get("blockers", []):
        if blocker.get("status") == "open" and blocker.get("severity") == "critical":
            errors.append(f"Open critical blocker: {blocker.get('blocker_id')}")
    for followup in state.get("followups", []):
        if followup.get("status") not in {"resolved", "rejected"} and (followup.get("severity") == "critical" or followup.get("blocking")):
            errors.append(f"Blocking/critical follow-up unresolved: {followup.get('followup_id')}")
    handoff = state.get("handoff")
    if not isinstance(handoff, dict):
        errors.append("Release requires a final handoff")
    else:
        if handoff.get("status") != "BUILD COMPLETE — RELEASE READY":
            errors.append("Handoff has invalid status")
        if handoff.get("candidate_revision") != candidate:
            errors.append("Handoff candidate does not match execution candidate")
        if handoff.get("blueprint_checksum") != state.get("inputs", {}).get("blueprint", {}).get("checksum"):
            errors.append("Handoff Blueprint checksum mismatch")
        if handoff.get("stepper_checksum") != state.get("inputs", {}).get("stepper", {}).get("checksum"):
            errors.append("Handoff Stepper checksum mismatch")
        if not handoff.get("artifact_refs"):
            errors.append("Handoff requires artifact refs")
    return errors


def find_step(state: dict[str, Any], step_id: str) -> dict[str, Any]:
    for step in state["steps"]:
        if step.get("step_id") == step_id:
            return step
    raise BuilderStateError(f"Unknown step: {step_id}")


def find_attempt(state: dict[str, Any], attempt_id: str) -> dict[str, Any]:
    for attempt in state["attempts"]:
        if attempt.get("attempt_id") == attempt_id:
            return attempt
    raise BuilderStateError(f"Unknown attempt: {attempt_id}")


def save_mutation(path: Path, state: dict[str, Any], *, expected: int | None = None) -> None:
    check_expected_revision(state, expected)
    touch(state)
    atomic_write(path, state)


def command_init(args: argparse.Namespace) -> int:
    state_path = Path(args.state).resolve()
    if state_path.exists() and not args.force:
        raise BuilderStateError(f"Refusing to overwrite existing state: {state_path}")
    state = make_state(args)
    atomic_write(state_path, state)
    print(f"Initialized Builder state: {state_path}")
    print(f"Status: {state['meta']['status']}")
    print(f"Revision: {state['meta']['revision']}")
    return 0


def command_validate(args: argparse.Namespace) -> int:
    path = Path(args.state).resolve()
    state = load_state(path)
    errors = validate_state(state)
    if errors:
        print("INVALID")
        for error in errors:
            print(f"- {error}")
        return 1
    print("VALID")
    print(f"Checksum: {state.get('meta', {}).get('checksum')}")
    return 0


def command_status(args: argparse.Namespace) -> int:
    state = load_state(Path(args.state).resolve())
    errors = validate_state(state)
    counts: dict[str, int] = {}
    for step in state.get("steps", []):
        key = str(step.get("stepper_status"))
        counts[key] = counts.get(key, 0) + 1
    gates: dict[str, int] = {}
    for gate in state.get("gates", []):
        key = str(gate.get("result"))
        gates[key] = gates.get(key, 0) + 1
    print(f"Project: {state['meta']['project_name']} ({state['meta']['project_id']})")
    print(f"Status: {state['meta']['status']}")
    print(f"Revision: {state['meta']['revision']}")
    print(f"Blueprint: {state['inputs']['blueprint']['version']} {state['inputs']['blueprint']['checksum']}")
    print(f"Stepper: {state['inputs']['stepper']['version']} {state['inputs']['stepper']['checksum']}")
    print(f"Progress: raw={state['execution']['raw_progress']:.1%} weighted={state['execution']['weighted_progress']:.1%}")
    print("Steps: " + (", ".join(f"{k}={v}" for k, v in sorted(counts.items())) or "none"))
    print("Gates: " + ", ".join(f"{k}={v}" for k, v in sorted(gates.items())))
    print(f"Open blockers: {sum(1 for b in state['blockers'] if b.get('status') == 'open')}")
    print(f"Next: {state['continuation'].get('next_exact_action') or 'none'}")
    if errors:
        print(f"Validation errors: {len(errors)}")
        return 1
    return 0


def command_sync_step(args: argparse.Namespace) -> int:
    path = Path(args.state).resolve()
    state = load_state(path)
    check_expected_revision(state, args.expected_revision)
    existing = next((s for s in state["steps"] if s.get("step_id") == args.step_id), None)
    timestamp = now()
    if existing is None:
        existing = {
            "step_id": args.step_id,
            "module_id": args.module_id,
            "slice_id": args.slice_id,
            "priority": args.priority,
            "risk": args.risk,
            "required_for_release": args.required_for_release,
            "stepper_status": args.stepper_status,
            "builder_state": "NOT_STARTED",
            "spec_hash": args.spec_hash,
            "attempt_ids": [],
            "evidence_refs": [],
            "updated_at": timestamp,
        }
        state["steps"].append(existing)
        event_type = "STEP_IMPORTED"
    else:
        existing.update(
            {
                "module_id": args.module_id,
                "slice_id": args.slice_id,
                "priority": args.priority,
                "risk": args.risk,
                "required_for_release": args.required_for_release,
                "stepper_status": args.stepper_status,
                "spec_hash": args.spec_hash,
                "updated_at": timestamp,
            }
        )
        event_type = "STEP_SYNCED"
    append_event(state, event_type, step_id=args.step_id, payload={"stepper_status": args.stepper_status, "spec_hash": args.spec_hash})
    touch(state)
    atomic_write(path, state)
    print(f"{event_type}: {args.step_id}")
    return 0


def command_claim(args: argparse.Namespace) -> int:
    path = Path(args.state).resolve()
    state = load_state(path)
    check_expected_revision(state, args.expected_revision)
    step = find_step(state, args.step_id)
    if step.get("stepper_status") != "READY":
        raise BuilderStateError(f"Step must be READY to claim, found {step.get('stepper_status')}")
    if step.get("builder_state") not in {"NOT_STARTED", "FAILED", "BLOCKED", "INTERRUPTED", "ROLLED_BACK"}:
        raise BuilderStateError(f"Step Builder state is not claimable: {step.get('builder_state')}")
    number = len(step.get("attempt_ids", [])) + 1
    attempt_id = args.attempt_id or uid("ATT")
    if any(a.get("attempt_id") == attempt_id for a in state["attempts"]):
        raise BuilderStateError(f"Attempt already exists: {attempt_id}")
    attempt = {
        "attempt_id": attempt_id,
        "step_id": args.step_id,
        "number": number,
        "state": "CLAIMED",
        "worker_id": args.worker_id,
        "spec_hash": step.get("spec_hash"),
        "context_hash": None,
        "prompt_hash": None,
        "base_revision": args.base_revision or state["repository"].get("current_revision"),
        "head_revision": None,
        "integrated_revision": None,
        "worktree": args.worktree,
        "branch": args.branch,
        "lock_ids": list(args.lock_id),
        "check_ids": [],
        "review_ids": [],
        "artifact_refs": [],
        "failure_class": None,
        "summary": None,
        "started_at": now(),
        "finished_at": None,
    }
    state["attempts"].append(attempt)
    step["attempt_ids"].append(attempt_id)
    step["builder_state"] = "CLAIMED"
    step["updated_at"] = now()
    state["execution"]["active_attempt_ids"].append(attempt_id)
    for lock_id in args.lock_id:
        if lock_id in state["execution"]["active_lock_ids"]:
            raise BuilderStateError(f"Lock already active: {lock_id}")
        state["execution"]["active_lock_ids"].append(lock_id)
    if state["meta"]["status"] == "BUILD PREFLIGHT":
        state["meta"]["status"] = "BUILD IN PROGRESS"
    append_event(state, "STEP_CLAIMED", step_id=args.step_id, attempt_id=attempt_id, actor=args.worker_id, payload={"locks": args.lock_id})
    touch(state)
    atomic_write(path, state)
    print(attempt_id)
    return 0


def command_transition(args: argparse.Namespace) -> int:
    path = Path(args.state).resolve()
    state = load_state(path)
    check_expected_revision(state, args.expected_revision)
    attempt = find_attempt(state, args.attempt_id)
    current = str(attempt.get("state"))
    target = args.to
    if target not in ATTEMPT_TRANSITIONS.get(current, set()):
        raise BuilderStateError(f"Invalid attempt transition: {current} -> {target}")
    attempt["state"] = target
    if args.context_hash:
        attempt["context_hash"] = args.context_hash
    if args.prompt_hash:
        attempt["prompt_hash"] = args.prompt_hash
    if args.head_revision:
        attempt["head_revision"] = args.head_revision
    if args.integrated_revision:
        attempt["integrated_revision"] = args.integrated_revision
    if args.summary:
        attempt["summary"] = args.summary
    if args.failure_class:
        attempt["failure_class"] = args.failure_class
    step = find_step(state, str(attempt["step_id"]))
    step["builder_state"] = target if target in BUILDER_STEP_STATES else step["builder_state"]
    if target in {"SUCCEEDED", "ABORTED", "ROLLED_BACK"}:
        attempt["finished_at"] = now()
    if target in ACTIVE_ATTEMPT_STATES:
        if args.attempt_id not in state["execution"]["active_attempt_ids"]:
            state["execution"]["active_attempt_ids"].append(args.attempt_id)
        for lock_id in attempt.get("lock_ids", []):
            if lock_id not in state["execution"]["active_lock_ids"]:
                state["execution"]["active_lock_ids"].append(lock_id)
    if target in {"SUCCEEDED", "FAILED", "BLOCKED", "INTERRUPTED", "ROLLED_BACK", "ABORTED"}:
        if args.attempt_id in state["execution"]["active_attempt_ids"]:
            state["execution"]["active_attempt_ids"].remove(args.attempt_id)
        for lock_id in list(attempt.get("lock_ids", [])):
            if lock_id in state["execution"]["active_lock_ids"]:
                state["execution"]["active_lock_ids"].remove(lock_id)
    append_event(state, f"ATTEMPT_{target}", step_id=attempt["step_id"], attempt_id=args.attempt_id, payload={"from": current, "to": target})
    touch(state)
    atomic_write(path, state)
    print(f"{args.attempt_id}: {current} -> {target}")
    return 0


def command_record_check(args: argparse.Namespace) -> int:
    path = Path(args.state).resolve()
    state = load_state(path)
    check_expected_revision(state, args.expected_revision)
    attempt = find_attempt(state, args.attempt_id)
    if any(c.get("check_id") == args.check_id for c in state["checks"]):
        raise BuilderStateError(f"Check already exists: {args.check_id}")
    check = {
        "check_id": args.check_id,
        "step_id": attempt["step_id"],
        "attempt_id": args.attempt_id,
        "kind": args.kind,
        "name": args.name,
        "input_hash": args.input_hash,
        "environment": args.environment,
        "result": args.result,
        "exit_code": args.exit_code,
        "summary": args.summary,
        "artifact_refs": list(args.evidence),
        "started_at": args.started_at or now(),
        "finished_at": now(),
    }
    state["checks"].append(check)
    attempt["check_ids"].append(args.check_id)
    append_event(state, "CHECK_PASSED" if args.result == "PASS" else "CHECK_RECORDED", step_id=attempt["step_id"], attempt_id=args.attempt_id, payload={"check_id": args.check_id, "result": args.result})
    touch(state)
    atomic_write(path, state)
    print(f"{args.check_id}: {args.result}")
    return 0


def command_mark_step(args: argparse.Namespace) -> int:
    path = Path(args.state).resolve()
    state = load_state(path)
    check_expected_revision(state, args.expected_revision)
    step = find_step(state, args.step_id)
    if args.stepper_status == "DONE":
        successful = [a for a in state["attempts"] if a.get("step_id") == args.step_id and a.get("state") == "SUCCEEDED"]
        if not successful:
            raise BuilderStateError("Cannot mirror Stepper DONE without a SUCCEEDED Builder attempt")
        if not args.evidence:
            raise BuilderStateError("Cannot mirror Stepper DONE without evidence refs")
        step["builder_state"] = "SUCCEEDED"
    step["stepper_status"] = args.stepper_status
    for ref in args.evidence:
        if ref not in step["evidence_refs"]:
            step["evidence_refs"].append(ref)
    step["updated_at"] = now()
    append_event(state, f"STEP_{args.stepper_status}", step_id=args.step_id, payload={"evidence_refs": args.evidence})
    touch(state)
    atomic_write(path, state)
    print(f"{args.step_id}: {args.stepper_status}")
    return 0


def command_gate(args: argparse.Namespace) -> int:
    path = Path(args.state).resolve()
    state = load_state(path)
    check_expected_revision(state, args.expected_revision)
    gate = next((g for g in state["gates"] if g.get("gate_id") == args.gate_id), None)
    if gate is None:
        raise BuilderStateError(f"Unknown gate: {args.gate_id}")
    if gate.get("critical") and args.result == "CONDITIONAL":
        raise BuilderStateError("Critical gate cannot be CONDITIONAL")
    if args.result == "PASS" and not args.evidence:
        raise BuilderStateError("PASS requires at least one evidence ref")
    if args.result == "N/A" and (gate.get("critical") or not args.condition or not args.evidence):
        raise BuilderStateError("N/A requires a noncritical gate, rationale, and evidence")
    gate.update(
        {
            "result": args.result,
            "candidate_revision": args.candidate_revision,
            "input_hash": args.input_hash,
            "evidence_refs": list(args.evidence),
            "blocker_ids": list(args.blocker_id),
            "condition": args.condition,
            "owner": args.owner,
            "evaluated_at": now(),
        }
    )
    append_event(state, "RELEASE_GATE_PASSED" if args.result == "PASS" else "RELEASE_GATE_EVALUATED", payload={"gate_id": args.gate_id, "result": args.result})
    touch(state)
    atomic_write(path, state)
    print(f"{args.gate_id}: {args.result}")
    return 0


def command_checkpoint(args: argparse.Namespace) -> int:
    path = Path(args.state).resolve()
    state = load_state(path)
    check_expected_revision(state, args.expected_revision)
    checkpoint_id = uid("CP")
    state["continuation"].update(
        {
            "checkpoint_id": checkpoint_id,
            "checkpoint_revision": state["meta"]["revision"] + 1,
            "checkpoint_checksum": None,
            "current_wave_id": state["execution"].get("current_wave_id"),
            "active_attempt_ids": list(state["execution"].get("active_attempt_ids", [])),
            "active_lock_ids": list(state["execution"].get("active_lock_ids", [])),
            "blocker_ids": [b["blocker_id"] for b in state["blockers"] if b.get("status") == "open"],
            "next_exact_action": args.next_action,
            "event_offset": len(state["events"]) + 1,
        }
    )
    append_event(state, "CHECKPOINT_CREATED", payload={"checkpoint_id": checkpoint_id, "reason": args.reason})
    touch(state)
    state["continuation"]["checkpoint_checksum"] = state_digest(state)
    atomic_write(path, state)
    print(checkpoint_id)
    return 0


def command_set_release(args: argparse.Namespace) -> int:
    path = Path(args.state).resolve()
    state = load_state(path)
    check_expected_revision(state, args.expected_revision)
    state["execution"]["candidate_revision"] = args.candidate_revision
    state["execution"]["stepper_release_result"] = args.stepper_release_result
    if args.raw_progress is not None:
        state["execution"]["raw_progress"] = args.raw_progress
    if args.weighted_progress is not None:
        state["execution"]["weighted_progress"] = args.weighted_progress
    append_event(state, "RELEASE_CANDIDATE_SET", payload={"candidate_revision": args.candidate_revision, "stepper_release_result": args.stepper_release_result})
    touch(state)
    atomic_write(path, state)
    print(f"Candidate: {args.candidate_revision}")
    return 0


def command_finalize(args: argparse.Namespace) -> int:
    path = Path(args.state).resolve()
    state = load_state(path)
    check_expected_revision(state, args.expected_revision)
    if not args.artifact:
        raise BuilderStateError("Final handoff requires at least one artifact ref")
    candidate = state["execution"].get("candidate_revision")
    state["handoff"] = {
        "handoff_id": args.handoff_id or uid("BLDH"),
        "status": "BUILD COMPLETE — RELEASE READY",
        "blueprint_checksum": state["inputs"]["blueprint"]["checksum"],
        "stepper_checksum": state["inputs"]["stepper"]["checksum"],
        "candidate_revision": candidate,
        "stepper_release_result": state["execution"].get("stepper_release_result"),
        "gate_results": [{"gate_id": gate["gate_id"], "result": gate["result"]} for gate in state["gates"]],
        "artifact_refs": list(args.artifact),
        "checksum": "",
        "created_at": now(),
    }
    errors = release_errors(state)
    if errors:
        state["handoff"] = None
        raise BuilderStateError("Cannot finalize:\n- " + "\n- ".join(errors))
    state["meta"]["status"] = "BUILD COMPLETE — RELEASE READY"
    append_event(state, "BUILD_RELEASE_READY", payload={"candidate_revision": candidate})
    touch(state)
    state["handoff"]["checksum"] = state_digest(state)
    atomic_write(path, state)
    print(state["handoff"]["handoff_id"])
    return 0


def command_release_check(args: argparse.Namespace) -> int:
    state = load_state(Path(args.state).resolve())
    validation = validate_state(state)
    semantic = release_errors(state)
    errors = []
    for error in validation + semantic:
        if error not in errors:
            errors.append(error)
    if errors:
        print("RELEASE_CHECK: FAIL")
        for error in errors:
            print(f"- {error}")
        return 1
    print("RELEASE_CHECK: PASS")
    print(f"Candidate: {state['execution']['candidate_revision']}")
    print(f"Handoff: {state['handoff']['handoff_id']}")
    return 0


def command_demo(_: argparse.Namespace) -> int:
    with tempfile.TemporaryDirectory(prefix="builder-os-demo-") as temp_dir:
        root = Path(temp_dir)
        state_path = root / "builder-state.json"
        ns = argparse.Namespace(
            state=str(state_path), project_id="demo", project_name="Builder Demo", repo=str(root),
            blueprint_ref=str(root), blueprint_version="1.0.0", blueprint_revision=1,
            blueprint_checksum="sha256:" + "1" * 64, blueprint_handoff_id="BPH-DEMO",
            stepper_ref=str(root), stepper_version="1.0.0", stepper_schema_version=1,
            stepper_checksum="sha256:" + "2" * 64, stepper_status="BUILD READY",
            tracker_ref="demo-tracker", graph_ref="demo-graph", release_target="P0",
            base_branch="main", base_revision="demo-base", integration_branch="main",
            worktree_root=None, max_parallel_steps=1, max_active_modules=1,
            max_repair_attempts=3, no_worktrees=False, allow_dirty_base=False,
            request="demo", actor="demo", locale="en", force=False,
        )
        state = make_state(ns)
        timestamp = now()
        state["steps"].append(
            {
                "step_id": "STEP-000001", "module_id": "MOD-001", "slice_id": "SLICE-001",
                "priority": "P0", "risk": "LOW", "required_for_release": True,
                "stepper_status": "DONE", "builder_state": "SUCCEEDED", "spec_hash": "sha256:demo",
                "attempt_ids": ["ATT-DEMO"], "evidence_refs": ["ART-DEMO"], "updated_at": timestamp,
            }
        )
        state["attempts"].append(
            {
                "attempt_id": "ATT-DEMO", "step_id": "STEP-000001", "number": 1,
                "state": "SUCCEEDED", "worker_id": "demo", "spec_hash": "sha256:demo",
                "context_hash": "sha256:context", "prompt_hash": "sha256:prompt",
                "base_revision": "demo-base", "head_revision": "demo-head",
                "integrated_revision": "demo-release", "worktree": None, "branch": "builder/demo",
                "lock_ids": [], "check_ids": ["CHK-DEMO"], "review_ids": [],
                "artifact_refs": ["ART-DEMO"], "failure_class": None, "summary": "demo",
                "started_at": timestamp, "finished_at": timestamp,
            }
        )
        state["checks"].append(
            {
                "check_id": "CHK-DEMO", "step_id": "STEP-000001", "attempt_id": "ATT-DEMO",
                "kind": "command", "name": "demo check", "input_hash": "sha256:demo-release",
                "environment": "demo", "result": "PASS", "exit_code": 0, "summary": "pass",
                "artifact_refs": ["ART-DEMO"], "started_at": timestamp, "finished_at": timestamp,
            }
        )
        state["artifacts"].append(
            {
                "artifact_id": "ART-DEMO", "kind": "test-report", "uri": "demo://report",
                "digest": "sha256:artifact", "producer": "demo", "step_id": "STEP-000001",
                "attempt_id": "ATT-DEMO", "confidentiality": "internal", "created_at": timestamp,
            }
        )
        state["execution"].update(
            {
                "candidate_revision": "demo-release", "stepper_release_result": "PASS",
                "raw_progress": 1.0, "weighted_progress": 1.0,
            }
        )
        for gate in state["gates"]:
            gate.update(
                {
                    "result": "PASS", "candidate_revision": "demo-release",
                    "input_hash": "sha256:demo-release", "evidence_refs": ["ART-DEMO"],
                    "evaluated_at": timestamp,
                }
            )
        state["handoff"] = {
            "handoff_id": "BLDH-DEMO", "status": "BUILD COMPLETE — RELEASE READY",
            "blueprint_checksum": state["inputs"]["blueprint"]["checksum"],
            "stepper_checksum": state["inputs"]["stepper"]["checksum"],
            "candidate_revision": "demo-release", "stepper_release_result": "PASS",
            "gate_results": [{"gate_id": g["gate_id"], "result": g["result"]} for g in state["gates"]],
            "artifact_refs": ["ART-DEMO"], "checksum": "", "created_at": timestamp,
        }
        state["meta"]["status"] = "BUILD COMPLETE — RELEASE READY"
        state["handoff"]["checksum"] = state_digest(state)
        atomic_write(state_path, state)
        errors = validate_state(load_state(state_path))
        release = release_errors(load_state(state_path))
        if errors or release:
            print("DEMO: FAIL")
            for error in errors + release:
                print(f"- {error}")
            return 1
        print("DEMO: PASS")
        print(f"State: {state_path}")
        return 0


def add_common_mutation_args(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("state")
    parser.add_argument("--expected-revision", type=int)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="builder-os", description="Deterministic Builder {OS} state CLI")
    parser.add_argument("--version", action="version", version=VERSION)
    sub = parser.add_subparsers(dest="command", required=True)

    init = sub.add_parser("init", help="Initialize canonical Builder state")
    init.add_argument("--state", required=True)
    init.add_argument("--project-id", required=True)
    init.add_argument("--project-name", required=True)
    init.add_argument("--repo", required=True)
    init.add_argument("--blueprint-ref")
    init.add_argument("--blueprint-version", required=True)
    init.add_argument("--blueprint-revision", type=int, default=0)
    init.add_argument("--blueprint-checksum")
    init.add_argument("--blueprint-handoff-id", required=True)
    init.add_argument("--stepper-ref")
    init.add_argument("--stepper-version", required=True)
    init.add_argument("--stepper-schema-version", type=int, default=1)
    init.add_argument("--stepper-checksum")
    init.add_argument("--stepper-status", default="BUILD READY")
    init.add_argument("--tracker-ref")
    init.add_argument("--graph-ref")
    init.add_argument("--release-target", default="P0")
    init.add_argument("--base-branch", default="main")
    init.add_argument("--base-revision")
    init.add_argument("--integration-branch")
    init.add_argument("--worktree-root")
    init.add_argument("--max-parallel-steps", type=int, default=4)
    init.add_argument("--max-active-modules", type=int, default=3)
    init.add_argument("--max-repair-attempts", type=int, default=5)
    init.add_argument("--no-worktrees", action="store_true")
    init.add_argument("--allow-dirty-base", action="store_true")
    init.add_argument("--request")
    init.add_argument("--actor", default="builder-os")
    init.add_argument("--locale")
    init.add_argument("--force", action="store_true")
    init.set_defaults(handler=command_init)

    validate = sub.add_parser("validate", help="Validate structure and semantic invariants")
    validate.add_argument("state")
    validate.set_defaults(handler=command_validate)

    status = sub.add_parser("status", help="Show evidence-backed Builder status")
    status.add_argument("state")
    status.set_defaults(handler=command_status)

    sync = sub.add_parser("sync-step", help="Import or refresh one Stepper step mirror")
    add_common_mutation_args(sync)
    sync.add_argument("--step-id", required=True)
    sync.add_argument("--module-id")
    sync.add_argument("--slice-id")
    sync.add_argument("--priority", default="P1")
    sync.add_argument("--risk", choices=sorted({"LOW", "MEDIUM", "HIGH", "CRITICAL"}), default="MEDIUM")
    sync.add_argument("--required-for-release", action="store_true")
    sync.add_argument("--stepper-status", choices=sorted(STEPPER_STATUSES), required=True)
    sync.add_argument("--spec-hash", required=True)
    sync.set_defaults(handler=command_sync_step)

    claim = sub.add_parser("claim", help="Claim a READY Stepper step")
    add_common_mutation_args(claim)
    claim.add_argument("--step-id", required=True)
    claim.add_argument("--attempt-id")
    claim.add_argument("--worker-id", required=True)
    claim.add_argument("--base-revision")
    claim.add_argument("--worktree")
    claim.add_argument("--branch")
    claim.add_argument("--lock-id", action="append", default=[])
    claim.set_defaults(handler=command_claim)

    transition = sub.add_parser("transition", help="Apply a valid Builder attempt transition")
    add_common_mutation_args(transition)
    transition.add_argument("--attempt-id", required=True)
    transition.add_argument("--to", choices=sorted(ATTEMPT_STATES), required=True)
    transition.add_argument("--context-hash")
    transition.add_argument("--prompt-hash")
    transition.add_argument("--head-revision")
    transition.add_argument("--integrated-revision")
    transition.add_argument("--summary")
    transition.add_argument("--failure-class")
    transition.set_defaults(handler=command_transition)

    check = sub.add_parser("record-check", help="Record deterministic check evidence")
    add_common_mutation_args(check)
    check.add_argument("--attempt-id", required=True)
    check.add_argument("--check-id", required=True)
    check.add_argument("--kind", choices=["command", "predicate", "visual", "eval", "artifact"], required=True)
    check.add_argument("--name", required=True)
    check.add_argument("--input-hash", required=True)
    check.add_argument("--environment", default="local")
    check.add_argument("--result", choices=["PASS", "FAIL", "BLOCKED", "N/A"], required=True)
    check.add_argument("--exit-code", type=int)
    check.add_argument("--summary", required=True)
    check.add_argument("--evidence", action="append", default=[])
    check.add_argument("--started-at")
    check.set_defaults(handler=command_record_check)

    mark = sub.add_parser("mark-step", help="Mirror a Stepper status after external Verifier decision")
    add_common_mutation_args(mark)
    mark.add_argument("--step-id", required=True)
    mark.add_argument("--stepper-status", choices=sorted(STEPPER_STATUSES), required=True)
    mark.add_argument("--evidence", action="append", default=[])
    mark.set_defaults(handler=command_mark_step)

    gate = sub.add_parser("gate", help="Evaluate BG01-BG20")
    add_common_mutation_args(gate)
    gate.add_argument("--gate-id", choices=list(GATE_NAMES), required=True)
    gate.add_argument("--result", choices=["PASS", "CONDITIONAL", "FAIL", "N/A"], required=True)
    gate.add_argument("--candidate-revision")
    gate.add_argument("--input-hash", required=True)
    gate.add_argument("--evidence", action="append", default=[])
    gate.add_argument("--blocker-id", action="append", default=[])
    gate.add_argument("--condition")
    gate.add_argument("--owner")
    gate.set_defaults(handler=command_gate)

    checkpoint = sub.add_parser("checkpoint", help="Create a recovery checkpoint")
    add_common_mutation_args(checkpoint)
    checkpoint.add_argument("--reason", required=True)
    checkpoint.add_argument("--next-action", required=True)
    checkpoint.set_defaults(handler=command_checkpoint)

    release = sub.add_parser("set-release", help="Set frozen candidate and Stepper release result")
    add_common_mutation_args(release)
    release.add_argument("--candidate-revision", required=True)
    release.add_argument("--stepper-release-result", choices=["PASS", "FAIL", "BLOCKED", "NOT_EVALUATED"], required=True)
    release.add_argument("--raw-progress", type=float)
    release.add_argument("--weighted-progress", type=float)
    release.set_defaults(handler=command_set_release)

    finalize = sub.add_parser("finalize", help="Create final handoff and terminal status if all gates pass")
    add_common_mutation_args(finalize)
    finalize.add_argument("--handoff-id")
    finalize.add_argument("--artifact", action="append", default=[])
    finalize.set_defaults(handler=command_finalize)

    release_check = sub.add_parser("release-check", help="Evaluate terminal release readiness")
    release_check.add_argument("state")
    release_check.set_defaults(handler=command_release_check)

    demo = sub.add_parser("demo", help="Run an in-memory end-to-end semantic self-test")
    demo.set_defaults(handler=command_demo)
    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    try:
        return int(args.handler(args))
    except BuilderStateError as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""Execution OS V2: deterministic, local-first personal execution engine."""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import sys
from datetime import datetime
from pathlib import Path
from typing import Any


VERSION = "2.0"
OUTCOME_STATUSES = {"candidate", "selected", "active", "at_risk", "verified", "stopped", "superseded"}
COMMITMENT_STATUSES = {"captured", "ready", "active", "blocked", "shipped", "verified", "deferred", "delegated", "cancelled"}
PROMISE_STATUSES = {"open", "at_risk", "delivered", "renegotiated", "cancelled"}
NONTERMINAL_OUTCOMES = {"candidate", "selected", "active", "at_risk"}
TERMINAL_COMMITMENTS = {"verified", "deferred", "delegated", "cancelled"}
SCHEDULER_KEYS = {"t0_capture", "t1_boot", "t2_halt", "t3_reset", "t4_audit"}


def now() -> str:
    return datetime.now().astimezone().isoformat(timespec="seconds")


def today() -> str:
    return datetime.now().astimezone().date().isoformat()


def parse_iso(value: str) -> str:
    try:
        datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError as exc:
        raise ValueError(f"Invalid ISO date/datetime: {value}") from exc
    return value


def next_id(items: list[dict[str, Any]], prefix: str) -> str:
    nums: list[int] = []
    for item in items:
        ident = str(item.get("id", ""))
        if ident.startswith(prefix + "-"):
            try:
                nums.append(int(ident.rsplit("-", 1)[1]))
            except ValueError:
                pass
    return f"{prefix}-{max(nums, default=0) + 1:03d}"


def find(items: list[dict[str, Any]], ident: str) -> dict[str, Any]:
    for item in items:
        if item.get("id") == ident:
            return item
    raise ValueError(f"Unknown id: {ident}")


def bounded_score(value: int, field: str) -> int:
    if value < 1 or value > 5:
        raise ValueError(f"{field} must be between 1 and 5")
    return value


def empty_state(owner: str, timezone: str, max_open: int, max_active: int) -> dict[str, Any]:
    return {
        "version": VERSION,
        "profile": {
            "owner": owner,
            "timezone": timezone,
            "max_open_commitments": max_open,
            "max_active_commitments": max_active,
            "capacity_utilization_target": 0.70,
        },
        "cycle": {
            "id": "SEA-001", "name": "", "starts_on": "", "ends_on": "",
            "primary_domain": "", "constraint": "", "status": "draft",
        },
        "outcomes": [], "milestones": [], "bets": [], "commitments": [],
        "blockers": [], "evidence": [], "signals": [], "captures": [],
        "focus_blocks": [], "promises": [], "reviews": [], "checkins": [],
        "decisions": [], "recovery_plans": [], "events": [],
        "scheduler": {
            "t0_capture": None, "t1_boot": None, "t2_halt": None,
            "t3_reset": None, "t4_audit": None,
            "tomorrow_first_action": "", "current_single_thread": "",
            "capacity_class": "", "usable_minutes": 0,
        },
        "calibration": {"focus_estimated_minutes": 0, "focus_actual_minutes": 0, "completed_blocks": 0},
    }


def migrate_state(state: dict[str, Any]) -> tuple[dict[str, Any], bool]:
    changed = state.get("version") != VERSION
    defaults = empty_state(
        str(state.get("profile", {}).get("owner", "Unknown")),
        str(state.get("profile", {}).get("timezone", "UTC")),
        int(state.get("profile", {}).get("max_open_commitments", 7)),
        int(state.get("profile", {}).get("max_active_commitments", 3)),
    )
    for key, value in defaults.items():
        if key not in state:
            state[key] = value
            changed = True
    for key, value in defaults["profile"].items():
        if key not in state["profile"]:
            state["profile"][key] = value
            changed = True
    for key, value in defaults["scheduler"].items():
        if key not in state["scheduler"]:
            state["scheduler"][key] = value
            changed = True
    for key, value in defaults["calibration"].items():
        if key not in state["calibration"]:
            state["calibration"][key] = value
            changed = True
    state["version"] = VERSION
    return state, changed


def load_state(path: Path) -> dict[str, Any]:
    if not path.exists():
        raise FileNotFoundError(f"State file not found: {path}")
    state = json.loads(path.read_text(encoding="utf-8"))
    return migrate_state(state)[0]


def record_event(state: dict[str, Any], event_type: str, payload: dict[str, Any]) -> str:
    ident = next_id(state["events"], "EVT")
    state["events"].append({"id": ident, "type": event_type, "payload": payload, "recorded_at": now()})
    return ident


def save_state(path: Path, state: dict[str, Any], event_type: str, payload: dict[str, Any]) -> None:
    record_event(state, event_type, payload)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(state, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def validation_errors(state: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    required = set(empty_state("x", "UTC", 7, 3))
    missing = required - set(state)
    if missing:
        return ["Missing top-level keys: " + ", ".join(sorted(missing))]
    if state.get("version") != VERSION:
        errors.append(f"State version must be {VERSION}")
    for key in SCHEDULER_KEYS:
        if key not in state["scheduler"]:
            errors.append(f"Scheduler missing {key}")

    outcome_ids = [x.get("id") for x in state["outcomes"]]
    if len(outcome_ids) != len(set(outcome_ids)):
        errors.append("Duplicate outcome ids")
    primaries = [x for x in state["outcomes"] if x.get("priority") == "primary" and x.get("status") in NONTERMINAL_OUTCOMES]
    if len(primaries) > 1:
        errors.append("More than one nonterminal primary outcome")
    active_outcomes = [x for x in state["outcomes"] if x.get("status") in {"selected", "active", "at_risk"}]
    if len(active_outcomes) > 3:
        errors.append("Active outcome ceiling 3 exceeded")
    for outcome in state["outcomes"]:
        fields = ("id", "title", "domain", "baseline", "target", "deadline", "definition_of_done", "proof_required", "priority", "status", "created_at")
        for field in fields:
            if outcome.get(field) in (None, ""):
                errors.append(f"{outcome.get('id', 'outcome')} missing {field}")
        if outcome.get("status") not in OUTCOME_STATUSES:
            errors.append(f"{outcome.get('id')} has invalid status")
        try:
            parse_iso(str(outcome.get("deadline", "")))
        except ValueError as exc:
            errors.append(str(exc))

    commitment_ids = [x.get("id") for x in state["commitments"]]
    if len(commitment_ids) != len(set(commitment_ids)):
        errors.append("Duplicate commitment ids")
    active = [x for x in state["commitments"] if x.get("status") == "active"]
    if len(active) > int(state["profile"].get("max_active_commitments", 3)):
        errors.append("Active commitment WIP limit exceeded")
    open_items = [x for x in state["commitments"] if x.get("status") not in TERMINAL_COMMITMENTS]
    if len(open_items) > int(state["profile"].get("max_open_commitments", 7)):
        errors.append("Open commitment WIP ceiling exceeded")
    evidence_commitments = {x.get("commitment_id") for x in state["evidence"]}
    blocker_commitments = {x.get("commitment_id") for x in state["blockers"] if x.get("status") == "open" and x.get("reason") and x.get("next_action")}
    for item in state["commitments"]:
        fields = ("id", "outcome_id", "owner", "title", "next_action", "definition_of_done", "estimate_minutes", "due_at", "impact", "urgency", "leverage", "confidence", "context_switch_cost", "status", "created_at")
        for field in fields:
            if item.get(field) in (None, ""):
                errors.append(f"{item.get('id', 'commitment')} missing {field}")
        if item.get("outcome_id") not in outcome_ids:
            errors.append(f"{item.get('id')} links unknown outcome")
        if item.get("status") not in COMMITMENT_STATUSES:
            errors.append(f"{item.get('id')} has invalid status")
        if item.get("status") == "verified" and item.get("id") not in evidence_commitments:
            errors.append(f"{item.get('id')} is verified without evidence")
        if item.get("status") == "blocked" and item.get("id") not in blocker_commitments:
            errors.append(f"{item.get('id')} is blocked without open blocker")
        try:
            parse_iso(str(item.get("due_at", "")))
        except ValueError as exc:
            errors.append(str(exc))

    active_focus = [x for x in state["focus_blocks"] if x.get("status") == "active"]
    if len(active_focus) > 1:
        errors.append("Single Thread violated: more than one active focus block")
    event_ids = [x.get("id") for x in state["events"]]
    if len(event_ids) != len(set(event_ids)):
        errors.append("Duplicate event ids")
    for promise in state["promises"]:
        if promise.get("status") not in PROMISE_STATUSES:
            errors.append(f"{promise.get('id')} has invalid promise status")
        for field in ("id", "stakeholder", "deliverable", "due_at", "notice_by", "consequence", "next_proof", "status"):
            if promise.get(field) in (None, ""):
                errors.append(f"{promise.get('id', 'promise')} missing {field}")
        for field in ("due_at", "notice_by"):
            try:
                parse_iso(str(promise.get(field, "")))
            except ValueError as exc:
                errors.append(str(exc))
    return errors


def cmd_init(args: argparse.Namespace, path: Path) -> dict[str, Any]:
    if path.exists() and not args.force:
        raise ValueError("State file already exists; use --force only when replacement is intentional")
    if args.max_open < 1 or args.max_active < 1 or args.max_active > args.max_open:
        raise ValueError("WIP limits must be positive and max-active cannot exceed max-open")
    state = empty_state(args.owner, args.timezone, args.max_open, args.max_active)
    save_state(path, state, "SYSTEM_INITIALIZED", {"owner": args.owner, "version": VERSION})
    return {"created": str(path), "owner": args.owner, "version": VERSION}


def cmd_migrate(args: argparse.Namespace, path: Path) -> dict[str, Any]:
    raw = json.loads(path.read_text(encoding="utf-8"))
    previous = str(raw.get("version", "unknown"))
    state, changed = migrate_state(raw)
    if changed:
        save_state(path, state, "STATE_MIGRATED", {"from": previous, "to": VERSION})
    return {"migrated": changed, "from": previous, "to": VERSION}


def cmd_add_outcome(args: argparse.Namespace, path: Path) -> dict[str, Any]:
    state = load_state(path)
    active_outcomes = [x for x in state["outcomes"] if x.get("status") in {"selected", "active", "at_risk"}]
    if len(active_outcomes) >= 3:
        raise ValueError("Active outcome ceiling 3 reached; stop, verify, or pause an outcome first")
    if args.priority == "primary":
        existing = [x for x in state["outcomes"] if x.get("priority") == "primary" and x.get("status") in NONTERMINAL_OUTCOMES]
        if existing:
            raise ValueError(f"Primary outcome already exists: {existing[0]['id']}")
    ident = next_id(state["outcomes"], "OUT")
    state["outcomes"].append({
        "id": ident, "title": args.title, "domain": args.domain, "baseline": args.baseline,
        "target": args.target, "deadline": parse_iso(args.deadline), "definition_of_done": args.done,
        "proof_required": args.proof, "priority": args.priority, "status": "active",
        "confidence": args.confidence, "created_at": now(),
    })
    save_state(path, state, "OUTCOME_CREATED", {"outcome_id": ident})
    return {"created": ident}


def cmd_close_outcome(args: argparse.Namespace, path: Path) -> dict[str, Any]:
    state = load_state(path)
    outcome = find(state["outcomes"], args.outcome)
    target_status = "verified" if args.status == "verified" else args.status
    if target_status == "verified" and not args.evidence:
        raise ValueError("Verified outcome closure requires --evidence")
    outcome["status"] = target_status
    outcome["closed_at"] = now()
    outcome["closing_note"] = args.note
    if args.evidence:
        outcome["closing_evidence"] = args.evidence
    save_state(path, state, "OUTCOME_CLOSED", {"outcome_id": outcome["id"], "status": target_status})
    return {"closed": outcome["id"], "status": target_status}


def cmd_add_commitment(args: argparse.Namespace, path: Path) -> dict[str, Any]:
    state = load_state(path)
    find(state["outcomes"], args.outcome)
    open_items = [x for x in state["commitments"] if x.get("status") not in TERMINAL_COMMITMENTS]
    ceiling = int(state["profile"].get("max_open_commitments", 7))
    if len(open_items) >= ceiling:
        raise ValueError(f"Open commitment ceiling {ceiling} reached; close, kill, renegotiate, delegate, or park one first")
    ident = next_id(state["commitments"], "COM")
    item = {
        "id": ident, "outcome_id": args.outcome, "owner": args.owner or state["profile"]["owner"],
        "title": args.title, "next_action": args.next_action, "definition_of_done": args.done,
        "estimate_minutes": args.minutes, "due_at": parse_iso(args.due),
        "impact": bounded_score(args.impact, "impact"), "urgency": bounded_score(args.urgency, "urgency"),
        "leverage": bounded_score(args.leverage, "leverage"), "confidence": bounded_score(args.confidence, "confidence"),
        "context_switch_cost": bounded_score(args.switch_cost, "switch-cost"), "status": "ready",
        "created_at": now(), "history": [],
    }
    state["commitments"].append(item)
    save_state(path, state, "COMMITMENT_CREATED", {"commitment_id": ident, "outcome_id": args.outcome})
    return {"created": ident}


def transition_commitment(state: dict[str, Any], ident: str, status: str, note: str, extra: dict[str, Any] | None = None) -> dict[str, Any]:
    item = find(state["commitments"], ident)
    before = item["status"]
    item["status"] = status
    item.setdefault("history", []).append({"from": before, "to": status, "note": note, "at": now()})
    if extra:
        item.update(extra)
    return item


def cmd_start(args: argparse.Namespace, path: Path) -> dict[str, Any]:
    state = load_state(path)
    item = find(state["commitments"], args.commitment)
    active = [x for x in state["commitments"] if x.get("status") == "active" and x.get("id") != item["id"]]
    limit = int(state["profile"].get("max_active_commitments", 3))
    if len(active) >= limit:
        raise ValueError(f"WIP limit {limit} reached; finish, block, defer, or cancel first")
    if item.get("status") not in {"captured", "ready", "blocked", "active"}:
        raise ValueError(f"Cannot start commitment in status {item.get('status')}")
    transition_commitment(state, item["id"], "active", "started", {"started_at": item.get("started_at") or now()})
    save_state(path, state, "COMMITMENT_STARTED", {"commitment_id": item["id"]})
    return {"started": item["id"], "definition_of_done": item["definition_of_done"], "first_action": item["next_action"]}


def cmd_complete(args: argparse.Namespace, path: Path) -> dict[str, Any]:
    state = load_state(path)
    item = find(state["commitments"], args.commitment)
    if item.get("status") in TERMINAL_COMMITMENTS:
        raise ValueError(f"Cannot complete commitment in status {item.get('status')}")
    evidence_id = next_id(state["evidence"], "EVD")
    state["evidence"].append({
        "id": evidence_id, "commitment_id": item["id"], "kind": args.kind,
        "value": args.evidence, "acceptance": args.acceptance, "captured_at": now(),
    })
    transition_commitment(state, item["id"], "verified", "evidence accepted", {"verified_at": now()})
    save_state(path, state, "COMMITMENT_VERIFIED", {"commitment_id": item["id"], "evidence_id": evidence_id})
    return {"verified": item["id"], "evidence": evidence_id}


def cmd_block(args: argparse.Namespace, path: Path) -> dict[str, Any]:
    state = load_state(path)
    item = find(state["commitments"], args.commitment)
    blocker_id = next_id(state["blockers"], "BLK")
    state["blockers"].append({
        "id": blocker_id, "commitment_id": item["id"], "reason": args.reason,
        "next_action": args.next_action, "owner": args.owner or item["owner"],
        "escalate_at": parse_iso(args.escalate_at) if args.escalate_at else "",
        "status": "open", "created_at": now(),
    })
    transition_commitment(state, item["id"], "blocked", args.reason, {"next_action": args.next_action})
    save_state(path, state, "COMMITMENT_BLOCKED", {"commitment_id": item["id"], "blocker_id": blocker_id})
    return {"blocked": item["id"], "blocker": blocker_id}


def cmd_unblock(args: argparse.Namespace, path: Path) -> dict[str, Any]:
    state = load_state(path)
    item = find(state["commitments"], args.commitment)
    for blocker in state["blockers"]:
        if blocker.get("commitment_id") == item["id"] and blocker.get("status") == "open":
            blocker["status"] = "resolved"
            blocker["resolved_at"] = now()
            blocker["resolution"] = args.resolution
    transition_commitment(state, item["id"], "ready", args.resolution, {"next_action": args.next_action})
    save_state(path, state, "COMMITMENT_UNBLOCKED", {"commitment_id": item["id"]})
    return {"unblocked": item["id"], "next_action": args.next_action}


def cmd_defer(args: argparse.Namespace, path: Path) -> dict[str, Any]:
    state = load_state(path)
    item = transition_commitment(state, args.commitment, "deferred", args.reason, {"review_on": parse_iso(args.review_on)})
    save_state(path, state, "COMMITMENT_DEFERRED", {"commitment_id": item["id"], "review_on": args.review_on})
    return {"deferred": item["id"], "review_on": args.review_on}


def cmd_cancel(args: argparse.Namespace, path: Path) -> dict[str, Any]:
    state = load_state(path)
    item = transition_commitment(state, args.commitment, "cancelled", args.reason, {"cancelled_at": now()})
    save_state(path, state, "COMMITMENT_CANCELLED", {"commitment_id": item["id"], "reason": args.reason})
    return {"cancelled": item["id"]}


def cmd_delegate(args: argparse.Namespace, path: Path) -> dict[str, Any]:
    state = load_state(path)
    item = transition_commitment(state, args.commitment, "delegated", args.note, {
        "delegate": args.to, "delegated_at": now(), "delegated_due_at": parse_iso(args.due),
    })
    save_state(path, state, "COMMITMENT_DELEGATED", {"commitment_id": item["id"], "delegate": args.to})
    return {"delegated": item["id"], "to": args.to, "follow_up": args.due}


def cmd_capture(args: argparse.Namespace, path: Path) -> dict[str, Any]:
    state = load_state(path)
    ident = next_id(state["captures"], "CAP")
    item = {"id": ident, "text": args.text, "kind": args.kind, "status": "inbox", "captured_at": now()}
    state["captures"].append(item)
    state["scheduler"]["t0_capture"] = now()
    save_state(path, state, "T0_CAPTURED", {"capture_id": ident, "kind": args.kind})
    return {"captured": ident, "instruction": "Return to the last explicit next action."}


def priority_score(item: dict[str, Any]) -> int:
    return int(item["impact"]) * 4 + int(item["urgency"]) * 3 + int(item["leverage"]) * 2 + int(item["confidence"]) - int(item["context_switch_cost"])


def choose_next(state: dict[str, Any]) -> dict[str, Any] | None:
    ready = [x for x in state["commitments"] if x.get("status") in {"ready", "active"}]
    if not ready:
        return None
    return sorted(ready, key=lambda x: (-priority_score(x), x["due_at"]))[0]


def cmd_next(args: argparse.Namespace, path: Path) -> dict[str, Any]:
    chosen = choose_next(load_state(path))
    if not chosen:
        return {"next": None, "message": "No ready commitment"}
    return {"next": chosen["id"], "title": chosen["title"], "first_action": chosen["next_action"], "definition_of_done": chosen["definition_of_done"], "score": priority_score(chosen)}


def cmd_boot(args: argparse.Namespace, path: Path) -> dict[str, Any]:
    state = load_state(path)
    chosen = find(state["commitments"], args.commitment) if args.commitment else choose_next(state)
    if not chosen:
        raise ValueError("T1 BOOT requires at least one ready commitment")
    capacity = args.capacity.upper()
    allowed = 3 if capacity == "GREEN" else 2 if capacity == "AMBER" else 1
    state["scheduler"].update({
        "t1_boot": now(), "capacity_class": capacity, "usable_minutes": args.usable_minutes,
        "current_single_thread": chosen["id"],
    })
    review_id = next_id(state["reviews"], "REV")
    state["reviews"].append({
        "id": review_id, "type": "boot", "capacity": capacity, "usable_minutes": args.usable_minutes,
        "must_win": args.must_win, "single_thread": chosen["id"], "first_action": chosen["next_action"],
        "not_today": args.not_today, "created_at": now(),
    })
    save_state(path, state, "T1_BOOT_COMPLETED", {"review_id": review_id, "single_thread": chosen["id"]})
    return {"capacity": capacity, "commitment_limit": allowed, "must_win": args.must_win, "single_thread": chosen["id"], "first_action": chosen["next_action"], "not_today": args.not_today}


def cmd_focus(args: argparse.Namespace, path: Path) -> dict[str, Any]:
    state = load_state(path)
    if any(x.get("status") == "active" for x in state["focus_blocks"]):
        raise ValueError("Single Thread already active; end the current focus block first")
    item = find(state["commitments"], args.commitment)
    if item["status"] not in {"ready", "active"}:
        raise ValueError(f"Cannot focus commitment in status {item['status']}")
    active_commitments = [x for x in state["commitments"] if x.get("status") == "active" and x.get("id") != item["id"]]
    if len(active_commitments) >= int(state["profile"]["max_active_commitments"]):
        raise ValueError("Active commitment WIP limit reached")
    transition_commitment(state, item["id"], "active", "focus block started", {"started_at": item.get("started_at") or now()})
    ident = next_id(state["focus_blocks"], "FOC")
    state["focus_blocks"].append({
        "id": ident, "commitment_id": item["id"], "planned_minutes": args.minutes,
        "distraction_rule": args.distraction_rule, "stop_condition": args.stop_condition or item["definition_of_done"],
        "status": "active", "started_at": now(),
    })
    state["scheduler"]["current_single_thread"] = item["id"]
    save_state(path, state, "FOCUS_STARTED", {"focus_id": ident, "commitment_id": item["id"]})
    return {"focus": ident, "commitment": item["id"], "minutes": args.minutes, "first_action": item["next_action"], "stop_condition": args.stop_condition or item["definition_of_done"]}


def cmd_focus_end(args: argparse.Namespace, path: Path) -> dict[str, Any]:
    state = load_state(path)
    block = find(state["focus_blocks"], args.focus)
    if block.get("status") != "active":
        raise ValueError("Focus block is not active")
    if args.actual_minutes < 1:
        raise ValueError("actual-minutes must be at least 1")
    item = find(state["commitments"], block["commitment_id"])
    block.update({"status": "completed", "ended_at": now(), "actual_minutes": args.actual_minutes, "output": args.output})
    item["next_action"] = args.next_action
    state["calibration"]["focus_estimated_minutes"] += int(block["planned_minutes"])
    state["calibration"]["focus_actual_minutes"] += args.actual_minutes
    state["calibration"]["completed_blocks"] += 1
    state["scheduler"]["current_single_thread"] = ""
    save_state(path, state, "FOCUS_ENDED", {"focus_id": block["id"], "actual_minutes": args.actual_minutes})
    ratio = round(state["calibration"]["focus_actual_minutes"] / max(1, state["calibration"]["focus_estimated_minutes"]), 2)
    return {"focus": block["id"], "output": args.output, "next_action": args.next_action, "estimate_ratio": ratio}


def cmd_halt(args: argparse.Namespace, path: Path) -> dict[str, Any]:
    state = load_state(path)
    if any(x.get("status") == "active" for x in state["focus_blocks"]):
        raise ValueError("End the active focus block before T2 HALT")
    ident = next_id(state["reviews"], "REV")
    state["reviews"].append({
        "id": ident, "type": "halt", "proof": args.proof, "classification": args.classification,
        "energy": bounded_score(args.energy, "energy"), "focus": bounded_score(args.focus, "focus"),
        "friction": args.friction, "tomorrow_first_action": args.tomorrow, "created_at": now(),
    })
    state["scheduler"]["t2_halt"] = now()
    state["scheduler"]["tomorrow_first_action"] = args.tomorrow
    save_state(path, state, "T2_HALT_COMPLETED", {"review_id": ident, "classification": args.classification})
    return {"halt": ident, "day_closed": True, "tomorrow_first_action": args.tomorrow}


def cmd_reset(args: argparse.Namespace, path: Path) -> dict[str, Any]:
    state = load_state(path)
    open_items = [x for x in state["commitments"] if x.get("status") not in TERMINAL_COMMITMENTS]
    stale = [x["id"] for x in open_items if x.get("status") in {"blocked", "captured"}]
    missing_next = [x["id"] for x in open_items if not x.get("next_action")]
    ident = next_id(state["reviews"], "REV")
    state["reviews"].append({
        "id": ident, "type": "reset", "weekly_truth": args.weekly_truth,
        "next_week_win": args.next_week_win, "system_experiment": args.system_experiment,
        "open_commitments": len(open_items), "stale_items": stale, "created_at": now(),
    })
    state["scheduler"]["t3_reset"] = now()
    save_state(path, state, "T3_RESET_COMPLETED", {"review_id": ident, "open_commitments": len(open_items)})
    return {"reset": ident, "weekly_truth": args.weekly_truth, "next_week_win": args.next_week_win, "open_commitments": len(open_items), "stale_items": stale, "missing_next_actions": missing_next}


def cmd_audit(args: argparse.Namespace, path: Path) -> dict[str, Any]:
    state = load_state(path)
    moved = {x.get("outcome_id") for x in state["signals"][-30:]}
    active = [x for x in state["outcomes"] if x.get("status") in {"active", "at_risk", "selected"}]
    no_signal = [x["id"] for x in active if x["id"] not in moved]
    ident = next_id(state["reviews"], "REV")
    state["reviews"].append({
        "id": ident, "type": "audit", "decision": args.decision, "system_change": args.system_change,
        "obsolete_killed": args.obsolete_killed, "active_without_recent_signal": no_signal, "created_at": now(),
    })
    state["scheduler"]["t4_audit"] = now()
    decision_id = next_id(state["decisions"], "DEC")
    state["decisions"].append({"id": decision_id, "decision": args.decision, "reason": args.reason, "created_at": now()})
    save_state(path, state, "T4_AUDIT_COMPLETED", {"review_id": ident, "decision_id": decision_id})
    return {"audit": ident, "decision": args.decision, "system_change": args.system_change, "active_without_recent_signal": no_signal}


def cmd_add_promise(args: argparse.Namespace, path: Path) -> dict[str, Any]:
    state = load_state(path)
    if args.commitment:
        find(state["commitments"], args.commitment)
    ident = next_id(state["promises"], "PRM")
    state["promises"].append({
        "id": ident, "stakeholder": args.stakeholder, "deliverable": args.deliverable,
        "due_at": parse_iso(args.due), "notice_by": parse_iso(args.notice_by),
        "consequence": args.consequence, "next_proof": args.next_proof,
        "commitment_id": args.commitment or "", "status": "open", "history": [], "created_at": now(),
    })
    save_state(path, state, "PROMISE_CREATED", {"promise_id": ident, "stakeholder": args.stakeholder})
    return {"created": ident}


def cmd_renegotiate_promise(args: argparse.Namespace, path: Path) -> dict[str, Any]:
    state = load_state(path)
    item = find(state["promises"], args.promise)
    item.setdefault("history", []).append({"due_at": item["due_at"], "next_proof": item["next_proof"], "note": args.note, "changed_at": now()})
    item.update({"due_at": parse_iso(args.due), "notice_by": parse_iso(args.notice_by), "next_proof": args.next_proof, "status": "renegotiated"})
    save_state(path, state, "PROMISE_RENEGOTIATED", {"promise_id": item["id"], "new_due_at": args.due})
    return {"renegotiated": item["id"], "new_due_at": args.due}


def cmd_deliver_promise(args: argparse.Namespace, path: Path) -> dict[str, Any]:
    state = load_state(path)
    item = find(state["promises"], args.promise)
    item.update({"status": "delivered", "delivered_at": now(), "evidence": args.evidence})
    save_state(path, state, "PROMISE_DELIVERED", {"promise_id": item["id"], "evidence": args.evidence})
    return {"delivered": item["id"], "evidence": args.evidence}


def cmd_promises(args: argparse.Namespace, path: Path) -> dict[str, Any]:
    state = load_state(path)
    current = datetime.now().astimezone()
    rows = []
    for item in state["promises"]:
        if item["status"] in {"delivered", "cancelled"} and not args.all:
            continue
        due = datetime.fromisoformat(item["due_at"].replace("Z", "+00:00"))
        notice = datetime.fromisoformat(item["notice_by"].replace("Z", "+00:00"))
        if due.tzinfo is None:
            due = due.replace(tzinfo=current.tzinfo)
        if notice.tzinfo is None:
            notice = notice.replace(tzinfo=current.tzinfo)
        risk = "OVERDUE" if due < current else "NOTICE" if notice <= current else "OPEN"
        rows.append({"id": item["id"], "stakeholder": item["stakeholder"], "deliverable": item["deliverable"], "due_at": item["due_at"], "status": item["status"], "risk": risk, "next_proof": item["next_proof"]})
    return {"promises": sorted(rows, key=lambda x: x["due_at"]), "count": len(rows)}


def cmd_signal(args: argparse.Namespace, path: Path) -> dict[str, Any]:
    state = load_state(path)
    find(state["outcomes"], args.outcome)
    ident = next_id(state["signals"], "SIG")
    state["signals"].append({"id": ident, "outcome_id": args.outcome, "fact": args.fact, "metric": args.metric, "captured_at": now()})
    save_state(path, state, "SIGNAL_RECORDED", {"signal_id": ident, "outcome_id": args.outcome})
    return {"signal": ident}


def cmd_context_capsule(args: argparse.Namespace, path: Path) -> dict[str, Any]:
    state = load_state(path)
    item = find(state["commitments"], args.commitment)
    outcome = find(state["outcomes"], item["outcome_id"])
    blocks = [x for x in state["focus_blocks"] if x.get("commitment_id") == item["id"]]
    blockers = [x for x in state["blockers"] if x.get("commitment_id") == item["id"] and x.get("status") == "open"]
    evidence = [x for x in state["evidence"] if x.get("commitment_id") == item["id"]]
    return {
        "commitment": item["id"], "outcome": {"id": outcome["id"], "title": outcome["title"], "target": outcome["target"]},
        "status": item["status"], "definition_of_done": item["definition_of_done"],
        "last_output": blocks[-1].get("output", "") if blocks else "", "open_blockers": blockers,
        "evidence": evidence, "resume_with": item["next_action"],
    }


def cmd_check_in(args: argparse.Namespace, path: Path) -> dict[str, Any]:
    state = load_state(path)
    ident = next_id(state["checkins"], "CHK")
    state["checkins"].append({"id": ident, "energy": bounded_score(args.energy, "energy"), "focus": bounded_score(args.focus, "focus"), "win": args.win, "friction": args.friction, "next": args.next, "created_at": now()})
    save_state(path, state, "CHECK_IN_RECORDED", {"check_in_id": ident})
    return {"check_in": ident}


def cmd_status(args: argparse.Namespace, path: Path) -> dict[str, Any]:
    state = load_state(path)
    commitments: dict[str, int] = {status: 0 for status in sorted(COMMITMENT_STATUSES)}
    outcomes: dict[str, int] = {status: 0 for status in sorted(OUTCOME_STATUSES)}
    for item in state["commitments"]:
        commitments[item["status"]] = commitments.get(item["status"], 0) + 1
    for item in state["outcomes"]:
        outcomes[item["status"]] = outcomes.get(item["status"], 0) + 1
    open_items = [x for x in state["commitments"] if x.get("status") not in TERMINAL_COMMITMENTS]
    ratio = round(state["calibration"]["focus_actual_minutes"] / max(1, state["calibration"]["focus_estimated_minutes"]), 2)
    return {
        "version": state["version"], "owner": state["profile"]["owner"], "scheduler": state["scheduler"],
        "outcomes": outcomes, "commitments": commitments, "open_commitments": sorted(open_items, key=lambda x: x["due_at"]),
        "open_commitment_count": len(open_items), "open_commitment_ceiling": state["profile"]["max_open_commitments"],
        "open_promises": len([x for x in state["promises"] if x["status"] not in {"delivered", "cancelled"}]),
        "evidence_count": len(state["evidence"]), "signal_count": len(state["signals"]),
        "event_count": len(state["events"]), "estimate_ratio": ratio,
    }


def cmd_backup(args: argparse.Namespace, path: Path) -> dict[str, Any]:
    if not path.exists():
        raise FileNotFoundError(f"State file not found: {path}")
    output = args.output or path.with_name(f"{path.stem}.backup-{today()}{path.suffix}")
    if output.resolve() == path.resolve():
        raise ValueError("Backup output must differ from the state path")
    output.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(path, output)
    digest = hashlib.sha256(output.read_bytes()).hexdigest()
    return {"backup": str(output), "sha256": digest}


def cmd_validate(args: argparse.Namespace, path: Path) -> dict[str, Any]:
    errors = validation_errors(load_state(path))
    return {"valid": not errors, "errors": errors}


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(description="Execution OS V2 state engine")
    root.add_argument("--state", required=True, type=Path)
    commands = root.add_subparsers(dest="command", required=True)

    p = commands.add_parser("init")
    p.add_argument("--owner", required=True); p.add_argument("--timezone", default="UTC")
    p.add_argument("--max-open", type=int, default=7); p.add_argument("--max-active", type=int, default=3); p.add_argument("--force", action="store_true")
    commands.add_parser("migrate")

    p = commands.add_parser("add-outcome")
    for flag in ("title", "domain", "baseline", "target", "deadline", "done", "proof"):
        p.add_argument("--" + flag.replace("_", "-"), required=True)
    p.add_argument("--priority", choices=["primary", "secondary", "maintenance"], required=True); p.add_argument("--confidence", type=int, default=3)
    p = commands.add_parser("close-outcome"); p.add_argument("outcome"); p.add_argument("--status", choices=["verified", "stopped", "superseded"], required=True); p.add_argument("--note", required=True); p.add_argument("--evidence", default="")

    p = commands.add_parser("add-commitment")
    p.add_argument("--outcome", required=True); p.add_argument("--owner"); p.add_argument("--title", required=True); p.add_argument("--next-action", required=True); p.add_argument("--done", required=True); p.add_argument("--minutes", type=int, required=True); p.add_argument("--due", required=True)
    p.add_argument("--impact", type=int, default=3); p.add_argument("--urgency", type=int, default=3); p.add_argument("--leverage", type=int, default=3); p.add_argument("--confidence", type=int, default=3); p.add_argument("--switch-cost", type=int, default=3)
    p = commands.add_parser("start"); p.add_argument("commitment")
    p = commands.add_parser("complete"); p.add_argument("commitment"); p.add_argument("--kind", required=True); p.add_argument("--evidence", required=True); p.add_argument("--acceptance", required=True)
    p = commands.add_parser("block"); p.add_argument("commitment"); p.add_argument("--reason", required=True); p.add_argument("--next-action", required=True); p.add_argument("--owner"); p.add_argument("--escalate-at")
    p = commands.add_parser("unblock"); p.add_argument("commitment"); p.add_argument("--resolution", required=True); p.add_argument("--next-action", required=True)
    p = commands.add_parser("defer"); p.add_argument("commitment"); p.add_argument("--reason", required=True); p.add_argument("--review-on", required=True)
    p = commands.add_parser("cancel"); p.add_argument("commitment"); p.add_argument("--reason", required=True)
    p = commands.add_parser("delegate"); p.add_argument("commitment"); p.add_argument("--to", required=True); p.add_argument("--due", required=True); p.add_argument("--note", required=True)

    p = commands.add_parser("capture"); p.add_argument("--text", required=True); p.add_argument("--kind", choices=["O", "C", "A", "?"], default="?")
    p = commands.add_parser("boot"); p.add_argument("--capacity", choices=["GREEN", "AMBER", "RED"], required=True); p.add_argument("--usable-minutes", type=int, required=True); p.add_argument("--must-win", required=True); p.add_argument("--not-today", default=""); p.add_argument("--commitment")
    p = commands.add_parser("focus"); p.add_argument("commitment"); p.add_argument("--minutes", type=int, choices=[15, 25, 50, 90], required=True); p.add_argument("--distraction-rule", default="Phone away; capture and return"); p.add_argument("--stop-condition", default="")
    p = commands.add_parser("focus-end"); p.add_argument("focus"); p.add_argument("--actual-minutes", type=int, required=True); p.add_argument("--output", required=True); p.add_argument("--next-action", required=True)
    p = commands.add_parser("halt"); p.add_argument("--proof", required=True); p.add_argument("--classification", choices=["SHIPPED", "VERIFIED", "PROGRESSED", "TOUCHED", "ABANDONED"], required=True); p.add_argument("--energy", type=int, required=True); p.add_argument("--focus", type=int, required=True); p.add_argument("--friction", required=True); p.add_argument("--tomorrow", required=True)
    p = commands.add_parser("reset"); p.add_argument("--weekly-truth", required=True); p.add_argument("--next-week-win", required=True); p.add_argument("--system-experiment", required=True)
    p = commands.add_parser("audit"); p.add_argument("--decision", required=True); p.add_argument("--reason", required=True); p.add_argument("--system-change", required=True); p.add_argument("--obsolete-killed", required=True)

    p = commands.add_parser("add-promise"); p.add_argument("--stakeholder", required=True); p.add_argument("--deliverable", required=True); p.add_argument("--due", required=True); p.add_argument("--notice-by", required=True); p.add_argument("--consequence", required=True); p.add_argument("--next-proof", required=True); p.add_argument("--commitment")
    p = commands.add_parser("renegotiate-promise"); p.add_argument("promise"); p.add_argument("--due", required=True); p.add_argument("--notice-by", required=True); p.add_argument("--next-proof", required=True); p.add_argument("--note", required=True)
    p = commands.add_parser("deliver-promise"); p.add_argument("promise"); p.add_argument("--evidence", required=True)
    p = commands.add_parser("promises"); p.add_argument("--all", action="store_true")

    p = commands.add_parser("signal"); p.add_argument("--outcome", required=True); p.add_argument("--fact", required=True); p.add_argument("--metric", default="")
    p = commands.add_parser("context-capsule"); p.add_argument("commitment")
    p = commands.add_parser("check-in"); p.add_argument("--energy", type=int, required=True); p.add_argument("--focus", type=int, required=True); p.add_argument("--win", required=True); p.add_argument("--friction", required=True); p.add_argument("--next", required=True)
    p = commands.add_parser("backup"); p.add_argument("--output", type=Path)
    commands.add_parser("next"); commands.add_parser("status"); commands.add_parser("validate")
    return root


COMMANDS = {
    "init": cmd_init, "migrate": cmd_migrate, "add-outcome": cmd_add_outcome, "close-outcome": cmd_close_outcome,
    "add-commitment": cmd_add_commitment, "start": cmd_start, "complete": cmd_complete, "block": cmd_block,
    "unblock": cmd_unblock, "defer": cmd_defer, "cancel": cmd_cancel, "delegate": cmd_delegate,
    "capture": cmd_capture, "boot": cmd_boot, "focus": cmd_focus, "focus-end": cmd_focus_end,
    "halt": cmd_halt, "reset": cmd_reset, "audit": cmd_audit, "add-promise": cmd_add_promise,
    "renegotiate-promise": cmd_renegotiate_promise, "deliver-promise": cmd_deliver_promise,
    "promises": cmd_promises, "signal": cmd_signal, "context-capsule": cmd_context_capsule,
    "check-in": cmd_check_in, "backup": cmd_backup, "next": cmd_next, "status": cmd_status, "validate": cmd_validate,
}


def main() -> int:
    args = parser().parse_args()
    try:
        result = COMMANDS[args.command](args, args.state)
        print(json.dumps(result, indent=2, ensure_ascii=False))
        return 1 if args.command == "validate" and not result["valid"] else 0
    except (ValueError, FileNotFoundError, json.JSONDecodeError) as exc:
        print(json.dumps({"error": str(exc)}, ensure_ascii=False), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())

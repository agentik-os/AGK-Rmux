#!/usr/bin/env python3
"""Deterministic state engine for Habit Tracker {OS}.

The LLM owns conversation, interpretation, and coaching. This module owns typed
state, append-only evidence, reproducible metrics, compact context, and Mermaid
output. It intentionally uses only Python's standard library.
"""

from __future__ import annotations

import argparse
import csv
import io
import json
import sqlite3
import sys
import uuid
from collections import Counter
from dataclasses import asdict, dataclass
from datetime import date, datetime, time, timedelta
from pathlib import Path
from statistics import median
from typing import Any, Iterable, Sequence
from zoneinfo import ZoneInfo, ZoneInfoNotFoundError


SCHEMA_VERSION = "1.0"
DEFAULT_DB = "habit-tracker.db"

KINDS = {"build", "maintain", "reduce", "stop"}
STATUSES = {"draft", "active", "paused", "recovering", "retired", "archived"}
SEASONS = {"build", "maintain", "recover", "travel", "crisis"}
TONES = {"gentle", "direct", "stoic", "strategic", "minimal"}
SCHEDULES = {"daily", "weekdays", "weekly_target", "interval", "event", "opportunity"}
PROVENANCE = {"explicit", "observed", "inferred", "proposed"}

BUILD_OUTCOMES = {"done", "minimum", "partial", "missed", "blocked", "excused", "unknown"}
REDUCE_OUTCOMES = {
    "abstained",
    "urge",
    "resisted",
    "substituted",
    "interrupted",
    "lapse",
    "no_exposure",
    "blocked",
    "excused",
    "unknown",
}
ALL_OUTCOMES = BUILD_OUTCOMES | REDUCE_OUTCOMES

BUILD_WEIGHTS = {"done": 1.0, "minimum": 0.7, "partial": 0.4, "missed": 0.0, "blocked": 0.0}
REDUCE_WEIGHTS = {
    "abstained": 1.0,
    "resisted": 1.0,
    "substituted": 0.85,
    "interrupted": 0.5,
    "urge": 0.0,
    "lapse": 0.0,
    "blocked": 0.0,
}


class HabitOSError(RuntimeError):
    """User-facing deterministic engine error."""


def emit(payload: Any) -> None:
    print(json.dumps(payload, ensure_ascii=False, indent=2, sort_keys=True))


def stable_id(prefix: str) -> str:
    return f"{prefix}-{uuid.uuid4().hex[:20].upper()}"


def validate_timezone(value: str) -> ZoneInfo:
    try:
        return ZoneInfo(value)
    except ZoneInfoNotFoundError as exc:
        raise HabitOSError(f"Unknown IANA timezone: {value}") from exc


def local_now(timezone: str) -> datetime:
    return datetime.now(validate_timezone(timezone)).replace(microsecond=0)


def parse_date(value: str) -> date:
    try:
        return date.fromisoformat(value)
    except ValueError as exc:
        raise HabitOSError(f"Expected date in YYYY-MM-DD format: {value}") from exc


def parse_datetime(value: str, timezone: str) -> datetime:
    try:
        parsed = datetime.fromisoformat(value)
    except ValueError as exc:
        raise HabitOSError(f"Expected RFC 3339 date-time: {value}") from exc
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=validate_timezone(timezone))
    return parsed.replace(microsecond=0)


def json_load(value: str | None, fallback: Any) -> Any:
    if value is None:
        return fallback
    return json.loads(value)


def json_dump(value: Any) -> str:
    return json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True)


def connect(path: str | Path) -> sqlite3.Connection:
    db_path = Path(path).expanduser().resolve()
    db_path.parent.mkdir(parents=True, exist_ok=True)
    connection = sqlite3.connect(db_path)
    connection.row_factory = sqlite3.Row
    connection.execute("PRAGMA foreign_keys = ON")
    connection.execute("PRAGMA journal_mode = WAL")
    install_schema(connection)
    return connection


def install_schema(connection: sqlite3.Connection) -> None:
    connection.executescript(
        """
        CREATE TABLE IF NOT EXISTS metadata (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS profiles (
            user_id TEXT PRIMARY KEY,
            preferred_name TEXT,
            timezone TEXT NOT NULL,
            language TEXT NOT NULL,
            coaching_tone TEXT NOT NULL,
            reflection_depth TEXT NOT NULL DEFAULT 'normal',
            notification_pressure TEXT NOT NULL DEFAULT 'low',
            week_start INTEGER NOT NULL DEFAULT 1,
            privacy_exclusions_json TEXT NOT NULL DEFAULT '[]',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS seasons (
            season_id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            kind TEXT NOT NULL,
            reason TEXT,
            started_at TEXT NOT NULL,
            review_at TEXT,
            ended_at TEXT,
            provenance_json TEXT NOT NULL,
            FOREIGN KEY(user_id) REFERENCES profiles(user_id)
        );

        CREATE TABLE IF NOT EXISTS habits (
            habit_id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            version INTEGER NOT NULL,
            name TEXT NOT NULL,
            kind TEXT NOT NULL,
            status TEXT NOT NULL,
            behavior_definition TEXT NOT NULL,
            why TEXT NOT NULL,
            goal_ids_json TEXT NOT NULL DEFAULT '[]',
            schedule_json TEXT NOT NULL,
            cue_json TEXT NOT NULL,
            target_json TEXT NOT NULL,
            minimum_json TEXT NOT NULL,
            deep_json TEXT,
            evidence_rule_json TEXT NOT NULL,
            fallback_plan TEXT NOT NULL,
            replacement_response TEXT,
            priority INTEGER NOT NULL DEFAULT 50,
            sensitivity TEXT NOT NULL DEFAULT 'normal',
            review_at TEXT NOT NULL,
            provenance_json TEXT NOT NULL,
            supersedes_version INTEGER,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY(user_id) REFERENCES profiles(user_id)
        );

        CREATE UNIQUE INDEX IF NOT EXISTS habits_user_name_active
        ON habits(user_id, name COLLATE NOCASE)
        WHERE status NOT IN ('retired', 'archived');

        CREATE TABLE IF NOT EXISTS habit_versions (
            habit_id TEXT NOT NULL,
            version INTEGER NOT NULL,
            snapshot_json TEXT NOT NULL,
            superseded_at TEXT NOT NULL,
            superseded_reason TEXT NOT NULL,
            PRIMARY KEY(habit_id, version)
        );

        CREATE TABLE IF NOT EXISTS habit_logs (
            log_id TEXT PRIMARY KEY,
            habit_id TEXT NOT NULL,
            occurred_at TEXT NOT NULL,
            local_date TEXT NOT NULL,
            outcome TEXT NOT NULL,
            value REAL,
            unit TEXT,
            context_json TEXT NOT NULL DEFAULT '{}',
            note TEXT,
            provenance_json TEXT NOT NULL,
            sensitivity TEXT NOT NULL DEFAULT 'normal',
            supersedes_log_id TEXT,
            idempotency_key TEXT UNIQUE,
            created_at TEXT NOT NULL,
            FOREIGN KEY(habit_id) REFERENCES habits(habit_id),
            FOREIGN KEY(supersedes_log_id) REFERENCES habit_logs(log_id)
        );

        CREATE INDEX IF NOT EXISTS habit_logs_habit_date
        ON habit_logs(habit_id, local_date, occurred_at);

        CREATE TABLE IF NOT EXISTS experiments (
            experiment_id TEXT PRIMARY KEY,
            habit_id TEXT NOT NULL,
            hypothesis TEXT NOT NULL,
            primary_change TEXT NOT NULL,
            start_date TEXT NOT NULL,
            end_date TEXT NOT NULL,
            evidence TEXT NOT NULL,
            success_threshold TEXT NOT NULL,
            stop_condition TEXT NOT NULL,
            rollback TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY(habit_id) REFERENCES habits(habit_id)
        );

        CREATE TABLE IF NOT EXISTS reviews (
            review_id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            window_start TEXT NOT NULL,
            window_end TEXT NOT NULL,
            timezone TEXT NOT NULL,
            evidence_completeness REAL NOT NULL,
            metrics_json TEXT NOT NULL,
            decisions_json TEXT NOT NULL DEFAULT '[]',
            source_log_ids_json TEXT NOT NULL DEFAULT '[]',
            invalidated_at TEXT,
            created_at TEXT NOT NULL,
            FOREIGN KEY(user_id) REFERENCES profiles(user_id)
        );
        """
    )
    connection.execute(
        "INSERT OR REPLACE INTO metadata(key, value) VALUES('schema_version', ?)",
        (SCHEMA_VERSION,),
    )
    connection.commit()


def get_profile(connection: sqlite3.Connection, user_id: str) -> sqlite3.Row:
    row = connection.execute("SELECT * FROM profiles WHERE user_id = ?", (user_id,)).fetchone()
    if row is None:
        raise HabitOSError(f"Unknown user '{user_id}'. Run init first.")
    return row


def get_current_season(connection: sqlite3.Connection, user_id: str) -> sqlite3.Row | None:
    return connection.execute(
        """SELECT * FROM seasons
           WHERE user_id = ? AND ended_at IS NULL
           ORDER BY started_at DESC LIMIT 1""",
        (user_id,),
    ).fetchone()


def row_to_habit(row: sqlite3.Row) -> dict[str, Any]:
    return {
        "habit_id": row["habit_id"],
        "user_id": row["user_id"],
        "version": row["version"],
        "name": row["name"],
        "kind": row["kind"],
        "status": row["status"],
        "behavior_definition": row["behavior_definition"],
        "why": row["why"],
        "goal_ids": json_load(row["goal_ids_json"], []),
        "schedule": json_load(row["schedule_json"], {}),
        "cue": json_load(row["cue_json"], {}),
        "target": json_load(row["target_json"], {}),
        "minimum": json_load(row["minimum_json"], {}),
        "deep": json_load(row["deep_json"], None),
        "evidence_rule": json_load(row["evidence_rule_json"], {}),
        "fallback_plan": row["fallback_plan"],
        "replacement_response": row["replacement_response"],
        "priority": row["priority"],
        "sensitivity": row["sensitivity"],
        "review_at": row["review_at"],
        "provenance": json_load(row["provenance_json"], {}),
        "supersedes_version": row["supersedes_version"],
        "created_at": row["created_at"],
        "updated_at": row["updated_at"],
    }


def row_to_log(row: sqlite3.Row) -> dict[str, Any]:
    return {
        "log_id": row["log_id"],
        "habit_id": row["habit_id"],
        "occurred_at": row["occurred_at"],
        "local_date": row["local_date"],
        "outcome": row["outcome"],
        "value": row["value"],
        "unit": row["unit"],
        "context": json_load(row["context_json"], {}),
        "note": row["note"],
        "provenance": json_load(row["provenance_json"], {}),
        "sensitivity": row["sensitivity"],
        "supersedes_log_id": row["supersedes_log_id"],
        "created_at": row["created_at"],
    }


def effective_logs(
    connection: sqlite3.Connection,
    habit_id: str,
    start: date | None = None,
    end: date | None = None,
) -> list[dict[str, Any]]:
    clauses = ["l.habit_id = ?", "newer.log_id IS NULL"]
    params: list[Any] = [habit_id]
    if start:
        clauses.append("l.local_date >= ?")
        params.append(start.isoformat())
    if end:
        clauses.append("l.local_date <= ?")
        params.append(end.isoformat())
    rows = connection.execute(
        f"""SELECT l.* FROM habit_logs l
            LEFT JOIN habit_logs newer ON newer.supersedes_log_id = l.log_id
            WHERE {' AND '.join(clauses)}
            ORDER BY l.occurred_at, l.created_at""",
        params,
    ).fetchall()
    return [row_to_log(row) for row in rows]


def resolve_habit(connection: sqlite3.Connection, user_id: str, reference: str) -> sqlite3.Row:
    exact = connection.execute(
        "SELECT * FROM habits WHERE user_id = ? AND habit_id = ?",
        (user_id, reference),
    ).fetchone()
    if exact:
        return exact
    exact_name = connection.execute(
        "SELECT * FROM habits WHERE user_id = ? AND name = ? COLLATE NOCASE",
        (user_id, reference),
    ).fetchall()
    if len(exact_name) == 1:
        return exact_name[0]
    candidates = connection.execute(
        "SELECT * FROM habits WHERE user_id = ? AND lower(name) LIKE ?",
        (user_id, f"%{reference.lower()}%"),
    ).fetchall()
    if not candidates:
        raise HabitOSError(f"No habit matches '{reference}'.")
    if len(candidates) > 1:
        names = ", ".join(row["name"] for row in candidates)
        raise HabitOSError(f"Ambiguous habit reference '{reference}': {names}")
    return candidates[0]


def daterange(start: date, end: date) -> Iterable[date]:
    current = start
    while current <= end:
        yield current
        current += timedelta(days=1)


def due_on(schedule: dict[str, Any], day: date) -> bool:
    mode = schedule.get("mode", "daily")
    if mode == "daily":
        return True
    if mode == "weekdays":
        return day.isoweekday() in set(schedule.get("weekdays", []))
    if mode == "interval":
        anchor_raw = schedule.get("anchor_date")
        interval = int(schedule.get("interval_days") or 1)
        if not anchor_raw:
            return False
        delta = (day - parse_date(anchor_raw)).days
        return delta >= 0 and delta % interval == 0
    return False


def schedule_denominator(schedule: dict[str, Any], start: date, end: date, logs: Sequence[dict[str, Any]]) -> int:
    mode = schedule.get("mode", "daily")
    if mode in {"daily", "weekdays", "interval"}:
        return sum(1 for day in daterange(start, end) if due_on(schedule, day))
    if mode == "weekly_target":
        target = int(schedule.get("target_per_week") or 1)
        weeks = {(day.isocalendar().year, day.isocalendar().week) for day in daterange(start, end)}
        return target * len(weeks)
    if mode in {"event", "opportunity"}:
        return len(
            {
                log["log_id"]
                for log in logs
                if log["outcome"] not in {"unknown", "excused", "no_exposure"}
            }
        )
    return 0


def validate_habit_contract(args: argparse.Namespace) -> None:
    if args.kind not in KINDS:
        raise HabitOSError(f"Unknown habit kind: {args.kind}")
    if args.status not in STATUSES:
        raise HabitOSError(f"Unknown habit status: {args.status}")
    if args.schedule not in SCHEDULES:
        raise HabitOSError(f"Unknown schedule mode: {args.schedule}")
    if not 0 <= args.priority <= 100:
        raise HabitOSError("Priority must be between 0 and 100.")
    if args.kind in {"reduce", "stop"} and not args.replacement:
        raise HabitOSError("Reduce/stop habits require --replacement.")
    if args.schedule == "weekdays" and not args.weekdays:
        raise HabitOSError("Weekday schedules require --weekdays, e.g. 1,2,3,4,5.")
    if args.schedule == "weekly_target" and not args.target_per_week:
        raise HabitOSError("Weekly-target schedules require --target-per-week.")
    if args.schedule == "interval" and (not args.interval_days or not args.anchor_date):
        raise HabitOSError("Interval schedules require --interval-days and --anchor-date.")


def command_init(args: argparse.Namespace) -> None:
    if args.tone not in TONES:
        raise HabitOSError(f"Unknown tone: {args.tone}")
    if args.season not in SEASONS:
        raise HabitOSError(f"Unknown season: {args.season}")
    validate_timezone(args.timezone)
    connection = connect(args.db)
    now = local_now(args.timezone).isoformat()
    connection.execute(
        """INSERT INTO profiles(
               user_id, preferred_name, timezone, language, coaching_tone,
               reflection_depth, notification_pressure, week_start,
               privacy_exclusions_json, created_at, updated_at
           ) VALUES(?, ?, ?, ?, ?, ?, ?, ?, '[]', ?, ?)
           ON CONFLICT(user_id) DO UPDATE SET
               preferred_name=excluded.preferred_name,
               timezone=excluded.timezone,
               language=excluded.language,
               coaching_tone=excluded.coaching_tone,
               reflection_depth=excluded.reflection_depth,
               notification_pressure=excluded.notification_pressure,
               week_start=excluded.week_start,
               updated_at=excluded.updated_at""",
        (
            args.user,
            args.name,
            args.timezone,
            args.language,
            args.tone,
            args.reflection_depth,
            args.notification_pressure,
            args.week_start,
            now,
            now,
        ),
    )
    current = get_current_season(connection, args.user)
    if current is None or current["kind"] != args.season:
        if current is not None:
            connection.execute("UPDATE seasons SET ended_at = ? WHERE season_id = ?", (now, current["season_id"]))
        season_id = stable_id("SEASON")
        connection.execute(
            """INSERT INTO seasons(
                   season_id, user_id, kind, reason, started_at, provenance_json
               ) VALUES(?, ?, ?, ?, ?, ?)""",
            (
                season_id,
                args.user,
                args.season,
                args.season_reason,
                now,
                json_dump({"type": "explicit", "source": "cli", "confidence": 1.0}),
            ),
        )
    connection.commit()
    emit(
        {
            "status": "initialized",
            "schema_version": SCHEMA_VERSION,
            "user_id": args.user,
            "timezone": args.timezone,
            "season": args.season,
            "db": str(Path(args.db).expanduser().resolve()),
        }
    )


def command_add(args: argparse.Namespace) -> None:
    validate_habit_contract(args)
    connection = connect(args.db)
    profile = get_profile(connection, args.user)
    timezone = profile["timezone"]
    now_dt = local_now(timezone)
    weekdays = []
    if args.weekdays:
        try:
            weekdays = sorted({int(value.strip()) for value in args.weekdays.split(",") if value.strip()})
        except ValueError as exc:
            raise HabitOSError("--weekdays must be comma-separated ISO weekdays 1-7.") from exc
        if any(value < 1 or value > 7 for value in weekdays):
            raise HabitOSError("ISO weekdays must be between 1 and 7.")
    schedule = {
        "mode": args.schedule,
        "timezone": timezone,
        "weekdays": weekdays,
        "target_per_week": args.target_per_week,
        "interval_days": args.interval_days,
        "anchor_date": args.anchor_date,
        "event_definition": args.event_definition,
        "local_times": args.at or [],
    }
    cue = {"type": args.cue_type, "description": args.cue, "context": args.context}
    target = {"description": args.target, "value": args.target_value, "unit": args.unit}
    minimum = {"description": args.minimum, "value": args.minimum_value, "unit": args.unit}
    deep = None
    if args.deep:
        deep = {"description": args.deep, "value": args.deep_value, "unit": args.unit}
    review_at = args.review_at or (now_dt + timedelta(days=7)).isoformat()
    review_at = parse_datetime(review_at, timezone).isoformat()
    habit_id = stable_id("HAB")
    provenance = {"type": "explicit", "source": "cli", "confidence": 1.0}
    connection.execute(
        """INSERT INTO habits(
               habit_id, user_id, version, name, kind, status,
               behavior_definition, why, goal_ids_json, schedule_json, cue_json,
               target_json, minimum_json, deep_json, evidence_rule_json,
               fallback_plan, replacement_response, priority, sensitivity,
               review_at, provenance_json, created_at, updated_at
           ) VALUES(?, ?, 1, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
        (
            habit_id,
            args.user,
            args.name,
            args.kind,
            args.status,
            args.behavior,
            args.why,
            json_dump(args.goal_id or []),
            json_dump(schedule),
            json_dump(cue),
            json_dump(target),
            json_dump(minimum),
            json_dump(deep) if deep else None,
            json_dump({"description": args.evidence, "allowed_sources": ["explicit", "observed"]}),
            args.fallback,
            args.replacement,
            args.priority,
            args.sensitivity,
            review_at,
            json_dump(provenance),
            now_dt.isoformat(),
            now_dt.isoformat(),
        ),
    )
    connection.commit()
    row = connection.execute("SELECT * FROM habits WHERE habit_id = ?", (habit_id,)).fetchone()
    emit({"status": "created", "habit": row_to_habit(row)})


def command_list(args: argparse.Namespace) -> None:
    connection = connect(args.db)
    get_profile(connection, args.user)
    clauses = ["user_id = ?"]
    params: list[Any] = [args.user]
    if args.status:
        clauses.append("status = ?")
        params.append(args.status)
    rows = connection.execute(
        f"SELECT * FROM habits WHERE {' AND '.join(clauses)} ORDER BY priority DESC, name",
        params,
    ).fetchall()
    emit({"count": len(rows), "habits": [row_to_habit(row) for row in rows]})


def command_update(args: argparse.Namespace) -> None:
    connection = connect(args.db)
    profile = get_profile(connection, args.user)
    row = resolve_habit(connection, args.user, args.habit)
    habit = row_to_habit(row)
    if habit["version"] != args.expected_version:
        raise HabitOSError(
            f"Version conflict: expected {args.expected_version}, current is {habit['version']}."
        )
    updates: dict[str, Any] = {}
    if args.status is not None:
        updates["status"] = args.status
    if args.priority is not None:
        if not 0 <= args.priority <= 100:
            raise HabitOSError("Priority must be between 0 and 100.")
        updates["priority"] = args.priority
    if args.why is not None:
        updates["why"] = args.why
    if args.cue is not None:
        cue = habit["cue"]
        cue["description"] = args.cue
        updates["cue_json"] = json_dump(cue)
    if args.target is not None:
        target = habit["target"]
        target["description"] = args.target
        if args.target_value is not None:
            target["value"] = args.target_value
        updates["target_json"] = json_dump(target)
    if args.minimum is not None:
        minimum = habit["minimum"]
        minimum["description"] = args.minimum
        if args.minimum_value is not None:
            minimum["value"] = args.minimum_value
        updates["minimum_json"] = json_dump(minimum)
    if args.deep is not None:
        deep = habit["deep"] or {"description": args.deep, "value": None, "unit": habit["target"].get("unit")}
        deep["description"] = args.deep
        if args.deep_value is not None:
            deep["value"] = args.deep_value
        updates["deep_json"] = json_dump(deep)
    if args.fallback is not None:
        updates["fallback_plan"] = args.fallback
    if args.replacement is not None:
        updates["replacement_response"] = args.replacement
    if args.review_at is not None:
        updates["review_at"] = parse_datetime(args.review_at, profile["timezone"]).isoformat()
    if not updates:
        raise HabitOSError("No contract changes supplied.")
    if habit["kind"] in {"reduce", "stop"}:
        resulting_replacement = updates.get("replacement_response", habit["replacement_response"])
        if not resulting_replacement:
            raise HabitOSError("Reduce/stop habits require a replacement response.")
    now = local_now(profile["timezone"]).isoformat()
    connection.execute(
        """INSERT INTO habit_versions(habit_id, version, snapshot_json, superseded_at, superseded_reason)
           VALUES(?, ?, ?, ?, ?)""",
        (habit["habit_id"], habit["version"], json_dump(habit), now, args.reason),
    )
    updates["version"] = habit["version"] + 1
    updates["supersedes_version"] = habit["version"]
    updates["updated_at"] = now
    assignments = ", ".join(f"{column} = ?" for column in updates)
    connection.execute(
        f"UPDATE habits SET {assignments} WHERE habit_id = ? AND version = ?",
        [*updates.values(), habit["habit_id"], habit["version"]],
    )
    connection.commit()
    updated = connection.execute("SELECT * FROM habits WHERE habit_id = ?", (habit["habit_id"],)).fetchone()
    emit({"status": "updated", "reason": args.reason, "habit": row_to_habit(updated)})


def allowed_outcomes(kind: str) -> set[str]:
    return REDUCE_OUTCOMES if kind in {"reduce", "stop"} else BUILD_OUTCOMES


def command_log(args: argparse.Namespace) -> None:
    connection = connect(args.db)
    profile = get_profile(connection, args.user)
    habit_row = resolve_habit(connection, args.user, args.habit)
    if args.outcome not in allowed_outcomes(habit_row["kind"]):
        allowed = ", ".join(sorted(allowed_outcomes(habit_row["kind"])))
        raise HabitOSError(f"Outcome '{args.outcome}' is invalid for {habit_row['kind']}. Allowed: {allowed}")
    if args.source not in {"explicit", "observed"}:
        raise HabitOSError("Only explicit or trusted observed sources may be logged as evidence.")
    if args.energy is not None and not 1 <= args.energy <= 5:
        raise HabitOSError("Energy must be 1-5.")
    if args.mood is not None and not 1 <= args.mood <= 5:
        raise HabitOSError("Mood must be 1-5.")
    if args.urge is not None and not 0 <= args.urge <= 10:
        raise HabitOSError("Urge must be 0-10.")
    timezone = profile["timezone"]
    if args.occurred_at:
        occurred = parse_datetime(args.occurred_at, timezone)
    elif args.date:
        local_day = parse_date(args.date)
        occurred = datetime.combine(local_day, time(12, 0), tzinfo=validate_timezone(timezone))
    else:
        occurred = local_now(timezone)
    local_day = occurred.astimezone(validate_timezone(timezone)).date()
    if args.idempotency_key:
        existing = connection.execute(
            "SELECT * FROM habit_logs WHERE idempotency_key = ?",
            (args.idempotency_key,),
        ).fetchone()
        if existing:
            emit({"status": "already_recorded", "log": row_to_log(existing)})
            return
    context = {
        "cue_observed": args.cue_observed,
        "energy": args.energy,
        "mood": args.mood,
        "urge": args.urge,
        "barrier_code": args.barrier,
        "trigger": args.trigger,
    }
    log_id = stable_id("LOG")
    created_at = local_now(timezone).isoformat()
    provenance = {"type": args.source, "source": args.source_ref or "conversation", "confidence": 1.0}
    connection.execute(
        """INSERT INTO habit_logs(
               log_id, habit_id, occurred_at, local_date, outcome, value, unit,
               context_json, note, provenance_json, sensitivity,
               idempotency_key, created_at
           ) VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
        (
            log_id,
            habit_row["habit_id"],
            occurred.isoformat(),
            local_day.isoformat(),
            args.outcome,
            args.value,
            args.unit,
            json_dump(context),
            args.note,
            json_dump(provenance),
            args.sensitivity or habit_row["sensitivity"],
            args.idempotency_key,
            created_at,
        ),
    )
    connection.commit()
    row = connection.execute("SELECT * FROM habit_logs WHERE log_id = ?", (log_id,)).fetchone()
    emit(
        {
            "status": "recorded",
            "habit": {"habit_id": habit_row["habit_id"], "name": habit_row["name"]},
            "log": row_to_log(row),
        }
    )


def command_correct(args: argparse.Namespace) -> None:
    connection = connect(args.db)
    profile = get_profile(connection, args.user)
    row = connection.execute(
        """SELECT l.*, h.kind, h.sensitivity AS habit_sensitivity
           FROM habit_logs l JOIN habits h ON h.habit_id = l.habit_id
           WHERE l.log_id = ? AND h.user_id = ?""",
        (args.log_id, args.user),
    ).fetchone()
    if row is None:
        raise HabitOSError(f"Unknown log '{args.log_id}' for this user.")
    if connection.execute("SELECT 1 FROM habit_logs WHERE supersedes_log_id = ?", (args.log_id,)).fetchone():
        raise HabitOSError("This log was already superseded; correct the latest log in the chain.")
    old = row_to_log(row)
    if args.energy is not None and not 1 <= args.energy <= 5:
        raise HabitOSError("Energy must be 1-5.")
    if args.mood is not None and not 1 <= args.mood <= 5:
        raise HabitOSError("Mood must be 1-5.")
    if args.urge is not None and not 0 <= args.urge <= 10:
        raise HabitOSError("Urge must be 0-10.")
    outcome = args.outcome or old["outcome"]
    if outcome not in allowed_outcomes(row["kind"]):
        raise HabitOSError(f"Outcome '{outcome}' is invalid for habit kind {row['kind']}.")
    timezone = profile["timezone"]
    occurred = parse_datetime(args.occurred_at, timezone) if args.occurred_at else parse_datetime(old["occurred_at"], timezone)
    context = dict(old["context"])
    for key, value in {
        "energy": args.energy,
        "mood": args.mood,
        "urge": args.urge,
        "barrier_code": args.barrier,
        "trigger": args.trigger,
    }.items():
        if value is not None:
            context[key] = value
    new_log_id = stable_id("LOG")
    created_at = local_now(timezone).isoformat()
    provenance = {
        "type": "explicit",
        "source": "user_correction",
        "source_ref": args.log_id,
        "confidence": 1.0,
    }
    connection.execute(
        """INSERT INTO habit_logs(
               log_id, habit_id, occurred_at, local_date, outcome, value, unit,
               context_json, note, provenance_json, sensitivity,
               supersedes_log_id, idempotency_key, created_at
           ) VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
        (
            new_log_id,
            old["habit_id"],
            occurred.isoformat(),
            occurred.astimezone(validate_timezone(timezone)).date().isoformat(),
            outcome,
            args.value if args.value is not None else old["value"],
            args.unit if args.unit is not None else old["unit"],
            json_dump(context),
            args.note if args.note is not None else old["note"],
            json_dump(provenance),
            args.sensitivity or old["sensitivity"],
            args.log_id,
            args.idempotency_key,
            created_at,
        ),
    )
    review_rows = connection.execute(
        "SELECT review_id, source_log_ids_json FROM reviews WHERE user_id = ? AND invalidated_at IS NULL",
        (args.user,),
    ).fetchall()
    invalidated = []
    for review in review_rows:
        if args.log_id in json_load(review["source_log_ids_json"], []):
            connection.execute("UPDATE reviews SET invalidated_at = ? WHERE review_id = ?", (created_at, review["review_id"]))
            invalidated.append(review["review_id"])
    connection.commit()
    corrected = connection.execute("SELECT * FROM habit_logs WHERE log_id = ?", (new_log_id,)).fetchone()
    emit(
        {
            "status": "corrected",
            "superseded_log_id": args.log_id,
            "log": row_to_log(corrected),
            "invalidated_review_ids": invalidated,
        }
    )


def most_recent_outcome(connection: sqlite3.Connection, habit_id: str, on_or_before: date) -> str | None:
    row = connection.execute(
        """SELECT l.outcome FROM habit_logs l
           LEFT JOIN habit_logs newer ON newer.supersedes_log_id = l.log_id
           WHERE l.habit_id = ? AND l.local_date <= ? AND newer.log_id IS NULL
           ORDER BY l.occurred_at DESC LIMIT 1""",
        (habit_id, on_or_before.isoformat()),
    ).fetchone()
    return row["outcome"] if row else None


def command_today(args: argparse.Namespace) -> None:
    connection = connect(args.db)
    profile = get_profile(connection, args.user)
    timezone = profile["timezone"]
    today = parse_date(args.date) if args.date else local_now(timezone).date()
    season_row = get_current_season(connection, args.user)
    season = season_row["kind"] if season_row else "build"
    hard_limit = min(args.limit, 5 if season in {"recover", "crisis"} else 7)
    rows = connection.execute(
        "SELECT * FROM habits WHERE user_id = ? AND status IN ('active', 'recovering')",
        (args.user,),
    ).fetchall()
    candidates: list[tuple[float, dict[str, Any], list[str]]] = []
    for row in rows:
        habit = row_to_habit(row)
        schedule = habit["schedule"]
        mode = schedule.get("mode", "daily")
        if mode in {"daily", "weekdays", "interval"} and not due_on(schedule, today):
            continue
        if mode == "weekly_target":
            start = today - timedelta(days=today.isoweekday() - 1)
            logs = effective_logs(connection, row["habit_id"], start, today)
            success_set = {"done", "minimum"} if row["kind"] in {"build", "maintain"} else {"abstained", "resisted", "substituted"}
            completed = sum(1 for log in logs if log["outcome"] in success_set)
            if completed >= int(schedule.get("target_per_week") or 1):
                continue
        if mode in {"event", "opportunity"}:
            continue
        score = float(row["priority"])
        reasons = [f"priority {row['priority']}"]
        last = most_recent_outcome(connection, row["habit_id"], today)
        if last in {"missed", "lapse", "blocked"}:
            score += 15
            reasons.append("protect recovery after disruption")
        review_at = parse_datetime(row["review_at"], timezone)
        if review_at.date() <= today:
            score += 5
            reasons.append("review due")
        if row["status"] == "recovering" or season == "recover":
            score += 5 if row["priority"] >= 80 else -20
            reasons.append("recovery load policy")
        candidates.append((score, habit, reasons))
    candidates.sort(key=lambda item: (-item[0], item[1]["name"].lower()))
    selected = candidates[:hard_limit]
    items = []
    for rank, (score, habit, reasons) in enumerate(selected, start=1):
        items.append(
            {
                "rank": rank,
                "habit_id": habit["habit_id"],
                "name": habit["name"],
                "cue": habit["cue"],
                "target": habit["target"],
                "minimum": habit["minimum"],
                "deep": habit["deep"],
                "why": habit["why"],
                "ranking_score": score,
                "ranking_reasons": reasons,
            }
        )
    emit(
        {
            "local_date": today.isoformat(),
            "timezone": timezone,
            "season": season,
            "primary_count": len(items),
            "deferred_count": max(0, len(candidates) - len(items)),
            "primary_items": items,
        }
    )


def choose_daily_outcome(kind: str, outcomes: Sequence[str]) -> str | None:
    if not outcomes:
        return None
    if kind in {"build", "maintain"}:
        order = ["done", "minimum", "partial", "missed", "blocked", "excused", "unknown"]
        for candidate in order:
            if candidate in outcomes:
                return candidate
    else:
        # A lapse dominates a same-day success because opportunity-level detail matters.
        order = ["lapse", "interrupted", "substituted", "resisted", "abstained", "urge", "blocked", "no_exposure", "excused", "unknown"]
        for candidate in order:
            if candidate in outcomes:
                return candidate
    return outcomes[-1]


def recovery_latencies(kind: str, logs: Sequence[dict[str, Any]]) -> list[int]:
    failures = {"missed"} if kind in {"build", "maintain"} else {"lapse"}
    successes = {"done", "minimum"} if kind in {"build", "maintain"} else {"abstained", "resisted", "substituted", "interrupted"}
    dated = [(parse_date(log["local_date"]), log["outcome"]) for log in logs]
    result: list[int] = []
    for index, (failed_on, outcome) in enumerate(dated):
        if outcome not in failures:
            continue
        for recovered_on, later_outcome in dated[index + 1 :]:
            if later_outcome in successes and recovered_on >= failed_on:
                result.append((recovered_on - failed_on).days)
                break
    return result


@dataclass
class HabitMetrics:
    habit_id: str
    name: str
    kind: str
    window_start: str
    window_end: str
    scheduled_opportunities: int
    known_opportunities: int
    unknown_opportunities: int
    data_completeness: float | None
    target_rate: float | None
    minimum_or_better_rate: float | None
    continuity_indicator: float | None
    outcome_counts: dict[str, int]
    recovery_latency_median_days: float | None
    recovery_latency_range_days: list[int] | None
    top_barrier: str | None
    top_barrier_count: int
    pattern_label: str | None
    source_log_ids: list[str]


def calculate_metrics(
    connection: sqlite3.Connection,
    habit_row: sqlite3.Row,
    start: date,
    end: date,
) -> HabitMetrics:
    habit = row_to_habit(habit_row)
    logs = effective_logs(connection, habit["habit_id"], start, end)
    schedule = habit["schedule"]
    denominator = schedule_denominator(schedule, start, end, logs)
    mode = schedule.get("mode", "daily")
    outcome_counts: Counter[str] = Counter()
    known = 0
    if mode in {"daily", "weekdays", "interval"}:
        by_date: dict[str, list[str]] = {}
        for log in logs:
            by_date.setdefault(log["local_date"], []).append(log["outcome"])
        for day in daterange(start, end):
            if not due_on(schedule, day):
                continue
            outcome = choose_daily_outcome(habit["kind"], by_date.get(day.isoformat(), []))
            if outcome and outcome not in {"unknown", "excused"}:
                known += 1
                outcome_counts[outcome] += 1
            elif outcome == "excused":
                denominator = max(0, denominator - 1)
                outcome_counts[outcome] += 1
            elif outcome == "unknown":
                outcome_counts[outcome] += 1
    else:
        relevant = [log for log in logs if log["outcome"] not in {"unknown", "excused", "no_exposure"}]
        known = min(denominator, len(relevant)) if denominator else len(relevant)
        for log in logs:
            outcome_counts[log["outcome"]] += 1
    unknown = max(0, denominator - known)
    completeness = round(known / denominator, 4) if denominator else None
    weights = BUILD_WEIGHTS if habit["kind"] in {"build", "maintain"} else REDUCE_WEIGHTS
    weighted = sum(weights.get(outcome, 0.0) * count for outcome, count in outcome_counts.items())
    continuity = round(weighted / denominator, 4) if denominator else None
    if habit["kind"] in {"build", "maintain"}:
        target_count = outcome_counts["done"]
        minimum_count = target_count + outcome_counts["minimum"]
    else:
        target_count = outcome_counts["abstained"] + outcome_counts["resisted"] + outcome_counts["substituted"]
        minimum_count = target_count + outcome_counts["interrupted"]
    target_rate = round(target_count / denominator, 4) if denominator else None
    minimum_rate = round(minimum_count / denominator, 4) if denominator else None
    latencies = recovery_latencies(habit["kind"], logs)
    barriers = Counter(
        log["context"].get("barrier_code")
        for log in logs
        if log["context"].get("barrier_code")
    )
    top_barrier = barriers.most_common(1)[0] if barriers else (None, 0)
    count = int(top_barrier[1])
    if count >= 6:
        pattern = "stable_pattern"
    elif count >= 3:
        pattern = "probable_pattern"
    elif count == 2:
        pattern = "early_signal"
    elif count == 1:
        pattern = "observation"
    else:
        pattern = None
    return HabitMetrics(
        habit_id=habit["habit_id"],
        name=habit["name"],
        kind=habit["kind"],
        window_start=start.isoformat(),
        window_end=end.isoformat(),
        scheduled_opportunities=denominator,
        known_opportunities=known,
        unknown_opportunities=unknown,
        data_completeness=completeness,
        target_rate=target_rate,
        minimum_or_better_rate=minimum_rate,
        continuity_indicator=continuity,
        outcome_counts=dict(sorted(outcome_counts.items())),
        recovery_latency_median_days=round(float(median(latencies)), 2) if latencies else None,
        recovery_latency_range_days=[min(latencies), max(latencies)] if latencies else None,
        top_barrier=top_barrier[0],
        top_barrier_count=count,
        pattern_label=pattern,
        source_log_ids=[log["log_id"] for log in logs],
    )


def review_payload(
    connection: sqlite3.Connection,
    user_id: str,
    start: date,
    end: date,
    habit_ref: str | None,
) -> dict[str, Any]:
    profile = get_profile(connection, user_id)
    if start > end:
        raise HabitOSError("Review start must not be after end.")
    if habit_ref:
        rows = [resolve_habit(connection, user_id, habit_ref)]
    else:
        rows = connection.execute(
            "SELECT * FROM habits WHERE user_id = ? AND status NOT IN ('archived') ORDER BY priority DESC, name",
            (user_id,),
        ).fetchall()
    metrics = [calculate_metrics(connection, row, start, end) for row in rows]
    denominator = sum(metric.scheduled_opportunities for metric in metrics)
    known = sum(metric.known_opportunities for metric in metrics)
    overall = round(known / denominator, 4) if denominator else None
    return {
        "window": {"start": start.isoformat(), "end": end.isoformat(), "timezone": profile["timezone"]},
        "overall_evidence_completeness": overall,
        "habit_metrics": [asdict(metric) for metric in metrics],
        "interpretation_policy": {
            "one_event": "observation",
            "two_comparable_events": "early_signal",
            "three_or_more": "probable_pattern",
            "causality": "requires_a_testable_experiment",
        },
    }


def command_review(args: argparse.Namespace) -> None:
    connection = connect(args.db)
    profile = get_profile(connection, args.user)
    end = parse_date(args.end) if args.end else local_now(profile["timezone"]).date()
    start = parse_date(args.start) if args.start else end - timedelta(days=args.days - 1)
    payload = review_payload(connection, args.user, start, end, args.habit)
    if args.save:
        review_id = stable_id("REV")
        source_ids = [
            log_id
            for metric in payload["habit_metrics"]
            for log_id in metric["source_log_ids"]
        ]
        now = local_now(profile["timezone"]).isoformat()
        connection.execute(
            """INSERT INTO reviews(
                   review_id, user_id, window_start, window_end, timezone,
                   evidence_completeness, metrics_json, source_log_ids_json, created_at
               ) VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (
                review_id,
                args.user,
                start.isoformat(),
                end.isoformat(),
                profile["timezone"],
                payload["overall_evidence_completeness"] or 0.0,
                json_dump(payload),
                json_dump(source_ids),
                now,
            ),
        )
        connection.commit()
        payload["review_id"] = review_id
        payload["status"] = "saved"
    emit(payload)


def command_chart(args: argparse.Namespace) -> None:
    connection = connect(args.db)
    profile = get_profile(connection, args.user)
    habit_row = resolve_habit(connection, args.user, args.habit)
    end = parse_date(args.end) if args.end else local_now(profile["timezone"]).date()
    start = end - timedelta(days=args.days - 1)
    habit = row_to_habit(habit_row)
    logs = effective_logs(connection, habit["habit_id"], start, end)
    by_date: dict[str, list[str]] = {}
    for log in logs:
        by_date.setdefault(log["local_date"], []).append(log["outcome"])
    weights = BUILD_WEIGHTS if habit["kind"] in {"build", "maintain"} else REDUCE_WEIGHTS
    points: list[tuple[str, int, str]] = []
    for day in daterange(start, end):
        if not due_on(habit["schedule"], day):
            continue
        outcome = choose_daily_outcome(habit["kind"], by_date.get(day.isoformat(), []))
        if outcome in weights:
            points.append((day.strftime("%m-%d"), round(weights[outcome] * 100), outcome))
    if len(points) < 3:
        lines = ["| Date | Outcome | Indicator |", "| --- | --- | ---: |"]
        lines.extend(f"| {day} | {outcome} | {value} |" for day, value, outcome in points)
        print("\n".join(lines) if points else "Insufficient known opportunities for a chart.")
        return
    labels = ", ".join(f'"{point[0]}"' for point in points)
    values = ", ".join(str(point[1]) for point in points)
    print("```mermaid")
    print("xychart-beta")
    print(f'    title "{habit["name"]} — known opportunities"')
    print(f"    x-axis [{labels}]")
    print('    y-axis "Continuity indicator" 0 --> 100')
    print(f"    line [{values}]")
    print("```")
    print(f"Known opportunities plotted: {len(points)}. Unknown opportunities are omitted, never converted to zero.")


def command_context(args: argparse.Namespace) -> None:
    connection = connect(args.db)
    profile = get_profile(connection, args.user)
    season = get_current_season(connection, args.user)
    rows = connection.execute(
        "SELECT * FROM habits WHERE user_id = ? AND status IN ('active', 'recovering', 'paused') ORDER BY priority DESC, name",
        (args.user,),
    ).fetchall()
    cutoff = local_now(profile["timezone"]).date() - timedelta(days=args.days - 1)
    habit_context = []
    for row in rows:
        habit = row_to_habit(row)
        logs = effective_logs(connection, habit["habit_id"], cutoff, None)
        habit_context.append(
            {
                "habit": habit,
                "recent_logs": logs[-args.logs_per_habit :],
                "open_experiments": [
                    dict(item)
                    for item in connection.execute(
                        "SELECT * FROM experiments WHERE habit_id = ? AND status IN ('proposed', 'active')",
                        (habit["habit_id"],),
                    ).fetchall()
                ],
            }
        )
    emit(
        {
            "schema_version": SCHEMA_VERSION,
            "profile": {
                "user_id": profile["user_id"],
                "preferred_name": profile["preferred_name"],
                "timezone": profile["timezone"],
                "language": profile["language"],
                "coaching_tone": profile["coaching_tone"],
                "reflection_depth": profile["reflection_depth"],
                "notification_pressure": profile["notification_pressure"],
            },
            "season": dict(season) if season else None,
            "habits": habit_context,
            "context_window_days": args.days,
        }
    )


def command_export(args: argparse.Namespace) -> None:
    connection = connect(args.db)
    profile = get_profile(connection, args.user)
    habits = [row_to_habit(row) for row in connection.execute("SELECT * FROM habits WHERE user_id = ?", (args.user,)).fetchall()]
    habit_ids = [habit["habit_id"] for habit in habits]
    logs: list[dict[str, Any]] = []
    for habit_id in habit_ids:
        logs.extend(
            row_to_log(row)
            for row in connection.execute(
                "SELECT * FROM habit_logs WHERE habit_id = ? ORDER BY occurred_at, created_at",
                (habit_id,),
            ).fetchall()
        )
    versions = [
        {
            "habit_id": row["habit_id"],
            "version": row["version"],
            "snapshot": json_load(row["snapshot_json"], {}),
            "superseded_at": row["superseded_at"],
            "superseded_reason": row["superseded_reason"],
        }
        for row in connection.execute(
            """SELECT v.* FROM habit_versions v
               JOIN habits h ON h.habit_id = v.habit_id
               WHERE h.user_id = ? ORDER BY v.habit_id, v.version""",
            (args.user,),
        ).fetchall()
    ]
    if args.format == "json":
        current_season = get_current_season(connection, args.user)
        if current_season is None:
            raise HabitOSError("State has no current season.")
        experiments = []
        for row in connection.execute(
            """SELECT e.* FROM experiments e JOIN habits h ON h.habit_id = e.habit_id
               WHERE h.user_id = ? ORDER BY e.start_date, e.experiment_id""",
            (args.user,),
        ).fetchall():
            experiments.append(
                {
                    "experiment_id": row["experiment_id"],
                    "habit_id": row["habit_id"],
                    "hypothesis": row["hypothesis"],
                    "primary_change": row["primary_change"],
                    "start_date": row["start_date"],
                    "end_date": row["end_date"],
                    "evidence": row["evidence"],
                    "success_threshold": row["success_threshold"],
                    "stop_condition": row["stop_condition"],
                    "rollback": row["rollback"],
                    "status": row["status"],
                }
            )
        reviews = []
        for row in connection.execute(
            "SELECT * FROM reviews WHERE user_id = ? AND invalidated_at IS NULL ORDER BY created_at",
            (args.user,),
        ).fetchall():
            stored = json_load(row["metrics_json"], {})
            reviews.append(
                {
                    "review_id": row["review_id"],
                    "window_start": row["window_start"],
                    "window_end": row["window_end"],
                    "timezone": row["timezone"],
                    "evidence_completeness": row["evidence_completeness"],
                    "metrics": stored,
                    "decisions": json_load(row["decisions_json"], []),
                    "source_log_ids": json_load(row["source_log_ids_json"], []),
                    "created_at": row["created_at"],
                }
            )
        exported_habits = []
        for habit in habits:
            item = dict(habit)
            item.pop("user_id", None)
            exported_habits.append(item)
        payload = {
            "schema_version": SCHEMA_VERSION,
            "user": {
                "user_id": profile["user_id"],
                "preferred_name": profile["preferred_name"],
                "timezone": profile["timezone"],
                "language": profile["language"],
                "week_start": profile["week_start"],
                "coaching_tone": profile["coaching_tone"],
                "reflection_depth": profile["reflection_depth"],
                "notification_pressure": profile["notification_pressure"],
                "privacy_exclusions": json_load(profile["privacy_exclusions_json"], []),
            },
            "season": {
                "season_id": current_season["season_id"],
                "kind": current_season["kind"],
                "reason": current_season["reason"],
                "started_at": current_season["started_at"],
                "review_at": current_season["review_at"],
                "provenance": json_load(current_season["provenance_json"], {}),
            },
            "identity_refs": [],
            "goal_refs": [],
            "habits": exported_habits,
            "habit_versions": versions,
            "logs": logs,
            "experiments": experiments,
            "reviews": reviews,
        }
        content = json.dumps(payload, ensure_ascii=False, indent=2, sort_keys=True)
    else:
        buffer = io.StringIO()
        fields = ["log_id", "habit_id", "occurred_at", "local_date", "outcome", "value", "unit", "note", "sensitivity"]
        writer = csv.DictWriter(buffer, fieldnames=fields)
        writer.writeheader()
        for log in logs:
            writer.writerow({field: log.get(field) for field in fields})
        content = buffer.getvalue()
    if args.output:
        output = Path(args.output).expanduser().resolve()
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(content, encoding="utf-8")
        emit({"status": "exported", "format": args.format, "path": str(output), "habit_count": len(habits), "log_count": len(logs)})
    else:
        print(content)


def command_season(args: argparse.Namespace) -> None:
    connection = connect(args.db)
    profile = get_profile(connection, args.user)
    now = local_now(profile["timezone"])
    current = get_current_season(connection, args.user)
    if current and current["kind"] == args.kind:
        emit({"status": "unchanged", "season": dict(current)})
        return
    if current:
        connection.execute("UPDATE seasons SET ended_at = ? WHERE season_id = ?", (now.isoformat(), current["season_id"]))
    review_at = parse_datetime(args.review_at, profile["timezone"]).isoformat() if args.review_at else None
    season_id = stable_id("SEASON")
    connection.execute(
        """INSERT INTO seasons(
               season_id, user_id, kind, reason, started_at, review_at, provenance_json
           ) VALUES(?, ?, ?, ?, ?, ?, ?)""",
        (
            season_id,
            args.user,
            args.kind,
            args.reason,
            now.isoformat(),
            review_at,
            json_dump({"type": "explicit", "source": "conversation", "confidence": 1.0}),
        ),
    )
    connection.commit()
    row = connection.execute("SELECT * FROM seasons WHERE season_id = ?", (season_id,)).fetchone()
    emit({"status": "changed", "season": dict(row)})


def command_experiment(args: argparse.Namespace) -> None:
    connection = connect(args.db)
    profile = get_profile(connection, args.user)
    habit = resolve_habit(connection, args.user, args.habit)
    start = parse_date(args.start)
    end = parse_date(args.end)
    if start > end:
        raise HabitOSError("Experiment start must not be after end.")
    if args.status == "active":
        existing = connection.execute(
            "SELECT experiment_id FROM experiments WHERE habit_id = ? AND status = 'active'",
            (habit["habit_id"],),
        ).fetchone()
        if existing:
            raise HabitOSError(f"Habit already has an active experiment: {existing['experiment_id']}")
    experiment_id = stable_id("EXP")
    now = local_now(profile["timezone"]).isoformat()
    connection.execute(
        """INSERT INTO experiments(
               experiment_id, habit_id, hypothesis, primary_change, start_date,
               end_date, evidence, success_threshold, stop_condition, rollback,
               status, created_at, updated_at
           ) VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
        (
            experiment_id,
            habit["habit_id"],
            args.hypothesis,
            args.change,
            start.isoformat(),
            end.isoformat(),
            args.evidence,
            args.success,
            args.stop,
            args.rollback,
            args.status,
            now,
            now,
        ),
    )
    connection.commit()
    row = connection.execute("SELECT * FROM experiments WHERE experiment_id = ?", (experiment_id,)).fetchone()
    emit({"status": "created", "experiment": dict(row)})


def invalidated_reviews_for_logs(
    connection: sqlite3.Connection,
    user_id: str,
    log_ids: set[str],
    invalidated_at: str,
) -> list[str]:
    invalidated: list[str] = []
    rows = connection.execute(
        "SELECT review_id, source_log_ids_json FROM reviews WHERE user_id = ? AND invalidated_at IS NULL",
        (user_id,),
    ).fetchall()
    for row in rows:
        sources = set(json_load(row["source_log_ids_json"], []))
        if sources & log_ids:
            connection.execute("UPDATE reviews SET invalidated_at = ? WHERE review_id = ?", (invalidated_at, row["review_id"]))
            invalidated.append(row["review_id"])
    return invalidated


def correction_chain(connection: sqlite3.Connection, root_log_id: str) -> list[str]:
    chain = [root_log_id]
    current = root_log_id
    while True:
        row = connection.execute("SELECT log_id FROM habit_logs WHERE supersedes_log_id = ?", (current,)).fetchone()
        if not row:
            break
        current = row["log_id"]
        chain.append(current)
    return chain


def command_delete(args: argparse.Namespace) -> None:
    connection = connect(args.db)
    profile = get_profile(connection, args.user)
    now = local_now(profile["timezone"]).isoformat()
    if args.scope in {"log", "habit"} and not args.target:
        raise HabitOSError("--target is required for log or habit deletion.")
    if args.scope == "log":
        expected = f"DELETE {args.target}"
        if args.confirm != expected:
            raise HabitOSError(f"Confirmation must exactly equal: {expected}")
        owned = connection.execute(
            """SELECT l.log_id FROM habit_logs l JOIN habits h ON h.habit_id = l.habit_id
               WHERE l.log_id = ? AND h.user_id = ?""",
            (args.target, args.user),
        ).fetchone()
        if not owned:
            raise HabitOSError(f"Unknown log '{args.target}' for this user.")
        chain = correction_chain(connection, args.target)
        invalidated = invalidated_reviews_for_logs(connection, args.user, set(chain), now)
        for log_id in reversed(chain):
            connection.execute("DELETE FROM habit_logs WHERE log_id = ?", (log_id,))
        connection.commit()
        emit({"status": "deleted", "scope": "log", "deleted_log_ids": chain, "invalidated_review_ids": invalidated})
        return
    if args.scope == "habit":
        habit = resolve_habit(connection, args.user, args.target)
        expected = f"DELETE {habit['habit_id']}"
        if args.confirm != expected:
            raise HabitOSError(f"Confirmation must exactly equal: {expected}")
        log_ids = {
            row["log_id"]
            for row in connection.execute("SELECT log_id FROM habit_logs WHERE habit_id = ?", (habit["habit_id"],)).fetchall()
        }
        invalidated = invalidated_reviews_for_logs(connection, args.user, log_ids, now)
        connection.execute("DELETE FROM experiments WHERE habit_id = ?", (habit["habit_id"],))
        connection.execute("DELETE FROM habit_logs WHERE habit_id = ?", (habit["habit_id"],))
        connection.execute("DELETE FROM habit_versions WHERE habit_id = ?", (habit["habit_id"],))
        connection.execute("DELETE FROM habits WHERE habit_id = ?", (habit["habit_id"],))
        connection.commit()
        emit({"status": "deleted", "scope": "habit", "habit_id": habit["habit_id"], "deleted_log_count": len(log_ids), "invalidated_review_ids": invalidated})
        return
    expected = f"DELETE ALL {args.user}"
    if args.confirm != expected:
        raise HabitOSError(f"Confirmation must exactly equal: {expected}")
    habit_ids = [
        row["habit_id"]
        for row in connection.execute("SELECT habit_id FROM habits WHERE user_id = ?", (args.user,)).fetchall()
    ]
    for habit_id in habit_ids:
        connection.execute("DELETE FROM experiments WHERE habit_id = ?", (habit_id,))
        connection.execute("DELETE FROM habit_logs WHERE habit_id = ?", (habit_id,))
        connection.execute("DELETE FROM habit_versions WHERE habit_id = ?", (habit_id,))
    connection.execute("DELETE FROM reviews WHERE user_id = ?", (args.user,))
    connection.execute("DELETE FROM habits WHERE user_id = ?", (args.user,))
    connection.execute("DELETE FROM seasons WHERE user_id = ?", (args.user,))
    connection.execute("DELETE FROM profiles WHERE user_id = ?", (args.user,))
    connection.commit()
    emit({"status": "deleted", "scope": "all", "user_id": args.user})


def command_doctor(args: argparse.Namespace) -> None:
    connection = connect(args.db)
    checks: list[dict[str, Any]] = []
    version = connection.execute("SELECT value FROM metadata WHERE key = 'schema_version'").fetchone()
    checks.append({"check": "schema_version", "ok": bool(version and version["value"] == SCHEMA_VERSION), "value": version["value"] if version else None})
    integrity = connection.execute("PRAGMA integrity_check").fetchone()[0]
    checks.append({"check": "sqlite_integrity", "ok": integrity == "ok", "value": integrity})
    foreign_keys = connection.execute("PRAGMA foreign_key_check").fetchall()
    checks.append({"check": "foreign_keys", "ok": not foreign_keys, "violations": len(foreign_keys)})
    malformed: list[str] = []
    json_columns = [
        ("profiles", "privacy_exclusions_json"),
        ("seasons", "provenance_json"),
        ("habits", "schedule_json"),
        ("habits", "cue_json"),
        ("habits", "target_json"),
        ("habits", "minimum_json"),
        ("habit_versions", "snapshot_json"),
        ("habit_logs", "context_json"),
        ("habit_logs", "provenance_json"),
    ]
    for table, column in json_columns:
        rows = connection.execute(f"SELECT rowid, {column} FROM {table}").fetchall()
        for row in rows:
            try:
                json.loads(row[column])
            except (TypeError, json.JSONDecodeError):
                malformed.append(f"{table}:{row['rowid']}:{column}")
    checks.append({"check": "json_fields", "ok": not malformed, "malformed": malformed})
    emit({"ok": all(check["ok"] for check in checks), "checks": checks})


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Habit Tracker {OS} deterministic state engine")
    parser.add_argument("--db", default=DEFAULT_DB, help="SQLite database path")
    subparsers = parser.add_subparsers(dest="command", required=True)

    init = subparsers.add_parser("init", help="Initialize or update a user profile")
    init.add_argument("--user", default="default")
    init.add_argument("--name")
    init.add_argument("--timezone", default="Europe/Madrid")
    init.add_argument("--language", default="fr")
    init.add_argument("--tone", choices=sorted(TONES), default="strategic")
    init.add_argument("--reflection-depth", choices=["micro", "normal", "deep"], default="normal")
    init.add_argument("--notification-pressure", choices=["low", "normal", "high"], default="low")
    init.add_argument("--week-start", type=int, choices=range(1, 8), default=1)
    init.add_argument("--season", choices=sorted(SEASONS), default="build")
    init.add_argument("--season-reason")
    init.set_defaults(func=command_init)

    add = subparsers.add_parser("add", help="Create an accepted habit contract")
    add.add_argument("--user", default="default")
    add.add_argument("--name", required=True)
    add.add_argument("--kind", choices=sorted(KINDS), required=True)
    add.add_argument("--status", choices=sorted(STATUSES), default="active")
    add.add_argument("--behavior", required=True)
    add.add_argument("--why", required=True)
    add.add_argument("--goal-id", action="append")
    add.add_argument("--schedule", choices=sorted(SCHEDULES), default="daily")
    add.add_argument("--weekdays")
    add.add_argument("--target-per-week", type=int)
    add.add_argument("--interval-days", type=int)
    add.add_argument("--anchor-date")
    add.add_argument("--event-definition")
    add.add_argument("--at", action="append", help="Local HH:MM; repeatable")
    add.add_argument("--cue-type", choices=["time", "event", "location", "social", "internal", "opportunity"], default="event")
    add.add_argument("--cue", required=True)
    add.add_argument("--context")
    add.add_argument("--target", required=True)
    add.add_argument("--target-value", type=float)
    add.add_argument("--minimum", required=True)
    add.add_argument("--minimum-value", type=float)
    add.add_argument("--deep")
    add.add_argument("--deep-value", type=float)
    add.add_argument("--unit")
    add.add_argument("--evidence", default="Explicit user report or trusted observation")
    add.add_argument("--fallback", required=True)
    add.add_argument("--replacement")
    add.add_argument("--priority", type=int, default=50)
    add.add_argument("--sensitivity", choices=["normal", "sensitive", "restricted"], default="normal")
    add.add_argument("--review-at")
    add.set_defaults(func=command_add)

    update = subparsers.add_parser("update", help="Create a superseding habit contract version")
    update.add_argument("--user", default="default")
    update.add_argument("--habit", required=True)
    update.add_argument("--expected-version", type=int, required=True)
    update.add_argument("--reason", required=True)
    update.add_argument("--status", choices=sorted(STATUSES))
    update.add_argument("--priority", type=int)
    update.add_argument("--why")
    update.add_argument("--cue")
    update.add_argument("--target")
    update.add_argument("--target-value", type=float)
    update.add_argument("--minimum")
    update.add_argument("--minimum-value", type=float)
    update.add_argument("--deep")
    update.add_argument("--deep-value", type=float)
    update.add_argument("--fallback")
    update.add_argument("--replacement")
    update.add_argument("--review-at")
    update.set_defaults(func=command_update)

    listing = subparsers.add_parser("list", help="List habit contracts")
    listing.add_argument("--user", default="default")
    listing.add_argument("--status", choices=sorted(STATUSES))
    listing.set_defaults(func=command_list)

    log = subparsers.add_parser("log", help="Append explicit or observed habit evidence")
    log.add_argument("--user", default="default")
    log.add_argument("--habit", required=True)
    log.add_argument("--outcome", choices=sorted(ALL_OUTCOMES), required=True)
    log.add_argument("--value", type=float)
    log.add_argument("--unit")
    log.add_argument("--date")
    log.add_argument("--occurred-at")
    log.add_argument("--energy", type=int)
    log.add_argument("--mood", type=int)
    log.add_argument("--urge", type=int)
    log.add_argument("--barrier")
    log.add_argument("--trigger")
    log.add_argument("--cue-observed", action=argparse.BooleanOptionalAction, default=None)
    log.add_argument("--note")
    log.add_argument("--source", choices=["explicit", "observed"], default="explicit")
    log.add_argument("--source-ref")
    log.add_argument("--sensitivity", choices=["normal", "sensitive", "restricted"])
    log.add_argument("--idempotency-key")
    log.set_defaults(func=command_log)

    correct = subparsers.add_parser("correct", help="Supersede an incorrect log and invalidate derived reviews")
    correct.add_argument("--user", default="default")
    correct.add_argument("--log-id", required=True)
    correct.add_argument("--outcome", choices=sorted(ALL_OUTCOMES))
    correct.add_argument("--value", type=float)
    correct.add_argument("--unit")
    correct.add_argument("--occurred-at")
    correct.add_argument("--energy", type=int)
    correct.add_argument("--mood", type=int)
    correct.add_argument("--urge", type=int)
    correct.add_argument("--barrier")
    correct.add_argument("--trigger")
    correct.add_argument("--note")
    correct.add_argument("--sensitivity", choices=["normal", "sensitive", "restricted"])
    correct.add_argument("--idempotency-key")
    correct.set_defaults(func=command_correct)

    today = subparsers.add_parser("today", help="Rank today's primary habits")
    today.add_argument("--user", default="default")
    today.add_argument("--date")
    today.add_argument("--limit", type=int, choices=range(1, 8), default=7)
    today.set_defaults(func=command_today)

    review = subparsers.add_parser("review", help="Compute an evidence-bounded review")
    review.add_argument("--user", default="default")
    review.add_argument("--habit")
    review.add_argument("--days", type=int, default=7)
    review.add_argument("--start")
    review.add_argument("--end")
    review.add_argument("--save", action="store_true")
    review.set_defaults(func=command_review)

    chart = subparsers.add_parser("chart", help="Render Mermaid from known opportunities")
    chart.add_argument("--user", default="default")
    chart.add_argument("--habit", required=True)
    chart.add_argument("--days", type=int, default=28)
    chart.add_argument("--end")
    chart.set_defaults(func=command_chart)

    context = subparsers.add_parser("context", help="Return compact LLM context")
    context.add_argument("--user", default="default")
    context.add_argument("--days", type=int, default=28)
    context.add_argument("--logs-per-habit", type=int, default=8)
    context.set_defaults(func=command_context)

    export = subparsers.add_parser("export", help="Export user-owned state")
    export.add_argument("--user", default="default")
    export.add_argument("--format", choices=["json", "csv"], default="json")
    export.add_argument("--output")
    export.set_defaults(func=command_export)

    season = subparsers.add_parser("season", help="Change the current operating season")
    season.add_argument("--user", default="default")
    season.add_argument("--kind", choices=sorted(SEASONS), required=True)
    season.add_argument("--reason", required=True)
    season.add_argument("--review-at")
    season.set_defaults(func=command_season)

    experiment = subparsers.add_parser("experiment", help="Create a bounded behavior experiment")
    experiment.add_argument("--user", default="default")
    experiment.add_argument("--habit", required=True)
    experiment.add_argument("--hypothesis", required=True)
    experiment.add_argument("--change", required=True)
    experiment.add_argument("--start", required=True)
    experiment.add_argument("--end", required=True)
    experiment.add_argument("--evidence", required=True)
    experiment.add_argument("--success", required=True)
    experiment.add_argument("--stop", required=True)
    experiment.add_argument("--rollback", required=True)
    experiment.add_argument("--status", choices=["proposed", "active"], default="proposed")
    experiment.set_defaults(func=command_experiment)

    delete = subparsers.add_parser("delete", help="Delete user-owned logs, habits, or all state")
    delete.add_argument("--user", default="default")
    delete.add_argument("--scope", choices=["log", "habit", "all"], required=True)
    delete.add_argument("--target", help="Required for log or habit scope")
    delete.add_argument("--confirm", required=True)
    delete.set_defaults(func=command_delete)

    doctor = subparsers.add_parser("doctor", help="Validate database integrity")
    doctor.set_defaults(func=command_doctor)
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        args.func(args)
        return 0
    except HabitOSError as exc:
        emit({"status": "error", "error": str(exc)})
        return 2
    except sqlite3.IntegrityError as exc:
        emit({"status": "error", "error": f"State constraint failed: {exc}"})
        return 3


if __name__ == "__main__":
    raise SystemExit(main())

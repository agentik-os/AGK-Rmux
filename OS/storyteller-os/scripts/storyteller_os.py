#!/usr/bin/env python3
"""Local-first Storyteller {OS} story bank.

The CLI stores canonical Story Objects in SQLite. It never calls a network or
an LLM. Its score is a structural-completeness heuristic, not a truth, quality,
or virality judgment.
"""

from __future__ import annotations

import argparse
import json
import re
import sqlite3
import sys
import unicodedata
import uuid
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Iterable


SCHEMA_VERSION = 1
STATUSES = {
    "seed", "captured", "mined", "interviewing", "verified", "shaped",
    "drafted", "approved", "published", "learned", "archived",
    "blocked_truth", "blocked_consent", "private_only", "needs_deepening",
    "retired", "do_not_publish",
}
STORY_CLASSES = {
    "moment", "decision", "failure", "transformation", "origin", "customer",
    "idea", "data", "vision", "cultural", "fiction",
}
TRUTH_CLASSES = {
    "documented", "corroborated", "remembered", "interpreted", "reconstructed",
    "composite", "hypothetical", "fictional",
}
PRIVACY_LEVELS = {"private", "confidential", "limited", "public"}
CONFIDENCE_LEVELS = {"low", "medium", "high"}
CONTRACTS = {"coach", "co-create", "write", "edit"}
JOBS = {
    "connect", "teach", "prove", "position", "lead", "sell", "recruit",
    "warn", "inspire", "remember", "entertain",
}


def now_iso() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat()


def slugify(value: str) -> str:
    normalized = unicodedata.normalize("NFKD", value)
    ascii_value = normalized.encode("ascii", "ignore").decode("ascii")
    slug = re.sub(r"[^a-zA-Z0-9]+", "_", ascii_value).strip("_").lower()
    return slug[:48] or "untitled"


def connect(db_path: str) -> sqlite3.Connection:
    path = Path(db_path).expanduser().resolve()
    path.parent.mkdir(parents=True, exist_ok=True)
    connection = sqlite3.connect(path)
    connection.row_factory = sqlite3.Row
    connection.execute("PRAGMA foreign_keys = ON")
    return connection


def init_db(connection: sqlite3.Connection) -> None:
    connection.executescript(
        """
        CREATE TABLE IF NOT EXISTS meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS stories (
            story_id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            status TEXT NOT NULL,
            story_class TEXT NOT NULL,
            truth_class TEXT NOT NULL,
            privacy_level TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            data_json TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_stories_status ON stories(status);
        CREATE INDEX IF NOT EXISTS idx_stories_class ON stories(story_class);
        CREATE INDEX IF NOT EXISTS idx_stories_updated ON stories(updated_at DESC);
        """
    )
    connection.execute(
        "INSERT OR REPLACE INTO meta(key, value) VALUES('schema_version', ?)",
        (str(SCHEMA_VERSION),),
    )
    connection.commit()


def ensure_db(connection: sqlite3.Connection) -> None:
    init_db(connection)


def base_story(
    story_id: str,
    title: str,
    raw_text: str,
    story_class: str,
    truth_class: str,
    privacy_level: str,
    job: str,
    audience: str,
    contract: str,
    tags: list[str],
) -> dict[str, Any]:
    timestamp = now_iso()
    return {
        "story_id": story_id,
        "title_working": title,
        "created_at": timestamp,
        "updated_at": timestamp,
        "status": "captured" if raw_text.strip() else "seed",
        "ownership": {
            "storyteller": "",
            "recorder": "",
            "primary_character": "",
            "other_people": [],
        },
        "intent": {
            "job": job,
            "audience": audience,
            "desired_update": "",
            "channels": [],
            "agency_contract": contract,
        },
        "source": {
            "source_type": "text" if raw_text.strip() else "memory",
            "raw_text": raw_text,
            "source_refs": [],
            "artifacts": [],
            "captured_at": timestamp,
        },
        "truth": {
            "primary_class": truth_class,
            "overall_confidence": "medium",
            "chronology": "unknown",
            "dialogue": "unknown",
            "claims": [],
            "unresolved": [],
        },
        "privacy": {
            "level": privacy_level,
            "identifiable_people": [],
            "consent_records": [],
            "confidential_topics": [],
            "release_constraints": [],
        },
        "dna": {
            "core_change": "",
            "pressure": "",
            "hinge": "",
            "proof_detail": "",
            "meaning": "",
            "truth_boundary": "",
            "voice_marker": "",
            "dignity_constraint": "",
        },
        "craft": {
            "story_class": story_class,
            "desire": "",
            "old_belief": "",
            "obstacle": "",
            "stakes": "",
            "choice": "",
            "external_consequence": "",
            "internal_update": "",
            "residue": "",
            "selected_structure": "",
            "opening": "",
            "ending": "",
            "beats": [],
        },
        "voice": {
            "source_samples": [],
            "fingerprint": [],
            "preserve": [],
            "avoid": [],
        },
        "versions": [],
        "derivatives": [],
        "evaluation": {
            "truth_gate": "pending",
            "consent_gate": "pending",
            "scores": [],
            "release_verdict": "needs_deepening",
        },
        "performance": {
            "publications": [],
            "audience_evidence": [],
            "learning": [],
        },
        "tags": tags,
        "connections": [],
        "next_action": "",
    }


def make_story_id(connection: sqlite3.Connection, title: str) -> str:
    date_part = datetime.now(timezone.utc).strftime("%Y%m%d")
    base = f"sto_{date_part}_{slugify(title)}"
    candidate = base
    while connection.execute(
        "SELECT 1 FROM stories WHERE story_id = ?", (candidate,)
    ).fetchone():
        candidate = f"{base}_{uuid.uuid4().hex[:6]}"
    return candidate


def sync_columns(story: dict[str, Any]) -> tuple[Any, ...]:
    return (
        story["title_working"],
        story["status"],
        story["craft"]["story_class"],
        story["truth"]["primary_class"],
        story["privacy"]["level"],
        story["created_at"],
        story["updated_at"],
        json.dumps(story, ensure_ascii=False, sort_keys=True),
    )


def insert_story(connection: sqlite3.Connection, story: dict[str, Any]) -> None:
    columns = sync_columns(story)
    connection.execute(
        """
        INSERT INTO stories(
            story_id, title, status, story_class, truth_class, privacy_level,
            created_at, updated_at, data_json
        ) VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (story["story_id"], *columns),
    )
    connection.commit()


def save_story(connection: sqlite3.Connection, story: dict[str, Any]) -> None:
    story["updated_at"] = now_iso()
    columns = sync_columns(story)
    connection.execute(
        """
        UPDATE stories SET
            title = ?, status = ?, story_class = ?, truth_class = ?,
            privacy_level = ?, created_at = ?, updated_at = ?, data_json = ?
        WHERE story_id = ?
        """,
        (*columns, story["story_id"]),
    )
    connection.commit()


def load_story(connection: sqlite3.Connection, story_id: str) -> dict[str, Any]:
    row = connection.execute(
        "SELECT data_json FROM stories WHERE story_id = ?", (story_id,)
    ).fetchone()
    if row is None:
        raise SystemExit(f"Story not found: {story_id}")
    return json.loads(row["data_json"])


def parse_value(raw: str) -> Any:
    try:
        return json.loads(raw)
    except json.JSONDecodeError:
        return raw


def set_path(data: dict[str, Any], dotted_path: str, value: Any) -> None:
    parts = [part for part in dotted_path.split(".") if part]
    if not parts:
        raise ValueError("Empty field path")
    cursor: Any = data
    for part in parts[:-1]:
        if not isinstance(cursor, dict):
            raise ValueError(f"Cannot descend through non-object at {part}")
        cursor = cursor.setdefault(part, {})
    if not isinstance(cursor, dict):
        raise ValueError(f"Cannot set field {dotted_path}")
    cursor[parts[-1]] = value


def get_path(data: dict[str, Any], dotted_path: str, default: Any = "") -> Any:
    cursor: Any = data
    for part in dotted_path.split("."):
        if not isinstance(cursor, dict) or part not in cursor:
            return default
        cursor = cursor[part]
    return cursor


def filled(value: Any) -> bool:
    if value is None:
        return False
    if isinstance(value, str):
        return bool(value.strip())
    if isinstance(value, (list, dict, tuple, set)):
        return bool(value)
    return True


def validate_story(story: dict[str, Any]) -> tuple[list[str], list[str]]:
    errors: list[str] = []
    warnings: list[str] = []

    for field in ("story_id", "title_working", "status", "created_at", "updated_at"):
        if not filled(story.get(field)):
            errors.append(f"Missing required field: {field}")

    if story.get("status") not in STATUSES:
        errors.append(f"Invalid status: {story.get('status')}")
    story_class = get_path(story, "craft.story_class")
    if story_class not in STORY_CLASSES:
        errors.append(f"Invalid story class: {story_class}")
    truth_class = get_path(story, "truth.primary_class")
    if truth_class not in TRUTH_CLASSES:
        errors.append(f"Invalid truth class: {truth_class}")
    privacy = get_path(story, "privacy.level")
    if privacy not in PRIVACY_LEVELS:
        errors.append(f"Invalid privacy level: {privacy}")
    confidence = get_path(story, "truth.overall_confidence")
    if confidence not in CONFIDENCE_LEVELS:
        errors.append(f"Invalid confidence: {confidence}")
    contract = get_path(story, "intent.agency_contract")
    if contract not in CONTRACTS:
        errors.append(f"Invalid agency contract: {contract}")
    job = get_path(story, "intent.job")
    if job not in JOBS:
        errors.append(f"Invalid story job: {job}")

    claims = get_path(story, "truth.claims", [])
    if not isinstance(claims, list):
        errors.append("truth.claims must be a list")
    else:
        high_risk_unverified = [
            claim for claim in claims
            if claim.get("consequence_if_wrong") == "high"
            and claim.get("verification_status") not in {"verified", "qualified", "removed"}
        ]
        if high_risk_unverified:
            warnings.append(
                f"{len(high_risk_unverified)} high-consequence claim(s) need verification"
            )

    if story.get("status") in {"approved", "published"}:
        truth_gate = get_path(story, "evaluation.truth_gate")
        consent_gate = get_path(story, "evaluation.consent_gate")
        if truth_gate != "pass":
            errors.append("Approved/published story requires evaluation.truth_gate=pass")
        if consent_gate != "pass":
            errors.append("Approved/published story requires evaluation.consent_gate=pass")

    if not filled(get_path(story, "source.raw_text")):
        warnings.append("No raw source text captured")
    if not filled(get_path(story, "dna.core_change")):
        warnings.append("Story DNA has no core change")
    if privacy != "public" and story.get("status") == "published":
        warnings.append("Published story is not marked public")

    return errors, warnings


def structural_score(story: dict[str, Any]) -> dict[str, Any]:
    dimensions: dict[str, dict[str, Any]] = {}

    def add(name: str, weight: int, checks: Iterable[bool]) -> None:
        check_list = list(checks)
        ratio = sum(1 for check in check_list if check) / max(1, len(check_list))
        points = round(weight * ratio)
        dimensions[name] = {"points": points, "weight": weight}

    add("core_change", 15, [
        filled(get_path(story, "dna.core_change")),
        filled(get_path(story, "craft.internal_update"))
        or filled(get_path(story, "craft.external_consequence")),
    ])
    add("tension_stakes", 15, [
        filled(get_path(story, "dna.pressure")),
        filled(get_path(story, "craft.obstacle")),
        filled(get_path(story, "craft.stakes")),
    ])
    add("specificity_proof", 10, [
        filled(get_path(story, "dna.proof_detail")),
        filled(get_path(story, "source.raw_text")),
    ])
    add("causality_choice", 10, [
        filled(get_path(story, "dna.hinge")),
        filled(get_path(story, "craft.choice")),
        filled(get_path(story, "craft.external_consequence")),
    ])
    beats = get_path(story, "craft.beats", [])
    add("structure_pacing", 10, [
        filled(get_path(story, "craft.selected_structure")),
        isinstance(beats, list) and len(beats) >= 3,
    ])
    add("meaning_boundary", 10, [
        filled(get_path(story, "dna.meaning")),
        filled(get_path(story, "dna.truth_boundary")),
    ])
    add("voice_ownership", 10, [
        filled(get_path(story, "dna.voice_marker")),
        filled(get_path(story, "voice.source_samples"))
        or filled(get_path(story, "voice.fingerprint")),
    ])
    add("audience_job_fit", 10, [
        filled(get_path(story, "intent.audience")),
        filled(get_path(story, "intent.desired_update")),
        get_path(story, "intent.job") in JOBS,
    ])
    add("opening_contract", 5, [filled(get_path(story, "craft.opening"))])
    add("ending_echo", 5, [filled(get_path(story, "craft.ending"))])

    total = sum(item["points"] for item in dimensions.values())
    missing = [name for name, item in dimensions.items() if item["points"] < item["weight"]]
    return {
        "score_type": "structural_completeness",
        "total": total,
        "maximum": 100,
        "dimensions": dimensions,
        "incomplete_dimensions": missing,
        "warning": (
            "This deterministic score does not judge truth, consent, literary quality, "
            "audience response, or virality."
        ),
    }


def story_summary(story: dict[str, Any]) -> dict[str, Any]:
    return {
        "story_id": story["story_id"],
        "title": story["title_working"],
        "status": story["status"],
        "story_class": get_path(story, "craft.story_class"),
        "truth_class": get_path(story, "truth.primary_class"),
        "privacy": get_path(story, "privacy.level"),
        "core_change": get_path(story, "dna.core_change"),
        "updated_at": story["updated_at"],
    }


def print_json(value: Any) -> None:
    print(json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True))


def cmd_init(args: argparse.Namespace) -> None:
    connection = connect(args.db)
    init_db(connection)
    print_json({"status": "ok", "database": str(Path(args.db).expanduser().resolve()), "schema_version": SCHEMA_VERSION})


def cmd_capture(args: argparse.Namespace) -> None:
    connection = connect(args.db)
    ensure_db(connection)
    raw_text = args.raw or ""
    if args.raw_file:
        raw_text = Path(args.raw_file).read_text(encoding="utf-8")
    story_id = make_story_id(connection, args.title)
    tags = [tag.strip() for tag in (args.tags or "").split(",") if tag.strip()]
    story = base_story(
        story_id=story_id,
        title=args.title,
        raw_text=raw_text,
        story_class=args.story_class,
        truth_class=args.truth_class,
        privacy_level=args.privacy,
        job=args.job,
        audience=args.audience or "",
        contract=args.contract,
        tags=tags,
    )
    insert_story(connection, story)
    print_json(story_summary(story))


def cmd_list(args: argparse.Namespace) -> None:
    connection = connect(args.db)
    ensure_db(connection)
    clauses: list[str] = []
    params: list[Any] = []
    if args.status:
        clauses.append("status = ?")
        params.append(args.status)
    if args.story_class:
        clauses.append("story_class = ?")
        params.append(args.story_class)
    if args.query:
        clauses.append("(title LIKE ? OR data_json LIKE ?)")
        needle = f"%{args.query}%"
        params.extend([needle, needle])
    where = f"WHERE {' AND '.join(clauses)}" if clauses else ""
    rows = connection.execute(
        f"SELECT data_json FROM stories {where} ORDER BY updated_at DESC LIMIT ?",
        (*params, args.limit),
    ).fetchall()
    stories = [story_summary(json.loads(row["data_json"])) for row in rows]
    print_json({"count": len(stories), "stories": stories})


def cmd_show(args: argparse.Namespace) -> None:
    connection = connect(args.db)
    ensure_db(connection)
    print_json(load_story(connection, args.story_id))


def cmd_update(args: argparse.Namespace) -> None:
    connection = connect(args.db)
    ensure_db(connection)
    story = load_story(connection, args.story_id)
    changes: list[str] = []
    for assignment in args.set_values:
        if "=" not in assignment:
            raise SystemExit(f"Invalid --set value, expected path=value: {assignment}")
        path, raw = assignment.split("=", 1)
        value = parse_value(raw)
        set_path(story, path, value)
        changes.append(path)
    errors, warnings = validate_story(story)
    if errors:
        print_json({"status": "error", "errors": errors, "warnings": warnings})
        raise SystemExit(2)
    save_story(connection, story)
    print_json({"status": "ok", "story": story_summary(story), "changed": changes, "warnings": warnings})


def cmd_add_claim(args: argparse.Namespace) -> None:
    connection = connect(args.db)
    ensure_db(connection)
    story = load_story(connection, args.story_id)
    claim = {
        "claim_id": f"clm_{uuid.uuid4().hex[:8]}",
        "text": args.text,
        "type": args.type,
        "truth_class": args.truth_class,
        "confidence": args.confidence,
        "source_refs": args.source_ref or [],
        "consequence_if_wrong": args.consequence,
        "verification_status": args.verification_status,
        "public_wording": args.public_wording or "",
        "notes": args.notes or "",
    }
    story["truth"]["claims"].append(claim)
    save_story(connection, story)
    print_json(claim)


def cmd_add_consent(args: argparse.Namespace) -> None:
    connection = connect(args.db)
    ensure_db(connection)
    story = load_story(connection, args.story_id)
    consent = {
        "consent_id": f"con_{uuid.uuid4().hex[:8]}",
        "person_ref": args.person,
        "status": args.status,
        "scope": {
            "versions": args.version or [],
            "channels": args.channel or [],
            "audience": args.audience or "",
        },
        "identity_mode": args.identity_mode,
        "evidence_ref": args.evidence_ref or "",
        "granted_at": args.granted_at or now_iso(),
        "expires_at": args.expires_at,
        "restrictions": args.restriction or [],
    }
    story["privacy"]["consent_records"].append(consent)
    save_story(connection, story)
    print_json(consent)


def cmd_validate(args: argparse.Namespace) -> None:
    connection = connect(args.db)
    ensure_db(connection)
    story = load_story(connection, args.story_id)
    errors, warnings = validate_story(story)
    print_json({"valid": not errors, "errors": errors, "warnings": warnings})
    if errors:
        raise SystemExit(2)


def cmd_score(args: argparse.Namespace) -> None:
    connection = connect(args.db)
    ensure_db(connection)
    story = load_story(connection, args.story_id)
    errors, warnings = validate_story(story)
    report = structural_score(story)
    report["validation_errors"] = errors
    report["validation_warnings"] = warnings
    print_json(report)


def markdown_story(story: dict[str, Any]) -> str:
    dna = story["dna"]
    craft = story["craft"]
    lines = [
        f"# {story['title_working']}",
        "",
        f"- Story ID: `{story['story_id']}`",
        f"- Status: `{story['status']}`",
        f"- Class: `{craft['story_class']}`",
        f"- Truth: `{story['truth']['primary_class']}`",
        f"- Privacy: `{story['privacy']['level']}`",
        "",
        "## Story DNA",
        "",
        f"- Core change: {dna['core_change'] or '—'}",
        f"- Pressure: {dna['pressure'] or '—'}",
        f"- Hinge: {dna['hinge'] or '—'}",
        f"- Proof detail: {dna['proof_detail'] or '—'}",
        f"- Meaning: {dna['meaning'] or '—'}",
        f"- Truth boundary: {dna['truth_boundary'] or '—'}",
        "",
        "## Raw source",
        "",
        story["source"]["raw_text"] or "—",
        "",
    ]
    return "\n".join(lines)


def cmd_export(args: argparse.Namespace) -> None:
    connection = connect(args.db)
    ensure_db(connection)
    if args.story_id:
        stories = [load_story(connection, args.story_id)]
    else:
        rows = connection.execute("SELECT data_json FROM stories ORDER BY updated_at").fetchall()
        stories = [json.loads(row["data_json"]) for row in rows]

    output = Path(args.output).expanduser().resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    if args.format == "jsonl":
        text = "\n".join(json.dumps(story, ensure_ascii=False, sort_keys=True) for story in stories) + ("\n" if stories else "")
    elif args.format == "json":
        text = json.dumps(stories, ensure_ascii=False, indent=2, sort_keys=True) + "\n"
    else:
        text = "\n---\n\n".join(markdown_story(story) for story in stories)
    output.write_text(text, encoding="utf-8")
    print_json({"status": "ok", "output": str(output), "format": args.format, "count": len(stories)})


def cmd_doctor(args: argparse.Namespace) -> None:
    connection = connect(args.db)
    ensure_db(connection)
    rows = connection.execute("SELECT data_json FROM stories ORDER BY story_id").fetchall()
    reports: list[dict[str, Any]] = []
    invalid = 0
    for row in rows:
        story = json.loads(row["data_json"])
        errors, warnings = validate_story(story)
        if errors:
            invalid += 1
        reports.append({
            "story_id": story.get("story_id"),
            "valid": not errors,
            "errors": errors,
            "warnings": warnings,
        })
    print_json({
        "schema_version": SCHEMA_VERSION,
        "stories": len(reports),
        "invalid": invalid,
        "reports": reports,
    })
    if invalid:
        raise SystemExit(2)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Local Storyteller {OS} story bank")
    parser.add_argument("--version", action="version", version=f"%(prog)s {SCHEMA_VERSION}.0")
    subparsers = parser.add_subparsers(dest="command", required=True)

    def with_db(subparser: argparse.ArgumentParser) -> None:
        subparser.add_argument("--db", default="storyteller.db", help="SQLite database path")

    init_parser = subparsers.add_parser("init", help="Initialize a story bank")
    with_db(init_parser)
    init_parser.set_defaults(func=cmd_init)

    capture = subparsers.add_parser("capture", help="Capture a new Story Object")
    with_db(capture)
    capture.add_argument("--title", required=True)
    source_group = capture.add_mutually_exclusive_group()
    source_group.add_argument("--raw", help="Raw source text")
    source_group.add_argument("--raw-file", help="UTF-8 file containing raw source")
    capture.add_argument("--story-class", choices=sorted(STORY_CLASSES), default="moment")
    capture.add_argument("--truth-class", choices=sorted(TRUTH_CLASSES), default="remembered")
    capture.add_argument("--privacy", choices=sorted(PRIVACY_LEVELS), default="private")
    capture.add_argument("--job", choices=sorted(JOBS), default="connect")
    capture.add_argument("--audience", default="")
    capture.add_argument("--contract", choices=sorted(CONTRACTS), default="coach")
    capture.add_argument("--tags", help="Comma-separated tags")
    capture.set_defaults(func=cmd_capture)

    list_parser = subparsers.add_parser("list", help="List/search Story Objects")
    with_db(list_parser)
    list_parser.add_argument("--status", choices=sorted(STATUSES))
    list_parser.add_argument("--story-class", choices=sorted(STORY_CLASSES))
    list_parser.add_argument("--query")
    list_parser.add_argument("--limit", type=int, default=50)
    list_parser.set_defaults(func=cmd_list)

    show = subparsers.add_parser("show", help="Show a complete Story Object")
    with_db(show)
    show.add_argument("story_id")
    show.set_defaults(func=cmd_show)

    update = subparsers.add_parser("update", help="Set dotted Story Object fields")
    with_db(update)
    update.add_argument("story_id")
    update.add_argument("--set", dest="set_values", action="append", required=True, help="path=value; value may be JSON")
    update.set_defaults(func=cmd_update)

    claim = subparsers.add_parser("add-claim", help="Add a claim-ledger entry")
    with_db(claim)
    claim.add_argument("story_id")
    claim.add_argument("--text", required=True)
    claim.add_argument("--type", default="fact", choices=[
        "fact", "number", "quotation", "chronology", "attribution", "motive",
        "causal_inference", "emotional_interpretation", "future_projection",
    ])
    claim.add_argument("--truth-class", choices=sorted(TRUTH_CLASSES), default="remembered")
    claim.add_argument("--confidence", choices=sorted(CONFIDENCE_LEVELS), default="medium")
    claim.add_argument("--source-ref", action="append")
    claim.add_argument("--consequence", choices=["low", "medium", "high"], default="medium")
    claim.add_argument("--verification-status", choices=[
        "unreviewed", "needs_source", "verified", "qualified", "disputed", "removed",
    ], default="unreviewed")
    claim.add_argument("--public-wording")
    claim.add_argument("--notes")
    claim.set_defaults(func=cmd_add_claim)

    consent = subparsers.add_parser("add-consent", help="Add a consent record")
    with_db(consent)
    consent.add_argument("story_id")
    consent.add_argument("--person", required=True)
    consent.add_argument("--status", choices=["unknown", "requested", "granted", "limited", "declined", "withdrawn"], required=True)
    consent.add_argument("--version", action="append")
    consent.add_argument("--channel", action="append")
    consent.add_argument("--audience")
    consent.add_argument("--identity-mode", choices=["named", "first_name_only", "role_only", "anonymized", "composite_labeled", "private_only"], default="private_only")
    consent.add_argument("--evidence-ref")
    consent.add_argument("--granted-at")
    consent.add_argument("--expires-at")
    consent.add_argument("--restriction", action="append")
    consent.set_defaults(func=cmd_add_consent)

    validate = subparsers.add_parser("validate", help="Validate one Story Object")
    with_db(validate)
    validate.add_argument("story_id")
    validate.set_defaults(func=cmd_validate)

    score = subparsers.add_parser("score", help="Score structural completeness")
    with_db(score)
    score.add_argument("story_id")
    score.set_defaults(func=cmd_score)

    export = subparsers.add_parser("export", help="Export Story Objects")
    with_db(export)
    export.add_argument("--story-id")
    export.add_argument("--format", choices=["jsonl", "json", "markdown"], default="jsonl")
    export.add_argument("--output", required=True)
    export.set_defaults(func=cmd_export)

    doctor = subparsers.add_parser("doctor", help="Validate the complete bank")
    with_db(doctor)
    doctor.set_defaults(func=cmd_doctor)

    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        args.func(args)
        return 0
    except (OSError, sqlite3.Error, ValueError) as error:
        print_json({"status": "error", "message": str(error)})
        return 2


if __name__ == "__main__":
    sys.exit(main())

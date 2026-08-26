#!/usr/bin/env python3
"""data-summarize.py — Normalize schema scans, ORM detection, migrations."""
from __future__ import annotations
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from _summarize_common import load_json, main_skeleton  # noqa: E402


def parse_convex(meta) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {}
    if isinstance(meta, dict):
        tables = meta.get("tables", []) or []
        metrics["convex_schema_lines"] = len(tables)
    return findings, metrics


def parse_prisma_validate(text: str) -> list[dict]:
    findings: list[dict] = []
    if not text:
        return findings
    if "Error" in text or "error" in text:
        findings.append({
            "tool": "prisma-validate", "severity": "high",
            "location": "schema.prisma", "rule": "prisma-invalid",
            "message": text.strip()[:300],
            "suggested_fix": "Fix the Prisma schema errors.",
        })
    return findings


def parse_orm_schemas(data) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {}
    if not isinstance(data, dict):
        return findings, metrics
    files = data.get("schema_files", []) or []
    metrics["orm_schema_files"] = files[:25]
    metrics["orm_schema_count"] = len(files)
    return findings, metrics


def parse_migrations(data) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {}
    if not isinstance(data, dict):
        return findings, metrics
    dirs = data.get("migration_dirs", []) or []
    metrics["migration_dirs"] = dirs
    total = sum(d.get("file_count", 0) for d in dirs)
    metrics["migration_file_count"] = total
    return findings, metrics


def parse_sql_files(data) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {}
    if not isinstance(data, dict):
        return findings, metrics
    files = data.get("sql_files", []) or []
    metrics["sql_files"] = files
    total_tables = sum(f.get("create_table_count", 0) for f in files)
    total_indexes = sum(f.get("create_index_count", 0) for f in files)
    metrics["sql_total_tables"] = total_tables
    metrics["sql_total_indexes"] = total_indexes
    # Heuristic: if create_table > 5 but no indexes, flag missing index strategy
    for f in files:
        tables = f.get("create_table_count", 0)
        idx = f.get("create_index_count", 0)
        if tables >= 3 and idx == 0:
            findings.append({
                "tool": "sql-file-scanner", "severity": "medium",
                "location": f.get("file"), "rule": "no-indexes",
                "message": f"{tables} tables defined but 0 indexes — likely missing index strategy",
                "suggested_fix": "Add indexes on FK columns and frequently-queried fields.",
            })
    return findings, metrics


def parse_sqlite(data) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {}
    if not isinstance(data, dict):
        return findings, metrics
    dbs = data.get("databases", []) or []
    metrics["sqlite_databases"] = dbs
    for db in dbs:
        if db.get("integrity") and "ok" not in str(db.get("integrity", "")).lower():
            findings.append({
                "tool": "sqlite-health", "severity": "critical",
                "location": db.get("file"), "rule": "integrity-fail",
                "message": f"SQLite integrity check: {db.get('integrity')}",
                "suggested_fix": "Backup + recover. Use `sqlite3 db .recover`.",
            })
    return findings, metrics


def parse_type_defs(data) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {}
    if not isinstance(data, dict):
        return findings, metrics
    types = data.get("type_defs", []) or []
    metrics["type_def_count"] = len(types)
    # Find duplicate type names across files (potential drift)
    by_name: dict[str, list[dict]] = {}
    for t in types:
        by_name.setdefault(t.get("name"), []).append(t)
    duplicates = {n: locs for n, locs in by_name.items() if len(locs) > 1}
    metrics["duplicate_type_names"] = len(duplicates)
    metrics["duplicate_type_sample"] = {
        n: locs for n, locs in list(duplicates.items())[:5]
    }
    for name, locs in list(duplicates.items())[:10]:
        files = ", ".join({l.get("file") for l in locs})
        findings.append({
            "tool": "type-def-scanner", "severity": "low",
            "location": locs[0].get("file") + ":" + str(locs[0].get("line")),
            "rule": "duplicate-type-name",
            "message": f"Type/interface '{name}' is defined in multiple files: {files}",
            "suggested_fix": "Consolidate to a single types module to prevent schema drift.",
        })
    return findings, metrics


def gather(raw_dir: Path, envelope: dict) -> list[dict]:
    all_findings: list[dict] = []
    metrics = envelope.setdefault("metrics", {})

    cm = load_json(raw_dir / "convex-schema-meta.json")
    if cm is not None:
        f, m = parse_convex(cm)
        all_findings.extend(f)
        metrics.update(m)

    pv_path = raw_dir / "prisma-validate.txt"
    if pv_path.exists() and pv_path.stat().st_size > 0:
        all_findings.extend(parse_prisma_validate(pv_path.read_text(errors="replace")))

    orm = load_json(raw_dir / "orm-schemas.json")
    if orm is not None:
        f, m = parse_orm_schemas(orm)
        all_findings.extend(f)
        metrics.update(m)

    mig = load_json(raw_dir / "migrations.json")
    if mig is not None:
        f, m = parse_migrations(mig)
        all_findings.extend(f)
        metrics.update(m)

    sql = load_json(raw_dir / "sql-files.json")
    if sql is not None:
        f, m = parse_sql_files(sql)
        all_findings.extend(f)
        metrics.update(m)

    sq = load_json(raw_dir / "sqlite-health.json")
    if sq is not None:
        f, m = parse_sqlite(sq)
        all_findings.extend(f)
        metrics.update(m)

    td = load_json(raw_dir / "type-defs.json")
    if td is not None:
        f, m = parse_type_defs(td)
        all_findings.extend(f)
        metrics.update(m)

    return all_findings


if __name__ == "__main__":
    main_skeleton("data", gather)

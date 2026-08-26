#!/usr/bin/env python3
"""_summarize_common.py — shared helpers for evidence summarizers.

Each <audit>-summarize.py reads $1 (raw dir) and writes evidence-summary.json
to stdout. The schema is unified:

    {
      "audit": "<name>",
      "tools_run": [...],
      "tools_skipped": [{"tool": "x", "reason": "..."}],
      "findings_total": int,
      "findings_by_severity": {"critical": n, "high": n, "medium": n, "low": n, "info": n},
      "findings": [
        {
          "tool": "...",
          "severity": "critical|high|medium|low|info",
          "location": "file:line" | "url" | null,
          "rule": "rule-id" | null,
          "message": "...",
          "suggested_fix": "..." | null,
          "raw_ref": "raw-file.json" | null
        }
      ],
      "metrics": { ... tool-specific quantitative data ... },
      "evidence_index": [{"file": "x.json", "size": n, "tool": "..."}]
    }
"""
from __future__ import annotations
import json
import os
import sys
from collections import defaultdict
from pathlib import Path
from typing import Any


SEVERITY_RANK = {"critical": 5, "high": 4, "medium": 3, "low": 2, "info": 1}
NOISE_DIRS = ("node_modules/", ".next/", "dist/", "build/", ".turbo/", ".cache/", "venv/", ".venv/")
NOISE_SUFFIXES = (".d.ts.map", ".min.js", ".min.css", ".lock")


def is_noise_path(path: str | None) -> bool:
    if not path:
        return False
    return any(noise in path for noise in NOISE_DIRS) or path.endswith(NOISE_SUFFIXES)


def normalize_severity(s: str | int | None) -> str:
    """Map any tool's severity nomenclature to our 5-level scale."""
    if s is None:
        return "info"
    if isinstance(s, int):
        # eslint: 2=error, 1=warn, 0=off
        return {2: "high", 1: "medium", 0: "info"}.get(s, "info")
    s = str(s).lower().strip()
    mapping = {
        "critical": "critical",
        "blocker": "critical",
        "fatal": "critical",
        "error": "high",
        "high": "high",
        "serious": "high",
        "moderate": "medium",
        "medium": "medium",
        "warning": "medium",
        "warn": "medium",
        "minor": "low",
        "low": "low",
        "info": "info",
        "informational": "info",
        "note": "info",
    }
    return mapping.get(s, "info")


def load_json(path: Path) -> Any:
    try:
        with open(path, "r") as f:
            content = f.read().strip()
            if not content or content == "[]":
                return None
            return json.loads(content)
    except (json.JSONDecodeError, FileNotFoundError, OSError):
        return None


def load_jsonl(path: Path) -> list[dict]:
    """Load a .jsonl file, one JSON object per line."""
    out: list[dict] = []
    if not path.exists():
        return out
    with open(path, "r") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                out.append(json.loads(line))
            except json.JSONDecodeError:
                continue
    return out


def list_raw_files(raw_dir: Path) -> list[Path]:
    """All non-meta JSON files in raw dir."""
    files: list[Path] = []
    for f in sorted(raw_dir.iterdir()):
        if f.is_file() and not f.name.startswith("_") and f.suffix in (".json", ".jsonl", ".txt", ".log"):
            files.append(f)
    return files


def dedupe_findings(findings: list[dict]) -> list[dict]:
    """Deduplicate by (location, rule, message[:80])."""
    seen: dict[tuple, dict] = {}
    for f in findings:
        key = (
            f.get("location") or "",
            f.get("rule") or "",
            (f.get("message") or "")[:80],
        )
        if key in seen:
            # Keep highest severity, merge tools
            existing = seen[key]
            if SEVERITY_RANK.get(f["severity"], 0) > SEVERITY_RANK.get(existing["severity"], 0):
                existing["severity"] = f["severity"]
            existing.setdefault("also_reported_by", []).append(f.get("tool"))
        else:
            seen[key] = f
    return list(seen.values())


def cap_per_file(findings: list[dict], cap: int = 5) -> list[dict]:
    """Limit findings per file:line to <cap> per file."""
    by_file: dict[str, int] = defaultdict(int)
    out: list[dict] = []
    for f in findings:
        loc = f.get("location") or ""
        file_part = loc.split(":")[0] if loc else ""
        if file_part and by_file[file_part] >= cap:
            continue
        if file_part:
            by_file[file_part] += 1
        out.append(f)
    return out


def sort_findings(findings: list[dict]) -> list[dict]:
    """Sort by severity (desc) then file path."""
    return sorted(
        findings,
        key=lambda f: (
            -SEVERITY_RANK.get(f.get("severity", "info"), 1),
            f.get("location") or "",
        ),
    )


def build_envelope(audit: str, raw_dir: Path) -> dict:
    runs = load_jsonl(raw_dir / "_tools-run.jsonl")
    skips = load_jsonl(raw_dir / "_tools-skipped.jsonl")
    return {
        "audit": audit,
        "tools_run": [r["tool"] for r in runs],
        "tools_skipped": [{"tool": s.get("tool"), "reason": s.get("reason")} for s in skips],
        "findings_total": 0,
        "findings_by_severity": {"critical": 0, "high": 0, "medium": 0, "low": 0, "info": 0},
        "findings": [],
        "metrics": {},
        "evidence_index": [
            {"file": f.name, "size": f.stat().st_size}
            for f in list_raw_files(raw_dir)
        ],
    }


def finalize(envelope: dict, findings: list[dict], cap_per_file_n: int = 5) -> dict:
    """Common post-processing: dedupe, cap, sort, count."""
    findings = [f for f in findings if not is_noise_path(f.get("location"))]
    findings = dedupe_findings(findings)
    findings = sort_findings(findings)
    findings = cap_per_file(findings, cap_per_file_n)
    envelope["findings"] = findings
    envelope["findings_total"] = len(findings)
    counts = {"critical": 0, "high": 0, "medium": 0, "low": 0, "info": 0}
    for f in findings:
        counts[f.get("severity", "info")] = counts.get(f.get("severity", "info"), 0) + 1
    envelope["findings_by_severity"] = counts
    return envelope


def emit(envelope: dict) -> None:
    json.dump(envelope, sys.stdout, indent=2, default=str)
    sys.stdout.write("\n")


def main_skeleton(audit_name: str, gather_fn) -> None:
    """Each summarizer wraps gather_fn(raw_dir, envelope) → list[finding]."""
    if len(sys.argv) < 2:
        print(f"Usage: {audit_name}-summarize.py <raw_dir>", file=sys.stderr)
        sys.exit(1)
    raw_dir = Path(sys.argv[1])
    if not raw_dir.exists():
        emit({"audit": audit_name, "error": f"raw dir not found: {raw_dir}"})
        return
    envelope = build_envelope(audit_name, raw_dir)
    try:
        findings = gather_fn(raw_dir, envelope) or []
    except Exception as e:
        envelope["summarizer_error"] = f"{type(e).__name__}: {e}"
        findings = []
    envelope = finalize(envelope, findings)
    emit(envelope)

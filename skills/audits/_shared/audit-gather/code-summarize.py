#!/usr/bin/env python3
"""code-summarize.py — Normalize ESLint, tsc, ts-prune, depcheck, madge, ruff."""
from __future__ import annotations
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from _summarize_common import (  # noqa: E402
    load_json,
    main_skeleton,
    normalize_severity,
)


def parse_eslint(data) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {"eslint_files_scanned": 0, "eslint_errors": 0, "eslint_warnings": 0}
    if not isinstance(data, list):
        return findings, metrics
    metrics["eslint_files_scanned"] = len(data)
    for fr in data:
        path = fr.get("filePath", "")
        rel = path.split("/")
        if "node_modules" in rel:
            continue
        for msg in fr.get("messages", []) or []:
            sev_int = msg.get("severity", 0)
            if sev_int == 0:
                continue
            sev = normalize_severity(sev_int)
            metrics["eslint_errors" if sev_int == 2 else "eslint_warnings"] += 1
            findings.append({
                "tool": "eslint",
                "severity": sev,
                "location": f"{path}:{msg.get('line', 0)}:{msg.get('column', 0)}",
                "rule": msg.get("ruleId"),
                "message": msg.get("message", ""),
                "suggested_fix": "Run `eslint --fix` or refactor as advised.",
            })
    return findings, metrics


def parse_tsc(text: str) -> list[dict]:
    """tsc output: 'path/to/file.ts(line,col): error TSxxxx: message'."""
    findings: list[dict] = []
    for line in text.splitlines():
        m = re.match(r"^(.+?)\((\d+),(\d+)\):\s+(error|warning)\s+(TS\d+):\s*(.+)$", line)
        if not m:
            continue
        path, ln, col, kind, code, msg = m.groups()
        findings.append({
            "tool": "tsc",
            "severity": "high" if kind == "error" else "medium",
            "location": f"{path}:{ln}:{col}",
            "rule": code,
            "message": msg,
            "suggested_fix": "Fix the type error.",
        })
    return findings


def parse_ts_prune(text: str) -> list[dict]:
    """ts-prune: 'path/to/file.ts:line - exportName'."""
    findings: list[dict] = []
    for line in text.splitlines():
        m = re.match(r"^(.+?\.tsx?):(\d+)\s+-\s+(.+?)(?:\s+\(used in module\))?$", line.strip())
        if not m:
            continue
        path, ln, name = m.groups()
        if "(used in module)" in line:
            continue  # used internally, not externally
        findings.append({
            "tool": "ts-prune",
            "severity": "low",
            "location": f"{path}:{ln}",
            "rule": "unused-export",
            "message": f"Export '{name}' is unused outside its module",
            "suggested_fix": "Make it private (remove `export`) or delete the symbol if unreachable.",
        })
    return findings


def parse_depcheck(data) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {}
    if not isinstance(data, dict):
        return findings, metrics
    unused = data.get("dependencies", []) or []
    unused_dev = data.get("devDependencies", []) or []
    missing = data.get("missing", {}) or {}
    metrics["depcheck"] = {
        "unused_deps_count": len(unused),
        "unused_dev_deps_count": len(unused_dev),
        "missing_deps_count": len(missing),
        "unused_deps_sample": unused[:10],
        "missing_deps_sample": list(missing.keys())[:10],
    }
    for d in unused[:20]:
        findings.append({
            "tool": "depcheck",
            "severity": "low",
            "location": f"package.json -> dependencies.{d}",
            "rule": "unused-dependency",
            "message": f"Dependency '{d}' is declared but unused",
            "suggested_fix": f"npm uninstall {d}",
        })
    for d in unused_dev[:20]:
        findings.append({
            "tool": "depcheck",
            "severity": "low",
            "location": f"package.json -> devDependencies.{d}",
            "rule": "unused-dev-dependency",
            "message": f"Dev dependency '{d}' is declared but unused",
            "suggested_fix": f"npm uninstall --save-dev {d}",
        })
    for d, files in list(missing.items())[:20]:
        findings.append({
            "tool": "depcheck",
            "severity": "high",
            "location": files[0] if isinstance(files, list) and files else "package.json",
            "rule": "missing-dependency",
            "message": f"Imported module '{d}' is not declared in package.json",
            "suggested_fix": f"npm install {d}",
        })
    return findings, metrics


def parse_madge(data) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {"circular_dep_groups": 0}
    cycles = data if isinstance(data, list) else []
    metrics["circular_dep_groups"] = len(cycles)
    for cycle in cycles[:20]:
        if not isinstance(cycle, list) or not cycle:
            continue
        chain = " -> ".join(cycle + [cycle[0]])
        findings.append({
            "tool": "madge",
            "severity": "high",
            "location": cycle[0],
            "rule": "circular-dependency",
            "message": f"Circular dep: {chain}",
            "suggested_fix": "Break the cycle: extract shared types to a 3rd module, or invert the dependency.",
        })
    return findings, metrics


def parse_ruff(data) -> list[dict]:
    findings: list[dict] = []
    if not isinstance(data, list):
        return findings
    for r in data:
        sev = "high" if r.get("code", "").startswith(("E", "F")) else "medium"
        findings.append({
            "tool": "ruff",
            "severity": sev,
            "location": f"{r.get('filename')}:{r.get('location', {}).get('row', 0)}",
            "rule": r.get("code"),
            "message": r.get("message", ""),
            "suggested_fix": (r.get("fix") or {}).get("message") if isinstance(r.get("fix"), dict) else "Run `ruff check --fix`",
        })
    return findings


def parse_flake8(text: str) -> list[dict]:
    findings: list[dict] = []
    for line in text.splitlines():
        m = re.match(r"^(.+?):(\d+):(\d+):\s+(\w+)\s+(.+)$", line)
        if not m:
            continue
        path, ln, col, code, msg = m.groups()
        sev = "high" if code.startswith(("E9", "F8")) else "medium" if code.startswith("E") else "low"
        findings.append({
            "tool": "flake8",
            "severity": sev,
            "location": f"{path}:{ln}:{col}",
            "rule": code,
            "message": msg,
            "suggested_fix": "Apply Python style fix.",
        })
    return findings


def parse_vulture(text: str) -> list[dict]:
    """vulture: 'path/to/file.py:line: unused (function|variable|...) 'name' (XX% confidence)'."""
    findings: list[dict] = []
    for line in text.splitlines():
        m = re.match(r"^(.+?):(\d+):\s+unused\s+(\w+)\s+'([^']+)'\s+\((\d+)%\s+confidence\)$", line)
        if not m:
            continue
        path, ln, kind, name, conf = m.groups()
        findings.append({
            "tool": "vulture",
            "severity": "low",
            "location": f"{path}:{ln}",
            "rule": f"unused-{kind}",
            "message": f"Unused {kind} '{name}' (confidence {conf}%)",
            "suggested_fix": "Delete if truly unused, or add # noqa: vulture if intentionally unused.",
        })
    return findings


def parse_large_files(data) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {}
    if not isinstance(data, dict):
        return findings, metrics
    big = data.get("large_files", []) or []
    metrics["top_largest_source_files"] = big[:5]
    for f in big[:5]:
        size = f.get("size", 0)
        if size > 50_000:
            sev = "high" if size > 150_000 else "medium"
            findings.append({
                "tool": "large-file-scanner",
                "severity": sev,
                "location": f.get("path"),
                "rule": "oversized-file",
                "message": f"File is {size//1024} KB — likely needs splitting (god file / kitchen sink)",
                "suggested_fix": "Split by domain. Aim for < 500 LOC per module.",
            })
    return findings, metrics


def gather(raw_dir: Path, envelope: dict) -> list[dict]:
    all_findings: list[dict] = []
    metrics = envelope.setdefault("metrics", {})

    es = load_json(raw_dir / "eslint.json")
    if es is not None:
        f, m = parse_eslint(es)
        all_findings.extend(f)
        metrics.update(m)

    tsc_path = raw_dir / "tsc.txt"
    if tsc_path.exists() and tsc_path.stat().st_size > 0:
        all_findings.extend(parse_tsc(tsc_path.read_text(errors="replace")))

    tsp_path = raw_dir / "ts-prune.txt"
    if tsp_path.exists() and tsp_path.stat().st_size > 0:
        all_findings.extend(parse_ts_prune(tsp_path.read_text(errors="replace")))

    dc = load_json(raw_dir / "depcheck.json")
    if dc is not None:
        f, m = parse_depcheck(dc)
        all_findings.extend(f)
        metrics.update(m)

    md = load_json(raw_dir / "madge-circular.json")
    if md is not None:
        f, m = parse_madge(md)
        all_findings.extend(f)
        metrics.update(m)

    ruff = load_json(raw_dir / "ruff.json")
    if ruff is not None:
        all_findings.extend(parse_ruff(ruff))

    fl_path = raw_dir / "flake8.txt"
    if fl_path.exists() and fl_path.stat().st_size > 0:
        all_findings.extend(parse_flake8(fl_path.read_text(errors="replace")))

    vu_path = raw_dir / "vulture.txt"
    if vu_path.exists() and vu_path.stat().st_size > 0:
        all_findings.extend(parse_vulture(vu_path.read_text(errors="replace")))

    lf = load_json(raw_dir / "large-files.json")
    if lf is not None:
        f, m = parse_large_files(lf)
        all_findings.extend(f)
        metrics.update(m)

    # Cross-tool boost: a finding flagged by 2+ tools is high-confidence
    # (already handled by dedupe_findings via also_reported_by; bump severity here)
    seen: dict[str, list[dict]] = {}
    for f in all_findings:
        loc = f.get("location") or ""
        seen.setdefault(loc, []).append(f)
    for loc, group in seen.items():
        if len(group) >= 2 and loc:
            tools = {x.get("tool") for x in group}
            if len(tools) >= 2:
                # promote highest severity in the group
                for x in group:
                    x["cross_tool_confirmed"] = True

    return all_findings


if __name__ == "__main__":
    main_skeleton("code", gather)

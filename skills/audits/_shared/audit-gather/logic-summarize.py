#!/usr/bin/env python3
"""logic-summarize.py — Normalize cycles, complexity, churn, dead code."""
from __future__ import annotations
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from _summarize_common import load_json, main_skeleton  # noqa: E402


def parse_madge_circular(data) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {}
    cycles = data if isinstance(data, list) else []
    metrics["circular_cycle_count"] = len(cycles)
    for cycle in cycles[:25]:
        if not isinstance(cycle, list) or not cycle:
            continue
        chain = " -> ".join(cycle + [cycle[0]])
        sev = "high" if len(cycle) <= 3 else "medium"
        findings.append({
            "tool": "madge-circular",
            "severity": sev,
            "location": cycle[0],
            "rule": "circular-dep",
            "message": f"Cycle ({len(cycle)} nodes): {chain[:200]}",
            "suggested_fix": "Extract shared piece into a leaf module, or invert one edge.",
        })
    return findings, metrics


def parse_madge_summary(data) -> dict:
    metrics: dict = {}
    if isinstance(data, dict):
        metrics["module_count"] = len(data)
        # Top fan-in (modules with most importers)
        fan_in: dict[str, int] = {}
        for src, deps in data.items():
            for d in deps or []:
                fan_in[d] = fan_in.get(d, 0) + 1
        metrics["top_fan_in"] = sorted(
            ({"module": k, "imported_by": v} for k, v in fan_in.items()),
            key=lambda r: -r["imported_by"],
        )[:10]
        metrics["top_fan_out"] = sorted(
            ({"module": k, "imports": len(v or [])} for k, v in data.items()),
            key=lambda r: -r["imports"],
        )[:10]
    return metrics


def parse_dep_cruiser(data) -> list[dict]:
    findings: list[dict] = []
    if not isinstance(data, dict):
        return findings
    summary = data.get("summary", {}) or {}
    for v in summary.get("violations", []) or []:
        rule = v.get("rule", {}) or {}
        sev = rule.get("severity") or "medium"
        findings.append({
            "tool": "dep-cruiser",
            "severity": sev,
            "location": v.get("from"),
            "rule": rule.get("name") or "rule-violation",
            "message": v.get("comment") or rule.get("comment") or f"violates {rule.get('name')}",
            "suggested_fix": "Apply the dep-cruiser rule guidance.",
        })
    return findings


def parse_cloc(data) -> dict:
    metrics: dict = {}
    if not isinstance(data, dict):
        return metrics
    summary = data.get("SUM") or {}
    metrics["cloc_total_loc"] = summary.get("code", 0)
    metrics["cloc_comment_loc"] = summary.get("comment", 0)
    metrics["cloc_blank_loc"] = summary.get("blank", 0)
    if summary.get("code", 0) > 0:
        metrics["cloc_comment_ratio"] = round(summary.get("comment", 0) / summary["code"], 3)
    metrics["cloc_per_lang"] = {
        k: {"code": v.get("code", 0), "files": v.get("nFiles", 0)}
        for k, v in data.items() if k != "header" and k != "SUM" and isinstance(v, dict)
    }
    return metrics


def parse_complexity(data) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {}
    if not isinstance(data, dict):
        return findings, metrics
    files = data.get("files", []) or []
    metrics["complexity_files_scanned"] = len(files)
    # Compute synthetic complexity score: branches/functions + nesting + LOC
    scored = []
    for f in files:
        branches = f.get("branches", 0)
        functions = max(f.get("functions", 0), 1)
        nest = f.get("max_nest", 0)
        loc = f.get("loc", 0)
        score = (branches / functions) * 2 + nest + (loc / 100)
        scored.append({**f, "complexity_score": round(score, 1)})
    scored.sort(key=lambda r: -r["complexity_score"])
    metrics["top_complex_files"] = scored[:10]
    for f in scored[:15]:
        score = f["complexity_score"]
        if score < 25:
            continue
        sev = "high" if score >= 60 else "medium"
        findings.append({
            "tool": "complexity-heuristic",
            "severity": sev,
            "location": f.get("file"),
            "rule": "high-complexity",
            "message": (
                f"complexity score {score} (LOC={f.get('loc')}, "
                f"branches/fn={f.get('branches')}/{f.get('functions')}, nest={f.get('max_nest')})"
            ),
            "suggested_fix": "Extract sub-functions, flatten nesting, split file by domain.",
        })
    return findings, metrics


def parse_git_churn(data) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {}
    if not isinstance(data, dict):
        return findings, metrics
    top = data.get("top_churn", []) or []
    metrics["git_churn_top_files"] = top
    for f in top[:5]:
        c = f.get("commits_in_last_200", 0)
        if c >= 30:
            findings.append({
                "tool": "git-churn",
                "severity": "medium",
                "location": f.get("file"),
                "rule": "high-churn",
                "message": f"File touched {c} times in last 200 commits — instability hotspot",
                "suggested_fix": "Add tests; consider redesign if churn is bug-fix driven.",
            })
    return findings, metrics


def parse_todo(data) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {}
    if not isinstance(data, dict):
        return findings, metrics
    total = data.get("total_count", 0)
    metrics["todo_total"] = total
    metrics["todo_samples"] = data.get("samples", [])[:5]
    if total >= 100:
        findings.append({
            "tool": "todo-density",
            "severity": "medium",
            "location": None,
            "rule": "todo-debt",
            "message": f"{total} TODO/FIXME/HACK markers in code",
            "suggested_fix": "Triage: convert each to a tracked ticket or delete if obsolete.",
        })
    return findings, metrics


def gather(raw_dir: Path, envelope: dict) -> list[dict]:
    all_findings: list[dict] = []
    metrics = envelope.setdefault("metrics", {})

    mc = load_json(raw_dir / "madge-circular.json")
    if mc is not None:
        f, m = parse_madge_circular(mc)
        all_findings.extend(f)
        metrics.update(m)

    ms = load_json(raw_dir / "madge-summary.json")
    if ms is not None:
        metrics.update(parse_madge_summary(ms))

    dc = load_json(raw_dir / "dep-cruiser.json")
    if dc is not None:
        all_findings.extend(parse_dep_cruiser(dc))

    cl = load_json(raw_dir / "cloc.json")
    if cl is not None:
        metrics.update(parse_cloc(cl))

    cx = load_json(raw_dir / "complexity-heuristic.json")
    if cx is not None:
        f, m = parse_complexity(cx)
        all_findings.extend(f)
        metrics.update(m)

    gc = load_json(raw_dir / "git-churn.json")
    if gc is not None:
        f, m = parse_git_churn(gc)
        all_findings.extend(f)
        metrics.update(m)

    td = load_json(raw_dir / "todo-density.json")
    if td is not None:
        f, m = parse_todo(td)
        all_findings.extend(f)
        metrics.update(m)

    return all_findings


if __name__ == "__main__":
    main_skeleton("logic", gather)

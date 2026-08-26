#!/usr/bin/env python3
"""a11y-summarize.py — Normalize axe + pa11y + Lighthouse a11y outputs."""
from __future__ import annotations
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from _summarize_common import (  # noqa: E402
    load_json,
    main_skeleton,
    normalize_severity,
)


def parse_axe(data) -> list[dict]:
    """axe outputs an array of result-runs; each has .violations[]."""
    findings: list[dict] = []
    runs = data if isinstance(data, list) else [data]
    for run in runs:
        if not isinstance(run, dict):
            continue
        for v in run.get("violations", []) or []:
            sev = normalize_severity(v.get("impact"))
            wcag = ", ".join(v.get("tags", []) or [])
            for n in v.get("nodes", []) or []:
                target = ", ".join(n.get("target", []) or [])
                html_snippet = (n.get("html") or "")[:140]
                findings.append({
                    "tool": "axe-core",
                    "severity": sev,
                    "location": target or run.get("url") or None,
                    "rule": v.get("id"),
                    "message": v.get("help") or v.get("description") or "",
                    "suggested_fix": v.get("helpUrl"),
                    "wcag": wcag,
                    "html_snippet": html_snippet,
                })
    return findings


def parse_pa11y(data) -> list[dict]:
    """pa11y outputs a flat array of issues."""
    findings: list[dict] = []
    issues = data if isinstance(data, list) else (data.get("issues", []) if isinstance(data, dict) else [])
    for i in issues or []:
        sev = normalize_severity(i.get("type"))  # error/warning/notice
        findings.append({
            "tool": "pa11y",
            "severity": sev,
            "location": i.get("selector"),
            "rule": i.get("code"),
            "message": i.get("message"),
            "suggested_fix": None,
            "wcag": i.get("code"),
        })
    return findings


def parse_lighthouse(data) -> list[dict]:
    """Lighthouse audits[] entries with score < 1 or score == null + accessibility category."""
    findings: list[dict] = []
    if not isinstance(data, dict):
        return findings
    audits = data.get("audits", {}) or {}
    cats = data.get("categories", {}) or {}
    a11y_cat = cats.get("accessibility", {}) or {}
    refs = a11y_cat.get("auditRefs", []) or []
    for ref in refs:
        aid = ref.get("id")
        a = audits.get(aid)
        if not a:
            continue
        score = a.get("score")
        if score == 1 or a.get("scoreDisplayMode") in ("notApplicable", "informative"):
            continue
        # Lighthouse: score 0 = critical, < 0.5 = high, < 0.9 = medium
        if score is None:
            sev = "medium"
        elif score == 0:
            sev = "critical"
        elif score < 0.5:
            sev = "high"
        elif score < 0.9:
            sev = "medium"
        else:
            sev = "low"
        items = (a.get("details", {}) or {}).get("items", []) or []
        if items:
            for it in items[:5]:
                node = it.get("node", {}) or {}
                findings.append({
                    "tool": "lighthouse-a11y",
                    "severity": sev,
                    "location": node.get("selector") or node.get("path"),
                    "rule": aid,
                    "message": a.get("title"),
                    "suggested_fix": a.get("description"),
                })
        else:
            findings.append({
                "tool": "lighthouse-a11y",
                "severity": sev,
                "location": None,
                "rule": aid,
                "message": a.get("title"),
                "suggested_fix": a.get("description"),
            })
    return findings


def gather(raw_dir: Path, envelope: dict) -> list[dict]:
    findings: list[dict] = []
    metrics = envelope.setdefault("metrics", {})

    axe = load_json(raw_dir / "axe.json")
    if axe is not None:
        findings.extend(parse_axe(axe))
        metrics["axe_violation_runs"] = len(axe) if isinstance(axe, list) else 1

    pa11y = load_json(raw_dir / "pa11y.json")
    if pa11y is not None:
        findings.extend(parse_pa11y(pa11y))
        metrics["pa11y_issue_count"] = (
            len(pa11y) if isinstance(pa11y, list) else len(pa11y.get("issues", []) or [])
        )

    lh = load_json(raw_dir / "lighthouse-a11y.json")
    if lh is not None:
        findings.extend(parse_lighthouse(lh))
        cats = (lh.get("categories") or {}).get("accessibility") or {}
        metrics["lighthouse_a11y_score"] = cats.get("score")

    return findings


if __name__ == "__main__":
    main_skeleton("a11y", gather)

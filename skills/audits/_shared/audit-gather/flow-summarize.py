#!/usr/bin/env python3
"""flow-summarize.py — Normalize route inventory, link probes, form coverage."""
from __future__ import annotations
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from _summarize_common import load_json, main_skeleton  # noqa: E402


def parse_routes(data) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {}
    if not isinstance(data, dict):
        return findings, metrics
    routes = data.get("routes", []) or []
    by_kind: dict[str, int] = {}
    for r in routes:
        k = r.get("kind", "?")
        by_kind[k] = by_kind.get(k, 0) + 1
    metrics["route_count"] = len(routes)
    metrics["routes_by_kind"] = by_kind
    metrics["route_sample"] = [
        {"file": r.get("file"), "path": r.get("path"), "framework": r.get("framework"), "kind": r.get("kind")}
        for r in routes[:30]
    ]
    return findings, metrics


def parse_internal_links(data) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {}
    if not isinstance(data, dict):
        return findings, metrics
    metrics["internal_links_found"] = len(data.get("internal", []) or [])
    metrics["external_links_found"] = len(data.get("external", []) or [])
    metrics["internal_link_sample"] = (data.get("internal") or [])[:15]
    return findings, metrics


def parse_link_probes(data) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {}
    if not isinstance(data, list):
        return findings, metrics
    metrics["link_probe_count"] = len(data)
    broken = [p for p in data if (p.get("status", 0) or 0) >= 400 or p.get("status") == 0]
    metrics["broken_links_count"] = len(broken)
    for p in broken[:25]:
        sev = "critical" if p.get("status", 0) == 0 else (
              "high" if p.get("status", 0) >= 500 else "medium")
        findings.append({
            "tool": "link-prober",
            "severity": sev,
            "location": p.get("url"),
            "rule": "broken-link",
            "message": f"Link returned status {p.get('status')}",
            "suggested_fix": "Update link target or add a redirect.",
        })
    return findings, metrics


def parse_interaction(data) -> dict:
    metrics: dict = {}
    if not isinstance(data, dict):
        return metrics
    metrics["forms_count"] = len(data.get("forms_in_code", []) or [])
    metrics["forms_sample"] = (data.get("forms_in_code") or [])[:5]
    metrics["buttons_with_handlers"] = data.get("buttons_with_handlers", 0)
    return metrics


def parse_state_coverage(data) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {}
    if not isinstance(data, dict):
        return findings, metrics
    empty = data.get("empty_state_indicators", 0) or 0
    err_bd = data.get("error_boundary_files", 0) or 0
    metrics["empty_state_indicators"] = empty
    metrics["error_boundary_files"] = err_bd
    if err_bd == 0 and empty == 0:
        findings.append({
            "tool": "state-coverage", "severity": "medium",
            "location": None, "rule": "missing-error-boundary",
            "message": "No ErrorBoundary or empty-state indicators detected anywhere",
            "suggested_fix": "Wrap pages with ErrorBoundary; add empty-state UI for lists.",
        })
    elif err_bd == 0:
        findings.append({
            "tool": "state-coverage", "severity": "low",
            "location": None, "rule": "no-error-boundary",
            "message": "No React ErrorBoundary detected (empty states found, but no error UI)",
            "suggested_fix": "Add ErrorBoundary to wrap risky subtrees.",
        })
    return findings, metrics


def gather(raw_dir: Path, envelope: dict) -> list[dict]:
    all_findings: list[dict] = []
    metrics = envelope.setdefault("metrics", {})

    routes = load_json(raw_dir / "route-inventory.json")
    if routes is not None:
        f, m = parse_routes(routes)
        all_findings.extend(f)
        metrics.update(m)

    il = load_json(raw_dir / "internal-links.json")
    if il is not None:
        f, m = parse_internal_links(il)
        all_findings.extend(f)
        metrics.update(m)

    lp = load_json(raw_dir / "link-probes.json")
    if lp is not None:
        f, m = parse_link_probes(lp)
        all_findings.extend(f)
        metrics.update(m)

    inter = load_json(raw_dir / "interaction-inventory.json")
    if inter is not None:
        metrics.update(parse_interaction(inter))

    sc = load_json(raw_dir / "state-coverage.json")
    if sc is not None:
        f, m = parse_state_coverage(sc)
        all_findings.extend(f)
        metrics.update(m)

    return all_findings


if __name__ == "__main__":
    main_skeleton("flow", gather)

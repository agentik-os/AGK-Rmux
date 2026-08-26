#!/usr/bin/env python3
"""perf-summarize.py — Normalize Lighthouse perf, size-limit, build-sizes."""
from __future__ import annotations
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from _summarize_common import load_json, main_skeleton  # noqa: E402


THRESHOLDS_BYTES = {
    "first_load_kb_high": 300 * 1024,
    "first_load_kb_critical": 600 * 1024,
    "single_chunk_kb_high": 200 * 1024,
}


def parse_lighthouse_perf(data) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {}
    if not isinstance(data, dict):
        return findings, metrics

    cats = (data.get("categories") or {}).get("performance") or {}
    metrics["lighthouse_perf_score"] = cats.get("score")

    audits = data.get("audits", {}) or {}
    # Core Web Vitals + key perf metrics
    cwv_keys = {
        "largest-contentful-paint": "LCP",
        "first-contentful-paint": "FCP",
        "total-blocking-time": "TBT",
        "cumulative-layout-shift": "CLS",
        "speed-index": "SI",
        "interactive": "TTI",
        "max-potential-fid": "max-FID",
    }
    for key, label in cwv_keys.items():
        a = audits.get(key)
        if a:
            metrics[f"cwv_{label}"] = {
                "value": a.get("numericValue"),
                "displayValue": a.get("displayValue"),
                "score": a.get("score"),
            }

    # Diagnostic audits with score < 0.9 → finding
    diag_audits = [
        "render-blocking-resources",
        "unused-javascript",
        "unused-css-rules",
        "uses-optimized-images",
        "uses-text-compression",
        "uses-responsive-images",
        "efficient-animated-content",
        "duplicated-javascript",
        "legacy-javascript",
        "modern-image-formats",
        "uses-rel-preload",
        "uses-rel-preconnect",
        "preload-lcp-image",
        "third-party-summary",
        "bootup-time",
        "mainthread-work-breakdown",
        "dom-size",
        "long-tasks",
    ]
    for aid in diag_audits:
        a = audits.get(aid)
        if not a:
            continue
        score = a.get("score")
        if score == 1 or score is None:
            continue
        sev = "high" if score < 0.5 else "medium"
        items = (a.get("details", {}) or {}).get("items", []) or []
        savings = a.get("displayValue") or ""
        if items:
            for it in items[:3]:
                url = it.get("url") or it.get("source") or ""
                size_kb = (it.get("totalBytes") or it.get("wastedBytes") or 0) / 1024
                findings.append({
                    "tool": "lighthouse-perf",
                    "severity": sev,
                    "location": url[:160],
                    "rule": aid,
                    "message": f"{a.get('title')} ({savings}) — {size_kb:.1f} KB",
                    "suggested_fix": a.get("description"),
                })
        else:
            findings.append({
                "tool": "lighthouse-perf",
                "severity": sev,
                "location": None,
                "rule": aid,
                "message": f"{a.get('title')} ({savings})",
                "suggested_fix": a.get("description"),
            })
    return findings, metrics


def parse_size_limit(data) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {}
    if not isinstance(data, list):
        return findings, metrics
    metrics["size_limit_entries"] = []
    for entry in data:
        name = entry.get("name", "<unnamed>")
        size = entry.get("size") or 0
        passed = entry.get("passed", True)
        metrics["size_limit_entries"].append({
            "name": name,
            "size_bytes": size,
            "passed": passed,
        })
        if not passed:
            findings.append({
                "tool": "size-limit",
                "severity": "high",
                "location": name,
                "rule": "size-budget-exceeded",
                "message": f"{name}: {size} bytes exceeds budget",
                "suggested_fix": "Reduce bundle size: remove unused deps, code-split, lazy load",
            })
    return findings, metrics


def parse_build_sizes(data) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {}
    if not isinstance(data, dict):
        return findings, metrics
    metrics["build_dirs"] = data.get("directories", {})
    top = data.get("top_files", []) or []
    metrics["top_static_files"] = top[:10]
    # Flag any single static file > 200KB
    for f in top:
        size = f.get("size", 0)
        if size > THRESHOLDS_BYTES["single_chunk_kb_high"]:
            sev = "high" if size > 500 * 1024 else "medium"
            findings.append({
                "tool": "build-size-analyzer",
                "severity": sev,
                "location": f.get("path"),
                "rule": "large-static-asset",
                "message": f"Static asset {size/1024:.0f} KB (threshold {THRESHOLDS_BYTES['single_chunk_kb_high']/1024:.0f} KB)",
                "suggested_fix": "Code-split, lazy-load, or move to CDN. Inspect with bundle-analyzer.",
            })
    return findings, metrics


def parse_node_modules(data) -> dict:
    metrics: dict = {}
    if not isinstance(data, dict):
        return metrics
    metrics["node_modules_bytes"] = data.get("bytes", 0)
    metrics["node_modules_pkg_count"] = data.get("package_dirs", 0)
    return metrics


def gather(raw_dir: Path, envelope: dict) -> list[dict]:
    all_findings: list[dict] = []
    metrics = envelope.setdefault("metrics", {})

    lh = load_json(raw_dir / "lighthouse-perf.json")
    if lh is not None:
        f, m = parse_lighthouse_perf(lh)
        all_findings.extend(f)
        metrics.update(m)

    sl = load_json(raw_dir / "size-limit.json")
    if sl is not None:
        f, m = parse_size_limit(sl)
        all_findings.extend(f)
        metrics.update(m)

    bs = load_json(raw_dir / "build-sizes.json")
    if bs is not None:
        f, m = parse_build_sizes(bs)
        all_findings.extend(f)
        metrics.update(m)

    nm = load_json(raw_dir / "node-modules.json")
    if nm is not None:
        metrics.update(parse_node_modules(nm))

    return all_findings


if __name__ == "__main__":
    main_skeleton("perf", gather)

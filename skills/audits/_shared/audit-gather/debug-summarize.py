#!/usr/bin/env python3
"""debug-summarize.py — Normalize Playwright runtime captures + curl head."""
from __future__ import annotations
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from _summarize_common import load_json, main_skeleton  # noqa: E402


def parse_console(data) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {"console_total": 0, "console_by_type": {}}
    if not isinstance(data, list):
        return findings, metrics
    metrics["console_total"] = len(data)
    by_type: dict[str, int] = {}
    for ev in data:
        t = ev.get("type", "log")
        by_type[t] = by_type.get(t, 0) + 1
        if t in ("error", "warning"):
            sev = "high" if t == "error" else "medium"
            loc = ev.get("location") or {}
            url = loc.get("url") or ""
            line = loc.get("lineNumber", 0)
            findings.append({
                "tool": "playwright-console",
                "severity": sev,
                "location": f"{url}:{line}" if url else None,
                "rule": f"console-{t}",
                "message": (ev.get("text") or "")[:300],
                "suggested_fix": "Trace the source, fix the underlying issue, ensure error boundary catches it.",
            })
    metrics["console_by_type"] = by_type
    return findings, metrics


def parse_page_errors(data) -> list[dict]:
    findings: list[dict] = []
    if not isinstance(data, list):
        return findings
    for err in data:
        msg = err.get("message", "")[:300]
        stack = err.get("stack", "")
        # Try to extract first useful frame
        loc = None
        m = re.search(r"\bat\s+\S+\s*\(([^)]+:\d+:\d+)\)", stack)
        if m:
            loc = m.group(1)
        findings.append({
            "tool": "playwright-pageerror",
            "severity": "critical",
            "location": loc,
            "rule": "uncaught-page-error",
            "message": msg,
            "suggested_fix": "Wrap with try/catch, add error boundary, verify nullable types.",
        })
    return findings


def parse_network_failures(data) -> list[dict]:
    findings: list[dict] = []
    if not isinstance(data, list):
        return findings
    for n in data:
        url = n.get("url", "")
        rt = n.get("resource_type", "")
        sev = "high"
        if rt in ("image", "font", "media") and "favicon" not in url:
            sev = "medium"
        elif rt == "xhr" or rt == "fetch":
            sev = "high"
        findings.append({
            "tool": "playwright-network",
            "severity": sev,
            "location": url,
            "rule": "network-request-failed",
            "message": f"[{n.get('method', '?')}] {rt} failed: {n.get('failure', 'unknown')}",
            "suggested_fix": "Verify URL is reachable, check CORS, retry policy, fallback.",
        })
    return findings


def parse_network_errors(data) -> list[dict]:
    findings: list[dict] = []
    if not isinstance(data, list):
        return findings
    for n in data:
        status = n.get("status", 0)
        sev = "critical" if status >= 500 else "high"
        findings.append({
            "tool": "playwright-http",
            "severity": sev,
            "location": n.get("url", ""),
            "rule": f"http-{status}",
            "message": f"[{n.get('method', '?')}] returned {status}",
            "suggested_fix": "Check server-side route + auth state. 5xx → server bug. 4xx → contract / auth.",
        })
    return findings


def parse_page_meta(data) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {}
    if not isinstance(data, dict):
        return findings, metrics
    metrics["page_meta"] = {
        "title": data.get("title"),
        "final_url": data.get("url_final"),
        "html_size": data.get("html_size"),
        "nav_error": data.get("nav_error"),
    }
    if data.get("nav_error"):
        findings.append({
            "tool": "playwright-nav",
            "severity": "critical",
            "location": data.get("url_final"),
            "rule": "navigation-error",
            "message": data.get("nav_error"),
            "suggested_fix": "Fix the page so it loads within 45s.",
        })
    if not data.get("title"):
        findings.append({
            "tool": "playwright-nav",
            "severity": "medium",
            "location": data.get("url_final"),
            "rule": "no-page-title",
            "message": "Page loaded with empty title",
            "suggested_fix": "Set <title> server-side or via document.title.",
        })
    return findings, metrics


def parse_curl_head(text: str) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {}
    if not text:
        return findings, metrics
    code_m = re.search(r"^HTTP_CODE:(\d+)", text, re.M)
    final_m = re.search(r"^FINAL_URL:(.+)$", text, re.M)
    time_m = re.search(r"^TIME:([\d.]+)", text, re.M)
    metrics["curl_head"] = {
        "http_code": int(code_m.group(1)) if code_m else None,
        "final_url": final_m.group(1).strip() if final_m else None,
        "time_sec": float(time_m.group(1)) if time_m else None,
    }
    if code_m and int(code_m.group(1)) >= 500:
        findings.append({
            "tool": "curl-head", "severity": "critical",
            "location": final_m.group(1).strip() if final_m else None,
            "rule": "server-error",
            "message": f"Server returned {code_m.group(1)}",
            "suggested_fix": "Inspect server logs for the failing route.",
        })
    return findings, metrics


def gather(raw_dir: Path, envelope: dict) -> list[dict]:
    all_findings: list[dict] = []
    metrics = envelope.setdefault("metrics", {})

    cn = load_json(raw_dir / "console.json")
    if cn is not None:
        f, m = parse_console(cn)
        all_findings.extend(f)
        metrics.update(m)

    pe = load_json(raw_dir / "page-errors.json")
    if pe is not None:
        all_findings.extend(parse_page_errors(pe))

    nf = load_json(raw_dir / "network-failures.json")
    if nf is not None:
        all_findings.extend(parse_network_failures(nf))

    ne = load_json(raw_dir / "network-4xx-5xx.json")
    if ne is not None:
        all_findings.extend(parse_network_errors(ne))

    pm = load_json(raw_dir / "page-meta.json")
    if pm is not None:
        f, m = parse_page_meta(pm)
        all_findings.extend(f)
        metrics.update(m)

    ch_path = raw_dir / "curl-head.txt"
    if ch_path.exists() and ch_path.stat().st_size > 0:
        f, m = parse_curl_head(ch_path.read_text(errors="replace"))
        all_findings.extend(f)
        metrics.update(m)

    # Note screenshot availability
    metrics["screenshot_available"] = (raw_dir / "screenshot.png").exists()

    return all_findings


if __name__ == "__main__":
    main_skeleton("debug", gather)

#!/usr/bin/env python3
"""api-summarize.py — Normalize OpenAPI, route discovery, endpoint probes."""
from __future__ import annotations
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from _summarize_common import load_json, main_skeleton  # noqa: E402


def parse_openapi_validation(data) -> list[dict]:
    findings: list[dict] = []
    if not isinstance(data, list):
        return findings
    for item in data:
        if isinstance(item, dict) and item.get("error"):
            findings.append({
                "tool": "swagger-cli",
                "severity": "high",
                "location": "openapi.json",
                "rule": "openapi-invalid",
                "message": str(item.get("error"))[:300],
                "suggested_fix": "Fix the OpenAPI spec syntax/schema.",
            })
    return findings


def parse_openapi_spec(data) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {}
    if not isinstance(data, dict):
        return findings, metrics
    paths = data.get("paths", {}) or {}
    metrics["openapi_path_count"] = len(paths)
    metrics["openapi_op_count"] = sum(
        sum(1 for k in (ops or {}) if k.lower() in ("get", "post", "put", "patch", "delete"))
        for ops in paths.values() if isinstance(ops, dict)
    )
    # Audit each operation for missing pieces
    for path, ops in paths.items():
        if not isinstance(ops, dict):
            continue
        for method, op in ops.items():
            if method.lower() not in ("get", "post", "put", "patch", "delete"):
                continue
            if not isinstance(op, dict):
                continue
            loc = f"openapi:{method.upper()} {path}"
            if not op.get("summary") and not op.get("description"):
                findings.append({
                    "tool": "openapi-auditor", "severity": "low", "location": loc,
                    "rule": "missing-doc", "message": "operation has no summary or description",
                    "suggested_fix": "Add summary + description to the operation.",
                })
            responses = op.get("responses", {}) or {}
            if not any(c for c in responses if str(c).startswith(("2", "3"))):
                findings.append({
                    "tool": "openapi-auditor", "severity": "high", "location": loc,
                    "rule": "no-success-response", "message": "no 2xx response defined",
                    "suggested_fix": "Document at least one 2xx response with schema.",
                })
            if "4" not in "".join(str(c)[:1] for c in responses) and method.lower() in ("post", "put", "patch", "delete"):
                findings.append({
                    "tool": "openapi-auditor", "severity": "medium", "location": loc,
                    "rule": "no-error-response", "message": "no 4xx response defined for mutation",
                    "suggested_fix": "Document 400 / 401 / 404 responses with error schema.",
                })
    return findings, metrics


def parse_routes(data) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {}
    if not isinstance(data, dict):
        return findings, metrics
    patterns = data.get("patterns", []) or []
    metrics["route_count"] = len(patterns)
    by_method: dict[str, int] = {}
    by_framework: dict[str, int] = {}
    for r in patterns:
        m = r.get("method", "?")
        f = r.get("framework", "?")
        by_method[m] = by_method.get(m, 0) + 1
        by_framework[f] = by_framework.get(f, 0) + 1
    metrics["routes_by_method"] = by_method
    metrics["routes_by_framework"] = by_framework
    metrics["route_sample"] = patterns[:25]
    return findings, metrics


def parse_endpoint_probes(data) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {}
    if not isinstance(data, dict):
        return findings, metrics
    probes = data.get("probes", []) or []
    metrics["endpoint_probes"] = probes
    for p in probes:
        status = p.get("status", 0)
        path = p.get("path")
        time_sec = p.get("time_sec", 0)
        # Only flag genuinely concerning responses for canonical paths
        if path == "/" and status == 0:
            findings.append({
                "tool": "endpoint-probe", "severity": "critical",
                "location": f"{data.get('root')}{path}",
                "rule": "root-unreachable", "message": "Root URL did not respond",
                "suggested_fix": "Verify the deployment is live.",
            })
        elif path in ("/health", "/healthz", "/api/health") and status >= 400:
            findings.append({
                "tool": "endpoint-probe", "severity": "high",
                "location": f"{data.get('root')}{path}",
                "rule": "health-endpoint-error",
                "message": f"Health probe at {path} returned {status}",
                "suggested_fix": "Implement / fix the health endpoint.",
            })
        if status and status >= 200 and status < 400 and time_sec > 3.0:
            findings.append({
                "tool": "endpoint-probe", "severity": "medium",
                "location": f"{data.get('root')}{path}",
                "rule": "slow-response",
                "message": f"Endpoint took {time_sec:.2f}s (>3s threshold)",
                "suggested_fix": "Investigate slow query / cold start / SSR overhead.",
            })
    return findings, metrics


def parse_auth_imports(data) -> dict:
    metrics: dict = {}
    if isinstance(data, dict):
        metrics["auth_imports"] = data.get("auth_imports", [])
        metrics["auth_imports_count"] = len(data.get("auth_imports", []) or [])
    return metrics


def gather(raw_dir: Path, envelope: dict) -> list[dict]:
    all_findings: list[dict] = []
    metrics = envelope.setdefault("metrics", {})

    val = load_json(raw_dir / "openapi-validation.json")
    if val is not None:
        all_findings.extend(parse_openapi_validation(val))

    spec = load_json(raw_dir / "openapi-spec.json")
    if spec is not None:
        f, m = parse_openapi_spec(spec)
        all_findings.extend(f)
        metrics.update(m)

    routes = load_json(raw_dir / "routes-discovered.json")
    if routes is not None:
        f, m = parse_routes(routes)
        all_findings.extend(f)
        metrics.update(m)

    probes = load_json(raw_dir / "endpoint-probes.json")
    if probes is not None:
        f, m = parse_endpoint_probes(probes)
        all_findings.extend(f)
        metrics.update(m)

    auth = load_json(raw_dir / "auth-imports.json")
    if auth is not None:
        metrics.update(parse_auth_imports(auth))

    return all_findings


if __name__ == "__main__":
    main_skeleton("api", gather)

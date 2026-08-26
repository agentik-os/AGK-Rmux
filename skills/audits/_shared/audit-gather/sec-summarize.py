#!/usr/bin/env python3
"""sec-summarize.py — Normalize npm audit, gitleaks, semgrep, pip-audit, headers."""
from __future__ import annotations
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from _summarize_common import (  # noqa: E402
    load_json,
    main_skeleton,
    normalize_severity,
)


def parse_npm_audit(data) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {}
    if not isinstance(data, dict):
        return findings, metrics
    meta = data.get("metadata", {}).get("vulnerabilities", {}) or {}
    metrics["npm_audit"] = {
        "info": meta.get("info", 0),
        "low": meta.get("low", 0),
        "moderate": meta.get("moderate", 0),
        "high": meta.get("high", 0),
        "critical": meta.get("critical", 0),
        "total": meta.get("total", 0),
    }
    vulns = data.get("vulnerabilities", {}) or {}
    for pkg, v in vulns.items():
        sev = normalize_severity(v.get("severity"))
        via = v.get("via", [])
        if isinstance(via, list) and via:
            messages = []
            for w in via:
                if isinstance(w, dict):
                    messages.append(w.get("title") or w.get("source") or pkg)
                else:
                    messages.append(str(w))
            msg = "; ".join(messages[:3])
        else:
            msg = pkg
        findings.append({
            "tool": "npm-audit",
            "severity": sev,
            "location": f"package: {pkg}",
            "rule": "vuln",
            "message": f"{pkg}: {msg}",
            "suggested_fix": v.get("fixAvailable") if isinstance(v.get("fixAvailable"), str) else (
                "npm audit fix" if v.get("fixAvailable") else "Manual upgrade required"),
            "range": v.get("range"),
        })
    return findings, metrics


def parse_pip_audit(data) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {"pip_audit_count": 0}
    deps = data.get("dependencies", []) if isinstance(data, dict) else (data if isinstance(data, list) else [])
    for d in deps or []:
        name = d.get("name") or d.get("package")
        version = d.get("version")
        for v in d.get("vulns", []) or []:
            sev = normalize_severity(v.get("severity") or "high")
            metrics["pip_audit_count"] += 1
            findings.append({
                "tool": "pip-audit",
                "severity": sev,
                "location": f"{name}=={version}",
                "rule": v.get("id"),
                "message": (v.get("description") or v.get("id") or "Python dependency vulnerability")[:200],
                "suggested_fix": f"Upgrade to {', '.join(v.get('fix_versions', []) or ['latest'])}",
            })
    return findings, metrics


def parse_gitleaks(data) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {}
    if not isinstance(data, list):
        return findings, metrics
    metrics["gitleaks_finding_count"] = len(data)
    for leak in data:
        findings.append({
            "tool": "gitleaks",
            "severity": "critical",
            "location": f"{leak.get('File')}:{leak.get('StartLine')}",
            "rule": leak.get("RuleID"),
            "message": leak.get("Description") or "Secret detected",
            "suggested_fix": "Rotate the credential immediately. Remove from git history (BFG / git filter-repo).",
            "commit": leak.get("Commit"),
            "match_preview": (leak.get("Secret") or "")[:20] + "…",
        })
    return findings, metrics


def parse_semgrep(data) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {}
    results = data.get("results", []) if isinstance(data, dict) else []
    metrics["semgrep_finding_count"] = len(results)
    for r in results:
        sev = normalize_severity(r.get("extra", {}).get("severity"))
        path = r.get("path", "")
        line = r.get("start", {}).get("line", 0)
        findings.append({
            "tool": "semgrep",
            "severity": sev,
            "location": f"{path}:{line}",
            "rule": r.get("check_id"),
            "message": r.get("extra", {}).get("message") or "",
            "suggested_fix": r.get("extra", {}).get("fix") or
                              (r.get("extra", {}).get("metadata", {}) or {}).get("fix"),
        })
    return findings, metrics


def parse_eslint_sec(data) -> list[dict]:
    findings: list[dict] = []
    if not isinstance(data, list):
        return findings
    for file_result in data:
        path = file_result.get("filePath", "")
        for msg in file_result.get("messages", []) or []:
            rule = msg.get("ruleId") or ""
            # Filter to security-flavored rules
            if not any(s in rule.lower() for s in ("security", "xss", "no-eval", "no-implied-eval",
                                                    "no-script-url", "no-new-func", "no-prototype-builtins")):
                continue
            sev = normalize_severity(msg.get("severity"))
            findings.append({
                "tool": "eslint-security",
                "severity": sev,
                "location": f"{path}:{msg.get('line', 0)}",
                "rule": rule,
                "message": msg.get("message", ""),
                "suggested_fix": "Refactor to avoid unsafe construct.",
            })
    return findings


def parse_env_files(data) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {}
    if not isinstance(data, dict):
        return findings, metrics
    files = data.get("env_files", []) or []
    metrics["env_files_total"] = len(files)
    metrics["env_files_in_git"] = sum(1 for f in files if f.get("tracked_by_git"))
    for f in files:
        if f.get("tracked_by_git"):
            findings.append({
                "tool": "env-file-scanner",
                "severity": "critical",
                "location": f.get("path"),
                "rule": "env-tracked-by-git",
                "message": f"Env/secrets file is tracked by git: {f.get('path')}",
                "suggested_fix": "git rm --cached + add to .gitignore + rotate any leaked secrets.",
            })
    return findings, metrics


def parse_http_headers(text: str) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {"security_headers_present": [], "security_headers_missing": []}
    required = {
        "strict-transport-security": "high",
        "content-security-policy": "high",
        "x-content-type-options": "medium",
        "x-frame-options": "medium",
        "referrer-policy": "low",
        "permissions-policy": "low",
    }
    headers_lower = {l.split(":", 1)[0].strip().lower(): l.split(":", 1)[1].strip()
                     for l in text.splitlines() if ":" in l and not l.startswith("HTTP/")}
    for h, sev in required.items():
        if h in headers_lower:
            metrics["security_headers_present"].append(h)
        else:
            metrics["security_headers_missing"].append(h)
            findings.append({
                "tool": "http-headers",
                "severity": sev,
                "location": "HTTP response",
                "rule": f"missing-{h}",
                "message": f"Missing security header: {h}",
                "suggested_fix": f"Add {h} to your reverse-proxy / framework config.",
            })
    return findings, metrics


def gather(raw_dir: Path, envelope: dict) -> list[dict]:
    all_findings: list[dict] = []
    metrics = envelope.setdefault("metrics", {})

    npm = load_json(raw_dir / "npm-audit.json")
    if npm is not None:
        f, m = parse_npm_audit(npm)
        all_findings.extend(f)
        metrics.update(m)

    pa = load_json(raw_dir / "pip-audit.json")
    if pa is not None:
        f, m = parse_pip_audit(pa)
        all_findings.extend(f)
        metrics.update(m)

    gl = load_json(raw_dir / "gitleaks.json")
    if gl is not None:
        f, m = parse_gitleaks(gl)
        all_findings.extend(f)
        metrics.update(m)

    sg = load_json(raw_dir / "semgrep.json")
    if sg is not None:
        f, m = parse_semgrep(sg)
        all_findings.extend(f)
        metrics.update(m)

    es = load_json(raw_dir / "eslint-sec.json")
    if es is not None:
        all_findings.extend(parse_eslint_sec(es))

    ef = load_json(raw_dir / "env-files.json")
    if ef is not None:
        f, m = parse_env_files(ef)
        all_findings.extend(f)
        metrics.update(m)

    hdr_path = raw_dir / "http-headers.txt"
    if hdr_path.exists() and hdr_path.stat().st_size > 0:
        try:
            text = hdr_path.read_text(errors="replace")
            f, m = parse_http_headers(text)
            all_findings.extend(f)
            metrics.update(m)
        except OSError:
            pass

    return all_findings


if __name__ == "__main__":
    main_skeleton("sec", gather)

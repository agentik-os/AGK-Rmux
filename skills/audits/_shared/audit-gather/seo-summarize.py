#!/usr/bin/env python3
"""seo-summarize.py — Normalize Lighthouse SEO + page-meta + sitemap data."""
from __future__ import annotations
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from _summarize_common import load_json, main_skeleton  # noqa: E402


def parse_lighthouse_seo(data) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {}
    if not isinstance(data, dict):
        return findings, metrics
    cats = (data.get("categories") or {}).get("seo") or {}
    metrics["lighthouse_seo_score"] = cats.get("score")
    audits = data.get("audits", {}) or {}
    refs = cats.get("auditRefs", []) or []
    for ref in refs:
        aid = ref.get("id")
        a = audits.get(aid)
        if not a:
            continue
        score = a.get("score")
        if score == 1 or a.get("scoreDisplayMode") in ("notApplicable", "informative"):
            continue
        sev = "high" if score == 0 else "medium"
        findings.append({
            "tool": "lighthouse-seo",
            "severity": sev,
            "location": None,
            "rule": aid,
            "message": a.get("title"),
            "suggested_fix": a.get("description"),
        })
    return findings, metrics


def parse_page_meta(data) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    metrics: dict = {}
    if not isinstance(data, dict):
        return findings, metrics
    metrics["page_meta"] = {
        "title_present": bool(data.get("title")),
        "title_length": len(data.get("title") or ""),
        "description_present": bool(data.get("meta_description")),
        "description_length": len(data.get("meta_description") or ""),
        "canonical": data.get("canonical"),
        "viewport": data.get("viewport"),
        "robots": data.get("robots"),
        "og_keys": list((data.get("og") or {}).keys()),
        "twitter_keys": list((data.get("twitter") or {}).keys()),
        "schema_count": len(data.get("schema_blocks") or []),
        "h1_count": data.get("h1_count"),
        "img_no_alt": data.get("img_no_alt"),
        "html_size_bytes": data.get("html_size_bytes"),
    }
    url = data.get("url")
    # Findings for missing essentials
    if not data.get("title"):
        findings.append({"tool": "head-parser", "severity": "critical", "location": url,
                         "rule": "missing-title", "message": "<title> missing", "suggested_fix": "Add <title> in <head>"})
    elif not (10 <= len(data.get("title") or "") <= 70):
        findings.append({"tool": "head-parser", "severity": "medium", "location": url,
                         "rule": "title-length", "message": f"<title> length {len(data['title'])} (recommended 10-70)",
                         "suggested_fix": "Adjust title to 50-60 chars."})
    if not data.get("meta_description"):
        findings.append({"tool": "head-parser", "severity": "high", "location": url,
                         "rule": "missing-description", "message": "meta description missing",
                         "suggested_fix": "Add <meta name=\"description\" content=\"…\"> 120-160 chars."})
    elif not (50 <= len(data.get("meta_description") or "") <= 200):
        findings.append({"tool": "head-parser", "severity": "low", "location": url,
                         "rule": "description-length",
                         "message": f"meta description length {len(data['meta_description'])} (recommended 120-160)",
                         "suggested_fix": "Adjust to 120-160 chars."})
    if not data.get("canonical"):
        findings.append({"tool": "head-parser", "severity": "medium", "location": url,
                         "rule": "missing-canonical", "message": "<link rel=canonical> missing",
                         "suggested_fix": "Add canonical URL."})
    if not data.get("viewport"):
        findings.append({"tool": "head-parser", "severity": "high", "location": url,
                         "rule": "missing-viewport", "message": "<meta viewport> missing",
                         "suggested_fix": "Add <meta name=viewport content=\"width=device-width, initial-scale=1\">"})
    if not (data.get("og") or {}):
        findings.append({"tool": "head-parser", "severity": "medium", "location": url,
                         "rule": "missing-opengraph", "message": "no Open Graph meta tags",
                         "suggested_fix": "Add og:title, og:description, og:image, og:url"})
    if not (data.get("twitter") or {}):
        findings.append({"tool": "head-parser", "severity": "low", "location": url,
                         "rule": "missing-twitter-card", "message": "no Twitter Card meta",
                         "suggested_fix": "Add twitter:card, twitter:title, twitter:description"})
    h1c = data.get("h1_count") or 0
    if h1c == 0:
        findings.append({"tool": "head-parser", "severity": "high", "location": url,
                         "rule": "missing-h1", "message": "no <h1> on page",
                         "suggested_fix": "Add a single <h1> at the top of the main content."})
    elif h1c > 1:
        findings.append({"tool": "head-parser", "severity": "medium", "location": url,
                         "rule": "multiple-h1", "message": f"{h1c} <h1> tags",
                         "suggested_fix": "Use a single <h1> per page."})
    img_alt_missing = data.get("img_no_alt") or 0
    if img_alt_missing > 0:
        sev = "medium" if img_alt_missing < 5 else "high"
        findings.append({"tool": "head-parser", "severity": sev, "location": url,
                         "rule": "img-no-alt", "message": f"{img_alt_missing} <img> tags without alt",
                         "suggested_fix": "Add alt text to all images for SEO + a11y."})
    if not (data.get("schema_blocks") or []):
        findings.append({"tool": "head-parser", "severity": "low", "location": url,
                         "rule": "no-structured-data", "message": "no JSON-LD schema blocks",
                         "suggested_fix": "Add structured data (Organization, Article, Product, etc.)"})
    return findings, metrics


def parse_sitemap_stats(data) -> dict:
    metrics: dict = {}
    if isinstance(data, dict):
        metrics["sitemap_url_count"] = data.get("url_count", 0)
        metrics["sitemap_sample_urls"] = data.get("sample_urls", [])[:5]
    return metrics


def gather(raw_dir: Path, envelope: dict) -> list[dict]:
    all_findings: list[dict] = []
    metrics = envelope.setdefault("metrics", {})

    lh = load_json(raw_dir / "lighthouse-seo.json")
    if lh is not None:
        f, m = parse_lighthouse_seo(lh)
        all_findings.extend(f)
        metrics.update(m)

    pm = load_json(raw_dir / "page-meta.json")
    if pm is not None:
        f, m = parse_page_meta(pm)
        all_findings.extend(f)
        metrics.update(m)

    sm = load_json(raw_dir / "sitemap-stats.json")
    if sm is not None:
        metrics.update(parse_sitemap_stats(sm))

    # robots.txt presence
    rob = raw_dir / "robots.txt"
    metrics["robots_txt_present"] = rob.exists() and rob.stat().st_size > 0

    return all_findings


if __name__ == "__main__":
    main_skeleton("seo", gather)

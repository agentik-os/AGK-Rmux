#!/usr/bin/env bash
# seo.sh — SEO gather. Lighthouse SEO category, head/meta inspection, sitemap fetch.
# Args: RAW_DIR PROJECT_PATH FILES URL
set -uo pipefail

RAW_DIR="${1:?missing raw_dir}"
PROJECT_PATH="${2:?missing project_path}"
FILES="${3:-}"
URL="${4:-}"
mkdir -p "$RAW_DIR"

# shellcheck disable=SC1091
source "$HOME/.omega/lib/audit-gather/_common.sh"

gather_header "seo"

if [ -z "$URL" ]; then
    log_skip "lighthouse-seo" "URL empty"
    log_skip "head-fetch" "URL empty"
    log_skip "sitemap-fetch" "URL empty"
    log_skip "robots-fetch" "URL empty"
    echo "seo: no URL — cannot run runtime tests"
    exit 0
fi

# ─── 1. Lighthouse SEO ──────────────────────────────────────────
if command -v lighthouse >/dev/null 2>&1 || check_npx_pkg "lighthouse"; then
    LH_BIN="$(command -v lighthouse || echo "npx --yes lighthouse")"
    safe_cmd "$RAW_DIR/lighthouse-seo.json" "lighthouse-seo" \
        $LH_BIN "$URL" \
            --output=json \
            --output-path="$RAW_DIR/lighthouse-seo.json" \
            --only-categories=seo \
            --chrome-flags="--headless --no-sandbox --disable-dev-shm-usage" \
            --quiet
fi

# ─── 2. Head/HTML fetch (canonical, og, twitter, schema) ────────
HEAD_HTML="$RAW_DIR/page.html"
curl -sLA "Mozilla/5.0 (compatible; AisbAuditBot/1.0)" --max-time 30 "$URL" -o "$HEAD_HTML" 2>>"$RAW_DIR/_stderr.log" || true
if [ -s "$HEAD_HTML" ]; then
    log_run "head-fetch" "page.html" 0
    # Extract metadata via Python
    python3 - "$URL" "$HEAD_HTML" > "$RAW_DIR/page-meta.json" 2>>"$RAW_DIR/_stderr.log" <<'PY'
import re, sys, json
url, path = sys.argv[1], sys.argv[2]
with open(path, "r", errors="replace") as f:
    html = f.read()
def find_one(pat, default=None, flags=re.I|re.S):
    m = re.search(pat, html, flags)
    return m.group(1).strip() if m else default
out = {
    "url": url,
    "title": find_one(r"<title>(.*?)</title>"),
    "meta_description": find_one(r'<meta\s+name=["\']description["\']\s+content=["\'](.*?)["\']'),
    "canonical": find_one(r'<link\s+rel=["\']canonical["\']\s+href=["\'](.*?)["\']'),
    "viewport": find_one(r'<meta\s+name=["\']viewport["\']\s+content=["\'](.*?)["\']'),
    "robots": find_one(r'<meta\s+name=["\']robots["\']\s+content=["\'](.*?)["\']'),
    "og": {},
    "twitter": {},
    "schema_blocks": [],
    "h1_count": len(re.findall(r"<h1[\s>]", html, re.I)),
    "img_no_alt": len(re.findall(r'<img(?![^>]*\balt=)', html, re.I)),
    "links_total": len(re.findall(r"<a[\s>]", html, re.I)),
    "html_size_bytes": len(html.encode("utf-8", errors="ignore")),
}
for m in re.finditer(r'<meta\s+property=["\']og:([^"\']+)["\']\s+content=["\'](.*?)["\']', html, re.I|re.S):
    out["og"][m.group(1)] = m.group(2).strip()[:300]
for m in re.finditer(r'<meta\s+name=["\']twitter:([^"\']+)["\']\s+content=["\'](.*?)["\']', html, re.I|re.S):
    out["twitter"][m.group(1)] = m.group(2).strip()[:300]
for m in re.finditer(r'<script[^>]*type=["\']application/ld\+json["\'][^>]*>(.*?)</script>', html, re.I|re.S):
    raw = m.group(1).strip()
    try:
        out["schema_blocks"].append(json.loads(raw))
    except Exception:
        out["schema_blocks"].append({"_unparseable": raw[:300]})
print(json.dumps(out, indent=2))
PY
    log_run "page-meta-parser" "page-meta.json" 0
fi

# ─── 3. robots.txt ──────────────────────────────────────────────
ROOT="$(echo "$URL" | sed -E 's#(https?://[^/]+).*#\1#')"
curl -sL --max-time 15 "$ROOT/robots.txt" -o "$RAW_DIR/robots.txt" 2>>"$RAW_DIR/_stderr.log" || true
[ -s "$RAW_DIR/robots.txt" ] && log_run "robots-fetch" "robots.txt" 0 || log_skip "robots-fetch" "no robots.txt"

# ─── 4. sitemap.xml ─────────────────────────────────────────────
curl -sL --max-time 30 "$ROOT/sitemap.xml" -o "$RAW_DIR/sitemap.xml" 2>>"$RAW_DIR/_stderr.log" || true
if [ -s "$RAW_DIR/sitemap.xml" ]; then
    log_run "sitemap-fetch" "sitemap.xml" 0
    python3 - "$RAW_DIR/sitemap.xml" > "$RAW_DIR/sitemap-stats.json" <<'PY'
import sys, re, json
with open(sys.argv[1], "r", errors="replace") as f:
    xml = f.read()
urls = re.findall(r"<loc>(.*?)</loc>", xml, re.I)
out = {"url_count": len(urls), "sample_urls": urls[:25]}
print(json.dumps(out, indent=2))
PY
    log_run "sitemap-parser" "sitemap-stats.json" 0
else
    log_skip "sitemap-fetch" "no sitemap.xml"
fi

echo "seo gather done"

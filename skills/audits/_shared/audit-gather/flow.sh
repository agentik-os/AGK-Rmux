#!/usr/bin/env bash
# flow.sh — user-flow gather. Broken-link checker, sitemap crawl, route inventory.
# Args: RAW_DIR PROJECT_PATH FILES URL
set -uo pipefail

RAW_DIR="${1:?missing raw_dir}"
PROJECT_PATH="${2:?missing project_path}"
FILES="${3:-}"
URL="${4:-}"
mkdir -p "$RAW_DIR"

# shellcheck disable=SC1091
source "$HOME/.omega/lib/audit-gather/_common.sh"

gather_header "flow"

# ─── 1. App-internal route inventory ───────────────────────────
{
    echo "{"
    echo '  "captured_at": "'"$(date -Iseconds)"'",'
    echo '  "routes": ['
    first=1

    # Next.js app router pages (not API)
    if [ -d "$PROJECT_PATH/app" ]; then
        find "$PROJECT_PATH/app" -name "page.ts" -o -name "page.tsx" -o -name "page.js" -o -name "page.jsx" 2>/dev/null | while read -r f; do
            rel="${f#$PROJECT_PATH/}"
            pseudo="${rel%/page.*}"
            pseudo="${pseudo#app}"
            [ -z "$pseudo" ] && pseudo="/"
            [ $first -eq 1 ] && first=0 || echo ","
            printf '    {"file": "%s", "framework": "next-app-router", "path": "%s", "kind": "page"}' "$rel" "$pseudo"
        done
        find "$PROJECT_PATH/app" -name "layout.ts" -o -name "layout.tsx" 2>/dev/null | while read -r f; do
            rel="${f#$PROJECT_PATH/}"
            [ $first -eq 1 ] && first=0 || echo ","
            printf '    {"file": "%s", "framework": "next-app-router", "path": null, "kind": "layout"}' "$rel"
        done
    fi

    # Next.js pages router
    if [ -d "$PROJECT_PATH/pages" ]; then
        find "$PROJECT_PATH/pages" -type f \( -name "*.tsx" -o -name "*.ts" -o -name "*.jsx" -o -name "*.js" \) \
            -not -path "*/api/*" 2>/dev/null | while read -r f; do
            rel="${f#$PROJECT_PATH/}"
            pseudo="${rel#pages}"
            pseudo="${pseudo%.*}"
            pseudo="${pseudo%/index}"
            [ -z "$pseudo" ] && pseudo="/"
            [ $first -eq 1 ] && first=0 || echo ","
            printf '    {"file": "%s", "framework": "next-pages", "path": "%s", "kind": "page"}' "$rel" "$pseudo"
        done
    fi

    # React Router routes (grep)
    grep -rnE "<Route\s+(path|element)" "$PROJECT_PATH" \
        --include="*.tsx" --include="*.jsx" --include="*.ts" --include="*.js" \
        --exclude-dir=node_modules --exclude-dir=.next --exclude-dir=dist 2>/dev/null \
        | head -50 | while IFS=: read -r file ln content; do
            rel="${file#$PROJECT_PATH/}"
            path=$(echo "$content" | grep -oE 'path=["\x27][^"\x27]+["\x27]' | head -1 | tr -d "'\"" | sed 's/path=//')
            [ -z "$path" ] && continue
            [ $first -eq 1 ] && first=0 || echo ","
            printf '    {"file": "%s", "line": %s, "framework": "react-router", "path": "%s", "kind": "route"}' "$rel" "$ln" "$path"
        done

    echo ""
    echo "  ]"
    echo "}"
} > "$RAW_DIR/route-inventory.json"
log_run "route-inventory" "route-inventory.json" 0

# ─── 2. Internal link probe (URL provided) ──────────────────────
if [ -n "$URL" ]; then
    ROOT="$(echo "$URL" | sed -E 's#(https?://[^/]+).*#\1#')"

    # Fetch the homepage and extract <a href> values
    PAGE_HTML="$RAW_DIR/_homepage.html"
    curl -sLA "Mozilla/5.0" --max-time 30 "$URL" -o "$PAGE_HTML" 2>>"$RAW_DIR/_stderr.log" || true

    if [ -s "$PAGE_HTML" ]; then
        log_run "homepage-fetch" "_homepage.html" 0
        # Extract internal links + dedupe
        python3 - "$PAGE_HTML" "$ROOT" > "$RAW_DIR/internal-links.json" 2>>"$RAW_DIR/_stderr.log" <<'PY'
import re, sys, json
from urllib.parse import urlparse, urljoin
with open(sys.argv[1], "r", errors="replace") as f:
    html = f.read()
root = sys.argv[2]
root_host = urlparse(root).netloc
hrefs = re.findall(r'<a[^>]*\bhref=["\']([^"\']+)["\']', html, re.I)
internal = []
external = []
seen = set()
for h in hrefs:
    if h.startswith(("javascript:", "mailto:", "tel:", "#")):
        continue
    if h in seen:
        continue
    seen.add(h)
    abs_url = urljoin(root, h)
    parsed = urlparse(abs_url)
    if parsed.netloc == root_host or not parsed.netloc:
        internal.append(abs_url)
    else:
        external.append(abs_url)
out = {"internal": internal[:50], "external": external[:25]}
print(json.dumps(out, indent=2))
PY
        log_run "internal-link-extractor" "internal-links.json" 0

        # Probe each internal link (HEAD) — limit 30 to keep fast
        PROBE_FILE="$RAW_DIR/link-probes.json"
        {
            echo "["
            first=1
            python3 -c "import json; d=json.load(open('$RAW_DIR/internal-links.json')); print('\n'.join(d.get('internal', [])[:30]))" 2>/dev/null \
                | while read -r link; do
                    [ -z "$link" ] && continue
                    code=$(curl -sLI -o /dev/null -w "%{http_code}" -A "Mozilla/5.0" --max-time 8 "$link" 2>/dev/null || echo 0)
                    [ $first -eq 1 ] && first=0 || echo ","
                    printf '  {"url": %s, "status": %s}' "$(json_str "$link")" "${code:-0}"
                done
            echo ""
            echo "]"
        } > "$PROBE_FILE"
        log_run "link-prober" "link-probes.json" 0
    fi
fi

# ─── 3. Form / interaction inventory (static scan) ──────────────
{
    echo "{"
    echo '  "forms_in_code": ['
    first=1
    grep -rnE "<form[^>]*\b(action|onSubmit|method)" "$PROJECT_PATH" \
        --include="*.tsx" --include="*.jsx" --include="*.html" --include="*.ts" --include="*.js" \
        --exclude-dir=node_modules --exclude-dir=.next --exclude-dir=dist 2>/dev/null \
        | head -50 | while IFS=: read -r file ln content; do
            rel="${file#$PROJECT_PATH/}"
            [ $first -eq 1 ] && first=0 || echo ","
            printf '    {"file": "%s", "line": %s, "snippet": %s}' "$rel" "$ln" "$(json_str "$(echo "$content" | xargs | head -c 200)")"
        done
    echo ""
    echo "  ],"
    echo '  "buttons_with_handlers": '
    grep -rocE "<button[^>]*\bonClick" "$PROJECT_PATH" \
        --include="*.tsx" --include="*.jsx" \
        --exclude-dir=node_modules 2>/dev/null \
        | awk -F: '{sum += $NF} END {print sum+0}'
    echo "}"
} > "$RAW_DIR/interaction-inventory.json"
log_run "interaction-inventory" "interaction-inventory.json" 0

# ─── 4. Empty-state / error-state heuristic ─────────────────────
{
    echo "{"
    echo '  "empty_state_indicators": '
    grep -rocE "(no\s+results|empty\s+state|nothing\s+yet|not\s+found|coming\s+soon)" "$PROJECT_PATH" \
        --include="*.tsx" --include="*.jsx" --include="*.html" \
        --exclude-dir=node_modules --exclude-dir=.next 2>/dev/null \
        | awk -F: '{sum += $NF} END {print sum+0}'
    echo ","
    echo '  "error_boundary_files": '
    grep -rlE "(ErrorBoundary|error-boundary|componentDidCatch|getDerivedStateFromError)" "$PROJECT_PATH" \
        --include="*.tsx" --include="*.jsx" --include="*.ts" --include="*.js" \
        --exclude-dir=node_modules --exclude-dir=.next 2>/dev/null \
        | wc -l
    echo "}"
} > "$RAW_DIR/state-coverage.json"
log_run "state-coverage" "state-coverage.json" 0

echo "flow gather done"

#!/usr/bin/env bash
# perf.sh — performance gather. Runs Lighthouse perf, size-limit, build-dir size analysis.
# Args: RAW_DIR PROJECT_PATH FILES URL
set -uo pipefail

RAW_DIR="${1:?missing raw_dir}"
PROJECT_PATH="${2:?missing project_path}"
FILES="${3:-}"
URL="${4:-}"
mkdir -p "$RAW_DIR"

# shellcheck disable=SC1091
source "$HOME/.omega/lib/audit-gather/_common.sh"

gather_header "perf"

# ─── 1. Lighthouse perf category (URL only) ─────────────────────
if [ -n "$URL" ]; then
    if command -v lighthouse >/dev/null 2>&1 || check_npx_pkg "lighthouse"; then
        LH_BIN="$(command -v lighthouse || echo "npx --yes lighthouse")"
        safe_cmd "$RAW_DIR/lighthouse-perf.json" "lighthouse-perf" \
            $LH_BIN "$URL" \
                --output=json \
                --output-path="$RAW_DIR/lighthouse-perf.json" \
                --only-categories=performance \
                --chrome-flags="--headless --no-sandbox --disable-dev-shm-usage" \
                --quiet
    fi
else
    log_skip "lighthouse-perf" "URL empty (no runtime perf)"
fi

# ─── 2. size-limit (if configured in package.json) ──────────────
if [ -f "$PROJECT_PATH/package.json" ]; then
    if grep -q '"size-limit"' "$PROJECT_PATH/package.json"; then
        if check_npx_pkg "size-limit"; then
            (cd "$PROJECT_PATH" && safe_cmd "$RAW_DIR/size-limit.json" "size-limit" \
                npx --yes size-limit --json)
        fi
    else
        log_skip "size-limit" "no size-limit config in package.json"
    fi
fi

# ─── 3. Build dir size analysis ─────────────────────────────────
SIZES_FILE="$RAW_DIR/build-sizes.json"
{
    echo "{"
    echo '  "captured_at": "'"$(date -Iseconds)"'",'
    echo '  "directories": {'
    first=1
    for dir in .next dist build out public/_next .nuxt .svelte-kit .turbo; do
        if [ -d "$PROJECT_PATH/$dir" ]; then
            size=$(du -sb "$PROJECT_PATH/$dir" 2>/dev/null | awk '{print $1}')
            file_count=$(find "$PROJECT_PATH/$dir" -type f 2>/dev/null | wc -l)
            [ $first -eq 1 ] && first=0 || echo ","
            printf '    "%s": {"bytes": %s, "files": %s}' "$dir" "${size:-0}" "${file_count:-0}"
        fi
    done
    echo ""
    echo "  },"
    # Top 20 largest static files
    echo '  "top_files": ['
    first=1
    found=""
    for d in .next/static dist build public/static; do
        if [ -d "$PROJECT_PATH/$d" ]; then
            found="$d"
            break
        fi
    done
    if [ -n "$found" ]; then
        find "$PROJECT_PATH/$found" -type f -printf '%s %p\n' 2>/dev/null \
            | sort -rn | head -20 \
            | while read -r size path; do
                rel="${path#$PROJECT_PATH/}"
                [ $first -eq 1 ] && first=0 || echo ","
                printf '    {"size": %s, "path": "%s"}' "$size" "$rel"
            done
    fi
    echo ""
    echo "  ]"
    echo "}"
} > "$SIZES_FILE"
log_run "build-size-analyzer" "build-sizes.json" 0

# ─── 4. node_modules size (warning signal) ──────────────────────
if [ -d "$PROJECT_PATH/node_modules" ]; then
    NM_SIZE=$(du -sb "$PROJECT_PATH/node_modules" 2>/dev/null | awk '{print $1}')
    NM_COUNT=$(find "$PROJECT_PATH/node_modules" -maxdepth 2 -type d 2>/dev/null | wc -l)
    cat > "$RAW_DIR/node-modules.json" <<EOF
{"bytes": ${NM_SIZE:-0}, "package_dirs": ${NM_COUNT:-0}}
EOF
    log_run "node-modules-size" "node-modules.json" 0
fi

echo "perf gather done"

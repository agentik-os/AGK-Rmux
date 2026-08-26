#!/usr/bin/env bash
# a11y.sh — accessibility gather. Runs axe-core, pa11y, and Lighthouse a11y.
# Args: RAW_DIR PROJECT_PATH FILES URL
set -uo pipefail

RAW_DIR="${1:?missing raw_dir}"
PROJECT_PATH="${2:?missing project_path}"
FILES="${3:-}"
URL="${4:-}"
mkdir -p "$RAW_DIR"

# shellcheck disable=SC1091
source "$HOME/.omega/lib/audit-gather/_common.sh"

gather_header "a11y"

# Without a URL, accessibility cannot be machine-tested.
if ! is_web_project; then
    log_skip "axe-core" "no URL provided and no web framework detected"
    log_skip "pa11y" "no URL provided"
    log_skip "lighthouse" "no URL provided"
    echo "a11y: project is not web-deployable, skipping URL-based tools"
    exit 0
fi

if [ -z "$URL" ]; then
    log_skip "axe-core" "URL empty"
    log_skip "pa11y" "URL empty"
    log_skip "lighthouse" "URL empty"
    echo "a11y: web project but URL not given; cannot test runtime"
    exit 0
fi

# ─── 1. axe-core via @axe-core/cli ────────────────────────────────
if check_npx_pkg "@axe-core/cli"; then
    safe_cmd "$RAW_DIR/axe.json" "axe-core" \
        npx --yes @axe-core/cli "$URL" --save "$RAW_DIR/axe.json" --exit
    # @axe-core/cli writes to file directly; ensure it exists
    [ -s "$RAW_DIR/axe.json" ] || echo "[]" > "$RAW_DIR/axe.json"
fi

# ─── 2. pa11y ────────────────────────────────────────────────────
if check_npx_pkg "pa11y"; then
    safe_cmd "$RAW_DIR/pa11y.json" "pa11y" \
        npx --yes pa11y --reporter json "$URL"
fi

# ─── 3. Lighthouse a11y category ─────────────────────────────────
if command -v lighthouse >/dev/null 2>&1 || check_npx_pkg "lighthouse"; then
    LH_BIN="$(command -v lighthouse || echo "npx --yes lighthouse")"
    safe_cmd "$RAW_DIR/lighthouse-a11y.json" "lighthouse-a11y" \
        $LH_BIN "$URL" \
            --output=json \
            --output-path="$RAW_DIR/lighthouse-a11y.json" \
            --only-categories=accessibility \
            --chrome-flags="--headless --no-sandbox --disable-dev-shm-usage" \
            --quiet
fi

echo "a11y gather done"

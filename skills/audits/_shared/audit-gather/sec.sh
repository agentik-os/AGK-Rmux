#!/usr/bin/env bash
# sec.sh — security gather. Runs npm audit, gitleaks, semgrep, pip-audit (Python).
# Args: RAW_DIR PROJECT_PATH FILES URL
set -uo pipefail

RAW_DIR="${1:?missing raw_dir}"
PROJECT_PATH="${2:?missing project_path}"
FILES="${3:-}"
URL="${4:-}"
mkdir -p "$RAW_DIR"

# shellcheck disable=SC1091
source "$HOME/.omega/lib/audit-gather/_common.sh"

gather_header "sec"

# ─── 1. npm audit ───────────────────────────────────────────────
if [ -f "$PROJECT_PATH/package.json" ]; then
    if [ -f "$PROJECT_PATH/package-lock.json" ] || [ -f "$PROJECT_PATH/yarn.lock" ] || [ -f "$PROJECT_PATH/pnpm-lock.yaml" ]; then
        if check_tool "npm"; then
            (cd "$PROJECT_PATH" && safe_cmd "$RAW_DIR/npm-audit.json" "npm-audit" \
                npm audit --json)
        fi
    else
        log_skip "npm-audit" "no lockfile present"
    fi
fi

# ─── 2. pip-audit / safety (Python) ─────────────────────────────
if [ -f "$PROJECT_PATH/requirements.txt" ] || [ -f "$PROJECT_PATH/pyproject.toml" ]; then
    if check_tool "pip-audit" "pip install pip-audit"; then
        (cd "$PROJECT_PATH" && safe_cmd "$RAW_DIR/pip-audit.json" "pip-audit" \
            pip-audit --format=json)
    fi
fi

# ─── 3. gitleaks (secrets) ──────────────────────────────────────
if check_tool "gitleaks" "brew install gitleaks"; then
    (cd "$PROJECT_PATH" && safe_cmd "$RAW_DIR/gitleaks.json" "gitleaks" \
        gitleaks detect --no-banner --report-format json --report-path "$RAW_DIR/gitleaks.json" --exit-code 0 --source .)
    [ -s "$RAW_DIR/gitleaks.json" ] || echo "[]" > "$RAW_DIR/gitleaks.json"
fi

# ─── 4. semgrep ─────────────────────────────────────────────────
if check_tool "semgrep" "pip install semgrep"; then
    (cd "$PROJECT_PATH" && safe_cmd "$RAW_DIR/semgrep.json" "semgrep" \
        semgrep --config=auto --json --quiet --severity=ERROR --severity=WARNING --error 2>&1 || true)
elif check_npx_pkg "semgrep"; then
    log_skip "semgrep" "semgrep is python-only; npm package is unrelated"
fi

# ─── 5. eslint security plugin (lightweight inline) ─────────────
# Only if there's an eslint config + node_modules
if [ -f "$PROJECT_PATH/.eslintrc.json" ] || [ -f "$PROJECT_PATH/.eslintrc.js" ] || [ -f "$PROJECT_PATH/eslint.config.js" ] || [ -f "$PROJECT_PATH/eslint.config.mjs" ]; then
    if [ -d "$PROJECT_PATH/node_modules" ] && command -v npx >/dev/null 2>&1; then
        # Use the project's local eslint, do NOT install
        (cd "$PROJECT_PATH" && safe_cmd "$RAW_DIR/eslint-sec.json" "eslint-security" \
            npx --no-install eslint . --format json --no-error-on-unmatched-pattern --no-eslintrc --rulesdir /tmp/__nope 2>/dev/null || \
            npx --no-install eslint . --format json --no-error-on-unmatched-pattern)
    fi
fi

# ─── 6. .env / secrets file scan ────────────────────────────────
ENV_REPORT="$RAW_DIR/env-files.json"
{
    echo "{"
    echo '  "captured_at": "'"$(date -Iseconds)"'",'
    echo '  "env_files": ['
    first=1
    while IFS= read -r f; do
        rel="${f#$PROJECT_PATH/}"
        case "$rel" in
            node_modules/*|.next/*|dist/*|build/*) continue ;;
        esac
        in_gitignore=false
        if [ -f "$PROJECT_PATH/.gitignore" ] && grep -qE "^${rel}$|^\.env" "$PROJECT_PATH/.gitignore" 2>/dev/null; then
            in_gitignore=true
        fi
        in_git=false
        if (cd "$PROJECT_PATH" && git ls-files --error-unmatch "$rel" >/dev/null 2>&1); then
            in_git=true
        fi
        [ $first -eq 1 ] && first=0 || echo ","
        printf '    {"path": "%s", "in_gitignore": %s, "tracked_by_git": %s}' \
            "$rel" "$in_gitignore" "$in_git"
    done < <(find "$PROJECT_PATH" -maxdepth 4 -type f \( -name ".env*" -o -name "secrets.*" -o -name "credentials.*" \) 2>/dev/null)
    echo ""
    echo "  ]"
    echo "}"
} > "$ENV_REPORT"
log_run "env-file-scanner" "env-files.json" 0

# ─── 7. HTTP security headers (if URL) ──────────────────────────
if [ -n "$URL" ]; then
    HDR_FILE="$RAW_DIR/http-headers.txt"
    curl -sLI -A "Mozilla/5.0" --max-time 15 "$URL" -o "$HDR_FILE" 2>>"$RAW_DIR/_stderr.log" || true
    if [ -s "$HDR_FILE" ]; then
        log_run "http-headers" "http-headers.txt" 0
    else
        log_skip "http-headers" "could not fetch URL"
    fi
fi

echo "sec gather done"

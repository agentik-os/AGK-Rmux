#!/usr/bin/env bash
# code.sh — code quality gather. ESLint, ts-prune, depcheck, madge, ruff/pylint (Python).
# Args: RAW_DIR PROJECT_PATH FILES URL
set -uo pipefail

RAW_DIR="${1:?missing raw_dir}"
PROJECT_PATH="${2:?missing project_path}"
FILES="${3:-}"
URL="${4:-}"
mkdir -p "$RAW_DIR"

# shellcheck disable=SC1091
source "$HOME/.omega/lib/audit-gather/_common.sh"

gather_header "code"

LANGS="$(detect_languages)"
echo "$LANGS" > "$RAW_DIR/_languages.json"

# ─── 1. ESLint (TypeScript / JavaScript) ────────────────────────
HAS_ESLINT_CFG=false
for cfg in .eslintrc.json .eslintrc.js .eslintrc.cjs .eslintrc.yaml .eslintrc.yml eslint.config.js eslint.config.mjs eslint.config.cjs; do
    [ -f "$PROJECT_PATH/$cfg" ] && HAS_ESLINT_CFG=true && break
done
if [ -f "$PROJECT_PATH/package.json" ] && grep -q '"eslintConfig"' "$PROJECT_PATH/package.json"; then
    HAS_ESLINT_CFG=true
fi

if [ "$HAS_ESLINT_CFG" = "true" ] && [ -d "$PROJECT_PATH/node_modules" ]; then
    if check_npx_pkg "eslint"; then
        (cd "$PROJECT_PATH" && safe_cmd "$RAW_DIR/eslint.json" "eslint" \
            npx --no-install eslint . --format json --no-error-on-unmatched-pattern --ext .js,.jsx,.ts,.tsx,.mjs,.cjs)
    fi
elif [ -f "$PROJECT_PATH/package.json" ]; then
    log_skip "eslint" "no eslint config or node_modules"
fi

# ─── 2. TypeScript --noEmit ────────────────────────────────────
if [ -f "$PROJECT_PATH/tsconfig.json" ] && [ -d "$PROJECT_PATH/node_modules" ]; then
    if check_npx_pkg "typescript"; then
        (cd "$PROJECT_PATH" && safe_cmd "$RAW_DIR/tsc.txt" "tsc" \
            npx --no-install tsc --noEmit --pretty false)
    fi
elif [ -f "$PROJECT_PATH/tsconfig.json" ]; then
    log_skip "tsc" "no node_modules"
fi

# ─── 3. ts-prune (unused exports) ───────────────────────────────
if [ -f "$PROJECT_PATH/tsconfig.json" ]; then
    if check_npx_pkg "ts-prune"; then
        (cd "$PROJECT_PATH" && safe_cmd "$RAW_DIR/ts-prune.txt" "ts-prune" \
            npx --yes ts-prune)
    fi
fi

# ─── 4. depcheck (unused npm deps) ──────────────────────────────
if [ -f "$PROJECT_PATH/package.json" ]; then
    if check_npx_pkg "depcheck"; then
        (cd "$PROJECT_PATH" && safe_cmd "$RAW_DIR/depcheck.json" "depcheck" \
            npx --yes depcheck --json)
    fi
fi

# ─── 5. madge (circular deps in JS/TS) ──────────────────────────
if [ -f "$PROJECT_PATH/package.json" ]; then
    if check_npx_pkg "madge"; then
        SRC_DIR="src"
        [ ! -d "$PROJECT_PATH/src" ] && [ -d "$PROJECT_PATH/app" ] && SRC_DIR="app"
        [ ! -d "$PROJECT_PATH/$SRC_DIR" ] && SRC_DIR="."
        (cd "$PROJECT_PATH" && safe_cmd "$RAW_DIR/madge-circular.json" "madge" \
            npx --yes madge --circular --json --extensions js,jsx,ts,tsx "$SRC_DIR")
    fi
fi

# ─── 6. Python: ruff / flake8 / pylint ──────────────────────────
if echo "$LANGS" | grep -q '"python"'; then
    if check_tool "ruff"; then
        (cd "$PROJECT_PATH" && safe_cmd "$RAW_DIR/ruff.json" "ruff" \
            ruff check --output-format json --exit-zero .)
    elif check_tool "flake8"; then
        (cd "$PROJECT_PATH" && safe_cmd "$RAW_DIR/flake8.txt" "flake8" \
            flake8 --max-line-length=120 .)
    fi

    if check_tool "vulture" "pip install vulture"; then
        (cd "$PROJECT_PATH" && safe_cmd "$RAW_DIR/vulture.txt" "vulture" \
            vulture . --min-confidence 70)
    fi
fi

# ─── 7. file size + line count summary (always free) ────────────
{
    echo "{"
    echo '  "captured_at": "'"$(date -Iseconds)"'",'
    echo '  "large_files": ['
    first=1
    while IFS= read -r line; do
        size="$(echo "$line" | awk '{print $1}')"
        path="$(echo "$line" | awk '{print $2}')"
        rel="${path#$PROJECT_PATH/}"
        case "$rel" in
            node_modules/*|.next/*|dist/*|build/*|.git/*|venv/*|.venv/*|*.lock|*.json) continue ;;
        esac
        [ $first -eq 1 ] && first=0 || echo ","
        printf '    {"size": %s, "path": "%s"}' "$size" "$rel"
    done < <(find "$PROJECT_PATH" -type f \
        \( -name "*.ts" -o -name "*.tsx" -o -name "*.js" -o -name "*.jsx" -o -name "*.py" -o -name "*.go" -o -name "*.rs" \) \
        -not -path "*/node_modules/*" -not -path "*/.next/*" -not -path "*/dist/*" -not -path "*/build/*" \
        -not -path "*/.git/*" -not -path "*/venv/*" -not -path "*/.venv/*" \
        -printf '%s %p\n' 2>/dev/null | sort -rn | head -15)
    echo ""
    echo "  ]"
    echo "}"
} > "$RAW_DIR/large-files.json"
log_run "large-file-scanner" "large-files.json" 0

echo "code gather done"

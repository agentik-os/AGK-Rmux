#!/usr/bin/env bash
# logic.sh — system-logic gather. Cycles, hot files, complexity, dead code.
# Args: RAW_DIR PROJECT_PATH FILES URL
set -uo pipefail

RAW_DIR="${1:?missing raw_dir}"
PROJECT_PATH="${2:?missing project_path}"
FILES="${3:-}"
URL="${4:-}"
mkdir -p "$RAW_DIR"

# shellcheck disable=SC1091
source "$HOME/.omega/lib/audit-gather/_common.sh"

gather_header "logic"

LANGS="$(detect_languages)"

# ─── 1. madge --circular (JS/TS) ────────────────────────────────
if [ -f "$PROJECT_PATH/package.json" ]; then
    if check_npx_pkg "madge"; then
        SRC_DIR="src"
        [ ! -d "$PROJECT_PATH/src" ] && [ -d "$PROJECT_PATH/app" ] && SRC_DIR="app"
        [ ! -d "$PROJECT_PATH/$SRC_DIR" ] && SRC_DIR="."
        (cd "$PROJECT_PATH" && safe_cmd "$RAW_DIR/madge-circular.json" "madge-circular" \
            npx --yes madge --circular --json --extensions js,jsx,ts,tsx "$SRC_DIR")
        # Full graph (sample summary)
        (cd "$PROJECT_PATH" && safe_cmd "$RAW_DIR/madge-summary.json" "madge-summary" \
            npx --yes madge --json --extensions js,jsx,ts,tsx "$SRC_DIR")
    fi
fi

# ─── 2. dep-cruiser (richer cycle/violation report) ─────────────
if [ -f "$PROJECT_PATH/package.json" ] && [ -d "$PROJECT_PATH/node_modules" ]; then
    if check_npx_pkg "dependency-cruiser"; then
        SRC_DIR="src"
        [ ! -d "$PROJECT_PATH/src" ] && [ -d "$PROJECT_PATH/app" ] && SRC_DIR="app"
        [ ! -d "$PROJECT_PATH/$SRC_DIR" ] && SRC_DIR="."
        (cd "$PROJECT_PATH" && safe_cmd "$RAW_DIR/dep-cruiser.json" "dep-cruiser" \
            npx --no-install depcruise --output-type json --include-only "^$SRC_DIR" "$SRC_DIR" 2>&1) || true
    fi
fi

# ─── 3. cloc (code volume + comment ratio) ──────────────────────
if check_tool "cloc"; then
    (cd "$PROJECT_PATH" && safe_cmd "$RAW_DIR/cloc.json" "cloc" \
        cloc --json --quiet --exclude-dir=node_modules,.next,dist,build,venv,.venv,.git .)
fi

# ─── 4. Complexity heuristic (LOC + nesting depth + branch density) ─
{
    echo "{"
    echo '  "files": ['
    first=1
    find "$PROJECT_PATH" -type f \
        \( -name "*.ts" -o -name "*.tsx" -o -name "*.js" -o -name "*.jsx" -o -name "*.py" -o -name "*.go" -o -name "*.rs" \) \
        -not -path "*/node_modules/*" -not -path "*/.next/*" -not -path "*/dist/*" -not -path "*/build/*" \
        -not -path "*/.git/*" -not -path "*/venv/*" -not -path "*/.venv/*" 2>/dev/null \
        | head -2000 \
        | while read -r f; do
            rel="${f#$PROJECT_PATH/}"
            loc=$(wc -l < "$f" 2>/dev/null || echo 0)
            [ "$loc" -lt 100 ] && continue
            branches=$(grep -cE "(^|[^a-zA-Z_])(if|else|switch|case|for|while|try|catch|elif)([^a-zA-Z_]|$)" "$f" 2>/dev/null || echo 0)
            functions=$(grep -cE "^(\s*)(export\s+)?(async\s+)?(function|const\s+\w+\s*=\s*(async\s*)?\(|def\s+\w+|fn\s+\w+|func\s+\w+)" "$f" 2>/dev/null || echo 0)
            # Crude nesting depth: max leading-whitespace runs
            max_nest=$(awk '{ match($0, /^[ \t]+/); print RLENGTH/2 }' "$f" 2>/dev/null | sort -rn | head -1)
            [ -z "$max_nest" ] && max_nest=0
            [ $first -eq 1 ] && first=0 || echo ","
            printf '    {"file": "%s", "loc": %s, "branches": %s, "functions": %s, "max_nest": %s}' \
                "$rel" "$loc" "$branches" "$functions" "$max_nest"
        done
    echo ""
    echo "  ]"
    echo "}"
} > "$RAW_DIR/complexity-heuristic.json"
log_run "complexity-heuristic" "complexity-heuristic.json" 0

# ─── 5. Hot file detection (git log churn) ──────────────────────
if [ -d "$PROJECT_PATH/.git" ] && check_tool "git"; then
    {
        echo "{"
        echo '  "top_churn": ['
        first=1
        # Files changed in last 200 commits
        (cd "$PROJECT_PATH" && git log --pretty=format: --name-only --max-count=200 2>/dev/null \
            | grep -v '^$' | sort | uniq -c | sort -rn | head -20) \
            | while read -r count file; do
                case "$file" in
                    node_modules/*|.next/*|dist/*|build/*|*.lock|*.json) continue ;;
                esac
                [ -z "$file" ] && continue
                [ $first -eq 1 ] && first=0 || echo ","
                printf '    {"file": "%s", "commits_in_last_200": %s}' "$file" "$count"
            done
        echo ""
        echo "  ]"
        echo "}"
    } > "$RAW_DIR/git-churn.json"
    log_run "git-churn" "git-churn.json" 0
fi

# ─── 6. TODO / FIXME / HACK density ─────────────────────────────
{
    echo "{"
    echo '  "total_count": '
    grep -rocE "(TODO|FIXME|HACK|XXX)\b" "$PROJECT_PATH" \
        --include="*.ts" --include="*.tsx" --include="*.js" --include="*.jsx" --include="*.py" --include="*.go" --include="*.rs" \
        --exclude-dir=node_modules --exclude-dir=.next --exclude-dir=dist --exclude-dir=build 2>/dev/null \
        | awk -F: '{sum += $NF} END {print sum+0}'
    echo ","
    echo '  "samples": ['
    first=1
    grep -rnE "(TODO|FIXME|HACK|XXX)\b" "$PROJECT_PATH" \
        --include="*.ts" --include="*.tsx" --include="*.js" --include="*.jsx" --include="*.py" \
        --exclude-dir=node_modules --exclude-dir=.next --exclude-dir=dist --exclude-dir=build 2>/dev/null \
        | head -25 | while IFS=: read -r file ln content; do
            rel="${file#$PROJECT_PATH/}"
            [ $first -eq 1 ] && first=0 || echo ","
            printf '    {"file": "%s", "line": %s, "snippet": %s}' "$rel" "$ln" "$(json_str "$(echo "$content" | xargs | head -c 200)")"
        done
    echo ""
    echo "  ]"
    echo "}"
} > "$RAW_DIR/todo-density.json"
log_run "todo-density" "todo-density.json" 0

echo "logic gather done"

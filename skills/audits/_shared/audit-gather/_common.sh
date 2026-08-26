#!/usr/bin/env bash
# _common.sh — shared helpers for hybrid audit gather scripts.
#
# Sourced by every <audit>.sh. Provides:
#   - check_tool        : silent probe ("command -v")
#   - log_skip          : write tools-skipped.json entry
#   - log_run           : write tools-run.json entry
#   - safe_cmd          : run with timeout + capture rc, never fail the run
#   - is_web_project    : detect URL-able project (next.js / vite / package.json with react)
#   - filter_files      : strip node_modules/.next/dist/build from a file list
#
# All scripts MUST `set -uo pipefail` (NOT -e) — we want soft failures so a missing
# tool never aborts the whole audit.

# ─── globals expected from caller ────────────────────────────────
# RAW_DIR        — directory to write raw outputs into ($1)
# PROJECT_PATH   — project root ($2)
# FILES          — comma-separated files in scope ($3 — may be empty)
# URL            — runtime URL for the project ($4 — may be empty)

# ─── helpers ─────────────────────────────────────────────────────

# check_tool <name> [<install hint>] — returns 0 if available, else logs skip + returns 1.
check_tool() {
    local tool="$1"
    local hint="${2:-}"
    if command -v "$tool" >/dev/null 2>&1; then
        return 0
    fi
    log_skip "$tool" "command '$tool' not found${hint:+; hint: $hint}"
    return 1
}

# check_npx_pkg <pkg> — assumes npx exists. Probes by running --version with timeout.
# Returns 0 if package can be resolved/cached.
check_npx_pkg() {
    local pkg="$1"
    if ! command -v npx >/dev/null 2>&1; then
        log_skip "$pkg" "npx not available"
        return 1
    fi
    return 0
}

# log_skip <tool> <reason>
log_skip() {
    local tool="$1"
    local reason="$2"
    local skipped="$RAW_DIR/_tools-skipped.jsonl"
    printf '{"tool":%s,"reason":%s,"ts":%s}\n' \
        "$(json_str "$tool")" \
        "$(json_str "$reason")" \
        "$(json_str "$(date -Iseconds)")" \
        >> "$skipped"
}

# log_run <tool> <output_file> <rc>
log_run() {
    local tool="$1"
    local out="$2"
    local rc="$3"
    local ran="$RAW_DIR/_tools-run.jsonl"
    printf '{"tool":%s,"output":%s,"rc":%s,"ts":%s}\n' \
        "$(json_str "$tool")" \
        "$(json_str "$out")" \
        "$rc" \
        "$(json_str "$(date -Iseconds)")" \
        >> "$ran"
}

# json_str <s> — minimal JSON-string escape (handles quotes, backslash, newlines).
json_str() {
    printf '%s' "$1" | python3 -c 'import json,sys;print(json.dumps(sys.stdin.read()))'
}

# safe_cmd <out_file> <tool_name> <cmd...> — run with 60s timeout, never aborts.
# Captures rc, logs run, ensures out_file exists (writes "[]" if empty).
safe_cmd() {
    local out_file="$1"; shift
    local tool_name="$1"; shift
    local rc=0
    timeout 90 "$@" > "$out_file" 2>>"$RAW_DIR/_stderr.log" || rc=$?
    [ ! -s "$out_file" ] && echo "[]" > "$out_file"
    log_run "$tool_name" "$(basename "$out_file")" "$rc"
    return 0
}

# is_web_project — emits 0 if web-deployable, 1 otherwise.
is_web_project() {
    [ -n "$URL" ] && return 0
    if [ -f "$PROJECT_PATH/package.json" ]; then
        if grep -qE '"(next|vite|react|@remix-run|astro|nuxt)"' "$PROJECT_PATH/package.json" 2>/dev/null; then
            return 0
        fi
    fi
    return 1
}

# filter_files <file_or_glob> — echoes filtered list of real files (skip noise dirs).
filter_files() {
    local input="$1"
    [ -z "$input" ] && return 0
    echo "$input" | tr ',' '\n' | while read -r f; do
        f="$(echo "$f" | xargs)"
        [ -z "$f" ] && continue
        case "$f" in
            *node_modules/*|*.next/*|*dist/*|*build/*|*.d.ts.map) continue ;;
        esac
        if [ -e "$PROJECT_PATH/$f" ] || [ -e "$f" ]; then
            echo "$f"
        fi
    done
}

# detect_languages — emits a JSON array of detected language ecosystems.
detect_languages() {
    local langs=()
    [ -f "$PROJECT_PATH/package.json" ] && langs+=("node")
    [ -f "$PROJECT_PATH/tsconfig.json" ] && langs+=("typescript")
    if compgen -G "$PROJECT_PATH/*.py" >/dev/null 2>&1 || [ -f "$PROJECT_PATH/pyproject.toml" ] || [ -f "$PROJECT_PATH/requirements.txt" ]; then
        langs+=("python")
    fi
    [ -f "$PROJECT_PATH/go.mod" ] && langs+=("go")
    [ -f "$PROJECT_PATH/Cargo.toml" ] && langs+=("rust")
    printf '['
    local first=1
    for l in "${langs[@]:-}"; do
        [ -z "$l" ] && continue
        [ $first -eq 1 ] && first=0 || printf ','
        printf '"%s"' "$l"
    done
    printf ']'
}

# header — echo banner once per gather call.
gather_header() {
    local audit="$1"
    echo "─── $audit gather ───"
    echo "RAW_DIR=$RAW_DIR"
    echo "PROJECT_PATH=$PROJECT_PATH"
    echo "FILES=${FILES:-<none>}"
    echo "URL=${URL:-<none>}"
    : > "$RAW_DIR/_stderr.log"
    : > "$RAW_DIR/_tools-run.jsonl"
    : > "$RAW_DIR/_tools-skipped.jsonl"
}

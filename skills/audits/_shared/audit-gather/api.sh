#!/usr/bin/env bash
# api.sh — API quality gather. OpenAPI validation, route discovery, endpoint probe.
# Args: RAW_DIR PROJECT_PATH FILES URL
set -uo pipefail

RAW_DIR="${1:?missing raw_dir}"
PROJECT_PATH="${2:?missing project_path}"
FILES="${3:-}"
URL="${4:-}"
mkdir -p "$RAW_DIR"

# shellcheck disable=SC1091
source "$HOME/.omega/lib/audit-gather/_common.sh"

gather_header "api"

# ─── 1. OpenAPI / spec discovery ────────────────────────────────
SPEC_FILE=""
for f in openapi.json openapi.yaml openapi.yml swagger.json swagger.yaml api/openapi.json docs/openapi.json; do
    if [ -f "$PROJECT_PATH/$f" ]; then
        SPEC_FILE="$PROJECT_PATH/$f"
        break
    fi
done

# Also check via runtime URL
if [ -z "$SPEC_FILE" ] && [ -n "$URL" ]; then
    ROOT="$(echo "$URL" | sed -E 's#(https?://[^/]+).*#\1#')"
    for path in /openapi.json /api/openapi.json /docs/openapi.json /swagger.json /api-docs; do
        if curl -sLI --max-time 10 "$ROOT$path" 2>/dev/null | head -1 | grep -q "200"; then
            curl -sL --max-time 30 "$ROOT$path" -o "$RAW_DIR/openapi-fetched.json" 2>>"$RAW_DIR/_stderr.log" || true
            if [ -s "$RAW_DIR/openapi-fetched.json" ]; then
                SPEC_FILE="$RAW_DIR/openapi-fetched.json"
                log_run "openapi-fetch" "openapi-fetched.json" 0
                break
            fi
        fi
    done
fi

if [ -n "$SPEC_FILE" ]; then
    cp "$SPEC_FILE" "$RAW_DIR/openapi-spec.json" 2>/dev/null || true
    log_run "openapi-spec-copy" "openapi-spec.json" 0
    if check_npx_pkg "@apidevtools/swagger-cli"; then
        safe_cmd "$RAW_DIR/openapi-validation.json" "swagger-cli" \
            npx --yes @apidevtools/swagger-cli validate "$SPEC_FILE"
    fi
else
    log_skip "openapi-validation" "no OpenAPI/Swagger spec found"
fi

# ─── 2. Route discovery (Express / Next.js / FastAPI / Flask / Django) ────
ROUTE_REPORT="$RAW_DIR/routes-discovered.json"
{
    echo "{"
    echo '  "captured_at": "'"$(date -Iseconds)"'",'
    echo '  "patterns": ['

    first=1
    emit_route() {
        [ $first -eq 1 ] && first=0 || echo ","
        printf '    {"file": "%s", "line": %s, "method": "%s", "path": "%s", "framework": "%s"}' "$1" "$2" "$3" "$4" "$5"
    }

    # Next.js app router
    if [ -d "$PROJECT_PATH/app" ]; then
        find "$PROJECT_PATH/app" -name "route.ts" -o -name "route.js" 2>/dev/null | while read -r f; do
            rel="${f#$PROJECT_PATH/}"
            grep -nE 'export (async )?function (GET|POST|PUT|PATCH|DELETE|OPTIONS|HEAD)' "$f" 2>/dev/null | while IFS=: read -r line content; do
                method=$(echo "$content" | grep -oE '(GET|POST|PUT|PATCH|DELETE|OPTIONS|HEAD)' | head -1)
                pseudo_path="${rel%/route.*}"
                pseudo_path="${pseudo_path#app}"
                emit_route "$rel" "$line" "$method" "$pseudo_path" "next-app-router"
            done
        done
    fi

    # Next.js pages router
    if [ -d "$PROJECT_PATH/pages/api" ]; then
        find "$PROJECT_PATH/pages/api" -type f \( -name "*.ts" -o -name "*.js" \) 2>/dev/null | while read -r f; do
            rel="${f#$PROJECT_PATH/}"
            pseudo_path="${rel#pages}"
            pseudo_path="${pseudo_path%.*}"
            emit_route "$rel" "1" "ANY" "$pseudo_path" "next-pages-api"
        done
    fi

    # Express / Hono / similar (grep app.get/post/etc)
    grep -rnE "^[[:space:]]*(app|router|api|server)\.(get|post|put|patch|delete|use)\(['\"]([^'\"]+)" "$PROJECT_PATH" \
        --include="*.ts" --include="*.tsx" --include="*.js" --include="*.mjs" \
        --exclude-dir=node_modules --exclude-dir=.next --exclude-dir=dist --exclude-dir=build 2>/dev/null \
        | head -100 | while IFS= read -r line; do
            file=$(echo "$line" | cut -d: -f1)
            rel="${file#$PROJECT_PATH/}"
            ln=$(echo "$line" | cut -d: -f2)
            content=$(echo "$line" | cut -d: -f3-)
            method=$(echo "$content" | grep -oE '\.(get|post|put|patch|delete|use)' | head -1 | tr -d '.' | tr 'a-z' 'A-Z')
            path=$(echo "$content" | grep -oE "['\"][^'\"]+['\"]" | head -1 | tr -d "'\"")
            [ -z "$method" ] || [ -z "$path" ] && continue
            emit_route "$rel" "$ln" "$method" "$path" "express-like"
        done

    # FastAPI / Flask
    grep -rnE "@(app|router|api|blueprint)\.(get|post|put|patch|delete|route)\(" "$PROJECT_PATH" \
        --include="*.py" --exclude-dir=venv --exclude-dir=.venv --exclude-dir=node_modules 2>/dev/null \
        | head -100 | while IFS= read -r line; do
            file=$(echo "$line" | cut -d: -f1)
            rel="${file#$PROJECT_PATH/}"
            ln=$(echo "$line" | cut -d: -f2)
            content=$(echo "$line" | cut -d: -f3-)
            method=$(echo "$content" | grep -oE '\.(get|post|put|patch|delete|route)' | head -1 | tr -d '.' | tr 'a-z' 'A-Z')
            path=$(echo "$content" | grep -oE "['\"][^'\"]+['\"]" | head -1 | tr -d "'\"")
            [ -z "$method" ] || [ -z "$path" ] && continue
            emit_route "$rel" "$ln" "$method" "$path" "python-decorator"
        done

    echo ""
    echo "  ]"
    echo "}"
} > "$ROUTE_REPORT"
log_run "route-discovery" "routes-discovered.json" 0

# ─── 3. Endpoint health probe (if URL) ──────────────────────────
if [ -n "$URL" ]; then
    ROOT="$(echo "$URL" | sed -E 's#(https?://[^/]+).*#\1#')"
    PROBE_FILE="$RAW_DIR/endpoint-probes.json"
    {
        echo "{"
        echo '  "root": "'"$ROOT"'",'
        echo '  "probes": ['
        first=1
        for path in / /health /healthz /api/health /api /api/v1; do
            full_url="$ROOT$path"
            response=$(curl -sLI -A "Mozilla/5.0" --max-time 8 -w "\nHTTP_CODE:%{http_code}\nTIME:%{time_total}\n" "$full_url" 2>/dev/null || echo "")
            code=$(echo "$response" | grep -oE 'HTTP_CODE:[0-9]+' | tail -1 | cut -d: -f2)
            time=$(echo "$response" | grep -oE 'TIME:[0-9.]+' | tail -1 | cut -d: -f2)
            ctype=$(echo "$response" | grep -i "^content-type:" | head -1 | cut -d: -f2- | xargs)
            [ $first -eq 1 ] && first=0 || echo ","
            printf '    {"path": "%s", "status": %s, "time_sec": %s, "content_type": %s}' \
                "$path" "${code:-0}" "${time:-0}" "$(json_str "${ctype:-}")"
        done
        echo ""
        echo "  ]"
        echo "}"
    } > "$PROBE_FILE"
    log_run "endpoint-probe" "endpoint-probes.json" 0
fi

# ─── 4. Auth/handler heuristics (server file scan) ──────────────
{
    echo "{"
    echo '  "auth_imports": ['
    first=1
    grep -rnE "from ['\"](next-auth|@clerk|@auth/|jsonwebtoken|passport|fastapi.security|flask_login|django.contrib.auth|@convex-dev/auth)" \
        "$PROJECT_PATH" \
        --include="*.ts" --include="*.tsx" --include="*.js" --include="*.py" \
        --exclude-dir=node_modules --exclude-dir=.next --exclude-dir=dist --exclude-dir=build 2>/dev/null \
        | head -50 | while IFS= read -r line; do
            file=$(echo "$line" | cut -d: -f1)
            rel="${file#$PROJECT_PATH/}"
            ln=$(echo "$line" | cut -d: -f2)
            [ $first -eq 1 ] && first=0 || echo ","
            printf '    {"file": "%s", "line": %s}' "$rel" "$ln"
        done
    echo ""
    echo '  ]'
    echo "}"
} > "$RAW_DIR/auth-imports.json"
log_run "auth-import-scanner" "auth-imports.json" 0

echo "api gather done"

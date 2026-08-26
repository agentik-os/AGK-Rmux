#!/usr/bin/env bash
# data.sh — data integrity gather. Schema discovery, type-defs, migration scan.
# Args: RAW_DIR PROJECT_PATH FILES URL
set -uo pipefail

RAW_DIR="${1:?missing raw_dir}"
PROJECT_PATH="${2:?missing project_path}"
FILES="${3:-}"
URL="${4:-}"
mkdir -p "$RAW_DIR"

# shellcheck disable=SC1091
source "$HOME/.omega/lib/audit-gather/_common.sh"

gather_header "data"

# ─── 1. Convex schema ──────────────────────────────────────────
if [ -d "$PROJECT_PATH/convex" ]; then
    SCHEMA="$PROJECT_PATH/convex/schema.ts"
    [ ! -f "$SCHEMA" ] && SCHEMA="$PROJECT_PATH/convex/schema.js"
    if [ -f "$SCHEMA" ]; then
        cp "$SCHEMA" "$RAW_DIR/convex-schema.txt"
        log_run "convex-schema-copy" "convex-schema.txt" 0
    fi
    # Inspect convex/ directory for tables
    {
        echo "{"
        echo '  "tables": ['
        first=1
        if [ -f "$SCHEMA" ]; then
            grep -nE "(defineTable|v\.\w+|\.index\(|\.searchIndex\()" "$SCHEMA" 2>/dev/null \
                | head -100 | while IFS=: read -r ln content; do
                    [ $first -eq 1 ] && first=0 || echo ","
                    printf '    {"line": %s, "snippet": %s}' "$ln" "$(json_str "$(echo "$content" | xargs)")"
                done
        fi
        echo ""
        echo "  ]"
        echo "}"
    } > "$RAW_DIR/convex-schema-meta.json"
    log_run "convex-schema-parser" "convex-schema-meta.json" 0
fi

# ─── 2. Prisma schema ──────────────────────────────────────────
PRISMA_SCHEMA=""
for p in prisma/schema.prisma schema.prisma; do
    if [ -f "$PROJECT_PATH/$p" ]; then
        PRISMA_SCHEMA="$PROJECT_PATH/$p"
        break
    fi
done
if [ -n "$PRISMA_SCHEMA" ]; then
    cp "$PRISMA_SCHEMA" "$RAW_DIR/prisma-schema.txt"
    log_run "prisma-schema-copy" "prisma-schema.txt" 0
    if [ -d "$PROJECT_PATH/node_modules" ] && check_npx_pkg "prisma"; then
        (cd "$PROJECT_PATH" && safe_cmd "$RAW_DIR/prisma-validate.txt" "prisma-validate" \
            npx --no-install prisma validate)
        (cd "$PROJECT_PATH" && safe_cmd "$RAW_DIR/prisma-format-check.txt" "prisma-format-check" \
            npx --no-install prisma format --check)
    fi
fi

# ─── 3. Drizzle / TypeORM / Sequelize / Mongoose schema scan ────
{
    echo "{"
    echo '  "schema_files": ['
    first=1
    grep -rlE "(pgTable|sqliteTable|mysqlTable)\(" "$PROJECT_PATH" \
        --include="*.ts" --include="*.js" \
        --exclude-dir=node_modules --exclude-dir=.next 2>/dev/null | while read -r f; do
            rel="${f#$PROJECT_PATH/}"
            [ $first -eq 1 ] && first=0 || echo ","
            printf '    {"orm": "drizzle", "file": "%s"}' "$rel"
        done
    grep -rlE "@(Entity|Schema|Table)\(" "$PROJECT_PATH" \
        --include="*.ts" \
        --exclude-dir=node_modules --exclude-dir=.next 2>/dev/null | while read -r f; do
            rel="${f#$PROJECT_PATH/}"
            [ $first -eq 1 ] && first=0 || echo ","
            printf '    {"orm": "typeorm-or-mongoose", "file": "%s"}' "$rel"
        done
    grep -rlE "mongoose\.Schema|new\s+Schema\(" "$PROJECT_PATH" \
        --include="*.ts" --include="*.js" \
        --exclude-dir=node_modules --exclude-dir=.next 2>/dev/null | while read -r f; do
            rel="${f#$PROJECT_PATH/}"
            [ $first -eq 1 ] && first=0 || echo ","
            printf '    {"orm": "mongoose", "file": "%s"}' "$rel"
        done
    echo ""
    echo "  ]"
    echo "}"
} > "$RAW_DIR/orm-schemas.json"
log_run "orm-schema-scanner" "orm-schemas.json" 0

# ─── 4. Migration directory scan ────────────────────────────────
{
    echo "{"
    echo '  "migration_dirs": ['
    first=1
    for d in prisma/migrations migrations db/migrations supabase/migrations alembic/versions; do
        if [ -d "$PROJECT_PATH/$d" ]; then
            count=$(find "$PROJECT_PATH/$d" -type f 2>/dev/null | wc -l)
            [ $first -eq 1 ] && first=0 || echo ","
            printf '    {"path": "%s", "file_count": %s}' "$d" "$count"
        fi
    done
    echo ""
    echo "  ]"
    echo "}"
} > "$RAW_DIR/migrations.json"
log_run "migration-scanner" "migrations.json" 0

# ─── 5. SQL file scan (schema.sql, init.sql, etc.) ──────────────
{
    echo "{"
    echo '  "sql_files": ['
    first=1
    find "$PROJECT_PATH" -maxdepth 4 -type f \( -name "*.sql" -o -name "schema.*" \) \
        -not -path "*/node_modules/*" -not -path "*/.git/*" -not -path "*/dist/*" 2>/dev/null \
        | head -25 | while read -r f; do
            rel="${f#$PROJECT_PATH/}"
            size=$(stat -c%s "$f" 2>/dev/null || echo 0)
            tables=$(grep -ciE "^[[:space:]]*CREATE TABLE" "$f" 2>/dev/null || echo 0)
            indexes=$(grep -ciE "^[[:space:]]*CREATE.*INDEX" "$f" 2>/dev/null || echo 0)
            [ $first -eq 1 ] && first=0 || echo ","
            printf '    {"file": "%s", "size": %s, "create_table_count": %s, "create_index_count": %s}' \
                "$rel" "$size" "$tables" "$indexes"
        done
    echo ""
    echo "  ]"
    echo "}"
} > "$RAW_DIR/sql-files.json"
log_run "sql-file-scanner" "sql-files.json" 0

# ─── 6. SQLite quick health (if a .db file exists) ──────────────
if check_tool "sqlite3"; then
    SQLITE_REPORT="$RAW_DIR/sqlite-health.json"
    {
        echo "{"
        echo '  "databases": ['
        first=1
        find "$PROJECT_PATH" -maxdepth 4 -type f \( -name "*.db" -o -name "*.sqlite" -o -name "*.sqlite3" \) \
            -not -path "*/node_modules/*" -not -path "*/.git/*" 2>/dev/null \
            | head -10 | while read -r dbfile; do
                rel="${dbfile#$PROJECT_PATH/}"
                size=$(stat -c%s "$dbfile" 2>/dev/null || echo 0)
                tables=$(timeout 10 sqlite3 "$dbfile" ".tables" 2>/dev/null | tr ' ' '\n' | grep -v '^$' | wc -l)
                pragma=$(timeout 10 sqlite3 "$dbfile" "PRAGMA integrity_check;" 2>/dev/null | head -1)
                [ $first -eq 1 ] && first=0 || echo ","
                printf '    {"file": "%s", "size": %s, "table_count": %s, "integrity": %s}' \
                    "$rel" "$size" "$tables" "$(json_str "${pragma:-unknown}")"
            done
        echo ""
        echo "  ]"
        echo "}"
    } > "$SQLITE_REPORT"
    log_run "sqlite-health-check" "sqlite-health.json" 0
fi

# ─── 7. TypeScript types/interfaces (data shapes) ───────────────
{
    echo "{"
    echo '  "type_defs": ['
    first=1
    grep -rnE "^(export )?(type|interface)\s+\w+" "$PROJECT_PATH" \
        --include="*.ts" --include="*.tsx" \
        --exclude-dir=node_modules --exclude-dir=.next --exclude-dir=dist 2>/dev/null \
        | head -200 | while IFS=: read -r file ln content; do
            rel="${file#$PROJECT_PATH/}"
            name=$(echo "$content" | grep -oE "(type|interface)\s+\w+" | awk '{print $2}')
            [ -z "$name" ] && continue
            [ $first -eq 1 ] && first=0 || echo ","
            printf '    {"file": "%s", "line": %s, "name": "%s"}' "$rel" "$ln" "$name"
        done
    echo ""
    echo "  ]"
    echo "}"
} > "$RAW_DIR/type-defs.json"
log_run "type-def-scanner" "type-defs.json" 0

echo "data gather done"

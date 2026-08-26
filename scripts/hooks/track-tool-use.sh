#!/usr/bin/env bash
# track-tool-use.sh — PostToolUse hook: per-tool tracking of dispatched agents.
#
# Logs every tool call from a dispatched oracle/worker to an append-only JSONL
# at ~/.omega/state/tracking/<session>.events.jsonl — the visibility /goal alone
# doesn't give. Shipped + installed by OmegaOS (rmux-native, no ~/.aisb dep).
#
# NEVER blocks a session: set +e, all failures swallowed, <5ms target.
# Hook stdin (Claude Code v2.1.139+): { session_id, tool_name, tool_input, tool_response }

set +e

TRACK_DIR="$HOME/.omega/state/tracking"
mkdir -p "$TRACK_DIR" 2>/dev/null

# rmux-native session detection (RMUX env first; fall back to the multiplexer).
SESSION="${RMUX_SESSION:-}"
[ -z "$SESSION" ] && SESSION=$(rmux display-message -p '#S' 2>/dev/null)
[ -z "$SESSION" ] && SESSION=$(tmux display-message -p '#S' 2>/dev/null)

# Skip interactive + non-dispatched sessions.
case "$SESSION" in
    Home|Home-2|Home-3|_menu|"") exit 0 ;;
esac
case "$SESSION" in
    *-worker-*|*-oracle|*-oracle-*|oracle-*|*-team-*|*-fix-*|*-dev-*|*-linear|godmode-*) ;;
    *) exit 0 ;;
esac

INPUT=""
[ ! -t 0 ] && INPUT=$(cat 2>/dev/null | head -c 16384)

TOOL=$(echo "$INPUT" | jq -r '.tool_name // empty' 2>/dev/null); [ -z "$TOOL" ] && TOOL="unknown"
INPUT_SIZE=$(echo "$INPUT" | jq -r '.tool_input | tostring | length' 2>/dev/null || echo 0)
OUTPUT_SIZE=$(echo "$INPUT" | jq -r '.tool_response | tostring | length' 2>/dev/null || echo 0)
ERROR=$(echo "$INPUT" | jq -r '.tool_response.is_error // false' 2>/dev/null || echo false)
TS=$(date -u +%Y-%m-%dT%H:%M:%S.%3NZ)
EVENT_FILE="$TRACK_DIR/${SESSION}.events.jsonl"

printf '{"ts":"%s","session":"%s","tool":"%s","input_size":%s,"output_size":%s,"error":%s}\n' \
    "$TS" "$SESSION" "$TOOL" "${INPUT_SIZE:-0}" "${OUTPUT_SIZE:-0}" "$ERROR" >> "$EVENT_FILE" 2>/dev/null

# Rotate beyond 5MB.
if [ -f "$EVENT_FILE" ]; then
    SIZE=$(stat -c %s "$EVENT_FILE" 2>/dev/null || echo 0)
    if [ "$SIZE" -gt 5242880 ]; then
        tail -c 4194304 "$EVENT_FILE" > "$EVENT_FILE.tmp" 2>/dev/null && mv "$EVENT_FILE.tmp" "$EVENT_FILE" 2>/dev/null
    fi
fi
exit 0

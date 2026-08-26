#!/usr/bin/env bash
# safe-npm-build.sh — Global mutex wrapper for `npm run build`.
#
# WHY: VPS has 2 cores. Concurrent `next build` saturates CPU (load 70+),
# corrupts .next/ via race, and kills workers pre-handoff. This wrapper
# serializes ALL builds VPS-wide under flock so only ONE runs at a time.
#
# Usage:
#   safe-npm-build.sh [cwd]
#     cwd defaults to $PWD
#     Runs `npm run build` in $cwd, returns its exit code.
#   Env:
#     SAFE_NPM_BUILD_TIMEOUT=1800   max seconds to wait for the lock (default 1800 = 30min)
#     SAFE_NPM_BUILD_CMD="npm run build"  override the build command (rarely needed)
#
# Lock: /tmp/omega-locks/global-next-build.lock (file lock, single flock fd)
# Log : ~/.omega/logs/safe-npm-build.log

set -uo pipefail

CWD="${1:-$PWD}"
LOCK_DIR="/tmp/omega-locks"
LOCK_FILE="$LOCK_DIR/global-next-build.lock"
LOG_FILE="$HOME/.omega/logs/safe-npm-build.log"
TIMEOUT="${SAFE_NPM_BUILD_TIMEOUT:-1800}"
BUILD_CMD="${SAFE_NPM_BUILD_CMD:-npm run build}"

mkdir -p "$LOCK_DIR" 2>/dev/null
chmod 0755 "$LOCK_DIR" 2>/dev/null || true
mkdir -p "$(dirname "$LOG_FILE")" 2>/dev/null

PROJECT=$(basename "$CWD")
MY_PID=$$
WAIT_START=$(date +%s)

log() {
    # timestamp | pid | project | message
    printf '%s | pid=%s | proj=%s | %s\n' \
        "$(date -Iseconds)" "$MY_PID" "$PROJECT" "$*" >> "$LOG_FILE"
}

# Detect existing holder for the friendly stderr message
holder_info() {
    if command -v fuser >/dev/null 2>&1; then
        local holder_pid
        holder_pid=$(fuser "$LOCK_FILE" 2>/dev/null | awk '{print $1}')
        if [ -n "${holder_pid:-}" ] && [ -d "/proc/$holder_pid" ]; then
            local started elapsed
            started=$(stat -c %Y "/proc/$holder_pid" 2>/dev/null || echo 0)
            elapsed=$(( $(date +%s) - started ))
            printf 'pid=%s elapsed=%ss' "$holder_pid" "$elapsed"
            return
        fi
    fi
    printf 'pid=? elapsed=?'
}

log "REQUEST cwd=$CWD timeout=${TIMEOUT}s cmd='$BUILD_CMD'"

# Check whether the lock is currently busy → print friendly message before waiting
if command -v flock >/dev/null 2>&1; then
    # Try a 0-timeout grab to detect contention
    if ! ( exec 9>"$LOCK_FILE"; flock -n 9 ) 2>/dev/null; then
        info=$(holder_info)
        echo "[safe-build] waiting for lock ($info)... (timeout ${TIMEOUT}s) cwd=$CWD" >&2
        log "WAITING $info"
    fi
else
    echo "[safe-build] WARNING: flock not available; running unsynchronized" >&2
    log "WARN no-flock running-unsynchronized"
fi

# Acquire the global build lock and run the build under it
exec 9>"$LOCK_FILE"
if command -v flock >/dev/null 2>&1; then
    if ! flock -w "$TIMEOUT" 9; then
        WAITED=$(( $(date +%s) - WAIT_START ))
        echo "[safe-build] FAILED to acquire lock within ${TIMEOUT}s (waited ${WAITED}s)" >&2
        log "TIMEOUT waited=${WAITED}s exit=124"
        exit 124
    fi
fi

WAITED=$(( $(date +%s) - WAIT_START ))
echo "[safe-build] acquired lock pid=$MY_PID (waited ${WAITED}s)" >&2
log "ACQUIRED waited=${WAITED}s"

START=$(date +%s)
# Run the build in the requested cwd
( cd "$CWD" && eval "$BUILD_CMD" )
RC=$?
DUR=$(( $(date +%s) - START ))

echo "[safe-build] released, exit=$RC duration=${DUR}s" >&2
log "RELEASED exit=$RC duration=${DUR}s"

exit $RC

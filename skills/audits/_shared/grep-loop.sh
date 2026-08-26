#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════
# grep-loop.sh — Bounded fix-and-verify loop (Pattern #3)
#
# Many of our audits (Step 8b) loop "fix → re-audit → fix → re-audit"
# without formal halt guarantees. Workers can spin indefinitely on
# subjective LLM judgment.
#
# This wrapper adds:
#   • OBJECTIVE verify-command gate (exit-code based, not LLM judgment)
#   • Max iteration budget (default 3)
#   • Scope-creep halt (cumulative diff > N lines = abort)
#   • Halt reasons written to .done.json so close-gate can decide
#
# USAGE:
#   grep-loop.sh <session> <verify_cmd> [--max=N] [--max-diff=N]
#
# EXIT CODES:
#   0  — verify_cmd passed (loop converged)
#   2  — halt: max iterations reached
#   3  — halt: scope creep (diff > max-diff)
#   4  — halt: worker died mid-loop
#   1  — usage error
#
# halt_reason is written to ~/.omega/state/grep-loop-<session>.halt.json
# ═══════════════════════════════════════════════════════════════
set -o pipefail

SESSION=""
VERIFY_CMD=""
MAX_ITER=3
MAX_DIFF=500
SLEEP_BETWEEN=15

while [ "$#" -gt 0 ]; do
    case "$1" in
        --max=*)      MAX_ITER="${1#--max=}" ;;
        --max-diff=*) MAX_DIFF="${1#--max-diff=}" ;;
        --sleep=*)    SLEEP_BETWEEN="${1#--sleep=}" ;;
        --help|-h)    sed -n '2,22p' "$0"; exit 0 ;;
        -*)           echo "ERR: unknown flag $1" >&2; exit 1 ;;
        *)
            if [ -z "$SESSION" ]; then SESSION="$1"
            elif [ -z "$VERIFY_CMD" ]; then VERIFY_CMD="$1"
            else echo "ERR: too many positional args" >&2; exit 1
            fi
            ;;
    esac
    shift
done

[ -z "$SESSION" ]    && { echo "Usage: grep-loop.sh <session> <verify_cmd> [--max=N] [--max-diff=N]" >&2; exit 1; }
[ -z "$VERIFY_CMD" ] && { echo "ERR: verify_cmd is required (objective gate)" >&2; exit 1; }

STATE_DIR="$HOME/.omega/state"
mkdir -p "$STATE_DIR"
HALT_FILE="$STATE_DIR/grep-loop-${SESSION}.halt.json"
LOG_FILE="$HOME/.omega/logs/grep-loop.log"
mkdir -p "$(dirname "$LOG_FILE")"

log() { printf '[%s] [%s] %s\n' "$(date -Iseconds)" "$SESSION" "$*" >> "$LOG_FILE"; }

write_halt() {
    local reason="$1" iter="$2" diff="${3:-0}"
    cat > "$HALT_FILE" <<EOF
{
  "session": "$SESSION",
  "halt_reason": "$reason",
  "iteration": $iter,
  "max_iter": $MAX_ITER,
  "cumulative_diff_lines": $diff,
  "max_diff": $MAX_DIFF,
  "verify_cmd": "$VERIFY_CMD",
  "halted_at": "$(date -Iseconds)"
}
EOF
    log "HALT: $reason (iter=$iter diff=$diff)"
}

cleanup_halt() { rm -f "$HALT_FILE"; }

# Initial baseline — git HEAD before any iteration
GIT_BASELINE=""
if git -C "${PWD:-.}" rev-parse HEAD >/dev/null 2>&1; then
    GIT_BASELINE=$(git rev-parse HEAD)
fi

log "START verify_cmd='$VERIFY_CMD' max_iter=$MAX_ITER max_diff=$MAX_DIFF"

ITER=0
while [ "$ITER" -lt "$MAX_ITER" ]; do
    ITER=$((ITER + 1))
    log "iteration $ITER/$MAX_ITER — running verify"

    # ─── Objective gate ───
    if eval "$VERIFY_CMD" >/dev/null 2>&1; then
        log "✓ verify PASSED at iteration $ITER"
        cleanup_halt
        exit 0
    fi

    # ─── Scope-creep gate ───
    if [ -n "$GIT_BASELINE" ]; then
        local_diff_lines=$(git diff "$GIT_BASELINE"..HEAD 2>/dev/null | wc -l | tr -d ' ')
        if [ "$local_diff_lines" -gt "$MAX_DIFF" ]; then
            write_halt "scope_creep" "$ITER" "$local_diff_lines"
            exit 3
        fi
    fi

    # ─── Liveness gate ───
    # OmegaOS is rmux-native; the old optional provider-specific helper and
    # tmux fallback made fresh installs silently skip this gate.
    if ! command -v omega >/dev/null 2>&1 \
        || ! omega capture "$SESSION" >/dev/null 2>&1; then
        write_halt "worker_died" "$ITER"
        exit 4
    fi

    # ─── Re-dispatch fix instruction ───
    if omega capture "$SESSION" >/dev/null 2>&1; then
        local_diff_summary=$(git diff "$GIT_BASELINE"..HEAD --shortstat 2>/dev/null | head -1)
        FIX_MSG="Verify command failed: $VERIFY_CMD
Current diff: $local_diff_summary
Iteration: $ITER of $MAX_ITER

Fix the failing assertions ONLY. Don't expand scope. Don't rename files.
Only modify files already in your brief.files_owned scope.
Re-run verify before calling worker-mark-done.sh."
        omega send "$SESSION" "$FIX_MSG" >/dev/null 2>&1
    else
        write_halt "session_missing" "$ITER"
        exit 4
    fi

    # ─── Wait before next check ───
    sleep "$SLEEP_BETWEEN"
done

write_halt "max_iter" "$ITER"
exit 2

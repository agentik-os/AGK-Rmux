#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════
# audit-notify.sh — Telegram notification helper for Quality Arsenal audits
#
# Usage: audit-notify.sh <audit-name> <event> [details]
#
# Events:
#   start     — audit begins (scope info)
#   progress  — phase milestone (every 3 phases)
#   iteration — fix-and-reaudit cycle (score trajectory)
#   verdict   — final score + path to verdict.md
#   abort     — audit aborted (reason)
#   sos       — 5-iter cap / lock collision / unrecoverable error
#
# Requires: telegram CLI at ~/.local/bin/telegram
# ═══════════════════════════════════════════════════════════════

set -euo pipefail

AUDIT="${1:?Usage: audit-notify.sh <audit-name> <event> [details]}"
EVENT="${2:?Usage: audit-notify.sh <audit-name> <event> [details]}"
DETAILS="${3:-}"
if [[ ! "$EVENT" =~ ^(start|progress|iteration|verdict|abort|sos)$ ]]; then
    echo "[audit-notify] unknown event: $EVENT" >&2
    exit 2
fi
if [ ${#DETAILS} -gt 8192 ]; then
    echo "[audit-notify] details exceed 8192 characters" >&2
    exit 2
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OMEGA_ROOT="${OMEGA_DIR:-$HOME/.omega}"
REGISTRY="$OMEGA_ROOT/skills/audits/registry.toml"
[ -f "$REGISTRY" ] || REGISTRY="$SCRIPT_DIR/../registry.toml"
if [ ! -f "$REGISTRY" ]; then
    echo "[audit-notify] canonical audit registry not found" >&2
    exit 2
fi
CANONICAL_AUDIT="$(python3 - "$REGISTRY" "$AUDIT" <<'PY'
import re
import sys
import tomllib

with open(sys.argv[1], "rb") as handle:
    registry = tomllib.load(handle)
entries = registry.get("audits") if isinstance(registry, dict) else None
if not isinstance(entries, list) or registry.get("meta", {}).get("total_audits") != len(entries):
    raise SystemExit(1)
ids = []
for entry in entries:
    audit_id = entry.get("id") if isinstance(entry, dict) else None
    if not isinstance(audit_id, str) or not re.fullmatch(r"[a-z0-9-]+audit", audit_id):
        raise SystemExit(1)
    ids.append(audit_id)
if len(ids) != len(set(ids)):
    raise SystemExit(1)
requested = sys.argv[2].strip().lower()
if not re.fullmatch(r"[a-z0-9-]+", requested):
    raise SystemExit(1)
for candidate in (requested, f"{requested}audit"):
    if candidate in ids:
        print(candidate)
        raise SystemExit
raise SystemExit(1)
PY
)" || {
    echo "[audit-notify] unknown audit or invalid registry: $AUDIT" >&2
    exit 2
}

# Telegram target — read from env, never hardcoded.
# If unset, notifications are skipped entirely (no DM to anyone).
TG_USER="${OMEGA_TELEGRAM_USER_ID:-}"
TELEGRAM="${HOME}/.local/bin/telegram"

# No target configured → skip silently (don't DM a maintainer, don't fail the audit)
if [[ -z "$TG_USER" ]]; then
    exit 0
fi

# Check telegram CLI exists
if [[ ! -x "$TELEGRAM" ]]; then
    echo "[audit-notify] WARNING: telegram CLI not found at $TELEGRAM. Notification skipped." >&2
    exit 0  # Don't fail the audit because notifications are down
fi

# Format message per event type
case "$EVENT" in
    start)
        MSG="🚦 /${CANONICAL_AUDIT} started — ${DETAILS:-no scope specified}"
        ;;
    progress)
        MSG="📊 /${CANONICAL_AUDIT} ${DETAILS:-progress update}"
        ;;
    iteration)
        MSG="🔁 /${CANONICAL_AUDIT} ${DETAILS:-fix-and-reaudit cycle}"
        ;;
    verdict)
        MSG="🎯 /${CANONICAL_AUDIT} done — ${DETAILS:-see audits/.${CANONICAL_AUDIT}/verdict.md}"
        ;;
    abort)
        MSG="🛑 /${CANONICAL_AUDIT} aborted — ${DETAILS:-unknown reason}"
        ;;
    sos)
        MSG="🆘 /${CANONICAL_AUDIT} SOS — ${DETAILS:-unrecoverable error}"
        ;;
esac

# Send notification (fire-and-forget, don't block the audit)
"$TELEGRAM" notify --user "$TG_USER" "$MSG" 2>/dev/null || {
    echo "[audit-notify] WARNING: telegram send failed for /${AUDIT} ${EVENT}" >&2
}

# Log to audit's own telemetry
AUDIT_DIR="audits/.${CANONICAL_AUDIT}"
NOTIFICATION_LOG="${AUDIT_DIR}/notifications.log"
if [[ -d "$AUDIT_DIR" && ! -L "$AUDIT_DIR" && ! -L "$NOTIFICATION_LOG" ]]; then
    if command -v flock >/dev/null 2>&1; then
        exec {NOTIFY_LOCK_FD}>>"$NOTIFICATION_LOG"
        if flock -w 2 "$NOTIFY_LOCK_FD"; then
            printf '%s %s %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$EVENT" "$DETAILS" >&"$NOTIFY_LOCK_FD"
        else
            echo "[audit-notify] WARNING: notification telemetry lock timed out" >&2
        fi
    else
        echo "[audit-notify] WARNING: flock unavailable; notification telemetry skipped" >&2
    fi
fi

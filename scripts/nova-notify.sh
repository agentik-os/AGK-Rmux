#!/usr/bin/env bash
# nova-notify.sh — Nova announces what she's doing. Called by Nova whenever she
# launches/acts (dispatch, workflow, generation, email, publish, account action).
# Destination: her companion bot DM by default; override with NOVA_NOTIFY_TOPIC
# (a group thread) + NOVA_NOTIFY_CHAT in nova-secrets.env.
set -u
OMEGA="${OMEGA_DIR:-$HOME/.omega}"
[ -f "$OMEGA/nova-secrets.env" ] && set -a && . "$OMEGA/nova-secrets.env" && set +a
MSG="${*:-}"; [ -n "$MSG" ] || exit 0
CHAT="${NOVA_NOTIFY_CHAT:-${NOVA_CHAT_ID:-}}"
[ -n "$CHAT" ] || exit 0
TOKEN="$(python3 -c 'import json,os
try:
    b=json.load(open(os.path.expanduser("~/.omega/agent-bots.json")))
    print(next(x["token"] for x in b.values() if isinstance(x,dict) and x.get("kind")=="companion" and x.get("token")))
except Exception: pass')"
[ -n "$TOKEN" ] || exit 0
ARGS=(-d chat_id="$CHAT" --data-urlencode text="$MSG")
[ -n "${NOVA_NOTIFY_TOPIC:-}" ] && ARGS+=(-d message_thread_id="$NOVA_NOTIFY_TOPIC")
curl -s -X POST "https://api.telegram.org/bot$TOKEN/sendMessage" "${ARGS[@]}" >/dev/null

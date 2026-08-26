#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════
# OmegaOS — canonical alert sender (Telegram "Alerts 🚨" topic)
# ───────────────────────────────────────────────────────────────────────────
# ONE funnel for every operational alert (stuck oracle, self-heal, token
# refresh failure, …): they all land in the dedicated Alerts forum topic —
# NEVER in the Atlas topic (Atlas = briefing + oracle reports + off-project
# work) and never scattered in the DM.
#
# The Alerts topic is UNDELETABLE by design: if it is missing or was deleted
# in the group, this script recreates it on the fly, persists the new id in
# telegram-groups.json, and resends. Fallback chain: alerts topic → plain-text
# retry (HTML parse errors die identically everywhere, so strip the tags like
# the bot's edit() does) → DM. A fully undeliverable alert exits 1 and is
# logged to ~/.omega/logs/alerts.log — never silently lost.
#
# Usage: omega-alert-send.sh "<HTML text>"
# ═══════════════════════════════════════════════════════════════════════════
set -uo pipefail
OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
TG_TOML="$OMEGA_DIR/telegram.toml"
GFILE="$OMEGA_DIR/telegram-groups.json"
MSG="${1:-}"
[ -n "$MSG" ] || { echo "usage: omega-alert-send.sh \"<html text>\"" >&2; exit 2; }

TOKEN="$(grep -E '^[[:space:]]*bot_token' "$TG_TOML" 2>/dev/null | head -1 | cut -d'"' -f2)"
DM="$(grep -E '^[[:space:]]*chat_id' "$TG_TOML" 2>/dev/null | head -1 | grep -oE '\-?[0-9]+' | head -1)"
[ -n "${TOKEN:-}" ] || exit 0
HUB="$(python3 -c "import json;print(json.load(open('$GFILE')).get('hub',''))" 2>/dev/null || true)"
ALERTS="$(python3 -c "import json;print(json.load(open('$GFILE')).get('alerts_topic','') or '')" 2>/dev/null || true)"

api() { curl -s --max-time 15 "https://api.telegram.org/bot${TOKEN}/$1" "${@:2}"; }

create_alerts_topic() { # → echoes new thread id (and persists it), empty on failure
    local r tid
    r="$(api createForumTopic -d "chat_id=$HUB" --data-urlencode "name=Alerts 🚨" -d "icon_color=16478047")"
    tid="$(printf '%s' "$r" | python3 -c "import json,sys;d=json.load(sys.stdin);print(d['result']['message_thread_id'] if d.get('ok') else '')" 2>/dev/null)"
    [ -n "$tid" ] || return 1
    python3 - "$tid" "$GFILE" <<'PY' 2>/dev/null
import json,sys
tid, p = sys.argv[1], sys.argv[2]
g=json.load(open(p))
# Drop any stale alerts mapping, then record the fresh one.
g["topics"]={k:v for k,v in (g.get("topics") or {}).items() if str(v).lower()!="alerts"}
g["alerts_topic"]=int(tid); g["topics"][tid]="alerts"
json.dump(g,open(p,"w"),indent=2)
PY
    echo "$tid"
}

send_to() { # $1=chat $2=thread("" = none) → echoes API response
    local a=(--data-urlencode "chat_id=$1" --data-urlencode "text=$MSG" --data-urlencode "parse_mode=HTML" --data-urlencode "disable_web_page_preview=true")
    [ -n "${2:-}" ] && a+=(--data-urlencode "message_thread_id=$2")
    api sendMessage "${a[@]}"
}

# Plain-text fallback: a caller passing unescaped <>& makes Telegram reject the
# HTML ("can't parse entities") on EVERY chat — strip the tags and drop parse_mode
# so the alert still lands (mirrors the bot's send()/edit() fallback).
send_plain() { # $1=chat $2=thread("" = none) → echoes API response
    local txt; txt="$(printf '%s' "$MSG" | sed 's/<[^>]*>//g')"
    local a=(--data-urlencode "chat_id=$1" --data-urlencode "text=$txt" --data-urlencode "disable_web_page_preview=true")
    [ -n "${2:-}" ] && a+=(--data-urlencode "message_thread_id=$2")
    api sendMessage "${a[@]}"
}

# Every failed API response is appended here so a lost alert is diagnosable.
log_fail() { # $1=target $2=response
    mkdir -p "$OMEGA_DIR/logs"
    echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) alert-send FAILED ($1): ${2:-no response}" >> "$OMEGA_DIR/logs/alerts.log"
}

if [ -n "${HUB:-}" ]; then
    # No alerts topic on file → create it first (undeletable invariant).
    [ -n "${ALERTS:-}" ] || ALERTS="$(create_alerts_topic || true)"
    if [ -n "${ALERTS:-}" ]; then
        res="$(send_to "$HUB" "$ALERTS")"
        if printf '%s' "$res" | grep -q '"ok":true'; then exit 0; fi
        # Topic deleted in the group → recreate + resend (the auto-heal).
        if printf '%s' "$res" | grep -qiE 'thread not found|TOPIC_DELETED|TOPIC_ID_INVALID'; then
            ALERTS="$(create_alerts_topic || true)"
            if [ -n "${ALERTS:-}" ]; then
                res="$(send_to "$HUB" "$ALERTS")"
                printf '%s' "$res" | grep -q '"ok":true' && exit 0
            fi
        # Malformed HTML dies identically on hub AND DM ($MSG unchanged) — retry
        # tags-stripped in place before falling back.
        elif printf '%s' "$res" | grep -qi "can't parse"; then
            res="$(send_plain "$HUB" "$ALERTS")"
            printf '%s' "$res" | grep -q '"ok":true' && exit 0
        fi
        log_fail "hub $HUB topic $ALERTS" "$res"
    fi
fi
# Last resort: operator DM (never lose an alert) — HTML first, then plain text.
if [ -n "${DM:-}" ]; then
    res="$(send_to "$DM" "")"
    printf '%s' "$res" | grep -q '"ok":true' && exit 0
    res="$(send_plain "$DM" "")"
    printf '%s' "$res" | grep -q '"ok":true' && exit 0
    log_fail "DM $DM" "$res"
fi
exit 1

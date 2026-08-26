#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════
# OmegaOS — Atlas morning briefing (daily portfolio digest)
# ───────────────────────────────────────────────────────────────────────────
# Once a day (cron OMEGA-CRON-ATLAS-BRIEF), Atlas posts a portfolio health digest
# to the Atlas topic: per managed project — uncommitted work, unpushed commits,
# last mission outcome, stuck/active oracles. Symbol aesthetic, no emoji. Idempotent
# via a 20h marker (~/.omega/state/atlas-brief.last-run) so a per-minute or hourly
# cron only fires it once a day around the scheduled hour.
# ═══════════════════════════════════════════════════════════════════════════
set -uo pipefail
OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
STATE="$OMEGA_DIR/state"
TG_TOML="$OMEGA_DIR/telegram.toml"
GFILE="$OMEGA_DIR/telegram-groups.json"
PROJECTS_FILE="$OMEGA_DIR/projects.json"
MARKER="$STATE/atlas-brief.last-run"

[ -f "$PROJECTS_FILE" ] || exit 0
TOKEN="$(grep -E '^[[:space:]]*bot_token' "$TG_TOML" 2>/dev/null | head -1 | cut -d'"' -f2)"
DM="$(grep -E '^[[:space:]]*chat_id' "$TG_TOML" 2>/dev/null | head -1 | grep -oE '\-?[0-9]+' | head -1)"
[ -n "${TOKEN:-}" ] && [ -n "${DM:-}" ] || exit 0

# Idempotent: skip if briefed < 20h ago.
if [ -f "$MARKER" ]; then
    now=$(date +%s); m=$(stat -c%Y "$MARKER" 2>/dev/null || stat -f%m "$MARKER" 2>/dev/null || echo 0)
    [ $((now - m)) -lt 72000 ] && exit 0
fi

HUB="$(python3 -c "import json;print(json.load(open('$GFILE')).get('hub',''))" 2>/dev/null || true)"
ATLAS="$(python3 -c "import json;print(json.load(open('$GFILE')).get('atlas_topic','') or '')" 2>/dev/null || true)"
esc() { printf '%s' "$1" | sed 's/&/\&amp;/g; s/</\&lt;/g; s/>/\&gt;/g'; }

clean=0; attention=""
while IFS='|' read -r name path; do
    [ -z "$name" ] || [ -z "$path" ] && continue
    [ -d "$path" ] || continue
    git -C "$path" rev-parse --is-inside-work-tree >/dev/null 2>&1 || continue
    flags=""
    # uncommitted
    [ -n "$(git -C "$path" status --porcelain 2>/dev/null)" ] && flags="${flags} ~uncommitted"
    # unpushed
    up="$(git -C "$path" rev-list --count @{u}..HEAD 2>/dev/null || echo 0)"
    [ "$up" -gt 0 ] 2>/dev/null && flags="${flags} ↑${up}-unpushed"
    # last mission outcome
    last=""; for df in "$STATE"/oracle-${name}*.done.json; do [ -e "$df" ] || continue; [ -z "$last" ] || [ "$df" -nt "$last" ] && last="$df"; done
    if [ -n "$last" ]; then
        st="$(python3 -c "import json;print(json.load(open('$last')).get('status','?'))" 2>/dev/null)"
        case "$st" in failed) flags="${flags} ✗mission-failed";; blocked) flags="${flags} ‖blocked";; pending) flags="${flags} …incomplete";; esac
    fi
    # active oracle (state.json without done.json)
    for sf in "$STATE"/oracle-${name}*.state.json; do [ -e "$sf" ] || continue; b="$(basename "$sf" .state.json)"; [ -f "$STATE/$b.done.json" ] || flags="${flags} ▸oracle-active"; break; done
    if [ -z "$flags" ]; then clean=$((clean+1)); else attention="${attention}
✗ <b>$(esc "$name")</b> —$(esc "$flags")"; fi
done < <(python3 -c "import json;d=json.load(open('$PROJECTS_FILE'));[print(p['name']+'|'+p['path']) for p in d.get('projects',[])]" 2>/dev/null)

total=$(python3 -c "import json;print(len(json.load(open('$PROJECTS_FILE')).get('projects',[])))" 2>/dev/null || echo 0)
date_str="$(date -u +%d/%m)"
if [ -n "$attention" ]; then
    msg="▸ <b>Atlas briefing</b> · ${date_str}
${clean}/${total} projects clean · needs a look:
${attention}"
else
    msg="✓ <b>Atlas briefing</b> · ${date_str}
Portfolio clean — ${total} projects, nothing to report."
fi

if [ -n "${ATLAS:-}" ] && [ -n "${HUB:-}" ]; then chat="$HUB"; th="$ATLAS"; else chat="$DM"; th=""; fi
a=(--data-urlencode "chat_id=$chat" --data-urlencode "text=$msg" --data-urlencode "parse_mode=HTML")
[ -n "$th" ] && a+=(--data-urlencode "message_thread_id=$th")
# curl exits 0 even on an HTTP 400 (parse error, bot kicked, dead thread id), so the
# marker must be gated on the API's ok:true — not on curl — or a rejected brief
# silently burns the day's 20h window. On hub/topic failure, retry once to the DM.
res="$(curl -s "https://api.telegram.org/bot${TOKEN}/sendMessage" "${a[@]}" 2>/dev/null)"
if ! printf '%s' "$res" | grep -q '"ok":true' && [ "$chat" != "$DM" ]; then
    res="$(curl -s "https://api.telegram.org/bot${TOKEN}/sendMessage" --data-urlencode "chat_id=$DM" --data-urlencode "text=$msg" --data-urlencode "parse_mode=HTML" 2>/dev/null)"
fi
if printf '%s' "$res" | grep -q '"ok":true'; then
    : > "$MARKER"
    echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) atlas-brief sent (${clean}/${total} clean)"
else
    mkdir -p "$OMEGA_DIR/logs"
    echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) atlas-brief FAILED: ${res:-no response}" >> "$OMEGA_DIR/logs/atlas-brief.log"
    exit 1
fi

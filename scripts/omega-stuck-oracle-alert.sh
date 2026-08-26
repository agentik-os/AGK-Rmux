#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════
# OmegaOS — stuck-oracle alert (1-min cron)
# ───────────────────────────────────────────────────────────────────────────
# A dispatched oracle that hasn't updated its progress (oracle-<key>.progress.json)
# OR its debug log (oracle-<key>.debug.log) for STALE_MIN minutes, and hasn't written
# a done.json, is probably stuck. Ping the operator ONCE (marker) in the project's
# Telegram topic (or the Atlas topic). Reuses the omega-done-notify send pattern.
# Cron marker: OMEGA-CRON-STUCK-ALERT-v1.
# ═══════════════════════════════════════════════════════════════════════════
set -uo pipefail
OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
STATE="$OMEGA_DIR/state"
TG_TOML="$OMEGA_DIR/telegram.toml"
GFILE="$OMEGA_DIR/telegram-groups.json"
STALE_MIN="${OMEGA_STUCK_MIN:-20}"

[ -d "$STATE" ] || exit 0
TOKEN="$(grep -E '^[[:space:]]*bot_token' "$TG_TOML" 2>/dev/null | head -1 | cut -d'"' -f2)"
DM="$(grep -E '^[[:space:]]*chat_id' "$TG_TOML" 2>/dev/null | head -1 | grep -oE '\-?[0-9]+' | head -1)"
[ -n "${TOKEN:-}" ] && [ -n "${DM:-}" ] || exit 0
HUB="$(python3 -c "import json;print(json.load(open('$GFILE')).get('hub',''))" 2>/dev/null || true)"
ATLAS="$(python3 -c "import json;print(json.load(open('$GFILE')).get('atlas_topic','') or '')" 2>/dev/null || true)"

send_tg() { # $1=chat $2=text $3=thread
    local a=(--data-urlencode "chat_id=$1" --data-urlencode "text=$2" --data-urlencode "parse_mode=HTML")
    [ -n "${3:-}" ] && a+=(--data-urlencode "message_thread_id=$3")
    curl -s "https://api.telegram.org/bot${TOKEN}/sendMessage" "${a[@]}" >/dev/null 2>&1
}
topic_for() {
    [ -f "$GFILE" ] || return 0
    python3 - "$GFILE" "$1" <<'PY' 2>/dev/null
import json,sys,re
try:
    g=json.load(open(sys.argv[1])); want=sys.argv[2].lower()
    cands=[want, re.sub(r'-[0-9]+$','',want)]
    for tid,name in (g.get("topics") or {}).items():
        if str(name).lower() in cands: print(tid); break
except Exception: pass
PY
}
now=$(date +%s)
RMUX_BIN="${RMUX_BIN:-$(command -v rmux || echo "$HOME/.local/bin/rmux")}"

# Each live oracle is an oracle-<key>.state.json without a matching done.json.
for sf in "$STATE"/oracle-*.state.json; do
    [ -e "$sf" ] || continue
    base="$(basename "$sf" .state.json)"           # oracle-<key>
    key="${base#oracle-}"
    # Skip stripped double-prefix variants (oracle-oracle-…).
    [ -f "$STATE/${base}.done.json" ] && continue   # already finished
    marker="$STATE/${base}.stuck-alerted"
    [ -f "$marker" ] && continue                     # alerted once already

    # A DEAD session can't be "stuck" — only alert for oracles the operator
    # can actually see and act on. Leftover state.json of killed/crashed
    # oracles used to fire "oracle bloqué depuis N min" forever (patrol's GC
    # re-armed the marker hourly while the state file stayed in place);
    # dead-oracle recovery and state GC are patrol's job, not this alert's.
    "$RMUX_BIN" has-session -t "$base" >/dev/null 2>&1 || continue

    # Last activity = newest mtime among progress.json, debug.log, state.json.
    newest=0
    for f in "$STATE/${base}.progress.json" "$STATE/${base}.debug.log" "$sf"; do
        [ -e "$f" ] || continue
        m="$(stat -c%Y "$f" 2>/dev/null || stat -f%m "$f" 2>/dev/null || echo 0)"
        [ "$m" -gt "$newest" ] && newest="$m"
    done
    [ "$newest" -gt 0 ] || continue
    idle=$(( (now - newest) / 60 ))
    [ "$idle" -lt "$STALE_MIN" ] && continue          # still active

    project="$(printf '%s' "$key" | sed -E 's/-[0-9]+$//')"

    # Mission progress, if the session mirrored its tracked plan out
    # (omega-plan-mirror.sh, PostToolUse on the plan tools). Turns "stalled"
    # into "stalled at 3 of 7, and here is what it still owes" — the difference
    # between an alert you can act on and one you can only acknowledge.
    plan_line="$(python3 - "$STATE" "$project" <<'PY' 2>/dev/null || true
import glob, json, os, sys
state, project = sys.argv[1], sys.argv[2].lower()
best = None
for p in glob.glob(os.path.join(state, "plan-*.json")):
    try:
        d = json.load(open(p))
    except Exception:
        continue
    if str(d.get("project", "")).lower() != project:
        continue
    if best is None or d.get("updated_at", 0) > best.get("updated_at", 0):
        best = d
if not best or not best.get("total"):
    raise SystemExit(0)
line = "Plan : <b>%d/%d</b> tâches faites." % (best.get("done", 0), best["total"])
items = best.get("open_items") or []
if items:
    line += " Reste : " + " · ".join(str(i)[:60] for i in items[:3])
    if len(items) > 3:
        line += " (+%d)" % (len(items) - 3)
print(line)
PY
)"
    msg="‖ <b>${project}</b> · oracle bloqué ?
Pas d'activité depuis <b>${idle} min</b> (oracle-${key})."
    [ -n "$plan_line" ] && msg="$msg
${plan_line}"
    msg="$msg
Inspecte : <code>omega capture oracle-${key}</code> · ferme : <code>omega kill oracle-${key}</code>"
    # Operational alert → the dedicated Alerts topic via the canonical sender
    # (auto-recreates the topic if deleted, DM fallback). NEVER the Atlas topic.
    bash "$OMEGA_DIR/bin/omega-alert-send.sh" "$msg"
    : > "$marker"
    echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) stuck-alert ${base} (${idle}m idle)"
done

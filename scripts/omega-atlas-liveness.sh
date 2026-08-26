#!/usr/bin/env bash
# OmegaOS Telegram liveness watchdog (cron, every 2 min).
# ───────────────────────────────────────────────────────────────────────────
# WHY: every OmegaOS Telegram bot — the master (Atlas) AND each project agent
# bot — runs the SAME 261KB core with a SINGLE poll loop. Every inbound update
# is processed with `await` INSIDE that loop, and some handler paths (voice
# transcription, file download) do an un-timed `fetch` with no watchdog. One
# stalled fetch blocks the loop forever: the bun process stays alive (the
# pollProgress/pollReports setIntervals keep the event loop up), so systemd
# still reports the unit `active` and `omega doctor` (which only checks the
# systemd state) reports "healthy". The bot goes DEAF but nothing restarts it —
# the operator finds it "locked" again and again.
#
# This probe closes that blind spot WITHOUT touching the bot core:
# getWebhookInfo (which does NOT conflict with getUpdates long-polling) exposes
# pending_update_count. A healthy bot drains pending to 0 within a second or two.
# If pending stays > 0 and is NOT draining across a ~25s window AND the bot is
# not legitimately busy (no claude/codex child), the poll loop is stuck → restart.
#
# COVERAGE: master + every agent bot in agent-bots.json. It used to watch the
# master ONLY, which left the project bots running the exact un-timed-fetch code
# this watchdog exists for, with nothing to recover them (found 2026-07-23: six
# agent bots had been up since 2026-07-09, unprotected).
#
# Guards against false positives / restart loops:
#   - busy-guard: never restart a bot with a live claude/codex child (a legit
#     long task can hold pending > 0 for minutes).
#   - drain-guard: only restart if pending2 >= pending1 (not consuming at all).
#   - cooldown: at most one restart per 15 min, PER UNIT.
#   - circuit breaker: after 3 restarts in 60 min the unit is left untouched
#     until an operator explicitly resets it with `--reset <unit>`.
#   - ONE shared 25s window for all bots, so covering N bots costs ~30s total
#     and never overruns the 2-minute cron interval.
set -uo pipefail
umask 077

API_BASE="${OMEGA_TG_API_BASE:-https://api.telegram.org/bot}"
OMEGA_ROOT="${OMEGA_DIR:-$HOME/.omega}"
DRAIN_SECONDS="${OMEGA_TG_DRAIN_SECONDS:-25}"
COOLDOWN_SECONDS="${OMEGA_TG_COOLDOWN_SECONDS:-900}"
WINDOW_SECONDS="${OMEGA_TG_WINDOW_SECONDS:-3600}"
MAX_RESTARTS="${OMEGA_TG_MAX_RESTARTS:-3}"
LOCK_STALE_SECONDS="${OMEGA_TG_LOCK_STALE_SECONDS:-120}"
STATE_DIR="${OMEGA_TG_STATE_DIR:-$OMEGA_ROOT/state}"
ALERT_BIN="${OMEGA_TG_ALERT_BIN:-$OMEGA_ROOT/bin/omega-alert-send.sh}"
case "$OMEGA_ROOT" in
    /*) ;;
    *) echo "tg-liveness: OMEGA_DIR must be an absolute path" >&2; exit 2 ;;
esac
if ! [[ "$DRAIN_SECONDS" =~ ^[0-9]+$ && "$COOLDOWN_SECONDS" =~ ^[0-9]+$ \
    && "$WINDOW_SECONDS" =~ ^[0-9]+$ && "$MAX_RESTARTS" =~ ^[1-9][0-9]*$ \
    && "$LOCK_STALE_SECONDS" =~ ^[1-9][0-9]*$ ]]; then
    echo "tg-liveness: invalid numeric circuit-breaker configuration" >&2
    exit 2
fi
if [ "$DRAIN_SECONDS" -gt 60 ]; then
    echo "tg-liveness: drain window must not exceed 60 seconds" >&2
    exit 2
fi
for dependency in curl systemctl timeout ps awk grep mktemp; do
    command -v "$dependency" >/dev/null 2>&1 || {
        echo "tg-liveness: missing required command: $dependency" >&2
        exit 2
    }
done
if [ -L "$STATE_DIR" ] || { [ -e "$STATE_DIR" ] && [ ! -d "$STATE_DIR" ]; }; then
    echo "tg-liveness: refusing non-directory or symlink state path" >&2
    exit 2
fi
mkdir -p "$STATE_DIR" || {
    echo "tg-liveness: cannot create state directory" >&2
    exit 2
}
chmod 0700 "$STATE_DIR" || {
    echo "tg-liveness: cannot secure state directory" >&2
    exit 2
}

valid_unit() {
    [[ "$1" =~ ^omega-tg-(bot|agent-[a-zA-Z0-9_-]+)\.service$ ]]
}

reset_circuit() {
    local requested="${1:-}"
    if [ "$requested" = "all" ]; then
        local state
        for state in \
            "$STATE_DIR"/tg-liveness-circuit-omega-tg-* \
            "$STATE_DIR"/tg-liveness-restarts-omega-tg-* \
            "$STATE_DIR"/tg-liveness-restart-omega-tg-* \
            "$STATE_DIR"/tg-liveness-lock-omega-tg-*; do
            if [ -d "$state" ]; then
                rm -f -- "$state/owner" 2>/dev/null || true
                rmdir -- "$state" 2>/dev/null || true
            elif [ -e "$state" ]; then rm -f -- "$state"
            fi
        done
        echo "tg-liveness: all Telegram circuit breakers reset"
        return 0
    fi
    [[ "$requested" == *.service ]] || requested="${requested}.service"
    valid_unit "$requested" || {
        echo "usage: omega-atlas-liveness.sh --reset <omega-tg-bot.service|omega-tg-agent-ID.service|all>" >&2
        return 2
    }
    local stem="${requested%.service}"
    rm -f -- \
        "$STATE_DIR/tg-liveness-circuit-$stem" \
        "$STATE_DIR/tg-liveness-restarts-$stem" \
        "$STATE_DIR/tg-liveness-restart-$stem"
    rm -f -- "$STATE_DIR/tg-liveness-lock-$stem/owner" 2>/dev/null || true
    rmdir -- "$STATE_DIR/tg-liveness-lock-$stem" 2>/dev/null || true
    echo "tg-liveness: circuit breaker reset for $requested"
}

if [ "${1:-}" = "--reset" ]; then
    reset_circuit "${2:-}"
    exit $?
fi

pending() {  # $1 = token
    [[ "$1" =~ ^[0-9]+:[A-Za-z0-9_-]+$ ]] || return 1
    local response
    response="$(printf 'url = "%s"\n' "${API_BASE}${1}/getWebhookInfo" \
        | curl --fail --silent --max-time 10 --config - 2>/dev/null)" || return 1
    printf '%s' "$response" | grep -qE '"ok"[[:space:]]*:[[:space:]]*true' || return 1
    printf '%s' "$response" \
        | grep -oP '"pending_update_count"[[:space:]]*:[[:space:]]*\K[0-9]+' \
        | head -1
}

# ── Build the watch list: "unit<TAB>token<TAB>label" ─────────────────────────
WATCH=""

TG="$OMEGA_ROOT/telegram.toml"
if [ -f "$TG" ]; then
    MASTER_TOKEN="$(grep -oP 'bot_token\s*=\s*"\K[^"]+' "$TG" 2>/dev/null | head -1)"
    if [ -n "${MASTER_TOKEN:-}" ]; then
        WATCH="omega-tg-bot.service	${MASTER_TOKEN}	Atlas"
    else
        echo "tg-liveness: Telegram config has no valid bot token" >&2
        exit 1
    fi
fi

# Agent bots: one entry per key in agent-bots.json ({"<name>": {"token": ...}}).
BOTS_JSON="$OMEGA_ROOT/agent-bots.json"
if [ -f "$BOTS_JSON" ]; then
    command -v python3 >/dev/null 2>&1 || {
        echo "tg-liveness: python3 is required to read the agent bot registry" >&2
        exit 2
    }
    if ! AGENTS="$(python3 - "$BOTS_JSON" <<'PY' 2>/dev/null
import json, sys
try:
    d = json.load(open(sys.argv[1]))
except Exception:
    sys.exit(1)
if isinstance(d, dict):
    for name, cfg in d.items():
        if isinstance(cfg, dict) and cfg.get("token"):
            print("omega-tg-agent-%s.service\t%s\t%s" % (name, cfg["token"], name))
PY
    )"; then
        echo "tg-liveness: agent bot registry is unreadable or malformed" >&2
        exit 1
    fi
    [ -n "${AGENTS:-}" ] && WATCH="${WATCH}
${AGENTS}"
fi

[ -n "${WATCH//[[:space:]]/}" ] || exit 0

export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
export DBUS_SESSION_BUS_ADDRESS="${DBUS_SESSION_BUS_ADDRESS:-unix:path=${XDG_RUNTIME_DIR}/bus}"

# busy-guard: a live agent child means the bot IS working a real task.
busy() {  # $1 = main pid
    # python/omega-transcribe = the local STT (faster-whisper) child transcribing a
    # voice note: legitimate work (2026-08-11: the guard missed it and the watchdog
    # kept restarting the dentistrygpt bot mid-transcription, every cooldown window).
    local children
    children="$(timeout 3s ps --ppid "$1" -o comm= 2>/dev/null)" || return 2
    printf '%s\n' "$children" | grep -qiE 'claude|codex|whisper|ffmpeg|python|omega-transcribe'
}

# ── Pass 1: snapshot pending for every bot; keep only the suspicious ones ────
SUSPECT=""
FAILURES=0
while IFS=$'\t' read -r unit token label; do
    [ -n "${unit:-}" ] && [ -n "${token:-}" ] || continue
    if ! valid_unit "$unit"; then
        echo "tg-liveness: invalid unit in watch registry" >&2
        FAILURES=1
        continue
    fi
    if ! p1="$(pending "$token")" || [ -z "${p1:-}" ]; then
        echo "tg-liveness: probe failed for $unit" >&2
        FAILURES=1
        continue
    fi
    [ "$p1" -gt 0 ] 2>/dev/null || continue
    if ! mpid="$(timeout 10s systemctl --user show -p MainPID --value "$unit" 2>/dev/null)"; then
        echo "tg-liveness: systemd status probe failed for $unit" >&2
        FAILURES=1
        continue
    fi
    [[ "${mpid:-}" =~ ^[1-9][0-9]*$ ]] || continue
    busy_status=0
    busy "$mpid" || busy_status=$?
    [ "$busy_status" -eq 0 ] && continue
    if [ "$busy_status" -gt 1 ]; then
        echo "tg-liveness: child-process probe failed for $unit" >&2
        FAILURES=1
        continue
    fi
    SUSPECT="${SUSPECT}${unit}	${token}	${label}	${p1}	${mpid}
"
done <<< "$WATCH"

[ -n "${SUSPECT//[[:space:]]/}" ] || exit "$FAILURES"

# ── One shared drain window, then re-probe only the suspects ────────────────
sleep "$DRAIN_SECONDS"

atomic_state() { # $1 target, $2 content
    local target="$1" content="$2" tmp
    mkdir -p "$STATE_DIR"
    tmp="$(mktemp "$STATE_DIR/.tg-liveness-state.XXXXXX")" || return 1
    chmod 0600 "$tmp"
    if ! printf '%s' "$content" > "$tmp" || ! mv -f -- "$tmp" "$target"; then
        rm -f -- "$tmp"
        return 1
    fi
}

send_alert() { # $1 message
    [ -x "$ALERT_BIN" ] || return 127
    timeout 15s bash "$ALERT_BIN" "$1" >/dev/null 2>&1
}

release_probe_lock() { # $1 lock, $2 owner token
    local lock="$1" token="$2" current=""
    [ -f "$lock/owner" ] && read -r current < "$lock/owner"
    if [ "$current" = "$token" ]; then
        rm -f -- "$lock/owner"
        rmdir -- "$lock" 2>/dev/null || true
    fi
}

handle_suspect() (
    unit="$1" token="$2" label="$3" p1="$4" mpid="$5"
    valid_unit "$unit" || { echo "tg-liveness: refusing invalid unit $unit" >&2; return 2; }
    mkdir -p "$STATE_DIR"
    stem="${unit%.service}"
    lock="$STATE_DIR/tg-liveness-lock-$stem"
    owner_token="${BASHPID}:$(date +%s)"
    if ! mkdir "$lock" 2>/dev/null; then
        lock_age=$(( $(date +%s) - $(stat -c %Y "$lock" 2>/dev/null || echo 0) ))
        lock_owner=""
        [ -f "$lock/owner" ] && read -r lock_owner < "$lock/owner"
        lock_pid="${lock_owner%%:*}"
        recovery="$lock.recovery"
        if [ "$lock_age" -lt "$LOCK_STALE_SECONDS" ] \
            || { [[ "$lock_pid" =~ ^[0-9]+$ ]] && kill -0 "$lock_pid" 2>/dev/null; } \
            || ! mkdir "$recovery" 2>/dev/null; then
            echo "[$(date '+%F %T')] tg-liveness: another probe owns $unit; skipping"
            return 0
        fi
        stale="$lock.stale-${BASHPID}"
        if ! mv -- "$lock" "$stale" 2>/dev/null || ! mkdir "$lock" 2>/dev/null; then
            rm -f -- "$stale/owner" 2>/dev/null || true
            rmdir -- "$stale" 2>/dev/null || true
            rmdir -- "$recovery" 2>/dev/null || true
            echo "[$(date '+%F %T')] tg-liveness: lock recovery raced for $unit; skipping"
            return 0
        fi
        rm -f -- "$stale/owner" 2>/dev/null || true
        rmdir -- "$stale" 2>/dev/null || true
        rmdir -- "$recovery" 2>/dev/null || true
        echo "[$(date '+%F %T')] tg-liveness: recovered stale probe lock for $unit"
    fi
    printf '%s\n' "$owner_token" > "$lock/owner" || return 1
    trap 'release_probe_lock "$lock" "$owner_token"' EXIT

    circuit="$STATE_DIR/tg-liveness-circuit-$stem"
    history="$STATE_DIR/tg-liveness-restarts-$stem"
    legacy_flag="$STATE_DIR/tg-liveness-restart-$stem"
    if [ -f "$circuit" ]; then
        echo "[$(date '+%F %T')] tg-liveness: CIRCUIT OPEN for $unit; no restart (repair, then run --reset $unit)"
        return 0
    fi

    p2="$(pending "$token")" || {
        echo "tg-liveness: confirmatory probe failed for $unit" >&2
        return 1
    }
    [ -n "${p2:-}" ] || return 1
    [ "$p2" -gt 0 ] 2>/dev/null || return 0      # drained → was just busy, healthy
    [ "$p2" -ge "$p1" ] 2>/dev/null || return 0  # still draining → consuming, healthy
    busy_status=0
    busy "$mpid" || busy_status=$?
    [ "$busy_status" -eq 0 ] && return 0           # a task started during the wait
    if [ "$busy_status" -gt 1 ]; then
        echo "tg-liveness: confirmatory child-process probe failed for $unit" >&2
        return 1
    fi

    now="$(date +%s)"
    cutoff=$(( now - WINDOW_SECONDS ))
    recent="$(awk -v cutoff="$cutoff" '$1 ~ /^[0-9]+$/ && $1 >= cutoff { print $1 }' "$history" 2>/dev/null)"
    restart_count="$(printf '%s\n' "$recent" | awk 'NF { n++ } END { print n + 0 }')"
    last_restart="$(printf '%s\n' "$recent" | tail -n 1)"
    if [ -n "$last_restart" ] && [ $(( now - last_restart )) -lt "$COOLDOWN_SECONDS" ]; then
        return 0
    fi
    # Honour the legacy cooldown stamp during the first run after upgrading.
    if [ ! -s "$history" ] && [ -f "$legacy_flag" ]; then
        legacy_age=$(( now - $(stat -c %Y "$legacy_flag" 2>/dev/null || echo 0) ))
        [ "$legacy_age" -lt "$COOLDOWN_SECONDS" ] && return 0
    fi

    if [ "$restart_count" -ge "$MAX_RESTARTS" ]; then
        atomic_state "$circuit" "OPEN $now"$'\n' || return 1
        echo "[$(date '+%F %T')] tg-liveness: CIRCUIT OPEN for $unit after $restart_count restart(s) in ${WINDOW_SECONDS}s; no restart"
        if ! send_alert "━━━━━━━━━━━━
<b>Ω  TELEGRAM CIRCUIT OPEN</b>
━━━━━━━━━━━━
 ⛔ ${label} stayed deaf after ${restart_count} restart(s) in ${WINDOW_SECONDS}s. Automatic restarts are stopped. Repair the root cause, then run: <code>omega-atlas-liveness.sh --reset ${unit}</code>."; then
            echo "tg-liveness: circuit-open alert delivery failed for $unit" >&2
            return 1
        fi
        return 0
    fi

    echo "[$(date '+%F %T')] tg-liveness: DEAF ${label} (pending ${p1}→${p2}, pid ${mpid}, no agent child) → restarting ${unit}"
    if timeout 15s systemctl --user restart "$unit"; then
        next_history="${recent}${recent:+$'\n'}${now}"$'\n'
        atomic_state "$history" "$next_history" || return 1
        atomic_state "$legacy_flag" "$now"$'\n' || return 1
        if ! send_alert "━━━━━━━━━━━━
<b>Ω  TELEGRAM AUTO-HEAL</b>
━━━━━━━━━━━━
 🔄 ${label} was alive but not consuming (${p1} msg stuck). Poll loop restarted ($(( restart_count + 1 ))/${MAX_RESTARTS} in the current window)."; then
            echo "tg-liveness: restart succeeded but alert delivery failed for $unit" >&2
            return 1
        fi
    else
        echo "[$(date '+%F %T')] tg-liveness: restart FAILED for $unit" >&2
        send_alert "OmegaOS Telegram auto-heal could not restart ${label} (${unit}). Manual intervention required." || \
            echo "tg-liveness: restart-failure alert delivery also failed for $unit" >&2
        return 1
    fi
)

while IFS=$'\t' read -r unit token label p1 mpid; do
    [ -n "${unit:-}" ] || continue
    if ! handle_suspect "$unit" "$token" "$label" "$p1" "$mpid"; then
        FAILURES=1
    fi
done <<< "$SUSPECT"

exit "$FAILURES"

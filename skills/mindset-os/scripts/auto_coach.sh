#!/usr/bin/env bash
# auto_coach.sh — the AI growth loop of the Six-Month Identity Challenge.
#
# It reads the challenge workspace's latest follow-ups (daily / weekly /
# monthly), runs the Mindset {OS} master agent (an LLM — `claude` by default)
# over them, and writes evidence-aware coaching into coaching/, then pushes a
# short card to the operator's Telegram. It picks the cadence automatically:
# a new week closed -> weekly coaching; a new month -> monthly; otherwise a
# daily nudge.
#
# DISARMED BY DEFAULT (same posture as every OmegaOS autonomous engine): it
# runs one pass on demand, but the daily cron only fires after `--arm`
# (touches <ws>/.coach-armed). `--disarm` removes it. Nothing autonomous runs
# until the operator arms it.
#
#   omega-mindset coach <workspace>            one coaching pass now
#   omega-mindset coach <workspace> --arm      + install the daily 07:00 cron
#   omega-mindset coach <workspace> --disarm   remove the cron
set -uo pipefail

OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
WS="${1:-}"
MODE="${2:-run}"
[ -n "$WS" ] || { echo "usage: auto_coach.sh <workspace> [--arm|--disarm]" >&2; exit 2; }
WS="$(cd "$WS" 2>/dev/null && pwd)" || { echo "auto_coach: workspace not found: $1" >&2; exit 2; }
[ -f "$WS/state.json" ] || { echo "auto_coach: not a challenge workspace (no state.json): $WS" >&2; exit 2; }

ARMED_FLAG="$WS/.coach-armed"
CRON_TAG="OMEGA-MINDSET-COACH:$WS"

arm() {
    touch "$ARMED_FLAG"
    local self; self="$(cd "$(dirname "$0")" && pwd)/auto_coach.sh"
    local line="0 7 * * * OMEGA_DIR=$HOME/.omega PATH=$HOME/.local/bin:/usr/local/bin:/usr/bin:/bin bash $self $WS run >> $WS/coaching/cron.log 2>&1 # $CRON_TAG"
    ( crontab -l 2>/dev/null | grep -v "$CRON_TAG"; echo "$line" ) | crontab -
    echo "  ✅ auto-coach ARMED — daily 07:00. Disarm: omega-mindset coach $WS --disarm"
    exit 0
}
disarm() {
    rm -f "$ARMED_FLAG"
    ( crontab -l 2>/dev/null | grep -v "$CRON_TAG" ) | crontab - 2>/dev/null || true
    echo "  auto-coach DISARMED for $WS"
    exit 0
}
case "$MODE" in
    --arm) arm ;;
    --disarm) disarm ;;
esac

# ── one coaching pass ──────────────────────────────────────────────────────
# Cadence: the highest-granularity artifact that changed since the last run.
STAMP="$WS/coaching/.last-run"
now_epoch=$(date +%s)
last_epoch=0; [ -f "$STAMP" ] && last_epoch=$(cat "$STAMP" 2>/dev/null || echo 0)

newest() { ls -t "$WS/$1"/*.md 2>/dev/null | head -1; }
DAILY="$(newest daily)"; WEEKLY="$(newest weekly)"; MONTHLY="$(newest monthly)"
CADENCE="daily"; FOCUS_FILE="$DAILY"
[ -n "$WEEKLY" ] && [ "$(stat -c %Y "$WEEKLY" 2>/dev/null || echo 0)" -gt "$last_epoch" ] && { CADENCE="weekly"; FOCUS_FILE="$WEEKLY"; }
[ -n "$MONTHLY" ] && [ "$(stat -c %Y "$MONTHLY" 2>/dev/null || echo 0)" -gt "$last_epoch" ] && { CADENCE="monthly"; FOCUS_FILE="$MONTHLY"; }

# Resolve the Mindset master persona (same brain as the OS session/bot).
PERSONA="$OMEGA_DIR/os/mindset-os/MASTER.md"
[ -f "$PERSONA" ] || PERSONA="$OMEGA_DIR/skills/mindset-os/SKILL.md"

# Assemble the context: constitution + plan + the latest follow-ups.
CTX="$(mktemp)"
{
    echo "# Six-Month Identity Challenge — $CADENCE coaching context"; echo
    for f in "$WS/IDENTITY_CONSTITUTION.md" "$WS/CHALLENGE_PLAN.md" "$FOCUS_FILE" "$DAILY" "$WEEKLY" "$MONTHLY"; do
        [ -n "$f" ] && [ -f "$f" ] && { echo "## $(basename "$f")"; cat "$f"; echo; }
    done
} > "$CTX"

PROMPT="You are running the $CADENCE growth check of my Six-Month Identity Challenge. \
Read the context (identity constitution, plan, and my latest follow-ups). As Mindset {OS}: \
(1) name the identity evidence you can actually see, \
(2) diagnose the SYSTEM behind any miss (never judge my character), \
(3) give ONE keystone adjustment for the next $CADENCE period, \
(4) protect-first check (sleep/health/relationships), \
(5) end with a single, doable next action. Keep it phone-readable, label any claim E1/E2/S/P/C, \
never give clinical/crisis/medication advice — route to a professional if you see risk. \
Context follows:

$(cat "$CTX")"

OUT="$WS/coaching/$(date +%Y-%m-%d)_${CADENCE}.md"
if command -v claude >/dev/null 2>&1; then
    { echo "# ${CADENCE^} coaching — $(date +%Y-%m-%d)"; echo;
      claude --append-system-prompt "$(cat "$PERSONA")" -p "$PROMPT" 2>/dev/null; } > "$OUT"
else
    { echo "# ${CADENCE^} coaching — $(date +%Y-%m-%d)"; echo;
      echo "(LLM 'claude' not on PATH — install it to enable the growth loop.)";
      echo; echo "Context was assembled at $CTX"; } > "$OUT"
fi
rm -f "$CTX"
echo "$now_epoch" > "$STAMP"

echo "  ✅ $CADENCE coaching written → $OUT"

# Push a short card to Telegram (operator-only), best-effort.
TG="$OMEGA_DIR/telegram.toml"
if [ -f "$TG" ]; then
    TOKEN="$(grep -E '^[[:space:]]*bot_token' "$TG" 2>/dev/null | head -1 | cut -d'"' -f2)"
    CHAT="$(grep -E '^[[:space:]]*chat_id' "$TG" 2>/dev/null | head -1 | grep -oE '\-?[0-9]+' | head -1)"
    if [ -n "${TOKEN:-}" ] && [ -n "${CHAT:-}" ]; then
        BODY="$(head -c 3500 "$OUT")"
        curl -s -X POST "https://api.telegram.org/bot${TOKEN}/sendMessage" \
            -d chat_id="$CHAT" --data-urlencode text="🧠 Mindset ${CADENCE} coaching
$BODY" -d disable_web_page_preview=true >/dev/null 2>&1 || true
    fi
fi

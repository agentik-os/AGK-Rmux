#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════
# omega-tg-up.sh — connect Telegram to the OmegaOS command bot, in one command.
# ───────────────────────────────────────────────────────────────────────────
# The OmegaOS command bot (telegram-bot/omega-tg-bot.ts) is the operator's phone
# control center: every command opens an inline-keyboard of sub-actions, each
# running an `omega` CLI action. This validates the bot token, writes it to
# ~/.omega/telegram.toml (operator-locked, R-TGSEC), and (re)starts the bot.
#
#   OMEGA_TG_TOKEN=<BOT_TOKEN> omega-tg-up <YOUR_TELEGRAM_USER_ID>
#   printf '%s\n' "$BOT_TOKEN" | omega-tg-up <YOUR_TELEGRAM_USER_ID>
# ═══════════════════════════════════════════════════════════════════════════
set -euo pipefail
OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
TG="$OMEGA_DIR/telegram.toml"
BOT="$OMEGA_DIR/telegram-bot/omega-tg-bot.ts"
PID_FILE="$OMEGA_DIR/state/omega-tg-bot.pid"
READY_FILE="$OMEGA_DIR/state/omega-tg-bot.ready.json"
TOKEN="${OMEGA_TG_TOKEN:-}"
USERID="${OMEGA_TG_USER_ID:-}"
SYSTEMD_READY_ENV=0

if [[ -n "$TOKEN" ]]; then
  USERID="${1:-$USERID}"
  # Do not leak the token into child-process environments.
  unset OMEGA_TG_TOKEN
elif [[ $# -ge 2 ]]; then
  # Backward-compatible only. Prefer env/stdin so `ps` and shell history never
  # receive the credential.
  echo "omega-tg-up: warning: positional token usage is deprecated; use OMEGA_TG_TOKEN or stdin" >&2
  TOKEN="$1"
  USERID="$2"
else
  USERID="${1:-$USERID}"
  if [[ -t 0 ]]; then
    read -r -s -p "Telegram bot token: " TOKEN
    echo >&2
  else
    IFS= read -r TOKEN || true
  fi
fi

[[ "$TOKEN" =~ ^[0-9]+:[A-Za-z0-9_-]+$ ]] || {
  echo "usage: OMEGA_TG_TOKEN=<BOT_TOKEN> omega-tg-up <YOUR_TELEGRAM_USER_ID> (or pipe token on stdin)" >&2
  exit 1
}
[[ "$USERID" =~ ^[0-9]+$ ]] || { echo "omega-tg-up: Telegram user id must be numeric" >&2; exit 1; }
EXPECTED_BOT_ID="${TOKEN%%:*}"
if command -v curl >/dev/null 2>&1; then
  # curl reads the credential-bearing URL from stdin config, never argv.
  printf 'url = "https://api.telegram.org/bot%s/getMe"\n' "$TOKEN" \
    | curl -sf --config - >/dev/null 2>&1 \
    || { echo "omega-tg-up: token rejected by Telegram (getMe) — get a fresh token from @BotFather"; exit 1; }
fi

umask 077
mkdir -p "$OMEGA_DIR" "$OMEGA_DIR/state" "$OMEGA_DIR/logs"
TG_TMP="$(mktemp "$OMEGA_DIR/.telegram.toml.XXXXXX")"
cleanup() {
  rm -f -- "${TG_TMP:-}" "${PID_TMP:-}" "$READY_FILE"
  if [[ "$SYSTEMD_READY_ENV" == 1 ]]; then
    systemctl --user unset-environment OMEGA_TG_READY_FILE >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT
cat > "$TG_TMP" <<EOF
bot_token = "${TOKEN}"
chat_id = ${USERID:-0}
allow_user_ids = [${USERID:-}]
relay_session = "aisb-master"
enabled = true
EOF
chmod 0600 "$TG_TMP"
mv -f -- "$TG_TMP" "$TG"
TG_TMP=""

# (Re)start the bot — systemd service if present, else a detached process.
rm -f -- "$READY_FILE"
if command -v systemctl >/dev/null 2>&1 && systemctl --user list-unit-files omega-tg-bot.service >/dev/null 2>&1; then
  systemctl --user enable omega-tg-bot.service >/dev/null \
    || { echo "omega-tg-up: failed to enable omega-tg-bot.service" >&2; exit 1; }
  systemctl --user set-environment OMEGA_TG_READY_FILE="$READY_FILE" >/dev/null \
    || { echo "omega-tg-up: failed to configure the Telegram readiness probe" >&2; exit 1; }
  SYSTEMD_READY_ENV=1
  systemctl --user restart omega-tg-bot.service >/dev/null \
    || { echo "omega-tg-up: failed to restart omega-tg-bot.service" >&2; exit 1; }
  for _ in {1..300}; do
    if [[ -s "$READY_FILE" ]] && grep -Eq '"bot_id"[[:space:]]*:[[:space:]]*'"$EXPECTED_BOT_ID"'([,}])' "$READY_FILE"; then
      break
    fi
    systemctl --user is-active --quiet omega-tg-bot.service \
      || { echo "omega-tg-up: omega-tg-bot.service exited before becoming ready" >&2; exit 1; }
    sleep 0.1
  done
  [[ -s "$READY_FILE" ]] && grep -Eq '"bot_id"[[:space:]]*:[[:space:]]*'"$EXPECTED_BOT_ID"'([,}])' "$READY_FILE" \
    || { echo "omega-tg-up: timed out waiting for omega-tg-bot.service readiness" >&2; exit 1; }
  systemctl --user unset-environment OMEGA_TG_READY_FILE >/dev/null \
    || { echo "omega-tg-up: bot is ready but its temporary systemd environment could not be cleared" >&2; exit 1; }
  SYSTEMD_READY_ENV=0
elif command -v bun >/dev/null 2>&1 && [[ -f "$BOT" ]]; then
  if [[ -f "$PID_FILE" ]]; then
    OLD_PID="$(<"$PID_FILE")"
    [[ "$OLD_PID" =~ ^[0-9]+$ ]] \
      || { echo "omega-tg-up: refusing malformed Telegram pidfile $PID_FILE" >&2; exit 1; }
    if [[ -n "$OLD_PID" ]] && kill -0 "$OLD_PID" 2>/dev/null; then
      OLD_CMD="$(ps -p "$OLD_PID" -o command= 2>/dev/null || true)"
      if [[ "$OLD_CMD" == *"$BOT"* && "$OLD_CMD" == *"--omega-main-bot"* ]]; then
        kill "$OLD_PID"
        for _ in {1..50}; do kill -0 "$OLD_PID" 2>/dev/null || break; sleep 0.1; done
        if kill -0 "$OLD_PID" 2>/dev/null; then
          echo "omega-tg-up: previous Telegram poller did not stop; refusing to start a duplicate" >&2
          exit 1
        fi
      else
        echo "omega-tg-up: pidfile points to an unverified process; refusing broad termination" >&2
        exit 1
      fi
    fi
  fi
  (
    cd "$OMEGA_DIR/telegram-bot"
    exec nohup env OMEGA_DIR="$OMEGA_DIR" OMEGA_MAIN_BOT=1 OMEGA_TG_READY_FILE="$READY_FILE" bun "$BOT" --omega-main-bot \
      >"$OMEGA_DIR/logs/omega-tg-bot.log" 2>&1
  ) &
  NEW_PID=$!
  for _ in {1..300}; do
    if [[ -s "$READY_FILE" ]] \
      && grep -Eq '"pid"[[:space:]]*:[[:space:]]*'"$NEW_PID"'([,}])' "$READY_FILE" \
      && grep -Eq '"bot_id"[[:space:]]*:[[:space:]]*'"$EXPECTED_BOT_ID"'([,}])' "$READY_FILE"; then
      break
    fi
    kill -0 "$NEW_PID" 2>/dev/null \
      || { echo "omega-tg-up: Bun bot exited before becoming ready" >&2; exit 1; }
    sleep 0.1
  done
  [[ -s "$READY_FILE" ]] \
    && grep -Eq '"pid"[[:space:]]*:[[:space:]]*'"$NEW_PID"'([,}])' "$READY_FILE" \
    && grep -Eq '"bot_id"[[:space:]]*:[[:space:]]*'"$EXPECTED_BOT_ID"'([,}])' "$READY_FILE" \
    || { echo "omega-tg-up: timed out waiting for Bun bot readiness" >&2; kill "$NEW_PID" 2>/dev/null || true; exit 1; }
  PID_TMP="$(mktemp "$OMEGA_DIR/state/.omega-tg-bot.pid.XXXXXX")"
  printf '%s\n' "$NEW_PID" > "$PID_TMP"
  chmod 0600 "$PID_TMP"
  mv -f -- "$PID_TMP" "$PID_FILE"
  PID_TMP=""
else
  echo "omega-tg-up: neither omega-tg-bot.service nor Bun bot runtime is available" >&2
  exit 1
fi
echo "omega-tg-up: ✓ connected. DM the bot and tap /menu. For a project hub: create a supergroup (enable Topics), add the bot as ADMIN, then /setupgroup + /sync inside it."

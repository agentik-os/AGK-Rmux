#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════
# inbox-bot-up.sh — connect the OmegaOS Deposit bot in one command.
# ───────────────────────────────────────────────────────────────────────────
# The Deposit bot (telegram-bot/inbox-bot.ts) is a private inbox: the operator
# sends photos/notes from their phone and an agent reads them in ~/.omega/inbox/.
# This validates the bot token, writes it to ~/.omega/deposit.toml (operator-
# locked, R-TGSEC), and (re)starts the bot.
#
#   inbox-bot-up <BOT_TOKEN> [YOUR_TELEGRAM_USER_ID]
#
# USER_ID is optional: omit it and this prints a one-time 6-digit pairing code —
# the bot locks to the FIRST chat whose message contains that code. (A blind
# first-sender self-lock would let a stranger who finds the bot's username own
# the operator's inbox, R-TGSEC.) Token can also be passed as
# $OMEGA_DEPOSIT_TOKEN (kept out of the process list):
#   OMEGA_DEPOSIT_TOKEN=<TOKEN> inbox-bot-up [USER_ID]
# ═══════════════════════════════════════════════════════════════════════════
set -euo pipefail
OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
CONF="$OMEGA_DIR/deposit.toml"
BOT="$OMEGA_DIR/telegram-bot/inbox-bot.ts"

# env-prefix form: OMEGA_DEPOSIT_TOKEN=<tok> inbox-bot-up [USER_ID]
TOKEN="${OMEGA_DEPOSIT_TOKEN:-${1:-}}"
if [[ -n "${OMEGA_DEPOSIT_TOKEN:-}" ]]; then USERID="${1:-}"; else USERID="${2:-}"; fi
[[ -n "$TOKEN" ]] || { echo "usage: inbox-bot-up <BOT_TOKEN> [YOUR_TELEGRAM_USER_ID]   (or OMEGA_DEPOSIT_TOKEN=<tok> inbox-bot-up [USER_ID])"; exit 1; }

if command -v curl >/dev/null 2>&1; then
  curl -sf "https://api.telegram.org/bot${TOKEN}/getMe" >/dev/null 2>&1 \
    || { echo "inbox-bot-up: token rejected by Telegram (getMe) — get a fresh token from @BotFather"; exit 1; }
fi

mkdir -p "$OMEGA_DIR"
umask 077
# No USER_ID → mint a one-time pairing code (urandom: macOS ships bash 3.2, no
# $SRANDOM). The bot only locks to the first chat whose message contains it.
PAIR=""
[[ -n "$USERID" ]] || PAIR="$(printf '%06d' "$(( $(od -An -N4 -tu4 /dev/urandom | tr -d ' ') % 1000000 ))")"
{
  echo "bot_token = \"${TOKEN}\""
  echo "chat_id = ${USERID:-0}"
  [[ -n "$PAIR" ]] && echo "pair_code = \"${PAIR}\""
  echo "enabled = true"
} > "$CONF"
chmod 0600 "$CONF"

# (Re)start the bot — systemd service if present, launchd on macOS, else detached.
if command -v systemctl >/dev/null 2>&1 && systemctl --user list-unit-files omega-inbox-bot.service >/dev/null 2>&1; then
  systemctl --user enable --now omega-inbox-bot.service 2>/dev/null || true
  systemctl --user restart omega-inbox-bot.service 2>/dev/null || true
elif [[ "$(uname -s)" == "Darwin" ]] && launchctl print "gui/$(id -u)/os.omega.inbox-bot" >/dev/null 2>&1; then
  launchctl kickstart -k "gui/$(id -u)/os.omega.inbox-bot" 2>/dev/null || true
elif command -v bun >/dev/null 2>&1 && [[ -f "$BOT" ]]; then
  pkill -f "telegram-bot/inbox-bot.ts" 2>/dev/null || true
  ( cd "$OMEGA_DIR/telegram-bot" && OMEGA_DIR="$OMEGA_DIR" nohup bun "$BOT" >/tmp/omega-inbox-bot.log 2>&1 & )
fi
if [[ -n "$PAIR" ]]; then
  echo "inbox-bot-up: ✓ connected. PAIRING CODE: ${PAIR} — DM it to the bot from YOUR Telegram to lock the inbox (first chat sending the code wins)."
else
  echo "inbox-bot-up: ✓ connected (@deposit bot, locked to ${USERID}). DM the bot a photo — the agent reads it in $OMEGA_DIR/inbox/."
fi

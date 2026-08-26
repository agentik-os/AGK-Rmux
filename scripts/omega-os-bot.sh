#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════
# omega-os-bot : wire a dedicated Telegram bot to ONE operative system of the
# AgentikOS suite (Mindset/Habits/Brainstorm/Blueprint/Stepper/Builder/Books).
#
# The bot's brain IS the OS's MASTER AGENT: an agent-bots.json entry of kind
# "persona" whose system prompt is the OS's MASTER.md (Books OS points at the
# full librarian persona instead — one canon, never forked) and whose working
# dir is the installed OS folder, so the ledger persists across sessions.
#
# The ONE step Telegram cannot automate: creating the bot. The operator does
# @BotFather -> /newbot, then:
#
#   omega-os-bot <os-slug> [token]     token prompted interactively if omitted
#   omega-os-bot --list                suite + which OS already has a bot
#
# Same shape as omega-agent-bot.sh (validate token -> agent-bots.json mode 600
# -> systemd unit omega-tg-agent-os-<slug> -> start + verify). TUI: OS tab ->
# select an OS -> T runs this script in a terminal session.
# ═══════════════════════════════════════════════════════════════════════════
set -uo pipefail
OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
BOTS="$OMEGA_DIR/agent-bots.json"
TG="$OMEGA_DIR/telegram.toml"
UNIT_DIR="$HOME/.config/systemd/user"

SLUGS=(ideation-os researcher-os blueprint-os designer-os stepper-os builder-os mindset-os habits-os execution-os storytelling-os alignment-os books-os ai-logic-os)

die() { echo "  ✗ $*" >&2; exit 1; }

# ── --list : the suite and which OS already has its bot ────────────────────
if [ "${1:-}" = "--list" ] || [ "${1:-}" = "-l" ]; then
    python3 - "$BOTS" "${SLUGS[@]}" <<'PY'
import json, os, sys
bots = json.load(open(sys.argv[1])) if os.path.exists(sys.argv[1]) else {}
print("AgentikOS OS suite — Telegram bots:")
for slug in sys.argv[2:]:
    key = f"os-{slug}"
    mark = "🤖 linked" if key in bots else "·  none   -> omega-os-bot " + slug
    print(f"   {slug:<15} {mark}")
PY
    exit 0
fi

SLUG="${1:-}"
[ -n "$SLUG" ] || { echo "usage: omega-os-bot <os-slug> [token]"; echo "       omega-os-bot --list"; exit 2; }
case " ${SLUGS[*]} " in
    *" $SLUG "*) ;;
    *) die "unknown OS '$SLUG' (expected one of: ${SLUGS[*]})" ;;
esac

OS_DIR="$OMEGA_DIR/os/$SLUG"
[ -d "$OS_DIR" ] || die "OS not installed: $OS_DIR (run OmegaOS install.sh)"

# ── the master-agent persona this bot embodies ─────────────────────────────
# Books OS: the canonical librarian persona is richer than the bootstrap
# MASTER.md — point the bot straight at the canon (anti-duplication).
PERSONA="$OS_DIR/MASTER.md"
if [ "$SLUG" = "books-os" ] && [ -f "$OMEGA_DIR/agents/librarian.md" ]; then
    PERSONA="$OMEGA_DIR/agents/librarian.md"
fi
[ -f "$PERSONA" ] || die "no master persona at $PERSONA (re-run install.sh)"

[ -f "$BOTS" ] || echo '{}' > "$BOTS"

# ── legacy absorption (Books OS): the pre-suite 'librarian' bot IS the Books
# OS master agent. Adopt its token when none is given, and remove the legacy
# entry + unit after wiring, so ONE service polls the token (two pollers on
# one token = the classic mute-bot getUpdates conflict).
LEGACY_KEY=""
LEGACY_TOKEN=""
if [ "$SLUG" = "books-os" ]; then
    LEGACY_TOKEN="$(python3 - "$BOTS" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
print(d.get("librarian", {}).get("token", ""))
PY
)"
    [ -n "$LEGACY_TOKEN" ] && LEGACY_KEY="librarian"
fi

TOKEN="${2:-}"
if [ -z "$TOKEN" ] && [ -n "$LEGACY_TOKEN" ]; then
    TOKEN="$LEGACY_TOKEN"
    echo "  → adopting the existing librarian bot (same brain) — no new token needed"
fi
if [ -z "$TOKEN" ]; then
    echo "  Create the bot first: Telegram -> @BotFather -> /newbot"
    printf "  Paste the bot token: "
    read -r TOKEN
fi
[ -n "$TOKEN" ] || die "no token given"

# ── the operator's own id, from the master bridge config ──────────────────
OPERATOR="$(grep -E '^[[:space:]]*chat_id' "$TG" 2>/dev/null | head -1 | grep -oE '[0-9]+' | head -1)"
[ -n "$OPERATOR" ] || die "no chat_id in $TG (run: omega telegram setup)"

# ── validate the token BEFORE touching anything (never wire a dead bot) ────
echo "  → validating token"
ME="$(curl -s --max-time 15 "https://api.telegram.org/bot${TOKEN}/getMe")"
USERNAME="$(printf '%s' "$ME" | python3 -c "import json,sys
d=json.load(sys.stdin)
print(d['result']['username'] if d.get('ok') else '')" 2>/dev/null)"
[ -n "$USERNAME" ] || die "invalid token: $(printf '%s' "$ME" | head -c 160)"
echo "    @${USERNAME} ✓"

KEY="os-${SLUG}"
mkdir -p "$OS_DIR/ledger"

# ── agent-bots.json (mode 600 — the token lives here and NOWHERE else) ─────
python3 - "$BOTS" "$KEY" "$TOKEN" "$OPERATOR" "$SLUG" "$PERSONA" "$OS_DIR" <<'PY'
import json, os, sys
f, key, token, op, slug, persona, osdir = sys.argv[1:8]
d = json.load(open(f))
d[key] = {
    "token": token,
    "allow": [int(op)],
    "project": slug,
    "kind": "persona",
    "persona": persona,
    "dir": osdir,
}
json.dump(d, open(f, "w"), indent=1, ensure_ascii=False)
os.chmod(f, 0o600)
PY
echo "  → agent-bots.json: '$KEY' → persona $(basename "$PERSONA") (mode 600)"

# ── retire the absorbed legacy entry BEFORE starting the new unit: never two
# pollers on one token, not even for a few seconds.
if [ -n "$LEGACY_KEY" ]; then
    export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
    systemctl --user disable --now "omega-tg-agent-${LEGACY_KEY}.service" >/dev/null 2>&1 || true
    rm -f "$UNIT_DIR/omega-tg-agent-${LEGACY_KEY}.service"
    python3 - "$BOTS" "$LEGACY_KEY" <<'PY'
import json, os, sys
f, key = sys.argv[1], sys.argv[2]
d = json.load(open(f))
d.pop(key, None)
json.dump(d, open(f, "w"), indent=1, ensure_ascii=False)
os.chmod(f, 0o600)
PY
    echo "  → legacy '$LEGACY_KEY' entry + unit retired (absorbed into $KEY — no duplicate poller)"
fi

# ── systemd unit: same shape as every other agent bot ──────────────────────
mkdir -p "$UNIT_DIR"
cat > "$UNIT_DIR/omega-tg-agent-${KEY}.service" <<UNIT
[Unit]
Description=OmegaOS OS-suite bot - ${KEY}
After=network-online.target

[Service]
Type=simple
Environment=OMEGA_DIR=%h/.omega
Environment=OMEGA_AGENT_BOT=${KEY}
WorkingDirectory=%h/.omega/telegram-bot
ExecStart=/bin/sh -c 'for c in "\$\$(command -v bun 2>/dev/null)" "\$\$HOME/.bun/bin/bun" /opt/homebrew/bin/bun /usr/local/bin/bun; do [ -n "\$\$c" ] && [ -x "\$\$c" ] && exec "\$\$c" "%h/.omega/telegram-bot/omega-tg-bot.ts"; done; echo "tg-agent: bun not found" >&2; exit 127'
Restart=always
RestartSec=3

[Install]
WantedBy=default.target
UNIT

export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
systemctl --user daemon-reload
systemctl --user enable --now "omega-tg-agent-${KEY}.service" >/dev/null 2>&1
sleep 2
if systemctl --user is-active --quiet "omega-tg-agent-${KEY}.service"; then
    echo "  → service omega-tg-agent-${KEY}: active ✓"
else
    echo "  ✗ service failed to start:" >&2
    journalctl --user -u "omega-tg-agent-${KEY}.service" -n 12 --no-pager >&2
    exit 1
fi

echo ""
echo "  ✅ @${USERNAME} is live — the ${SLUG} MASTER AGENT answers every DM."
echo "     Ledger: $OS_DIR/ledger/  ·  unlink: systemctl --user disable --now omega-tg-agent-${KEY}"

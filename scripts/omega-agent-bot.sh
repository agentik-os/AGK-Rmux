#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════
# omega-agent-bot : wire a dedicated Telegram agent-bot to a project.
#
# A project bot is the operator's private DM channel to ONE project: every
# message he sends it becomes a real oracle dispatch into that project's
# directory (kind "oracle"), and the reports land in the project's topic.
#
# The ONE step that cannot be automated: Telegram has no API to create a bot.
# @BotFather is interactive and needs the operator's own account. So he creates
# the bot, and this script does absolutely everything else.
#
#   1. Telegram -> @BotFather -> /newbot -> name it -> username ending in "bot"
#   2. Add that bot as ADMIN of the OmegaOS hub group (else it cannot post)
#   3. omega-agent-bot <project> <token>
#
# It validates the token, writes the agent-bots.json entry (mode 600), installs
# the systemd unit (OMEGA_AGENT_BOT=<name>, same pattern as every other project
# bot), starts it, and verifies it is actually up.
#
#   omega-agent-bot verba 1234567890:AA...     wire the bot for project "verba"
#   omega-agent-bot --list                     show which projects still lack a bot
# ═══════════════════════════════════════════════════════════════════════════
set -uo pipefail
OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
REG="$OMEGA_DIR/projects.json"
BOTS="$OMEGA_DIR/agent-bots.json"
TG="$OMEGA_DIR/telegram.toml"
UNIT_DIR="$HOME/.config/systemd/user"

die() { echo "  ✗ $*" >&2; exit 1; }

# ── --list : which projects still have no dedicated bot ────────────────────
if [ "${1:-}" = "--list" ] || [ "${1:-}" = "-l" ]; then
    python3 - "$REG" "$BOTS" <<'PY'
import json, sys, os
reg = json.load(open(sys.argv[1])).get("projects", [])
bots = json.load(open(sys.argv[2])) if os.path.exists(sys.argv[2]) else {}
linked = {(b.get("project") or "").lower() for b in bots.values()}
have, miss = [], []
for p in reg:
    (have if p["name"].lower() in linked else miss).append(p["name"])
print("Projects WITH a dedicated bot (%d):" % len(have))
for n in sorted(have): print("   🤖", n)
print("\nProjects WITHOUT one (%d)  -> omega-agent-bot <name> <token>:" % len(miss))
for n in sorted(miss): print("   ·", n)
PY
    exit 0
fi

PROJECT="${1:-}"; TOKEN="${2:-}"
[ -n "$PROJECT" ] && [ -n "$TOKEN" ] || {
    echo "usage: omega-agent-bot <project> <token @BotFather>"
    echo "       omega-agent-bot --list"
    exit 2
}
[ -f "$REG" ]  || die "no project registry: $REG"
[ -f "$BOTS" ] || echo '{}' > "$BOTS"

# ── the project must exist in the registry ────────────────────────────────
NAME="$(python3 - "$REG" "$PROJECT" <<'PY'
import json, sys
reg = json.load(open(sys.argv[1])).get("projects", [])
want = sys.argv[2].lower()
print(next((p["name"] for p in reg if p["name"].lower() == want), ""))
PY
)"
[ -n "$NAME" ] || die "unknown project: $PROJECT  (omega-agent-bot --list)"

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

KEY="$(printf '%s' "$NAME" | tr '[:upper:]' '[:lower:]' | tr -cd 'a-z0-9-')"

# ── agent-bots.json (mode 600 — the token lives here and NOWHERE else) ─────
python3 - "$BOTS" "$KEY" "$TOKEN" "$OPERATOR" "$NAME" <<'PY'
import json, os, sys
f, key, token, op, project = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4]), sys.argv[5]
d = json.load(open(f))
d[key] = {"token": token, "allow": [op], "project": project}
json.dump(d, open(f, "w"), indent=1, ensure_ascii=False)
os.chmod(f, 0o600)
PY
echo "  → agent-bots.json: '$KEY' → project '$NAME' (mode 600)"

# ── systemd unit: same shape as every other project bot ───────────────────
mkdir -p "$UNIT_DIR"
cat > "$UNIT_DIR/omega-tg-agent-${KEY}.service" <<UNIT
[Unit]
Description=OmegaOS agent bot - ${KEY}
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
echo "  ✅ @${USERNAME} is live for project '${NAME}'."
echo "     DM it: every message dispatches an oracle into that project."
echo "     Make sure it is ADMIN of the hub group so it can post in the topic."

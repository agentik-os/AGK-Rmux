#!/usr/bin/env bash
# OmegaOS self-heal daemon (cron, every few hours).
# ───────────────────────────────────────────────────────────────────────────
# Runs `omega doctor --fix` to apply the safe mechanical fixes it knows how to
# resolve (duplicate Telegram pollers, dead bot service, stale usage cache,
# expired oauth). If warnings REMAIN after the fix pass, it sends ONE branded
# Telegram alert to the operator so they can tap /status → "🛠 Fix it" (which
# dispatches an oracle). It does NOT auto-dispatch an oracle itself — that costs
# tokens and should stay a human decision (R-BUDGET).
#
# Scheduled by install.sh (OMEGA-CRON-SELFHEAL-v1) so a fresh clone self-heals.
set -uo pipefail

OMEGA="${OMEGA_BIN:-$HOME/.local/bin/omega}"
[ -x "$OMEGA" ] || OMEGA="$(command -v omega 2>/dev/null || echo omega)"

# 1) Apply mechanical fixes (prints what it did + the post-fix check list).
OUT="$("$OMEGA" doctor --fix 2>&1)"
echo "$OUT"

# 1b) Heal ~/.claude/skills links (create-if-missing, never overwrite).
# gstack's bin/gstack-relink rm -f's any flat ~/.claude/skills/<name> symlink
# whose basename collides with a gstack skill name, with NO provenance check
# (runtime-proven: it deleted our `diagram` link). That relink can re-fire
# OUTSIDE install-third-party-skills.sh — /gstack-upgrade re-runs ./setup, and
# `gstack-config set skill_prefix` re-runs it too — so the install-phase heal
# alone does not cover a later re-fire. Replicate `omega sync`'s exact semantic
# (the source of truth is ~/.omega/skills): for every ~/.omega/skills/<name>
# with no ~/.claude/skills/<name> entry, recreate the symlink. Never overwrites
# anything existing, so any deletion auto-repairs within the 3h cron cadence.
healed=0
if [ -d "$HOME/.omega/skills" ]; then
    mkdir -p "$HOME/.claude/skills"
    for od in "$HOME/.omega/skills"/*/; do
        [ -d "$od" ] || continue
        target="$HOME/.claude/skills/$(basename "$od")"
        if [ ! -e "$target" ] && [ ! -L "$target" ]; then
            ln -sfn "${od%/}" "$target" && healed=$((healed + 1))
        fi
    done
fi
[ "$healed" -gt 0 ] && echo "self-heal: healed $healed claude skill link(s)"

# 2) Count warnings/fails REMAINING after the fix pass.
REMAIN="$("$OMEGA" doctor 2>&1 | grep -cE '^\s*\[[!x]\]')"
[ "${REMAIN:-0}" -gt 0 ] 2>/dev/null || { echo "self-heal: clean"; exit 0; }

# 3) Best-effort Telegram alert (same creds the usage monitor uses).
TG="$HOME/.omega/telegram.toml"
[ -f "$TG" ] || exit 0
TOKEN="$(grep -oP 'bot_token\s*=\s*"\K[^"]+' "$TG" 2>/dev/null | head -1)"
CHAT="$(grep -oP 'allow_user_ids\s*=\s*\[\s*\K[0-9]+' "$TG" 2>/dev/null | head -1)"
[ -n "$CHAT" ] || CHAT="$(grep -oP 'chat_id\s*=\s*\K-?[0-9]+' "$TG" 2>/dev/null | head -1)"
[ -n "$TOKEN" ] && [ -n "$CHAT" ] || exit 0

# Cooldown: at most one self-heal alert per 6h (don't nag).
FLAG="$HOME/.omega/state/selfheal-alert-sent"
if [ -f "$FLAG" ]; then
    AGE=$(( $(date +%s) - $(stat -c %Y "$FLAG" 2>/dev/null || echo 0) ))
    [ "$AGE" -lt 21600 ] && exit 0
fi
mkdir -p "$HOME/.omega/state"

RULE="━━━━━━━━━━━━"
MSG="${RULE}
<b>Ω  AUTO-HEAL</b>
${RULE}
 🩺 Heal pass ran — <b>${REMAIN}</b> warning(s) remaining.
 Tap /status → <b>🛠 Fix it</b> to send an oracle."

# Operational alert → the dedicated Alerts topic via the canonical sender
# (auto-recreates the topic if deleted, DM fallback). Arm the 6h cooldown
# ONLY on a successful send — arming before (and swallowing the failure)
# silenced the NEXT alert for 6h whenever this one failed.
if bash "$HOME/.omega/bin/omega-alert-send.sh" "$MSG"; then
    : > "$FLAG"
fi

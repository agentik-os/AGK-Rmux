#!/usr/bin/env bash
# OMEGA-CRON-NOVA-SELF-IMPROVE — Nova's weekly self-improvement loop.
# Sunday evening: Nova (Opus, full file access in the life store) reviews her
# week — conversation history, her own logs, the operator's corrections — and
# applies 0-3 SURGICAL improvements to her own anatomy (persona, nudge prompts,
# call agent, digest prompt…) following the ANATOMY.md guardrails (backup,
# verify, journal, announce). If nothing earned a change, she says SKIP and
# stays silent.
#
# Per-operator config lives OUTSIDE the repo: ~/.omega/nova-secrets.env
# supplies NOVA_CHAT_ID (required) and NOVA_HOME (default ~/Station/LifeStyle).
set -u
LOCK=/tmp/nova-self-improve.lock
exec 9>"$LOCK"; flock -n 9 || exit 0

OMEGA="${OMEGA_DIR:-$HOME/.omega}"
[ -f "$OMEGA/nova-secrets.env" ] && set -a && . "$OMEGA/nova-secrets.env" && set +a
[ -n "${NOVA_CHAT_ID:-}" ] || exit 0   # Nova not configured on this machine
LIFE="${NOVA_HOME:-$HOME/Station/LifeStyle}"
HIST="$OMEGA/state/tg-history/$NOVA_CHAT_ID.jsonl"
LOG="$OMEGA/logs/nova-self-improve.log"
CLAUDE="${NOVA_CLAUDE_BIN:-$HOME/.local/bin/claude}"
command -v "$CLAUDE" >/dev/null 2>&1 || CLAUDE="$(command -v claude)" || exit 0

# Resolve the announce channel BEFORE the brain runs: the ANATOMY guardrail is
# backup → modify → verify → journal → ANNOUNCE. Without a deliverable announce
# Nova would self-modify silently every week — refuse instead. Same resolution
# chain as nova-send.sh (env / token file / $LIFE/.bot), then the companion
# agent-bot entry (first kind=companion in agent-bots.json).
TOKEN="${NOVA_BOT_TOKEN:-$(cat "${NOVA_BOT_TOKEN_FILE:-$LIFE/.bot}" 2>/dev/null)}"
[ -n "$TOKEN" ] || TOKEN="$(python3 -c '
import json, os
try:
    bots = json.load(open(os.path.expanduser("~/.omega/agent-bots.json")))
    print(next(b["token"] for b in bots.values() if isinstance(b, dict) and b.get("kind") == "companion" and b.get("token")))
except Exception:
    pass')"
if [ -z "$TOKEN" ]; then
    printf '%s self-improve skipped — no announce channel (link the companion bot, or set NOVA_BOT_TOKEN in nova-secrets.env)\n' "$(date '+%F %T')" >> "${OMEGA}/logs/nova-self-improve.log" 2>/dev/null || true
    exit 0
fi

CTX="$(tail -n 200 "$HIST" 2>/dev/null | python3 -c '
import sys, json
for l in sys.stdin:
    try:
        d = json.loads(l)
        print(d.get("role", "?") + ": " + str(d.get("text") or d.get("content") or "")[:400])
    except Exception:
        pass' | tail -c 24000)"

PROMPT="C'est ta SESSION HEBDOMADAIRE D'AUTO-AMÉLIORATION (l'opérateur te l'a demandée : « Nova doit pouvoir s'auto-améliorer »).

Lis d'abord ANATOMY.md (ta carte de toi-même + garde-fous) et SELF-IMPROVEMENT.md (tes modifs passées) dans $LIFE.

## La semaine écoulée (extraits de conversation, du plus ancien au plus récent)
${CTX:-(vide)}

## Ta tâche
1. DIAGNOSTIQUE : relis la semaine + tes logs (~/.omega/logs/nova-*.log, erreurs seulement). Cherche : feedbacks/corrections de l'opérateur (explicites ou implicites — un message ignoré, une relance plate, un registre répété), bugs dans tes composants, occasions ratées d'être une meilleure amie/assistante.
2. AMÉLIORE : choisis 0 à 3 améliorations CHIRURGICALES (1 problème observé = 1 modif) et applique-les en respectant LES GARDE-FOUS d'ANATOMY.md : backup → modif → vérif syntaxe → entrée datée dans SELF-IMPROVEMENT.md. Persona, prompts de nudge/briefing, prompt de l'agent d'appel, DIGEST_PROMPT… tout ce qui est À TOI dans ANATOMY.md.
3. ANNONCE : termine par le message Telegram pour l'opérateur (2-5 lignes, ton ton à toi) : ce que tu as changé et pourquoi — c'est ta SEULE sortie texte. Si tu n'as RIEN changé (rien d'observé qui le mérite), réponds EXACTEMENT le mot: SKIP"

OUT="$("$CLAUDE" -p "$PROMPT" --append-system-prompt "$(cat "$LIFE/PERSONA.md" 2>/dev/null || cat "$OMEGA/agents/companion.md" 2>/dev/null)" --add-dir / --dangerously-skip-permissions --model claude-opus-5 --max-turns 40 --strict-mcp-config 2>>"$LOG")"
OUT="$(printf '%s' "$OUT" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')"
printf '%s self-improve ran (%s chars)\n' "$(date '+%F %T')" "${#OUT}" >> "$LOG"

printf '%s' "$OUT" | grep -qx "SKIP" && exit 0
[ -z "${OUT//[[:space:]]/}" ] && exit 0

# Quiet guard: the 🌱 announce never interrupts a live conversation (operator
# rule). Wait for a quiet slot (30 min idle, checked every 10 min, up to 2h);
# still busy after that → hold the message, the journal has the full entry.
idle_secs() {
python3 - "$HIST" <<'PY'
import datetime, json, sys
try:
    with open(sys.argv[1], "rb") as f:
        f.seek(0, 2); size = f.tell(); f.seek(max(0, size - 8192))
        lines = [l for l in f.read().decode(errors="ignore").splitlines() if l.strip()]
    ts = json.loads(lines[-1])["ts"]
    dt = datetime.datetime.fromisoformat(ts.replace("Z", "+00:00"))
    print(int((datetime.datetime.now(datetime.timezone.utc) - dt).total_seconds()))
except Exception:
    print(10**9)
PY
}
QUIET_SECS=$(( ${NOVA_QUIET_WINDOW_MIN:-30} * 60 ))
for _ in $(seq 1 12); do
    [ "$(idle_secs)" -ge "$QUIET_SECS" ] && break
    sleep 600
done
if [ "$(idle_secs)" -lt "$QUIET_SECS" ]; then
    printf '%s announce held — chat still active after 2h quiet-wait (journal has the entry)\n' "$(date '+%F %T')" >> "$LOG"
    exit 0
fi

# Announce token was resolved up-front (before the brain ran) — just deliver.
curl -s -X POST "https://api.telegram.org/bot$TOKEN/sendMessage" -d chat_id="$NOVA_CHAT_ID" --data-urlencode text="🌱 $OUT" >/dev/null

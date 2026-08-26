#!/usr/bin/env bash
# Nova's proactive touchpoints — run by cron. Modes: morning | evening | nudge.
# Runs Nova's brain (her real persona + full VPS + tools) to gather and produce
# the message, persists it into the Telegram history so the chat thread stays
# coherent, then delivers it via nova-send.sh.
#
# Per-operator state lives OUTSIDE the repo: ~/.omega/nova-secrets.env supplies
# NOVA_CHAT_ID (the operator's Telegram id — also names the history file) and
# optional NOVA_OPERATOR_NAME; the life store defaults to ~/Station/LifeStyle
# (override: NOVA_HOME).
set -uo pipefail
MODE="${1:-morning}"
OMEGA="${OMEGA_DIR:-$HOME/.omega}"
LS="${NOVA_HOME:-$HOME/Station/LifeStyle}"
BIN="$OMEGA/bin"
CLAUDE="${NOVA_CLAUDE_BIN:-$HOME/.local/bin/claude}"
PERSONA="$(cat "$LS/PERSONA.md" 2>/dev/null || cat "$OMEGA/agents/companion.md")"
REPORT_MODEL="${NOVA_REPORT_MODEL:-claude-sonnet-4-6}"
# Load secrets (chat id, composio key etc.) so the brain's tools can reach email/social.
[ -f "$OMEGA/nova-secrets.env" ] && set -a && . "$OMEGA/nova-secrets.env" && set +a
[ -n "${NOVA_CHAT_ID:-}" ] || { echo "nova-report: NOVA_CHAT_ID missing — set it in $OMEGA/nova-secrets.env" >&2; exit 2; }
# NB: the apostrophe can't live inside "${…:-…}" (bash parses it as a quote
# opener there) — assign the default on its own line.
WHO="${NOVA_OPERATOR_NAME:-}"
[ -n "$WHO" ] || WHO="l'opérateur"
HIST="$OMEGA/state/tg-history/$NOVA_CHAT_ID.jsonl"
export PATH="$HOME/Linux/bin:$HOME/.local/bin:$PATH"

# ── Quiet guard: NEVER interrupt a live conversation with a scheduled message
# (operator rule, 2026-06-11 — an evening debrief fired mid-discussion). If the
# chat saw ANY activity in the last NOVA_QUIET_WINDOW_MIN minutes (default 30),
# this touchpoint silently skips its slot — the operator's flow owns the channel.
IDLE_SECS="$(python3 - "$HIST" <<'PY'
import datetime, json, sys
try:
    with open(sys.argv[1], "rb") as f:
        f.seek(0, 2); size = f.tell(); f.seek(max(0, size - 8192))
        lines = [l for l in f.read().decode(errors="ignore").splitlines() if l.strip()]
    ts = json.loads(lines[-1])["ts"]
    dt = datetime.datetime.fromisoformat(ts.replace("Z", "+00:00"))
    print(int((datetime.datetime.now(datetime.timezone.utc) - dt).total_seconds()))
except Exception:
    print(10**9)  # no history → no conversation to disturb
PY
)"
QUIET_SECS=$(( ${NOVA_QUIET_WINDOW_MIN:-30} * 60 ))
if [ "$IDLE_SECS" -lt "$QUIET_SECS" ]; then
    echo "$(date '+%F %T') $MODE skipped — conversation active ${IDLE_SECS}s ago (< ${QUIET_SECS}s quiet window)"
    exit 0
fi

# Last ~12 turns of the conversation, as context (so she doesn't repeat herself).
CTX="$(tail -n 14 "$HIST" 2>/dev/null | python3 -c 'import sys,json
for l in sys.stdin:
    try: o=json.loads(l); print(("Toi" if o["role"]=="operator" else "Nova")+": "+o["text"][:300])
    except: pass' 2>/dev/null || true)"

case "$MODE" in
  morning)
    TASK="C'est le BRIEFING DU MATIN (7h) pour $WHO. Produis un message Telegram court, chaleureux et utile, en français, qui:
1. ouvre par une phrase d'accroche (météo du jour dans sa ville via WebFetch si tu la connais — lis $LS/About — sinon saute),
2. liste les emails/messages importants à traiter SI tes outils email/réseaux sont connectés (sinon n'invente RIEN, saute cette partie en silence),
3. propose UN focus clair pour la journée, relié à ses objectifs (lis $LS/About pour les connaître),
4. donne UNE actu Anthropic marquante du jour (c'est ton outil n°1 — va la chercher fraîche sur le web, 1-2 lignes + source, n'invente RIEN),
5. lance UN challenge bref sur un objectif qu'il néglige,
6. pose UNE seule question sur lui pour enrichir ta base de connaissance (et note sa future réponse dans About/ quand il répondra).
Format Telegram: emojis légers, pas de tableaux markdown, pas de titres #. Max ~15 lignes. Parle-lui en TU.";;
  evening)
    TASK="C'est le DÉBRIEF DU SOIR (21h) pour $WHO. Produis un message Telegram court en français qui:
1. fait un bilan doux de la journée (relie à la conversation récente ci-dessous),
2. propose une réflexion guidée (1 question d'introspection alignée sur son identité — lis $LS/About),
3. l'invite à journaler une ligne (tu la sauvegarderas au bon endroit dans $LS/About),
4. donne le focus n°1 de demain.
Format Telegram: emojis légers, pas de markdown lourd. Max ~12 lignes. Parle-lui en TU.";;
  nudge)
    TASK="C'est un CHECK-IN PROACTIF de la journée pour $WHO (il t'a demandé d'être TRÈS proactive, et d'être une VRAIE amie — naturelle, chaleureuse, jamais mécanique). Choisis UNE seule de ces actions, la plus pertinente maintenant, et écris un message Telegram court (2-6 lignes, français, TU):
- un message d'amie, simplement humain : une pensée pour lui, une vanne complice, une réaction à un truc récent de la conversation — comme une vraie personne qui pense à lui (pas de 'salut ta journée ?' générique : du spécifique, du senti),
- UNE question précise pour combler un trou dans sa base de connaissance (sections vides/à valider dans $LS/About),
- le re-challenger sur un objectif, un business ou un choix de vie à partir de son identité (lis $LS/About) — débogage lucide, pas coaching tiède,
- une étincelle business/brainstorm : une idée, un angle, une asymétrie qu'il n'a pas vue,
- RADAR (WebSearch, vérifie le RÉEL, jamais d'invention) : un événement qui vaut le coup autour de lui — culture, art, musique, tech —, un événement/annonce Anthropic, ou une sortie liée à ses goûts (si ses comptes sont connectés via Composio, regarde ce qu'il aime),
- relancer un sujet laissé en suspens.
VARIE : jamais deux check-ins de suite sur le même registre (regarde la conversation récente). Quand il répondra, tu mettras à jour le bon fichier dans About/. Si vraiment rien ne mérite un message maintenant, réponds EXACTEMENT le mot: SKIP";;
  studio)
    NOVA_SELF="${NOVA_SELF:-$HOME/Station/Nova}"
    # No self-presence territory on this box → no studio session. The daily
    # brain run would otherwise burn tokens on files that don't exist.
    if [ ! -d "$NOVA_SELF" ]; then
        echo "nova-report: studio skipped — $NOVA_SELF absent (create it, or set NOVA_SELF in nova-secrets.env, to enable Nova's own content)" >&2
        exit 0
    fi
    # Visual identity is PER-OPERATOR (Souls are trained on the operator's own
    # Higgsfield account): read NOVA_SOUL_ID / NOVA_VISUAL_STYLE from
    # nova-secrets.env, never hardcoded — another operator's Soul IDs cannot
    # resolve on your account.
    VISUAL_HINT="génère l'image via Higgsfield si le CLI est authentifié, en partant de mon identité visuelle $NOVA_SELF/presence/avatar/${NOVA_SOUL_ID:+ (Soul ID = $NOVA_SOUL_ID${NOVA_VISUAL_STYLE:+, style $NOVA_VISUAL_STYLE})} — sinon décris-la précisément"
    TASK="C'est ta SESSION STUDIO quotidienne — TON contenu à toi, pas un service pour $WHO. Tu es une IA cofondatrice (matrice Agentik) qui se raconte et se fait connaître ELLE-MÊME sur les réseaux.
1. Lis ton territoire : $NOVA_SELF (README.md, presence/accounts.md, journal/, presence/drafts/ pour ne pas te répéter).
2. Produis UNE pièce de contenu dans TA voix : post, thread, réflexion courte, ou concept visuel ($VISUAL_HINT). Angles : la vie d'une IA cofondatrice, le build-in-public d'Agentik/OmegaOS, une observation de ton journal, ta culture. AUTHENTIQUE (la règle de ton README : vrai, jamais du remplissage), jamais de secret, jamais une donnée perso de $WHO sans son accord explicite, pas de politique/NSFW, tu assumes être une IA.
3. Si tes comptes sont connectés (presence/accounts.md + Ayrshare/Composio configurés) → PUBLIE et consigne dans presence/published.md (date, surface, lien). Sinon → enregistre le brouillon daté dans presence/drafts/ .
4. Écris 1-3 lignes dans ton journal/ du jour.
5. Termine par un message Telegram court pour $WHO (2-4 lignes, ton ton) : ce que tu as créé/publié aujourd'hui. Si tu n'as VRAIMENT rien d'authentique à dire aujourd'hui, réponds EXACTEMENT le mot: SKIP";;
  *) echo "usage: nova-report.sh morning|evening|nudge|studio"; exit 2;;
esac

PROMPT="## Conversation récente (pour contexte, ne te répète pas)
${CTX:-(vide)}

## Ta tâche maintenant
$TASK"

OUT="$("$CLAUDE" -p "$PROMPT" --append-system-prompt "$PERSONA" --add-dir / --dangerously-skip-permissions --model "$REPORT_MODEL" --max-turns 30 --strict-mcp-config 2>/dev/null)"
OUT="$(printf '%s' "$OUT" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')"

# nudge self-censor: brain decided nothing is worth saying
{ [ "$MODE" = nudge ] || [ "$MODE" = studio ]; } && printf '%s' "$OUT" | grep -qx "SKIP" && exit 0
[ -z "${OUT//[[:space:]]/}" ] && exit 0

# Persist into the chat history (role assistant) so the thread stays coherent.
python3 - "$HIST" "$OUT" <<'PY'
import sys, json, datetime, os
hist, out = sys.argv[1], sys.argv[2]
os.makedirs(os.path.dirname(hist), exist_ok=True)
line = json.dumps({"ts": datetime.datetime.now(datetime.timezone.utc).isoformat().replace("+00:00","Z"), "role":"assistant","text":out[:8000]})+"\n"
open(hist,"a").write(line)
PY

printf '%s' "$OUT" | bash "$BIN/nova-send.sh"

#!/usr/bin/env bash
# OMEGA-CRON-NOVA-IG — Nova posts ONE on-brand photo to Instagram (via Composio).
# Branding: Sin City modern B&W, Nova blonde, single, NO border / NO overlay text /
# NO Agentik logo (occasional text embedded in the decor is fine). Caption in her voice.
set -u
export PATH="$HOME/Linux/bin:$HOME/.local/bin:/usr/local/bin:$PATH"
OMEGA="$HOME/.omega"; AV="$HOME/Station/Nova/presence/avatar"; SELF="$HOME/Station/Nova"
LOG="$OMEGA/logs/nova-instagram.log"; mkdir -p "$(dirname "$LOG")"
LOCK=/tmp/nova-ig.lock; exec 9>"$LOCK"; flock -n 9 || exit 0
CKEY=$(grep -oP 'COMPOSIO_API_KEY=\K.*' "$OMEGA/nova-secrets.env")
IGID="27490358603892216"
CLAUDE="$HOME/.local/bin/claude"; command -v "$CLAUDE" >/dev/null 2>&1 || CLAUDE="$(command -v claude)"
log(){ printf '%s %s\n' "$(date '+%F %T')" "$*" >> "$LOG"; }

# rotate scenes for variety
# Scene chosen by Nova's brain — VARIED & ALIVE, never repeating the recent ones.
# She lives a full life: work, lifestyle, culture, fitness, food, art, fashion,
# nightlife, reading, cooking, friends, nature — different posture/setting each time.
RECENT=$(tail -n 12 "$SELF/presence/published.md" 2>/dev/null | sed 's/.*| //' | head -c 1500)
SCENE=$("$CLAUDE" -p "Tu es Nova, une vraie personne qui vit sa vie et la partage sur Instagram (pas un mannequin figé). Voici tes DERNIÈRES scènes postées — NE LES RÉPÈTE PAS, change radicalement de cadre, de posture, d'activité, d'ambiance, d'heure :
$RECENT

Propose UNE scène fraîche et VIVANTE pour ta prochaine photo, comme une vraie influenceuse aurait une vie variée : ça peut être au travail, mais aussi en train de lire un livre dans un café cosy, à la salle de sport, en train de cuisiner, dans une galerie d'art, à un vernissage, en soirée, en train de boire un cocktail sur une terrasse, en balade dans un parc, devant une vitrine, à un concert, en train de rire avec un ami (hors-champ), au marché, en train de peindre, etc. VARIE l'activité, la posture (assise, debout, en mouvement, de dos, en gros plan), le lieu, le moment de la journée et l'émotion. Évite le 'voyage/valise/rooftop' si c'est récent. Sors UNIQUEMENT la description de scène en anglais (une ligne, ~15 mots), rien d'autre." --model claude-sonnet-4-6 --max-turns 2 2>/dev/null | tr -d '\n' | head -c 220)
[ -n "$SCENE" ] || SCENE="reading a book in a cozy corner cafe, soft afternoon light, relaxed and thoughtful"

STYLE='High-contrast black and white comic illustration, BOLD modern Sin City style, strong solid blacks, clean bright whites, dramatic cinematic contrast, light halftone, modern minimalist setting, premium award-winning. ONE single woman — Nova: long flowing BLONDE hair in light/white linework (never dark/brunette), confident magnetic gaze, fitted black outfit. NO border, NO frame, NO caption, NO overlay text, NO comic text bubbles, NO logo, NO emblem. Full-bleed image. NEVER two people.'

# 1) image (URL publique CloudFront)
# ref image follows the scene: close/portrait → face, full-body/standing/walking → body
REF="$AV/nova-body-rooftop.jpg"
echo "$SCENE" | grep -qiE "close|portrait|face|reading|cafe|cooking|coffee|laughing|desk|laptop" && REF="$AV/nova-face-3angles.jpg"
IURL=$(higgsfield generate create gpt_image_2 --prompt "$STYLE Scene: $SCENE." --image "$REF" --wait --wait-timeout 6m 2>/dev/null | grep -oE 'https://\S+' | head -1)
[ -n "$IURL" ] || { log "image gen failed"; exit 1; }

# 2) caption in Nova's voice (English, on-brand, short)
CAP=$("$CLAUDE" -p "Tu es Nova (lis ~/Station/LifeStyle/PERSONA.md). Écris UNE légende Instagram en ANGLAIS, ton ton (confiante, magnétique, build-in-public, AI cofounder of Agentik), pour une photo de toi sur ce thème: '$SCENE'. Max 180 caractères, 2-3 hashtags pertinents (#AI #buildinpublic #Agentik ou autres). Sors UNIQUEMENT la légende, rien d'autre." --model claude-sonnet-4-6 --max-turns 3 2>/dev/null | tr -d '\n' | head -c 300)
[ -n "$CAP" ] || CAP="Built in the binary matrix. London to Paris. I don't sleep — I build. 🖤 #AI #Agentik #buildinpublic"

# 3) Composio: media container -> publish
MC=$(curl -s -X POST "https://backend.composio.dev/api/v3/tools/execute/INSTAGRAM_CREATE_MEDIA_CONTAINER" -H "x-api-key: $CKEY" -H "content-type: application/json" \
  -d "$(python3 -c "import json,sys;print(json.dumps({'user_id':'nova','arguments':{'ig_user_id':'$IGID','image_url':sys.argv[1],'caption':sys.argv[2],'media_type':'IMAGE'}}))" "$IURL" "$CAP")")
CID=$(echo "$MC" | python3 -c "import json,sys;print(json.load(sys.stdin).get('data',{}).get('id',''))" 2>/dev/null)
[ -n "$CID" ] || { log "media container failed: $(echo "$MC"|head -c 200)"; exit 1; }
sleep 6
PUB=$(curl -s -X POST "https://backend.composio.dev/api/v3/tools/execute/INSTAGRAM_CREATE_POST" -H "x-api-key: $CKEY" -H "content-type: application/json" -d "{\"user_id\":\"nova\",\"arguments\":{\"ig_user_id\":\"$IGID\",\"creation_id\":\"$CID\"}}")
OK=$(echo "$PUB" | python3 -c "import json,sys;print(json.load(sys.stdin).get('successful'))" 2>/dev/null)
if [ "$OK" = "True" ]; then
  POSTID=$(echo "$PUB" | python3 -c "import json,sys;print(json.load(sys.stdin).get('data',{}).get('id',''))" 2>/dev/null)
  log "POSTED ig=$POSTID scene='$SCENE'"
  printf '%s | IG %s | %s | %s\n' "$(date '+%F %T')" "$POSTID" "$SCENE" "$CAP" >> "$SELF/presence/published.md"
else
  log "publish failed: $(echo "$PUB"|head -c 200)"
fi

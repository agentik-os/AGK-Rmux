#!/usr/bin/env bash
# OMEGA-CRON-NOVA-IG — Nova posts ONE on-brand REEL (video) to Instagram via Composio.
# Brain-picked VARIED living scene → gpt_image_2 keyframe → Seedance vertical motion
# → posted as a REELS container (direct public CloudFront URL). Branding: Sin City
# B&W, blonde, single, NO border / overlay text / logo. Caption in her voice.
set -u
export PATH="$HOME/Linux/bin:$HOME/.local/bin:/usr/local/bin:$PATH"
OMEGA="$HOME/.omega"; AV="$HOME/Station/Nova/presence/avatar"; SELF="$HOME/Station/Nova"
LOG="$OMEGA/logs/nova-instagram.log"; mkdir -p "$(dirname "$LOG")"
LOCK=/tmp/nova-ig.lock; exec 9>"$LOCK"; flock -n 9 || exit 0
CKEY=$(grep -oP 'COMPOSIO_API_KEY=\K.*' "$OMEGA/nova-secrets.env")
IGID="27490358603892216"
CLAUDE="$HOME/.local/bin/claude"; command -v "$CLAUDE" >/dev/null 2>&1 || CLAUDE="$(command -v claude)"
log(){ printf '%s %s\n' "$(date '+%F %T')" "$*" >> "$LOG"; }

# 1) Brain picks a fresh, varied, ALIVE scene (anti-repeat)
RECENT=$(tail -n 12 "$SELF/presence/published.md" 2>/dev/null | sed 's/.*| //' | head -c 1500)
SCENE=$("$CLAUDE" -p "Tu es Nova, vraie personne qui partage sa vie en REEL sur Instagram. NE RÉPÈTE PAS ces scènes récentes: $RECENT . Propose UNE scène fraîche, vivante et avec du MOUVEMENT (idéal pour une vidéo verticale): café/lecture, sport, danse, cuisine, galerie d'art, soirée, cocktail terrasse, balade parc, concert, marché, peinture, travail varié, marche en ville… Varie posture/lieu/heure/émotion. Évite voyage/valise/rooftop si récent. UNIQUEMENT la description en anglais, une ligne ~15 mots, avec un mouvement clair." --model claude-sonnet-4-6 --max-turns 2 2>/dev/null | tr -d '\n' | head -c 220)
[ -n "$SCENE" ] || SCENE="dancing alone in a sunlit loft, slow spin, eyes closed, pure joy"

STYLE='High-contrast black and white comic illustration, BOLD modern Sin City style, strong solid blacks, clean bright whites, dramatic cinematic contrast, light halftone, premium. ONE single woman — Nova: long flowing BLONDE hair in light/white linework (never dark/brunette), confident magnetic gaze, fitted black outfit. NO border, NO frame, NO caption, NO overlay text, NO comic bubbles, NO logo. Full-bleed vertical. NEVER two people.'

REF="$AV/nova-body-rooftop.jpg"
echo "$SCENE" | grep -qiE "close|portrait|face|reading|cafe|cooking|coffee|laughing|paint|desk|laptop" && REF="$AV/nova-face-3angles.jpg"

# 2) keyframe (vertical) then Seedance motion (vertical 9:16, public CloudFront URL)
KF=$(higgsfield generate create gpt_image_2 --prompt "$STYLE Scene: $SCENE. Vertical 9:16 composition." --image "$REF" --wait --wait-timeout 6m 2>/dev/null | grep -oE 'https://\S+' | head -1)
[ -n "$KF" ] || { log "reel: keyframe failed"; exit 1; }
VURL=$(higgsfield generate create kling3_0 --prompt "Nova: $SCENE. Black and white modern Sin City noir, blonde hair, mouth closed not talking, real dynamic cinematic motion, premium film look." --image "$KF" --duration 6 --aspect_ratio 9:16 --mode pro --sound off --wait --wait-timeout 10m 2>/dev/null | grep -oE 'https://\S+' | head -1)
[ -n "$VURL" ] || { log "reel: video failed"; exit 1; }

# 3) caption in Nova's voice
CAP=$("$CLAUDE" -p "Tu es Nova (lis ~/Station/LifeStyle/PERSONA.md). Légende Instagram en ANGLAIS pour ce reel: '$SCENE'. Ton ton (confiante, magnétique, vivante, AI cofounder of Agentik). Max 150 caractères, 2-3 hashtags. UNIQUEMENT la légende." --model claude-sonnet-4-6 --max-turns 2 2>/dev/null | tr -d '\n' | head -c 280)
[ -n "$CAP" ] || CAP="Living it, not waiting for it. 🖤 #AI #Agentik #buildinpublic"

# 4) Composio REELS container → poll processing → publish
MC=$(curl -s -X POST "https://backend.composio.dev/api/v3/tools/execute/INSTAGRAM_CREATE_MEDIA_CONTAINER" -H "x-api-key: $CKEY" -H "content-type: application/json" \
  -d "$(python3 -c "import json,sys;print(json.dumps({'user_id':'nova','arguments':{'ig_user_id':'$IGID','video_url':sys.argv[1],'caption':sys.argv[2],'media_type':'REELS'}}))" "$VURL" "$CAP")")
CID=$(echo "$MC" | python3 -c "import json,sys;print(json.load(sys.stdin).get('data',{}).get('id',''))" 2>/dev/null)
[ -n "$CID" ] || { log "reel: container failed: $(echo "$MC"|head -c 200)"; exit 1; }
# poll until video processed (FINISHED)
for i in $(seq 1 30); do
  ST=$(curl -s -X POST "https://backend.composio.dev/api/v3/tools/execute/INSTAGRAM_GET_POST_STATUS" -H "x-api-key: $CKEY" -H "content-type: application/json" -d "{\"user_id\":\"nova\",\"arguments\":{\"creation_id\":\"$CID\"}}" | python3 -c "import json,sys;d=json.load(sys.stdin).get('data',{});print(d.get('status_code') or d.get('status') or '')" 2>/dev/null)
  [ "$ST" = "FINISHED" ] && break
  [ "$ST" = "ERROR" ] && { log "reel: processing ERROR"; exit 1; }
  sleep 12
done
PUB=$(curl -s -X POST "https://backend.composio.dev/api/v3/tools/execute/INSTAGRAM_CREATE_POST" -H "x-api-key: $CKEY" -H "content-type: application/json" -d "{\"user_id\":\"nova\",\"arguments\":{\"ig_user_id\":\"$IGID\",\"creation_id\":\"$CID\"}}")
OK=$(echo "$PUB" | python3 -c "import json,sys;print(json.load(sys.stdin).get('successful'))" 2>/dev/null)
if [ "$OK" = "True" ]; then
  PID=$(echo "$PUB" | python3 -c "import json,sys;print(json.load(sys.stdin).get('data',{}).get('id',''))" 2>/dev/null)
  log "POSTED REEL ig=$PID scene='$SCENE'"
  printf '%s | REEL %s | %s | %s\n' "$(date '+%F %T')" "$PID" "$SCENE" "$CAP" >> "$SELF/presence/published.md"
else
  log "reel: publish failed: $(echo "$PUB"|head -c 200)"
fi

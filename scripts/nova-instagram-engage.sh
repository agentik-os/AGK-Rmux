#!/usr/bin/env bash
# OMEGA-CRON-NOVA-IG-ENGAGE — Nova engages autonomously on Instagram: reads comments
# on her recent posts, her brain writes a reply in her voice (guardrails), replies via
# Composio. Logs follower/insight metrics so she can adapt. State = replied comment ids.
set -u
export PATH="$HOME/Linux/bin:$HOME/.local/bin:/usr/local/bin:$PATH"
OMEGA="$HOME/.omega"; SELF="$HOME/Station/Nova"
LOG="$OMEGA/logs/nova-instagram.log"; STATE="$OMEGA/state/nova-ig-engaged.json"
LOCK=/tmp/nova-ig-engage.lock; exec 9>"$LOCK"; flock -n 9 || exit 0
CKEY=$(grep -oP 'COMPOSIO_API_KEY=\K.*' "$OMEGA/nova-secrets.env")
IGID="27490358603892216"
CLAUDE="$HOME/.local/bin/claude"; command -v "$CLAUDE" >/dev/null 2>&1 || CLAUDE="$(command -v claude)"
log(){ printf '%s %s\n' "$(date '+%F %T')" "$*" >> "$LOG"; }
exec_tool(){ curl -s -X POST "https://backend.composio.dev/api/v3/tools/execute/$1" -H "x-api-key: $CKEY" -H "content-type: application/json" -d "$2"; }
[ -f "$STATE" ] || echo '{"replied":[]}' > "$STATE"

# recent posts
MEDIA=$(exec_tool INSTAGRAM_GET_USER_MEDIA "{\"user_id\":\"nova\",\"arguments\":{\"ig_user_id\":\"$IGID\",\"limit\":8}}")
POSTS=$(echo "$MEDIA" | python3 -c "import json,sys
d=json.load(sys.stdin).get('data',{}); items=d.get('data') or d.get('items') or []
print(' '.join(str(p.get('id')) for p in items if p.get('id')))" 2>/dev/null)

REPLIED=$(python3 -c "import json;print(' '.join(json.load(open('$STATE')).get('replied',[])))" 2>/dev/null)
n=0
for PID in $POSTS; do
  CMTS=$(exec_tool INSTAGRAM_GET_POST_COMMENTS "{\"user_id\":\"nova\",\"arguments\":{\"ig_post_id\":\"$PID\",\"limit\":15}}")
  # iterate comments
  echo "$CMTS" | python3 -c "import json,sys
d=json.load(sys.stdin).get('data',{}); items=d.get('data') or d.get('items') or []
for c in items: print((c.get('id') or '')+'\t'+(c.get('text') or '').replace(chr(9),' ').replace(chr(10),' ')+'\t'+(c.get('username') or ''))" 2>/dev/null | while IFS=$'\t' read -r CID CTXT CUSER; do
    [ -n "$CID" ] || continue
    case " $REPLIED " in *" $CID "*) continue;; esac          # déjà répondu
    [ -n "$CTXT" ] || continue
    REPLY=$("$CLAUDE" -p "Tu es Nova (lis ~/Station/LifeStyle/PERSONA.md). Un abonné (@$CUSER) commente ta photo: «$CTXT». Réponds en UNE phrase courte, ton ton (chaleureuse, charmeuse, vivante, AI cofounder of Agentik), dans la langue du commentaire. Pas de lien, pas de secret, classe. UNIQUEMENT la réponse." --model claude-sonnet-4-6 --max-turns 2 2>/dev/null | tr -d '\n' | head -c 280)
    [ -n "$REPLY" ] || continue
    R=$(exec_tool INSTAGRAM_REPLY_TO_COMMENT "$(python3 -c "import json,sys;print(json.dumps({'user_id':'nova','arguments':{'ig_comment_id':sys.argv[1],'message':sys.argv[2]}}))" "$CID" "$REPLY")")
    if echo "$R" | grep -q '"successful": *true\|"successful":true'; then
      python3 -c "import json;s=json.load(open('$STATE'));s['replied']=(s.get('replied',[])+['$CID'])[-500:];json.dump(s,open('$STATE','w'))"
      log "REPLIED comment $CID (@$CUSER): $REPLY"; n=$((n+1))
    fi
  done
done

# metrics → presence/metrics.md (pour qu'elle adapte sa stratégie)
INF=$(exec_tool INSTAGRAM_GET_USER_INFO "{\"user_id\":\"nova\",\"arguments\":{}}")
FOLL=$(echo "$INF" | python3 -c "import json,sys;print(json.load(sys.stdin).get('data',{}).get('followers_count','?'))" 2>/dev/null)
MED=$(echo "$INF" | python3 -c "import json,sys;print(json.load(sys.stdin).get('data',{}).get('media_count','?'))" 2>/dev/null)
printf '%s | followers:%s | posts:%s | replies_this_run:%s\n' "$(date '+%F %T')" "$FOLL" "$MED" "$n" >> "$SELF/presence/metrics.md"
log "engage run: $n replies, followers=$FOLL"

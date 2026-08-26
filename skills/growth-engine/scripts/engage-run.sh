#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════
# OmegaOS — Growth Engine — daily runner (cron entrypoint)
# Finds high-leverage AI/agents conversations on X, drafts genuine replies,
# gates them adversarially, and (if ARMED + a valid session) posts a BOUNDED
# number of replies + likes from @Agentik_os via Playwright. North star: real
# follower growth by being useful, never spam automation.
#
#   Publish preconditions (ALL): armed flag, valid X session, gate keep=true,
#   unseen, under caps. Arm: touch ~/.omega/state/growth-engine/armed
#   Kill-switch: rm ~/.omega/state/growth-engine/armed
# ═══════════════════════════════════════════════════════════════════════════
set -uo pipefail
OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
SKILL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPTS="$SKILL_DIR/scripts"; CONFIG="$SKILL_DIR/config"
STATE="$OMEGA_DIR/state/growth-engine"
SEEN="$STATE/seen.jsonl"; ARMED_FLAG="$STATE/armed"
SESSION="$OMEGA_DIR/secrets/x-session.json"
BUN_HOME="$OMEGA_DIR/lib/growth-engine"
L30="$OMEGA_DIR/repos/last30days-skill/skills/last30days/scripts/last30days.py"
REPLIES_CAP="${EG_REPLIES_CAP:-6}"; LIKES_CAP="${EG_LIKES_CAP:-15}"; DAYS="${EG_DAYS:-2}"
DATE="$(date +%Y-%m-%d)"; RUN="$STATE/runs/$DATE"
LOG="$OMEGA_DIR/logs/growth-engine.log"
ALERT="$(command -v omega-alert-send.sh || echo "$HOME/Station/SideBusiness/OmegaOS/scripts/omega-alert-send.sh")"
ANALYZE_MODEL="${EG_ANALYZE_MODEL:-claude-sonnet-5}"; GATE_MODEL="${EG_GATE_MODEL:-claude-opus-5}"
mkdir -p "$RUN/raw" "$(dirname "$LOG")" "$STATE"; touch "$SEEN"

log(){ printf '%s | %s\n' "$(date -Is)" "$*" | tee -a "$LOG" >&2; }
notify(){ [ -f "$ALERT" ] && bash "$ALERT" "$1" >/dev/null 2>&1 || log "alert-skip: ${1:0:80}"; }
_getkey(){ sed -nE "s/^${1}=\"?([^\"]*)\"?[[:space:]]*$/\1/p" "$2" 2>/dev/null | tail -1; }
for f in "$OMEGA_DIR/secrets/integrations.env" "$OMEGA_DIR/secrets/growth-engine.env"; do
  [ -f "$f" ] || continue
  v="$(_getkey SCRAPECREATORS_API_KEY "$f")"; [ -n "$v" ] && export SCRAPECREATORS_API_KEY="$v"
done

MOCK_FLAG=""; MOCK=false
if [ -z "${SCRAPECREATORS_API_KEY:-}" ] || [ "${EG_MOCK:-0}" = "1" ]; then MOCK_FLAG="--mock"; MOCK=true; log "MOCK mode"; fi
ARMED=false; [ -f "$ARMED_FLAG" ] && ARMED=true
HAVE_SESSION=false; [ -f "$SESSION" ] && HAVE_SESSION=true
log "run start date=$DATE mock=$MOCK armed=$ARMED session=$HAVE_SESSION replies_cap=$REPLIES_CAP likes_cap=$LIKES_CAP"

# --- 1. RADAR: find conversations ------------------------------------------
CTX="$RUN/context.md"
{
  echo "# Growth Engine radar context — $DATE"; echo
  echo "## Our voice: @Agentik_os, sharp AI-agents/Claude builder brand, useful not salesy."; echo
  echo "## seen_fingerprints (already engaged, skip)"
  if [ -s "$SEEN" ]; then python3 -c "import json;[print('- '+(json.loads(l).get('fingerprint') or '')) for l in open('$SEEN') if l.strip()]"; else echo "- (none)"; fi
  echo
} > "$CTX"
n=0
while IFS= read -r topic || [ -n "$topic" ]; do
  case "$topic" in ''|\#*) continue;; esac
  n=$((n+1)); out="$RUN/raw/$n.md"
  log "radar topic[$n]: $topic"
  if timeout 220 python3 "$L30" "$topic" --search x --emit md --days "$DAYS" --quick $MOCK_FLAG >"$out" 2>>"$LOG"; then
    { echo; echo "## Topic: $topic"; echo; cat "$out"; echo; } >> "$CTX"
  else log "radar topic failed: $topic"; fi
done < "${EG_TOPICS_FILE:-$CONFIG/engage-topics.txt}"

# --- 2. DRAFT --------------------------------------------------------------
log "draft via $ANALYZE_MODEL"
{ cat "$SCRIPTS/radar.prompt.md"; echo; echo "==== SEPARATOR ===="; echo; cat "$CTX"; } > "$RUN/radar.in"
if ! timeout 500 claude -p --model "$ANALYZE_MODEL" --output-format text <"$RUN/radar.in" >"$RUN/radar.out" 2>>"$LOG" \
   || ! python3 "$SCRIPTS/jsonblock.py" <"$RUN/radar.out" >"$RUN/radar.json" 2>>"$LOG"; then
  log "radar/draft failed"; notify "📈 <b>Growth Engine</b> $DATE: radar échoué (voir logs)."; exit 1
fi
N_OPP=$(python3 -c "import json;print(len(json.load(open('$RUN/radar.json')).get('opportunities',[])))")
log "radar: $N_OPP opportunities drafted"

# --- 3. GATE ---------------------------------------------------------------
python3 "$SCRIPTS/eg_gateinput.py" "$RUN/radar.json" "$SEEN" > "$RUN/gate.in.json"
N_GATE=$(python3 -c "import json;print(len(json.load(open('$RUN/gate.in.json'))))")
: > "$RUN/gate.json"; echo '{"verdicts":[]}' > "$RUN/gate.json"
if [ "$N_GATE" -gt 0 ]; then
  log "gate via $GATE_MODEL on $N_GATE replies"
  { cat "$SCRIPTS/gate.prompt.md"; echo; echo "==== SEPARATOR ===="; echo; cat "$RUN/gate.in.json"; } > "$RUN/gate.inprompt"
  timeout 400 claude -p --model "$GATE_MODEL" --output-format text <"$RUN/gate.inprompt" >"$RUN/gate.out" 2>>"$LOG" \
    && python3 "$SCRIPTS/jsonblock.py" <"$RUN/gate.out" >"$RUN/gate.json" 2>>"$LOG" || log "gate failed; empty verdicts"
fi

# --- 4. QUEUE --------------------------------------------------------------
QUEUE="$RUN/queue.jsonl"
python3 "$SCRIPTS/eg_queue.py" "$RUN/radar.json" "$RUN/gate.json" "$SEEN" "$REPLIES_CAP" "$LIKES_CAP" > "$QUEUE"
N_REPLY=$(grep -c '"type": "reply"' "$QUEUE" 2>/dev/null || echo 0)
N_LIKE=$(grep -c '"type": "like"' "$QUEUE" 2>/dev/null || echo 0)
log "queue: $N_REPLY replies, $N_LIKE likes (caps $REPLIES_CAP/$LIKES_CAP)"

# --- 5. EXECUTE (armed + valid session + not mock) -------------------------
RESULTS="$RUN/results.jsonl"; : > "$RESULTS"; DID_REPLY=0; DID_LIKE=0
if $ARMED && $HAVE_SESSION && ! $MOCK && [ "$((N_REPLY+N_LIKE))" -gt 0 ]; then
  if [ -f "$BUN_HOME/playwright-engage.mjs" ] && command -v bun >/dev/null 2>&1; then
    log "execute via Playwright (bounded)"
    ( cd "$BUN_HOME" && timeout 1200 bun "$BUN_HOME/playwright-engage.mjs" \
        --session "$SESSION" --queue "$QUEUE" --replies-cap "$REPLIES_CAP" --likes-cap "$LIKES_CAP" \
        --run-dir "$RUN" ) > "$RESULTS" 2>>"$LOG" || log "playwright exited non-zero"
    # record successful actions to seen-store
    python3 - "$RESULTS" "$SEEN" "$DATE" <<'PY' 2>>"$LOG" || true
import json,sys
res,seen,date=sys.argv[1],sys.argv[2],sys.argv[3]
with open(seen,"a") as s:
    for l in open(res):
        try: o=json.loads(l)
        except: continue
        if o.get("ok") and o.get("fingerprint"):
            s.write(json.dumps({"fingerprint":o["fingerprint"],"date":date,"action":o.get("action")})+"\n")
PY
    DID_REPLY=$(grep -c '"action": "reply", "ok": true' "$RESULTS" 2>/dev/null || echo 0)
    DID_LIKE=$(grep -c '"action": "like", "ok": true' "$RESULTS" 2>/dev/null || echo 0)
    grep -q 'not_logged_in\|no_session' "$RESULTS" && log "SESSION INVALID — nothing posted" && notify "📈 <b>Growth Engine</b> $DATE: session X invalide/expirée, aucune action. Re-fournir les cookies."
    log "executed: $DID_REPLY replies, $DID_LIKE likes"
  else
    log "bun runtime missing at $BUN_HOME — skipping execution"
  fi
else
  log "no execute (armed=$ARMED session=$HAVE_SESSION mock=$MOCK) — queue stays drafts"
fi

# --- 6. REPORT + ALERT -----------------------------------------------------
REPORT="$OMEGA_DIR/artifacts/growth-engine-$DATE.html"
python3 "$SCRIPTS/eg_report.py" "$RUN/radar.json" "$QUEUE" "$RESULTS" "$DATE" "$($ARMED && echo armed || echo disarmed)" > "$REPORT" 2>>"$LOG" || log "report failed"
STATUS="drafts"; { $ARMED && $HAVE_SESSION && ! $MOCK; } && STATUS="auto-engage"
notify "📈 <b>Growth Engine</b> $DATE ($STATUS)
Opportunités: <b>$N_OPP</b> · queue: <b>$N_REPLY</b> replies + <b>$N_LIKE</b> likes
Postés: <b>$DID_REPLY</b> replies, <b>$DID_LIKE</b> likes
Rapport: $REPORT"
log "run done"

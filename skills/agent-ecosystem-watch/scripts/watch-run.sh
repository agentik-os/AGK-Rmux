#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════
# OmegaOS — Agent-Ecosystem Watch — daily runner (cron entrypoint)
# Reads the X ecosystem (Claude + agents), extracts integratable improvements,
# writes an HTML report + Telegram alert, and (if ARMED) auto-publishes vetted
# best-practice tweets to @Agentik_os via zernio, behind an adversarial gate.
#
#   Publishing preconditions (ALL required): armed flag present, real key
#   (not mock), gate keep=true, not already in seen-store, under daily cap.
#   Arm:    touch ~/.omega/state/ecosystem-watch/armed
#   Disarm: rm    ~/.omega/state/ecosystem-watch/armed        (kill-switch)
# ═══════════════════════════════════════════════════════════════════════════
set -uo pipefail

OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
SKILL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPTS="$SKILL_DIR/scripts"; CONFIG="$SKILL_DIR/config"
STATE="$OMEGA_DIR/state/ecosystem-watch"
SEEN="$STATE/seen.jsonl"; ARMED_FLAG="$STATE/armed"
ART="$OMEGA_DIR/artifacts"
L30="$OMEGA_DIR/repos/last30days-skill/skills/last30days/scripts/last30days.py"
ZPROJ="${EW_ZERNIO_PROJECT:-agentik-os}"
CAP="${EW_DAILY_TWEET_CAP:-5}"; DAYS="${EW_DAYS:-2}"
DATE="$(date +%Y-%m-%d)"; RUN="$STATE/runs/$DATE"
LOG="$OMEGA_DIR/logs/ecosystem-watch.log"
ALERT="$(command -v omega-alert-send.sh || echo "$HOME/Station/SideBusiness/OmegaOS/scripts/omega-alert-send.sh")"
ANALYZE_MODEL="${EW_ANALYZE_MODEL:-claude-sonnet-5}"
GATE_MODEL="${EW_GATE_MODEL:-claude-opus-5}"
mkdir -p "$RUN/raw" "$ART" "$(dirname "$LOG")" "$STATE"; touch "$SEEN"

log(){ printf '%s | %s\n' "$(date -Is)" "$*" | tee -a "$LOG" >&2; }
notify(){ [ -f "$ALERT" ] && bash "$ALERT" "$1" >/dev/null 2>&1 || log "alert-skip: ${1:0:80}"; }

# --- secrets (R-ENV: extract only the key we need; never source arbitrary env) ---
_getkey(){ sed -nE "s/^${1}=\"?([^\"]*)\"?[[:space:]]*$/\1/p" "$2" 2>/dev/null | tail -1; }
for f in "$OMEGA_DIR/secrets/integrations.env" "$OMEGA_DIR/secrets/ecosystem-watch.env"; do
  [ -f "$f" ] || continue
  v="$(_getkey SCRAPECREATORS_API_KEY "$f")"; [ -n "$v" ] && export SCRAPECREATORS_API_KEY="$v"
done

MOCK_FLAG=""; MOCK=false
if [ -z "${SCRAPECREATORS_API_KEY:-}" ] || [ "${EW_MOCK:-0}" = "1" ]; then
  MOCK_FLAG="--mock"; MOCK=true; log "MOCK mode (no SCRAPECREATORS_API_KEY or EW_MOCK=1)"
fi
ARMED=false; [ -f "$ARMED_FLAG" ] && ARMED=true
log "run start date=$DATE mock=$MOCK armed=$ARMED cap=$CAP days=$DAYS proj=$ZPROJ"

# --- 1. READ ---------------------------------------------------------------
CTX="$RUN/context.md"
{
  echo "# Agent-Ecosystem Watch context — $DATE"; echo
  echo "## Reference accounts"; grep -vE '^\s*#|^\s*$' "$CONFIG/accounts.txt" | paste -sd, -; echo
  echo "## seen_fingerprints (already shared, skip these)"
  if [ -s "$SEEN" ]; then
    python3 -c "import json,sys;[print('- '+ (json.loads(l).get('fingerprint') or '')) for l in open('$SEEN') if l.strip()]"
  else echo "- (none)"; fi
  echo
} > "$CTX"

n=0
while IFS= read -r topic || [ -n "$topic" ]; do
  case "$topic" in ''|\#*) continue;; esac
  n=$((n+1)); out="$RUN/raw/$n.md"
  log "read topic[$n]: $topic"
  if timeout 220 python3 "$L30" "$topic" --search x --emit md --days "$DAYS" --quick $MOCK_FLAG >"$out" 2>>"$LOG"; then
    { echo; echo "## Topic: $topic"; echo; cat "$out"; echo; } >> "$CTX"
  else
    log "topic failed (skipped): $topic"
  fi
done < "${EW_TOPICS_FILE:-$CONFIG/topics.txt}"
log "read done: $n topics attempted, context=$(wc -c <"$CTX") bytes"

# --- 2. ANALYZE ------------------------------------------------------------
log "analyze via $ANALYZE_MODEL"
{ cat "$SCRIPTS/analyze.prompt.md"; echo; echo "==== SEPARATOR ===="; echo; cat "$CTX"; } > "$RUN/analyze.in"
if ! timeout 500 claude -p --model "$ANALYZE_MODEL" --output-format text <"$RUN/analyze.in" >"$RUN/analyze.out" 2>>"$LOG"; then
  log "analyze call failed"; notify "🔭 <b>Ecosystem Watch</b> $DATE: analyse échouée (voir logs)."; exit 1
fi
if ! python3 "$SCRIPTS/jsonblock.py" <"$RUN/analyze.out" >"$RUN/analysis.json" 2>>"$LOG"; then
  log "analysis json parse failed"; notify "🔭 <b>Ecosystem Watch</b> $DATE: JSON analyse invalide."; exit 1
fi
N_IMP=$(python3 -c "import json;print(len(json.load(open('$RUN/analysis.json')).get('improvements',[])))")
N_CAND=$(python3 -c "import json;print(len(json.load(open('$RUN/analysis.json')).get('candidates',[])))")
log "analysis: $N_IMP improvements, $N_CAND candidate tweets"

# --- 3. GATE (adversarial, independent lens per R-VERIFY) -------------------
PUBLISHED="$RUN/published.jsonl"; : > "$PUBLISHED"
python3 "$SCRIPTS/ew_gateinput.py" "$RUN/analysis.json" "$SEEN" > "$RUN/gate.in.json"
N_GATE=$(python3 -c "import json;print(len(json.load(open('$RUN/gate.in.json'))))")
SURVIVORS="$RUN/survivors.jsonl"; : > "$SURVIVORS"
if [ "$N_GATE" -gt 0 ]; then
  log "gate via $GATE_MODEL on $N_GATE unseen candidates"
  { cat "$SCRIPTS/gate.prompt.md"; echo; echo "==== SEPARATOR ===="; echo; cat "$RUN/gate.in.json"; } > "$RUN/gate.inprompt"
  if timeout 400 claude -p --model "$GATE_MODEL" --output-format text <"$RUN/gate.inprompt" >"$RUN/gate.out" 2>>"$LOG" \
     && python3 "$SCRIPTS/jsonblock.py" <"$RUN/gate.out" >"$RUN/gate.json" 2>>"$LOG"; then
    python3 "$SCRIPTS/ew_survivors.py" "$RUN/analysis.json" "$RUN/gate.json" "$SEEN" "$CAP" > "$SURVIVORS"
  else
    log "gate failed; publishing nothing this run"
  fi
fi
N_SURV=$(wc -l <"$SURVIVORS" | tr -d ' ')
log "gate done: $N_SURV survivors (cap=$CAP)"

# --- 4. PUBLISH (only if armed AND real key) --------------------------------
if $ARMED && ! $MOCK && [ "$N_SURV" -gt 0 ]; then
  while IFS= read -r line; do
    [ -n "$line" ] || continue
    fp=$(python3 -c "import json,sys;print(json.loads(sys.argv[1])['fingerprint'])" "$line")
    txt=$(python3 -c "import json,sys;print(json.loads(sys.argv[1])['text'])" "$line")
    ZFLAG=""; [ "${EW_ZERNIO_DRYRUN:-0}" = "1" ] && ZFLAG="--dry-run"
    log "publish fp=$fp dryrun=${EW_ZERNIO_DRYRUN:-0}"
    if omega-zernio post "$ZPROJ" --text "$txt" --platforms twitter $ZFLAG >>"$LOG" 2>&1; then
      if [ "${EW_ZERNIO_DRYRUN:-0}" != "1" ]; then
        python3 -c "import json,sys;print(json.dumps({'fingerprint':sys.argv[1],'date':sys.argv[2]}))" "$fp" "$DATE" >> "$SEEN"
        echo "$line" >> "$PUBLISHED"
      fi
    else
      log "zernio post failed fp=$fp"
    fi
  done < "$SURVIVORS"
  log "published $(wc -l <"$PUBLISHED" | tr -d ' ') tweets"
else
  log "no publish (armed=$ARMED mock=$MOCK survivors=$N_SURV) — tweets stay drafts"
fi

# --- 5. REPORT -------------------------------------------------------------
ARM_ARG="disarmed"; $ARMED && ARM_ARG="armed"
MOCK_ARG="-"; $MOCK && MOCK_ARG="mock"
REPORT="$ART/ecosystem-watch-$DATE.html"
python3 "$SCRIPTS/report.py" "$RUN/analysis.json" "$PUBLISHED" "$ARM_ARG" "$MOCK_ARG" > "$REPORT" 2>>"$LOG" \
  && cp "$REPORT" "$RUN/report.html" && log "report -> $REPORT" || log "report render failed"

# --- 6. ALERT --------------------------------------------------------------
N_PUB=$(wc -l <"$PUBLISHED" | tr -d ' ')
TOP=$(python3 -c "import json;i=sorted(json.load(open('$RUN/analysis.json')).get('improvements',[]),key=lambda x:-float(x.get('integratable_to_omega',0) or 0));print(i[0]['title'] if i else 'aucune')" 2>/dev/null)
STATUS="drafts"; { $ARMED && ! $MOCK; } && STATUS="auto-publish"
notify "🔭 <b>Ecosystem Watch</b> $DATE ($STATUS)
Améliorations: <b>$N_IMP</b> · tweets vettés: <b>$N_SURV</b> · publiés: <b>$N_PUB</b>
Top: $TOP
Rapport: $REPORT"
log "run done"

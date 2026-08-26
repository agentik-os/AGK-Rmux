#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════
# OmegaOS — Claude Changelog Adopt — daily runner (cron entrypoint)
# Reads the OFFICIAL Claude Code changelog, diffs it against the last version
# OmegaOS absorbed, classifies each NEW entry for an OmegaOS/agent improvement,
# gates the proposals adversarially, writes an HTML report + Telegram alert, and
# (only if ARMED) dispatches each vetted, in-scope adoption to an oracle behind
# the full quality gate (In-Review handoff — never auto-Done, never force-push).
#
#   Adoption preconditions (ALL required): armed flag, gate keep=true,
#   in_scope=true (doctrine/agents/skills/install.sh — never core-rust),
#   fingerprint unseen, under the daily cap.
#   Arm:    touch ~/.omega/state/changelog-adopt/armed
#   Disarm: rm    ~/.omega/state/changelog-adopt/armed        (kill-switch)
# ═══════════════════════════════════════════════════════════════════════════
set -uo pipefail

OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
SKILL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPTS="$SKILL_DIR/scripts"
STATE="$OMEGA_DIR/state/changelog-adopt"
SEEN="$STATE/seen.jsonl"; ARMED_FLAG="$STATE/armed"; WATERMARK="$STATE/last_version"
ART="$OMEGA_DIR/artifacts"
DATE="$(date +%Y-%m-%d)"; RUN="$STATE/runs/$DATE"
LOG="$OMEGA_DIR/logs/changelog-adopt.log"
ALERT="$(command -v omega-alert-send.sh || echo "$HOME/Station/SideBusiness/OmegaOS/scripts/omega-alert-send.sh")"

CHANGELOG_URL="${CA_CHANGELOG_URL:-https://raw.githubusercontent.com/anthropics/claude-code/main/CHANGELOG.md}"
CLASSIFY_MODEL="${CA_CLASSIFY_MODEL:-claude-opus-5}"
GATE_MODEL="${CA_GATE_MODEL:-claude-opus-5}"
CAP="${CA_DAILY_ADOPT_CAP:-3}"
PROJECT="${CA_DISPATCH_PROJECT:-OmegaOS}"
SEED_VERSIONS="${CA_SEED_VERSIONS:-1}"
DRYRUN="${CA_DRYRUN:-0}"

mkdir -p "$RUN" "$ART" "$(dirname "$LOG")" "$STATE"; touch "$SEEN"

log(){ printf '%s | %s\n' "$(date -Is)" "$*" | tee -a "$LOG" >&2; }
notify(){ [ -f "$ALERT" ] && bash "$ALERT" "$1" >/dev/null 2>&1 || log "alert-skip: ${1:0:80}"; }

ARMED=false; [ -f "$ARMED_FLAG" ] && ARMED=true
LAST="$(cat "$WATERMARK" 2>/dev/null || echo "")"
[ -n "${CA_FORCE_VERSION:-}" ] && LAST="$CA_FORCE_VERSION"
log "run start date=$DATE armed=$ARMED dryrun=$DRYRUN last=${LAST:-seed} cap=$CAP proj=$PROJECT"

# --- 1. FETCH --------------------------------------------------------------
RAW="$RUN/CHANGELOG.md"
if ! curl -fsSL --max-time 40 "$CHANGELOG_URL" -o "$RAW" 2>>"$LOG" || [ ! -s "$RAW" ]; then
  log "fetch failed: $CHANGELOG_URL"
  notify "📜 <b>Changelog Adopt</b> $DATE: fetch du changelog échoué (voir logs)."
  exit 1
fi
log "fetched $(wc -l <"$RAW") lines from $CHANGELOG_URL"

# --- 2. DIFF vs watermark --------------------------------------------------
NEW="$RUN/new.json"
if ! python3 "$SCRIPTS/parse_changelog.py" "${LAST:--}" "$SEED_VERSIONS" <"$RAW" >"$NEW" 2>>"$LOG"; then
  log "parse failed"; notify "📜 <b>Changelog Adopt</b> $DATE: parse du changelog échoué."; exit 1
fi
LATEST="$(python3 -c "import json;print(json.load(open('$NEW')).get('latest',''))")"
N_NEW="$(python3 -c "import json;print(len(json.load(open('$NEW')).get('new_entries',[])))")"
log "diff: latest=$LATEST new_entries=$N_NEW (since ${LAST:-seed})"

write_watermark(){ [ -n "$LATEST" ] && printf '%s\n' "$LATEST" > "$WATERMARK"; }

if [ "$N_NEW" -eq 0 ]; then
  log "no new entries — up to date at $LATEST"
  write_watermark
  # Quiet success: no alert (avoid daily noise when nothing shipped).
  exit 0
fi

# --- 3. CLASSIFY (opus) ----------------------------------------------------
log "classify via $CLASSIFY_MODEL on $N_NEW entries"
{ cat "$SCRIPTS/classify.prompt.md"; echo; echo "==== SEPARATOR ===="; echo;
  python3 -c "import json;print(json.dumps(json.load(open('$NEW'))['new_entries'],ensure_ascii=False))"; } > "$RUN/classify.in"
if ! timeout 600 claude -p --model "$CLASSIFY_MODEL" --output-format text <"$RUN/classify.in" >"$RUN/classify.out" 2>>"$LOG" \
   || ! python3 "$SCRIPTS/jsonblock.py" <"$RUN/classify.out" >"$RUN/analysis.json" 2>>"$LOG"; then
  log "classify failed"; notify "📜 <b>Changelog Adopt</b> $DATE: classification échouée (voir logs)."; exit 1
fi
N_HIGH="$(python3 -c "import json;a=json.load(open('$RUN/analysis.json')).get('assessments',[]);print(sum(1 for x in a if x.get('relevance')=='high'))")"
N_MED="$(python3 -c "import json;a=json.load(open('$RUN/analysis.json')).get('assessments',[]);print(sum(1 for x in a if x.get('relevance')=='medium'))")"
log "classify: $N_HIGH high, $N_MED medium"

# --- 4. GATE (opus, adversarial) -------------------------------------------
# Candidates: in_scope AND relevance in {high, medium} AND not already seen.
python3 - "$RUN/analysis.json" "$SEEN" > "$RUN/gate.in.json" <<'PY'
import json,sys
analysis=json.load(open(sys.argv[1]))
seen=set()
try:
    for l in open(sys.argv[2]):
        l=l.strip()
        if l: seen.add(json.loads(l).get("fingerprint"))
except FileNotFoundError: pass
out=[]
for a in analysis.get("assessments",[]):
    if a.get("in_scope") and a.get("relevance") in ("high","medium") and a.get("fingerprint") not in seen:
        out.append({k:a.get(k) for k in ("fingerprint","version","entry","category","surface","proposal","integratability")})
print(json.dumps(out,ensure_ascii=False))
PY
N_CAND="$(python3 -c "import json;print(len(json.load(open('$RUN/gate.in.json'))))")"
GATE_JSON="-"
if [ "$N_CAND" -gt 0 ]; then
  log "gate via $GATE_MODEL on $N_CAND candidates"
  { cat "$SCRIPTS/gate.prompt.md"; echo; echo "==== SEPARATOR ===="; echo; cat "$RUN/gate.in.json"; } > "$RUN/gate.in"
  if timeout 500 claude -p --model "$GATE_MODEL" --output-format text <"$RUN/gate.in" >"$RUN/gate.out" 2>>"$LOG" \
     && python3 "$SCRIPTS/jsonblock.py" <"$RUN/gate.out" >"$RUN/gate.json" 2>>"$LOG"; then
    GATE_JSON="$RUN/gate.json"
  else
    log "gate failed — nothing will be dispatched this run"
  fi
fi
N_KEPT=0
[ "$GATE_JSON" != "-" ] && N_KEPT="$(python3 -c "import json;print(sum(1 for v in json.load(open('$GATE_JSON')).get('verdicts',[]) if v.get('keep')))")"
log "gate: $N_KEPT kept"

# --- 5. ADOPT (only if armed, not dryrun) ----------------------------------
DISPATCHED="$RUN/dispatched.json"; echo "[]" > "$DISPATCHED"
if $ARMED && [ "$DRYRUN" != "1" ] && [ "$N_KEPT" -gt 0 ] && command -v omega >/dev/null 2>&1; then
  # kept, in-scope fingerprints joined to their proposal, capped.
  python3 - "$RUN/analysis.json" "$GATE_JSON" "$CAP" > "$RUN/kept.jsonl" <<'PY'
import json,sys
a={x["fingerprint"]:x for x in json.load(open(sys.argv[1])).get("assessments",[])}
keep=[v["fingerprint"] for v in json.load(open(sys.argv[2])).get("verdicts",[]) if v.get("keep")]
cap=int(sys.argv[3]); n=0
for fp in keep:
    x=a.get(fp)
    if not x or not x.get("in_scope"): continue
    print(json.dumps(x,ensure_ascii=False)); n+=1
    if n>=cap: break
PY
  DONE_FPS=()
  while IFS= read -r line; do
    [ -n "$line" ] || continue
    fp="$(python3 -c "import json,sys;print(json.loads(sys.argv[1])['fingerprint'])" "$line")"
    ver="$(python3 -c "import json,sys;print(json.loads(sys.argv[1])['version'])" "$line")"
    entry="$(python3 -c "import json,sys;print(json.loads(sys.argv[1])['entry'])" "$line")"
    prop="$(python3 -c "import json,sys;print(json.loads(sys.argv[1])['proposal'])" "$line")"
    surface="$(python3 -c "import json,sys;print(json.loads(sys.argv[1])['surface'])" "$line")"
    brief="Adopt Claude Code changelog entry (v$ver) into OmegaOS. ENTRY: \"$entry\". PROPOSED ADOPTION (surface: $surface): $prop. Make the surgical change on the named surface only (doctrine rules.rs / agents / skills / install.sh — NOT core Rust), keep install.sh parity (L0), run verify-install, and hand back In-Review — do NOT self-mark Done, do NOT force-push. If the proposal is wrong on inspection, report why instead of forcing it (L2)."
    log "dispatch fp=$fp v$ver surface=$surface"
    if omega dispatch "$PROJECT" "$brief" >>"$LOG" 2>&1; then
      python3 -c "import json,sys;print(json.dumps({'fingerprint':sys.argv[1],'version':sys.argv[2],'date':sys.argv[3]}))" "$fp" "$ver" "$DATE" >> "$SEEN"
      DONE_FPS+=("$fp")
    else
      log "dispatch failed fp=$fp"
    fi
  done < "$RUN/kept.jsonl"
  python3 -c "import json,sys;print(json.dumps(sys.argv[1:]))" "${DONE_FPS[@]}" > "$DISPATCHED" 2>/dev/null || echo "[]" > "$DISPATCHED"
  log "dispatched ${#DONE_FPS[@]} adoptions"
else
  log "no adopt (armed=$ARMED dryrun=$DRYRUN kept=$N_KEPT) — proposals stay report-only"
fi

# --- 6. REPORT -------------------------------------------------------------
META="$RUN/meta.json"
python3 -c "import json,sys;print(json.dumps({'date':sys.argv[1],'latest':sys.argv[2],'last_version':sys.argv[3],'armed':sys.argv[4]=='1','dryrun':sys.argv[5]=='1','new_count':int(sys.argv[6]),'dispatched':json.load(open(sys.argv[7]))}))" \
  "$DATE" "$LATEST" "${LAST:-}" "$($ARMED && echo 1 || echo 0)" "$DRYRUN" "$N_NEW" "$DISPATCHED" > "$META"
REPORT="$ART/changelog-adopt-$DATE.html"
python3 "$SCRIPTS/report.py" "$RUN/analysis.json" "$GATE_JSON" "$META" > "$REPORT" 2>>"$LOG" \
  && cp "$REPORT" "$RUN/report.html" && log "report -> $REPORT" || log "report render failed"

# --- 7. ALERT --------------------------------------------------------------
N_DISP="$(python3 -c "import json;print(len(json.load(open('$DISPATCHED'))))" 2>/dev/null || echo 0)"
TOP="$(python3 -c "import json;a=[x for x in json.load(open('$RUN/analysis.json')).get('assessments',[]) if x.get('relevance')=='high'];print(a[0]['entry'][:120] if a else 'aucune entrée haute pertinence')" 2>/dev/null)"
STATE_TXT="proposals (disarmed)"; { $ARMED && [ "$DRYRUN" != "1" ]; } && STATE_TXT="armed"
notify "📜 <b>Changelog Adopt</b> $DATE ($STATE_TXT)
Claude Code <b>$LATEST</b> · $N_NEW new · high <b>$N_HIGH</b> · gate-kept <b>$N_KEPT</b> · dispatched <b>$N_DISP</b>
Top: $TOP
Rapport: $REPORT"

# --- 8. advance watermark (last, so a mid-run crash re-processes) -----------
write_watermark
log "run done — watermark=$LATEST"

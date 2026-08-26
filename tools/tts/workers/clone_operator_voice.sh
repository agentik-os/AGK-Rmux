#!/usr/bin/env bash
# clone_operator_voice.sh — full pipeline: deposited audio → best windows →
# transcripts → OmniVoice clones → MOS+WER gate → report.
# Usage: clone_operator_voice.sh <audio-file> [audio-file ...]
set -uo pipefail
OUT=$HOME/Station/Tools/OmniVoice/bench/operator
JUDGE=$HOME/.omega/tts/venvs/judge/bin/python
PY=$HOME/Station/Tools/OmniVoice/.venv/bin/python
W=$HOME/.omega/tts/workers
mkdir -p "$OUT/refs"

TEXT="Oh, attends… tu ne vas pas y croire ! Je viens de finir l'analyse, et franchement ? C'est excellent. Bon, on fait quoi maintenant : on lance tout de suite, ou tu préfères jeter un œil d'abord ?"

# 1) Normalize + window every source, keep per-window MOS
for src in "$@"; do
  base=$(basename "$src" | tr -c 'A-Za-z0-9._-' '_')
  dur=$(ffprobe -v error -show_entries format=duration -of csv=p=0 "$src" 2>/dev/null | cut -d. -f1)
  [[ -z "$dur" || "$dur" -lt 6 ]] && dur=10
  step=8; [[ "$dur" -le 14 ]] && step=4
  for ((t=0; t<=dur-8; t+=step)); do
    ffmpeg -y -loglevel error -ss "$t" -i "$src" -t 10 -ac 1 -ar 24000 "$OUT/refs/${base}_${t}.wav"
  done
done
echo "== window MOS (top 6) =="
"$PY" "$W/judge_mos.py" "$OUT"/refs/*.wav 2>/dev/null | sort -rn | tee "$OUT/refs_mos.txt" | head -6

# 2) Clone from the 3 best windows
i=0
while read -r mos ref; do
  i=$((i+1)); [[ $i -gt 3 ]] && break
  rt=$("$JUDGE" "$W/judge_wer.py" "$ref" x fr 2>/dev/null | python3 -c "import json,sys; print(json.load(sys.stdin)['transcript'])")
  echo "ref$i ($mos): $rt"
  payload=$(python3 -c "import json,sys; print(json.dumps({'engine':'omnivoice','text':sys.argv[1],'voice':sys.argv[2],'params':{'ref_text':sys.argv[3],'num_step':32,'language':'fr'}}))" "$TEXT" "$ref" "$rt")
  code=$(curl -s -X POST localhost:8765/tts -H 'Content-Type: application/json' -d "$payload" -o "$OUT/clone$i.ogg" -w '%{http_code}')
  echo "clone$i: HTTP $code (ref: $ref)"
  cp "$ref" "$OUT/clone$i.ref.wav"; printf '%s\n' "$rt" > "$OUT/clone$i.ref.txt"
  ffmpeg -y -loglevel error -i "$OUT/clone$i.ogg" "$OUT/clone$i.wav"
done < "$OUT/refs_mos.txt"

# 3) Gate
echo "== clone MOS =="
"$PY" "$W/judge_mos.py" "$OUT"/clone*.wav 2>/dev/null | sort -rn
echo "== clone WER =="
for f in "$OUT"/clone*.wav; do [[ "$f" == *ref* ]] && continue; echo -n "$(basename "$f"): "; "$JUDGE" "$W/judge_wer.py" "$f" "$TEXT" fr 2>/dev/null | python3 -c "import json,sys; print(json.load(sys.stdin)['wer'])"; done
echo DONE

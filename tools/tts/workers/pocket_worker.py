#!/usr/bin/env python3
"""Pocket TTS (Kyutai) worker — French model, CPU real-time.

Protocol: {"text","out"} per line on stdin → {"ok"} per line on stdout;
prints {"ready": true} once the model + voice state are in memory.
"""
import json
import os
import sys

# The daemon protocol is one JSON per line on fd1 — but HF/torch libraries print
# progress bars and warnings to stdout. Keep a private dup of fd1 for the
# protocol and point everything else at stderr (captured in logs/<engine>.log).
PROTO = os.fdopen(os.dup(1), "w", buffering=1)
os.dup2(2, 1)
sys.stdout = sys.stderr
emit = lambda obj: (PROTO.write(json.dumps(obj) + "\n"), PROTO.flush())

import scipy.io.wavfile
from pocket_tts import TTSModel

LANG = "french_24l"          # undistilled 24-layer French model (May 2026 release)
VOICE = "estelle"            # the catalog's French voice

try:
    model = TTSModel.load_model(language=LANG)
except TypeError:            # older pocket-tts without the language kwarg
    model = TTSModel.load_model()
# Voice states (kvcache embeddings) are slow to compute and cheap to keep —
# cache one per voice so switching voices (the casting bench, the operator's
# pick) costs nothing after the first use. `voice` = catalog name, local wav,
# or hf:// path; default = the French catalog voice.
_states = {}


def state_for(voice: str):
    if voice not in _states:
        _states[voice] = model.get_state_for_audio_prompt(voice)
    return _states[voice]


state_for(VOICE)
emit({"ready": True})

for line in sys.stdin:
    try:
        job = json.loads(line)
        audio = model.generate_audio(state_for(job.get("voice") or VOICE), job["text"])
        scipy.io.wavfile.write(job["out"], model.sample_rate, audio.numpy())
        emit({"ok": True})
    except Exception as e:  # noqa: BLE001 — a bad job must never kill the worker
        emit({"ok": False, "error": str(e)[:300]})

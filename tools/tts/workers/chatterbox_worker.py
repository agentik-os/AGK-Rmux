#!/usr/bin/env python3
"""Chatterbox (Resemble AI) worker — multilingual French, top quality, slow on CPU.

Protocol: {"text","out"} per line on stdin → {"ok"} per line on stdout;
prints {"ready": true} once the model is in memory.
"""
import json
import os
import sys

# Protocol channel = private dup of fd1; library prints (HF progress bars…) go
# to stderr → logs/<engine>.log. See pocket_worker.py.
PROTO = os.fdopen(os.dup(1), "w", buffering=1)
os.dup2(2, 1)
sys.stdout = sys.stderr
emit = lambda obj: (PROTO.write(json.dumps(obj) + "\n"), PROTO.flush())

import torchaudio

try:
    from chatterbox.mtl_tts import ChatterboxMultilingualTTS
    model = ChatterboxMultilingualTTS.from_pretrained(device="cpu")
    MULTI = True
except Exception:  # noqa: BLE001 — fall back to the English-only model
    from chatterbox.tts import ChatterboxTTS
    model = ChatterboxTTS.from_pretrained(device="cpu")
    MULTI = False
emit({"ready": True})

for line in sys.stdin:
    try:
        job = json.loads(line)
        # voice = path to a reference wav (zero-shot cloning); absent → default voice.
        kw = {"audio_prompt_path": job["voice"]} if job.get("voice") else {}
        if MULTI:
            kw["language_id"] = "fr"
        # Tuning knobs (exaggeration ↓ = softer delivery, cfg_weight ↓ = slower,
        # more deliberate pacing, temperature ↓ = steadier) — set per voice.
        for k in ("exaggeration", "cfg_weight", "temperature"):
            if k in (job.get("params") or {}):
                kw[k] = job["params"][k]
        wav = model.generate(job["text"], **kw)
        torchaudio.save(job["out"], wav, model.sr)
        emit({"ok": True})
    except Exception as e:  # noqa: BLE001
        emit({"ok": False, "error": str(e)[:300]})

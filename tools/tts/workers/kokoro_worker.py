#!/usr/bin/env python3
"""Kokoro 82M worker — lightweight CPU TTS, French voice ff_siwis.

Protocol: {"text","out"} per line on stdin → {"ok"} per line on stdout;
prints {"ready": true} once the pipeline is in memory. Needs espeak-ng.
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

import numpy as np
import soundfile as sf
from kokoro import KPipeline

pipeline = KPipeline(lang_code="f")  # f = French
VOICE = "ff_siwis"
SAMPLE_RATE = 24000
emit({"ready": True})

for line in sys.stdin:
    try:
        job = json.loads(line)
        chunks = []
        for _, _, audio in pipeline(job["text"], voice=job.get("voice") or VOICE):
            chunks.append(audio.numpy() if hasattr(audio, "numpy") else np.asarray(audio))
        if not chunks:
            raise RuntimeError("kokoro produced no audio")
        sf.write(job["out"], np.concatenate(chunks), SAMPLE_RATE)
        emit({"ok": True})
    except Exception as e:  # noqa: BLE001
        emit({"ok": False, "error": str(e)[:300]})

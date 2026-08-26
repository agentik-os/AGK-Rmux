#!/usr/bin/env python3
"""OmniVoice worker — k2-fsa OmniVoice (1B diffusion-LM TTS, 600+ languages,
zero-shot voice cloning). Model loaded once and kept in RAM; CPU inference.

Protocol: {"text","out"} per line on stdin → {"ok"} per line on stdout;
prints {"ready": true} once the model is loaded.
Optional job keys (top-level or inside "params"): "voice" (path to a reference
wav to clone; needs "ref_text" or a sibling .txt with the transcription),
"language" (default fr), "num_step" (default 16 — quality/speed knob, 32 =
paper default), "instruct" (voice design), "speed" (float, default model-picked).

Two measured constraints: the cloning reference should be 3-10 s and its
ref_text transcribed in the REFERENCE's language (an EN ref with a FR-forced
transcript yields gibberish); "instruct" only accepts the model's closed tag
vocabulary (male/female, age bands, pitch levels, whisper, accents) as
comma-separated items — free prose is rejected with "Unsupported instruct items".
"""
import json
import os
import sys

# Protocol channel = private dup of fd1; library prints go to stderr → logs.
PROTO = os.fdopen(os.dup(1), "w", buffering=1)
os.dup2(2, 1)
sys.stdout = sys.stderr
emit = lambda obj: (PROTO.write(json.dumps(obj) + "\n"), PROTO.flush())

import torch  # noqa: E402
from omnivoice.models.omnivoice import OmniVoice  # noqa: E402
import soundfile as sf  # noqa: E402

MODEL_ID = os.environ.get("OMNIVOICE_MODEL", "k2-fsa/OmniVoice")
# bf16: native on this EPYC (avx512_bf16); fp16 on CPU is emulated and slower.
model = OmniVoice.from_pretrained(MODEL_ID, device_map="cpu", dtype=torch.bfloat16)
emit({"ready": True})

for line in sys.stdin:
    try:
        job = json.loads(line)
        p = job.get("params") or {}
        ref_audio = job.get("voice") or p.get("ref_audio") or None
        ref_text = job.get("ref_text") or p.get("ref_text") or None
        if ref_audio and not ref_text:
            txt = os.path.splitext(ref_audio)[0] + ".txt"
            if os.path.exists(txt):
                ref_text = open(txt).read().strip()
        audios = model.generate(
            text=job["text"],
            language=job.get("language") or p.get("language") or "fr",
            ref_audio=ref_audio,
            ref_text=ref_text,
            instruct=job.get("instruct") or p.get("instruct") or None,
            speed=(lambda s: float(s) if s else None)(job.get("speed") or p.get("speed")),
            num_step=int(job.get("num_step") or p.get("num_step") or 16),
        )
        sf.write(job["out"], audios[0], model.sampling_rate)
        emit({"ok": True})
    except Exception as e:  # noqa: BLE001
        emit({"ok": False, "error": str(e)})

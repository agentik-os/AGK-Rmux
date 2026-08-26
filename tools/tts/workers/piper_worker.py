#!/usr/bin/env python3
"""Piper worker — fastest CPU TTS, voice fr_FR-siwis-medium.

Protocol: {"text","out"} per line on stdin → {"ok"} per line on stdout;
prints {"ready": true} once the voice is loaded. The voice model is
downloaded at install time into $TTS_DIR/models/piper.
"""
import json
import os
import subprocess
import sys

# Protocol channel = private dup of fd1; library prints go to stderr → logs.
PROTO = os.fdopen(os.dup(1), "w", buffering=1)
os.dup2(2, 1)
sys.stdout = sys.stderr
emit = lambda obj: (PROTO.write(json.dumps(obj) + "\n"), PROTO.flush())

TTS_DIR = os.environ.get("TTS_DIR", os.path.expanduser("~/.omega/tts"))
MODEL_DIR = os.path.join(TTS_DIR, "models", "piper")
VOICE = "fr_FR-siwis-medium"

# One loaded PiperVoice per model name; fall back to the CLI (which also
# auto-downloads a missing voice) when the Python API is unavailable.
_voices: dict = {}


def load_voice(name: str):
    if name not in _voices:
        try:
            from piper import PiperVoice
            _voices[name] = PiperVoice.load(os.path.join(MODEL_DIR, f"{name}.onnx"))
        except Exception:  # noqa: BLE001
            _voices[name] = None
    return _voices[name]


load_voice(VOICE)
emit({"ready": True})

for line in sys.stdin:
    try:
        job = json.loads(line)
        name = job.get("voice") or VOICE
        voice = load_voice(name)
        if voice is not None:
            import wave
            with wave.open(job["out"], "wb") as f:
                voice.synthesize_wav(job["text"], f)
        else:
            r = subprocess.run(
                [sys.executable, "-m", "piper", "--model", name,
                 "--data-dir", MODEL_DIR, "--download-dir", MODEL_DIR,
                 "-f", job["out"], "--", job["text"]],
                capture_output=True, timeout=120,
            )
            if r.returncode != 0:
                raise RuntimeError(r.stderr.decode()[-200:])
        emit({"ok": True})
    except Exception as e:  # noqa: BLE001
        emit({"ok": False, "error": str(e)[:300]})

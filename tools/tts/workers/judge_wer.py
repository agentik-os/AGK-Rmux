#!/usr/bin/env python3
"""Transcribe a wav with faster-whisper (small, CPU) and print WER vs reference text.

Usage: judge_wer.py <wav> <reference text> [lang]
Prints JSON: {"transcript": ..., "wer": ..., "duration_s": ...}
"""
import json
import sys

from faster_whisper import WhisperModel
from jiwer import wer

wav, ref = sys.argv[1], sys.argv[2]
lang = sys.argv[3] if len(sys.argv) > 3 else "fr"

model = WhisperModel("small", device="cpu", compute_type="int8")
segments, info = model.transcribe(wav, language=lang)
text = " ".join(s.text.strip() for s in segments)

norm = lambda s: "".join(c.lower() if c.isalnum() or c.isspace() else " " for c in s).split()
score = wer(" ".join(norm(ref)), " ".join(norm(text)))
print(json.dumps({"transcript": text, "wer": round(score, 3),
                  "duration_s": round(info.duration, 2)}, ensure_ascii=False))

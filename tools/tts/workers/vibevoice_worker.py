#!/usr/bin/env python3
"""VibeVoice-Realtime-0.5B worker — Microsoft's streaming TTS, CPU-only.

ENGLISH ONLY, and that is measured, not assumed. Benchmarked 2026-08-07 on this
box by round-tripping the output through Whisper and scoring word error rate
against the source text: English 0.0%, French 53.8% (piper and kokoro both score
5.1% on the same French sentence). So this engine is for English long-form and
podcast work; French belongs to piper / kokoro / elevenlabs.

Speed: RTF ~2.3 on CPU (2 s of compute per 1 s of speech). Fine for offline
batch, too slow for a live call.

Protocol: {"text","out","voice"} per line on stdin -> {"ok"} per line on stdout;
prints {"ready": true} once the model is in memory. Mirrors kokoro_worker.py.
"""
import json
import os
import sys

# Protocol channel = private dup of fd1; library prints (HF progress bars, the
# transformers "newly initialized weights" banner) go to stderr -> logs/<engine>.log.
PROTO = os.fdopen(os.dup(1), "w", buffering=1)
os.dup2(2, 1)
sys.stdout = sys.stderr
emit = lambda obj: (PROTO.write(json.dumps(obj) + "\n"), PROTO.flush())

import glob

import soundfile as sf
import torch
from transformers.cache_utils import DynamicCache
from transformers.modeling_outputs import BaseModelOutputWithPast

from vibevoice.modular.modeling_vibevoice_streaming_inference import (
    VibeVoiceStreamingForConditionalGenerationInference,
)
from vibevoice.processor.vibevoice_streaming_processor import VibeVoiceStreamingProcessor

TTS_DIR = os.environ.get("OMEGA_TTS_DIR", os.path.expanduser("~/.omega/tts"))
REPO = os.path.join(TTS_DIR, "vibevoice")
MODEL = os.environ.get("VIBEVOICE_MODEL", "microsoft/VibeVoice-Realtime-0.5B")
# en-Carter_man measured the most varied intonation of the 25 shipped presets
# (81 Hz F0 std vs 24-45 Hz for most), i.e. the least robotic default.
DEFAULT_VOICE = os.environ.get("VIBEVOICE_VOICE", "en-Carter_man")
SAMPLE_RATE = 24000
CFG_SCALE = 1.5          # swept 1.3-3.0: prosody barely moves, keep the default
DDPM_STEPS = 5

voices = {
    os.path.splitext(os.path.basename(p))[0].lower(): p
    for p in glob.glob(os.path.join(REPO, "demo/voices/streaming_model/**/*.pt"), recursive=True)
}
if not voices:
    emit({"ready": False, "error": f"no voice presets under {REPO}/demo/voices/streaming_model"})
    sys.exit(1)

processor = VibeVoiceStreamingProcessor.from_pretrained(MODEL)
model = VibeVoiceStreamingForConditionalGenerationInference.from_pretrained(
    MODEL, torch_dtype=torch.float32, device_map="cpu", attn_implementation="sdpa",
)
model.eval()
model.set_ddpm_inference_steps(num_steps=DDPM_STEPS)


def load_prompt(path):
    """torch >= 2.6 refuses these presets even under safe_globals: the restricted
    unpickler rejects SETITEMS on BaseModelOutputWithPast, which is not a dict
    subclass. Audited every shipped preset with pickletools -- they reference only
    OrderedDict, torch BFloat16Storage, torch._utils._rebuild_tensor_v2, DynamicCache
    and BaseModelOutputWithPast -- so loading them unrestricted is safe."""
    with torch.serialization.safe_globals([BaseModelOutputWithPast, DynamicCache]):
        return torch.load(path, map_location="cpu", weights_only=False)


def resolve(name):
    n = (name or DEFAULT_VOICE).lower()
    if n in voices:
        return voices[n]
    hits = [p for k, p in voices.items() if n in k]
    return hits[0] if hits else voices[DEFAULT_VOICE.lower()]


emit({"ready": True})

for line in sys.stdin:
    try:
        job = json.loads(line)
        text = (job.get("text") or "").replace("’", "'").replace("“", '"').replace("”", '"')
        if not text.strip():
            raise RuntimeError("empty text")
        prompt = load_prompt(resolve(job.get("voice")))
        inputs = processor.process_input_with_cached_prompt(
            text=text, cached_prompt=prompt, padding=True,
            return_tensors="pt", return_attention_mask=True,
        )
        out = model.generate(
            **inputs, max_new_tokens=None, cfg_scale=CFG_SCALE,
            tokenizer=processor.tokenizer, generation_config={"do_sample": False},
            verbose=False, all_prefilled_outputs=load_prompt(resolve(job.get("voice"))),
        )
        speech = out.speech_outputs[0] if out.speech_outputs else None
        if speech is None:
            raise RuntimeError("vibevoice produced no audio")
        sf.write(job["out"], speech.detach().float().cpu().numpy().reshape(-1), SAMPLE_RATE)
        emit({"ok": True})
    except Exception as e:  # noqa: BLE001
        emit({"ok": False, "error": str(e)[:300]})

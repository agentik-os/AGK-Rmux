#!/usr/bin/env python3
"""omega-ttsd — OmegaOS TTS gateway daemon (stdlib only, no pip deps).

One local HTTP endpoint in front of every voice engine installed on the box,
so the Telegram bot (and any omega tool) can synthesize speech with a single
POST whatever the engine. Local engines run as persistent worker subprocesses
(one per engine, lazy-spawned, model kept in RAM between requests); ElevenLabs
is proxied straight to their API. Output is always Telegram-ready OGG/Opus.

  GET  /health            → {"ok": true}
  GET  /engines           → [{id, label, kind, available, loaded, note}]
  POST /tts {engine,text} → audio/ogg bytes (200) or {"error": ...} (4xx/5xx)

Workers speak one JSON object per line on stdin/stdout:
  in:  {"text": "...", "out": "/abs/path.wav"}
  out: {"ok": true} | {"ok": false, "error": "..."}
and print {"ready": true} once their model is loaded.
"""
import json
import os
import re
import subprocess
import threading
import time
import urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

OMEGA_DIR = os.environ.get("OMEGA_DIR", os.path.expanduser("~/.omega"))
TTS_DIR = os.path.join(OMEGA_DIR, "tts")
VENVS = os.path.join(TTS_DIR, "venvs")
OUT_DIR = os.path.join(TTS_DIR, "out")
LOG_DIR = os.path.join(TTS_DIR, "logs")
PORT = int(os.environ.get("OMEGA_TTSD_PORT", "8765"))
MAX_TEXT = 4000          # hard cap — beyond this a voice note stops being a note
JOB_TIMEOUT = 420        # chatterbox on CPU is slow; the bot fire-and-forgets anyway
READY_TIMEOUT = 900      # first spawn may download a multi-GB model

# Engine catalog. `venv` + `worker` define a local persistent worker; kind "api"
# engines are handled inline. Order = display order in the Telegram menu.
ENGINES = {
    "pocket": {
        "label": "Pocket TTS (Kyutai)", "kind": "local",
        "venv": f"{VENVS}/pocket", "worker": "pocket_worker.py",
        "note": "temps réel CPU, voix fr native",
    },
    "chatterbox": {
        "label": "Chatterbox (Resemble)", "kind": "local",
        "venv": f"{VENVS}/chatterbox", "worker": "chatterbox_worker.py",
        "note": "qualité max, lent sur CPU (async ok)",
    },
    "kokoro": {
        "label": "Kokoro 82M", "kind": "local",
        "venv": f"{VENVS}/kokoro", "worker": "kokoro_worker.py",
        "note": "léger et rapide, fr correct",
    },
    "piper": {
        "label": "Piper (fr_FR-siwis)", "kind": "local",
        "venv": f"{VENVS}/piper", "worker": "piper_worker.py",
        "note": "ultra-rapide, voix plus robotique",
    },
    "vibevoice": {
        "label": "VibeVoice 0.5B (EN)", "kind": "local",
        "venv": f"{VENVS}/vibevoice", "worker": "vibevoice_worker.py",
        "note": "ANGLAIS uniquement — long-form/podcast, ~2 s de calcul par s",
    },
    "omnivoice": {
        "label": "OmniVoice 1B (k2-fsa)", "kind": "local",
        "venv": f"{VENVS}/omnivoice", "worker": "omnivoice_worker.py",
        "note": "600+ langues, clonage de voix, top qualité — ~2 s de calcul par s (async ok)",
    },
    "elevenlabs": {
        "label": "ElevenLabs (API)", "kind": "api",
        "note": "qualité référence — nécessite ELEVENLABS_API_KEY",
    },
}

# Operator preference: ~/.omega/tts/config.json {"disabled": ["pocket", …]}
# hides engines from /engines and rejects them on /tts — the catalog stays
# shipped/generic, the machine decides what it serves.
def disabled_engines() -> set:
    try:
        with open(os.path.join(TTS_DIR, "config.json")) as f:
            return set(json.load(f).get("disabled") or [])
    except OSError:
        return set()


_workers: dict = {}
_locks = {eid: threading.Lock() for eid in ENGINES}


def log(msg: str):
    print(f"[ttsd] {time.strftime('%H:%M:%S')} {msg}", flush=True)


def read_services_env(var: str) -> str:
    """Read a key from provisioning/services.env (export VAR="..." lines)."""
    path = os.path.join(OMEGA_DIR, "provisioning", "services.env")
    try:
        with open(path) as f:
            for line in f:
                m = re.match(rf'^\s*(?:export\s+)?{var}\s*=\s*"?([^"\n]*)"?\s*$', line)
                if m:
                    return m.group(1).strip()
    except OSError:
        pass
    return ""


def elevenlabs_config() -> dict:
    cfg = {"voice_id": "EXAVITQu4vr4xnSDxMaL", "model_id": "eleven_multilingual_v2"}
    try:
        with open(os.path.join(TTS_DIR, "elevenlabs.json")) as f:
            cfg.update(json.load(f))
    except OSError:
        pass
    return cfg


def engine_available(eid: str) -> bool:
    e = ENGINES[eid]
    if e["kind"] == "api":
        return bool(read_services_env("ELEVENLABS_API_KEY"))
    return os.path.exists(os.path.join(e["venv"], "bin", "python"))


class Worker:
    """A persistent engine subprocess; model loaded once, jobs serialized."""

    def __init__(self, eid: str):
        e = ENGINES[eid]
        self.eid = eid
        os.makedirs(LOG_DIR, exist_ok=True)
        self.errlog = open(os.path.join(LOG_DIR, f"{eid}.log"), "ab", buffering=0)
        self.proc = subprocess.Popen(
            [os.path.join(e["venv"], "bin", "python"),
             os.path.join(TTS_DIR, "workers", e["worker"])],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=self.errlog,
            text=True, bufsize=1, env={**os.environ, "OMEGA_DIR": OMEGA_DIR, "TTS_DIR": TTS_DIR},
        )
        log(f"{eid}: worker spawned (pid {self.proc.pid}), loading model…")
        line = self._readline(READY_TIMEOUT)
        if not (line and json.loads(line).get("ready")):
            raise RuntimeError(f"{eid}: worker failed to become ready: {line!r}")
        log(f"{eid}: ready")

    def _readline(self, timeout: float) -> str:
        out: list = []
        t = threading.Thread(target=lambda: out.append(self.proc.stdout.readline()), daemon=True)
        t.start()
        t.join(timeout)
        return out[0].strip() if out else ""

    def run(self, text: str, out_path: str, voice: str = "", params: dict | None = None) -> dict:
        if self.proc.poll() is not None:
            raise RuntimeError("worker died")
        self.proc.stdin.write(json.dumps({"text": text, "out": out_path, "voice": voice, "params": params or {}}) + "\n")
        self.proc.stdin.flush()
        line = self._readline(JOB_TIMEOUT)
        if not line:
            self.proc.kill()
            raise RuntimeError("job timed out — worker killed")
        return json.loads(line)


def synth_local(eid: str, text: str, voice: str = "", params: dict | None = None) -> bytes:
    with _locks[eid]:
        w = _workers.get(eid)
        if w is None or w.proc.poll() is not None:
            w = _workers[eid] = Worker(eid)
        os.makedirs(OUT_DIR, exist_ok=True)
        wav = os.path.join(OUT_DIR, f"{eid}-{int(time.time() * 1000)}.wav")
        try:
            r = w.run(text, wav, voice, params)
            if not r.get("ok"):
                raise RuntimeError(r.get("error", "unknown worker error"))
            return to_ogg(wav)
        finally:
            try:
                os.remove(wav)
            except OSError:
                pass


def synth_elevenlabs(text: str, voice: str = "") -> bytes:
    key = read_services_env("ELEVENLABS_API_KEY")
    if not key:
        raise RuntimeError("ELEVENLABS_API_KEY manquante dans provisioning/services.env")
    cfg = elevenlabs_config()
    if voice:
        cfg["voice_id"] = voice
    # Pronunciation fixes for the voice (the name is written "Agentik" but spoken
    # "Ajentik" — soft g). Applied to the TTS text only, never the written brand.
    import re as _re
    text = _re.sub(r"\bAgentik\b", "Ajentik", text)
    body = {"text": text, "model_id": cfg["model_id"]}
    # Optional voice_settings passthrough from elevenlabs.json — e.g.
    # {"speed": 1.05, "stability": 0.5} to tune pace/character per machine.
    if isinstance(cfg.get("voice_settings"), dict):
        body["voice_settings"] = cfg["voice_settings"]
    req = urllib.request.Request(
        f"https://api.elevenlabs.io/v1/text-to-speech/{cfg['voice_id']}?output_format=mp3_44100_128",
        data=json.dumps(body).encode(),
        headers={"xi-api-key": key, "content-type": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=120) as r:
        mp3 = r.read()
    os.makedirs(OUT_DIR, exist_ok=True)
    src = os.path.join(OUT_DIR, f"el-{int(time.time() * 1000)}.mp3")
    with open(src, "wb") as f:
        f.write(mp3)
    try:
        return to_ogg(src)
    finally:
        try:
            os.remove(src)
        except OSError:
            pass


def to_ogg(src: str) -> bytes:
    """Telegram voice notes want OGG/Opus mono — convert whatever the engine made.

    A gentle mastering chain rides along: rumble cut, a touch of low-mid warmth,
    tamed harsh highs, and broadcast loudness — every engine sounds noticeably
    cleaner and softer through it.
    """
    ogg = src.rsplit(".", 1)[0] + ".ogg"
    fx = ("highpass=f=70,"
          "equalizer=f=220:t=q:w=1.2:g=1.5,"
          "equalizer=f=6500:t=q:w=1.5:g=-2,"
          "loudnorm=I=-19:TP=-2:LRA=9")
    r = subprocess.run(
        ["ffmpeg", "-y", "-i", src, "-af", fx, "-ac", "1", "-ar", "48000",
         "-c:a", "libopus", "-b:a", "48k", ogg],
        capture_output=True, timeout=120,
    )
    if r.returncode != 0:
        raise RuntimeError(f"ffmpeg: {r.stderr.decode()[-200:]}")
    try:
        with open(ogg, "rb") as f:
            return f.read()
    finally:
        try:
            os.remove(ogg)
        except OSError:
            pass


class Handler(BaseHTTPRequestHandler):
    def log_message(self, *a):  # quiet — we log ourselves
        pass

    def _json(self, code: int, obj):
        body = json.dumps(obj).encode()
        self.send_response(code)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        if self.path == "/health":
            return self._json(200, {"ok": True})
        if self.path == "/engines":
            off = disabled_engines()
            return self._json(200, [
                {"id": eid, "label": e["label"], "kind": e["kind"], "note": e["note"],
                 "available": engine_available(eid),
                 "loaded": eid in _workers and _workers[eid].proc.poll() is None}
                for eid, e in ENGINES.items() if eid not in off
            ])
        return self._json(404, {"error": "not found"})

    def do_POST(self):
        if self.path != "/tts":
            return self._json(404, {"error": "not found"})
        try:
            body = json.loads(self.rfile.read(int(self.headers.get("content-length", 0))))
            eid = body.get("engine", "")
            text = (body.get("text") or "").strip()[:MAX_TEXT]
            voice = str(body.get("voice") or "")[:300]
            params = body.get("params") if isinstance(body.get("params"), dict) else None
            if eid not in ENGINES or eid in disabled_engines():
                return self._json(400, {"error": f"unknown engine {eid!r}"})
            if not text:
                return self._json(400, {"error": "empty text"})
            if not engine_available(eid):
                return self._json(409, {"error": f"{eid} indisponible (non installé ou clé manquante)"})
            t0 = time.time()
            ogg = synth_elevenlabs(text, voice) if eid == "elevenlabs" else synth_local(eid, text, voice, params)
            log(f"{eid}: {len(text)} chars → {len(ogg)} bytes in {time.time() - t0:.1f}s")
            self.send_response(200)
            self.send_header("content-type", "audio/ogg")
            self.send_header("content-length", str(len(ogg)))
            self.end_headers()
            self.wfile.write(ogg)
        except Exception as e:  # noqa: BLE001 — any engine failure must reach the caller
            log(f"ERROR {e}")
            self._json(500, {"error": str(e)[:300]})


if __name__ == "__main__":
    os.makedirs(OUT_DIR, exist_ok=True)
    log(f"listening on 127.0.0.1:{PORT} — engines: {', '.join(ENGINES)}")
    ThreadingHTTPServer(("127.0.0.1", PORT), Handler).serve_forever()

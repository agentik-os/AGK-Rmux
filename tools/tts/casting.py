#!/usr/bin/env python3
"""Voice casting bench — synthesize numbered samples and send them to Telegram.

Reads a voices manifest (casting-voices.json), synthesizes one numbered sample
per entry through the omega-ttsd gateway, sends each as a numbered voice note
via an agent bot, and writes the resolved manifest to
~/.omega/tts/casting-manifest.json — the Telegram bot uses it to honor
«voix N» (the operator picks a voice by number).

Usage: python3 casting.py [--manifest casting-voices.json] [--bot lifestyle]
                          [--start N] [--only engine]
Stdlib only. Reference prompts (voice cloning) are downloaded once into
~/.omega/tts/voices/ and trimmed to 30s (embedding a 10-min wav is slow).
"""
import argparse
import json
import os
import subprocess
import sys
import time
import urllib.request

OMEGA_DIR = os.environ.get("OMEGA_DIR", os.path.expanduser("~/.omega"))
TTS_DIR = os.path.join(OMEGA_DIR, "tts")
VOICES_DIR = os.path.join(TTS_DIR, "voices")
TTSD = f"http://127.0.0.1:{os.environ.get('OMEGA_TTSD_PORT', '8765')}"
HF = "https://huggingface.co/kyutai/tts-voices/resolve/main"

TEXT = ("Numéro {n}. Hey toi… c'est Nova. Alors, elle te plaît, cette voix-là ? "
        "Dis-moi mon numéro, et je la garde rien que pour toi.")


def log(msg):
    print(msg, flush=True)


def fetch_prompt(rel_path: str) -> str:
    """Download a reference wav from kyutai/tts-voices once, trimmed to 30s."""
    os.makedirs(VOICES_DIR, exist_ok=True)
    name = rel_path.replace("/", "__")
    dest = os.path.join(VOICES_DIR, f"{name}.30s.wav")
    if os.path.exists(dest):
        return dest
    raw = os.path.join(VOICES_DIR, name)
    urllib.request.urlretrieve(f"{HF}/{rel_path}", raw)
    r = subprocess.run(["ffmpeg", "-y", "-i", raw, "-t", "30", "-ac", "1", dest],
                       capture_output=True, timeout=120)
    os.remove(raw)
    if r.returncode != 0:
        raise RuntimeError(f"ffmpeg trim failed: {r.stderr.decode()[-150:]}")
    return dest


def synth(engine: str, voice: str, text: str, params=None, timeout=600) -> bytes:
    req = urllib.request.Request(
        f"{TTSD}/tts",
        data=json.dumps({"engine": engine, "text": text, "voice": voice, "params": params or {}}).encode(),
        headers={"content-type": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return r.read()


def send_voice(token: str, chat: int, ogg: bytes, caption: str) -> bool:
    # stdlib multipart — keep this tool dependency-free like the daemon.
    boundary = f"----omegacasting{int(time.time() * 1000)}"
    parts = b""
    for k, v in (("chat_id", str(chat)), ("caption", caption)):
        parts += (f"--{boundary}\r\nContent-Disposition: form-data; name=\"{k}\"\r\n\r\n{v}\r\n").encode()
    parts += (f"--{boundary}\r\nContent-Disposition: form-data; name=\"voice\"; "
              f"filename=\"nova.ogg\"\r\nContent-Type: audio/ogg\r\n\r\n").encode()
    parts += ogg + f"\r\n--{boundary}--\r\n".encode()
    req = urllib.request.Request(
        f"https://api.telegram.org/bot{token}/sendVoice", data=parts,
        headers={"content-type": f"multipart/form-data; boundary={boundary}"},
    )
    with urllib.request.urlopen(req, timeout=60) as r:
        return json.load(r).get("ok", False)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--manifest", default=os.path.join(os.path.dirname(__file__), "casting-voices.json"))
    ap.add_argument("--bot", default="lifestyle")
    ap.add_argument("--start", type=int, default=1, help="resume from this number")
    ap.add_argument("--only", default="", help="restrict to one engine")
    ap.add_argument("--text", default=TEXT, help="sample text ({n} = voice number)")
    args = ap.parse_args()

    bots = json.load(open(os.path.join(OMEGA_DIR, "agent-bots.json")))
    token, chat = bots[args.bot]["token"], bots[args.bot]["allow"][0]
    entries = json.load(open(args.manifest))

    resolved, sent, failed = [], 0, 0
    for e in entries:
        n, engine, label = e["n"], e["engine"], e["label"]
        voice = e.get("voice", "")
        if e.get("prompt"):  # cloning entry → local trimmed reference wav
            try:
                voice = fetch_prompt(e["prompt"])
            except Exception as ex:  # noqa: BLE001
                log(f"N°{n} [{engine}] PROMPT FAIL: {ex}")
                failed += 1
                continue
        resolved.append({"n": n, "engine": engine, "voice": voice, "label": label, **({"params": e["params"]} if e.get("params") else {})})
        if n < args.start or (args.only and engine != args.only):
            continue
        t0 = time.time()
        try:
            ogg = synth(engine, voice, args.text.format(n=n), e.get("params"))
            ok = send_voice(token, chat, ogg, f"N°{n} — {label} ({engine})")
            log(f"N°{n} [{engine}] {'sent' if ok else 'SEND FAIL'} in {time.time() - t0:.0f}s — {label}")
            sent += ok
            failed += not ok
            time.sleep(2.5)  # Telegram per-chat rate limit
        except Exception as ex:  # noqa: BLE001
            log(f"N°{n} [{engine}] FAIL after {time.time() - t0:.0f}s: {str(ex)[:200]}")
            failed += 1

    # The bot reads this to honor «voix N» — write it even on partial failure.
    # MERGE by number so a patch run (a few re-cast numbers) never wipes the rest.
    out = os.path.join(TTS_DIR, "casting-manifest.json")
    merged = {}
    try:
        merged = {x["n"]: x for x in json.load(open(out))}
    except Exception:  # noqa: BLE001 — first run
        pass
    merged.update({x["n"]: x for x in resolved})
    json.dump(sorted(merged.values(), key=lambda x: x["n"]), open(out, "w"), ensure_ascii=False, indent=1)
    log(f"done: {sent} sent, {failed} failed → manifest {out}")
    sys.exit(1 if failed and not sent else 0)


if __name__ == "__main__":
    main()

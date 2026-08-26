#!/usr/bin/env python3
"""ElevenLabs voice casting — build a numbered bench of French female voices.

Queries the ElevenLabs voice library for native-French female voices (soft /
warm / seductive descriptors first), adds the best shared voices to the
account library (slot-limited on free tier — failures are skipped), completes
with the account's premade female voices speaking French, then hands the
numbered manifest to casting.py which synthesizes + sends the Telegram notes.

Usage: python3 elevenlabs-casting.py [--start 61] [--max 8]
Stdlib only; key read from ~/.omega/provisioning/services.env.
"""
import argparse
import json
import os
import re
import subprocess
import sys
import urllib.parse
import urllib.request

OMEGA_DIR = os.environ.get("OMEGA_DIR", os.path.expanduser("~/.omega"))
API = "https://api.elevenlabs.io/v1"
WANT = re.compile(r"seductive|sensual|sexy|soft|warm|gentle|intimate|calm|douce|chaleureuse|sensuelle", re.I)


def key() -> str:
    try:
        for line in open(os.path.join(OMEGA_DIR, "provisioning", "services.env")):
            m = re.match(r'^\s*(?:export\s+)?ELEVENLABS_API_KEY\s*=\s*"?([^"\n]+)"?\s*$', line)
            if m:
                return m.group(1).strip()
    except OSError:
        pass
    return ""


def req(path: str, method="GET", body=None) -> dict:
    r = urllib.request.Request(f"{API}{path}", method=method,
                               data=json.dumps(body).encode() if body else None,
                               headers={"xi-api-key": key(), "content-type": "application/json"})
    with urllib.request.urlopen(r, timeout=60) as resp:
        return json.load(resp)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--start", type=int, default=61)
    ap.add_argument("--max", type=int, default=8)
    args = ap.parse_args()
    if not key():
        sys.exit("no ELEVENLABS_API_KEY in provisioning/services.env")

    entries, n = [], args.start

    # 1) Native-French female voices from the public library, best-loved first.
    q = urllib.parse.urlencode({"language": "fr", "gender": "female", "page_size": 100})
    shared = (req(f"/shared-voices?{q}").get("voices") or [])
    shared.sort(key=lambda v: (bool(WANT.search(f"{v.get('descriptive','')} {v.get('use_case','')} {v.get('description','')}")),
                               v.get("cloned_by_count", 0)), reverse=True)
    for v in shared:
        if len(entries) >= max(3, args.max - 4):
            break
        try:
            req(f"/voices/add/{v['public_owner_id']}/{v['voice_id']}", "POST", {"new_name": f"nova-{v['name']}"[:40]})
        except Exception as e:  # noqa: BLE001 — slot full / already added → still try to use it
            print(f"add {v['name']}: {str(e)[:80]}")
        entries.append({"n": n, "engine": "elevenlabs", "voice": v["voice_id"],
                        "label": f"EL {v['name']} — {str(v.get('descriptive') or v.get('use_case') or 'fr')[:30]}"})
        n += 1

    # 2) Premade female voices already in the account (multilingual, no slot cost).
    mine = (req("/voices").get("voices") or [])
    for v in mine:
        if len(entries) >= args.max:
            break
        lbl = v.get("labels") or {}
        if str(lbl.get("gender", "")).lower() == "female" and v.get("category") == "premade":
            entries.append({"n": n, "engine": "elevenlabs", "voice": v["voice_id"],
                            "label": f"EL {v['name']} — {lbl.get('descriptive', lbl.get('accent', 'premade'))}"})
            n += 1

    if not entries:
        sys.exit("no voices found (library query empty?)")
    manifest = "/tmp/casting-elevenlabs.json"
    json.dump(entries, open(manifest, "w"), ensure_ascii=False, indent=1)
    print(f"{len(entries)} voices → casting from n°{args.start}")
    sys.exit(subprocess.run([sys.executable, os.path.join(os.path.dirname(os.path.abspath(__file__)), "casting.py"),
                             "--manifest", manifest, "--text",
                             "Numéro {n}… Coucou toi. Alors… c'est ma voix que tu attendais ? Approche… écoute comme je suis douce. Si tu me veux, murmure mon numéro."]).returncode)


if __name__ == "__main__":
    main()

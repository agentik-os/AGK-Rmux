#!/usr/bin/env python3
"""OMEGA-CRON-NOVA-CALL-KB — feed the live-call agent the operator's real context.

The ConvAI call agent can't read the machine, so we bring the mountain to it:
Opus digests the life store (About/) + the projects map into one compact
document (no secrets), uploaded to the ElevenLabs knowledge base and attached
to the agent (usage_mode "prompt": injected whole, no RAG latency). Runs daily
so calls never go stale; the previous doc is deleted after swap.

Per-operator config (outside the repo): ~/.omega/nova-secrets.env supplies
NOVA_CHAT_ID/NOVA_HOME (life store, default ~/Station/LifeStyle) and optional
NOVA_STATION (projects root, default ~/Station); the call agent id comes from
~/.omega/state/nova-call.json. Absent pieces → silent exit (fresh installs).

State: ~/.omega/state/nova-call-kb.json {doc_id, updated}
Log:   ~/.omega/logs/nova-call-kb.log
"""
import json
import os
import re
import shutil
import subprocess
import time
import urllib.request

OMEGA = os.environ.get("OMEGA_DIR", os.path.expanduser("~/.omega"))
STATE = f"{OMEGA}/state/nova-call-kb.json"
CALL_FILE = f"{OMEGA}/state/nova-call.json"
LOG = f"{OMEGA}/logs/nova-call-kb.log"
LOCK = "/tmp/nova-call-kb.lock"
CLAUDE = os.environ.get("NOVA_CLAUDE_BIN", os.path.expanduser("~/.local/bin/claude"))
if not os.path.exists(CLAUDE):
    CLAUDE = shutil.which("claude") or CLAUDE


def env_file(path: str) -> dict:
    out = {}
    try:
        for line in open(path):
            m = re.match(r'^\s*(?:export\s+)?([A-Z_][A-Z0-9_]*)\s*=\s*"?([^"\n]*)"?\s*$', line)
            if m:
                out[m.group(1)] = m.group(2).strip()
    except OSError:
        pass
    return out


SECRETS = env_file(f"{OMEGA}/nova-secrets.env")
LIFE = os.path.expanduser(SECRETS.get("NOVA_HOME") or "~/Station/LifeStyle")
STATION = os.path.expanduser(SECRETS.get("NOVA_STATION") or "~/Station")

DIGEST_PROMPT = f"""Construis le DOSSIER DE CONTEXTE de Nova pour ses appels vocaux avec l'opérateur.
Sortie : UNIQUEMENT le document markdown final (aucun préambule), en français, 12000-18000 caractères max.

Sources à lire :
1. {LIFE}/About/ — la base de connaissance de l'opérateur (identité, objectifs, santé, mind,
   skills, relations, système ; ignore _archive).
2. {STATION}/ — le panorama de ses projets et clients : lis les README/fiches disponibles, n'invente rien.

Structure attendue :
# Qui est l'opérateur — identité, choix de vie, piliers de sa vision
# Objectifs actuels — ses objectifs vivants
# Santé / Mind / Skills — l'essentiel actionnable
# Relations — l'essentiel
# Business & projets — par projet : 2-4 lignes (quoi, état, enjeu). Sépare CLIENTS et PROJETS PERSO
# OmegaOS & la machine — son OS d'agents (lis le README du repo OmegaOS s'il est présent) : ce que
  sont les oracles, workers, workflows dynamiques, les skills (dont la suite marketing /omg-*),
  Atlas vs Nova, la voix. Assez précis pour qu'en appel Nova comprenne « lance la machine marketing »
  ou une question sur un workflow — ET sache dire que les actions demandées en appel seront exécutées
  juste après par la vraie Nova (qui lit le transcript et peut dispatcher des missions).

Règles : factuel et dense (pas de blabla), AUCUN secret/token/clé/credential, pas de données
bancaires, pas d'adresse postale précise. Ce document sera lu par la version "appel" de Nova."""


def log(msg: str) -> None:
    os.makedirs(os.path.dirname(LOG), exist_ok=True)
    with open(LOG, "a") as f:
        f.write(f"{time.strftime('%F %T')} {msg}\n")


def el_key() -> str:
    for path in (f"{OMEGA}/provisioning/services.env", f"{OMEGA}/secrets/integrations.env"):
        v = env_file(path).get("ELEVENLABS_API_KEY")
        if v:
            return v
    return ""


def agent_id() -> str:
    try:
        return json.load(open(CALL_FILE)).get("agent_id") or ""
    except Exception:
        return ""


def api(path: str, method: str = "GET", body: dict | None = None) -> dict:
    req = urllib.request.Request(
        f"https://api.elevenlabs.io/v1{path}", method=method,
        data=json.dumps(body).encode() if body is not None else None,
        headers={"xi-api-key": el_key(), "content-type": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=60) as r:
        raw = r.read()
        return json.loads(raw) if raw.strip() else {}


def main() -> None:
    aid = agent_id()
    t0 = time.time()
    digest = subprocess.run(
        [CLAUDE, "-p", DIGEST_PROMPT, "--model", "claude-opus-5",
         "--dangerously-skip-permissions", "--max-turns", "40"],
        cwd=STATION, capture_output=True, text=True, timeout=1800,
    ).stdout.strip()
    if len(digest) < 2000:
        log(f"digest too short ({len(digest)} chars) — keeping the previous KB doc")
        return
    log(f"digest built: {len(digest)} chars in {time.time() - t0:.0f}s")

    doc = api("/convai/knowledge-base/text", "POST",
              {"name": f"onelife-{time.strftime('%Y%m%d')}", "text": digest})
    doc_id = doc.get("id")
    if not doc_id:
        log(f"KB upload failed: {doc}")
        return

    api(f"/convai/agents/{aid}", "PATCH", {
        "conversation_config": {"agent": {"prompt": {"knowledge_base": [
            {"type": "text", "id": doc_id, "name": "onelife", "usage_mode": "prompt"}
        ]}}},
    })
    log(f"agent patched with KB doc {doc_id}")

    old = {}
    try:
        old = json.load(open(STATE))
    except Exception:
        pass
    if old.get("doc_id") and old["doc_id"] != doc_id:
        try:
            api(f"/convai/knowledge-base/{old['doc_id']}", "DELETE")
            log(f"old doc {old['doc_id']} deleted")
        except Exception as e:  # noqa: BLE001
            log(f"old doc delete failed: {e}")
    json.dump({"doc_id": doc_id, "updated": time.strftime("%F %T")}, open(STATE, "w"))


if __name__ == "__main__":
    import fcntl
    lk = open(LOCK, "w")
    try:
        fcntl.flock(lk, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except OSError:
        raise SystemExit(0)
    if not el_key() or not agent_id():
        raise SystemExit(0)
    main()

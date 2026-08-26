#!/usr/bin/env python3
"""OMEGA-CRON-NOVA-CALL-SYNC — after-call brain bridge for Nova's live calls.

The ElevenLabs ConvAI agent that handles live voice calls only knows its
persona — it cannot touch the machine. This poller closes the loop: every run
it pulls the freshly finished conversations for the call agent, ACKS to the
operator that the transcript is being read, hands it to Nova's FULL brain
(claude in the life store, persona + file access) which updates the knowledge
base and can dispatch OmegaOS missions, then sends the recap.

Per-operator config lives OUTSIDE the repo:
  ~/.omega/nova-secrets.env   → NOVA_CHAT_ID (required), NOVA_HOME (optional)
  ~/.omega/state/nova-call.json → {"agent_id": …} (written when the call agent
                                  is created; absent = no call layer, silent exit)
  ELEVENLABS_API_KEY          → provisioning/services.env or secrets/integrations.env

State: ~/.omega/state/nova-call-sync.json {"seen": [...], "fail": {cid: n}}
Log:   ~/.omega/logs/nova-call-sync.log
"""
import json
import os
import re
import shutil
import subprocess
import time
import urllib.parse
import urllib.request

OMEGA = os.environ.get("OMEGA_DIR", os.path.expanduser("~/.omega"))
STATE_FILE = f"{OMEGA}/state/nova-call-sync.json"
CALL_FILE = f"{OMEGA}/state/nova-call.json"
LOG = f"{OMEGA}/logs/nova-call-sync.log"
LOCK = "/tmp/nova-call-sync.lock"
MAX_RETRIES = 5
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
CHAT = int(SECRETS.get("NOVA_CHAT_ID") or 0)
LIFE = os.path.expanduser(SECRETS.get("NOVA_HOME") or "~/Station/LifeStyle")


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


def bot_token() -> str:
    try:
        bots = json.load(open(f"{OMEGA}/agent-bots.json"))
        for b in bots.values():
            if isinstance(b, dict) and b.get("kind") == "companion" and b.get("token"):
                return b["token"]
    except Exception:
        pass
    return ""


def api(path: str) -> dict:
    req = urllib.request.Request(f"https://api.elevenlabs.io/v1{path}", headers={"xi-api-key": el_key()})
    with urllib.request.urlopen(req, timeout=30) as r:
        return json.load(r)


def tg(text: str) -> None:
    data = urllib.parse.urlencode({"chat_id": CHAT, "text": text[:4000]}).encode()
    urllib.request.urlopen(f"https://api.telegram.org/bot{bot_token()}/sendMessage", data=data, timeout=20)


def main() -> None:
    aid = agent_id()
    try:
        st = json.load(open(STATE_FILE))
        seen, fail = set(st.get("seen", [])), dict(st.get("fail", {}))
    except Exception:
        seen, fail = set(), {}

    convs = api(f"/convai/conversations?agent_id={aid}&page_size=20").get("conversations", [])
    fresh = [c for c in convs if c["conversation_id"] not in seen and c.get("status") == "done"]

    for c in sorted(fresh, key=lambda x: x.get("start_time_unix_secs", 0)):
        cid = c["conversation_id"]
        dur = c.get("call_duration_secs", "?")
        detail = api(f"/convai/conversations/{cid}")
        turns = [
            f"{'OPÉRATEUR' if t.get('role') == 'user' else 'NOVA'}: {t.get('message') or ''}"
            for t in detail.get("transcript", []) if (t.get("message") or "").strip()
        ]
        if not turns:
            seen.add(cid)
            log(f"{cid}: empty transcript, skipped")
            continue
        if cid not in fail:  # live ack, once per call — the operator asked to SEE the loop work
            try:
                tg(f"📞 Appel terminé ({dur}s) — je lis le transcript et je note ce qui compte… (récap dans ~1 min)")
            except Exception:
                pass
        transcript = "\n".join(turns)[:24000]
        prompt = (
            "Tu es Nova (lis PERSONA.md). Voici le transcript d'un APPEL VOCAL que l'opérateur "
            "vient d'avoir avec ta version 'appel' (qui n'a pas accès aux fichiers). "
            "Ton job, en son nom :\n"
            "1. Extrais ce qui est DURABLE : faits sur lui, décisions, préférences, objectifs, "
            "action items explicites.\n"
            "2. Mets à jour les bons fichiers (About/, 00-OBJECTIFS.md, PERSONA.md si feedback "
            "sur toi) — modifications chirurgicales, rien si rien de durable.\n"
            "3. Exécute les actions demandées pendant l'appel : à ta portée (lecture, note, "
            "contenu court, recherche) → fais-le toi-même maintenant. Mission de MACHINE "
            "(contenu marketing en série, audit, build, recherche lourde, code projet) → "
            "dispatche un oracle via Bash : omega dispatch <Projet> \"<brief de mission en "
            "anglais : contexte, objectif, critères de done>\" — et dis-le dans le récap.\n"
            "4. Termine par un RÉCAP TELEGRAM (court, direct, ton ton à toi) : ce que tu as "
            "retenu, fichiers modifiés (chemins), actions faites/à venir. Ce récap est ta "
            "SEULE sortie texte.\n\n"
            f"TRANSCRIPT ({dur}s):\n{transcript}"
        )
        t0 = time.time()
        try:
            out = subprocess.run(
                [CLAUDE, "-p", prompt, "--model", "claude-opus-5",
                 "--add-dir", "/",  # the companion is the operator's god-mode assistant
                 "--dangerously-skip-permissions", "--max-turns", "60"],  # a 2h call needs room
                cwd=LIFE, capture_output=True, text=True, timeout=1500,
            ).stdout.strip()
            # An error string is NOT a recap — route it through the retry path.
            if re.search(r"reached max turns|^\s*(error|execution error)\b", out, re.I):
                raise RuntimeError(out[:200])
            recap = out or "Rien de durable à retenir de cet appel — pas de fichier touché."
            tg(f"✅ Transcript traité ({dur}s) :\n\n{recap}")
            log(f"{cid}: processed in {time.time() - t0:.0f}s, recap sent")
            seen.add(cid)
            fail.pop(cid, None)
        except Exception as e:  # noqa: BLE001 — retry next run, notify only once
            n = fail.get(cid, 0) + 1
            fail[cid] = n
            log(f"{cid}: brain failed (try {n}/{MAX_RETRIES}): {e}")
            if n == 1:
                try:
                    tg("⚠️ J'ai le transcript mais le traitement a planté — je réessaie en silence.")
                except Exception:
                    pass
            if n >= MAX_RETRIES:
                seen.add(cid)
                fail.pop(cid, None)
                try:
                    tg(f"❌ J'abandonne le traitement de l'appel {cid} après {MAX_RETRIES} essais — log : ~/.omega/logs/nova-call-sync.log")
                except Exception:
                    pass

    json.dump({"seen": sorted(seen), "fail": fail}, open(STATE_FILE, "w"))


if __name__ == "__main__":
    import fcntl
    lk = open(LOCK, "w")
    try:
        fcntl.flock(lk, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except OSError:
        raise SystemExit(0)
    # No chat, no key, no call agent → silently not configured (fresh installs).
    if not CHAT or not el_key() or not agent_id() or not bot_token():
        raise SystemExit(0)
    main()

#!/usr/bin/env python3
"""argv: radar.json gate.json seen.jsonl replies_cap likes_cap
Prints an action queue as JSONL: {type:reply|like, url, text?, fingerprint}.
Replies: gate keep=true, unseen, capped, dash-stripped, <=280, on-topic. Likes: unseen, capped."""
import sys, json

def strip_dashes(s): return s.replace("—", ", ").replace("–", "-")

radar = json.load(open(sys.argv[1]))
gate = json.load(open(sys.argv[2]))
seen = set()
try:
    for l in open(sys.argv[3]):
        if l.strip(): seen.add(json.loads(l).get("fingerprint"))
except FileNotFoundError:
    pass
replies_cap = int(sys.argv[4]); likes_cap = int(sys.argv[5])

opp = {o.get("fingerprint"): o for o in radar.get("opportunities", [])}
verdict = {v.get("fingerprint"): v for v in gate.get("verdicts", [])}

n = 0
emitted = set()
for fp, v in verdict.items():
    if n >= replies_cap: break
    if not v.get("keep"): continue
    if fp in seen or fp in emitted: continue
    o = opp.get(fp)
    if not o: continue
    text = strip_dashes(v.get("fixed_text") or o.get("reply_text", "")).strip()
    url = o.get("target_url", "")
    if not text or len(text) > 280 or "—" in text or "–" in text: continue
    if not url.startswith("http"): continue
    emitted.add(fp); n += 1
    print(json.dumps({"type": "reply", "url": url, "text": text, "fingerprint": fp}, ensure_ascii=False))

ln = 0
for lk in radar.get("likes", []):
    if ln >= likes_cap: break
    fp = lk.get("fingerprint"); url = lk.get("target_url", "")
    if not url.startswith("http") or fp in seen or fp in emitted: continue
    emitted.add(fp); ln += 1
    print(json.dumps({"type": "like", "url": url, "fingerprint": fp}, ensure_ascii=False))

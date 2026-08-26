#!/usr/bin/env python3
"""argv: analysis.json gate.json seen.jsonl cap
Prints survivor tweets as JSONL {fingerprint,text,source_url}, deduped vs seen, capped, dash-stripped."""
import sys, json

def strip_dashes(s): return s.replace("—", ", ").replace("–", "-")

analysis = json.load(open(sys.argv[1]))
gate = json.load(open(sys.argv[2]))
seen = set()
try:
    for l in open(sys.argv[3]):
        if l.strip(): seen.add(json.loads(l).get("fingerprint"))
except FileNotFoundError:
    pass
cap = int(sys.argv[4])

verdict = {v.get("fingerprint"): v for v in gate.get("verdicts", [])}
cand = {c.get("fingerprint"): c for c in analysis.get("candidates", [])}
count = 0
emitted = set()
for fp, v in verdict.items():
    if count >= cap: break
    if not v.get("keep"): continue
    if fp in seen or fp in emitted: continue
    c = cand.get(fp)
    if not c: continue
    text = strip_dashes(v.get("fixed_text") or c.get("text", "")).strip()
    if not text or len(text) > 280: continue
    if "—" in text or "–" in text: continue
    if "http" not in text: continue  # must carry its source url
    emitted.add(fp); count += 1
    print(json.dumps({"fingerprint": fp, "text": text,
                      "source_url": c.get("source_url", "")}, ensure_ascii=False))

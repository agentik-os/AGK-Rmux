#!/usr/bin/env python3
"""argv: radar.json seen.jsonl -> JSON array of reply candidates (unseen) for the gate."""
import sys, json
radar = json.load(open(sys.argv[1]))
seen = set()
try:
    for l in open(sys.argv[2]):
        if l.strip(): seen.add(json.loads(l).get("fingerprint"))
except FileNotFoundError:
    pass
out = []
for o in radar.get("opportunities", []):
    if o.get("fingerprint") in seen:
        continue
    out.append({"fingerprint": o.get("fingerprint"), "target_url": o.get("target_url"),
                "author": o.get("author"), "reply_text": o.get("reply_text"),
                "target_context": o.get("rationale", "")})
sys.stdout.write(json.dumps(out, ensure_ascii=False))

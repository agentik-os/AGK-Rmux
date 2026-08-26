#!/usr/bin/env python3
"""argv: analysis.json seen.jsonl -> prints JSON array of gate candidates (unseen only)."""
import sys, json
analysis = json.load(open(sys.argv[1]))
seen = set()
try:
    for l in open(sys.argv[2]):
        if l.strip(): seen.add(json.loads(l).get("fingerprint"))
except FileNotFoundError:
    pass
imp_by_fp = {i.get("fingerprint"): i for i in analysis.get("improvements", [])}
out = []
for c in analysis.get("candidates", []):
    fp = c.get("fingerprint")
    if fp in seen:
        continue
    ev = imp_by_fp.get(fp, {}).get("why_it_matters", "")
    out.append({"fingerprint": fp, "text": c.get("text", ""),
                "source_url": c.get("source_url", ""), "evidence": ev})
sys.stdout.write(json.dumps(out, ensure_ascii=False))

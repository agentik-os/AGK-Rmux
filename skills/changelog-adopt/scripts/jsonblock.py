#!/usr/bin/env python3
"""Extract and validate the last fenced ```json block from stdin; print it. Exit 1 if none valid."""
import sys, re, json
txt = sys.stdin.read()
cands = re.findall(r"```(?:json)?\s*\n(.*?)\n```", txt, re.DOTALL)
if not cands:
    m = re.search(r"(\{.*\})", txt, re.DOTALL)  # fallback: first {...}
    cands = [m.group(1)] if m else []
for block in reversed(cands):
    try:
        obj = json.loads(block)
        sys.stdout.write(json.dumps(obj))
        sys.exit(0)
    except Exception:
        continue
sys.stderr.write("jsonblock: no valid json block found\n")
sys.exit(1)

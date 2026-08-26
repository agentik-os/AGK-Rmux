#!/usr/bin/env python3
"""Provider-neutral reference runtime for an Omega OS package.

This runtime is deliberately small: it validates the package, resolves command
routing, and writes append-only events. It does not call an LLM or external API.
Production adapters should enforce authentication, encryption and user consent.
"""
from __future__ import annotations
import argparse, json, hashlib
from pathlib import Path
from datetime import datetime, timezone

ROOT = Path(__file__).resolve().parents[1]

def load_json(rel: str):
    return json.loads((ROOT / rel).read_text(encoding='utf-8'))

def validate() -> dict:
    manifest = load_json('MANIFEST.json')
    failures=[]
    for item in manifest['files']:
        p=ROOT/item['path']
        if not p.exists():
            failures.append({'path':item['path'],'error':'missing'})
            continue
        digest=hashlib.sha256(p.read_bytes()).hexdigest()
        if digest != item['sha256']:
            failures.append({'path':item['path'],'error':'sha256 mismatch'})
    return {'ok':not failures,'failures':failures,'checked':len(manifest['files'])}

def resolve(command: str) -> dict:
    router=load_json('config/router.json')
    key=command.split()[0].lower()
    route=router.get('commands',{}).get(key)
    return {'command':key,'route':route,'known':route is not None}

def append_event(kind: str, payload: str) -> Path:
    log=ROOT/'runtime'/'data'/'events.jsonl'
    log.parent.mkdir(parents=True,exist_ok=True)
    try: obj=json.loads(payload)
    except json.JSONDecodeError: obj={'text':payload}
    event={'at':datetime.now(timezone.utc).isoformat(),'kind':kind,'payload':obj}
    with log.open('a',encoding='utf-8') as f:
        f.write(json.dumps(event,ensure_ascii=False)+'\n')
    return log

def main():
    ap=argparse.ArgumentParser()
    sub=ap.add_subparsers(dest='cmd',required=True)
    sub.add_parser('info')
    sub.add_parser('validate')
    p=sub.add_parser('route'); p.add_argument('command')
    p=sub.add_parser('event'); p.add_argument('kind'); p.add_argument('payload')
    a=ap.parse_args()
    if a.cmd=='info':
        m=load_json('MANIFEST.json'); print(json.dumps({k:m[k] for k in ('name','version','slug','purpose')},indent=2,ensure_ascii=False))
    elif a.cmd=='validate': print(json.dumps(validate(),indent=2))
    elif a.cmd=='route': print(json.dumps(resolve(a.command),indent=2,ensure_ascii=False))
    elif a.cmd=='event': print(append_event(a.kind,a.payload))
if __name__=='__main__': main()

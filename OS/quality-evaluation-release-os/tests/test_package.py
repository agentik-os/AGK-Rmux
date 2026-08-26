from pathlib import Path
import json, hashlib, py_compile
ROOT=Path(__file__).resolve().parents[1]

def test_manifest():
    m=json.loads((ROOT/'MANIFEST.json').read_text())
    assert m['version']
    for item in m['files']:
        p=ROOT/item['path']
        assert p.exists(), item['path']
        assert hashlib.sha256(p.read_bytes()).hexdigest()==item['sha256']

def test_router():
    r=json.loads((ROOT/'config/router.json').read_text())
    assert r['commands']

def test_runtime_compiles():
    py_compile.compile(str(ROOT/'runtime/os_runtime.py'), doraise=True)

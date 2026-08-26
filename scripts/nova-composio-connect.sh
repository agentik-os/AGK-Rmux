#!/usr/bin/env bash
# Connecte une app à Nova via Composio — catalogue: ~/.omega/nova-apps.json
#   sans argument        -> affiche le menu (apps groupées + statut connecté)
#   nova-composio-connect.sh <slug>   -> lance la connexion de l'app
# OAuth2 : imprime l'URL d'autorisation à ouvrir.  api_key : exige la clé dans
# nova-secrets.env (champ "env" du catalogue).  Multi-compte : relancer par compte.
set -uo pipefail

OMEGA="${OMEGA_DIR:-$HOME/.omega}"
CATALOG="$OMEGA/nova-apps.json"
export PATH="$HOME/Linux/bin:$HOME/.local/bin:$PATH"
[ -f "$OMEGA/nova-secrets.env" ] && set -a && . "$OMEGA/nova-secrets.env" && set +a

if [ -z "${COMPOSIO_API_KEY:-}" ]; then
  echo "❌ COMPOSIO_API_KEY manquante — colle-la dans $OMEGA/nova-secrets.env d'abord." >&2
  exit 1
fi
[ -f "$CATALOG" ] || { echo "❌ Catalogue introuvable: $CATALOG" >&2; exit 1; }

# --- sans argument : afficher le menu ---------------------------------------
if [ $# -eq 0 ]; then
  echo "📲 Apps connectables par Nova (Composio)"
  python3 - "$CATALOG" <<'PY'
import json,sys
apps=json.load(open(sys.argv[1]))["apps"]
cats={}
for a in apps: cats.setdefault(a["category"],[]).append(a)
for cat,items in cats.items():
    print(f"\n— {cat} —")
    for a in items:
        mark="🟢" if a.get("connected") else "⚪"
        kind={"oauth2":"OAuth","api_key":"clé API","dcr_oauth":"OAuth-MCP"}.get(a["auth"],a["auth"])
        need=f"  (clé: {a['env']})" if a["auth"]=="api_key" else ""
        print(f"  {mark} {a['label']:<34} {a['slug']:<16} [{kind}]{need}")
print("\n→ Connecter : nova-composio-connect.sh <slug>")
PY
  exit 0
fi

# --- avec argument : connecter une app --------------------------------------
APP="$1"
read -r AUTH ENVVAR < <(python3 - "$CATALOG" "$APP" <<'PY'
import json,sys
apps=json.load(open(sys.argv[1]))["apps"]
m={a["slug"]:a for a in apps}
a=m.get(sys.argv[2])
if not a: print("__MISSING__ -"); sys.exit(0)
print(a["auth"], a.get("env","-"))
PY
)

if [ "$AUTH" = "__MISSING__" ]; then
  echo "❌ '$APP' absent du catalogue. Liste : nova-composio-connect.sh (sans argument)." >&2
  exit 1
fi

# ── Composio API v3 (le CLI composio-core 0.7.x est MORT : HTTP 410) ────────
# Flow v3 : auth_config par toolkit (géré Composio) → connected_account → URL OAuth.
connect_v3() {  # $1=slug  $2=auth  $3=envvar
python3 - "$1" "$2" "${3:-}" <<'PYEOF'
import json, os, sys, urllib.request, urllib.error

slug, auth, envvar = sys.argv[1], sys.argv[2], sys.argv[3]
KEY = os.environ["COMPOSIO_API_KEY"]
BASE = "https://backend.composio.dev/api/v3"

def api(path, body=None):
    req = urllib.request.Request(BASE + path, method="POST" if body else "GET",
        data=json.dumps(body).encode() if body else None,
        headers={"x-api-key": KEY, "content-type": "application/json"})
    try:
        with urllib.request.urlopen(req, timeout=30) as r:
            return json.load(r), None
    except urllib.error.HTTPError as e:
        return None, f"HTTP {e.code}: {e.read().decode()[:300]}"

# 1) auth_config existant pour ce toolkit, sinon création (auth gérée par Composio)
cfgs, err = api(f"/auth_configs?toolkit_slug={slug}")
if err: sys.exit(f"❌ auth_configs: {err}")
ac = next((i["id"] for i in (cfgs.get("items") or []) if i.get("id")), None)
if not ac:
    # api_key toolkits have no Composio-managed credentials → custom auth config
    body = ({"toolkit": {"slug": slug},
             "auth_config": {"type": "use_custom_auth", "authScheme": "API_KEY", "credentials": {}}}
            if auth == "api_key" else
            {"toolkit": {"slug": slug}, "auth_config": {"type": "use_composio_managed_auth"}})
    created, err = api("/auth_configs", body)
    if err: sys.exit(f"❌ création auth_config: {err}")
    ac = created.get("auth_config", {}).get("id") or created.get("id")
if not ac: sys.exit("❌ pas d'auth_config id dans la réponse")

# 2) connected_account → URL de redirection OAuth (ou état direct pour une clé API)
conn = {"user_id": "nova"}
if auth == "api_key":
    val = os.environ.get(envvar or "", "")
    if not val: sys.exit(f"🔑 clé manquante: ajoute {envvar}=... dans ~/.omega/nova-secrets.env")
    conn["state"] = {"authScheme": "API_KEY", "val": {"api_key": val, "generic_api_key": val, "status": "ACTIVE"}}
acc, err = api("/connected_accounts", {"auth_config": {"id": ac}, "connection": conn})
if err: sys.exit(f"❌ connected_account: {err}")

def find_url(o):
    if isinstance(o, str) and o.startswith("http") and ("redirect" not in o[:8]): return o if "composio" in o or "oauth" in o or "auth" in o else None
    if isinstance(o, dict):
        for k in ("redirect_url", "redirectUrl", "redirect_uri"):
            if isinstance(o.get(k), str): return o[k]
        for v in o.values():
            u = find_url(v)
            if u: return u
    if isinstance(o, list):
        for v in o:
            u = find_url(v)
            if u: return u
    return None

url = find_url(acc)
status = (acc.get("connection") or {}).get("status") or acc.get("status") or "?"
if url:
    print(f"🔗 Ouvre cette URL pour autoriser '{slug}' :\n{url}")
elif str(status).upper() in ("ACTIVE", "CONNECTED"):
    print(f"✅ '{slug}' connecté directement (status {status}).")
else:
    print(f"⚠️ Compte créé (status {status}) mais pas d'URL retournée — réponse: {json.dumps(acc)[:400]}")
PYEOF
}

case "$AUTH" in
  api_key)
    echo "→ Connexion de '$APP' (clé API depuis \$$ENVVAR)…"
    connect_v3 "$APP" api_key "$ENVVAR"
    ;;
  *)
    echo "→ Connexion de '$APP' à Composio :"
    connect_v3 "$APP" oauth2 ""
    ;;
esac

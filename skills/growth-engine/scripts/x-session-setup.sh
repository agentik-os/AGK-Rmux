#!/usr/bin/env bash
# Build a Playwright storageState from your X session cookies (auth_token + ct0),
# so the growth engine can drive the @Agentik_os account. Cookies live ONLY in
# ~/.omega/secrets (gitignored, R-ENV), never the repo.
#
# Get the two cookies: log into x.com in your browser → DevTools → Application →
# Cookies → https://x.com → copy the VALUES of `auth_token` and `ct0`.
#
# Usage:
#   x-session-setup.sh <auth_token> <ct0>
#   or set X_AUTH_TOKEN / X_CT0 in the env and run with no args.
set -euo pipefail
OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
OUT="$OMEGA_DIR/secrets/x-session.json"
AUTH="${1:-${X_AUTH_TOKEN:-}}"; CT0="${2:-${X_CT0:-}}"
[ -n "$AUTH" ] && [ -n "$CT0" ] || { echo "usage: x-session-setup.sh <auth_token> <ct0>" >&2; exit 2; }
mkdir -p "$OMEGA_DIR/secrets"
AUTH="$AUTH" CT0="$CT0" OUT="$OUT" python3 - <<'PY'
import os, json
auth, ct0, out = os.environ["AUTH"], os.environ["CT0"], os.environ["OUT"]
exp = 2000000000  # far-future; X will invalidate server-side when the session ends
def ck(name, value, domain):
    return {"name": name, "value": value, "domain": domain, "path": "/",
            "expires": exp, "httpOnly": name == "auth_token", "secure": True, "sameSite": "None"}
cookies = []
for dom in (".x.com", ".twitter.com"):
    cookies.append(ck("auth_token", auth, dom))
    cookies.append(ck("ct0", ct0, dom))
json.dump({"cookies": cookies, "origins": []}, open(out, "w"))
print("wrote", out, "with", len(cookies), "cookies")
PY
chmod 600 "$OUT"
echo "Now verify: bun \$HOME/.omega/lib/growth-engine/playwright-engage.mjs --check"

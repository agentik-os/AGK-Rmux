#!/bin/bash
# ═══════════════════════════════════════════════════════════════
# Claude OAuth Token Exchange — Headless VPS (v2 — No Cache)
#
# Philosophy: NEVER cache tokens. When 401 hits:
#   1. Generate auth URL → send to the operator on Telegram
#   2. the operator clicks → gets code
#   3. Exchange code → write fresh tokens to .credentials.json
#   4. All sessions auto-recover
#
# Usage:
#   claude-oauth reauth              → Full flow: generate URL + wait for code
#   claude-oauth generate            → Just generate auth URL
#   claude-oauth exchange <code>     → Exchange code for tokens
#   claude-oauth check               → Check if current token is valid
#   claude-oauth status              → Show current auth status
#   claude-oauth try-refresh         → Attempt refresh (best-effort, no cache)
# ═══════════════════════════════════════════════════════════════
set -uo pipefail

CREDENTIALS="$HOME/.claude/.credentials.json"
PKCE_STATE="$HOME/.aisb/status/claude-oauth-pkce.json"
CLIENT_ID="9d1c250a-e61b-44d9-88ed-5944d1962f5e"
REDIRECT_URI="https://platform.claude.com/oauth/code/callback"
TOKEN_URL="https://platform.claude.com/v1/oauth/token"
SCOPES="org:create_api_key+user:profile+user:inference+user:sessions:claude_code+user:mcp_servers+user:file_upload"

cmd="${1:-help}"
shift 2>/dev/null || true

case "$cmd" in
    generate)
        # Generate PKCE params and auth URL
        CODE_VERIFIER=$(openssl rand -base64 32 | tr -d '=' | tr '+/' '-_')
        CODE_CHALLENGE=$(echo -n "$CODE_VERIFIER" | openssl dgst -sha256 -binary | openssl base64 -A | tr -d '=' | tr '+/' '-_')
        STATE=$(openssl rand -base64 32 | tr -d '=' | tr '+/' '-_')

        # Save PKCE state for exchange step (ephemeral, deleted after use)
        mkdir -p "$(dirname "$PKCE_STATE")"
        cat > "$PKCE_STATE" << EOF
{"code_verifier":"$CODE_VERIFIER","state":"$STATE","created":"$(date -u +%Y-%m-%dT%H:%M:%SZ)"}
EOF

        AUTH_URL="https://claude.com/cai/oauth/authorize?code=true&client_id=${CLIENT_ID}&response_type=code&redirect_uri=$(python3 -c "import urllib.parse; print(urllib.parse.quote('$REDIRECT_URI'))")&scope=${SCOPES}&code_challenge=${CODE_CHALLENGE}&code_challenge_method=S256&state=${STATE}"

        echo "$AUTH_URL"
        ;;

    exchange)
        # Exchange authorization code for fresh tokens → write to .credentials.json
        FULL_CODE="${1:-}"
        if [ -z "$FULL_CODE" ]; then
            echo '{"ok":false,"error":"No code provided"}'
            exit 1
        fi

        # Extract the auth code, tolerant of how the operator pastes it:
        #   • raw code                     → as-is
        #   • code#state                   → drop the #state suffix
        #   • full callback URL (?code=…)  → pull the code= query param
        # Then strip any stray whitespace/newlines from copy-paste.
        if echo "$FULL_CODE" | grep -q 'code='; then
            AUTH_CODE=$(echo "$FULL_CODE" | sed -E 's/.*[?&]code=([^&#[:space:]]+).*/\1/')
        else
            AUTH_CODE=$(echo "$FULL_CODE" | cut -d'#' -f1)
        fi
        AUTH_CODE=$(echo "$AUTH_CODE" | tr -d '[:space:]')

        # Load PKCE state
        if [ ! -f "$PKCE_STATE" ]; then
            echo '{"ok":false,"error":"No PKCE state. Run generate first."}'
            exit 1
        fi

        CODE_VERIFIER=$(python3 -c "import json; print(json.load(open('$PKCE_STATE'))['code_verifier'])")

        # Token exchange (must be x-www-form-urlencoded per RFC 6749)
        RESPONSE=$(curl -s -X POST "$TOKEN_URL" \
            -H "Content-Type: application/x-www-form-urlencoded" \
            -H "User-Agent: claude-cli/2.1.87" \
            --data-urlencode "grant_type=authorization_code" \
            --data-urlencode "code=${AUTH_CODE}" \
            --data-urlencode "redirect_uri=${REDIRECT_URI}" \
            --data-urlencode "client_id=${CLIENT_ID}" \
            --data-urlencode "code_verifier=${CODE_VERIFIER}" \
            2>&1)

        # Cleanup PKCE state immediately
        rm -f "$PKCE_STATE"

        # Parse response and write credentials
        python3 -c "
import json, time, sys
try:
    resp = json.loads('''$RESPONSE''')
    if 'access_token' not in resp:
        print(json.dumps({'ok': False, 'error': resp.get('error_description', resp.get('error', 'unknown'))}))
        sys.exit(1)

    creds = {'claudeAiOauth': {
        'accessToken': resp['access_token'],
        'refreshToken': resp['refresh_token'],
        'expiresAt': int(time.time() * 1000) + resp['expires_in'] * 1000,
        'scopes': resp['scope'].split(' '),
        'subscriptionType': 'max',
        'rateLimitTier': 'default_claude_max_20x'
    }}

    # Atomic write through the symlink chain: open('w') on the credentials
    # path truncated the SHARED target in place — a concurrent reader saw a
    # 0-byte file and 401'd into "Please run /login". Stage a temp next to
    # the resolved target (mode preserved), then os.replace (rename(2)) so
    # readers always see the old-or-new complete file.
    import os
    target = os.path.realpath('$CREDENTIALS')
    mode = 0o600
    try:
        mode = os.stat(target).st_mode & 0o777
    except OSError:
        pass
    tmp = target + '.oauth.tmp'
    with open(tmp, 'w') as f:
        json.dump(creds, f, indent=2)
    os.chmod(tmp, mode)
    os.replace(tmp, target)

    email = resp.get('account', {}).get('email_address', 'unknown')
    expires_min = resp['expires_in'] // 60
    print(json.dumps({'ok': True, 'email': email, 'expires_min': expires_min}))
except Exception as e:
    print(json.dumps({'ok': False, 'error': str(e)}))
    sys.exit(1)
"
        ;;

    check)
        # Check if current token is still valid (not expired)
        python3 -c "
import json, time, sys
try:
    creds = json.load(open('$CREDENTIALS'))
    exp = creds['claudeAiOauth']['expiresAt']
    now = int(time.time() * 1000)
    remaining_min = (exp - now) // 60000
    expired = now > exp
    # Also warn if expiring in < 30 min
    warning = remaining_min < 30 and not expired
    print(json.dumps({
        'valid': not expired,
        'warning': warning,
        'remaining_min': remaining_min,
        'tier': creds['claudeAiOauth'].get('rateLimitTier', '?')
    }))
    sys.exit(0 if not expired else 1)
except Exception as e:
    print(json.dumps({'valid': False, 'error': str(e)}))
    sys.exit(1)
"
        ;;

    try-refresh)
        # Best-effort token refresh using current refresh_token
        # No caching — just updates .credentials.json directly
        python3 -c "
import json, subprocess, time, sys

creds = json.load(open('$CREDENTIALS'))
refresh_token = creds['claudeAiOauth']['refreshToken']

result = subprocess.run([
    'curl', '-s', '-X', 'POST', '$TOKEN_URL',
    '-H', 'Content-Type: application/x-www-form-urlencoded',
    '-H', 'User-Agent: claude-cli/2.1.87',
    '--data-urlencode', 'grant_type=refresh_token',
    '--data-urlencode', f'refresh_token={refresh_token}',
    '--data-urlencode', 'client_id=$CLIENT_ID',
    '--data-urlencode', 'scope=user:file_upload user:inference user:mcp_servers user:profile user:sessions:claude_code'
], capture_output=True, text=True)

try:
    resp = json.loads(result.stdout)
    if 'access_token' not in resp:
        print(json.dumps({'ok': False, 'error': 'refresh_failed', 'need_reauth': True}))
        sys.exit(1)

    creds['claudeAiOauth']['accessToken'] = resp['access_token']
    if 'refresh_token' in resp:
        creds['claudeAiOauth']['refreshToken'] = resp['refresh_token']
    creds['claudeAiOauth']['expiresAt'] = int(time.time() * 1000) + resp['expires_in'] * 1000

    # Atomic write through the symlink chain (see `exchange`): never truncate
    # the shared target in place — temp next to the resolved target, mode
    # preserved, then os.replace. The cron try-refresh rotates the refresh
    # token; a torn write here would strand every session on a dead token.
    import os
    target = os.path.realpath('$CREDENTIALS')
    mode = 0o600
    try:
        mode = os.stat(target).st_mode & 0o777
    except OSError:
        pass
    tmp = target + '.oauth.tmp'
    with open(tmp, 'w') as f:
        json.dump(creds, f, indent=2)
    os.chmod(tmp, mode)
    os.replace(tmp, target)

    print(json.dumps({'ok': True, 'expires_min': resp['expires_in'] // 60}))
except Exception as e:
    print(json.dumps({'ok': False, 'error': str(e), 'need_reauth': True}))
    sys.exit(1)
"
        ;;

    status)
        # Show current auth status
        claude auth status 2>&1
        echo "---"
        $0 check
        ;;

    reauth)
        # Full interactive reauth flow
        echo "🔐 Generating fresh auth URL..."
        AUTH_URL=$($0 generate)
        echo ""
        echo "📋 Click this link and authorize:"
        echo "$AUTH_URL"
        echo ""
        echo "Then paste the code here (from the callback URL):"
        read -r CODE
        echo ""
        echo "🔄 Exchanging code for tokens..."
        $0 exchange "$CODE"
        ;;

    *)
        echo "Usage: claude-oauth {reauth|generate|exchange|check|try-refresh|status}"
        echo ""
        echo "  reauth       Full flow: generate URL + exchange code"
        echo "  generate     Generate auth URL (send to user)"
        echo "  exchange     Exchange auth code for fresh tokens"
        echo "  check        Check if current token is valid"
        echo "  try-refresh  Attempt token refresh (best-effort)"
        echo "  status       Show full auth status"
        ;;
esac

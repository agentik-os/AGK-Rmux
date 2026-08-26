#!/usr/bin/env bash
# OmegaOS ZernFlow installer — clones the pinned ZernFlow app and wires its
# .env from the OmegaOS secrets vault. OPT-IN: not run by install.sh (a full
# Next.js npm install is heavy), same boundary as higgsfield / browser-use.
# Idempotent. No secret is ever written into the OmegaOS repo (R-ENV / L0).
set -uo pipefail

OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
REPO_URL="https://github.com/zernio-dev/zernflow"
PIN="78f79294e4a57de3d1b375fe05effdea4429f81c"
DST="$OMEGA_DIR/repos/zernflow"
CREDS="$OMEGA_DIR/secrets/zernflow.env"

info() { printf '\033[36m[zernflow]\033[0m %s\n' "$*"; }
warn() { printf '\033[33m[zernflow]\033[0m %s\n' "$*"; }
die()  { printf '\033[31m[zernflow]\033[0m %s\n' "$*"; exit 1; }

command -v git >/dev/null 2>&1 || die "git not found on PATH"

# 1. Clone (or fetch) the pinned commit.
mkdir -p "$OMEGA_DIR/repos"
if [[ -d "$DST/.git" ]]; then
    info "repo present — fetching pinned commit"
    git -C "$DST" fetch --depth 1 origin "$PIN" 2>/dev/null || git -C "$DST" fetch origin
else
    info "cloning $REPO_URL"
    git clone "$REPO_URL" "$DST" || die "clone failed"
fi
git -C "$DST" checkout -q "$PIN" 2>/dev/null \
    || warn "could not pin $PIN — tree left on default branch"
info "checked out $(git -C "$DST" rev-parse --short HEAD)"

# 2. Materialize .env from the vault (never from the repo).
if [[ -f "$CREDS" ]]; then
    # Copy only the keys the app reads; the account-wide sbp_ token is NOT here.
    grep -E '^(NEXT_PUBLIC_SUPABASE_URL|NEXT_PUBLIC_SUPABASE_ANON_KEY|SUPABASE_SERVICE_ROLE_KEY|CRON_SECRET|NEXT_PUBLIC_APP_URL|AI_GATEWAY_API_KEY)=' \
        "$CREDS" > "$DST/.env" && chmod 600 "$DST/.env"
    info ".env written from $CREDS ($(grep -c '=' "$DST/.env") keys)"
else
    warn "no $CREDS — copy .env.example to $DST/.env and fill it in manually"
    [[ -f "$DST/.env.example" ]] && cp "$DST/.env.example" "$DST/.env"
fi

# 3. Install node deps.
if command -v npm >/dev/null 2>&1; then
    info "npm install (this can take a minute)…"
    (cd "$DST" && npm install --no-fund --no-audit) || warn "npm install had warnings"
else
    warn "npm not found — install Node 18+ then run 'npm install' in $DST"
fi

info "done → $DST"
info "next: cd $DST && npm run dev   (deploy: vercel --prod --token=\$VERCEL_TOKEN)"

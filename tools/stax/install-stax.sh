#!/usr/bin/env bash
# OmegaOS Stax installer — clone/refresh the canonical framework checkout that the
# /stax skill reads from. Unlike the pinned zernflow/higgsfield installers, Stax
# TRACKS LATEST main by design ("the dashboard vision structure" — update every time
# main updates). Cheap: git clone only, no bun/npm install (the specimen app build
# stays opt-in). Idempotent. No secret ever touches the repo (R-ENV / L0).
set -uo pipefail

OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
REPO_URL="https://github.com/agentik-os/stax"
DST="$OMEGA_DIR/repos/stax"

info(){ printf '\033[36m[stax]\033[0m %s\n' "$*"; }
warn(){ printf '\033[33m[stax]\033[0m %s\n' "$*"; }
ok(){   printf '\033[32m[stax]\033[0m %s\n' "$*"; }
die(){  printf '\033[31m[stax]\033[0m %s\n' "$*"; exit 1; }

command -v git >/dev/null 2>&1 || die "git not found on PATH"
mkdir -p "$OMEGA_DIR/repos"

if [[ -d "$DST/.git" ]]; then
  info "framework present — fetching latest main"
  git -C "$DST" fetch --quiet origin 2>/dev/null || warn "fetch failed (offline?) — keeping current tree"
  if git -C "$DST" symbolic-ref --quiet --short HEAD 2>/dev/null | grep -qx main; then
    git -C "$DST" merge --ff-only --quiet origin/main 2>/dev/null || warn "not fast-forward — left as-is (local commits?)"
  fi
else
  info "cloning $REPO_URL → $DST"
  git clone --quiet "$REPO_URL" "$DST" || die "clone failed — check network / gh auth"
fi

HEAD="$(git -C "$DST" rev-parse --short HEAD 2>/dev/null || echo '?')"
ok "Stax framework ready at $DST (main @ $HEAD)"
info "The /stax skill reads this checkout live. Daily sync: OMEGA-CRON-STAX-SYNC-v1."
info "Convert a project: bash \$OMEGA_DIR/skills/stax/scripts/stax-scaffold.sh <target-dir>"

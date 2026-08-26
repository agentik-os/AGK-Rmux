#!/usr/bin/env bash
# OmegaOS Lidless installer — clones the pinned Lidless app and runs its own
# installer. OPT-IN: never run by install.sh, because Lidless writes a sudoers
# rule and needs an admin password (same boundary as zernflow / browser-use).
# macOS only: the whole point is `pmset disablesleep`, which does not exist
# elsewhere. Idempotent. No secret involved anywhere (R-ENV / L0).
set -uo pipefail

OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
REPO_URL="https://github.com/agentik-os/Lidless"
PIN="162e00132c34a87f74f49fe6ab8c40c93ad1ae1f"
DST="$OMEGA_DIR/repos/lidless"

info() { printf '\033[36m[lidless]\033[0m %s\n' "$*"; }
warn() { printf '\033[33m[lidless]\033[0m %s\n' "$*"; }
die()  { printf '\033[31m[lidless]\033[0m %s\n' "$*"; exit 1; }

[[ "$(uname -s)" == "Darwin" ]] || die "macOS only — Lidless toggles pmset disablesleep"
command -v git >/dev/null 2>&1 || die "git not found on PATH"
command -v swiftc >/dev/null 2>&1 || die "swiftc not found — install the Xcode command line tools"

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

# 2. Hand over to the app's own installer.
#
# It asks for the admin password ONCE, to write /etc/sudoers.d/lidless scoped
# to exactly two pmset commands. Without a TTY it opens the native macOS
# password dialog instead of hanging on an invisible prompt.
info "running the Lidless installer (it will ask for your admin password)"
bash "$DST/install.sh" || die "Lidless install failed — see the output above"

info "done → /Applications/Lidless.app"
info "the icon lives in the menu bar; remove everything with: bash $DST/uninstall.sh"

#!/usr/bin/env bash
# Install the anthropics/claude-cookbooks CORPUS locally (optional).
#
# The discovery half of this integration — tools/cookbooks/recipes.json — ships
# in the OmegaOS repo and needs nothing from this script: `omega-skills --rag`
# surfaces all 94 recipes on a bare clone and hands back the upstream URL.
#
# This script adds the OFFLINE half: the actual notebooks under
# ~/.omega/cookbooks, so the /cookbook skill can read and adapt real code
# without a network round trip. It is deliberately NOT run by install.sh
# (~70M clone) — the same ship-the-markdown / opt-in-the-payload boundary
# OmegaOS already draws for zernflow, higgsfield and browser-use.
#
#   install-cookbooks.sh            # clone the pinned commit
#   install-cookbooks.sh --update   # move the pin to upstream main, rewrite the lock
#   install-cookbooks.sh --status   # what is installed, and does it match the pin
#
# images/ (141M of notebook screenshots) is excluded via sparse-checkout: the
# recipes are the code, not the pictures.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOCK="$SCRIPT_DIR/COOKBOOKS.lock"
OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
DEST="$OMEGA_DIR/cookbooks"

c_ok=$'\033[32m'; c_info=$'\033[36m'; c_warn=$'\033[33m'; c_err=$'\033[31m'; c_off=$'\033[0m'
ok()   { printf '%s✓%s %s\n'  "$c_ok"   "$c_off" "$*"; }
info() { printf '%s·%s %s\n'  "$c_info" "$c_off" "$*"; }
warn() { printf '%s!%s %s\n'  "$c_warn" "$c_off" "$*" >&2; }
die()  { printf '%s✗%s %s\n'  "$c_err"  "$c_off" "$*" >&2; exit 1; }

[[ -f "$LOCK" ]] || die "missing pin: $LOCK"
UPSTREAM="$(sed -n 's/^UPSTREAM=//p' "$LOCK" | head -1)"
COMMIT="$(sed -n 's/^COMMIT=//p'   "$LOCK" | head -1)"
[[ -n "$UPSTREAM" && -n "$COMMIT" ]] || die "pin is malformed: $LOCK"

command -v git >/dev/null 2>&1 || die "git is required"

current_sha() { git -C "$DEST" rev-parse HEAD 2>/dev/null || true; }

status() {
    if [[ ! -d "$DEST/.git" ]]; then
        info "corpus not installed (discovery still works: recipes.json ships in the repo)"
        info "pin: ${COMMIT:0:7} — install with: $0"
        return 0
    fi
    local have; have="$(current_sha)"
    printf '  dest    %s (%s)\n' "$DEST" "$(du -sh "$DEST" 2>/dev/null | cut -f1)"
    printf '  pinned  %s\n' "${COMMIT:0:7}"
    printf '  local   %s\n' "${have:0:7}"
    if [[ "$have" == "$COMMIT" ]]; then
        ok "corpus matches the pin"
    else
        warn "corpus DRIFTS from the pin — re-run: $0"
    fi
    local n; n="$(find "$DEST" -name '*.ipynb' 2>/dev/null | wc -l)"
    printf '  notebooks %s\n' "$n"
}

update_pin() {
    info "resolving upstream main…"
    local newest
    newest="$(git ls-remote "$UPSTREAM" refs/heads/main | awk '{print $1}')"
    [[ -n "$newest" ]] || die "could not resolve upstream main"
    if [[ "$newest" == "$COMMIT" ]]; then
        ok "already at the newest upstream commit (${newest:0:7})"
        return 0
    fi
    info "pin ${COMMIT:0:7} -> ${newest:0:7}"
    COMMIT="$newest"
    # rewrite only the pin lines; the explanatory header is preserved
    local tmp="$LOCK.tmp"
    sed "s|^COMMIT=.*|COMMIT=$newest|" "$LOCK" > "$tmp" && mv "$tmp" "$LOCK"
    ok "lock updated — re-run without --update to fetch, then rebuild recipes.json"
}

install_corpus() {
    mkdir -p "$OMEGA_DIR"
    if [[ -d "$DEST/.git" ]]; then
        if [[ "$(current_sha)" == "$COMMIT" ]]; then
            ok "corpus already at the pinned commit (${COMMIT:0:7}) — nothing to do"
            return 0
        fi
        info "corpus present at a different commit; fetching ${COMMIT:0:7}…"
    else
        info "cloning ${UPSTREAM##*/} @ ${COMMIT:0:7} (images/ excluded)…"
        rm -rf "$DEST"
        mkdir -p "$DEST"
        git -C "$DEST" init -q                       || die "git init failed"
        git -C "$DEST" remote add origin "$UPSTREAM" || die "git remote add failed"
    fi

    # sparse-checkout in cone mode, then drop images/ — keeps the recipes, not
    # the 141M of screenshots.
    git -C "$DEST" config core.sparseCheckout true
    git -C "$DEST" sparse-checkout init --cone 2>/dev/null || true
    git -C "$DEST" sparse-checkout set --no-cone '/*' '!/images' 2>/dev/null \
        || printf '/*\n!/images/\n' > "$DEST/.git/info/sparse-checkout"

    # --filter=blob:none is what actually keeps this small: a plain --depth 1
    # still downloads every blob in the tree, images included, so the .git dir
    # came out at 161M while the checkout was 70M. With the filter, blobs
    # outside the sparse set are never fetched at all.
    if ! git -C "$DEST" fetch --depth 1 --filter=blob:none origin "$COMMIT" 2>/dev/null; then
        warn "filtered shallow fetch failed; retrying without the blob filter"
        if ! git -C "$DEST" fetch --depth 1 origin "$COMMIT" 2>/dev/null; then
            warn "shallow fetch of the exact commit failed; falling back to a full fetch"
            git -C "$DEST" fetch origin || die "fetch failed — is the network reachable?"
        fi
    fi
    git -C "$DEST" checkout -q FETCH_HEAD 2>/dev/null || git -C "$DEST" checkout -q "$COMMIT" \
        || die "checkout of $COMMIT failed"

    local have; have="$(current_sha)"
    [[ "$have" == "$COMMIT" ]] || die "checked out $have but the pin is $COMMIT"

    local nb size
    nb="$(find "$DEST" -name '*.ipynb' | wc -l)"
    size="$(du -sh "$DEST" 2>/dev/null | cut -f1)"
    ok "corpus installed: $nb notebooks, $size at $DEST (@ ${COMMIT:0:7})"
    info "the /cookbook skill will now read local notebooks instead of upstream URLs"
}

case "${1:-}" in
    --status|status) status ;;
    --update|update) update_pin ;;
    -h|--help)       sed -n '2,25p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//' ;;
    "")              install_corpus ;;
    *)               die "unknown argument: $1 (try --help)" ;;
esac

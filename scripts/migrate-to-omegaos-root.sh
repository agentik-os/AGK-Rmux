#!/usr/bin/env bash
# migrate-to-omegaos-root.sh — consolidate OmegaOS into one clean root:
#
#   ~/OmegaOS/
#     ├── VibeCoding/   your projects        (symlink → existing projects dir)
#     ├── Setup/        the OmegaOS git repo (symlink → this clone)
#     └── System/       all state + secrets  (the real ~/.omega, moved here)
#
# WHY symlinks, not a hard move: ~/.omega is in active use by the patrol cron,
# the systemd Telegram bridge, and any running agents holding file-locks. Moving
# the real directory and leaving a symlink behind means every existing
# ~/.omega caller keeps resolving, with zero code change and zero downtime. The
# binary's config::omega_dir() also prefers ~/OmegaOS/System when it exists, so
# new processes use the new path directly.
#
# This is OPT-IN and IDEMPOTENT. Re-running after a successful migration is a
# no-op. It is reversible (see the printed instructions at the end).
#
# What it does NOT touch (these belong to other tools, not OmegaOS):
#   ~/.claude ~/.codex ~/.config ~/.cargo ~/.rmux.conf ~/.zshrc systemd units
#   ~/.local/bin (the omega/rmux binaries stay on the standard XDG PATH).

set -euo pipefail

ROOT="$HOME/OmegaOS"
SYS="$ROOT/System"
OMEGA="$HOME/.omega"

info() { echo "[INFO] $*"; }
ok()   { echo "[OK]   $*"; }
warn() { echo "[WARN] $*" >&2; }

# 1. Idempotency guard — if ~/.omega is already a symlink, we're migrated.
if [[ -L "$OMEGA" ]]; then
    ok "Already migrated: ~/.omega → $(readlink "$OMEGA")"
    exit 0
fi

mkdir -p "$ROOT"

# 2. Quiesce writers so nothing writes into ~/.omega mid-move.
STOPPED_SERVICE=""
if command -v systemctl >/dev/null 2>&1 && systemctl --user is-active --quiet omega-tg-bot.service 2>/dev/null; then
    systemctl --user stop omega-tg-bot.service && STOPPED_SERVICE=1
    info "Stopped omega-tg-bot.service for the move"
fi
CRON_BACKUP=""
if command -v crontab >/dev/null 2>&1 && crontab -l >/dev/null 2>&1; then
    CRON_BACKUP="$(crontab -l 2>/dev/null || true)"
    crontab -l 2>/dev/null | grep -v 'omega patrol\|omega usage' | crontab - || true
    info "Paused omega crons for the move"
fi

# 3. Move the real state dir into System, leave a symlink at the old path.
if [[ -d "$OMEGA" && ! -L "$OMEGA" ]]; then
    if [[ -e "$SYS" ]]; then
        warn "$SYS already exists — not overwriting. Inspect manually."
    else
        mv "$OMEGA" "$SYS"
        ln -s "$SYS" "$OMEGA"
        ok "Moved ~/.omega → $SYS (symlink left at ~/.omega)"
    fi
elif [[ ! -e "$OMEGA" ]]; then
    # Fresh machine with no ~/.omega yet: just create System + the compat link.
    mkdir -p "$SYS"
    ln -s "$SYS" "$OMEGA"
    ok "Created $SYS (no prior ~/.omega) + compat symlink"
fi

# 4. Projects: symlink the existing project root under the new root. NEVER move
#    a multi-GB repo tree live (open editors, git indexes, running agents with
#    absolute CWDs). A symlink is enough to satisfy the "one root" layout.
if [[ ! -e "$ROOT/VibeCoding" ]]; then
    PROJECTS=""
    for c in VibeCoding projects Projects code dev work; do
        if [[ -d "$HOME/$c" ]]; then PROJECTS="$HOME/$c"; break; fi
    done
    if [[ -n "$PROJECTS" ]]; then
        ln -s "$PROJECTS" "$ROOT/VibeCoding"
        ok "Linked projects: $ROOT/VibeCoding → $PROJECTS"
    else
        mkdir -p "$ROOT/VibeCoding"
        ok "Created empty $ROOT/VibeCoding (no existing project root found)"
    fi
fi

# 5. Setup: point at this clone so the whole product lives under the root too.
if [[ ! -e "$ROOT/Setup" ]]; then
    SRC="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
    ln -s "$SRC" "$ROOT/Setup"
    ok "Linked repo: $ROOT/Setup → $SRC"
fi

# 6. Restart writers. Restore the exact marker-bearing crontab captured above;
# synthesizing one patrol line used to drop usage/update/self-heal jobs.
if [[ -n "$CRON_BACKUP" ]] && command -v crontab >/dev/null 2>&1; then
    printf '%s\n' "$CRON_BACKUP" | crontab - || true
    info "Restored omega crons"
fi
[[ -n "$STOPPED_SERVICE" ]] && systemctl --user start omega-tg-bot.service 2>/dev/null || true

echo
ok "Migration complete. Layout:"
echo "    $ROOT/"
echo "    ├── VibeCoding/  → your projects"
echo "    ├── Setup/       → the OmegaOS repo"
echo "    └── System/      → state + secrets (was ~/.omega; ~/.omega now symlinks here)"
echo
echo "Reversible:  rm ~/.omega && mv $SYS ~/.omega   (then remove ~/OmegaOS)"
echo "Note: ~/.claude, ~/.cargo, ~/.config and other tools' dotfolders are left"
echo "      in place by design — they are not OmegaOS's to move."

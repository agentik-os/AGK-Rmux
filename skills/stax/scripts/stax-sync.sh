#!/usr/bin/env bash
# stax-sync.sh — keep every local Stax checkout current with github.com/agentik-os/stax
# main, and Telegram-notify the operator ONLY when main actually moved. Run daily by the
# OMEGA-CRON-STAX-SYNC-v1 cron, or on demand. Fast-forward ONLY: a checkout with local
# commits is never clobbered (it is reported and skipped — R-DESTRUCT / R-SYNC).
set -uo pipefail

OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
REMOTE_MATCH="agentik-os/stax"

# Checkouts to keep in sync: the OmegaOS-internal template source + the operator's dev
# checkout, plus any colon-separated extras in $STAX_CHECKOUTS.
CHECKOUTS=(
  "$OMEGA_DIR/repos/stax"
  "$HOME/Station/SideBusiness/Stax"
)
if [[ -n "${STAX_CHECKOUTS:-}" ]]; then
  IFS=':' read -r -a _extra <<< "$STAX_CHECKOUTS"
  CHECKOUTS+=("${_extra[@]}")
fi

log(){ printf '%s [stax-sync] %s\n' "$(date -Is 2>/dev/null || echo now)" "$*"; }

notify(){ # $1 = message — best effort, never fatal
  local msg="$1"
  if command -v omega >/dev/null 2>&1; then omega send "$msg" >/dev/null 2>&1 || true
  elif [[ -x "$OMEGA_DIR/bin/nova-send.sh" ]]; then "$OMEGA_DIR/bin/nova-send.sh" "$msg" >/dev/null 2>&1 || true
  fi
}

changed_report=""
seen_paths=""

for DIR in "${CHECKOUTS[@]}"; do
  case ":$seen_paths:" in *":$DIR:"*) continue;; esac   # dedup
  seen_paths="$seen_paths:$DIR"

  [[ -d "$DIR/.git" ]] || { log "skip (not a git checkout): $DIR"; continue; }
  # Only touch checkouts that actually track the Stax repo.
  if ! git -C "$DIR" remote -v 2>/dev/null | grep -qi "$REMOTE_MATCH"; then
    log "skip (not a stax remote): $DIR"; continue
  fi

  branch="$(git -C "$DIR" symbolic-ref --quiet --short HEAD 2>/dev/null || echo detached)"
  old="$(git -C "$DIR" rev-parse HEAD 2>/dev/null || echo unknown)"
  git -C "$DIR" fetch --quiet origin 2>/dev/null || { log "fetch failed: $DIR"; continue; }

  if [[ "$branch" != "main" ]]; then
    log "on '$branch' (not main) — leaving $DIR untouched"; continue
  fi
  # Fast-forward only. Non-ff (local commits) → report, never force.
  if git -C "$DIR" merge-base --is-ancestor HEAD origin/main 2>/dev/null; then
    git -C "$DIR" merge --ff-only --quiet origin/main 2>/dev/null || { log "ff-merge failed: $DIR"; continue; }
  else
    log "non-fast-forward (local commits ahead) — skipping $DIR"; continue
  fi

  new="$(git -C "$DIR" rev-parse HEAD 2>/dev/null || echo unknown)"
  if [[ "$old" != "$new" ]]; then
    subjects="$(git -C "$DIR" log --oneline "$old..$new" 2>/dev/null | head -8)"
    n="$(printf '%s\n' "$subjects" | grep -c . )"
    log "updated $DIR : $old → $new ($n commit(s))"
    changed_report="${changed_report}\n• $(basename "$DIR") (${n}):\n${subjects}"
  else
    log "up to date: $DIR"
  fi
done

if [[ -n "$changed_report" ]]; then
  notify "$(printf '🟥 Stax updated from main:%b' "$changed_report")"
  log "notified operator of changes"
else
  log "no changes across all checkouts"
fi

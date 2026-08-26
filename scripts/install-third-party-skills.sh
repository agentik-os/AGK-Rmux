#!/usr/bin/env bash
# install-third-party-skills.sh — wire two MIT third-party skill collections into
# a fresh OmegaOS install, SHA-pinned and fully reproducible (L0).
#
# WHAT
#   1) obra/superpowers  — 14 process skills (brainstorming, systematic-debugging,
#      test-driven-development, using-git-worktrees, …) + a SessionStart hook that
#      injects the using-superpowers skill into every session's context.
#   2) garrytan/gstack   — 50+ engineering skills, namespaced gstack-* (qa, ship,
#      review, browse, design-review, …) + the `browse` bun binary, built by its
#      own ./setup.
#
# WHY PINNED
#   Both collections are cloned at a REVIEWED commit SHA (safety-reviewed CLEAN),
#   never a moving branch tip. A fresh `git clone OmegaOS && ./install.sh`
#   reproduces the exact same skills every time (L0 install-parity). Bumping a pin
#   is a deliberate, reviewed act, never a silent upstream drift.
#
# UPDATE PATH
#   Edit SUPERPOWERS_PIN / GSTACK_PIN below (or env-override for a candidate test),
#   then re-run `bash scripts/install-third-party-skills.sh` (or ./install.sh).
#   Full doctrine + removal steps: docs/third-party-skills.md.
#
# OPT-IN / OPT-OUT
#   install.sh only runs this script when OMEGA_WITH_THIRD_PARTY=1.
#   OMEGA_SKIP_THIRD_PARTY=1 skips the whole step (wins if both are set).
#   Running this script directly still installs (unless SKIP is set).
#
# ADDITIVE GUARANTEES (zero regression on existing skills/hooks)
#   - Never overwrites an existing non-superpowers skill dir (collision → skip).
#   - Only appends/refreshes the ONE SessionStart superpowers hook entry in
#     settings.json; every other key and hook is preserved byte-for-byte.
#   - Never writes any CLAUDE.md, never touches crontab / systemd.
#   - Best-effort: a network / chromium / bun failure warns and returns non-fatal;
#     it never aborts the caller (install.sh).
#
# Runs standalone or sourced/bashed by install.sh Phase 6.91.

set -uo pipefail

# Standalone-safe logging. Guard on `declare -F` (function-only), never
# `command -v`, so the guard yields to a sourcing parent's function but NOT to an
# unrelated external binary of the same name (e.g. `info` = GNU texinfo). This
# script is bashed (not sourced) by install.sh Phase 6.91, so it always defines
# its own here.
declare -F ok   >/dev/null 2>&1 || ok()   { printf '  \033[32m[OK]\033[0m %s\n'   "$*"; }
declare -F info >/dev/null 2>&1 || info() { printf '  \033[36m[INFO]\033[0m %s\n' "$*"; }
declare -F warn >/dev/null 2>&1 || warn() { printf '  \033[33m[WARN]\033[0m %s\n' "$*" >&2; }
declare -F step >/dev/null 2>&1 || step() { printf '\n\033[1m==> %s\033[0m\n'    "$*"; }

# Non-interactive by construction — never block on a git credential prompt.
export GIT_TERMINAL_PROMPT=0

OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
CLAUDE_SKILLS="$HOME/.claude/skills"
CLAUDE_SETTINGS="${CLAUDE_SETTINGS:-$HOME/.claude/settings.json}"

# ─── Pins (env-overridable for a candidate test) ─────────────────────────────
SUPERPOWERS_REPO="https://github.com/obra/superpowers.git"
SUPERPOWERS_PIN="${SUPERPOWERS_PIN:-d884ae04edebef577e82ff7c4e143debd0bbec99}"
GSTACK_REPO="https://github.com/garrytan/gstack.git"
GSTACK_PIN="${GSTACK_PIN:-11de390be1be6849eb9a15f91ff4922dd16c589a}"

if [ "${OMEGA_SKIP_THIRD_PARTY:-0}" = "1" ]; then
  info "OMEGA_SKIP_THIRD_PARTY=1 — skipping third-party skill collections"
  return 0 2>/dev/null || exit 0
fi

step "Phase 6.91: Third-party skill collections (superpowers + gstack)"

if ! command -v git >/dev/null 2>&1; then
  warn "git not found — cannot install third-party skill collections"
  return 1 2>/dev/null || exit 1
fi

# ─── Helper: idempotent pinned checkout ──────────────────────────────────────
# pin_clone <url> <dest> <sha> — leave <dest> checked out at exactly <sha>.
# Returns 1 (never exits) on any final failure so the caller degrades gracefully.
pin_clone() {
  local url="$1" dest="$2" sha="$3"

  if [ -d "$dest/.git" ]; then
    if [ "$(git -C "$dest" rev-parse HEAD 2>/dev/null)" = "$sha" ]; then
      ok "$(basename "$dest") already at pinned $sha"
      return 0
    fi
    # Existing clone at a different SHA — fetch the target and detach onto it.
    if git -C "$dest" fetch --depth 1 origin "$sha" >/dev/null 2>&1 \
       && git -C "$dest" checkout --detach "$sha" >/dev/null 2>&1; then
      ok "$(basename "$dest") checked out $sha"
      return 0
    fi
    if git -C "$dest" fetch origin >/dev/null 2>&1 \
       && git -C "$dest" checkout --detach "$sha" >/dev/null 2>&1; then
      ok "$(basename "$dest") checked out $sha (full fetch)"
      return 0
    fi
    warn "$(basename "$dest"): could not check out $sha (network?)"
    return 1
  fi

  # Fresh clone. Prefer a shallow fetch-by-SHA to avoid pulling full history.
  # Destructive-fallback guard: only rm -rf "$dest" below if THIS run created it.
  # A pre-existing non-git directory (dest exists but has no .git — we passed the
  # check above) must never be destroyed.
  local we_created=0
  [ -e "$dest" ] || we_created=1
  mkdir -p "$dest"
  if git init -q "$dest" >/dev/null 2>&1 \
     && git -C "$dest" remote add origin "$url" >/dev/null 2>&1 \
     && git -C "$dest" fetch --depth 1 origin "$sha" >/dev/null 2>&1 \
     && git -C "$dest" checkout --detach FETCH_HEAD >/dev/null 2>&1; then
    ok "$(basename "$dest") cloned + pinned $sha"
    return 0
  fi

  # Fallback: full clone then detach (some hosts refuse fetch-by-SHA).
  if [ "$we_created" -ne 1 ]; then
    warn "$(basename "$dest"): pre-existing non-git directory — refusing to rm -rf; skipping"
    return 1
  fi
  rm -rf "$dest"
  if git clone "$url" "$dest" >/dev/null 2>&1 \
     && git -C "$dest" checkout --detach "$sha" >/dev/null 2>&1; then
    ok "$(basename "$dest") cloned (full) + pinned $sha"
    return 0
  fi

  warn "$(basename "$dest"): clone failed for $url"
  return 1
}

SP_OK=0
GS_OK=0

# ─── superpowers ─────────────────────────────────────────────────────────────
SP_DIR="$OMEGA_DIR/repos/superpowers"
mkdir -p "$OMEGA_DIR/repos"
if pin_clone "$SUPERPOWERS_REPO" "$SP_DIR" "$SUPERPOWERS_PIN"; then
  mkdir -p "$CLAUDE_SKILLS"
  sp_linked=0
  for skdir in "$SP_DIR"/skills/*/; do
    [ -f "$skdir/SKILL.md" ] || continue
    name="$(basename "$skdir")"
    target="$CLAUDE_SKILLS/$name"
    # Collision guard: only (re)link if absent OR already OUR superpowers symlink.
    if [ -e "$target" ] || [ -L "$target" ]; then
      link="$(readlink "$target" 2>/dev/null || true)"
      case "$link" in
        *repos/superpowers*) : ;;  # ours — refresh below
        *) warn "collision, skipping $name (existing skill preserved)"; continue ;;
      esac
    fi
    ln -sfn "${skdir%/}" "$target" && sp_linked=$((sp_linked + 1))
  done
  ok "superpowers: $sp_linked skill symlink(s) → $CLAUDE_SKILLS"

  # Hook merge — append/refresh the ONE SessionStart superpowers entry.
  HOOK_CMD="CLAUDE_PLUGIN_ROOT=$SP_DIR bash $SP_DIR/hooks/session-start"
  mkdir -p "$(dirname "$CLAUDE_SETTINGS")"
  [ -f "$CLAUDE_SETTINGS" ] || echo '{}' > "$CLAUDE_SETTINGS"
  if command -v jq >/dev/null 2>&1; then
    # Never replace a dotfile-manager symlink with a regular file: resolve it to
    # its real target so the mv rewrites the file the symlink points at.
    [ -L "$CLAUDE_SETTINGS" ] && CLAUDE_SETTINGS="$(readlink -f "$CLAUDE_SETTINGS")"
    # Preserve the file's mode across the mv (mktemp defaults to 600; don't let
    # the merge silently tighten or loosen the user's chosen mode).
    orig_mode="$(stat -c %a "$CLAUDE_SETTINGS" 2>/dev/null || echo 600)"
    # Same-directory temp so the final mv is an atomic same-filesystem rename
    # (a cross-fs /tmp mktemp would make mv a non-atomic copy). Fall back to a
    # plain mktemp if the same-dir template fails.
    TMP="$(mktemp "${CLAUDE_SETTINGS}.XXXXXX" 2>/dev/null || mktemp)"
    if jq --arg cmd "$HOOK_CMD" '
          .hooks = (.hooks // {})
          | .hooks.SessionStart = ((.hooks.SessionStart // [])
              | map(select(((.hooks[0].command // "") | test("superpowers/hooks/session-start")) | not))
              + [{"matcher":"startup|clear|compact","hooks":[{"type":"command","command":$cmd}]}])
        ' "$CLAUDE_SETTINGS" > "$TMP" 2>/dev/null \
       && [ -s "$TMP" ] && jq empty "$TMP" >/dev/null 2>&1; then
      mv "$TMP" "$CLAUDE_SETTINGS" \
        && chmod "$orig_mode" "$CLAUDE_SETTINGS" 2>/dev/null \
        && ok "superpowers SessionStart hook registered (additive merge)"
    else
      rm -f "$TMP"; warn "superpowers hook merge skipped (jq error) — skills still linked"
    fi
  else
    warn "jq not found — superpowers skills linked; hook NOT registered (install jq)"
  fi

  # Runtime self-check (L1): the hook must actually emit the superpowers context.
  hook_out="$(CLAUDE_PLUGIN_ROOT="$SP_DIR" bash "$SP_DIR/hooks/session-start" 2>/dev/null || true)"
  if printf '%s' "$hook_out" | grep -q "You have superpowers" \
     && printf '%s' "$hook_out" | grep -q "hookSpecificOutput"; then
    ok "superpowers hook runtime-verified (emits 'You have superpowers')"
    SP_OK=1
  else
    warn "superpowers hook produced no expected output (check $SP_DIR/hooks/session-start)"
    SP_OK=1  # skills are linked; hook is best-effort
  fi
else
  warn "superpowers: clone failed — collection not installed (rerun to retry)"
fi

# ─── gstack ──────────────────────────────────────────────────────────────────
# gstack skills hardcode ~/.claude/skills/gstack internally, so the clone MUST
# live exactly there.
GS_DIR="$CLAUDE_SKILLS/gstack"
mkdir -p "$CLAUDE_SKILLS"
if pin_clone "$GSTACK_REPO" "$GS_DIR" "$GSTACK_PIN"; then
  if ! command -v bun >/dev/null 2>&1; then
    warn "bun not found — gstack setup skipped (build needs bun). Install bun, then: cd $GS_DIR && ./setup --prefix --no-plan-tune-hooks"
  else
    # An inherited, non-writable PLAYWRIGHT_BROWSERS_PATH (e.g. a root-owned
    # /Tool/ms-playwright from a parent env) makes gstack setup's chromium
    # install EACCES-fail and link 0 skills. Unset it for THIS process only when
    # its dir is not writable (or does not exist and its parent is not writable).
    if [ -n "${PLAYWRIGHT_BROWSERS_PATH:-}" ]; then
      if [ -d "$PLAYWRIGHT_BROWSERS_PATH" ]; then
        [ -w "$PLAYWRIGHT_BROWSERS_PATH" ] || { info "PLAYWRIGHT_BROWSERS_PATH ($PLAYWRIGHT_BROWSERS_PATH) not writable — unsetting for gstack setup"; unset PLAYWRIGHT_BROWSERS_PATH; }
      elif [ ! -w "$(dirname "$PLAYWRIGHT_BROWSERS_PATH")" ]; then
        info "PLAYWRIGHT_BROWSERS_PATH ($PLAYWRIGHT_BROWSERS_PATH) parent not writable — unsetting for gstack setup"; unset PLAYWRIGHT_BROWSERS_PATH
      fi
    fi
    # --prefix         → namespace as gstack-* (flat mode would collide with the
    #                    existing design + diagram skills)
    # --no-plan-tune-hooks → skip the only interactive settings.json prompt
    # -q               → quiet; </dev/null → never read a TTY prompt
    if ( cd "$GS_DIR" && ./setup --prefix --no-plan-tune-hooks -q < /dev/null ); then
      ok "gstack setup completed"
    else
      warn "gstack setup failed (chromium/network/bun?) — rerun: cd $GS_DIR && ./setup --prefix --no-plan-tune-hooks"
    fi
  fi

  # Post-checks (best-effort, informational).
  gs_count="$(ls -d "$CLAUDE_SKILLS"/gstack-*/ 2>/dev/null | wc -l | tr -d ' ')"
  if [ "${gs_count:-0}" -ge 50 ]; then
    ok "gstack: $gs_count gstack-* skills present"
    GS_OK=1
  else
    warn "gstack: only ${gs_count:-0} gstack-* skills (expected >=50) — setup may be incomplete"
    [ "${gs_count:-0}" -gt 0 ] && GS_OK=1
  fi
  if [ -x "$GS_DIR/browse/dist/browse" ]; then
    ok "gstack browse binary built ($GS_DIR/browse/dist/browse)"
  else
    warn "gstack browse binary missing ($GS_DIR/browse/dist/browse) — rerun ./setup"
  fi
else
  warn "gstack: clone failed — collection not installed (rerun to retry)"
fi

# ─── Convergence guard (heal OmegaOS skill links after gstack relink) ─────────
# gstack setup silently invokes bin/gstack-relink, which rm -f's ANY flat
# ~/.claude/skills symlink whose basename matches one of its skill names, with NO
# provenance check (runtime-proven: it deleted the pre-existing OmegaOS `diagram`
# link; `design` survives only because it is in relink's skip-list). ~/.omega/skills
# is the source of truth and `omega sync` links each of its dirs into
# ~/.claude/skills create-if-missing — replicate that exact semantic here so THIS
# phase always converges: anything relink removed (or never linked) comes back in
# the same run, every run. Runs unconditionally, even when gstack setup failed or
# was skipped.
healed=0
if [ -d "$OMEGA_DIR/skills" ]; then
  mkdir -p "$CLAUDE_SKILLS"
  for od in "$OMEGA_DIR/skills"/*/; do
    [ -d "$od" ] || continue
    oname="$(basename "$od")"
    otarget="$CLAUDE_SKILLS/$oname"
    if [ ! -e "$otarget" ] && [ ! -L "$otarget" ]; then
      ln -sfn "${od%/}" "$otarget" && healed=$((healed + 1))
    fi
  done
fi
[ "$healed" -gt 0 ] \
  && ok "healed $healed ~/.omega/skills link(s) (gstack relink removes colliding flat names)" \
  || info "omega skill links complete (nothing to heal)"

# ─── Epilogue ────────────────────────────────────────────────────────────────
sp_n="$(ls -d "$CLAUDE_SKILLS"/using-superpowers 2>/dev/null | wc -l | tr -d ' ')"
gs_n="$(ls -d "$CLAUDE_SKILLS"/gstack-*/ 2>/dev/null | wc -l | tr -d ' ')"
info "Third-party skills: superpowers=$([ "$SP_OK" = 1 ] && echo installed || echo MISSING) (using-superpowers present: $sp_n), gstack-* count=$gs_n"

if [ "$SP_OK" = 0 ] && [ "$GS_OK" = 0 ]; then
  warn "Both third-party collections failed to install (network?) — rerun later"
  exit 1
fi
exit 0

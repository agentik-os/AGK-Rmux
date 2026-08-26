#!/usr/bin/env bash
# install-skill-atlas.sh — the OmegaOS Skill Atlas: a discovery surface that
# lists every skill in the system and how to run it (R-SKILL-ATLAS).
#
# WHAT (universal, runs for everyone)
#   1) Install the `omega-skills` CLI + its atlas generator into ~/.omega/bin,
#      symlink `omega-skills` onto PATH.
#   2) Build ~/.omega/skills-atlas.json + ~/.omega/artifacts/omega-skill-atlas.html
#      from whatever skills are installed. Idempotent; safe to re-run.
#
# WHAT (optional, operator-only, auth-gated)
#   3) If the operator's PRIVATE skill library repo is reachable (their own
#      authenticated machine), pull it into ~/.omega/skills-library/ so a fresh
#      install recovers the purchased Power-Up corpus too. A public / unauth
#      install cannot reach the private repo and simply SKIPS this step — the
#      paid third-party IP is NEVER shipped in the public OmegaOS repo.
#
# OPT-OUT   OMEGA_SKIP_ATLAS=1 skips the whole step.
#           OMEGA_SKIP_SKILL_LIBRARY=1 skips only the private-library pull.
# ENV       OMEGA_SKILL_LIBRARY_REPO overrides the private repo (default below).
#
# Runs standalone or bashed by install.sh. Never fatal to the caller.
set -uo pipefail
declare -F ok   >/dev/null 2>&1 || ok()   { printf '  \033[32m[OK]\033[0m %s\n'   "$*"; }
declare -F info >/dev/null 2>&1 || info() { printf '  \033[36m[INFO]\033[0m %s\n' "$*"; }
declare -F warn >/dev/null 2>&1 || warn() { printf '  \033[33m[WARN]\033[0m %s\n' "$*" >&2; }
export GIT_TERMINAL_PROMPT=0

[[ "${OMEGA_SKIP_ATLAS:-0}" == "1" ]] && { info "Skill Atlas skipped (OMEGA_SKIP_ATLAS=1)"; exit 0; }

OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
OMEGA_SRC="${OMEGA_SRC:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
mkdir -p "$OMEGA_DIR/bin" "$OMEGA_DIR/artifacts" "$INSTALL_DIR"

# ── 1) CLI + generator ───────────────────────────────────────────────────────
if [[ -f "$OMEGA_SRC/scripts/omega-skills" && -f "$OMEGA_SRC/scripts/omega-skills-atlas.py" ]]; then
    cp -f "$OMEGA_SRC/scripts/omega-skills"          "$OMEGA_DIR/bin/omega-skills"
    cp -f "$OMEGA_SRC/scripts/omega-skills-atlas.py" "$OMEGA_DIR/bin/omega-skills-atlas.py"
    [[ -f "$OMEGA_SRC/scripts/omega-skills-rag.py" ]] && \
        cp -f "$OMEGA_SRC/scripts/omega-skills-rag.py" "$OMEGA_DIR/bin/omega-skills-rag.py"
    chmod +x "$OMEGA_DIR/bin/omega-skills" "$OMEGA_DIR/bin/omega-skills-atlas.py" \
             "$OMEGA_DIR/bin/omega-skills-rag.py" 2>/dev/null || true
    ln -sf "$OMEGA_DIR/bin/omega-skills" "$INSTALL_DIR/omega-skills" 2>/dev/null || true
    # Open Design driver CLI (the daemon itself is an opt-in install:
    # tools/open-design/install-open-design.sh — heavy Docker dep, not auto-run).
    if [[ -f "$OMEGA_SRC/scripts/omega-design" ]]; then
        cp -f "$OMEGA_SRC/scripts/omega-design" "$OMEGA_DIR/bin/omega-design"
        chmod +x "$OMEGA_DIR/bin/omega-design"
        ln -sf "$OMEGA_DIR/bin/omega-design" "$INSTALL_DIR/omega-design" 2>/dev/null || true
    fi
    ok "Skill Atlas CLI installed (omega-skills)"
else
    warn "omega-skills sources not found in $OMEGA_SRC/scripts — atlas CLI not installed"
fi

# ── 3) optional private library pull (auth-gated, operator-only) ─────────────
LIB_ROOT="$OMEGA_DIR/skills-library"
REPO="${OMEGA_SKILL_LIBRARY_REPO:-https://github.com/agentik-os/Agentik-Skills.git}"
# Prefer the operator's agentik-os credential if it is already restored, so the
# private clone works even when a different global git helper would shadow it
# (the credential-chain footgun). Falls back to whatever ambient auth exists.
GITC=(git)
_CRED="$OMEGA_DIR/secrets/agentik-os.git-credentials"
[[ -f "$_CRED" ]] && GITC=(git -c credential.helper= -c "credential.helper=store --file=$_CRED")
if [[ "${OMEGA_SKIP_SKILL_LIBRARY:-0}" != "1" ]]; then
    if [[ -d "$LIB_ROOT/youraipowerup" ]]; then
        info "Power-Up library already present ($LIB_ROOT/youraipowerup) — kept"
    elif "${GITC[@]}" ls-remote "$REPO" >/dev/null 2>&1; then
        mkdir -p "$LIB_ROOT"
        if "${GITC[@]}" clone --depth 1 "$REPO" "$LIB_ROOT/.agentik-skills" >/dev/null 2>&1; then
            # the purchased corpus lives under youraipowerup/ in the private repo
            if [[ -d "$LIB_ROOT/.agentik-skills/youraipowerup" ]]; then
                cp -a "$LIB_ROOT/.agentik-skills/youraipowerup" "$LIB_ROOT/"
                ok "Power-Up library restored from private repo ($(find "$LIB_ROOT/youraipowerup" -name SKILL.md 2>/dev/null | wc -l) skills)"
            fi
        else
            info "Private skill library reachable but clone failed — skipped (non-fatal)"
        fi
    else
        info "Private skill library not reachable (public/unauth install) — Power-Up corpus not shipped, as intended"
    fi
fi

# ── 2a) install OmegaOS-native discovery/method skills into the active namespace
# These are OmegaOS's own skills (no third-party IP) that must always be active so
# oracles/workers and the RAG see them. Copy → ~/.omega/skills, symlink → ~/.claude.
for native_skill in product-development-system open-design; do
    src="$OMEGA_SRC/skills/$native_skill"
    if [[ -d "$src" ]]; then
        mkdir -p "$OMEGA_DIR/skills/$native_skill"
        if command -v rsync >/dev/null 2>&1; then
            rsync -a "$src/" "$OMEGA_DIR/skills/$native_skill/" 2>/dev/null || true
        else
            cp -r "$src/." "$OMEGA_DIR/skills/$native_skill/" 2>/dev/null || true
        fi
        ln -sfn "$OMEGA_DIR/skills/$native_skill" "$HOME/.claude/skills/$native_skill" 2>/dev/null || true
        ok "Native skill active: $native_skill"
    fi
done

# ── 2b) install the ONE always-on router skill (only when the library exists) ─
# powerup-library is a single active-namespace skill (one description, negligible
# context) so Claude auto-matches a relevant prompt and routes into the 907-skill
# library. Installed only if the library is present, so it never dangles.
if [[ -d "$LIB_ROOT/youraipowerup" && -f "$OMEGA_SRC/skills/powerup-library/SKILL.md" ]]; then
    mkdir -p "$HOME/.claude/skills/powerup-library"
    cp -f "$OMEGA_SRC/skills/powerup-library/SKILL.md" "$HOME/.claude/skills/powerup-library/SKILL.md"
    ok "Power-Up router skill installed (auto-discovered by Claude)"
fi

# ── 2) build the atlas from whatever is installed ────────────────────────────
# Compile the typed source catalog first. Atlas and RAG consume this artifact
# when available; a failed compile leaves the previous catalog untouched and
# is never disguised as a successful fresh index.
OMEGA_BIN="$INSTALL_DIR/omega"
[[ -x "$OMEGA_BIN" ]] || OMEGA_BIN="$(command -v omega 2>/dev/null || true)"
if [[ -n "$OMEGA_BIN" && -d "$OMEGA_SRC/skills" ]]; then
    "$OMEGA_BIN" skills compile \
        --root "$OMEGA_SRC/skills" \
        --out "$OMEGA_DIR/skill-catalog-v1.json" >/dev/null \
        && ok "SkillCatalogV1 compiled ($OMEGA_DIR/skill-catalog-v1.json)" \
        || warn "canonical skill catalog compile failed; Atlas will use its bounded legacy fallback"
fi

if [[ -f "$OMEGA_DIR/bin/omega-skills-atlas.py" ]]; then
    python3 "$OMEGA_DIR/bin/omega-skills-atlas.py" >/dev/null 2>&1 \
        && ok "Skill Atlas built ($OMEGA_DIR/artifacts/omega-skill-atlas.html)" \
        || warn "Skill Atlas build had warnings (re-run: omega-skills)"
fi

# ── 4) build the semantic RAG index (embeddings if OPENAI_API_KEY, else BM25) ─
if [[ -f "$OMEGA_DIR/bin/omega-skills-rag.py" ]]; then
    python3 "$OMEGA_DIR/bin/omega-skills-rag.py" build >/dev/null 2>&1 \
        && ok "Skill RAG index built (omega-skills --rag \"<need>\")" \
        || warn "Skill RAG build had warnings (re-run: omega-skills-rag.py build)"
fi

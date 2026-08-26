#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════
# OmegaOS Agent Reach installer — internet reach for every agent.
#
# Agent Reach (MIT, github.com/Panniantong/agent-reach) gives an agent the
# ability to actually READ the internet: X/Twitter, Reddit, YouTube transcripts,
# Bilibili, Xiaohongshu, RSS, GitHub, plus a clean web reader and AI search.
# Without it an agent asked to "go look this up" comes back with HTML soup or a
# 403.
#
# Unlike zernflow/higgsfield this one IS run by install.sh: it is a modest pip
# install with no paid account required, and the operator asked for every future
# install to have it.
#
# SECURITY BOUNDARY — read before changing this file:
#   * The BASE package only is installed. The `cookies` extra is deliberately
#     NOT installed: it pulls browser-cookie3, which reads the local browser's
#     cookie store (that is its documented job — login-gated platforms — but it
#     can read the session cookie of ANY site in that profile). Installing it is
#     the operator's decision, not the installer's:
#         agent-reach-venv/bin/pip install browser-cookie3
#   * API keys are optional and live in ~/.omega/secrets/, never in the repo
#     (R-ENV / L0). The tool works without any of them.
#   * Pinned to a reviewed commit. Bumping the pin means re-reviewing.
#
# Idempotent, and NEVER fatal: a failure here must not abort an OmegaOS install.
# ═══════════════════════════════════════════════════════════════════════════
set -uo pipefail

OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
REPO_URL="https://github.com/Panniantong/agent-reach"
# Reviewed 2026-07-29 (v1.5.0): no sudo, no curl|sh executed, no obfuscation,
# outbound hosts are the platforms it reads + user-configured APIs.
PIN="b4d52c46c9113cb0f653d6df4cf71ebadf4930ac"
SRC="$OMEGA_DIR/repos/agent-reach"
VENV="$OMEGA_DIR/tools/agent-reach/.venv"

info() { printf '\033[36m[agent-reach]\033[0m %s\n' "$*"; }
warn() { printf '\033[33m[agent-reach]\033[0m %s\n' "$*"; }

command -v git >/dev/null 2>&1 || { warn "git not found — skipping"; exit 0; }
PY="$(command -v python3 || true)"
[[ -n "$PY" ]] || { warn "python3 not found — skipping"; exit 0; }
# pyproject declares requires-python >= 3.10; without this guard the failure
# surfaces as an opaque pip resolution error 40 lines later.
"$PY" -c 'import sys; sys.exit(0 if sys.version_info >= (3,10) else 1)' 2>/dev/null \
    || { warn "python3 >= 3.10 required (found $("$PY" -V 2>&1)) — skipping"; exit 0; }
# On Debian/Ubuntu the venv module ships in a SEPARATE package; a missing
# python3-venv is the single most common way this installer dies.
"$PY" -c 'import venv' 2>/dev/null \
    || { warn "python3-venv missing (apt install python3-venv) — skipping"; exit 0; }

# ── 1. Clone or fetch, then pin ──────────────────────────────────────────────
mkdir -p "$OMEGA_DIR/repos" "$OMEGA_DIR/tools/agent-reach"
if [[ -d "$SRC/.git" ]]; then
    git -C "$SRC" fetch --quiet origin 2>/dev/null || warn "fetch failed — using the local copy"
else
    info "cloning $REPO_URL"
    git clone --quiet "$REPO_URL" "$SRC" 2>/dev/null || { warn "clone failed — skipping"; exit 0; }
fi
git -C "$SRC" checkout --quiet "$PIN" 2>/dev/null \
    || warn "could not pin $PIN — tree left on its default branch"
info "source at $(git -C "$SRC" rev-parse --short HEAD 2>/dev/null || echo unknown)"

# ── 2. Install into a dedicated venv ─────────────────────────────────────────
# A venv, not `pip install --user`: agent-reach pulls yt-dlp and friends, and
# they have no business in the system site-packages of a machine that also runs
# other Python tooling.
if [[ ! -x "$VENV/bin/python" ]]; then
    "$PY" -m venv "$VENV" 2>/dev/null || { warn "venv creation failed — skipping"; exit 0; }
fi
"$VENV/bin/pip" install --quiet --upgrade pip 2>/dev/null || true
# Base package only — see the security boundary at the top of this file.
# Prefer upstream's tested constraint set so a machine installed today and one
# installed in six months resolve the SAME dependency versions (L0). yt-dlp in
# particular ships breaking changes often. Fall back to an unconstrained install
# rather than leaving the operator with no tool at all.
if [[ -f "$SRC/constraints.txt" ]] \
   && "$VENV/bin/pip" install --quiet -c "$SRC/constraints.txt" "$SRC" 2>/dev/null; then
    info "installed against upstream constraints.txt (pinned dependency set)"
elif "$VENV/bin/pip" install --quiet "$SRC" 2>/dev/null; then
    warn "constraints.txt unusable — installed with unpinned dependencies"
else
    warn "pip install failed — agent-reach not available (OmegaOS install continues)"
    exit 0
fi

# ── 3. Put it on PATH ────────────────────────────────────────────────────────
mkdir -p "$OMEGA_DIR/bin" "$INSTALL_DIR"
if [[ -x "$VENV/bin/agent-reach" ]]; then
    ln -sf "$VENV/bin/agent-reach" "$OMEGA_DIR/bin/agent-reach"
    ln -sf "$VENV/bin/agent-reach" "$INSTALL_DIR/agent-reach" 2>/dev/null || true
    VER="$("$VENV/bin/agent-reach" --version 2>/dev/null | head -1 || echo '?')"
    info "installed: agent-reach $VER → $INSTALL_DIR/agent-reach"
else
    warn "the agent-reach entry point is missing after install"
    exit 0
fi

# ── 4. Register the skill in the OmegaOS SSOT, not ~/.claude directly ────────
# `agent-reach skill --install` writes straight into ~/.claude/skills, which on
# an OmegaOS box is a SYNCED directory: gstack's relink removes flat names that
# collide with its own basenames, so a skill dropped there can silently vanish.
# The canonical home is ~/.omega/skills (R-SKILLPUB); `omega sync` links it into
# every agent config dir from there. English locale — the default SKILL.md is
# Chinese, and the operator does not read it.
SKILL_SRC="$SRC/agent_reach/skill"
SKILL_DST="$OMEGA_DIR/skills/agent-reach"
if [[ -d "$SKILL_SRC" ]]; then
    mkdir -p "$SKILL_DST"
    SKILL_INPUT=""
    if [[ -f "$SKILL_SRC/SKILL_en.md" ]]; then
        SKILL_INPUT="$SKILL_SRC/SKILL_en.md"
    elif [[ -f "$SKILL_SRC/SKILL.md" ]]; then
        SKILL_INPUT="$SKILL_SRC/SKILL.md"
    fi
    if [[ -n "$SKILL_INPUT" ]]; then
        # Upstream's OpenClaw-specific nested `metadata` mapping is outside the
        # deliberately bounded OmegaOS skill schema. Keep the complete skill
        # body and supported scalar frontmatter, but remove that foreign block
        # before publishing into the canonical skill store. Publish atomically
        # so `omega sync` can never observe a half-written protocol.
        SKILL_TMP="$(mktemp "$SKILL_DST/.SKILL.md.XXXXXX")" || {
            warn "could not create a private skill staging file — registration skipped"
            SKILL_TMP=""
        }
        if [[ -n "$SKILL_TMP" ]]; then
            if awk '
                NR == 1 && $0 == "---" { in_frontmatter = 1; print; next }
                in_frontmatter && $0 == "---" {
                    in_frontmatter = 0; skip_metadata = 0; print; next
                }
                in_frontmatter && /^[^[:space:]][^:]*:/ {
                    if ($0 ~ /^metadata:[[:space:]]*$/) {
                        skip_metadata = 1; next
                    }
                    skip_metadata = 0
                }
                in_frontmatter && skip_metadata { next }
                { print }
            ' "$SKILL_INPUT" > "$SKILL_TMP"; then
                chmod 0644 "$SKILL_TMP"
                mv -f "$SKILL_TMP" "$SKILL_DST/SKILL.md"
            else
                rm -f "$SKILL_TMP"
                warn "could not normalize Agent Reach skill metadata — registration skipped"
            fi
        fi
    fi
    [[ -d "$SKILL_SRC/references" ]] && cp -r "$SKILL_SRC/references" "$SKILL_DST/" 2>/dev/null
    if [[ -f "$SKILL_DST/SKILL.md" ]] \
       && "$INSTALL_DIR/omega" skills validate --root "$SKILL_DST" >/dev/null 2>&1; then
        info "skill registered + schema-validated → $SKILL_DST (run 'omega sync' to link it into agent config dirs)"
    else
        rm -f "$SKILL_DST/SKILL.md"
        warn "Agent Reach skill failed OmegaOS schema validation — tool kept, skill registration skipped"
    fi
    unset SKILL_INPUT SKILL_TMP
fi

# ── 5. Optional keys, from the vault only ────────────────────────────────────
# Every one of these is optional; the tool degrades to its free paths without
# them. They are READ from ~/.omega/secrets, never written into the repo.
CREDS="$OMEGA_DIR/secrets/integrations.env"
if [[ -f "$CREDS" ]] && grep -qE '^(EXA_API_KEY|GROQ_API_KEY)=' "$CREDS" 2>/dev/null; then
    info "optional API keys detected in secrets/integrations.env"
else
    info "no optional keys set — free paths still work (add EXA_API_KEY / GROQ_API_KEY to $CREDS to unlock search + transcription)"
fi

exit 0

#!/usr/bin/env bash
# install-companion-tools.sh — opt-out companion tools + skills for a fresh
# OmegaOS install. Sourced by install.sh (Phase 6.9), or runnable standalone.
#
# DESIGN
# - Best-effort: a network/registry hiccup on ONE tool must never fail the whole
#   OmegaOS install (same contract as the optional xvfb/claude steps). No `set -e`.
# - Idempotent: every step checks "already installed?" first and no-ops if so.
# - SST / multi-LLM: skills install through the `skills` CLI with `-g`, which
#   writes into EVERY supported agent's discovery dir (claude / codex / gemini /
#   glm / ...), so one install serves all backends. Claude-Code-specific plugins
#   (superpowers, claude-mem) install into ~/.claude.
# - Opt-out: OMEGA_SKIP_COMPANION=1 skips everything; OMEGA_SKIP_MEMPALACE=1
#   skips only the heavy Python/uv tool.
#
# Runs AFTER the agent CLIs (Phase 6.5d) so `claude` is on PATH for the plugins.

set -uo pipefail

# Standalone-safe logging (install.sh defines these; redefine if run alone).
command -v ok   >/dev/null 2>&1 || ok()   { printf '  \033[32m[OK]\033[0m %s\n'   "$*"; }
command -v info >/dev/null 2>&1 || info() { printf '  \033[36m[INFO]\033[0m %s\n' "$*"; }
command -v warn >/dev/null 2>&1 || warn() { printf '  \033[33m[WARN]\033[0m %s\n' "$*" >&2; }
command -v step >/dev/null 2>&1 || step() { printf '\n\033[1m==> %s\033[0m\n'    "$*"; }

OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
have() { command -v "$1" >/dev/null 2>&1; }

if [ "${OMEGA_SKIP_COMPANION:-0}" = "1" ]; then
  info "OMEGA_SKIP_COMPANION=1 — skipping companion tools"
  return 0 2>/dev/null || exit 0
fi

step "Phase 6.9: Companion tools + skills (SST, multi-LLM)"

# A JS runner for the skills CLI + npx-based installers. bun preferred.
NPX=""
if   have bunx; then NPX="bunx"
elif have npx;  then NPX="npx"
else warn "no bunx/npx — skipping JS-based companion tools (install bun or node)"
fi

# 1. planning-with-files — Manus-style persistent-markdown planning skill.
#    `skills add ... -g` is the SST primitive: lands in every agent's skill dir.
if [ -n "$NPX" ]; then
  if "$NPX" -y skills add OthmanAdi/planning-with-files --skill planning-with-files -g </dev/null >/dev/null 2>&1; then
    ok "planning-with-files skill installed (all agents)"
  else
    warn "planning-with-files: install manually -> npx skills add OthmanAdi/planning-with-files -g"
  fi
fi

# 1b. Design/taste skill packs (SST -g — every agent dir). Best-effort.
if [ -n "$NPX" ]; then
  if "$NPX" -y skills add Leonxlnx/taste-skill -g </dev/null >/dev/null 2>&1; then
    ok "taste-skill installed (all agents)"
  else
    warn "taste-skill: install manually -> npx skills add Leonxlnx/taste-skill -g"
  fi
  if "$NPX" -y skills add emilkowalski/skill -g </dev/null >/dev/null 2>&1; then
    ok "impeccable (emilkowalski/skill) installed (all agents)"
  else
    warn "impeccable: install manually -> npx skills add emilkowalski/skill -g"
  fi
fi

# 2. Higgsfield — image/video generation CLI + its skill pack.
if have npm; then
  if have higgsfield; then ok "higgsfield CLI already installed"
  elif npm install -g @higgsfield/cli >/dev/null 2>&1; then ok "higgsfield CLI installed"
  else warn "higgsfield CLI: install manually -> npm i -g @higgsfield/cli"; fi
fi
if [ -n "$NPX" ]; then
  if "$NPX" -y skills add higgsfield-ai/skills -g </dev/null >/dev/null 2>&1; then
    ok "higgsfield skills installed (all agents)"
  else
    warn "higgsfield skills: install manually -> npx skills add higgsfield-ai/skills -g"
  fi
fi

# 3. claude-mem — session-memory compression plugin (Claude Code hooks).
#    npx installer (NOT npm -g, which ships the SDK only without hooks).
if [ -n "$NPX" ] && have claude; then
  if [ -d "$HOME/.claude/plugins/cache/thedotmack/claude-mem" ] || [ -d "$HOME/.claude-mem" ]; then
    ok "claude-mem already installed"
  elif "$NPX" -y claude-mem install </dev/null >/dev/null 2>&1; then
    ok "claude-mem installed (session memory + hooks)"
  else
    warn "claude-mem: install manually -> npx claude-mem install"
  fi
elif ! have claude; then
  info "claude CLI absent — claude-mem skipped (install Claude Code, then: npx claude-mem install)"
fi

# 4. Superpowers — TDD/debugging skills framework (Claude Code plugin).
if have claude; then
  if ls "$HOME/.claude/plugins/cache"/*superpower* >/dev/null 2>&1 \
     || ls "$HOME/.claude/skills" 2>/dev/null | grep -qi 'brainstorm\|superpower'; then
    ok "superpowers already present"
  elif claude plugin marketplace add obra/superpowers >/dev/null 2>&1 \
       && claude plugin install superpowers@superpowers-marketplace >/dev/null 2>&1; then
    ok "superpowers plugin installed"
  else
    warn "superpowers: in Claude Code run -> /plugin install superpowers@claude-plugins-official"
  fi
fi

# 5. MemPalace — local-first verbatim memory (Python CLI + MCP). Heavy-ish; opt-out.
if [ "${OMEGA_SKIP_MEMPALACE:-0}" != "1" ]; then
  if have mempalace; then
    ok "mempalace already installed"
  else
    if ! have uv; then
      info "installing uv (for mempalace)..."
      curl -LsSf https://astral.sh/uv/install.sh | sh >/dev/null 2>&1 || true
      [ -f "$HOME/.local/bin/env" ] && . "$HOME/.local/bin/env" 2>/dev/null || true
      export PATH="$HOME/.local/bin:$PATH"
    fi
    if have uv && uv tool install mempalace >/dev/null 2>&1; then
      ok "mempalace installed (uv tool)"
    elif have pipx && pipx install mempalace >/dev/null 2>&1; then
      ok "mempalace installed (pipx)"
    else
      warn "mempalace: install manually -> uv tool install mempalace (or pipx install mempalace)"
    fi
  fi
fi

# 6. Remotion CLI — programmatic video (global; projects scaffold via create-video).
if have npm; then
  if have remotion; then ok "remotion CLI already installed"
  elif npm install -g @remotion/cli >/dev/null 2>&1; then ok "remotion CLI installed"
  else warn "remotion CLI: install manually -> npm i -g @remotion/cli"; fi
fi

# 7. claude-code-best-practice — reference docs (read-only clone, not a tool).
REF_DIR="$OMEGA_DIR/docs/references/claude-code-best-practice"
if have git; then
  if [ -d "$REF_DIR/.git" ]; then
    ok "claude-code-best-practice reference present"
  else
    mkdir -p "$(dirname "$REF_DIR")"
    if git clone --depth 1 https://github.com/shanraisshan/claude-code-best-practice "$REF_DIR" >/dev/null 2>&1; then
      ok "claude-code-best-practice reference cloned -> $REF_DIR"
    else
      warn "claude-code-best-practice: clone manually into $REF_DIR"
    fi
  fi
fi

ok "Companion tools phase complete (per-project shadcn theme kit + Remotion handled by /omega-new-project)"

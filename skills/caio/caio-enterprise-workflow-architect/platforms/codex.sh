#!/usr/bin/env bash
set -euo pipefail

SKILL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

[[ -f "$SKILL_DIR/SKILL.md" ]] || { echo "[codex adapter] SKILL.md missing"; exit 1; }

if [[ ! -e "$SKILL_DIR/AGENTS.md" ]]; then
  ln -s SKILL.md "$SKILL_DIR/AGENTS.md"
fi

echo "[codex adapter] caio-enterprise-workflow-architect ready in ${SKILL_DIR}"
echo "[codex adapter] try in Codex: /caio-enterprise-workflow-architect"

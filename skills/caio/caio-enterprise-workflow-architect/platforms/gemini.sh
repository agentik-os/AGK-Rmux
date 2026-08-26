#!/usr/bin/env bash
set -euo pipefail

SKILL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

[[ -f "$SKILL_DIR/SKILL.md" ]] || { echo "[gemini adapter] SKILL.md missing"; exit 1; }

cat > "$SKILL_DIR/GEMINI.md" <<'GMINI'
# CAIO Enterprise Workflow Architect (Gemini activation pointer)

Entry: ./SKILL.md
References: ./references/*.md
GMINI

echo "[gemini adapter] caio-enterprise-workflow-architect ready in ${SKILL_DIR}"
echo "[gemini adapter] activate with: gemini skill activate caio-enterprise-workflow-architect"

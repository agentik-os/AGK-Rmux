#!/usr/bin/env bash
set -euo pipefail

SKILL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

[[ -f "$SKILL_DIR/SKILL.md" ]] || { echo "[gemini adapter] SKILL.md missing"; exit 1; }

cat > "$SKILL_DIR/GEMINI.md" <<'GMINI'
# CAIO Implementation Runbook (Gemini activation pointer)

Entry: ./SKILL.md
References: ./references/*.md
Templates: ./assets/templates/*
GMINI

echo "[gemini adapter] caio-implementation-runbook ready in ${SKILL_DIR}"
echo "[gemini adapter] activate with: gemini skill activate caio-implementation-runbook"

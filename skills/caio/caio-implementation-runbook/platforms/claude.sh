#!/usr/bin/env bash
set -euo pipefail

SKILL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

[[ -f "$SKILL_DIR/SKILL.md" ]] || { echo "[claude adapter] SKILL.md missing"; exit 1; }
[[ -d "$SKILL_DIR/references" ]] || { echo "[claude adapter] references/ missing"; exit 1; }

echo "[claude adapter] caio-implementation-runbook ready in ${SKILL_DIR}"
echo "[claude adapter] try in Claude Code: /caio-implementation-runbook"

#!/usr/bin/env bash
# Audit doc d'un projet (LECTURE SEULE). Classe les .md en PROTECTED / SCRATCH / REVIEW.
# Usage: doc-audit.sh <project_root>
set -uo pipefail
ROOT="${1:?usage: doc-audit.sh <project_root>}"
[ -d "$ROOT" ] || { echo "introuvable: $ROOT"; exit 1; }

PROTECT_RE='(/vision/|vision\.md|prd|feature|step|/Vision/|OMEGA\.md|CLAUDE\.md|AGENTS?\.md|README|CHANGELOG|LICEN[CS]E|SECURITY\.md|CODE_OF_CONDUCT|CONTRIBUTING|/\.omega/rules/|fiche)'
SCRATCH_RE='(/\.oracles/|/\.audit/|/outcomes/|/report\.md|/update\.md|-report-|\.rubric\.md|FIXES-SUMMARY|READY-TO-DEPLOY|DEPLOY-NOW|HANDOFF|-results\.md|-RESULTS\.md|\.planner/.*report)'

# 1) Repérer les dossiers "contenu produit" (>20 .md) -> protégés en bloc
declare -A CONTENT_DIR
while read -r dir cnt; do
  [ "${cnt:-0}" -gt 20 ] && CONTENT_DIR["$dir"]=1
done < <(find "$ROOT" -name '*.md' -not -path '*/node_modules/*' -not -path '*/.git/*' -not -path '*/target/*' -printf '%h\n' 2>/dev/null | sort | uniq -c | awk '{print $2, $1}')

is_content() { local f="$1" d; d=$(dirname "$f"); while [ "$d" != "$ROOT" ] && [ "$d" != "/" ]; do [ -n "${CONTENT_DIR[$d]:-}" ] && return 0; d=$(dirname "$d"); done; [ -n "${CONTENT_DIR[$ROOT]:-}" ] && return 0; return 1; }

prot=0; scr=0; rev=0; scr_bytes=0
SCRATCH_LIST=$(mktemp)
while IFS= read -r f; do
  if echo "$f" | grep -qiE "$PROTECT_RE" || is_content "$f"; then prot=$((prot+1)); continue; fi
  if echo "$f" | grep -qiE "$SCRATCH_RE"; then
    scr=$((scr+1)); b=$(stat -c%s "$f" 2>/dev/null||echo 0); scr_bytes=$((scr_bytes+b))
    echo "$f" >> "$SCRATCH_LIST"; continue
  fi
  rev=$((rev+1))
done < <(find "$ROOT" -name '*.md' -not -path '*/node_modules/*' -not -path '*/.git/*' -not -path '*/target/*' 2>/dev/null)

echo "### $(basename "$ROOT")"
echo "  🔒 PROTECTED (canon + contenu produit) : $prot"
echo "  🗑️  SCRATCH (proposés suppression)      : $scr  (~$((scr_bytes/1024)) Ko)"
echo "  🔎 REVIEW (à trancher avec toi)         : $rev"
if [ "$scr" -gt 0 ]; then
  echo "  -- échantillon SCRATCH --"
  head -25 "$SCRATCH_LIST" | sed "s#^$ROOT/#     #"
fi
rm -f "$SCRATCH_LIST"

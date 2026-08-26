#!/usr/bin/env bash
# Range un projet : déplace SEULEMENT ce qui est positivement identifié comme doc humaine
# (→docs/) ou junk agentic visible (→agentic/). TOUT le reste (code, config, inconnu) RESTE.
# Ne MODIFIE jamais de contenu. Réversible (manifest + git).
# Usage: tidy-apply.sh <project_dir> [--apply]   (défaut: dry-run)
set -uo pipefail
HERE="$(cd "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")" && pwd)"
P="${1:?usage: tidy-apply.sh <project_dir> [--apply]}"
APPLY=0; [ "${2:-}" = "--apply" ] && APPLY=1
[ -d "$P" ] || { echo "introuvable: $P"; exit 1; }
cd "$P" || exit 1
OWNER=$(stat -c %U . 2>/dev/null || echo root)
BK=${OMEGA_CLEANUP_BK:-$HOME/.omega/cleanup-backups}; mkdir -p "$BK" 2>/dev/null || true
MAN="$BK/tidy-$(basename "$P" | tr ' /' '__')-manifest.txt"

# DOC humaine -> docs/   (positif uniquement, JAMAIS le canon ni les fichiers d'instruction agent)
is_doc() {
  local f="$1"
  # Canon / instructions agent / vision-PRD-feature-step : NE JAMAIS déplacer
  case "$f" in
    README*|CLAUDE.md|AGENTS.md|AGENT.md|@*.md|OMEGA.md|RULES.md|PROGRESS.md|WARP.md|\
    CHANGELOG*|SECURITY*|CONTRIBUTING*|LICEN[CS]E*|CODE_OF_CONDUCT*|.cursorrules|.windsurfrules) return 1;;
    *[Vv]ision*|*VISION*|PRD*|*PRD*|*[Ff]eature*|*FEATURE*|*[Ss]tep*|*STEP*) return 1;;
  esac
  case "$f" in
    *.md|Doc|knowledge|guide|guides|documentation|*.pdf) return 0;;
    *) return 1;;
  esac
}
# JUNK agentic visible -> agentic/<sous-cat>   (positif uniquement ; jamais du code)
route_agentic() {
  case "$1" in
    audit|audits|*audit) echo audits; return 0;;
    report.md|*REPORT*|RAPPORT*|qa-*|qa_*) echo reports; return 0;;
    deep-test-*|probe-*|_capture-*|*-preflight.*|maniac-*|*.log|log|logs|playwright-report|test-results|coverage) echo tests; return 0;;
    prd-*.json|status.json|outcomes) echo specs; return 0;;
    to\ order|to-order|to_order) echo archive; return 0;;
    *) return 1;;
  esac
}

declare -a DOCS=() AG=() SKIP=()
shopt -s nullglob
for e in *; do
  [ -e "$e" ] || continue
  case "$e" in .*) continue;; esac          # dotfiles/dotfolders : jamais touchés
  if sub=$(route_agentic "$e"); then AG+=("$sub|$e"); continue; fi
  if is_doc "$e"; then DOCS+=("$e"); continue; fi
  SKIP+=("$e")                               # TOUT le reste (code, config, inconnu) : on NE touche pas
done

echo "### $(basename "$P")  [$([ $APPLY = 1 ] && echo APPLY || echo DRY)]  →docs:${#DOCS[@]} →agentic:${#AG[@]} keep:${#SKIP[@]}"
[ ${#DOCS[@]} -gt 0 ] && printf '   →docs/    %s\n' "${DOCS[@]}"
[ ${#AG[@]}  -gt 0 ] && printf '   →agentic/%s\n' "${AG[@]}"

[ $APPLY = 0 ] && exit 0
[ ${#DOCS[@]} -eq 0 ] && [ ${#AG[@]} -eq 0 ] && { echo "  (rien à ranger)"; exit 0; }

run() { if [ "$OWNER" != root ]; then sudo -u "$OWNER" "$@"; else "$@"; fi; }
run mkdir -p docs agentic/audits agentic/reports agentic/tests agentic/specs agentic/archive
for e in "${DOCS[@]}"; do [ -e "$e" ] && [ ! -e "docs/$e" ] && mv "$e" docs/ && echo "$(date +%F) $e -> docs/" >> "$MAN"; done
for pair in "${AG[@]}"; do sub="${pair%%|*}"; e="${pair#*|}"; [ -e "$e" ] && [ ! -e "agentic/$sub/$e" ] && mv "$e" "agentic/$sub/" && echo "$(date +%F) $e -> agentic/$sub/" >> "$MAN"; done
chown -R "$OWNER:$OWNER" docs agentic 2>/dev/null || true
if [ -f CLAUDE.md ] && ! grep -q 'AGENTIK-LAYOUT:START' CLAUDE.md; then
  printf '\n' >> CLAUDE.md; cat "$HERE/claude-md-block.md" >> CLAUDE.md
  echo "  ✅ convention CLAUDE.md injectée"
fi
[ -f CLAUDE.md ] && [ ! -e AGENTS.md ] && run ln -s CLAUDE.md AGENTS.md 2>/dev/null
echo "  ✅ rangé (manifest: $MAN)"

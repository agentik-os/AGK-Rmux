#!/usr/bin/env bash
# Plan de rangement d'un projet (LECTURE SEULE). Catégorise les entrées VISIBLES de la racine.
# Ignore entièrement les dotfiles/dotfolders (ils restent à la racine).
# Usage: tidy-scan.sh <project_dir> [--stale N]
set -uo pipefail
P="${1:?usage: tidy-scan.sh <project_dir> [--stale N]}"
STALE=0; [ "${2:-}" = "--stale" ] && STALE="${3:-7}"
cd "$P" 2>/dev/null || { echo "introuvable: $P"; exit 1; }
now=$(date +%s)

CANON='^(README.*|CLAUDE\.md|AGENTS\.md|RULES\.md|PROGRESS\.md|LICEN[CS]E.*|CHANGELOG.*|SECURITY\.md|CODE_OF_CONDUCT.*|CONTRIBUTING.*|[Vv]ision|VISION.*|PRD.*|.*[Ff]eature.*|.*[Ss]tep.*)$'
CODE='^(app|src|pages|component|components|lib|convex|hooks|contexts|service|services|server|types|config|style|styles|script|scripts|public|packages|crates|utils|api|assets|locales|messages|prisma|drizzle|supabase|desktop-app|ios|android|website|middleware\..*|instrumentation\..*|test|tests|__tests__|cypress|stories)$'
CONFIG='(^package\.json$|^package-lock\.json$|.*\.lock$|.*lock\.json$|^tsconfig.*|^jsconfig.*|.*\.config\.[mc]?[jt]s$|.*\.config\.json$|^next-env\.d\.ts$|^Cargo\.(toml|lock)$|^Makefile$|^Dockerfile.*|^docker-compose.*|^components\.json$|^vercel\.json$|.*\.code-workspace$|^requirements\.txt$|^pyproject\.toml$|^go\.(mod|sum)$|^babel\.config.*|^jest\.config.*)$'
HOUSE='^(docs|agentic)$'
AGENTIC='^(audit|audits|outcomes|to.?order|log|logs|playwright-report|test-results|coverage|reports?|qa[-_].*|QA[-_].*|deep-test-.*|probe-.*|.*-test-.*\.(mjs|cjs|js|ts)$|.*\.log$|^status\.json$|.*[-_]REPORT.*|RAPPORT.*|^report\.md$|^prd-.*\.json$|.*-audit.*\.(md|json)$|.*\.(jpe?g|png|gif|webp|mp4)$|^squirrel\.toml$|^maniac-.*)$'
DOCSLIKE='(\.md$|^Doc$|^knowledge$|^spec$|^specs$|^guide$|^guides$|^documentation$|\.pdf$)'

nkeep=0; ndocs=0; nag=0; nrev=0
shopt -s nullglob
for e in *; do
  [ -e "$e" ] || continue
  if   echo "$e" | grep -qE  "$CANON";   then echo "KEEP·canon   $e"; nkeep=$((nkeep+1)); continue; fi
  if   echo "$e" | grep -qE  "$CODE";    then echo "KEEP·code    $e"; nkeep=$((nkeep+1)); continue; fi
  if   echo "$e" | grep -qE  "$CONFIG";  then echo "KEEP·config  $e"; nkeep=$((nkeep+1)); continue; fi
  if   echo "$e" | grep -qE  "$HOUSE";   then echo "KEEP·house   $e"; nkeep=$((nkeep+1)); continue; fi
  if   echo "$e" | grep -qiE "$AGENTIC"; then
        flag=""
        if [ "$STALE" -gt 0 ]; then
          mt=$(find "$e" -type f -printf '%T@\n' 2>/dev/null | sort -rn | head -1 | cut -d. -f1)
          [ -z "$mt" ] && mt=$(stat -c %Y "$e" 2>/dev/null || echo "$now")
          age=$(( (now - mt) / 86400 )); [ "$age" -gt "$STALE" ] && flag="   [dormant ${age}j → suppr?]"
        fi
        echo "→agentic/    $e$flag"; nag=$((nag+1)); continue
  fi
  if   echo "$e" | grep -qiE "$DOCSLIKE"; then echo "→docs/       $e"; ndocs=$((ndocs+1)); continue; fi
  echo "??REVIEW     $e"; nrev=$((nrev+1))
done
echo "------------------------------------------"
echo "Projet: $P"
echo "KEEP=$nkeep   →docs/=$ndocs   →agentic/=$nag   ??REVIEW=$nrev"
[ "$STALE" -gt 0 ] && echo "(seuil dormant: ${STALE}j)"

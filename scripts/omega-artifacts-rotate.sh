#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════
# Rotation des artefacts auto-generes (fix de la cause du "bordel").
#
# Les rapports quotidiens (changelog-adopt, ecosystem-watch, growth-engine, ...)
# ecrivent un fichier par jour dans ~/.omega/artifacts (-> ~/Station/Artifacts)
# et ne nettoient jamais. Ce script garde les KEEP derniers de chaque famille et
# deplace le reste dans _archive/ (jamais supprime, juste range).
#
# Lance par cron chaque matin AVANT les crons de rapport. Idempotent.
# ═══════════════════════════════════════════════════════════════════════════
set -euo pipefail
OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
DIR="${1:-$OMEGA_DIR/artifacts}"   # chemin doctrine (peut etre un symlink)
KEEP="${KEEP:-2}"
cd "$DIR" 2>/dev/null || exit 0
mkdir -p _archive

# Familles a rotationner (prefixe suivi d'une date). Ajouter ici toute nouvelle
# famille de rapport quotidien.
PREFIXES="changelog-adopt ecosystem-watch growth-engine marketing-actions"

moved=0
for prefix in $PREFIXES; do
  mapfile -t files < <(ls -1 ${prefix}-*.html 2>/dev/null | sort)
  n=${#files[@]}
  [ "$n" -gt "$KEEP" ] || continue
  for f in "${files[@]:0:$((n-KEEP))}"; do
    mv "$f" _archive/ && moved=$((moved+1))
  done
done

[ "$moved" -gt 0 ] && echo "$(date -Iseconds) rotate: $moved artefact(s) archive(s)" >> "$HOME/.omega/logs/artifacts-rotate.log" || true
exit 0

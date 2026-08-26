#!/usr/bin/env bash
# Purge les artefacts de build REBUILDABLES dans Station : target/ Rust, node_modules, .next.
# NE touche PAS au code source ni aux binaires déjà installés (~/.local/bin/omega etc.).
# Garde-fou anti-build. Usage: clean-builds.sh [base_dir]  (défaut: dossier courant)
set -uo pipefail
BASE="${1:-$PWD}"
if pgrep -x cargo >/dev/null 2>&1 || pgrep -x rustc >/dev/null 2>&1; then
  echo "⛔ build Rust actif — annulé."; exit 1
fi
before=$(df -B1 / | awk 'NR==2{print $4}')

echo "== target/ Rust (à côté d'un Cargo.toml) =="
find "$BASE" -name Cargo.toml -not -path '*/node_modules/*' -not -path '*/target/*' 2>/dev/null | while read -r c; do
  t="$(dirname "$c")/target"; [ -d "$t" ] && { rm -rf "$t" && echo "  $t"; }
done
echo "== node_modules =="
find "$BASE" -type d -name node_modules -prune -exec rm -rf {} + 2>/dev/null && echo "  ok"
echo "== .next =="
find "$BASE" -type d -name .next -prune -exec rm -rf {} + 2>/dev/null && echo "  ok"

after=$(df -B1 / | awk 'NR==2{print $4}')
echo "== libéré ~$(( (after-before)/1024/1024/1024 )) Go =="
# Vérif que le CLI omega installé répond toujours (preuve non-régression)
v=$(omega --version 2>/dev/null || echo '?')
echo "omega CLI: $v (inchangé attendu)"

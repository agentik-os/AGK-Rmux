#!/usr/bin/env bash
# Analyse disque (lecture seule).
set -uo pipefail
HERE="$(cd "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")" && pwd)"
echo "==== DISQUE ===="
df -h / | awk 'NR==2{print "Utilisé "$3" / "$2" ("$5") — dispo "$4}'
echo; echo "==== RACINE (top) ===="
du -hx --max-depth=1 / 2>/dev/null | sort -rh | head -8
echo; echo "==== /home par user ===="
du -shx /home/* 2>/dev/null | sort -rh
echo; echo "==== /tmp (top 12) ===="
du -shx /tmp/* 2>/dev/null | sort -rh | head -12
echo; echo "==== DOCKER ===="
docker system df 2>/dev/null
echo; echo "==== ÉTAT SESSIONS ===="
"$HERE/idle-check.sh"

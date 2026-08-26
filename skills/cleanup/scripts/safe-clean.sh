#!/usr/bin/env bash
# Purge AUTO-SÛRE : cache build Docker + cache APT + vacuum journal.
# Ne touche NI Station, NI doc, NI images/volumes Docker. Garde-fou repos obligatoire.
set -uo pipefail
HERE="$(cd "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")" && pwd)"

if ! "$HERE/idle-check.sh" >/dev/null 2>&1; then
  echo "⛔ Travail actif détecté — purge annulée."
  "$HERE/idle-check.sh"
  exit 1
fi

before=$(df -B1 / | awk 'NR==2{print $4}')
echo "== Cache build Docker =="; docker builder prune -f 2>&1 | tail -1
echo "== Cache APT =="; apt-get clean && echo "ok"
echo "== Vacuum journal (100M) =="; journalctl --vacuum-size=100M 2>&1 | tail -1
after=$(df -B1 / | awk 'NR==2{print $4}')
freed=$(( (after - before) / 1024 / 1024 ))
echo "== Espace libéré : ${freed} Mo =="
df -h / | awk 'NR==2{print "Dispo maintenant : "$4" ("$5" utilisé)"}'

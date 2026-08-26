#!/usr/bin/env bash
# Résidu /tmp de builds/repros/audits. Usage: tmp-residue.sh list|purge
# Exclut TOUT ce qui est actif (fichiers ouverts), les sessions, et applique le garde-fou repos.
set -uo pipefail
HERE="$(cd "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")" && pwd)"
MODE="${1:-list}"

# Motifs de noms = résidu de build/test OmegaOS (jamais les sessions)
NAME_RE='omega|repro|verify|probe|mutant|audit-|ratrepro|ffcba|bright|fresh-install|stale|-target'
# Jamais toucher ça
PROTECT_RE='claude-|rmux|tmux|rx-socketna|cloudflared|\.log$|-cwd$|\.conf$|\.sock'

# Ensemble des top-dirs /tmp actuellement OUVERTS par un process (donc actifs)
active_set() {
  { lsof 2>/dev/null | awk '$NF ~ "^/tmp/"{print $NF}'
    for p in $(pgrep -x -d' ' rustc; pgrep -x -d' ' cargo) ; do readlink "/proc/$p/cwd" 2>/dev/null; done
  } | sed -E 's#(/tmp/[^/]+).*#\1#' | sort -u
}

mapfile -t ACTIVE < <(active_set)
is_active() { local d="$1"; for a in "${ACTIVE[@]}"; do [ "$d" = "$a" ] && return 0; done; return 1; }

candidates() {
  for d in /tmp/*; do
    [ -d "$d" ] || continue
    base=$(basename "$d")
    echo "$base" | grep -qiE "$PROTECT_RE" && continue
    echo "$base" | grep -qiE "$NAME_RE" || continue
    is_active "$d" && continue
    echo "$d"
  done
}

is_git() { [ -e "$1/.git" ]; }
# Propre = AUCUNE modif/untracked ET HEAD présent sur un remote (poussé).
# IMPORTANT: tourne git en tant que PROPRIÉTAIRE du dossier — en root, git refuse
# les repos d'un autre user ("dubious ownership") et renverrait un faux "propre".
git_clean() {
  local d="$1" own; own=$(stat -c%U "$d" 2>/dev/null)
  [ -z "$own" ] && return 1
  [ -n "$(sudo -u "$own" git -C "$d" status --porcelain 2>/dev/null)" ] && return 1
  [ -n "$(sudo -u "$own" git -C "$d" branch -r --contains HEAD 2>/dev/null)" ] || return 1
  return 0
}

if [ "$MODE" = "list" ]; then
  echo "== Candidats /tmp (résidu, inactifs) =="
  total=0; gtotal=0
  while read -r d; do
    [ -z "$d" ] && continue
    k=$(du -sx -B1 "$d" 2>/dev/null | cut -f1)
    if is_git "$d"; then
      gtotal=$((gtotal+k))
      printf "  %6s  %s  ⚠️ git (code — vérifier avant suppr.)\n" "$(du -shx "$d" 2>/dev/null|cut -f1)" "$d"
    else
      total=$((total+k))
      printf "  %6s  %s\n" "$(du -shx "$d" 2>/dev/null|cut -f1)" "$d"
    fi
  done < <(candidates)
  printf "  ----\n  Pur résidu (sans git) : %s Mo\n" "$((total/1024/1024))"
  printf "  Dossiers git à vérifier : %s Mo\n" "$((gtotal/1024/1024))"
  [ ${#ACTIVE[@]} -gt 0 ] && { echo "  (exclus car actifs : ${ACTIVE[*]})"; }
  exit 0
fi

if [ "$MODE" = "purge" ]; then
  if ! "$HERE/idle-check.sh" >/dev/null 2>&1; then
    echo "⛔ Travail actif — purge /tmp annulée."; "$HERE/idle-check.sh"; exit 1
  fi
  n=0; skipped=0
  while read -r d; do
    [ -z "$d" ] && continue
    if is_git "$d"; then
      if git_clean "$d"; then rm -rf "$d" && { echo "supprimé (git propre) $d"; n=$((n+1)); }
      else echo "⏭️  GARDÉ (git avec travail non sauvé) : $d"; skipped=$((skipped+1)); fi
      continue
    fi
    rm -rf "$d" && { echo "supprimé $d"; n=$((n+1)); }
  done < <(candidates)
  echo "== $n supprimé(s), $skipped dossier(s) git gardé(s) (à traiter à part) =="
  exit 0
fi
echo "usage: tmp-residue.sh list|purge"; exit 1

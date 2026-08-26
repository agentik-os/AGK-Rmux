#!/usr/bin/env bash
# blueprint-diff.sh — comparer deux blueprints, ou tous ceux d'un dossier.
#
# Pourquoi : à un blueprint, la question ne se pose pas. À cinq, la famille d'OS
# partage forcément du socle (auth, RGPD, audit, facturation, recherche, email) et
# chaque nouveau projet le reconstruit sans le savoir. Ce script rend le partage
# visible, pour qu'il devienne une brique commune au lieu d'un cinquième copier-coller.
#
# Il compare ce qui est comparable mécaniquement : les primitives, les tables du
# schéma, les verdicts de parité. Pas le positionnement, pas le GTM : ceux-là DOIVENT
# diverger, et deux OS qui se positionnent pareil ont un problème que ce script ne
# saurait pas nommer.
#
# Usage:
#   blueprint-diff.sh <blueprintA> <blueprintB>   compare deux blueprints
#   blueprint-diff.sh --all <dossier-blueprints>  matrice de tout le portefeuille
set -uo pipefail

bold(){ printf '\033[1m%s\033[0m\n' "$*"; }
dim(){  printf '\033[2m%s\033[0m\n' "$*"; }

tables_of(){ # $1 = dossier blueprint
  local s="$1/09-data/schema.ts"
  [[ -f "$s" ]] || return 0
  grep -oE '^  [a-zA-Z_][a-zA-Z0-9_]*: defineTable' "$s" | sed 's/^  //; s/: defineTable//' | sort -u
}
prim_of(){
  local j="$1/blueprint.json"
  [[ -f "$j" ]] || { echo "?"; return; }
  python3 -c "import json;print(json.load(open('$j')).get('primitive') or '?')" 2>/dev/null || echo "?"
}
# Les capacités CONSTRUIRE ou ACHETER : c'est le travail réel. DIFFÉRER et REFUSER
# ne coûtent rien, donc les partager n'économise rien.
built_of(){
  local f
  f="$(find "$1/05-parite" -type f -name '*.md' ! -name README.md 2>/dev/null | head -1)"
  [[ -n "$f" ]] || return 0
  grep -oE '^\| [A-H][0-9]+ \| [^|]+\| \*\*(CONSTRUIRE|ACHETER)\*\*' "$f" \
    | sed -E 's/^\| ([A-H][0-9]+) \| ([^|]+)\|.*/\1 \2/' | sed 's/[[:space:]]*$//' | sort -u
}

pair(){ # $1 $2 = dossiers
  local A="$1" B="$2" na nb
  na="$(basename "$A")"; nb="$(basename "$B")"
  bold "═══ $na  ⇄  $nb ═══"

  echo
  bold "Primitives"
  printf '  %-14s %s\n' "$na" "$(prim_of "$A")"
  printf '  %-14s %s\n' "$nb" "$(prim_of "$B")"
  if [[ "$(prim_of "$A")" == "$(prim_of "$B")" ]]; then
    printf '  \033[31m! MÊME PRIMITIVE\033[0m — deux OS sur le même objet central sont probablement un seul produit\n'
  fi

  echo
  bold "Tables du schéma"
  local ta tb common onlya onlyb
  ta="$(tables_of "$A")"; tb="$(tables_of "$B")"
  common="$(comm -12 <(printf '%s\n' "$ta") <(printf '%s\n' "$tb"))"
  onlya="$(comm -23 <(printf '%s\n' "$ta") <(printf '%s\n' "$tb"))"
  onlyb="$(comm -13 <(printf '%s\n' "$ta") <(printf '%s\n' "$tb"))"
  local nc; nc="$(printf '%s\n' "$common" | grep -c . || true)"
  printf '  communes (%s): %s\n' "$nc" "$(printf '%s' "$common" | tr '\n' ' ')"
  printf '  %s seul   : %s\n' "$na" "$(printf '%s' "$onlya" | tr '\n' ' ')"
  printf '  %s seul   : %s\n' "$nb" "$(printf '%s' "$onlyb" | tr '\n' ' ')"
  if [[ "$nc" -ge 3 ]]; then
    dim "  → $nc tables communes. entries et syntheses sont attendues partout (doctrine); au-delà, il y a une brique à factoriser."
  fi

  echo
  bold "Parité à construire ou acheter"
  local ba bb sh
  ba="$(built_of "$A")"; bb="$(built_of "$B")"
  sh="$(comm -12 <(printf '%s\n' "$ba" | cut -d' ' -f1) <(printf '%s\n' "$bb" | cut -d' ' -f1))"
  local ns nta ntb
  ns="$(printf '%s\n' "$sh" | grep -c . || true)"
  nta="$(printf '%s\n' "$ba" | grep -c . || true)"
  ntb="$(printf '%s\n' "$bb" | grep -c . || true)"
  if [[ "$nta" -eq 0 || "$ntb" -eq 0 ]]; then
    dim "  matrice illisible sur l'un des deux (format de tableau attendu: | A1 | Capacité | **VERDICT** |)"
  else
    printf '  %s en construit/achète %s · %s en construit/achète %s\n' "$na" "$nta" "$nb" "$ntb"
    printf '  \033[1mpartagées: %s\033[0m' "$ns"
    if [[ "$nta" -gt 0 ]]; then printf ' (%d%% du travail de %s)' $(( ns * 100 / nta )) "$na"; fi
    printf '\n'
    if [[ "$ns" -gt 0 ]]; then
      printf '%s\n' "$sh" | while read -r id; do
        [[ -z "$id" ]] && continue
        printf '    %s %s\n' "$id" "$(printf '%s\n' "$ba" | grep "^$id " | cut -d' ' -f2- | head -1)"
      done
    fi
    if [[ "$ns" -ge 8 ]]; then
      echo
      printf '  \033[33m→ %s capacités de socle construites deux fois.\033[0m C'"'"'est une brique commune, pas une coïncidence.\n' "$ns"
    fi
  fi
  echo
}

if [[ "${1:-}" == "--all" ]]; then
  DIR="${2:-}"
  [[ -d "$DIR" ]] || { echo "usage: blueprint-diff.sh --all <dossier-blueprints>" >&2; exit 2; }
  mapfile -t BPS < <(find "$DIR" -maxdepth 1 -mindepth 1 -type d ! -name '_TEMPLATE' | sort)
  if [[ "${#BPS[@]}" -lt 2 ]]; then
    echo "un seul blueprint dans $DIR — rien à comparer. Le partage devient visible à partir de deux."
    exit 0
  fi
  bold "═══ portefeuille : ${#BPS[@]} blueprints ═══"
  echo
  for b in "${BPS[@]}"; do
    printf '  %-16s %-34s %s tables\n' "$(basename "$b")" "$(prim_of "$b")" "$(tables_of "$b" | grep -c . || true)"
  done
  echo
  for ((i=0;i<${#BPS[@]};i++)); do
    for ((j=i+1;j<${#BPS[@]};j++)); do
      pair "${BPS[$i]}" "${BPS[$j]}"
    done
  done
  exit 0
fi

A="${1:-}"; B="${2:-}"
if [[ ! -d "$A" || ! -d "$B" ]]; then
  echo "usage: blueprint-diff.sh <blueprintA> <blueprintB>" >&2
  echo "       blueprint-diff.sh --all <dossier-blueprints>" >&2
  exit 2
fi
pair "$A" "$B"

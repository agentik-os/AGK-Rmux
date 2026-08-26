#!/usr/bin/env bash
# convex-validate.sh — faire valider un schema.ts par CONVEX lui-même, pas par l'oeil.
#
# Pourquoi : `tsc` sur l'app scaffoldée ne voit rien tant que `convex/_generated`
# n'existe pas, donc tout est `any` et un schéma cassé passe. Et `npx convex codegen`
# exige un déploiement, donc un compte, donc un humain. Entre les deux il y a une
# troisième voie : typechecker le schéma SEUL contre le package `convex`, dont les
# types génériques contraignent réellement les index aux champs du document.
#
# Ce que ça attrape, prouvé en test :
#   - un index sur un champ qui n'existe pas         → TS2769
#   - un validator inventé (v.numbre)                → TS2551
#   - toute erreur de forme de defineTable / defineSchema
#
# Ce que ça n'attrape PAS : les règles sémantiques de Convex qui ne sont pas dans les
# types (limites de taille, index redondants). blueprint-check.sh couvre celles-là.
#
# Usage:  convex-validate.sh <chemin-vers-schema.ts> [--quiet]
# Sortie: 0 = valide · 1 = invalide · 2 = impossible de valider (deps absentes)
set -uo pipefail

SCHEMA="${1:-}"; QUIET=0
[[ "${2:-}" == "--quiet" ]] && QUIET=1

c_info(){ [[ $QUIET -eq 1 ]] || printf '\033[36m[convex]\033[0m %s\n' "$*"; }
c_ok(){   [[ $QUIET -eq 1 ]] || printf '\033[32m[convex]\033[0m %s\n' "$*"; }
c_no(){   printf '\033[31m[convex]\033[0m %s\n' "$*" >&2; }

if [[ -z "$SCHEMA" || ! -f "$SCHEMA" ]]; then
  echo "usage: convex-validate.sh <schema.ts> [--quiet]" >&2; exit 2
fi
command -v node >/dev/null 2>&1 || { c_no "node absent — validation impossible"; exit 2; }
command -v npx  >/dev/null 2>&1 || { c_no "npx absent — validation impossible"; exit 2; }

OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
SANDBOX="$OMEGA_DIR/cache/convex-validate"

# Le bac à sable est monté une fois puis réutilisé : sans lui chaque validation
# paierait un npm install complet.
if [[ ! -d "$SANDBOX/node_modules/convex" ]]; then
  c_info "premier lancement, montage du bac à sable (npm install convex)"
  mkdir -p "$SANDBOX/convex"
  cat > "$SANDBOX/package.json" <<'JSON'
{ "name": "convex-validate-sandbox", "private": true, "type": "module" }
JSON
  cat > "$SANDBOX/tsconfig.json" <<'JSON'
{
  "compilerOptions": {
    "target": "ESNext", "module": "ESNext", "moduleResolution": "bundler",
    "strict": true, "noEmit": true, "skipLibCheck": true, "types": []
  },
  "include": ["convex/schema.ts"]
}
JSON
  ( cd "$SANDBOX" && npm install convex typescript --silent >/dev/null 2>&1 ) \
    || { c_no "npm install a échoué — pas de réseau ? validation impossible"; exit 2; }
  c_ok "bac à sable prêt (réutilisé ensuite)"
fi

cp -f "$SCHEMA" "$SANDBOX/convex/schema.ts"
OUT="$( cd "$SANDBOX" && npx tsc --noEmit 2>&1 )"

if [[ -z "$OUT" ]]; then
  NT="$(grep -cE '^  [a-zA-Z_][a-zA-Z0-9_]*: defineTable' "$SCHEMA" || true)"
  c_ok "schéma valide pour Convex ($NT tables, index et validators vérifiés par les types)"
  exit 0
fi

# Les erreurs de types génériques de Convex sont illisibles brutes (des pages de
# ExtractFieldPaths). On garde la ligne, le code, et on traduit le cas fréquent.
c_no "schéma REFUSÉ par les types Convex :"
printf '%s\n' "$OUT" | grep -E '^convex/schema\.ts\([0-9]+,' | while IFS= read -r line; do
  LN="$(printf '%s' "$line" | sed -E 's/^convex\/schema\.ts\(([0-9]+),.*/\1/')"
  SRC="$(sed -n "${LN}p" "$SCHEMA" 2>/dev/null | sed 's/^[[:space:]]*//')"
  case "$line" in
    *TS2769*|*"is not assignable to parameter of type"*)
      printf '  \033[31m✗\033[0m ligne %s — index invalide : un champ liste ne correspond a aucun champ de la table\n' "$LN"
      printf '      %s\n' "$SRC" ;;
    *TS2551*|*"does not exist on type"*)
      HINT="$(printf '%s' "$line" | grep -oE "Did you mean '[^']+'" || true)"
      printf '  \033[31m✗\033[0m ligne %s — validator inconnu. %s\n' "$LN" "$HINT"
      printf '      %s\n' "$SRC" ;;
    *)
      printf '  \033[31m✗\033[0m ligne %s — %s\n' "$LN" "$(printf '%s' "$line" | cut -c1-140)"
      printf '      %s\n' "$SRC" ;;
  esac
done
exit 1

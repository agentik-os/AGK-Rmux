#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════
# omega-open : ouvrir un projet dans la session de ton choix (Claude, Codex, ...)
#
# Le "step intermediaire" : il liste tes projets groupes par type, tu choisis le
# projet, puis l'agent, et il lance la session rmux au bon endroit.
#
#   omega-open                      menu interactif (projet -> agent)
#   omega-open verba                menu agent seulement (projet deja choisi)
#   omega-open verba codex          direct, zero question
#   omega-open --artifacts          ouvre le dossier des artefacts + l'URL tailnet
#
# Sous le capot : omega new <session> --agent <agent> --dir <projet>
# ═══════════════════════════════════════════════════════════════════════════
set -uo pipefail
OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
REG="$OMEGA_DIR/projects.json"
# Chemin doctrine (R-ARTIFACT). Peut etre un symlink vers ~/Station/Artifacts.
ART="$OMEGA_DIR/artifacts"
# URL tailnet derivee a l'execution : jamais un hostname en dur (portable).
ART_URL="$(tailscale serve status 2>/dev/null | grep -oE 'https://[^ ]+:8443' | head -1)/"
[ "$ART_URL" = "/" ] && ART_URL=""

b=$'\e[1m'; dim=$'\e[2m'; gold=$'\e[33m'; grn=$'\e[32m'; r=$'\e[0m'

[ -f "$REG" ] || { echo "registre introuvable: $REG" >&2; exit 1; }

# ── raccourci artefacts ────────────────────────────────────────────────────
if [ "${1:-}" = "--artifacts" ] || [ "${1:-}" = "-a" ]; then
  echo "${b}Artefacts${r}  ${dim}$ART${r}"
  [ -n "$ART_URL" ] && echo "${gold}$ART_URL${r}  (tailnet, Tailscale requis)"
  ls -1t "$ART"/*.html 2>/dev/null | head -12 | while read -r f; do
    echo "  ${dim}·${r} $(basename "$f")"
  done
  exit 0
fi

# ── charger les projets (name|category|path), tries ────────────────────────
mapfile -t ROWS < <(python3 - "$REG" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
ORDER = {"Partners":0, "SideBusiness":1, "Achats":2, "Nova":3, "Marketing":4, "Autres":9}
def cat(p):
    path = p["path"]
    if "/Station/" not in path: return "Autres"
    return path.split("/Station/")[1].split("/")[0]
ps = [p for p in d.get("projects", []) if p.get("path")]
ps.sort(key=lambda p: (ORDER.get(cat(p), 8), p["name"].lower()))
for p in ps:
    print(f'{p["name"]}|{cat(p)}|{p["path"]}')
PY
)
[ "${#ROWS[@]}" -gt 0 ] || { echo "aucun projet dans le registre" >&2; exit 1; }

PICK="${1:-}"
IDX=-1

# projet passe en argument ?
if [ -n "$PICK" ]; then
  for i in "${!ROWS[@]}"; do
    n="${ROWS[$i]%%|*}"
    [ "${n,,}" = "${PICK,,}" ] && IDX=$i && break
  done
  [ "$IDX" -ge 0 ] || { echo "projet inconnu: $PICK" >&2; echo "lance 'omega-open' sans argument pour voir la liste." >&2; exit 1; }
fi

# ── etape 1 : choisir le projet ────────────────────────────────────────────
if [ "$IDX" -lt 0 ]; then
  echo ""
  echo "  ${b}Quel projet ?${r}"
  last=""
  for i in "${!ROWS[@]}"; do
    IFS='|' read -r name cat path <<< "${ROWS[$i]}"
    if [ "$cat" != "$last" ]; then echo ""; echo "  ${gold}── $cat ──${r}"; last="$cat"; fi
    printf "   ${b}%2d${r}  %s\n" "$((i+1))" "$name"
  done
  echo ""
  printf "   ${b} a${r}  ${dim}artefacts (rapports, boards)${r}\n"
  echo ""
  read -rp "  numero > " sel
  [ "$sel" = "a" ] && exec "$0" --artifacts
  [[ "$sel" =~ ^[0-9]+$ ]] && [ "$sel" -ge 1 ] && [ "$sel" -le "${#ROWS[@]}" ] || { echo "  choix invalide."; exit 1; }
  IDX=$((sel-1))
fi

IFS='|' read -r NAME CAT DIR <<< "${ROWS[$IDX]}"
[ -d "$DIR" ] || { echo "dossier introuvable: $DIR" >&2; exit 1; }

# ── etape 2 : choisir l'agent ──────────────────────────────────────────────
AGENT="${2:-}"
if [ -z "$AGENT" ]; then
  echo ""
  echo "  ${b}$NAME${r} ${dim}($CAT)${r}"
  echo "  ${dim}$DIR${r}"
  echo ""
  echo "  ${b}Quel agent ?${r}"
  echo "   ${b}1${r}  codex    ${dim}Codex (OpenAI / Sol), le defaut (meilleur pour coder)${r}"
  echo "   ${b}2${r}  claude   ${dim}Claude Code (raisonnement lourd)${r}"
  echo "   ${b}3${r}  glm      ${dim}GLM (Z.AI), worker code${r}"
  echo "   ${b}4${r}  gemini   ${dim}Gemini (Google)${r}"
  echo "   ${b}5${r}  pi       ${dim}Pi${r}"
  echo "   ${b}6${r}  shell    ${dim}shell nu${r}"
  echo ""
  read -rp "  numero [1] > " a
  case "${a:-1}" in
    1|"") AGENT=codex ;; 2) AGENT=claude ;; 3) AGENT=glm ;;
    4) AGENT=gemini ;;   5) AGENT=pi ;;    6) AGENT=shell ;;
    *) echo "  choix invalide."; exit 1 ;;
  esac
fi

SESSION="${NAME,,}"
echo ""
echo "  ${grn}→${r} omega new ${b}$SESSION${r} --agent ${b}$AGENT${r} --dir $DIR"
echo ""
exec omega new "$SESSION" --agent "$AGENT" --dir "$DIR"

#!/usr/bin/env bash
# Garde-fou "repos" PORTABLE : exit 0 si aucun build actif pour l'utilisateur courant, 1 sinon.
# Aucune dependance externe (pas de script VPS). Utilise par les autres scripts du skill.
set -uo pipefail
U=$(id -un)
busy=""
for pat in rustc cargo cloudflared; do
  pgrep -u "$U" -x "$pat" >/dev/null 2>&1 && busy+=" $pat"
done
pgrep -u "$U" -f 'next build|vite build|webpack|turbo run build' >/dev/null 2>&1 && busy+=" jsbuild"
if [ -n "$busy" ]; then echo "BUSY:$busy"; exit 1; fi
echo "IDLE"; exit 0

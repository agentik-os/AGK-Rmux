#!/usr/bin/env bash
# Purge les caches rebuildables des users de dev (XDG ~/Linux/cache).
# Ne touche NI Station, NI data/state, NI config. Tout se reremplit à l'usage.
set -uo pipefail
for u in "$(id -un)"; do
  home=$(getent passwd "$u" | cut -d: -f6)
  cache="$home/Linux/cache"
  [ -d "$cache" ] || continue
  echo "== $u : $cache =="
  for sub in .bun go-build npm/_cacache npm/_npx node-gyp .cache/puppeteer ms-playwright; do
    p="$cache/$sub"
    [ -e "$p" ] || continue
    sz=$(du -shx "$p" 2>/dev/null | cut -f1)
    rm -rf "$p" && echo "  purgé $sub ($sz)"
  done
done

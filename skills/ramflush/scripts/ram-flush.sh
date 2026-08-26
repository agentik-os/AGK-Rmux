#!/usr/bin/env bash
# Purge RAM/cache noyau + rapport perf. Sans risque (caches réclamables).
# drop_caches exige root → lancer via sudo. Portable.
set -uo pipefail

echo "== RAM AVANT =="
free -h | awk '/Mem:/{print "  used "$3" | free "$4" | buff/cache "$6" | dispo "$7} /Swap:/{print "  swap: used "$3" / "$2}'

sync
if [ -w /proc/sys/vm/drop_caches ]; then
  echo 3 > /proc/sys/vm/drop_caches
  echo 1 > /proc/sys/vm/compact_memory 2>/dev/null || true
  echo "  ✅ caches purgés (pagecache + dentries + inodes) + mémoire compactée"
else
  echo "  ⚠️ root requis (sudo) pour drop_caches — purge non effectuée"
fi

echo "== RAM APRÈS =="
free -h | awk '/Mem:/{print "  used "$3" | free "$4" | buff/cache "$6" | dispo "$7}'

echo "== TOP 5 RAM =="
ps -eo comm,rss --sort=-rss 2>/dev/null | awk 'NR>1 && NR<=6{printf "  %-22s %dM\n",$1,$2/1024}'

echo "== TOP 5 CPU =="
ps -eo comm,pcpu --sort=-pcpu 2>/dev/null | awk 'NR>1 && NR<=6{printf "  %-22s %s%%\n",$1,$2}'

echo "== LOAD =="
uptime | sed -E 's/.*(load average.*)/  \1/'

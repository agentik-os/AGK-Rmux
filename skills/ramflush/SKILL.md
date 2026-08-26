---
name: ramflush
description: >
  Purge la RAM et fait le point perf du VPS — `sync` + drop des caches noyau
  (pagecache/dentries/inodes), compaction mémoire, puis rapport avant/après + top
  consommateurs RAM/CPU + load average. À lancer quand le VPS est sous charge / rame.
  Sans risque (les caches sont reconstruits à la demande) ; nécessite root (drop_caches).
argument-hint: ""
allowed-tools: ["Bash"]
domain: maintenance
read_only: false
triggers: ["ramflush", "purge ram", "free ram", "vider la ram", "nettoyer la ram", "ram cleanup", "flush cache", "perf cleanup", "libérer la ram"]
---

# ramflush — purge RAM + point perf

Libère la mémoire mise en cache par le noyau et donne un état perf rapide. Idéal quand le VPS est sous forte charge.

## Quand
- Le VPS rame / `free` montre énormément de `buff/cache` / la latence monte.
- Avant de lancer une grosse charge (build, batch) pour repartir propre.

## Ce que ça fait (et ne fait pas)
- `sync` (flush des écritures en attente) puis `echo 3 > /proc/sys/vm/drop_caches` → libère pagecache + dentries + inodes (réclamables de toute façon).
- `compact_memory` (best-effort) pour défragmenter la mémoire.
- Rapport : RAM avant/après, top 5 RAM, top 5 CPU, load average.
- **Ne tue aucun process**, ne touche pas au swap par défaut, n'impacte pas les données. Le seul coût : les caches se reremplissent à l'usage (léger ralentissement transitoire).

## Lancer
- `sudo bash scripts/ram-flush.sh`  (drop_caches exige root)
- Depuis Telegram : menu **🧹 Clean → 💾 Purge RAM** (atlas / @AgentikMonitorBot).

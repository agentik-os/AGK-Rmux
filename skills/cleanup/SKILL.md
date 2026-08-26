---
name: cleanup
description: >
  Nettoyage intelligent du VPS/projets — analyse disque, fermeture des sessions de travail
  détachées (protège la connexion + l'infra), purge sûre (cache Docker/APT/journal, résidu /tmp,
  caches Bun/npm/go), purge des artefacts de build rebuildables (target/ Rust, node_modules, .next)
  avec garde-fou anti-session-active, et audit de cohérence de la documentation des projets.
argument-hint: "[analyze|safe|tmp|caches|builds]   (optionnel → interactif)"
allowed-tools: ["Read", "Write", "Edit", "Bash", "Glob", "Grep", "AskUserQuestion"]
domain: maintenance
read_only: false
triggers: ["cleanup", "nettoyage", "menage disque", "ménage disque", "clean", "purge cache", "purge tmp", "free disk", "disk space", "espace disque"]
---

# Cleanup — ménage VPS Agentik

Objectif : libérer de l'espace et garder une doc cohérente, **sans jamais casser un travail en cours ni supprimer du contenu produit ou du canon projet**.

## Règle d'or — JAMAIS de suppression sans deux conditions
1. **Machine au repos** : lancer `scripts/idle-check.sh`. S'il renvoie `BUSY:…` → **STOP**, lister le travail actif, ne rien supprimer.
2. **Validation utilisateur** : pour tout ce qui touche `~/Station`, les caches d'un user, ou de la doc → présenter la liste (chemins + tailles + raison) et attendre l'accord. Les étapes "auto-sûres" (cache Docker/APT/journal) peuvent se faire sans valider chaque item, mais toujours après le garde-fou repos.

## RÈGLE ABSOLUE — jamais de kill sur claude / rmux (opérateur, 2026-08-11)

Pendant un cleanup, ne **JAMAIS** kill un process `claude` ni un daemon ou une
session rmux vivante — même détachée, même vieille, même à 0% CPU. "Détaché" ≠
"mort" : une session détachée porte des agents en plein travail, un claude à
faible CPU attend souvent une réponse API entre deux turns, et tuer le daemon
rmux tue TOUTES les sessions d'un coup. Incident fondateur : le 2026-08-11 un
cleanup RAM a tué des claude actifs pris pour idle, y compris sa propre
session. Fermer une session est un geste de l'**opérateur** (`omega kill
<name>`, `rmux kill-server`), jamais du cleanup — même avec un go générique
« clean up ». La récupération de RAM se limite à : workers de modèles idle
(TTS/STT), dev servers oubliés, stacks Docker prouvées idle (avec accord),
caches. `close-sessions.sh` porte ce garde-fou en dur et refuse tout pid
claude/rmux.

## Périmètre INTOUCHABLE (ne jamais supprimer)
- Tout `~/Station/**` **sauf** des artefacts de build explicitement rebuildables (`target/`, `.next/`, `dist/`) — et encore, seulement avec accord.
- Docs canon : `vision/**`, fichiers `*PRD*`, `*feature*`, `*step*`, `Vision/`, `OMEGA.md`, `CLAUDE.md`, `AGENTS.md`, `README*`, `CHANGELOG`, `LICENSE`, `SECURITY.md`, `~/.omega/rules/**`, les **fiches** projet.
- Tout dossier avec **beaucoup de `.md`** (>20) = probablement du **contenu produit** (cours, blog, data), PAS de la doc → protéger par défaut, demander.
- Sessions / sockets actifs : `claude-*`, `rmux-*`, `tmux-*`, `rx-socketna-*`, `cloudflared`, la worktree d'un build en cours.

## Déroulé
0. **Fermeture des tunnels/mosh détachés** (si demandé) : `scripts/close-sessions.sh` ferme les tunnels cloudflared + serveurs mosh détachés de `vibe`/`lab` — et RIEN d'autre : claude et rmux sont hors périmètre par la Règle absolue ci-dessus (le script refuse ces pids en dur). **Protège toujours** la connexion courante (root) et l'infra (Telegram bots, daemons sécurité, tailscale, sshd). Tester d'abord avec `--dry`.
1. **Garde-fou** : `scripts/idle-check.sh`. Si occupé → rapport + stop.
2. **Analyse** : `scripts/analyze.sh` → df, top du `/home` `/tmp` `/var`, `docker system df`.
3. **Purge auto-sûre** : `scripts/safe-clean.sh` → `docker builder prune`, `apt-get clean`, `journalctl --vacuum-size=100M`. (rien de Station, rien de doc.)
4. **Résidu /tmp** : `scripts/tmp-residue.sh list` → présenter candidats (dirs de build/repro/audit, exclut tout ce qui est ouvert/actif). Après accord : `tmp-residue.sh purge`.
5. **Caches user rebuildables** : `scripts/clean-caches.sh` → purge `~/Linux/cache` (Bun, go-build, npm `_cacache`/`_npx`, node-gyp, playwright) pour `vibe`/`lab`. Ne touche ni config, ni data/state, ni Station.
6. **Artefacts de build dans Station** (sur demande) : `scripts/clean-builds.sh [base]` → purge `target/` Rust (à côté d'un `Cargo.toml`), `node_modules`, `.next`. **Rebuildables** mais coût : prochain build/`install` recompile. Garde-fou anti-build + re-teste `omega --version` (preuve non-régression). NE touche pas au code source ni aux binaires installés.
7. **Audit doc** : `scripts/doc-audit.sh <projet>` → classe en PROTECTED / SCRATCH / REVIEW. Présenter la liste SCRATCH pour validation ; ne jamais auto-supprimer. Proposer ensuite la création de `PROGRESS.md` (recap canon) + complétion de `RULES.md`/fiche par projet.

## Notes
- Les images Docker et les volumes par-agent ne sont PAS du déchet par défaut (peuvent être réutilisés par OmegaOS) → ne pas purger sans confirmation ciblée.
- Toujours afficher l'espace AVANT/APRÈS.
- Si un doute → ne pas supprimer, demander.

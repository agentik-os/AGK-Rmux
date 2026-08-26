#!/usr/bin/env bash
# Ferme les tunnels cloudflared + serveurs mosh détachés des users de dev (vibe, lab).
# PROTÈGE toujours : la connexion courante (root), l'infra (Telegram bots,
# daemons sécurité agentik-*, tailscaled, sshd, systemd). Usage: close-sessions.sh [--dry]
#
# ═══ RÈGLE ABSOLUE (opérateur, 2026-08-11, après incident) ═════════════════════
# Ce script ne touche JAMAIS un process `claude` ni un daemon/session rmux.
# "Détaché" ≠ "mort" : une session rmux détachée porte des agents en plein
# travail, et tuer le daemon rmux tue TOUTES les sessions d'un coup — c'est
# exactement l'incident du 2026-08-11 (des claude actifs tués par un cleanup,
# y compris la session qui exécutait le cleanup). Fermer une session rmux est
# un geste de l'OPÉRATEUR (`omega kill <name>` / `rmux kill-server`), jamais
# d'un script de ménage. Le garde-fou ci-dessous refuse même un pid passé par
# erreur dont la commande correspond à claude/rmux.
# ═══════════════════════════════════════════════════════════════════════════════
set -uo pipefail
DRY="${1:-}"

forbidden() { # pid → 0 si le process est claude/rmux (interdit), 1 sinon
    local args
    args="$(ps -o comm=,args= -p "$1" 2>/dev/null)"
    case "$args" in
        *claude*|*rmux*) return 0 ;;
        *) return 1 ;;
    esac
}

act() {
    if forbidden "$1"; then
        echo "  🛑 REFUS pid $1 (claude/rmux — périmètre interdit au cleanup)"
        return
    fi
    if [ "$DRY" = "--dry" ]; then echo "  [dry] fermerait pid $*"; else kill "$@" 2>/dev/null && echo "  ✅ fermé pid $*"; fi
}

echo "== Tunnels cloudflared =="
for p in $(pgrep -x cloudflared 2>/dev/null); do act "$p"; done

echo "== Serveurs mosh détachés (vibe + lab ; root/connexion courante protégés) =="
for p in $(pgrep -x mosh-server -U vibe 2>/dev/null; pgrep -x mosh-server -U lab 2>/dev/null); do act "$p"; done

echo "(claude, rmux, root, infra bots/sécurité/tailscale/sshd : JAMAIS touchés)"

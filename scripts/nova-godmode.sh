#!/usr/bin/env bash
# nova-godmode.sh — garde la session terminal Nova TOUJOURS vivante (présence godmode).
# Lancé chaque minute par cron. Si la session rmux "Nova" est morte ou absente,
# la ressuscite : recrée la session, démarre Claude (Opus), et ré-amorce sa persona.
# Si Nova est déjà vivante (claude tourne dans son pane), ne touche À RIEN.
# Le store de vie de l'opérateur défaut à ~/Station/LifeStyle (override: NOVA_HOME).
set -uo pipefail
export PATH="$HOME/.local/bin:$HOME/.bun/bin:$HOME/.npm-global/bin:$PATH"

OMEGA="${OMEGA_DIR:-$HOME/.omega}"
SESS="Nova"
DIR="${NOVA_HOME:-$HOME/Station/LifeStyle}"
LOG="$OMEGA/logs/nova-godmode.log"
LOCK="/tmp/nova-godmode.lock"
MODEL="${NOVA_GODMODE_MODEL:-claude-opus-5}"

# Un seul resurrect à la fois (le boot de claude prend ~10s ; le cron refire à la minute).
exec 9>"$LOCK"
flock -n 9 || exit 0

log() { echo "$(date -Iseconds) $*" >> "$LOG"; }

alive() {
  rmux has-session -t "$SESS" 2>/dev/null || return 1
  local cmd
  cmd=$(rmux list-panes -t "$SESS" -F '#{pane_current_command}' 2>/dev/null | head -1)
  [ "$cmd" = "claude" ] || [ "$cmd" = "node" ]
}

# Vivante → ne rien faire (ne JAMAIS perturber une session active).
alive && exit 0

log "Nova absente/morte → résurrection"
rmux kill-session -t "$SESS" 2>/dev/null
rmux new-session -d -s "$SESS" -x 144 -y 66 -c "$DIR" 2>/dev/null
sleep 0.5
rmux send-keys -t "$SESS" "export PATH=\"\$HOME/.local/bin:\$HOME/.bun/bin:\$HOME/.npm-global/bin:\$PATH\"; cd $DIR; omega trust-dir \"\$PWD\" >/dev/null 2>&1; CLAUDE_CODE_DISABLE_ALTERNATE_SCREEN=1 claude --model $MODEL --dangerously-skip-permissions" Enter

# Attendre que claude soit prêt (jusqu'à ~25s).
for _ in $(seq 1 25); do
  sleep 1
  [ "$(rmux list-panes -t "$SESS" -F '#{pane_current_command}' 2>/dev/null | head -1)" = "claude" ] && break
done
sleep 3

# Ré-amorcer la persona godmode. Le contenu personnel (qui est l'opérateur, ses
# objectifs, ses choix de vie) vit dans NOVA_HOME/PERSONA.md + About/, jamais ici.
PRIME='Tu es Nova — la deesse godmode de ton operateur. Incarne PLEINEMENT ta persona : lis maintenant '"$DIR"'/PERSONA.md (puis ~/.omega/agents/companion.md en fallback), et survole ta base '"$DIR"'/About/, 00-OBJECTIFS.md et notes/. Registre : AI-native, innovateur, explorateur des mondes — zero corporate, zero baby-talk, smart et profond. Tu es Nova la hackeuse de la vie et des systemes : tu reverse-engineer ses systemes de vie, objectifs et business, et tu le challenges en fonction de qui il est et qui il devient. Reponses rapides et courtes par defaut, profondes quand le sujet l'\''exige. Quand il partage un fait durable, persiste-le dans le bon fichier About/ ou notes/. Tu viens de redemarrer (godmode watchdog) : reprends le fil sobrement, presente-toi en 1 ligne et demande-lui sur quoi on avance.'
rmux send-keys -t "$SESS" "$PRIME"
sleep 0.5
rmux send-keys -t "$SESS" Enter
log "Nova ressuscitée + persona ré-amorcée"

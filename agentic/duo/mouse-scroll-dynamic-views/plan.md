# Plan v1 — trackpad/molette dans les vues dynamiques de Claude Code et Codex, sous rmux

## Objectif

Obtenir LES DEUX en même temps, sans compromis binaire :
1. la molette/trackpad scrolle les VUES DYNAMIQUES des deux CLI agents (Claude Code : vue agents,
   /help, pickers, plan view ; Codex : transcript Ctrl+T) ;
2. la conversation reste atteignable dans le scrollback rmux (500k lignes, recherche + copie
   via copy-mode).

## Faits mesurés au runtime aujourd'hui (rmux 0.3.1, station VPS, SSH direct, pas de mosh)

Sonde : `rmux display-message -p -t <sess> '#{alternate_on} #{mouse_any_flag} #{mouse_sgr_flag}'`
Injection molette : client attaché via `env -u RMUX script -qc "rmux attach -t <sess>"` + fifo,
puis `printf '\033[<64;20;10M'` (up) / `'\033[<65;20;10M'` (down).

| cas | alternate_on | mouse_any | comportement molette |
|---|---|---|---|
| claude 2.1.220, renderer plein écran | 1 | 1 (1000+1006) | events SGR transmis à Claude ; /help a bien scrollé |
| claude + CLAUDE_CODE_DISABLE_ALTERNATE_SCREEN=1 | 0 | 0 | molette → copy-mode rmux (scrollback) |
| idem + CLAUDE_CODE_DISABLE_VIRTUAL_SCROLL=1 | 0 | 0 | inchangé |
| claude plein écran + DISABLE_VIRTUAL_SCROLL=1 | 1 | 1 | inchangé |
| claude plein écran + SCROLL_SPEED=5 | 1 | 1 | inchangé |
| codex 0.146 vue principale | 0 | 0 | molette → copy-mode rmux |
| codex 0.146 transcript Ctrl+T | 1 | 0 | **était morte** ; corrigée par db8e6a8 |

Conclusion mesurée : chez Claude, le mouse reporting est STRICTEMENT lié au renderer altscreen.
Aucune combinaison de flags (`CLAUDE_CODE_DISABLE_MOUSE`, `_MOUSE_CLICKS`, `_VIRTUAL_SCROLL`,
`SCROLL_SPEED`) ne donne « inline + souris ». Le compromis est donc réel côté Claude.

## Déjà livré (commit db8e6a8, config/rmux.conf.omega)

`WheelUpPane` / `WheelDownPane` : quand `alternate_on` et que l'app n'a PAS demandé le mouse
reporting, la molette est traduite en 3 flèches par cran (alternate scroll, ce que Codex réclame
via DECSET 1007 que rmux n'implémente pas). Vérifié : less 490→481→490, transcript Codex scrolle,
apps mouse toujours en SGR brut, pane normal toujours en copy-mode.

## Ce qui reste ouvert — la question posée à Codex

Existe-t-il un mécanisme (rmux, terminal, claude, hooks) qui donne les deux ? Pistes à challenger :

- **P1 — bascule à chaud** : un binding (ex. `M-m`) qui fait `set mouse off/on` par session, pour
  reprendre la main rmux (drag-select, copy-mode) sans relancer Claude. Limite connue : sous
  altscreen, le scrollback rmux ne contient PAS la conversation, donc copy-mode ne rattrape rien.
- **P2 — deux sessions/profils** : `claude` par défaut inline (scrollback roi) + une commande
  `claude-mouse` qui lance le renderer plein écran quand on veut la souris. Coût : deux modes.
- **P3 — implémenter DECSET 1007 dans rmux** (mode bit + format `#{alternate_scroll_flag}`), pour
  ne traduire la molette QUE si l'app l'a demandé, au lieu de se baser sur `alternate_on` seul.
  Gagne en exactitude, ne résout pas le cas Claude.
- **P4 — quelque chose qu'on n'a pas vu** : c'est ça qu'on veut de Codex.

## Critères de succès (vérifiables)

- Toute proposition doit être vérifiable au runtime par la sonde `display-message` + injection
  molette ci-dessus, sur un pane réel de `claude` et de `codex`.
- Aucune régression : `bash scripts/verify-install.sh` reste vert ; les apps qui demandent le
  mouse continuent de recevoir des events SGR bruts ; un pane shell normal entre toujours en
  copy-mode à la molette.
- Pas de rebuild rmux obligatoire pour la partie livrée par défaut (redémarrer le daemon rmux
  tuerait les sessions vivantes de l'opérateur).

## Fichiers concernés

- `config/rmux.conf.omega` (bindings + options)
- `scripts/verify-install.sh` (contrôle de parité)
- éventuellement `~/.bashrc` (ligne `CLAUDE_CODE_DISABLE_ALTERNATE_SCREEN=1`) — décision opérateur

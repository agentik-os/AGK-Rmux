# duo — le binome Claude ⇄ Codex

Claude reste le **stratège / arbitre**, Codex (`gpt-5.6-sol`) est le **coder**, avec
**bascule automatique sur Claude** quand le quota Codex est épuisé.

Deux pièces, une frontière nette :

- **`bin/omega-duo`** (Bun) — le *muscle* déterministe. Une seule responsabilité :
  exécuter une tâche sur le meilleur agent dispo (Codex d'abord, Claude en fallback)
  et rapporter **honnêtement** qui l'a faite. Il ne connaît ni plan ni stratégie.
- **`skills/duo/SKILL.md`** — le *cerveau*. La boucle plan → critique → code → revue,
  bornée à 3 tours (R-LOOP). Il appelle toujours le bridge, jamais `codex` en direct.

## Bridge

```
omega-duo run --task <file.md> --cwd <projet> --mode <plan|code|review> [--agent codex|claude]
omega-duo status      # Codex marqué épuisé cette fenêtre ?
omega-duo reset       # réarme Codex
omega-duo --self-test # prouve quota-detect + fallback SANS appel API
```

`run` émet une ligne JSON : `{ agent, ok, output, fell_back, reason, exit_code, log }`.

## Détection de quota

Une erreur de **quota/limite** déclenche le fallback ; **toute autre** erreur (compile,
test rouge, mauvais chemin) est un vrai échec de tâche et remonte tel quel — jamais
masquée derrière « le quota ». La regex est un best-effort : chaque échec Codex est
loggé verbatim dans `~/.omega/logs/duo/`, pour resserrer la détection sur le vrai
message la première fois qu'on tape la limite.

## État

- `~/.omega/state/duo-codex-exhausted` — Codex indisponible cette session (peut contenir
  l'heure de reset ; auto-nettoyé une fois passée).
- `~/.omega/logs/duo/*.log` — trace complète de chaque run (agent, exit, sortie brute).

---
name: reel-lint
description: Note un script de reel /100 contre les patterns viraux mesurés (hook, rétention, format, CTA, outil branché) AVANT de tourner. Use when the user wants to check, score, or validate a reel script or short-form video script.
---

# Reel Lint — le score avant le tournage

Lance le linter sur le script fourni (fichier .md avec une section `## SCRIPT FACE CAM`, ou texte brut) :

```bash
python3 ~/.claude/skills/reel-lint/scripts/lint_script.py <fichier.md>
python3 ~/.claude/skills/reel-lint/scripts/lint_script.py --text "…le script…"
python3 ~/.claude/skills/reel-lint/scripts/lint_script.py script-A.md script-B.md   # comparer des variantes
```

## Lecture du rapport

Score /100 en 6 blocs : **HOOK** (35) · **RÉTENTION** (25) · **FORMAT** (20) · **CTA** (10) · **VOIX** (10) · **PUISSANCE** (5 — un outil concret branché : repo GitHub, MCP, plugin, skill).

- 🟢 ≥ 85 : GO tournage.
- 🟡 70-84 : corriger les ❌ listés, relancer.
- 🔴 < 70 ou règle bloquante (script > 210 mots, accents absents) : réécrire.

Après chaque correction, relancer le linter jusqu'au 🟢. Présenter à l'utilisateur : le score, les ❌ restants avec la correction proposée, et le verdict.

## Ce que ça évite

Un script trop long (le piège n°1 : 285 mots = 1 min 38 au téléprompteur pour un format 60 s), un hook sans chiffre, une promesse « 3 étapes » non tenue, un CTA sans appel au commentaire, zéro outil nommé. Le linter attrape tout ça en 1 seconde, gratuitement — avant que tu tournes.

⚠️ Un 100/100 = « conforme aux patterns mesurés », pas « viral garanti ». C'est un filtre d'erreurs, pas une boule de cristal.

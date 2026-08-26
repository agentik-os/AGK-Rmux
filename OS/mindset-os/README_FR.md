# Mindset OS

Système opératif AgentikOS du groupe Personnel, intégré depuis le pack Jim Rohn Extended v2.

Mindset OS est un coach conscient du niveau de preuve pour l'identité, le bien-être, la performance et le rapport à la richesse. Il transforme la philosophie, les objectifs, l'énergie, les habitudes, les relations et les revues en un système cohérent, pas en une liste de motivation.

## Contenu

| Chemin | Rôle |
| --- | --- |
| `SKILL.md` et `references/` | Contrat opératoire, routeur, sécurité et protocoles de transformation |
| `pack/` | Archive intégrée du pack d'origine |
| `MASTER.md` | Agent principal utilisé par le panneau OS et Telegram |
| `bin/omega-mindset` | CLI locale pour les espaces hebdomadaires, scorecards, challenges et coaching |
| `scripts/` | Moteurs déterministes et tests |

## Commandes

```bash
omega-mindset new --name "Vous" --output ~/mindset
omega-mindset score ~/mindset/04_WEEKLY_SCORECARD.json
omega-mindset challenge --output ~/challenge --start 2026-08-11
omega-mindset coach ~/challenge
omega-mindset coach ~/challenge --arm
omega-mindset coach ~/challenge --disarm
```

Les interfaces conversationnelles sont `/mindset` et `/mindset-os` dans Claude et Codex.

## Boucle opératoire

```text
STABILISER -> OBSERVER -> CLARIFIER -> DESSINER L'IDENTITE -> CHOISIR
-> INSTALLER L'ENVIRONNEMENT -> EXECUTER -> MESURER -> APPRENDRE -> AJUSTER
```

Chaque recommandation sensible est classée E1 (établie), E2 (prometteuse ou conditionnelle), S (spirituelle), P (préférence personnelle) ou C (clinique).

## Sécurité

La vie, la santé, le sommeil, la stabilité mentale, l'intégrité et les relations passent avant l'optimisation. La richesse reste un résultat possible, jamais une promesse. Toute situation clinique ou de crise est orientée vers un professionnel qualifié.

## Intégration

Mindset OS fournit des contrats d'identité et de comportement à Habit Tracker OS et Execution OS. Le détail machine des événements et des payloads se trouve dans `OMEGA_INTEGRATION.md` et `MANIFEST.json`.

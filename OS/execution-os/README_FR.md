# Execution OS

Execution OS est un système personnel piloté par LLM qui transforme ambitions, obligations et idées en engagements ciblés, travail protégé, preuves livrées, revues et récupération adaptative.

## Boucle fermée

```text
CAPTURER -> CLARIFIER -> SELECTIONNER -> S'ENGAGER -> SE CONCENTRER
-> PROUVER -> REVOIR -> ADAPTER
```

Ce système complète Mindset OS pour l'identité et Habit Tracker OS pour la régularité. Il ne remplace pas la chaîne logicielle Blueprint, Stepper et Builder.

## Contenu

| Chemin | Rôle |
| --- | --- |
| `SKILL.md` et `references/` | Contrat de coaching, protocoles et schémas |
| `pack/` | Pack intégré avec modèles, exemples et moteur v2 |
| `bin/omega-execution` | CLI déterministe et état local |
| `MASTER.md` | Agent principal du panneau OS et Telegram |

## Commandes

```bash
omega-execution init --owner "Vous"
omega-execution boot --capacity GREEN --usable-minutes 240 --must-win "Livrer X"
omega-execution focus <engagement> --minutes 50
omega-execution complete <engagement> --kind ship --evidence "..." --acceptance "..."
omega-execution halt --classification SHIPPED --energy 7 --focus 8 --proof "..."
```

Les commandes conversationnelles sont `/execute` et `/execution-os`.

## Confidentialité

Le profil distribué est générique. Le profil réel de l'utilisateur reste dans `~/.omega/os/execution-os/ledger/profile.md`, hors du dépôt public.

## Intégration

Execution consomme les contraintes de capacité, les habitudes et les objectifs stratégiques, puis publie des preuves de résultat et des signaux de réalité. Le contrat complet est dans `OMEGA_INTEGRATION.md` et `MANIFEST.json`.

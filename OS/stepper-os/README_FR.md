# Stepper OS

Stepper OS transforme une définition produit validée en graphe de travail exécutable, traçable et impossible à déclarer terminé sans preuve.

## Rôle

Il se place entre Blueprint et Builder. Blueprint définit le produit, Stepper compile le plan et ses gates, Builder réalise les artefacts.

```text
Blueprint validé -> graphe Stepper -> étapes verrouillées -> preuves -> handoff Builder
```

## Contenu

| Chemin | Rôle |
| --- | --- |
| `SKILL.md` | Contrat de planification et d'exécution |
| `engine/` | Moteur déterministe, modèles de données et vérificateur |
| `pack/` | Protocole complet du pack livré |
| `bin/omega-stepper` | CLI locale |
| `MASTER.md` | Agent principal du panneau OS |

## Commandes

Les interfaces conversationnelles sont `/stepper` et `/stepper-os`. La CLI `omega-stepper` initialise, inspecte et vérifie les graphes d'étapes. Une étape passe à DONE uniquement lorsque son critère d'acceptation et sa preuve sont satisfaits.

## Intégration

Stepper consomme le handoff gelé de Blueprint et publie un graphe gelé pour Builder. Les payloads, propriétaires et règles de changement sont définis dans `OMEGA_INTEGRATION.md` et `MANIFEST.json`.

# Market Research OS, v1.0.0

**Catégorie :** pile Business, preuves de marché et validation  
**Position Omega :** support de Strategy et Blueprint  
**Interface principale :** conversation et dossiers versionnés  
**Statut :** implémentation de référence installable

## Objectif

Transformer une idée ou une question de marché en hypothèses explicites, sources traçables, modèles auditables, expériences falsifiables et décision bornée avant tout Blueprint ou développement.

## Promesse

Chaque affirmation importante conserve sa source, sa méthode, son niveau de confiance, ses liens de traçabilité et ses éléments négatifs. Une recherche documentaire seule ne suffit jamais à déclarer une idée validée.

## Position dans la chaîne

```text
Idée -> Market Research OS -> décision -> Blueprint OS -> Stepper OS
-> Builder OS -> retour marché -> révision de la recherche
```

## Boucle opératoire

```text
CADRER -> RECUPERER -> FORMULER -> CONCEVOIR -> PREFLIGHT -> COLLECTER
-> TRIANGULER -> MODELISER -> EXPERIMENTER -> CRITIQUER -> DECIDER -> TRANSMETTRE
```

## Contenu

- `SKILL.md` décrit 9 modes, de NEW à DELTA, et trois profondeurs de recherche.
- `references/` contient le contrat de recherche, les méthodes, les gates, la conformité et les handoffs.
- `assets/` contient les schémas, profils de rôle, outils machine et modèles.
- `scripts/market_research_os.py` fournit le runtime déterministe.
- `MANIFEST.json` inventorie les fichiers et les événements du pack.

## Commandes

La racine est `/market-research`, avec les modes `scan`, `validate`, `diligence`, `recover`, `deep`, `monitor`, `audit`, `delta`, `continue`, `status`, `score`, `export` et `handoff`. L'alias `/market-research-os` ouvre le même système.

Les décisions possibles sont `GO`, `PIVOT`, `HOLD`, `NO-GO` et `INSUFFICIENT EVIDENCE`.

## Limites

Market Research OS ne définit pas tout le produit, ne construit pas le DAG d'implémentation et ne lance pas de campagne réelle sans autorité explicite. Il peut produire des guides d'entretien, sondages, expériences, prototypes de recherche et offres factices.

## Intégration

Un handoff gelé transmet à Blueprint les hypothèses, preuves, segments, disposition à payer, inconnues et critères de décision. Le contrat complet est dans `OMEGA_INTEGRATION.md`.

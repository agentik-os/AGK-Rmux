# Design OS, v1.0.0

**Catégorie :** Product Stack / Design UX, interaction et système visuel  
**Position Omega :** Product Stack : compilation du design UX/interaction/visuel, deuxième étape de la branche IMPLEMENT (`Blueprint -> Design -> Stepper -> Builder`)  
**Interface principale :** conversationnelle + handoff lisible par machine  
**Statut :** implémentation de référence installable

## Mission
Compiler un Blueprint produit approuvé en une définition UX/UI cohérente, moderne et challengée, plus un Design Handoff lisible par machine destiné à Stepper. Design OS agit comme un compilateur de design produit et un challenger adversarial des parcours utilisateurs : il transforme la vérité produit en comportements, structure, surfaces, états et contrats de design testables, puis remet à Stepper un graphe de design résolu (pas de la prose inspirationnelle).

## Promesse
Prendre en charge la façon dont les gens comprennent, naviguent, agissent, récupèrent et font confiance au produit, tout en préservant l'intention produit et en challengeant l'interface proposée. Chaque exigence critique est tracée jusqu'à un flow, une surface, un état, un contrat de composant et un test d'acceptation avant tout label `STEPPER_READY`.

## Position dans la chaîne de valeur

```text
Idée/contexte -> Blueprint {OS} -> Design {OS} -> Stepper {OS} -> Builder {OS}
```

Blueprint est le contrat du quoi et du pourquoi. Design OS possède le comment (comportements, structure, surfaces, états, contrats testables) et s'arrête avant l'implémentation de production, sauf demande explicite d'un prototype non destiné à la production.

## Boucle d'opération

```text
RÉCUPÉRER LE BLUEPRINT -> CHALLENGER LA THÈSE -> DÉRIVER IA/NAV -> COMPILER PARCOURS/ÉTATS -> DÉFINIR L'INTERACTION -> DÉFINIR LE SYSTÈME VISUEL -> COMPILER SURFACES/COMPOSANTS -> PROTOTYPER/VALIDER -> ÉMETTRE LE HANDOFF STEPPER
```

## Ce que contient cet OS
- Contrat de compilation canonique avec douze lois directrices et un protocole de décision (`SKILL.md`)
- Une seule skill de compilation de design adversariale, exécutée en neuf passes
- 10 protocoles de référence : workflow du compilateur et gates, contrat de sortie Stepper, protocole de flow-challenge, système d'interaction chat/agent, intelligence produit IA, architecture STAX et shadcn, protocole de système visuel moderne, contrat responsive et accessibilité, validation et evals de design, et un prompt système maître prêt à coller
- 2 schémas JSON : `blueprint-intake.schema.json` et `design-handoff.schema.json`
- 3 validateurs Python et un smoke test : `validate_blueprint_intake.py`, `validate_design_handoff.py`, `self_test.py`
- 1 descripteur d'interface (`agents/openai.yaml`) et 1 asset d'icône (`assets/icon.svg`)
- Note : ce pack se livre comme une skill de compilation unique plus des protocoles de référence et des validateurs. Il ne porte PAS les répertoires séparés `skills/`, `protocols/`, `knowledge/`, `memory/`, `database/` ou `evals/` que certains packs OS plus riches utilisent : les protocoles de référence vivent sous `references/` et le matériel d'eval est `references/validation-evals.md`.

## Commandes
Le pack expose une commande par défaut ; la profondeur d'exécution est choisie par le mode.

| Commande | Mode | Rôle |
| --- | --- | --- |
| `/design` | dispatch | Ouvrir Design OS et compiler un Blueprint en handoff UX/UI validé |

Modes d'opération (annoncés au début d'une exécution) :

| Mode | Rôle |
| --- | --- |
| `FULL` | Exécuter toutes les passes et émettre le Design Definition Pack complet (par défaut) |
| `AUDIT` | Challenger un design ou un codebase existant et émettre les écarts plus un handoff de réparation |
| `FLOW` | Se concentrer sur des parcours choisis tout en gardant la traçabilité et les gates d'états limites |
| `AI_APP` | Prioriser le composer, le contexte, l'état d'agent, les outils, les artefacts, les sources et le comportement mémoire |
| `STAX_FIT` | Décider si, où et comment utiliser STAX |
| `REVISION` | Mettre à jour les IDs et contrats impactés sans réécrire les sections intactes |

## Principaux handoffs
- Blueprint OS fournit la vérité produit et système (le quoi et le pourquoi) ; Design OS consomme `blueprint.completed`.
- Stepper OS reçoit le Design Handoff lisible par machine (`design-handoff.json`) ; Design OS produit `design.handoff.completed`.
- Context & Memory OS conserve le design handoff canonique : Design OS lit `memory.context.compiled`, écrit `memory.record.staged` et reçoit `memory.record.verified` en retour.
- Review & Governance OS approuve les changements de frontières, schémas ou quality gates en production.

## Installation
Voir `OMEGA_INTEGRATION.md` pour l'enregistrement (ID `design`, commande par défaut `/design`), l'ordre d'injection de contexte, le câblage des événements et le contrôle des changements.

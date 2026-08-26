# Context & Memory OS, Guide français

## Mission
Créer une couche mémoire unique, fiable, inspectable et gouvernée afin qu’Omega et tous ses OS puissent récupérer le bon contexte sans mélanger faits, inférences, états temporaires, projets ou identités.

## Ce que cet OS doit changer
Fournir au bon OS le contexte minimal suffisant, au bon moment, avec source, date, confiance, permissions et historique des contradictions.

## Principe directeur

```text
TRUSTED CONTEXT = RELEVANCE × PROVENANCE × RECENCY × CONSENT × RETRIEVABILITY × CORRECTABILITY
```

## Boucle d'opération

```text
CAPTURE → HASH → CLASSIFY → EXTRACT → PROVENANCE → RESOLVE → STORE → RETRIEVE → COMPILE → REVIEW → ARCHIVE / FORGET
```

## Règle d'indépendance
Cet OS produit des décisions, des preuves, des plans, des contrôles et des handoffs. Il ne doit pas absorber les responsabilités des autres OS. Sa frontière principale est : **Context & Memory OS stores and compiles authorized knowledge; it does not decide strategy, invent facts, silently profile the user, or let one OS read everything.**.

## Utilisation
Le mode conversationnel reste l'entrée principale. Les données structurées servent à conserver les décisions et preuves, pas à remplacer la discussion.

## Ce que contient cet OS

Le pack livre 14 agents, 20 skills, 7 protocoles, 8 schémas, un runtime, un modèle de données, des règles de confidentialité et des tests.

## Commandes

`/memory`, `/remember`, `/ingest`, `/context`, `/snapshot`, `/decision-log`, `/contradiction`, `/memory-audit`, `/forget` et `/export-memory`.

## Handoffs principaux

Chaque OS demande un contexte borné et stage ses écritures. Review transmet les learning packs. Books transmet des insights confirmés avec source et interprétation séparées. Strategy reçoit des snapshots versionnés.

## Installation

Le pack est installé sous `~/.omega/os/context-memory-os` et comme skill. Il reste l'unique autorité canonique pour la mémoire partagée.

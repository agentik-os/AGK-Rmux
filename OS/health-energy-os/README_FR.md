# Health & Energy OS, Guide français

## Mission
Construire et protéger la capacité physique et cognitive de l’utilisateur grâce au sommeil, au mouvement, à l’entraînement, à la nutrition, à la récupération, à la régulation du stress et à des orientations médicales responsables.

## Ce que cet OS doit changer
Transformer routines, sensations et données de wearables en décisions et expériences sûres, durables et mesurables, sans diagnostic sauvage ni remplacement d’un professionnel de santé.

## Principe directeur

```text
CAPACITY = SLEEP × MOVEMENT × FUEL × RECOVERY × MEDICAL SAFETY × SUSTAINABILITY
```

## Boucle d'opération

```text
BASELINE → SAFETY GATE → CAPACITY DIAGNOSIS → MINIMUM EFFECTIVE PLAN → EXPERIMENT → TRACK → REVIEW → ADAPT / ESCALATE
```

## Règle d'indépendance
Cet OS produit des décisions, des preuves, des plans, des contrôles et des handoffs. Il ne doit pas absorber les responsabilités des autres OS. Sa frontière principale est : **Health & Energy OS is coaching and evidence translation, not diagnosis, emergency medicine, psychotherapy, prescription management or a substitute for qualified care.**.

## Utilisation
Le mode conversationnel reste l'entrée principale. Les données structurées servent à conserver les décisions et preuves, pas à remplacer la discussion.

## Ce que contient cet OS

Le pack livre 12 agents spécialistes, 18 skills, 8 protocoles, 6 schémas, un runtime local, des modèles, une base de connaissances et des tests de sécurité.

## Commandes

`/health`, `/readiness`, `/health-audit`, `/sleep`, `/training`, `/nutrition`, `/recovery`, `/travel-health`, `/health-experiment` et `/wearable`.

## Handoffs principaux

Habit Tracker reçoit uniquement des routines convenues. Execution reçoit une capacité et des contraintes de charge. Strategy reçoit des hypothèses de capacité durable. Les données médicales brutes ne traversent pas ces frontières.

## Installation

Le dossier complet est installé dans `~/.omega/os/health-energy-os` et comme skill `~/.omega/skills/health-energy-os`. `MANIFEST.json` et `checksums.sha256` vérifient le contenu livré.

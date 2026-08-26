# Quality, Evaluation & Release OS, Guide français

## Mission
Prouver qu’un produit respecte ses contrats, maîtrise ses risques, peut être observé et restauré, et est prêt pour une mise en production contrôlée.

## Ce que cet OS doit changer
Remplacer « ça semble fini » par des preuves traçables sur la fonctionnalité, l’UX, l’accessibilité, la performance, la fiabilité, la sécurité, la confidentialité, la donnée, l’IA, les opérations et le rollback.

## Principe directeur

```text
RELEASE CONFIDENCE = REQUIREMENT TRACEABILITY × RISK-BASED EVIDENCE × SECURITY × RELIABILITY × OBSERVABILITY × RECOVERABILITY
```

## Boucle d'opération

```text
CONTRACTS → RISK MODEL → TEST/EVAL PLAN → EXECUTE → TRIAGE → FIX/RETEST → GATES → RELEASE CANDIDATE → DEPLOY → VERIFY → MONITOR / ROLLBACK
```

## Règle d'indépendance
Cet OS produit des décisions, des preuves, des plans, des contrôles et des handoffs. Il ne doit pas absorber les responsabilités des autres OS. Sa frontière principale est : **Builder OS builds and repairs; Quality, Evaluation & Release OS independently defines evidence, evaluates, gates and authorizes release. It does not certify absent evidence.**.

## Utilisation
Le mode conversationnel reste l'entrée principale. Les données structurées servent à conserver les décisions et preuves, pas à remplacer la discussion.

## Ce que contient cet OS

Le pack livre 16 agents, 26 skills, 7 protocoles, 10 schémas, des standards de test, un runtime, des modèles de release et des tests adversariaux.

## Commandes

`/quality`, `/test-plan`, `/traceability`, `/qa`, `/ai-eval`, `/security-review`, `/accessibility`, `/release-candidate`, `/release-gate`, `/deploy` et `/rollback`.

## Handoffs principaux

Builder fournit les artefacts. Content fournit les candidats de publication. Operations fournit les observations de production. Review & Governance autorise les changements et exceptions de politique.

## Installation

Le pack complet est copié sous `~/.omega/os/quality-evaluation-release-os` et exposé comme skill. Le manifeste, les checksums et les preuves de gate restent la source de vérité.

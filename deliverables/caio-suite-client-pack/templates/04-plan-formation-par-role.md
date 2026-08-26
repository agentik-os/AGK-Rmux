# Plan de formation par rôle — exec / manager / opérateur / champion

# Plan de formation & adoption — {{NOM_CLIENT}}

> Document INTERNE + support client. Une formation réussie = des gens qui UTILISENT le système sans nous. On forme par rôle, dans le langage du rôle. Objectif : autonomie, pas démo.

**Phase :** 4 — Formation & handoff (skill : caio-enablement-and-transfer)
**Système concerné :** {{Company AI OS / workflows F00X}} · **Date :** {{JJ/MM}}

---

## 1. Principe
Chaque rôle apprend **ce qu'il doit faire**, pas comment l'IA marche sous le capot. On mesure l'adoption, pas la présence en salle.

## 2. Curricula par rôle

### A. Exécutif / Sponsor ({{durée : 1 h}})
- **Objectif :** lire le dashboard, comprendre la valeur, défendre l'adoption en interne.
- **Contenu :** {{lecture des KPIs, coûts IA, où sont les décisions humaines, comment arbitrer}}
- **Sait faire à la fin :** {{ouvrir le dashboard, lire le QBR, valider/refuser une extension}}

### B. Manager ({{2 h}})
- **Objectif :** piloter son équipe avec le système, repérer les frictions, remonter le backlog.
- **Contenu :** {{suivi des workflows de son service, métriques d'équipe, escalade des incidents}}
- **Sait faire :** {{interpréter les logs, prioriser une amélioration, déclencher une revue}}

### C. Opérateur (utilisateur quotidien) ({{2-3 h + shadowing}})
- **Objectif :** faire son travail AVEC le workflow, valider les sorties IA (human-in-the-loop).
- **Contenu :** {{golden path pas-à-pas, quand valider/corriger, que faire si erreur}}
- **Sait faire :** {{exécuter le workflow de bout en bout, corriger une sortie, signaler un cas hors-norme}}

### D. Champion (relais interne / power user) ({{1 jour + runbooks}})
- **Objectif :** devenir le 1er niveau de support interne, former les nouveaux, possède les runbooks.
- **Contenu :** {{architecture du système, runbooks complets, debug niveau 1, ajout d'un cas}}
- **Sait faire :** {{résoudre un incident courant, onboarder un nouvel opérateur, escalader vers Agentik}}

## 3. Format & calendrier
| Rôle | Participants | Format | Date | Durée | Formateur |
|------|-------------|--------|------|-------|-----------|
| Exec | {{N}} | {{visio}} | {{JJ/MM}} | {{1 h}} | {{}} |
| Manager | {{N}} | {{atelier}} | {{}} | {{2 h}} | {{}} |
| Opérateur | {{N}} | {{atelier + shadowing}} | {{}} | {{3 h}} | {{}} |
| Champion | {{1-2}} | {{1:1 approfondi}} | {{}} | {{1 j}} | {{}} |

## 4. Supports remis
- {{Runbook par workflow (`06-handoff`)}}
- {{Cheat-sheet 1 page par rôle}}
- {{Vidéo golden path ({{X}} min)}}
- {{FAQ + arbre de décision “que faire si…”}}

## 5. Mesure de l'adoption (critère de réussite)
| Indicateur | Cible | Mesure à T+30 |
|-----------|-------|---------------|
| Opérateurs actifs / total | {{≥ 80%}} | {{}} |
| Workflows exécutés sans aide | {{≥ 90%}} | {{}} |
| Tickets support internes vers Champion vs Agentik | {{≥ 70% résolus en interne}} | {{}} |
| Score de confiance (sondage 1-5) | {{≥ 4}} | {{}} |

## 6. Plan de conduite du changement
- {{Communication sponsor avant lancement}}
- {{Quick-win visible en semaine 1}}
- {{Boucle de feedback hebdo les 4 premières semaines}}

---
**Mini-exemple (opérateur rempli) :** « Opérateur support — 6 agents Tier 1. À la fin : ouvrir un ticket entrant, lire la classification + le brouillon de réponse proposés par F001, le corriger en 2 clics si besoin, l'envoyer, et marquer “cas étrange” pour les rares tickets que l'agent n'a pas su classer. Atelier 3 h + 2 jours de shadowing. Cible : 90% des tickets traités sans aide à J+30. »
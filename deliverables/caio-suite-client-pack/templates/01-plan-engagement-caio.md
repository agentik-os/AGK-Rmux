# Plan d'engagement CAIO — phases, jalons, livrables, RACI, planning

# Plan d'engagement CAIO — {{NOM_CLIENT}}

> Document INTERNE Agentik. Source de vérité du déroulé de la mission. Une ligne = un engagement responsable. Pas de jalon sans livrable, pas de livrable sans owner, pas d'owner sans date.

**Client :** {{NOM_CLIENT}} · **Secteur :** {{SECTEUR}} · **Taille :** {{1-10 / 11-50 / 51-200 / 201-1000 / 1000+}}
**Sponsor exécutif (client) :** {{NOM, TITRE}} · **Lead Agentik (CAIO) :** {{NOM}}
**Palier vendu :** {{Good / Better / Best}} · **Date de démarrage :** {{JJ/MM/AAAA}} · **Durée cible :** {{X semaines}}
**Réf. proposition / SOW :** {{02-proposition / 03-sow}}

---

## 1. Objectif de la mission (en une phrase)
> Rendre {{NOM_CLIENT}} `lisible → automatisable → agentique`, dans cet ordre, et livrer un Company AI OS exploité et possédé par les équipes.

**Résultats mesurables visés (3 max) :**
- {{Ex : 12 h/semaine récupérées sur le triage support — mesuré sur F001}}
- {{Ex : délai de réponse devis 48 h → 4 h}}
- {{Ex : 1 dashboard IA en prod, 4 workflows automatisés en run}}

## 2. Les 7 phases (parcours CAIO)

| # | Phase | Skill exécutante | Livrable principal | Sortie | Durée |
|---|-------|------------------|--------------------|--------|-------|
| 0 | AI-Readiness (go/no-go pré-signature) | caio-ai-readiness-assessment | Scorecard 9-dim + verdict GO/NOT-YET/REDIRECT | `./caio-readiness/` | {{0,5 sem}} |
| 1 | Offre & cadrage | offer-and-revenue-architect + market-proposal | Proposition signée + SOW | `02`, `03` | {{1 sem}} |
| 2 | Découverte | caio-discovery-interview | 1 ZIP standardisé / personne | `discovery/*.zip` | {{1-2 sem}} |
| 3 | Diagnostic + Architecture + Roadmap | caio-enterprise-workflow-architect | Company AI OS (10 livrables) | `./company-ai-os/` | {{2-4 sem}} |
| 4 | Build (Company-AI-OS fédéré) | caio-implementation-runbook | Système live vérifié, build par ship-gate | `./caio-build/` | {{3-8 sem}} |
| 5 | Formation & transfert | caio-enablement-and-transfer | Runbooks + transfert de propriété | `./caio-enablement/` | {{1-2 sem}} |
| 6 | Run & optimisation | caio-run-and-optimize | KPIs, QBR, amélioration continue | `./caio-run/` | {{retainer mensuel}} |

> **Orchestration :** `/caio-master` gap-checke chaque relais (livrable réel présent + suffisant avant d'avancer) et écrit `./caio-engagement/` (plan d'engagement + tracker de phases).

## 3. Jalons & critères de sortie (gates)
> Un jalon ne se franchit que si son critère est VÉRIFIÉ (runtime, pas déclaratif — L1).

| Jalon | Critère de sortie (vérifiable) | Date cible | Statut |
|-------|-------------------------------|-----------|--------|
| J0 — Kickoff | SOW signé + sponsor + accès garantis | {{JJ/MM}} | {{À faire}} |
| J1 — Découverte close | {{N}} entretiens faits, {{N}} ZIP livrés | {{JJ/MM}} | {{}} |
| J2 — Company AI OS validé | 10 livrables présents + backlog scoré + ROI signé CFO | {{JJ/MM}} | {{}} |
| J3 — 1er workflow en prod | /omg-acceptance vert + golden path écrit réel | {{JJ/MM}} | {{}} |
| J4 — Handoff accepté | checklist d'acceptation signée (cf. `06`) | {{JJ/MM}} | {{}} |
| J5 — 1er QBR | KPIs vs promis présentés au sponsor | {{JJ/MM}} | {{}} |

## 4. RACI
> R=Réalise · A=Approuve (1 seul) · C=Consulté · I=Informé

| Activité | Lead CAIO | Builder Agentik | Sponsor client | Champion client | IT/Sécu client | Légal/RGPD |
|----------|:---------:|:---------------:|:--------------:|:---------------:|:--------------:|:----------:|
| Cadrage & SOW | R | I | A | C | C | C |
| Entretiens découverte | R | I | I | C | I | I |
| Architecture Company AI OS | R | C | A | C | C | C |
| Build / déploiement | A | R | I | C | C | I |
| Accès données & permissions | C | C | A | I | R | A |
| Formation | R | C | I | A | I | I |
| Run & QBR | A | R | C | R | C | I |

## 5. Planning (vue Gantt simplifiée)
```
Sem.  1   2   3   4   5   6   7   8   9  10  11  12
P0   [==]
P1       [====]
P2            [========]
P3                     [============]
P4                                  [====]
P5                                       [retainer →]
```

## 6. Conditions de réussite (engagements croisés)
**Côté client (sans quoi on bloque, R-FICHE) :**
- {{Sponsor disponible 2 h/sem}}
- {{Accès aux {{N}} interviewés sous 5 jours}}
- {{Accès lecture aux outils {{CRM/ERP/…}} + creds de prod sous NDA}}

**Côté Agentik :**
- {{Lead CAIO + 1 builder dédiés}}
- {{Reporting hebdo + démo de fin de phase}}

## 7. Risques & mitigations (top 5)
| Risque | Prob. | Impact | Mitigation | Owner |
|--------|:-----:|:------:|-----------|-------|
| {{Données sales / non lisibles}} | {{M}} | {{H}} | {{phase 2 — data map avant tout build}} | {{Lead}} |
| {{Sponsor absent}} | {{}} | {{}} | {{escalade contractuelle J0}} | {{}} |

## 8. Budget & jalons de facturation
| Phase | Montant | Déclencheur de facture |
|-------|--------:|------------------------|
| P0-1 | {{€}} | {{à la signature}} |
| P2 | {{€}} | {{validation Company AI OS}} |
| P3 | {{€}} | {{1er workflow en prod}} |
| P4-5 | {{€/mois}} | {{retainer mensuel}} |

---
**Mini-exemple (extrait J3) :** « J3 — Agent de triage support Tier 1 (F001) en prod : /omg-acceptance vert sur 14 routes, golden path “ticket entrant → classé → réponse brouillon → validé humain → envoyé” écrit réel en base, 0 erreur console app-bundle. Date 14/07, statut Fait. »
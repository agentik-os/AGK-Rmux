# QBR — Quarterly Business Review (KPIs vs promis, incidents, backlog, décisions)

# QBR — Revue trimestrielle · {{NOM_CLIENT}}
## Trimestre {{Q?-AAAA}}

> Document client-facing, présenté au sponsor. Règle d'or : on compare le RÉEL au PROMIS, sans maquiller. Un QBR honnête vaut mieux qu'un beau slide. (Phase 5 — caio-run-and-optimize.)

**Période :** {{JJ/MM}} → {{JJ/MM}} · **Présenté par :** {{Lead CAIO}} · **Présents :** {{sponsor, champion, …}}

---

## 1. Synthèse (1 slide)
- **État global :** {{🟢 sur la trajectoire / 🟡 vigilance / 🔴 action requise}}
- **Le chiffre du trimestre :** {{ex : 540 h récupérées cumulées}}
- **La décision à prendre aujourd'hui :** {{ex : valider le palier Best pour Q3}}

## 2. KPIs : promis vs réalisé
| KPI | Promis (baseline → cible) | Réalisé ce trimestre | Écart | Tendance |
|-----|---------------------------|---------------------:|:-----:|:--------:|
| {{Temps gagné / sem}} | {{0 → 20 h}} | {{17 h}} | {{-3 h}} | {{↗}} |
| {{Délai réponse devis}} | {{48 h → 4 h}} | {{6 h}} | {{+2 h}} | {{↗}} |
| {{Tickets / ETP}} | {{x → 2x}} | {{1,8x}} | {{}} | {{→}} |
| {{Coût IA / mois}} | {{< €…}} | {{€…}} | {{}} | {{}} |
| {{Taux de validation humaine}} | {{}} | {{}} | {{}} | {{}} |

> Lecture honnête de l'écart : {{pourquoi -3 h ? adoption à 80%, 2 opérateurs encore en montée — comblé d'ici T+30.}}

## 3. Valeur livrée vs investissement (cumul)
| | Cumul à ce jour |
|---|--:|
| Valeur générée (temps × coût chargé) | {{€…}} |
| Coûts (build + run + retainer) | {{€…}} |
| Ratio / payback | {{€ rendus pour 1 € investi · payback atteint à {{mois}}}} |

## 4. Incidents & fiabilité
| Incident | Date | Impact | Cause racine | Résolu | Préventif |
|----------|------|--------|--------------|:------:|-----------|
| {{Agent a mal classé X}} | {{}} | {{}} | {{}} | {{✓}} | {{ajout règle + revue}} |

**Disponibilité du système :** {{99,x%}} · **Erreurs nécessitant intervention humaine :** {{N}}

## 5. Backlog & priorisation pour le prochain trimestre
| Opportunité (réf. backlog) | Score (10 critères) | Effort | Valeur | Décision |
|----------------------------|:-------------------:|:------:|:------:|----------|
| {{F004 — relances impayés}} | {{82}} | {{M}} | {{H}} | {{✅ go Q3}} |
| {{F005 — reporting auto}} | {{67}} | {{}} | {{}} | {{⏸ plus tard}} |

## 6. Décisions demandées au sponsor
1. {{Valider le go sur F004 ({{€}} build)}}
2. {{Reconduire le retainer {{palier}}}}
3. {{Arbitrer {{…}}}}

## 7. Actions & owners (pour le prochain QBR)
| Action | Owner | Échéance |
|--------|-------|----------|
| {{Combler l'adoption des 2 opérateurs}} | {{Champion}} | {{T+30}} |
| {{Builder F004}} | {{Agentik}} | {{T+60}} |

---
**Mini-exemple (synthèse remplie) :** « État 🟡 vigilance. Chiffre du trimestre : 540 h récupérées en cumul (cible trimestre 600 h — adoption à rattraper). Décision : valider F004 (relances impayés, payback estimé 2 mois) et reconduire le retainer Better pour Q3. »
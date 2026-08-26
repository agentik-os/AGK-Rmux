# Business case — acheter l'accompagnement (coût vs valeur, payback, sensibilité)

# Business case — Investir dans l'accompagnement CAIO
## {{NOM_CLIENT}}

> Outil d'aide à la décision pour le CEO/CFO. Objectif : montrer noir sur blanc que le coût de l'accompagnement est inférieur à la valeur récupérée, avec le payback et la sensibilité. On ne gonfle aucun chiffre — chaque hypothèse est sourcée et testable (méthode maison : ROI vient des entretiens, pas de l'imagination).

**Date :** {{JJ/MM}} · **Auteur :** {{Lead CAIO}} · **Horizon d'analyse :** {{12 mois}}

---

## 1. La question à trancher
> Faut-il investir **{{€ total}}** dans l'accompagnement, pour récupérer **{{€ / an}}** et libérer {{X}} h/semaine ?

**Réponse en une ligne :** {{Oui — payback en {{X}} mois, ROI {{Y}}× sur 12 mois, même dans le scénario pessimiste.}}

## 2. Hypothèses (toutes sourcées)
| Hypothèse | Valeur | Source |
|-----------|-------:|--------|
| Coût horaire chargé moyen | {{€…/h}} | {{RH client}} |
| ETP concernés | {{N}} | {{entretiens découverte}} |
| Heures répétitives / sem automatisables | {{X h}} | {{Role-And-Workflow-Inventory}} |
| % réellement automatisable (prudent) | {{60%}} | {{scoring backlog}} |
| Semaines travaillées / an | {{45}} | {{}} |

## 3. Coûts (côté investissement)
| Poste | Montant | Type |
|-------|--------:|------|
| Accompagnement (build + archi) | {{€}} | one-time |
| Run / retainer | {{€/mois × 12}} | récurrent |
| Coûts IA (LLM/API) | {{€/mois × 12}} | récurrent |
| Temps interne client (sponsor, champion) | {{€}} | one-time |
| **Total an 1** | **{{€}}** | |

## 4. Valeur (côté retour)
| Workflow | h/sem gagnées | Valeur/an | Note |
|----------|--------------:|----------:|------|
| {{Triage support}} | {{12}} | {{€…}} | {{}} |
| {{Génération devis}} | {{8}} | {{€…}} | {{}} |
| Gains qualitatifs (non chiffrés) | — | — | {{délai client, erreurs évitées, rétention}} |
| **Total valeur an 1** | **{{20}}** | **{{€…}}** | |

Valeur/an d'un workflow = h/sem × 45 sem × €/h × % automatisable.

## 5. Synthèse financière
| Indicateur | Valeur |
|-----------|-------:|
| Valeur an 1 | {{€…}} |
| Coût an 1 | {{€…}} |
| **Bénéfice net an 1** | **{{€…}}** |
| **ROI an 1** | **{{Y×}}** |
| **Payback** | **{{X mois}}** |
| Valeur an 2+ (coûts récurrents seuls) | {{€…/an}} |

## 6. Analyse de sensibilité
> « Et si on s'est trompés ? » On teste les 3 scénarios.

| Scénario | % automatisable | Adoption | Valeur an 1 | Payback | ROI |
|----------|:---------------:|:--------:|------------:|--------:|----:|
| Pessimiste | {{40%}} | {{60%}} | {{€…}} | {{X mois}} | {{}} |
| Réaliste ⭐ | {{60%}} | {{80%}} | {{€…}} | {{X mois}} | {{}} |
| Optimiste | {{75%}} | {{95%}} | {{€…}} | {{X mois}} | {{}} |

**Seuil de rentabilité :** l'investissement est remboursé dès {{X}} h/sem récupérées — soit {{Z%}} de l'objectif. {{Marge de sécurité confortable.}}

## 7. Coût de l'inaction (do-nothing)
Ne rien faire coûte {{€…/an}} en temps perdu + {{risques : turnover sur tâches ingrates, concurrents AI-native, plafond de croissance sans embauche}}.

## 8. Recommandation
{{Démarrer en palier {{Better}} : périmètre suffisant pour prouver le ROI sur {{X}} workflows, décision d'extension prise au 1er QBR sur données réelles.}}

---
**Mini-exemple (synthèse remplie) :** « 3 collaborateurs × 14 h/sem récupérées × 45 sem × 35 €/h × 60% automatisable = ~119 000 €/an de valeur. Coût an 1 : 48 000 € (build 30 k + run 18 k). Bénéfice net ~71 000 €, ROI 2,5×, payback 4 mois. Scénario pessimiste (40%/60%) : payback toujours < 7 mois. → Décision : go Better. »
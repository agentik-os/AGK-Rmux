# Dossier de handoff + checklist d'acceptation (transfert de propriété)

# Dossier de handoff & checklist d'acceptation — {{NOM_CLIENT}}

> Phase 4. Le handoff matérialise le transfert de PROPRIÉTÉ : à la fin, le Client possède et exploite son système sans dépendance à Agentik. Aucun item coché sans preuve (L1 — runtime, pas déclaratif).

**Système livré :** {{Company AI OS + workflows F00X}} · **Date de transfert :** {{JJ/MM}}
**Remis par :** {{Lead CAIO}} · **Reçu par :** {{Champion + Sponsor client}}

---

## 1. Inventaire des actifs transférés
| Actif | Localisation / accès | Propriétaire final | Transféré |
|-------|----------------------|--------------------|:---------:|
| Code source | {{repo GitHub {{org client}}}} | {{Client}} | {{☐}} |
| Workflows / agents en prod | {{URL + compte Vercel/Convex client}} | {{Client}} | {{☐}} |
| Dashboard IA | {{URL}} | {{Client}} | {{☐}} |
| Credentials & secrets | {{coffre client, hors dépôt}} | {{Client}} | {{☐}} |
| Company AI OS (10 livrables) | {{`./company-ai-os/`}} | {{Client}} | {{☐}} |
| Runbooks & docs | {{wiki client}} | {{Client}} | {{☐}} |
| Comptes & abonnements (LLM/API) | {{au nom du client, facturation client}} | {{Client}} | {{☐}} |

## 2. Runbooks remis (1 par workflow)
Chaque runbook contient : objectif, input/output, owner, sources de données, permissions, points de validation humaine, que faire en cas d'erreur, qui escalader, coûts attendus.
- {{☐ Runbook F001 — …}}
- {{☐ Runbook F002 — …}}

## 3. Checklist d'acceptation (gate de transfert)
> Le transfert est ACCEPTÉ uniquement si tous les items critiques sont ✅ avec preuve.

### Technique
- ☐ Tous les workflows tournent en prod — preuve : {{/omg-acceptance vert, lien rapport}}
- ☐ Golden path de chaque workflow exécuté avec écriture réelle — preuve : {{capture/log}}
- ☐ 0 erreur console app-bundle sur les routes clés — preuve : {{}}
- ☐ Audit code passé (/codeaudit) — preuve : {{score/lien}}
- ☐ Audit sécurité passé (/secaudit) — preuve : {{score/lien}}
- ☐ Secrets hors dépôt, accès au coffre transmis — preuve : {{}}
- ☐ Human-in-the-loop actif sur toute décision sensible — preuve : {{}}
- ☐ Le Client peut déployer une modif sans Agentik — preuve : {{déploiement test fait par le Champion}}

### Données & conformité
- ☐ Data map à jour (sources, permissions) — preuve : {{`04-Data-And-Permission-Map`}}
- ☐ DPA / RGPD : durées, localisation, anonymisation vérifiées — preuve : {{}}
- ☐ Logs, statut, coûts, confiance visibles dans le dashboard — preuve : {{}}

### Adoption & autonomie
- ☐ Formations tenues pour les 4 rôles — preuve : {{`04-plan-formation` + feuilles de présence}}
- ☐ Champion capable de résoudre un incident niveau 1 — preuve : {{incident-test résolu en autonomie}}
- ☐ Taux d'adoption ≥ {{cible}} — preuve : {{métriques}}
- ☐ Procédure de support / escalade vers Agentik documentée — preuve : {{}}

### Contractuel
- ☐ Paiement intégral des livrables → IP cédée (cf. SOW §8)
- ☐ Réversibilité effective (aucune dépendance bloquante à Agentik)

## 4. Réserves & points ouverts à la livraison
| Réserve | Criticité | Plan de résolution | Échéance | Owner |
|---------|:---------:|--------------------|----------|-------|
| {{}} | {{}} | {{}} | {{}} | {{}} |

## 5. Modalités post-handoff
- **Garantie / hypercare :** {{30 jours de support inclus}}
- **Bascule en Run :** {{retainer phase 5 — cf. `01-plan-engagement`}}
- **Contact escalade :** {{}}

## 6. Procès-verbal d'acceptation
Le Client reconnaît avoir reçu les actifs ci-dessus et accepte le transfert {{avec / sans}} réserves listées §4.

| Le Client (Sponsor) | Le Champion | Le Prestataire |
|---------------------|-------------|----------------|
| Nom/Date/Signature | Nom/Date/Signature | Nom/Date/Signature |

---
**Mini-exemple (item coché avec preuve) :** « ☐→✅ Le Client peut déployer une modif sans Agentik — preuve : le 12/08, le Champion a modifié le message de relance de F004 et redéployé sur Vercel en autonomie (commit a1b2c3d, build vert), Agentik en simple observation. »
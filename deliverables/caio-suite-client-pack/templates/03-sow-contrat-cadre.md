# SOW / Contrat-cadre d'accompagnement CAIO

# Énoncé de travaux (SOW) — Accompagnement CAIO
## entre {{AGENTIK_OS_ENTITE}} (« le Prestataire ») et {{NOM_CLIENT}} (« le Client »)

> Document contractuel. À faire relire par le juridique des deux parties avant signature. Les montants et délais font foi ; en cas de conflit avec la proposition, le présent SOW prévaut.

**Réf. :** {{SOW-AAAA-NNN}} · **Date :** {{JJ/MM/AAAA}} · **Proposition liée :** {{02}}

---

## 1. Objet
Le Prestataire accompagne le Client pour le transformer en organisation AI-native via le parcours CAIO (Découverte → Diagnostic/Architecture → Build → Formation → Run).

## 2. Périmètre des travaux (scope)
| Phase | Livrables contractuels | Critère d'acceptation |
|-------|------------------------|-----------------------|
| Découverte | {{N}} entretiens + {{N}} dossiers standardisés | {{ZIP remis pour chaque interviewé}} |
| Company AI OS | 10 livrables `./company-ai-os/` | {{remis + revue de validation tenue}} |
| Build | {{X}} workflows déployés en prod | {{gate /omg-acceptance vert + golden path}} |
| Formation | curricula {{N}} rôles + runbooks | {{sessions tenues + runbooks remis}} |
| Run | {{N}} mois d'exploitation + {{N}} QBR | {{rapports KPIs remis}} |

## 3. Hors-scope (explicite)
Ne sont PAS inclus, sauf avenant écrit : {{migration/nettoyage de données legacy, refonte ERP/CRM, achat de licences tierces, support 24/7, certifications réglementaires (SOC2/ISO), développement hors workflows listés, hébergement au-delà de {{X}}}}.

## 4. Responsabilités du Client
- {{Désigner un sponsor exécutif et un champion}}
- {{Garantir l'accès aux interviewés et aux systèmes sous {{X}} jours ouvrés}}
- {{Fournir les accès/credentials de prod nécessaires (sous NDA)}}
- {{Valider chaque jalon sous {{X}} jours ouvrés (silence = acceptation tacite passé ce délai)}}

## 5. Modalités d'exécution
**Lieu :** {{remote / sur site / hybride}} · **Comité de pilotage :** {{hebdomadaire}} · **Langue :** Français.

## 6. Calendrier & jalons de facturation
| Jalon | Date cible | Montant HT | Échéance |
|-------|-----------|-----------:|----------|
| Signature | {{JJ/MM}} | {{€}} | {{à réception facture, {{30 j}}}} |
| Validation Company AI OS | {{JJ/MM}} | {{€}} | {{…}} |
| 1er workflow en prod | {{JJ/MM}} | {{€}} | {{…}} |
| Run mensuel | {{récurrent}} | {{€/mois}} | {{…}} |

**Montant total HT :** {{€}} · **TVA :** {{20%}} · **Conditions :** {{retard = pénalités au taux légal}}.

## 7. Confidentialité & RGPD
- **Confidentialité :** chaque partie protège les informations de l'autre ({{durée : pendant la mission + {{X}} ans}}).
- **Données personnelles :** le Prestataire agit en **sous-traitant** (art. 28 RGPD). Annexe DPA jointe : finalités, durées, mesures de sécurité, sous-traitants ultérieurs, localisation des données ({{UE}}).
- **Minimisation :** seules les données nécessaires aux workflows ciblés sont traitées ; anonymisation des verbatims d'entretien par défaut.
- **Secrets :** credentials gérés hors dépôt, jamais en clair dans le code (politique Prestataire).
- **Sous-traitants/IA tierces :** {{lister les API/LLM utilisés + localisation}} — soumis à l'accord du Client.

## 8. Propriété intellectuelle
- **Livrables & code spécifiques** produits pour le Client : cédés au Client à **paiement intégral**.
- **Briques réutilisables, méthodes, skills, templates** du Prestataire : restent sa propriété ; licence d'usage non-exclusive accordée au Client.
- Le Client possède ses données, ses agents déployés et ses comptes de prod.

## 9. Garanties & limites
- Obligation de **moyens** sur les résultats business (ROI estimé, non garanti — dépend de l'adoption).
- Obligation de **résultat** sur les livrables contractuels listés §2.
- **Limitation de responsabilité :** plafonnée à {{montant des honoraires des {{X}} derniers mois}}.

## 10. Durée, réversibilité & résiliation
- **Durée :** {{X}} mois reconductibles. 
- **Réversibilité :** à la fin, transfert complet de propriété (cf. dossier de handoff `06`) sous {{X}} jours.
- **Résiliation :** préavis {{30 jours}} ; travaux réalisés dus au prorata.

## 11. Signatures
| Le Client | Le Prestataire |
|-----------|----------------|
| Nom : {{}} | Nom : {{}} |
| Titre : {{}} | Titre : {{}} |
| Date : {{}} | Date : {{}} |
| Signature : | Signature : |

---
**Mini-exemple (hors-scope rempli) :** « Hors-scope : la reprise des 8 ans d'archives papier non numérisées, l'achat des licences Microsoft 365, et tout workflow comptable autre que “extraction de factures” et “relance pièces manquantes”. Toute extension fait l'objet d'un avenant chiffré. »
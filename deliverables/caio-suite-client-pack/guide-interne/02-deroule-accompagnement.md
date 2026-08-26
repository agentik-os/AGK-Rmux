## Playbook interne — Suite CAIO « Agentik OS »

**À qui ce guide s'adresse :** l'équipe Agentik qui DÉLIVRE l'accompagnement entreprise. Pas un deck client — le déroulé opérationnel phase par phase : commandes exactes, inputs, livrables, dossiers de sortie, gates et qui fait quoi.

**La doctrine qui gouverne le parcours** (héritée de `caio-enterprise-workflow-architect`) :

> Entreprise **lisible** → workflows **cartographiés** → données **propres** → automatisations **utiles** → agents **supervisés** → équipe **formée et autonome** → **OS IA d'entreprise mesuré qui tourne et s'améliore**. Dans cet ordre. Jamais l'inverse.

> Un CAIO **ne commence pas par construire des agents. Il commence par rendre l'entreprise lisible.** L'erreur des 80 % : ouvrir par « quelle IA utilisez-vous ? » au lieu de « racontez-moi votre dernier lundi matin ». Et avant même ça, **il qualifie** : un GO / NO-GO honnête (Phase 0) — on n'embarque pas une entreprise qui n'est pas prête, on la perdrait à la livraison.

**Les trois principes non négociables de l'offre** (le « pourquoi » de l'architecture livrée en Phase 4) : **(1) Centralisation** (une seule surface lisible, pas des îlots de SaaS) ; **(2) Interconnexion C-Level** (dashboards fédérés câblés par un contrat d'API inter-dashboard — une métrique COO peut déclencher une alerte CFO) ; **(3) Maîtrise interne** (l'équipe client devient le gardien ; le CAIO **se rend inutile, exprès**).

**Règle de séquence (loi du parcours) :** chaque phase est un prérequis de la suivante — pas de GO de readiness, pas de vente ; pas d'offre verrouillée + mandat signé, pas de découverte ; pas d'interviews, pas de diagnostic ; pas de backlog scoré + arbitrage, pas de build ; pas de système livré, pas de formation ; pas d'adoption + transfert, pas de run. **La chaîne s'auto-câble** (chaque skill lit le livrable de la précédente) ; `/caio-master` ne recâble rien — il **gap-checke** chaque relais avant d'autoriser la suite, puis **route** (voir §9).

---

## 1. Vue d'ensemble — le parcours en 7 phases

| # | Phase | Skill(s) / commande(s) exactes | Livrable de sortie (dossier) | Durée typique |
|---|---|---|---|---|
| 0 | **AI-Readiness** (go/no-go pré-signature) | `/caio-ai-readiness-assessment` (doctrine `marketing-master` : mm-10 primaire + mm-03/02/01) | `./caio-readiness/` — scorecard 9 dimensions, Go-No-Go-Brief (1 p.), Recommended-Engagement, Gap-To-Target | ~1–3 j (appel 30 min + scorecard) |
| 1 | **Offre & Vente** | `offer-and-revenue-architect` + `market-proposal` + doctrine `marketing-master` (mm-04 offre, mm-08 prix, mm-10 vente) | `business-os/` (offre+prix+sell-sheet+unit-economics) + `CLIENT-PROPOSAL.md` (paliers Good-Better-Best + ROI) | 3–7 j jusqu'à signature |
| 2 | **Découverte** | `/caio-discovery-interview` (1 passe/personne) + `consolidate.py` | 1 ZIP standardisé/personne (18 fichiers) + `company-rollup.md/.json` | 1–3 semaines |
| 3 | **Diagnostic + Architecture + Roadmap** | `/caio-enterprise-workflow-architect` | `./company-ai-os/` — 10 livrables (audit, backlog scoré, blueprints, spec dashboard, roadmap 30/60/90, ROI, gouvernance) + `features/F-XXX` | 90 min → 4–12 sem (selon mode) |
| 4 | **Build** (Company-AI-OS fédéré) | `/caio-implementation-runbook` (délègue `agentic-systems-builder` par F-XXX + `agentik-skill-forge`) | `./caio-build/` — 1 serveur dédié + 1 micro-SaaS/C-Level + contrat API inter-dashboard + Composio + rapports automatisés + monitoring + ship-gate ledger | 2–8 semaines/vague |
| 5 | **Formation + Transfert** | `/caio-enablement-and-transfer` | `./caio-enablement/` — onboarding, training, doc interne, Extension-Playbook, Autonomy-Readiness-Gate, Adoption-Tracker | 2–4 semaines |
| 6 | **Run + Optimisation** | `/caio-run-and-optimize` | `./caio-run/` — ROI réel mesuré, monitoring, boucle d'optimisation, QBR, expansion (→ re-rentre dans l'architecte) | Continu (trimestriel) |

Orchestration globale : **`/caio-master`** (un seul Workflow au sommet, pattern `marketing-master`) → écrit `./caio-engagement/` (plan d'engagement + tracker de phases) — voir §9.

---

## 2. Phase 0 — AI-READINESS (go/no-go pré-signature)

**Objectif.** Qualifier **avant** de vendre. Un appel de ~30 min produit un verdict honnête **GO / NOT-YET / REDIRECT** + un investissement indicatif. Seul un **GO** ouvre la suite — signer une entreprise non prête, c'est la perdre à la livraison (un client raté coûte plus qu'un prospect refusé). C'est la **porte d'entrée du parcours**, avant l'offre.

**Skill / commande :** `/caio-ai-readiness-assessment`. La skill **EST** l'appel de vente diagnostic de `marketing-master` mm-10 (diagnostic avant pitch, disqualification honnête, banque d'objections *feel-felt-found*). Elle score **9 dimensions de maturité**, applique le scoring **4-forces** de mm-03 — `(Push + Pull) > (Anxiety + Habit)` est une **condition de GO** —, positionne l'offre (mm-02) contre les deux alternatives (empilement de SaaS génériques, agence boîte-noire), et ancre le « pourquoi maintenant » sur la fenêtre 2026-2027 (mm-01), sans fausse urgence.

**Inputs requis :** contexte entreprise (secteur, taille, C-levels) ; ce que le C-level dit sur l'appel ; un scan rapide du site public. Optionnellement le `vision-os/` du CAIO. **Jamais** le rollup de découverte ni `company-ai-os/` — ils n'existent pas encore.

**Livrable produit :** `./caio-readiness/` — `AI-Readiness-Scorecard.md` (9 dimensions), `Go-No-Go-Brief.md` (1 page exec), `Recommended-Engagement.md` (palier + investissement indicatif), `Gap-To-Target-Plan.md`, `metadata.json`.

**GATE pour passer en Phase 1 :**
- Verdict **GO** — sinon la suite ne s'ouvre pas. **NOT-YET** → repart avec le `Gap-To-Target-Plan`, re-qualif à 30–90 j. **REDIRECT** → l'alternative nommée (SaaS point / data engineer / recrutement / partenaire conformité) ; un REDIRECT honnête vaut mieux qu'une mission qui échoue (mm-10).
- Conditions non négociables : `(Push + Pull) > (Anxiety + Habit)` ; un **sponsor exécutif identifiable**.

**Durée :** appel ~30 min + scorecard → verdict en ~1–3 j.

**Qui :** côté Agentik — le **CAIO** (une consultation, pas un pitch). Côté client — le **C-level** qui exprime l'intérêt (futur sponsor).

---

## 3. Phase 1 — OFFRE & VENTE

**Objectif.** Sur un **GO**, transformer la capacité Agentik en **une offre productisée, un prix unique défendable, une proposition signable**. Pas de découverte avant mandat signé + acompte : la découverte coûte du temps de consultant, elle se vend. La propale incorpore le palier indicatif de `./caio-readiness/Recommended-Engagement.md`.

**Skills / commandes (dans l'ordre) :**
1. `offer-and-revenue-architect` — UNE offre, UN prix, sell-sheet 1 page, unit-economics (mode `full-business-os` ou `price-lock`).
2. `market-proposal` — la proposition : executive summary, situation, stratégie phasée, scope, **paliers Good-Better-Best**, projection ROI, case studies, objections → `CLIENT-PROPOSAL.md` (la main que reçoit le GO de la Phase 0 pour générer le SOW signé).
3. Doctrine `marketing-master` en lentilles : `mm-04` (offre/copy), `mm-08` (pricing = positionnement), `mm-10` (vente B2B, objections, follow-up).

**Inputs requis :** le GO + `Recommended-Engagement.md` de la Phase 0 ; capacité Agentik à packager ; taux horaire effectif réel ; capacité de delivery (h/sem) ; réseau chaud pour les 3 premières ventes. Côté prospect : secteur/business model, situation actuelle, pains, objectifs chiffrés, budget, timeline, décideurs, **LTV d'un client** (ancre du ROI).

**Livrable produit :** `./business-os/` (Offer-Architecture, Pricing-Model, Sell-Sheet, Unit-Economics) + `CLIENT-PROPOSAL.md` (3 paliers, ROI ancré sur les mêmes chiffres que la section Investment, valable 30 j).

**GATE pour passer en Phase 2 :**
- **UNE** offre, pas un menu ; **UN** prix unique, jamais une fourchette (interdit par Iron Law 2).
- `single_price ≥ 5 × taux horaire effectif × heures` ET marge de contribution ≥ 70 %.
- Proposition **signée** + **acompte encaissé** (50 %) + **sponsor exécutif confirmé**.
- *Garde-fou :* le pack n'est « prouvé » qu'après **3 ventes au plein prix sans remise**.

**Durée :** offer-extract 1–2 h, price-lock 45–90 min, propale quelques heures → ~3–7 j jusqu'à signature.

**Qui :** côté Agentik — founder/sales. Côté client — l'**economic buyer** (CEO/CFO), futur sponsor exécutif.

---

## 4. Phase 2 — DÉCOUVERTE

**Objectif.** Rendre chaque employé **lisible dans SON langage métier**, avant toute techno. On capture le travail réel — semaine type, tâches répétitives (temps × fréquence), handoffs, outils, shadow IT, frictions verbatim, écart actuel→idéal — pas « quelle IA utilisez-vous ».

**Skill / commande :** `/caio-discovery-interview`, **une passe par personne** (C-level puis managers, élargir ensuite). Boot : langue → cadre rassurant (« on retire le pénible, pas la personne ») → consentement + niveau de partage → carte d'identité → scan du site → routage par famille de poste → marche en 14 chapitres, une question à la fois. Puis **`scripts/build_bundle.py`** (1 ZIP/personne) et, plusieurs ZIP empilés, **`scripts/consolidate.py`** (vue entreprise).

**Inputs requis :** liste des personnes (org chart + scope du palier vendu) ; URL du site ; consentement de chacun ; ~15–25 min/personne.

**Livrable produit :** par personne, **1 ZIP standardisé identique** — 18 fichiers (`metadata.json`, `company-context.md`, `summary.md`, `00-identity` → `13-gap-analysis`, `transcript.md`) ; `metadata.json` indexe outils, frictions, `ai_appetite` (champion/neutre/sceptique), gros titre du gap. Puis `company-rollup.md/.json` : outils par fréquence, IA + shadow IT, frictions par famille de poste, carte handoffs/reporting, spread actuel→idéal.

**GATE pour passer en Phase 3 :**
- **≥ 10 interviews verbatim** pour un `full-company-workflow-audit` (l'architecte **REFUSE** sous 10 ; sinon `department-discovery`).
- Chaque ZIP passe ses discipline checks : ≥ 1 tâche répétitive avec temps × fréquence, handoffs capturés, shadow IT demandé, ≥ 1 verbatim dans `08-frictions`, gap rempli, recap validé avant export, **rien d'inventé**.
- `company-rollup.md` généré (sinon on a des dossiers, pas un diagnostic).

**Durée :** 15–25 min/personne ; 1–3 semaines pour 10–40 personnes (parallélisable — un interviewer ≠ un fichier partagé, R-SCOPE).

**Qui :** côté Agentik — l'interviewer (la skill, pilotée par un consultant), neutre, jamais en vente. Côté client — chaque employé, un par un.

---

## 5. Phase 3 — DIAGNOSTIC + ARCHITECTURE + ROADMAP

**Objectif.** Transformer les N dossiers en **un Company AI OS** : audit, cartographie des workflows, backlog **scoré sur 10 critères**, 8 verdicts, blueprints agentiques, spec dashboard, roadmap 30/60/90, ROI par workflow, gouvernance + risques.

**Skill / commande :** `/caio-enterprise-workflow-architect`. Choisir le **mode** (`quick-executive-audit` 90 min / `department-discovery` 1–2 sem / `full-company-workflow-audit` 4–12 sem / `dashboard-architecture` / `implementation-roadmap`). Multi-départements : orchestration Workflow interne (un sous-agent par cluster, fichiers disjoints, vérif 2-de-3 avant qu'une opportunité entre au backlog, synthèse par toi).

**Inputs requis :** les ZIP + `company-rollup` de la Phase 2 ; taille/secteur/stack (CRM, support, comms, docs, PM, finance, HR) ; contraintes réglementaires (RGPD/SOC2/HIPAA…) ; sponsor exécutif + veto IT/sécurité ; objectif business principal.

**Livrable produit :** `./company-ai-os/` — **10 fichiers** : `00-Executive-Summary`, `01-Stakeholder-Interview-Plan`, `02-Role-And-Workflow-Inventory`, `03-Tool-And-Integration-Map`, `04-Data-And-Permission-Map`, `05-Automation-Opportunity-Backlog` (scoring /100 + verdict), `06-Agentic-System-Blueprints`, `07-Dashboard-Feature-Specs`, `08-Implementation-Roadmap` (30/60/90 + coût + équipe + stack), `09-ROI-Governance-And-Risks`, + `features/F-XXX-*.md` par feature prioritaire.

**GATE pour passer en Phase 4 (discipline checks non négociables) :**
- Chaque opportunité porte ses **11 champs Atomic Insight** dont **≥ 1 verbatim de douleur avec source+timestamp** (sinon REFUSÉ).
- Tout chiffre ROI ancré dans `heures × coût chargé × fréquence`.
- Verdicts : 5 « build now » + 3 « not yet » (data-cleanup / executive-decision / **REFUSED** classe 8 sur toute décision RH/légale/financière/publique sensible — non négociable).
- Chaque feature spec : 12 champs ; chaque agent sur décision sensible a un **HITL** explicite ; dashboard exposant sources/logs/statut/erreurs/coûts/confiance (pas de boîte noire).
- Roadmap 30/60/90 avec **coût + ROI + payback par phase** ; executive summary lisible en < 5 min par un CEO non-technique.
- **Arbitrage sponsor :** quelles opportunités (≥ 80/100 d'abord) entrent en build. Sans arbitrage signé, pas de build.

**Durée :** 90 min (quick-executive) à 4–12 semaines (full-company) ; `implementation-roadmap` seul ~1 semaine.

**Qui :** côté Agentik — le CAIO architecte. Côté client — chefs de département (validation), sponsor (arbitrage), IT/sécurité (veto).

---

## 6. Phase 4 — BUILD (Company-AI-OS fédéré)

**Objectif.** Passer du backlog scoré + blueprints à un **OS IA d'entreprise live, centralisé et fédéré** — pas une collection d'automatisations éparses. Les **trois principes** de l'offre (Centralisation / Interconnexion C-Level / Maîtrise interne — voir intro) gouvernent le build. Distinction stricte : **design ≠ écriture du code**, sous **un seul propriétaire de phase**.

**Skill / commande :** `/caio-implementation-runbook` — **le propriétaire canonique du build**. Elle **réalise** le blueprint de `./company-ai-os/` dans la topologie cible, puis la **construit** avec une discipline de **ship-gate par livrable** : **1 serveur dédié possédé par le client** + **1 micro-SaaS par C-Level** + le **contrat d'API inter-dashboard** (une métrique d'un dashboard déclenche une action dans un autre) + les **connecteurs Composio** + les **rapports automatisés** + le **monitoring intégré** (logs/coûts/confiance/statut/erreurs). Elle **délègue, ne ré-implémente pas** (R-KARPATHY) : chaque spec `F-XXX` part à `agentic-systems-builder` (mission, inputs, tools, **approval gates HITL** sur toute action irréversible — email, publish, paiement, suppression, légal, santé, finance, RH —, MVP + V2), les skills récurrentes du client codifiées via `agentik-skill-forge`. Elle produit d'abord l'`Architecture-Realization` (design gaté), **puis** le code.

> ⚠️ **Routage corrigé.** Le build n'est **plus** routé vers un pipeline généraliste : `/caio-implementation-runbook` est le propriétaire dédié de la phase, réalise la topologie fédérée et tient les ship-gates ; la vérification golden-path est **intégrée à ses ship-gates par livrable**, pas un gate séparé.

**Inputs requis :** `company-ai-os/05` + `06` + `07` (critères d'acceptation = **source de chaque ship-gate**) + `08` + `09` + `features/F-XXX-*.md` ; optionnellement `company-rollup.md`. Accès client aux intégrations (clés API via Composio), infra du serveur dédié, contraintes data (résidence, RGPD, audit logging) du `04-Data-And-Permission-Map`.

**Livrable produit :** `./caio-build/` — `Architecture-Realization` (design gaté), runbook serveur, plans micro-SaaS par C-Level, **contrat d'API inter-dashboard**, guide Composio, specs rapports automatisés, **guide monitoring/instrumentation** (pose la **baseline t0** — mm-11), **registre des ship-gates**, **plan de communication sponsor** (mm-04 — canaliser le désir du sponsor signé, pas de fausse urgence), journal de build, `metadata.json`. Concrètement : système **live** (dashboards fédérés, agents supervisés en prod) avec **sa preuve golden-path par livrable**.

**GATE pour passer en Phase 5 :**
- **Ship-gate vert par livrable** — preuve golden-path (route + render + **écriture persistée réelle**) ; « ça build » ≠ « ça marche » (L1). Aucun livrable shippé sans son ship-gate.
- **Topologie fédérée réalisée** : serveur dédié debout, ≥ 1 micro-SaaS/C-Level opérationnel, **contrat d'API inter-dashboard câblé** (sinon dashboards isolés, pas un OS fédéré).
- Tout **HITL** de la Phase 3 câblé (email/publish/paiement/légal → approval gate ; **classe-8 REFUSED** respectée).
- **Baseline t0 posée** — sinon la Phase 6 ne peut pas mesurer un ROI réel.
- **Iron test build :** top-opportunité **shippée en prod**, quick wins 30 j livrés, sponsor a reçu le **dashboard MVP**.
- L0 : `git clone && install` reproduit la livraison, poussé. Secrets hors repo (`~/.omega`).

**Durée :** réalisation d'architecture quelques jours ; V1/agent 1–3 h, V2 ~2 sem ; 1 vague = 2–8 semaines.

**Qui :** côté Agentik — `/caio-implementation-runbook` + ses délégués (`agentic-systems-builder`, `agentik-skill-forge`). Côté client — IT (creds/intégrations/serveur), data owner (permissions).

---

## 7. Phase 5 — FORMATION + TRANSFERT

**Objectif.** Faire **adopter** le système ET **transférer la maîtrise** : sans adoption, pas de ROI ; sans transfert, le client reste dépendant du CAIO (contredit le principe 3). Deux temps : **enablement** (onboarding, formation, validation des premiers cas d'usage en conditions réelles) puis **transfert** (l'équipe ajoute un agent / connecte un outil / ajuste un rapport, sans aide).

**Skill / commande :** `/caio-enablement-and-transfer`. Lit `./caio-build/` (dashboards live, runbooks, pointeurs code/config, carte des secrets, preuves golden-path) + `company-ai-os/02/06/07` + les dossiers de découverte (chapitre 7 IA/shadow-IT + `ai_appetite` champion/neutre/sceptique, pour cibler par profil). Exécute curricula par famille de poste, onboarding, **runbooks** (agent en échec, lecture logs/coûts/confiance, qui approuve quoi), transfert de propriété.

**Inputs requis :** le système live de la Phase 4 + son monitoring ; les **champions** identifiés en découverte (chapitre 7) ; les owners désignés par département.

**Livrable produit :** `./caio-enablement/` — **Enablement** : `01-Onboarding-Session-Plans`, `02-Internal-Documentation-Pack`, `03-End-User-Training-Curriculum`, `04-Validated-Use-Cases-Log`. **Transfert** : `05-Extension-Playbook` (add-agent / connect-tool / adjust-report, **dimensionné au niveau technique réel** — mm-12 : config → flux guidé → pointeur code), `06-Ownership-Handover-Checklist`, `07-Autonomy-Readiness-Gate`, `08-Adoption-Tracker`. Plus `00-Enablement-Summary` + `metadata.json`.

**GATE pour passer en Phase 6 :**
- L'équipe **utilise le système en production** (usage réel, pas pilote) — adoption = rétention avant expansion (mm-11 : cas d'usage prouvés, pas un nombre de « formés »).
- **Autonomy-Readiness Gate franchi** : le transfert n'est complet que quand l'équipe exécute le **vrai geste sans aide** en conditions réelles (Savoir ≠ Pouvoir — Kotter/ADKAR).
- Doc interne + `Extension-Playbook` remis ; ≥ 1 **champion par département** capable d'opérer sans Agentik.
- Propriété transférée (un owner par agent/workflow), budgets et escalades documentés.

**Durée :** 2–4 semaines.

**Qui :** côté Agentik — enablement lead. Côté client — champions/owners par département, sponsor (sponsorise l'adoption).

---

## 8. Phase 6 — RUN + OPTIMISATION

**Objectif.** Exploiter le système live, **prouver le ROI réel** (mesuré, pas projeté), maintenir la santé, faire tourner la boucle d'optimisation, compounder par land-and-expand — en retainer **léger**.

**Skill / commande :** `/caio-run-and-optimize`. Mesure le **ROI réel post-go-live vs la projection de l'architecte**, depuis la **télémétrie + les reçus** (jamais inventé — mm-11 : UNE North Star Metric = valeur reçue, pas activité ; ROI par cohorte de go-live ; rétention des économies AVANT toute expansion). Monitore la santé, fait tourner la boucle hebdo/mensuelle, opère le **quota stratégique 1h/sem** (léger par design — mm-08 : un retainer lourd contredirait la promesse de transfert ; dépassements = mini-engagements scopés), pilote rétention + land-and-expand (mm-09 : département / C-Level suivant / client-comme-référence-interne, pas du pipeline perso).

**Inputs requis :** `./caio-enablement/06/08/04` ; `./company-ai-os/09` + `05` (ROI projeté + backlog scoré + matrice gouvernance/HITL) ; le guide monitoring/instrumentation de `./caio-build/` (télémétrie, baseline t0) ; télémétrie live, reçus de coûts modèles, feuilles de temps/factures.

**Livrable produit :** `./caio-run/` — `ROI-Measurement-Model`, `Monitoring-Health-Spec`, `Optimization-Loop-Cadence`, `Weekly-Quota-Agenda`, `Quarterly-Business-Review`, `Expansion-And-Referral-Play`, `metadata.json`.

**GATE (boucle continue — iron test à 90 j) :**
1. Top-opportunité shippée en prod ? 2. Quick wins 30 j (3/3) ? 3. Dashboard MVP reçu ? 4. Agent adopté (usage prod, `Adoption-Tracker`) ? 5. ROI **tient à la ré-mesure** depuis télémétrie + reçus ?
- **≥ 4/5 → renouveler + scaler.** **< 3/5 → ré-auditer** (mauvaises opportunités ou gouvernance sautée), pas re-coder à l'aveugle.
- Sur verdict **« Expand »** → la chaîne **re-rentre dans `/caio-enterprise-workflow-architect`** (Phase 3) pour la vague suivante ; expansion commerciale → `market-proposal` (SOW d'expansion).
- Iron test 12 mois : de « on utilise ChatGPT parfois » à « N agents en prod, M dashboards **fédérés**, K h/sem économisées **vérifiées**, $X de ROI réel » — et une 2e vague démarre **sans le CAIO d'origine** = OS auto-compoundant (principe 3 tenu).

**Durée :** continu ; QBR trimestriel ; vagues d'amélioration à la demande.

**Qui :** côté Agentik — account/run lead (1h/sem). Côté client — ops owner, sponsor (QBR).

---

## 9. Orchestration par `/caio-master` (gap-check de chaque relais → `./caio-engagement/`)

`/caio-master` est l'**apex du parcours**, calqué sur `marketing-master` : **un seul Workflow au sommet** qui pilote le parcours en une passe gap-checkée et auto-corrective, et écrit **`./caio-engagement/`** (plan d'engagement + tracker de phases). Il **ne construit rien** — il route. La chaîne s'auto-câble (`/caio-implementation-runbook` lit `./company-ai-os/` ; `/caio-enablement-and-transfer` reçoit `./caio-build/` ; `/caio-run-and-optimize` lit `./caio-enablement/` et reboucle vers `/caio-enterprise-workflow-architect`) ; `caio-master` ne recâble pas — il **vérifie** que le livrable réel de chaque phase existe et suffit **avant** d'autoriser la suivante.

**Pourquoi un Workflow et pas un `/goal` géant :** R-GOAL — `/goal` est un thermostat à une condition, jamais une campagne. Une mission CAIO = un **Workflow dynamique de petits goals par étape** (loop-until-dry / -count par phase). R-ORCH : on parallélise le file-disjoint, on sérialise ce qui partage un fichier (R-SCOPE).

**Protocole (par relais de phase) :**
1. **Load.** Charger les assets réels de la phase précédente (`./caio-readiness/`, `./business-os/`, les ZIP, `./company-ai-os/`, `./caio-build/`, `./caio-enablement/`, `./caio-run/` ; l'URL prod via Playwright, jamais un MCP browser — R-TEST/R-BROWSER). L1 : on note la réalité runtime.
2. **Gap-check.** Un agent **lit le SKILL.md de la phase comme rubrique** (discipline checks + iron test) et score : `{ phase, aligned, score, gaps[], corrections[] }` avec gaps concrets (verbatim manquant, ROI non sourcé, HITL absent, ship-gate rouge, API inter-dashboard non câblée). L'agent **lit la doctrine, il n'est pas lui-même un Workflow** (pas d'imbrication).
3. **Correct.** Appliquer/proposer le fix au livrable (un seul writer par fichier).
4. **Vérifier adversarialement (2-de-3 skeptics, R-VERIFY).** Skeptic preuve (verbatim réel ? ship-gate vert ?), skeptic chiffres (ROI = h×coût×fréquence depuis télémétrie ?), skeptic intervention/sécurité (bon verdict, classe-8 REFUSED, HITL ?). Consensus **2-de-3** ; le « done » d'un sous-agent est un input, jamais le verdict.
5. **Loop-until-dry** sur les phases à taille inconnue (découverte, backlog, vagues de build) ; passer à la suivante **seulement si son GATE est vert**. Séquence : **readiness → offre → découverte → diagnostic → build → formation → run**.
6. **Report.** Synthèse par toi (jamais le copier-coller d'un sous-agent), R-CITE : chaque verdict porte sa citation (fichier:ligne, log, capture). Plan d'engagement + tracker → `./caio-engagement/`.

`/caio-master` **respecte chaque gate** avant de débloquer la suivante et **reboucle** sur la phase qui échoue son iron test plutôt que d'avancer sur une base cassée — le filet adversarial qui empêche de « shipper des agents dans le chaos », et le GO de readiness qui empêche d'embarquer une entreprise non prête.

---

## 10. Tableau récap — parcours → skill → livrable → gate

| Phase | Objectif | Skill / commande exacte | Livrable (dossier) | GATE de sortie |
|---|---|---|---|---|
| 0 AI-Readiness | Qualifier GO/NO-GO avant de vendre | `/caio-ai-readiness-assessment` (mm-10 primaire + mm-03/02/01) | `./caio-readiness/` (scorecard 9-dim, Go-No-Go-Brief, Recommended-Engagement, Gap-To-Target) | **Verdict GO** (sinon NOT-YET + plan d'écart / REDIRECT honnête) ; sponsor identifiable ; (Push+Pull)>(Anxiety+Habit) |
| 1 Offre & Vente | Offre + prix + propale signable | `offer-and-revenue-architect`, `market-proposal`, doctrine `marketing-master` (mm-04/08/10) | `business-os/` + `CLIENT-PROPOSAL.md` (G-B-B, ROI) | 1 offre, 1 prix unique, marge ≥70 %, **propale signée + acompte + sponsor confirmé** |
| 2 Découverte | Chaque employé lisible dans son langage | `/caio-discovery-interview` + `consolidate.py` | 1 ZIP/personne (18 fichiers) + `company-rollup` | **≥10 interviews verbatim**, checks ZIP passés, rollup généré |
| 3 Diagnostic+Archi+Roadmap | Company AI OS scoré + roadmap + ROI | `/caio-enterprise-workflow-architect` | `./company-ai-os/` (10 livrables + F-XXX) | 11 champs+verbatim/opportunité, ROI=h×coût×fréq, HITL+classe-8, roadmap chiffrée, **arbitrage sponsor** |
| 4 Build (fédéré) | OS IA live, centralisé, fédéré | `/caio-implementation-runbook` (+ `agentic-systems-builder` par F-XXX, `agentik-skill-forge`) | `./caio-build/` (serveur dédié + micro-SaaS/C-Level + API inter-dashboard + Composio + monitoring + ship-gate ledger) | **Ship-gate vert par livrable**, topologie fédérée réalisée, HITL câblé, baseline t0 posée, top-opportunité en prod |
| 5 Formation+Transfert | Adoption + transfert de maîtrise | `/caio-enablement-and-transfer` | `./caio-enablement/` (onboarding, training, doc, Extension-Playbook, Adoption-Tracker) | **Usage prod réel**, **Autonomy-Readiness-Gate franchi**, doc+playbook remis, propriété transférée |
| 6 Run+Optimisation | ROI réel prouvé, scaling | `/caio-run-and-optimize` | `./caio-run/` (ROI mesuré, monitoring, QBR, Expansion-Play) | **ROI réel ≥ promis** (télémétrie+reçus), iron test ≥4/5 ; sur « Expand » → re-rentre dans l'architecte |
| ⟳ Orchestration | Gap-check de chaque relais | `/caio-master` | `./caio-engagement/` (plan d'engagement + tracker de phases) | gap-check **≥2/3** de chaque relais avant la phase suivante |

---

## 11. Anatomie de la suite (à connaître avant de délivrer)

**6 skills de phase + 1 orchestrateur + 2 skills commerciales routées** — la chaîne canonique du pack.

- **4 skills de phase dédiées :** `/caio-ai-readiness-assessment` (Phase 0 → `./caio-readiness/`), `/caio-implementation-runbook` (Phase 4 → `./caio-build/`), `/caio-enablement-and-transfer` (Phase 5 → `./caio-enablement/`), `/caio-run-and-optimize` (Phase 6 → `./caio-run/`).
- **2 skills de phase réutilisées** (existantes, non ré-implémentées — R-KARPATHY) : `/caio-discovery-interview` (Phase 2 → 1 ZIP/personne + `company-rollup`), `/caio-enterprise-workflow-architect` (Phase 3 → `./company-ai-os/`).
- **1 orchestrateur :** `/caio-master` — pilote le parcours de bout en bout (pattern `marketing-master`), gap-checke chaque relais (≥ 2/3), écrit `./caio-engagement/`. Il **route, il ne construit pas**.
- **Couche commerciale routée** (zéro ré-implémentation) : `offer-and-revenue-architect` + `market-proposal`, nourris par la doctrine `marketing-master` (mm-04 offre / mm-08 prix / mm-10 vente).
- **Délégués de build :** `agentic-systems-builder` (un dispatch par spec `F-XXX`), `agentik-skill-forge` (codification des skills récurrentes du client).

> **Aucun saut de gate.** Chaque phase est un prérequis de la suivante (doctrine de l'intro) ; la chaîne s'auto-câble ; `/caio-master` gap-checke chaque relais avant d'autoriser la suite. Et tout en amont, le **GO de readiness** (Phase 0) : on n'embarque que les entreprises prêtes.

---

**--- Resume :** Playbook interne CAIO réécrit sur la suite canonique en **7 phases** (0 AI-Readiness go/no-go → 1 Offre & Vente → 2 Découverte → 3 Diagnostic+Archi+Roadmap → 4 Build fédéré → 5 Formation+Transfert → 6 Run+Optimisation), chaque phase avec objectif, skill(s) exacte(s), inputs, livrable (+ dossier `./caio-*/`), gate, durée et acteurs ; le build est piloté par `/caio-implementation-runbook` (topologie fédérée : 1 serveur + 1 micro-SaaS/C-Level + API inter-dashboard + Composio + monitoring + ship-gates, avec `agentic-systems-builder`/`agentik-skill-forge` en délégués) — plus de pipeline généraliste ; orchestration par `/caio-master` (Workflow gap-checké 2-de-3, écrit `./caio-engagement/`).

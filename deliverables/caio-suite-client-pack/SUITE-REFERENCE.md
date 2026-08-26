# Carte de la suite CAIO — parcours → skill → livrable → gate

La suite CAIO accompagne une entreprise classique, de bout en bout, pour devenir *AI-native*. Elle se
compose de **6 skills de phase** (4 dédiées + 2 réutilisées) qui s'enchaînent en une **chaîne
auto-câblée** (chaque skill lit le livrable de la précédente et passe la main à la suivante), plus un
**orchestrateur** (`caio-master`) qui pilote tout le parcours en une passe gap-checkée. La couche
commerciale (offre, prix, proposition) est **routée** vers des skills existantes — jamais ré-implémentée
(R-KARPATHY).

| # | Phase | Skill qui l'exécute | Type | Écrit | Gate pour avancer |
|---|-------|--------------------|------|-------|-------------------|
| 0 | **AI-Readiness** (go/no-go pré-signature) | `/caio-ai-readiness-assessment` | suite | `./caio-readiness/` | verdict **GO** (sinon NOT-YET / REDIRECT honnête) |
| 1 | **Offre & vente** | `offer-and-revenue-architect` + `market-proposal` (+ doctrine `marketing-master` mm-04/08/10) | **routé** | offre, prix, sell-sheet, proposition (paliers Good-Better-Best), projection ROI | proposition signée + sponsor confirmé |
| 2 | **Découverte** | `/caio-discovery-interview` | suite (existant) | 1 ZIP standardisé par employé | entretiens capturés (verbatim) |
| 3 | **Diagnostic + Architecture + Roadmap** | `/caio-enterprise-workflow-architect` | suite (existant) | `./company-ai-os/` (10 livrables : backlog scoré, blueprints, spec dashboard, roadmap 30/60/90, ROI, gouvernance) | backlog priorisé + roadmap validés |
| 4 | **Build** (Company-AI-OS fédéré) | `/caio-implementation-runbook` | suite | `./caio-build/` (1 serveur client + 1 micro-SaaS par C-Level, contrat API inter-dashboard, connecteurs Composio, ship-gates) | ship-gates par livrable verts |
| 5 | **Formation + Transfert** | `/caio-enablement-and-transfer` | suite | `./caio-enablement/` (onboarding, training, doc interne, transfert de maîtrise) | adoption mesurée + handover de propriété accepté |
| 6 | **Run + Optimisation** | `/caio-run-and-optimize` | suite | `./caio-run/` (ROI réel mesuré, monitoring, boucle d'optimisation, QBR, rétention/expansion) | KPIs vs promis tenus ; expansion → re-rentre dans l'architect |
| ⟳ | **Orchestration de tout le parcours** | `/caio-master` | suite (orchestrateur) | `./caio-engagement/` (plan d'engagement + tracker de phases) | gap-check de chaque relais (≥2/3) |

> **La chaîne s'auto-câble.** Chaque skill de phase lit le livrable de la précédente (`caio-implementation-runbook`
> lit `./company-ai-os/` ; `caio-enablement-and-transfer` reçoit le système livré ; `caio-run-and-optimize` lit
> `./caio-enablement/` et reboucle vers l'architect). `caio-master` ne recâble pas ces relais — il **gap-checke**
> que le livrable réel de chaque phase existe et est suffisant avant d'autoriser la suivante, puis **route**.

---

## Anatomie de la suite

**6 skills de phase + 1 orchestrateur + 2 skills commerciales routées.**

- **4 skills de phase dédiées** (le cœur de l'accompagnement) :
  `caio-ai-readiness-assessment` (front gate go/no-go), `caio-implementation-runbook` (build de l'OS
  fédéré), `caio-enablement-and-transfer` (adoption + transfert), `caio-run-and-optimize` (run + ROI réel).
- **2 skills de phase réutilisées** (existantes, non ré-implémentées) :
  `caio-discovery-interview` (entretiens) + `caio-enterprise-workflow-architect` (audit → architecture →
  roadmap → ROII → gouvernance, écrit `./company-ai-os/`).
- **1 orchestrateur** : `caio-master` — pilote le parcours de bout en bout (pattern *marketing-master* :
  un Workflow au sommet qui applique chaque skill de phase comme exécuteur, gap-checke chaque relais,
  vérifie en adversarial ≥2/3, et livre **un plan d'engagement + un tracker de phases**). Il ne construit
  rien lui-même — il **route**.
- **Couche commerciale routée** (R-KARPATHY, zéro ré-implémentation) : l'offre, le prix et la proposition
  passent par `offer-and-revenue-architect` + `market-proposal`, eux-mêmes nourris par la doctrine
  `marketing-master` (mm-04 offre / mm-08 prix / mm-10 vente).

**Le principe.** *Entreprise lisible → workflows cartographiés → données propres → automatisations utiles →
agents supervisés → équipe formée et autonome → OS IA mesuré qui tourne et s'améliore.* Chaque phase est un
prérequis de la suivante ; aucun saut de gate.

---

*Carte de la suite CAIO · OmegaOS · 2026-06-28.*

# Pack CAIO — Suite d'accompagnement « Chief AI Officer / Company AI OS »

**Offre Agentik OS** — accompagner une entreprise classique, de bout en bout, pour devenir *AI-native*.

Ce pack est le **kit interne de l'équipe Agentik** qui délivre l'accompagnement CAIO : il contient le
guide opérationnel (onboarding, déroulé de l'accompagnement, manuel de l'offre) et les templates
réutilisables d'un engagement. Les PDF sont générés via le moteur branded OmegaOS (`pdfgen`).

> **À qui ça sert.** Au lead CAIO et à l'équipe de delivery. Ce n'est pas un document marketing : c'est
> le playbook qui transforme une promesse commerciale en machine d'exécution reproductible.

---

## Contenu du pack

```
caio-suite-client-pack/
├── README.md                      ← ce fichier (index + carte de la suite)
├── SUITE-REFERENCE.md             ← la carte parcours → skill → livrable → gate (1 page)
├── guide-interne/
│   ├── 01-onboarding.md / .pdf            Embarquer un client (1er contact → J+1)
│   ├── 02-deroule-accompagnement.md / .pdf  Le déroulé phase par phase (LE cœur)
│   └── 03-manuel-offre.md / .pdf          L'offre : paliers, prix, valeur, ROI, vente
├── templates/                     ← 7 templates prêts à remplir
│   ├── 01-plan-engagement-caio.md
│   ├── 02-proposition-commerciale.md
│   ├── 03-sow-contrat-cadre.md
│   ├── 04-plan-formation-par-role.md
│   ├── 05-qbr.md
│   ├── 06-handoff-acceptation.md
│   └── 07-business-case.md
└── skills-reference/              ← copie des SKILL.md de la suite (référence)
    ├── caio-master/                   (orchestrateur de bout en bout)
    ├── caio-ai-readiness-assessment/  (P0 — go/no-go)
    ├── caio-discovery-interview/      (P2 — réutilisé)
    ├── caio-enterprise-workflow-architect/ (P3 — réutilisé)
    ├── caio-implementation-runbook/   (P4 — build fédéré)
    ├── caio-enablement-and-transfer/  (P5 — formation + transfert)
    └── caio-run-and-optimize/         (P6 — run + optimisation)
```

---

## La suite CAIO en une phrase

> *Une entreprise lisible → des workflows cartographiés → des données propres → des automatisations
> utiles → des agents supervisés → un tableau de bord mesurable → un OS IA d'entreprise qui tourne.*

Le parcours est **orchestré par `/caio-master`** (pattern *marketing-master* : un Workflow au sommet qui
route chaque phase vers la skill qui l'exécute, gap-checke le passage de relais, et livre un plan
d'engagement). Voir `SUITE-REFERENCE.md` pour la carte complète.

**La suite en bref.** **6 skills de phase** (4 dédiées — AI-readiness, build/implementation-runbook,
formation/transfert, run/optimisation — + 2 réutilisées : découverte, architecture) enchaînées en une
**chaîne auto-câblée**, plus l'**orchestrateur** `/caio-master` qui gap-checke chaque relais et route tout
le parcours. La couche commerciale (offre, prix, proposition) est **routée** vers `offer-and-revenue-architect`
+ `market-proposal` — jamais ré-implémentée (R-KARPATHY).

---

## Comment utiliser ce pack

1. **Nouveau prospect ?** → `guide-interne/01-onboarding.md` (qualifier, découverte, cadrer le palier).
2. **Engagement signé ?** → `guide-interne/02-deroule-accompagnement.md` + lancer `/caio-master`.
3. **Construire/ajuster l'offre ?** → `guide-interne/03-manuel-offre.md`.
4. **Produire un document client ?** → `templates/` (proposition, SOW, plan d'engagement, QBR, handoff…).

Les commandes citées (`/caio-master`, `/caio-discovery-interview`, `/caio-enterprise-workflow-architect`,
`/caio-enablement-and-transfer`, `/caio-run-and-optimize`, et les skills routées) sont installées avec
OmegaOS. Régénérer un PDF : `omega pdf --template=doc --data=<json> --out=<chemin.pdf>`.

---

*Pack généré par OmegaOS · Suite CAIO · 2026-06-28 · PDF via `pdfgen` (R-PDF).*

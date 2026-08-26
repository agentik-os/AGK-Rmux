# Manuel de l'offre — La Suite CAIO

**Pack :** offre entreprise « Agentik OS » (accompagnement CAIO end-to-end).
**Lecteur :** l'équipe Agentik qui **vend et délivre** l'accompagnement. Document **interne**. Certains blocs (sell-sheet, paliers Good-Better-Best, projection ROI) se transposent client-facing via `/offer-and-revenue-architect` et `/market-proposal` — ce manuel est la doctrine en amont, pas la propale.
**Doctrine appliquée :** Hormozi (équation de valeur), pricing à la valeur (mm-08), offre irrésistible (mm-04), vente B2B 5 étapes (mm-10). Orchestration : `offer-and-revenue-architect` + `market-proposal`.

> Règle d'or de tout le manuel (Iron Law 4 d'`offer-and-revenue-architect`) : **on vend un résultat dans le business du client, jamais des heures de consultant.** L'heure est notre coût ; elle n'est jamais la valeur de l'acheteur.

> ⚠️ Tous les chiffres de prix, de valeur et de ROI ci-dessous sont des **ordres de grandeur de cadrage** — des gabarits à recalculer sur les vrais nombres de chaque prospect (R-CITE : aucune valeur acheteur n'est inventée, elle se chiffre en découverte). Ils donnent la fourchette défendable, pas un tarif gravé.

---

## 1. Positionnement — ce qu'on vend, contre quoi on se positionne

**La phrase de positionnement (format Dunford, mm-04) :**

> Pour une **entreprise établie** (PME / ETI, business classique) qui sait que l'IA va la déclasser mais ne sait **ni par où commencer ni quoi industrialiser**, **la Suite CAIO d'Agentik** est le **Chief AI Officer fractionné end-to-end** qui **transforme l'entreprise en organisation AI-native — de l'audit au système live exploité —**, contrairement **(a) aux cabinets de conseil** qui livrent un PowerPoint puis s'en vont, **(b) aux agences de dev** qui codent un chatbot sans comprendre les workflows métier, et **(c) au « on le fera en interne »** qui n'a ni le temps, ni la littéracie agentique, ni le droit à l'erreur.

**Catégorie qu'on revendique :** *AI transformation operator* — on **diagnostique, architecture, construit, déploie en réel (creds clients), forme, transfère la propriété, et exploite.** On n'est ni un cabinet (qui s'arrête au slide) ni une ESN (qui code sans stratégie). La preuve de cette catégorie, c'est le **système live vérifié au navigateur** (`/omg-acceptance`) — pas un livrable papier.

**Le vrai concurrent, c'est le statu quo** (mm-04 : l'alternative n'est presque jamais un concurrent direct). L'acheteur compare l'accompagnement à « ne rien faire / bricoler trois prompts ChatGPT en interne ». Tout le positionnement consiste à rendre ce statu quo visiblement coûteux.

**Notre asset distinctif :** on est les seuls à **fermer la boucle du diagnostic au run**, avec un OS d'entreprise (`./company-ai-os/`, 10 livrables) et des systèmes **réellement déployés et vérifiés en prod sur les creds du client** — pas un POC sandbox.

---

## 2. L'échelle d'offre — 4 paliers

Quatre paliers, un par grand moment du parcours client. **On ne présente jamais les quatre en bloc** (mm-08 : trop de tiers = paralysie). On qualifie en découverte, puis on présente **le palier d'entrée pertinent + le suivant comme suite logique** (montée naturelle, ascenseur de valeur). L'enchaînement naturel est : **Audit → Transformation → Build → Retainer**.

| Palier | Scope (skills du parcours) | Livrable principal | Durée | Prix indicatif | Profil client idéal |
|---|---|---|---|---|---|
| **1. Audit-Sprint** | `caio-discovery-interview` (échantillon) + `caio-enterprise-workflow-architect` (mode diagnostic) | `./company-ai-os/` allégé : cartographie des workflows, scoring d'opportunités 10 critères, **roadmap 30/60/90**, ROI par workflow, note de gouvernance | **2–3 sem.** | **8 k–20 k €** (forfait) | Dirigeant curieux mais prudent ; veut une preuve chiffrée avant d'engager le capital. **Porte d'entrée bas-risque.** |
| **2. Transformation** | `caio-discovery-interview` (par employé) + `caio-enterprise-workflow-architect` **complet (10 livrables)** | Le **Company AI OS** complet : architecture cible, blueprints agentiques, spec du dashboard, roadmap, ROI par workflow, modèle de gouvernance | **4–8 sem.** | **25 k–60 k €** (forfait) | ETI 50–300 pers., comité de direction aligné. Veut **le plan d'ensemble** avant de coder. |
| **3. Build** | `/caio-implementation-runbook` — réalise la topologie **fédérée** (1 serveur client + 1 micro-SaaS par C-Level + contrat API inter-dashboard + connecteurs Composio + rapports automatisés + monitoring), build **par ship-gate** (de la valeur dès la semaine 1), vérifié **live au navigateur** | Le Company-AI-OS **live, vérifié**, branché sur les vrais outils & données du client | **8–16 sem.** par vague | **60 k–180 k €** par lot de workflows | Boîte qui a validé l'architecture et veut **le système en production**, pas une maquette. |
| **4. Retainer-Managé** | `caio-enablement-and-transfer` (formation + runbooks + transfert) puis `caio-run-and-optimize` (KPIs, observabilité, QBR, amélioration continue) | Système exploité, équipe formée, propriété transférée, **amélioration continue + reporting trimestriel (QBR)** | **Récurrent**, engagement 6–12 mois | **4 k–15 k €/mois** selon périmètre & SLA | Système live ; veut l'exploitation, l'adoption durable et un partenaire qui **fait monter la maturité interne**. |

**Lecture stratégique :**
- **Audit-Sprint = le « foot in the door ».** Petit ticket, risque quasi nul pour l'acheteur, livrable qui **vend le Build de lui-même** (le scoring d'opportunités chiffre ce que le Build rapporte). Crédit du forfait Audit déductible du palier suivant s'il signe sous 30 jours — classique d'ascenseur.
- **Transformation = la pièce maîtresse de conviction.** C'est le `company-ai-os` complet ; c'est lui qui transforme un « peut-être » en feuille de route budgétée.
- **Build = le gros du revenu**, vendu **par lot/vague de workflows** (jamais « tout le SI d'un coup ») — chaque vague est un palier validé avant la suivante (cohérent avec la roadmap 30/60/90).
- **Retainer = la rente et le NRR** (mm-08, NRR > 100 %). C'est là que l'économie de l'agence devient saine : revenu récurrent + expansion (nouveaux workflows, nouveaux services) à coût d'acquisition quasi nul.

**Iron Law 1 (`offer-and-revenue-architect`) — une offre avant un menu.** En face d'un prospect, on **n'ouvre jamais le catalogue** : on diagnostique, on recommande **un** point d'entrée. Les quatre paliers sont notre architecture interne, pas un menu jeté sur la table.

---

## 3. Pricing à la valeur — jamais à l'heure

### Les trois logiques de prix (mm-08) — on n'en utilise qu'une

- **Cost-plus** (mes coûts + marge) → **interdit comme méthode.** Sert uniquement de **plancher** : `prix ≥ 5 × taux horaire effectif × heures estimées` (Iron Law 3). En dessous, on s'achète un job, pas un business.
- **Competitive** (ce que facture un cabinet) → **balise** seulement. Un Big 4 facture une transfo IA 150–500 k € ; ça borne le marché par le haut, ça ne fixe pas notre prix.
- **Value-based** → **la seule méthode.** On part de **la valeur économique que le système crée pour le client** et on en capture **10–30 %** (mm-08), en lui laissant 70–90 % — c'est ce surplus qui fait qu'il signe, reste et recommande.

### Les règles non-négociables

1. **Un prix, jamais une fourchette** (Iron Law 2 / mm-08). « 40 k à 90 k » = deux prix qui se disputent ; l'acheteur ancre sur 40 et négocie à 32. En interne on raisonne en fourchette de cadrage ; **à l'écrit dans la propale, un seul nombre par palier choisi**, défendu.
2. **Le prix EST un signal de positionnement** (mm-08 / Dunford). Tarifer une transfo à 6 k € nous range mentalement parmi les freelances jetables. Le prix premium **aligne** la Suite CAIO avec les solutions sérieuses — *à condition que l'offre suive*. En B2B premium, on utilise des **prix ronds** (45 k, 120 k), pas du charm pricing.
3. **Taux de refus sain : 20–30 %** (mm-08). Si **aucun** prospect qualifié ne trouve jamais ça cher, on est trop bas — les founders se trompent vers le bas dans 95 % des cas. Un « c'est cher » occasionnel est une donnée, pas un échec.
4. **Pas de remise avant 3 ventes au plein tarif** (Iron Law 7). On valide l'offre d'abord ; on ne casse jamais l'ancre de prix par une « remise de lancement » qui devient le vrai prix.
5. **Marge ou revenu vanité** (Iron Law 8). On suit la **marge de contribution par mission**, pas le top line. Sur le Build, surveiller le coût de tokens/infra : **< ~25 % du prix payé** (mm-08, piège des tokens) ; au-delà, on plafonne ou on passe en palier supérieur.

### Pourquoi jamais à l'heure (le piège du technicien)

Facturer à l'heure punit notre efficacité : plus nos agents accélèrent le Build, **moins** on est payé pour la même valeur livrée. À la valeur, c'est l'inverse — la valeur créée pour le client est découplée de notre coût de production (proche de zéro en marginal). **Le forfait au résultat est la seule structure cohérente avec une agence agentique.**

---

## 4. L'équation de valeur, travaillée sur un exemple

**Hormozi (mm-04 / mm-08) :**

> **Valeur = (Résultat rêvé × Probabilité perçue de l'atteindre) ÷ (Délai × Effort & sacrifice)**

On ne baisse **jamais** le prix : on **gonfle le numérateur** et on **écrase le dénominateur**. Voici comment chaque levier est porté par un actif concret de la Suite :

| Levier | Ce qu'on actionne | Skill / livrable Agentik qui le prouve |
|---|---|---|
| **Résultat rêvé ↑** | On vend « une entreprise AI-native qui tourne », pas « des agents ». Le scoring d'opportunités chiffre le résultat en €/an. | `caio-enterprise-workflow-architect` → ROI par workflow |
| **Probabilité perçue ↑** | **Garanties + preuve.** Le diagnostic chiffré avant l'engagement, et surtout la **vérification navigateur live** (`/omg-acceptance`) + `/codeaudit` + `/secaudit` : on prouve que ça **marche en prod**, on ne le promet pas. | Audit-Sprint comme « money-back proof », acceptance gate |
| **Délai ↓** | **Première victoire rapide :** la roadmap 30/60/90 livre un premier workflow live en quelques semaines, pas un big-bang à 18 mois. | Roadmap 30/60/90, build par vagues |
| **Effort & sacrifice ↓** | **Done-for-you de bout en bout :** on provisionne le serveur + les micro-SaaS, on câble les creds réels (Composio), on forme, on transfère. L'équipe interne ne réoutille rien. | `caio-implementation-runbook`, `caio-enablement-and-transfer` |

### Exemple chiffré (gabarit — ETI distributeur, ~150 pers., ~18 M€ CA)

Découverte → on identifie 6 workflows scorés. Valeur annuelle estimée (à recalculer sur leurs vrais chiffres) :

```
Workflow                          Valeur/an estimée
- Devis & chiffrage (1,4 ETP)            ~85 000 €
- SAV / tickets niveau 1 (1,1 ETP)       ~70 000 €
- Saisie & suivi commandes (0,9 ETP)     ~55 000 €
- Reporting/consolidation (0,6 ETP)      ~40 000 €
- Erreurs de saisie évitées              ~60 000 €
- Délai devis→signature -40 %    ~ +120 000 € de CA capté
-------------------------------------------------------
Valeur annuelle totale créée        ~430 000 €/an
```

**Application de l'équation au pricing :**
- Capture value-based 10–30 % de 430 k € → **43 k–129 k €/an** de prix défendable.
- Plancher de coût (Iron Law 3) : Build ≈ 60 h effectives × 5 × ~120 €/h ≈ 36 k € → loin sous le plafond de valeur ⇒ **on price à la valeur, pas au coût.**
- **Prix proposé (un nombre, pas une fourchette) :** Transformation **45 k €** + Build vague 1 (3 workflows prioritaires) **110 k €** = **155 k € en année 1**, puis Retainer **6 k €/mois**.

**Hormozi en action :** le prospect dit « 155 k €, c'est beaucoup ». On ne baisse pas. On **recharge le numérateur** : garantie sur l'Audit, premier workflow live à J+45, on porte tout le build done-for-you. Face à 430 k €/an de valeur, **155 k € one-time devient évident.** Le prix n'est jamais le problème ; c'est l'offre qui est trop faible quand il bloque.

---

## 5. Le ROI pour l'acheteur — le business case « pourquoi financer l'accompagnement »

> **À distinguer absolument** du *ROI par-workflow* que produit `caio-enterprise-workflow-architect` (combien rapporte CHAQUE agent). Ici on répond à la question du **DAF / dirigeant** : *« pourquoi payer Agentik plutôt que faire autrement ? »* C'est le business case de **l'accompagnement lui-même**, pas des workflows.

L'acheteur compare trois trajectoires. Notre job : rendre les deux alternatives visiblement plus chères.

| Trajectoire | Coût réel | Délai au 1ᵉʳ système live | Risque |
|---|---|---|---|
| **Ne rien faire** | Le **coût d'inaction** : 430 k €/an de valeur non capturée + perte de terrain concurrentiel composée | ∞ | Déclassement |
| **Le faire en interne** | 1–2 recrutements senior IA (**150–250 k €/an chargés**) + 9–18 mois de montée en littéracie + **culs-de-sac** (POC qui ne passent jamais en prod) | 12–18 mois | Élevé : pas de droit à l'erreur, pas de méthode |
| **Cabinet de conseil** | 150–500 k € pour **un slide deck**, zéro système live, facturé à l'heure | « Recommandations » à 3 mois, **rien en prod** | Livrable papier, exécution à la charge du client |
| **Suite CAIO Agentik** | 155 k € an 1 + retainer ; **système live et vérifié** | **45 jours** (1ᵉʳ workflow) | Bas : audit chiffré d'abord, acceptance gate, transfert de propriété |

**Les 4 arguments du business case acheteur (à mettre dans la propale, section ROI) :**

1. **Vitesse-au-résultat.** On livre en semaines ce qu'un recrutement interne livrerait en 12–18 mois. **L'accompagnement, c'est l'achat d'~un an d'avance** — sur un marché où l'avance composée.
2. **Évitement des culs-de-sac.** La majorité des POC IA d'entreprise ne passent jamais en prod. Chaque cul-de-sac interne coûte facilement 150–250 k € (temps + recrutement + opportunité). **On en évite plusieurs ; le coût de l'accompagnement est inférieur à un seul échec interne.**
3. **Capital de littéracie agentique transféré** (`caio-enablement-and-transfer`). On ne laisse pas une dépendance : on **transfère la propriété et les runbooks**. L'entreprise sort **capable**, pas captive. C'est un actif au bilan, pas une dépense IT.
4. **Système réel, pas un livrable papier.** Différenciateur central vs cabinet : `/omg-acceptance` + `/secaudit` prouvent que ça tourne **sur leurs creds, en prod**. Le ROI n'est pas projeté sur slide, il est **mesuré** dans le dashboard livré.

**Formule de pitch (à recadrer chaque fois sur leurs chiffres) :** *« Vous investissez 155 k € pour capturer ~430 k €/an de valeur, avec un premier résultat à 45 jours. Payback ~5 mois, puis la valeur tombe chaque année pendant que le système vous appartient. »*

---

## 6. Le processus de vente — découverte → proposition → closing

On suit les **5 étapes de la vente B2B fondateur** (mm-10). Notre ACV (forfaits 25 k–180 k € + retainer) nous met clairement en **sales-led pur** (mm-10 : > ~25 k €/an = conversation humaine obligatoire). Pas de PLG ici.

### Étape 1 — Discovery (diagnostic, on ne pitche RIEN)

C'est l'erreur n°1 du technicien : balancer la démo. **Un médecin qui prescrit avant de diagnostiquer commet une faute.** On parle 30 %, le prospect 70 %. Questions JTBD (Christensen/Moesta) :
- *« Comment vous gérez [le workflow] aujourd'hui ? Ça vous coûte quoi — temps, argent, erreurs, frustration ? »*
- *« Qu'est-ce qui se passe si vous ne réglez rien dans 12 mois ? »*
- **Le compelling event :** *« Pourquoi maintenant ? »* — décide si la vente se fait ou traîne 6 mois.
- *« Qui d'autre décide ? »* (qualifie l'objection autorité **dès le départ**, mm-10).

On note **leurs mots exacts** — c'est la matière première de la propale (mm-04 : verbatims) et du Company AI OS.

> **L'Audit-Sprint EST notre meilleur outil de discovery payant.** Plutôt qu'une découverte gratuite qui traîne, on vend un petit ticket (8–20 k €) qui produit le diagnostic chiffré — et qui vend le Build de lui-même.

### Étape 2 — Demo / livraison du diagnostic

On ne fait **pas** le tour des features. On montre **uniquement** ce que la discovery a révélé : le scoring d'opportunités sur LEURS workflows, le ROI par-workflow chiffré sur LEURS nombres. Une démo = **une histoire qui résout leur problème**, pas une visite guidée de l'outil.

### Étape 3 — Proposition (`/market-proposal` + `/offer-and-revenue-architect`)

Document client-ready généré via `/market-proposal`, structure éprouvée :
1. **Executive summary** qui rejoue leurs mots (preuve qu'on a écouté).
2. **Situation analysis** = le diagnostic Audit-Sprint (data-backed close à 2–3× le taux d'une propale générique).
3. **Stratégie phasée** = roadmap 30/60/90.
4. **Investissement en paliers Good-Better-Best** (mm-08 : ancrage + aversion aux extrêmes). On présente **3 options**, le **tier du milieu est la cible** :

| Composant | **Cadrage** (Good) | **Transformation+Build** (Better — *recommandé*) | **Programme complet** (Best — ancre haute) |
|---|---|---|---|
| Périmètre | Audit + roadmap | Architecture + 3 workflows live | Architecture + 6 workflows + retainer 12 mois |
| Investissement | 15 k € | **155 k €** | 290 k € + 6 k €/mois |

> L'ancre haute (Best) rend le **Better** raisonnable. La majorité signe le tier du milieu — c'est **par design** (mm-08). Le crédit Audit se déduit du Better.

5. **ROI projection** = la section §5 ci-dessus, **mêmes nombres de base** que l'Investissement (cohérence obligatoire, sinon la propale perd la confiance).
6. **Next step daté** + date de validité (30 j).

**Iron Law 6 (`offer-and-revenue-architect`) — cash avant scale :** les 3 premières missions se vendent **à la main, au plein tarif**, avant tout funnel ou automatisation de vente. Trois closes valident l'offre — pas la landing page.

### Étape 4 — Closing

On **demande explicitement** la décision, puis on **se tait** (mm-10). Beaucoup font une superbe démo puis attendent. Assumptive close propre : *« Je bloque le kickoff pour lundi prochain, je vous envoie l'accès — ça vous va ? »* Toute objection se traite (§7), elle n'arrête pas le closing.

### Étape 5 — Onboarding / kickoff

La vente n'est **pas finie à la signature**, elle est finie quand le client **atteint la valeur** (le 1ᵉʳ workflow live). C'est le pont vers le Build et le Retainer — donc vers la LTV. Récap écrit systématique après chaque étape.

### Suivi — la fortune est dans le follow-up (mm-10)

3 à 5 touches espacées, **chacune avec une raison + de la valeur** (un cas, une réponse à une objection, une ressource) — jamais « alors, vous en pensez quoi ? ». La plupart des concurrents lâchent après 1–2 relances ; on persiste poliment au-delà. Cadence de référence : J0 envoi, J2, J5, J7, J14, J21 (breakup propre).

---

## 7. Matrice de traitement des objections

**Principe (mm-10) : une objection n'est pas un rejet, c'est une demande d'information.** On **déballe** avant de répondre, avec **feel-felt-found** (je comprends ce que vous ressentez → d'autres l'ont ressenti → voici ce qu'ils ont découvert), appuyé sur un **vrai** cas (jamais de preuve sociale inventée — la confiance est le seul actif composé).

| Objection | Ce qu'elle cache (à creuser d'abord) | Réponse (feel-felt-found + recadrage) |
|---|---|---|
| **« C'est trop cher »** | Trop cher *par rapport à quoi* ? Budget ? Valeur perçue (alors la discovery a raté) ? Cabinet ? | Recadrer sur le **ROI** (§5) : *« 155 k € face à ~430 k €/an de valeur, payback ~5 mois. »* Proposer le **palier Cadrage** (15 k €) comme entrée bas-risque — pas une remise, une **descente de scope**. |
| **« Ce n'est pas le moment / on verra l'an prochain »** | Pas de compelling event, ou peur déguisée. | *« D'autres dirigeants disaient pareil — puis ont calculé que chaque trimestre d'attente, c'est ~107 k € de valeur non captée + de l'avance perdue. »* Offrir le **Audit-Sprint** maintenant pour figer le plan sans gros engagement. |
| **« On le fera en interne »** | Sous-estime le coût réel et le risque d'exécution. | Coût chargé d'**1–2 seniors IA = 150–250 k €/an + 12–18 mois**, et la majorité des POC ne passent jamais en prod. *« On vous fait gagner l'année d'avance ET on transfère la propriété — vous sortez capables, pas dépendants »* (`caio-enablement-and-transfer`). |
| **« Et si l'IA hallucine / se trompe ? » (risque IA)** | Peur de la fiabilité et de la responsabilité. | *« C'est exactement pourquoi rien ne passe en prod sans `/omg-acceptance`, `/codeaudit`, `/secaudit` — vérification live, observabilité, human-in-the-loop sur les actions sensibles, garde-fous documentés dans les runbooks. »* La fiabilité est **prouvée au runtime**, pas promise. |
| **« Et le RGPD / nos données ? »** | Conformité + souveraineté. | *« La gouvernance est un livrable du Company AI OS, pas un après-coup : DPA, minimisation, hébergement EU, registre de traitement, données qui restent sur vos infra et vos creds. »* Le diagnostic adresse la conformité **avant** tout code. |
| **« On a déjà essayé l'IA, ça n'a rien donné »** | Mauvaise exécution passée (un POC sandbox sans workflow réel). | *« Qu'est-ce qui a bloqué précisément ? »* Différencier : on part des **workflows métier scorés** et on **déploie en prod sur vos vrais outils**, pas un chatbot démo. Proposer l'Audit comme pilote à succès mesurable. |
| **« On parle à d'autres prestataires »** | Comparaison, recherche de réassurance. | Bienvenue. Différencier sur la **méthode** (parcours end-to-end audit→run, système live vérifié), **pas sur le prix**. *« Demandez-leur s'ils livrent un système vérifié en prod sur vos creds, ou des slides. »* |

---

## 8. Indicateurs commerciaux

On pilote la machine de vente avec des chiffres, pas des impressions. Cibles de cadrage (à étalonner sur le réel après les 10 premières missions) :

| Indicateur | Définition | Cible de cadrage |
|---|---|---|
| **Cycle de vente** | Premier contact → signature | Audit-Sprint : **2–4 sem.** · Transformation/Build : **6–12 sem.** (sales-led, plusieurs décideurs) |
| **Taux de closing** | Propales envoyées → signées | **25–40 %** sur prospects qualifiés en discovery (data-backed grâce à l'Audit). En dessous de 20 % → discovery ou qualification ratée, pas le prix. |
| **Taux de refus prix** | Prospects qualifiés trouvant « trop cher » | **20–30 %** = on est bien placé (mm-08). **0 %** → on est trop bas, on monte. |
| **Panier moyen (ACV)** | Valeur contractuelle moyenne an 1 | **Transformation+Build : ~120–170 k €** · plus **retainer ~6–10 k €/mois** |
| **Conversion ascenseur** | Audit-Sprint → Transformation/Build | **≥ 50 %** (l'Audit est conçu pour vendre le Build) |
| **Taux d'attache retainer** | Builds livrés → Retainer signé | **≥ 60 %** — c'est la rente et la LTV |
| **NRR** (mm-08) | Expansion − churn sur les comptes retainer | **> 100 %** (cible 110–130 %) : nouveaux workflows, nouveaux services sur la base installée, acquisition quasi nulle |
| **Marge de contribution** | Par mission (Iron Law 8) | **≥ 70 %** ; coût tokens/infra **< 25 %** du prix payé |

**Falsification de l'offre (Iron Test, `offer-and-revenue-architect`) :** au bout des 3 premières ventes — ont-elles closé **au plein tarif, sans remise**, à **≥ 70 % de marge**, dans la **capacité de livraison** modélisée ? Si oui → l'offre est un business, on **monte les prix de 20–40 %**. Si 0/3 après 8 conversations qualifiées → l'**outcome ou le prix** est faux, on ré-architecture la découverte et le positionnement — **on n'auto-discounte pas**.

---

## 9. Comment exécuter (orchestration)

- **`/offer-and-revenue-architect`** → écrit `./business-os/` : verrouille **un** prix par palier (jamais de fourchette), modèle les **unit economics** (CAC quasi nul en sales-led founder-network, LTV = Build + retainer, marge, payback), produit le **sell-sheet** et la **discipline check** avant tout envoi.
- **`/market-proposal`** → génère `CLIENT-PROPOSAL.md` : la propale persuasive, **paliers Good-Better-Best**, **ROI projection** alignée sur les mêmes nombres, objection-handling, next step daté. Réutilise les données de l'Audit-Sprint (data-backed = ×2–3 sur le close).
- **Doctrine** : `/mm-04` (offre + value equation), `/mm-08` (prix à la valeur, tiering, NRR), `/mm-10` (vente 5 étapes, objections, follow-up). On **invoque les vrais skills**, on ne paraphrase pas (R-MARKETING / R-AUDIT).
- **Tout le parcours** est orchestré en une passe gap-checkée par **`/caio-master`** (pattern `marketing-master`).

---

**--- Resume :** Le Manuel de l'offre packe la Suite CAIO en 4 paliers (Audit-Sprint 8–20 k → Transformation 25–60 k → Build 60–180 k → Retainer 4–15 k/mois), price **à la valeur** (capture 10–30 %, un seul prix, jamais à l'heure), arme le **business case acheteur** (vitesse, évitement des culs-de-sac, littéracie transférée, système live vs slide), et déroule **découverte→proposition Good-Better-Best→closing** avec une matrice d'objections feel-felt-found et des KPIs commerciaux — le tout exécuté via `offer-and-revenue-architect` + `market-proposal` et la doctrine mm-04/08/10.
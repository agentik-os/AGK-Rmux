# Onboarding client CAIO — Guide interne (équipe Agentik)

> **À qui ça sert.** Ce document est le playbook INTERNE de l'équipe qui délivre l'accompagnement CAIO. Il couvre la **phase 0→1 du parcours** : du premier contact jusqu'à J+1 (lendemain du kickoff). Objectif : qu'un nouveau lead CAIO puisse embarquer un client sans rien oublier, dans l'ordre, avec les bons garde-fous.
>
> **Où on est dans le parcours.** Onboarding = la couture entre la **phase 0 (offre/vente)** et la **phase 1 (découverte)**. Le contrat est signé ou en train de l'être ; il faut transformer une promesse commerciale en machine d'exécution. Tout est orchestré in fine par `/caio-master`, mais l'onboarding est le moment où **un humain (le lead CAIO) pose les fondations** que la machine exploitera.
>
> **Règle d'or de l'onboarding.** *Un client mal embarqué ne se rattrape jamais en phase build.* 80 % des engagements qui dérapent ont raté l'une de ces 3 choses : le sponsor exécutif n'existait pas vraiment, le périmètre n'était pas écrit, ou la donnée/les accès n'ont jamais été livrés. Tout ce guide existe pour fermer ces 3 trous **avant** d'écrire la moindre ligne.

---

## 0. Vue d'ensemble — les 6 jalons de l'onboarding

| # | Jalon | Output concret | Qui pilote | Gate de sortie |
|---|-------|----------------|-----------|----------------|
| 1 | Premier contact & qualification | Fiche lead qualifiée (GO/NO-GO) | Lead CAIO | Score qualif ≥ seuil + sponsor identifié |
| 2 | Appel de découverte commercial | Compte-rendu + douleurs chiffrées | Lead CAIO | Au moins 1 workflow douloureux quantifié |
| 3 | Cadrage de l'engagement | Proposition (`market-proposal`) + palier choisi + SOW signé | Lead CAIO + sponsor | Signature + bon de commande |
| 4 | Pré-kickoff (logistique & accès) | Workspace `./company-ai-os/` initialisé, accès demandés | Lead CAIO | Repo + accès en cours, champions nommés |
| 5 | Kickoff | Réunion de lancement + planning des entretiens découverte | Lead CAIO + sponsor + champions | Calendrier des entretiens posé |
| 6 | J+1 — amorçage découverte | Premier `caio-discovery-interview` planifié/lancé | Lead CAIO | Phase 1 démarrée pour de vrai |

> ⚠️ **Ne jamais sauter un jalon.** Chaque gate empêche un échec en aval. Un client qui veut « accélérer et zapper la découverte » est un drapeau rouge (voir §10).

---

## 1. Premier contact & qualification

### 1.1 D'où vient le lead
- **Inbound** (sell-sheet, contenu marketing-master, démo) → déjà tiède, qualifier la maturité réelle.
- **Outbound** (cold-email, intro réseau) → qualifier d'abord l'**intention**, pas juste la curiosité.
- **Référence client** → fast-track, mais re-qualifier quand même le périmètre.

### 1.2 La grille de qualification (à remplir AVANT de donner un créneau de découverte)
On ne passe pas un lead en découverte tant que ces 6 cases ne sont pas instruites. C'est un filtre, pas un interrogatoire : ça se fait en 15-20 min de pré-call ou par mail structuré.

1. **Douleur réelle & coûteuse** — quel processus leur fait mal *aujourd'hui*, et combien ça coûte (heures/personne, erreurs, délai, CA perdu) ? Pas de douleur chiffrable → pas de ROI à vendre.
2. **Sponsor exécutif** — y a-t-il **un dirigeant** (C-level / DG / fondateur) qui *porte* le projet et signe le budget ? Si le contact est un manager isolé « qui veut explorer l'IA », c'est NO-GO ou à requalifier.
3. **Budget & horizon** — ordre de grandeur connu et réaliste vs nos paliers (Good-Better-Best). On ne découvre pas un client qui imagine un Company AI OS à 3 000 €.
4. **Accès aux données & systèmes** — sont-ils prêts à donner accès à leurs outils (CRM, ERP, drive, mails) et à leurs gens ? Un client qui verrouille tout ne peut pas être servi.
5. **Capacité interne** — ont-ils des humains à libérer (champions) pour les entretiens et l'adoption ? L'accompagnement n'est pas « faites tout à notre place sans nous » — voir §6.
6. **Timing & déclencheur** — pourquoi maintenant ? (nouveau dirigeant, pression concurrentielle, levée, douleur aiguë). Sans déclencheur, l'engagement traîne et meurt.

### 1.3 Décision
- **GO** : ≥ 5/6 instruits favorablement **ET** sponsor exécutif identifié (case 2 non négociable). → on planifie la découverte.
- **NURTURE** : douleur réelle mais timing/budget pas mûrs → on renvoie vers la doctrine `marketing-master` (contenu, séquence email) et on recontacte.
- **NO-GO** : pas de sponsor, ou pas de douleur chiffrable, ou refus d'accès. → on décline proprement. *Décliner un mauvais client est une décision rentable.*

> **Livrable interne du jalon 1 :** une fiche lead (1 page) dans `./company-ai-os/00-onboarding/qualification.md` une fois le repo créé — ou en scratch avant. Verdict GO/NURTURE/NO-GO explicite et daté.
>
> **Version formelle scorée (phase 0 du parcours) :** dès qu'un GO commercial se dessine, on lance `/caio-ai-readiness-assessment` — la passe go/no-go pré-signature : 9 dimensions de maturité IA notées avec preuves → **Readiness Index + verdict GO / NOT-YET / REDIRECT** + investissement indicatif, écrit dans `./caio-readiness/`. C'est le filtre honnête de l'offre (« si votre cas ne colle pas, je vous redirige »). Seul un **GO** ouvre la phase Offre.

---

## 2. L'appel de découverte (commercial)

> ⚠️ **Ne pas confondre** cet appel avec la **phase 1 `caio-discovery-interview`** (entretiens par employé, post-signature). Ici c'est l'appel **commercial** de 45-60 min avec le sponsor + 1-2 décideurs, pour cadrer la vente. La découverte profonde par métier vient APRÈS signature.

### 2.1 Objectif de l'appel
Sortir avec **assez de matière pour écrire une proposition chiffrée** : douleurs hiérarchisées, au moins un workflow dont on peut estimer le coût actuel et le gain potentiel, et la confirmation du sponsor + de l'autorité budgétaire.

### 2.2 Déroulé (structure type, 50 min)
1. **Cadre (5 min)** — qui est dans la pièce, ce qu'on va faire, le fait qu'on cherche la vérité pas à vendre du rêve (L2 : researcher, not sycophant — on challenge un brief flou).
2. **Cartographie rapide (15 min)** — comment l'entreprise gagne de l'argent, les 3-5 processus critiques, où ça frotte. On écoute le langage métier, on ne projette pas nos solutions.
3. **Douleur chiffrée (15 min)** — on creuse 1-2 douleurs : volume, temps, erreurs, coût. On transforme « on perd du temps » en « 4 personnes × 2 h/jour sur de la ressaisie ».
4. **Vision & ambition (8 min)** — où ils veulent être dans 6-12 mois ; ça calibre Good vs Better vs Best.
5. **Logistique & suite (7 min)** — qui décide, sous quel délai, qui seraient les champions, quels systèmes/données. On annonce qu'on revient avec une proposition à paliers.

### 2.3 Ce qu'on capte absolument
- 1 workflow « héros » qu'on pourra mettre en avant dans la projection ROI.
- Le **coût actuel** estimé d'au moins ce workflow (matière brute pour `offer-and-revenue-architect` / unit-economics).
- La liste des **systèmes** (CRM, ERP, drive, compta, support…) et le niveau d'accès envisageable.
- Les **noms** des futurs champions et du sponsor confirmé.

> **Livrable interne du jalon 2 :** compte-rendu de découverte (`./company-ai-os/00-onboarding/discovery-call.md`) — douleurs hiérarchisées + workflow héros chiffré. C'est l'intrant de la proposition.

---

## 3. Le cadrage de l'engagement (choisir le palier d'offre)

### 3.1 La proposition
On génère la proposition avec **`market-proposal`** (paliers Good-Better-Best, projection ROI) en s'appuyant sur **`offer-and-revenue-architect`** (offre, prix, unit-economics) et la doctrine **`marketing-master`** (mm-04 offre, mm-08 prix, mm-10 vente). On ne réinvente pas une proposition à la main : on invoque les vraies skills (R-MARKETING, ne jamais paraphraser).

### 3.2 Les trois paliers — comment cadrer le bon
> Les noms et montants exacts sont définis par `offer-and-revenue-architect` ; voici la **logique de cadrage** pour choisir avec le client.

| Palier | Pour qui | Périmètre type | Profondeur découverte | Build |
|--------|----------|----------------|----------------------|-------|
| **Good (Diagnostic)** | Boîte prudente, veut une preuve avant d'investir | Phases 1-2 : découverte + diagnostic/architecture/roadmap (les 10 livrables `caio-enterprise-workflow-architect`), **pas de build** | Quelques métiers clés | — |
| **Better (Pilote)** | Boîte décidée, veut un résultat tangible vite | Good + **build d'1-2 workflows** héros déployés en prod & vérifiés (planner + new-project + acceptance + audits) | Élargie | 1-2 systèmes |
| **Best (Company AI OS)** | Boîte qui veut devenir AI-native end-to-end | Better + build étendu + **formation/handoff** + **run & retainer** (phases 4-5) | Toute l'entreprise | Plateforme complète |

**Comment on aide le client à choisir :**
- S'il doute → **Good**. Le diagnostic seul crée déjà une valeur vendable et désamorce le risque ; il upgrade naturellement vers Better une fois le ROID démontré.
- S'il a une douleur aiguë et un sponsor fort → **Better**. Un pilote en prod tue le scepticisme interne mieux que n'importe quel slide.
- S'il a la maturité, le budget et la volonté de transformer l'org → **Best** avec retainer. C'est là qu'est la vraie valeur récurrente.

> 🎯 **Anti-piège tarifaire :** on ancre la valeur sur le **ROI du workflow héros**, pas sur le nombre de jours-homme. La projection ROI de `market-proposal` est l'argument central (mm-08 pricing).

### 3.3 Le SOW (Statement of Work) — ce qui doit être ÉCRIT avant kickoff
Le périmètre non écrit est la première cause de litige. Le SOW (ou bon de commande + annexe) doit fixer :
- **Périmètre** : quels workflows / métiers sont dans le scope, lesquels sont explicitement HORS scope.
- **Livrables** : la liste exacte (ex. les 10 livrables `./company-ai-os/` pour la phase 2).
- **Jalons & planning** : découverte → diagnostic → (build) → handoff, avec dates cibles.
- **Responsabilités client** : accès données/systèmes, disponibilité des champions, validation des jalons (RACI léger).
- **Conditions de done** : on s'aligne sur L4 (done = 100 % vérifié) et L0/L1 (prod vérifiée en live pour tout build). Un build n'est « livré » que vérifié navigateur via `acceptance`.
- **Prix, échéancier de paiement, durée du retainer** le cas échéant.

> **Gate de sortie jalon 3 :** SOW signé + bon de commande. **Pas de signature = pas de kickoff.** On peut initialiser le workspace en parallèle, mais on ne lance pas la découverte profonde sans engagement ferme.

---

## 4. Le kickoff

### 4.1 Pré-kickoff (logistique — à faire AVANT la réunion)
- Créer le workspace `./company-ai-os/` (voir §5).
- Envoyer un **mail de bienvenue** au sponsor : agenda du kickoff, ce qu'on attend d'eux, demande des accès, demande de nommer les champions.
- Préparer le **plan d'entretiens découverte** : quels employés, quels métiers, combien d'entretiens, sur quelle fenêtre de temps.
- Lancer la **demande d'accès** (systèmes, comptes, drive) — c'est le chemin critique le plus lent, on le déclenche tôt (voir piège §10.4).

### 4.2 La réunion de kickoff (60-90 min)
**Présents :** lead CAIO (nous), sponsor exécutif, futurs champions, et idéalement un représentant IT/sécurité pour les accès.

**Agenda type :**
1. **Pourquoi on est là (10 min)** — sponsor rappelle la vision et le mandat ; *c'est lui qui parle*, pas nous. Ça signale à l'org que le projet est porté d'en haut.
2. **Le parcours & ce qui va se passer (15 min)** — on déroule les phases, les livrables, le rôle de chacun, le calendrier.
3. **Les rôles & règles du jeu (15 min)** — voir §6 et §7. On pose explicitement les attentes côté client.
4. **La découverte (15 min)** — on explique `caio-discovery-interview` : chaque employé sera interviewé *dans son langage métier*, ça produit un ZIP standardisé par personne, c'est confidentiel et non-évaluatif (rassurer : ce n'est PAS un audit RH). On cale le calendrier.
5. **Accès & données (15 min)** — on liste précisément ce dont on a besoin, qui le fournit, sous quel délai. On nomme un **point de contact accès**.
6. **Prochaines étapes & engagements (10 min)** — qui fait quoi d'ici J+7, dates des premiers entretiens.

### 4.3 Posture en kickoff
- **Sponsor = porte-voix.** Plus le dirigeant parle, plus l'adoption sera facile. Si le sponsor délègue le kickoff à un manager, c'est un signal faible — on le note et on re-sollicite le sponsor.
- **On rassure sur l'emploi.** La hantise n°1 des employés interviewés : « l'IA va me remplacer ». Message cadré : *on augmente les gens, on supprime la corvée, vous gardez le jugement*. Sans ça, les entretiens découverte sont pollués par la peur.

> **Gate de sortie jalon 5 :** calendrier des entretiens découverte posé + point de contact accès nommé + champions confirmés.

---

## 5. Mise en place de l'espace de travail `./company-ai-os/`

C'est le **dépôt unique de vérité** de l'engagement client. Tout vit ici ; rien d'important ne traîne dans des mails ou des docs épars.

### 5.1 Principe
- Un **repo/dossier par client**, racine `./company-ai-os/`.
- C'est l'output canonique de `caio-enterprise-workflow-architect` (10 livrables en phase 2), donc on prépare la structure dès l'onboarding pour que la phase 2 s'y déverse proprement.
- **Secrets/creds client = JAMAIS dans le repo.** Les accès réels (clés API, tokens CRM/ERP) vivent hors-repo, dans un coffre dédié (cf. R-ENV / L0 : secrets hors du dépôt). Le repo ne contient que des *références* aux secrets, pas les secrets.

### 5.2 Arborescence de départ (à créer au pré-kickoff)
```
./company-ai-os/
  00-onboarding/            # cette phase
    qualification.md         # fiche lead (jalon 1)
    discovery-call.md        # CR appel commercial (jalon 2)
    proposal/                # sortie market-proposal
    sow.md                   # périmètre signé
    kickoff.md               # CR kickoff + plan d'entretiens
    access-register.md       # registre des accès demandés/reçus (statut)
    stakeholders.md          # sponsor, champions, contacts (RACI)
  01-discovery/             # phase 1 — ZIP par employé (caio-discovery-interview)
  02-architecture/          # phase 2 — les 10 livrables (audit, cartographie,
                            #   scoring 10 critères, blueprints, spec dashboard,
                            #   roadmap 30/60/90, ROI/workflow, gouvernance)
  03-build/                 # phase 3 — briefs + liens vers les repos de prod
  04-enablement/            # phase 4 — curricula, runbooks, handoff
  05-run/                   # phase 5 — KPIs, observabilité, QBR
  README.md                 # carte de l'engagement, statut, prochaine action
```

### 5.3 Hygiène
- **Le `README.md` de la racine = le tableau de bord** : où on en est, le prochain jalon, le prochain owner. On le tient à jour à chaque jalon.
- **`access-register.md` est sacré** : chaque système, qui le fournit, statut (demandé/reçu/testé), date. C'est le chemin critique n°1 — voir §10.4.
- On versionne tout (git) ; on ne stocke pas de secret en clair ; on log les décisions importantes.

---

## 6. Les rôles

| Rôle | Côté | Responsabilité | Sans lui… |
|------|------|----------------|-----------|
| **Lead CAIO** | Agentik (nous) | Pilote l'engagement de bout en bout, orchestre `/caio-master`, garant des gates qualité (L0-L5), interface unique du client | personne ne tient le fil |
| **Sponsor exécutif** | Client | Dirigeant qui porte le projet, débloque budget & accès, arbitre les priorités, légitime le projet en interne | l'org ne suit pas, le projet meurt |
| **Champions** | Client | 1 par métier clé : relais terrain, font les entretiens découverte, testent les pilotes, portent l'adoption | la connaissance métier n'entre jamais dans le système |
| **Point de contact accès** | Client (souvent IT) | Fournit et débloque les accès systèmes/données | le build est gelé faute d'accès |
| **Workers/oracle OmegaOS** | Agentik (machine) | Exécutent build/audit/acceptance sous orchestration ; le lead CAIO ne code pas à la main (R-MASTER/R-ORCH) | — |

> **Le rôle qui fait ou défait l'engagement : le sponsor exécutif.** S'il est absent, fantôme ou junior, *tout le reste s'effondre*. C'est pour ça qu'on le verrouille dès la qualification (case 2). Un champion ne remplace jamais un sponsor.

---

## 7. Attentes & règles du jeu (à poser explicitement au kickoff)

On les énonce à voix haute et on les écrit dans le `kickoff.md`. Mieux vaut un client qui dit « non » maintenant qu'un malentendu en phase build.

**Ce qu'on attend du client :**
1. **Disponibilité des champions** — créneaux réservés pour les entretiens et les tests (sinon la découverte traîne sur des semaines).
2. **Accès en temps voulu** — les systèmes/données livrés sous le délai convenu. Tout retard d'accès décale le build d'autant.
3. **Validation des jalons** — le sponsor valide chaque livrable dans un délai borné (ex. 3 jours ouvrés). Le silence n'est pas une validation.
4. **Vérité, pas politesse** — en découverte on veut le réel, pas la version flatteuse. On augmente l'org, on ne la juge pas.
5. **Un seul canal de décision** — le sponsor (ou son délégué nommé) tranche. On évite le comité à 8 têtes qui ne décide jamais.

**Ce qu'on s'engage à fournir (nos règles à nous) :**
1. **Done = vérifié en prod, pas « ça build »** (L0/L1/L4). Tout build passe le gate `acceptance` (sweep navigateur, golden path réel) + `codeaudit`/`secaudit`.
2. **Reproductible & livré** — pas de bidouille locale ; ce qu'on livre, le client le possède et peut le rejouer (handoff phase 4).
3. **Transparence** — pas de fausse confiance, pas de « ça devrait marcher » sans preuve (L2). On montre les preuves (captures, logs, démos).
4. **Sécurité & confidentialité** — secrets hors repo, accès au moindre privilège, données client jamais exfiltrées.
5. **Périmètre tenu** — ce qui est hors SOW est hors scope ; toute extension passe par un avenant, pas par du scope creep silencieux.

---

## 8. Checklist d'onboarding (J-7 → J+1)

> Fenêtre indicative autour du **kickoff = J0**. Adapter, mais ne rien retirer.

### J-7 — Cadrage signé
- [ ] SOW / bon de commande **signé** et classé dans `00-onboarding/sow.md`
- [ ] Palier d'offre confirmé (Good / Better / Best) et périmètre écrit
- [ ] Sponsor exécutif **nommément** confirmé
- [ ] Workspace `./company-ai-os/` créé avec l'arborescence de départ
- [ ] `stakeholders.md` initialisé (sponsor + contacts pressentis)

### J-5 — Logistique & accès lancés
- [ ] Mail de bienvenue envoyé au sponsor (agenda kickoff + attentes + demandes)
- [ ] **Demande d'accès** envoyée (chemin critique) → `access-register.md` ouvert
- [ ] Champions pressentis identifiés, demande de nomination formelle au sponsor
- [ ] Plan d'entretiens découverte ébauché (qui / quels métiers / combien)
- [ ] Date du kickoff calée avec sponsor + champions

### J-2 — Préparation kickoff
- [ ] Agenda kickoff finalisé et envoyé
- [ ] CR appel de découverte (`discovery-call.md`) relu — workflow héros + douleurs prêts à reprojeter
- [ ] Coffre à secrets prêt (hors repo) pour recevoir les futurs creds
- [ ] Champions confirmés par le sponsor ; créneaux entretiens pré-réservés

### J0 — Kickoff
- [ ] Réunion de kickoff tenue, sponsor a porté la vision (lui, pas nous)
- [ ] Rôles + règles du jeu posés et **écrits** dans `kickoff.md`
- [ ] Calendrier des entretiens découverte **fixé** (dates réelles, pas « on verra »)
- [ ] Point de contact accès nommé + délai d'accès convenu
- [ ] Message anti-peur passé aux champions (augmenter ≠ remplacer)

### J+1 — Amorçage phase 1
- [ ] CR de kickoff diffusé + actions assignées avec dates
- [ ] `README.md` racine mis à jour : statut = « découverte en cours », prochaine action claire
- [ ] **Premier `caio-discovery-interview` planifié ou lancé**
- [ ] Statut des accès suivi dans `access-register.md` (relance si rien reçu)
- [ ] GO formel : la phase 1 est démarrée → on passe le relais à `/caio-master`

---

## 9. Quand passer le relais à `/caio-master`

L'onboarding est le moment **humain** (le lead CAIO pose les fondations relationnelles et contractuelles). Une fois la phase 1 amorcée (J+1, checklist verte), c'est `/caio-master` qui orchestre le parcours en une passe gap-checkée et auto-corrective (pattern `marketing-master`, un Workflow au sommet). Le lead CAIO reste l'**interface client et le garant des gates** ; la machine exécute les phases. On ne lance pas `/caio-master` sur un onboarding bâclé — il amplifierait les trous, pas les comblerait.

---

## 10. Pièges classiques (et comment les désamorcer)

**10.1 — Le sponsor fantôme.** Le contact est enthousiaste mais n'a ni budget ni autorité. *Symptôme :* il « doit en parler à la direction » à chaque étape. *Parade :* exiger le sponsor exécutif dès la qualification (case 2, non négociable) ; s'il ne vient pas au kickoff, escalader avant de continuer.

**10.2 — Vouloir zapper la découverte.** « On connaît déjà nos process, construisez直接. » *Risque :* on automatise une version fantasmée du process, pas le réel → le pilote ne sert personne. *Parade :* la découverte est dans le SOW comme prérequis ; on explique que `caio-discovery-interview` capte le langage métier réel que personne n'a jamais écrit.

**10.3 — Le scope creep silencieux.** Chaque champion ajoute « tant qu'on y est, on pourrait aussi… ». *Parade :* périmètre écrit dans le SOW + liste explicite du HORS scope ; toute extension = avenant. On dit oui à l'idée, non au scope gratuit.

**10.4 — Les accès qui n'arrivent jamais.** Le chemin critique n°1. Le build est prêt, mais on attend 3 semaines un accès CRM. *Parade :* demande d'accès dès J-5, `access-register.md` suivi quotidiennement, point de contact accès nommé au kickoff, relance proactive. **Un accès non démarré à J-5 est un risque rouge.**

**10.5 — La peur de l'employé.** Les interviewés croient qu'on prépare des licenciements → ils minimisent ou enjolivent. *Parade :* message anti-peur porté par le sponsor au kickoff ; rappeler que l'entretien est non-évaluatif et confidentiel.

**10.6 — Le comité qui ne décide pas.** 8 personnes valident, donc personne ne valide. *Parade :* un seul canal de décision (le sponsor ou un délégué nommé) écrit dans les règles du jeu (§7).

**10.7 — Confondre « ça build » et « ça marche ».** Tentation de livrer un pilote sur la foi d'un build vert. *Parade :* L1/L0/L4 — tout build passe `acceptance` (sweep navigateur + golden path réel avec écriture persistée) avant d'être déclaré livré. On montre la preuve, pas le slide.

**10.8 — Sur-promettre en vente, sous-livrer en build.** L'offre a vendu un ROI mirobolant non étayé. *Parade :* la projection ROI vient de `market-proposal`/unit-economics (chiffrée, traçable), pas d'un enthousiasme commercial. L2 : researcher, not sycophant — on chiffre, on ne flatte pas.

---

## 11. Récap des artefacts produits pendant l'onboarding

| Artefact | Emplacement | Produit par |
|----------|-------------|-------------|
| Fiche qualification (GO/NO-GO) | `00-onboarding/qualification.md` | Lead CAIO |
| CR appel de découverte | `00-onboarding/discovery-call.md` | Lead CAIO |
| Proposition à paliers + ROI | `00-onboarding/proposal/` | `market-proposal` + `offer-and-revenue-architect` |
| SOW signé | `00-onboarding/sow.md` | Lead CAIO + sponsor |
| Registre des accès | `00-onboarding/access-register.md` | Lead CAIO + contact accès |
| Parties prenantes (RACI) | `00-onboarding/stakeholders.md` | Lead CAIO |
| CR kickoff + règles du jeu | `00-onboarding/kickoff.md` | Lead CAIO |
| Plan d'entretiens découverte | `00-onboarding/kickoff.md` | Lead CAIO + champions |
| Workspace initialisé | `./company-ai-os/` | Lead CAIO |

---

**--- Resume :** Guide interne d'onboarding CAIO en 6 jalons (premier contact → J+1) : qualifier (sponsor exécutif = non négociable), faire l'appel de découverte commercial avec un workflow héros chiffré, cadrer le palier Good/Better/Best via `market-proposal` + SOW signé, monter le workspace `./company-ai-os/`, tenir un kickoff porté par le sponsor, poser rôles et règles du jeu, dérouler la checklist J-7→J+1, puis passer le relais à `/caio-master` — en désamorçant les 8 pièges classiques (sponsor fantôme, découverte zappée, accès en retard, scope creep, « ça build » ≠ « ça marche »).
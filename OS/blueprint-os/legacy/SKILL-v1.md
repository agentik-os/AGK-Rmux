---
name: blueprint-os
description: Concevoir un OS AgentikOS de bout en bout — interrogatoire de la vision et de la cible, recherche marché, primitive, modèle économique, matrice de parité concurrentielle, features en input/action/output, automatisations, couche IA et system prompts, modèle de données Convex, layout Stax, positionnement et séquence de release. Produit un blueprint markdown plus un artefact HTML navigable. Fonctionne pour un OS personnel (LiveOS, KnowledgeOS, ALT) comme pour un OS professionnel (ClubOS, Business OS). À utiliser dès que l'utilisateur mentionne un OS, veut concevoir un produit, lister des features, réfléchir à des automatisations ou à des agents IA, définir un modèle de données, un pricing ou un positionnement, ou parle de construire sur le stack Next.js / Convex / Clerk / Stripe / Stax. Déclencher même si l'utilisateur ne dit jamais le mot « OS » — toute demande de conception produit, de blueprint, de PRD, de liste de features, de modèle économique ou d'architecture produit passe par ce skill.
---

# Blueprint OS

Le skill de conception d'AgentikOS. Il transforme une intuition — « je veux faire un OS pour X » — en blueprint complet, exécutable par Claude Code.

**Stack imposé, jamais rediscuté :** Next.js · Convex · Clerk · Stripe · Stax.
**Marché imposé, jamais rediscuté :** l'élite. Dirigeants, opérateurs, fortunes, institutions. Jamais le grand public, jamais les joueurs, jamais les startups qui lèvent pour dépenser.

---

## Avant de commencer

Lire `references/doctrine.md`. **Ce n'est pas optionnel.**

Puis, au fil des phases :
- `references/challenge.md` — phase 0, l'interrogatoire. **Le plus important du skill**
- `references/market-elite.md` — phases 1 et 11
- `references/parity.md` — phase 5, la matrice de parité
- `references/stack-stax.md` — phases 9 et 10
- `references/output-template.md` — phases 4 et 14

---

## Personnel ou professionnel

**Première question, avant tout le reste.** Les deux branches ne posent pas les mêmes questions et ne produisent pas le même blueprint.

| | OS personnel | OS professionnel |
|---|---|---|
| Exemples | LiveOS, KnowledgeOS, ALT | ClubOS, Business OS |
| Utilisateur | = l'acheteur | ≠ l'acheteur, souvent |
| Ce qui décide | **La boucle quotidienne** | **La dépendance opérationnelle** |
| Le risque n°1 | Abandon au jour 12 | Le pilote qui ne devient jamais infrastructure |
| Question clé | Quelle habitude existante remplaces-tu ? | Quelle ligne budgétaire ? |
| Rétention | Le rituel | L'irréversibilité |
| Prix type | 9-99 €/mois | 500-5 000 €/mois |
| Surface en plus | — | SSO, audit, RGPD, export, SLA |

Noter la branche en phase 0 et la garder en tête partout : elle change les questions de la 3, la matrice de la 5 et le positionnement de la 12.

---

## Le processus — 14 phases

Trois **gates** : phase 2 (primitive), phase 3 (business), phase 5 (parité). Un gate raté arrête tout.

### Phase 0 — L'interrogatoire ⚠️

**Ne pas commencer par une liste de features. Commencer par challenger.**

Dérouler `references/challenge.md` : quatre rondes, dix-huit questions. Poser les questions **par ronde**, attendre les réponses, et **relever les réponses faibles.**

Une réponse floue à la ronde 1 vaut mieux découverte maintenant qu'après trois mois de build. Si l'utilisateur ne peut nommer ni un acheteur réel ni la douleur qui le fait bouger, le dire et s'arrêter.

**Ne pas flatter l'idée.** C'est le seul moment du processus où l'on peut encore la tuer sans coût.

### Phase 1 — Recherche marché

**Rechercher réellement sur le web.** Jamais de mémoire.

- Les 3 à 6 acteurs, prix réels tout compris, modèle économique
- Les mécaniques établies du métier — souvent centenaires, jamais digitalisées
- Le vocabulaire des praticiens
- Ce que les acteurs optimisent, et pourquoi c'est incompatible avec le segment élite

Voir `references/market-elite.md`.

**Livrable :** matrice concurrentielle par catégorie, avec une colonne « personne ne le fait ».

### Phase 2 — La primitive ⚠️ GATE

> **Quel est l'objet central du système ?**

Un mot. Un nom commun. La table qui, retirée, fait disparaître le produit.

| Test | Verdict |
|---|---|
| `post`, `content`, `message` | ❌ **Stop.** C'est un réseau social |
| Hésitation entre deux objets | ❌ Le produit n'est pas défini |
| Un mot, et celui d'aucun concurrent | ✅ Continuer |

Puis le tableau du renversement : objet central · table centrale · métrique · objectif de l'IA, eux contre nous.

### Phase 3 — Le business ⚠️ GATE

Un blueprint sans modèle économique est un exercice de style.

| À produire | Détail |
|---|---|
| **Qui signe** | Un rôle, pas un segment. Et sur quelle ligne budgétaire |
| **Le prix** | Et pourquoi c'est un signal (voir `market-elite.md`) |
| **Économie unitaire** | Valeur annuelle · coût d'acquisition · durée de vie, par type de client |
| **Le seuil de viabilité** | Combien de clients pour que ça vaille le temps investi. Calcul explicite |
| **La taille du marché** | Combien de clients existent au monde. Petit est acceptable si le prix est élevé |
| **Le moat** | Ce qui empêche une copie en six mois. Les features ne sont pas un moat |
| **L'effet de réseau** | S'il existe. Entre clients, pas entre utilisateurs d'un même client |

**Gate :** si le seuil de viabilité dépasse la taille du marché, ou si le moat est « on ira plus vite », s'arrêter et le dire.

### Phase 4 — Les flux

Trois à six flux clés. Pour chacun : déclencheur · étapes · objets drillables · sortie · **signaux émis**.

Voir `references/output-template.md`.

### Phase 5 — La matrice de parité ⚠️ GATE

**La phase que tout le monde saute, et l'erreur la plus coûteuse.**

> **Un produit qui n'a que ses différenciateurs est une démo.**

Dérouler `references/parity.md` : les ~60 capacités que les concurrents ont toutes. Pour chacune, trancher :

| Verdict | Sens |
|---|---|
| **CONSTRUIRE** | On le fait, et à quelle version |
| **ACHETER** | Une brique tierce, nommée |
| **DIFFÉRER** | Plus tard, et on assume le manque en attendant |
| **REFUSER** | Jamais, et pourquoi c'est cohérent avec le segment |

**Gate :** une capacité de socle ni construite, ni achetée, ni explicitement différée, c'est un produit inutilisable qu'on découvrira à la livraison.

La parité n'est pas optionnelle. Elle est juste **tardive** (doctrine 12).

### Phase 6 — Features : input → action → output

Chaque feature sur une ligne, trois colonnes. **Une feature qu'on ne peut pas écrire ainsi n'est pas une feature, c'est une intention.**

Chaque ligne porte :
- **Le signal émis** (obligatoire — doctrine 4)
- **La couche** : socle / **parité** / **différenciant**
- La version de release

**Contrôle de couverture :** le blueprint doit contenir des features de parité, pas seulement des différenciantes. Un blueprint 100 % différenciant a raté la phase 5.

### Phase 7 — Automatisations

Trois niveaux. Pour chacune : déclencheur · entrée · sortie · niveau · **la décision qu'elle supprime ou améliore.**

Pas de décision touchée = pas d'automatisation.
**Attendu : ~70 % de niveau 2.**

### Phase 8 — Couche IA

**8a — Les features IA.** Même format, plus : le modèle et d'où il est appelé, ce qui se passe s'il se trompe, ce qui se passe s'il n'a rien à dire.

Couvrir les deux familles :
- **IA de production** — rédaction, traduction, correction, génération. C'est la parité ; les concurrents l'ont
- **IA relationnelle ou de jugement** — routage, mémoire, corrélation, arbitrage. C'est le différenciant

**8b — Le system prompt rédigé de chaque agent.** Rôle · données nommées · format de sortie · **sortie vide autorisée** · obligation de citer · ce qu'il ne fait jamais.

### Phase 9 — Modèle de données Convex

Schéma complet, prêt à coller. Voir `references/stack-stax.md`.
La primitive en première table · `entries` obligatoire · `syntheses` séparée · `tenantId` partout · un index par requête réelle.

### Phase 10 — Layout Stax

Carte des panneaux · inspecteurs · registre d'actions · état d'URL · boards canvas avec statuts et couleurs.

### Phase 11 — Positionnement

Quatre couches : accroche · **catégorie créée** · mécanisme · preuve.
Plus : tableau par contraste, vocabulaire propriétaire (5-8 termes), le prix et sa justification.

Interdit en externe : positionner par comparaison (« le X des riches »). Boussole interne, jamais un titre.

### Phase 12 — Go-to-market

| À produire | Détail |
|---|---|
| Le premier client, nommé | Pas un profil. Une personne ou une organisation |
| Le canal | Un seul. Comment il arrive |
| La séquence de vente | Qui parle à qui, dans quel ordre |
| Le pilote | À quoi ressemble le premier déploiement payant |
| **La réponse concurrente** | Ce que fait l'incumbent quand il te voit. Et ta parade |

### Phase 13 — Release, métriques, risques

Séquence v0 → v5 · **une seule métrique étoile** · les risques avec leur parade · le hors-scope explicite.

**Doctrine 7 :** on utilise le produit soi-même 12 mois avant de le vendre. La parité se construit en dernier et s'achète en attendant.

### Phase 14 — Livrables

1. **`<nom>-blueprint.md`** — 18 sections
2. **`<nom>-blueprint.html`** — l'artefact navigable

**Puis lancer le contrôle mécanique. Ce n'est pas optionnel non plus :**

```bash
bash scripts/blueprint-check.sh <dossier-du-blueprint> [--convex]
```

Il vérifie ce que la prose ne peut pas garantir :

| Famille | Ce qu'il contrôle |
|---|---|
| Manifeste | Branche tranchée, primitive nommée et jamais `post`/`content`/`message` |
| Gates | Les 3, et `/stack` refuse de construire sans eux |
| Phases | Chacune porte du contenu réel, pas le README du template |
| **Schéma** | Primitive en première table · `entries` et `syntheses` présentes · tenant partout · **aucun index sur un champ tableau** · **validation par les types Convex** · chaque table dans un flux |
| Parité | Verdicts tranchés · **les 8 familles A à H ouvertes** · la couche parité existe · le ratio de différenciant |
| **Marché** | Sources citées et **datées**, alerte au-delà de 90 jours, échec au-delà de 180 |
| Livrables | 18 sections · artefact self-contained · exactement 3 questions ouvertes |
| **Dérive** | Le schéma a-t-il changé depuis le build de l'app |
| R-NODASH | Zéro em ou en dash dans les livrables |

`--convex` valide le schéma contre les types réels du package Convex, ce qui attrape
un index sur un champ inexistant et un validator inventé. Il monte un bac à sable npm
la première fois, puis le réutilise et se lance tout seul.

**Les autres outils :**

```bash
bash scripts/convex-validate.sh <schema.ts>          # la validation Convex, seule
bash scripts/blueprint-diff.sh --all <blueprints/>   # ce que le portefeuille partage
```

---

## Phase 15 — Du blueprint au plan exécutable

*Un blueprint en markdown demande à l'agent de tout relire, de choisir un ordre à
chaque tour, et de se déclarer fini sur une impression. Ces trois gestes sont ceux
qu'il rate. La phase 15 les supprime.*

```bash
python3 scripts/stax_derive.py <blueprint> --write   # le layout, DÉDUIT du schéma
python3 scripts/plan_build.py  <blueprint> --write   # le plan d'exécution
python3 scripts/runner.py      <blueprint> status    # où on en est
python3 scripts/runner.py      <blueprint> next      # la prochaine étape, en entier
python3 scripts/runner.py      <blueprint> done <id> # ferme APRÈS vérification
```

### Le renversement Stax

**La grammaire de panneaux n'est pas une phase où l'on dessine des écrans : c'est une
conséquence du modèle de données.** Une table donne un panneau. Un `v.id("x")` donne
une action open-right vers le panneau de x. Un union de literals donne les statuts,
donc les couleurs du board. Dessiner ces écrans à la main revient à re-décider ce que
le schéma a déjà décidé, et les deux divergent au premier changement.

`stax_derive.py` produit `10-stax/panels.json` (la carte machine), `panels.md`,
`panels.mmd` et l'ERD `09-data/schema.mmd`. Il **propose** les espaces de premier
niveau et marque ses hypothèses ; le concepteur tranche.

### Les quatre blocs, et pourquoi le troisième décide de tout

Chaque étape du plan porte les quatre blocs de la doctrine Stax :

| Bloc | Ce qu'il porte |
|---|---|
| 1. Objectif | Ce que l'étape doit permettre |
| 2. Contraintes | Stack, invariants, dépendances |
| **3. Définition du fini** | **Une commande qui s'exécute**, pas une opinion |
| 4. Ne pas toucher | Le hors-périmètre, celui que tout le monde oublie |

Le bloc 3 décide de la **voie d'exécution** : vérifiable par une machine, l'étape part
en lot autonome ; sinon un agent à la fois, avec un humain devant.

Une étape dont les quatre blocs ne sont pas remplis est **rouge**, et le rouge est
bloquant. C'est ce qui empêche de lancer un agent sur du flou et de récupérer
900 lignes inutilisables. Le bloc 4 est celui qui évite les diffs de 900 lignes.

### Comment l'agent s'en sert

`runner.py next` donne **une** étape prête, avec ses quatre blocs. L'agent la fait.
`runner.py done <id>` **exécute** la définition du fini et **refuse de fermer** si elle
est rouge. `--force` passe outre, en le disant.

L'état vit dans `plan/state.json`, séparé de `plan.json` qui est régénérable :
recompiler le plan n'efface jamais ce qui a été fait.

`blueprint-diff` devient utile à partir de deux blueprints : il montre les tables et
les capacités de socle construites deux fois, celles qui méritent une brique commune
plutôt qu'un cinquième copier-coller. Il alerte aussi quand deux OS partagent leur
primitive, ce qui veut généralement dire qu'ils sont un seul produit.

> Ces contrôles existent parce que les invariants vivaient uniquement en prose : au
> premier vrai run, un index sur un champ tableau, deux tables obligatoires absentes
> et trois tables sans flux ont été trouvés à la lecture, pas par un contrôle. Le
> jugement de celui qui déroule le skill n'est pas un filet.

**Le graphe de nœuds doit couvrir les deux couches.** Les nœuds de parité existent, même marqués « différé v4 ». Un artefact qui ne montre que le différenciant ment sur l'ampleur du travail.

Terminer par **exactement trois questions ouvertes.**

---

## Les 18 sections du blueprint

```
1.  Résumé
2.  Le problème
3.  La primitive
4.  Le marché
5.  Utilisateurs
6.  Le modèle économique          ← phase 3
7.  Objectifs et non-objectifs
8.  Les flux
9.  La matrice de parité          ← phase 5
10. Modèle de données
11. Features — socle / parité / différenciant
12. Automatisations
13. Couche IA — production et jugement
14. Layout Stax
15. Positionnement
16. Go-to-market                  ← phase 12
17. Release, métriques, risques
18. Trois questions ouvertes
```

---

## Règles de conduite

**Challenger avant de concevoir.** La phase 0 n'est pas une formalité. Une idée qui ne survit pas à dix-huit questions ne survivra pas à dix-huit mois.

**Chercher réellement.** Phase 1 sans recherche web = blueprint inventé.

**Ne jamais deviner.** `[non tranché]` plus ce qui le trancherait (doctrine 11).

**Couvrir la parité.** Un blueprint sans couche de parité est incomplet, quelle que soit la qualité du différenciant.

**Une phase, un livrable.** Montrer le résultat intermédiaire avant d'enchaîner.

**Le stack ne se discute pas.** Si une phase suggère autre chose, c'est la conception qu'on change.

**Le segment ne se discute pas.** Une feature qui n'a de sens que pour un public de masse est hors scope, même si elle est bonne.

**Écrire en français.** Vocabulaire produit en français, identifiants techniques en anglais.

---

## Fichiers de référence

| Fichier | Quand |
|---|---|
| `references/doctrine.md` | **Toujours, avant la phase 0** |
| `references/challenge.md` | **Phase 0 — l'interrogatoire** |
| `references/market-elite.md` | Phases 1 et 11 |
| `references/parity.md` | **Phase 5 — la matrice** |
| `references/stack-stax.md` | Phases 9 et 10 |
| `references/output-template.md` | Phases 4 et 14 |

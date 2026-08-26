# Blueprint OS, v1.0.0

**Categorie :** Product Stack
**Position Omega :** Product Stack : Pack de Definition Produit + Technique, premiere etape de la branche IMPLEMENT (`Strategy -> Blueprint -> Design -> Stepper -> Builder`)
**Interface principale :** conversationnelle, sur un etat de projet canonique unique
**Statut :** implementation de reference installable

## Objet

Compiler des idees de logiciel, IA, plateforme, service, marketplace, mobile, web ou outil interne, ainsi que le contexte d'un projet existant, en un Pack de Definition Produit + Technique complet, coherent et tracable, avant toute planification d'implementation ou tout code. Blueprint OS n'ecrit jamais de code produit et ne cree jamais d'etapes DEV atomiques : il definit la verite produit et systeme, puis s'arrete et passe la main.

## Frontiere

`Idea -> Blueprint {OS} -> Stepper {OS} -> Build {OS} -> Ship`

Blueprint definit les contrats produit/UX/domaine/donnees/API/IA/securite/operations/test et s'arrete a `BLUEPRINT COMPLETE, STEPPER READY`. Stepper cree le DAG d'implementation et s'arrete a `BUILD READY`. Build ecrit le code et modifie les systemes. Blueprint n'invoque jamais Stepper ni Build de maniere implicite.

## Ce que contient cet OS

- 7 documents de reference sous `references/` (prompt systeme, contrat blueprint, orchestration et portes, reponse et continuation, fonctions et etat, guide approfondi, integration Omega OS)
- 10 scripts python/shell sous `scripts/`, dont les outils CLI `blueprint-check.sh` et `blueprint-diff.sh` (verifications mecaniques des invariants et comparaison entre blueprints), le typechecker de schema `convex-validate.sh`, la CLI d'etat local `blueprint_os.py`, `install_omega_os.py`, et le pipeline de derivation Stax (`stax_derive.py`, `stax_emit.py`, `plan_build.py`, `runner.py`, `scripts/lib/schema_parse.py`)
- 4 assets JSON sous `assets/` : la config des prompts de role (`blueprint-role-prompts.json`), le schema de l'etat canonique (`blueprint-state.schema.json`), les definitions d'appels d'outils (`blueprint-tools.json`) et le manifest du plugin Omega OS (`omega-os.manifest.json`)
- un dossier `legacy/` (`SKILL-v1.md` plus 6 documents de reference anterieurs) conserve pour l'historique uniquement ; il est supplante et n'est jamais lu comme verite courante

Aucun fichier `agents/*.md` n'existe dans ce pack. Les roles sont pilotes entierement par `assets/blueprint-role-prompts.json` (context-librarian et les autres roles specialises), et non par des fichiers d'agent autonomes.

## Modes

`NEW` / `RECOVER` / `EXTEND` / `REVISE` / `AUDIT` / `DELTA`, selectionnes par execution et preserves d'une continuation a l'autre.

## Commandes

| Commande | Mode | Objet |
| --- | --- | --- |
| `/blueprint` | infere | Compiler ou poursuivre le Pack de Definition Produit + Technique |

## Principaux passages de relais

- Strategy & Portfolio OS fournit le pari produit approuve (`strategy.product_bet.approved`).
- Market Research OS fournit les preuves validees (`market.validation.completed`).
- Brainstorm OS fournit le concept selectionne (`brainstorm.concept.selected`).
- Blueprint OS produit `blueprint.completed`, consomme par Design OS et Stepper OS.

## Declencheurs

`/blueprint`, "Blueprint {OS}", "product blueprint", "product-definition audit", "product-definition recovery", "product-definition revision", "product-definition extension", "product-definition delta", "prepare for Stepper {OS}".

### Declencheurs (FR)

"compile ce blueprint", "definition produit et technique", "audit de definition produit", "prepare le blueprint pour le stepper", "recupere l'etat canonique du projet", "etends ce blueprint", "revise cette decision produit".

## Installation

Voir `OMEGA_INTEGRATION.md` pour l'enregistrement, l'ordre d'injection du contexte, les passages de relais, les types d'evenements et la classification d'etat.

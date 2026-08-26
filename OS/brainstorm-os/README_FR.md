# Brainstorm {OS}

## Objectif

Faire tourner un conseil rigoureux, multi-agents, d'imagination, d'evolution et de decision, qui transforme une idee brute en une population de concepts challenges, evolues et prets a la decision, en protegeant l'intention du fondateur tout en exigeant une vraie dissidence, une discipline de la preuve et une convergence explicite.

## Position

Core Stack (support) : le point d'entree de l'ideation de la suite, en amont de Market Research OS et de Blueprint OS.

`Idee brute -> Brainstorm {OS} -> Market Research (optionnel) -> Blueprint {OS} -> Stepper {OS} -> Builder {OS}`

## Ce que contient cet OS

- Un adaptateur d'agent, `agents/openai.yaml`, un manifeste d'interface (nom affiche, description, prompt par defaut, icone) qui pilote les roles du conseil nommes dans `assets/council-profiles.json`. Il n'y a pas de fichiers `.md` par agent ; les roles (cellules Expansion, Reality, Adversarial, plus les chambres d'imagination et d'evolution et les chapeaux de specialistes) sont definis dans le JSON des profils et dans `references/council-and-debate.md`, et sont lances comme des passes d'agent independantes par le contrat operatoire, non comme des fichiers d'agent separes.
- 12 documents de reference sous `references/` : `operating-contract.md` (lois, etapes, portes), `council-and-debate.md` (topologie, roles, rounds de debat), `imagination-and-evolution.md`, `methods-and-lenses.md`, `omega-os-integration.md`, `output-and-handoffs.md`, `quality-and-evals.md`, `research-and-evidence.md`, `specialist-councils.md`, `surface-lab.md`, `system-prompt.md`, `agent-prompts.md`.
- 3 scripts Python sous `scripts/` : `brainstorm_os.py` (etat de session : init, migrate, enregistrement des frames/genomes/generations, comparaison des surfaces, incubation, audit, freeze, export, handoff, validate), `install_omega_os.py` (installation Omega OS explicite uniquement) et `test_brainstorm_os.py` (la suite de tests du moteur de session).
- 5 assets sous `assets/` : `council-profiles.json` (definitions des chambres/cellules/specialistes), `session.schema.json` (JSON Schema d'une session), `surface-profiles.json` (criteres d'incarnation portables), `omega-extension.json` et `package-manifest.json` (metadonnees de packaging), et `icon.svg`.

## Commandes

Commande racine : `/brainstorm`

Aucune commande alias n'est enregistree au niveau de la suite ; les commandes d'interaction (`/brainstorm --deep`, `/ideate --wild`, `/frame-fission`, `/evolve`, `/collision`, `/worlds`, `/converge`, `/handoff blueprint`, et les autres) sont des sous-commandes conversationnelles documentees dans `SKILL.md`, non des entrees racine separees.

## Principaux handoffs

- Produit `brainstorm.concept.selected`, consomme par Market Research OS (pour la validation) et Blueprint OS (pour un concept pret a la decision quand la validation est explicitement sautee).
- Ne consomme rien en amont : Brainstorm {OS} est le point d'entree de l'ideation de la suite.
- Emet aussi `brainstorm.session.completed` et depose la lignee du concept dans Context & Memory OS.

## Declencheurs

A utiliser pour : `/brainstorm`, Brainstorm {OS}, ideation, challenger ou faire evoluer une idee, directions plus folles et non evidentes, conseils d'agents, angles morts, red teams, premortems, retour de preuve, audits de debat, ou convergence. Aussi pour choisir une incarnation mobile, web, desktop, multi-surface, chat, API, ambiante, physique, service, ou sans interface, et pour preparer des idees vers Market Research {OS}, Blueprint {OS}, des decisions, des experimentations, ou des briefs creatifs/projet.

## Declencheurs (FR)

- "brainstorm sur cette idee"
- "session de creativite"
- "conseil de personas"
- "genere des concepts"
- "fais evoluer cette idee"
- "challenge mon idee"
- "quelle direction choisir"

## Plus de detail

Voir `OMEGA_INTEGRATION.md` pour l'enregistrement, l'ordre d'injection du contexte, les types d'evenements et la classification des etats.

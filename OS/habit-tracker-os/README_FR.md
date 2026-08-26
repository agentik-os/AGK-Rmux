# Habit Tracker {OS}, v1.0.0

**Categorie :** Stack Personnelle / Intelligence du comportement et des habitudes  
**Position Omega :** Stack Personnelle : contrats de comportement recurrents, preuves de check-in quotidiennes et revues adaptatives  
**Interface principale :** conversationnelle + moteur d'etat deterministe  
**Statut :** implementation de reference installable

## Objectif
Faire tourner un systeme d'habitudes conversationnel assiste par LLM : creer de bonnes habitudes, reduire ou arreter celles qui ne sont pas souhaitees, mener les check-in du matin et du soir, gerer les envies et les rechutes, et produire des revues adaptatives et des rapports de progression visuels. Le chat est l'interface, jamais la base de donnees : le coaching humain est couple a un etat deterministe, a des preuves explicites et a des adaptations reversibles.

## Promesse
Permettre a l'utilisateur de parler de son comportement aussi naturellement qu'avec n'importe quel LLM, pendant que chaque completion, serie, revue et adaptation est adossee a un enregistrement type et etiquete par provenance, qui reste inspectable, modifiable, exportable et supprimable.

## Boucle operationnelle

```text
ORIENTER -> RECUPERER -> INTERPRETER -> ENREGISTRER -> COACHER -> ADAPTER -> CLORE
```

## Position dans la chaine de valeur
Mindset {OS} possede les valeurs, l'identite, les croyances, les intentions et la direction de vie. Habit Tracker {OS} possede la moitie comportementale : contrats, observations, interventions, experiences et revues. Il recoit l'intention de Mindset {OS}, la transforme en contrats suivables, collecte les preuves quotidiennes, et renvoie vers le haut les motifs et les reflexions. Il ne redefinit jamais l'identite ni les objectifs, et il ne revendique jamais `BUILD READY` (ce statut appartient a Stepper {OS}).

```text
Intention Mindset -> Contrat d'habitude -> Preuve quotidienne -> Motif/revue -> Reflexion Mindset
```

## Ce que contient cet OS
- Prompt systeme canonique (`references/system-prompt.md`) et limites de securite explicites
- 9 documents de reference : prompt systeme, protocoles de conversation, modele de domaine, science du comportement, analytique et visuels, securite et limites, integration Omega, catalogue de fonctionnalites, suite d'evaluation
- 1 manifeste d'interface produit sous `agents/` (`openai.yaml`, la surface ChatGPT/Codex/API/Atlas) ; il n'y a pas de roster d'agents specialises separe
- Pas de repertoire `skills/` dedie : le pack est lui-meme un Skill, ouvert via `SKILL.md`
- Pas de repertoire `protocols/` dedie : les protocoles operationnels (setup, check-in, envie, rechute, revue, adaptation, recuperation) vivent dans `references/conversation-protocols.md`
- 3 scripts : `scripts/habit_os.py` (moteur d'etat deterministe), `scripts/install_omega_os.py` (installeur), `scripts/test_habit_os.py` (suite de tests)
- 4 assets : `habit-state.schema.json` (schema d'etat), `tool-contracts.json` (13 contrats d'outils types), `omega-os.manifest.json` (enregistrement Omega), `icon.svg`
- Moteur d'etat local adosse a SQLite avec une suite d'evaluation verifiable par machine (`references/evaluation-suite.md`)

## Routeur de session
La boucle de coaching selectionne l'un des neuf modes par tour : `SETUP`, `TODAY`, `CHECK_IN`, `URGE`, `LAPSE`, `REVIEW`, `RECOVER`, `ADAPT`, `VISUALIZE`. Voir la table du routeur dans `SKILL.md` pour les signaux declencheurs et l'action requise par mode.

## Commandes
La commande d'entree par defaut est `/habits` (enregistree dans `OMEGA_INTEGRATION.md`). Le pack declare cette surface de commandes dans `assets/omega-os.manifest.json` :

| Commande | Objet |
| --- | --- |
| `/habit setup` | Construire un contrat d'habitude lie a l'identite et une base de reference |
| `/habit today` | Classer au plus sept items prioritaires et expliquer pourquoi |
| `/habit checkin` | Analyser la preuve, enregistrer le resultat, renvoyer le prochain pas |
| `/habit correct` | Remplacer un journal errone et invalider les revues derivees |
| `/habit urge` | Lancer le protocole d'envie pour reduire la latence avant analyse |
| `/habit review` | Calculer une revue bornee par les preuves, avec confiance et lacunes de donnees |
| `/habit recover` | Entrer ou proposer une saison de recuperation, reduire la charge, preserver l'essentiel |
| `/habit adapt` | Creer une experience versionnee avec criteres de succes et de retour arriere |
| `/habit experiment` | Creer une experience de comportement bornee, a un seul changement |
| `/habit chart` | Rendre le plus petit visuel Mermaid ou tableau valide |
| `/habit export` | Exporter l'etat et l'historique detenus par l'utilisateur (JSON ou CSV) |
| `/habit delete` | Supprimer ou caviarder les journaux, habitudes ou tout l'etat de l'utilisateur |

### Moteur local
`scripts/habit_os.py` fournit la persistance deterministe, les calculs, les exports et la generation Mermaid derriere ces commandes. Sous-commandes : `init`, `add`, `update`, `list`, `log`, `correct`, `today`, `review`, `chart`, `context`, `export`, `season`, `experiment`, `delete`, `doctor`. Lancer `python3 scripts/habit_os.py --help` pour le detail.

## Principaux echanges
- Mindset {OS} fournit les contrats de comportement a suivre (consomme `mindset.behavior_contract.created`) et recoit la reflexion qui boucle la chaine (`habit.review.completed`).
- Health & Energy {OS} fournit les routines convenues (consomme `handoff.habits.created`).
- Context & Memory {OS} stocke les observations de check-in canoniques : chaque observation confirmee est mise en scene via `memory.record.staged` et renvoyee en `memory.record.verified` ; cet OS ne garde qu'une projection locale indexee pour des recherches rapides de series et d'analytique, jamais la source de verite.
- Stepper {OS} possede le statut `BUILD READY` ; les statuts de Habit Tracker sont `DRAFT`, `ACTIVE`, `PAUSED`, `RECOVERING`, `RETIRED`, `ARCHIVED`.
- Review & Governance {OS} approuve les changements de limites, de schemas ou de portes qualite en production.
- Toute application externe de type "Life OS" est une dependance explicite hors suite, jamais un membre implicite de cette suite.

## Installation
Voir `OMEGA_INTEGRATION.md` pour l'enregistrement Omega et l'ordre d'injection du contexte, et lancer `scripts/install_omega_os.py` pour installer le moteur local.

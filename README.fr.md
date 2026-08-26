# OmegaOS

Un plan de contrôle en terminal pour piloter en parallèle une flotte d'agents de codage IA, où chaque agent obéit au même corpus de règles typé.

[English](README.md) | Français | [Русский](README.ru.md) | [中文](README.zh.md) | [Español](README.es-ES.md)

> Le [README anglais](README.md) est la version canonique et la plus à jour ; cette traduction peut avoir un temps de retard.

[![CI](https://github.com/agentik-os/OmegaOS/actions/workflows/ci.yml/badge.svg)](https://github.com/agentik-os/OmegaOS/actions/workflows/ci.yml) ![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg) ![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange.svg)

OmegaOS n'est pas une bibliothèque qu'on importe. On l'installe sur une machine Linux et on récupère la commande `omega`, une TUI pour surveiller et tuer les sessions, une couche d'orchestration qui distribue le travail aux agents, et un pont Telegram pour piloter le tout depuis son téléphone. Les nouvelles sessions utilisent OpenAI Codex par défaut. Claude Code, Google Antigravity, Gemini CLI pour comptes Enterprise/API, OpenRouter/Pi, Hermes, GLM et Kimi restent disponibles comme choix explicites. Chaque agent reçoit un contexte de règles compact, typé et cadré à son rôle, compilé depuis la même doctrine.

Version courante : voir [CHANGELOG.md](CHANGELOG.md) (`omega -V` sur une machine installée). Je m'en sers tous les jours ; il faut s'attendre à quelques aspérités.

## Installation

Une seule commande sur une machine Linux (macOS fonctionne dans l'ensemble) :

```
npx omega-os
```

Elle clone le dépôt et lance l'installateur derrière un écran de progression interactif en pluie de code Matrix (taper pour injecter des glyphes, `space` pour une impulsion ; `npx omega-os --plain` pour une barre classique). Pour le faire à la main :

```
git clone https://github.com/agentik-os/OmegaOS
cd OmegaOS
./install.sh
```

L'installateur télécharge des binaires `rmux` + `omega` + `omega-gatewayd` précompilés pour la plateforme courante lorsqu'une release est publiée (vérifiés par somme de contrôle), et retombe sinon sur une compilation depuis les sources — un clone frais reproduit donc toujours le système, simplement plus vite quand un binaire existe. Pour forcer la compilation depuis les sources : `OMEGA_FROM_SOURCE=1 ./install.sh`.

## Mise à jour

```
omega update           # fetch + fast-forward + réinstallation
omega update --check   # qu'est-ce qui changerait ? (ne touche à rien)
```

La commande met à jour la copie locale qu'elle trouve (`$OMEGA_SRC`, le
répertoire courant, puis `~/Station/SideBusiness/OmegaOS`, `~/Station/OmegaOS`,
`~/OmegaOS` — ou passer `--dir`), recompile depuis les sources, et relance
l'installateur. L'état de `~/.omega` est préservé : les secrets, les projets,
la configuration Telegram et `config.toml` ne sont jamais écrasés.

En présence de modifications locales ou de commits non poussés dans la copie
locale, la mise à jour **s'arrête et le signale** au lieu de toucher au travail
en cours — commiter, mettre de côté (`git stash`) ou pousser, puis la relancer.

Elle se tient aussi à jour toute seule : chaque installation vérifie tous les
jours à 03:30 et installe ce qu'elle trouve, en sautant toute nuit où la copie
locale contient du travail en cours, où un agent est en plein tour, ou bien où
le même commit a déjà échoué trois fois. Installer automatiquement revient à
faire confiance au dépôt chaque nuit, donc le changement tient en une commande :

```
omega config set auto_update check   # me prévenir au lieu d'installer
omega config set auto_update off     # ne rien faire du tout
```

## Les 5 premières minutes

La stack s'installe toute seule ; il ne reste que les pièces personnelles.
**`omega guide` affiche le pas-à-pas complet** (également enregistré dans
`~/.omega/GETTING-STARTED.md`, et affiché à la fin de l'installation). En
résumé :

1. **Connecter Codex** *(indispensable pour le runtime par défaut)* : lancer `omega codex-login`, puis vérifier avec `omega codex-login-status`. Claude reste optionnel via `claude auth login`.
2. **Pilotage Telegram à distance** *(recommandé)* — le token vient de [@BotFather](https://t.me/BotFather), l'identifiant de [@userinfobot](https://t.me/userinfobot), puis `OMEGA_TG_TOKEN=<TOKEN> omega telegram setup <ID> --user-id <ID>` (la forme avec variable d'environnement garde le token hors de la liste des processus). Pour un topic par projet : groupe + Topics activés + bot administrateur → `/setupgroup` → `/sync`.
3. **Clés de services** *(facultatif)* — `~/.omega/provisioning/services.env` (Vercel / GitHub / Convex / Stripe / OpenAI pour la voix) alimente le provisionnement automatique des nouvelles applications.
4. **Ajouter un projet** — `omega` → **[N] New Project**, Telegram → *Import from GitHub*, ou simplement déposer un dépôt sous `~/Station/<Category>/`.
5. **Vérifier** — `omega doctor` : toutes les lignes en `[+]`.

Voici une exécution réelle de `omega doctor` :

```
OmegaOS doctor

  [+] binary           omega 0.1.6
  [+] rmux daemon      connected, 6 live session(s)
  [+] rmux socket      /tmp/rmux-1000/default
  [+] doctrine         7 Laws + 47 Rules
  [+] agent CLI        codex available
  [+] state dir        /home/vibe/.omega/state
  [+] telegram service omega-tg-bot active
  [+] hooks            track + verify present, registered in settings.json
  [+] secrets dir      /home/vibe/.omega present
  [+] memory           249088MB available
  [+] usage cache      usage cache 1 min old
  [+] codex auth       Codex login valid
  [+] telegram poller  1 poller
  [+] provisioning     provisioning: VERCEL_TOKEN, CONVEX_TEAM_TOKEN, STRIPE_SECRET_KEY
```

Les lignes `[!]` sont des avertissements dont la commande de réparation est indiquée sur la ligne même ; `omega doctor --fix` répare celles qui sont mécaniques.

## Ce qu'on peut faire

- **Dépêcher des missions.** `omega dispatch <Project> "<mission>"` confie le travail à l'oracle de ce projet, qui planifie, engendre des workers et soumet le résultat au contrôle qualité. `omega orchestrate` déroule en une seule commande le pipeline complet classer → planifier → dépêcher → surveiller → contrôler.
- **Exécuter des plans typés.** `/omg-planner` décompose une construction en un DAG typé (`.planner/tracker.json`) ; `omega plan-run` l'exécute avec une contrainte structurelle qui interdit de sauter une étape (Gate) et la preuve d'une commande de vérification indépendante (Guardian).
- **Amorcer des applications entières.** `/omg-new-project` provisionne Vercel/Convex/GitHub/Clerk/Stripe à partir des clés fournies, échafaude la stack, puis déroule vision → PRD → plan → build.
- **Paralléliser sans risque.** Les workers revendiquent leurs fichiers avec de vrais verrous consultatifs (`fs2`), et `omega spawn-worker --worktree` donne à chaque worker parallèle son propre worktree git, avec une fusion propre à la fin. Un signal de fin crée un résultat candidat. Seuls un vérificateur indépendant et la porte d'acceptation de la mission peuvent le clore.
- **Tout auditer.** Un Quality Arsenal de 23 audits forensiques Gestalt-Popper (`codeaudit`, `secaudit`, `perfaudit`, `a11yaudit`, …) sélectionnés automatiquement selon ce qui a changé, plus `/omg-acceptance` — une porte d'acceptation autonome, dans le navigateur, qui balaie chaque route et corrige ce qu'elle trouve.
- **Convoquer un conseil.** `/omg-llm-council` pose une même question à quatre modèles Claude différents en parallèle, les fait s'évaluer mutuellement de façon anonyme, et synthétise un verdict en préservant les désaccords — sans clé d'API, tout tourne dans la session existante.
- **Naviguer de façon agentique.** `/omg-browser-use` pilote un navigateur dans le cloud pour les tâches qu'un script Playwright ne sait pas exprimer.
- **Faire aussi le go-to-market.** Un pack marketing embarqué (étude de marché, positionnement, stratégie de contenu, social, cold email, création publicitaire, stratégie de lancement) et la paire d'identité visuelle Higgsfield.
- **Recevoir les rapports sur son téléphone.** Chaque mission se termine par un rapport PDF aux couleurs de la marque dans le topic Telegram du projet, et une carte de progression en direct se met à jour sur place pendant qu'elle se déroule. Un bot de dépôt donne aux agents une boîte de réception privée pour les fichiers envoyés depuis le téléphone.
- **L'exploiter.** `omega doctor` (santé de toute la stack), `patrol` (chien de garde des sessions), `usage` (budget de tokens + alertes Telegram), `backup` (l'état non reproductible de `~/.omega` → un seul tgz), `cleanup` / `kill-all`, `timeline` (rejouer une mission), `resurrect` (ranimer un oracle planté), `provision` (groupes d'identifiants par client).
- **Résoudre des tickets Linear de bout en bout.** `/omg-linear` corrige, capture les preuves, audite jusqu'à 100/100, commente et déplace le ticket en revue — jamais en Done ; c'est un humain qui le fait. Voir [Intégration Linear](#intégration-linear).

Trois portes d'entrée : la TUI `ratatui` (5 onglets : Sessions, Menu, Agentic, Settings, Help), la CLI `omega` (plus de 40 commandes) et le hub Telegram. Un mode RPC (JSONL sur stdin/stdout) permet de le piloter depuis d'autres outils. En dessous, tout repose sur [rmux](https://github.com/agentik-os/rmux), un multiplexeur de terminal en Rust — sans dépendance à tmux.

## La doctrine

Il existe un registre typé de 7 Lois et de 47 Règles opérationnelles nommées. `omega rules list` affiche l'ensemble courant. Le compilateur vit dans `crates/omega-core/src/rules.rs` ; il émet un contexte déterministe et adapté au fournisseur, sous un budget OmegaOS strict de 24 Ko.

**Les Lois sont inviolables.** Elles s'imposent à chaque agent et priment sur toute règle et toute tâche. Il y en a sept :

- **L0 — Livrer la vérité.** Un changement n'est terminé que lorsqu'une recompilation propre le reproduit et qu'il est poussé. En dessous, c'est un brouillon.
- **L1 — Le runtime est la seule vérité.** Le code et les commentaires énoncent l'intention. Seule l'exécution révèle la réalité. En cas de désaccord, c'est le runtime qui tranche.
- **L2 — Chercheur, pas flagorneur.** Quand une prémisse est bancale, on la conteste avec des arguments avant d'agir. Pas de fausse assurance. « Ça devrait marcher » sans preuve, c'est un mensonge.
- **L3 — Décider et avancer.** Un agent dépêché est autonome. Il ne s'arrête jamais pour demander « est-ce que je continue ? » Il décide, exécute, et rend compte ensuite.
- **L4 — Terminé veut dire 100 %, vérifié.** 92 %, ce n'est pas terminé. On énumère les tâches, on finit chacune, on vérifie chacune face au runtime.
- **L5 — La qualité avant la vitesse.** Pas de variante allégée, simplifiée ou expédiée d'un vrai protocole. Un 403 ou un 401, c'est un abandon, pas un succès.
- **L6 — Finir la mission.** Énumérer, exécuter, vérifier et rapporter chaque livrable demandé. Un plan ou une phase partielle n'est pas un arrêt valide.

**Les Règles sont opérationnelles.** Nommées (R-SCOPE, R-VERIFY, R-CITE…), réparties entre Universal, QualityGate, Orchestration, Reporting et Safety. Chaque Règle est cadrée selon les rôles auxquels elle s'applique : Master, Oracle, Worker. On n'encombre pas un worker de règles d'orchestration sur lesquelles il ne peut rien, et un oracle n'hérite pas de la discipline de verrouillage de fichiers du worker. Même registre, tranches différentes.

### L'entonnoir

C'est là qu'est le mécanisme. `rules::compile_rule_context_for_provider` combine le noyau compact des Lois, le contrat du rôle, les Règles pertinentes pour la mission, la mécanique du fournisseur et les références aux skills. Il refuse toute sortie dépassant le budget de contexte au lieu de la tronquer en silence. Chaque contexte compilé porte une empreinte déterministe qui permet d'en détecter la dérive.

Un worker situé trois niveaux plus bas dans l'arbre porte les sept mêmes Lois que le Master. Les procédures opérationnelles ne sont chargées que lorsque le rôle, la mission, le risque et le fournisseur l'exigent. Les invariants restent ainsi universels sans injecter tous les runbooks à chaque tour.

Pour tout voir :

```
omega rules list
```

![omega rules list — les Lois et les Règles, affichées par OmegaOS](assets/omega-rules.svg)

## Architecture

Quatre niveaux, de haut en bas :

```
┌─────────────────────────────────────────────────────────────────┐
│  Niveau 1 — Interface humaine                                   │
│  TUI (5 onglets) · CLI (40+ cmds) · hub Telegram                │
│                      ↓ intention                                │
├─────────────────────────────────────────────────────────────────┤
│  Niveau 2 — Master (cerveau persistant — le topic Atlas)        │
│  14 templates d'agents nommés Matrix, classer → aiguiller       │
│                      ↓ dépêche                                  │
├─────────────────────────────────────────────────────────────────┤
│  Niveau 3 — Oracle (1 par projet)                               │
│  Classer → Planifier → Dépêcher les workers → Contrôle qualité  │
│                      ↓ décomposition                            │
├─────────────────────────────────────────────────────────────────┤
│  Niveau 4 — Workers (éphémères, parallèles, cadrés par verrou)  │
│  Exécuter → Vérifier → done.json → Oracle accuse → clôture      │
└─────────────────────────────────────────────────────────────────┘
```

**Niveau 2 — le Master.** Un agent persistant qui reste actif, redémarre tout seul s'il meurt, et reprend sa propre conversation. Il embarque 14 templates d'agents nommés d'après des personnages de Matrix (Oracle, Morpheus, Seraph, Keymaker, Smith, Niobe, Architect, Merovingian, Neo, Zion, Link, Construct, Pythia, Council). Le Master est un répartiteur. Il se contente de classer le travail et de l'aiguiller vers les oracles.

**Niveau 3 — Oracle.** Un par projet. Il classe la demande, planifie, dépêche les workers, et passe le contrôle qualité à la fin. Un oracle orchestre. Il n'édite pas lui-même le code du projet, si bien que le correcteur et le rédacteur ne sont jamais le même agent.

**Niveau 4 — Workers.** Éphémères. Ils tournent en parallèle, et chacun est cadré à ses propres fichiers par une revendication de verrou (verrous consultatifs via `fs2`) — et, en option, à son propre worktree git. Un worker signale qu'il a fini en écrivant un `done.json` dont le statut vaut `done_clean`, `pending` ou `failed` ; sans ce statut, ce n'est pas terminé.

### Comment se déroule une mission

Une requête arrive par la TUI, la CLI ou Telegram. Quel que soit son point de départ, elle atterrit sur le Master, qui la lit, la classe, et l'aiguille vers l'oracle responsable du projet concerné. L'oracle planifie la mission, la découpe en tâches, et dépêche un worker par tâche. Les workers confrontent leurs propres résultats au comportement réel à l'exécution et écrivent leur `done.json` ; l'oracle le lit, passe le contrôle qualité, et fait remonter le rapport dans la chaîne.

Un worker n'est pas obligé d'enchaîner ses sous-tâches une à une. Il peut exécuter un Workflow dans son propre processus : engendrer des sous-agents en parallèle, vérifier leurs sorties, et les fondre en une seule réponse. La revue de code s'en sert, tout comme la recherche, les audits et le travail de design.

La vérification est délibérément contradictoire : un worker qui rapporte « terminé » ne clôt pas le contrôle ; son affirmation passe par des agents indépendants, et ne survit que si une majorité (deux sur trois) s'accorde. Les audits du Quality Arsenal se branchent précisément ici, au niveau du contrôle.

Tout cela repose sur l'entonnoir doctrinal décrit plus haut : chaque agent, à chaque niveau, reçoit les Lois et Règles cadrées à son rôle au moment même où il est dépêché.

Cette section du README en est elle-même un exemple. Elle est née d'un Workflow. Un agent en a rédigé le brouillon, des relecteurs indépendants l'ont passé au crible en traquant la prose générée par IA, un autre agent l'a révisé d'après leurs signalements, et des locuteurs natifs en ont assuré la traduction. Ainsi, aucune partie de ce texte n'est issue d'une seule passe non relue.

## Stack

C'est un workspace Rust avec trois crates :

- `omega-core` — l'orchestration, le registre de règles, le doctor, la timeline, le nettoyage, la patrouille, le verrouillage par portée de fichiers.
- `omega-cli` — le binaire `omega`, bâti sur `clap`.
- `omega-tui` — le gestionnaire de sessions, bâti sur `ratatui`.

En dessous, ça repose sur [rmux](https://github.com/agentik-os/rmux), un multiplexeur de terminal en Rust : un daemon, un SDK typé, et la gestion des PTY. rmux est une bibliothèque Rust typée, donc OmegaOS l'appelle directement au lieu de lancer tmux en sous-processus et d'en parser la sortie texte. Il n'y a aucune dépendance à tmux nulle part.

Bun et TypeScript se chargent du rendu des rapports PDF (via Next.js et Playwright) et des bots Telegram. Bash n'apparaît qu'à un seul endroit : le bootstrap d'installation.

## Se connecter à distance

Le daemon rmux possède chaque session, donc les agents continuent de tourner une fois la connexion coupée. Pour les retrouver, il faut faire un **attach** — rebrancher son terminal sur une session déjà en cours :

```
rmux attach              # se rebrancher sur la dernière session
rmux attach -t claude-1  # se brancher sur une session précise
rmux list-sessions       # voir ce qui tourne
```

Se détacher à nouveau avec `Ctrl-b d` — la session et ses agents continuent de tourner de leur côté.

`omega` regroupe les points d'entrée dont on se sert vraiment :

```
omega                       # ouvrir le gestionnaire de sessions en TUI (parcourir / lancer / surveiller)
omega attach -t claude-1    # entrer directement dans une session pour y travailler
omega master                # sauter à la session Master
omega list                  # lister toutes les sessions actives
```

Le menu (`omega`) sert à gérer et à lancer ; l'attach direct (`omega attach -t …`, ou `rmux attach -t …`) sert à taper tête baissée dans une seule session — l'aperçu du menu *reflète* le panneau, tandis que l'attach direct est le chemin de plus faible latence.

En SSH depuis un portable, le SSH classique attend un aller-retour réseau complet avant d'afficher chaque frappe : sur une machine lointaine, la saisie paraît donc poussive et la sortie des agents arrive par à-coups — quelle que soit la puissance de la machine, parce que c'est de la latence, pas du CPU. `install.sh` installe [`mosh`](https://mosh.org) pour ça : il affiche les frappes en local et envoie les diffs d'écran en UDP, si bien que la saisie est instantanée et le flux fluide à n'importe quelle latence. Pour se connecter directement dans une session :

```
mosh user@host -- omega attach -t claude-1
```

Dans un client comme **Termius** : renseigner l'IP et le port de la machine, activer l'interrupteur **mosh**, et ajouter un snippet de démarrage — `omega` pour le menu, ou `omega attach -t <session>` pour atterrir directement dans une session.

(Pour le défilement arrière, utiliser `Alt+Up/Down` de rmux, pas le PageUp de mosh.) L'installateur câble aussi `/etc/rmux.conf` et une locale UTF-8 à l'échelle du système, si bien que chaque compte — root comme les utilisateurs à venir — hérite de la même session durcie (molette de souris, sélection à la souris vers le presse-papiers local à travers SSH, touches réactives, truecolor) sans aucune configuration par utilisateur.

## Intégration Linear

Quand les retours utilisateurs sont suivis dans [Linear](https://linear.app), OmegaOS résout les tickets de bout en bout. Deux commandes.

`/omg-linear-setup` est un assistant à lancer une seule fois, à l'intérieur de l'application. Il installe un widget de retour intégré (il enregistre une capture d'écran, l'URL de la page, l'élément cliqué et la console du navigateur au moment du signalement), les labels Linear sur lesquels le pipeline s'appuie, et la route d'API qui transforme un signalement du widget en issue Linear. Il détecte d'abord la stack, le fournisseur d'authentification et la bibliothèque d'UI, de sorte qu'il écrit du code adapté au projet plutôt qu'un gabarit générique.

`/omg-linear` fait le travail. Il lit les tickets ouverts et, pour chacun, corrige le code, capture les preuves avant/après, puis lance les audits du Quality Arsenal adaptés au changement. Un ticket n'avance que si ces audits atteignent 100/100. Il publie ensuite un commentaire de vérification du correctif sur le ticket et le déplace dans un état de revue — `In Review` si l'équipe en a un, sinon un `Omega Review` neutre qu'il crée lui-même. Il ne passe jamais un ticket en Done ; c'est un humain qui le fait après contrôle. Le moteur v2 déroule tout ça dans un Workflow : il trie les tickets ouverts, répartit en parallèle la correction et l'audit de chaque ticket, et vérifie chaque résolution de façon contradictoire avant de commenter.

Le déclenchement est verrouillé. OmegaOS ne touche à Linear que lorsqu'on le lui demande par son nom (`/omg-linear`, `fix linear`, un identifiant de ticket comme `KOM-7`, ou un lien `linear.app`). Le simple mot « feedback » ne le déclenche jamais, et il ne mentionnera pas Linear tant qu'on ne l'aura pas fait soi-même.

```
omega_dir=~/.omega          # le protocole est installé dans ~/.omega/skills/linear/
/omg-linear-setup           # une fois par application — installe le widget + les labels + la route
/omg-linear                 # résoudre les tickets ouverts : correction -> audit -> commentaire -> In Review
```

## Limites

Autant les connaître avant de se lancer.

- **Linux d'abord.** Développé sur un VPS sans interface graphique. Pas de Windows. macOS reçoit de vrais correctifs (services launchd, chemin Homebrew) mais est moins éprouvé.
- La TUI suppose un terminal 256 couleurs. Sur un terminal 16 couleurs, ce sera moche.
- Le runtime d'agent par défaut est OpenAI Codex. La CLI `codex` doit donc être connectée. Claude Code, Antigravity, Gemini CLI pour Enterprise/API, OpenRouter/Pi, Hermes, GLM et Kimi sont des alternatives prises en charge.
- **Une seule machine.** Le daemon rmux est local. Il n'y a pas d'orchestration multi-hôtes.
- C'est du 0.1.x. Je m'en sers tous les jours, mais on tombera sur des aspérités que je n'ai pas encore rencontrées.

## À lire ensuite : GUIDE.md

**[GUIDE.md](GUIDE.md)** est le manuel de l'opérateur : le vocabulaire (mission, oracle, worker, goal, plan, Atlas), les trois cockpits, les premières missions, le catalogue de skills, et la façon dont le travail est vérifié. Pour aller plus loin :

- [docs/FEATURES.md](docs/FEATURES.md) — **le catalogue complet des fonctionnalités** (chaque sous-système + comment y accéder).
- [docs/README.md](docs/README.md) — l'index de la documentation.
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — la référence du système complet.
- [docs/MAP.md](docs/MAP.md) — où vit chaque chose sur le disque.
- [docs/THEMES.md](docs/THEMES.md) — la galerie des palettes de la TUI.
- [docs/RESET-RECOVERY.md](docs/RESET-RECOVERY.md) — sauvegarder et reconstruire une machine.
- [CHANGELOG.md](CHANGELOG.md) — ce qui est sorti, version après version.

## Remerciements

OmegaOS s'appuie sur le travail de beaucoup d'autres gens :

La plus grosse dette, c'est [rmux](https://github.com/agentik-os/rmux), le multiplexeur de terminal en Rust sur lequel tourne tout ce qui est ici.

Le reste de la stack Rust :

- [ratatui](https://github.com/ratatui/ratatui) et [crossterm](https://github.com/crossterm-rs/crossterm) — la TUI.
- [tokio](https://github.com/tokio-rs/tokio) — le runtime asynchrone.
- [clap](https://github.com/clap-rs/clap) et `clap_complete` — la CLI et les complétions de shell.
- [serde](https://github.com/serde-rs/serde) avec `serde_json`, `serde_yaml` et `toml` — la config et l'état.
- [anyhow](https://github.com/dtolnay/anyhow) et [thiserror](https://github.com/dtolnay/thiserror) — la gestion d'erreurs.
- `chrono` (horodatages), `dirs` (chemins), `fs2` (les verrous de fichiers consultatifs derrière les revendications de portée), `regex`, `tempfile`, `tracing` avec `tracing-subscriber` (logs), et `reqwest` (le HTTP pour Telegram et les PDF).

[Claude Code](https://www.anthropic.com) d'Anthropic est le runtime d'agent.

## Licence

Sous double licence, [MIT](LICENSE-MIT) ou [Apache-2.0](LICENSE-APACHE), au choix. Convention Rust classique. Prendre celle qu'on préfère.

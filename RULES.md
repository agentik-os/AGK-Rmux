# OmegaOS — Règles projet (canon · à garder & vérifier)

> Fiche de référence **load-bearing** : un agent ou un humain qui reprend le repo lit ça en premier.
> ⚠️ Ce fichier ne **duplique pas** la doctrine. Les instructions universelles agents vivent dans `OMEGA.md` (racine) et les règles opérationnelles dans `~/.omega/rules/`. Ici : ce qui est **spécifique à ce repo** (build, stack Rust, install).
> Reste synchronisée avec `~/.omega/state/oracle-OmegaOS*`. **Ne pas supprimer.** Mettre à jour quand une règle change.

## Identité
- **Projet** : OmegaOS, *agentic terminal operating system*. Plan de contrôle terminal pour piloter une flotte d'agents de code IA en parallèle, où chaque agent obéit au même rulebook typé (7 Lois + 65 Règles compilées dans le binaire ; `omega rules list` affiche l'ensemble courant).
- **Owner** : agentik-os (org GitHub). Mainteneur humain : à préciser.
- **Repo** : `github.com/agentik-os/OmegaOS` · branche `main`
- **Forme** : produit installable (pas une lib). `npx omega-os` ou `git clone … && ./install.sh` → commande `omega`, TUI `ratatui` (7 onglets), bridge Telegram. Version workspace `0.1.6` (npm `omega-os@1.5.4`).
- **Runtime agent par défaut** : Claude Code. Codex / Gemini / GLM / Pi / Hermes installables via `omega install`.

## Stack (R-STACK)
- **Rust workspace**, edition 2021, resolver 2, 3 crates :

| Crate | Rôle |
|---|---|
| `omega-core` | orchestration, registre de règles, doctor, timeline, cleanup, patrol, file-scope locking (32 modules) |
| `omega-cli` | binaire `omega` (40+ commandes), bâti sur `clap` |
| `omega-tui` | session manager, bâti sur `ratatui` + `crossterm` |

- **Infra** : tourne sur [rmux](https://github.com/agentik-os/rmux) (multiplexeur terminal Rust, SDK typé, PTY) — pin `rev = 4455da0`. **Aucune dépendance tmux.**
- **Concurrence** : `tokio` (full), verrous fichiers via `fs2`.
- **PDF reports** : Bun + TypeScript via Next.js + Playwright (`omega pdf`).
- **Bash** : à un seul endroit, le bootstrap d'install.
- **Release profile** : `lto = "fat"`, `codegen-units = 1`, `strip = "symbols"`, panic `unwind` conservé (une task qui panique ne doit pas tuer l'orchestrateur).

## Intouchable (source de vérité)
- `crates/omega-core/src/rules.rs` = SSOT de la doctrine (7 Lois L0 à L6 + 65 Règles nommées typées `RuleKind::{Law, Rule}`). Artefact compilé, pas un YAML oublié. **Ne pas contourner.**
- `OMEGA.md` = instructions universelles chargées par TOUS les agents LLM. **Ne pas éditer ici sans intention.**
- `~/.omega/` = état runtime + credentials + rules éditables. **Hors repo, jamais commité** (gitignored).

## Règles spécifiques (à respecter)
- **R-STACK** : tout le code est Rust et en anglais (même commentaires). Pas de réécriture vers une autre stack.
- **LAW 0 — INSTALL PARITY (non-négociable)** : toute amélioration DOIT garder `install.sh` complet. Une feature n'est PAS done tant qu'un `git clone … && ./install.sh` frais ne la reproduit pas. Nouvel asset (agent/commande/config/template/cron/dir) → ajouter l'étape de copie/setup dans `install.sh` (les changements de binaire suivent automatiquement, install.sh build from source).
- **verify-install passe** : avant de déclarer done, lancer `./scripts/verify-install.sh` — il DOIT passer (binaire-from-source, agents, commandes, configs, crons, **aucun secret tracké**, git clean, remote en sync).
- **Secrets hors repo** : tokens / creds vivent dans `~/.omega/` uniquement (gitignored), JAMAIS dans le repo.
- **Build propre** : commits passent build + lint + typecheck. CI build le workspace avec `-D warnings` + suite de tests comme gates durs (clippy/rustfmt advisory).
- **Pas de `--force`, pas de `--no-verify`.** Vérification par runtime live avant merge.
- **R-ORACLE-LEDGER (contrat de cycle de vie d'un oracle)** : un oracle énumère chaque demande avant d'agir, persiste cette énumération en plan durable (`omega progress <session> --plan "a|b|c"` écrit `oracle-<clé>.progress.json`), garde exactement UNE tâche `doing`, et ne ferme que sur une preuve qu'il a vérifiée lui-même. `omega done <session> done_clean` est REFUSÉ tant qu'un de ses workers tourne encore ; une fermeture propre ne cascade que les workers FINIS, relâche chaque `scope-<session>.json` et ne détruit jamais de travail non commité. Un plan incomplet annoncé `done_clean` est exactement le défaut que cette règle interdit. Contrat complet : `docs/ORACLE-LIFECYCLE-CONTRACT.md`. Règle compilée dans `rules.rs`, exportée vers `~/.omega/rules/`.
- **R-GRAPH-EXEC (couche graphe persistée)** : une mission dont la forme réelle est un GRAPHE (branchement sur une classification, boucle jusqu'à épuisement, étape qui ne doit pas tourner sans personne devant) se DÉCLARE comme un `Graph` d'`omega-core` et se pilote par `graph_executor::advance` ; elle ne se re-dérive pas à la main à chaque run. `graph.rs` = vocabulaire persisté + `validate()`, `graph_executor.rs` = noyau de décision pur (ensemble prêt, retries bornés, propagation d'échec), `graph_risk.rs` = gate human-in-the-loop. Trois défauts porteurs, à ne pas inverser : un nœud NON classifié vaut `Elevated` (jamais `Safe`), la tentative est comptée AVANT le test du plafond (`max_attempts: 3` = exactement trois exécutions), et un re-seed de boucle ne rembourse pas les tentatives (le plafond R-LOOP tient d'une itération à l'autre). En `Unattended`, un nœud irréversible produit un `EscalationRecord` durable, jamais un prompt que personne ne lira ; une approbation non signée est refusée. Aucun des trois modules ne lance de process, n'ouvre de socket, n'écrit sur disque ni ne lit d'horloge : c'est ce qui rend un run rejouable. Contrat complet : `docs/GRAPH-EXECUTION-LAYER.md`. Complète R-GRAPH (la FORME d'un fan-out Workflow), R-LOOP (le plafond) et R-DESTRUCT (le gate). Règle compilée dans `rules.rs`, exportée vers `~/.omega/rules/`.

## Pointeurs (à lire, ne pas recopier ici)
- `OMEGA.md` — instructions universelles agents (les 7 Lois L0–L6, orchestration, behavior, tools).
- `~/.omega/rules/` — règles opérationnelles éditables, IDs nommés (R-PLANE control plane vs Hermes brain, R-AGK four scopes, R-ORCH orchestration workflow-first, R-RUBRIC rubrique avant exécution, R-VERIFY vérification adversariale 2-de-3, R-CITE citation obligatoire, R-BUDGET budget mission, R-PROD prod-verify, R-SCOPE un writer par fichier…).
- `README.md` (+ `.fr/.ru/.zh`) — présentation produit, doctrine, « How a mission runs ».
- `GUIDE.md` — le manuel opérateur complet (vocabulaire, cockpits, skills, vérification).
- `docs/MAP.md` — où vit quoi (source vs binaire installé `~/.local/bin/omega` vs runtime `~/.omega/`).
- `docs/ARCHITECTURE.md`, `docs/GETTING-STARTED.md`, `CLAUDE.md`, `SECURITY.md`, `CHANGELOG.md`.

## Objectif
Donner à n'importe quel VPS/machine Linux une plateforme multi-agents autonome : un humain dispatch une mission (TUI / CLI / Telegram) → AISB Master classe et route → Oracle (1/projet) planifie et délègue → Workers éphémères file-lock-scoped exécutent → quality gate adversarial → ship vérifié. La doctrine typée, injectée dans le prompt de chaque agent à chaque niveau, garantit que personne ne peut faire tomber une Loi pour aller plus vite.

# Duo v2 — design: roles by model strength + a complete bridge

## Objectif

Faire du binome /duo un vrai systeme multi-modele qui exploite chaque LLM la ou il est
le plus fort, et completer le bridge `omega-duo` (tools/duo/bin/omega-duo, Bun, ~1900 l.)
avec les commandes de confort qui manquent. Directive operateur: la reflexion de Codex
est plus profonde — le review de code et la reflexion par Codex deviennent des chemins
de premiere classe, pas seulement "Codex = coder".

## Matrice des roles (le coeur du design)

| Fonction | Modele | Pourquoi |
|---|---|---|
| Plan initial, orchestration | Claude | contexte de session, outils, doctrine OmegaOS |
| Critique de plan (deep) | Codex | raisonnement deliberatif profond, adversarial |
| Reflexion architecture (read-only) | Codex | meme force, prompt de reflexion libre |
| Implementation complexe | Codex | comprehension long-horizon du code |
| Implementation mecanique/bulk | GLM (opt-in explicite) | cout minimal, codegen rapide |
| Deep code review d'un diff | Codex | le point fort signale par l'operateur |
| Verdict final, synthese | Claude | arbitre, tenu par R-VERIFY/L1, jamais delegue |

Regle de review croisee: le modele qui a ecrit le code ne rend jamais le verdict sur
son propre diff. Codex code -> Claude review (+ verify runtime). Claude ou GLM code ->
Codex deep-review (mode review, lecture seule) PUIS verdict Claude. Le verdict reste
TOUJOURS a Claude (arbitre unique), la review Codex est un input de plus (R-VERIFY).

## Profils de boucle (skill-level)

1. `build` (defaut): Claude plan -> Codex critique -> plan v2 -> Codex code
   (checkpoint + --verify auto) -> Claude lit le diff reel + verdict. FIX <= 3 (R-LOOP).
2. `build --coder glm`: GLM code, Codex deep-review du diff, Claude verdict.
   Le triangle complet: 3 modeles, chacun a sa place. GLM jamais choisi automatiquement.
3. `review` (nouveau, sans ecriture): Codex deep-review d'un diff/branche/fichiers
   existants (mode review) -> Claude synthese + verdict. Utilisable seul: "fais relire
   ca par Codex".
4. `reflect` (nouveau, sans ecriture): question d'architecture/design -> Codex mode
   plan avec prompt de reflexion -> Claude synthese. Pas de code du tout.
5. Fallback quota: Claude code seul (single_model, annonce obligatoire). Restauration
   de la validation croisee: des que Codex redevient disponible (`status`/`reset`),
   passer le diff accumule en mode review Codex avant de fermer la mission.

## Bridge — nouvelles capacites

### a) `omega-duo doctor [--json]` (zero appel API)
- codex: binaire present + version, auth locale (`codex login status`), age du flag quota.
- claude: binaire + version. bwrap present ou non (info sandbox).
- glm: cle presente dans providers.toml [glm] (masquee), modele.
- dernier run: fichier log le plus recent + sa premiere ligne.

### b) `omega-duo init <slug> --cwd <projet>`
Scaffold `agentic/duo/<slug>/{plan.md,critique-task.md,code-task.md}` avec templates
pre-remplis (criteres de succes verifiables obligatoires dans plan.md). Refuse
d'ecraser un slug existant.

### c) `omega-duo history [--limit N] [--json]`
Parse ~/.omega/logs/duo/*.log: timestamp, mode, cwd, agent final, ok/fallback.

### d) `--verify "<cmd>"` (mode code)
Apres un run code reussi, le bridge execute la commande dans --cwd (bash -lc, timeout
10 min) et rapporte `verify: {cmd, exit_code, ok, tail}` dans le JSON. verify qui
echoue => ok:false, reason:"verify-failed" (le run n'est pas un succes si le critere
runtime est rouge — L1 mecanise).

### e) Checkpoint git + diffstat (mode code, automatique)
Avant le run: `git rev-parse HEAD` + `git stash create` -> ref `refs/duo/<ts>`.
Apres: `git diff --stat` + untracked. JSON: `checkpoint: {head, ref}`, `diffstat`.
Jamais de restore automatique (R-DESTRUCT): le JSON donne les coordonnees, l'arbitre
decide.

### f) `--agent glm` (table d'agents etendue)
GLM = CLAUDE_BIN + env `ANTHROPIC_BASE_URL=https://api.z.ai/api/anthropic`,
`ANTHROPIC_AUTH_TOKEN=<providers.toml [glm].api_key>`, `--model <glm.model>` —
exactement le chemin de `omega new --agent glm` (agents.rs:605-620). Opt-in only:
jamais dans la chaine de fallback automatique; un echec GLM (quota inclus) ne marque
PAS le flag codex-exhausted et ne bascule sur personne — il est rapporte tel quel.
Preflight worktree identique aux autres agents (meme preuve de lecture).

### g) Selftest etendu (toujours zero API)
doctor --json parseable; init cree/refuse d'ecraser; history lit un log synthetique;
verify vert/rouge; checkpoint present + diffstat; glm force via DUO_GLM_BIN mock,
echec glm sans fallback ni flag quota.

## Contraintes

- Invariants existants intacts: preuve de lecture worktree, quota vs auth vs other,
  mode degrade fail-closed, scrubbing anti-echo, JSON une ligne sur stdout.
- Un seul writer sur le bridge (R-SCOPE): implementation sequentielle.
- SSOT: tools/duo/bin/omega-duo + skills/duo/SKILL.md, copies installees
  ~/.omega/skills/duo/ et ~/.claude/skills/duo/, miroir Agentik-Skills (R-SKILLPUB).
- Critere de succes global: `omega-duo selftest` vert de bout en bout + doctor live OK.

## Annexe A — critique Codex integree (plan v2, 2026-08-10)

Critique deep obtenue via le bridge lui-meme (mode plan, clone prive apres deux
vetos legitimes du garde read-only sur le repo actif). Points integres :
- `agent_ok` separe de `ok` (BLOCKER 2) : verify rouge = FIX input, jamais une
  raison de re-implementer ou de basculer d'agent.
- `verify.timed_out` + garantie anti-echo : la sortie verify n'entre jamais
  dans la classification quota/auth (teste).
- Checkpoint : champ `stash`, diffstat mesure contre le snapshot (delta de
  l'agent, pas delta depuis le dernier commit), ref anti-collision
  (ts+pid+random), retention 20 refs, semantique honnete documentee
  (untracked non captures, listes dans diffstat).
- Historique JSONL versionne (`history.jsonl`, schema 1) ecrit a l'emission ;
  le parse des logs texte est retrograde en fallback legacy.
- init : lstat (symlinks), slug anti-traversal (teste).
- Protocole de restauration de la validation croisee apres fallback : defini
  au niveau skill (status → profil REVIEW avant cloture, sinon mention
  explicite « cross-review pending » dans le rapport).
Ecarte avec justification (L2) : snapshot complet untracked/ignored (le
checkpoint est des coordonnees de revert, pas un backup — cout/complexite
disproportionnes pour le bridge), schema JSON versionne sur `run` (additif
suffirait, aucun consommateur casse aujourd'hui).

## Annexe B — spawn-worker --agent (evalue, reporte)

Verdict d'exploration : PETIT (~60-90 lignes), la resolution existe deja
(`dispatch.rs:1264-1272`, pattern a copier dans `main.rs:6763`). A faire :
flag `agent: Option<String>` sur SpawnWorker (main.rs:436-462), threading
(1037-1056), remplacement de la resolution globale (6763), fix du bug latent
main.rs:6822 (`config.agent_command` au lieu de l'agent resolu — un override
serait ignore sur la branche non-Claude), durcissement du bras GLM
agents.rs:596-630 (`trust_prefix` + `--dangerously-skip-permissions`, sinon un
worker GLM detache pend sur le dialogue trust), restreindre a
`claude|codex|glm` (les trois couverts par le finish-guard). REPORTE : au
moment du chantier, `main.rs` et `dispatch.rs` portaient le WIP non-committe
d'une autre session (R-SCOPE, un writer par fichier). A executer sur arbre
calme.

FAIT (2026-08-10 soir, commit 3b60fa8) : arbre libere, recette executee
integralement — flag --agent allow-liste, resolution par invocation, bug
main.rs:6822 corrige (create_session_with_agent avec l'agent resolu), bras GLM
durci (trust-dir + permission mode honore + rendu inline, 2 tests unitaires),
7 tests agents verts, clippy sans nouvelle plainte, binaire 0.1.9 reinstalle
via install.sh et flag verifie sur le binaire installe.

## Criteres de succes verifiables

1. `bun ~/.omega/skills/duo/bin/omega-duo selftest` -> exit 0, toutes sections PASS.
2. `omega-duo doctor --json` -> JSON valide avec codex/claude/glm/quota/last_run.
3. `omega-duo init demo --cwd /tmp/x && omega-duo init demo --cwd /tmp/x` -> 2e refuse.
4. `omega-duo history --limit 3` -> 3 lignes parsees des vrais logs.
5. Run mock code avec --verify "false" -> ok:false, reason:"verify-failed".

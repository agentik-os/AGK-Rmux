---
name: duo
description: "Binome multi-modele Claude ⇄ Codex/Sol (⇄ GLM opt-in). Chaque modele joue sa force : Claude stratege et arbitre, Codex critique profonde + deep code review + implementation complexe, GLM coder mecanique sur demande explicite. Quatre profils : build (defaut), build coder GLM, review (Codex relit un diff existant), reflect (reflexion architecture sans code). Bascule automatique sur Claude si le quota Codex est epuise. Use when the user types /duo, /omg-duo, or says: build/implement/refactor/fix with the binome, code this with Codex, delegate to Codex/Sol, pair Claude and Codex, deep review this diff with Codex, fais relire par Codex, or FR 'code ça avec Codex', 'délègue à Sol', 'binome Claude Codex', 'fais coder Codex et relis', 'review profonde de ce diff'. NOT for pure questions, reads, or one-liners (Claude handles those directly — no need to burn the Codex quota)."
---

# /duo — le binome multi-modele Claude ⇄ Codex (⇄ GLM)

Le muscle deterministe est le binaire `omega-duo` : Claude ne lance **jamais**
`codex` en direct. Le bridge verifie l'authentification reelle, la lecture du
worktree, le sandbox, les mutations interdites et la detection de quota avant
de rapporter le resultat.

## La matrice des roles — chaque modele la ou il est le plus fort

| Fonction | Modele | Pourquoi |
|---|---|---|
| Plan initial, orchestration, synthese | **Claude** | contexte de session, outils, doctrine |
| Critique de plan (deep, adversariale) | **Codex** | raisonnement deliberatif profond |
| Reflexion architecture (read-only) | **Codex** | meme force, prompt libre |
| Implementation complexe | **Codex** | comprehension long-horizon du code |
| Implementation mecanique / bulk | **GLM** (opt-in explicite) | codegen rapide, cout minimal |
| Deep code review d'un diff | **Codex** | son point fort — a exploiter largement |
| Verdict final | **Claude, toujours** | l'arbitre ne se delegue pas (R-VERIFY) |

**Review croisee** : le modele qui a ecrit le code ne rend jamais le verdict sur
son propre diff. Codex code → Claude relit (+ runtime). Claude ou GLM code →
**Codex deep-review** (mode review) PUIS verdict Claude. La review d'un modele
est un input de plus, jamais le verdict.

## Quand l'utiliser

- `/duo <tâche>` ou « code ça avec Codex / Sol / le binome » → profil **build**.
- « fais relire / review profonde de ce diff par Codex » → profil **review**.
- « réfléchis avec Codex à <architecture/design> » → profil **reflect**.
- « fais coder GLM » (explicite uniquement) → **build --coder glm**.

**Ne PAS** declencher pour une question, une lecture, un one-liner : Claude
repond seul (routage explicite, R-BUDGET). GLM n'est JAMAIS choisi
automatiquement (doctrine GLM opt-in only).

## Le bridge

Resolu dans l'ordre : `omega-duo` sur le PATH, sinon
`<OMEGA_DIR>/skills/duo/bin/omega-duo`, sinon
`~/Station/SideBusiness/OmegaOS/tools/duo/bin/omega-duo`. Le repertoire
canonique est `$OMEGA_DIR` s'il est defini, sinon `~/OmegaOS/System` s'il
existe, sinon `~/.omega`.

```
omega-duo run --task <file.md> --cwd <projet> --mode <plan|code|review>
              [--agent codex|claude|glm] [--verify "<commande shell>"]
  → une ligne JSON : { agent, ok, agent_ok, output, fell_back, reason,
                       exit_code, log, sandbox_degraded,
                       capabilities: { shell_exec, worktree_read },
                       verify, checkpoint, diffstat }

omega-duo doctor [--json]        # sante locale : bins, auth codex, quota, glm, dernier run
omega-duo init <slug> --cwd <p>  # scaffold agentic/duo/<slug>/ (plan + critique + code)
omega-duo history [--limit N] [--json]   # historique JSONL versionne des runs
omega-duo status | reset         # flag quota Codex
```

- `mode plan` / `mode review` → essai zero toujours en mode natif lecture seule
  (`Codex --sandbox read-only` ou `Claude --permission-mode plan`), preuve de
  lecture du depot obligatoire.
- `mode code` → l'agent edite les fichiers (full-auto). Le bridge pose alors un
  **checkpoint automatique** avant le run (`checkpoint.head` + `checkpoint.stash`
  + une ref `refs/duo/checkpoint-…`, 20 conservees) et rapporte le
  **`diffstat`** du delta de l'agent (mesure contre le snapshot, AVANT verify).
  Le checkpoint capture l'etat SUIVI uniquement — les untracked sont listes
  dans le diffstat. Jamais de restore automatique (R-DESTRUCT) : le JSON donne
  les coordonnees, TOI tu decides.
- `--verify "<cmd>"` → apres un run code reussi, le bridge execute la commande
  de succes du plan (timeout 10 min) et l'attache en `verify`. **`verify` rouge
  ⇒ `ok:false, reason:"verify-failed"` mais `agent_ok:true`** : l'implementation
  existe, son critere est rouge — c'est un input de boucle FIX, jamais une
  raison de re-implementer de zero ni de basculer d'agent. La sortie de verify
  n'entre jamais dans la classification quota/auth.
- `fell_back: true` → quota Codex epuise, **Claude** a fait le travail. Tu DOIS
  le dire. Resultat `single_model`, jamais une validation independante.
- `sandbox_degraded: true` → l'essai natif n'a pas pu prouver la lecture ; le
  bridge a utilise l'acces externe garde et verifie le worktree avant/apres.
  Tu DOIS le signaler.
- `capabilities.shell_exec` et `worktree_read` doivent etre `true`, sinon ou si
  `ok:false` (hors `verify-failed`), **STOP** : ne lis pas `output` comme une
  critique ou revue valide.
- `reason: "codex-unauthenticated"` → session Codex inutilisable. **STOP**,
  demande une reparation. Jamais de fallback, jamais de flag quota.
- `--agent glm` → GLM (= Claude Code redirige vers z.ai, cle providers.toml
  [glm]). **Opt-in only** : jamais dans la chaine de fallback, un echec GLM
  (`glm-quota`, `glm-unauthenticated`, `glm-task-failure`, `glm-unconfigured`)
  ne marque jamais le flag Codex et ne bascule sur personne.

L'appelant fournit un **fichier**, le bridge transmet son contenu sur stdin,
jamais en argv. `omega-duo init <slug> --cwd <projet>` scaffolde le dossier
`agentic/duo/<slug>/` avec les trois templates.

**Repo bruyant = preuve impossible.** La preuve read-only echoue legitimement
(`readonly-violation`) si une AUTRE session ecrit dans le repo pendant le run —
y compris via le `.git` partage d'un worktree lie. Pour une critique/review sur
un repo actif : clone prive jetable (`git clone <repo> /tmp/…`), et pointe
`--cwd` dessus. Un `git worktree add` ne suffit PAS (git-dir commun).

L'etat d'un enfant en cours reste observation-only. Pour annuler un login
Codex : `omega codex-login-abort --pid <n>`.

## Profil BUILD (defaut) — la boucle, dans l'ordre

Cree une TODO par etape. `omega-duo init <slug> --cwd <projet>` d'abord.

### 1. Plan (Claude)
Remplis `plan.md` : objectif, **criteres de succes verifiables** (commande de
test/build — elle deviendra le `--verify`), fichiers touches, approche.

### 2. Critique deep (Codex, lecture seule)
Colle le plan dans `critique-task.md` (garde l'interdiction de build du
template : une review read-only qui lance cargo/npm/tests ecrit des artefacts
et se fait rejeter en `readonly-violation` par le garde — lecture de fichiers
UNIQUEMENT), puis :
```
omega-duo run --task agentic/duo/<slug>/critique-task.md --cwd <projet> --mode plan
```
Verifie `ok`, `sandbox_degraded`, les deux `capabilities`. Une critique sans
lecture prouvee du worktree n'est pas une critique.

### 3. Plan v2 (Claude)
Integre la critique (ou justifie en 1 ligne pourquoi tu l'ecartes — L2).

### 4. Implementation (Codex par defaut ; GLM si demande explicitement)
Plan v2 dans `code-task.md`, puis :
```
omega-duo run --task agentic/duo/<slug>/code-task.md --cwd <projet> --mode code \
  --verify "<la commande de succes du plan>"
```
(`--agent glm` uniquement si l'operateur a demande GLM.)

### 5. Review croisee
- Lis le **diff reel** (`git -C <projet> diff` + le `diffstat` du bridge),
  jamais le recit de l'agent (R-VERIFY).
- `verify` a deja mecanise le critere runtime (L1) ; relance-le toi-meme au
  moindre doute.
- **Si le coder etait GLM ou Claude (fallback)** : passe le diff en deep-review
  Codex avant ton verdict :
  ```
  omega-duo run --task agentic/duo/<slug>/review-task.md --cwd <projet> --mode review
  ```
  (review-task.md = « Deep-review this diff: correctness, edge cases, hidden
  regressions, style drift. Do not write code. » + le diff.)

### 6. Verdict (Claude) — le verdict est a TOI
- **PASS** → resume ce qui a change, cite le diff et le verify vert.
- **FIX** → `fix-task.md` (probleme precis + fichier:ligne + le `verify.tail`
  rouge) et relance l'etape 4. `agent_ok:true + verify-failed` = FIX, pas
  re-implementation.

### Plafond (R-LOOP)
**Maximum 3 tours de FIX sur le meme echec.** Au 3e, STOP : escalade a
l'operateur avec l'etat exact (dernier diff, `verify.tail`, log du bridge).

## Profil REVIEW — Codex comme deep reviewer, sans le faire coder

Pour un diff/une branche/un fichier EXISTANT (code de Claude, d'un humain,
d'un worker) : ecris `review-task.md` (le diff ou les chemins + « challenge
correctness, edge cases, security, regressions — do NOT write code »), lance
`--mode review`, puis synthese et verdict par Claude. C'est le chemin a
privilegier des que la profondeur compte : la reflexion de Codex est son
point fort.

## Profil REFLECT — reflexion architecture, zero code

Question de design/architecture dans `reflect-task.md` (contexte + la question
+ « answer in structured text, no code »), `--mode plan`, synthese Claude.
Ideal avant un blueprint, un refactor lourd, un choix de stack.

## Fallback quota (gere par le bridge)

Rien a parser. Si `fell_back: true` : dis-le simplement :
> ⚠️ Quota Codex epuise — j'ai code moi-meme (Claude) le reste.

Codex est marque indisponible pour la session (`omega-duo status`), les etapes
suivantes basculent direct. `omega-duo reset` rearme. **Restauration de la
validation croisee** : un resultat fallback est `single_model` ; avant de
clore la mission, verifie `omega-duo status` — si Codex est revenu, passe le
diff accumule en profil REVIEW ; sinon, note explicitement dans le rapport
« cross-review Codex en attente » + la commande exacte a rejouer. Ne jamais
presenter un fallback comme une validation a deux modeles.

Une erreur `codex-unauthenticated` est differente : pas de fallback, pas de
flag, pas de resultat vert. Repare l'authentification d'abord.

## Duo parallele (R-SCOPE)

3+ taches **file-disjointes** → un run `omega-duo` par tache, chacun dans son
**clone/worktree prive** (jamais deux writers sur un fichier, et jamais deux
runs sur le meme worktree — le garde de l'un verrait les ecritures de
l'autre). Merge ensuite via le flux git normal. Les runs read-only (critique,
review) peuvent partager un meme clone prive s'ils sont sequentiels.

## Perimetre

- **Gemini** n'est pas cable (redondant avec Codex). La table d'agents du
  bridge rend l'ajout trivial si un jour necessaire.
- Le mode natif lecture seule est toujours l'essai zero pour plan/review. Un
  `bwrap` absent n'est PAS une preuve de sandbox casse et ne declenche pas le
  mode degrade. Seul un refus de lecture demontre le fait.
- Apres echec demontre du probe natif : `sandbox_degraded: true`, garde
  worktree complete (index, untracked, metadonnees git incluses), echec ferme
  sur toute mutation observee ou si aucun watcher live n'est etabli.
- Un echec reseau/5xx/malforme n'est jamais une preuve de refus sandbox et ne
  declenche jamais le bypass.
- Les sorties `doctor` / `init` / `history` ont leurs propres schemas JSON —
  seul `run` emet le contrat BridgeResult complet.

## Rappels

- Reponds en francais (R-STYLE) ; code, commits, identifiants en anglais.
- Un livrable (lien/fichier) part aussi sur Telegram (R-TGDELIVER).
- Termine par : `--- **Résumé:** …` (qui a code quoi, qui a reviewé quoi,
  verdict, quota).

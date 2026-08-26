You are reviewing a design plan for upgrading the `omega-duo` bridge in THIS repository.
The bridge source is `tools/duo/bin/omega-duo` (Bun/TypeScript, ~1900 lines) and the
skill doctrine is `skills/duo/SKILL.md`. Read both, plus the plan below.

Challenge this plan as a deep adversarial reviewer:
- Blind spots: what breaks in the existing invariants (worktree-read proof, quota vs
  auth classification, degraded-sandbox guard, anti-echo scrubbing, single-line JSON
  contract) if we add verify/checkpoint/glm/doctor/init/history as specified?
- Design flaws: is "verify failure => ok:false, reason:verify-failed" the right
  contract, or does it conflate agent success with task success in a way callers
  will misread? Is the git checkpoint approach (rev-parse HEAD + stash create ->
  refs/duo/<ts>) sound for dirty worktrees, empty repos, worktree checkouts?
- The role matrix and cross-review loop: any incoherence or missed opportunity given
  the models' actual strengths (deep reasoning review on Codex, arbitration on
  Claude, cheap codegen on GLM)?
- Better approaches, in concrete terms, for anything you would do differently.

Do NOT write any code. Do NOT modify any file. Answer in structured text, ranked by
severity: BLOCKER / MAJOR / MINOR, each with the exact plan section it targets.

--- PLAN UNDER REVIEW ---

# Duo v2 — design: roles by model strength + a complete bridge

## Objectif

Faire du binome /duo un vrai systeme multi-modele qui exploite chaque LLM la ou il est
le plus fort, et completer le bridge omega-duo avec les commandes de confort qui
manquent. Directive operateur: la reflexion de Codex est plus profonde — le review de
code et la reflexion par Codex deviennent des chemins de premiere classe.

## Matrice des roles

| Fonction | Modele |
|---|---|
| Plan initial, orchestration | Claude |
| Critique de plan (deep) | Codex |
| Reflexion architecture (read-only) | Codex |
| Implementation complexe | Codex |
| Implementation mecanique/bulk | GLM (opt-in explicite) |
| Deep code review d'un diff | Codex |
| Verdict final, synthese | Claude |

Regle de review croisee: le modele qui a ecrit le code ne rend jamais le verdict sur
son propre diff. Codex code -> Claude review (+ verify runtime). Claude ou GLM code ->
Codex deep-review (mode review, lecture seule) PUIS verdict Claude. Le verdict reste
TOUJOURS a Claude.

## Profils de boucle (skill-level)

1. build (defaut): Claude plan -> Codex critique -> plan v2 -> Codex code
   (checkpoint + --verify auto) -> Claude lit le diff reel + verdict. FIX <= 3.
2. build --coder glm: GLM code, Codex deep-review du diff, Claude verdict.
3. review (nouveau, sans ecriture): Codex deep-review d'un diff/branche existant
   (mode review) -> Claude synthese + verdict.
4. reflect (nouveau, sans ecriture): question d'architecture -> Codex mode plan
   avec prompt de reflexion -> Claude synthese.
5. Fallback quota: Claude code seul (single_model, annonce). Restauration de la
   validation croisee: des que Codex redevient disponible, passer le diff accumule
   en mode review Codex avant de fermer la mission.

## Bridge — nouvelles capacites

a) omega-duo doctor [--json], zero appel API: codex bin+version+auth locale
   (codex login status), age du flag quota, claude bin, bwrap, glm key masquee,
   dernier log.
b) omega-duo init <slug> --cwd <projet>: scaffold agentic/duo/<slug>/ avec
   templates (plan.md exige des criteres de succes verifiables). Refuse d'ecraser.
c) omega-duo history [--limit N] [--json]: parse ~/.omega/logs/duo/*.log.
d) --verify "<cmd>" (mode code): apres run reussi, bash -lc dans --cwd, timeout
   10 min, JSON verify:{cmd,exit_code,ok,tail}; echec => ok:false,
   reason:"verify-failed".
e) Checkpoint git automatique (mode code): avant = git rev-parse HEAD + git stash
   create -> ref refs/duo/<ts>; apres = git diff --stat + untracked. JSON:
   checkpoint:{head,ref}, diffstat. Jamais de restore automatique.
f) --agent glm: CLAUDE_BIN + ANTHROPIC_BASE_URL=https://api.z.ai/api/anthropic +
   ANTHROPIC_AUTH_TOKEN=<providers.toml [glm].api_key> + --model <glm.model>.
   Opt-in only, jamais en fallback automatique; echec GLM ne marque pas le flag
   codex-exhausted et ne bascule sur personne. Meme preflight worktree.
g) Selftest etendu, zero API, couvrant a-f.

## Contraintes

Invariants existants intacts (preuve lecture worktree, quota vs auth, degrade
fail-closed, scrubbing anti-echo, JSON une ligne). Verdict humain-side inchange.

> **Historical concept note (pre-0.1)** — superseded by [README.md](../../README.md) and [GUIDE.md](../../GUIDE.md).
> Kept for context: it still describes the original "Three Laws" doctrine; the live doctrine is 7 Laws (L0–L6) + named Rules — `omega rules list`.

# OmegaOS — Concept & Principes

> Le document qui explique *pourquoi* le système est construit ainsi.
> Pas la liste des fichiers (ça c'est `MAP.md`) — la **thèse** derrière.

---

## 1. Le problème qu'OmegaOS résout

Tu as plusieurs LLM CLIs (Claude Code, Gemini, Codex, GLM, Qwen, Aider…). Chacun :
- lit un **fichier de contexte différent** (`CLAUDE.md`, `GEMINI.md`, `AGENTS.md`…)
- a sa **propre auth** (OAuth, clé API, abonnement)
- tourne dans son **propre process** sans mémoire de ce que font les autres
- n'a **aucune garantie** que son travail est réellement fini (il *dit* "c'est fait")

Sans système, tu te retrouves à :
1. Copier-coller les mêmes règles dans 6 fichiers qui divergent
2. Lancer chaque agent à la main, sans coordination
3. Croire l'agent sur parole quand il dit "terminé"

**OmegaOS = la couche d'orchestration qui transforme N LLMs isolés en un seul système cohérent, gouverné, et vérifiable.**

---

## 2. Les 3 principes fondateurs (Les Trois Lois)

Tout le reste découle de ça. Ce sont les lois que **chaque** agent, à **chaque** niveau, hérite.

| Loi | Énoncé | Pourquoi |
|---|---|---|
| **Loi 1** | Le code ment. Seul le runtime dit la vérité. | Un LLM "pense" que son code marche. La preuve est dans les logs/tests/screenshots, pas dans son intention. |
| **Loi 2** | Chercheur, pas courtisan. | Un LLM qui dit toujours "oui" produit de la merde polie. On veut qu'il *challenge* la prémisse avant de coder. |
| **Loi 3** | Décide et avance. | Un agent dispatché qui s'arrête pour demander "quelle option ?" bloque tout le système. Il décide, exécute, rapporte après. |

Ces 3 lois ne sont pas décoratives : elles sont **embarquées dans le fichier canonique** (§4) et **vérifiées** (un audit scanne les prompts d'agents pour confirmer qu'ils les référencent).

---

## 3. Principe #1 — Un cerveau, plusieurs mains (multi-agent)

### Pourquoi pas un seul gros agent ?

Un seul LLM qui fait tout = context window saturé, pas de spécialisation, pas de parallélisme, pas de vérification indépendante. La solution : **une hiérarchie de rôles spécialisés**, chacun avec un context frais et une responsabilité unique.

### Les 5 niveaux + le toit de gouvernance

```
L0   GOUVERNANCE        — Paperclip : registre des 14 agents, lignes de reporting fixes
L1   HUMAIN             — toi (Telegram, CLI `omega`, web)
L2   HERMÈS             — méta-compagnon, budget isolé (clé API propre)
L3   AISB MASTER        — le cerveau : classe, route, délègue aux 14 agents
L4   ORACLE             — 1 par projet : planifie, dispatche les workers
L5   WORKERS            — éphémères, parallèles, scope-lockés, vérifiés
```

**Pourquoi cette séparation précise ?**

- **L2 Hermès isolé** : il a sa propre clé API (budget séparé). Si tu cames une boucle infinie, ça ne brûle pas ton abonnement Max. C'est le *compagnon* qui te parle, pas un worker.
- **L3 AISB ≠ L4 Oracle** : AISB est généraliste (il connaît tous les projets). Oracle est spécialiste d'UN projet (il connaît son code, son histoire, ses décisions). Mélanger les deux = un agent qui sait tout vaguement et rien précisément.
- **L5 Workers éphémères** : ils naissent pour UNE tâche, avec un brief auto-contenu, et meurent en écrivant `.done.json`. Pas d'état partagé = pas de corruption croisée. Parallélisme safe.

### Les 15 agents (Hermès + 14 AISB Matrix)

| Tier | Agent | Mission unique |
|---|---|---|
| Lead | **AISB** | Dispatch vers les managers |
| Manager | **Oracle** | Classe l'intent → route → dispatche |
| Worker | **Morpheus** | Exécute le code |
| Worker | **Construct** | Build le scaffolding (skills, audits) |
| Worker | **Architect** | Analyse de design système |
| Worker | **Keymaker** | Construit la rubrique + le DAG du plan |
| Worker | **Niobe** | Recherche + sources citées |
| Worker | **Smith** | Extrait les leçons (apprentissage) |
| Worker | **Merovingian** | Persiste les patterns (mémoire) |
| Worker | **Neo** | Surveille (watchdog, détection de stall) |
| Worker | **Zion** | Dashboards depuis les outcomes |
| Worker | **Link** | Pont webhook → Telegram |
| Checker | **Seraph** | Audite le travail des workers — **verdict par défaut : FAIL** |
| Watcher | **Pythia** | Surveille les docs Anthropic (read-only) |

**Le pattern LMC (Lead-Manager-Checker)** : le Lead délègue, le Manager exécute via les workers, le Checker valide **indépendamment** (Seraph n'a pas écrit le code, donc il ne croit pas sur parole). C'est ce qui rend la Loi 1 opérationnelle.

---

## 4. Principe #2 — Une vérité, plusieurs dialectes (le dossier maître)

### Le problème central

Chaque LLM CLI lit un nom de fichier différent. Si tu maintiens 6 fichiers à la main, ils **divergent** en une semaine.

### La solution : un fichier canonique → mirroré

```
~/.omega/OMEGA.md   ⭐ LA SOURCE DE VÉRITÉ UNIQUE
        │
        │  (mirroré à l'install + à chaque `omega sync`)
        ▼
chat-contexts/<session>/
        ├── CLAUDE.md              ← Claude Code lit ça
        ├── GEMINI.md              ← Gemini CLI lit ça
        ├── AGENTS.md              ← Codex lit ça
        ├── QWEN.md                ← Qwen lit ça
        ├── .opencode/CONTEXT.md   ← OpenCode + OpenRouter + DeepSeek (partagé)
        ├── .continue/CONTEXT.md   ← Continue.dev lit ça
        ├── CONVENTIONS.md         ← Aider lit ça
        ├── HERMES.md              ← Hermès lit ça
        ├── GLM.md / OLLAMA.md / LM_STUDIO.md
        └── …
```

**Tu édites `OMEGA.md`. Tous les LLMs voient le même contenu, chacun via son nom natif. Zéro dérive.**

C'est ÇA l'idée maîtresse : **un cerveau commun, distribué dans le dialecte de chaque outil.** Que tu lances `claude`, `gemini`, ou `aider` dans le dossier d'une session, ils boot tous avec les mêmes Trois Lois, les mêmes règles, la même connaissance d'OmegaOS.

### Pourquoi un dossier maître `~/.omega/` séparé du repo ?

| Couche | Où | Rôle |
|---|---|---|
| **Code source** | `~/VibeCoding/work/OmegaOS/` (le repo git) | Ce que tu édites + push |
| **Binaire compilé** | `~/.local/bin/omega` | L'exécutable Rust (8.9 MB) |
| **Données runtime** | `~/.omega/` | Ce que **tous les LLMs lisent** — credentials, règles, agents, état |

Séparer le code (versionné, public) de la config runtime (locale, secrète, mutable) = principe de base d'un OS. Le repo est immuable ; `~/.omega/` est ton état vivant.

---

## 5. Principe #3 — La complétion est dérivée, jamais déclarée

### Le problème

Un LLM dit "j'ai fini" alors que le code ne compile même pas. C'est le mensonge le plus courant.

### La solution : `.done.json` + audit gate

Aucun agent ne peut **déclarer** son travail fini. Le moteur **dérive** la complétion depuis un signal structuré :

```
~/.omega/state/sessions/<task_id>/.done.json
{
  "status": "done_clean | pending | failed",
  "consensus_score": 2,        // ≥2/3 graders d'accord (R-21)
  "adversarial_pass": "passed", // ≥12 challenges Popper (R-30)
  "regressions": [],            // aucune régression (R-22)
  "audit_score": 87            // ≥85/100 (gate)
}
```

Le moteur lit `.done.json`. Si `status != done_clean` OU si un audit a échoué → la mission n'est **pas** finie, peu importe ce que l'agent prétend.

**C'est la Loi 1 transformée en contrat machine** : pas "l'agent dit que c'est bon", mais "le runtime a prouvé que c'est bon".

---

## 6. rmux — pourquoi un multiplexeur (le substrat)

Le nouveau projet remplace `tmux` (shell-out fragile) par **rmux** (SDK Rust typé).

**Pourquoi ?**
- Les agents tournent dans des **sessions persistantes détachables** : tu lances une mission, tu fermes ton laptop, elle continue sur le VPS.
- rmux a un **SDK** → le code Rust pilote les sessions **programmatiquement** (pas `subprocess.run(["tmux", ...])` qui peut hang ou deadlock — le bug exact qu'on a chassé pendant des heures dans l'ancienne version Python).
- **Inspectable** : `omega` peut lire l'état de chaque session (snapshot structuré) sans parser du texte tmux fragile.

```
rmux daemon
  ├── session: AISB-master      (toujours allumée, L3)
  ├── session: <Project>-oracle (L4, 1 par projet)
  └── session: <Project>-worker-N-<task> (L5, éphémères)
```

Chaque niveau de la hiérarchie multi-agent = une session rmux. L'orchestration **est** la gestion de ces sessions.

---

## 7. Le dossier maître — ancien (Python) → nouveau (Rust)

Ton ancienne archi Python `~/Omega/Agentik_*/` se mappe sur le nouveau `~/.omega/` Rust comme suit :

| Ancien (Python `~/Omega/`) | Nouveau (Rust `~/.omega/`) | Rôle inchangé |
|---|---|---|
| `Agentik_SSOT/personas/OMEGAOS-CONTEXT.md` | `~/.omega/OMEGA.md` | Le fichier canonique ⭐ |
| `Agentik_SSOT/agents/aisb/*.md` | `~/.omega/agents/` (+ `agents/aisb/`) | 14 prompts d'agents |
| `Agentik_SSOT/rules/*.md` | `~/.omega/rules/` | Règles de gouvernance |
| `Agentik_SSOT/skills/*/SKILL.md` | `~/.omega/skills/audits/` | 18 audits + orchestrators |
| `Agentik_SSOT/llm-providers/providers-catalog.yaml` | `~/.omega/providers.toml` | Catalogue providers |
| `Agentik_SSOT/providers/router.yaml` | (généré, `omega-core/router.rs`) | Routing role→provider |
| `Agentik_Extra/etc/secrets/` (age vault) | `~/.omega/credentials/` | Tous les credentials LLM |
| `Agentik_Extra/var/active-llm-provider` | `~/.omega/config.toml` | Provider actif |
| `Agentik_Runtime/eventlog/omega.db` | `~/.omega/state/` (+ `logs/`, `audit/`) | État runtime, sessions, `.done.json` |
| `Agentik_Engine/omega_engine/*.py` | `crates/omega-core/src/*.rs` (32 modules) | Le moteur |
| `Agentik_Tools/bin/omega` (symlink venv) | `~/.local/bin/omega` (binaire Rust compilé) | La CLI |
| `Agentik_Coding/projects/<slug>/` | (inchangé conceptuellement) | Isolation per-projet |
| `Agentik_Coding/chat-contexts/<label>/` | (inchangé) | Dossiers persona par session |

**Différence clé de philosophie** : l'ancien Python avait un "8-block rack" très formel (`Agentik_*`). Le nouveau Rust suit la convention Unix standard : **code dans le repo, binaire dans `~/.local/bin`, données dans `~/.omega/`**. Plus simple, plus idiomatique, séparation source/runtime nette.

---

## 8. Le cycle de vie d'une mission (end-to-end)

Pour rendre tout ça concret, voici ce qui se passe quand tu tapes une demande :

```
1. INTENT      Tu écris "fix le bug d'auth" (Telegram ou CLI)
                  ↓ omega-core/intent.rs classe : bug-fix, MEDIUM
2. ROUTE       AISB (L3) décide : 1 oracle suffit
                  ↓ omega-core/router.rs
3. PLAN        Oracle (L4) → Keymaker construit la rubrique + DAG
                  ↓ rubric.rs écrit les critères de succès AVANT (R-19)
4. DISPATCH    Oracle spawne un Worker rmux (L5), scope-locké sur auth.rs
                  ↓ dispatch.rs + scope.rs (file-lock anti-conflit)
5. EXECUTE     Morpheus corrige, observe le runtime (Loi 1)
                  ↓ écrit .done.json status=pending
6. AUDIT       Seraph (Checker) audite INDÉPENDAMMENT
                  ↓ gate.rs : consensus ≥2/3, Popper ≥12 challenges
7. VERIFY      Le gate calcule : audit_score ≥ 85 ? regressions = 0 ?
                  ↓ si oui → .done.json status=done_clean
8. SHIP        (si demandé) build → test → deploy → vérifie 200 (R-14)
9. REPORT      Link (L5) → Telegram : "✓ done, voici la preuve"
10. LEARN      Smith extrait la leçon → Merovingian la persiste
```

À aucun moment un agent ne "déclare" la victoire. Le **moteur** la dérive du gate.

---

## 9. La thèse — pourquoi tout ça

Si je devais résumer en une phrase :

> **OmegaOS transforme une collection de LLM CLIs isolés et faillibles en un système d'exploitation agentique gouverné — un cerveau commun (le fichier canonique), des mains spécialisées (les 14 agents), et une vérité machine (la complétion vérifiée) — pour qu'un humain puisse déléguer du travail réel à des agents et avoir la *preuve* que c'est fait, pas juste la promesse.**

Les 3 piliers, encore :

1. **Cohérence** — un fichier canonique mirroré → tous les LLMs pensent pareil.
2. **Spécialisation + parallélisme** — la hiérarchie L0-L5 → chaque agent fait une chose bien, en parallèle, sans se marcher dessus.
3. **Vérifiabilité** — `.done.json` + audit gate → la complétion est prouvée, jamais crue.

rmux est le substrat (sessions persistantes pilotables par code). Rust est le langage (type-safe, rapide, pas de subprocess fragile). `~/.omega/` est l'état vivant. Le repo est la source immuable.

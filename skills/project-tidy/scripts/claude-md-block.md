<!-- AGENTIK-LAYOUT:START — convention de placement des fichiers (gérée par /project-tidy) -->
## 📁 Où écrire les fichiers (convention — À RESPECTER)

Pour garder la codebase propre, chaque fichier créé va à un endroit précis :

- **Code** → dossiers de code existants (`app/`, `components/`, `lib/`, `convex/`, `services/`…). Jamais de code à la racine.
- **Doc humaine** (specs, guides, knowledge, notes de conception) → **`docs/`**.
- **Sorties/tracking d'agents VISIBLES** (rapports d'audit, tests jetables, logs, captures, plans exportés, fourre-tout) → **`agentic/`** :
  `agentic/audits/` · `agentic/reports/` · `agentic/tests/` · `agentic/specs/` · `agentic/archive/`.
- **Système OmegaOS** (`.planner/`, `.audit/`, `.oracles/`) → **restent à la racine** (gérés par OmegaOS, ne pas déplacer).
- **Canon** (`README.md`, `CLAUDE.md`, `RULES.md`, `PROGRESS.md`, `vision/`, `PRD…`, `*feature*`, `*step*`) → restent à la racine, ne pas déplacer.

**Interdit** : créer un nouveau dossier de tracking visible à la racine (`audits/`, `report.md`, `to order/`, `deep-test-*.mjs`…). → tout ça va dans `agentic/`.

**Règle d'or** : si ce n'est ni du code, ni du canon, ni un dotfolder système → ça va dans `docs/` (humain) ou `agentic/` (agent).
<!-- AGENTIK-LAYOUT:END -->

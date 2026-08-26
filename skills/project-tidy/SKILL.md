---
name: project-tidy
description: >
  Range un projet pollué par le sprawl des agents IA — regroupe la doc humaine dans docs/ et le
  tracking/junk agent visible dans agentic/, réécrit CLAUDE.md/AGENTS.md pour définir où va chaque
  fichier (les futurs agents s'y conforment), propose la suppression du périmé et signale les
  incohérences doc↔app. Ne touche JAMAIS au code, aux dotfolders système (.git/.planner/.audit/
  .oracles…), ni au canon (vision/PRD/feature/step).
argument-hint: "[project_dir] [--apply]   (défaut: dry-run)"
allowed-tools: ["Read", "Write", "Edit", "Bash", "Glob", "Grep", "AskUserQuestion", "Task"]
domain: maintenance
read_only: false
triggers: ["project-tidy", "tidy", "ranger projet", "ranger les fichiers", "organize project", "clean project structure", "docs agentic", "rangement projet"]
---

# project-tidy — organiseur de projet

Combat le sprawl des agents IA (docs et dossiers de tracking éparpillés) sans rien casser.

## Convention cible (par projet)
```
<projet>/
  README.md  CLAUDE.md  AGENTS.md→CLAUDE.md  RULES.md  PROGRESS.md   ← canon racine
  app/ src/ components/ lib/ convex/ hooks/ services/ …              ← CODE : INTOUCHABLE
  .git .github .vercel .claude .vscode .cursor .convex               ← outillage : RESTE racine
  .planner .audit .oracles  (+ autres dotfolders OmegaOS)            ← système agents : RESTE racine (géré par OmegaOS)
  docs/        ← TOUTE la doc humaine visible (specs, knowledge, guides, rapports à garder)
  agentic/     ← TOUT le tracking/junk agent VISIBLE, regroupé :
      audits/     (dossier 'audits' visible, rapports d'audit)
      reports/    (report.md, *_REPORT, RAPPORT-*.md/pdf)
      tests/      (deep-test-*.mjs, probe-*.mjs, e2e jetable, playwright-report, *.log, logs/)
      specs/      (prd-*.json générés, status.json, captures *.jpeg/png de travail)
      archive/    (fourre-tout type « to order » en attente de tri)
```

## Règles de sécurité (inviolables)
- **JAMAIS déplacer** : le code, les fichiers de config (package.json, tsconfig, *.config.*, Cargo.toml, lockfiles, Dockerfile…), les **dotfolders/dotfiles** (ils restent à la racine — `.planner/.audit/.oracles` sont gérés par OmegaOS), et le canon (`README* CLAUDE.md AGENTS.md RULES.md PROGRESS.md LICENSE CHANGELOG SECURITY.md` + `vision/ PRD* *feature* *step*`).
- **Ne traiter QUE les entrées VISIBLES** (non-dot) de la racine. Ignorer entièrement tout ce qui commence par `.`.
- **Déplacements traçables** : `git mv` pour les fichiers suivis (réversible), `mv` sinon. Écrire un **manifest** de chaque déplacement.
- **Mettre à jour les liens** : après un déplacement de `.md`, corriger les liens relatifs cassés.
- **Preview d'abord** : `scripts/tidy-scan.sh <projet>` produit le PLAN (sans rien bouger). N'appliquer qu'après lecture.
- En cas de doute sur une entrée → catégorie `??REVIEW`, demander, ne pas déplacer.

## Phases
1. **RANGER + DÉFINIR** : `tidy-scan.sh <projet>` → plan. Après accord : créer `docs/` + `agentic/{audits,reports,tests,specs,archive}`, déplacer le visible (git mv/mv + manifest), corriger les liens, puis **réécrire `CLAUDE.md`** en injectant la section « 📁 Où écrire les fichiers » (voir `scripts/claude-md-block.md`) et créer le symlink `AGENTS.md→CLAUDE.md`. But : les futurs agents écrivent au bon endroit.
2. **PROPOSER SUPPRESSIONS** : `tidy-scan.sh <projet> --stale 7` liste les fichiers/dossiers de `agentic/` (et audits) non modifiés depuis >7 jours → présenter pour validation, jamais auto-supprimer. Si un dossier ne contient que des fichiers périmés → proposer le dossier entier.
3. **SIGNALER INCOHÉRENCES** : comparer docs vs réalité (stack/route/feature disparue, plan obsolète, RULES vs code). Lister les écarts ; ne rien réécrire sans accord.

## Déploiement multi-projets
Piloter d'abord sur 1 projet (valider la catégorisation), puis fan-out via sous-agents (1 par projet) appliquant ce SKILL.md. Toujours `tidy-scan` (plan) avant d'appliquer.

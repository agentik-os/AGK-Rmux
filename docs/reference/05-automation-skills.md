# Skills Automation & Workflows

> Agents autonomes et workflows de développement automatisés.

---

## Vue d'ensemble

| Commande | Description |
|----------|-------------|
| `/team` | Agent Teams with tmux split-pane coordination |
| `/sentinel-loop` | Tests continus avec checkpoints |
| `/bmad` | BMAD Method workflows |
| `brainstorming` | Design collaboratif avant implémentation |

---

## 1. /sentinel-loop - Continuous Testing

**Source:** `~/.claude/commands/sentinel-loop.md`

### Usage

```bash
/sentinel <project>
/sentinel kommu
```

### Ce que Sentinel fait

1. **Reconnaissance** - Analyse du projet, user stories
2. **Test Plan** - Génère plan de test basé sur features
3. **Exécution** - Tests continus avec checkpoints
4. **Notifications** - Updates Telegram à chaque phase
5. **Rapport** - Génère rapport complet

### Caractéristiques

- **Pas de time limit** - Tourne jusqu'à COMPLETE
- **Checkpoints** - Sauvegarde après chaque test
- **Auto-fix** - Peut corriger les bugs trouvés
- **Git integration** - Commits les fixes automatiquement

### Structure output

```
{projectPath}/.sentinel/
├── test-plan.md
├── test-results/
├── bugs/
└── REPORT.md
```

---

## 2. /bmad - BMAD Method Workflows

**Source:** `~/.claude/commands/bmad.md`

### Qu'est-ce que BMAD?

**B**uild **M**ore, **A**rchitect **D**reams - Framework AI-driven agile development.

### Usage

```bash
/bmad             # Menu workflows
/bmad init        # Initialiser BMAD dans projet
/bmad prd         # Créer Product Requirements Document
/bmad architect   # Workflow d'architecture
/bmad stories     # Découpage en user stories
```

### Agents BMAD disponibles

| Agent | Fichier | Expertise |
|-------|---------|-----------|
| PM | `pm.agent.yaml` | Product management, PRDs |
| Architect | `architect.agent.yaml` | System design |
| Dev | `dev.agent.yaml` | Implementation |
| Analyst | `analyst.agent.yaml` | Business analysis |
| SM | `sm.agent.yaml` | Scrum, agile |
| UX | `ux-designer.agent.yaml` | User experience |
| TEA | `tea.agent.yaml` | Testing |

### Workflow typique

```
1. /bmad prd         → Créer le PRD
2. /bmad architect   → Définir l'architecture
3. /bmad stories     → Découper en stories
4. /team [story]     → Implémenter chaque story
```

### Installation

```
~/.bmad-method/
├── src/
│   ├── bmm/
│   │   ├── agents/
│   │   ├── workflows/
│   │   ├── templates/
│   │   └── data/
│   └── core/
└── docs/
```

---

## 3. brainstorming - Design Before Code

**Source:** `~/.agents/skills/brainstorming/`

### Quand l'utiliser

**OBLIGATOIRE avant tout travail créatif:**
- Créer des features
- Construire des composants
- Ajouter des fonctionnalités
- Modifier des comportements

### Le processus

#### 1. Comprendre l'idée
- Check le contexte projet (files, docs, commits récents)
- Poser des questions **une à la fois**
- Préférer les questions à choix multiples
- Focus: purpose, constraints, success criteria

#### 2. Explorer les approches
- Proposer 2-3 approches différentes avec trade-offs
- Lead avec la recommandation et expliquer pourquoi

#### 3. Présenter le design
- Sections de 200-300 mots
- Demander validation après chaque section
- Couvrir: architecture, composants, data flow, error handling, testing

### Après le design

1. **Documentation:**
   - Écrire dans `docs/plans/YYYY-MM-DD-<topic>-design.md`
   - Commit le document

2. **Implementation (si continue):**
   - "Ready to set up for implementation?"
   - Créer workspace isolé (git worktrees)
   - Créer plan d'implémentation détaillé

### Principes clés

- **One question at a time** - Ne pas submerger
- **Multiple choice preferred** - Plus facile à répondre
- **YAGNI ruthlessly** - Retirer features inutiles
- **Explore alternatives** - Toujours 2-3 approches
- **Incremental validation** - Valider chaque section

---

## Comparaison des outils d'automation

| Outil | Best for |
|-------|----------|
| `/team` | Multi-agent coordination with tmux |
| `/sentinel-loop` | Tests continus, régression |
| `/bmad` | Planning, architecture, stories |
| `brainstorming` | Design avant code |

---

## Workflow recommandé

```
1. brainstorming    → Design la solution
2. /bmad prd        → Document requirements
3. /bmad architect  → Plan architecture
4. /bmad stories    → Break into stories
5. /team [story]    → Implement each story
6. /maniac --mode full → Full QA
7. Deploy!
```

---

*Dernière mise à jour: 2026-01-27*

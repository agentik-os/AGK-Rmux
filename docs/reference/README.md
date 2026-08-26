# Documentation des Skills Claude Code

> Guide complet de toutes les skills disponibles pour Claude Code sur ce VPS.

---

## Vue d'ensemble

Ce VPS dispose de **43+ skills** organisées en catégories, plus **14 commandes personnalisées** dans `~/.claude/commands/`.

### Emplacements

| Type | Chemin | Description |
|------|--------|-------------|
| **Skills globales** | `~/.agents/skills/` | Skills installées via `npx skills add` |
| **Commandes Claude** | `~/.claude/commands/` | Commandes `/slash` personnalisées |
| **Agents** | `~/.claude/agents/` | Agents spécialisés |

---

## Navigation Rapide

| Document | Description |
|----------|-------------|
| [01-debugging-skills.md](./01-debugging-skills.md) | **Debugging** - 4 skills de débogage systématique |
| [02-testing-skills.md](./02-testing-skills.md) | **Testing** - E2E, browser, QA autonome |
| [03-frontend-skills.md](./03-frontend-skills.md) | **Frontend** - UI/UX, shadcn, design |
| [04-backend-skills.md](./04-backend-skills.md) | **Backend** - Convex, Stripe, auth |
| [05-automation-skills.md](./05-automation-skills.md) | **Automation** - MCP, agents, /team |
| [06-marketing-skills.md](./06-marketing-skills.md) | **Marketing** - SEO, analytics, content |
| [07-utility-skills.md](./07-utility-skills.md) | **Utilities** - Context7, memes, etc. |
| [08-claude-commands.md](./08-claude-commands.md) | **Commandes /slash** personnalisées |
| [09-command-reference.md](./09-command-reference.md) | **🔥 RÉFÉRENCE COMPLÈTE** - Toutes les commandes avec options/flags/modes |

---

## Installation de nouvelles skills

```bash
# Installer une skill globalement
npx skills add https://github.com/REPO --skill SKILL_NAME -y -g

# Lister les skills d'un repo
npx skills list https://github.com/REPO

# Voir les skills installées
ls ~/.agents/skills/
```

---

## Comment utiliser une skill

Les skills sont **automatiquement chargées** par Claude Code selon le contexte. Tu peux aussi:

1. **Mentionner la skill** dans ta demande:
   ```
   "Utilise la skill debugging-wizard pour analyser cette erreur"
   ```

2. **Invoquer une commande** (si c'est une commande `/`):
   ```
   /team Fix the TypeScript errors
   /maniac kommu --mode full
   /e2e http://localhost:3000
   ```

3. **Laisser Claude choisir** - Claude sélectionne automatiquement la skill appropriée selon le contexte.

---

## Résumé par catégorie

### Debugging (4 skills)
| Skill | Source | Usage |
|-------|--------|-------|
| `debugging-wizard` | jeffallan/claude-skills | Erreurs, stack traces, root cause |
| `debugging` | mrgoonie/claudekit-skills | Framework 4 phases + sous-skills |
| `debugging-strategies` | wshobson/agents | Méthode scientifique, outils par langage |
| `systematic-debugging` | obra/superpowers | NO FIXES WITHOUT ROOT CAUSE |

### Testing (6 skills/commandes)
| Skill/Commande | Usage |
|----------------|-------|
| `webapp-testing` | Scripts Playwright pour tester les apps web |
| `agent-browser` | Automatisation browser CLI (click, fill, screenshot) |
| `e2e-testing-patterns` | Patterns E2E avec Playwright/Cypress |
| `/verify` | Vérification rapide (console, réseau, screenshots) |
| `/e2e` | Tests E2E autonomes complets |
| `/maniac` | Agent QA senior paranoïaque (le plus complet) |

### Frontend (5 skills)
| Skill | Usage |
|-------|-------|
| `frontend-design` | Interfaces production-grade, anti-AI-slop |
| `shadcn-ui` | Composants React accessibles avec Tailwind |
| `web-design-guidelines` | Review UI selon Web Interface Guidelines |
| `remotion-best-practices` | Création vidéo en React |
| `vercel-react-best-practices` | Performance React/Next.js |

### Backend (5 skills)
| Skill | Usage |
|-------|-------|
| `convex-best-practices` | Patterns Convex production-ready |
| `convex-realtime` | Subscriptions, optimistic updates |
| `stripe-best-practices` | Intégrations Stripe |
| `better-auth-best-practices` | Auth TypeScript |
| `mcp-builder` | Création de serveurs MCP |

### Automation (3 commandes)
| Commande | Usage |
|----------|-------|
| `/team` | Agent Teams with tmux coordination |
| `/sentinel-loop` | Tests continus avec checkpoints |
| `/bmad` | BMAD Method workflows |

### Marketing (8 skills)
| Skill | Usage |
|-------|-------|
| `seo-audit` | Audit SEO technique |
| `programmatic-seo` | Pages SEO à scale |
| `analytics-tracking` | GA4, GTM, tracking |
| `page-cro` | Optimisation conversion |
| `marketing-ideas` | 139 idées marketing |
| `marketing-psychology` | 70+ mental models |
| `social-content` | Contenu réseaux sociaux |
| `email-sequence` | Séquences email |

### Utilities (5 skills)
| Skill | Usage |
|-------|-------|
| `context7` | Documentation libraries up-to-date |
| `meme-factory` | Génération de memes |
| `brainstorming` | Design collaboratif |
| `skill-creator` | Créer de nouvelles skills |
| `gemini` | Code review avec Gemini CLI |

---

## Voir aussi

- **Règles Claude**: `/home/hacker/.claude/rules/`
- **Configuration globale**: `/home/hacker/CLAUDE.md`
- **goto.md**: `/home/hacker/goto.md`

---

*Dernière mise à jour: 2026-01-27*

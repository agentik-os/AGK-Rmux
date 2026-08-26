# Documentation Complète des Commandes

> **Référence exhaustive de TOUTES les commandes exécutables**
> Includes: Slash commands, agents, modes, options, flags, et alias bash

---

## Table des Matières

1. [Commandes de Test & QA](#1-commandes-de-test--qa)
2. [Commandes BMAD Method](#2-commandes-bmad-method)
3. [Commandes Vidéo (Remotion)](#3-commandes-vidéo-remotion)
4. [Alias Bash (tmux & navigation)](#4-alias-bash-tmux--navigation)
5. [Outils CLI Intégrés](#5-outils-cli-intégrés)
6. [Raccourcis Tmux](#6-raccourcis-tmux)

---

## 1. Commandes de Test & QA

### `/verify` - Vérification Rapide

**Description:** Test rapide post-développement (console, réseau, fonctionnalité basique).

```bash
/verify [url]
```

#### Ce qu'il vérifie
1. **Console** - Erreurs JavaScript
2. **Réseau** - Requêtes 4xx/5xx
3. **Fonctionnalité** - Page charge, contenu visible
4. **Interactions** - CTA principal, navigation, formulaire

#### Output
```
VERIFY RESULTS - [URL]
━━━━━━━━━━━━━━━━━━━━━━
Console Errors: 0 ✅
Network Errors: 0 ✅
Page Load: OK ✅
Main CTA: Works ✅
━━━━━━━━━━━━━━━━━━━━━━
VERDICT: ✅ PASS / ❌ FAIL
```

#### Durée
~30 secondes max

---

### `/responsive` - Test Responsive (Screenshots Only)

**Description:** Screenshots aux breakpoints principaux, sans interactions.

```bash
/responsive [url]
```

#### Viewports Testés
| Device | Largeur | Hauteur |
|--------|---------|---------|
| Desktop | 1440px | 900px |
| Tablet | 768px | 1024px |
| Mobile | 375px | 812px |

#### Output
Screenshots dans `/tmp/browser-screenshots/responsive/`

---

### `/e2e` - Tests E2E Autonomes (AI-Powered)

**Description:** Agent QA autonome qui découvre et teste tout de manière intelligente.

```bash
/e2e <project> [options]
```

#### Options
| Option | Description |
|--------|-------------|
| `--user free\|paid\|owner` | Type d'utilisateur pour auth |
| `--section <name>` | Focus sur une section |
| `--fix` | Trouver ET corriger les bugs |
| `--depth deep\|shallow` | Profondeur des tests |

#### Exemples
```bash
/e2e kommu                           # Test complet
/e2e kommu --user owner              # Avec compte owner
/e2e kommu --section checkout        # Focus checkout
/e2e kommu --fix                     # Trouve et corrige
/e2e kommu --depth deep              # Tests approfondis
```

#### Philosophie
> "Test like a curious human. Every screenshot reveals new possibilities."

#### Loop de Découverte
```
📸 SCREENSHOT → 🔍 ANALYZE → 📝 UPDATE PLAN → 🎯 EXECUTE → 🔄 REPEAT
```

#### Output
Rapport `REPORT.md` avec:
- Pages explorées
- Éléments découverts/testés
- Screenshots
- Bugs trouvés avec sévérité

---

### `/sentinel` - Boucle Test-Fix Autonome

**Description:** Tests autonomes sans limite de temps. Qualité over speed.

```bash
/sentinel <project> [options]
```

#### Options
| Option | Description |
|--------|-------------|
| `--resume` | Reprendre depuis checkpoint |

#### Phases
```
INIT → RECON → TEST → FIX → VERIFY → COMPLETE
```

#### Notifications
Envoie des updates Telegram à chaque phase.

---

### `/sentinel-loop` - Tests Continus avec Checkpoints

**Description:** Mode vraiment long-running avec persistance d'état. Peut tourner des jours.

```bash
/sentinel-loop <project> [options]
```

#### Options
| Option | Description |
|--------|-------------|
| `--resume` | Reprendre depuis checkpoint |
| `--max-hours N` | Limite de temps (défaut: 24h) |

#### Exemples
```bash
/sentinel-loop kommu                 # Start fresh
/sentinel-loop kommu --resume        # Resume from checkpoint
/sentinel-loop kommu --max-hours 48  # Up to 48 hours
```

#### State Machine
```
INIT → RECONNAISSANCE → TESTING → FIXING → VERIFYING → COMPLETE
         ↑                          ↓
         └──────── LOOP ────────────┘
```

#### Structure Working Dir
```
/tmp/sentinel-{project}/
├── state.json          # Full checkpoint
├── plan.json           # Test plan
├── bugs.json           # Bugs found
├── fixes.json          # Fixes applied
├── screenshots/
└── SENTINEL-REPORT.md
```

---

### `/test` - Spawn Agent Sentinel

**Description:** Lance l'agent Sentinel pour tests complets.

```bash
/test <project> [options]
```

#### Options
| Option | Description |
|--------|-------------|
| `--focus <area>` | Focus sur une zone |
| `--quick` | Smoke test rapide |

#### Exemples
```bash
/test kommu                    # Full test
/test kommu --focus checkout   # Focus checkout
/test gluten-libre --quick     # Quick smoke test
```

---

### `/test-mobile` - Tests Applications Mobiles

**Description:** Tests spécifiques pour apps Expo/React Native.

```bash
/test-mobile [app] [scope] [platform]
```

#### Apps Supportées
- `lifepixels` - LifePixels app
- `sagaforge` - SagaForge app

#### Scopes
| Scope | Description |
|-------|-------------|
| `quick` | Smoke test (launch, login, home) |
| `auth` | Tests authentification |
| `capture` | Tests capture photo |
| `timeline` | Tests timeline |
| `insights` | Tests insights |
| `settings` | Tests settings |
| `full` | Toutes les features |
| `offline` | Tests hors-ligne |
| `permissions` | Tests permissions |

#### Platforms
| Platform | Description |
|----------|-------------|
| `ios` | iOS uniquement |
| `android` | Android uniquement |
| `web` | Expo web export |
| `all` | Toutes les plateformes |

#### Exemples
```bash
/test-mobile lifepixels quick
/test-mobile lifepixels auth
/test-mobile lifepixels full ios
/test-mobile lifepixels offline
/test-mobile sagaforge permissions android
```

---

### `/maniac` - Agent de Test Senior Paranoïaque v3.0

**Description:** L'agent de test le plus intelligent et paranoïaque. Pense avant d'agir, formule des attentes explicites.

```bash
/maniac <project|url> [options]
```

#### Options Principales
| Option | Description |
|--------|-------------|
| `--resume` | Reprendre depuis checkpoint |
| `--mode <mode>` | Mode de test (voir ci-dessous) |
| `--depth <level>` | `quick\|normal\|deep\|exhaustive` |
| `--max-hours N` | Limite de temps |
| `--fix` | Corriger les bugs trouvés |
| `--report-only` | Générer rapport sans nouveaux tests |

#### Modes Disponibles

| Mode | Description | Résolution | Durée |
|------|-------------|------------|-------|
| `assault` | Tests agressifs + tous les flows (DEFAULT) | Desktop 1440x900 | 1-4h |
| `security` | Audit sécu (XSS, SQLi, CSRF, IDOR...) | Desktop | 2-6h |
| `chaos` | Chaos engineering (race conditions, multi-tab) | Desktop | 1-2h |
| `seo` | Audit SEO avec Squirrel (150+ règles) | Desktop + Mobile | 30min-2h |
| `a11y` | Accessibilité (axe-core + keyboard + ARIA) | Desktop | 30min-1h |
| `perf` | Performance + stress tests + Core Web Vitals | Desktop | 30min-1h |
| `ux` | Audit UX (99 guidelines, usability) | Desktop | 1-2h |
| `responsive` | TOUS les breakpoints (9 résolutions) | Mobile→4K | 1-3h |
| `full` | **TOUT** (10 phases complètes) | All | **6-24h** |

#### Breakpoints Mode Responsive (9 résolutions)
| Device | Résolution |
|--------|------------|
| Mobile S | 320x568 |
| Mobile M | 375x812 |
| Mobile L | 414x896 |
| Tablet Portrait | 768x1024 |
| Tablet Landscape | 1024x768 |
| Laptop | 1366x768 |
| Desktop | 1440x900 |
| Desktop L | 1920x1080 |
| 4K | 2560x1440 |

#### Depth Levels
| Level | Description |
|-------|-------------|
| `quick` | Tests de base seulement |
| `normal` | Tests standard (défaut) |
| `deep` | Tests approfondis |
| `exhaustive` | Tests ultra-exhaustifs |

#### Exemples Complets
```bash
# Test standard (desktop)
/maniac kommu

# MODE FULL - TOUT TESTER (avant release majeure)
/maniac kommu --mode full

# Test responsive UNIQUEMENT (9 breakpoints)
/maniac kommu --mode responsive

# Test UX UNIQUEMENT (99 guidelines)
/maniac kommu --mode ux

# Reprendre un test interrompu
/maniac kommu --resume

# Sécurité uniquement
/maniac kommu --mode security

# SEO avec Squirrel
/maniac kommu --mode seo

# Chaos engineering
/maniac kommu --mode chaos

# Accessibilité
/maniac kommu --mode a11y

# Performance
/maniac kommu --mode perf

# Tests exhaustifs (le plus profond)
/maniac kommu --mode full --depth exhaustive

# URL directe
/maniac https://example.com --mode security

# Avec correction automatique
/maniac kommu --fix
```

#### Protocole THINKING-FIRST
```
1. OBSERVER    → Qu'est-ce que je vois?
2. IDENTIFIER  → Quels éléments interactifs?
3. COMPRENDRE  → Contexte utilisateur?
4. ATTENDRE    → Que DEVRAIT-il se passer?
5. AGIR        → Exécuter l'action
6. COMPARER    → Résultat = attente?
7. ANALYSER    → Si non, pourquoi?
```

#### Classification Bugs
| Sévérité | Emoji | Exemples |
|----------|-------|----------|
| CRITICAL | 🔴 | XSS, auth bypass, crash, data loss |
| HIGH | 🟠 | Core flow cassé, 500 errors |
| MEDIUM | 🟡 | Edge case, validation manquante |
| LOW | 🟢 | Typo, alignement |
| UX | 🟣 | Confusion, friction, usability |

#### Verdict
| Condition | Verdict |
|-----------|---------|
| CRITICAL > 0 | 🚨 **NO-GO** |
| HIGH > 0 | 🚨 **NO-GO** |
| MEDIUM > 5 | ⚠️ **CONDITIONAL** |
| UX > 10 | ⚠️ **CONDITIONAL** |
| Sinon | ✅ **GO** |

#### Projets Pré-configurés
| Alias | URL Dev |
|-------|---------|
| `kommu` | http://72.61.197.216:33001 |
| `devlens` | http://72.61.197.216:33010 |
| `dent` | http://72.61.197.216:22002 |
| `gluten` | http://72.61.197.216:22001 |

#### Structure Working Dir
```
/tmp/maniac-{project}-{timestamp}/
├── state.json              # Checkpoint
├── maniac-test.log         # Log temps réel
├── recon/                  # Reconnaissance
├── discovery/              # Éléments + flows
├── bugs/                   # CRITICAL/HIGH/MEDIUM/LOW/UX
├── evidence/               # Screenshots, logs
├── security/
├── seo/
├── a11y/
├── performance/
├── responsive/             # Par breakpoint
└── reports/
    └── MANIAC-REPORT.md
```

---

## 2. Commandes BMAD Method

### `/bmad` - Workflows BMAD

**Description:** Framework AI-driven agile development avec agents spécialisés.

```bash
/bmad [workflow|agent] [args]
```

#### Workflows Disponibles
| Workflow | Description | Commande |
|----------|-------------|----------|
| `init` | Initialize project with BMAD | `/bmad init` |
| `prd` | Create Product Requirements Doc | `/bmad prd` |
| `architect` | Architecture planning | `/bmad architect` |
| `stories` | User story breakdown | `/bmad stories` |
| `dev` | Development workflow | `/bmad dev` |

#### Agents Disponibles
| Agent | Rôle | Utilisation |
|-------|------|-------------|
| `pm` | Product Manager | PRDs, requirements, roadmaps |
| `architect` | Solution Architect | System design, tech decisions |
| `dev` | Developer | Implementation, coding |
| `analyst` | Business Analyst | Analysis, specifications |
| `sm` | Scrum Master | Sprint planning, agile process |
| `ux` | UX Designer | User experience, wireframes |
| `tea` | Test & QA | Testing strategies |

#### Tracks
| Track | Meilleur Pour | Durée Planning |
|-------|---------------|----------------|
| Quick Flow | Bug fixes, small features | ~5 minutes |
| BMad Method | Products and platforms | ~15 minutes |
| Enterprise | Compliance-heavy systems | ~30 minutes |

#### Exemples
```bash
/bmad                    # Menu principal
/bmad init               # Initialiser BMAD
/bmad prd                # Créer un PRD
/bmad architect          # Planning architecture
/bmad stories            # Découpage user stories
/bmad pm                 # Activer agent PM
```

#### Fichiers
- Local: `/home/hacker/.bmad-method/`
- Docs: http://docs.bmad-method.org/

---

## 3. Commandes Vidéo (Remotion)

### `/remotion` - Création Vidéo en React

**Description:** Best practices pour Remotion - création vidéo programmatique.

```bash
/remotion
```

#### Workflow de Base
```bash
# 1. Créer projet
npx create-video@latest my-video
cd my-video

# 2. Développer
npm start  # Preview localhost:3000

# 3. Render
npx remotion render MyVideo out/video.mp4
npx remotion render MyVideo out/video.mp4 --codec h264  # Production
```

#### Topics Couverts
- Compositions, animations, timing
- Vidéos, audio, images, GIFs
- 3D (Three.js), captions, charts
- Transitions, text animations
- Maps, Lottie, Tailwind

---

## 4. Alias Bash (tmux & navigation)

### Commandes Principales

| Commande | Description |
|----------|-------------|
| `ts` | **Sélecteur global** - gérer sessions + clean RAM |
| `tps` | Liste rapide des sessions actives |

### Alias par Projet

#### Work
| Alias | Projet | Path |
|-------|--------|------|
| `c-kommu` | Kommu | `/home/hacker/VibeCoding/work/kommu` |
| `c-devlens` | DevLensPro | `/home/hacker/VibeCoding/work/DevLensPro` |
| `c-formation` | Formation-AI | `/home/hacker/VibeCoding/work/Formation-AI` |

#### Clients
| Alias | Projet | Path |
|-------|--------|------|
| `c-dent` | DentistryGPT | `/home/hacker/VibeCoding/clients/DentistryGPT` |
| `c-gluten` | Gluten-Libre | `/home/hacker/VibeCoding/clients/Gluten-Libre` |
| `c-resonant` | Resonant | `/home/hacker/VibeCoding/clients/resonant` |

#### AgentikOS
| Alias | Projet | Path |
|-------|--------|------|
| `c-life` | LifePixels | `/home/hacker/VibeCoding/agentic-os/LifePixels/App` |
| `c-saga` | SagaForge | `/home/hacker/VibeCoding/agentic-os/SagaForge` |
| `c-vision` | Vision | `/home/hacker/VibeCoding/agentic-os/vision` |
| `c-agentik` | AgentikDev | `/home/hacker/VibeCoding/agentic-os/AgentikDev` |
| `c-agtclaude` | Agentik-Claude | `/home/hacker/VibeCoding/agentic-os/Agentik-Claude` |

#### Life
| Alias | Projet | Path |
|-------|--------|------|
| `c-1life` | 1-Life | `/home/hacker/VibeCoding/1-life` |

#### Home
| Alias | Description |
|-------|-------------|
| `c-home` | Shell simple sans Claude |

### Menu Interactif (quand session existe)

| Touche | Action |
|--------|--------|
| `1-9` | Attacher à session correspondante |
| `N` / `n` | Nouvelle session |
| `D` / `d` | Supprimer UNE session |
| `K` / `k` | Supprimer TOUTES les sessions |
| `C` / `c` | Clean cache/RAM (SAFE) |
| `I` / `i` | Init/refresh contexte Claude |
| `X` / `x` / `kkc` | **NUCLEAR** (kill all + clean) |
| `Q` / `q` | Annuler |

### Autres Alias Utiles

| Alias | Description |
|-------|-------------|
| `save` | Git add + commit rapide |
| `sap` | Git add + commit + push |
| `st` | Git status |
| `gl` | Git log oneline |
| `gd` | Git diff |
| `gco` | Git checkout |
| `gcb` | Git checkout -b |
| `pn` | pnpm |
| `bn` | bun |

---

## 5. Outils CLI Intégrés

### agent-browser

**Description:** Outil de contrôle browser pour découverte et tests.

```bash
# Snapshot interactif - voir tous les éléments cliquables
agent-browser snapshot -i

# Console logs
agent-browser console

# Erreurs JavaScript
agent-browser errors

# Screenshot
agent-browser screenshot
```

### Squirrel (squirrelscan)

**Description:** Audit SEO, perf, sécurité, a11y avec 150+ règles.

```bash
# Audit complet
squirrel audit <url> --format llm

# SEO only
squirrel audit <url> --category seo

# Security only
squirrel audit <url> --category security
```

### Chrome Controller (Mac Tunnel)

**Description:** Contrôle Chrome sur Mac via tunnel SSH pour tests visuels LIVE.

```bash
# Test visuel avec screenshots responsive
node /home/hacker/.claude/lib/chrome-controller.js <url>

# Screenshots saved to /tmp/browser-screenshots/
```

---

## 6. Raccourcis Tmux

### Dans une session tmux

| Raccourci | Action |
|-----------|--------|
| `Ctrl+b d` | Detach (session continue en background) |
| `Ctrl+b c` | Nouvelle fenêtre |
| `Ctrl+b n` | Fenêtre suivante |
| `Ctrl+b p` | Fenêtre précédente |
| `Ctrl+b %` | Split vertical |
| `Ctrl+b "` | Split horizontal |
| `Ctrl+b o` | Switch pane |
| `Ctrl+b x` | Fermer pane |
| `Ctrl+b [` | Mode scroll (q pour quitter) |

### Status Bar

```
[ Session ] ─── CC: X% │ RAM: X% │ CPU: X% │ Disk: X% [ Agentik_OS ]
```

---

## Comparaison des Outils de Test

| Aspect | `/verify` | `/e2e` | `/maniac` |
|--------|-----------|--------|-----------|
| **Objectif** | Vérification rapide | Tests E2E complets | **PENSER puis CASSER** |
| **Durée** | ~30s-2min | ~30min-2h | **6-24h** (mode full) |
| **Approche** | "Ça marche?" | User stories | **Attentes explicites** |
| **Profondeur** | Console, réseau | Flows documentés | **TOUT** (10 phases) |
| **SEO** | Non | Non | **Squirrel (150+ règles)** |
| **Sécurité** | Non | Basique | **14 catégories** |
| **Responsive** | 3 breakpoints | Non | **9 breakpoints** |
| **Fix auto** | Non | `--fix` | `--fix` |

---

## Résumé Rapide

### Tests Rapides
```bash
/verify <url>              # Test rapide (30s)
/responsive <url>          # Screenshots 3 breakpoints
```

### Tests Complets
```bash
/e2e <project>             # Tests E2E autonomes
/test <project>            # Spawn Sentinel
/sentinel <project>        # Boucle test-fix
/sentinel-loop <project>   # Tests long-running
```

### Tests Maximum
```bash
/maniac <project>                    # Test standard
/maniac <project> --mode full        # TOUT tester
/maniac <project> --mode security    # Sécurité
/maniac <project> --mode responsive  # 9 breakpoints
/maniac <project> --resume           # Reprendre
```

### BMAD
```bash
/bmad init                 # Init BMAD
/bmad prd                  # Créer PRD
/bmad architect            # Architecture
/bmad stories              # User stories
```

### Navigation
```bash
ts                         # Sélecteur global
c-kommu                    # Session Kommu
c-devlens                  # Session DevLensPro
```

---

*Dernière mise à jour: 2026-01-27*
*Documentation complète des commandes Claude Code*

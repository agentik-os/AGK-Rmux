# Skills de Testing & QA

> Tools et agents pour tester les applications web de manière automatisée.

---

## Vue d'ensemble

| Outil | Type | Usage |
|-------|------|-------|
| `agent-browser` | Skill | CLI automatisation browser |
| `webapp-testing` | Skill | Scripts Playwright |
| `e2e-testing-patterns` | Skill | Patterns E2E |
| `/verify` | Commande | Vérification rapide |
| `/e2e` | Commande | Tests E2E autonomes |
| `/maniac` | Commande | Agent QA senior paranoïaque |

---

## 1. agent-browser

**Source:** `~/.agents/skills/agent-browser/`

### Quick Start

```bash
agent-browser open <url>        # Naviguer vers une page
agent-browser snapshot -i       # Éléments interactifs avec refs
agent-browser click @e1         # Cliquer élément par ref
agent-browser fill @e2 "text"   # Remplir input par ref
agent-browser close             # Fermer browser
```

### Workflow principal

```
1. Navigate: agent-browser open <url>
2. Snapshot: agent-browser snapshot -i (retourne @e1, @e2, etc.)
3. Interact: Utiliser les refs du snapshot
4. Re-snapshot: Après navigation ou changement DOM
```

### Commandes essentielles

#### Navigation
```bash
agent-browser open <url>      # Naviguer
agent-browser back            # Retour
agent-browser forward         # Avancer
agent-browser reload          # Recharger
agent-browser close           # Fermer
```

#### Snapshot (analyse page)
```bash
agent-browser snapshot            # Arbre accessibilité complet
agent-browser snapshot -i         # Éléments interactifs seulement (RECOMMANDÉ)
agent-browser snapshot -c         # Output compact
agent-browser snapshot -s "#main" # Scope CSS selector
```

#### Interactions
```bash
agent-browser click @e1           # Clic
agent-browser fill @e2 "text"     # Effacer et taper
agent-browser type @e2 "text"     # Taper sans effacer
agent-browser press Enter         # Touche
agent-browser hover @e1           # Survol
agent-browser select @e1 "value"  # Sélectionner option
agent-browser scroll down 500     # Scroller
agent-browser upload @e1 file.pdf # Upload fichiers
```

#### Get info
```bash
agent-browser get text @e1        # Texte élément
agent-browser get value @e1       # Valeur input
agent-browser get url             # URL courante
agent-browser get title           # Titre page
```

#### Screenshots & PDF
```bash
agent-browser screenshot          # Save temp
agent-browser screenshot path.png # Save spécifique
agent-browser screenshot --full   # Page entière
agent-browser pdf output.pdf      # PDF
```

#### Video recording
```bash
agent-browser record start ./demo.webm  # Démarrer
agent-browser record stop               # Arrêter et sauver
```

#### Wait
```bash
agent-browser wait @e1                     # Attendre élément
agent-browser wait 2000                    # Attendre ms
agent-browser wait --text "Success"        # Attendre texte
agent-browser wait --load networkidle      # Attendre réseau idle
```

### Exemple: Soumission formulaire

```bash
agent-browser open https://example.com/form
agent-browser snapshot -i
# Output: textbox "Email" [ref=e1], textbox "Password" [ref=e2], button "Submit" [ref=e3]

agent-browser fill @e1 "user@example.com"
agent-browser fill @e2 "password123"
agent-browser click @e3
agent-browser wait --load networkidle
agent-browser snapshot -i  # Vérifier résultat
```

---

## 2. webapp-testing

**Source:** `~/.agents/skills/webapp-testing/`

### Principe

Écrire des scripts Playwright natifs Python pour tester les apps web locales.

### Script helper disponible

```bash
# Gérer le cycle de vie serveur
python scripts/with_server.py --help
```

### Usage

#### Single server
```bash
python scripts/with_server.py --server "npm run dev" --port 5173 -- python your_automation.py
```

#### Multiple servers
```bash
python scripts/with_server.py \
  --server "cd backend && python server.py" --port 3000 \
  --server "cd frontend && npm run dev" --port 5173 \
  -- python your_automation.py
```

### Script d'automatisation type

```python
from playwright.sync_api import sync_playwright

with sync_playwright() as p:
    browser = p.chromium.launch(headless=True)
    page = browser.new_page()
    page.goto('http://localhost:5173')
    page.wait_for_load_state('networkidle')  # CRITIQUE!

    # Screenshot pour inspection
    page.screenshot(path='/tmp/inspect.png', full_page=True)

    # Automation logic
    page.click('button:has-text("Submit")')

    browser.close()
```

### Piège commun

❌ Inspecter le DOM **avant** `networkidle` sur apps dynamiques
✅ Toujours attendre `page.wait_for_load_state('networkidle')` avant inspection

---

## 3. /verify - Vérification Rapide

**Source:** `~/.claude/commands/verify.md`

### Usage

```bash
/verify <url>
/verify http://72.61.197.216:33001
```

### Ce que ça fait

1. **Navigation** vers l'URL
2. **Console** - Vérifier 0 erreurs JS
3. **Réseau** - Vérifier pas de 4xx/5xx
4. **Screenshots** - Desktop, Tablet, Mobile
5. **Analyse** visuelle du rendu

### Breakpoints testés

| Device | Résolution |
|--------|------------|
| Desktop | 1440x900 |
| Tablet | 768x1024 |
| Mobile | 375x812 |

### Quand utiliser

- Après chaque modification UI
- Vérification rapide post-dev
- ~2-5 minutes

---

## 4. /e2e - Tests E2E Autonomes

**Source:** `~/.claude/commands/e2e.md`

### Usage

```bash
/e2e <project|url> [options]

Options:
  --user free|paid|owner   # Type utilisateur
  --section <name>         # Section spécifique
  --fix                    # Trouver ET corriger
  --depth deep|shallow     # Profondeur
```

### Exemples

```bash
/e2e kommu
/e2e http://localhost:3000 --user paid
/e2e kommu --section chat --fix
```

### Philosophie

> "Test like a curious human. Every screenshot reveals new possibilities."

Ce n'est PAS une suite de tests statique. L'IA continuellement:
1. Prend des screenshots
2. Analyse ce qui est visible
3. Découvre nouveaux éléments testables
4. Décide quoi tester ensuite
5. Adapte la stratégie

### Discovery Loop

```
📸 SCREENSHOT
    ↓
🔍 ANALYZE (AI Vision)
    - Quels boutons existent?
    - Quels formulaires sont visibles?
    - Quels liens cliquables?
    ↓
📝 UPDATE TEST PLAN
    - Ajouter éléments découverts
    - Prioriser chemins non testés
    ↓
🎯 EXECUTE NEXT TEST
    ↓
📸 SCREENSHOT (nouvel état)
    ↓
🔄 REPEAT jusqu'à exhaustion
```

### Output

```
QA_REPORT/
├── bugs/
│   ├── CRITICAL/
│   ├── HIGH/
│   ├── MEDIUM/
│   └── LOW/
├── evidence/
│   └── screenshots/
└── REPORT.md
```

---

## 5. /maniac - Agent QA Senior Paranoïaque

**Source:** `~/.claude/commands/maniac.md`

### LE PLUS COMPLET

> "Je ne clique pas pour voir si ça marche. Je PENSE à ce qui DEVRAIT arriver."

### Usage

```bash
/maniac <project|url> [options]

Options:
  --resume              # Reprendre depuis checkpoint
  --mode <mode>         # Mode de test
  --depth <level>       # quick|normal|deep|exhaustive
  --fix                 # Corriger les bugs trouvés
```

### Modes disponibles

| Mode | Description | Durée |
|------|-------------|-------|
| `assault` | Tests agressifs (DEFAULT) | 1-4h |
| `security` | XSS, SQLi, CSRF, IDOR... | 2-6h |
| `chaos` | Race conditions, multi-tab | 1-2h |
| `seo` | Squirrel 150+ règles | 30min-2h |
| `a11y` | Accessibilité axe-core | 30min-1h |
| `perf` | Performance, Core Web Vitals | 30min-1h |
| `ux` | 99 guidelines usability | 1-2h |
| `responsive` | 9 breakpoints (320px→4K) | 1-3h |
| **`full`** | **TOUT** | **6-24h** |

### Exemples

```bash
# Test standard
/maniac kommu

# MODE FULL - AVANT RELEASE MAJEURE
/maniac kommu --mode full

# Responsive uniquement
/maniac kommu --mode responsive

# Sécurité
/maniac kommu --mode security

# Reprendre un test interrompu
/maniac kommu --resume

# Tests exhaustifs
/maniac kommu --mode full --depth exhaustive
```

### Protocole THINKING-FIRST

```
1. OBSERVER    → Qu'est-ce que je vois?
2. IDENTIFIER  → TOUS les éléments interactifs?
3. COMPRENDRE  → Que veut faire l'utilisateur?
4. ATTENDRE    → Que DEVRAIT-il se passer?
5. AGIR        → Exécuter l'action
6. COMPARER    → Résultat = attente?
7. ANALYSER    → Si non, pourquoi?
```

### Différence avec les autres

| Aspect | /verify | /e2e | /maniac |
|--------|---------|------|---------|
| Philosophie | "Ça marche?" | User stories | **Attentes explicites** |
| Durée | 2-5 min | 30min-2h | **6-24h** |
| SEO | Non | Non | **150+ règles** |
| Sécurité | Non | Basique | **14 catégories** |
| Responsive | 3 breakpoints | Non | **9 breakpoints** |

### Classification bugs

| Sévérité | Tolérance Prod |
|----------|----------------|
| 🔴 CRITICAL | 0 |
| 🟠 HIGH | 0 |
| 🟡 MEDIUM | 5 |
| 🟢 LOW | 10 |
| 🟣 UX | 10 |

### Verdict

| Condition | Verdict |
|-----------|---------|
| CRITICAL > 0 | 🚨 **NO-GO** |
| HIGH > 0 | 🚨 **NO-GO** |
| MEDIUM > 5 | ⚠️ **CONDITIONAL** |
| Sinon | ✅ **GO** |

### Output

```
/tmp/maniac-{project}-{timestamp}/
├── state.json              # Checkpoint
├── maniac-test.log         # Log temps réel
├── recon/
├── discovery/
├── bugs/
├── evidence/
├── responsive/             # Screenshots par breakpoint
└── reports/
    └── MANIAC-REPORT.md
```

---

## Comparaison complète

| Outil | Cas d'usage | Durée | Output |
|-------|-------------|-------|--------|
| `agent-browser` | Automatisation CLI rapide | Minutes | Console |
| `webapp-testing` | Scripts Playwright custom | Variable | Script output |
| `/verify` | Post-dev check | 2-5 min | Inline |
| `/e2e` | QA pre-prod standard | 30min-2h | `QA_REPORT/` |
| `/maniac` | Audit complet, release | 6-24h | `/tmp/maniac-*/` |

---

## Projets pré-configurés

| Alias | URL Dev |
|-------|---------|
| `kommu` | http://72.61.197.216:33001 |
| `devlens` | http://72.61.197.216:33010 |
| `dent` | http://72.61.197.216:22002 |
| `gluten` | http://72.61.197.216:22001 |

---

*Dernière mise à jour: 2026-01-27*

# Commandes Claude Personnalisées

> Commandes `/slash` disponibles dans `~/.claude/commands/`

---

## Vue d'ensemble

| Commande | Description |
|----------|-------------|
| `/verify` | Vérification rapide |
| `/e2e` | Tests E2E autonomes |
| `/maniac` | Agent QA senior |
| `/sentinel` | Spawner agent QA |
| `/sentinel-loop` | Tests continus |
| `/test` | Spawner Sentinel |
| `/test-mobile` | Tests mobile |
| `/responsive` | Tests responsive |
| `/bmad` | BMAD workflows |
| `/remotion` | Création vidéo |

---

## Commandes de test

### /verify

**Fichier:** `~/.claude/commands/verify.md`

```bash
/verify <url>
```

Vérification rapide après modifications:
- Console errors
- Network requests
- Screenshots (Desktop, Tablet, Mobile)

**Exemple:**
```bash
/verify http://72.61.197.216:33001
```

---

### /e2e

**Fichier:** `~/.claude/commands/e2e.md`

```bash
/e2e <project|url> [options]

Options:
  --user free|paid|owner
  --section <name>
  --fix
  --depth deep|shallow
```

Tests E2E autonomes avec discovery loop.

**Exemples:**
```bash
/e2e kommu
/e2e http://localhost:3000 --fix
/e2e kommu --section chat --user paid
```

---

### /maniac

**Fichier:** `~/.claude/commands/maniac.md`

```bash
/maniac <project|url> [options]

Options:
  --resume
  --mode <mode>       assault|security|chaos|seo|a11y|perf|ux|responsive|full
  --depth <level>     quick|normal|deep|exhaustive
  --fix
  --max-hours <N>
```

Agent QA senior ultra-complet.

**Exemples:**
```bash
/maniac kommu
/maniac kommu --mode full
/maniac kommu --mode security
/maniac kommu --resume
```

---

### /sentinel

**Fichier:** `~/.claude/commands/sentinel.md`

```bash
/sentinel <project>
```

Spawne un agent Sentinel pour tests autonomes.

---

### /sentinel-loop

**Fichier:** `~/.claude/commands/sentinel-loop.md`

```bash
/sentinel-loop <project>
```

Tests continus avec checkpoints. Peut tourner pendant des heures/jours.

---

### /test

**Fichier:** `~/.claude/commands/test.md`

```bash
/test
```

Spawne l'agent Sentinel QA.

---

### /test-mobile

**Fichier:** `~/.claude/commands/test-mobile.md`

```bash
/test-mobile <app>
```

Tests spécifiques pour applications mobiles (Expo/React Native).

---

### /responsive

**Fichier:** `~/.claude/commands/responsive.md`

```bash
/responsive <url>
```

Tests responsive design multi-breakpoints.

---

## Commandes de workflow

### /bmad

**Fichier:** `~/.claude/commands/bmad.md`

```bash
/bmad [subcommand]

Subcommands:
  init        Initialiser BMAD
  prd         Créer PRD
  architect   Workflow architecture
  stories     Découpage user stories
```

Accès aux workflows BMAD Method.

**Exemples:**
```bash
/bmad
/bmad prd
/bmad architect
/bmad stories
```

---

## Commande création

### /remotion

**Fichier:** `~/.claude/commands/remotion.md`

```bash
/remotion [prompt]
```

Création de vidéos avec Remotion (React).

---

## Script wrapper

### verify-wrapper.sh

**Fichier:** `~/.claude/commands/verify-wrapper.sh`

```bash
./verify-wrapper.sh <url>
```

Script bash wrapper pour /verify.

---

## Créer une nouvelle commande

1. Créer le fichier dans `~/.claude/commands/`:
   ```bash
   touch ~/.claude/commands/my-command.md
   ```

2. Structure du fichier:
   ```markdown
   # /my-command - Description

   ## Usage

   ```
   /my-command [args]
   ```

   ## Examples

   ```bash
   /my-command example1
   /my-command example2
   ```

   ## What This Command Does

   Description détaillée...

   ---

   $ARGUMENTS

   Instructions pour Claude quand la commande est invoquée...
   ```

3. La commande est immédiatement disponible.

---

## Résumé par catégorie

### Testing
| Commande | Durée | Usage |
|----------|-------|-------|
| `/verify` | 2-5 min | Quick check |
| `/e2e` | 30min-2h | QA complet |
| `/maniac` | 6-24h | Audit exhaustif |
| `/sentinel-loop` | Heures/jours | Tests continus |
| `/responsive` | 5-10 min | Multi-breakpoints |
| `/test-mobile` | Variable | Apps mobiles |

### Workflow
| Commande | Usage |
|----------|-------|
| `/bmad` | BMAD Method |
| `/remotion` | Création vidéo |

---

## Voir aussi

- **Skills:** `/home/hacker/.agents/skills/`
- **Agents:** `/home/hacker/.claude/agents/`
- **Rules:** `/home/hacker/.claude/rules/`

---

*Dernière mise à jour: 2026-01-27*

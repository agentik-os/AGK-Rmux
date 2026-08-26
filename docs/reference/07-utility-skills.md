# Skills Utilitaires

> Documentation, brainstorming, memes, et autres outils pratiques.

---

## Vue d'ensemble

| Skill | Usage |
|-------|-------|
| `context7` | Documentation libraries up-to-date |
| `meme-factory` | Génération de memes |
| `skill-creator` | Créer de nouvelles skills |
| `gemini` | Code review avec Gemini CLI |
| `agent-md-refactor` | Refactorer fichiers AGENTS.md |
| `expo-tailwind-setup` | Setup Tailwind dans Expo |
| `upgrading-expo` | Upgrade Expo SDK |

---

## 1. context7

**Source:** `~/.agents/skills/context7/`

### Quand l'utiliser

- Documentation de libraries up-to-date
- Exemples de code pour APIs
- Vérifier usage correct de fonctions
- Information sur APIs qui ont changé

### Comment ça marche

Récupère la documentation à jour via l'API Context7 pour n'importe quelle library ou framework.

### Avantage

La documentation est **à jour** - contrairement aux connaissances de Claude qui ont un cutoff.

### Usage typique

```
"Utilise context7 pour récupérer la doc de react-query v5"
"Check la doc à jour de Next.js 15 pour les server actions"
```

---

## 2. meme-factory

**Source:** `~/.agents/skills/meme-factory/`

### Quand l'utiliser

- Générer des memes
- Ajouter de l'humour au contenu
- Visual aids pour social media

### Comment ça marche

Utilise l'API memegen.link avec 100+ templates populaires.

### Templates populaires

- Drake
- Distracted Boyfriend
- Two Buttons
- Change My Mind
- Expanding Brain
- Et 95+ autres...

### Exemple

```
"Crée un meme Drake avec:
- Haut: 'Writing documentation'
- Bas: 'Asking Claude to write it'"
```

---

## 3. skill-creator

**Source:** `~/.agents/skills/skill-creator/`

### Quand l'utiliser

- Créer une nouvelle skill
- Mettre à jour une skill existante
- Étendre les capacités de Claude

### Structure d'une skill

```
my-skill/
├── SKILL.md          # Main skill file
├── references/       # Documentation additionnelle
│   ├── topic-a.md
│   └── topic-b.md
└── templates/        # Templates réutilisables
```

### Format SKILL.md

```markdown
---
name: my-skill
description: Description claire de quand utiliser cette skill
triggers:
  - keyword1
  - keyword2
---

# My Skill

## When to Use

- Use case 1
- Use case 2

## Instructions

Detailed instructions...

## Examples

Example usage...
```

### Output patterns

Voir `references/output-patterns.md` pour les formats de sortie standards.

---

## 4. gemini

**Source:** `~/.agents/skills/gemini/`

### Quand l'utiliser

- Code review nécessitant grand contexte (>200k tokens)
- Plan review
- Big context processing

### Pourquoi Gemini?

Gemini 3 Pro a une fenêtre de contexte plus grande que Claude pour certaines analyses qui nécessitent beaucoup de contexte.

### Usage

```bash
"Run Gemini CLI for code review of this codebase"
```

---

## 5. agent-md-refactor

**Source:** `~/.agents/skills/agent-md-refactor/`

### Quand l'utiliser

Refactorer des fichiers `AGENTS.md`, `CLAUDE.md` ou similaires qui sont devenus trop volumineux.

### Ce qu'il fait

- Split fichiers monolithiques
- Organisation progressive disclosure
- Liens entre documents

### Pattern de sortie

```
.claude/
├── CLAUDE.md           # Main, concis
├── rules/
│   ├── 01-core.md
│   ├── 02-testing.md
│   └── ...
└── agents/
    └── specialized/
```

---

## 6. expo-tailwind-setup

**Source:** `~/.agents/skills/expo-tailwind-setup/`

### Quand l'utiliser

Setup Tailwind CSS v4 dans un projet Expo avec:
- react-native-css
- NativeWind v5
- Styling universel

### Compatibilité

- Expo SDK 52+
- Tailwind CSS v4
- NativeWind v5

---

## 7. upgrading-expo

**Source:** `~/.agents/skills/upgrading-expo/`

### Quand l'utiliser

- Upgrade Expo SDK versions
- Fix dependency issues
- Résoudre problèmes de compatibilité

### Problèmes communs traités

- Versions natives incompatibles
- Peer dependency conflicts
- Build errors après upgrade

---

## Autres skills utilitaires

### sales-engineer

Expert en ventes techniques:
- Product demonstrations
- Technical validation (PoC)
- Solution design
- Bridge sales/engineering

---

## Installation de nouvelles skills

### Depuis GitHub

```bash
# Lister skills disponibles
npx skills list https://github.com/REPO

# Installer une skill
npx skills add https://github.com/REPO --skill SKILL_NAME -y -g
```

### Créer une skill custom

1. Créer le dossier dans `~/.agents/skills/my-skill/`
2. Créer `SKILL.md` avec le format approprié
3. Ajouter références si nécessaire

---

## Tips pour utiliser les skills

1. **Sois spécifique** - Mentionne la skill par nom si tu veux l'utiliser
2. **Laisse Claude choisir** - Claude sélectionne automatiquement selon le contexte
3. **Combine les skills** - Plusieurs skills peuvent être utilisées ensemble
4. **Check les références** - Certaines skills ont de la doc additionnelle

---

*Dernière mise à jour: 2026-01-27*

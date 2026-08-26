# Skills de Debugging

> 4 skills complémentaires pour un débogage systématique et efficace.

---

## Vue d'ensemble

| Skill | Source | Philosophie |
|-------|--------|-------------|
| `debugging-wizard` | jeffallan/claude-skills | Expert 15+ ans, workflow 6 étapes |
| `debugging` | mrgoonie/claudekit-skills | Collection de 4 sous-skills |
| `debugging-strategies` | wshobson/agents | Méthode scientifique + outils |
| `systematic-debugging` | obra/superpowers | **NO FIXES WITHOUT ROOT CAUSE** |

---

## 1. debugging-wizard

**Source:** `~/.agents/skills/debugging-wizard/`

### Quand l'utiliser
- Investigation d'erreurs et exceptions
- Analyse de stack traces
- Bugs intermittents
- Performance debugging
- Memory leaks
- Race conditions

### Workflow en 6 étapes

```
1. REPRODUCE   → Établir des étapes de reproduction consistantes
2. ISOLATE     → Réduire au plus petit cas qui échoue
3. HYPOTHESIZE → Former des théories testables
4. TEST        → Vérifier/infirmer chaque hypothèse
5. FIX         → Implémenter et vérifier la solution
6. PREVENT     → Ajouter tests/safeguards contre régression
```

### Références disponibles

| Fichier | Quand charger |
|---------|---------------|
| `references/debugging-tools.md` | Setup debuggers par langage |
| `references/common-patterns.md` | Reconnaître les patterns de bugs |
| `references/strategies.md` | Binary search, git bisect, time travel |
| `references/quick-fixes.md` | Solutions erreurs courantes |
| `references/systematic-debugging.md` | Bugs complexes, root cause |

### Output attendu

```markdown
1. **Root Cause**: Ce qui a spécifiquement causé le problème
2. **Evidence**: Stack trace, logs, ou test qui le prouve
3. **Fix**: Changement de code qui résout
4. **Prevention**: Test ou safeguard contre récurrence
```

---

## 2. debugging (Collection)

**Source:** `~/.agents/skills/debugging/`

### 4 Sous-skills incluses

#### 2.1 systematic-debugging
**Chemin:** `systematic-debugging/SKILL.md`

Framework 4 phases: Root Cause Investigation → Pattern Analysis → Hypothesis Testing → Implementation.

**Iron Law:** NO FIXES WITHOUT ROOT CAUSE INVESTIGATION FIRST.

#### 2.2 root-cause-tracing
**Chemin:** `root-cause-tracing/SKILL.md`

Tracer les bugs en remontant la call stack. Ne pas fixer les symptômes - trouver où les données invalides ont été créées.

#### 2.3 defense-in-depth
**Chemin:** `defense-in-depth/SKILL.md`

Valider à chaque couche pour rendre les bugs structurellement impossibles.

4 couches: Entry Point → Business Logic → Environment Guards → Debug Instrumentation.

#### 2.4 verification-before-completion
**Chemin:** `verification-before-completion/SKILL.md`

Exécuter les commandes de vérification AVANT de dire "c'est fait".

**Iron Law:** NO COMPLETION CLAIMS WITHOUT FRESH VERIFICATION EVIDENCE.

### Quick Dispatch

| Symptôme | Sous-skill |
|----------|-----------|
| Test failure, comportement inattendu | systematic-debugging |
| Erreur au mauvais endroit | root-cause-tracing |
| Bug récurrent | defense-in-depth |
| Confirmer qu'un fix marche | verification-before-completion |

---

## 3. debugging-strategies

**Source:** `~/.agents/skills/debugging-strategies/`

### Quand l'utiliser
- Tracking de bugs élusifs
- Investigation de performance
- Comprendre un codebase inconnu
- Debug production
- Analyse crash dumps
- Memory leaks
- Systèmes distribués

### Méthode Scientifique

```
1. OBSERVE    → Quel est le comportement réel?
2. HYPOTHESIZE → Qu'est-ce qui pourrait causer ça?
3. EXPERIMENT → Tester l'hypothèse
4. ANALYZE    → A-t-elle été prouvée/réfutée?
5. REPEAT     → Jusqu'à trouver la root cause
```

### Outils par langage

#### JavaScript/TypeScript
```typescript
// Chrome DevTools
debugger; // Pause l'exécution

// Console techniques
console.log("Value:", value);
console.table(arrayOfObjects);
console.time("operation");
console.trace(); // Stack trace
console.assert(condition, "Message");
```

#### Python
```python
import pdb
pdb.set_trace()  # Debugger démarre ici

# Python 3.7+
breakpoint()  # Plus pratique

# Profiling
import cProfile
cProfile.run('slow_function()', 'profile_stats')
```

#### Go
```go
import "runtime/debug"
debug.PrintStack()  // Print stack trace
```

### Techniques avancées

#### Binary Search avec Git Bisect
```bash
git bisect start
git bisect bad                # Commit actuel est cassé
git bisect good v1.0.0        # v1.0.0 fonctionnait
# Git checkout le commit du milieu
git bisect good/bad           # Répéter
git bisect reset              # Quand terminé
```

#### Differential Debugging
```markdown
| Aspect       | Working     | Broken         |
| ------------ | ----------- | -------------- |
| Environment  | Development | Production     |
| Node version | 18.16.0     | 18.15.0        |
| Data         | Empty DB    | 1M records     |
```

### Checklist rapide

```markdown
Quand bloqué, vérifier:
- [ ] Erreurs de typo (noms de variables)
- [ ] Sensibilité à la casse
- [ ] Valeurs null/undefined
- [ ] Off-by-one dans les arrays
- [ ] Race conditions (timing async)
- [ ] Problèmes de scope
- [ ] Type mismatches
- [ ] Dependencies manquantes
- [ ] Variables d'environnement
- [ ] Chemins de fichiers
- [ ] Cache (vider le cache)
```

---

## 4. systematic-debugging (standalone)

**Source:** `~/.agents/skills/systematic-debugging/`

### Philosophie

> "Random fixes waste time and create new bugs. Quick patches mask underlying issues."

**Core principle:** ALWAYS find root cause before attempting fixes. Symptom fixes are failure.

### The Iron Law

```
NO FIXES WITHOUT ROOT CAUSE INVESTIGATION FIRST
```

Si tu n'as pas complété Phase 1, tu ne peux PAS proposer de fixes.

### Les 4 Phases

#### Phase 1: Root Cause Investigation

1. **Lire les messages d'erreur attentivement**
   - Ne pas sauter les erreurs/warnings
   - Lire les stack traces complètement
   - Noter les numéros de ligne, chemins, codes erreur

2. **Reproduire de façon consistante**
   - Peut-on le déclencher de façon fiable?
   - Si pas reproductible → gather more data, don't guess

3. **Vérifier les changements récents**
   - Git diff, commits récents
   - Nouvelles dépendances, config changes

4. **Collecter des preuves (systèmes multi-composants)**
   ```bash
   # Pour chaque frontière de composant:
   # - Log ce qui entre
   # - Log ce qui sort
   # - Vérifier propagation env/config
   ```

5. **Tracer le flux de données**
   - Où la mauvaise valeur a-t-elle été créée?
   - Qui a appelé cette fonction avec cette valeur?
   - Remonter jusqu'à la source

#### Phase 2: Pattern Analysis

1. Trouver des exemples fonctionnels similaires
2. Comparer avec des références
3. Identifier les différences
4. Comprendre les dépendances

#### Phase 3: Hypothesis and Testing

1. Former UNE hypothèse: "Je pense que X est la cause parce que Y"
2. Faire le PLUS PETIT changement possible pour tester
3. Vérifier avant de continuer
4. Si ça ne marche pas → Nouvelle hypothèse (ne pas empiler les fixes)

#### Phase 4: Implementation

1. Créer un test case qui échoue
2. Implémenter UN SEUL fix
3. Vérifier le fix
4. **Si 3+ fixes ont échoué → QUESTIONNER L'ARCHITECTURE**

### Red Flags - STOP et suivre le process

Si tu penses:
- "Quick fix for now, investigate later"
- "Just try changing X and see"
- "Add multiple changes, run tests"
- "I don't fully understand but this might work"
- "Here are the main problems: [lists fixes without investigation]"

**STOP. Retourne à Phase 1.**

### Impact réel

| Approche | Temps pour fix | First-time fix rate |
|----------|----------------|---------------------|
| Systématique | 15-30 min | 95% |
| Random fixes | 2-3 heures | 40% |

---

## Quand utiliser quelle skill?

| Situation | Skill recommandée |
|-----------|-------------------|
| Bug simple, erreur claire | `debugging-wizard` |
| Bug complexe, multiple fails | `systematic-debugging` |
| Besoin d'outils spécifiques | `debugging-strategies` |
| Collection complète | `debugging` (accès aux 4 sous-skills) |
| Plusieurs fixes ont échoué | `systematic-debugging` (Phase 4.5) |
| Performance issue | `debugging-strategies` |
| Memory leak | `debugging-strategies` |

---

## Commandes utiles

```bash
# Voir les skills de debugging installées
ls ~/.agents/skills/debugging* ~/.agents/skills/systematic-debugging

# Lire une skill
cat ~/.agents/skills/debugging-wizard/SKILL.md

# Lire les références
cat ~/.agents/skills/debugging-wizard/references/strategies.md
```

---

*Dernière mise à jour: 2026-01-27*

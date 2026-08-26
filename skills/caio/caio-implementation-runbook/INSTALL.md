# Install — CAIO Implementation Runbook

This skill ships with OmegaOS and installs to `~/.omega/skills/caio-implementation-runbook/` (a fresh `git clone … && ./install.sh` reproduces it; `omega sync` re-links it).

## Manual install (standalone)

```bash
# Claude Code
mkdir -p ~/.claude/skills/caio-implementation-runbook
cp -r ./* ~/.claude/skills/caio-implementation-runbook/
bash ~/.claude/skills/caio-implementation-runbook/platforms/claude.sh

# Codex
bash ./platforms/codex.sh        # symlinks SKILL.md -> AGENTS.md

# Gemini CLI
bash ./platforms/gemini.sh       # writes GEMINI.md activation pointer
```

## Trigger

In Claude Code:

```
/caio-implementation-runbook
```

Or describe the task in natural language — the skill fires on EN triggers ("CAIO implementation", "build the company AI OS", "realize the architecture", "micro-SaaS per C-level", "inter-dashboard API", "Composio wiring", "ship-gate", "go-live") and FR triggers ("runbook d'implémentation", "construire l'OS IA de l'entreprise", "réaliser l'architecture", "micro-SaaS par C-level", "API inter-dashboards", "intégration Composio", "mise en production", "livrer de la valeur en semaine 1").

## Prerequisite (chain)

This is **Phase 2 (BUILD)**. It REQUIRES the architect's output:

```
caio-enterprise-workflow-architect  ->  ./company-ai-os/   (blueprint + backlog + feature specs + roadmap + ROI)
                                          |
                                          v
caio-implementation-runbook         ->  ./caio-build/      (realize + build the federated topology)
                                          |
                                          v
caio-enablement-and-transfer        (Phase 3/4: adoption + transfer)
```

If `./company-ai-os/` does not exist, run `/caio-enterprise-workflow-architect` first. This skill **delegates** per-agent builds to `agentic-systems-builder` and repeatable-skill codification to `agentik-skill-forge` — install those too if you will run a full build.

## What gets written

A `./caio-build/` directory (see README.md → "What it produces"). Nothing is written outside it except via the chosen tool (e.g. the actual server/app code your build targets).

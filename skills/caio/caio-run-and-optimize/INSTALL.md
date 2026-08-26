# Install

1. Drop the `caio-run-and-optimize/` folder into your skills directory
   (e.g. `~/.omega/skills/`, `/mnt/skills/user/`, or your Claude Code skills path).
2. Trigger it: "run the AI system", "measure actual ROI", "re-measure ROI",
   "cohort ROI", "AI health dashboard", "optimization loop", "1h/week quota",
   "Quarterly Business Review / QBR", "next department expansion", "client-as-reference",
   or in FR "mesurer le ROI réel", "re-mesurer le ROI", "tableau de bord de santé IA",
   "boucle d'optimisation", "revue trimestrielle", "département suivant".
3. One run writes `caio-run/` with 7 deliverables (ROI-Measurement-Model,
   Monitoring-Health-Spec, Optimization-Loop-Cadence, Weekly-Quota-Agenda,
   Quarterly-Business-Review, Expansion-And-Referral-Play, metadata.json).

## Chain position (Phase 5 — the closing, compounding step)

Runs AFTER `caio-enablement-and-transfer`. Reads:
- `caio-enablement/` (06-Ownership-Handover-Checklist + 08-Adoption-Tracker + 04-Validated-Use-Cases-Log — enablement)
- `company-ai-os/09-ROI-Governance-And-Risks.md` (projected ROI + governance — architect)
- `company-ai-os/05-Automation-Opportunity-Backlog.md` (scored backlog — architect)
- `caio-build/07-Monitoring-And-Instrumentation.md` (§5.8 telemetry wiring — implementation runbook)

Hands to:
- `caio-enterprise-workflow-architect` — the "Expand" verdict re-enters the architect for the next-wave audit (the chain loop closes)
- `creator-media-engine` — the public case study (with client consent)
- `agentic-systems-builder` / `agentik-skill-forge` — a "Build" verdict
- `/market-proposal` — the SOW for an approved expansion scope

It RUNS, MEASURES, OPTIMIZES, RETAINS, EXPANDS. It does NOT train (enablement) or build (implementation), and it does NOT write the CAIO's public marketing (creator-media-engine, with consent).

## Structure

- `SKILL.md` ............ operating protocol (boot, 7 phases, NSM, cohort ROI, re-measure, health, loop, quota, QBR, expansion)
- `references/01-roi-measurement-methodology.md` ... baseline → actual, cohort curves, prove/falsify the projection (mm-11)
- `references/02-monitoring-health-and-alerting.md` ... the operating dashboard + thresholds (reactive → piloted)
- `references/03-optimization-loop-and-quota.md` ... the compounding loop (mm-11) + the 1h/week quota economics (mm-08)
- `references/04-retention-expansion-referral.md` ... retention + land-and-expand + QBR (mm-08 NRR + mm-09)
- `assets/templates/` ... the 7 standardized client deliverables (fill-in, `{{placeholders}}`)
- `platforms/{claude,codex,gemini}.sh` ... activation adapters

## Adapters

```bash
bash platforms/claude.sh   # verifies SKILL.md + references/, prints the /trigger
bash platforms/codex.sh    # symlinks AGENTS.md -> SKILL.md
bash platforms/gemini.sh   # writes GEMINI.md activation pointer
```

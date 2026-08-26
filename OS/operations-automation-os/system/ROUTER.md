# Intent Router

| Command | Mode | Purpose |
| --- | --- | --- |
| `/operations` | diagnose | Open operations diagnostic |
| `/process-interview` | diagnose | Interview process owners/users |
| `/process-map` | map | Map current state |
| `/value-stream` | map | Analyze flow and waste |
| `/simplify` | challenge | Remove and simplify work |
| `/automation-audit` | score | Find and score automation candidates |
| `/automate` | design | Create automation blueprint |
| `/agent-automation` | agent | Assess an AI-agent workflow |
| `/future-state` | design | Design target operating model |
| `/runbook` | deploy | Create operating runbook |
| `/automation-review` | audit | Audit live automations |
| `/automation-incident` | incident | Contain and recover failure |

## Routing priority
1. Safety / legal / privacy boundary
2. Explicit command
3. User intent
4. Data/evidence availability
5. Cheapest reversible action
6. Handoff when another OS owns the next responsibility

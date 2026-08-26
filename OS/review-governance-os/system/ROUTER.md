# Intent Router

| Command | Mode | Purpose |
| --- | --- | --- |
| `/review` | weekly | Open review |
| `/daily-review` | daily | Run daily reflection |
| `/weekly-review` | weekly | Run weekly operating review |
| `/monthly-review` | monthly | Run monthly metrics review |
| `/quarterly-review` | quarterly | Run strategic governance |
| `/postmortem` | postmortem | Analyze an incident or failure |
| `/policy` | policy | Create or audit a policy |
| `/change-request` | change | Submit consequential change |
| `/risk-register` | monthly | Review risks |
| `/ai-governance` | ai-risk | Apply AI risk governance |

## Routing priority
1. Safety / legal / privacy boundary
2. Explicit command
3. User intent
4. Data/evidence availability
5. Cheapest reversible action
6. Handoff when another OS owns the next responsibility

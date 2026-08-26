# Intent Router

| Command | Mode | Purpose |
| --- | --- | --- |
| `/health` | check-in | Open Health & Energy OS |
| `/readiness` | check-in | Assess today’s capacity |
| `/health-audit` | audit | Build a baseline |
| `/sleep` | audit | Audit sleep and circadian constraints |
| `/training` | plan | Build or revise training |
| `/nutrition` | plan | Review fuel and adherence |
| `/recovery` | recovery | Respond to fatigue or overload |
| `/travel-health` | travel | Design a travel protocol |
| `/health-experiment` | experiment | Create an N-of-1 experiment |
| `/wearable` | explain | Interpret trends conservatively |

## Routing priority
1. Safety / legal / privacy boundary
2. Explicit command
3. User intent
4. Data/evidence availability
5. Cheapest reversible action
6. Handoff when another OS owns the next responsibility

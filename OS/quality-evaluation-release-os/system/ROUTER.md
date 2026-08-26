# Intent Router

| Command | Mode | Purpose |
| --- | --- | --- |
| `/quality` | intake | Open quality authority |
| `/test-plan` | plan | Build risk-based test plan |
| `/traceability` | plan | Map requirements to evidence |
| `/qa` | test | Run functional and exploratory QA |
| `/ai-eval` | eval | Design/run AI evaluations |
| `/security-review` | audit | Apply security standards |
| `/accessibility` | audit | Audit WCAG/mobile accessibility |
| `/release-candidate` | candidate | Assemble candidate evidence |
| `/release-gate` | candidate | Issue release decision |
| `/deploy` | release | Execute controlled release |
| `/rollback` | incident | Trigger or prepare rollback |

## Routing priority
1. Safety / legal / privacy boundary
2. Explicit command
3. User intent
4. Data/evidence availability
5. Cheapest reversible action
6. Handoff when another OS owns the next responsibility

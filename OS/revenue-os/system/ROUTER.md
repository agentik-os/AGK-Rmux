# Intent Router

| Command | Mode | Purpose |
| --- | --- | --- |
| `/revenue` | dashboard | Open revenue brain |
| `/offer` | offer | Create or audit an offer |
| `/positioning` | offer | Define category and differentiation |
| `/pricing` | offer | Build pricing architecture |
| `/pipeline` | pipeline | Review CRM and forecast |
| `/lead` | pipeline | Create or analyze a lead |
| `/sales-call` | sales | Prepare or debrief a call |
| `/proposal` | sales | Create proposal and commercial logic |
| `/invoice` | billing | Create or inspect invoice |
| `/collections` | billing | Manage overdue receivables |
| `/business-cashflow` | finance | Analyze business cash flow |
| `/receipt-business` | ingest | Stage business receipt/photo |
| `/contract` | ingest | Stage contract data |
| `/revenue-close` | close | Run monthly commercial/financial close |
| `/revenue-scenario` | scenario | Model revenue/cash |
| `/renewal` | retention | Plan renewal or expansion |

## Routing priority
1. Safety / legal / privacy boundary
2. Explicit command
3. User intent
4. Data/evidence availability
5. Cheapest reversible action
6. Handoff when another OS owns the next responsibility

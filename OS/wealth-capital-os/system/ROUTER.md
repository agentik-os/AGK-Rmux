# Intent Router

| Command | Mode | Purpose |
| --- | --- | --- |
| `/wealth` | dashboard | Open personal CFO dashboard |
| `/money-close` | close | Reconcile the month |
| `/cashflow` | dashboard | Analyze personal cash flow |
| `/saving` | plan | Create or revise savings plan |
| `/emergency-fund` | plan | Size and fund resilience reserve |
| `/debt` | decision | Choose a debt strategy |
| `/invest-policy` | plan | Create an investment policy statement |
| `/purchase` | decision | Evaluate a major purchase |
| `/money-scenario` | scenario | Model a financial scenario |
| `/receipt` | ingest | Stage a personal document or receipt |

## Routing priority
1. Safety / legal / privacy boundary
2. Explicit command
3. User intent
4. Data/evidence availability
5. Cheapest reversible action
6. Handoff when another OS owns the next responsibility

# Integration Wiring Guide — {{company}} (5.4, Composio)

> Connect the existing tools that expose an API via **the 6 critical connectors that actually work** — each proven by a **live-read test**. Not a 200-connector list. (Iron Law 5.)

- **Auth layer:** Composio (managed OAuth refresh) · API key in {{client_secret_manager}} (never the repo)
- **Wired by:** {{caio_or_subagent}}

---

## The 6 critical connectors

| # | Connector | System-of-record | Auth | Scopes (least-privilege) | Feeds | Rate limit | Cadence |
|---|---|---|---|---|---|---|---|
| 1 | {{CRM}} | {{SoR}} | {{OAuth/key}} | {{read scopes}} | {{CSO/CMO}} | {{limit}} | {{daily}} |
| 2 | {{ERP/Finance}} | {{SoR}} | {{OAuth/key}} | {{...}} | {{CFO}} | {{...}} | {{hourly}} |
| 3 | {{Marketing}} | {{SoR}} | {{OAuth/key}} | {{...}} | {{CMO}} | {{...}} | {{daily}} |
| 4 | {{Product analytics}} | {{SoR}} | {{OAuth/key}} | {{...}} | {{CIO-CTO/CDO}} | {{...}} | {{...}} |
| 5 | {{HR/HRIS}} | {{SoR}} | {{OAuth/key}} | {{...}} | {{CHRO}} | {{...}} | {{daily}} |
| 6 | {{Comms/Ops}} | {{SoR}} | {{OAuth/key}} | {{...}} | {{COO + report delivery}} | {{...}} | {{realtime}} |

_A 7th connector ONLY with a proven live-dashboard need:_ {{none / connector + the dashboard that needs it}}

---

## Live-read test (per connector — the only thing that makes it "wired")

| # | Connector | Real read executed | Real record returned? | rateRemaining OK? | integrations row written | Evidence (timestamp + redacted sample) |
|---|---|---|---|---|---|---|
| 1 | {{CRM}} | {{e.g. fetch 10 recent open deals}} | {{yes/no}} | {{yes/no}} | {{yes/no}} | {{ts + sample}} |
| 2 | {{ERP}} | {{e.g. fetch this month's invoices}} | {{yes/no}} | {{yes/no}} | {{yes/no}} | {{ts + sample}} |
| ... | | | | | | |

> "Connected" in Composio's UI is **not** "wired" until the live read returns a real record.

---

## Rate-limit handling

- [ ] Each connector's limit read BEFORE building the poll loop
- [ ] Cadence matched to freshness need (cache + Convex Scheduler, don't hammer)
- [ ] `rateRemaining` stored in `integrations` + surfaced in the monitoring view
- [ ] On 429: exponential backoff + panel marked "data delayed" (never stale-as-fresh)

---

## Integration acceptance (ship-gate for 5.4)

| Check | Pass? | Evidence |
|---|---|---|
| ≤6 connectors enabled (7th only with proven need) | {{y/n}} | {{...}} |
| Each enabled connector passed the live-read test | {{y/n}} | {{evidence row above}} |
| Auth least-privilege; tokens in secret manager not repo | {{y/n}} | {{grep clean}} |
| Rate limits read; cadence matched; backoff + "delayed" surface | {{y/n}} | {{...}} |
| Each connector maps to a SoR + a live dashboard | {{y/n}} | {{realization spec}} |
| No write scope beyond architect-authorized; sensitive writes via HITL | {{y/n}} | {{...}} |

**Verdict:** {{WIRED / INCOMPLETE}} · recorded in `08-Ship-Gate-Ledger.md`

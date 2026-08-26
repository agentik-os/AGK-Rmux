# Monitoring & Instrumentation Setup — {{company}} (5.8 + mm-11)

> Operational transparency built in **by default**: which agents execute, model costs, real usage, reports consulted. Plus the **mm-11 baseline** wired at t0 so ROI is measurable later. Built in provisioning STEP 6, **before** any feature. (Iron Law 6.)

---

## Part 1 — Observability (5.8)

The monitoring view reads `agentRuns`, `costEvents`, `reports`, `integrations`, `alerts`:

| Panel | Source table | Live? | Notes |
|---|---|---|---|
| Agent activity (who ran, status, errors, confidence) | `agentRuns` | {{y/n}} | {{...}} |
| Model cost ($ + tokens by agent/seat/day vs budget) | `costEvents` | {{y/n}} | surfaced to CFO + CIO/CTO |
| Real usage (used vs built-but-idle) | `baselineEvents(usage)` | {{y/n}} | {{...}} |
| Reports consulted (shipped + opened) | `reports` + receipts | {{y/n}} | unread report = leak |
| Integration health (last live read, rateRemaining, auth) | `integrations` | {{y/n}} | {{...}} |
| Errors / failures (owned, not hidden) | `agentRuns.error`, `alerts` | {{y/n}} | downstream panel marked "delayed" |

**Rules**
- [ ] Monitoring view live BEFORE any micro-SaaS feature (STEP 6)
- [ ] Model cost visible to CFO (no hidden spend)
- [ ] Failed runs owned + surfaced; no stale-as-fresh numbers (L1)
- [ ] Confidence travels from agent → panel

---

## Part 2 — Baseline instrumentation (mm-11 — instrument-for-baseline slice)

> Three clean events per dashboard. You build the **instrument**, not the verdict. `caio-run-and-optimize` computes ROI later.

| Seat | North-Star event (NSM) | Cost/usage event | Value-delivered event | Firing since t0? | t0 |
|---|---|---|---|---|---|
| {{seat}} | {{architect success metric}} | {{model$+tokens+run}} | {{workflow outcome}} | {{y/n}} | {{go-live ts}} |

**No-vanity-metric gate (mm-11 — the North-Star test):** for each NSM, "if it doubled, is the client objectively better off?"

| Candidate NSM | Doubling = better off? | Verdict |
|---|---|---|
| {{"dashboard opened"}} | {{no — climbs while idle}} | REJECTED (vanity) |
| {{"brief shipped on time + read"}} | {{yes}} | ACCEPTED |
| {{"agent runs"}} | {{no — a cost, not value}} | REJECTED (use as usage event) |

**Pre-build manual baseline (if known):** {{e.g. report cycle = 12h/week before go-live}} → recorded so Phase 5 compares against the real prior state.

**Handoff to `caio-run-and-optimize`:** t0 + the 3 events per dashboard + the architect's projection in `09-ROI-Governance-And-Risks.md` → "compute the delta." (Recorded in `metadata.json`.)

---

## Monitoring acceptance

| Check | Pass? |
|---|---|
| Monitoring view live before any feature (STEP 6) | {{y/n}} |
| Agent runs, cost, usage, reports, integration health all visible | {{y/n}} |
| Model cost surfaced to CFO + CIO/CTO | {{y/n}} |
| Failed runs owned; no stale-as-fresh | {{y/n}} |
| 3 baseline events per dashboard firing since t0 (mm-11) | {{y/n}} |
| t0 recorded in metadata.json | {{y/n}} |

**Verdict:** {{LIVE / INCOMPLETE}} · recorded in `08-Ship-Gate-Ledger.md`

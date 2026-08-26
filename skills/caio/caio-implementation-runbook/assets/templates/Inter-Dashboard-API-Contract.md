# Inter-Dashboard API Contract — {{company}} (5.3)

> The differentiator. Each dashboard **exposes** + **consumes** APIs on the shared Convex event bus, so a metric on one C-Level's instrument raises an alert on another's. The system runs as **one interconnected organism**. (Iron Law 4.)

- **Bus:** Convex `metrics` (exposed) + `alerts` (consumed) tables · reactive, typed — not a webhook web
- **Author:** {{caio_name}} · **Version:** {{version}}

---

## Exposed metrics (per seat — what each dashboard PUBLISHES)

| Seat | metricId | Value shape | confidence | sourceUrl | Cadence |
|---|---|---|---|---|---|
| {{seat}} | {{metricId}} | {{shape}} | {{exact / 0..1}} | {{always present}} | {{realtime/scheduled}} |

---

## The contract map (every cross-dashboard rule)

| # | Source · metric | Trigger condition (threshold in-contract) | → Target · alert | Payload (incl. sourceUrl+confidence) | HITL approver | Tested end-to-end? |
|---|---|---|---|---|---|---|
| 1 | {{seat·metricId}} | {{condition}} | {{seat·alertId}} | {{fields}} | {{role/none}} | {{yes — evidence / no}} |
| 2 | {{...}} | {{...}} | {{...}} | {{...}} | {{...}} | {{...}} |

---

## Implementation pattern (typed event + reactive subscriber rule)

```
// EXPOSE
metrics.insert({ seatKey:"{{src}}", metricId:"{{metric}}", value:{{...}}, ts:now,
                 confidence:"{{exact|0..1}}", sourceUrl:"{{link}}" })

// SUBSCRIBE (declared in this contract, evaluated reactively)
ON  metrics WHERE seatKey="{{src}}" AND metricId="{{metric}}" AND {{condition}}
RAISE alerts.insert({ fromSeat:"{{src}}", toSeat:"{{dst}}", alertId:"{{alert}}",
                      payload:{{...}}, threshold:"{{...}}",
                      hitlRequired:{{true|false}},
                      status: hitlRequired ? "pending_approval" : "raised", ts:now })
```

---

## Federation safety rules

- [ ] Sensitive alerts (financial/headcount/customer/regulated) are `hitlRequired` — never auto-fire (Iron Law 9)
- [ ] Every alert payload carries `sourceUrl` + `confidence` (Iron Law 6)
- [ ] Thresholds stored in-contract + surfaced in the governance view (sponsor-changeable, not hard-coded)
- [ ] Debounce/throttle present (e.g. {{1 alert per account per day}}) — no alert storms
- [ ] No A→B→A loops in the map

---

## Federation acceptance (ship-gate for 5.3)

| Check | Pass? | Evidence |
|---|---|---|
| Every map row implemented (rule on the bus) | {{y/n}} | {{...}} |
| ≥1 row tested end-to-end on real data | {{y/n}} | {{the real metric→alert that fired}} |
| Every sensitive alert is hitlRequired + renders approval | {{y/n}} | {{...}} |
| Every payload carries sourceUrl + confidence | {{y/n}} | {{...}} |
| Thresholds in-contract + governance-visible | {{y/n}} | {{...}} |
| Debounce present; no storm in load test | {{y/n}} | {{...}} |
| No A→B→A loop | {{y/n}} | {{...}} |

**Verdict:** {{SHIPPED / BLOCKED}} · recorded in `08-Ship-Gate-Ledger.md`

---

## Worked example (reference)

```
COO.fulfillment_sla_breach (severity:high, 31h late, $84k ARR account, sourceUrl:<ticket>)
  → ALERT cfo.cash_impact_alert { costOfDelay, accountArr, ticket }  [HITL: CFO review]
  → CFO dashboard shows pending alert WITH ticket link + cost-of-delay + confidence
  → CFO approves/dismisses (approvals row) — financial, so NO auto-act
  → baselineEvents records the value-delivered event (mm-11)
```

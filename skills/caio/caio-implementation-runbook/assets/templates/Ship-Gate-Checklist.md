# Ship-Gate Ledger — {{company}}

> Each deliverable goes live ONLY when its acceptance test passes — on **real data**, with a clean console. Value in week 1, not a POC. Acceptance criteria are pulled **verbatim from the architect's `07-Dashboard-Feature-Specs.md`** — never invented here. (Iron Law 7 + R-RUBRIC.)

---

## Ledger (one row per deliverable)

| Deliverable | Acceptance criterion (verbatim from architect) | Source F-XXX | Run date | Real data? | /omg-acceptance | Anti-black-box | 3 baseline events | Adversarial (2-of-3) | Verdict | Evidence |
|---|---|---|---|---|---|---|---|---|---|---|
| Server provision | (provision acceptance, reference 02 §6) | — | {{date}} | n/a | n/a | n/a | n/a | {{pass/fail}} | {{PROVISIONED}} | {{...}} |
| {{CFO dashboard}} | "{{criterion}}" | {{F-XXX}} | {{date}} | {{y/n}} | {{green/red}} | {{y/n}} | {{y/n}} | {{pass/fail}} | {{SHIPPED/RED}} | {{log}} |
| {{Federation rule #1}} | (federation acceptance, reference 03 §B.5) | — | {{date}} | {{y/n}} | n/a | {{y/n}} | n/a | {{pass/fail}} | {{SHIPPED/RED}} | {{the metric→alert}} |
| {{Connector: CRM}} | (live-read, reference 04 §A.4) | — | {{date}} | {{y/n}} | n/a | n/a | n/a | {{pass/fail}} | {{WIRED/RED}} | {{sample}} |
| {{Report R-CFO-WEEKLY}} | (report acceptance, reference 04 §B.5) | — | {{date}} | {{y/n}} | n/a | {{y/n}} | n/a | {{pass/fail}} | {{SHIPPED/RED}} | {{receipt}} |
| {{Monitoring}} | (monitoring acceptance, reference 05 §B.3) | — | {{date}} | n/a | n/a | n/a | {{y/n}} | {{pass/fail}} | {{LIVE/RED}} | {{...}} |

---

## Per-deliverable gate procedure (reference 05 §C)

- [ ] **1. Criteria** — copied verbatim from `07-Dashboard-Feature-Specs.md` (not invented)
- [ ] **2. Real data** — runs against the client's REAL connected data (L1)
- [ ] **3. Acceptance** — `/omg-acceptance`: every route 200+render · console owned · golden path with a real persisted write
- [ ] **4. Anti-black-box** — every number shows source + freshness + confidence (Iron Law 6)
- [ ] **5. Baseline** — the 3 mm-11 events fire (reference 05 §A)
- [ ] **6. HITL** — every sensitive action renders an approval step (Iron Law 9)
- [ ] **7. Verdict** — GREEN ships now; RED returns to builder (no "mostly working")
- [ ] **8. Record** — this ledger: criteria + date + verdict + evidence (R-CITE)

---

## "It builds ≠ it works" reminder (L1 / R-PROD)

A green compile with a red console is **not** shipped. The gate observes the **real golden path on real data** + the **browser console + network**. A 500 on the CFO's actual data is RED, no matter how good the demo looked.

---

## Go-live summary

- **Deliverables shipped (week-by-week):** {{week 1: X · week 2: Y · ...}}
- **First value delivered (week 1):** {{the deliverable + the real outcome}}
- **All gates green for go-live?** {{yes/no}} → hand to `caio-enablement-and-transfer`

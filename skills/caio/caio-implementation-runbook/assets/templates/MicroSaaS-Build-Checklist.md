# Micro-SaaS Build Checklist — {{seat}} dashboard · {{company}}

> One function-specific micro-SaaS per C-Level, built for **this person's real job** — not a recycled template. (Iron Law 3.)

- **Seat:** {{CIO/CTO | CMO | CFO | CDO | COO | CHRO | CSO}}
- **The instrument (not "a dashboard"):** {{e.g. "finance + runway instrument"}}
- **The one job a generic dashboard gets wrong:** {{real_job_line from realization spec}}
- **Builder:** {{caio_or_subagent}} · **Build dossier:** `builds/{{seat}}-dashboard.md`

---

## Build steps

- [ ] **1. SCOPE** — views from `07-Dashboard-Feature-Specs.md` (P1 first): {{views}} · acceptance criteria EXIST: {{yes/no}}
- [ ] **2. DATA** — system(s)-of-record: {{SoR}} · connector(s) live-read PASSED: {{yes/no}} (reference 04)
- [ ] **3. SCHEMA** — seat metrics added to shared `metrics` table (each: value, ts, confidence, sourceUrl)
- [ ] **4. AGENTS** — F-XXX agents DISPATCHED to agentic-systems-builder (NOT re-implemented): {{F-XXX list}} · agentRuns+costEvents wired
- [ ] **5. VIEWS** — Next.js views in `app/({{seat}})`, Clerk-gated · every number shows source+freshness+confidence (Iron Law 6)
- [ ] **6. EXPOSE** — key metrics published to the federation bus: {{exposed metricIds}}
- [ ] **7. CONSUME** — alerts subscribed from other seats: {{consumed alertIds}} · HITL where required
- [ ] **8. INSTRUMENT** — 3 baseline events wired (NSM / cost / value); t0 = go-live (mm-11, reference 05 §A)
- [ ] **9. REPORT** — automated report wired: {{report_id}} (reference 04 §B)
- [ ] **10. SHIP-GATE** — `/omg-acceptance` on real data → GREEN ships, RED back to builder (reference 05 §C)

---

## Anti-black-box per-panel check (Iron Law 6)

For each panel asserting a number:

| Panel | Source (sourceUrl) | Freshness | Confidence | Error-state handled |
|---|---|---|---|---|
| {{panel}} | {{link}} | {{last read}} | {{exact / 0..1}} | {{yes/no}} |

---

## HITL gates inside this dashboard (Iron Law 9)

| Sensitive action | Architect-matrix class | Approver role (Clerk) | Renders as approval step? |
|---|---|---|---|
| {{action}} | {{financial/headcount/customer/regulated}} | {{role}} | {{yes/no}} |

---

## Ship-gate verdict

- **Acceptance criterion (verbatim from architect):** "{{criterion}}"
- **Run on real data:** {{yes/no}} · **/omg-acceptance:** {{green/red}} · **Console clean:** {{yes/no}}
- **3 baseline events firing:** {{yes/no}} · **Adversarial verify (2-of-3):** {{pass/fail}}
- **Verdict:** {{SHIPPED (date) / BACK-TO-BUILDER}} · evidence: {{log/screenshot}}
- Recorded in `08-Ship-Gate-Ledger.md`.

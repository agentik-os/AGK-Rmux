# Automated-Report Spec — {{report_id}} · {{company}} (5.5)

> The report **falls out automatically** so analysts analyze instead of copy-pasting. Frequency, format, indicators, recipients — specified once, then it runs itself.

---

## Spec sheet

| Field | Value |
|---|---|
| **Report ID** | {{R-CFO-WEEKLY-CASH}} |
| **Owner seat** | {{cfo}} |
| **Audience** | {{exact recipients}} via {{channel: Slack #c-level / email / Notion DB}} |
| **Frequency** | {{cron — e.g. Mon 07:00 {{timezone}}}} |
| **Format** | {{1-page markdown + Notion row + PDF via `omega pdf`}} |
| **Scheduler** | {{Convex Scheduler / Trigger.dev}} |
| **HITL before send?** | {{yes — {{role}} reviews / no}} |

---

## Indicators (each with source + notable-threshold)

| Indicator | Computed by (Convex query) | sourceUrl | Threshold that makes it notable |
|---|---|---|---|
| {{cash position}} | {{query}} | {{Stripe link}} | {{<X / WoW drop >Y%}} |
| {{margin}} | {{query}} | {{ERP link}} | {{...}} |
| {{forecast vs actual}} | {{query}} | {{...}} | {{variance >Z%}} |

> **Numbers-are-cited rule (L1):** the model **never does math**. Every figure is computed in a Convex action and cited with its `sourceUrl`. The LLM drafts only the **narrative** ("cash down 4% WoW, driven by two slipped renewals"). An uncited number is refused.

---

## Narrative (LLM-drafted, numbers cited)

- **Draft scope:** {{"what changed + why", referencing cited figures only}}
- **Tone / length:** {{audience-shaped — CEO=1 page, analyst=drill-down link}} (mm-04: clear beats clever)

---

## Delivery proof

- [ ] Each run writes `reports{lastRunAt, status}` + a delivery receipt
- [ ] A missed/failed run is visible (monitoring alert) — silence is not allowed (mm-04)
- [ ] Read-receipt tracked where possible (a built-but-unread report is a leak)

---

## Report acceptance (ship-gate for 5.5)

| Check | Pass? | Evidence |
|---|---|---|
| Frequency, format, indicators, recipients all specified | {{y/n}} | {{this sheet}} |
| Every number query-computed + cited; model does no math | {{y/n}} | {{sample report}} |
| Ran on schedule on real data + reached recipients | {{y/n}} | {{delivery receipt}} |
| Missed/failed run visible | {{y/n}} | {{monitoring}} |
| Sensitive report passes HITL before send | {{y/n}} | {{approval row}} |
| Audience-shaped; PDFs via `omega pdf` (R-PDF) | {{y/n}} | {{...}} |

**Verdict:** {{SHIPPED (date) / BLOCKED}} · recorded in `08-Ship-Gate-Ledger.md`
**Value recovered (for mm-11 baseline):** {{e.g. 12h/week → 5 min read}}

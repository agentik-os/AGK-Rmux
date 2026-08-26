# ROI Measurement Model — {{client_name}}

> Baseline → actual, by cohort, vs the architect's projection (09-ROI). Every number on this page traces to a telemetry event or a receipt. No figure is estimated. (CAIO Run & Optimize · Iron Law 1)

- **Prepared by:** {{caio_name}}
- **Measurement window:** {{start_date}} → {{end_date}}
- **Sponsor:** {{sponsor_name}}
- **System go-live (wave 1):** {{wave1_date}}

---

## 1. System North Star Metric (mm-11)

- **NSM:** {{nsm_name}} — _(value-received, not activity)_
- **Telemetry source:** {{nsm_event_source}}
- **Vanity test ("if it doubles, are people better off?"):** {{pass_with_reason}}
- **This window:** {{nsm_value}} ({{nsm_trend}} vs last window)

---

## 2. Baseline reconciliation

| Workflow | Baseline hrs/wk | People × role | Loaded $/hr | Baseline $/yr | Source | Status |
|---|---|---|---|---|---|---|
| {{wf_id}} | {{hrs}} | {{n}} × {{role}} | {{rate}} | {{baseline_yr}} | {{baseline_source}} | {{verified / baseline-unverified / no-baseline}} |

> Loaded cost = salary × {{loading_factor}} ÷ 2080. Client-provided figure used where available: {{yes_no}}.

---

## 3. Actuals (measured from telemetry/receipts)

For each workflow, an atomic ROI receipt:

### {{wf_id}} — {{wf_name}}
- **Cohort (go-live):** {{YYYY_MM}}
- **Actual hrs/wk now:** {{actual_hrs}} _(baseline {{baseline_hrs}} − residual manual {{residual_hrs}})_ — source: {{event_source}} + {{timesheet_source}}
- **Cost avoided:** {{cost_avoided}} — receipt: {{invoice_or_payroll_line}}
- **Value created:** {{value_created}} — source: {{crm_telemetry}} (directional attribution)
- **Adoption:** {{active}}/{{target}} users ({{adoption_pct}}) — source: {{usage_telemetry}}
- **Decisions improved:** {{decision_count}} human-confirmed — source: {{decision_artefact}}
- **System cost:** model {{model_spend}} + infra {{infra}} + ops {{ops}} = {{system_cost}} — meter: {{cost_meter}}
- **Net annual benefit (durable):** {{net_yr}}

_(repeat per workflow)_

---

## 4. Cohort ROI — savings-retention curve

| Cohort (go-live) | Workflows | M1 saving | M3 saving | M6 saving | Shape | Durable saving |
|---|---|---|---|---|---|---|
| {{YYYY_MM}} | {{wf_list}} | {{m1}} | {{m3}} | {{m6}} | {{plateau / decay}} | {{durable}} |

**Cohort read:** {{which waves hold, which leak, is later-wave retention better than wave-1}}

---

## 5. The re-measure — actual vs projected

| Workflow | Projected $/yr (09-ROI) | Actual $/yr (durable) | Ratio | Verdict | Why the delta |
|---|---|---|---|---|---|
| {{wf_id}} | {{projected}} | {{actual}} | {{ratio_pct}} | {{Proven / Partial / Falsified}} | {{cause}} |

### Aggregate
- **Projected annual benefit (architect):** {{total_projected}}
- **Actual durable annual benefit (measured):** {{total_actual}}
- **Realization rate:** {{realization_pct}}
- **System cost (annual):** {{total_system_cost}} → **Net:** {{net}} → **Payback to date:** {{payback_months}} mo
- **Honest verdict (CFO-trustable, one line):** {{verdict_line}}

> Falsifications are kept and explained, never smoothed into greens. A projection the telemetry contradicts is a finding, not a failure. (mm-11 · L2)

---

## 6. Next-loop actions (per workflow)

| Workflow | Verdict | Next action | Hypothesis (if Tweak/Build) |
|---|---|---|---|
| {{wf_id}} | {{Proven/Partial/Falsified}} | {{Tweak / Build / Expand / Kill}} | {{Because… I believe… I'll know if…}} |

---
*Every number above is sourced. Where the architect's projection and the telemetry disagree, the telemetry wins.*

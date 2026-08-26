# Architecture-Realization Spec — {{company}}

> **The design gate.** Translates the architect's generic `company-ai-os/` blueprint into the offer's centralized federated topology. **Approved by the sponsor before any build starts.** (Iron Law 1: no realization spec, no build.)

- **Realized from:** `company-ai-os/` (architect) — version {{blueprint_version}}
- **Author (CAIO):** {{caio_name}}
- **Spec version:** {{spec_version}}
- **Status:** {{draft | in-review | APPROVED}}
- **Executive sponsor:** {{sponsor_name}} ({{sponsor_title}})
- **Approval date:** {{YYYY-MM-DD or _(pending)_}}
- **Approval conditions:** {{conditions_or_none}}

---

## 1. The value-prop the sponsor signs (mm-04)

> For **{{c_suite}}** who **{{need_legible_and_automated}}**, this is the **centralized federated Company-AI-OS** that **{{turns_seven_blind_tools_into_one_organism}}**, unlike **{{status_quo_disconnected_dashboards}}** that **{{keeps_each_c_level_blind}}**.

---

## 2. Seat map (build only seats that exist — Iron Law 3)

| Seat | Exists? | If no → absorbed by | Backlog opportunities it owns | F-XXX agents | The one job a generic dashboard gets wrong |
|---|---|---|---|---|---|
| CIO/CTO | {{yes/no}} | {{seat}} | {{opps}} | {{F-XXX}} | {{real_job_line}} |
| CMO | {{yes/no}} | {{seat}} | {{opps}} | {{F-XXX}} | {{real_job_line}} |
| CFO | {{yes/no}} | {{seat}} | {{opps}} | {{F-XXX}} | {{real_job_line}} |
| CDO | {{yes/no}} | {{seat}} | {{opps}} | {{F-XXX}} | {{real_job_line}} |
| COO | {{yes/no}} | {{seat}} | {{opps}} | {{F-XXX}} | {{real_job_line}} |
| CHRO | {{yes/no}} | {{seat}} | {{opps}} | {{F-XXX}} | {{real_job_line}} |
| CSO | {{yes/no}} | {{seat}} | {{opps}} | {{F-XXX}} | {{real_job_line}} |

---

## 3. Federation contract map (5.3 — the differentiator)

| # | Source seat · metric | Trigger condition | → Target seat · alert | Payload | HITL? |
|---|---|---|---|---|---|
| 1 | {{seat}} · {{metricId}} | {{threshold}} | {{seat}} · {{alertId}} | {{fields incl. sourceUrl+confidence}} | {{approver_role or none}} |
| 2 | {{...}} | {{...}} | {{...}} | {{...}} | {{...}} |

_At least one row must be implemented and tested end-to-end during the build (reference 03 §B)._

---

## 4. Composio topology (5.4 — the 6 critical connectors)

| Connector | System-of-record | Auth method | Feeds dashboard(s) | Data returned |
|---|---|---|---|---|
| {{CRM}} | {{SoR}} | {{OAuth/key}} | {{seats}} | {{data}} |
| {{ERP}} | {{SoR}} | {{OAuth/key}} | {{seats}} | {{data}} |
| {{Marketing}} | {{SoR}} | {{OAuth/key}} | {{seats}} | {{data}} |
| {{Analytics}} | {{SoR}} | {{OAuth/key}} | {{seats}} | {{data}} |
| {{HR}} | {{SoR}} | {{OAuth/key}} | {{seats}} | {{data}} |
| {{Comms/Ops}} | {{SoR}} | {{OAuth/key}} | {{seats}} | {{data}} |

_A 7th connector only with a proven live-dashboard need._

---

## 5. Server shape (5.1)

- **Region:** {{region}} · **Residency posture:** {{GDPR/SOC2/HIPAA/none}}
- **Stack (justified per layer):** Next.js {{why}} · Convex {{why}} · Clerk {{why}} · Stripe {{why/optional}} · Composio {{why}} · Claude Code SDK {{why}}
- **Adaptation:** {{existing ERP/CRM/warehouse/air-gapped notes}}
- **Export path (client owns + can leave):** {{data export + repo ownership + secrets handover + redeploy runbook ref}}

---

## 6. Instrumentation plan (5.8 + mm-11)

Per dashboard, the **three baseline events** + t0:

| Seat | North-Star event (NSM) | Cost/usage event | Value-delivered event | t0 |
|---|---|---|---|---|
| {{seat}} | {{architect success metric}} | {{model$+tokens+run}} | {{workflow outcome}} | {{go-live ts}} |

_t0 recorded in metadata.json. The pre-build manual baseline (if known): {{e.g. 12h/week}}._

---

## 7. Ship-gate map (acceptance pulled VERBATIM from the architect)

| Deliverable | Acceptance criterion (verbatim from 07-Dashboard-Feature-Specs.md) | Source feature |
|---|---|---|
| {{micro-SaaS/report/rule}} | "{{criterion}}" | {{F-XXX}} |

---

## 8. Realization checklist (before "APPROVED")

- [ ] Every built seat exists in the real org (rollup-confirmed)
- [ ] Every built seat has its "real job a generic dashboard gets wrong" line
- [ ] Non-existent seats folded into an existing seat (recorded)
- [ ] Federation map has ≥1 real exposes→consumes rule with threshold + payload
- [ ] Every sensitive cross-dashboard alert routes through the HITL matrix
- [ ] The ≤6 connectors mapped to system-of-record + dashboard
- [ ] Server shape states region + residency + export path
- [ ] Each stack layer carries a one-line justification
- [ ] Per dashboard, the 3 baseline events named + t0 located (mm-11)
- [ ] Every ship-gate criterion copied verbatim from the architect's feature spec
- [ ] Spec sponsor-approved with a date in the build log (mm-04)

---

_Approved → unlocks the build. Recorded in `00-Build-Log.md`._

# Monitoring & Health Spec — {{client_name}}

> The operating dashboard + alert thresholds on top of the runbook §5.8 telemetry wiring. Turns a reactive company into a piloted one. Every threshold has an owner and a runbook line. (CAIO Run & Optimize · Iron Law 6)

- **Prepared by:** {{caio_name}}
- **Telemetry wiring source:** {{path_to_caio-build/07-Monitoring-And-Instrumentation.md}} (runbook §5.8)
- **System owner (internal):** {{system_owner}}

---

## 1. The five health dimensions

### Liveness — "is it running?"
| Metric | Source (§5.8) | Threshold (warn / page) | Owner | Runbook line |
|---|---|---|---|---|
| Scheduled-agent runs | {{job_log}} | {{missed run}} | {{owner}} | {{action}} |
| Run success rate (24h) | {{agentRuns}} | {{< 95% / < 80%}} | {{owner}} | {{action}} |
| Last-run timestamp | {{heartbeat}} | {{interval × 1.5}} | {{owner}} | {{action}} |
| End-to-end latency | {{run_duration}} | {{p95 > 2× baseline}} | {{eng}} | {{action}} |

### Cost — "is it economical?" (mm-08 margin)
| Metric | Source | Threshold | Owner | Runbook line |
|---|---|---|---|---|
| Model spend vs budget | {{cost_meter}} | {{forecast > 80% / > 100% cap}} | {{CAIO}} | {{action}} |
| Cost per NSM unit | {{cost ÷ nsm}} | {{> 30% WoW rise}} | {{CAIO}} | {{action}} |
| Cost per workflow | {{tagged_meter}} | {{> tier budget}} | {{owner}} | {{action}} |
| Token-cost share of value | {{cost ÷ value}} | {{> 25-30%}} | {{CAIO}} | {{action}} |

### Usage / adoption — "are people using it?" (leading leak indicator)
| Metric | Source | Threshold | Owner | Runbook line |
|---|---|---|---|---|
| WAU per feature | {{analytics}} | {{> 30% drop vs 4-wk avg}} | {{owner}} | {{root-cause before decay}} |
| Runs per workflow | {{agentRuns}} | {{decline 2+ wks}} | {{owner}} | {{action}} |
| Active vs target users | {{telemetry ÷ roster}} | {{< 50% after ramp}} | {{CAIO}} | {{re-onboard → enablement}} |
| Dashboard opens | {{dash_analytics}} | {{health view unopened}} | {{CAIO}} | {{simplify or retire (mm-11)}} |

### Quality — "is output good and still reviewed?"
| Metric | Source | Threshold | Owner | Runbook line |
|---|---|---|---|---|
| Error / exception rate | {{run_logs}} | {{spike vs baseline}} | {{eng}} | {{hotfix / rollback}} |
| HITL approval rate | {{approval_queue}} | {{falls (worse) OR ~100% (rubber-stamp)}} | {{CAIO}} | {{fix agent / sample-audit}} |
| Drift signal | {{distribution_monitor}} | {{shift vs reference}} | {{eng}} | {{re-prompt / re-eval}} |
| Customer-facing escapes | {{complaint_tag}} | {{any wrong sensitive output}} | {{CAIO}} | {{incident review; tighten HITL}} |

### Value — "is it delivering, and is anyone steering by it?"
| Metric | Source | Threshold | Owner | Runbook line |
|---|---|---|---|---|
| System NSM trend | {{nsm_event}} | {{flat/down 2 wks}} | {{CAIO}} | {{open loop hypothesis}} |
| Decisions improved | {{decision_artefacts}} | {{declining}} | {{CAIO}} | {{sample decisions}} |
| Reports consulted | {{dash_analytics}} | {{value view unopened by sponsor}} | {{CAIO}} | {{fix view or accept QBR steers}} |

---

## 2. Dashboard layout

### Operator one-screen (CAIO + owner, weekly)
1. {{NSM + trend}}
2. {{Cohort savings-retention mini-table}}
3. {{Cost vs budget + cost-per-NSM-unit}}
4. {{Liveness strip — every agent green/amber/red}}
5. {{Open alerts by severity, owner, age}}

### Executive view (sponsor, monthly / QBR)
1. {{NSM trend (quarter)}}
2. {{Realization rate actual vs projected}}
3. {{Hours saved / cost avoided / decisions improved to date}}
4. {{Adoption across departments}}
5. {{The one risk needing an executive decision}}

---

## 3. Reactive → piloted (the transition this spec delivers)

| Reactive (before) | Piloted (after) |
|---|---|
| {{learns of dead agent from a complaint}} | {{liveness alert → owner re-triggers first}} |
| {{cost overrun on the invoice}} | {{80% forecast warn → capped mid-month}} |
| {{abandoned feature found at QBR}} | {{WAU-drop alert → leak fixed weeks earlier}} |

**Validation:** thresholds must catch ≥ 1 real issue BEFORE the client complains. Last validated: {{date_and_issue_caught}}.

---

## 4. Threshold hygiene
- Noisy/known sources suppressed: {{list}}
- Thresholds reviewed in the monthly loop: {{date}}
- Thresholds that never fired (too loose?) / fired with no action (too tight or mis-owned?): {{notes}}

---
*Plumbing makes events; thresholds make a piloted company; an owner makes an alert.*

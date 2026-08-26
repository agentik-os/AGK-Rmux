# CAIO Run & Optimize

> A delivered system is not a finished system. Measured, it either compounds or it leaks. This skill keeps it compounding — on real numbers, not on the deck.

> Phase 5 of the CAIO accompaniment chain: RUN the live Company AI OS, MEASURE actual ROI from telemetry, MONITOR health, OPTIMIZE in a loop, operate the deliberately-light 1h/week quota, and EXPAND the engagement back into the architect for the next wave.

Built by [Agentik OS](https://agentik-os.com). The closing, compounding step of the **Chief AI Officer On Demand** accompaniment suite. Composes with [caio-enablement-and-transfer](https://skills.agentik-os.com/caio-enablement-and-transfer) (upstream), [caio-enterprise-workflow-architect](https://skills.agentik-os.com/caio-enterprise-workflow-architect) (upstream projection + downstream next-wave loop), [caio-implementation-runbook](https://skills.agentik-os.com/caio-implementation-runbook) (telemetry wiring), [agentic-systems-builder](https://skills.agentik-os.com/agentic-systems-builder), [agentik-skill-forge](https://skills.agentik-os.com/agentik-skill-forge), [creator-media-engine](https://skills.agentik-os.com/creator-media-engine), and [market-proposal](https://skills.agentik-os.com/market-proposal).

---

## What it produces

`caio-run/` directory with 7 deliverables:

1. **ROI-Measurement-Model.md** — baseline → actual, by cohort, vs the architect's projection (09-ROI). Every number cited to telemetry or a receipt. Proven / Partial / Falsified verdicts and a realization rate.
2. **Monitoring-Health-Spec.md** — the operating dashboard (liveness, cost, usage, quality, value) + alert thresholds with owners and runbook lines. Turns a reactive company into a piloted one.
3. **Optimization-Loop-Cadence.md** — the weekly one-screen read + the monthly re-score of the architect backlog against ACTUAL data; the next-best improvement as a falsifiable hypothesis.
4. **Weekly-Quota-Agenda.md** — the deliberately-light 1h/week strategic-quota agenda template + the overage-is-a-mini-SOW boundary.
5. **Quarterly-Business-Review.md** — the C-Level QBR: NSM trend, cohort ROI actual vs projected, cost, adoption, decisions improved, the honest red, and a renew/expand decision.
6. **Expansion-And-Referral-Play.md** — renewal + next-department land-and-expand, NRR logic, client-as-internal-reference, and the public-case-study handoff to creator-media-engine (with consent).
7. **metadata.json** — machine-readable header: NSM, cohorts, roi_actual_vs_projected, health, optimization backlog, quota, retention/expansion.

## When to use it

After the system is live and the team is autonomous (post caio-enablement-and-transfer). Use it to: measure actual post-go-live ROI, re-measure against the projection, monitor system health, run the weekly/monthly optimization loop, operate the 1h/week strategic quota, prepare a Quarterly Business Review, or drive retention + the next-department expansion.

**Not** for training/adoption (that's `caio-enablement-and-transfer`), **not** for building agents (that's `caio-implementation-runbook` / `agentic-systems-builder`), and **not** for the CAIO's own public case-study marketing (delegate to `creator-media-engine`).

## Chain position

```
1. caio-ai-readiness-assessment   (pre-sign go/no-go)
   → /market-proposal (signed SOW)
2. caio-discovery-interview        (Phase 1 — per-person immersion)
3. caio-enterprise-workflow-architect (Phase 1 — company-ai-os/ blueprint + backlog + ROI projection)
4. caio-implementation-runbook     (Phase 2 — realize the topology + build; wires telemetry §5.8)
5. caio-enablement-and-transfer    (Phase 3 adoption + Phase 4 transfer-to-autonomy; hands over adoption tracker)
6. caio-run-and-optimize  ← THIS   (Phase 5 — measure ROI, monitor, optimize, 1h/week quota, expand)
                                     └──── "Expand" verdict loops back to #3 (architect) for the next department
```

## The chain contract

| Direction | Contract |
|---|---|
| **Reads** | `caio-enablement/` (06-Ownership-Handover-Checklist + 08-Adoption-Tracker + 04-Validated-Use-Cases-Log, from enablement) · `company-ai-os/09-ROI` (projected ROI + governance) · `company-ai-os/05-backlog` (scored opportunities) · `caio-build/07-Monitoring-And-Instrumentation.md` (§5.8 telemetry wiring) · live telemetry, model-cost receipts, timesheets/invoices |
| **Writes** | `caio-run/` (7 deliverables) |
| **Hands to** | `caio-enterprise-workflow-architect` (next-wave audit — the loop closes) · `creator-media-engine` (public case study, with consent) · `agentic-systems-builder` / `agentik-skill-forge` (a "build" verdict) · `/market-proposal` (expansion SOW) |
| **Depends on** | a delivered, handed-over Company AI OS — refuses to fabricate a baseline if absent |

## The doctrine behind it (Marketing Mastery)

- **mm-11 (measure-loops-retention)** — the system's North Star Metric (value-received, vanity-guarded), cohort ROI measured against the architect's projection, the compounding optimization loop, and `retention > monetization > acquisition` applied to a delivered system: fix a leaking cohort before financing the next wave.
- **mm-08 (pricing/monetization)** — the 1h/week quota is light *by design* (price/format = positioning; a heavy retainer would contradict the autonomy delivered); the overage-is-a-mini-SOW boundary is mm-08's margin discipline; expansion is priced by scope-under-management with NRR > 100% of delivered value.
- **mm-09 (partnerships / network-effects)** — the satisfied client as an *internal* reference (alignment > size), the next-department expansion as a land-and-expand network effect inside one account, give-first before the intro ask, and the public case study delegated to creator-media-engine (with consent).

## The 10 Iron Laws

1. Every number from telemetry or a receipt — never imagination.
2. Always compare actual against the architect's projected ROI (09-ROI).
3. The system has ONE NSM, and it is value-received, not activity.
4. Measure ROI in cohorts, by go-live wave — read the savings-retention curve.
5. Retention before expansion before acquisition.
6. The dashboard exposes liveness, cost, usage, quality, value — with alert thresholds.
7. The 1h/week quota is light by design, not by accident.
8. Anything bigger than the hour is a scoped mini-engagement, not absorbed.
9. Human-in-the-loop on sensitive decisions persists after autonomy.
10. Expansion is the engagement's land-and-expand, not the CAIO's pipeline marketing.

## The re-measure (the engagement's word of honour)

Pull actuals from telemetry/receipts → lay them beside the architect's 09-ROI projection → verdict per workflow:
- **Proven** (≥80% of projected) — bank it.
- **Partial** (50-80%) — real value, optimistic projection; name the cause.
- **Falsified** (<50%) — the architect mis-scoped it; say so plainly and route it back into the loop.

A QBR with an honest red outsells one full of unverifiable greens, because the greens are believed.

## Installation

```bash
bash <(curl -sL https://skills.agentik-os.com/install) caio-run-and-optimize
```

Then in Claude Code:

```
/caio-run-and-optimize
```

## What it refuses

- ROI without a telemetry source or receipt
- Claiming the projection "held" without laying actual beside projected
- Calling agent-runs / tokens / reports-generated the NSM
- Averaging ROI instead of measuring by cohort
- Expanding to the next department while a wave-1 cohort decays
- Letting the 1h/week absorb a new build (mm-08 margin bomb)
- Dropping HITL on sensitive decisions because the team "got good"
- Writing the CAIO's public case-study marketing here
- Publishing any client reference without explicit consent
- A health dashboard with no thresholds or no owners

## Iron Test (90 days, then quarterly)

1. Every QBR number traceable to telemetry/receipt? 2. Projection proven OR honestly falsified? 3. Wave-1 cohorts still delivering (plateau, not decay)? 4. The loop shipped ≥1 measured improvement? 5. Alerts caught ≥1 issue before the client complained? 6. The QBR drove a renew/expand decision?

5+ of 6 = compounding. 12-month test: NRR > 100% of delivered value + the "expand" verdict looped to the architect for a 2nd department + a willing internal reference. Yes to all three = a self-compounding engagement.

## License

MIT.

---

*Version 1.0.0 :: a CAIO does not stop at delivery — a CAIO RUNS the system, proves the ROI on real numbers, keeps it compounding, and expands the engagement back into the architect for the next wave.*

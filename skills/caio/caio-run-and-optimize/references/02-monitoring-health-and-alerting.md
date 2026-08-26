# 02 — Monitoring, Health & Alerting (the operational dashboard that pilots the company)

Phase 3 of CAIO Run & Optimize. Outputs `caio-run/Monitoring-Health-Spec.md` and the `health` block of `metadata.json`.

The implementation runbook wired the raw telemetry (§5.8): events fire, a model-cost meter reads, run logs persist. That wiring is plumbing, not piloting. This phase SPECs the **operating dashboard + alert thresholds** on top of it — the layer that turns a *reactive* company (waits for a complaint, then panics) into a *piloted* one (gets an alert, acts on a runbook line, before anyone outside notices).

> A black-box agent with no dashboard is not enterprise-grade (architect Iron Law 8). A dashboard with no thresholds is not piloting — it is wallpaper. A threshold with no owner is not an alert — it is a notification nobody answers.

The mm-11 lens applies here too: the health dashboard is itself subject to the vanity-metric guard. If the screen counts "agent runs" and nobody opens it, you have built a vanity dashboard. Health is measured by *whether issues get caught before the client feels them*, not by how many charts render.

---

## Part A — The five health dimensions

Every Company AI OS is monitored across five dimensions. Each has: a **metric**, a **source** (which §5.8 event/meter), a **threshold**, an **owner**, and a **runbook line** (what the owner does when it trips).

### A.1 Liveness — "is the system actually running?"

The most basic failure: a scheduled agent silently stops and nobody notices for a week. Liveness catches it.

```
Metric                 | Source (§5.8)              | Threshold (alert)                  | Owner        | Runbook line
Scheduled-agent runs   | cron/job success log       | a scheduled run is missed          | system owner | check job logs; re-trigger; page eng if 2 consecutive
Run success rate (24h) | agentRuns table            | < 95% over rolling 24h             | system owner | inspect failing runs; classify transient vs systemic
Last-run timestamp     | per-agent heartbeat        | no run in expected interval × 1.5  | system owner | confirm scheduler alive; check upstream API health
End-to-end latency     | run duration percentiles   | p95 > 2× baseline                  | eng          | profile the slow step; check vendor/API latency
```

### A.2 Cost — "is it still economical?"

mm-08's margin discipline lives here. Cost that drifts up unwatched is how a profitable system quietly becomes a loss.

```
Metric                 | Source (§5.8)              | Threshold (alert)                  | Owner        | Runbook line
Model spend vs budget  | cost meter                 | forecast > 80% of monthly cap      | CAIO + owner | identify driver; cap/route to cheaper model; flag to loop
Cost per NSM unit      | cost meter ÷ NSM telemetry | > 30% week-over-week rise          | CAIO         | a workflow's economics degraded; open a loop ticket
Cost per workflow      | cost meter, tagged         | any workflow > its tier's budget   | owner        | inspect for a runaway power-user / loop; add cap/overage
Token-cost share       | cost ÷ value captured      | > ~25-30% of the value (mm-08)     | CAIO         | re-price/cap; the workflow is eating its own margin
```

### A.3 Usage / adoption — "are people actually using it?"

Adoption is the multiplier on all ROI (reference 01 §B.4). A drop in usage is the *leading* indicator of a cohort about to leak — it precedes the savings-decay by weeks.

```
Metric                 | Source (§5.8)              | Threshold (alert)                  | Owner        | Runbook line
WAU per feature        | product analytics          | > 30% drop vs trailing 4-week avg  | owner        | the cohort is starting to leak; root-cause before it decays
Runs per workflow      | agentRuns, tagged          | sustained decline 2+ weeks         | owner        | interview a user; is it broken, distrusted, or obsolete?
Active vs target users | telemetry ÷ target roster  | < 50% of target after ramp window  | CAIO         | re-onboard (route to enablement); not an acquisition problem
Dashboard opens        | dashboard analytics        | the health view itself goes unopened | CAIO       | a report nobody reads is vanity (mm-11); simplify or re-aim it
```

### A.4 Quality — "is the output still good — and still reviewed?"

Two failure modes, opposite directions. The agent degrades (errors rise, approval falls). OR the humans stop reviewing (approval climbs to ~100% — rubber-stamping), which silently removes the HITL safety the governance promised.

```
Metric                 | Source (§5.8)              | Threshold (alert)                  | Owner        | Runbook line
Error / exception rate | run logs                   | spike vs baseline                  | eng          | classify; hotfix or roll back the prompt/version
HITL approval rate     | approval queue             | FALLS (agent worse) OR ~100% (rubber-stamp) | CAIO | falling: fix the agent. ~100%: sample-audit; HITL is theatre
Drift signal           | output distribution monitor| distribution shift vs reference    | eng          | re-prompt / re-eval; check upstream data change
Customer-facing escapes| ticket/complaint tag       | any sensitive output reached a customer wrong | CAIO | incident review; tighten HITL; log per governance
```

The rubber-stamp alert matters as much as the error alert. mm-11's honesty gate applies: a 100% approval rate *looks* like a healthy metric but can mean nobody is actually reviewing — a vanity green. Sample-audit periodically regardless of the rate.

### A.5 Value — "is it delivering, and is anyone steering by it?"

This dimension is the NSM telemetry feeding the ROI model (reference 01). It is the dimension the C-Level cares about; the other four exist to protect it.

```
Metric                 | Source (§5.8)              | Threshold (alert)                  | Owner        | Runbook line
System NSM trend       | NSM event                  | flat or down 2 consecutive weeks   | CAIO         | open a loop hypothesis; is it cohort decay or a real plateau?
Decisions improved     | decision artefacts         | declining (quality objective)      | CAIO         | sample decisions; is the AI still changing calls?
Reports consulted      | dashboard analytics        | the value view unopened by sponsor | CAIO         | the QBR is doing the steering; fix the live view or accept it
```

---

## Part B — Threshold design (how to set a number that isn't noise)

A threshold too tight cries wolf and gets muted; too loose and the issue lands as a client complaint. Principles:

1. **Anchor to a baseline, not an absolute.** "p95 latency > 2× baseline" survives a system that is legitimately slow; "p95 > 3s" breaks the day the workload changes. Most thresholds should be *relative to a trailing window*.
2. **Two tiers: warn and page.** A *warn* (forecast > 80% of cost cap) gives the owner time to act calmly. A *page* (cost cap exceeded, or success rate < 80%) interrupts. Don't page on warns; people learn to ignore a pager that fires on yellows.
3. **Every threshold has an owner and a runbook line.** Iron Law 6. An alert that routes to "the team" routes to nobody. Name the role; write the first action.
4. **Suppress the known-noisy.** Third-party API blips, scheduled-maintenance windows. An alert channel full of false positives is worse than no alert channel — the real one gets missed in the noise.
5. **Review thresholds in the monthly loop.** A threshold that never fired in 90 days is probably too loose; one that fired weekly with no action is too tight or mis-owned. Tune them like any other parameter.

---

## Part B2 — Alert fatigue is a failure mode (treat it as one)

The fastest way to make a piloted company reactive again is to flood its alert channel. Once owners learn the channel cries wolf, they mute it — and the one real page lands in a muted channel. Defend against it:
- **Budget the alert rate.** If a dimension fires more than ~2-3 times a week with no action taken, the threshold is wrong (too tight, or mis-owned) — tune it in the monthly loop, do not let it ride.
- **Page only on page-tier.** Warns go to a dashboard tile or a daily digest, never to the pager. Reserve the interrupt for breaches that genuinely need a human now.
- **Auto-resolve transient noise.** A single liveness blip that self-heals on the next run is a digest line, not a page. Page on *sustained* or *consecutive* failures.
- **Every page must be actionable.** If the owner's honest response to a page is "nothing I can do about that", it should not be a page. An alert nobody can act on is pure fatigue.

A monitoring spec is only as good as the channel staying trusted. An unread alert channel is the same vanity failure as an unopened dashboard (mm-11).

---

## Part C — The dashboard layout (what the screen shows)

Two audiences, two views — mirroring the architect's 6-level dual view, but operational:

### C.1 The operator view (CAIO + system owner — checked daily/weekly)

One screen, top to bottom:
```
[1] System NSM this week + trend sparkline           (value — the headline)
[2] Cohort savings-retention mini-table              (are wave-1..n holding?)
[3] Cost vs monthly budget + cost-per-NSM-unit       (margin)
[4] Liveness strip: every agent green/amber/red      (is it running?)
[5] Open alerts, by severity, with owner + age       (what needs a human now)
```
If the operator opens one screen a week, it is this one (mm-11 — the single weekly view).

### C.2 The executive view (sponsor — checked monthly / in the QBR)

```
[1] NSM trend (quarter)
[2] Realization rate: actual vs projected ROI
[3] Hours saved / cost avoided / decisions improved — to date
[4] Adoption across departments
[5] The one risk or leak that needs an executive decision
```

The executive view is the live precursor to the Quarterly Business Review (reference 04). If the sponsor never opens it, that is *fine* — the QBR carries the steering — but note it: a live view nobody reads is a candidate to retire (mm-11 anti-vanity).

---

## Part D — From reactive to piloted (the transition this phase delivers)

The deliverable is not "a dashboard exists". It is a measurable shift:

| Reactive company (before) | Piloted company (after) |
|---|---|
| Learns an agent died when a customer complains | Liveness alert fires; owner re-triggers before anyone notices |
| Discovers cost overrun on the monthly invoice | Cost-forecast warn at 80%; capped mid-month |
| Finds out a feature was abandoned at the QBR | WAU-drop alert weeks earlier; cohort retention fix opened |
| Trusts the agent because "it's been fine" | Drift + rubber-stamp audits catch silent degradation |
| Steers by anecdote | Steers by the NSM + cohort curves |

The iron test for this phase (folds into the skill's overall iron test): **did the thresholds catch at least one real issue *before* the client complained?** If the first time the client learns of a problem is from the client, the monitoring spec failed — re-tune.

---

## Part D2 — The composite health score (one number for the QBR)

The C-Level wants one health number, not five dimensions. Roll the five into a 0-100 composite, but keep it honest — a composite that hides a red on one dimension is a vanity metric (mm-11).

```
Health = weighted( liveness, cost, usage, quality, value ), each 0-100
Default weights: value 30 · usage 25 · quality 20 · liveness 15 · cost 10
RULE: any single dimension in the red (a page-level breach) CAPS the composite at 69 ("amber"),
      regardless of the weighted average. A green composite over a red dimension is a lie.
```

So a system with perfect liveness/cost/quality but a leaking cohort (usage red) cannot show "green health" — the cap forces the truth onto the QBR. This mirrors reference 01's rule that an averaged ROI must not hide a dead workflow.

---

## Part D3 — Worked example (reactive vs piloted, same incident)

A nightly data-sync the support-triage agent depends on starts failing silently on a Tuesday.

**Reactive company (no thresholds):**
```
Tue  sync fails; agent runs on stale data, mis-routes ~30% of tickets
Wed  reps notice "the agent is being weird" but don't report it
Thu  reps quietly revert to manual; agent WAU starts dropping
Fri  a customer escalates a mis-routed ticket to the sponsor
Mon  sponsor asks the CAIO "is the AI broken?" — first the CAIO has heard of it
→ outcome: a week of degraded service, a shaken sponsor, a leaking cohort, and the
  CAIO learning of the problem from the client. The monitoring spec failed.
```

**Piloted company (this spec live):**
```
Tue 02:10  liveness alert: scheduled sync missed its run → owner paged
Tue 02:40  owner re-triggers sync; quality alert (error rate spike) auto-clears once data is fresh
Tue 09:00  owner notes the root cause (upstream API auth expired); files a 1-line incident
→ outcome: ~30 min of exposure, caught before a single customer noticed, no cohort damage.
  The threshold did its only job: the client learned of nothing because there was nothing to learn.
```

The difference is not the technology — both ran the same agent. The difference is a threshold with an owner and a runbook line. That is the entire deliverable of Phase 3.

---

## Part E — Phase 3 discipline checks

| Check | Pass = |
|---|---|
| All five dimensions covered (liveness, cost, usage, quality, value) | yes |
| Every metric names its §5.8 source | yes |
| Every threshold has a warn/page tier, an owner, and a runbook line | yes |
| The rubber-stamp (HITL ~100%) alert exists, not only the error alert | yes |
| Cost-per-NSM-unit and token-cost-share thresholds present (mm-08 margin) | yes |
| WAU-drop wired as the leading indicator of cohort leak | yes |
| The operator one-screen and the executive view both specified | yes |
| At least one alert demonstrably fired and was acted on (or a plan to validate it) | yes |
| Noisy/known sources suppressed so real alerts surface | yes |

If any fails, the company is still reactive on that dimension. Close it before declaring the system piloted.

---

*Plumbing makes events. Thresholds make a piloted company. An alert with no owner makes noise.*

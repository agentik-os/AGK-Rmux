# Adoption Tracker — {{company}}

> Who uses what, how often — measured as a retention curve by cohort, NOT an average (mm-11). This is the reinforcement instrument (it shows who's lapsing before they're gone) and the usage baseline handed to caio-run-and-optimize.
> Adoption = value received, not vanity. Logins / seats / sessions-attended are NOT adoption.

- **Last updated:** {{date}} · **Owner (after handover):** {{name}}

## North Star Metric per component (value received)
| Component | Adoption NSM (value-received) | Target / active operator | This week |
|---|---|---|---|
| {{agent}} | {{e.g. tickets triaged & accepted/rep/wk}} | {{> X}} | {{value}} |
| {{dashboard}} | {{e.g. decisions made off the dashboard/wk}} | {{> X}} | {{value}} |

## Retention curve by cohort (the real proof)
| Cohort (onboarding week) | Operators | W1 active | W2 | W4 | W8 | Shape |
|---|---|---|---|---|---|---|
| {{2026-Wxx}} | {{n}} | {{%}} | {{%}} | {{%}} | {{%}} | {{collapsing / flattening / plateau}} |

- **Diagnosis:** {{collapsing → fit problem, do NOT just train more; flattening on plateau → adoption proven, plateau height = real adoption}}
- **Are later cohorts retaining better than earlier? (onboarding improving?):** {{yes/no}}

## Per-operator status (reinforcement / lapse flags)
| Operator | Role | AI-literacy | Reached unaided run (aha)? | Last real use | ADKAR step | Flag |
|---|---|---|---|---|---|---|
| {{name}} | {{role}} | {{champ/neutral/skeptic}} | {{yes/no + date}} | {{date}} | {{A/D/K/A/R}} | {{lapsing / converted / re-onboard}} |

## Skeptic watch
| Skeptic | Status | Personal win felt? | Next action |
|---|---|---|---|
| {{name}} | {{converted / pending}} | {{yes/no}} | {{re-onboard date / —}} |

## The one weekly view (what the guardian routine reads)
- NSM this week: {{...}}
- Last cohort retention: {{...}}
- Lapsing operators (flagged): {{names}}
- Biggest friction this week: {{...}} → ONE improvement decided: {{...}}

## Leaky-bucket rule
- [ ] Retention stabilized (curve flattening) BEFORE expanding to the next department.
- Expansion (next dept / more agents) is `caio-run-and-optimize`'s job — not before the bucket holds.

## Handoff
- [ ] Handed to the team + to caio-run-and-optimize as the usage baseline on {{date}}.

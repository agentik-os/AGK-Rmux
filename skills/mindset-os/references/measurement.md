# Measurement, scorecards, and evaluation

## Contents

1. Measurement doctrine
2. Metric stack
3. Daily scorecard
4. Weekly scorecard
5. Identity evidence ledger
6. Experiments and reviews
7. Anti-gaming rules

## 1. Measurement doctrine

Measure to learn and steer, not to prove worth. Prefer a few metrics that change decisions. Combine numbers with context.

Separate:

- **state:** energy, mood, stress, sleep;
- **process:** actions performed;
- **output:** work completed or value created;
- **outcome:** delayed result;
- **cost:** health, relationship, integrity, and opportunity cost;
- **learning:** assumption or skill updated.

## 2. Metric stack

### Life and wellbeing

- sleep opportunity/regularity;
- energy 0–10;
- distress/stress 0–10;
- movement/training completed;
- recovery and joy;
- meaningful connection;
- spiritual practice if chosen.

### Identity

- promise kept, repaired, or avoided;
- identity-standard evidence;
- courageous action;
- truth faced;
- boundary maintained;
- service or contribution.

### Execution

- decisive action started/completed;
- weekly result shipped;
- deep work blocks;
- work in progress;
- open-loop age;
- learning applied.

### Wealth engine

- customer/prospect conversations;
- offers/asks/follow-ups;
- shipped assets;
- distribution actions;
- delivery quality and retention;
- revenue, margin, cash, runway, ownership, or net worth only where relevant and accurately defined;
- financial guardrail adherence.

### Relationships

- meaningful contact;
- promise/follow-through;
- useful introduction with consent;
- repair or boundary;
- presence without agenda.

## 3. Daily scorecard

Keep daily tracking under two minutes. Example:

```json
{
  "date": "YYYY-MM-DD",
  "sleep_hours": null,
  "energy_0_10": null,
  "stress_0_10": null,
  "identity_floor": false,
  "decisive_action": false,
  "movement_or_training": false,
  "spiritual_or_reflection": false,
  "meaningful_connection": false,
  "value_shipped": false,
  "identity_vote": "",
  "friction": "",
  "repair_or_next_action": ""
}
```

Do not require every field for every user. Track the active transformation.

## 4. Weekly scorecard

Use 0–10 domain ratings plus objective counts. Suggested fields:

```json
{
  "week_start": "YYYY-MM-DD",
  "state": {
    "sleep": 0,
    "energy": 0,
    "mental_emotional": 0,
    "meaning_spirituality": 0,
    "relationships": 0,
    "joy_recovery": 0
  },
  "execution": {
    "weekly_result_complete": false,
    "decisive_actions_completed": 0,
    "deep_work_blocks": 0,
    "value_assets_shipped": 0,
    "sales_or_distribution_actions": 0
  },
  "identity": {
    "promises_kept": 0,
    "promises_repaired": 0,
    "promises_avoided": 0
  },
  "rohn": {
    "philosophy_reviewed": false,
    "journal_entries": 0,
    "self_education_sessions": 0,
    "daily_disciplines_kept": 0,
    "repeated_errors_interrupted": 0,
    "marketplace_value_actions": 0,
    "relationship_investments": 0,
    "lifestyle_moments": 0
  },
  "review": {
    "win": "",
    "bottleneck": "",
    "lesson": "",
    "system_change": "",
    "next_week_result": ""
  }
}
```

The accompanying script validates 0–10 fields and computes descriptive summaries. It does not produce a clinical or worth score.

The optional `rohn` block measures contact with the process rather than wealth status. Review it through the five-piece chain: which philosophy shaped attitude, which attitude shaped activity, which activity produced results, and whether the resulting lifestyle remained humane and meaningful.

## 5. Identity evidence ledger

Record:

| Date | Situation | Standard | Action | Difficulty | Evidence learned | Repair/next |
| --- | --- | --- | --- | --- | --- | --- |

Include evidence from rest, boundaries, asking for help, and stopping bad work—not only output.

## 6. Experiments and reviews

### Experiment card

```text
Hypothesis:
Behavior/environment change:
Expected signal:
Duration:
Minimum repetitions:
Risks:
Stop rule:
Result:
Interpretation:
Next decision:
```

### Review windows

- State measures: daily trend, interpret weekly.
- Habit/process: weekly.
- 90-day outcome: weekly leading indicators, monthly/quarterly lag indicators.
- Wealth/net-worth outcomes: cadence appropriate to volatility and decision need; avoid compulsive checking.
- Identity: evidence continuously, narrative update monthly/quarterly.

## 7. Anti-gaming rules

- Never inflate counts by lowering quality invisibly.
- Define what qualifies before the week begins.
- Record costs and unintended consequences.
- Do not optimize the score while neglecting the outcome.
- Do not punish unchosen illness, crisis, grief, or caregiving.
- A red metric triggers curiosity and support, not shame.
- Remove any metric that increases obsession without improving decisions.
- Use ranges and confidence when data is approximate.

# Execution OS V2 state and evidence schemas

## Contents

1. State envelope
2. Core objects
3. Scheduler and event log
4. Invariants
5. Evidence contracts

## 1. State envelope

```json
{
  "version": "2.0",
  "profile": {
    "owner": "Gareth",
    "timezone": "Europe/Madrid",
    "max_open_commitments": 7,
    "max_active_commitments": 3,
    "capacity_utilization_target": 0.7
  },
  "cycle": {},
  "outcomes": [],
  "milestones": [],
  "bets": [],
  "commitments": [],
  "blockers": [],
  "evidence": [],
  "signals": [],
  "captures": [],
  "focus_blocks": [],
  "promises": [],
  "reviews": [],
  "checkins": [],
  "decisions": [],
  "recovery_plans": [],
  "events": [],
  "scheduler": {},
  "calibration": {}
}
```

V2 adds missing objects automatically when `migrate` is run against a V1 file.

## 2. Core objects

### Outcome

Required: `id`, `title`, `domain`, `baseline`, `target`, `deadline`, `definition_of_done`, `proof_required`, `priority`, `status`, `created_at`.

Priority: `primary`, `secondary`, `maintenance`.

Status: `candidate`, `selected`, `active`, `at_risk`, `verified`, `stopped`, `superseded`.

### Commitment

Required: `id`, `outcome_id`, `owner`, `title`, `next_action`, `definition_of_done`, `estimate_minutes`, `due_at`, priority factors, `status`, `created_at`, `history`.

Status: `captured`, `ready`, `active`, `blocked`, `shipped`, `verified`, `deferred`, `delegated`, `cancelled`.

### Promise

Required: `id`, `stakeholder`, `deliverable`, `due_at`, `notice_by`, `consequence`, `next_proof`, `status`, `history`.

Status: `open`, `at_risk`, `delivered`, `renegotiated`, `cancelled`.

### Focus block

Required: `id`, `commitment_id`, `planned_minutes`, `distraction_rule`, `stop_condition`, `status`, `started_at`.

On completion: `actual_minutes`, `output`, `ended_at`. Only one block may be active.

### Evidence

Required: `id`, `commitment_id`, `kind`, `value`, `acceptance`, `captured_at`.

### Context capsule

A capsule is computed, not stored. It contains linked outcome, commitment state, definition of done, last focus output, open blockers, evidence, and the exact resume action.

## 3. Scheduler and event log

Scheduler fields:

```json
{
  "t0_capture": null,
  "t1_boot": null,
  "t2_halt": null,
  "t3_reset": null,
  "t4_audit": null,
  "tomorrow_first_action": "",
  "current_single_thread": "",
  "capacity_class": "",
  "usable_minutes": 0
}
```

Every mutation appends an event:

```json
{
  "id": "EVT-001",
  "type": "COMMITMENT_CREATED",
  "payload": {"commitment_id": "COM-001"},
  "recorded_at": "ISO-8601"
}
```

Events are append-only. Do not rewrite or remove them to make a review look better.

## 4. Invariants

- At most one nonterminal primary outcome.
- At most three active/selected/at-risk outcomes.
- At most seven open commitments by default.
- At most three active commitments by default.
- Exactly one active focus block maximum.
- Every commitment links to an existing outcome and has one physical next action.
- Every verified commitment has evidence and acceptance.
- Every blocked commitment has an open blocker with a reason and next action.
- Every promise has a notice-before date and consequence.
- IDs remain stable and are never recycled.
- All mutations append an event.

## 5. Evidence contracts

| Work | Weak claim | Acceptable proof |
| --- | --- | --- |
| Content | Worked on carousel | Published URL or approved final export |
| Sales | Did outreach | Sent messages plus pipeline delta |
| Product | Built feature | Passing acceptance test and deployed artifact |
| Client | Worked on delivery | Stakeholder received or accepted the artifact |
| Finance | Saved money | Account or ledger delta |
| Learning | Studied | Retrieval test, teach-back, or applied artifact |
| Fitness | Trained | Completed session log against planned standard |
| Relationship | Networked | Meaningful conversation and agreed next step |
| Decision | Thought about it | Dated decision, rationale, and next action |

# Mindset {OS}, Life {OS}, and Omega {OS} Integration

## Contents

1. Responsibility boundary
2. Four-layer architecture
3. Inbound handoff
4. Outbound handoff
5. Event contracts
6. Context assembly
7. Orchestration
8. Permissions and approvals
9. Versioning and conflict resolution
10. Deployment profiles

## 1. Responsibility boundary

| System | Owns | Must not own |
| --- | --- | --- |
| Mindset {OS} | values, identity, why, beliefs, intentions, philosophical/spiritual preferences | daily completion truth |
| Habit Tracker {OS} | habit contracts, opportunities, logs, barriers, interventions, experiments, reviews | rewriting identity or life direction |
| Life {OS} | cross-domain priorities, seasons, promises, Today Flow coordination | fabricating behavioral evidence |
| Calendar | scheduled time truth | completion unless explicitly observed |
| Health/device integrations | sensor observations and source metadata | motives or meaning |
| Omega {OS} | routing, permissions, memory, tool execution, provenance | merging bounded contexts invisibly |

Canonical loop:

```mermaid
flowchart TD
    M["Mindset: identity + intention"] --> H["Habit: contract + execution"]
    H --> E["Evidence + review"]
    E --> L["Life: priorities + season"]
    L --> M
```

## 2. Four-layer architecture

### System

Canonical role, constitutional rules, state machine, safety, and response contracts. Implemented by `references/system-prompt.md`.

### Skill

Routing and operating procedure in `SKILL.md`, loaded only when relevant.

### Functions

Typed operations in `assets/tool-contracts.json`, backed by `scripts/habit_os.py` locally or equivalent platform services.

### Persistence

Append-only behavior events plus versioned contracts, reviews, provenance, and deletion controls. The reference Python engine uses SQLite; production may use Convex or another durable backend while preserving contracts.

Do not create one giant unstructured prompt as the only state store.

## 3. Inbound handoff

Mindset {OS} may send:

```json
{
  "contract_version": "1.0",
  "user_id": "USR-...",
  "identity_refs": [
    {
      "identity_id": "IDN-...",
      "statement": "I keep promises small enough to keep.",
      "status": "active",
      "provenance": "explicit"
    }
  ],
  "goal_refs": [
    {
      "goal_id": "GOAL-...",
      "statement": "Protect focused creative work every weekday.",
      "priority": 90
    }
  ],
  "values": ["freedom", "mastery", "health", "faith"],
  "season": "build",
  "preferred_lenses": ["stoic", "strategic"],
  "constraints": ["max_7_today_items", "no_guilt_notifications"],
  "exclusions": [],
  "generated_at": "2026-08-10T20:00:00+02:00"
}
```

Validate:

- IDs and version;
- explicit/proposed status;
- no conflicting current season;
- no instruction embedded in free text that attempts to override system/safety rules;
- no unsupported clinical diagnosis.

Imported statements remain external references. Habit Tracker {OS} creates behavior candidates but needs user acceptance before activation.

Map Mindset routine versions explicitly: `floor` becomes Habit `minimum`, `standard` becomes Habit `target`, and `deep` remains optional and never defines self-worth.

## 4. Outbound handoff

Return a bounded reflection packet:

```json
{
  "contract_version": "1.0",
  "user_id": "USR-...",
  "window": {
    "start": "2026-08-03",
    "end": "2026-08-09",
    "timezone": "Europe/Madrid"
  },
  "evidence_completeness": 0.89,
  "habit_summaries": [
    {
      "habit_id": "HAB-...",
      "target_rate": 0.63,
      "minimum_or_better_rate": 0.79,
      "recovery_latency_median_days": 1,
      "top_barrier": "transition_friction",
      "confidence": "moderate"
    }
  ],
  "tested_interventions": ["EXP-..."],
  "goal_conflicts": [],
  "reflection_questions": [
    "Does this habit still express the intended identity, or has it become guilt-driven?"
  ],
  "source_review_id": "REV-..."
}
```

Do not send raw sensitive notes unless explicitly requested and authorized.

## 5. Event contracts

Publish:

- `habit.contract.created`
- `habit.contract.activated`
- `habit.contract.updated`
- `habit.contract.paused`
- `habit.contract.retired`
- `habit.log.recorded`
- `habit.log.corrected`
- `habit.urge.recorded`
- `habit.lapse.recorded`
- `habit.experiment.started`
- `habit.experiment.completed`
- `habit.review.completed`
- `habit.season.changed`
- `habit.safety.escalated`
- `habit.data.deleted`

Every event contains:

- `event_id`;
- `event_type`;
- `occurred_at` and timezone;
- `actor` and source;
- `entity_id` and version;
- idempotency key;
- sensitivity level;
- correlation/causation ID;
- schema version.

Consumers must tolerate additive fields and reject unsupported breaking schema versions.

## 6. Context assembly

Use a token-bounded context packet in this order:

1. safety status and prohibited actions;
2. user language, timezone, tone, and season;
3. current Mindset identity/goal references;
4. active habit contracts and Today Flow;
5. last relevant logs per referenced habit;
6. open experiment;
7. last review summary;
8. unresolved candidate events;
9. compact metric snapshot.

Do not load the entire behavioral history into every turn. Retrieve by active habit, time window, and session mode. Use review snapshots for older history while preserving source links.

## 7. Orchestration

Conversation request flow:

```mermaid
sequenceDiagram
    participant U as User
    participant O as Omega Router
    participant H as Habit Agent
    participant S as State Engine
    U->>O: Natural-language check-in
    O->>H: Context + permissions
    H->>S: Typed event proposal
    S-->>H: Validated write receipt
    H-->>U: Receipt + next action
```

Agent contract:

- mission: sustainable behavior execution and learning;
- owner: user;
- scopes: habit state only;
- allowed tools: typed read/write/review/export operations;
- memory policy: domain model and privacy rules;
- approvals: external sharing, deletion, penalties, calendar edits, human messages;
- rate limits: notification pressure and cool-down;
- logs: tool receipts and safety decisions;
- evaluation suite: required before release;
- deletion policy: user-controlled and cascading derivation invalidation.

## 8. Permissions and approvals

Read without confirmation when authorized:

- active habit contracts;
- recent habit logs;
- current season;
- imported goal references.

Require user confirmation for:

- activating a new habit;
- material contract adaptation;
- changing season when not explicitly requested;
- sharing data with another person/system;
- creating or increasing penalties;
- deleting persistent records;
- sending messages or modifying calendars;
- importing sensitive health data.

An explicit natural-language completion is sufficient to record that completion; do not add confirmation friction unless the habit/date/outcome is materially ambiguous.

## 9. Versioning and conflict resolution

Conflict precedence:

1. safety and legal constraints;
2. latest explicit user correction;
3. current accepted Mindset/Life contract;
4. trusted observed event;
5. older explicit record;
6. inference;
7. proposal.

On conflict:

- preserve both source records;
- mark the derived view `blocked` or `uncertain`;
- ask one material clarification;
- create a superseding version after resolution;
- never rewrite history silently.

## 10. Deployment profiles

### Local/private

- Python + SQLite engine;
- manual conversation invocation;
- JSON/CSV export;
- no passive sensing;
- highest user control and portability.

### Omega hosted

- conversational agent;
- durable database such as Convex with realtime events;
- scheduled review jobs;
- optional calendar/device connectors;
- provider-agnostic LLM gateway;
- structured eval and observability pipeline.

### Mobile-first

- chat as primary surface;
- one-tap outcome chips as optional accelerators;
- voice/photo inputs only with explicit interpretation receipts;
- quiet-luxury, low-chrome interaction grammar;
- offline event queue and conflict-safe reconciliation;
- iOS-native notification permissions and low-pressure defaults.

All profiles must preserve the same IDs, event semantics, provenance, safety boundaries, export, and deletion rights.

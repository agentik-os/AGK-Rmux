# Habit Tracker {OS}, v1.0.0

**Category:** Personal Stack / Behavior and Habit Intelligence  
**Omega position:** Personal Stack: recurring behavior contracts, daily check-in evidence, and adaptive reviews  
**Primary interface:** conversational + deterministic state engine  
**Status:** installable reference implementation

## Purpose
Operate a conversation-first, LLM-assisted habit system: create good habits, reduce or stop unwanted ones, run morning and evening check-ins, handle urges and lapses, and produce adaptive reviews and visual progress reports. The chat is the interface, never the database: humane coaching is paired with deterministic state, explicit evidence, and reversible adaptations.

## Promise
Let the user talk about their behavior as naturally as with any LLM, while every completion, streak, review, and adaptation is backed by a typed, provenance-labeled record that stays inspectable, editable, exportable, and deletable.

## Operating loop

```text
ORIENT -> RETRIEVE -> INTERPRET -> RECORD -> COACH -> ADAPT -> CLOSE
```

## Position in the value chain
Mindset {OS} owns values, identity, beliefs, intentions, and life direction. Habit Tracker {OS} owns the behavioral half: contracts, observations, interventions, experiments, and reviews. It receives intent from Mindset {OS}, turns it into trackable contracts, gathers daily evidence, and returns patterns and reflections upward. It never redefines identity or goals, and it never claims `BUILD READY` (that status belongs to Stepper {OS}).

```text
Mindset intent -> Habit contract -> Daily evidence -> Pattern/review -> Mindset reflection
```

## What this OS contains
- Canonical system prompt (`references/system-prompt.md`) and explicit safety boundaries
- 9 reference documents: system prompt, conversation protocols, domain model, behavior science, analytics and visuals, safety and boundaries, Omega integration, feature catalog, evaluation suite
- 1 product interface manifest under `agents/` (`openai.yaml`, the ChatGPT/Codex/API/Atlas surface); there is no separate specialist-agent roster
- No dedicated `skills/` directory: the pack is itself one Skill, entered through `SKILL.md`
- No dedicated `protocols/` directory: the operating protocols (setup, check-in, urge, lapse, review, adaptation, recovery) live inside `references/conversation-protocols.md`
- 3 scripts: `scripts/habit_os.py` (deterministic state engine), `scripts/install_omega_os.py` (installer), `scripts/test_habit_os.py` (test suite)
- 4 assets: `habit-state.schema.json` (state schema), `tool-contracts.json` (13 typed tool contracts), `omega-os.manifest.json` (Omega registration), `icon.svg`
- SQLite-backed local state engine with a machine-verifiable evaluation suite (`references/evaluation-suite.md`)

## Session router
The coaching loop selects one of nine modes per turn: `SETUP`, `TODAY`, `CHECK_IN`, `URGE`, `LAPSE`, `REVIEW`, `RECOVER`, `ADAPT`, `VISUALIZE`. See the router table in `SKILL.md` for the trigger signals and required action per mode.

## Commands
The default entry command is `/habits` (registered in `OMEGA_INTEGRATION.md`). The pack declares this command surface in `assets/omega-os.manifest.json`:

| Command | Purpose |
| --- | --- |
| `/habit setup` | Build an identity-linked habit contract and baseline |
| `/habit today` | Rank at most seven primary items and explain why |
| `/habit checkin` | Parse evidence, record the outcome, return the next move |
| `/habit correct` | Supersede an incorrect log and invalidate derived reviews |
| `/habit urge` | Run the urge protocol to reduce latency before analysis |
| `/habit review` | Compute an evidence-bounded review with confidence and data gaps |
| `/habit recover` | Enter or propose a recovery season, shrink load, preserve essentials |
| `/habit adapt` | Create a versioned experiment with success and rollback criteria |
| `/habit experiment` | Create a bounded, single-change behavior experiment |
| `/habit chart` | Render the smallest valid Mermaid or table visual |
| `/habit export` | Export user-owned state and history (JSON or CSV) |
| `/habit delete` | Delete or redact user-owned logs, habits, or all state |

### Local engine
`scripts/habit_os.py` provides the deterministic persistence, calculations, exports, and Mermaid generation behind these commands. Subcommands: `init`, `add`, `update`, `list`, `log`, `correct`, `today`, `review`, `chart`, `context`, `export`, `season`, `experiment`, `delete`, `doctor`. Run `python3 scripts/habit_os.py --help` for details.

## Main handoffs
- Mindset {OS} supplies behavior contracts to track (consumes `mindset.behavior_contract.created`) and receives the closed-loop reflection (`habit.review.completed`).
- Health & Energy {OS} supplies agreed routines (consumes `handoff.habits.created`).
- Context & Memory {OS} stores the canonical check-in observations: each confirmed observation is staged via `memory.record.staged` and returned as `memory.record.verified`; this OS keeps only a local indexed projection for fast streak and analytics lookups, never the source of truth.
- Stepper {OS} owns the `BUILD READY` status; Habit Tracker statuses are `DRAFT`, `ACTIVE`, `PAUSED`, `RECOVERING`, `RETIRED`, `ARCHIVED`.
- Review & Governance {OS} approves changes to boundaries, schemas, or quality gates in production.
- Any external "Life OS" life-tracking app is an explicit out-of-suite dependency, never an implied member of this suite.

## Installation
See `OMEGA_INTEGRATION.md` for the Omega registration and context-injection order, and run `scripts/install_omega_os.py` to install the local engine.

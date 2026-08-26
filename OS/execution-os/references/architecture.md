# Execution OS architecture

## Contents

1. Purpose and doctrine
2. Boundaries and handoffs
3. Object model
4. Lifecycle
5. Portfolio and horizons
6. Measurement
7. Gates and invariants
8. Failure taxonomy

## 1. Purpose and doctrine

Execution OS is a personal control system for translating chosen aims into observable change. Its unit of truth is not a task, hour, intention, streak, or feeling. Its unit of truth is an accepted outcome delta backed by evidence.

The OS must answer six questions at any moment:

1. What matters now?
2. What does finished look like?
3. What is the next physical move?
4. What is blocking movement?
5. What proof exists?
6. What must change after observing reality?

### Doctrine

- Outcomes over activity.
- Evidence over self-report.
- Constraints before ambition.
- One chosen tradeoff over many vague priorities.
- Fast feedback over long hidden work.
- Sustainable intensity over chronic emergency.
- Recovery as maintenance, not moral failure.
- Compounding systems over heroic willpower.

## 2. Boundaries and handoffs

| System | Owns | Execution OS receives | Execution OS returns |
| --- | --- | --- | --- |
| Mindset OS | Identity, values, beliefs, emotional framing | Purpose, identity direction, values, fears | Behavioral evidence and decision conflicts |
| Habit Tracker OS | Repeated behaviors and streak data | Habit adherence, friction, trend | Critical habits and execution-linked behavior requests |
| Calendar | Reserved time | Availability, immovable events | Focus blocks, buffers, reviews |
| Knowledge system | Notes, references, learning | Context and source material | Decisions, lessons, evidence links |
| Project/task tool | Work inventory | Tasks, owners, dependencies | Active commitments and status |
| Execution OS | Selection, commitment, proof, adaptation | All above | Outcome movement and corrected plan |

Do not absorb psychotherapy, medical diagnosis, financial promises, or software-build orchestration into this OS.

## 3. Object model

### North Star

Long-horizon direction. It informs selection but is not a daily task.

Fields: statement, values, anti-goals, horizon, source, review date.

### Season

A bounded 6-12 week emphasis. It states what wins, what merely stays healthy, and what is deliberately quiet.

Fields: theme, dates, primary domain, capacity class, primary constraint, exclusions, success narrative.

### Outcome

An observable state change.

Fields: baseline, target, deadline, definition of done, proof, leading indicators, lagging indicator, confidence, owner, domain, status.

### Bet

A chosen theory of leverage connecting actions to an outcome.

Fields: hypothesis, expected mechanism, cost, signal, review date, stop rule.

### Milestone

A verified intermediate state that reduces uncertainty or unlocks the next phase.

### Commitment

A promised deliverable or decision that can be completed in one bounded work session.

Every open commitment has one owner, one reality date, and exactly one physical next action. Gareth's default ceiling is seven open commitments. Only three may be `ACTIVE` simultaneously.

### Focus Block

A protected execution window linked to one commitment. It contains a start ritual, distraction boundary, and stop condition.

### Blocker

A named obstacle with class, owner, next action, escalation time, and impact.

### Evidence

An observable artifact or metric supporting a claim.

### Signal

One dated, immutable line recording what reality did in relation to an outcome. Prefer a number when possible. A signal is not a feeling or retrospective story; it feeds drift detection.

### Review

A time-stamped comparison of intention, evidence, variance, lesson, and correction.

### Recovery Plan

A temporary operating mode that restores minimum viability after overload, illness, travel, sleep loss, or failure.

### External Promise

A stakeholder-facing expectation that can create consequence beyond the internal plan. It records the person, deliverable, reality date, notice-by date, consequence, next proof, linked commitment, status, and renegotiation history.

### Event

An immutable record of a state mutation. Events make missed commitments, renegotiations, evidence, focus blocks, and system corrections auditable. Events are append-only; summaries may be regenerated, but history is not cosmetically rewritten.

## 4. Lifecycle

### Season lifecycle

`DRAFT -> COMMITTED -> ACTIVE -> ADAPTING -> CLOSED`

- Enter `COMMITTED` only after exclusions and capacity are explicit.
- Enter `ACTIVE` only after the first milestone and next commitment exist.
- Enter `ADAPTING` when evidence invalidates the plan but the outcome remains valid.
- Enter `CLOSED` with achieved, partially achieved, stopped, or superseded status and a retrospective.

### Outcome lifecycle

`CANDIDATE -> SELECTED -> ACTIVE -> AT_RISK -> VERIFIED | STOPPED | SUPERSEDED`

### Commitment lifecycle

`CAPTURED -> READY -> ACTIVE -> BLOCKED -> SHIPPED -> VERIFIED`

Allow `CANCELLED` and `DEFERRED` from any nonterminal state. A completed commitment without evidence remains `SHIPPED`, not `VERIFIED`.

### Kernel limits

- Three active outcomes maximum: one primary growth outcome plus up to two secondary/maintenance outcomes.
- Seven open commitments maximum.
- Three active commitments maximum.
- Two deep blocks per day by default.
- One Single Thread per day and per block.

### Persistence contract

- The V2 JSON state is the local source of execution truth.
- Every mutation appends an event.
- T0-T4 timestamps reveal which scheduler cycles actually ran.
- Context capsules are computed from the latest commitment, focus, blocker, and evidence state.
- Backups precede migrations and material manual edits.
- External promises remain distinct from internal commitments so stakeholder risk cannot be hidden by reprioritization.

## 5. Portfolio and horizons

Use a nested horizon stack:

- Life direction: 3-10 years; review semiannually.
- Strategic year: annual themes and constraints; review quarterly.
- Season: 6-12 weeks; review weekly.
- Week: 1-3 deliverables; review weekly.
- Day: one must-win proof; review daily.
- Block: one commitment; review immediately.

The horizon stack is a trace chain, not a waterfall. Daily evidence may invalidate a weekly bet or even a seasonal assumption.

### Default domain portfolio

Use these only as a starting vocabulary: work/client, owned venture, wealth, content/reputation, learning, health, faith/meaning, relationships/network, life administration, play/travel.

Classify each domain for the season:

- `WIN`: receives growth capacity; only one default.
- `MAINTAIN`: receives a minimum viable standard.
- `RECOVER`: temporarily prioritized to restore capacity.
- `PAUSE`: consciously receives no growth work.

## 6. Measurement

### Execution score, 0-100

Use as a diagnostic, never as a worth score.

- Outcome progress: 35
- Commitment reliability: 20
- Focused execution: 15
- Proof and quality: 15
- Recovery integrity: 10
- Strategic alignment: 5

Calculate only from available data. Mark missing components as unknown rather than awarding points.

### Useful measures

- Outcome delta: movement from baseline toward target.
- Reliability: verified commitments / commitments due.
- Focus conversion: verified output per focused hour.
- Cycle time: ready to verified.
- WIP age: time active without proof.
- Rework rate: rejected or repeated output / shipped output.
- Recovery debt: planned recovery missed or sleep/health constraints carried forward.
- Avoidance rate: repeatedly deferred high-impact commitments / high-impact commitments due.

Avoid vanity metrics such as raw hours, raw task count, or streak length without outcome connection.

## 7. Gates and invariants

### Outcome gate

Do not activate an outcome unless:

- the finish line is observable;
- the user has authority or a dependency strategy;
- a deadline or review date exists;
- capacity and exclusions are named;
- the first evidence checkpoint exists.

### Week gate

Reject or shrink the week plan when focused estimates exceed 70% of real discretionary capacity. Reserve the remainder for admin, transitions, recovery, and uncertainty.

### Day gate

Do not schedule more than 80% of usable time. Protect at least one buffer. If energy is low, reduce scope before extending time.

### Completion gate

Do not mark verified without proof and an acceptance condition.

### WIP gate

When seven commitments are open, require finish, stop, renegotiate, defer, delegate, or park before accepting another. When three commitments are active, require a state change before starting another.

### Drift test

Run when an outcome has not moved for two consecutive weeks. Answer from the Signal Log:

1. Did the planned work blocks actually happen?
2. When the work happened, did the expected signal move?
3. Knowing the evidence now, would the user choose this outcome again?

Classify:

- No work: capacity drift. Keep the outcome and repair capacity/WIP.
- Work and signal: on course. Hold.
- Work and no signal: strategy drift. Keep the outcome and change one mechanism.
- Work and signal but no longer desired: value drift. Close at T4 rather than grinding automatically.

## 8. Failure taxonomy

Use the taxonomy in SKILL.md, plus these signals:

| Class | Signal | First intervention |
| --- | --- | --- |
| Clarity | Rewriting the task repeatedly | Define artifact and acceptance |
| Priority | Many starts, few finishes | Choose one primary and kill list |
| Capacity | Chronic spillover | Re-budget from actual availability |
| Friction | Delayed starts | Prepare environment and two-minute ignition |
| Skill | Slow, uncertain execution | Narrow learning sprint or expert help |
| Dependency | Waiting without escalation | Owner, deadline, fallback |
| Emotion | Perfectionism or avoidance language | Safe ugly first proof |
| Feedback | Long work without signal | Earlier test or external check |
| Commitment | Rationalization and repeated deferral | Recommit, redesign, or stop honestly |

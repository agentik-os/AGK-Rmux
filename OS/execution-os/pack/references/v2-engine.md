# Execution OS V2 engine

## Contents

1. Source-of-truth contract
2. T0-T4 commands
3. Promise Ledger
4. Commitment transitions
5. Context capsules
6. Calibration and backups
7. Safe operating sequence

## 1. Source-of-truth contract

The V2 JSON state is local-first and event-backed. Every mutation appends an immutable event. Preserve the file and its IDs; do not remove missed work to improve the record cosmetically.

Initialize once:

```bash
python3 scripts/execution_engine.py --state execution-state.json init \
  --owner Gareth --timezone Europe/Madrid --max-open 7 --max-active 3
```

For an existing V1 state, back it up and run `migrate`.

## 2. T0-T4 commands

```bash
# T0: capture in under 20 seconds, then return
python3 scripts/execution_engine.py --state execution-state.json capture \
  --kind ? --text "Potential idea; do not process now"

# T1: select the day thread from real capacity
python3 scripts/execution_engine.py --state execution-state.json boot \
  --capacity AMBER --usable-minutes 180 \
  --must-win "Client A receives the accepted audit" \
  --not-today "0Ra redesign and new tooling"

# Start and close one focus thread
python3 scripts/execution_engine.py --state execution-state.json focus COM-001 \
  --minutes 50 --distraction-rule "Messages closed; capture and return"
python3 scripts/execution_engine.py --state execution-state.json focus-end FOC-001 \
  --actual-minutes 55 --output "Audit draft complete" \
  --next-action "Run acceptance checklist"

# T2: close the day and write the first move
python3 scripts/execution_engine.py --state execution-state.json halt \
  --proof "Audit draft" --classification PROGRESSED --energy 4 --focus 4 \
  --friction "Messages at 11:00" --tomorrow "Open acceptance checklist"

# T3 and T4
python3 scripts/execution_engine.py --state execution-state.json reset \
  --weekly-truth "First blocks produced all meaningful output" \
  --next-week-win "Five client proofs accepted" \
  --system-experiment "No messages before first block"
python3 scripts/execution_engine.py --state execution-state.json audit \
  --decision "Continue primary outcome" --reason "Signals moved" \
  --system-change "Protect the first block" --obsolete-killed "Old redesign"
```

## 3. Promise Ledger

Create one promise per external expectation, linked to a commitment when possible:

```bash
python3 scripts/execution_engine.py --state execution-state.json add-promise \
  --stakeholder "Client A" --deliverable "Accepted automation audit" \
  --due 2026-08-14T17:00:00+02:00 --notice-by 2026-08-13T12:00:00+02:00 \
  --consequence "Client implementation remains blocked" \
  --next-proof "Send reviewed outline" --commitment COM-001
```

Use `promises` to expose open, notice, and overdue risk. Use `renegotiate-promise` before silent lateness and `deliver-promise` with observable evidence.

## 4. Commitment transitions

- `block`: name the reason, next action, owner, and optional escalation time.
- `unblock`: resolve the blocker and write the new next action.
- `defer`: remove execution capacity and name a review date.
- `delegate`: name the person and follow-up date.
- `cancel`: preserve the reason in history.
- `complete`: attach evidence and acceptance, then mark verified.
- `close-outcome`: verify, stop, or supersede the outcome explicitly.

Never edit statuses by hand unless repairing a corrupted state with a backup present.

## 5. Context capsules

Run `context-capsule COM-###` before a project switch and when resuming work. It returns the linked outcome, current status, definition of done, last output, blockers, evidence, and exact re-entry action.

## 6. Calibration and backups

Every `focus-end` compares planned with actual minutes. `status` reports the accumulated estimate ratio. Use the ratio to reduce scope or correct future block estimates; do not treat it as a performance score.

Create a dated backup with `backup` or provide `--output`. The command returns a SHA-256 digest.

## 7. Safe operating sequence

1. `backup` before migrations or manual edits.
2. `migrate` once for V1 state.
3. `validate` before planning.
4. Run the required T-cycle or state transition.
5. `validate` again.
6. Use `status`, `promises`, and `context-capsule` for read-only inspection.

# Execution OS V2 — Command Walkthrough

Run from the package root.

## 1. Initialize

```bash
ENGINE="skill/execution-os/scripts/execution_engine.py"
STATE="my-execution-state.json"

python3 "$ENGINE" --state "$STATE" init \
  --owner Gareth --timezone Europe/Madrid --max-open 7 --max-active 3
```

## 2. Create the primary outcome

```bash
python3 "$ENGINE" --state "$STATE" add-outcome \
  --title "Run Execution OS V2 for 30 days" \
  --domain "personal operating system" \
  --baseline "No persistent T0-T4 history" \
  --target "Thirty honest days and first T4 audit" \
  --deadline 2026-09-10 \
  --done "Daily closures, four weekly resets, and first audit exist" \
  --proof "V2 state file and review records" \
  --priority primary --confidence 4
```

## 3. Add one commitment

```bash
python3 "$ENGINE" --state "$STATE" add-commitment \
  --outcome OUT-001 \
  --title "Complete the first closed day" \
  --next-action "Open 01_ONE_PAGE.md and list every open loop" \
  --done "T1 and T2 records exist and tomorrow's first action is written" \
  --minutes 50 --due 2026-08-12T18:00:00+02:00 \
  --impact 5 --urgency 5 --leverage 5 --confidence 4 --switch-cost 1
```

## 4. Run T0, T1, and Focus

```bash
python3 "$ENGINE" --state "$STATE" capture \
  --kind ? --text "New idea to inspect during T2"

python3 "$ENGINE" --state "$STATE" boot \
  --capacity AMBER --usable-minutes 180 \
  --must-win "First closed day recorded" \
  --not-today "New projects and OS redesign"

python3 "$ENGINE" --state "$STATE" focus COM-001 \
  --minutes 50 --distraction-rule "Messages closed; capture and return"
```

## 5. End focus and close the day

```bash
python3 "$ENGINE" --state "$STATE" focus-end FOC-001 \
  --actual-minutes 55 --output "One Page and Loop Register complete" \
  --next-action "Open the T2 Halt template"

python3 "$ENGINE" --state "$STATE" halt \
  --proof "One Page and Loop Register" --classification PROGRESSED \
  --energy 4 --focus 4 --friction "Messages at midday" \
  --tomorrow "Open the T2 Halt template"
```

## 6. Promise Ledger

```bash
python3 "$ENGINE" --state "$STATE" add-promise \
  --stakeholder "Client A" --deliverable "Accepted automation audit" \
  --due 2026-08-14T17:00:00+02:00 \
  --notice-by 2026-08-13T12:00:00+02:00 \
  --consequence "Implementation remains blocked" \
  --next-proof "Reviewed outline sent" --commitment COM-001

python3 "$ENGINE" --state "$STATE" promises
```

## 7. Block, unblock, defer, delegate, or cancel

```bash
python3 "$ENGINE" --state "$STATE" block COM-001 \
  --reason "Awaiting client input" --next-action "Ask for missing credentials" \
  --escalate-at 2026-08-12T10:00:00+02:00

python3 "$ENGINE" --state "$STATE" unblock COM-001 \
  --resolution "Credentials received" --next-action "Run connection test"
```

Use `defer`, `delegate`, or `cancel` rather than deleting history.

## 8. Re-entry capsule

```bash
python3 "$ENGINE" --state "$STATE" context-capsule COM-001
```

## 9. T3 and T4

```bash
python3 "$ENGINE" --state "$STATE" reset \
  --weekly-truth "Protected first blocks created every useful proof" \
  --next-week-win "Five client promises delivered" \
  --system-experiment "No communication before first focus block"

python3 "$ENGINE" --state "$STATE" audit \
  --decision "Continue the primary outcome" --reason "Signals moved" \
  --system-change "Protect the first daily block" \
  --obsolete-killed "Unneeded dashboard redesign"
```

## 10. Validate and back up

```bash
python3 "$ENGINE" --state "$STATE" status
python3 "$ENGINE" --state "$STATE" validate
python3 "$ENGINE" --state "$STATE" backup
```

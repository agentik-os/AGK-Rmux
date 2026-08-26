# Omega OS Integration

## Contents

1. Integration architecture
2. Files and entry points
3. Commands
4. Function layer
5. Persistence and eventing
6. Blueprint and Stepper adapters
7. Agent adapters and permissions
8. Installation and validation

## 1. Integration architecture

```text
Omega command router
→ Builder Director
→ Blueprint/Stepper adapters
→ canonical Builder state + event journal
→ repository/worktree executor
→ agent/tool adapters
→ independent verification/reviews
→ Stepper Tracker/Verifier writeback
→ release report and operations handoff
```

Keep probabilistic implementation behind deterministic state transitions, locks, hashes, command results, and gates.

## 2. Files and entry points

Install the skill folder as the extension source. Bind:

- system prompt: `references/system-prompt.md`;
- skill: `SKILL.md`;
- role prompts: `assets/builder-role-prompts.json`;
- function definitions: `assets/builder-tools.json`;
- state schema: `assets/builder-state.schema.json`;
- manifest: `assets/omega-os.manifest.json`;
- local state CLI: `scripts/builder_os.py`.

Recommended Omega targets:

```text
extensions/builder-os/
prompts/system/builder-os.md
prompts/roles/builder-os.json
tools/builder-os.json
schemas/builder-state.schema.json
config/plugins/builder-os.json
```

## 3. Commands

| Command | Effect |
| --- | --- |
| `/build <project>` | initialize/resume and execute toward release readiness |
| `/build preflight` | validate contracts, repository, environment, baseline |
| `/build status` | report Tracker/Builder/gate truth |
| `/build plan` | show Stepper-selected safe wave; do not invent graph |
| `/build run` | execute Planner waves autonomously until blocked/paused/complete |
| `/build step <id>` | execute only if Stepper says runnable |
| `/build test [scope]` | run registered verification without falsifying status |
| `/build verify [id]` | run deterministic checks/reviews |
| `/build repair <id>` | repair from registered failure evidence |
| `/build audit` | inspect drift, traceability, evidence, gates, repository health |
| `/build resume` | reconcile interruption and continue exact program |
| `/build pause` | checkpoint safely and release/retain resources by policy |
| `/build release-check` | evaluate Stepper plus BG01–BG20 |
| `/build report` | export status/final handoff artifacts |

## 4. Function layer

Implement functions from `assets/builder-tools.json`. Mutation functions require project ID, expected state revision, actor/worker, idempotency key, and evidence/result payload where applicable. Reject stale optimistic-concurrency revisions.

Separate:

- query functions: status, plan, context, evidence, audit, release preview;
- state functions: initialize, claim, transition, record, block, checkpoint;
- execution functions: implement, verify, repair, review, integrate;
- governance functions: decision request, change set, stale marking;
- export functions: report and operations handoff.

## 5. Persistence and eventing

Use a transactional store (SQLite is sufficient for one local runtime; server deployments may use a transactional database) plus immutable artifact storage. Maintain:

- project state and foreign-input fingerprints;
- attempts, events, checks, reviews, artifacts, locks/leases;
- decision requests, blockers, changesets, follow-ups;
- docs ledger, gate snapshots, checkpoints, final handoffs.

Emit append-only events such as:

```text
BUILD_INITIALIZED
PREFLIGHT_PASSED
WAVE_SELECTED
STEP_CLAIMED
CONTEXT_COMPILED
IMPLEMENTATION_RECORDED
VERIFY_STARTED
CHECK_PASSED/CHECK_FAILED
REPAIR_STARTED
REVIEW_PASSED/REVIEW_FAILED
INTEGRATION_STARTED/INTEGRATED
POST_MERGE_PASSED/FAILED
STEP_EVIDENCE_SUBMITTED
STEP_DONE/BLOCKED/STALE
CHECKPOINT_CREATED
RELEASE_GATE_PASSED/FAILED
BUILD_RELEASE_READY
```

## 6. Blueprint and Stepper adapters

Blueprint adapter is read-only except for submitting evidence/delta requests through explicit APIs. Verify frozen handoff status/checksum.

Stepper adapter must expose validation, status, resume, plan, claim/locks or coordinated claim, attempt evidence, verification transition, blocker/decision/change-set writeback, and release-check. If Stepper lacks a required mutation API, Builder must not create a competing silent status; use a compatible journal/adapter and reconcile explicitly.

## 7. Agent adapters and permissions

Grant each adapter only required filesystem, shell, repository, browser, cloud, and external-service capabilities. Sanitize context and outputs. Model-generated shell input must use argv-safe execution or an allowlisted command runner; avoid implicit `shell=True`. Enforce path roots, timeouts, output limits, redaction, and destructive-action policy.

## 8. Installation and validation

Run a dry-run first:

```bash
python3 scripts/install_omega_os.py /absolute/path/to/omega-os
```

Apply after reviewing targets:

```bash
python3 scripts/install_omega_os.py /absolute/path/to/omega-os --apply
```

Validate the deterministic substrate:

```bash
python3 scripts/builder_os.py demo
python3 scripts/builder_os.py validate /path/to/builder-state.json
python3 scripts/builder_os.py status /path/to/builder-state.json
python3 scripts/builder_os.py release-check /path/to/builder-state.json
```

The installer must be idempotent, preserve differing existing files without `--force`, and support the structured skill layout.

# Orchestration Engine — Gate + Driver + Guardian

**Status:** Design — approved direction, pending spec review
**Date:** 2026-05-31
**Author:** Oracle (brainstormed with the operator)
**Scope:** OmegaOS (`~/VibeCoding/work/OmegaOS`) — Rust core, Bun fallback, rmux workers
**Supersedes:** the prose-only sequential-execution guarantee in the VPS `/planner` skill

---

## 1. Problem

When agents execute a multi-step plan, they **skip sub-steps** — jumping from
STEP-150 to STEP-250 without doing 151–249. This is the single most damaging
orchestration failure: the plan looks "advanced" but the foundation is full of
holes.

### Why the current fixes fail

The VPS planner skill (`~/.claude/commands/planner.md` v6.0) already "fixed" this
**in prose**:

> Rule 4: ABSOLUTE sequential execution — ZERO tolerance. NEVER skip a step. Not even one.

And its own anti-pattern table already lists `Jumped STEP-150 to STEP-450`. The
fix was more emphatic prose. **It still fails** — because prose is an instruction
to an LLM, and under context pressure (1500-task plans, parallel waves, saturated
context) the LLM skips anyway. *Code lies; comments lie; only runtime tells the
truth* (L1) — and runtime says the prose does not hold.

### Root cause (evidence-grounded)

OmegaOS has **two disconnected plan systems**:

| System | File | State | Evidence |
|---|---|---|---|
| **DAG gate** (can't-skip) | `planner.rs` `PlanTracker` | ✅ exists, tested, **orphaned** | Used only by `omega-tui/src/ui.rs:1624` for display. `orchestration.rs` does **not** import it. |
| **Live driver** (LLM in loop) | `orchestration.rs` `Orchestrator` | ⚠️ runs, but **delegates to an LLM oracle** | `execute()` (lines 116-134) dispatches `plan.tasks` in a flat `for` loop with **no `depends_on`, no waves, no ready-set**; for non-trivial missions it spawns an LLM oracle that decomposes + dispatches via `omega spawn-worker` (lines 261-269). |
| **Per-step verify** (proof) | `PlanStep.verify_command` | ❌ **executed nowhere** | grep: `verify_command` is read only by display; no caller runs it. |

`planner.rs` even has a passing test `cannot_skip_steps` proving `start_step("STEP-003")`
bails when deps are unmet. **The gate works — but the execution path doesn't go
through it.** The beautiful DAG enforcer sits unused while a free-decomposing LLM
drives the real loop.

**Therefore the fix is integration, not greenfield.** Wire the existing gate into
a real driver, add the missing guardian, take the LLM out of the dispatch decision.

---

## 2. Goals / Non-Goals

### Goals
- **G1 — Can't-skip by construction.** The execution loop selects the next work
  from the DAG (`PlanTracker`), never from LLM judgment. Skipping becomes
  structurally impossible, not prose-discouraged.
- **G2 — Verified completion.** No step is marked `done` until a Guardian
  independently proves it (runs `verify_command`; adversarial consensus for
  high-stakes steps). A worker's own `done.json` is an *input, never the verdict*
  (R-VERIFY).
- **G3 — Maximum autonomy.** `omega run` drives the whole plan: ready-set → spawn
  → guardian → advance. The LLM only (a) generates plan content and (b) executes a
  single pre-chosen step inside a worker.
- **G4 — Install parity (L0).** Everything ships via `OmegaOS/install.sh`. A fresh
  `git clone && ./install.sh` on any VPS reproduces the complete orchestration
  system: Rust binary + skills + slash commands + Bun fallback + crons.
- **G5 — Audits as first-class.** Replace the separate "Phase 7 verify" with an
  `audit` wave inside the plan itself.

### Non-Goals
- Rewriting the rmux SDK, session manager, or done.json protocol — all reused.
- Touching the TUI (`app.rs`, `ui.rs`), `agents.rs`, or `providers.rs` — under
  concurrent edit elsewhere; zero overlap with orchestration (R-SCOPE).
- Building a new quality-gate engine — `gate.rs` (rubric / MultiGrader ≥2/3 /
  Popper) is reused as the Guardian's adversarial tier.

---

## 3. Architecture

```
                    ┌─────────────────────────────────────────────┐
  omega plan   ───► │ GATE — planner::PlanTracker        (EXISTS)  │  DAG, acyclic,
                    │  ready_steps / start_step / deps_satisfied   │  can't-skip (tested)
                    └───────────────────────┬─────────────────────┘
                                            │  single source of truth (.planner/tracker.json)
  omega run    ───► ┌───────────────────────▼─────────────────────┐
                    │ DRIVER — executor.rs               (NEW)     │  ready-set loop,
                    │  spawn  ◄── orchestration::dispatch_task     │  parallel waves,
                    │  wait   ◄── orchestration::wait_for_worker.. │  parallelism cap
                    └───────────────────────┬─────────────────────┘
                                            │  before any mark_done
                    ┌───────────────────────▼─────────────────────┐
                    │ GUARDIAN — guardian.rs             (NEW)     │  Tier 1: verify_command
                    │  T1 deterministic ◄── verifier.rs            │  (always, every step)
                    │  T2 adversarial   ◄── gate.rs MultiGrader    │  Tier 2: ≥2/3 consensus
                    └─────────────────────────────────────────────┘  (high-stakes + audit wave)
```

**Reuse map (what already exists and is wired in):**

| Need | Existing primitive | File |
|---|---|---|
| DAG / ready-set / no-skip | `PlanTracker`, `next_step`, `deps_satisfied`, `start_step`, `is_acyclic` | `planner.rs` |
| Spawn a worker for one step | `Orchestrator::dispatch_task` | `orchestration.rs:292` |
| Event-driven done.json wait | `Orchestrator::wait_for_worker_done` | `orchestration.rs:326` |
| File-scope claim (parallel safety) | `scope::ScopeClaim` | `scope.rs` |
| Run verify commands | `verifier.rs` | `verifier.rs` |
| Adversarial consensus / Popper | `gate.rs` `MultiGrader`, `PopperFalsifier` | `gate.rs` |
| Audit selection | `audit::select_audits` | `audit.rs` |
| Done signal types | `DoneSignal`, `DoneStatus` | `done.rs` |

**New code is small and focused:** `executor.rs` (the loop) + `guardian.rs` (the
verification policy) + minor additions to `planner.rs` + CLI wiring.

---

## 4. Components

### 4.1 `planner.rs` additions (minimal)

Existing `PlanStep` already carries `step_id, phase, title, description,
files_to_touch, done_criteria, verify_command, depends_on, status`. Add:

- `wave: Option<Wave>` — enum `{ Foundation, W1, W2, W3, Audit, Deploy }`. Optional
  sugar over the DAG for the terminal tiers; ordering remains enforced by
  `depends_on`. If absent, the step's wave is inferred from its dependency depth.
- `attempt: u8` — retry counter (Guardian feedback loop).
- `PlanTracker::ready_steps(&self, cap: usize) -> Vec<&PlanStep>` — all `Pending`
  steps whose deps are `Done` **and** whose `files_to_touch` are pairwise disjoint
  within the returned set (safe parallel dispatch), capped at `cap`. Wave gating:
  a step is withheld if an earlier-wave step is still unfinished.

`ready_steps` is the **only** entry point the driver uses to choose work.

### 4.2 `executor.rs` (NEW) — the Driver

```
pub async fn run(project_dir, opts) -> Result<RunReport>:
    tracker = PlanTracker::load(project_dir).ok_or(NoPlan)?
    ensure tracker.is_acyclic()                       # else abort
    loop:
        ready = tracker.ready_steps(opts.parallelism)  # DAG + disjoint files + wave gate
        if ready.is_empty():
            if tracker.status().is_complete(): return SUCCESS
            else: return HALTED { blocked, failed }    # deps Failed upstream
        for step in ready:
            tracker.start_step(step.id)?               # GATE: bails if deps unmet
            ScopeClaim::claim(step.files_to_touch)?    # parallel safety
            spawn = dispatch_task(step_brief(step))    # reuse orchestration spawn
            inflight.push(spawn)
        done = wait_for_any(inflight)                  # reuse wait_for_worker_done
        verdict = Guardian::verify(&tracker, done.step, project_dir).await
        match verdict:
            Pass        => tracker.mark_done(done.step)?
            Retry if a<N => tracker.bump_attempt(); re-dispatch with failure feedback
            Fail        => tracker.mark_failed(done.step)   # downstream stays blocked
        tracker.save(project_dir)?                     # persist after EVERY transition
```

Key properties:
- The LLM is **never** asked "what next?". `ready_steps` decides.
- Parallelism within a wave is bounded and file-disjoint (no two workers touch the
  same file — R-SCOPE).
- Crash-safe: tracker persisted after every transition; `omega run` resumes from
  `.planner/tracker.json`.
- `step_brief(step)` renders the worker prompt from the step's typed fields
  (description, files, criteria, verify_command) + the role-scoped Laws/Rules via
  `rules::agent_context_block(Worker)`. For `audit`-wave steps the brief is
  `/{name}audit …` on line 1, never paraphrased (R-AUDIT).

### 4.3 `guardian.rs` (NEW) — verified completion (R-VERIFY)

A worker claiming `done_clean` is an input. The Guardian decides.

```
pub async fn verify(tracker, step_id, project_dir) -> Verdict:
    step = tracker.get_step(step_id)
    # Tier 1 — deterministic, ALWAYS
    t1 = verifier::run(&step.verify_command, project_dir)   # exit 0 == proof
    if !t1.passed: return Retry|Fail (with t1.output as feedback)
    # Tier 2 — adversarial, conditional
    if step.wave == Audit:
        # audit verdict.json score must clear threshold
        return audit_threshold_check(step, project_dir, min=85)
    if step.is_high_stakes():                # touches auth/payments/schema/deploy
        challenges = gate::MultiGrader::evaluate(step_diff)   # ≥2/3 consensus
        falsify   = gate::PopperFalsifier::validate(challenges)
        return consensus_verdict(challenges, falsify)
    Pass
```

- **Tier 1** closes the "verify_command runs nowhere" gap — every step is proven by
  re-running its own falsifiable command, independent of the worker's claim.
- **Tier 2** escalates to `gate.rs` adversarial consensus for sensitive steps and
  for the whole `audit` wave (verdict ≥85/100).
- `is_high_stakes()` is a typed predicate over `files_to_touch` globs +
  `wave == Deploy`; configurable in `~/.omega/config.toml`.

### 4.4 Plan generation (`omega plan`)

The LLM generates step **content**; Rust validates **structure** and rejects
otherwise (R-RUBRIC — rubric before execution, applied to the plan itself):

- every step has all mandatory fields; `files_to_touch` non-empty;
  `verify_command` present and non-trivial; `depends_on` references existing steps;
  `is_acyclic()`; ≤25 steps/phase.
- Invalid → reject with the specific violation → regenerate. Only a structurally
  valid, acyclic plan is persisted to `.planner/tracker.json`.

---

## 5. CLI surface (Rust = source of truth)

| Command | Purpose |
|---|---|
| `omega plan [idea]` | Generate + validate `PlanTracker` (LLM content, Rust validation) |
| `omega run [project]` | Autonomous Driver loop (gate + driver + guardian) |
| `omega next` | Print the current ready-set (advisory) |
| `omega status` | Progress dashboard (reuse `PlanStatus`) |
| `omega step done\|block\|retry <id>` | Guarded manual overrides |
| `omega replan` | Regenerate from current state, preserving completed steps |

---

## 6. Skills as OmegaOS assets + install parity (L0)

Skills are **OmegaOS-owned repo assets**, not references to live VPS files. They
follow the **existing Quality-Arsenal install pattern** (`install.sh:218-246`).

| Asset | Repo path | install.sh ships to |
|---|---|---|
| `/new` pipeline skill | `skills/new/SKILL.md` | `~/.omega/skills/new/` + stub `~/.claude/commands/new.md` |
| `/planner` skill | `skills/planner/SKILL.md` | `~/.omega/skills/planner/` + stub `~/.claude/commands/planner.md` |
| Bun fallback | `skills/<name>/fallback/*.ts` | `~/.omega/skills/<name>/fallback/` (used iff `omega` binary absent — R-STACK) |
| Engine | `crates/omega-core/src/{executor,guardian}.rs` | `~/.local/bin/omega` (build-from-source, automatic) |

- The slash-command stubs are generated by the **same loop** that already generates
  `/codeaudit` et al. — the stub points the agent at the omega skill / `omega run`.
- `skill_registry.rs` discovers them at runtime from `~/.omega/skills/`.
- `scripts/verify-install.sh` is extended to assert: `omega run --help` and
  `omega plan --help` succeed; `~/.omega/skills/{new,planner}/SKILL.md` exist;
  the `/new` and `/planner` stubs exist in `~/.claude/commands/`.
- **L0 gate:** the change is not "done" until `verify-install.sh` passes and a
  fresh clone+install reproduces it.

**Stack discipline (R-STACK):** Rust = engine/CLI/orchestration; Bun/TS = fallback +
tooling; bash = bootstrap (`install.sh`) only; rmux = worker sessions.

---

## 7. Phase 7 → `audit` wave

Delete `/new` Phase 7 (a separate `/debugaudit` at the end). Instead the plan's
terminal `audit` wave holds **one step per relevant Quality-Arsenal audit**
(`audit::select_audits` already chooses them):

- brief = `/{name}audit …` on line 1 (R-AUDIT, never paraphrased);
- `depends_on` = all implementation steps;
- `verify_command` = the audit's `verdict.json` score ≥ threshold;
- the Guardian gates the audit wave like any other wave (Tier 2, min 85/100).

Audits become DAG citizens, gated and parallel, instead of a bolt-on final phase.

---

## 8. Migration milestones

- **M1 — Build alongside.** `executor.rs` + `guardian.rs` + `planner.rs` additions
  + `omega plan/run/next/status`. Fully unit-tested with a fake spawner. Prove on a
  real micro-project. **`Orchestrator` untouched.** Ship via install.sh + verify.
- **M2 — Cut over.** `Orchestrator::execute()` stops spawning an LLM oracle for
  decomposition; it builds a `PlanTracker` and calls `executor::run()`. The flat
  `for task in plan.tasks` dispatch (lines 116-134) is replaced by the DAG driver.
  `mission::Plan/Task` maps onto `PlanStep` (adapter) or is deprecated.
- **M3 — Unify gates.** The mission-level quality gate (step 6) is subsumed by the
  per-step Guardian plus a final mission-level `gate.rs` pass.

Each milestone is independently shippable and install-parity-verified.

---

## 9. File scope (R-SCOPE — one writer per file)

- **NEW:** `crates/omega-core/src/executor.rs`, `crates/omega-core/src/guardian.rs`
- **EDIT:** `planner.rs` (+`ready_steps`, +`wave`, +`attempt`), `orchestration.rs`
  (M2 only), `crates/omega-cli/src/main.rs` (CLI), `crates/omega-core/src/lib.rs`
  (module decls), `install.sh`, `scripts/verify-install.sh`
- **NEW skill assets:** `skills/new/`, `skills/planner/` (+ Bun fallback)
- **DO NOT TOUCH (concurrent edit elsewhere):** `app.rs`, `ui.rs`, `agents.rs`,
  `providers.rs`. The orchestration work has zero overlap with the TUI/agents work.

---

## 10. Testing strategy

- **executor (unit, fake spawner):** ready-set ordering respects the DAG; parallel
  set is file-disjoint; guardian-fail → retry → block; wave advance only when prior
  wave is 100% `done`; resume from a persisted half-done tracker.
- **guardian (unit):** Tier-1 pass/fail on a real `verify_command`; Tier-2
  consensus mocked; audit-threshold check.
- **planner (unit, existing + new):** keep the existing `cannot_skip_steps`,
  `sequential_execution`, `acyclic_check`; add `ready_steps` disjointness +
  wave-gating tests.
- **integration:** a 3-step plan run end-to-end against a stub agent.
- **live (L1):** run `omega run` on a real micro-project; observe in logs that a
  downstream step is **never** started before its deps are `done` — runtime proof,
  not prose.

---

## 11. Risks & open questions

- **R1 — Worker brief fidelity.** A step brief must be self-sufficient for a
  one-shot worker. Mitigation: render from typed fields + reference_files; the
  granularity gut-check (one task = one worker-session) stays in the `/planner`
  generation prompt.
- **R2 — Parallelism vs file collisions.** Mitigated by `ScopeClaim` + disjoint
  `files_to_touch`; a step whose files intersect a running step is withheld from
  the ready-set.
- **R3 — Guardian cost.** Tier 2 (adversarial) is expensive; gated to high-stakes
  steps + audit wave only. Tier 1 (verify_command) is cheap and universal.
- **OQ1 — Explicit waves vs DAG-emergent.** Proposed: DAG is the enforced core;
  `wave` is optional sugar for terminal tiers (audit/deploy). Confirm during review.
- **OQ2 — `mission::Plan/Task` fate at M2.** Adapter vs deprecation — decide when
  M2 is planned.

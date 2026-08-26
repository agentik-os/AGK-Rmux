# Orchestration Engine M1 — Gate wiring + Driver + Guardian Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wire the orphaned DAG `PlanTracker` into a new autonomous `executor.rs` Driver gated by a `guardian.rs` verified-completion check, so a plan runs task-by-task without an LLM in the dispatch decision and no step is marked done without independent proof.

**Architecture:** Build the engine **alongside** the existing `Orchestrator` (untouched in M1). `executor::run` selects work only via `PlanTracker::ready_steps` (DAG + file-disjoint + wave gate), spawns one worker per step through a `WorkerRuntime` trait (real `RmuxRuntime`, test `FakeRuntime`), and before marking any step done runs `Guardian::verify` (Tier 1 = re-run the step's `verify_command` via the existing `IntentVerifier`). Source of truth: `.planner/tracker.json`.

**Tech Stack:** Rust (omega-core lib + omega-cli), tokio async, serde, the existing `planner.rs` / `verifier.rs` / `done.rs` / `scope.rs` / `session.rs` modules. Tests with `tempfile`. No new dependencies.

**Reference spec:** `docs/specs/2026-05-31-orchestration-engine-design.md`

**Decisions locked (from brainstorming):** OQ1 → DAG is the enforced core, `wave` is optional sugar for terminal tiers (audit/deploy). OQ2 → `mission::Plan/Task` adapter deferred to M2. M1 Guardian = Tier 1 (deterministic `verify_command`); Tier 2 (gate.rs adversarial consensus) is a named follow-on (Task 9), not in the M1 critical path.

**Out of scope for M1 (separate plans):** install-parity + skills-as-assets packaging; M2 Orchestrator cutover; M3 gate unification.

---

## File Structure

| File | Responsibility | Action |
|---|---|---|
| `crates/omega-core/src/planner.rs` | DAG tracker; add `Wave`, `wave`/`attempt` fields, `ready_steps`, `bump_attempt`, `reset_to_pending` | Modify |
| `crates/omega-core/src/guardian.rs` | Verified-completion policy: `Verdict`, `Guardian`, Tier-1 verify via `IntentVerifier` | Create |
| `crates/omega-core/src/executor.rs` | The Driver: `WorkerRuntime` trait, `FakeRuntime`, `RmuxRuntime`, `RunReport`, `run` loop | Create |
| `crates/omega-core/src/lib.rs` | Register `guardian` + `executor` modules | Modify |
| `crates/omega-cli/src/main.rs` | `omega plan-run` / `omega plan-status` CLI surface over the engine | Modify |

**Boundary note:** Do NOT touch `app.rs`, `ui.rs`, `agents.rs`, `providers.rs` (concurrent edits elsewhere — R-SCOPE). `orchestration.rs` stays untouched in M1.

---

## Task 1: Add `Wave` enum + step fields to planner.rs

**Files:**
- Modify: `crates/omega-core/src/planner.rs` (PlanStep struct ~13-26, builder ~436-487)
- Test: same file `#[cfg(test)] mod tests`

- [ ] **Step 1: Write the failing test**

Add to the `tests` module in `planner.rs`:

```rust
#[test]
fn wave_and_attempt_defaults() {
    let s = step("STEP-001", 1)
        .title("X").criteria("ok").verify("true").build();
    assert_eq!(s.attempt, 0);
    assert!(s.wave.is_none());
}

#[test]
fn wave_ordinal_ordering() {
    assert!(Wave::Foundation.ordinal() < Wave::W1.ordinal());
    assert!(Wave::W1.ordinal() < Wave::Audit.ordinal());
    assert!(Wave::Audit.ordinal() < Wave::Deploy.ordinal());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p omega-core planner::tests::wave_ -- --nocapture`
Expected: FAIL — `Wave` not found, `attempt`/`wave` fields missing.

- [ ] **Step 3: Add the `Wave` enum + fields + builder support**

Add after the `StepStatus` impl (around line 58):

```rust
/// Optional scheduling tier. The DAG (`depends_on`) is the enforced order;
/// `Wave` is sugar that lets the driver hold terminal tiers (audit/deploy)
/// until all implementation work is done.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Wave {
    Foundation,
    W1,
    W2,
    W3,
    Audit,
    Deploy,
}

impl Wave {
    pub fn ordinal(&self) -> u8 {
        match self {
            Wave::Foundation => 0,
            Wave::W1 => 1,
            Wave::W2 => 1,
            Wave::W3 => 1,
            Wave::Audit => 2,
            Wave::Deploy => 3,
        }
    }
    pub fn is_terminal(&self) -> bool {
        matches!(self, Wave::Audit | Wave::Deploy)
    }
}
```

In `PlanStep` (struct around line 13), add two fields after `depends_on`:

```rust
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub wave: Option<Wave>,
    #[serde(default)]
    pub attempt: u8,
    pub status: StepStatus,
```

In `PlanStepBuilder` (struct around line 436) add `wave: Option<Wave>,` to the struct fields and initialize it in `pub fn step(...)` (around line 423) with `wave: None,`. Add a builder method:

```rust
    pub fn wave(mut self, w: Wave) -> Self {
        self.wave = Some(w);
        self
    }
```

In `PlanStepBuilder::build` (around line 472) set the new fields:

```rust
            depends_on: self.depends_on,
            wave: self.wave,
            attempt: 0,
            status: StepStatus::Pending,
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p omega-core planner::tests::wave_ -- --nocapture`
Expected: PASS (both tests).

- [ ] **Step 5: Commit**

```bash
git add crates/omega-core/src/planner.rs
git commit -m "feat(planner): add optional Wave tier + attempt counter to PlanStep"
```

---

## Task 2: `ready_steps` — DAG + file-disjoint + wave gate

**Files:**
- Modify: `crates/omega-core/src/planner.rs` (impl PlanTracker, after `next_step` ~179)
- Test: same file

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn ready_steps_respects_dag() {
    let tracker = sample_tracker(); // STEP-001, 002(dep 001), 003(dep 002)
    let ready = tracker.ready_steps(10);
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].step_id, "STEP-001");
}

#[test]
fn ready_steps_parallel_disjoint_files() {
    let mut t = PlanTracker::new("P");
    let p = t.add_phase("F", "g");
    t.add_step(p, step("A", p).title("a").files(&["a.rs"]).criteria("ok").verify("true").build()).unwrap();
    t.add_step(p, step("B", p).title("b").files(&["b.rs"]).criteria("ok").verify("true").build()).unwrap();
    t.add_step(p, step("C", p).title("c").files(&["a.rs"]).criteria("ok").verify("true").build()).unwrap();
    // A and B are disjoint → both ready; C shares a.rs with A → withheld this round
    let ready: Vec<_> = t.ready_steps(10).iter().map(|s| s.step_id.clone()).collect();
    assert_eq!(ready, vec!["A".to_string(), "B".to_string()]);
}

#[test]
fn ready_steps_cap() {
    let mut t = PlanTracker::new("P");
    let p = t.add_phase("F", "g");
    for i in 0..5 {
        t.add_step(p, step(&format!("S{i}"), p).title("x")
            .files(&[&format!("f{i}.rs")]).criteria("ok").verify("true").build()).unwrap();
    }
    assert_eq!(t.ready_steps(2).len(), 2);
}

#[test]
fn ready_steps_holds_audit_until_impl_done() {
    let mut t = PlanTracker::new("P");
    let p = t.add_phase("F", "g");
    t.add_step(p, step("IMPL", p).title("impl").files(&["x.rs"]).criteria("ok").verify("true").build()).unwrap();
    t.add_step(p, step("AUD", p).title("audit").files(&["y.rs"]).criteria("ok").verify("true")
        .wave(Wave::Audit).build()).unwrap();
    // IMPL (no wave) ready; AUD (audit) withheld until all non-terminal steps done
    let ready: Vec<_> = t.ready_steps(10).iter().map(|s| s.step_id.clone()).collect();
    assert_eq!(ready, vec!["IMPL".to_string()]);
    t.start_step("IMPL").unwrap();
    t.mark_done("IMPL").unwrap();
    let ready2: Vec<_> = t.ready_steps(10).iter().map(|s| s.step_id.clone()).collect();
    assert_eq!(ready2, vec!["AUD".to_string()]);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p omega-core planner::tests::ready_steps -- --nocapture`
Expected: FAIL — `ready_steps` not found.

- [ ] **Step 3: Implement `ready_steps` + `wave_open`**

Add inside `impl PlanTracker` after `next_step` (around line 179):

```rust
    /// All pending steps whose deps are Done, whose wave is open, and whose
    /// files_to_touch are pairwise disjoint within the returned set — capped.
    /// This is the ONLY selector the driver uses. Parallel-safe.
    pub fn ready_steps(&self, cap: usize) -> Vec<&PlanStep> {
        let mut out: Vec<&PlanStep> = Vec::new();
        let mut claimed: Vec<&str> = Vec::new();
        for s in &self.steps {
            if out.len() >= cap {
                break;
            }
            if s.status != StepStatus::Pending {
                continue;
            }
            if !self.deps_satisfied(&s.step_id) {
                continue;
            }
            if !self.wave_open(s) {
                continue;
            }
            let disjoint = s
                .files_to_touch
                .iter()
                .all(|f| !claimed.contains(&f.as_str()));
            if !disjoint {
                continue;
            }
            for f in &s.files_to_touch {
                claimed.push(f.as_str());
            }
            out.push(s);
        }
        out
    }

    /// A terminal-tier step (audit/deploy) is withheld until every step of a
    /// strictly-lower wave ordinal is Done. Non-waved steps are gated only by
    /// the DAG (always "open" here).
    fn wave_open(&self, step: &PlanStep) -> bool {
        let Some(w) = step.wave else {
            return true;
        };
        if !w.is_terminal() {
            return true;
        }
        let my = w.ordinal();
        self.steps.iter().all(|other| {
            let other_ord = other.wave.map(|x| x.ordinal()).unwrap_or(0);
            other_ord >= my || other.status == StepStatus::Done
        })
    }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p omega-core planner::tests::ready_steps -- --nocapture`
Expected: PASS (all four).

- [ ] **Step 5: Commit**

```bash
git add crates/omega-core/src/planner.rs
git commit -m "feat(planner): ready_steps selector (DAG + file-disjoint + wave gate)"
```

---

## Task 3: `bump_attempt` + `reset_to_pending` (retry plumbing)

**Files:**
- Modify: `crates/omega-core/src/planner.rs` (impl PlanTracker, after `mark_failed` ~240)
- Test: same file

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn retry_resets_to_pending_and_bumps_attempt() {
    let mut t = sample_tracker();
    t.start_step("STEP-001").unwrap();
    t.bump_attempt("STEP-001").unwrap();
    t.reset_to_pending("STEP-001").unwrap();
    let s = t.get_step("STEP-001").unwrap();
    assert_eq!(s.attempt, 1);
    assert_eq!(s.status, StepStatus::Pending);
    // can be selected again
    assert_eq!(t.ready_steps(10)[0].step_id, "STEP-001");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p omega-core planner::tests::retry_resets -- --nocapture`
Expected: FAIL — methods not found.

- [ ] **Step 3: Implement the two methods**

Add inside `impl PlanTracker` after `mark_failed` (around line 240):

```rust
    /// Increment the retry counter for a step.
    pub fn bump_attempt(&mut self, step_id: &str) -> Result<()> {
        let step = self.get_step_mut(step_id).context("step not found")?;
        step.attempt = step.attempt.saturating_add(1);
        Ok(())
    }

    /// Return an in-progress step to pending so it can be re-dispatched.
    pub fn reset_to_pending(&mut self, step_id: &str) -> Result<()> {
        let step = self.get_step_mut(step_id).context("step not found")?;
        step.status = StepStatus::Pending;
        step.started_at = None;
        Ok(())
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p omega-core planner::tests::retry_resets -- --nocapture`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/omega-core/src/planner.rs
git commit -m "feat(planner): bump_attempt + reset_to_pending for guardian retry loop"
```

---

## Task 4: Guardian — `Verdict` enum + skeleton

**Files:**
- Create: `crates/omega-core/src/guardian.rs`
- Modify: `crates/omega-core/src/lib.rs` (add `pub mod guardian;`)
- Test: in `guardian.rs`

- [ ] **Step 1: Create the module with the `Verdict` type + a unit test**

Create `crates/omega-core/src/guardian.rs`:

```rust
//! Guardian — verified completion. A worker's own `done.json` is an input,
//! never the verdict (R-VERIFY). Before any step is marked Done, the Guardian
//! independently proves it.
//!
//! Tier 1 (M1, always): re-run the step's `verify_command` via IntentVerifier.
//! Tier 2 (follow-on): adversarial consensus via gate.rs for high-stakes steps.

use crate::planner::PlanStep;
use crate::verifier::{IntentSpec, IntentVerifier};
use std::path::Path;

/// Outcome of an independent verification of a completed step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// Proven done — safe to mark the step Done.
    Pass,
    /// Not proven, but attempts remain — re-dispatch with this feedback.
    Retry { feedback: String },
    /// Not proven and out of attempts — mark the step Failed.
    Fail { reason: String },
}

pub struct Guardian {
    max_attempts: u8,
}

impl Guardian {
    pub fn new(max_attempts: u8) -> Self {
        Self {
            max_attempts: max_attempts.max(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guardian_min_one_attempt() {
        let g = Guardian::new(0);
        assert_eq!(g.max_attempts, 1);
    }
}
```

Add to `crates/omega-core/src/lib.rs` (alphabetical with the other `pub mod` lines):

```rust
pub mod guardian;
```

- [ ] **Step 2: Run test to verify it compiles + passes**

Run: `cargo test -p omega-core guardian::tests::guardian_min_one_attempt -- --nocapture`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add crates/omega-core/src/guardian.rs crates/omega-core/src/lib.rs
git commit -m "feat(guardian): Verdict enum + Guardian skeleton"
```

---

## Task 5: Guardian Tier 1 — independent `verify_command`

**Files:**
- Modify: `crates/omega-core/src/guardian.rs`
- Test: same file

- [ ] **Step 1: Write the failing tests**

Add a builder helper + tests to the `tests` module:

```rust
    use crate::planner::step;

    fn done_step(verify: &str) -> PlanStep {
        step("STEP-001", 1)
            .title("Create thing")
            .description("make the thing")
            .files(&["thing.rs"])
            .criteria("compiles")
            .verify(verify)
            .build()
    }

    #[tokio::test]
    async fn tier1_pass_when_command_exits_zero() {
        let g = Guardian::new(2);
        let dir = tempfile::tempdir().unwrap();
        let s = done_step("true");
        assert_eq!(g.verify(&s, dir.path(), 1).await, Verdict::Pass);
    }

    #[tokio::test]
    async fn tier1_retry_then_fail_when_command_exits_nonzero() {
        let g = Guardian::new(2);
        let dir = tempfile::tempdir().unwrap();
        let s = done_step("false");
        // attempt 1 of 2 → Retry
        assert!(matches!(g.verify(&s, dir.path(), 1).await, Verdict::Retry { .. }));
        // attempt 2 of 2 → Fail
        assert!(matches!(g.verify(&s, dir.path(), 2).await, Verdict::Fail { .. }));
    }

    #[tokio::test]
    async fn tier1_fail_on_empty_verify_command() {
        let g = Guardian::new(2);
        let dir = tempfile::tempdir().unwrap();
        let s = done_step("   ");
        assert!(matches!(g.verify(&s, dir.path(), 1).await, Verdict::Fail { .. }));
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p omega-core guardian::tests::tier1 -- --nocapture`
Expected: FAIL — `verify` method not found.

- [ ] **Step 3: Implement `Guardian::verify` (Tier 1)**

Add inside `impl Guardian`:

```rust
    /// Tier 1 — deterministic. Re-run the step's verify_command in the project
    /// dir, independent of the worker's claim. `attempt` is 1-based.
    pub async fn verify(&self, step: &PlanStep, project_dir: &Path, attempt: u8) -> Verdict {
        if step.verify_command.trim().is_empty() {
            return Verdict::Fail {
                reason: format!("step {} has an empty verify_command", step.step_id),
            };
        }
        let intent = IntentSpec {
            action: step.title.clone(),
            target: step.files_to_touch.join(","),
            success_criteria: vec![step.done_criteria.clone()],
            verify_commands: vec![step.verify_command.clone()],
        };
        let verifier = IntentVerifier::default();
        match verifier.verify_once(&intent, project_dir, attempt as u32).await {
            Ok(result) => {
                let proven = !result.command_results.is_empty()
                    && result.command_results.iter().all(|c| c.passed);
                if proven {
                    Verdict::Pass
                } else if attempt < self.max_attempts {
                    let feedback = result
                        .command_results
                        .iter()
                        .map(|c| {
                            format!("$ {}\nexit {}\n{}", c.command, c.exit_code, c.stderr.trim())
                        })
                        .collect::<Vec<_>>()
                        .join("\n---\n");
                    Verdict::Retry { feedback }
                } else {
                    Verdict::Fail {
                        reason: format!(
                            "verify_command failed after {} attempt(s): {}",
                            attempt, step.verify_command
                        ),
                    }
                }
            }
            Err(e) => Verdict::Fail {
                reason: format!("guardian could not run verify_command: {e}"),
            },
        }
    }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p omega-core guardian::tests::tier1 -- --nocapture`
Expected: PASS (all three).

- [ ] **Step 5: Commit**

```bash
git add crates/omega-core/src/guardian.rs
git commit -m "feat(guardian): Tier-1 deterministic verify_command (R-VERIFY)"
```

---

## Task 6: Executor — `WorkerRuntime` trait + `FakeRuntime`

**Files:**
- Create: `crates/omega-core/src/executor.rs`
- Modify: `crates/omega-core/src/lib.rs` (add `pub mod executor;`)
- Test: in `executor.rs`

- [ ] **Step 1: Create the module: trait, options, report, FakeRuntime + a test**

Create `crates/omega-core/src/executor.rs`:

```rust
//! Executor — the autonomous Driver. Selects work ONLY via
//! PlanTracker::ready_steps (the LLM is never asked "what next?"), spawns one
//! worker per step through a WorkerRuntime, and gates every completion through
//! the Guardian before marking a step Done.

use crate::done::DoneSignal;
use crate::guardian::{Guardian, Verdict};
use crate::planner::{PlanStep, PlanTracker, StepStatus};
use anyhow::{bail, Context, Result};
use std::path::Path;
use std::time::Duration;

/// Pluggable worker backend — real rmux sessions in prod, scripted in tests.
pub trait WorkerRuntime {
    /// Spawn a worker for `step`; return its session name.
    async fn spawn(&self, step: &PlanStep, brief: &str, cwd: &Path) -> Result<String>;
    /// Block until the worker's done.json appears (or timeout).
    async fn wait_done(&self, session: &str, timeout: Duration) -> Result<DoneSignal>;
}

#[derive(Debug, Clone)]
pub struct RunOptions {
    pub parallelism: usize,
    pub max_attempts: u8,
    pub worker_timeout: Duration,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            parallelism: 3,
            max_attempts: 2,
            worker_timeout: Duration::from_secs(60 * 30),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct RunReport {
    pub completed: Vec<String>,
    pub failed: Vec<String>,
    pub blocked: Vec<String>,
    pub success: bool,
}

/// Render the worker prompt for one step from its typed fields.
pub fn render_brief(step: &PlanStep) -> String {
    format!(
        "{title}\n\n{description}\n\nFiles you own (touch ONLY these): {files}\n\nDone criteria: {criteria}\n\nVerify before done.json: {verify}",
        title = step.title,
        description = step.description,
        files = step.files_to_touch.join(", "),
        criteria = step.done_criteria,
        verify = step.verify_command,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::done::DoneStatus;
    use std::collections::HashMap;

    /// Scripts a DoneStatus per step_id; session names are "fake-<step_id>".
    pub struct FakeRuntime {
        pub script: HashMap<String, DoneStatus>,
    }

    impl WorkerRuntime for FakeRuntime {
        async fn spawn(&self, step: &PlanStep, _brief: &str, _cwd: &Path) -> Result<String> {
            Ok(format!("fake-{}", step.step_id))
        }
        async fn wait_done(&self, session: &str, _t: Duration) -> Result<DoneSignal> {
            let step_id = session.strip_prefix("fake-").unwrap_or(session);
            let status = self
                .script
                .get(step_id)
                .cloned()
                .unwrap_or(DoneStatus::DoneClean);
            Ok(DoneSignal::stub(step_id, status))
        }
    }

    #[test]
    fn render_brief_includes_files_and_verify() {
        use crate::planner::step;
        let s = step("STEP-001", 1)
            .title("T").description("D").files(&["a.rs"]).criteria("ok").verify("true").build();
        let b = render_brief(&s);
        assert!(b.contains("a.rs"));
        assert!(b.contains("true"));
    }
}
```

Add to `crates/omega-core/src/lib.rs`:

```rust
pub mod executor;
```

- [ ] **Step 2: Add a `DoneSignal::stub` test constructor**

The `FakeRuntime` needs to build a `DoneSignal`. Open `crates/omega-core/src/done.rs`, find the `DoneSignal` struct + `DoneStatus`, and add a cheap constructor guarded for tests/use. Add inside `impl DoneSignal`:

```rust
    /// Minimal constructor for tests and synthetic signals.
    pub fn stub(session: &str, status: DoneStatus) -> Self {
        Self {
            session: session.to_string(),
            status,
            summary: String::new(),
            commit: None,
            ..Default::default()
        }
    }
```

If `DoneSignal` does not derive `Default`, instead fill every field explicitly. **Before writing, read `done.rs` to get the exact field list** and match it. The constructor must set `session`, `status`, and zero/empty the rest.

- [ ] **Step 3: Run test to verify it compiles + passes**

Run: `cargo test -p omega-core executor::tests::render_brief -- --nocapture`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add crates/omega-core/src/executor.rs crates/omega-core/src/done.rs crates/omega-core/src/lib.rs
git commit -m "feat(executor): WorkerRuntime trait + FakeRuntime + render_brief"
```

---

## Task 7: Executor — the `run` loop (the Driver)

**Files:**
- Modify: `crates/omega-core/src/executor.rs`
- Test: same file

- [ ] **Step 1: Write the failing integration tests**

Add to the `tests` module a helper that builds a 3-step linear plan and saves it, plus two tests:

```rust
    use crate::planner::{step, PlanTracker};

    fn save_linear_plan(dir: &Path, verify: &str) {
        let mut t = PlanTracker::new("P");
        let p = t.add_phase("F", "g");
        t.add_step(p, step("STEP-001", p).title("a").files(&["a.rs"]).criteria("ok").verify(verify).build()).unwrap();
        t.add_step(p, step("STEP-002", p).title("b").files(&["b.rs"]).criteria("ok").verify(verify).depends(&["STEP-001"]).build()).unwrap();
        t.add_step(p, step("STEP-003", p).title("c").files(&["c.rs"]).criteria("ok").verify(verify).depends(&["STEP-002"]).build()).unwrap();
        t.save(dir).unwrap();
    }

    #[tokio::test]
    async fn run_completes_all_steps_in_order() {
        let dir = tempfile::tempdir().unwrap();
        save_linear_plan(dir.path(), "true"); // guardian passes
        let rt = FakeRuntime { script: HashMap::new() }; // all DoneClean
        let report = run(dir.path(), &rt, RunOptions::default()).await.unwrap();
        assert!(report.success);
        assert_eq!(report.completed, vec!["STEP-001", "STEP-002", "STEP-003"]);
        // persisted tracker reflects completion
        let t = PlanTracker::load(dir.path()).unwrap();
        assert!(t.status().is_complete());
    }

    #[tokio::test]
    async fn guardian_overrides_worker_done_claim() {
        // Worker SAYS DoneClean, but verify_command is `false` → Guardian blocks.
        let dir = tempfile::tempdir().unwrap();
        save_linear_plan(dir.path(), "false");
        let rt = FakeRuntime { script: HashMap::new() }; // worker claims DoneClean
        let report = run(dir.path(), &rt, RunOptions { max_attempts: 1, ..Default::default() }).await.unwrap();
        assert!(!report.success);
        assert_eq!(report.failed, vec!["STEP-001"]);
        // STEP-002/003 never ran — gated behind the failed dep
        assert!(report.completed.is_empty());
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p omega-core executor::tests::run_ executor::tests::guardian_overrides -- --nocapture`
Expected: FAIL — `run` not found.

- [ ] **Step 3: Implement `run`**

Add at module top-level in `executor.rs` (after `render_brief`):

```rust
/// Drive a plan to completion. Selects work via ready_steps only; gates every
/// completion through the Guardian. Resumable: persists after each transition.
pub async fn run<R: WorkerRuntime>(
    project_dir: &Path,
    runtime: &R,
    opts: RunOptions,
) -> Result<RunReport> {
    let mut tracker = PlanTracker::load(project_dir)
        .context("no .planner/tracker.json — run `omega plan` first")?;
    if !tracker.is_acyclic() {
        bail!("plan DAG contains a cycle — aborting");
    }
    let guardian = Guardian::new(opts.max_attempts);
    let mut report = RunReport::default();

    loop {
        let ready: Vec<String> = tracker
            .ready_steps(opts.parallelism)
            .iter()
            .map(|s| s.step_id.clone())
            .collect();

        if ready.is_empty() {
            let st = tracker.status();
            report.success = st.is_complete();
            if !st.is_complete() {
                // record what is stuck
                for s in &tracker.steps {
                    match s.status {
                        StepStatus::Failed => report.failed.push(s.step_id.clone()),
                        StepStatus::Pending => report.blocked.push(s.step_id.clone()),
                        _ => {}
                    }
                }
            }
            return Ok(report);
        }

        // Spawn the whole ready-set (workers run concurrently in the backend).
        let mut inflight: Vec<(String, String)> = Vec::new();
        for sid in &ready {
            tracker.start_step(sid)?; // GATE: bails if deps not satisfied
            let step = tracker.get_step(sid).context("step vanished")?.clone();
            let brief = render_brief(&step);
            let session = runtime.spawn(&step, &brief, project_dir).await?;
            inflight.push((sid.clone(), session));
        }
        tracker.save(project_dir)?;

        // Collect + Guardian-gate each.
        for (sid, session) in inflight {
            let _done: DoneSignal = runtime.wait_done(&session, opts.worker_timeout).await?;
            let step = tracker.get_step(&sid).context("step vanished")?.clone();
            let attempt = step.attempt + 1;
            match guardian.verify(&step, project_dir, attempt).await {
                Verdict::Pass => {
                    tracker.mark_done(&sid)?;
                    report.completed.push(sid.clone());
                }
                Verdict::Retry { feedback } => {
                    tracing::warn!(step = %sid, %feedback, "guardian: retry");
                    tracker.bump_attempt(&sid)?;
                    tracker.reset_to_pending(&sid)?; // re-selected next loop
                }
                Verdict::Fail { reason } => {
                    tracing::error!(step = %sid, %reason, "guardian: fail");
                    tracker.mark_failed(&sid)?;
                    report.failed.push(sid.clone());
                }
            }
            tracker.save(project_dir)?;
        }
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p omega-core executor::tests -- --nocapture`
Expected: PASS — `run_completes_all_steps_in_order` proves DAG order; `guardian_overrides_worker_done_claim` proves the Guardian beats the worker's `done_clean` claim and downstream steps stay gated.

- [ ] **Step 5: Commit**

```bash
git add crates/omega-core/src/executor.rs
git commit -m "feat(executor): autonomous run loop (ready-set -> spawn -> guardian -> advance)"
```

---

## Task 8: `RmuxRuntime` — the real worker backend

**Files:**
- Modify: `crates/omega-core/src/executor.rs`

- [ ] **Step 1: Read the exact APIs to call**

Before writing, read these to confirm signatures (do not guess):
- `crates/omega-core/src/session.rs` → `SessionManager::create_session_with_agent(name, cwd: Option<&str>, agent: Agent, prompt: Option<&str>)` (used at `orchestration.rs:313`).
- `crates/omega-core/src/scope.rs` → `claim_or_reject(state_dir, session_name, files: Vec<String>)`.
- `crates/omega-core/src/done.rs` → `DoneSignal` shape + the `worker-{session}.done.json` path convention (`orchestration.rs:336`).

- [ ] **Step 2: Implement `RmuxRuntime` (no unit test — covered by the live run in Task 10)**

Add to `executor.rs`:

```rust
use crate::agents::Agent;
use crate::scope;
use crate::session::SessionManager;
use std::path::PathBuf;
use std::time::Instant;

/// Real backend: spawns rmux worker sessions and waits on their done.json.
pub struct RmuxRuntime<'a> {
    pub mgr: &'a SessionManager,
    pub state_dir: PathBuf,
    pub project: String,
    pub agent: Agent,
    pub poll: Duration,
}

impl<'a> WorkerRuntime for RmuxRuntime<'a> {
    async fn spawn(&self, step: &PlanStep, brief: &str, cwd: &Path) -> Result<String> {
        let session = format!("{}-step-{}", self.project, step.step_id.to_lowercase());
        if !step.files_to_touch.is_empty() {
            scope::claim_or_reject(&self.state_dir, &session, step.files_to_touch.clone())
                .with_context(|| format!("scope claim for {session}"))?;
        }
        self.mgr
            .create_session_with_agent(
                &session,
                Some(&cwd.to_string_lossy()),
                self.agent,
                Some(brief),
            )
            .await
            .with_context(|| format!("spawning {session}"))?;
        Ok(session)
    }

    async fn wait_done(&self, session: &str, timeout: Duration) -> Result<DoneSignal> {
        let done_path = self.state_dir.join(format!("worker-{session}.done.json"));
        let deadline = Instant::now() + timeout;
        loop {
            if done_path.exists() {
                let content = std::fs::read_to_string(&done_path)?;
                let signal: DoneSignal = serde_json::from_str(&content)?;
                return Ok(signal);
            }
            if Instant::now() >= deadline {
                bail!("worker {session} timed out after {}s", timeout.as_secs());
            }
            tokio::time::sleep(self.poll).await;
        }
    }
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build -p omega-core`
Expected: builds clean (warnings about unused `RmuxRuntime` are fine until Task 10 wires it).

- [ ] **Step 4: Commit**

```bash
git add crates/omega-core/src/executor.rs
git commit -m "feat(executor): RmuxRuntime real backend (session spawn + done.json wait)"
```

---

## Task 9: (Follow-on, named) Guardian Tier 2 — adversarial consensus

**Status:** specified, NOT in the M1 critical path. Implement after M1 lands.

**Files:**
- Modify: `crates/omega-core/src/guardian.rs`

- [ ] **Step 1:** Read `crates/omega-core/src/gate.rs:242-311` to get the exact `MultiGrader::evaluate` signature, `GradeResult`, `AdversarialChallenge`, and `PopperFalsifier::validate`.
- [ ] **Step 2:** Add `is_high_stakes(step) -> bool` (true if any `files_to_touch` glob matches `**/auth/**`, `**/payment*/**`, `**/schema*`, or `step.wave == Some(Wave::Deploy)`).
- [ ] **Step 3:** Add an `audit_threshold` check for `step.wave == Some(Wave::Audit)`: read the audit's `audits/.<name>/verdict.json`, pass iff score ≥ 85.
- [ ] **Step 4:** In `Guardian::verify`, after a Tier-1 Pass, escalate to Tier 2 when `is_high_stakes(step)` or the step is an audit-wave step. Return the consensus verdict.
- [ ] **Step 5:** Tests with mocked grades; commit.

---

## Task 10: CLI surface + live verification

**Files:**
- Modify: `crates/omega-cli/src/main.rs`

- [ ] **Step 1: Read the existing command enum**

Read `crates/omega-cli/src/main.rs:13-130` (the `#[derive(Subcommand)]` enum) to match the existing style, then add two subcommands following that exact pattern:
- `PlanRun { #[arg(default_value = ".")] path: String }` → calls `executor::run`.
- `PlanStatus { #[arg(default_value = ".")] path: String }` → loads `PlanTracker`, prints `PlanStatus`.

- [ ] **Step 2: Implement `cmd_plan_status` (read-only, safe to verify first)**

Add a handler:

```rust
fn cmd_plan_status(path: &str) -> anyhow::Result<()> {
    let dir = std::path::Path::new(path);
    let tracker = omega_core::planner::PlanTracker::load(dir)
        .ok_or_else(|| anyhow::anyhow!("no .planner/tracker.json in {path}"))?;
    let st = tracker.status();
    println!(
        "Plan: {} | {:.0}% ({}/{} done) | ready {} | blocked {} | failed {} | phase {}/{}",
        tracker.project, st.progress_pct(), st.done, st.total,
        st.ready, st.blocked, st.failed, st.active_phase, st.total_phases,
    );
    for s in &tracker.steps {
        println!("  {} {} {}", s.status.icon(), s.step_id, s.title);
    }
    Ok(())
}
```

- [ ] **Step 3: Implement `cmd_plan_run` wiring `RmuxRuntime`**

```rust
async fn cmd_plan_run(path: &str) -> anyhow::Result<()> {
    use omega_core::executor::{run, RmuxRuntime, RunOptions};
    let dir = std::path::Path::new(path);
    let config = omega_core::config::OmegaConfig::load().unwrap_or_default();
    let mgr = omega_core::session::SessionManager::connect().await?; // match the real ctor used elsewhere
    let project = dir.file_name().and_then(|n| n.to_str()).unwrap_or("project").to_string();
    let runtime = RmuxRuntime {
        mgr: &mgr,
        state_dir: config.state_dir.clone(),
        project,
        agent: omega_core::agents::Agent::Claude,
        poll: std::time::Duration::from_secs(5),
    };
    let report = run(dir, &runtime, RunOptions::default()).await?;
    println!(
        "Run finished: success={} | completed {} | failed {:?} | blocked {:?}",
        report.success, report.completed.len(), report.failed, report.blocked,
    );
    if !report.success {
        std::process::exit(1);
    }
    Ok(())
}
```

**Note:** match `SessionManager`'s real constructor — read `session.rs` / how `Orchestrator::new` builds `self.mgr` and copy that exact call. Replace `SessionManager::connect()` if the real ctor differs.

- [ ] **Step 4: Wire the match arms** in the command dispatch (follow the existing `Action::...` arms), then build:

Run: `cargo build --release`
Expected: builds clean.

- [ ] **Step 5: Live verification (L1 — runtime is the only truth)**

```bash
# Create a tiny throwaway project with a 3-step plan whose verify_commands are real.
cd /tmp && rm -rf omega-run-smoke && mkdir omega-run-smoke && cd omega-run-smoke
# (hand-write .planner/tracker.json: STEP-001 -> 002 -> 003, each verify_command="test -f stepN.flag";
#  the worker brief tells the agent to `touch stepN.flag`. Or, for a pure engine smoke,
#  set verify_command="true" and use a stub agent.)
omega plan-status .
omega plan-run .
```

Expected runtime evidence: `plan-status` shows steps; `plan-run` advances STEP-001 → 002 → 003 **in order**, and the logs show STEP-002 is never started before STEP-001 is Done. Capture the log lines as proof.

- [ ] **Step 6: Commit**

```bash
git add crates/omega-cli/src/main.rs
git commit -m "feat(cli): omega plan-run + plan-status driving the executor"
```

---

## Task 11: Full suite green + final commit

- [ ] **Step 1: Run the whole workspace test suite**

Run: `cargo test --workspace`
Expected: all green, including the new planner/guardian/executor tests and the pre-existing 169.

- [ ] **Step 2: Lint**

Run: `cargo clippy --workspace --all-targets -- -D warnings`
Expected: no warnings in the new modules (fix any). Pre-existing dead-code warnings elsewhere are out of scope.

- [ ] **Step 3: Commit any lint fixes**

```bash
git add -A
git commit -m "chore(executor): clippy clean for orchestration engine M1"
```

---

## Self-Review (completed by plan author)

**Spec coverage:** Gate (PlanTracker.ready_steps) → Tasks 1-3. Driver (executor.run) → Tasks 6-8, 10. Guardian Tier 1 → Tasks 4-5; Tier 2 → Task 9 (named follow-on, per locked decision). "Can't-skip by construction" proved by `guardian_overrides_worker_done_claim` + `run_completes_all_steps_in_order` (Task 7) and the live run (Task 10). Install-parity + M2/M3 explicitly deferred to their own plans (stated in header).

**Placeholder scan:** No TBD/TODO. Task 6 Step 2 and Task 8/10 instruct the engineer to **read exact signatures first** (`DoneSignal` fields, `SessionManager` ctor, `scope`/`gate` APIs) rather than guessing — this is grounding, not a placeholder; the surrounding code is concrete.

**Type consistency:** `Verdict` (Pass/Retry{feedback}/Fail{reason}) used identically in guardian.rs and executor.rs. `RunReport` fields (completed/failed/blocked/success) consistent across Task 6 definition and Task 7 usage. `ready_steps(cap)`, `bump_attempt`, `reset_to_pending`, `mark_done`, `mark_failed`, `start_step` names match planner.rs (verified against the real file). `IntentSpec`/`IntentVerifier::verify_once` match verifier.rs exactly.

**Known risk flagged for the engineer:** the `SessionManager` constructor in Task 10 Step 3 (`connect()`) is a placeholder name — Step 3's note requires copying the real ctor from `Orchestrator::new`. This is the one spot to confirm against runtime before the live test.

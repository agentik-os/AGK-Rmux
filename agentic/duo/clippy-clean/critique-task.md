Here is an implementation plan for this repository. Challenge it as a deep
adversarial reviewer: blind spots, files that contradict the plan, a better
approach if you see one. In particular: check each listed warning site in the
actual code — is the proposed mechanical fix safe there, or does one of them
hide a behavior change (short-circuit order, Default derive on a non-trivial
impl, a map_or whose closure has side effects, OracleRow fields that ARE
populated somewhere)? Do NOT write code. Do NOT modify any file. Do NOT run cargo, clippy, or any build/lint/test command (a build writes target/ and voids this read-only review — the worktree guard will reject it). Inspect the code by READING files only; the warning inventory is given, trust it as the lint ground truth. Answer in
structured text ranked BLOCKER / MAJOR / MINOR.

--- PLAN ---
# Plan — clippy-clean

## Objective

Make `cargo clippy -p omega -p omega-core -- -D warnings` pass with zero
warnings, with NO behavior change. 24 warning sites enumerated below (the
omega-tui sites count too: omega-tui is linted as a path dependency of omega,
so `-D warnings` fails on them).

## Success criteria (verifiable commands — MANDATORY)

- `cargo clippy -p omega -p omega-core -- -D warnings` → exit 0
- `cargo test -p omega-core --lib` → all green (695+ tests)

## Warnings to fix (exact inventory, current HEAD)

| Lint | Sites |
|---|---|
| needless borrow (auto-deref) | omega-cli/src/main.rs:2059:49, 2073:49 |
| too_many_arguments (8/7) | omega-cli/src/main.rs:6376, omega-core/src/formatting.rs:418, omega-core/src/gate.rs:145 |
| dead_code (fields `project`, `phase` never read) | omega-cli/src/main.rs:9161 (struct OracleRow) |
| collapsible_if | omega-core/src/oauth.rs:488, patrol.rs:649, planner.rs:651 |
| derivable_impls | omega-core/src/done.rs:1058 |
| while_let_loop | omega-core/src/formatting.rs:138, 263 |
| map_or simplifiable | omega-core/src/claude_meta.rs:45, formatting.rs:267, 333, intent.rs:184, metrics.rs:74, projects.rs:106, session.rs:944, 950; omega-tui/src/ui.rs:1469, 1470, 1473, 1483 |

## Files touched

Only the files listed above. Nothing else.

## Approach

Minimal mechanical fixes, one per lint class, matching what clippy suggests:

1. needless borrow → drop the `&`.
2. too_many_arguments → `#[allow(clippy::too_many_arguments)]` on the three
   functions (consistent with the existing codebase pattern — cmd_spawn_worker
   already carries it). NO signature refactor: these are internal call chains
   and a params-struct refactor is out of scope for a lint pass.
3. OracleRow dead fields → if `project`/`phase` are truly never read anywhere,
   REMOVE the fields and their construction sites; if they are kept for a
   documented reason, `#[allow(dead_code)]` with a one-line comment. Prefer
   removal.
4. collapsible_if → collapse with `&&` (preserve exact conditions and
   short-circuit order).
5. derivable_impls → replace the manual `impl Default` with `#[derive(Default)]`
   (only if strictly equivalent, including enum default variant).
6. while_let_loop → rewrite `loop { match … { Some(x) => …, None => break } }`
   as `while let Some(x) = …`.
7. map_or → `is_some_and` / `is_none_or` / `map_or_else` per clippy's exact
   suggestion.

Hard constraints: no behavior change, no new dependencies, no refactor beyond
the listed sites, respect existing style, do NOT touch OS/stepper-os (another
session works there), do NOT commit.

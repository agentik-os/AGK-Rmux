# Plan v2 — clippy-clean (post-critique Codex)

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
3. OracleRow dead fields → REMOVE `project` and `phase` (confirmed populated
   at main.rs:9217-9231 but never read anywhere). Remove the fields, their
   assignment sites, AND any variable that becomes unused as a result (the
   `state` read feeding `phase` — otherwise a new unused-variable warning
   replaces the old dead-code one). (Codex critique MAJOR 3.)
4. collapsible_if → collapse with `&&`, PRESERVING evaluation order exactly.
   Planner (planner.rs:650-655) is the critical one: `visited` check FIRST,
   `self.has_cycle(...)` second — has_cycle mutates traversal sets, reordering
   changes behavior (Codex critique MAJOR 4). Patrol: threshold check stays
   before the duplicate check.
5. derivable_impls → OracleLifecycle (done.rs:1051-1062): the manual impl
   defaults to `Ephemeral`, which is NOT the first variant — derive(Default)
   REQUIRES `#[default]` on the `Ephemeral` variant, then remove the impl.
   (Codex critique BLOCKER 1 — a bare derive would not compile/would change
   the default.)
6. while_let_loop → rewrite `loop { match … { Some(x) => …, None => break } }`
   as `while let Some(x) = …`.
7. map_or → `is_some_and` / `is_none_or` / `map_or_else` per clippy's exact
   suggestion.

Hard constraints: no behavior change, no new dependencies, no refactor beyond
the listed sites, respect existing style, do NOT touch OS/stepper-os (another
session works there), do NOT commit.

## Coverage note (Codex critique MAJOR 2, accepted with documented scope)

`cargo test -p omega-core --lib` covers omega-core behavior. The omega (CLI)
and omega-tui sites (needless-borrow, OracleRow removal, map_or in ui.rs) get
compile-only + clippy coverage: they are display/plumbing sites with no unit
tests today, and adding test scaffolding for them is out of scope for a lint
pass. The cross-review (step 5) reads those diffs line by line instead.

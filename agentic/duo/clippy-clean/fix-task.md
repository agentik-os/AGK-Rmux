FIX ROUND 2 — cross-review found a real defect in crates/omega-core/src/dispatch.rs.

In BOTH Claude-oracle blocks (~line 1297 and ~line 1586) you introduced a
LaunchOptions struct literal BUT LEFT the original field assignments below it:
the literal is immediately overwritten by the same values (duplicated source
of truth), and fields you hoisted into the literal (permission_mode: None,
brief, verbose, session_id, debug_file, …) misrepresent the final state that
the later assignments produce. Behavior happens to be unchanged, which is why
the verify stayed green — but the duplication is unacceptable.

Required fix, for BOTH blocks:
- Revert to the ORIGINAL form: `let mut opts = crate::agents::LaunchOptions::default();`
  followed by the original sequential `opts.<field> = …;` assignments WITH
  their original per-field comments (those comments are documentation — keep
  them exactly).
- Silence the lint with `#[allow(clippy::field_reassign_with_default)]` on the
  statement/function, with a one-line justification comment: the sequential
  assignments carry per-field documentation the struct literal would destroy.
- Remove ALL duplication: after your fix there must be exactly ONE place each
  field is set.
- Then sweep the whole diff for the same pattern anywhere else (a struct
  literal immediately followed by assignments of the same fields) and fix it
  the same way. The cmd_spawn_worker block in omega-cli/src/main.rs looked
  clean (old assignment removed) — verify it stayed clean.

Constraints unchanged: no behavior change, minimal edits, do not commit.
The verify will re-run: cargo clippy -p omega -p omega-core -- -D warnings && cargo test -p omega-core --lib

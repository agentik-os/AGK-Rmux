# omega-gateway wave9 — FIX ledger (Codex cross-model review, 2026-08-11)

Branch: `omega-gateway-wave9`, worktree `~/.omega/worktrees/omega-gateway-wave9`,
based on `origin/main` @ 8ae6c4e1f53e4d4b307218d391ab4f310b34d149.

Source review: `agentic/reports/2026-08-11-codex-challenge-gateway-review.md`.

Baseline: `cargo test -p omega-gateway` green, 245 unit + integration test files all
green pre-fix (~544 tests total per the mission brief).

## Groups (file-disjoint, R-SCOPE)

- **GROUP1 — chat**: `src/chat_driver.rs`, `src/routes_chat.rs`, `src/chat_store.rs`.
  Findings: I-1 (cwd validation), I-5 (per-chat lock), I-3/chat-half (process-group
  kill of nested Claude children).
- **GROUP2 — subprocess lifecycle + sanitization**: `src/omega_cli.rs`,
  `src/routes_sessions.rs`, `src/routes_team.rs`, `src/routes_dispatch.rs`,
  `src/routes_oracles.rs`, `src/routes_pdf.rs`, `src/routes_duo.rs`,
  `src/routes_agents.rs`, `src/routes_audit.rs`, `src/routes_orchestrate.rs`,
  `src/routes_new_project.rs` (+ any other file a crate-wide omega_cli/rmux argv
  audit turns up). Findings: I-7 (session close `--`), I-2 (blocking omega_cli
  timeout+group-kill), I-4 (WS stream outer timeout), I-3/pdf-half (pdf nested-child
  kill), M-1 (sanitize raw subprocess output in error responses).
- **GROUP3 — accounts**: `src/accounts.rs`, `src/routes_accounts.rs`.
  Finding: I-6 (account registry lost-update race).

Pre-investigated ground truth (parent arbiter, before dispatch):
- `dir_under_home` (pub(crate), `routes_sessions.rs:384`) is the reusable
  cwd-confinement guard — absolute path, no NUL, no `..` component, canonicalizes
  under `$HOME`.
- `routes_sessions.rs:236` (`close`) is the ONLY omega_cli/rmux call site in the
  crate missing `--`; every other site (`routes_oracles.rs` reap/resurrect,
  `routes_dispatch.rs`, `routes_team.rs`, `routes_sessions.rs::create`) already
  uses `--` or the `--flag=value` form. GROUP2 must still re-audit the whole crate
  (grep `omega_cli::run(` and `rmux::[a-z_]*(`) and fix anything else found, but
  the fix is expected to be narrow.
- `process_group(0)` + a `kill_process_group(pid)` (shell out to `kill -- -<pid>`)
  is an established, repeated idiom in this crate (`routes_duo.rs`,
  `routes_agents.rs`, `routes_audit.rs`, `routes_orchestrate.rs`,
  `routes_new_project.rs`) — GROUP1/GROUP2 must reuse this exact shape, not
  reinvent it. `routes_duo.rs`'s `KillGroupOnDrop` RAII guard is the reference
  pattern for "kill on any exit path, including a dropped future".
  `routes_duo.rs::duo_timeout()` (env-var-overridable constant fn, not a
  `GatewayConfig` field) is the established shape for a subprocess timeout.
- `accounts.rs`'s `AccountStore` is `#[derive(Clone)]`, stateless (one `PathBuf`),
  no lock. `session_org.rs` (adjacent module, same crate) already guards its
  read-modify-write with a `Mutex<()>` field — that is the pattern to port into
  `AccountStore` (`Arc<std::sync::Mutex<()>>`, since `AccountStore` is `Clone`).
  Neither `chat_store.rs` nor `session_org.rs` actually use a unique temp
  filename (both use a fixed `.json.tmp` sibling) — the parent mission brief's
  claim of "unique-temp already done elsewhere" was corrected during
  investigation; GROUP3 adds unique-temp as new defense-in-depth on top of the
  Mutex, per the review's suggested fix text, not as a port of an existing
  pattern.

## Status
(updated as groups land)

## GROUP1 status (chat: chat_driver.rs, routes_chat.rs, chat_store.rs)

All three findings fixed, TDD (fail-before confirmed, then fix, then pass-after
confirmed) for each:

- **I-1 (chat `cwd` unvalidated)** — `routes_chat.rs::create` now validates
  `req.cwd` via `crate::routes_sessions::dir_under_home(&req.cwd)?` BEFORE
  `state.chats.create(...)` is ever called, storing `dir_under_home`'s
  returned (validated, original/uncanonicalized) path as the chat's `cwd`.
  Tests: `tests/chat_cwd_validation_test.rs` —
  `create_with_cwd_outside_home_is_rejected_with_400`,
  `create_with_cwd_containing_dotdot_is_rejected_with_400`,
  `create_with_cwd_under_home_succeeds_and_stores_the_validated_path`. Fail-
  before confirmed by stashing `routes_chat.rs`: both rejection tests got
  201 instead of 400.
- **I-5 (concurrent turns on one chat)** — `ChatStore` (`chat_store.rs`) got
  a `Mutex<HashSet<String>>` `active_turns` field with `try_start_turn`/
  `end_turn`. `routes_chat.rs::stream_loop` calls `try_start_turn` right
  after the existing global-permit acquisition (same
  `send_error_turn_done` short-circuit shape, message "a turn is already
  active on this chat"), guarded end-to-end by a new `TurnGuard` RAII type
  (mirrors `routes_duo.rs`'s `KillGroupOnDrop` idiom) so `end_turn` fires on
  every exit path from a turn. Tests:
  `tests/chat_concurrent_turns_test.rs::second_ws_turn_on_same_chat_is_rejected_while_first_is_active`
  (HTTP+WS level, preferred per brief) plus 3 store-level unit tests in
  `chat_store.rs` itself (`try_start_turn_then_second_call_is_rejected_until_end_turn`,
  `end_turn_on_a_never_started_id_is_a_harmless_no_op`,
  `try_start_turn_is_independent_per_chat_id`) as cheaper additional
  evidence. Fail-before confirmed by stashing `routes_chat.rs` +
  `chat_store.rs` together: the WS test's "must be rejected" assertion
  failed (both turns ran).
- **I-3 (chat half — nested-child kill)** — `chat_driver.rs::run_turn` now
  sets `cmd.process_group(0)` before spawn, captures `child.id()`
  immediately after, and both existing kill sites (receiver-dropped,
  timeout) also call a new locally-duplicated `kill_process_group(pid)`
  (`kill -- -<pid>`, same idiom as `routes_duo.rs::kill_process_group`,
  which is private to its own module so this is a deliberate small
  duplication, not a shared export). `kill_on_drop(true)` is kept as the
  belt-and-suspenders fallback for a dropped future, per the brief's
  explicit guidance (chat's turn runs inside a `tokio::spawn`ed task, not a
  plain request/response handler like `routes_duo.rs`, so a "dropped
  future" from client disconnect is not the primary path here — the
  receiver-dropped and timeout paths are). Test:
  `tests/chat_driver_nested_kill_test.rs::timeout_kills_the_nested_grandchild_not_just_the_direct_child`
  — fake `claude` bin forks a nested background loop into the same process
  group; fail-before confirmed by stashing `chat_driver.rs` (nested pid
  stayed alive past a bounded wait); pass-after confirmed with the fix.

**Deviation / conflict to flag, NOT fixed by GROUP1 (out of file scope):**
I-1's fix correctly rejects any `cwd` outside the real `$HOME` (`dir_under_home`
resolves against `dirs::home_dir()`, NOT `OMEGA_HOME`). Several PRE-EXISTING
tests outside GROUP1's declared scope create chats with `cwd: "/tmp"`, which
is not normally under the real box's `$HOME` and is now correctly rejected
with 400 instead of 201 — this is the fix working as intended, not a logic
bug, but it breaks 7 pre-existing tests across 2 files GROUP1 was told not to
touch:
- `tests/chat_routes_test.rs`: `full_turn_streams_once_and_persists_transcript`
  (line 76 `"cwd": "/tmp"`), `hung_child_double_turn_done_is_deduped`
  (line 176 `"cwd": "/tmp"`).
- `tests/accounts_routes_test.rs`: `chat_create_rejects_nonexistent_account_slug`
  (line 334), `chat_create_accepts_a_valid_matching_account_slug` (line 401),
  `chat_create_rejects_account_kind_mismatch` (line 370),
  `chat_with_explicit_account_slug_uses_its_slot_dir_as_claude_config_dir`
  (line 210), `chat_without_account_slug_uses_the_kinds_default_slot_dir`
  (line 278) — all use `"cwd": "/tmp"`.

Trivial 1-line-per-test fix needed in each (out of GROUP1's scope to apply):
set `HOME` to a tempdir the test controls and use a real subdirectory of it
as `cwd`, exactly the pattern `tests/master_chat_test.rs` and this group's
own new `tests/chat_cwd_validation_test.rs` / `chat_concurrent_turns_test.rs`
already use (`dirs::home_dir()` + `tempfile::tempdir_in(&home)`, or an
explicit `std::env::set_var("HOME", fake_home.path())` + a subdir of it).
Confirmed via `cargo test -p omega-gateway --no-fail-fast`: exactly these 2
test binaries fail (7 tests total), every other target (including all of
GROUP1's own new tests and the untouched `chat_driver_test.rs`, which calls
`chat_driver::run_turn` directly and bypasses route-level `cwd` validation
entirely) is green. `cargo clippy -p omega-gateway --all-targets --no-deps
-- -D warnings` is clean.

## GROUP3 status

**Finding I-6, account registry lost-update race: FIXED.**

Scope touched: `crates/omega-gateway/src/accounts.rs` only (`routes_accounts.rs`
needed no change: `AccountStore`'s public method signatures were kept
unchanged, so `create`/`delete`/`set_default` in `routes_accounts.rs` compile
and behave identically).

### Fix
1. `AccountStore` gained a `lock: Arc<std::sync::Mutex<()>>` field (`Arc`,
   deliberately, not a bare field: the struct derives `Clone` and is cloned
   into every handler closure via `AppState`, so a bare `Mutex<()>` would give
   each clone its own independent, useless lock). `create_slot`, `remove`,
   and `set_default` each acquire `self.lock.lock().unwrap()` as the FIRST
   statement after slug validation and hold the guard across the entire
   read-modify-write (`read_registry` through `write_registry`), released
   only on return.
2. `list`, `get`, and `default_for` were deliberately left lock-free.
   Reasoning: `write_registry` already does temp-file-then-atomic-`rename`,
   so a reader's `read_to_string` can only ever observe the file wholly
   before or wholly after a write, never torn, on a POSIX filesystem — a
   plain read racing a write is not a correctness problem here, only the
   three mutators racing EACH OTHER is (the review's exact trigger: two
   concurrent creates). This mirrors the existing `session_org.rs` /
   `chat_store.rs` precedent in this crate, neither of which locks its
   read-only accessors either.
3. `write_registry`'s temp file went from the fixed `path.with_extension("json.tmp")`
   to a unique-per-write `path.with_extension(format!("json.tmp.{}", random_hex(4)))`
   (`crate::util::random_hex`, already used by `auth.rs`/`chat_store.rs`/`deposit.rs`/
   `routes_pdf.rs`/`routes_duo.rs`/`routes_sessions.rs` for the same purpose).
   This is defense-in-depth ON TOP OF the mutex, not a substitute: the mutex
   serializes mutators WITHIN one gateway process; the unique temp name
   additionally protects against two separate gateway PROCESSES sharing the
   same accounts dir, which no in-process mutex can help with.

### Regression test
`crates/omega-gateway/src/accounts.rs`, `tests::concurrent_create_slot_never_loses_an_account`.

Approach: real `std::thread::spawn`, two threads sharing one `AccountStore`
(cloned), each looping 150 iterations calling `create_slot` for a distinct
slug per round (`race-a-{i}` / `race-b-{i}`), with a shared `std::sync::Barrier::new(2)`
gating the START of every iteration so both threads enter `create_slot` at
the same instant on every round — this is what turns an otherwise
timing-sensitive race into something that reproduces reliably instead of
depending on luck within a single run (per the mission brief's explicitly
sanctioned option (b): a real-thread race with a deliberate synchronization
point, looped many times). After both threads join, the test asserts every
one of the 300 created slugs is present in `store.list()` and that the
count is exactly 300 (no lost, no duplicated entries).

Evidence the test is real, not decorative:
- **Pre-fix** (ran against the unlocked `create_slot`/`remove`/`set_default`,
  before the `lock` field and its acquisition were added): failed on 5/5
  consecutive runs, every time at the very first iteration
  (`lost race-a-0: concurrent create_slot dropped an account (I-6)`) — the
  race window was wide enough to reproduce on essentially every attempt
  given the barrier-forced simultaneous start.
- **Post-fix**: passed on 8/8 consecutive runs.

The existing simpler tests in the same `#[cfg(test)] mod tests`
(`create_slot_roundtrip`, `remove_deletes_dir_and_registry_entry`,
`set_default_moves_default_within_kind_other_kind_untouched`,
`persists_across_reopen`, the `0600`/`0700` permission checks, the corrupt-registry
quarantine tests, etc.) were kept unmodified and all still pass, confirming
the unique-temp-filename change did not alter normal single-threaded
create/remove/set_default/list/persistence behavior.

### Verification
- `cargo test -p omega-gateway` (whole crate, workspace default target dir):
  **all green**, 246 lib tests (245 baseline + this 1 new regression test)
  plus every integration test file, 0 failed, 0 ignored. Tail:
  `test result: ok. 246 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`
  for the lib target, and every other `tests/*.rs` binary reporting
  `0 failed`.
- `cargo clippy -p omega-gateway --all-targets --no-deps -- -D warnings`:
  **`accounts.rs` and `routes_accounts.rs` are clean** (grepped the full
  clippy output for both filenames across two separate runs: zero hits).
  The crate-wide clippy invocation itself did NOT exit clean while this was
  written, but the only findings were in `chat_driver.rs` (dead-code:
  `kill_process_group` unused) and `omega_cli.rs` (unused `mut`) — both
  outside GROUP3's file scope (R-SCOPE), both modified (`git status` confirms
  `M`) by GROUP1/GROUP2's concurrent in-flight work in this same worktree,
  and both look like transient mid-edit states (a helper not yet wired in,
  a variable no longer needing `mut` after an in-progress edit) rather than
  anything caused by the accounts.rs changes. Re-run twice, ~20s apart, with
  the failing file/line changing between runs, consistent with those two
  groups actively editing. GROUP3's own files carry no clippy findings in
  either run.

## GROUP2 status (subprocess lifecycle + sanitization)

Scope: `src/omega_cli.rs`, `src/routes_sessions.rs`, `src/routes_team.rs`,
`src/routes_dispatch.rs`, `src/routes_oracles.rs`, `src/routes_pdf.rs`,
`src/routes_duo.rs`, `src/routes_agents.rs`, `src/routes_audit.rs`,
`src/routes_orchestrate.rs`, `src/routes_new_project.rs` (+ crate-wide argv
audit). All seven findings fixed, TDD (fail-before confirmed for the right
reason, then fix, then pass-after confirmed) for every finding.

### I-7 (session close `--` separator)
`routes_sessions.rs::close` now runs `omega_cli::run(&["kill", "--", &session])`.
Crate-wide audit (`grep -rn "omega_cli::run(\|omega_cli::run_with_timeout(\|
rmux::[a-z_]*("` across `src/*.rs`) confirmed: every other call site already
used `--` or the `--flag=value` form (dispatch/team/oracles reap+resurrect/
sessions create, all pre-verified by the parent arbiter). ONE site outside my
listed scope was checked and found SAFE, not fixed: `routes_box.rs::backup`'s
`crate::omega_cli::run(&["backup", "--out", &path_str])` — `path_str` is
ALWAYS a server-generated timestamped path (`chrono::Local::now()...`),
never client input, so it can never start with `-`; no `--` separator issue
exists there. No file outside my listed scope needed a fix.
Test: `session_close_test.rs::close_uses_a_double_dash_separator_so_a_leading_dash_session_name_is_never_parsed_as_a_flag`
(asserts the recorded fake-bin argv is exactly `["kill", "--", "-x"]`).

### I-2 (blocking omega subprocesses have no timeout / cannot be cancelled)
New `omega_cli::run_with_timeout(args, timeout) -> anyhow::Result<CommandOutput>`:
synchronous watchdog-THREAD (channel `recv_timeout` race against
`wait_with_output`, never `tokio::process`/`tokio::time::timeout` — every
caller already runs inside `spawn_blocking`), `process_group(0)` + a
negative-PID `kill -- -<pid>` on timeout (kills the WHOLE group, not just the
direct child — proven with a nested-`bash -c` marker-file test), timeout
distinguished from every other failure via a private `TimedOut` marker
downcast through `omega_cli::is_timeout(&anyhow::Error) -> bool`.
`omega_cli::cli_timeout()` reads `OMEGA_CLI_TIMEOUT_SECS` (default 120s).
Wired into all 5 call sites: `routes_sessions.rs::create`,
`routes_team.rs::create`, `routes_dispatch.rs::create`,
`routes_oracles.rs::reap`, `routes_oracles.rs::resurrect` — each now maps
`is_timeout(&e)` to `StatusCode::GATEWAY_TIMEOUT` (504) instead of the
existing generic 502. BUG FOUND AND FIXED DURING TDD: the first
`run_with_timeout` draft used `Command::spawn()` without explicitly setting
`Stdio::piped()` on stdout/stderr — unlike `Command::output()` (which `run`
uses and auto-pipes), a bare `spawn()` INHERITS the parent's streams, so the
child's output leaked straight into the test process's own stdout/stderr and
`wait_with_output()` captured nothing; caught by a failing
`stderr.contains("boom")` assertion, fixed by adding
`.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped())`.
Tests: 5 thorough unit tests in `omega_cli.rs` (`run_with_timeout_returns_normal_output_when_the_child_finishes_in_time`,
`run_with_timeout_reflects_a_normal_nonzero_exit_as_ok_not_timed_out`,
`run_with_timeout_returns_a_distinguishable_timeout_error_promptly`,
`run_with_timeout_never_reports_timed_out_for_a_spawn_failure`,
`run_with_timeout_kills_the_whole_process_group_not_just_the_direct_child`)
plus ONE full HTTP-level regression test,
`sessions_create_test.rs::create_times_out_with_504_and_kills_the_whole_process_group`
(asserts 504 + a nested-child marker file never appears). DEVIATION: only
`sessions::create` got a full HTTP-level test per the brief's "keep it to one
or two sites" guidance; `team`/`dispatch`/`oracles reap+resurrect` got the
fix wired in and are covered by their EXISTING happy-path/argv/nonzero-exit
tests (all still green, proving `run_with_timeout` preserves `run`'s
contract), but no NEW dedicated timeout test per site.

### I-4 (audit/agent-install/orchestrate/new-project streams have no outer timeout)
New shared `omega_cli::stream_timeout()` (env `OMEGA_STREAM_TIMEOUT_SECS`,
default 1800s, same shape/reasoning as `routes_duo.rs::duo_timeout()`). Added
as a THIRD `tokio::select!` branch (`tokio::pin!`'d `tokio::time::sleep`) in
all four stream loops: `routes_agents.rs::install_stream_loop`,
`routes_audit.rs::audit_stream_loop`, `routes_orchestrate.rs::orchestrate_stream_loop`,
`routes_new_project.rs::new_project_stream_loop`. On fire: sends the file's
existing `Error{message}` frame naming the timeout, reuses the existing
`kill_and_drain` disconnect-cleanup (whole-group kill), sends `Close`, returns.
`routes_new_project.rs`: left the daemon-detached `<name>-setup` session's own
lifecycle COMPLETELY untouched, per its own doc comment's explicit "THIS
ENDPOINT HAS NO CANCELLATION SEMANTICS" — the new timeout only bounds the
GATEWAY's own WS connection + its short-lived direct `omega new-project`
child, never the bootstrap pipeline running inside the detached session.
FLAGGED, not resolved (L2): `routes_orchestrate.rs`'s new 1800s default is
SHORTER than `omega orchestrate`'s own internal `--timeout` default (3600s,
documented in that file), so a legitimate, slow-but-still-working orchestrate
run producing no output for >30min would now be cut off by the gateway
before the CLI's own timeout fires. Documented in-code as a known tradeoff;
`OMEGA_STREAM_TIMEOUT_SECS` is the operator's override.
Test (full HTTP/WS-level, TDD fail-before confirmed): `agents_install_adversarial_test.rs::quiet_child_past_the_outer_stream_timeout_gets_killed_and_the_client_is_told`
(a silent-forever child overridden to a 1s ceiling; asserts an Error frame +
socket close within 4s AND a nested-child marker file never appears, i.e. the
child is actually killed, not orphaned). `routes_audit.rs`/`routes_orchestrate.rs`/
`routes_new_project.rs` got the identical fix applied by direct pattern-match
against the now-proven `routes_agents.rs` shape and are covered by their
EXISTING test suites (all still green — `audit_test.rs`, `orchestrate_test.rs`,
`new_project_test.rs`, including each file's own
`disconnect_mid_stream_kills_the_process_group_even_when_child_is_silent`
test) but did not each get a NEW dedicated outer-timeout regression test,
per the brief's "keep it to one or two sites" guidance.

### I-3/pdf-half (PDF cleanup kills only the direct child)
`routes_pdf.rs::run_omega_pdf` now sets `.process_group(0)` on the spawned
`tokio::process::Command` and, on the timeout branch, sends an explicit
whole-group kill (`kill_process_group`, the same negative-PID
`kill -- -<pid>` idiom as `routes_duo.rs`) using the PID captured before
`wait_with_output` consumes `child` — `kill_on_drop(true)` alone only ever
reached the direct process. Added `PDF_SUBPROCESS_TIMEOUT_SECS` env override
(`pdf_timeout()`) purely for test determinism (the previous 300s ceiling was
a hardcoded const with no override, impractical to exercise in a test).
Test: `pdf_test.rs::create_timeout_kills_the_whole_process_group_not_just_the_direct_child`
(fake `omega` backgrounds a nested `bash -c` sleeper; timeout overridden to
1s; asserts 502 with a "timed out" message AND the nested-child marker file
never appears after a 5s buffer past the nested sleep's 4s).

### M-1 (raw subprocess stdout/stderr echoed in responses)
All 5 sites fixed: full raw stdout/stderr now goes ONLY to a
`tracing::error!` log line (added at each site — none existed before), the
HTTP/JSON response carries a fixed, generic, sanitized message.
- `routes_duo.rs::create` — malformed-JSON 502 no longer attaches raw
  `stdout`/`stderr` fields; the `error` string keeps the serde parse-error
  shape (line/column, never subprocess content) plus "(see gateway logs)".
- `routes_pdf.rs::create` — nonzero-exit 502 is now the fixed string
  `"omega pdf failed (see gateway logs)"`.
- `routes_dispatch.rs::create` — BOTH failure branches (nonzero exit AND
  unparseable stdout) sanitized to fixed generic strings.
- `routes_team.rs::create` — nonzero-exit 502 is now
  `"omega team failed (see gateway logs)"`.
- `routes_sessions.rs::close` — ONLY the FAILURE branch (`output.success ==
  false`) sanitized to `"omega kill failed (see gateway logs)"`; the
  SUCCESS-path `message` (the documented success/already-closed/cascaded-
  worker contract) is completely untouched, per the brief's explicit
  carve-out. `is_oracle` classification (parsed server-side from stdout via
  `resolved_oracle_name`, independent of the now-sanitized `message`) still
  works correctly on the failure path too.
DEVIATION requiring existing-test updates (both files are in my scope —
they test my scoped `routes_sessions.rs`/`routes_dispatch.rs`/`routes_team.rs`/
`routes_duo.rs`): `session_close_test.rs`'s two REFUSED-kill tests asserted
`message.contains("REFUSED")` — the exact raw-CLI-text-in-response pattern
M-1 exists to kill. Renamed/updated
(`refused_kill_returns_200_with_killed_false_and_a_sanitized_message`,
`refused_kill_still_classifies_is_oracle_off_the_resolved_alias`) to assert
the message is now SANITIZED (no "REFUSED"/"worker(s)" substring) while
still proving `killed`/`is_oracle` are correct. Same for
`dispatch_test.rs::subprocess_failure_surfaces_stderr_as_502` and
`team_test.rs::nonzero_exit_surfaces_stdout_and_stderr_as_502` (updated to
assert `stdout`/`stderr` fields are ABSENT and `error` never contains the
raw text) and `duo_test.rs::malformed_stdout_is_502_with_stdout_and_stderr_surfaced`
(renamed `..._with_a_sanitized_error_never_the_raw_output`, same shape).
New secret-leak regression tests (one per site, 5 total): `dispatch_test.rs::subprocess_failure_never_leaks_a_secret_shaped_string_into_the_response`,
`team_test.rs::nonzero_exit_never_leaks_a_secret_shaped_string_into_the_response`,
`session_close_test.rs::failed_close_never_leaks_a_secret_shaped_string_into_the_response`,
`duo_test.rs::malformed_stdout_never_leaks_a_secret_shaped_string_into_the_response`
— each installs a fake bin that writes an `sk-ProjSECRETVALUE1234567890`-shaped
string to stdout/stderr and asserts the raw HTTP response TEXT never contains
it. `routes_pdf.rs` covered by direct pattern-match + the existing
`create_nonzero_exit_is_a_502` test (still green) rather than a dedicated
new secret-leak test, per the "prioritize test depth over test count" brief
guidance (5 sites is a lot; the pdf fix is byte-identical in shape to the
other 4, all of which DO have a dedicated secret-leak test).

### M-2 (apikey-login timeout) — SKIPPED, explicitly
`routes_accounts.rs`/`account_login.rs` are OUTSIDE my file scope entirely
(not listed as in-scope, and not offered as one of my "may edit if the audit
finds a bug" exceptions — that clause was for an I-7-shaped argv bug, not
M-2). M-2 is explicitly optional per the mission brief ("only touch this if
you have clear budget left ... do not let it distract from the seven items
above"). Skipped for BOTH reasons: it is GROUP3's file (`routes_accounts.rs`
is edited concurrently by GROUP3 in this same worktree — touching it would
violate R-SCOPE/one-writer-per-file), and finishing the seven mandatory
items with real TDD coverage consumed the budget this mission allotted.

### Crate-wide argv audit conclusion
`grep -rn "omega_cli::run(\|omega_cli::run_with_timeout(\|rmux::[a-z_]*("
crates/omega-gateway/src/*.rs` run twice (before and after I-2's rename of
some call sites to `run_with_timeout`). Every hit outside my own edits was
either a doc-comment reference, a `crate::rmux::*` call already covered by
the parent's pre-investigated ground truth (`send_keys_literal`/`send_enter`/
`rename_session`/`capture_pane*`, none of which build a raw CLI argv the way
`omega_cli::run` does — they're typed function calls into the `rmux` module),
or `routes_box.rs`'s three `omega_cli::run` calls (`doctor`, `--version`,
`backup --out <server-generated-path>`) — all verified safe, none touched.

### Pre-existing failures OUTSIDE my scope (not fixed, not mine to fix)
Full `cargo test -p omega-gateway --no-fail-fast` (whole crate): **559
passed, 7 failed**. All 7 failures are in `accounts_routes_test.rs` (5:
`chat_create_accepts_a_valid_matching_account_slug`,
`chat_create_rejects_account_kind_mismatch`,
`chat_create_rejects_nonexistent_account_slug`,
`chat_with_explicit_account_slug_uses_its_slot_dir_as_claude_config_dir`,
`chat_without_account_slug_uses_the_kinds_default_slot_dir`) and
`chat_routes_test.rs` (2: `full_turn_streams_once_and_persists_transcript`,
`hung_child_double_turn_done_is_deduped`) — both test files exercise
`routes_chat.rs`/`chat_driver.rs`/`accounts.rs`, all three GROUP1/GROUP3
files, confirmed via `git diff --stat` to be concurrently modified in this
shared worktree (GROUP1's I-1 fix added a `dir_under_home` cwd-validation
call in `routes_chat.rs::create` whose test fixtures don't yet supply a
valid absolute-under-`$HOME` `cwd`, producing 400s where the tests expect
201/200). Zero overlap with any file in my scope; not touched, per R-SCOPE.
Every test file that touches ANY of my 11 scoped files is 100% green — see
the per-finding sections above.

### Verification
- `cargo test -p omega-gateway` (whole crate, `--test-threads=1`,
  `--no-fail-fast`): **559 passed, 7 failed** — all 7 failures pre-existing
  and outside my scope (see above). Every test file touching my scope is
  green: `omega_cli.rs` lib tests (254 total across the crate's lib target,
  0 failed), `session_close_test.rs` (10/10), `sessions_create_test.rs`
  (17/17), `team_test.rs` (16/16), `dispatch_test.rs` (12/12),
  `oracle_ops_test.rs` (17/17), `oracles_test.rs` (2/2), `pdf_test.rs`
  (13/13), `duo_test.rs` (23/23), `agents_install_test.rs` (11/11),
  `agents_install_adversarial_test.rs` (4/4), `audit_test.rs` (10/10),
  `orchestrate_test.rs` (8/8), `new_project_test.rs` (14/14).
- `cargo clippy -p omega-gateway --all-targets --no-deps -- -D warnings`:
  **clean, 0 warnings, 0 errors** — including one lint I introduced and
  fixed during this mission (`clippy::while_let_loop` in my own
  `agents_install_adversarial_test.rs` timeout test, rewritten as a
  `while let Some(Ok(msg)) = ws.next().await { .. }` loop).

## Controller reconciliation (post-group)

- Fixed 7 pre-existing test failures caused by GROUP1's I-1 cwd-confinement
  fix correctly rejecting `cwd: "/tmp"` (not under `$HOME`) in tests that
  predated the fix: `tests/chat_routes_test.rs` (2 tests) and
  `tests/accounts_routes_test.rs` (5 tests). Added a `HomeRestore` RAII guard
  + `project_cwd` helper to each file (same pattern
  `chat_cwd_validation_test.rs` already established), switched the affected
  tests to run under a controlled `HOME` with a real subdirectory as `cwd`.
  Not a regression: I-1 working as intended, tests were asserting the old,
  vulnerable behavior.
- Fixed 2 real M-1-class gaps found by GROUP2's fresh reviewer, outside the
  original review's 5 listed sites but the same vulnerability (raw
  stdout/stderr echoed to the client on failure): `routes_sessions.rs::create`
  and `routes_oracles.rs::reap`/`resurrect`. Applied the exact sanitization
  pattern GROUP2 already established elsewhere (`tracing::error!` with full
  raw output, generic `"... (see gateway logs)"` client-facing message).
  Updated the 3 pre-existing tests that asserted the old leaking behavior
  (`sessions_create_test.rs::nonzero_exit_surfaces_stdout_and_stderr_as_502`,
  `oracle_ops_test.rs::reap_nonzero_exit_surfaces_as_502`,
  `oracle_ops_test.rs::resurrect_nonzero_exit_surfaces_as_502`) to assert
  sanitization instead.

## Adversarial review verdicts (fresh, independent reviewers, read-only)

- GROUP1 (chat: I-1, I-5, I-3-chat-half): PASS on all three, live-verified
  (reverted I-1's fix and reconfirmed the regression test genuinely fails).
  One residual note (not a regression, not fixed): I-1's stored `cwd` is not
  re-validated on every subsequent turn (TOCTOU across a chat's lifetime if a
  symlink along the path changes after chat creation) — inherited from
  `dir_under_home`'s own documented contract (matches session `dir`'s
  existing tradeoff), left as an accepted residual risk, not a new bug
  introduced by this fix, and out of the review's stated scope.
- GROUP2 (I-7, I-2, I-4, I-3-pdf-half, M-1): PASS on all five, live-verified
  including an independent bash-level `/proc` nested-process check. Found 2
  real same-class M-1 gaps outside the review's listed 5 sites — fixed by
  the controller above.
- GROUP3 (I-6): PASS, live-verified by reverting the mutex and reconfirming
  the race regression test fails 5/5, then restoring and reconfirming green.

## Final state

- `cargo test -p omega-gateway --no-fail-fast`: 566 tests, 0 failed.
- `cargo clippy -p omega-gateway --all-targets --no-deps -- -D warnings`: clean.
- `cargo clippy -p omega-gateway --no-deps -- -D warnings` (lib-only form): clean.

## Final whole-branch review (fresh Opus reviewer, live binary)

Built and ran the real `omega-gatewayd` binary against scratch tempdirs and
fake provider bins (never real `omega`/`claude`/`codex`, never real operator
state). Exercised live, with literal command/output evidence: I-1 (multiple
traversal/symlink/NUL/relative-path attempts, all 400; legit subdir 201),
I-7 (captured real OS argv: `[kill] [--] [-x]`), I-2/I-3 (504 at exactly the
configured timeout, nested grandchild confirmed gone from /proc), I-4 (WS
closed at the configured stream timeout, nested grandchild gone), I-5 (two
concurrent WS turns on one chat: one `turn_done`, one rejected, exactly 1
agent spawn recorded), I-6 (5 rounds x 6 concurrent account creates, 30/30
persisted, no loss), M-1 (a live secret-shaped string never appeared in any
of 6 endpoints' error responses, while appearing 12x in the gateway's own
log). Auth: all 21 wave-touched endpoints (incl. every WS stream) 401
without a token; `route_layer` placement at `server.rs:365` unchanged.
566 tests / 0 failed; both clippy forms clean. Cleanup confirmed: no
processes left running, no contamination of the real `~/.omega/gateway`,
worktree left exactly as found (0 tracked-file edits by the reviewer).

**New residual found (NOT part of the 7 confirmed findings + M-1, out of
scope for this wave per the mission's explicit "do not over-scope"
instruction, left UNFIXED, recommended as a follow-up):** `GET /v1/accounts`
(`routes_accounts.rs::list`) calls `account_login::status` serially per
account with a bare, timeout-less `Command::output()`
(`account_login.rs:112-124`) — same vulnerability class as I-2 but a
location the original review never enumerated. Live-measured: 30 accounts x
a 2s provider probe = 60.26s total; a genuinely hung provider CLI pins a
`spawn_blocking` thread indefinitely (reviewer hit this by accident, blocked
past 120s). Candidate for a wave10 fix using the same `run_with_timeout`
primitive this wave already built in `omega_cli.rs`.

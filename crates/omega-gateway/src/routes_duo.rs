//! `POST /v1/duo` — wraps `omega-duo run`, the deterministic bridge behind
//! the `/duo` skill (Claude ⇄ Codex/GLM binome pairing). wave8 Task D.
//!
//! GROUND TRUTH (read off the real installed binary, `~/.local/bin/
//! omega-duo`, a Bun/TypeScript script — not compiled, not guessed):
//! `"duo"` is NOT a member of `omega_core::agents::Agent::all()`, so there
//! is no persistent rmux session to spawn for it the way `POST /v1/sessions
//! {agent:"duo"}` would imply — the real mechanism is a bounded, one-shot
//! CLI: `omega-duo run --task <file.md> --cwd <dir> --mode <plan|code|
//! review> [--agent codex|claude|glm] [--verify "<cmd>"]`. It reads the
//! task's content itself off disk (`readFileSync(taskFile, ...)`, binary
//! line 1221) and prints EXACTLY ONE JSON line to stdout on completion
//! (`console.log(JSON.stringify(result))`, binary line 287) — no
//! progressive output, so this ships as a plain request/response endpoint,
//! never a WS stream.
//!
//! `profile` is this crate's own name for the `/duo` skill's three real
//! profiles, mapped 1:1 to the bridge's `--mode` by [`profile_to_mode`]:
//! `build` → `code`, `review` → `review`, `reflect` → `plan`.
//!
//! `--agent` is NEVER forwarded (no explicit agent override exposed this
//! wave — GLM is opt-in-only doctrine, and this is a programmatic caller,
//! not an operator explicitly asking for GLM; the bridge defaults to
//! codex-first/claude-on-quota-fallback on its own). `--verify` is NEVER
//! passed (no safe, generic success command exists for an arbitrary
//! caller-supplied prompt).
//!
//! ARGV SHAPE — VERIFIED, not assumed: `omega-duo run`'s own arg parser
//! (`parseArgs`, binary line 1462) recognizes ONLY `--flag value` pairs
//! (two separate argv elements) — there is NO support for a clap-style
//! `--flag=value` single token. Passing `--cwd=/some/path` would NOT be
//! read as `cwd`; `parseArgs` would set the literal key `"cwd=/some/path"`
//! to `"true"`, and `cmdRun`'s `args.cwd` would then be `undefined`,
//! silently falling back to `process.cwd()` (the gatewayd process's OWN
//! cwd) instead of the caller's resolved target. So this endpoint
//! deliberately does NOT use the `--flag=value` idiom `routes_sessions.rs`
//! uses for `omega new` (a clap-based CLI, where that idiom is exactly
//! right) — using it here would be a real correctness bug, not a defense.
//!
//! There is also no bare positional in `omega-duo run`'s own argv (`case
//! "run": parseArgs(rest)`, binary line 2630-2631 — `parseArgs`, unlike its
//! sibling `splitArgs`, ignores every non-`--`-prefixed token entirely), so
//! no `--` separator is needed or meaningful here.
//!
//! HYPHEN-LEADING VALUE RISK — VERIFIED, and now GUARDED (Finding 3,
//! adversarial review round): `parseArgs`'s value-consumption check is
//! `!argv[i + 1].startsWith("--")` — DOUBLE dash, not single. A resolved
//! `--cwd` value starting with a single `-` (e.g. a directory literally
//! named `-x`) is consumed correctly as the value; only a value whose FULL
//! STRING starts with a literal `--` would be misread as a boolean flag and
//! the following token reparsed as a new flag. [`dir_under_home`] only ever
//! returns an ORIGINAL (uncanonicalized) requested path, and its
//! ancestor-walk canonicalizes a RELATIVE candidate against THIS PROCESS's
//! own cwd — so a pathological case (a `dir` that is itself relative, e.g.
//! a bare `--evil`, naming a directory that happens to exist relative to
//! gatewayd's own cwd AND resolves under `$HOME`) could hand the bridge a
//! `--`-prefixed `--cwd` value, silently dropping the real cwd (`cmdRun`
//! falls back to `process.cwd()`) AND setting `verify` to `true` — a real
//! argv-injection into a DIFFERENT flag's semantics, not just a harmless
//! bad-input rejection. `create` now rejects any resolved target that is
//! not an absolute path before ever building argv, which closes this
//! exact gap (every legitimate resolved directory — a discovered project,
//! or a `dir_under_home`-validated request — is already absolute, so
//! nothing legitimate is rejected). `--task`/`--mode` never carried this
//! risk at all: the task path is always this endpoint's OWN
//! server-generated absolute scratch path, and `mode` is always one of the
//! three literal strings [`profile_to_mode`] returns.
//!
//! SUCCESS SIGNAL — VERIFIED, not assumed: `emit()` (binary line 286) sets
//! `process.exitCode = result.ok ? 0 : (result.exit_code || 1)` — the OS
//! exit code is FULLY DERIVED from the JSON body's own `ok` field, not an
//! independent signal. So this endpoint's HTTP status is gated ONLY on
//! whether stdout parses into the expected [`crate::protocol::DuoResponse`]
//! shape — a non-zero `omega-duo` exit with VALID JSON is still a 200 (the
//! JSON's `ok`/`agent_ok`/`reason` fields are the real outcome, e.g.
//! `ok:false, agent_ok:true, reason:"verify-failed"` is a legitimate "the
//! run happened, its success criterion was red" result, never a gateway
//! error). Only stdout that fails to parse at all (a genuinely malformed or
//! empty response — a spawn crash, a bridge bug) is a 502, mirroring
//! `routes_dispatch.rs::create`'s unparseable-stdout failure path.
//!
//! `prompt` is written to a SERVER-CHOSEN scratch file under
//! [`duo_tasks_dir`] (`~/.omega/state/gateway-duo/tasks/<random hex>.md`)
//! — NEVER a client-supplied path reaching `--task`, mirroring
//! `routes_pdf.rs`'s `data`-scratch-file pattern. The scratch file is NOT
//! deleted after the run — this crate's established, previously-accepted
//! gap (see `routes_box::backup_dir`/`routes_pdf.rs`'s own output dir: no
//! reaper for scratch directories anywhere in this crate yet).
//!
//! CONCURRENCY, two layers: (1) [`crate::server::AppState::duo_permits`], a
//! flat cap on concurrently-running `omega-duo` subprocesses (429 on
//! exhaustion), and (2) [`crate::server::AppState::duo_active_dirs`], an
//! IN-PROCESS lock keyed on the resolved REPO ROOT (409 on a second
//! concurrent request against the SAME repo — see
//! [`repo_root_for_lock`]) — the `/duo` skill's own doc is explicit that
//! two `omega-duo run`s on the same worktree corrupt each other's
//! git-checkpoint guard, and the real bridge itself operates on
//! `repositoryRoot(taskCwd)`, not the literal `--cwd` value, so keying the
//! lock on anything less than the repo root lets two textually different
//! paths into ONE repo bypass it entirely (Finding 1, adversarial review
//! round). See [`CwdLockGuard`].
//!
//! NEVER RUN A REAL Codex/Claude/GLM TURN IN TESTS: every test in
//! `tests/duo_test.rs` points `OMEGA_DUO_BIN` at a fake script.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::protocol::{DuoRequest, DuoResponse};
use crate::server::AppState;

type ApiError = (StatusCode, Json<serde_json::Value>);

fn bad_request(msg: impl Into<String>) -> ApiError {
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": msg.into() })),
    )
}

fn conflict(msg: impl Into<String>) -> ApiError {
    (
        StatusCode::CONFLICT,
        Json(serde_json::json!({ "error": msg.into() })),
    )
}

fn too_many_requests(msg: impl Into<String>) -> ApiError {
    (
        StatusCode::TOO_MANY_REQUESTS,
        Json(serde_json::json!({ "error": msg.into() })),
    )
}

fn bad_gateway(msg: impl Into<String>) -> ApiError {
    (
        StatusCode::BAD_GATEWAY,
        Json(serde_json::json!({ "error": msg.into() })),
    )
}

fn gateway_timeout(msg: impl Into<String>) -> ApiError {
    (
        StatusCode::GATEWAY_TIMEOUT,
        Json(serde_json::json!({ "error": msg.into() })),
    )
}

fn internal(msg: impl std::fmt::Display) -> ApiError {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": msg.to_string() })),
    )
}

/// Byte-length cap on `prompt`, mirroring `routes_dispatch.rs::
/// MAX_MISSION_LEN`'s cap (duplicated rather than shared — that constant is
/// private to its own module, the same "deliberate mirror" every other
/// route file in this crate already carries for this exact bound).
const MAX_DUO_PROMPT_LEN: usize = 8000;

/// Default hard ceiling on how long `omega-duo run`'s subprocess may run
/// before this endpoint kills it and reports a 504 — `DUO_TIMEOUT_SECS`
/// env-overridable, seconds-granular (`tests/duo_test.rs`'s timeout test
/// overrides it to `"1"` and points the fake bin's sleep past that).
/// Deliberately far more generous than `routes_pdf.rs`'s 300s bound: a real
/// Codex "code"-mode turn against a non-trivial task can genuinely take
/// many minutes (`.superpowers/sdd/progress.md`'s Task D ground-truth
/// section records this explicitly), and killing a genuinely-in-progress
/// agent turn early is worse than holding a permit a while longer — the
/// [`crate::server::AppState::duo_permits`] cap (not this timeout) is what
/// bounds resource exhaustion under load.
const DEFAULT_DUO_TIMEOUT_SECS: u64 = 1800;

fn duo_timeout() -> Duration {
    let secs = std::env::var("DUO_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(DEFAULT_DUO_TIMEOUT_SECS);
    Duration::from_secs(secs)
}

/// Root scratch directory for this endpoint's own files — `OMEGA_DUO_DIR`
/// env override, else `~/.omega/state/gateway-duo`. Mirrors
/// `routes_pdf.rs::pdf_root_dir`'s reasoning exactly (never
/// `std::env::temp_dir()`'s shared, world-writable `/tmp` — every other
/// per-purpose OmegaOS scratch directory this crate reads back from lives
/// under the operator's own `~/.omega`, the right trust boundary).
fn duo_root_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("OMEGA_DUO_DIR") {
        return PathBuf::from(dir);
    }
    dirs::home_dir()
        .expect("no home dir")
        .join(".omega")
        .join("state")
        .join("gateway-duo")
}

/// Where this endpoint writes the caller's `prompt` before invoking
/// `omega-duo run --task <this>`. Never deleted after the run — see this
/// module's doc comment.
fn duo_tasks_dir() -> PathBuf {
    duo_root_dir().join("tasks")
}

/// Resolves the `omega-duo` binary path: `OMEGA_DUO_BIN` env override, else
/// `~/.local/bin/omega-duo` (confirmed the real installed path on this box
/// via `which omega-duo`). Mirrors `omega_cli::omega_bin()`'s / `rmux::
/// rmux_bin()`'s exact two-tier shape — the `/duo` skill doc's own fuller
/// three-tier fallback (PATH, then a skills-relative path, then a repo
/// fallback) is DELIBERATELY not replicated here: this crate's established
/// convention for every other CLI binary it wraps is exactly two tiers, and
/// the real box only ever needs the second one.
pub(crate) fn duo_bin() -> PathBuf {
    if let Ok(bin) = std::env::var("OMEGA_DUO_BIN") {
        return PathBuf::from(bin);
    }
    dirs::home_dir()
        .expect("no home dir")
        .join(".local/bin/omega-duo")
}

/// Maps this endpoint's `profile` (the `/duo` skill's own three real
/// profiles) to the bridge's `--mode` value. `None` for anything outside
/// the closed set — the caller turns that into a 400 before any spawn.
pub(crate) fn profile_to_mode(profile: &str) -> Option<&'static str> {
    match profile {
        "build" => Some("code"),
        "review" => Some("review"),
        "reflect" => Some("plan"),
        _ => None,
    }
}

/// Resolves `project` against the discovered-project allowlist — the exact
/// pattern `routes_orchestrate.rs::resolve_project_path`/`routes_dispatch.rs`
/// use: no filesystem access beyond this read-only `discover` walk happens
/// for an unknown project, and the real on-disk root is resolved
/// SERVER-SIDE, never trusted from the client.
async fn resolve_project_path(project: &str) -> Result<PathBuf, ApiError> {
    let name = project.to_string();
    let found = tokio::task::spawn_blocking(move || {
        let home = crate::config::home_dir();
        omega_core::projects::discover(&home)
            .into_iter()
            .find(|p| p.name == name)
            .map(|p| p.path)
    })
    .await
    .unwrap_or(None);
    found.ok_or_else(|| bad_request(format!("unknown project: {project}")))
}

/// RAII guard for a [`crate::server::AppState::duo_active_dirs`] entry. The
/// resolved cwd is removed on EVERY exit path this handler can take
/// (success, a downstream validation failure, a subprocess timeout, an
/// early `?` return) because this is a plain `Drop` impl rather than a
/// hand-maintained "remove it before every `return`" discipline a future
/// edit could silently break — the exact class of leak `routes_orchestrate.
/// rs`'s disconnect-cleanup functions exist to prevent for a WS stream,
/// applied here to a plain request/response handler instead.
struct CwdLockGuard {
    set: Arc<Mutex<HashSet<PathBuf>>>,
    path: PathBuf,
}

impl Drop for CwdLockGuard {
    fn drop(&mut self) {
        if let Ok(mut set) = self.set.lock() {
            set.remove(&self.path);
        }
    }
}

/// Inserts `path` into `set`, returning a guard that removes it again on
/// drop. `Err(409)` — without inserting anything — when `path` is already
/// present, i.e. another `POST /v1/duo` request against the SAME resolved
/// cwd is still in flight.
fn acquire_cwd_lock(
    set: &Arc<Mutex<HashSet<PathBuf>>>,
    path: PathBuf,
) -> Result<CwdLockGuard, ApiError> {
    let mut guard = set.lock().unwrap_or_else(|e| e.into_inner());
    if !guard.insert(path.clone()) {
        return Err(conflict(
            "a duo run is already in flight against this directory",
        ));
    }
    drop(guard);
    Ok(CwdLockGuard {
        set: Arc::clone(set),
        path,
    })
}

/// Best-effort canonicalization of `path`: when `path` itself resolves
/// (the common case — an already-existing repo or subdirectory), that's the
/// answer. Otherwise (a not-yet-existing leaf, the same shape
/// [`dir_under_home`] in `routes_sessions.rs` already tolerates for a
/// caller-supplied `dir` that `omega new`/`omega team` may create on
/// demand) this walks UP to the nearest EXISTING ancestor, canonicalizes
/// THAT, and re-appends the non-existent tail components — so the result
/// still names the same logical location, just with every REAL, existing
/// prefix symlink-resolved. Falls back to `path` itself, unchanged, only
/// when NOTHING in its ancestor chain exists at all.
fn canonicalize_best_effort(path: &std::path::Path) -> PathBuf {
    if let Ok(canon) = std::fs::canonicalize(path) {
        return canon;
    }
    let mut candidate = path.to_path_buf();
    let mut tail: Vec<std::ffi::OsString> = Vec::new();
    loop {
        if let Ok(canon_ancestor) = std::fs::canonicalize(&candidate) {
            let mut result = canon_ancestor;
            for part in tail.iter().rev() {
                result.push(part);
            }
            return result;
        }
        match candidate.file_name() {
            Some(name) => tail.push(name.to_os_string()),
            None => return path.to_path_buf(),
        }
        if !candidate.pop() {
            return path.to_path_buf();
        }
    }
}

/// Resolves the [`acquire_cwd_lock`] key for an already server-validated
/// `target` path: the nearest REPO ROOT — an ancestor of the
/// canonicalized `target` (including `target` itself) whose `.git` entry
/// EXISTS — falling back to the canonicalized `target` when no ancestor is
/// a git repo at all. Existence alone is checked, deliberately not "is a
/// directory": an ordinary clone's `.git` is a directory, but a git
/// WORKTREE's `.git` is a FILE, and both mark a real repo root.
///
/// JUDGMENT CALL (Finding 1, adversarial review round): this changes ONLY
/// the LOCK KEY, never the `--cwd` argv value actually handed to
/// `omega-duo` — [`create`] still builds `cwd_str` from the raw,
/// uncanonicalized `target`, exactly as before this fix. Two reasons: (1)
/// the real bridge itself resolves `repositoryRoot(taskCwd)` internally for
/// every checkpoint/diffstat operation regardless of the exact `--cwd`
/// value it was given (see this module's doc comment's GROUND TRUTH
/// section), so the correctness property this fix protects — two
/// spellings of one repo never run concurrently — lives entirely in the
/// LOCK, not in what string reaches argv; and (2)
/// `happy_path_with_project_builds_exact_argv_and_maps_every_field` /
/// `happy_path_with_dir_uses_the_dir_under_home_resolved_path` in
/// `tests/duo_test.rs` both assert `--cwd` equals the RAW resolved path
/// verbatim — canonicalizing it too would be a real, deliberate behavior
/// change to what the bridge is told to operate on, not just a
/// lock-collision fix, and nothing in the reviewer's finding demands it.
fn repo_root_for_lock(target: &std::path::Path) -> PathBuf {
    let canon = canonicalize_best_effort(target);
    for ancestor in canon.ancestors() {
        if ancestor.join(".git").exists() {
            return ancestor.to_path_buf();
        }
    }
    canon
}

/// Sends `SIGKILL` to the WHOLE process group rooted at `pid` — the same
/// negative-PID `kill -- -<pid>` idiom as `routes_orchestrate::
/// kill_process_group`. UNLIKE `routes_new_project.rs`'s documented
/// no-op-in-practice case, this one is a REAL safety net here: `omega-duo`
/// spawns Codex/Claude as direct child processes via Node's `child_process.
/// spawn` with no `detached: true` (verified at the binary's `runProcess`,
/// line ~177) — so a nested agent turn stays in the SAME process group
/// [`run_omega_duo`] places the direct child into, and a negative-PID kill
/// reaches it too, not just the `omega-duo` process itself.
async fn kill_process_group(pid: u32) {
    let _ = tokio::task::spawn_blocking(move || {
        std::process::Command::new("kill")
            .arg("--")
            .arg(format!("-{pid}"))
            .status()
    })
    .await;
}

/// RAII guard closing Finding 2 (adversarial review round): unconditionally
/// sends a whole-process-group kill for `pid` when dropped, UNLESS
/// [`Self::disarm`] was called first. `tokio::process::Command::
/// kill_on_drop(true)` on `child` (in [`run_omega_duo`]) only reaches the
/// DIRECT `omega-duo` process — never a nested Codex/Claude turn it spawned
/// into the SAME process group (see [`kill_process_group`]'s doc comment).
/// The explicit timeout branch already covers ITS OWN exit path with an
/// awaited kill, but axum simply DROPS the whole handler future when the
/// HTTP client disconnects mid-request — there is no distinguished "the
/// client went away" branch to hang an explicit kill off, unlike a WS
/// stream's read-driven disconnect detection in
/// `routes_orchestrate.rs`/`routes_audit.rs`. Rust's `Drop` runs
/// unconditionally, including when a `Future` is dropped mid-poll, so a
/// local of this type dropped alongside the rest of [`run_omega_duo`]'s
/// stack frame is the one mechanism that fires on EVERY exit path —
/// disarmed on the two that already have their own, more specific handling
/// (normal completion: nothing to kill; the explicit timeout branch: its
/// own synchronous kill already ran) so this never fires a
/// harmless-but-redundant second kill there.
struct KillGroupOnDrop {
    pid: u32,
    armed: bool,
}

impl KillGroupOnDrop {
    fn new(pid: u32) -> Self {
        Self { pid, armed: true }
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for KillGroupOnDrop {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let pid = self.pid;
        // `Drop::drop` is synchronous and cannot `.await`; this fires from
        // within the Tokio worker thread tearing down the dropped future's
        // stack (a client disconnect — or the ordinary return path, before
        // an explicit `disarm()`, as a belt-and-suspenders default), which
        // is always still inside a live Tokio runtime context, so
        // `tokio::spawn` is safe to call here.
        tokio::spawn(async move {
            kill_process_group(pid).await;
        });
    }
}

struct DuoOutput {
    stdout: String,
    stderr: String,
}

/// Runs `omega-duo <args>` as a killable, time-bounded async child —
/// `tokio::process::Command`, NOT this crate's usual blocking `omega_cli::
/// run` (which cannot be cancelled once spawned), mirroring `routes_pdf.
/// rs::run_omega_pdf`'s exact idiom. `process_group(0)` here is a REAL
/// cancellation mechanism, not decorative — see [`kill_process_group`]'s
/// doc comment for why this differs from `routes_new_project.rs`'s
/// documented no-op case. Two things can trigger a kill: [`duo_timeout`]
/// firing (handled explicitly below), AND a CLIENT disconnect (Finding 2,
/// adversarial review round) — this endpoint is a plain request/response
/// handler with no WebSocket read loop to detect a disconnect the way
/// `routes_orchestrate.rs`/`routes_audit.rs` do, so axum's only signal is
/// simply DROPPING this whole handler future mid-poll. [`KillGroupOnDrop`]
/// is the mechanism that reaches that case: it fires unconditionally on
/// `Drop`, which Rust guarantees even for a future dropped mid-await, and
/// is explicitly disarmed on the paths that already have their own
/// handling (normal completion, and the timeout branch's own synchronous
/// kill).
async fn run_omega_duo(args: &[&str]) -> Result<DuoOutput, ApiError> {
    let bin = duo_bin();
    let mut cmd = tokio::process::Command::new(&bin);
    cmd.args(args);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.kill_on_drop(true);
    cmd.process_group(0);

    let child = cmd
        .spawn()
        .map_err(|e| bad_gateway(format!("failed to spawn omega-duo: {e}")))?;
    // Captured now, before the child is ever consumed by `wait_with_output`
    // (which takes `self` by value) — `Child::id()` returns `None` once the
    // child has been polled to completion.
    let child_pid = child.id();

    // Finding 2 (adversarial review round): armed the moment the child
    // exists, so a client disconnect ANYWHERE past this point — which
    // simply drops this whole future, axum's only signal for a plain POST
    // handler — still reaches the nested agent turn via `Drop`. See
    // [`KillGroupOnDrop`]'s doc comment.
    let mut kill_guard = child_pid.map(KillGroupOnDrop::new);

    match tokio::time::timeout(duo_timeout(), child.wait_with_output()).await {
        Ok(Ok(out)) => {
            // Completed on its own — nothing to kill.
            if let Some(g) = kill_guard.as_mut() {
                g.disarm();
            }
            Ok(DuoOutput {
                stdout: String::from_utf8_lossy(&out.stdout).to_string(),
                stderr: String::from_utf8_lossy(&out.stderr).to_string(),
            })
        }
        Ok(Err(e)) => {
            // `wait_with_output` itself failed — an unusual case where the
            // child's real state is unknown. Deliberately left ARMED: the
            // guard's `Drop` firing a kill here is the safe default when
            // "is anything left to clean up" cannot be answered with
            // confidence.
            Err(bad_gateway(format!("omega-duo process error: {e}")))
        }
        Err(_) => {
            // The timed-out future owned `child`; dropping it here (it was
            // already dropped when the future was) triggers `kill_on_drop`
            // for the DIRECT child. That alone does not reach a nested
            // Codex/Claude turn, so also send an explicit whole-group kill
            // using the PID captured above — see [`kill_process_group`].
            // Awaited (unlike `KillGroupOnDrop`'s fire-and-forget spawn) so
            // the response is only returned once the kill syscall has
            // actually been issued; the guard is then disarmed so its own
            // `Drop` does not fire a harmless-but-redundant second kill on
            // the way out.
            if let Some(pid) = child_pid {
                kill_process_group(pid).await;
            }
            if let Some(g) = kill_guard.as_mut() {
                g.disarm();
            }
            Err(gateway_timeout(format!(
                "omega-duo timed out after {}s and was killed",
                duo_timeout().as_secs()
            )))
        }
    }
}

/// `POST /v1/duo` — see this module's doc comment for the full ground
/// truth. Validates everything BEFORE any subprocess spawn, in order:
/// concurrency permit, `profile`, `prompt`, exactly-one-of `project`/`dir`,
/// the resolved target path, the per-cwd lock — then writes the scratch
/// task file and runs `omega-duo run`.
pub async fn create(
    State(state): State<AppState>,
    Json(req): Json<DuoRequest>,
) -> Result<Json<DuoResponse>, ApiError> {
    // Step 0: acquire a concurrency permit as the FIRST thing this handler
    // does — the cheapest possible short-circuit, mirroring
    // `routes_pdf::create`'s / `routes_dispatch::create`'s own Step 0.
    let Ok(_permit) = state.duo_permits.clone().try_acquire_owned() else {
        return Err(too_many_requests(
            "too many concurrent duo runs, try again shortly",
        ));
    };

    // Step 1: `profile` -> `--mode`, validated against the literal closed
    // set before anything else touches the filesystem or a subprocess.
    let Some(mode) = profile_to_mode(&req.profile) else {
        return Err(bad_request(format!(
            "unknown profile '{}' (expected one of: build, review, reflect)",
            req.profile
        )));
    };

    // Step 2: `prompt` — non-empty, no NUL byte, length-capped.
    if req.prompt.trim().is_empty() {
        return Err(bad_request("prompt must not be empty"));
    }
    if req.prompt.contains('\0') {
        return Err(bad_request("prompt must not contain a NUL byte"));
    }
    if req.prompt.len() > MAX_DUO_PROMPT_LEN {
        return Err(bad_request(format!(
            "prompt too long (max {MAX_DUO_PROMPT_LEN} bytes)"
        )));
    }

    // Step 3: exactly one of `project`/`dir`, then resolve it to a real,
    // server-validated on-disk path — no subprocess is ever spawned for an
    // ambiguous, missing, unknown, or out-of-bounds target.
    let target = match (&req.project, &req.dir) {
        (Some(_), Some(_)) => {
            return Err(bad_request(
                "ambiguous target: give exactly one of project or dir",
            ))
        }
        (None, None) => return Err(bad_request("no target: give project or dir")),
        (Some(p), None) => resolve_project_path(p).await?,
        (None, Some(d)) => crate::routes_sessions::dir_under_home(d)?,
    };

    // Step 4 (Finding 3, adversarial review round): the resolved target
    // must be an absolute path BEFORE it is ever used as a lock key or
    // reaches argv. `dir_under_home` returns the caller's ORIGINAL,
    // uncanonicalized string, and its ancestor-walk canonicalizes a
    // RELATIVE candidate against THIS PROCESS's own cwd — so a `dir` that
    // is itself relative (e.g. a bare `--verify`) could, in the
    // pathological case where a same-named directory exists relative to
    // gatewayd's own cwd AND resolves under `$HOME`, slip through as a
    // target whose full string starts with `--`. See this module's doc
    // comment's HYPHEN-LEADING VALUE RISK section for why that specific
    // shape is a real argv-injection into `omega-duo`'s `--verify` flag,
    // not just a harmless bad-input rejection. Requiring an absolute path
    // is simpler and strictly more robust than a narrow
    // `starts_with("--")` check: every legitimate resolved directory (a
    // discovered project, or a `dir_under_home`-validated request) is
    // already absolute, so nothing legitimate is ever rejected here.
    if !target.is_absolute() {
        return Err(bad_request(
            "resolved target directory must be an absolute path",
        ));
    }

    // Step 5: per-cwd in-process lock, BEFORE writing the scratch file or
    // spawning anything — keyed on the resolved REPO ROOT, not the raw
    // `target` path (Finding 1, adversarial review round: see
    // [`repo_root_for_lock`]'s doc comment for the full reasoning and the
    // judgment call on why `--cwd` itself stays unchanged below).
    let lock_key = repo_root_for_lock(&target);
    let _cwd_guard = acquire_cwd_lock(&state.duo_active_dirs, lock_key)?;

    // Step 6: write `prompt` to a SERVER-CHOSEN scratch file — never a
    // client-supplied path reaching `--task`.
    let tasks_dir = duo_tasks_dir();
    let task_path = tasks_dir.join(format!("{}.md", crate::util::random_hex(8)));
    let task_path_str = task_path.to_string_lossy().to_string();
    let prompt = req.prompt.clone();
    tokio::task::spawn_blocking({
        let tasks_dir = tasks_dir.clone();
        let task_path = task_path.clone();
        move || -> Result<(), ApiError> {
            std::fs::create_dir_all(&tasks_dir)
                .map_err(|e| internal(format!("mkdir {}: {e}", tasks_dir.display())))?;
            std::fs::write(&task_path, prompt)
                .map_err(|e| internal(format!("write task file: {e}")))?;
            Ok(())
        }
    })
    .await
    .map_err(|e| internal(format!("duo setup task panicked: {e}")))??;

    // Step 7: run `omega-duo run --task <scratch> --cwd <resolved> --mode
    // <mode>` — plain space-separated argv, no `--` separator and no
    // `=`-joined flags (see this module's doc comment for why both would be
    // wrong for THIS binary's own arg parser).
    let cwd_str = target.to_string_lossy().to_string();
    let output = run_omega_duo(&[
        "run",
        "--task",
        &task_path_str,
        "--cwd",
        &cwd_str,
        "--mode",
        mode,
    ])
    .await?;

    // Step 8: parse EXACTLY one JSON line from stdout — the LAST non-empty
    // line, not the whole buffer (Finding 4, adversarial review round): the
    // real bridge's own test harness deliberately parses only the last
    // non-empty line (`stdout.trim().split("\n").pop()`), documenting that
    // the JSON result IS the last line, not necessarily the whole stream —
    // a stray banner/log/warning line before it is possible. Parsing the
    // ENTIRE buffer turned any such stray line into a 502 for what may have
    // been a successful, quota-burning, potentially file-mutating run, with
    // no `checkpoint`/`diffstat`/`agent_ok` in the response — the worst
    // possible failure shape, since the caller could not tell whether the
    // worktree was touched. A non-zero `omega-duo` exit is still NOT gated
    // on here — see this module's doc comment ("SUCCESS SIGNAL") for why
    // the JSON body's own `ok`/`agent_ok` fields are the real outcome,
    // never this endpoint's own HTTP status.
    // M-1 (Codex cross-model review, 2026-08-11): the raw stdout/stderr used
    // to be echoed straight into the HTTP response on an unparseable
    // result, which can leak environment-derived secrets or other sensitive
    // text Codex/Claude/GLM (or the bridge itself) wrote. The FULL raw text
    // still goes to the gateway's own tracing log; the client only ever
    // sees the parse error (a generic "expected value at line X" shape,
    // never the offending text itself) with no raw dump attached.
    let last_line = output
        .stdout
        .lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("");
    match serde_json::from_str::<DuoResponse>(last_line.trim()) {
        Ok(resp) => Ok(Json(resp)),
        Err(e) => {
            tracing::error!(
                stdout = %output.stdout,
                stderr = %output.stderr,
                "omega-duo produced no parseable result: {e}"
            );
            Err((
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({
                    "error": format!("omega-duo produced no parseable result: {e} (see gateway logs)"),
                })),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Both tests below mutate process-global env vars; without a lock they
    // race across cargo's parallel test threads (same pattern as
    // `omega_cli.rs`'s own LOCK / `routes_pdf.rs`'s own LOCK).
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn duo_bin_honors_env_override() {
        let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::set_var("OMEGA_DUO_BIN", "/tmp/fake-omega-duo");
        assert_eq!(duo_bin(), PathBuf::from("/tmp/fake-omega-duo"));
        std::env::remove_var("OMEGA_DUO_BIN");
    }

    #[test]
    fn duo_bin_default_is_local_bin() {
        let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("OMEGA_DUO_BIN");
        let bin = duo_bin();
        assert!(
            bin.ends_with(".local/bin/omega-duo"),
            "expected ~/.local/bin/omega-duo, got {}",
            bin.display()
        );
    }

    #[test]
    fn profile_to_mode_maps_the_skills_three_real_profiles() {
        assert_eq!(profile_to_mode("build"), Some("code"));
        assert_eq!(profile_to_mode("review"), Some("review"));
        assert_eq!(profile_to_mode("reflect"), Some("plan"));
    }

    #[test]
    fn profile_to_mode_rejects_anything_else() {
        assert_eq!(profile_to_mode(""), None);
        assert_eq!(profile_to_mode("code"), None);
        assert_eq!(profile_to_mode("plan"), None);
        assert_eq!(profile_to_mode("Build"), None);
    }

    #[test]
    fn cwd_lock_guard_removes_its_entry_on_drop() {
        let set: Arc<Mutex<HashSet<PathBuf>>> = Arc::new(Mutex::new(HashSet::new()));
        let path = PathBuf::from("/tmp/some-project");
        {
            let _guard = acquire_cwd_lock(&set, path.clone()).unwrap();
            assert!(set.lock().unwrap().contains(&path));
            // A second acquire against the SAME path while the guard is
            // still alive is refused.
            assert!(acquire_cwd_lock(&set, path.clone()).is_err());
        }
        // Dropped — the entry is gone, and the path can be locked again.
        assert!(!set.lock().unwrap().contains(&path));
        assert!(acquire_cwd_lock(&set, path).is_ok());
    }
}

//! Generic subprocess wrapper around the `omega` CLI binary.
//!
//! This module mirrors the SHAPE of `rmux.rs` (argv-only `Command::new`, an
//! env-var override for the binary path) but deliberately deviates from its
//! error-handling: `rmux::run()` treats a non-zero exit as an `Err`, but for
//! `omega` a non-zero exit (e.g. "unknown project") is a NORMAL outcome the
//! caller inspects via `CommandOutput::success` — never an error. Only a
//! genuine spawn failure (binary missing/not executable) returns `Err`.
//!
//! This module has no route handlers and no knowledge of dispatch/oracle
//! semantics — it is a thin "run the omega binary with these args, hand back
//! stdout/stderr/success" primitive that Tasks 7 and 8 build typed logic on
//! top of.

use anyhow::{Context, Result};
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;

/// Resolves the `omega` binary path: `OMEGA_BIN` env override, else
/// `~/.local/bin/omega` (confirmed via `which omega` to match the real
/// installed path on this box, matching every other OmegaOS component's
/// PATH convention — same shape as `rmux::rmux_bin()`).
pub fn omega_bin() -> PathBuf {
    if let Ok(bin) = std::env::var("OMEGA_BIN") {
        return PathBuf::from(bin);
    }
    dirs::home_dir()
        .expect("no home dir")
        .join(".local/bin/omega")
}

/// Captured output of an `omega` subprocess invocation.
#[derive(Debug)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

/// Runs `omega <args>` and captures stdout/stderr separately. A non-zero
/// exit is a normal outcome reflected in `CommandOutput::success`, never an
/// `Err` — only a spawn failure (binary missing/not executable) errors.
pub fn run(args: &[&str]) -> Result<CommandOutput> {
    let out = Command::new(omega_bin()).args(args).output()?;
    Ok(CommandOutput {
        stdout: String::from_utf8_lossy(&out.stdout).to_string(),
        stderr: String::from_utf8_lossy(&out.stderr).to_string(),
        success: out.status.success(),
    })
}

/// I-2 (Codex cross-model review, 2026-08-11): `run`'s blocking
/// `Command::output()` has no timeout and no cancellation path once
/// spawned — a caller who drops the future (an HTTP client disconnect) does
/// not stop the child, and a hung/adversarially-slow `omega` subprocess can
/// pin a `spawn_blocking` thread (and whatever concurrency permit the
/// caller holds) forever. `run_with_timeout` bounds the child's wall-clock
/// lifetime: past `timeout`, the WHOLE process group is killed
/// (`process_group(0)` + a negative-PID `kill -- -<pid>`, the crate's
/// established idiom — see `routes_duo.rs::kill_process_group`'s doc
/// comment for why a single-PID kill is not enough for a CLI that may run
/// its own nested foreground children), not just the direct child.
///
/// Deliberately synchronous (a watchdog THREAD racing `wait_with_output`
/// via a channel, never `tokio::process`/`tokio::time::timeout`): every
/// existing caller of this module already runs `run` inside a
/// `spawn_blocking` task, so `run_with_timeout` keeps that exact shape —
/// swapping one blocking call for another — rather than pulling a tokio
/// runtime dependency into a module that otherwise has none. The main
/// thread's own `wait_with_output()` correctly drains stdout/stderr with no
/// pipe-deadlock risk; the watchdog thread only ever does one thing: kill
/// the group if it is not told "the child already finished" within
/// `timeout`.
///
/// On a timeout, the returned `anyhow::Error` wraps a [`TimedOut`] marker —
/// distinct from a spawn failure or an ordinary non-zero exit (which is
/// still reflected in `Ok(CommandOutput { success: false, .. })`, exactly
/// like `run`) — so a caller can `downsize` it via [`is_timeout`] and map
/// it to a 504 instead of the usual 502. See [`is_timeout`].
pub fn run_with_timeout(args: &[&str], timeout: Duration) -> Result<CommandOutput> {
    let mut cmd = Command::new(omega_bin());
    cmd.args(args);
    // `Command::spawn()` (unlike `Command::output()`, which `run` uses)
    // does NOT pipe stdout/stderr by default — it inherits the parent's,
    // which would leak straight into the gateway process's own streams and
    // leave `wait_with_output()` capturing nothing. Explicit here since
    // this function bypasses `output()`'s auto-piping to get a killable,
    // time-bounded `Child` instead.
    cmd.stdin(std::process::Stdio::null());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    // Standard "detach into a new group" idiom (the group ID becomes the
    // child's own PID) — see routes_duo.rs::run_omega_duo's identical use
    // for the full rationale: this is what lets a group-kill reach a nested
    // foreground child `omega` itself spawns, not just `omega` directly.
    cmd.process_group(0);

    let child = cmd
        .spawn()
        .with_context(|| format!("failed to spawn omega {args:?}"))?;
    let pid = child.id();

    // `killed` is the ONLY source of truth for "did the watchdog actually
    // fire the kill" — `wait_with_output()` returning `Ok` after a SIGKILL
    // looks identical (from the return type alone) to the child exiting on
    // its own with a killed-by-signal status, so the timeout branch below
    // cannot be inferred from the wait result alone.
    let killed = Arc::new(AtomicBool::new(false));
    let (done_tx, done_rx) = mpsc::channel::<()>();
    let watchdog_killed = Arc::clone(&killed);
    let watchdog = std::thread::spawn(move || {
        // `recv_timeout` returns `Err` on EITHER a timeout OR the sender
        // being dropped without a send — both cases here only ever mean
        // "the main thread never signalled done in time", since the main
        // thread always sends before dropping `done_tx`. A signal received
        // in time means the child already finished; nothing to kill.
        if done_rx.recv_timeout(timeout).is_err() {
            watchdog_killed.store(true, Ordering::SeqCst);
            kill_process_group_sync(pid);
        }
    });

    let wait_result = child.wait_with_output();
    // Whatever the outcome, tell the watchdog the race is over. A send
    // failure means the watchdog already fired (it's not receiving
    // anymore) — harmless, `killed` already reflects that.
    let _ = done_tx.send(());
    let _ = watchdog.join();

    if killed.load(Ordering::SeqCst) {
        return Err(anyhow::Error::new(TimedOut {
            args: args.join(" "),
            timeout,
        }));
    }

    let out = wait_result.with_context(|| format!("omega {args:?} process error"))?;
    Ok(CommandOutput {
        stdout: String::from_utf8_lossy(&out.stdout).to_string(),
        stderr: String::from_utf8_lossy(&out.stderr).to_string(),
        success: out.status.success(),
    })
}

/// Env-var override for [`run_with_timeout`]'s default ceiling — the same
/// env-var-overridable-constant-fn shape `routes_duo.rs::duo_timeout()`
/// establishes for a per-endpoint subprocess timeout in this crate. These
/// callers (session/team/dispatch create, oracle reap/resurrect) are meant
/// to be quick "spawn a session/mission" calls, not a long agent turn — far
/// tighter than `duo_timeout()`'s 1800s default.
const DEFAULT_CLI_TIMEOUT_SECS: u64 = 120;

pub fn cli_timeout() -> Duration {
    let secs = std::env::var("OMEGA_CLI_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(DEFAULT_CLI_TIMEOUT_SECS);
    Duration::from_secs(secs)
}

/// Marker error wrapped inside [`run_with_timeout`]'s `anyhow::Error` on a
/// timeout — see [`is_timeout`] for how a caller detects it.
#[derive(Debug)]
struct TimedOut {
    args: String,
    timeout: Duration,
}

impl std::fmt::Display for TimedOut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "omega {} timed out after {}s and was killed",
            self.args,
            self.timeout.as_secs()
        )
    }
}

impl std::error::Error for TimedOut {}

/// `true` iff `e` is the specific [`TimedOut`] marker [`run_with_timeout`]
/// wraps on a timeout — never true for a spawn failure or a `wait` error,
/// both of which stay generic `anyhow` context. A caller uses this to
/// choose a 504 (`StatusCode::GATEWAY_TIMEOUT`) instead of the usual 502
/// (`StatusCode::BAD_GATEWAY`) `run_with_timeout`'s other error paths get.
pub fn is_timeout(e: &anyhow::Error) -> bool {
    e.downcast_ref::<TimedOut>().is_some()
}

/// Sends `SIGKILL` to the WHOLE process group rooted at `pid` — the same
/// negative-PID `kill -- -<pid>` idiom as `routes_duo.rs::
/// kill_process_group`, run synchronously since this function (unlike that
/// one) is itself always called from a plain OS thread, never a tokio task.
fn kill_process_group_sync(pid: u32) {
    let _ = Command::new("kill")
        .arg("--")
        .arg(format!("-{pid}"))
        .status();
}

/// Default outer wall-clock bound on a WS-stream endpoint's WHOLE
/// connection lifetime — I-4 (Codex cross-model review, 2026-08-11):
/// `routes_agents::install_stream_loop`, `routes_audit::audit_stream_loop`,
/// `routes_orchestrate::orchestrate_stream_loop`, and
/// `routes_new_project::new_project_stream_loop` each already kill their
/// process group on client DISCONNECT, but none of them bounded how long a
/// QUIET-BUT-ALIVE child (no output, client still connected) could hold the
/// connection and subprocess open. `OMEGA_STREAM_TIMEOUT_SECS`
/// env-overridable, same env-var-overridable-constant-fn shape as
/// `routes_duo.rs::duo_timeout()` — whose 1800s default this mirrors, since
/// these are the same kind of genuinely long-running operation (an install,
/// a full audit, an end-to-end orchestrate run) `duo_timeout()`'s own doc
/// comment reasons about.
const DEFAULT_STREAM_TIMEOUT_SECS: u64 = 1800;

pub fn stream_timeout() -> Duration {
    let secs = std::env::var("OMEGA_STREAM_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(DEFAULT_STREAM_TIMEOUT_SECS);
    Duration::from_secs(secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Both tests mutate the process-global OMEGA_BIN env var, so they must
    // never run concurrently with each other. Acquire this lock at the start
    // of each test to serialize them regardless of the test harness's thread
    // count (same pattern as tests/sessions_test.rs's OMEGA_RMUX_BIN LOCK).
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Writes an executable fake `omega` script and points OMEGA_BIN at it.
    fn install_fake_omega(dir: &std::path::Path, script_body: &str) {
        use std::os::unix::fs::PermissionsExt;
        let path = dir.join("omega");
        std::fs::write(&path, format!("#!/usr/bin/env bash\n{script_body}\n")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        std::env::set_var("OMEGA_BIN", &path);
    }

    #[test]
    fn success_captures_stdout() {
        let _g = LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        install_fake_omega(
            dir.path(),
            r#"printf 'oracle-Verba-1\nworker-a\n'; printf 'note: nothing wrong' >&2; exit 0"#,
        );
        let out = run(&["projects", "--json"]).unwrap();
        assert_eq!(out.stdout, "oracle-Verba-1\nworker-a\n");
        assert!(out.success);
    }

    #[test]
    fn nonzero_exit_is_not_an_error() {
        let _g = LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        install_fake_omega(dir.path(), "echo 'unknown project' >&2; exit 1");
        let out = run(&["projects", "--json"]).unwrap();
        assert!(!out.success);
        assert!(out.stderr.contains("unknown project"));
    }

    // ── run_with_timeout (I-2) ──────────────────────────────────────────

    #[test]
    fn run_with_timeout_returns_normal_output_when_the_child_finishes_in_time() {
        let _g = LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        install_fake_omega(dir.path(), "printf 'hello\\n'; exit 0");
        let out = run_with_timeout(&["x"], Duration::from_secs(5)).unwrap();
        assert_eq!(out.stdout, "hello\n");
        assert!(out.success);
    }

    #[test]
    fn run_with_timeout_reflects_a_normal_nonzero_exit_as_ok_not_timed_out() {
        let _g = LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        install_fake_omega(dir.path(), "echo 'boom' >&2; exit 1");
        let out = run_with_timeout(&["x"], Duration::from_secs(5)).unwrap();
        assert!(!out.success);
        assert!(out.stderr.contains("boom"));
    }

    #[test]
    fn run_with_timeout_returns_a_distinguishable_timeout_error_promptly() {
        let _g = LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        // Sleeps far longer than the timeout below -- if the child were not
        // actually killed, this test would hang for 5s instead of returning
        // promptly.
        install_fake_omega(dir.path(), "sleep 5; exit 0");
        let start = std::time::Instant::now();
        let err = run_with_timeout(&["x"], Duration::from_millis(200)).unwrap_err();
        let elapsed = start.elapsed();
        assert!(is_timeout(&err), "expected a TimedOut error, got: {err}");
        assert!(
            elapsed < Duration::from_secs(2),
            "run_with_timeout did not return promptly after killing the child: {elapsed:?}"
        );
    }

    #[test]
    fn run_with_timeout_never_reports_timed_out_for_a_spawn_failure() {
        let _g = LOCK.lock().unwrap();
        std::env::set_var("OMEGA_BIN", "/no/such/binary/anywhere");
        let err = run_with_timeout(&["x"], Duration::from_secs(5)).unwrap_err();
        assert!(
            !is_timeout(&err),
            "a spawn failure must never be classified as a timeout"
        );
        std::env::remove_var("OMEGA_BIN");
    }

    /// The whole POINT of `process_group(0)` + a negative-PID kill: a timeout
    /// must reach a NESTED child the direct process spawned, not just the
    /// direct process itself -- same marker-file idiom
    /// `agents_install_adversarial_test.rs` uses for the WS-stream version
    /// of this same property.
    #[test]
    fn run_with_timeout_kills_the_whole_process_group_not_just_the_direct_child() {
        let _g = LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let marker = dir.path().join("marker");
        // The direct child backgrounds a nested `bash -c` (inheriting its
        // process group) and waits on it -- mirroring a real CLI that runs
        // its own foreground nested work. The nested sleep (2s) is longer
        // than the timeout (200ms) but the outer `wait` (which would
        // otherwise finish around 2s) is what proves the KILL, not a normal
        // return, ever happens.
        install_fake_omega(
            dir.path(),
            &format!(
                "bash -c 'sleep 2; touch \"{}\"' &\nwait\n",
                marker.display()
            ),
        );

        let err = run_with_timeout(&["x"], Duration::from_millis(200)).unwrap_err();
        assert!(is_timeout(&err));

        // Generous buffer past the nested sleep's 2s -- if the group kill
        // missed the nested child, the marker appears around the 2s mark.
        std::thread::sleep(Duration::from_secs(3));
        assert!(!marker.exists(), "the nested child survived the group kill");
    }
}

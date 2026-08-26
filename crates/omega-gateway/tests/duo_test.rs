//! `POST /v1/duo` — `crates/omega-gateway/src/routes_duo.rs` (wave8 Task D).
//!
//! Fake-`OMEGA_DUO_BIN`-tested ONLY: no test in this file ever invokes a
//! real Codex/Claude/GLM turn. `OMEGA_DUO_DIR` scopes every scratch file to
//! a tempdir, never the operator's real `~/.omega/state/gateway-duo`.
//! `project`-based tests point `OMEGA_HOME` at a fake discoverable-project
//! tree (mirrors `dispatch_test.rs::install_fake_home`); `dir`-based tests
//! override the REAL `HOME` env var instead, since `dir_under_home`
//! deliberately reads `dirs::home_dir()` rather than the `OMEGA_HOME`
//! override (mirrors `sessions_create_test.rs`'s own dir-based tests).

use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};

// OMEGA_DUO_BIN / OMEGA_DUO_DIR / OMEGA_HOME / HOME / DUO_TIMEOUT_SECS are
// process-global; serialize every test in this binary that touches any of
// them (same pattern as pdf_test.rs's / dispatch_test.rs's own LOCK).
static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn spawn(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

async fn app_and_token(gateway_dir: &std::path::Path) -> (axum::Router, String) {
    let (_, token) = DeviceStore::open(gateway_dir).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.to_path_buf(),
        GatewayConfig::default(),
    ));
    (app, token)
}

/// Creates a fake `$OMEGA_HOME` containing exactly one discoverable project
/// (a `.git`-marked directory named `name`), and points `OMEGA_HOME` at it
/// — the override `config::home_dir()` / `routes_duo::resolve_project_path`
/// respects (identical to `dispatch_test.rs::install_fake_home`).
fn install_fake_home(home_dir: &std::path::Path, project_name: &str) {
    std::fs::create_dir_all(home_dir.join(project_name).join(".git")).unwrap();
    std::env::set_var("OMEGA_HOME", home_dir);
}

/// Writes an executable fake `omega-duo` that: (1) appends its full argv
/// (one per line) to `capture_file`; (2) optionally sleeps `sleep_secs`
/// (fractional seconds, bash `sleep` accepts e.g. `"0.15"`) BEFORE printing,
/// so concurrency/timeout tests can hold a run "in flight"; (3) prints
/// `stdout_body` VERBATIM (no trailing newline added by this helper -- the
/// caller decides); (4) exits `exit_code`. `stdout_body` must not contain a
/// single-quote character (this helper single-quotes it into the script).
fn install_fake_duo(
    bin_dir: &std::path::Path,
    capture_file: &std::path::Path,
    sleep_secs: &str,
    stdout_body: &str,
    exit_code: i32,
) {
    use std::os::unix::fs::PermissionsExt;
    assert!(
        !stdout_body.contains('\''),
        "test stdout body must not contain a single quote"
    );
    let path = bin_dir.join("omega-duo");
    let capture = capture_file.display();
    let body = format!(
        "#!/usr/bin/env bash\nprintf '%s\\n' \"$@\" > '{capture}'\nsleep {sleep_secs}\nprintf '%s' '{stdout_body}'\nexit {exit_code}\n"
    );
    std::fs::write(&path, body).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::env::set_var("OMEGA_DUO_BIN", &path);
}

fn clear_env() {
    std::env::remove_var("OMEGA_DUO_BIN");
    std::env::remove_var("OMEGA_DUO_DIR");
    std::env::remove_var("OMEGA_HOME");
    std::env::remove_var("HOME");
    std::env::remove_var("DUO_TIMEOUT_SECS");
}

fn argv_lines(capture_file: &std::path::Path) -> Vec<String> {
    std::fs::read_to_string(capture_file)
        .unwrap_or_default()
        .lines()
        .map(str::to_string)
        .collect()
}

/// Finding 2 (adversarial review round): writes an executable fake
/// `omega-duo` that forks a NESTED grandchild background loop -- writing a
/// growing marker file so its aliveness is directly observable, and its own
/// PID to `pid_file` via bash's `$!` -- into the SAME process group as the
/// outer script (no `setsid`/job-control subshell: bash job control is off
/// by default in non-interactive script mode, so a plain `&` background job
/// inherits the parent's pgid, mirroring the REAL `omega-duo` -> Codex/
/// Claude spawn shape this crate's `routes_duo.rs` doc comment verifies: no
/// `detached: true`, so a nested agent turn stays in the same process group
/// `run_omega_duo` places the direct child into). The outer script then
/// sleeps far longer than any test's disconnect window before ever printing
/// anything, so the request can only complete within test patience via the
/// disconnect-triggered kill this finding adds.
fn install_fake_duo_with_nested_child(
    bin_dir: &std::path::Path,
    capture_file: &std::path::Path,
    pid_file: &std::path::Path,
    marker_file: &std::path::Path,
) {
    use std::os::unix::fs::PermissionsExt;
    let path = bin_dir.join("omega-duo");
    let body = format!(
        "#!/usr/bin/env bash\nprintf '%s\\n' \"$@\" > '{capture}'\n( while true; do date +%s%N >> '{marker}'; sleep 0.05; done ) &\necho $! > '{pid}'\nsleep 120\nprintf '%s' '{{}}'\nexit 0\n",
        capture = capture_file.display(),
        marker = marker_file.display(),
        pid = pid_file.display(),
    );
    std::fs::write(&path, body).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::env::set_var("OMEGA_DUO_BIN", &path);
}

/// Finding 4 (adversarial review round): like [`install_fake_duo`], but
/// prints a harmless stray `banner` line to stdout BEFORE the real
/// `stdout_body` JSON line -- reproducing the real bridge's own documented
/// shape ("the JSON result is the last line, not necessarily the whole
/// stream").
fn install_fake_duo_with_banner_line(
    bin_dir: &std::path::Path,
    capture_file: &std::path::Path,
    banner: &str,
    stdout_body: &str,
    exit_code: i32,
) {
    use std::os::unix::fs::PermissionsExt;
    assert!(
        !stdout_body.contains('\''),
        "test stdout body must not contain a single quote"
    );
    assert!(
        !banner.contains('\''),
        "test banner must not contain a single quote"
    );
    let path = bin_dir.join("omega-duo");
    let capture = capture_file.display();
    let body = format!(
        "#!/usr/bin/env bash\nprintf '%s\\n' \"$@\" > '{capture}'\nprintf '%s\\n' '{banner}'\nprintf '%s' '{stdout_body}'\nexit {exit_code}\n"
    );
    std::fs::write(&path, body).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::env::set_var("OMEGA_DUO_BIN", &path);
}

/// RAII guard restoring the process's current directory on drop -- used
/// only by [`dash_prefixed_resolved_dir_is_rejected_before_any_spawn`],
/// which must temporarily move the test binary's own cwd to reproduce
/// Finding 3's exact precondition (a RELATIVE `dir` value that
/// `dir_under_home` resolves against this PROCESS's own cwd). Restoring on
/// `Drop` (rather than only at the end of the happy path) keeps a panic mid
/// -test from leaving every later test in this same test binary running
/// from the wrong directory.
struct CwdRestore(std::path::PathBuf);
impl Drop for CwdRestore {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.0);
    }
}

/// The full real `BridgeResult` shape, filled with distinguishable
/// non-default values in every field so a field-mapping bug (a swapped or
/// dropped field) cannot hide behind a coincidental default.
fn fake_bridge_result_json() -> serde_json::Value {
    serde_json::json!({
        "agent": "codex",
        "ok": true,
        "output": "implemented the thing",
        "fell_back": false,
        "reason": null,
        "exit_code": 0,
        "log": "/home/x/.omega/logs/duo/20260101-run.log",
        "sandbox_degraded": false,
        "capabilities": { "shell_exec": true, "worktree_read": true },
        "guard_error": null,
        "verify": null,
        "checkpoint": { "head": "abc123", "stash": null, "ref": "refs/heads/main" },
        "diffstat": "1 file changed, 3 insertions(+)",
        "agent_ok": true
    })
}

// ── happy paths ───────────────────────────────────────────────────────────

#[tokio::test]
async fn happy_path_with_project_builds_exact_argv_and_maps_every_field() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    install_fake_home(home_dir.path(), "TestProj");
    let body = fake_bridge_result_json();
    install_fake_duo(bin_dir.path(), &capture_file, "0", &body.to_string(), 0);
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "TestProj", "prompt": "fix the bug", "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let resp: serde_json::Value = res.json().await.unwrap();
    // Field-for-field: the response IS the fake's JSON, mapped through
    // DuoResponse without loss or renaming (`ref` -> `git_ref` in Rust, but
    // still `ref` on the wire).
    assert_eq!(resp, body);

    let argv = argv_lines(&capture_file);
    let project_path = home_dir.path().join("TestProj");
    assert_eq!(argv[0], "run");
    let task_idx = argv.iter().position(|l| l == "--task").unwrap();
    let task_path = &argv[task_idx + 1];
    assert!(
        task_path.starts_with(duo_scratch.path().join("tasks").to_str().unwrap()),
        "argv: {argv:?}"
    );
    let cwd_idx = argv.iter().position(|l| l == "--cwd").unwrap();
    assert_eq!(argv[cwd_idx + 1], project_path.to_str().unwrap());
    let mode_idx = argv.iter().position(|l| l == "--mode").unwrap();
    assert_eq!(argv[mode_idx + 1], "code");
    // --agent / --verify are NEVER passed by this endpoint.
    assert!(
        !argv.iter().any(|l| l == "--agent" || l == "--verify"),
        "argv: {argv:?}"
    );
    // No `--` separator (omega-duo's own parser has no positionals to
    // protect) and no `=`-joined flags (its parser does not understand
    // them -- see routes_duo.rs's doc comment).
    assert!(!argv.iter().any(|l| l == "--"), "argv: {argv:?}");
    assert!(!argv.iter().any(|l| l.contains('=')), "argv: {argv:?}");

    // The scratch task file genuinely contains the caller's prompt.
    let on_disk = std::fs::read_to_string(task_path).unwrap();
    assert_eq!(on_disk, "fix the bug");

    clear_env();
}

#[tokio::test]
async fn happy_path_with_dir_uses_the_dir_under_home_resolved_path() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let fake_home = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    std::env::set_var("HOME", fake_home.path());
    let body = fake_bridge_result_json();
    install_fake_duo(bin_dir.path(), &capture_file, "0", &body.to_string(), 0);
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let dir_str = fake_home
        .path()
        .join("Station")
        .join("Proj")
        .display()
        .to_string();
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "dir": dir_str, "prompt": "think about the architecture", "profile": "reflect" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let argv = argv_lines(&capture_file);
    let cwd_idx = argv.iter().position(|l| l == "--cwd").unwrap();
    assert_eq!(argv[cwd_idx + 1], dir_str);
    let mode_idx = argv.iter().position(|l| l == "--mode").unwrap();
    assert_eq!(argv[mode_idx + 1], "plan");

    clear_env();
}

#[tokio::test]
async fn profile_review_maps_to_mode_review() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    install_fake_home(home_dir.path(), "TestProj");
    install_fake_duo(
        bin_dir.path(),
        &capture_file,
        "0",
        &fake_bridge_result_json().to_string(),
        0,
    );
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "TestProj", "prompt": "review this diff", "profile": "review" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let argv = argv_lines(&capture_file);
    let mode_idx = argv.iter().position(|l| l == "--mode").unwrap();
    assert_eq!(argv[mode_idx + 1], "review");

    clear_env();
}

// ── validation, before any spawn ─────────────────────────────────────────

#[tokio::test]
async fn both_project_and_dir_given_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");
    install_fake_duo(
        bin_dir.path(),
        &capture_file,
        "0",
        &fake_bridge_result_json().to_string(),
        0,
    );
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "X", "dir": "/tmp/y", "prompt": "p", "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("ambiguous"));
    assert!(!capture_file.exists(), "no subprocess was ever spawned");

    clear_env();
}

#[tokio::test]
async fn neither_project_nor_dir_given_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");
    install_fake_duo(
        bin_dir.path(),
        &capture_file,
        "0",
        &fake_bridge_result_json().to_string(),
        0,
    );
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "prompt": "p", "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("no target"));
    assert!(!capture_file.exists());

    clear_env();
}

#[tokio::test]
async fn unknown_project_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    install_fake_home(home_dir.path(), "RealProj");
    install_fake_duo(
        bin_dir.path(),
        &capture_file,
        "0",
        &fake_bridge_result_json().to_string(),
        0,
    );
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "GhostProj", "prompt": "p", "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(
        !capture_file.exists(),
        "no subprocess was ever spawned for an unknown project"
    );

    clear_env();
}

#[tokio::test]
async fn dir_outside_home_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let fake_home = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    std::env::set_var("HOME", fake_home.path());
    install_fake_duo(
        bin_dir.path(),
        &capture_file,
        "0",
        &fake_bridge_result_json().to_string(),
        0,
    );
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    // /etc definitely exists and is definitely not under the fake $HOME.
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "dir": "/etc", "prompt": "p", "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(!capture_file.exists());

    clear_env();
}

#[tokio::test]
async fn dir_with_parent_dir_component_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let fake_home = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    std::env::set_var("HOME", fake_home.path());
    install_fake_duo(
        bin_dir.path(),
        &capture_file,
        "0",
        &fake_bridge_result_json().to_string(),
        0,
    );
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let escaping = fake_home
        .path()
        .join("does-not-exist-yet")
        .join("..")
        .join("..")
        .join("etc");
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "dir": escaping.display().to_string(), "prompt": "p", "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(!capture_file.exists());

    clear_env();
}

#[tokio::test]
async fn empty_prompt_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    install_fake_home(home_dir.path(), "TestProj");
    install_fake_duo(
        bin_dir.path(),
        &capture_file,
        "0",
        &fake_bridge_result_json().to_string(),
        0,
    );
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "TestProj", "prompt": "   ", "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(!capture_file.exists());

    clear_env();
}

#[tokio::test]
async fn oversized_prompt_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    install_fake_home(home_dir.path(), "TestProj");
    install_fake_duo(
        bin_dir.path(),
        &capture_file,
        "0",
        &fake_bridge_result_json().to_string(),
        0,
    );
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let too_long = "x".repeat(8001);
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "TestProj", "prompt": too_long, "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(!capture_file.exists());

    clear_env();
}

#[tokio::test]
async fn nul_byte_prompt_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    install_fake_home(home_dir.path(), "TestProj");
    install_fake_duo(
        bin_dir.path(),
        &capture_file,
        "0",
        &fake_bridge_result_json().to_string(),
        0,
    );
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "TestProj", "prompt": "bad\u{0}prompt", "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(!capture_file.exists());

    clear_env();
}

#[tokio::test]
async fn unknown_profile_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    install_fake_home(home_dir.path(), "TestProj");
    install_fake_duo(
        bin_dir.path(),
        &capture_file,
        "0",
        &fake_bridge_result_json().to_string(),
        0,
    );
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "TestProj", "prompt": "p", "profile": "yolo" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("build"));
    assert!(!capture_file.exists());

    clear_env();
}

// ── malformed output ─────────────────────────────────────────────────────

#[tokio::test]
async fn malformed_stdout_is_502_with_a_sanitized_error_never_the_raw_output() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    install_fake_home(home_dir.path(), "TestProj");
    install_fake_duo(
        bin_dir.path(),
        &capture_file,
        "0",
        "this is not json at all",
        1,
    );
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "TestProj", "prompt": "p", "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 502);
    let body: serde_json::Value = res.json().await.unwrap();
    // M-1 (Codex cross-model review, 2026-08-11): raw stdout/stderr is no
    // longer echoed into the response body -- only the parse error itself
    // (a generic "expected value at..." shape). The full raw text still
    // goes to the gateway's own tracing log, never the HTTP response.
    assert!(
        body.get("stdout").is_none(),
        "must not echo raw stdout: {body}"
    );
    assert!(
        body.get("stderr").is_none(),
        "must not echo raw stderr: {body}"
    );
    assert!(
        !body["error"].as_str().unwrap().contains("not json"),
        "error message must not contain the raw subprocess text: {body}"
    );

    clear_env();
}

/// M-1 (Codex cross-model review, 2026-08-11): a secret-shaped string
/// written to stdout/stderr by a malformed `omega-duo` run must never reach
/// the HTTP response body -- only the gateway's own log.
#[tokio::test]
async fn malformed_stdout_never_leaks_a_secret_shaped_string_into_the_response() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    install_fake_home(home_dir.path(), "TestProj");
    install_fake_duo(
        bin_dir.path(),
        &capture_file,
        "0",
        "sk-ProjSECRETVALUE1234567890",
        1,
    );
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "TestProj", "prompt": "p", "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 502);
    let raw_body = res.text().await.unwrap();
    assert!(
        !raw_body.contains("sk-ProjSECRETVALUE1234567890"),
        "response body leaked the raw secret-shaped subprocess output: {raw_body}"
    );

    clear_env();
}

/// A non-zero `omega-duo` exit code with a VALID `BridgeResult` JSON body
/// (`ok:false`, a legitimate outcome like "verify-failed") is still a 200 —
/// the JSON body's own `ok`/`agent_ok` fields are the real signal, never
/// this endpoint's own HTTP status. See `routes_duo.rs`'s doc comment.
#[tokio::test]
async fn nonzero_exit_with_valid_json_is_still_200() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    install_fake_home(home_dir.path(), "TestProj");
    let mut body = fake_bridge_result_json();
    body["ok"] = serde_json::json!(false);
    body["agent_ok"] = serde_json::json!(true);
    body["reason"] = serde_json::json!("verify-failed");
    body["exit_code"] = serde_json::json!(1);
    install_fake_duo(bin_dir.path(), &capture_file, "0", &body.to_string(), 1);
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "TestProj", "prompt": "p", "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let resp: serde_json::Value = res.json().await.unwrap();
    assert_eq!(resp["ok"], false);
    assert_eq!(resp["agent_ok"], true);
    assert_eq!(resp["reason"], "verify-failed");

    clear_env();
}

// ── timeout ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn subprocess_past_a_short_overridden_timeout_returns_bounded_with_a_clear_error() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    install_fake_home(home_dir.path(), "TestProj");
    // Sleeps 3s; the endpoint's own timeout is overridden to 1s, so this
    // must return well under the full sleep, never the full 1800s default.
    install_fake_duo(
        bin_dir.path(),
        &capture_file,
        "3",
        &fake_bridge_result_json().to_string(),
        0,
    );
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());
    std::env::set_var("DUO_TIMEOUT_SECS", "1");

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let started = std::time::Instant::now();
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "TestProj", "prompt": "p", "profile": "build" }))
        .send()
        .await
        .unwrap();
    let elapsed = started.elapsed();
    assert_eq!(res.status(), 504);
    assert!(
        elapsed < std::time::Duration::from_secs(3),
        "took {elapsed:?}, expected well under the 3s sleep"
    );
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("timed out"));

    clear_env();
}

// ── concurrency ───────────────────────────────────────────────────────────

#[tokio::test]
async fn concurrency_cap_returns_429_when_duo_permits_exhausted() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    // Two DIFFERENT discoverable projects so the per-cwd lock (a SEPARATE
    // mechanism) never interferes with this test's concurrency-cap concern.
    std::fs::create_dir_all(home_dir.path().join("ProjA").join(".git")).unwrap();
    std::fs::create_dir_all(home_dir.path().join("ProjB").join(".git")).unwrap();
    std::fs::create_dir_all(home_dir.path().join("ProjC").join(".git")).unwrap();
    std::env::set_var("OMEGA_HOME", home_dir.path());
    install_fake_duo(
        bin_dir.path(),
        &capture_file,
        "0.15",
        &fake_bridge_result_json().to_string(),
        0,
    );
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;
    let client = reqwest::Client::new();

    // Must match server.rs's MAX_CONCURRENT_DUO_RUNS.
    const MAX_CONCURRENT_DUO_RUNS: usize = 2;
    let projects = ["ProjA", "ProjB"];

    let mut in_flight = Vec::new();
    for project in projects.iter().take(MAX_CONCURRENT_DUO_RUNS) {
        let client = client.clone();
        let base = base.clone();
        let token = token.clone();
        let project = project.to_string();
        in_flight.push(tokio::spawn(async move {
            client
                .post(format!("{base}/v1/duo"))
                .bearer_auth(&token)
                .json(&serde_json::json!({ "project": project, "prompt": "p", "profile": "build" }))
                .send()
                .await
                .unwrap()
                .status()
                .as_u16()
        }));
    }

    tokio::time::sleep(std::time::Duration::from_millis(60)).await;

    let busy_res = client
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "ProjC", "prompt": "p", "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(busy_res.status(), 429);
    let body: serde_json::Value = busy_res.json().await.unwrap();
    assert!(body["error"]
        .as_str()
        .unwrap()
        .contains("too many concurrent"));

    for task in in_flight {
        assert_eq!(task.await.unwrap(), 200);
    }

    clear_env();
}

#[tokio::test]
async fn per_cwd_lock_rejects_a_second_concurrent_run_then_releases_after_the_first_completes() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    install_fake_home(home_dir.path(), "TestProj");
    install_fake_duo(
        bin_dir.path(),
        &capture_file,
        "0.25",
        &fake_bridge_result_json().to_string(),
        0,
    );
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;
    let client = reqwest::Client::new();

    let first = {
        let client = client.clone();
        let base = base.clone();
        let token = token.clone();
        tokio::spawn(async move {
            client
                .post(format!("{base}/v1/duo"))
                .bearer_auth(&token)
                .json(&serde_json::json!({ "project": "TestProj", "prompt": "p1", "profile": "build" }))
                .send()
                .await
                .unwrap()
                .status()
                .as_u16()
        })
    };

    // Give the first request time to pass validation and acquire the
    // per-cwd lock before the second one (targeting the SAME project/cwd)
    // fires.
    tokio::time::sleep(std::time::Duration::from_millis(60)).await;

    let second_res = client
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "TestProj", "prompt": "p2", "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(second_res.status(), 409);
    let body: serde_json::Value = second_res.json().await.unwrap();
    assert!(body["error"]
        .as_str()
        .unwrap()
        .contains("already in flight"));

    assert_eq!(first.await.unwrap(), 200);

    // The lock was actually released, not leaked: a THIRD request against
    // the same project/cwd, fired only after the first completed, is
    // accepted.
    let third_res = client
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "TestProj", "prompt": "p3", "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(third_res.status(), 200);

    clear_env();
}

// ── adversarial review round fixes ───────────────────────────────────────

/// Finding 1: the per-cwd lock used to key on the raw, literal `dir` string
/// rather than the underlying repo it resolves to, so two textually
/// different paths into the SAME repo (a project root and a subdirectory of
/// it, both under one shared `.git`) bypassed the lock entirely and could
/// run concurrently against one worktree's checkpoint guard. After the fix,
/// the lock key is the resolved REPO ROOT, so the second request collides
/// exactly like the identical-path case already covered above.
#[tokio::test]
async fn per_cwd_lock_keys_on_repo_root_so_two_spellings_of_the_same_repo_collide() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let fake_home = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    std::env::set_var("HOME", fake_home.path());
    // A real repo root (marked by a `.git` directory) and a nested
    // subdirectory of it -- two textually different `dir` values that
    // resolve to the SAME underlying repo.
    let repo_root = fake_home.path().join("Proj");
    let nested = repo_root.join("src");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::create_dir_all(repo_root.join(".git")).unwrap();

    install_fake_duo(
        bin_dir.path(),
        &capture_file,
        "0.25",
        &fake_bridge_result_json().to_string(),
        0,
    );
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;
    let client = reqwest::Client::new();

    let first = {
        let client = client.clone();
        let base = base.clone();
        let token = token.clone();
        let dir_str = repo_root.display().to_string();
        tokio::spawn(async move {
            client
                .post(format!("{base}/v1/duo"))
                .bearer_auth(&token)
                .json(&serde_json::json!({ "dir": dir_str, "prompt": "p1", "profile": "build" }))
                .send()
                .await
                .unwrap()
                .status()
                .as_u16()
        })
    };

    // Give the first request time to pass validation and acquire the lock
    // before the second one (a DIFFERENT spelling of the SAME repo) fires.
    tokio::time::sleep(std::time::Duration::from_millis(60)).await;

    let second_dir_str = nested.display().to_string();
    let second_res = client
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "dir": second_dir_str, "prompt": "p2", "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        second_res.status(),
        409,
        "two textually different paths into the same repo must collide on the per-cwd lock"
    );
    let body: serde_json::Value = second_res.json().await.unwrap();
    assert!(body["error"]
        .as_str()
        .unwrap()
        .contains("already in flight"));

    assert_eq!(first.await.unwrap(), 200);

    clear_env();
}

/// Finding 2: a CLIENT disconnect (not a server-side timeout) used to only
/// `SIGKILL` the direct `omega-duo` child (`kill_on_drop(true)` on the
/// `tokio::process::Child` itself), never the nested Codex/Claude turn it
/// spawned into the same process group -- and the per-cwd lock was released
/// regardless, since `CwdLockGuard::drop` does not know or care WHY the
/// future was dropped. So the orphaned agent kept editing files, unbounded,
/// while a second request against the same worktree was free to start a
/// genuinely concurrent run. After the fix, dropping the whole handler
/// future (a real client disconnect, simulated here by aborting the client-
/// side task) must reach the nested grandchild too, and the lock must only
/// ever be usable again once that kill has actually happened.
#[tokio::test]
async fn client_disconnect_kills_the_nested_agent_and_releases_the_cwd_lock() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let fake_home = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");
    let pid_file = bin_dir.path().join("nested.pid");
    let marker_file = bin_dir.path().join("marker.txt");

    std::env::set_var("HOME", fake_home.path());
    let proj = fake_home.path().join("Proj");
    std::fs::create_dir_all(&proj).unwrap();
    install_fake_duo_with_nested_child(bin_dir.path(), &capture_file, &pid_file, &marker_file);
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;
    let client = reqwest::Client::new();
    let dir_str = proj.display().to_string();

    let first = {
        let client = client.clone();
        let base = base.clone();
        let token = token.clone();
        let dir_str = dir_str.clone();
        tokio::spawn(async move {
            let _ = client
                .post(format!("{base}/v1/duo"))
                .bearer_auth(&token)
                .json(&serde_json::json!({ "dir": dir_str, "prompt": "p1", "profile": "build" }))
                .send()
                .await;
        })
    };

    // Wait for the nested grandchild to actually exist.
    let mut nested_pid: Option<u32> = None;
    for _ in 0..150 {
        if let Ok(s) = std::fs::read_to_string(&pid_file) {
            if let Ok(p) = s.trim().parse::<u32>() {
                nested_pid = Some(p);
                break;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    let nested_pid = nested_pid.expect("nested grandchild never started");
    assert!(
        std::path::Path::new(&format!("/proc/{nested_pid}")).exists(),
        "the nested grandchild should be alive right after it announced its pid"
    );

    // Prove it is genuinely alive (not just a stale pid file) by watching
    // the marker file actually grow.
    let before = std::fs::read_to_string(&marker_file).unwrap_or_default();
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    let grew = std::fs::read_to_string(&marker_file).unwrap_or_default();
    assert!(
        grew.len() > before.len(),
        "nested grandchild marker was not growing before the disconnect"
    );

    // CLIENT-side disconnect: abort the task holding the connection, never
    // a server-side timeout.
    first.abort();

    // Bounded wait for the server's disconnect-drop kill to actually reach
    // the nested grandchild (R-LOOP: a small number of short polls).
    let mut died = false;
    for _ in 0..100 {
        if !std::path::Path::new(&format!("/proc/{nested_pid}")).exists() {
            died = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    assert!(
        died,
        "the nested grandchild (pid {nested_pid}) survived the client disconnect"
    );

    // The per-cwd lock must have been released too, ONLY once the kill
    // happened -- not leaked, and not letting a real orphan race a second
    // run. Point OMEGA_DUO_BIN at a fast, ordinary fake bin so this third
    // request does not also have to sit out a long sleep.
    install_fake_duo(
        bin_dir.path(),
        &capture_file,
        "0",
        &fake_bridge_result_json().to_string(),
        0,
    );
    let third_res = client
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "dir": dir_str, "prompt": "p3", "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        third_res.status(),
        200,
        "the per-cwd lock must be usable again after the disconnect-triggered cleanup"
    );

    clear_env();
}

/// Finding 3: `dir_under_home` returns the caller's ORIGINAL,
/// uncanonicalized string, and its ancestor-walk canonicalizes a RELATIVE
/// candidate against THIS PROCESS's own cwd -- so a relative `dir` value
/// (e.g. a bare `--verify`) that happens to name a real directory relative
/// to gatewayd's own cwd AND resolves under `$HOME` used to reach the
/// bridge's argv as `--cwd --verify`, which `omega-duo`'s own `parseArgs`
/// reads as: `cwd` gets no value (falls back to the gateway's own process
/// cwd) and `verify` gets silently set to `true` -- breaking the "we never
/// pass `--verify`" guarantee this endpoint's own doc comment asserts.
#[tokio::test]
async fn dash_prefixed_resolved_dir_is_rejected_before_any_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let fake_home = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    std::env::set_var("HOME", fake_home.path());
    // A directory literally named `--verify`, existing under $HOME.
    let evil_dir = fake_home.path().join("--verify");
    std::fs::create_dir_all(&evil_dir).unwrap();

    // Move this process's own cwd into $HOME so the RELATIVE `dir:
    // "--verify"` below canonicalizes to `$HOME/--verify` -- Finding 3's
    // exact precondition. Restored via `CwdRestore`'s `Drop`.
    let original_cwd = std::env::current_dir().unwrap();
    let _cwd_restore = CwdRestore(original_cwd);
    std::env::set_current_dir(fake_home.path()).unwrap();

    install_fake_duo(
        bin_dir.path(),
        &capture_file,
        "0",
        &fake_bridge_result_json().to_string(),
        0,
    );
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "dir": "--verify", "prompt": "p", "profile": "build" }))
        .send()
        .await
        .unwrap();

    drop(_cwd_restore); // restore cwd before any assertion can early-return

    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(
        body["error"].as_str().unwrap().contains("absolute"),
        "error: {body}"
    );
    assert!(!capture_file.exists(), "no subprocess was ever spawned");

    clear_env();
}

/// Finding 4: the endpoint used to parse the ENTIRE stdout buffer as one
/// JSON value, but the real bridge's own test harness deliberately parses
/// only the LAST non-empty line (the JSON result is documented as the last
/// line, not necessarily the whole stream). Today, any stray line BEFORE
/// the real JSON line turned a successful, quota-burning, potentially
/// file-mutating run into a 502 with no `checkpoint`/`diffstat`/`agent_ok`
/// in the response.
#[tokio::test]
async fn stray_banner_line_before_the_json_is_still_parsed_from_the_last_line() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    install_fake_home(home_dir.path(), "TestProj");
    let body = fake_bridge_result_json();
    install_fake_duo_with_banner_line(
        bin_dir.path(),
        &capture_file,
        "warning: some harmless banner text",
        &body.to_string(),
        0,
    );
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "TestProj", "prompt": "p", "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        200,
        "a stray banner line before the real JSON line must not 502"
    );
    let resp: serde_json::Value = res.json().await.unwrap();
    assert_eq!(resp, body);

    clear_env();
}

// ── auth ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn create_requires_auth() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .json(&serde_json::json!({ "project": "X", "prompt": "p", "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);

    clear_env();
}

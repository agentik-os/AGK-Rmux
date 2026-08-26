//! `POST /v1/sessions/{name}/keys` — sends keystrokes to a live rmux
//! session. Mirrors `dispatch_test.rs`'s fake-subprocess-with-argv-capture
//! idiom: a fake `rmux` script appends its full argv to a capture file, so
//! tests can prove exactly what was invoked and how many times.

use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};

// tokio::sync::Mutex (not std): mirrors stream_test.rs / dispatch_test.rs —
// OMEGA_RMUX_BIN is process-global, so every test in this binary that sets
// it must serialize.
static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Writes an executable fake `rmux` script that APPENDS its full argv (one
/// call per block, `--CALL--`-separated) to `capture_file`, and points
/// `OMEGA_RMUX_BIN` at it. Appending (not overwriting) is what lets a test
/// prove two separate subprocess invocations happened rather than one.
fn install_fake_rmux(dir: &std::path::Path, capture_file: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("rmux");
    let capture = capture_file.display();
    std::fs::write(
        &path,
        format!(
            "#!/usr/bin/env bash\n{{ printf '%s\\n' \"$@\"; printf -- '--CALL--\\n'; }} >> '{capture}'\nexit 0\n"
        ),
    )
    .unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::env::set_var("OMEGA_RMUX_BIN", &path);
}

/// Same shape but the script fails loudly if invoked at all, proving a
/// subprocess spawn never happened (validation rejected before touching it).
fn install_fake_rmux_that_must_not_run(dir: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("rmux");
    std::fs::write(
        &path,
        "#!/usr/bin/env bash\necho 'SHOULD NEVER RUN' >&2\nexit 1\n",
    )
    .unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::env::set_var("OMEGA_RMUX_BIN", &path);
}

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

/// Splits the capture file's contents into per-call argv vectors on the
/// `--CALL--` separator.
fn parse_calls(capture_file: &std::path::Path) -> Vec<Vec<String>> {
    let recorded = std::fs::read_to_string(capture_file).unwrap_or_default();
    recorded
        .split("--CALL--\n")
        .map(str::trim_end)
        .filter(|s| !s.is_empty())
        .map(|block| block.lines().map(str::to_string).collect())
        .collect()
}

#[tokio::test]
async fn happy_path_with_enter_sends_two_separate_calls() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    install_fake_rmux(bin_dir.path(), &capture_file);
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions/oracle-Foo-1/keys"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"data": "ls -la", "enter": true}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["ok"], true);

    let calls = parse_calls(&capture_file);
    assert_eq!(
        calls.len(),
        2,
        "expected exactly two separate subprocess invocations"
    );
    assert_eq!(
        calls[0],
        vec!["send-keys", "-t", "oracle-Foo-1", "-l", "--", "ls -la"]
    );
    assert_eq!(calls[1], vec!["send-keys", "-t", "oracle-Foo-1", "Enter"]);

    std::env::remove_var("OMEGA_RMUX_BIN");
}

#[tokio::test]
async fn enter_false_sends_only_one_call() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    install_fake_rmux(bin_dir.path(), &capture_file);
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    // Omitted entirely: #[serde(default)] means this is the same as false.
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions/oracle-Foo-1/keys"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"data": "echo hi"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let calls = parse_calls(&capture_file);
    assert_eq!(
        calls.len(),
        1,
        "enter omitted/false must send exactly one call"
    );
    assert_eq!(
        calls[0],
        vec!["send-keys", "-t", "oracle-Foo-1", "-l", "--", "echo hi"]
    );

    std::env::remove_var("OMEGA_RMUX_BIN");
}

/// Direct regression test for bug B4 (mirrors
/// `dispatch_test.rs::dash_prefixed_values_land_as_positionals_after_separator`,
/// the sibling bug already fixed for `/v1/dispatch`): a keystroke value that
/// starts with `-` (an entirely normal thing to type in a terminal) must not
/// be misparsed as a flag by rmux's own clap-based argument parser. Proves
/// the recorded argv carries a literal `--` immediately before the value.
#[tokio::test]
async fn dash_prefixed_values_land_as_literal_after_separator() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    install_fake_rmux(bin_dir.path(), &capture_file);
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions/oracle-Foo-1/keys"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"data": "-N", "enter": false}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let calls = parse_calls(&capture_file);
    assert_eq!(calls.len(), 1);
    assert_eq!(
        calls[0],
        vec!["send-keys", "-t", "oracle-Foo-1", "-l", "--", "-N"]
    );

    std::env::remove_var("OMEGA_RMUX_BIN");
}

#[tokio::test]
async fn path_traversal_session_name_rejects_before_any_subprocess_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();

    install_fake_rmux_that_must_not_run(bin_dir.path());
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    for bad_name in ["..%2F..%2Fetc", "foo%2Fbar"] {
        let res = reqwest::Client::new()
            .post(format!("{base}/v1/sessions/{bad_name}/keys"))
            .bearer_auth(&token)
            .json(&serde_json::json!({"data": "x"}))
            .send()
            .await
            .unwrap();
        assert_eq!(
            res.status(),
            400,
            "expected 400 for session name {bad_name}"
        );
    }

    std::env::remove_var("OMEGA_RMUX_BIN");
}

#[tokio::test]
async fn oversized_data_rejects_before_any_subprocess_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();

    install_fake_rmux_that_must_not_run(bin_dir.path());
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let oversized = "a".repeat(8193);
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions/oracle-Foo-1/keys"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"data": oversized}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);

    std::env::remove_var("OMEGA_RMUX_BIN");
}

#[tokio::test]
async fn post_keys_requires_auth() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions/oracle-Foo-1/keys"))
        .json(&serde_json::json!({"data": "x"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}

//! `POST /v1/sessions/{name}/rename` — runs
//! `rmux rename-session -t <name> <new_name>` via `crate::rmux::rename_session`.
//! Mirrors `session_keys_test.rs`'s fake-rmux-with-argv-capture idiom.

use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};

// OMEGA_RMUX_BIN is process-global; serialize every test in this binary that
// sets it (same pattern as session_keys_test.rs).
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

/// Writes an executable fake `rmux` script that APPENDS its full argv
/// (one call per block, `--CALL--`-separated) to `capture_file`, and points
/// `OMEGA_RMUX_BIN` at it — same idiom as `session_keys_test.rs`.
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

/// Same shape but fails loudly if invoked at all, proving a subprocess
/// spawn never happened (validation rejected before touching it).
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
async fn happy_path_rename_records_exact_argv() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    install_fake_rmux(bin_dir.path(), &capture_file);
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions/oracle-Foo-1/rename"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"new_name": "oracle-Foo-renamed"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["name"], "oracle-Foo-renamed");

    let calls = parse_calls(&capture_file);
    assert_eq!(calls.len(), 1);
    assert_eq!(
        calls[0],
        vec!["rename-session", "-t", "oracle-Foo-1", "oracle-Foo-renamed"]
    );

    std::env::remove_var("OMEGA_RMUX_BIN");
}

#[tokio::test]
async fn new_name_with_dot_rejects_before_any_subprocess_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();

    install_fake_rmux_that_must_not_run(bin_dir.path());
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    for bad_new_name in ["foo.bar", "foo:bar", "foo/bar", "foo bar"] {
        let res = reqwest::Client::new()
            .post(format!("{base}/v1/sessions/oracle-Foo-1/rename"))
            .bearer_auth(&token)
            .json(&serde_json::json!({"new_name": bad_new_name}))
            .send()
            .await
            .unwrap();
        assert_eq!(
            res.status(),
            400,
            "expected 400 for new_name {bad_new_name}"
        );
    }

    std::env::remove_var("OMEGA_RMUX_BIN");
}

#[tokio::test]
async fn leading_dash_new_name_rejects_before_any_subprocess_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();

    // Live-verified against the real rmux binary: `rmux rename-session -t
    // <session> -q` actually renames the session to "q", silently trimming
    // the leading "-" daemon-side (a `--` separator does not help). Reject
    // before ever spawning a subprocess.
    install_fake_rmux_that_must_not_run(bin_dir.path());
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    for bad_new_name in ["-q", ".hidden"] {
        let res = reqwest::Client::new()
            .post(format!("{base}/v1/sessions/oracle-Foo-1/rename"))
            .bearer_auth(&token)
            .json(&serde_json::json!({"new_name": bad_new_name}))
            .send()
            .await
            .unwrap();
        assert_eq!(
            res.status(),
            400,
            "expected 400 for new_name {bad_new_name}"
        );
    }

    std::env::remove_var("OMEGA_RMUX_BIN");
}

#[tokio::test]
async fn invalid_path_session_name_rejects_before_any_subprocess_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();

    install_fake_rmux_that_must_not_run(bin_dir.path());
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions/..%2F..%2Fetc/rename"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"new_name": "safe-name"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);

    std::env::remove_var("OMEGA_RMUX_BIN");
}

#[tokio::test]
async fn post_rename_requires_auth() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions/oracle-Foo-1/rename"))
        .json(&serde_json::json!({"new_name": "safe-name"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}

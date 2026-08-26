//! Integration tests for Task C — agent install (`POST
//! /v1/agents/{name}/install` pre-flight + `GET
//! /v1/agents/{name}/install/stream` WebSocket).
//!
//! SAFETY (R-SEC / this task's explicit brief): every test here that could
//! reach a real subprocess spawn sets `OMEGA_BIN` to a FAKE script FIRST.
//! No test in this file ever calls the install-stream WS route with
//! `OMEGA_BIN` unset, which would spawn the box's real `~/.local/bin/omega
//! install <agent>` and run a real `curl|sh`/`npm install` against the
//! internet. The "rejects before spawning" tests go one step further and
//! point `OMEGA_BIN` at a script that errors loudly if it is ever invoked
//! at all (mirrors `session_keys_test.rs::install_fake_rmux_that_must_not_run`).

use futures_util::StreamExt;
use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};
use tokio_tungstenite::connect_async;

// OMEGA_BIN is process-global; serialize every test in this binary that
// touches it (same pattern as dispatch_test.rs's LOCK).
static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn spawn(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

fn ws_url(base: &str, path: &str, token: &str) -> String {
    format!("{}{path}?token={token}", base.replacen("http", "ws", 1))
}

/// Writes an executable fake `omega` script that, when invoked as `omega
/// install <agent>`, prints canned stdout/stderr lines then exits with the
/// given code. Points `OMEGA_BIN` at it.
fn install_fake_omega(dir: &std::path::Path, script_body: &str) {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("omega");
    std::fs::write(&path, format!("#!/usr/bin/env bash\n{script_body}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::env::set_var("OMEGA_BIN", &path);
}

/// A fake `omega` that errors loudly if it is EVER invoked — proves the
/// caller rejected the request before spawning anything (mirrors
/// `session_keys_test.rs::install_fake_rmux_that_must_not_run`).
fn install_fake_omega_that_must_not_run(dir: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("omega");
    std::fs::write(
        &path,
        "#!/usr/bin/env bash\necho 'SHOULD NEVER RUN' >&2\nexit 1\n",
    )
    .unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::env::set_var("OMEGA_BIN", &path);
}

// ── POST /v1/agents/{name}/install ──────────────────────────────────────

#[tokio::test]
async fn install_check_rejects_unknown_agent_name() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    install_fake_omega_that_must_not_run(dir.path());
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/agents/not-a-real-agent/install"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);

    std::env::remove_var("OMEGA_BIN");
}

#[tokio::test]
async fn install_check_rejects_shell() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    install_fake_omega_that_must_not_run(dir.path());
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/agents/shell/install"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    let msg = body["error"].as_str().unwrap();
    assert!(
        msg.to_lowercase().contains("shell"),
        "message should name shell: {msg}"
    );

    std::env::remove_var("OMEGA_BIN");
}

#[tokio::test]
async fn install_check_accepts_a_real_installable_agent() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    // install_check spawns nothing (pure pre-flight), so OMEGA_BIN is left
    // pointed at the must-not-run script too, to prove it.
    install_fake_omega_that_must_not_run(dir.path());
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/agents/codex/install"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["agent"], "codex");
    assert!(body["display_name"].as_str().unwrap().contains("Codex"));
    // is_available() is real-PATH-dependent on this box, so only assert the
    // field is present and well-typed, never its value.
    assert!(body["already_available"].is_boolean());

    std::env::remove_var("OMEGA_BIN");
}

#[tokio::test]
async fn install_check_is_case_insensitive() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    install_fake_omega_that_must_not_run(dir.path());
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/agents/CODEX/install"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["agent"], "codex");

    std::env::remove_var("OMEGA_BIN");
}

#[tokio::test]
async fn install_check_requires_auth() {
    let dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/agents/codex/install"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}

// ── GET /v1/agents/{name}/install/stream ────────────────────────────────

#[tokio::test]
async fn install_stream_rejects_unknown_agent_before_any_spawn() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    install_fake_omega_that_must_not_run(dir.path());
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    // A REAL WS handshake attempt (valid auth token, real Upgrade headers
    // via connect_async) proves this hits our handler's own agent-name
    // check, not just axum's upgrade-header extraction: the server answers
    // with a plain HTTP 400 instead of 101 Switching Protocols, so
    // connect_async errors (same pattern as events_test.rs's auth-rejection
    // test) rather than completing a handshake.
    let url = ws_url(&base, "/v1/agents/not-a-real-agent/install/stream", &token);
    let err = connect_async(url).await.unwrap_err();
    assert!(err.to_string().contains("400"), "unexpected error: {err}");

    std::env::remove_var("OMEGA_BIN");
}

#[tokio::test]
/// `shell` is the one agent left with no `install_command` — kimi gained a
/// real installer, so it is no longer the exemplar of "not installable".
async fn install_stream_rejects_a_non_installable_agent_before_any_spawn() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    install_fake_omega_that_must_not_run(dir.path());
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let url = ws_url(&base, "/v1/agents/shell/install/stream", &token);
    let err = connect_async(url).await.unwrap_err();
    assert!(err.to_string().contains("400"), "unexpected error: {err}");

    std::env::remove_var("OMEGA_BIN");
}

#[tokio::test]
async fn install_stream_happy_path_streams_lines_then_success_exit() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    install_fake_omega(
        dir.path(),
        r#"
if [ "$1" = "install" ]; then
    echo "line one"
    echo "line two"
    echo "warning: something" >&2
    echo "line three"
    exit 0
fi
exit 1
"#,
    );
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let url = ws_url(&base, "/v1/agents/codex/install/stream", &token);
    let (mut ws, _) = connect_async(url).await.unwrap();

    let mut lines: Vec<serde_json::Value> = Vec::new();
    let exit = loop {
        let msg = ws.next().await.unwrap().unwrap();
        let v: serde_json::Value = serde_json::from_str(&msg.into_text().unwrap()).unwrap();
        if v["type"] == "exit" {
            break v;
        }
        lines.push(v);
    };

    assert_eq!(
        lines.len(),
        4,
        "3 stdout + 1 stderr line expected, got {lines:?}"
    );
    let texts: Vec<&str> = lines.iter().map(|l| l["text"].as_str().unwrap()).collect();
    assert!(texts.contains(&"line one"));
    assert!(texts.contains(&"line two"));
    assert!(texts.contains(&"line three"));
    assert!(texts.contains(&"warning: something"));

    let stdout_count = lines.iter().filter(|l| l["stream"] == "stdout").count();
    let stderr_count = lines.iter().filter(|l| l["stream"] == "stderr").count();
    assert_eq!(stdout_count, 3);
    assert_eq!(stderr_count, 1);

    assert_eq!(exit["success"], true);
    assert_eq!(exit["code"], 0);

    std::env::remove_var("OMEGA_BIN");
}

#[tokio::test]
async fn install_stream_nonzero_exit_reports_failure() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    install_fake_omega(
        dir.path(),
        r#"
echo "trying..."
echo "boom" >&2
exit 7
"#,
    );
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let url = ws_url(&base, "/v1/agents/codex/install/stream", &token);
    let (mut ws, _) = connect_async(url).await.unwrap();

    let exit = loop {
        let msg = ws.next().await.unwrap().unwrap();
        let v: serde_json::Value = serde_json::from_str(&msg.into_text().unwrap()).unwrap();
        if v["type"] == "exit" {
            break v;
        }
    };

    assert_eq!(exit["success"], false);
    assert_eq!(exit["code"], 7);

    std::env::remove_var("OMEGA_BIN");
}

#[tokio::test]
async fn install_stream_passes_agent_name_as_argv() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    let capture = dir.path().join("argv.txt");
    install_fake_omega(
        dir.path(),
        &format!("printf '%s\\n' \"$@\" > '{}'\nexit 0", capture.display()),
    );
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let url = ws_url(&base, "/v1/agents/codex/install/stream", &token);
    let (mut ws, _) = connect_async(url).await.unwrap();

    // Drain until the exit frame so the fake script has fully run.
    loop {
        let msg = ws.next().await.unwrap().unwrap();
        let v: serde_json::Value = serde_json::from_str(&msg.into_text().unwrap()).unwrap();
        if v["type"] == "exit" {
            break;
        }
    }

    let argv = std::fs::read_to_string(&capture).unwrap();
    assert_eq!(argv, "install\ncodex\n");

    std::env::remove_var("OMEGA_BIN");
}

//! Integration tests for `GET /v1/orchestrate/stream` —
//! `crates/omega-gateway/src/routes_orchestrate.rs`.
//!
//! SAFETY (R-SEC / this task's explicit brief): every test here points
//! `OMEGA_BIN` at a FAKE script. `omega orchestrate` dispatches an actual
//! oracle end-to-end (real rmux sessions, real scope claims, a real quality
//! gate) — nothing in this file ever invokes the real binary.

use futures_util::StreamExt;
use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};
use tokio_tungstenite::connect_async;

// OMEGA_HOME and OMEGA_BIN are process-global; serialize every test in this
// binary that touches either (same pattern as audit_test.rs's LOCK).
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

/// Creates a fake `$HOME` containing exactly one discoverable project (a
/// `.git`-marked directory named `name`) and points `OMEGA_HOME` at it —
/// same pattern as `audit_test.rs::install_fake_home`. Returns the
/// project's real on-disk root.
fn install_fake_home(home_dir: &std::path::Path, project_name: &str) -> std::path::PathBuf {
    let root = home_dir.join(project_name);
    std::fs::create_dir_all(root.join(".git")).unwrap();
    std::env::set_var("OMEGA_HOME", home_dir);
    root
}

fn install_fake_omega(dir: &std::path::Path, script_body: &str) {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("omega");
    std::fs::write(&path, format!("#!/usr/bin/env bash\n{script_body}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::env::set_var("OMEGA_BIN", &path);
}

/// A fake `omega` that errors loudly if it is EVER invoked — proves the
/// caller rejected the request before spawning anything.
fn install_fake_omega_that_must_not_run(dir: &std::path::Path) {
    install_fake_omega(dir, "echo 'SHOULD NEVER RUN' >&2\nexit 1");
}

fn clear_env() {
    std::env::remove_var("OMEGA_HOME");
    std::env::remove_var("OMEGA_BIN");
}

#[tokio::test]
async fn stream_rejects_unknown_project_before_any_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    install_fake_home(home_dir.path(), "TestProj");
    install_fake_omega_that_must_not_run(bin_dir.path());
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    // A REAL WS handshake attempt proves this hits our handler's own
    // validation (a plain HTTP 400), never an upgrade followed by an
    // in-loop error.
    let url =
        ws_url(&base, "/v1/orchestrate/stream", &token) + "&project=nope-not-real&mission=do+it";
    let err = connect_async(url).await.unwrap_err();
    assert!(err.to_string().contains("400"), "unexpected error: {err}");

    clear_env();
}

#[tokio::test]
async fn stream_rejects_unknown_agent_before_any_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    install_fake_home(home_dir.path(), "TestProj");
    install_fake_omega_that_must_not_run(bin_dir.path());
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let url = ws_url(&base, "/v1/orchestrate/stream", &token)
        + "&project=TestProj&mission=do+it&agent=not-a-real-agent";
    let err = connect_async(url).await.unwrap_err();
    assert!(err.to_string().contains("400"), "unexpected error: {err}");

    clear_env();
}

#[tokio::test]
async fn stream_rejects_empty_mission_before_any_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    install_fake_home(home_dir.path(), "TestProj");
    install_fake_omega_that_must_not_run(bin_dir.path());
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let url = ws_url(&base, "/v1/orchestrate/stream", &token) + "&project=TestProj&mission=";
    let err = connect_async(url).await.unwrap_err();
    assert!(err.to_string().contains("400"), "unexpected error: {err}");

    clear_env();
}

#[tokio::test]
async fn stream_happy_path_streams_lines_then_success_exit_never_forwards_agent() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let project_root = install_fake_home(home_dir.path(), "TestProj");
    let capture = bin_dir.path().join("argv.txt");
    install_fake_omega(
        bin_dir.path(),
        &format!(
            r#"
printf '%s\n' "$@" > '{capture}'
if [ "$1" = "orchestrate" ]; then
    echo "Mission dispatched"
    echo "warming up" >&2
    echo "Mission completed successfully"
    exit 0
fi
exit 1
"#,
            capture = capture.display()
        ),
    );
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    // agent=known-agent-name is supplied but must never reach argv (see
    // routes_orchestrate.rs's doc comment: omega orchestrate has no --agent
    // flag at all).
    let agents = omega_core::agents::Agent::all();
    let agent_name = agents[0].name();
    let url = ws_url(&base, "/v1/orchestrate/stream", &token)
        + &format!("&project=TestProj&mission=do+the+thing&agent={agent_name}");
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
        3,
        "2 stdout + 1 stderr line expected, got {lines:?}"
    );
    let texts: Vec<&str> = lines.iter().map(|l| l["text"].as_str().unwrap()).collect();
    assert!(texts.iter().any(|t| t.contains("Mission dispatched")));
    assert!(texts
        .iter()
        .any(|t| t.contains("Mission completed successfully")));
    assert_eq!(exit["success"], true);
    assert_eq!(exit["code"], 0);

    // argv proves: --dir <resolved project root>, then --, then the two
    // positionals — and NO --agent flag anywhere (there is nothing to
    // forward it to).
    let recorded = std::fs::read_to_string(&capture).unwrap();
    let argv: Vec<&str> = recorded.lines().collect();
    assert_eq!(argv[0], "orchestrate");
    assert_eq!(argv[1], "--dir");
    assert_eq!(std::path::Path::new(argv[2]), project_root.as_path());
    assert_eq!(argv[3], "--");
    assert_eq!(argv[4], "TestProj");
    assert_eq!(argv[5], "do the thing");
    assert!(
        !argv.contains(&"--agent"),
        "agent must never be forwarded: {argv:?}"
    );
    assert!(
        !argv.contains(&"--timeout"),
        "timeout must never be forwarded: {argv:?}"
    );

    clear_env();
}

#[tokio::test]
async fn stream_nonzero_exit_reports_failure() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    install_fake_home(home_dir.path(), "TestProj");
    install_fake_omega(bin_dir.path(), "echo 'trying...'; echo 'boom' >&2; exit 1");
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let url = ws_url(&base, "/v1/orchestrate/stream", &token) + "&project=TestProj&mission=do+it";
    let (mut ws, _) = connect_async(url).await.unwrap();

    let exit = loop {
        let msg = ws.next().await.unwrap().unwrap();
        let v: serde_json::Value = serde_json::from_str(&msg.into_text().unwrap()).unwrap();
        if v["type"] == "exit" {
            break v;
        }
    };
    assert_eq!(exit["success"], false);
    assert_eq!(exit["code"], 1);

    clear_env();
}

/// Disconnect-mid-stream: proves the spawned `omega orchestrate` child's
/// PROCESS GROUP is actually killed when the client disconnects while the
/// fake command has gone SILENT — mirrors `audit_test.rs::
/// disconnect_mid_stream_kills_the_process_group_even_when_child_is_silent`.
#[tokio::test]
async fn disconnect_mid_stream_kills_the_process_group_even_when_child_is_silent() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    install_fake_home(home_dir.path(), "TestProj");
    let marker = bin_dir.path().join("silent-orphan-still-ran.marker");
    install_fake_omega(
        bin_dir.path(),
        &format!(
            r#"
if [ "$1" = "orchestrate" ]; then
    echo "starting"
    bash -c '
        sleep 5
        touch "{marker}"
    '
fi
"#,
            marker = marker.display()
        ),
    );
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let url = ws_url(&base, "/v1/orchestrate/stream", &token) + "&project=TestProj&mission=do+it";
    let (mut ws, _) = connect_async(url).await.unwrap();

    let first = ws.next().await.unwrap().unwrap();
    let v: serde_json::Value = serde_json::from_str(&first.into_text().unwrap()).unwrap();
    assert_eq!(v["text"], "starting");
    ws.close(None).await.unwrap();
    drop(ws);

    assert!(
        !marker.exists(),
        "marker must not exist yet — the nested child hasn't reached it"
    );

    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(8);
    while tokio::time::Instant::now() < deadline {
        assert!(
            !marker.exists(),
            "the SILENT nested child kept running after a clean disconnect"
        );
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
    assert!(
        !marker.exists(),
        "the silent nested child survived the disconnect"
    );

    clear_env();
}

#[tokio::test]
async fn concurrency_cap_returns_429_when_orchestrate_permits_exhausted() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    install_fake_home(home_dir.path(), "TestProj");
    // Long-running fake so the permit stays held while the extra connection
    // attempt fires.
    install_fake_omega(bin_dir.path(), "echo starting; sleep 5; exit 0");
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    // Must match server.rs's MAX_CONCURRENT_ORCHESTRATIONS.
    const MAX_CONCURRENT_ORCHESTRATIONS: usize = 2;

    let mut held = Vec::new();
    for _ in 0..MAX_CONCURRENT_ORCHESTRATIONS {
        let url =
            ws_url(&base, "/v1/orchestrate/stream", &token) + "&project=TestProj&mission=do+it";
        let (mut ws, _) = connect_async(url).await.unwrap();
        let first = ws.next().await.unwrap().unwrap();
        let v: serde_json::Value = serde_json::from_str(&first.into_text().unwrap()).unwrap();
        assert_eq!(v["text"], "starting");
        held.push(ws);
    }

    let url =
        ws_url(&base, "/v1/orchestrate/stream", &token) + "&project=TestProj&mission=one+too+many";
    let err = connect_async(url).await.unwrap_err();
    assert!(err.to_string().contains("429"), "unexpected error: {err}");

    clear_env();
}

#[tokio::test]
async fn stream_requires_auth() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;
    let url = format!(
        "{}/v1/orchestrate/stream?project=x&mission=y",
        base.replacen("http", "ws", 1)
    );
    let err = connect_async(url).await.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("401") || msg.contains("Unauthorized"),
        "unexpected error: {msg}"
    );
}

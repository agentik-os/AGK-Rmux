use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};

// Both tests mutate the process-global OMEGA_RMUX_BIN env var, so they must
// never run concurrently with each other. Acquire this lock at the start of
// each test to serialize them regardless of the test harness's thread count.
// tokio::sync::Mutex (not std): the guard is held across .await points below,
// and clippy::await_holding_lock correctly flags a std guard doing that.
static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn spawn(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

/// Writes an executable fake rmux script and points OMEGA_RMUX_BIN at it.
fn install_fake_rmux(dir: &std::path::Path, script_body: &str) {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("rmux");
    std::fs::write(&path, format!("#!/usr/bin/env bash\n{script_body}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::env::set_var("OMEGA_RMUX_BIN", &path);
}

#[tokio::test]
async fn lists_sessions_from_rmux() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    install_fake_rmux(
        dir.path(),
        r#"
if [ "$1" = "ls" ]; then printf 'oracle-Verba-1\nworker-a\n'; exit 0; fi
exit 1"#,
    );
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;
    let body: serde_json::Value = reqwest::Client::new()
        .get(format!("{base}/v1/sessions"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let names: Vec<&str> = body["sessions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["oracle-Verba-1", "worker-a"]);
}

#[tokio::test]
async fn rmux_failure_yields_empty_list_with_error_not_500() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    install_fake_rmux(dir.path(), "echo 'no server running' >&2; exit 1");
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;
    let res = reqwest::Client::new()
        .get(format!("{base}/v1/sessions"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["sessions"].as_array().unwrap().len(), 0);
    assert!(body["error"]
        .as_str()
        .unwrap()
        .contains("no server running"));
}

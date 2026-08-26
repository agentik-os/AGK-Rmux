use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};

// Both OMEGA_RMUX_BIN and OMEGA_STATE_DIR are process-global; serialize
// every test in this binary that touches either (same pattern as
// sessions_test.rs / missions_test.rs).
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
async fn composes_live_session_with_progress_ledger() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_STATE_DIR", state_dir.path());
    install_fake_rmux(
        bin_dir.path(),
        r#"
if [ "$1" = "ls" ]; then printf 'oracle-dentistrygpt\n'; exit 0; fi
exit 1"#,
    );

    // This one has a live rmux session: expect live: true.
    std::fs::write(
        state_dir.path().join("oracle-dentistrygpt.progress.json"),
        r#"{
            "oracle":"oracle-dentistrygpt","project":"dentistrygpt",
            "mission":"Audit code reset vs addition",
            "done":6,"total":6,"ts":"2026-08-08T08:48:20Z",
            "tasks":[{"s":"done","t":"Audit code reset vs addition","updated_at":"a"}],
            "bot":1,"chat":2,"thread":null,"msgId":3
        }"#,
    )
    .unwrap();
    // No matching live session for this one: expect live: false.
    std::fs::write(
        state_dir.path().join("oracle-verba.progress.json"),
        r#"{
            "oracle":"oracle-verba","project":"verba","mission":"Ship the thing",
            "done":1,"total":3,"ts":"2026-08-09T00:00:00Z",
            "tasks":[{"s":"todo","t":"Thing","updated_at":"c"}],
            "bot":1,"chat":2,"thread":null,"msgId":3
        }"#,
    )
    .unwrap();

    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/oracles"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    let oracles = body["oracles"].as_array().unwrap();
    assert_eq!(oracles.len(), 2);

    // missions::list() sorts updated_at desc: verba (08-09) before dentistrygpt (08-08).
    let verba = &oracles[0];
    assert_eq!(verba["key"], "oracle-verba");
    assert_eq!(verba["session"], "oracle-verba");
    assert_eq!(verba["live"], false);
    assert_eq!(verba["mission"]["done"], 1);
    assert_eq!(verba["mission"]["total"], 3);

    let dentistry = &oracles[1];
    assert_eq!(dentistry["key"], "oracle-dentistrygpt");
    assert_eq!(dentistry["session"], "oracle-dentistrygpt");
    assert_eq!(dentistry["live"], true);
    assert_eq!(dentistry["mission"]["done"], 6);
    assert_eq!(dentistry["mission"]["total"], 6);

    std::env::remove_var("OMEGA_STATE_DIR");
    std::env::remove_var("OMEGA_RMUX_BIN");
}

#[tokio::test]
async fn get_oracles_requires_auth() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/oracles"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}

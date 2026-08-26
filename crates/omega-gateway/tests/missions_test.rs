use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};

// OMEGA_STATE_DIR is process-global; serialize every test in this binary
// that mutates it (same pattern as chat_routes_test.rs's LOCK).
static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn spawn(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

#[tokio::test]
async fn get_missions_returns_parsed_ledgers_and_excludes_workers() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_STATE_DIR", state_dir.path());

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
    // Worker ledger: must not appear in the mission mirror.
    std::fs::write(
        state_dir.path().join("oracle-dentistrygpt-worker-1.progress.json"),
        r#"{
            "oracle":"oracle-dentistrygpt-worker-1","project":"dentistrygpt","mission":"worker task",
            "done":1,"total":1,"ts":"2026-08-08T09:00:00Z","tasks":[],
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
        .get(format!("{base}/v1/missions"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    let missions = body["missions"].as_array().unwrap();
    assert_eq!(
        missions.len(),
        1,
        "worker ledger must be excluded from the mirror"
    );
    assert_eq!(missions[0]["key"], "oracle-dentistrygpt");
    assert_eq!(missions[0]["project"], "dentistrygpt");
    assert_eq!(missions[0]["title"], "Audit code reset vs addition");
    assert_eq!(missions[0]["done"], 6);
    assert_eq!(missions[0]["total"], 6);
    assert_eq!(missions[0]["tasks"][0]["status"], "done");
    assert_eq!(
        missions[0]["tasks"][0]["title"],
        "Audit code reset vs addition"
    );

    std::env::remove_var("OMEGA_STATE_DIR");
}

#[tokio::test]
async fn get_missions_requires_auth() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/missions"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}

use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};

async fn spawn(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

#[tokio::test]
async fn get_rules_returns_laws_and_operational_rules() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/rules"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();

    let laws = body["laws"].as_array().unwrap();
    assert_eq!(laws.len(), 7, "expected exactly the 7 laws L0..L6");
    assert!(laws.iter().any(|l| l["id"] == "L0"), "L0 must be present");

    let rules = body["rules"].as_array().unwrap();
    assert!(
        rules.len() >= 40,
        "expected at least 40 operational rules, got {}",
        rules.len()
    );
    assert!(
        rules.iter().any(|r| r["id"] == "R-CLI"),
        "R-CLI must be present"
    );
}

#[tokio::test]
async fn get_rules_requires_auth() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/rules"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}

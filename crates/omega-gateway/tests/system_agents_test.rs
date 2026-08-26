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
async fn system_agents_are_the_aisb_registry_not_dispatch_engines() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let base = spawn(build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    )))
    .await;
    let response = reqwest::Client::new()
        .get(format!("{base}/v1/system-agents"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    let agents = body["agents"].as_array().unwrap();
    assert_eq!(agents.len(), 15);
    assert!(agents.iter().any(|agent| agent["name"] == "ORACLE"));
    assert!(!agents.iter().any(|agent| agent["name"] == "claude"));
}

#[tokio::test]
async fn system_agents_require_auth() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let base = spawn(build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    )))
    .await;
    assert_eq!(
        reqwest::Client::new()
            .get(format!("{base}/v1/system-agents"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
}

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
async fn get_agents_returns_the_canonical_agent_roster() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/agents"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();

    let agents = body["agents"].as_array().unwrap();
    assert_eq!(
        agents.len(),
        omega_core::agents::Agent::all().len(),
        "gateway roster must mirror Agent::all()"
    );

    let claude = agents
        .iter()
        .find(|a| a["name"] == "claude")
        .expect("claude must be present");
    assert!(claude["display_name"].is_string());
    assert!(
        claude["available"].is_boolean(),
        "available must be a boolean (value is PATH-dependent, not asserted)"
    );
    assert!(agents.iter().any(|agent| agent["name"] == "antigravity"));
    assert!(agents.iter().any(|agent| agent["name"] == "openrouter"));
}

#[tokio::test]
async fn get_agents_requires_auth() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/agents"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}

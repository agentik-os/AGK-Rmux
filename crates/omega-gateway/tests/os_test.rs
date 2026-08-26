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
async fn os_catalog_is_dynamic_and_never_exposes_absolute_paths() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let base = spawn(build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    )))
    .await;
    let response = reqwest::Client::new()
        .get(format!("{base}/v1/os"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    let os = body["os"].as_array().unwrap();
    assert_eq!(os.len(), omega_core::os_products::OsProduct::all().len());
    assert!(os.iter().all(|entry| entry["path"]
        .as_str()
        .is_some_and(|path| path.starts_with("OS/") && !path.starts_with('/'))));
    assert!(os.iter().all(|entry| entry["bot"].is_string()));
}

#[tokio::test]
async fn os_catalog_requires_auth() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let base = spawn(build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    )))
    .await;
    assert_eq!(
        reqwest::Client::new()
            .get(format!("{base}/v1/os"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
}

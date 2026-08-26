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
async fn whoami_requires_valid_token() {
    let dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(dir.path()).issue("iphone");
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;
    let client = reqwest::Client::new();

    // no token → 401
    assert_eq!(
        client
            .get(format!("{base}/v1/whoami"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    // bad token → 401
    assert_eq!(
        client
            .get(format!("{base}/v1/whoami"))
            .bearer_auth("bad")
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    // good token via header → 200 with device name
    let res = client
        .get(format!("{base}/v1/whoami"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["name"], "iphone");
    assert!(!body["device_id"].as_str().unwrap().is_empty());
    // good token via query param → 200
    assert_eq!(
        client
            .get(format!("{base}/v1/whoami?token={token}"))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    // health stays public
    assert_eq!(
        client
            .get(format!("{base}/v1/health"))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
}

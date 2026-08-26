use omega_gateway::auth::PairingCode;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};

async fn spawn(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

#[tokio::test]
async fn pair_with_valid_code_once_then_reject() {
    let dir = tempfile::tempdir().unwrap();
    let pairing = PairingCode::create(dir.path(), 300).unwrap();
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;
    let client = reqwest::Client::new();

    let res = client
        .post(format!("{base}/v1/pair"))
        .json(&serde_json::json!({ "code": pairing.code, "device_name": "iphone" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["token"].as_str().unwrap().len(), 64);

    // second use of the same code: refused
    let res2 = client
        .post(format!("{base}/v1/pair"))
        .json(&serde_json::json!({ "code": pairing.code, "device_name": "mac" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res2.status(), 403);
}

#[tokio::test]
async fn expired_code_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let pairing = PairingCode::create(dir.path(), -1).unwrap(); // already expired
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/pair"))
        .json(&serde_json::json!({ "code": pairing.code, "device_name": "x" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 403);
}

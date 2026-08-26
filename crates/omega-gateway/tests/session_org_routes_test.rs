//! `GET /v1/session-org` + `PUT /v1/session-org/{name}` — the session
//! organization overlay. Unlike `session_keys_test.rs`/`session_close_test.rs`
//! this route never touches rmux or any subprocess (it's a pure file-backed
//! label store), so no fake-bin plumbing is needed here.

use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};

async fn spawn(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

async fn app_and_token(gateway_dir: &std::path::Path) -> (axum::Router, String) {
    let (_, token) = DeviceStore::open(gateway_dir).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.to_path_buf(),
        GatewayConfig::default(),
    ));
    (app, token)
}

#[tokio::test]
async fn put_then_get_roundtrips_over_the_wire() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;
    let client = reqwest::Client::new();

    let res = client
        .put(format!("{base}/v1/session-org/oracle-Foo-1"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"label": "Sprint 3", "folder": "/work/foo", "pinned": true}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let echoed: serde_json::Value = res.json().await.unwrap();
    assert_eq!(echoed["label"], "Sprint 3");
    assert_eq!(echoed["folder"], "/work/foo");
    assert_eq!(echoed["pinned"], true);

    let res = client
        .get(format!("{base}/v1/session-org"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    let entry = &body["entries"]["oracle-Foo-1"];
    assert_eq!(entry["label"], "Sprint 3");
    assert_eq!(entry["folder"], "/work/foo");
    assert_eq!(entry["pinned"], true);
}

#[tokio::test]
async fn get_all_on_a_fresh_gateway_is_an_empty_map() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/session-org"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["entries"], serde_json::json!({}));
}

#[tokio::test]
async fn put_is_a_full_replace_not_a_merge() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;
    let client = reqwest::Client::new();

    client
        .put(format!("{base}/v1/session-org/oracle-Foo-1"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"label": "old", "folder": "/a", "pinned": false}))
        .send()
        .await
        .unwrap();

    // Second PUT omits `folder` and `pinned` entirely.
    let res = client
        .put(format!("{base}/v1/session-org/oracle-Foo-1"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"label": "new"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let echoed: serde_json::Value = res.json().await.unwrap();
    assert_eq!(echoed["label"], "new");
    assert_eq!(
        echoed["folder"],
        serde_json::Value::Null,
        "full replace: folder must not survive from the first PUT"
    );
    assert_eq!(
        echoed["pinned"], false,
        "full replace: pinned must reset to its default, not survive as true"
    );
}

#[tokio::test]
async fn put_a_different_key_preserves_the_first_entry() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;
    let client = reqwest::Client::new();

    client
        .put(format!("{base}/v1/session-org/oracle-Foo-1"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"label": "first", "pinned": true}))
        .send()
        .await
        .unwrap();
    client
        .put(format!("{base}/v1/session-org/oracle-Bar-2"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"label": "second"}))
        .send()
        .await
        .unwrap();

    let res = client
        .get(format!("{base}/v1/session-org"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["entries"].as_object().unwrap().len(), 2);
    assert_eq!(body["entries"]["oracle-Foo-1"]["label"], "first");
    assert_eq!(body["entries"]["oracle-Foo-1"]["pinned"], true);
    assert_eq!(body["entries"]["oracle-Bar-2"]["label"], "second");
}

#[tokio::test]
async fn invalid_session_name_rejects_400_before_any_file_write() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    for bad_name in ["..%2Fetc", "foo%2Fbar"] {
        let res = reqwest::Client::new()
            .put(format!("{base}/v1/session-org/{bad_name}"))
            .bearer_auth(&token)
            .json(&serde_json::json!({"label": "x"}))
            .send()
            .await
            .unwrap();
        assert_eq!(
            res.status(),
            400,
            "expected 400 for session name {bad_name}"
        );
    }

    assert!(
        !gateway_dir.path().join("session_org.json").exists(),
        "an invalid name must never reach the store's write path"
    );
}

#[tokio::test]
async fn oversized_label_rejects_400() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let oversized = "a".repeat(201);
    let res = reqwest::Client::new()
        .put(format!("{base}/v1/session-org/oracle-Foo-1"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"label": oversized}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(!gateway_dir.path().join("session_org.json").exists());
}

#[tokio::test]
async fn oversized_folder_rejects_400() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let oversized = "a".repeat(201);
    let res = reqwest::Client::new()
        .put(format!("{base}/v1/session-org/oracle-Foo-1"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"folder": oversized}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(!gateway_dir.path().join("session_org.json").exists());
}

#[tokio::test]
async fn get_requires_auth() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/session-org"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn put_requires_auth() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .put(format!("{base}/v1/session-org/oracle-Foo-1"))
        .json(&serde_json::json!({"label": "x"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}

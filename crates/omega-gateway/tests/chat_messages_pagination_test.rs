//! HTTP-level tests for `GET /v1/chats/{id}/messages` (chat history
//! pagination at scale). Kept in its own file, isolated from
//! `chat_routes_test.rs`, whose existing `GET /v1/chats/{id}` assertions
//! must never be touched.

use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::protocol::{ChatAgent, ChatMessage};
use omega_gateway::server::{build_router, AppState};

async fn spawn(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

fn seed(state: &AppState, id: &str, count: usize) {
    for i in 0..count {
        state.chats.append_message(
            id,
            &ChatMessage {
                role: "user".to_string(),
                text: format!("m{i}"),
                ts: format!("t{i}"),
            },
        );
    }
}

#[tokio::test]
async fn no_before_returns_the_newest_page() {
    let dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let state = AppState::new(dir.path().to_path_buf(), GatewayConfig::default());
    let meta = state
        .chats
        .create(ChatAgent::Claude, "/tmp".to_string(), None, None);
    seed(&state, &meta.id, 5);
    let base = spawn(build_router(state)).await;

    let res: serde_json::Value = reqwest::Client::new()
        .get(format!("{base}/v1/chats/{}/messages?limit=3", meta.id))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let messages = res["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0]["text"], "m4", "newest first");
    assert_eq!(messages[1]["text"], "m3");
    assert_eq!(messages[2]["text"], "m2");
    assert!(
        res["next_cursor"].is_u64(),
        "2 older messages remain -> a cursor must be present"
    );
}

#[tokio::test]
async fn next_cursor_returns_the_next_older_page() {
    let dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let state = AppState::new(dir.path().to_path_buf(), GatewayConfig::default());
    let meta = state
        .chats
        .create(ChatAgent::Claude, "/tmp".to_string(), None, None);
    seed(&state, &meta.id, 5);
    let base = spawn(build_router(state)).await;
    let client = reqwest::Client::new();

    let page1: serde_json::Value = client
        .get(format!("{base}/v1/chats/{}/messages?limit=3", meta.id))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let cursor = page1["next_cursor"].as_u64().unwrap();

    let page2: serde_json::Value = client
        .get(format!(
            "{base}/v1/chats/{}/messages?before={cursor}&limit=3",
            meta.id
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let messages = page2["messages"].as_array().unwrap();
    assert_eq!(
        messages.len(),
        2,
        "exactly the 2 remaining older messages, no gap, no dupe"
    );
    assert_eq!(messages[0]["text"], "m1");
    assert_eq!(messages[1]["text"], "m0");
    assert!(page2["next_cursor"].is_null());
}

#[tokio::test]
async fn fewer_messages_than_limit_returns_everything_with_null_cursor() {
    let dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let state = AppState::new(dir.path().to_path_buf(), GatewayConfig::default());
    let meta = state
        .chats
        .create(ChatAgent::Claude, "/tmp".to_string(), None, None);
    seed(&state, &meta.id, 1);
    let base = spawn(build_router(state)).await;

    let res: serde_json::Value = reqwest::Client::new()
        .get(format!("{base}/v1/chats/{}/messages?limit=50", meta.id))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let messages = res["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0]["text"], "m0");
    assert!(res["next_cursor"].is_null());
}

#[tokio::test]
async fn missing_limit_defaults_to_a_sane_page_size() {
    let dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let state = AppState::new(dir.path().to_path_buf(), GatewayConfig::default());
    let meta = state
        .chats
        .create(ChatAgent::Claude, "/tmp".to_string(), None, None);
    seed(&state, &meta.id, 3);
    let base = spawn(build_router(state)).await;

    let res: serde_json::Value = reqwest::Client::new()
        .get(format!("{base}/v1/chats/{}/messages", meta.id))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let messages = res["messages"].as_array().unwrap();
    assert_eq!(
        messages.len(),
        3,
        "3 messages all fit under the default limit"
    );
    assert!(res["next_cursor"].is_null());
}

#[tokio::test]
async fn unknown_chat_id_is_404() {
    let dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let base = spawn(build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    )))
    .await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/chats/0123456789abcdef/messages"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn invalid_chat_id_shape_is_404_not_500() {
    let dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let base = spawn(build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    )))
    .await;

    // "%2e%2e" decodes to "..", a path-traversal-shaped single path segment
    // that must never reach ChatStore's filesystem joins.
    let res = reqwest::Client::new()
        .get(format!("{base}/v1/chats/%2e%2e/messages"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn no_auth_token_is_401() {
    let dir = tempfile::tempdir().unwrap();
    let base = spawn(build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    )))
    .await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/chats/0123456789abcdef/messages"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}

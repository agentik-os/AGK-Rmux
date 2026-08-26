//! Integration tests for account CRUD + login routes and per-chat account
//! selection (Task 3). Every provider CLI is a FAKE bin pointed at via
//! OMEGA_CLAUDE_BIN / OMEGA_CODEX_BIN / OMEGA_CHAT_BIN — never the real
//! claude/codex/key.

use futures_util::{SinkExt, StreamExt};
use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

type Ws = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

// Every test in this file that mutates a process-global env var
// (OMEGA_CLAUDE_BIN / OMEGA_CODEX_BIN / OMEGA_CHAT_BIN / PIDFILE /
// CAPTURE_FILE / HOME) must serialize against every other one — same pattern
// as chat_routes_test.rs's LOCK. tokio::sync::Mutex because the guard is
// held across .await points below.
static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// RAII guard restoring (or clearing) the `HOME` env var on drop. I-1 made
/// `POST /v1/chats`'s `cwd` confined under `$HOME` via `dir_under_home`, so
/// any test in this file that creates a chat and cares about a DIFFERENT
/// validation branch (account lookup, kind mismatch, ...) must run under a
/// `HOME` it controls rather than the ambient one, so `cwd` clears the
/// confinement check and the test actually exercises the branch it names.
/// Same shape as `chat_cwd_validation_test.rs::HomeRestore`.
struct HomeRestore(Option<std::ffi::OsString>);
impl HomeRestore {
    fn set(new_home: &std::path::Path) -> Self {
        let prev = std::env::var_os("HOME");
        std::env::set_var("HOME", new_home);
        Self(prev)
    }
}
impl Drop for HomeRestore {
    fn drop(&mut self) {
        match &self.0 {
            Some(v) => std::env::set_var("HOME", v),
            None => std::env::remove_var("HOME"),
        }
    }
}

/// Creates `<dir>/proj` and returns it as a string, for use as a chat's
/// `cwd` once `HomeRestore::set(dir)` is in effect.
fn project_cwd(dir: &std::path::Path) -> String {
    let project_dir = dir.join("proj");
    std::fs::create_dir_all(&project_dir).unwrap();
    project_dir.to_string_lossy().into_owned()
}

async fn spawn(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

fn ws_url(base: &str, path: &str, token: &str) -> String {
    format!("{}{path}?token={token}", base.replacen("http", "ws", 1))
}

async fn recv_json(ws: &mut Ws) -> serde_json::Value {
    let msg = ws.next().await.unwrap().unwrap();
    serde_json::from_str(&msg.into_text().unwrap()).unwrap()
}

/// Writes an executable fake CLI at `dir/name` and returns its path.
fn fake_bin(dir: &std::path::Path, name: &str, script: &str) -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join(name);
    std::fs::write(&path, format!("#!/bin/sh\n{script}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path
}

#[tokio::test]
async fn create_list_and_default_flow() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    let bin = fake_bin(dir.path(), "fake-claude", "echo 'Not logged in'\nexit 1");
    std::env::set_var("OMEGA_CLAUDE_BIN", &bin);

    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;
    let client = reqwest::Client::new();

    // POST /v1/accounts -> 201 Account
    let create_res = client
        .post(format!("{base}/v1/accounts"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "slug": "work-1", "label": "Work", "kind": "claude" }))
        .send()
        .await
        .unwrap();
    assert_eq!(create_res.status(), 201);
    let account: serde_json::Value = create_res.json().await.unwrap();
    assert_eq!(account["slug"], "work-1");
    assert_eq!(
        account["is_default"], true,
        "first account of a kind is its default"
    );

    // A second account of the same kind, so set_default has something to do.
    let create_res2 = client
        .post(format!("{base}/v1/accounts"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "slug": "work-2", "label": "Work 2", "kind": "claude" }))
        .send()
        .await
        .unwrap();
    assert_eq!(create_res2.status(), 201);

    // GET /v1/accounts lists both with a merged live status (fake claude -> logged_out).
    let list_res: serde_json::Value = client
        .get(format!("{base}/v1/accounts"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let accounts = list_res["accounts"].as_array().unwrap();
    assert_eq!(accounts.len(), 2);
    let work1 = accounts.iter().find(|a| a["slug"] == "work-1").unwrap();
    assert_eq!(work1["status"], "logged_out");
    assert_eq!(work1["label"], "Work");

    // POST /v1/accounts/work-2/default -> 200, and it becomes the default.
    let default_res = client
        .post(format!("{base}/v1/accounts/work-2/default"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(default_res.status(), 200);

    let list_res2: serde_json::Value = client
        .get(format!("{base}/v1/accounts"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let accounts2 = list_res2["accounts"].as_array().unwrap();
    let work1_after = accounts2.iter().find(|a| a["slug"] == "work-1").unwrap();
    let work2_after = accounts2.iter().find(|a| a["slug"] == "work-2").unwrap();
    assert_eq!(work1_after["is_default"], false);
    assert_eq!(work2_after["is_default"], true);

    // DELETE /v1/accounts/work-1 -> 204, and it's gone from the list.
    let delete_res = client
        .delete(format!("{base}/v1/accounts/work-1"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(delete_res.status(), 204);

    let list_res3: serde_json::Value = client
        .get(format!("{base}/v1/accounts"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let slugs: Vec<&str> = list_res3["accounts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["slug"].as_str().unwrap())
        .collect();
    assert!(!slugs.contains(&"work-1"));
    assert!(slugs.contains(&"work-2"));

    std::env::remove_var("OMEGA_CLAUDE_BIN");
}

#[tokio::test]
async fn create_rejects_traversal_slug() {
    let dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/accounts"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "slug": "../x", "label": "bad", "kind": "claude" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);

    // Nothing was created on disk for the traversal attempt.
    let list_res: serde_json::Value = reqwest::Client::new()
        .get(format!("{base}/v1/accounts"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(list_res["accounts"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn slug_path_param_routes_reject_invalid_slug_before_fs() {
    let dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;
    let client = reqwest::Client::new();

    // "UP" fails valid_slug (uppercase) without touching the store/fs.
    let delete_res = client
        .delete(format!("{base}/v1/accounts/UP"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(delete_res.status(), 400);

    let default_res = client
        .post(format!("{base}/v1/accounts/UP/default"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(default_res.status(), 400);

    let apikey_res = client
        .post(format!("{base}/v1/accounts/UP/apikey"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "api_key": "sk-whatever" }))
        .send()
        .await
        .unwrap();
    assert_eq!(apikey_res.status(), 400);
}

#[tokio::test]
async fn chat_with_explicit_account_slug_uses_its_slot_dir_as_claude_config_dir() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    let _home = HomeRestore::set(dir.path());
    let cwd = project_cwd(dir.path());
    let env_capture = dir.path().join("captured-env.txt");
    let bin = fake_bin(
        dir.path(),
        "fake-chat",
        &format!(
            r#"printf '%s' "$CLAUDE_CONFIG_DIR" > {}
printf '%s\n' '{{"type":"system","subtype":"init","session_id":"sess-X","cwd":"/tmp","model":"m"}}'
printf '%s\n' '{{"type":"result","is_error":false,"stop_reason":"end_turn","result":"ok"}}'
"#,
            env_capture.display()
        ),
    );
    std::env::set_var("OMEGA_CHAT_BIN", &bin);

    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let state = AppState::new(dir.path().to_path_buf(), GatewayConfig::default());
    let app = build_router(state.clone());
    let base = spawn(app).await;
    let client = reqwest::Client::new();

    // Create account A.
    let create_res = client
        .post(format!("{base}/v1/accounts"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "slug": "acct-a", "label": "A", "kind": "claude" }))
        .send()
        .await
        .unwrap();
    assert_eq!(create_res.status(), 201);
    let expected_slot_dir = state.accounts.slot_dir("acct-a");

    // Create a chat pinned to account A.
    let chat_res = client
        .post(format!("{base}/v1/chats"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "agent": "claude", "cwd": cwd, "account_slug": "acct-a" }))
        .send()
        .await
        .unwrap();
    assert_eq!(chat_res.status(), 201);
    let meta: serde_json::Value = chat_res.json().await.unwrap();
    assert_eq!(meta["account_slug"], "acct-a");
    let chat_id = meta["id"].as_str().unwrap().to_string();

    // Run a turn and check the fake chat bin saw CLAUDE_CONFIG_DIR = account A's slot dir.
    let url = ws_url(&base, &format!("/v1/chats/{chat_id}/stream"), &token);
    let (mut ws, _) = connect_async(url).await.unwrap();
    ws.send(Message::Text(
        serde_json::json!({ "type": "user_message", "text": "hi" }).to_string(),
    ))
    .await
    .unwrap();
    loop {
        let frame = recv_json(&mut ws).await;
        if frame["type"] == "turn_done" {
            break;
        }
    }

    let captured = std::fs::read_to_string(&env_capture).unwrap();
    assert_eq!(captured, expected_slot_dir.to_string_lossy());

    std::env::remove_var("OMEGA_CHAT_BIN");
}

#[tokio::test]
async fn chat_without_account_slug_uses_the_kinds_default_slot_dir() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    let _home = HomeRestore::set(dir.path());
    let cwd = project_cwd(dir.path());
    let env_capture = dir.path().join("captured-env.txt");
    let bin = fake_bin(
        dir.path(),
        "fake-chat",
        &format!(
            r#"printf '%s' "$CLAUDE_CONFIG_DIR" > {}
printf '%s\n' '{{"type":"system","subtype":"init","session_id":"sess-Y","cwd":"/tmp","model":"m"}}'
printf '%s\n' '{{"type":"result","is_error":false,"stop_reason":"end_turn","result":"ok"}}'
"#,
            env_capture.display()
        ),
    );
    std::env::set_var("OMEGA_CHAT_BIN", &bin);

    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let state = AppState::new(dir.path().to_path_buf(), GatewayConfig::default());
    let app = build_router(state.clone());
    let base = spawn(app).await;
    let client = reqwest::Client::new();

    // Create a default Claude account (first of its kind is default by construction).
    let create_res = client
        .post(format!("{base}/v1/accounts"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "slug": "the-default", "label": "Default", "kind": "claude" }))
        .send()
        .await
        .unwrap();
    assert_eq!(create_res.status(), 201);
    assert_eq!(
        create_res.json::<serde_json::Value>().await.unwrap()["is_default"],
        true
    );
    let expected_slot_dir = state.accounts.slot_dir("the-default");

    // Create a chat with NO account_slug.
    let chat_res = client
        .post(format!("{base}/v1/chats"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "agent": "claude", "cwd": cwd }))
        .send()
        .await
        .unwrap();
    assert_eq!(chat_res.status(), 201);
    let meta: serde_json::Value = chat_res.json().await.unwrap();
    assert!(meta["account_slug"].is_null());
    let chat_id = meta["id"].as_str().unwrap().to_string();

    let url = ws_url(&base, &format!("/v1/chats/{chat_id}/stream"), &token);
    let (mut ws, _) = connect_async(url).await.unwrap();
    ws.send(Message::Text(
        serde_json::json!({ "type": "user_message", "text": "hi" }).to_string(),
    ))
    .await
    .unwrap();
    loop {
        let frame = recv_json(&mut ws).await;
        if frame["type"] == "turn_done" {
            break;
        }
    }

    let captured = std::fs::read_to_string(&env_capture).unwrap();
    assert_eq!(captured, expected_slot_dir.to_string_lossy());

    std::env::remove_var("OMEGA_CHAT_BIN");
}

#[tokio::test]
async fn chat_create_rejects_traversal_account_slug() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    let _home = HomeRestore::set(dir.path());
    let cwd = project_cwd(dir.path());
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/chats"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "agent": "claude", "cwd": cwd, "account_slug": "../x" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn chat_create_rejects_nonexistent_account_slug() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    let _home = HomeRestore::set(dir.path());
    let cwd = project_cwd(dir.path());
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let state = AppState::new(dir.path().to_path_buf(), GatewayConfig::default());
    let app = build_router(state.clone());
    let base = spawn(app).await;

    // "does-not-exist" is shape-valid but was never created.
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/chats"))
        .bearer_auth(&token)
        .json(
            &serde_json::json!({ "agent": "claude", "cwd": cwd, "account_slug": "does-not-exist" }),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["error"], "account not found");

    assert!(
        state.chats.list().is_empty(),
        "no chat should be persisted when the account pin is invalid"
    );
}

#[tokio::test]
async fn chat_create_rejects_account_kind_mismatch() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    let _home = HomeRestore::set(dir.path());
    let cwd = project_cwd(dir.path());
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let state = AppState::new(dir.path().to_path_buf(), GatewayConfig::default());
    let app = build_router(state.clone());
    let base = spawn(app).await;
    let client = reqwest::Client::new();

    // A Codex account slot.
    let create_res = client
        .post(format!("{base}/v1/accounts"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "slug": "codex-slot", "label": "Codex", "kind": "codex" }))
        .send()
        .await
        .unwrap();
    assert_eq!(create_res.status(), 201);

    // A Claude chat pinned to a Codex slot must be rejected before the chat
    // is created (no OMEGA_CHAT_BIN installed, so this test would also hang
    // trying to spawn a real `claude` if the guard didn't short-circuit).
    let res = client
        .post(format!("{base}/v1/chats"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "agent": "claude", "cwd": cwd, "account_slug": "codex-slot" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["error"], "account kind does not match agent");

    assert!(
        state.chats.list().is_empty(),
        "no chat should be persisted on a kind mismatch"
    );
}

#[tokio::test]
async fn chat_create_accepts_a_valid_matching_account_slug() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    let _home = HomeRestore::set(dir.path());
    let cwd = project_cwd(dir.path());
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let state = AppState::new(dir.path().to_path_buf(), GatewayConfig::default());
    let app = build_router(state.clone());
    let base = spawn(app).await;
    let client = reqwest::Client::new();

    client
        .post(format!("{base}/v1/accounts"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "slug": "claude-slot", "label": "Claude", "kind": "claude" }))
        .send()
        .await
        .unwrap();

    let res = client
        .post(format!("{base}/v1/chats"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "agent": "claude", "cwd": cwd, "account_slug": "claude-slot" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);
    let meta: serde_json::Value = res.json().await.unwrap();
    assert_eq!(meta["account_slug"], "claude-slot");
    assert_eq!(state.chats.list().len(), 1);
}

#[tokio::test]
async fn apikey_route_pipes_key_to_fake_codex_without_leaking_it_in_response() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    let capture = dir.path().join("captured-stdin.txt");
    let bin = fake_bin(dir.path(), "fake-codex", "cat > \"$CAPTURE_FILE\"");
    std::env::set_var("OMEGA_CODEX_BIN", &bin);
    std::env::set_var("CAPTURE_FILE", &capture);

    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;
    let client = reqwest::Client::new();

    let create_res = client
        .post(format!("{base}/v1/accounts"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "slug": "codex-1", "label": "Codex", "kind": "codex" }))
        .send()
        .await
        .unwrap();
    assert_eq!(create_res.status(), 201);

    let apikey_res = client
        .post(format!("{base}/v1/accounts/codex-1/apikey"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "api_key": "sk-super-secret-123" }))
        .send()
        .await
        .unwrap();
    assert_eq!(apikey_res.status(), 200);
    let body = apikey_res.text().await.unwrap();
    assert!(
        !body.contains("sk-super-secret-123"),
        "the api key must never appear in the response body"
    );

    let captured = std::fs::read_to_string(&capture).unwrap();
    assert_eq!(captured.trim(), "sk-super-secret-123");

    std::env::remove_var("OMEGA_CODEX_BIN");
    std::env::remove_var("CAPTURE_FILE");
}

#[tokio::test]
async fn apikey_route_rejects_a_claude_account() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;
    let client = reqwest::Client::new();

    client
        .post(format!("{base}/v1/accounts"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "slug": "claude-1", "label": "Claude", "kind": "claude" }))
        .send()
        .await
        .unwrap();

    // No OMEGA_CODEX_BIN installed: if the kind guard didn't short-circuit,
    // this would try to spawn a real `codex` and fail loudly / hang.
    let res = client
        .post(format!("{base}/v1/accounts/claude-1/apikey"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "api_key": "sk-whatever" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn login_ws_needs_box_when_no_url_emitted() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    let bin = fake_bin(dir.path(), "fake-claude", "exit 1");
    std::env::set_var("OMEGA_CLAUDE_BIN", &bin);

    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;
    let client = reqwest::Client::new();

    client
        .post(format!("{base}/v1/accounts"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "slug": "needs-box", "label": "NB", "kind": "claude" }))
        .send()
        .await
        .unwrap();

    let url = ws_url(&base, "/v1/accounts/needs-box/login", &token);
    let (mut ws, _) = connect_async(url).await.unwrap();
    let frame = recv_json(&mut ws).await;
    assert_eq!(frame["type"], "login_needs_box");

    std::env::remove_var("OMEGA_CLAUDE_BIN");
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn login_ws_reaps_the_oauth_child_when_the_client_disconnects() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    let pidfile = dir.path().join("login.pid");
    // Differentiate `auth login` (prints its PID + a URL then sleeps, as if
    // waiting on a real browser OAuth flow) from `auth status` (must answer
    // fast, plainly logged-out, so the poll loop never blocks on it).
    let bin = fake_bin(
        dir.path(),
        "fake-claude",
        &format!(
            r#"if [ "$1" = "auth" ] && [ "$2" = "login" ]; then
  echo $$ > {}
  echo 'Visit https://example.com/oauth?x=1 to continue'
  sleep 30
elif [ "$1" = "auth" ] && [ "$2" = "status" ]; then
  echo 'Not logged in'
  exit 1
fi
"#,
            pidfile.display()
        ),
    );
    std::env::set_var("OMEGA_CLAUDE_BIN", &bin);

    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;
    let client = reqwest::Client::new();

    client
        .post(format!("{base}/v1/accounts"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "slug": "reap-me", "label": "Reap", "kind": "claude" }))
        .send()
        .await
        .unwrap();

    let url = ws_url(&base, "/v1/accounts/reap-me/login", &token);
    let (mut ws, _) = connect_async(url).await.unwrap();
    let frame = recv_json(&mut ws).await;
    assert_eq!(frame["type"], "login_url");

    // The pidfile is written by the script BEFORE it emits the URL line we
    // just received, so it is guaranteed present by now.
    let pid: i32 = std::fs::read_to_string(&pidfile)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    assert!(
        std::path::Path::new(&format!("/proc/{pid}")).exists(),
        "the fake login process should be alive right after login_url"
    );

    // Client disconnects mid-login (no login_done ever arrives).
    drop(ws);

    // Bounded wait for the server to notice and reap the child (R-LOOP: a
    // small number of short polls, never an unbounded spin).
    let mut reaped = false;
    for _ in 0..50 {
        if !std::path::Path::new(&format!("/proc/{pid}")).exists() {
            reaped = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    assert!(
        reaped,
        "the OAuth child (pid {pid}) must be reaped after the client disconnects"
    );

    std::env::remove_var("OMEGA_CLAUDE_BIN");
}

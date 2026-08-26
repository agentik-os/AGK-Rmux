//! I-1 regression test: `POST /v1/chats` must confine `cwd` under `$HOME`
//! via `routes_sessions::dir_under_home`, the same guard already used for a
//! session's `dir`. Before the fix, an authed device could point a chat at
//! `/etc`, `/`, or any other filesystem path with zero validation.
//!
//! `dir_under_home` resolves against the REAL `dirs::home_dir()`, not
//! `OMEGA_HOME` (see its own doc comment in `routes_sessions.rs`), so every
//! test in this file that needs a real "home" it controls sets the process
//! -global `HOME` env var and must therefore be serialized against the
//! others (same pattern `tests/master_chat_test.rs` already uses).

use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};

static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn spawn(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

/// RAII guard restoring (or clearing) the `HOME` env var on drop, so a
/// panicking assertion mid-test never leaves a later test in this binary
/// running against the wrong home directory.
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

#[tokio::test]
async fn create_with_cwd_outside_home_is_rejected_with_400() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let fake_home = tempfile::tempdir().unwrap();
    let _home = HomeRestore::set(fake_home.path());

    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    // /etc is a real, existing absolute path outside the fake home — the
    // review's own trigger example.
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/chats"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "agent": "claude", "cwd": "/etc" }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        400,
        "cwd outside home must be rejected before the chat is ever created"
    );
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(
        body["error"].as_str().unwrap().contains("home"),
        "error body should name the home confinement, got: {body}"
    );

    // Confirm no chat was actually persisted for the rejected request.
    let state = AppState::new(gateway_dir.path().to_path_buf(), GatewayConfig::default());
    assert!(
        state.chats.list().is_empty(),
        "a rejected create must not persist any chat"
    );
}

#[tokio::test]
async fn create_with_cwd_containing_dotdot_is_rejected_with_400() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let fake_home = tempfile::tempdir().unwrap();
    let _home = HomeRestore::set(fake_home.path());

    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let escaping = format!("{}/../../../etc", fake_home.path().display());
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/chats"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "agent": "claude", "cwd": escaping }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400, "a `..`-escaping cwd must be rejected");
}

#[tokio::test]
async fn create_with_cwd_under_home_succeeds_and_stores_the_validated_path() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let fake_home = tempfile::tempdir().unwrap();
    let proj = fake_home.path().join("proj");
    std::fs::create_dir_all(&proj).unwrap();
    let _home = HomeRestore::set(fake_home.path());

    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let cwd_str = proj.display().to_string();
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/chats"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "agent": "claude", "cwd": cwd_str }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        201,
        "a real subdirectory of $HOME must be accepted"
    );
    let meta: serde_json::Value = res.json().await.unwrap();
    assert_eq!(
        meta["cwd"], cwd_str,
        "the stored cwd must be dir_under_home's validated (original) path"
    );
}

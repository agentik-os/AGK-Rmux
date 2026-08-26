//! Integration tests for the Telegram bridge control endpoints —
//! `crates/omega-gateway/src/routes_telegram.rs` (wave7 task D):
//! `GET /v1/telegram/status`, `POST /v1/telegram/enable`,
//! `POST /v1/telegram/disable`.
//!
//! `omega_core::monitor::OmegaTelegramConfig::path()` hardcodes
//! `dirs::home_dir()` (NOT `$OMEGA_DIR`/`$OMEGA_HOME`), so every test here
//! overrides the `$HOME` env var itself (same pattern `box_test.rs` uses
//! for `UsageSnapshot`) -- NEVER the operator's real `~/.omega/telegram.toml`.

use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};

// $HOME is process-global; serialize every test in this binary that
// touches it (same pattern as box_test.rs's LOCK).
static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

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

fn clear_env() {
    std::env::remove_var("HOME");
}

fn write_fake_telegram_toml(home: &std::path::Path, enabled: bool) {
    let dir = home.join(".omega");
    std::fs::create_dir_all(&dir).unwrap();
    let toml = format!(
        "bot_token = \"123456:FAKE-super-secret-token\"\nchat_id = 555\nallow_user_ids = [555]\nrelay_session = \"aisb-master\"\nlabel = \"test-profile\"\nenabled = {enabled}\n"
    );
    std::fs::write(dir.join("telegram.toml"), toml).unwrap();
}

#[tokio::test]
async fn status_reports_unconfigured_when_no_toml_exists() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    std::env::set_var("HOME", home.path());
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/telegram/status"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        200,
        "unconfigured is a normal response, never an error"
    );
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["configured"], false);
    assert!(body["enabled"].is_null());

    clear_env();
}

#[tokio::test]
async fn status_redacts_the_bot_token() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    std::env::set_var("HOME", home.path());
    write_fake_telegram_toml(home.path(), true);
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/telegram/status"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body_text = res.text().await.unwrap();
    assert!(
        !body_text.contains("FAKE-super-secret-token"),
        "the bot token must never round-trip"
    );
    let body: serde_json::Value = serde_json::from_str(&body_text).unwrap();
    assert_eq!(body["configured"], true);
    assert_eq!(body["enabled"], true);
    assert_eq!(body["bot_token_set"], true);
    assert_eq!(body["chat_id"], 555);
    assert_eq!(body["relay_session"], "aisb-master");
    assert_eq!(body["label"], "test-profile");

    clear_env();
}

#[tokio::test]
async fn enable_flips_a_disabled_config_and_persists_it() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    std::env::set_var("HOME", home.path());
    write_fake_telegram_toml(home.path(), false);
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/telegram/enable"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["enabled"], true);

    let raw = std::fs::read_to_string(home.path().join(".omega/telegram.toml")).unwrap();
    assert!(raw.contains("enabled = true"));
    // Every other field survives the flip untouched.
    assert!(raw.contains("FAKE-super-secret-token"));
    assert!(raw.contains("chat_id = 555"));

    clear_env();
}

#[tokio::test]
async fn disable_flips_an_enabled_config_and_persists_it() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    std::env::set_var("HOME", home.path());
    write_fake_telegram_toml(home.path(), true);
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/telegram/disable"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["enabled"], false);

    let raw = std::fs::read_to_string(home.path().join(".omega/telegram.toml")).unwrap();
    assert!(raw.contains("enabled = false"));

    clear_env();
}

#[tokio::test]
async fn enable_on_an_unconfigured_bridge_is_404_not_a_fabricated_config() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    std::env::set_var("HOME", home.path());
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/telegram/enable"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
    assert!(
        !home.path().join(".omega/telegram.toml").exists(),
        "never fabricates a config on enable"
    );

    clear_env();
}

#[tokio::test]
async fn disable_on_an_unconfigured_bridge_is_404() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    std::env::set_var("HOME", home.path());
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/telegram/disable"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);

    clear_env();
}

#[tokio::test]
async fn status_requires_auth() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    std::env::set_var("HOME", home.path());
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/telegram/status"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);

    clear_env();
}

#[tokio::test]
async fn enable_and_disable_require_auth() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    std::env::set_var("HOME", home.path());
    write_fake_telegram_toml(home.path(), false);
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res_en = reqwest::Client::new()
        .post(format!("{base}/v1/telegram/enable"))
        .send()
        .await
        .unwrap();
    assert_eq!(res_en.status(), 401);
    let res_dis = reqwest::Client::new()
        .post(format!("{base}/v1/telegram/disable"))
        .send()
        .await
        .unwrap();
    assert_eq!(res_dis.status(), 401);

    // Neither unauthenticated call touched the config.
    let raw = std::fs::read_to_string(home.path().join(".omega/telegram.toml")).unwrap();
    assert!(raw.contains("enabled = false"));

    clear_env();
}

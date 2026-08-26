//! Integration tests for `GET`/`PUT /v1/config` —
//! `crates/omega-gateway/src/routes_config.rs` (wave7 task C).
//!
//! Pure in-process reads/writes of `omega_core::providers::ProvidersConfig`
//! — no subprocess involved. `ProvidersConfig::path()` honors `$OMEGA_DIR`,
//! so every test overrides that env var to a scratch dir, never the
//! operator's real `providers.toml`.

use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};

// $OMEGA_DIR is process-global; serialize every test in this binary that
// touches it (same pattern as oracle_ops_test.rs's LOCK).
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
    std::env::remove_var("OMEGA_DIR");
}

#[tokio::test]
async fn get_config_on_a_fresh_box_returns_defaults_with_no_keys_set() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let omega_dir = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_DIR", omega_dir.path());
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/config"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["claude"]["api_key_set"], false);
    assert_eq!(body["codex"]["api_key_set"], false);
    assert_eq!(body["antigravity"]["dangerously_skip_permissions"], true);
    assert_eq!(body["kimi"]["api_key_set"], false);
    // Never a raw `api_key` field on the wire at all.
    assert!(body["claude"].get("api_key").is_none());
    assert!(body["kimi"].get("api_key").is_none());

    clear_env();
}

#[tokio::test]
async fn put_config_keeps_kimi_antigravity_and_hermes_in_cli_parity() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let omega_dir = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_DIR", omega_dir.path());
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;
    let client = reqwest::Client::new();

    for (key, value) in [
        ("kimi.provider_type", "anthropic"),
        ("kimi.model", "k3"),
        ("antigravity.effort", "high"),
        ("hermes.provider", "openrouter"),
    ] {
        let response = client
            .put(format!("{base}/v1/config"))
            .bearer_auth(&token)
            .json(&serde_json::json!({ "key": key, "value": value }))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 200, "{key}");
    }

    let body: serde_json::Value = client
        .get(format!("{base}/v1/config"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(body["kimi"]["provider_type"], "anthropic");
    assert_eq!(body["kimi"]["model"], "k3");
    assert_eq!(body["antigravity"]["effort"], "high");
    assert_eq!(body["hermes"]["provider"], "openrouter");
    clear_env();
}

#[tokio::test]
async fn put_config_sets_a_known_key_and_never_echoes_the_secret_back() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let omega_dir = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_DIR", omega_dir.path());
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .put(format!("{base}/v1/config"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "key": "claude.api_key", "value": "sk-super-secret-value" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body_text = res.text().await.unwrap();
    assert!(
        !body_text.contains("sk-super-secret-value"),
        "the secret must never round-trip on the wire"
    );
    let body: serde_json::Value = serde_json::from_str(&body_text).unwrap();
    assert_eq!(body["claude"]["api_key_set"], true);

    // Persisted -- a follow-up GET reflects it too.
    let res2 = reqwest::Client::new()
        .get(format!("{base}/v1/config"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    let body2: serde_json::Value = res2.json().await.unwrap();
    assert_eq!(body2["claude"]["api_key_set"], true);

    // The on-disk file genuinely holds the value (proves it was actually
    // written, not just faked in the response).
    let raw = std::fs::read_to_string(omega_dir.path().join("providers.toml")).unwrap();
    assert!(raw.contains("sk-super-secret-value"));

    clear_env();
}

#[tokio::test]
async fn put_config_sets_a_non_secret_key() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let omega_dir = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_DIR", omega_dir.path());
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .put(format!("{base}/v1/config"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "key": "claude.model", "value": "opus" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["claude"]["model"], "opus");

    clear_env();
}

#[tokio::test]
async fn put_config_rejects_unknown_key() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let omega_dir = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_DIR", omega_dir.path());
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .put(format!("{base}/v1/config"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "key": "claude.totally_made_up", "value": "x" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);

    // Never persisted anything for an unknown key.
    assert!(!omega_dir.path().join("providers.toml").exists());

    clear_env();
}

#[tokio::test]
async fn put_config_rejects_malformed_boolean() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let omega_dir = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_DIR", omega_dir.path());
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .put(format!("{base}/v1/config"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "key": "claude.dangerously_skip_permissions", "value": "yes" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);

    clear_env();
}

#[tokio::test]
async fn get_config_requires_auth() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let omega_dir = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_DIR", omega_dir.path());
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/config"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);

    clear_env();
}

/// Review-fix regression (an independent adversarial reviewer found this
/// before the branch shipped): `ProvidersConfig::load()` silently returns
/// defaults on a parse failure, so a naive PUT against a corrupt-but-present
/// `providers.toml` would load-default, apply the caller's ONE field, and
/// save that mostly-empty struct -- wiping every other provider's api_key.
/// This must now be a clean refusal that leaves the corrupt file untouched.
#[tokio::test]
async fn put_config_refuses_to_write_over_a_corrupt_providers_toml() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let omega_dir = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_DIR", omega_dir.path());
    let corrupt = "this is not valid TOML {{{ [[[ = = =";
    std::fs::write(omega_dir.path().join("providers.toml"), corrupt).unwrap();
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .put(format!("{base}/v1/config"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "key": "claude.model", "value": "opus" }))
        .send()
        .await
        .unwrap();
    // A server-side data-integrity refusal, not a client validation error.
    assert_eq!(res.status(), 500);

    // The corrupt file was NEVER overwritten -- proves no silent reset.
    let raw = std::fs::read_to_string(omega_dir.path().join("providers.toml")).unwrap();
    assert_eq!(raw, corrupt);

    clear_env();
}

/// Review-fix regression (final whole-branch review, Minor): the PUT-side
/// fix above closed the write path, but GET still silently reported "nothing
/// configured" on a corrupt file -- misleading in the same spirit as the
/// write-side bug (a box that IS fully configured renders as empty to an
/// app). GET now refuses the same way PUT does.
#[tokio::test]
async fn get_config_on_a_corrupt_providers_toml_is_500_not_a_silent_empty_view() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let omega_dir = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_DIR", omega_dir.path());
    let corrupt = "this is not valid TOML {{{ [[[ = = =";
    std::fs::write(omega_dir.path().join("providers.toml"), corrupt).unwrap();
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/config"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 500);

    clear_env();
}

/// Review-fix regression: a genuine server-side save failure (here: the
/// configured `$OMEGA_DIR` is itself an existing FILE, so `create_dir_all`
/// for its parent and the subsequent write both fail) must be a 500, never
/// the same 400 an allowlist-validation failure returns -- it is not the
/// client's fault.
#[tokio::test]
async fn put_config_save_io_failure_is_500_not_400() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let parent = tempfile::tempdir().unwrap();
    // $OMEGA_DIR points at a path whose PARENT component is a plain file,
    // not a directory -- create_dir_all must fail there.
    let blocked_parent = parent.path().join("not-a-directory");
    std::fs::write(&blocked_parent, b"x").unwrap();
    let bogus_omega_dir = blocked_parent.join("omega");
    std::env::set_var("OMEGA_DIR", &bogus_omega_dir);
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .put(format!("{base}/v1/config"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "key": "claude.model", "value": "opus" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 500);

    clear_env();
}

#[tokio::test]
async fn put_config_requires_auth() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let omega_dir = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_DIR", omega_dir.path());
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .put(format!("{base}/v1/config"))
        .json(&serde_json::json!({ "key": "claude.model", "value": "opus" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);

    clear_env();
}

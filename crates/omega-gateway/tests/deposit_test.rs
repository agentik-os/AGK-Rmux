//! `POST /v1/deposit` — HTTP-level tests.
//!
//! Every test in this file sets `OMEGA_DEPOSIT_DIR` to a FRESH tempdir before
//! it runs and removes the env var afterward — the real `~/.omega/deposit`
//! and `~/.omega/inbox` are never touched. `OMEGA_DEPOSIT_DIR` is
//! process-global, so tests here are serialized on `LOCK` (same pattern as
//! `dispatch_test.rs`'s `OMEGA_HOME`/`OMEGA_BIN` lock).

use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::routes_deposit::MAX_DEPOSIT_BYTES;
use omega_gateway::server::{build_router, AppState};

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

#[tokio::test]
async fn happy_path_fans_out_to_all_four_boxes() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let deposit_dir = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_DEPOSIT_DIR", deposit_dir.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let part = reqwest::multipart::Part::bytes(b"hello world".to_vec()).file_name("note.txt");
    let form = reqwest::multipart::Form::new().part("file", part);

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/deposit"))
        .bearer_auth(&token)
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["held"], false);
    let mut boxes: Vec<String> = serde_json::from_value(body["boxes"].clone()).unwrap();
    boxes.sort();
    assert_eq!(boxes, vec!["AltReality", "Box", "Home", "Omega"]);

    let file_name = body["file"].as_str().unwrap();
    assert!(deposit_dir.path().join("inbox").join(file_name).exists());
    for b in ["Home", "AltReality", "Omega", "Box"] {
        assert!(deposit_dir
            .path()
            .join("deposit")
            .join(b)
            .join(file_name)
            .exists());
    }

    std::env::remove_var("OMEGA_DEPOSIT_DIR");
}

#[tokio::test]
async fn secret_looking_upload_is_held_and_reaches_no_box() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let deposit_dir = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_DEPOSIT_DIR", deposit_dir.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let part = reqwest::multipart::Part::bytes(b"-----BEGIN KEY-----".to_vec()).file_name("id_rsa");
    let form = reqwest::multipart::Form::new().part("file", part);

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/deposit"))
        .bearer_auth(&token)
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["held"], true);
    assert_eq!(body["boxes"].as_array().unwrap().len(), 0);

    let deposit_root = deposit_dir.path().join("deposit");
    for b in ["Home", "AltReality", "Omega", "Box"] {
        let box_dir = deposit_root.join(b);
        let has_files = box_dir.exists() && std::fs::read_dir(&box_dir).unwrap().next().is_some();
        assert!(!has_files, "{b} must not have received the held file");
    }

    std::env::remove_var("OMEGA_DEPOSIT_DIR");
}

#[tokio::test]
async fn unknown_box_field_rejects_with_400() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let deposit_dir = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_DEPOSIT_DIR", deposit_dir.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let part = reqwest::multipart::Part::bytes(b"hello".to_vec()).file_name("note.txt");
    let form = reqwest::multipart::Form::new()
        .part("file", part)
        .text("box", "NotARealBox");

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/deposit"))
        .bearer_auth(&token)
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);

    std::env::remove_var("OMEGA_DEPOSIT_DIR");
}

/// Bug B1 regression test: a body strictly BETWEEN axum's own 2 MiB
/// multipart default and the crate's documented `MAX_DEPOSIT_BYTES` (50 MiB)
/// must succeed. Before the `DefaultBodyLimit` layer was scoped onto this
/// route, axum's own 2 MiB internal default silently rejected anything past
/// 2 MiB with a generic "failed to read file field" error — `MAX_DEPOSIT_BYTES`
/// was dead code above that point.
#[tokio::test]
async fn upload_between_axum_default_and_max_deposit_bytes_succeeds() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let deposit_dir = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_DEPOSIT_DIR", deposit_dir.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    // 5 MiB: comfortably past axum's 2 MiB internal default, comfortably
    // under MAX_DEPOSIT_BYTES (50 MiB).
    let mid_sized = vec![7u8; 5 * 1024 * 1024];
    let part = reqwest::multipart::Part::bytes(mid_sized.clone()).file_name("mid.bin");
    let form = reqwest::multipart::Form::new().part("file", part);

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/deposit"))
        .bearer_auth(&token)
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["held"], false);
    let file_name = body["file"].as_str().unwrap();
    let written = std::fs::read(deposit_dir.path().join("inbox").join(file_name)).unwrap();
    assert_eq!(written.len(), mid_sized.len());

    std::env::remove_var("OMEGA_DEPOSIT_DIR");
}

/// A body genuinely over `MAX_DEPOSIT_BYTES` (but still under the layer's
/// own +8192 margin, so it clears the `DefaultBodyLimit` layer) must still
/// be rejected with 400 from the CRATE's own size check, proven here by
/// asserting the error message shape (`"file too large"`, from
/// `routes_deposit.rs`) rather than a generic axum multipart-read error.
#[tokio::test]
async fn oversized_upload_rejects_with_400_from_the_crates_own_check() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let deposit_dir = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_DEPOSIT_DIR", deposit_dir.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let oversized = vec![0u8; MAX_DEPOSIT_BYTES + 1];
    let part = reqwest::multipart::Part::bytes(oversized).file_name("big.bin");
    let form = reqwest::multipart::Form::new().part("file", part);

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/deposit"))
        .bearer_auth(&token)
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(
        body["error"].as_str().unwrap().contains("file too large"),
        "expected the crate's own size-cap message, got: {body}"
    );

    std::env::remove_var("OMEGA_DEPOSIT_DIR");
}

#[tokio::test]
async fn post_deposit_requires_auth() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let deposit_dir = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_DEPOSIT_DIR", deposit_dir.path());

    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let part = reqwest::multipart::Part::bytes(b"hello".to_vec()).file_name("note.txt");
    let form = reqwest::multipart::Form::new().part("file", part);

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/deposit"))
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);

    std::env::remove_var("OMEGA_DEPOSIT_DIR");
}

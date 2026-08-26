use futures_util::StreamExt;
use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};
use tokio_tungstenite::connect_async;

// tokio::sync::Mutex (not std): the guard is held across .await points below,
// and clippy::await_holding_lock correctly flags a std guard doing that.
static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn install_fake_rmux(dir: &std::path::Path, script_body: &str) {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("rmux");
    std::fs::write(&path, format!("#!/usr/bin/env bash\n{script_body}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::env::set_var("OMEGA_RMUX_BIN", &path);
}

#[tokio::test]
async fn stream_sends_frame_then_only_on_change() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    // fake rmux: capture-pane output changes based on a counter file
    install_fake_rmux(
        dir.path(),
        &format!(
            r#"
counter="{}/count"
n=$(cat "$counter" 2>/dev/null || echo 0)
echo $((n+1)) > "$counter"
if [ $n -lt 2 ]; then echo "SCREEN-A"; else echo "SCREEN-B"; fi"#,
            dir.path().display()
        ),
    );
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let cfg = GatewayConfig {
        stream_interval_ms: 50,
        ..GatewayConfig::default()
    };
    let app = build_router(AppState::new(dir.path().to_path_buf(), cfg));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    let url = format!("ws://{addr}/v1/sessions/demo/stream?token={token}");
    let (mut ws, _) = connect_async(url).await.unwrap();

    let first = ws.next().await.unwrap().unwrap().into_text().unwrap();
    let f1: serde_json::Value = serde_json::from_str(&first).unwrap();
    assert_eq!(f1["type"], "frame");
    assert!(f1["text"].as_str().unwrap().contains("SCREEN-A"));

    // next frame arrives only when content changes to SCREEN-B
    let second = ws.next().await.unwrap().unwrap().into_text().unwrap();
    let f2: serde_json::Value = serde_json::from_str(&second).unwrap();
    assert!(f2["text"].as_str().unwrap().contains("SCREEN-B"));
}

#[tokio::test]
async fn color_query_param_passes_dash_e_to_capture_pane() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    // fake rmux: echoes COLOR-MODE only when invoked with a bare -e argument,
    // proving the handler actually passed -e through to capture-pane.
    install_fake_rmux(
        dir.path(),
        r#"if [[ " $* " == *" -e "* ]]; then echo "COLOR-MODE"; else echo "NO-COLOR"; fi"#,
    );
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let cfg = GatewayConfig {
        stream_interval_ms: 50,
        ..GatewayConfig::default()
    };
    let app = build_router(AppState::new(dir.path().to_path_buf(), cfg));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    let url = format!("ws://{addr}/v1/sessions/demo/stream?token={token}&color=1");
    let (mut ws, _) = connect_async(url).await.unwrap();

    let first = ws.next().await.unwrap().unwrap().into_text().unwrap();
    let f1: serde_json::Value = serde_json::from_str(&first).unwrap();
    assert_eq!(f1["type"], "frame");
    assert!(
        f1["text"].as_str().unwrap().contains("COLOR-MODE"),
        "expected -e to be passed to capture-pane when color=1, got: {first}"
    );
}

#[tokio::test]
async fn no_color_param_does_not_pass_dash_e() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    // Same sentinel script: absence of -e must render NO-COLOR, proving the
    // default (no query param) path is unchanged plain-text behavior.
    install_fake_rmux(
        dir.path(),
        r#"if [[ " $* " == *" -e "* ]]; then echo "COLOR-MODE"; else echo "NO-COLOR"; fi"#,
    );
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let cfg = GatewayConfig {
        stream_interval_ms: 50,
        ..GatewayConfig::default()
    };
    let app = build_router(AppState::new(dir.path().to_path_buf(), cfg));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    // No query param at all.
    let url = format!("ws://{addr}/v1/sessions/demo/stream?token={token}");
    let (mut ws, _) = connect_async(url).await.unwrap();
    let first = ws.next().await.unwrap().unwrap().into_text().unwrap();
    let f1: serde_json::Value = serde_json::from_str(&first).unwrap();
    assert!(f1["text"].as_str().unwrap().contains("NO-COLOR"));

    // Explicit color=0 behaves the same as absent.
    let url2 = format!("ws://{addr}/v1/sessions/demo2/stream?token={token}&color=0");
    let (mut ws2, _) = connect_async(url2).await.unwrap();
    let first2 = ws2.next().await.unwrap().unwrap().into_text().unwrap();
    let f2: serde_json::Value = serde_json::from_str(&first2).unwrap();
    assert!(f2["text"].as_str().unwrap().contains("NO-COLOR"));
}

#[tokio::test]
async fn capture_failure_becomes_error_frame_and_loop_survives() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    install_fake_rmux(dir.path(), "echo 'session not found' >&2; exit 1");
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let cfg = GatewayConfig {
        stream_interval_ms: 50,
        ..GatewayConfig::default()
    };
    let app = build_router(AppState::new(dir.path().to_path_buf(), cfg));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    let url = format!("ws://{addr}/v1/sessions/ghost/stream?token={token}");
    let (mut ws, _) = tokio_tungstenite::connect_async(url).await.unwrap();
    let msg = ws.next().await.unwrap().unwrap().into_text().unwrap();
    let frame: serde_json::Value = serde_json::from_str(&msg).unwrap();
    assert_eq!(frame["type"], "error");
    assert!(frame["message"]
        .as_str()
        .unwrap()
        .contains("session not found"));
    // the connection is still alive: another error frame arrives instead of a close
    let msg2 = ws.next().await.unwrap().unwrap();
    assert!(msg2.is_text());
}

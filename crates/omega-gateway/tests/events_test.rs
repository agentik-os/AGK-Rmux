use futures_util::StreamExt;
use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::protocol::GatewayEvent;
use omega_gateway::server::{build_router, AppState};
use tokio_tungstenite::connect_async;

#[tokio::test]
async fn authed_device_receives_emitted_alert_frame() {
    let dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let state = AppState::new(dir.path().to_path_buf(), GatewayConfig::default());
    // Hold our own clone of the hub so we can emit into the same bus the
    // router forwards from, exactly as the brief prescribes (no test-only
    // emit endpoint needed).
    let hub = state.events.clone();
    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    let url = format!("ws://{addr}/v1/events?token={token}");
    let (mut ws, _) = connect_async(url).await.unwrap();

    // Give the server task a beat to register the subscription before we
    // emit, so the broadcast isn't sent before anyone is listening.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    hub.emit(GatewayEvent::Alert {
        message: "disk full".into(),
        ts: "2026-08-10T00:00:00Z".into(),
    });

    let msg = ws.next().await.unwrap().unwrap().into_text().unwrap();
    let frame: serde_json::Value = serde_json::from_str(&msg).unwrap();
    assert_eq!(frame["type"], "alert");
    assert_eq!(frame["message"], "disk full");
    assert_eq!(frame["ts"], "2026-08-10T00:00:00Z");
}

#[tokio::test]
async fn events_requires_auth() {
    let dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    let url = format!("ws://{addr}/v1/events");
    let err = connect_async(url).await.unwrap_err();
    // No Authorization header and no ?token= query: the auth middleware
    // rejects the upgrade with 401 before the WS handshake completes.
    let msg = err.to_string();
    assert!(
        msg.contains("401") || msg.contains("Unauthorized"),
        "unexpected error: {msg}"
    );
}

#[tokio::test]
async fn mission_updated_and_heartbeat_frames_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let state = AppState::new(dir.path().to_path_buf(), GatewayConfig::default());
    let hub = state.events.clone();
    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    let url = format!("ws://{addr}/v1/events?token={token}");
    let (mut ws, _) = connect_async(url).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    hub.emit(GatewayEvent::MissionUpdated {
        key: "oracle-x".into(),
        updated_at: "t1".into(),
    });
    let msg1 = ws.next().await.unwrap().unwrap().into_text().unwrap();
    let f1: serde_json::Value = serde_json::from_str(&msg1).unwrap();
    assert_eq!(f1["type"], "mission_updated");
    assert_eq!(f1["key"], "oracle-x");
    assert_eq!(f1["updated_at"], "t1");

    hub.emit(GatewayEvent::Heartbeat { ts: "t2".into() });
    let msg2 = ws.next().await.unwrap().unwrap().into_text().unwrap();
    let f2: serde_json::Value = serde_json::from_str(&msg2).unwrap();
    assert_eq!(f2["type"], "heartbeat");
    assert_eq!(f2["ts"], "t2");
}

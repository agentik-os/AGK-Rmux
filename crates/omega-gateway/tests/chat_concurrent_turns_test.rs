//! I-5 regression test: two WebSocket connections to the SAME chat must not
//! run concurrent turns. Before the fix, `chat_permits` was only a GLOBAL
//! cap — a second connection to the same chat id could acquire its own
//! permit and spawn a second `run_turn` while the first was still running,
//! racing the transcript append and `set_provider_session` writes.
//!
//! This is the HTTP+WS-level regression test (preferred per the task
//! brief). A store-level unit test for `ChatStore::try_start_turn`/
//! `end_turn` also lives directly in `src/chat_store.rs`'s own `#[cfg(test)]`
//! module as additional, cheaper-to-run evidence for the same guard.

use futures_util::{SinkExt, StreamExt};
use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

type Ws = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

// Mutates the process-global OMEGA_CHAT_BIN env var, same serialization
// pattern as chat_routes_test.rs / chat_driver_test.rs.
static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn spawn(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

fn ws_url(base: &str, path: &str, token: &str) -> String {
    format!("{}{path}?token={token}", base.replacen("http", "ws", 1))
}

async fn send_user_message(ws: &mut Ws, text: &str) {
    let payload = serde_json::json!({ "type": "user_message", "text": text }).to_string();
    ws.send(Message::Text(payload)).await.unwrap();
}

async fn recv_json(ws: &mut Ws) -> serde_json::Value {
    let msg = ws.next().await.unwrap().unwrap();
    serde_json::from_str(&msg.into_text().unwrap()).unwrap()
}

/// Writes a fake agent that signals it has actually started (touching
/// `started_file`) before sleeping for `sleep_secs`, then emits a normal
/// successful turn. Long enough that a second concurrent attempt is
/// genuinely racing an in-flight turn, not a race against process startup.
fn install_slow_fake_agent(dir: &std::path::Path, started_file: &std::path::Path, sleep_secs: u64) {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("slow-fake-agent");
    let body = format!(
        "#!/usr/bin/env bash\ntouch '{started}'\nprintf '%s\\n' '{{\"type\":\"system\",\"subtype\":\"init\",\"session_id\":\"sess-slow\",\"cwd\":\"/tmp\",\"model\":\"m\"}}'\nsleep {sleep_secs}\nprintf '%s\\n' '{{\"type\":\"result\",\"is_error\":false,\"stop_reason\":\"end_turn\",\"result\":\"done\"}}'\n",
        started = started_file.display(),
    );
    std::fs::write(&path, body).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::env::set_var("OMEGA_CHAT_BIN", &path);
}

#[tokio::test]
async fn second_ws_turn_on_same_chat_is_rejected_while_first_is_active() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    let started_file = dir.path().join("started");
    install_slow_fake_agent(dir.path(), &started_file, 2);

    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let state = AppState::new(dir.path().to_path_buf(), GatewayConfig::default());
    let app = build_router(state.clone());
    let base = spawn(app).await;
    let client = reqwest::Client::new();

    // I-1's cwd confinement rejects a bare "/tmp" (outside the real
    // $HOME), so the chat's cwd here just needs to be a real, existing
    // absolute path under $HOME — this test is about I-5, not I-1, so a
    // simple tempdir directly under $HOME is created and used rather than
    // spinning up a fake HOME here too.
    let home = dirs::home_dir().expect("no home dir on this test box");
    let cwd_dir = tempfile::tempdir_in(&home).expect("failed to create a tempdir under $HOME");
    let cwd_str = cwd_dir.path().display().to_string();

    let create_res = client
        .post(format!("{base}/v1/chats"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "agent": "claude", "cwd": cwd_str }))
        .send()
        .await
        .unwrap();
    assert_eq!(create_res.status(), 201);
    let meta: serde_json::Value = create_res.json().await.unwrap();
    let chat_id = meta["id"].as_str().unwrap().to_string();

    // WS #1 starts a turn that will sleep for a few seconds.
    let url = ws_url(&base, &format!("/v1/chats/{chat_id}/stream"), &token);
    let (mut ws1, _) = connect_async(&url).await.unwrap();
    send_user_message(&mut ws1, "first").await;

    // Wait for the fake agent to genuinely have started before racing the
    // second connection against it.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !started_file.exists() {
        assert!(
            std::time::Instant::now() < deadline,
            "first turn's agent never started"
        );
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    // WS #2 attempts a turn on the SAME chat id while #1 is still active.
    let (mut ws2, _) = connect_async(&url).await.unwrap();
    send_user_message(&mut ws2, "second").await;

    let mut saw_busy_error = false;
    loop {
        let frame = recv_json(&mut ws2).await;
        match frame["type"].as_str().unwrap() {
            "error" => {
                assert!(
                    frame["message"]
                        .as_str()
                        .unwrap()
                        .contains("already active"),
                    "unexpected error message on ws2: {frame}"
                );
                saw_busy_error = true;
            }
            "turn_done" => break,
            other => panic!("unexpected frame type on ws2: {other}"),
        }
    }
    assert!(
        saw_busy_error,
        "a second concurrent turn on the same chat must be rejected"
    );

    // The rejected attempt must never have spawned a real turn: no
    // "assistant" role message can exist yet (the only agent process that
    // ever ran is the first turn's, still sleeping at this point — same
    // "no turn ran" shape the busy-semaphore test already asserts for the
    // global cap).
    let transcript_before_first_finishes = state.chats.transcript(&chat_id);
    assert!(
        transcript_before_first_finishes.iter().all(|m| m.role != "assistant"),
        "no assistant turn should have run yet for either user message: {transcript_before_first_finishes:?}"
    );

    // Drain WS #1 to completion so the sleeping fake agent doesn't outlive
    // this test.
    loop {
        let frame = recv_json(&mut ws1).await;
        if frame["type"] == "turn_done" {
            break;
        }
    }

    // Now that the first turn has ended, a new turn on the same chat must
    // be allowed again (the guard is released, not permanently wedged).
    let (mut ws3, _) = connect_async(&url).await.unwrap();
    send_user_message(&mut ws3, "third").await;
    let mut saw_error_on_third = false;
    loop {
        let frame = recv_json(&mut ws3).await;
        match frame["type"].as_str().unwrap() {
            "error" => saw_error_on_third = true,
            "turn_done" => break,
            _ => {}
        }
    }
    assert!(
        !saw_error_on_third,
        "the per-chat guard must release once the prior turn has ended"
    );

    std::env::remove_var("OMEGA_CHAT_BIN");
}

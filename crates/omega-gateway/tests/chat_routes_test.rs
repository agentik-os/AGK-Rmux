use futures_util::{SinkExt, StreamExt};
use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::protocol::ChatAgent;
use omega_gateway::server::{build_router, AppState};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

type Ws = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

// Both OMEGA_CHAT_BIN-mutating tests must never run concurrently with each
// other (same pattern as chat_driver_test.rs's lock). tokio::sync::Mutex
// because the guard is held across .await points below. These two tests
// also mutate the process-global HOME env var (see `HomeRestore` below, same
// pattern `chat_cwd_validation_test.rs` uses) since `POST /v1/chats` now
// confines `cwd` under `$HOME` (I-1 fix) — the same LOCK already serializes
// that too.
static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// RAII guard restoring (or clearing) the `HOME` env var on drop, so a
/// panicking assertion mid-test never leaves a later test in this binary
/// running against the wrong home directory. Same shape as
/// `chat_cwd_validation_test.rs::HomeRestore`.
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

async fn spawn(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

fn ws_url(base: &str, path: &str, token: &str) -> String {
    format!("{}{path}?token={token}", base.replacen("http", "ws", 1))
}

/// Writes an executable fake agent script that logs its own args (one line
/// per invocation) to `argv_file`, then points OMEGA_CHAT_BIN at it.
fn install_fake_agent(bin_dir: &std::path::Path, argv_file: &std::path::Path, script_body: &str) {
    use std::os::unix::fs::PermissionsExt;
    let path = bin_dir.join("fake-agent");
    std::fs::write(
        &path,
        format!(
            "#!/usr/bin/env bash\necho \"$@\" >> {}\n{script_body}\n",
            argv_file.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::env::set_var("OMEGA_CHAT_BIN", &path);
}

async fn send_user_message(ws: &mut Ws, text: &str) {
    let payload = serde_json::json!({ "type": "user_message", "text": text }).to_string();
    ws.send(Message::Text(payload)).await.unwrap();
}

async fn recv_json(ws: &mut Ws) -> serde_json::Value {
    let msg = ws.next().await.unwrap().unwrap();
    serde_json::from_str(&msg.into_text().unwrap()).unwrap()
}

#[tokio::test]
async fn full_turn_streams_once_and_persists_transcript() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    // I-1: `POST /v1/chats` now confines `cwd` under `$HOME` via
    // `dir_under_home`, so this test's chat must run under a `HOME` it
    // controls rather than the ambient ("/tmp" is never under home).
    let _home = HomeRestore::set(dir.path());
    let project_dir = dir.path().join("proj");
    std::fs::create_dir_all(&project_dir).unwrap();
    let cwd = project_dir.to_string_lossy().into_owned();
    let argv_file = dir.path().join("argv.log");
    install_fake_agent(
        dir.path(),
        &argv_file,
        r#"
printf '%s\n' '{"type":"system","subtype":"init","session_id":"sess-A","cwd":"/tmp","model":"m"}'
printf '%s\n' '{"type":"assistant","message":{"content":[{"type":"text","text":"PONG"}]}}'
printf '%s\n' '{"type":"result","is_error":false,"stop_reason":"end_turn","result":"PONG"}'
"#,
    );

    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;
    let client = reqwest::Client::new();

    // POST /v1/chats -> 201 ChatMeta
    let create_res = client
        .post(format!("{base}/v1/chats"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "agent": "claude", "cwd": cwd }))
        .send()
        .await
        .unwrap();
    assert_eq!(create_res.status(), 201);
    let meta: serde_json::Value = create_res.json().await.unwrap();
    let chat_id = meta["id"].as_str().unwrap().to_string();
    assert_eq!(meta["agent"], "claude");

    // GET /v1/chats lists it
    let list_res: serde_json::Value = client
        .get(format!("{base}/v1/chats"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let ids: Vec<&str> = list_res["chats"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["id"].as_str().unwrap())
        .collect();
    assert!(ids.contains(&chat_id.as_str()));

    // Open the chat WS and run the first turn.
    let url = ws_url(&base, &format!("/v1/chats/{chat_id}/stream"), &token);
    let (mut ws, _) = connect_async(url).await.unwrap();
    send_user_message(&mut ws, "hi").await;

    let mut saw_assistant_text = false;
    let mut turn_done_count = 0;
    loop {
        let frame = recv_json(&mut ws).await;
        match frame["type"].as_str().unwrap() {
            "assistant_message" => {
                assert_eq!(frame["text"], "PONG");
                saw_assistant_text = true;
            }
            "turn_done" => {
                turn_done_count += 1;
                break;
            }
            other => panic!("unexpected frame type: {other}"),
        }
    }
    assert!(
        saw_assistant_text,
        "expected an assistant_message frame before turn_done"
    );
    assert_eq!(turn_done_count, 1, "exactly one turn_done per turn");

    // GET /v1/chats/{id} shows the persisted user + assistant messages.
    let get_res: serde_json::Value = client
        .get(format!("{base}/v1/chats/{chat_id}"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let messages = get_res["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0]["role"], "user");
    assert_eq!(messages[0]["text"], "hi");
    assert_eq!(messages[1]["role"], "assistant");
    assert_eq!(messages[1]["text"], "PONG");
    assert_eq!(get_res["meta"]["provider_session_id"], "sess-A");

    // Second turn on the SAME socket: it must resume the provider session.
    send_user_message(&mut ws, "again").await;
    let mut turn_done_count2 = 0;
    loop {
        let frame = recv_json(&mut ws).await;
        if frame["type"] == "turn_done" {
            turn_done_count2 += 1;
            break;
        }
    }
    assert_eq!(turn_done_count2, 1);

    let argv_log = std::fs::read_to_string(&argv_file).unwrap();
    let lines: Vec<&str> = argv_log.lines().collect();
    assert_eq!(
        lines.len(),
        2,
        "fake agent should have been invoked exactly twice"
    );
    assert!(
        !lines[0].contains("--resume"),
        "the first turn must not pass --resume"
    );
    assert!(
        lines[1].contains("--resume sess-A"),
        "the second turn must resume the first turn's session"
    );

    std::env::remove_var("OMEGA_CHAT_BIN");
}

#[tokio::test]
async fn hung_child_double_turn_done_is_deduped() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    // I-1: same HOME/cwd confinement as `full_turn_streams_once_and_persists_transcript`.
    let _home = HomeRestore::set(dir.path());
    let project_dir = dir.path().join("proj");
    std::fs::create_dir_all(&project_dir).unwrap();
    let cwd = project_dir.to_string_lossy().into_owned();
    let argv_file = dir.path().join("argv.log");
    // A misbehaving/hung-then-recovered agent that emits TWO result lines
    // (i.e. two TurnDone-worthy events) in one stream, with more assistant
    // text after the first. The route must forward/persist only the first.
    install_fake_agent(
        dir.path(),
        &argv_file,
        r#"
printf '%s\n' '{"type":"system","subtype":"init","session_id":"sess-B","cwd":"/tmp","model":"m"}'
printf '%s\n' '{"type":"assistant","message":{"content":[{"type":"text","text":"FIRST"}]}}'
printf '%s\n' '{"type":"result","is_error":false,"stop_reason":"end_turn","result":"FIRST"}'
printf '%s\n' '{"type":"assistant","message":{"content":[{"type":"text","text":"SHOULD-BE-IGNORED"}]}}'
printf '%s\n' '{"type":"result","is_error":false,"stop_reason":"end_turn","result":"SECOND"}'
"#,
    );

    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;
    let client = reqwest::Client::new();

    let create_res = client
        .post(format!("{base}/v1/chats"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "agent": "claude", "cwd": cwd }))
        .send()
        .await
        .unwrap();
    let meta: serde_json::Value = create_res.json().await.unwrap();
    let chat_id = meta["id"].as_str().unwrap().to_string();

    let url = ws_url(&base, &format!("/v1/chats/{chat_id}/stream"), &token);
    let (mut ws, _) = connect_async(url).await.unwrap();
    send_user_message(&mut ws, "hi").await;

    let mut turn_done_count = 0;
    let mut assistant_texts = Vec::new();
    loop {
        let frame = recv_json(&mut ws).await;
        match frame["type"].as_str().unwrap() {
            "assistant_message" => {
                assistant_texts.push(frame["text"].as_str().unwrap().to_string())
            }
            "turn_done" => {
                turn_done_count += 1;
                break;
            }
            other => panic!("unexpected frame type: {other}"),
        }
    }
    assert_eq!(turn_done_count, 1, "client must see exactly one turn_done");
    assert_eq!(
        assistant_texts,
        vec!["FIRST"],
        "text after the first turn_done must never be forwarded"
    );

    let get_res: serde_json::Value = client
        .get(format!("{base}/v1/chats/{chat_id}"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let messages = get_res["messages"].as_array().unwrap();
    assert_eq!(
        messages.len(),
        2,
        "exactly one user + one assistant message, no duplicate persist"
    );
    assert_eq!(messages[1]["text"], "FIRST");

    std::env::remove_var("OMEGA_CHAT_BIN");
}

#[tokio::test]
async fn create_with_invalid_agent_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/chats"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "agent": "not-a-real-agent", "cwd": "/tmp" }))
        .send()
        .await
        .unwrap();
    // axum's default Json extractor rejects an undeserializable body with
    // 422 (a serde-driven rejection); either 400 or 422 satisfies "the
    // server refuses a bad agent value" per the task brief.
    assert_eq!(res.status(), 422);
}

#[tokio::test]
async fn get_unknown_chat_is_404() {
    let dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/chats/nonexistent"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn busy_semaphore_reports_error_and_turn_done_without_persisting_assistant() {
    let dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let state = AppState::new(dir.path().to_path_buf(), GatewayConfig::default());
    let meta = state
        .chats
        .create(ChatAgent::Claude, "/tmp".to_string(), None, None);
    let chat_id = meta.id.clone();

    // Exhaust every permit and hold it for the whole test: no OMEGA_CHAT_BIN
    // is installed, so if the busy check didn't short-circuit before
    // run_turn, this would fail loudly by trying to spawn a real `claude`.
    let mut held = Vec::new();
    while let Ok(p) = state.chat_permits.clone().try_acquire_owned() {
        held.push(p);
    }
    assert!(
        !held.is_empty(),
        "the semaphore must have had at least one permit to exhaust"
    );

    let app = build_router(state.clone());
    let base = spawn(app).await;

    let url = ws_url(&base, &format!("/v1/chats/{chat_id}/stream"), &token);
    let (mut ws, _) = connect_async(url).await.unwrap();
    send_user_message(&mut ws, "hi").await;

    let mut saw_busy_error = false;
    loop {
        let frame = recv_json(&mut ws).await;
        match frame["type"].as_str().unwrap() {
            "error" => {
                assert!(frame["message"].as_str().unwrap().contains("busy"));
                saw_busy_error = true;
            }
            "turn_done" => break,
            other => panic!("unexpected frame before turn_done: {other}"),
        }
    }
    assert!(saw_busy_error);

    let transcript = state.chats.transcript(&chat_id);
    assert_eq!(
        transcript.len(),
        1,
        "only the user message should be persisted, no assistant turn ran"
    );
    assert_eq!(transcript[0].role, "user");

    drop(held);
}

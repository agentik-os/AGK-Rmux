use omega_gateway::chat_driver;
use omega_gateway::protocol::{ChatAgent, ChatMeta, ChatStreamServerMsg};
use std::time::Duration;

// Both tests mutate the process-global OMEGA_CHAT_BIN env var, so they must
// never run concurrently with each other (same pattern as the rmux tests'
// OMEGA_RMUX_BIN lock). tokio::sync::Mutex because the guard is held across
// .await points below.
static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Writes an executable fake agent script and points OMEGA_CHAT_BIN at it.
fn install_fake_agent(dir: &std::path::Path, script_body: &str) {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("fake-agent");
    std::fs::write(&path, format!("#!/usr/bin/env bash\n{script_body}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::env::set_var("OMEGA_CHAT_BIN", &path);
}

fn test_meta(cwd: &std::path::Path) -> ChatMeta {
    ChatMeta {
        id: "chat1".to_string(),
        title: None,
        agent: ChatAgent::Claude,
        cwd: cwd.to_string_lossy().to_string(),
        created_at: "t".to_string(),
        updated_at: "t".to_string(),
        provider_session_id: None,
        account_slug: None,
    }
}

#[tokio::test]
async fn run_turn_forwards_frames_and_returns_session_id() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    install_fake_agent(
        dir.path(),
        r#"
printf '%s\n' '{"type":"system","subtype":"init","session_id":"fake-session-1","cwd":"/tmp","model":"m"}'
printf '%s\n' '{"type":"assistant","message":{"content":[{"type":"text","text":"PONG"}]}}'
printf '%s\n' '{"type":"result","is_error":false,"stop_reason":"end_turn","result":"PONG"}'
"#,
    );

    let meta = test_meta(dir.path());
    let (tx, mut rx) = tokio::sync::mpsc::channel(16);
    let driver = tokio::spawn(async move {
        chat_driver::run_turn(&meta, "hi", None, None, Duration::from_secs(5), tx).await
    });

    let mut frames = Vec::new();
    while let Some(frame) = rx.recv().await {
        frames.push(frame);
    }
    let session_id = driver.await.unwrap();

    assert_eq!(session_id.as_deref(), Some("fake-session-1"));
    assert_eq!(
        frames.len(),
        2,
        "expected an AssistantMessage then a TurnDone"
    );
    assert!(matches!(
        &frames[0],
        ChatStreamServerMsg::AssistantMessage { text } if text == "PONG"
    ));
    assert!(matches!(&frames[1], ChatStreamServerMsg::TurnDone));
}

#[tokio::test]
async fn run_turn_appends_turn_done_when_stream_lacks_a_result_line() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    // No "result" line at all — run_turn must still synthesize a TurnDone
    // once the process exits (EOF), per the "always send a final TurnDone"
    // contract.
    install_fake_agent(
        dir.path(),
        r#"printf '%s\n' '{"type":"assistant","message":{"content":[{"type":"text","text":"hi there"}]}}'"#,
    );

    let meta = test_meta(dir.path());
    let (tx, mut rx) = tokio::sync::mpsc::channel(16);
    let driver = tokio::spawn(async move {
        chat_driver::run_turn(&meta, "hi", None, None, Duration::from_secs(5), tx).await
    });

    let mut frames = Vec::new();
    while let Some(frame) = rx.recv().await {
        frames.push(frame);
    }
    let session_id = driver.await.unwrap();

    assert!(session_id.is_none(), "no init line was ever printed");
    assert_eq!(frames.len(), 2);
    assert!(matches!(
        &frames[0],
        ChatStreamServerMsg::AssistantMessage { text } if text == "hi there"
    ));
    assert!(matches!(&frames[1], ChatStreamServerMsg::TurnDone));
}

#[tokio::test]
async fn run_turn_kills_child_and_reports_timeout() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    let pidfile = dir.path().join("child.pid");
    // `exec` replaces the bash script's own process image with `sleep`
    // *in place* (same PID), so: (1) killing the spawned child kills the
    // sleep directly instead of leaving an orphaned grandchild running for
    // 999s, and (2) `$$` captured before the `exec` is still valid as the
    // sleep's PID afterward, which is what lets this test prove the process
    // is actually gone rather than just trusting that a signal was sent.
    install_fake_agent(
        dir.path(),
        &format!("echo $$ > {}\nexec sleep 999", pidfile.display()),
    );

    let meta = test_meta(dir.path());
    let (tx, mut rx) = tokio::sync::mpsc::channel(16);
    let started = std::time::Instant::now();
    let driver = tokio::spawn(async move {
        chat_driver::run_turn(&meta, "hi", None, None, Duration::from_millis(200), tx).await
    });

    // Wait for the fake agent to report its PID before the timeout fires,
    // so the leak check below has a concrete PID to test against.
    let child_pid: u32 = {
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            if let Ok(s) = std::fs::read_to_string(&pidfile) {
                if let Ok(pid) = s.trim().parse() {
                    break pid;
                }
            }
            assert!(
                std::time::Instant::now() < deadline,
                "fake agent never wrote its pidfile"
            );
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    };

    let mut frames = Vec::new();
    while let Some(frame) = rx.recv().await {
        frames.push(frame);
    }
    let session_id = driver.await.unwrap();
    let elapsed = started.elapsed();

    assert!(
        elapsed < Duration::from_secs(5),
        "run_turn should have killed the sleeping child well before its own 999s sleep, took {elapsed:?}"
    );
    assert!(session_id.is_none(), "no init line was ever printed");
    assert_eq!(frames.len(), 2);
    assert!(matches!(
        &frames[0],
        ChatStreamServerMsg::Error { message } if message.contains("timed out")
    ));
    assert!(matches!(&frames[1], ChatStreamServerMsg::TurnDone));

    // The real leak check: after run_turn has returned (so the kill+wait it
    // performs internally has already completed), the killed process must
    // no longer exist in the process table.
    #[cfg(target_os = "linux")]
    {
        // The kernel can take a moment to reap a killed process; poll
        // briefly rather than asserting on the first sample.
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        loop {
            let alive = std::path::Path::new(&format!("/proc/{child_pid}")).exists();
            if !alive {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "child process {child_pid} is still present in /proc after run_turn returned \
                 — the timeout path leaked it"
            );
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
}

#[tokio::test]
async fn run_turn_streams_codex_json_and_writes_prompt_to_stdin() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    install_fake_agent(
        dir.path(),
        r#"
IFS= read -r prompt || true
test "$prompt" = "hi" || { echo "wrong stdin prompt" >&2; exit 2; }
printf '%s\n' '{"type":"thread.started","thread_id":"codex-thread-1"}'
printf '%s\n' '{"type":"item.completed","item":{"id":"item_1","type":"agent_message","text":"CODEX PONG"}}'
printf '%s\n' '{"type":"turn.completed","usage":{}}'
"#,
    );

    let mut meta = test_meta(dir.path());
    meta.agent = ChatAgent::Codex;
    let (tx, mut rx) = tokio::sync::mpsc::channel(16);
    let driver = tokio::spawn(async move {
        chat_driver::run_turn(&meta, "hi", None, None, Duration::from_secs(5), tx).await
    });

    let mut frames = Vec::new();
    while let Some(frame) = rx.recv().await {
        frames.push(frame);
    }
    let session_id = driver.await.unwrap();

    assert_eq!(session_id.as_deref(), Some("codex-thread-1"));
    assert_eq!(frames.len(), 2);
    assert!(matches!(
        &frames[0],
        ChatStreamServerMsg::AssistantMessage { text } if text == "CODEX PONG"
    ));
    assert!(matches!(&frames[1], ChatStreamServerMsg::TurnDone));
}

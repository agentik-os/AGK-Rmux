//! I-3 (chat half) regression test: `run_turn`'s kill paths must reach a
//! NESTED process the agent spawned, not just the direct `claude` process.
//! Before the fix, `child.kill()` alone terminated only the direct child;
//! a background grandchild in the same process group survived.
//!
//! Mirrors `tests/duo_test.rs`'s `install_fake_duo_with_nested_child` /
//! `client_disconnect_kills_the_nested_agent_and_releases_the_cwd_lock`
//! pattern, adapted to `chat_driver::run_turn`'s TIMEOUT kill path (chat's
//! turn is driven from a `tokio::spawn`ed task per `routes_chat.rs::
//! stream_loop`, not a plain request/response handler, so unlike
//! `routes_duo.rs` there is no ordinary "client disconnect drops the whole
//! handler future" trigger here — the timeout path is the one that fires in
//! practice, and it is exactly what the review's I-3 trigger describes:
//! "start a chat turn ... wait for timeout").

use omega_gateway::chat_driver;
use omega_gateway::protocol::{ChatAgent, ChatMeta};
use std::time::Duration;

// Mutates the process-global OMEGA_CHAT_BIN env var, same serialization
// pattern as chat_driver_test.rs's own LOCK.
static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

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

/// Writes a fake `claude` binary that forks a NESTED grandchild background
/// loop into the SAME process group as the outer script (a plain `&`
/// background job, no `setsid` — bash job control is off by default in a
/// non-interactive script, so the job inherits the parent's pgid, mirroring
/// a real nested tool process Claude might spawn). The grandchild writes a
/// growing marker file (so its aliveness is directly observable) and its
/// own PID to `pid_file` via `$!`. The outer script then sleeps far longer
/// than the test's timeout and never prints a "result" line, so the turn
/// can only finish within test patience via the timeout-triggered kill.
fn install_fake_agent_with_nested_child(
    dir: &std::path::Path,
    pid_file: &std::path::Path,
    marker_file: &std::path::Path,
) {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("fake-agent-nested");
    let body = format!(
        "#!/usr/bin/env bash\n( while true; do date +%s%N >> '{marker}'; sleep 0.05; done ) &\necho $! > '{pid}'\nsleep 120\n",
        marker = marker_file.display(),
        pid = pid_file.display(),
    );
    std::fs::write(&path, body).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::env::set_var("OMEGA_CHAT_BIN", &path);
}

#[tokio::test]
async fn timeout_kills_the_nested_grandchild_not_just_the_direct_child() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    let pid_file = dir.path().join("nested.pid");
    let marker_file = dir.path().join("marker.txt");
    install_fake_agent_with_nested_child(dir.path(), &pid_file, &marker_file);

    let meta = test_meta(dir.path());
    let (tx, mut rx) = tokio::sync::mpsc::channel(16);
    let driver = tokio::spawn(async move {
        chat_driver::run_turn(&meta, "hi", None, None, Duration::from_millis(300), tx).await
    });

    // Wait for the nested grandchild to actually exist.
    let mut nested_pid: Option<u32> = None;
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while nested_pid.is_none() {
        if let Ok(s) = std::fs::read_to_string(&pid_file) {
            if let Ok(p) = s.trim().parse::<u32>() {
                nested_pid = Some(p);
            }
        }
        assert!(
            std::time::Instant::now() < deadline,
            "nested grandchild never started"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    let nested_pid = nested_pid.expect("nested grandchild never started");
    assert!(
        std::path::Path::new(&format!("/proc/{nested_pid}")).exists(),
        "the nested grandchild should be alive right after it announced its pid"
    );

    // Prove it is genuinely alive (not a stale pid file) before the kill.
    let before = std::fs::read_to_string(&marker_file).unwrap_or_default();
    tokio::time::sleep(Duration::from_millis(150)).await;
    let grew = std::fs::read_to_string(&marker_file).unwrap_or_default();
    assert!(
        grew.len() > before.len(),
        "nested grandchild marker was not growing before the timeout"
    );

    // Let run_turn finish (its own 300ms timeout fires well before the
    // outer script's 120s sleep).
    let mut frames = Vec::new();
    while let Some(frame) = rx.recv().await {
        frames.push(frame);
    }
    let _ = driver.await.unwrap();
    assert!(
        frames.iter().any(|f| matches!(f, omega_gateway::protocol::ChatStreamServerMsg::Error { message } if message.contains("timed out"))),
        "run_turn should have reported the timeout"
    );

    // The real assertion this test exists for: the nested grandchild must
    // actually be gone, not just the direct `claude` process.
    #[cfg(target_os = "linux")]
    {
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        loop {
            if !std::path::Path::new(&format!("/proc/{nested_pid}")).exists() {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "nested grandchild (pid {nested_pid}) survived run_turn's timeout kill — \
                 a process-group kill did not reach it"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    std::env::remove_var("OMEGA_CHAT_BIN");
}

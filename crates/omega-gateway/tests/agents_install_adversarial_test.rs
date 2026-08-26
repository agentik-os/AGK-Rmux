//! Adversarial verification tests written by an independent reviewer (not
//! the implementer) for the `GET /v1/agents/{name}/install/stream` WS route
//! in `crates/omega-gateway/src/routes_agents.rs`.
//!
//! SAFETY: exactly like `agents_install_test.rs`, every test here sets
//! `OMEGA_BIN` to a fake, purely-local bash script BEFORE making any
//! request that could reach `install_stream`. No test in this file ever
//! touches the real `~/.local/bin/omega`, `curl`, or `npm`, and no test
//! reaches the network.
//!
//! Two things are verified here that the existing suite does not cover:
//!
//! 1. `dual_stream_large_volume_...`: a few thousand lines on BOTH stdout
//!    and stderr, concurrently, prove the two `forward_lines` tasks feeding
//!    one mpsc channel don't deadlock or drop lines under volume.
//!
//! 2. `mid_stream_client_disconnect_...`: a client that disconnects while
//!    the fake installer is still producing output. This is the important
//!    one — it does NOT merely check "the test process doesn't hang" (that
//!    alone would pass even if the bug below existed, because
//!    `forward_lines` returning on a failed `tx.send` after `drop(rx)`
//!    eventually unblocks *this* test regardless). It checks whether the
//!    REAL child process tree is actually terminated on disconnect, by
//!    giving the fake "installer" a NESTED child process (mirroring
//!    `omega-cli`'s real `cmd_install`, which runs `bash -c
//!    <install_command>` via `.status()` — a genuine child of the `omega`
//!    process, not the process the gateway itself spawned) that would write
//!    a marker file well after the point where the gateway kills on
//!    disconnect. `install_stream_loop` now spawns `omega` into its own
//!    process group (`process_group(0)`) and, on disconnect, kills the
//!    WHOLE group (`kill -- -<pid>`, see `kill_process_group` in
//!    `routes_agents.rs`) instead of the single direct PID — which reaches
//!    the nested `bash -c` too, since it inherits the group. If the marker
//!    file is written anyway, the group-kill failed to actually stop the
//!    real work — the socket closed but the install kept running
//!    unsupervised.

use futures_util::StreamExt;
use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};
use tokio_tungstenite::connect_async;

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

fn install_fake_omega(dir: &std::path::Path, script_body: &str) {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("omega");
    std::fs::write(&path, format!("#!/usr/bin/env bash\n{script_body}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::env::set_var("OMEGA_BIN", &path);
}

/// (1) Large-volume dual-stream: 3000 stdout lines + 3000 stderr lines,
/// printed with NO delay so the OS pipe buffers (typically 64 KiB) fill up
/// many times over on both fds concurrently. Proves the two `forward_lines`
/// tasks (one per fd) reading concurrently into one mpsc channel neither
/// deadlock nor drop lines.
#[tokio::test]
async fn dual_stream_large_volume_neither_deadlocks_nor_drops_lines() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    install_fake_omega(
        dir.path(),
        r#"
if [ "$1" = "install" ]; then
    for i in $(seq 1 3000); do
        echo "out $i"
        echo "err $i" >&2
    done
    exit 0
fi
exit 1
"#,
    );
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let url = ws_url(&base, "/v1/agents/codex/install/stream", &token);
    let (mut ws, _) = connect_async(url).await.unwrap();

    let mut stdout_lines: Vec<i64> = Vec::new();
    let mut stderr_lines: Vec<i64> = Vec::new();
    let mut exit_frame: Option<serde_json::Value> = None;

    // Bound the WHOLE drain: if the two forwarder tasks (or the mpsc
    // channel between them and the socket-writer loop) deadlock under
    // volume, this fires instead of the test hanging forever.
    let drain = tokio::time::timeout(std::time::Duration::from_secs(30), async {
        loop {
            let msg = ws.next().await.unwrap().unwrap();
            let v: serde_json::Value = serde_json::from_str(&msg.into_text().unwrap()).unwrap();
            if v["type"] == "exit" {
                exit_frame = Some(v);
                break;
            }
            let stream = v["stream"].as_str().unwrap();
            let text = v["text"].as_str().unwrap();
            let n: i64 = text.split_whitespace().nth(1).unwrap().parse().unwrap();
            if stream == "stdout" {
                stdout_lines.push(n);
            } else {
                stderr_lines.push(n);
            }
        }
    })
    .await;

    assert!(
        drain.is_ok(),
        "dual-stream drain did not complete within 30s — deadlock"
    );
    let exit_frame = exit_frame.expect("exit frame must have been received");
    assert_eq!(exit_frame["success"], true);
    assert_eq!(exit_frame["code"], 0);

    assert_eq!(stdout_lines.len(), 3000, "dropped/duplicated stdout lines");
    assert_eq!(stderr_lines.len(), 3000, "dropped/duplicated stderr lines");
    stdout_lines.sort_unstable();
    stderr_lines.sort_unstable();
    let expected: Vec<i64> = (1..=3000).collect();
    assert_eq!(
        stdout_lines, expected,
        "stdout sequence has a gap or a duplicate"
    );
    assert_eq!(
        stderr_lines, expected,
        "stderr sequence has a gap or a duplicate"
    );

    std::env::remove_var("OMEGA_BIN");
}

/// (1.5) Silent-child mid-stream disconnect: the P1 gap found by the
/// whole-branch live-binary review of `mid_stream_client_disconnect_...`
/// above. That test's fake nested installer keeps printing output every
/// second even AFTER the disconnect, which means `install_stream_loop`'s
/// next `send_install_frame` attempt fails and its (pre-existing)
/// send-driven disconnect handling fires — so that test proves the
/// process-group kill works, but cannot catch a loop that would otherwise
/// never even ATTEMPT a send once the client is gone.
///
/// Here the nested installer prints exactly ONE line, then goes SILENT
/// (`sleep`s) for well past this test's disconnect+wait window before
/// touching its marker — mirroring the realistic case of a real
/// `curl|sh`/`npm install` that goes quiet between progress lines, exactly
/// when a user is most likely to get impatient and cancel. A gateway loop
/// that only detects disconnection via a failed send would sit parked on
/// `rx.recv().await` for the whole silent window, never notice the client
/// closed the socket, and never kill the process group — so the marker
/// WOULD eventually appear. The fix under test (`install_stream_loop`
/// concurrently watching `socket.recv()` via `tokio::select!`) must notice
/// the client's disconnect immediately, independent of any child output.
///
/// Uses a CLEAN WebSocket close handshake (`ws.close(None)`) rather than a
/// raw drop — the realistic and sufficient case per the review brief (a
/// clean close is the harder case for a send-driven loop to ever notice at
/// all, since a raw TCP-RST at least eventually surfaces as a socket read
/// error on some platforms/timings, whereas a byte-perfect clean close
/// frame sitting unread in the kernel buffer produces no error signal to a
/// loop that never reads the socket).
#[tokio::test]
async fn silent_child_after_clean_disconnect_still_gets_killed() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("silent-orphan-still-ran.marker");
    install_fake_omega(
        dir.path(),
        &format!(
            r#"
if [ "$1" = "install" ]; then
    echo "starting"
    # Mirrors cmd_install's `Command::new("bash").args(["-c", cmd]).status()`:
    # a foreground child of THIS script, not of the gateway's own Command.
    # Prints nothing further, then goes silent (no output at all — the case
    # a send-driven-only disconnect check can never observe) well past this
    # test's disconnect+wait window before touching the marker.
    bash -c '
        sleep 5
        touch "{marker}"
    '
fi
"#,
            marker = marker.display()
        ),
    );
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let url = ws_url(&base, "/v1/agents/codex/install/stream", &token);
    let (mut ws, _) = connect_async(url).await.unwrap();

    // Read exactly the first frame ("starting"), then send a CLEAN close
    // handshake — before the nested installer's silent sleep even starts
    // counting down, and well before it would ever touch the marker.
    let first = ws.next().await.unwrap().unwrap();
    let v: serde_json::Value = serde_json::from_str(&first.into_text().unwrap()).unwrap();
    assert_eq!(v["text"], "starting");
    ws.close(None).await.unwrap();
    drop(ws);

    assert!(
        !marker.exists(),
        "marker must not exist yet — installer hasn't reached it"
    );

    // Give the server up to 8s — comfortably past the nested installer's 5s
    // silent sleep — to (a) notice the clean close via the socket-read
    // branch (no send is ever attempted here, since the child produces no
    // further output) and (b) group-kill the child. Poll rather than check
    // once instantly, exactly like the chatty-disconnect test above, so a
    // pass is never an accident of timing.
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(8);
    while tokio::time::Instant::now() < deadline {
        assert!(
            !marker.exists(),
            "the marker WAS written: the SILENT nested installer kept running to \
             completion after a clean client disconnect the gateway never noticed. \
             A send-driven-only disconnect check never attempts a send when the child \
             produces no further output, so it never fires — the loop must also read \
             the socket itself (tokio::select! on socket.recv()) to catch this."
        );
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
    assert!(
        !marker.exists(),
        "the marker WAS written at the deadline: the silent nested installer survived \
         the disconnect — the read-driven fix did not actually catch a client that \
         disconnects while the child has gone quiet."
    );

    std::env::remove_var("OMEGA_BIN");
}

/// (1.6) I-4 (Codex cross-model review, 2026-08-11): a QUIET-BUT-ALIVE child
/// (no output, client STILL CONNECTED) used to hold the WS connection and
/// subprocess open indefinitely -- the loop only ever killed on client
/// disconnect, never on its own elapsed wall-clock time.
/// `OMEGA_STREAM_TIMEOUT_SECS` overrides the default 1800s ceiling so this
/// test stays fast. The fake installer prints one line then goes silent for
/// well past the (test-overridden) timeout before touching a marker file --
/// proving the CHILD is actually killed when the outer bound fires, not
/// merely that the socket closes.
#[tokio::test]
async fn quiet_child_past_the_outer_stream_timeout_gets_killed_and_the_client_is_told() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("outlived-the-outer-timeout.marker");
    install_fake_omega(
        dir.path(),
        &format!(
            r#"
if [ "$1" = "install" ]; then
    echo "starting"
    sleep 5
    touch "{marker}"
fi
"#,
            marker = marker.display()
        ),
    );
    std::env::set_var("OMEGA_STREAM_TIMEOUT_SECS", "1");
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let url = ws_url(&base, "/v1/agents/codex/install/stream", &token);
    let (mut ws, _) = connect_async(url).await.unwrap();

    let first = ws.next().await.unwrap().unwrap();
    let v: serde_json::Value = serde_json::from_str(&first.into_text().unwrap()).unwrap();
    assert_eq!(v["text"], "starting");

    // Drain until the socket itself closes -- bounded so a regression (the
    // outer timeout never firing at all) fails the test instead of hanging
    // forever. The child's own silent sleep is 5s; the timeout is
    // overridden to 1s, so the stream must close well before 5s elapses.
    let drained = tokio::time::timeout(std::time::Duration::from_secs(4), async {
        let mut saw_error_frame = false;
        while let Some(Ok(msg)) = ws.next().await {
            if let Ok(text) = msg.into_text() {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                    if v["type"] == "error" {
                        saw_error_frame = true;
                    }
                }
            }
        }
        saw_error_frame
    })
    .await;

    assert!(
        drained.is_ok(),
        "the stream did not close within 4s -- the outer stream timeout never fired \
         even though the child was silent well past the 1s override"
    );
    assert!(
        drained.unwrap(),
        "expected an Error frame naming the timeout before the stream closed"
    );

    std::env::remove_var("OMEGA_STREAM_TIMEOUT_SECS");

    // The nested sleep touches the marker at ~5s; poll well past that to
    // prove the child (not just the WS connection) was actually killed.
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(8);
    while tokio::time::Instant::now() < deadline {
        assert!(
            !marker.exists(),
            "the marker WAS written: the child survived the outer stream-timeout kill -- \
             only the WS connection was closed, the subprocess kept running unsupervised."
        );
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
    assert!(!marker.exists(), "the marker WAS written at the deadline");

    std::env::remove_var("OMEGA_BIN");
}

/// (2) Mid-stream client disconnect: proves (or falsifies) the claim that
/// killing on client-disconnect actually stops the real work, including
/// the nested process the direct child itself spawns.
///
/// The fake `omega install` here has the SAME process shape as the real
/// `omega-cli::cmd_install`: the top-level process (the one the gateway's
/// `Command::new(omega_bin())` actually spawns, and the one whose PID the
/// gateway directly controls) runs a NESTED `bash -c '<installer>'` in the
/// foreground (mirroring `cmd_install`'s `Command::new("bash").args(["-c",
/// cmd]).status()`), which is a process-tree CHILD OF THE FAKE OMEGA
/// SCRIPT, not of the gateway. That nested process prints a few more lines
/// (so the gateway's next post-disconnect send attempt fails and triggers
/// the kill path) and then, well after the kill should have landed, would
/// write a marker file.
///
/// `install_stream_loop` spawns the fake `omega` into its own process group
/// (`process_group(0)`) and, on disconnect, kills the WHOLE group
/// (`kill_process_group`, a `kill -- -<pid>` negative-PID kill) rather than
/// only the direct PID. Because the nested `bash -c` inherits that same
/// group, the group-kill reaches it too, so the marker must NEVER appear —
/// proving a disconnect now actually aborts the whole install tree, not
/// just the outer wrapper.
#[tokio::test]
async fn mid_stream_client_disconnect_kills_the_nested_installer_too() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("orphan-still-ran.marker");
    install_fake_omega(
        dir.path(),
        &format!(
            r#"
if [ "$1" = "install" ]; then
    echo "starting"
    # Mirrors cmd_install's `Command::new("bash").args(["-c", cmd]).status()`:
    # a foreground child of THIS script, not of the gateway's own Command.
    bash -c '
        for i in 1 2 3; do
            sleep 1
            echo "installer progress $i"
        done
        touch "{marker}"
    '
fi
"#,
            marker = marker.display()
        ),
    );
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let url = ws_url(&base, "/v1/agents/codex/install/stream", &token);
    let (mut ws, _) = connect_async(url).await.unwrap();

    // Read exactly the first frame ("starting"), then disconnect immediately
    // — before any of the nested installer's own output arrives. This is
    // the "client closes mid-stream" scenario from the review brief.
    let first = ws.next().await.unwrap().unwrap();
    let v: serde_json::Value = serde_json::from_str(&first.into_text().unwrap()).unwrap();
    assert_eq!(v["text"], "starting");
    drop(ws); // client-side disconnect

    assert!(
        !marker.exists(),
        "marker must not exist yet — installer hasn't reached it"
    );

    // Give the server up to 8s to (a) notice the disconnect on its next
    // send attempt (triggered by "installer progress 1" at ~1s) and (b)
    // group-kill the child. The nested installer's OWN loop needs ~3s to
    // finish and touch the marker if it is NOT actually killed, so poll for
    // up to that same ~8s bound rather than checking once instantly (which
    // could pass by accident before the nested process even had a chance to
    // run) — a bounded wait that only PASSES once we're confident the
    // marker will never appear, not merely that it hasn't appeared yet.
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(8);
    while tokio::time::Instant::now() < deadline {
        assert!(
            !marker.exists(),
            "the marker WAS written: the nested installer process kept running to \
             completion after the gateway's disconnect-kill fired. Killing only the \
             direct child (the omega wrapper) does not reach a nested `bash -c \
             <install_command>` process the wrapper itself spawned, exactly as \
             omega-cli::cmd_install's real Command::new(\"bash\")...status() does — \
             the group-kill (process_group(0) + kill -- -<pid>) must reach it too."
        );
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
    assert!(
        !marker.exists(),
        "the marker WAS written at the deadline: the nested installer survived the \
         disconnect-triggered group-kill — the fix did not actually stop the real work."
    );

    std::env::remove_var("OMEGA_BIN");
}

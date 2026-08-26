//! `GET /v1/agents` — the dispatch-target agent roster, from
//! `omega_core::agents::Agent::all()`. Pure, synchronous, side-effect-free
//! data apart from `is_available()`'s per-agent PATH lookup (cheap, ~8
//! calls, no network) — same `spawn_blocking` idiom `routes_rules::list`
//! uses, kept off the async runtime thread.
//!
//! Also `POST /v1/agents/{name}/install` (a pure pre-flight validation
//! check) and `GET /v1/agents/{name}/install/stream` (the WebSocket that
//! actually runs `omega install <name>` and streams its output) — see each
//! function's doc comment.

use crate::protocol::{
    AgentEntry, AgentInstallCheckResponse, AgentInstallStreamMsg, AgentsResponse,
};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use omega_core::agents::Agent;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

pub async fn list() -> Json<AgentsResponse> {
    let response = tokio::task::spawn_blocking(|| {
        let agents = omega_core::agents::Agent::all()
            .iter()
            .map(|a| AgentEntry {
                name: a.name().to_string(),
                display_name: a.display_name().to_string(),
                available: a.is_available(),
            })
            .collect();
        AgentsResponse { agents }
    })
    .await
    .unwrap_or(AgentsResponse { agents: vec![] });
    Json(response)
}

/// Resolves `{name}` (case-insensitive, via `Agent::from_name`) into an
/// `Agent` that `omega install` actually supports, or the `(StatusCode,
/// message)` to reject with. Shared by [`install_check`] (C1) and
/// [`install_stream`] (C2) so both endpoints reject the same unknown/
/// uninstallable name the same way, for the same reason, with the same
/// message. `install_command().is_some()` is `None` for exactly `Kimi` and
/// `Shell` (`crates/omega-core/src/agents.rs`) — no curl|sh/npm one-liner we
/// vouch for.
fn resolve_installable_agent(name: &str) -> Result<Agent, (StatusCode, String)> {
    let agent = Agent::from_name(name)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, format!("unknown agent: {name}")))?;
    if agent.install_command().is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "{} has no installer — omega install does not support {}",
                agent.display_name(),
                agent.name()
            ),
        ));
    }
    Ok(agent)
}

/// `POST /v1/agents/{name}/install` — a THIN, synchronous pre-flight check.
/// It validates `{name}` (unknown -> 400; installable check -> 400 for
/// `shell`, the only agent with no `install_command`) and
/// reports `already_available`, but it spawns NOTHING: the real install
/// runs only over the `install/stream` WebSocket (C2), because `omega
/// install` shells out to `curl|sh`/`npm install`, which can run for tens of
/// seconds — far too long to hold open a plain synchronous POST. This route
/// is the app's pre-flight call before it opens that WebSocket.
pub async fn install_check(
    Path(name): Path<String>,
) -> Result<Json<AgentInstallCheckResponse>, (StatusCode, Json<serde_json::Value>)> {
    let agent = resolve_installable_agent(&name)
        .map_err(|(code, msg)| (code, Json(serde_json::json!({ "error": msg }))))?;

    // is_available() only does a PATH/well-known-path lookup (no subprocess,
    // no network) — same cost `list` already pays per-agent above — but
    // keep it off the async runtime thread anyway, matching `list`'s idiom.
    let response = tokio::task::spawn_blocking(move || AgentInstallCheckResponse {
        agent: agent.name().to_string(),
        display_name: agent.display_name().to_string(),
        already_available: agent.is_available(),
    })
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("install_check task panicked: {e}") })),
        )
    })?;

    Ok(Json(response))
}

/// `GET /v1/agents/{name}/install/stream` — validates `{name}` with the
/// SAME two checks as [`install_check`] BEFORE ever upgrading the
/// connection or spawning anything: on a bad name this returns
/// `StatusCode::BAD_REQUEST.into_response()` straight from this non-async
/// wrapper, mirroring `routes_accounts::login`'s exact shape (a WS route
/// this file was told to copy for the WS-with-a-bad-path-param
/// discipline) rather than upgrading and then erroring inside the loop.
///
/// On success, `install_stream_loop` spawns `omega install <name>` (via
/// `omega_cli::omega_bin()`, respecting `OMEGA_BIN`) and streams its
/// stdout/stderr lines, then a final exit frame.
pub async fn install_stream(ws: WebSocketUpgrade, Path(name): Path<String>) -> Response {
    match resolve_installable_agent(&name) {
        Ok(agent) => ws.on_upgrade(move |socket| install_stream_loop(socket, agent)),
        Err((code, _msg)) => code.into_response(),
    }
}

/// Serializes and sends one install-stream frame. `Err` means the socket is
/// dead.
async fn send_install_frame(
    socket: &mut WebSocket,
    frame: &AgentInstallStreamMsg,
) -> Result<(), axum::Error> {
    let text = serde_json::to_string(frame).expect("serialize AgentInstallStreamMsg");
    socket.send(Message::Text(text.into())).await
}

/// Reads `reader` line-by-line (tokio's `AsyncBufReadExt`, same idiom as
/// `chat_driver::run_turn`'s stdout loop) and forwards each line as a
/// `Line{stream: stream_name, ..}` frame on `tx`. Returns on EOF, a read
/// error, or the receiver having been dropped (the loop below stopped
/// forwarding, e.g. because the client socket died).
///
/// `stdout` and `stderr` are each read by their OWN spawned instance of this
/// function (see `install_stream_loop`) rather than sequentially, so
/// neither pipe can starve the other: a child that interleaves output
/// across both streams, with one OS pipe buffer filling while the loop is
/// blocked reading the other, would otherwise stall or reorder badly.
async fn forward_lines<R: AsyncRead + Unpin>(
    reader: R,
    stream_name: &'static str,
    tx: mpsc::Sender<AgentInstallStreamMsg>,
) {
    let mut lines = BufReader::new(reader).lines();
    loop {
        match lines.next_line().await {
            Ok(Some(text)) => {
                let frame = AgentInstallStreamMsg::Line {
                    stream: stream_name.to_string(),
                    text,
                };
                if tx.send(frame).await.is_err() {
                    return;
                }
            }
            // EOF or a read error both mean this pipe is done.
            Ok(None) | Err(_) => return,
        }
    }
}

/// Sends `SIGKILL` to the WHOLE process group rooted at `pid` — the
/// negative-PID `kill` idiom (`kill -- -<pid>` targets every process whose
/// PGID equals `pid`, not just `pid` itself). `pid` must be the PID of a
/// process spawned with `process_group(0)` (see [`install_stream_loop`]),
/// so its PGID equals its own PID and this reaches both that process AND
/// every child it forked that did not itself call `setpgid` — in
/// particular the nested `bash -c <install_command>` `omega install`
/// actually runs the real work in.
///
/// Shells out to the `kill` binary instead of adding a `libc`/`nix`
/// dependency: neither crate is anywhere in this workspace today (checked
/// `Cargo.lock`), this is a Linux-only VPS deployment where `kill` is
/// always present, and one more `Command::new` keeps this fix the same
/// shape as every other subprocess call in this crate
/// (`omega_cli::run`/`rmux::run`) instead of introducing a new
/// unsafe-FFI-adjacent dependency for a single call site. Run off the async
/// runtime thread via `spawn_blocking`, same idiom the rest of this crate
/// uses for any blocking subprocess call.
async fn kill_process_group(pid: u32) {
    let _ = tokio::task::spawn_blocking(move || {
        std::process::Command::new("kill")
            .arg("--")
            .arg(format!("-{pid}"))
            .status()
    })
    .await;
}

/// Shared disconnect-cleanup, called from BOTH places [`install_stream_loop`]
/// can learn the client is gone: a failed send on the mpsc-forwarding branch
/// (the pre-existing belt-and-suspenders path — still fires when the child
/// is chatty enough to hit a dead socket) and the socket-read branch (the
/// fix this function exists for — catches a disconnect even when the child
/// has gone quiet and no send would ever be attempted). Drops `rx` FIRST so
/// any forwarder currently blocked on a full channel gets an immediate send
/// error and returns, instead of `stdout_task`/`stderr_task` below
/// deadlocking on a channel nobody is draining anymore, then kills the WHOLE
/// process group (see [`kill_process_group`]'s doc comment) rather than just
/// the direct `omega` PID, falling back to the single-PID `child.kill()`
/// only in the near-impossible case `child.id()` already returned `None`.
async fn kill_and_drain(
    rx: mpsc::Receiver<AgentInstallStreamMsg>,
    mut child: tokio::process::Child,
    child_pid: Option<u32>,
    stdout_task: tokio::task::JoinHandle<()>,
    stderr_task: tokio::task::JoinHandle<()>,
) {
    drop(rx);
    match child_pid {
        Some(pid) => {
            kill_process_group(pid).await;
            let _ = child.wait().await;
        }
        None => {
            let _ = child.kill().await;
        }
    }
    let _ = stdout_task.await;
    let _ = stderr_task.await;
}

/// Runs `omega install <agent.name()>` and streams its output to `socket`:
/// every stdout/stderr line as a `Line` frame (in the order EACH stream
/// produced them; the interleaving ORDER BETWEEN the two streams is not
/// guaranteed, since they're read by two concurrently-polled tasks feeding
/// one channel — see [`forward_lines`]), then a final `Exit` frame carrying
/// the real exit status.
///
/// The spawned `omega` process is placed into its OWN process group
/// (`process_group(0)`, the standard "detach into a new group" idiom: the
/// group ID becomes the child's own PID) rather than inheriting the
/// gateway's. This matters because `omega install <name>`'s real work
/// happens one level deeper: `omega_cli::cmd_install` runs
/// `Command::new("bash").args(["-c", cmd]).status()` — a foreground child OF
/// THE OMEGA PROCESS, not of the gateway. On POSIX, killing a single PID
/// does NOT propagate to that PID's own children, so a plain
/// `Child::kill()` (which signals only the direct `omega` PID) kills the
/// wrapper and ORPHANS the real `curl|sh`/`npm install` subprocess, which
/// then keeps running unsupervised even though the client believes the
/// install was cancelled. Putting `omega` in its own process group lets a
/// disconnect kill the WHOLE group at once — see [`kill_process_group`] —
/// which reaches the nested `bash -c` too, since a child inherits its
/// parent's process group unless it changes its own.
///
/// `kill_on_drop(true)` on the `Command` (the same mechanism
/// `chat_driver::run_turn` relies on, not a custom guard type) is enough to
/// guarantee the DIRECT child is reaped if this future is ever dropped
/// mid-stream without reaching the explicit disconnect-kill path below
/// (e.g. the gateway process itself panics or exits) — its own `Drop` does
/// not kill by default, `kill_on_drop` is what makes it do so. That
/// fallback stays single-PID (tokio's `Drop` impl has no group-aware
/// variant), so it does not fully close the same hole on that much rarer
/// path; the deliberate-disconnect path this function owns is the one
/// [`kill_process_group`] actually fixes.
async fn install_stream_loop(mut socket: WebSocket, agent: Agent) {
    let mut cmd = Command::new(crate::omega_cli::omega_bin());
    cmd.arg("install").arg(agent.name());
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.kill_on_drop(true);
    cmd.process_group(0);

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => {
            let _ = send_install_frame(
                &mut socket,
                &AgentInstallStreamMsg::Error {
                    message: format!("failed to spawn omega: {e}"),
                },
            )
            .await;
            let _ = socket.send(Message::Close(None)).await;
            return;
        }
    };
    // Captured now, before any consuming/`.await`ing method (`kill`/`wait`)
    // is ever called on `child`: `Child::id()` returns `None` once the
    // child has been polled to completion (POSIX PID reuse guard), so this
    // is the only point in the function guaranteed to observe it.
    let child_pid = child.id();

    let stdout = child.stdout.take().expect("stdout was piped");
    let stderr = child.stderr.take().expect("stderr was piped");

    let (tx, mut rx) = mpsc::channel::<AgentInstallStreamMsg>(64);
    let stdout_task = tokio::spawn(forward_lines(stdout, "stdout", tx.clone()));
    let stderr_task = tokio::spawn(forward_lines(stderr, "stderr", tx.clone()));
    // Drop our own sender so the channel closes once BOTH forwarder tasks
    // have dropped their clones (i.e. both pipes hit EOF), not before.
    drop(tx);

    // I-4 (Codex cross-model review, 2026-08-11): an outer wall-clock bound
    // on the WHOLE connection lifetime, on top of the disconnect detection
    // below — a QUIET-BUT-ALIVE child (no output, client still connected)
    // used to hold this stream and its subprocess open indefinitely, since
    // every other branch only fires on a disconnect or a forwarded frame.
    // `tokio::time::sleep` pinned once before the loop so the SAME timer
    // (not a fresh one per iteration) is polled by every `select!` round —
    // see `omega_cli::stream_timeout`'s doc comment for the default/env var.
    let sleep = tokio::time::sleep(crate::omega_cli::stream_timeout());
    tokio::pin!(sleep);

    // Read-driven disconnect detection: watch BOTH the internal mpsc channel
    // (frames to forward) AND the socket itself (the client's side of the
    // connection) concurrently. Send-failure alone is NOT enough — it only
    // fires as a side effect of attempting to forward a frame, so a child
    // that goes quiet after the client disconnects (the common, realistic
    // case for a real `curl|sh`/`npm install` between progress lines) would
    // otherwise leave this loop parked on `rx.recv().await` forever, never
    // noticing the client is gone and never killing the process group. The
    // socket-read branch below fixes that: it drains inbound messages so a
    // clean WS close frame, a socket error, or the stream simply ending
    // (TCP-RST-style) are all noticed WITHOUT depending on any outbound send
    // ever being attempted.
    loop {
        tokio::select! {
            _ = &mut sleep => {
                let _ = send_install_frame(
                    &mut socket,
                    &AgentInstallStreamMsg::Error {
                        message: format!(
                            "install timed out after {}s and was killed",
                            crate::omega_cli::stream_timeout().as_secs()
                        ),
                    },
                )
                .await;
                kill_and_drain(rx, child, child_pid, stdout_task, stderr_task).await;
                let _ = socket.send(Message::Close(None)).await;
                return;
            }
            frame = rx.recv() => {
                match frame {
                    Some(frame) => {
                        if send_install_frame(&mut socket, &frame).await.is_err() {
                            // Belt-and-suspenders: still handle the
                            // send-failure case exactly as before (a chatty
                            // child that keeps producing output after the
                            // client is gone will hit this before the
                            // socket-read branch below even fires).
                            kill_and_drain(rx, child, child_pid, stdout_task, stderr_task).await;
                            return;
                        }
                    }
                    // Both forwarders finished (their tx clones dropped ->
                    // both pipes hit EOF): fall through to the post-loop
                    // "wait on child, send Exit frame" tail below.
                    None => break,
                }
            }
            msg = socket.recv() => {
                match msg {
                    // Stream ended, a socket error, or the client's own
                    // clean close frame: this IS the client disconnecting,
                    // regardless of whether the child has produced (or will
                    // ever produce) any more output for us to try sending.
                    None | Some(Err(_)) | Some(Ok(Message::Close(_))) => {
                        kill_and_drain(rx, child, child_pid, stdout_task, stderr_task).await;
                        return;
                    }
                    // Any other inbound message (Ping/Pong/Text/Binary) —
                    // this endpoint is server-push-only, the client isn't
                    // expected to send app-level messages, but the socket
                    // still needs draining to actually observe its
                    // lifecycle. Not a disconnect: keep looping.
                    Some(Ok(_)) => {}
                }
            }
        }
    }

    // Both forwarders finished (their tx clones dropped -> both pipes hit
    // EOF), so the exit status is available immediately or very shortly.
    let _ = stdout_task.await;
    let _ = stderr_task.await;

    let exit_frame = match child.wait().await {
        Ok(status) => AgentInstallStreamMsg::Exit {
            success: status.success(),
            code: status.code(),
        },
        Err(e) => AgentInstallStreamMsg::Error {
            message: format!("failed to wait on child: {e}"),
        },
    };
    let _ = send_install_frame(&mut socket, &exit_frame).await;
    let _ = socket.send(Message::Close(None)).await;
}

#[cfg(test)]
mod resolve_installable_agent_tests {
    use super::resolve_installable_agent;

    #[test]
    fn accepts_a_real_installable_agent_case_insensitively() {
        let agent = resolve_installable_agent("Claude").unwrap();
        assert_eq!(agent.name(), "claude");
    }

    #[test]
    fn rejects_unknown_agent_name() {
        let err = resolve_installable_agent("not-a-real-agent").unwrap_err();
        assert_eq!(err.0, axum::http::StatusCode::BAD_REQUEST);
    }

    /// Kimi shipped a real installer (`code.kimi.com/kimi-code/install.sh`) and
    /// gained an `install_command`, so it is now accepted like any other agent.
    /// This test used to assert the opposite and outlived the fact.
    #[test]
    fn accepts_kimi_now_that_it_ships_an_installer() {
        let agent = resolve_installable_agent("kimi").unwrap();
        assert_eq!(agent.name(), "kimi");
    }

    #[test]
    fn rejects_shell_with_a_clear_message() {
        let err = resolve_installable_agent("shell").unwrap_err();
        assert_eq!(err.0, axum::http::StatusCode::BAD_REQUEST);
        assert!(
            err.1.contains("shell"),
            "message should name the agent: {}",
            err.1
        );
    }
}

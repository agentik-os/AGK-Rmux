//! `GET /v1/orchestrate/stream?project=&mission=&agent=` — runs `omega
//! orchestrate <project> <mission>` end-to-end (classify -> plan -> dispatch
//! -> monitor -> gate) and streams its output over a WebSocket.
//!
//! DEVIATION FROM THE LITERAL TASK BRIEF (`POST /v1/orchestrate`), already
//! recorded in `.superpowers/sdd/progress.md`'s Task B ground-truth section:
//! a WebSocket upgrade is GET-only per spec (a browser refuses to upgrade a
//! POST), so this ships as `GET /v1/orchestrate/stream`, mirroring
//! `routes_audit.rs`'s `check`/`stream` pair 1:1 (pre-upgrade rejection on a
//! bad param, the same disconnect-safe process-group-kill loop) —
//! deliberately NOT sharing code with `routes_audit.rs::audit_stream_loop`,
//! for the exact reason its own doc comment gives: duplicating a ~150-line
//! carefully-commented state machine is cheaper and safer than forcing a
//! shared abstraction across two files under separate scope rules (R-SCOPE)
//! in the same wave.
//!
//! SECOND DEVIATION, found while implementing (new — not previously recorded
//! in progress.md): `omega orchestrate` genuinely has NO `--agent` flag.
//! Confirmed at `crates/omega-cli/src/main.rs`'s `Commands::Orchestrate`
//! variant (`project`, `mission`, `--dir`, `--timeout`, `--no-gate` only,
//! ~line 420) and `cmd_orchestrate`'s body (~line 6038), which builds an
//! `Orchestrator` from `OmegaConfig` alone — the agent it uses comes from
//! `config.agent_command` (the box's configured default runtime, read deep
//! inside `orchestration.rs`), never a per-call override. The `?agent=`
//! query param is still accepted (it is part of the endpoint signature
//! already recorded in progress.md) and still VALIDATED against
//! `omega_core::agents::Agent::all()` before any spawn — an unknown/typo'd
//! name is a clean pre-upgrade 400, the same posture `routes_dispatch.rs`
//! takes — but it is NEVER forwarded into the `omega orchestrate` argv:
//! there is no flag to forward it to. This is an honest, pre-existing
//! capability gap in what `omega orchestrate` itself supports, not a defect
//! in this endpoint — the same posture Task A took on the AISB local-inbox
//! wiring gap.
//!
//! TIMEOUT: never passed. `omega orchestrate`'s own `--timeout` defaults to
//! 3600s (see `Commands::Orchestrate`); this endpoint deliberately never
//! overrides it, per this task's brief — a client wanting a shorter bound
//! tracks wall-clock itself and disconnects, exactly like every other
//! WS-stream endpoint in this crate already lets a client do.
//!
//! `--dir <project_path>` IS passed, using the SAME server-resolved project
//! root `resolve_project_path` below validates (never a client-supplied
//! path) — `cmd_orchestrate` otherwise defaults to the gatewayd process's
//! own (arbitrary) current directory when `--dir` is omitted, which would be
//! wrong for a long-running daemon; explicitly pointing it at the real
//! project root is the correct, safe choice, mirroring
//! `routes_audit.rs::audit_stream_loop`'s identical `--dir` usage.
//!
//! CONCURRENCY: unlike `routes_audit::stream` (fast, side-effect-free per
//! its own doc comment) this spawns the heaviest, longest-running, most
//! state-mutating operation exposed by this whole crate — a REAL oracle,
//! real workers, a real quality gate. Capped separately via
//! `AppState::orchestrate_permits` (see `server.rs`), lower than
//! `dispatch_permits`, one permit held for the WHOLE connection lifetime —
//! the same "heavier and longer-held than a single chat turn" reasoning
//! `master_chat_permits` documents.
//!
//! NEVER RUN FOR REAL IN TESTS (R-SEC / this task's explicit brief): every
//! test in `tests/orchestrate_test.rs` points `OMEGA_BIN` at a fake script.
//! Nothing in this crate's test suite or live-verify ever invokes a real
//! `omega orchestrate`, which dispatches an actual oracle end-to-end.

use crate::protocol::OrchestrateStreamMsg;
use crate::server::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::Command;
use tokio::sync::{mpsc, OwnedSemaphorePermit};

/// Byte-length cap on `mission`, mirroring `routes_dispatch.rs::
/// MAX_MISSION_LEN` (duplicated rather than shared — see this module's doc
/// comment on why routes in this crate don't share stream-loop machinery).
const MAX_MISSION_LEN: usize = 8000;

/// Resolves `project` against the discovered-project allowlist — the exact
/// pattern `routes_audit.rs::resolve_project_path` / `routes_dispatch.rs`
/// use: no filesystem access beyond this read-only `discover` walk happens
/// for an unknown project, and the real on-disk root is resolved
/// SERVER-SIDE, never trusted from the client.
async fn resolve_project_path(project: &str) -> Result<PathBuf, (StatusCode, String)> {
    let name = project.to_string();
    let found = tokio::task::spawn_blocking(move || {
        let home = crate::config::home_dir();
        omega_core::projects::discover(&home)
            .into_iter()
            .find(|p| p.name == name)
            .map(|p| p.path)
    })
    .await
    .unwrap_or(None);
    found.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            format!("unknown project: {project}"),
        )
    })
}

/// Validates `project`/`mission`/`agent` and resolves the real project root,
/// BEFORE any upgrade or spawn — see this module's doc comment for why
/// `agent` is validated but never forwarded.
async fn resolve_orchestrate_request(
    project: &str,
    mission: &str,
    agent: &str,
) -> Result<PathBuf, (StatusCode, String)> {
    if project.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "project must not be empty".to_string(),
        ));
    }
    if mission.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "mission must not be empty".to_string(),
        ));
    }
    if project.contains('\0') || mission.contains('\0') {
        return Err((
            StatusCode::BAD_REQUEST,
            "project/mission must not contain a NUL byte".to_string(),
        ));
    }
    if mission.len() > MAX_MISSION_LEN {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("mission too long (max {MAX_MISSION_LEN} bytes)"),
        ));
    }
    if !agent.is_empty() {
        let name = agent.to_string();
        let is_known = tokio::task::spawn_blocking(move || {
            omega_core::agents::Agent::all()
                .iter()
                .any(|a| a.name() == name)
        })
        .await
        .unwrap_or(false);
        if !is_known {
            return Err((StatusCode::BAD_REQUEST, format!("unknown agent: {agent}")));
        }
    }
    resolve_project_path(project).await
}

/// `GET /v1/orchestrate/stream?project=&mission=&agent=` — validates
/// BEFORE ever upgrading (a bad param returns a plain
/// `StatusCode::BAD_REQUEST.into_response()`, mirroring
/// `routes_audit::stream`'s exact shape), then acquires an
/// `orchestrate_permits` permit BEFORE upgrading (mirrors
/// `routes_master::chat`'s reject-before-upgrade shape — a bare 429 on
/// exhaustion, never an upgrade followed by an in-loop rejection).
pub async fn stream(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    let project = query.get("project").cloned().unwrap_or_default();
    let mission = query.get("mission").cloned().unwrap_or_default();
    let agent = query.get("agent").cloned().unwrap_or_default();

    match resolve_orchestrate_request(&project, &mission, &agent).await {
        Ok(project_path) => {
            let Ok(permit) = state.orchestrate_permits.clone().try_acquire_owned() else {
                return StatusCode::TOO_MANY_REQUESTS.into_response();
            };
            ws.on_upgrade(move |socket| {
                orchestrate_stream_loop(socket, project, mission, project_path, permit)
            })
        }
        Err((code, _msg)) => code.into_response(),
    }
}

/// Serializes and sends one orchestrate-stream frame. `Err` means the socket
/// is dead.
async fn send_orchestrate_frame(
    socket: &mut WebSocket,
    frame: &OrchestrateStreamMsg,
) -> Result<(), axum::Error> {
    let text = serde_json::to_string(frame).expect("serialize OrchestrateStreamMsg");
    socket.send(Message::Text(text.into())).await
}

/// Reads `reader` line-by-line and forwards each line as a `Line{stream:
/// stream_name, ..}` frame on `tx`. Returns on EOF, a read error, or the
/// receiver having been dropped. Identical shape to
/// `routes_audit::forward_lines` — `stdout` and `stderr` are each read by
/// their OWN spawned instance of this function (see
/// [`orchestrate_stream_loop`]) so neither pipe can starve the other.
async fn forward_lines<R: AsyncRead + Unpin>(
    reader: R,
    stream_name: &'static str,
    tx: mpsc::Sender<OrchestrateStreamMsg>,
) {
    let mut lines = BufReader::new(reader).lines();
    loop {
        match lines.next_line().await {
            Ok(Some(text)) => {
                let frame = OrchestrateStreamMsg::Line {
                    stream: stream_name.to_string(),
                    text,
                };
                if tx.send(frame).await.is_err() {
                    return;
                }
            }
            Ok(None) | Err(_) => return,
        }
    }
}

/// Sends `SIGKILL` to the WHOLE process group rooted at `pid` — identical to
/// `routes_audit::kill_process_group` (same negative-PID `kill -- -<pid>`
/// idiom, same `spawn_blocking`, same rationale: `pid` must be the PID of a
/// process spawned with `process_group(0)`, so this reaches both it and
/// every nested child it forked, in particular `omega`'s own nested
/// subprocess work).
async fn kill_process_group(pid: u32) {
    let _ = tokio::task::spawn_blocking(move || {
        std::process::Command::new("kill")
            .arg("--")
            .arg(format!("-{pid}"))
            .status()
    })
    .await;
}

/// Shared disconnect-cleanup, called from BOTH places
/// [`orchestrate_stream_loop`] can learn the client is gone — identical
/// structure to `routes_audit::kill_and_drain`.
async fn kill_and_drain(
    rx: mpsc::Receiver<OrchestrateStreamMsg>,
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

/// Runs `omega orchestrate --dir <project_path> -- <project> <mission>` and
/// streams its output to `socket`. Deliberately NOT shared code with
/// `routes_audit::audit_stream_loop` — see this module's doc comment.
///
/// The spawned `omega` process is placed into its OWN process group
/// (`process_group(0)`) for the same reason `audit_stream_loop` documents at
/// length: `omega orchestrate`'s own real work runs nested foreground
/// children (dispatched oracle/worker sessions), and a plain single-PID
/// `kill()` would orphan them rather than stop them. `kill_on_drop(true)`
/// guarantees the DIRECT child is reaped even if this future is ever dropped
/// without reaching the explicit disconnect-kill path.
///
/// The `tokio::select!` loop watches BOTH the internal mpsc frame channel
/// AND the socket's own read side, so a disconnect is noticed even when the
/// child has gone quiet — the same read-driven fix `audit_stream_loop`
/// carries.
///
/// `_permit` is held for the WHOLE lifetime of this function and released
/// automatically when it completes, whatever the reason.
async fn orchestrate_stream_loop(
    mut socket: WebSocket,
    project: String,
    mission: String,
    project_path: PathBuf,
    _permit: OwnedSemaphorePermit,
) {
    let mut cmd = Command::new(crate::omega_cli::omega_bin());
    // Flags first, then `--`, then the two client-supplied positionals LAST
    // — `project`/`mission` could themselves start with `-`, and everything
    // after `--` is treated as a positional value by clap even then (same
    // clap-safe argv construction `routes_dispatch.rs::create` documents).
    cmd.arg("orchestrate")
        .arg("--dir")
        .arg(&project_path)
        .arg("--")
        .arg(&project)
        .arg(&mission);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.kill_on_drop(true);
    cmd.process_group(0);

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => {
            let _ = send_orchestrate_frame(
                &mut socket,
                &OrchestrateStreamMsg::Error {
                    message: format!("failed to spawn omega: {e}"),
                },
            )
            .await;
            let _ = socket.send(Message::Close(None)).await;
            return;
        }
    };
    // Captured now, before any consuming/`.await`ing method (`kill`/`wait`)
    // is ever called on `child` — `Child::id()` returns `None` once the
    // child has been polled to completion.
    let child_pid = child.id();

    let stdout = child.stdout.take().expect("stdout was piped");
    let stderr = child.stderr.take().expect("stderr was piped");

    let (tx, mut rx) = mpsc::channel::<OrchestrateStreamMsg>(64);
    let stdout_task = tokio::spawn(forward_lines(stdout, "stdout", tx.clone()));
    let stderr_task = tokio::spawn(forward_lines(stderr, "stderr", tx.clone()));
    drop(tx);

    // I-4 (Codex cross-model review, 2026-08-11): an outer wall-clock bound
    // on the WHOLE connection lifetime — see `routes_agents.rs::
    // install_stream_loop`'s identical comment and
    // `omega_cli::stream_timeout`'s doc comment for the default/env var. A
    // QUIET-BUT-ALIVE child (no output, client still connected) used to
    // hold this stream and its subprocess open indefinitely.
    //
    // KNOWN TRADEOFF (flagged, not resolved, by this fix): `stream_timeout`'s
    // 1800s default is SHORTER than `omega orchestrate`'s own internal
    // `--timeout` default (3600s, this module's own doc comment) — a real,
    // legitimately slow-but-still-working orchestrate mission that produces
    // no output for 30 minutes would now be cut off by the GATEWAY before
    // the CLI's own timeout ever fires. `OMEGA_STREAM_TIMEOUT_SECS` is the
    // operator's override for this specific endpoint's deployment if 1800s
    // proves too short in practice; a per-endpoint default was deliberately
    // not introduced here to keep one shared, consistent constant across
    // every WS-stream route in this crate (see `omega_cli::stream_timeout`).
    let sleep = tokio::time::sleep(crate::omega_cli::stream_timeout());
    tokio::pin!(sleep);

    loop {
        tokio::select! {
            _ = &mut sleep => {
                let _ = send_orchestrate_frame(
                    &mut socket,
                    &OrchestrateStreamMsg::Error {
                        message: format!(
                            "orchestrate timed out after {}s and was killed",
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
                        if send_orchestrate_frame(&mut socket, &frame).await.is_err() {
                            kill_and_drain(rx, child, child_pid, stdout_task, stderr_task).await;
                            return;
                        }
                    }
                    None => break,
                }
            }
            msg = socket.recv() => {
                match msg {
                    None | Some(Err(_)) | Some(Ok(Message::Close(_))) => {
                        kill_and_drain(rx, child, child_pid, stdout_task, stderr_task).await;
                        return;
                    }
                    Some(Ok(_)) => {}
                }
            }
        }
    }

    let _ = stdout_task.await;
    let _ = stderr_task.await;

    let exit_frame = match child.wait().await {
        Ok(status) => OrchestrateStreamMsg::Exit {
            success: status.success(),
            code: status.code(),
        },
        Err(e) => OrchestrateStreamMsg::Error {
            message: format!("failed to wait on child: {e}"),
        },
    };
    let _ = send_orchestrate_frame(&mut socket, &exit_frame).await;
    let _ = socket.send(Message::Close(None)).await;
}

#[cfg(test)]
mod resolve_orchestrate_request_tests {
    use super::resolve_orchestrate_request;
    use axum::http::StatusCode;

    #[tokio::test]
    async fn rejects_empty_project() {
        let err = resolve_orchestrate_request("", "mission", "")
            .await
            .unwrap_err();
        assert_eq!(err.0, StatusCode::BAD_REQUEST);
        assert!(err.1.contains("project"));
    }

    #[tokio::test]
    async fn rejects_empty_mission() {
        let err = resolve_orchestrate_request("SomeProj", "", "")
            .await
            .unwrap_err();
        assert_eq!(err.0, StatusCode::BAD_REQUEST);
        assert!(err.1.contains("mission"));
    }

    #[tokio::test]
    async fn rejects_nul_byte() {
        let err = resolve_orchestrate_request("Some\u{0}Proj", "mission", "")
            .await
            .unwrap_err();
        assert_eq!(err.0, StatusCode::BAD_REQUEST);
        assert!(err.1.contains("NUL"));
    }

    #[tokio::test]
    async fn rejects_mission_over_length_cap() {
        let too_long = "x".repeat(8001);
        let err = resolve_orchestrate_request("SomeProj", &too_long, "")
            .await
            .unwrap_err();
        assert_eq!(err.0, StatusCode::BAD_REQUEST);
        assert!(err.1.contains("too long"));
    }
}

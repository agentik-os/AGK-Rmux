//! `GET /v1/audits` — the Quality Arsenal catalog, from
//! `omega_core::audit::all_audits()`. Pure, synchronous, in-process registry
//! (parsed once from an embedded TOML, cached in a `OnceLock`) — same
//! `spawn_blocking` idiom `routes_rules::list` uses, kept off the async
//! runtime thread even though this call is cheap/synchronous, for
//! consistency.
//!
//! Also `POST /v1/audit` (a pure pre-flight validation check) and `GET
//! /v1/audit/stream` (the WebSocket that actually runs `omega audit run
//! <kind> --dir <project_path>` and streams its output) — see each
//! function's doc comment.
//!
//! GROUND TRUTH (confirmed live against this box, not guessed): `omega
//! audit run <id> --dir <dir>` does NOT itself run a multi-phase audit — it
//! is fast and side-effect-free, printing the audit's own metadata
//! (name/phases/max score) plus a `omega spawn-worker ...` command line an
//! operator would run separately to actually dispatch the real audit
//! worker, then exits. This endpoint's streaming infrastructure is real and
//! correctly built regardless; it simply streams whatever the underlying
//! command prints.
//!
//! WIRE-PROTOCOL PATTERN, mirroring `routes_agents.rs`'s agent-install pair
//! 1:1 (same disconnect-safety contract, deliberately NOT shared code — see
//! `audit_stream_loop`'s doc comment): `check` mirrors `install_check`,
//! `stream`/`audit_stream_loop` mirror `install_stream`/`install_stream_loop`.
//! `project` is validated against the SAME discovered-project allowlist
//! `routes_dispatch.rs`/`routes_files.rs` use — a client never supplies a raw
//! filesystem path, only a project NAME resolved server-side to its real root.

use crate::protocol::{
    AuditCheckResponse, AuditEntry, AuditRequest, AuditStreamMsg, AuditsResponse,
};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use omega_core::audit::AuditSkill;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

type ApiError = (StatusCode, Json<serde_json::Value>);

pub async fn list() -> Json<AuditsResponse> {
    let response = tokio::task::spawn_blocking(|| {
        let audits = omega_core::audit::all_audits()
            .iter()
            .map(|a| AuditEntry {
                id: a.id.to_string(),
                name: a.name.to_string(),
                domain: a.domain.label().to_string(),
                phases: a.phases,
                max_score: a.max_score,
                read_only: a.read_only,
            })
            .collect();
        AuditsResponse { audits }
    })
    .await
    .unwrap_or(AuditsResponse { audits: vec![] });
    Json(response)
}

/// Resolves `kind` into a real `AuditSkill` via `omega_core::audit::find_audit`
/// (exact id match, e.g. `"codeaudit"` — the same ids `GET /v1/audits`
/// returns). Pure/synchronous (the registry is an in-process `OnceLock`
/// parse of an embedded TOML, no filesystem walk), so this needs no
/// `spawn_blocking` of its own.
fn resolve_audit_kind(kind: &str) -> Result<AuditSkill, (StatusCode, String)> {
    omega_core::audit::find_audit(kind).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            format!("unknown audit kind: {kind}"),
        )
    })
}

/// Resolves `project` against the discovered-project allowlist — the exact
/// pattern `routes_dispatch.rs::create` (Step 3) and
/// `routes_files.rs::resolve_project_root` use for the same reason: no
/// filesystem access beyond this read-only `discover` walk happens for an
/// unknown project, and the real on-disk root is resolved SERVER-SIDE, never
/// trusted from the client. Involves a filesystem walk, so `spawn_blocking`ed.
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

/// Shared by [`check`] (C1) and [`stream`] (C2) so both endpoints reject the
/// same unknown-kind / unknown-project the same way, with the same message,
/// and resolve the SAME real project path a spawn would use — mirrors
/// `routes_agents.rs::resolve_installable_agent`'s shared-validator role.
async fn resolve_audit_request(
    project: &str,
    kind: &str,
) -> Result<(AuditSkill, PathBuf), (StatusCode, String)> {
    let audit = resolve_audit_kind(kind)?;
    let path = resolve_project_path(project).await?;
    Ok((audit, path))
}

/// `POST /v1/audit` — a THIN, synchronous-shaped pre-flight check, mirroring
/// `routes_agents::install_check`'s shape exactly. Validates `kind` (unknown
/// -> 400) and `project` (unknown -> 400) and echoes back the resolved
/// audit's own metadata so a client can render a confirm dialog, but spawns
/// NOTHING: the real `omega audit run` happens only over the
/// `audit/stream` WebSocket (C2), for the same reason `install_check`
/// documents.
pub async fn check(Json(req): Json<AuditRequest>) -> Result<Json<AuditCheckResponse>, ApiError> {
    let (audit, _path) = resolve_audit_request(&req.project, &req.kind)
        .await
        .map_err(|(code, msg)| (code, Json(serde_json::json!({ "error": msg }))))?;

    Ok(Json(AuditCheckResponse {
        kind: audit.id.to_string(),
        name: audit.name.to_string(),
        phases: audit.phases,
        max_score: audit.max_score,
    }))
}

/// `GET /v1/audit/stream?token=&project=&kind=` — validates `project`/`kind`
/// with the SAME check as [`check`] BEFORE ever upgrading the connection or
/// spawning anything: on a bad param this returns
/// `StatusCode::BAD_REQUEST.into_response()` from this pre-upgrade wrapper,
/// mirroring `routes_agents::install_stream`'s exact shape (reject before
/// upgrade, never upgrade-then-error-inside-the-loop).
///
/// On success, `audit_stream_loop` spawns `omega audit run <kind> --dir
/// <project_path>` (via `omega_cli::omega_bin()`, respecting `OMEGA_BIN`)
/// where `project_path` is the SAME server-resolved path
/// [`resolve_project_path`] just validated — never a client-supplied path —
/// and streams its stdout/stderr lines, then a final exit frame.
pub async fn stream(
    ws: WebSocketUpgrade,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    let project = query.get("project").cloned().unwrap_or_default();
    let kind = query.get("kind").cloned().unwrap_or_default();
    match resolve_audit_request(&project, &kind).await {
        Ok((audit, path)) => ws.on_upgrade(move |socket| audit_stream_loop(socket, audit, path)),
        Err((code, _msg)) => code.into_response(),
    }
}

/// Serializes and sends one audit-stream frame. `Err` means the socket is
/// dead.
async fn send_audit_frame(
    socket: &mut WebSocket,
    frame: &AuditStreamMsg,
) -> Result<(), axum::Error> {
    let text = serde_json::to_string(frame).expect("serialize AuditStreamMsg");
    socket.send(Message::Text(text.into())).await
}

/// Reads `reader` line-by-line and forwards each line as a `Line{stream:
/// stream_name, ..}` frame on `tx`. Returns on EOF, a read error, or the
/// receiver having been dropped. Identical shape to
/// `routes_agents::forward_lines` — `stdout` and `stderr` are each read by
/// their OWN spawned instance of this function (see [`audit_stream_loop`])
/// so neither pipe can starve the other.
async fn forward_lines<R: AsyncRead + Unpin>(
    reader: R,
    stream_name: &'static str,
    tx: mpsc::Sender<AuditStreamMsg>,
) {
    let mut lines = BufReader::new(reader).lines();
    loop {
        match lines.next_line().await {
            Ok(Some(text)) => {
                let frame = AuditStreamMsg::Line {
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
/// `routes_agents::kill_process_group` (same negative-PID `kill -- -<pid>`
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

/// Shared disconnect-cleanup, called from BOTH places [`audit_stream_loop`]
/// can learn the client is gone — identical structure to
/// `routes_agents::kill_and_drain`.
async fn kill_and_drain(
    rx: mpsc::Receiver<AuditStreamMsg>,
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

/// Runs `omega audit run <audit.id> --dir <project_path>` and streams its
/// output to `socket`. Deliberately NOT shared code with
/// `routes_agents::install_stream_loop` — a separate, parallel function that
/// follows the exact same shape and the exact same disconnect-safety
/// contract, per this task's brief: refactoring the already-shipped, tested
/// `install_stream_loop` was explicitly out of scope, and duplicating a
/// ~150-line, carefully-commented state machine is cheaper and safer than
/// forcing a shared abstraction across two files under separate scope rules
/// (R-SCOPE) in the same wave.
///
/// The spawned `omega` process is placed into its OWN process group
/// (`process_group(0)`) rather than inheriting the gateway's, for the same
/// reason `install_stream_loop` documents at length: `omega`'s own real work
/// can run a foreground nested child, and a plain single-PID `kill()` would
/// orphan it rather than stop it. `kill_on_drop(true)` guarantees the DIRECT
/// child is reaped even if this future is ever dropped without reaching the
/// explicit disconnect-kill path.
///
/// The `tokio::select!` loop watches BOTH the internal mpsc frame channel
/// AND the socket's own read side, so a disconnect is noticed even when the
/// child has gone quiet and no send would ever be attempted — the same
/// read-driven fix `install_stream_loop` carries (see its doc comment for
/// the full history: a send-failure-only check misses exactly this case).
async fn audit_stream_loop(mut socket: WebSocket, audit: AuditSkill, project_path: PathBuf) {
    let mut cmd = Command::new(crate::omega_cli::omega_bin());
    cmd.arg("audit")
        .arg("run")
        .arg(audit.id)
        .arg("--dir")
        .arg(&project_path);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.kill_on_drop(true);
    cmd.process_group(0);

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => {
            let _ = send_audit_frame(
                &mut socket,
                &AuditStreamMsg::Error {
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

    let (tx, mut rx) = mpsc::channel::<AuditStreamMsg>(64);
    let stdout_task = tokio::spawn(forward_lines(stdout, "stdout", tx.clone()));
    let stderr_task = tokio::spawn(forward_lines(stderr, "stderr", tx.clone()));
    drop(tx);

    // I-4 (Codex cross-model review, 2026-08-11): an outer wall-clock bound
    // on the WHOLE connection lifetime — see `routes_agents.rs::
    // install_stream_loop`'s identical comment and
    // `omega_cli::stream_timeout`'s doc comment for the default/env var. A
    // QUIET-BUT-ALIVE child (no output, client still connected) used to
    // hold this stream and its subprocess open indefinitely.
    let sleep = tokio::time::sleep(crate::omega_cli::stream_timeout());
    tokio::pin!(sleep);

    loop {
        tokio::select! {
            _ = &mut sleep => {
                let _ = send_audit_frame(
                    &mut socket,
                    &AuditStreamMsg::Error {
                        message: format!(
                            "audit timed out after {}s and was killed",
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
                        if send_audit_frame(&mut socket, &frame).await.is_err() {
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
        Ok(status) => AuditStreamMsg::Exit {
            success: status.success(),
            code: status.code(),
        },
        Err(e) => AuditStreamMsg::Error {
            message: format!("failed to wait on child: {e}"),
        },
    };
    let _ = send_audit_frame(&mut socket, &exit_frame).await;
    let _ = socket.send(Message::Close(None)).await;
}

#[cfg(test)]
mod resolve_audit_kind_tests {
    use super::resolve_audit_kind;

    #[test]
    fn accepts_a_real_audit_id() {
        let audit = resolve_audit_kind("codeaudit").unwrap();
        assert_eq!(audit.id, "codeaudit");
    }

    #[test]
    fn rejects_unknown_audit_kind() {
        let err = resolve_audit_kind("not-a-real-audit").unwrap_err();
        assert_eq!(err.0, axum::http::StatusCode::BAD_REQUEST);
        assert!(
            err.1.contains("not-a-real-audit"),
            "message should name the kind: {}",
            err.1
        );
    }

    #[test]
    fn rejects_wrong_case_variant() {
        // find_audit is an exact, case-sensitive id match — the same ids
        // GET /v1/audits returns, and the same ids `omega audit run` takes
        // on the CLI. "CodeAudit" is not "codeaudit".
        let err = resolve_audit_kind("CodeAudit").unwrap_err();
        assert_eq!(err.0, axum::http::StatusCode::BAD_REQUEST);
    }
}

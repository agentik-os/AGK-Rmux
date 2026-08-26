//! `POST /v1/dispatch` — the one mutating endpoint in this plan: launches an
//! oracle (or delivers a followup into a live one) via the real `omega`
//! CLI's `dispatch` subcommand. SECURITY-CRITICAL INVARIANT: the
//! `omega_cli::run` subprocess is NEVER spawned for a `project` that isn't
//! in the discovered-project list — validation completes fully before any
//! process touches the filesystem beyond the read-only `discover` walk.
//!
//! ## Output contract (confirmed against `omega_core::dispatch.rs` and
//! `omega_cli::main.rs::cmd_dispatch`, not guessed):
//!
//! - `DispatchOutcome::report_lines()` (dispatch.rs:195) ALWAYS emits line 0
//!   as `"◆ Oracle dispatched: {name}"` — a leading glyph + space precede
//!   "Oracle", so the parser searches for the substring `"Oracle dispatched"`
//!   rather than assuming the line starts with it. A `Followup` /
//!   `SpawnedPaneNotReady` / `FollowupUnconfirmed` delivery inserts ONE
//!   additional French annotation line after line 0; `Spawned` does not. The
//!   LAST line `report_lines()` itself emits is always
//!   `"DISPATCH_DELIVERY={tag}"` where `tag` is one of `spawned` / `followup`
//!   / `spawned_pane_not_ready` / `followup_unconfirmed`
//!   (`DispatchDelivery::tag()`, dispatch.rs:246).
//! - BUT `cmd_dispatch` (main.rs:6096-6128), the actual CLI entry point, prints
//!   ONE MORE line AFTER `report_lines()`: `"  Mission: {mission}"`. So real
//!   CLI stdout does NOT end with the `DISPATCH_DELIVERY=` line — the parser
//!   below finds the line that STARTS WITH `"DISPATCH_DELIVERY="` by scanning
//!   all lines, never by indexing `lines.last()`.
//! - The `omega dispatch` argv shape (`Commands::Dispatch`, main.rs:402-417):
//!   positional `project`, positional `mission`, `--agent <value>` (only when
//!   given), `--new` (a bare flag, only when the caller wants to force a new
//!   oracle rather than let the followup router decide). This handler emits
//!   every named flag FIRST, then a bare `"--"` separator, then the two
//!   positional values LAST (`["dispatch", ..flags.., "--", project,
//!   mission]`) — standard clap-safe argv construction, so a `project` or
//!   `mission` value that itself starts with `-` is never misparsed as a
//!   flag by `omega`'s own clap parser.
//!
//! ## Hardening (Task D)
//!
//! Four checks run BEFORE the project-discovery filesystem walk and before
//! any subprocess spawn, cheapest first: a concurrency permit (this endpoint
//! spawns a whole oracle session, so it is capped separately from and lower
//! than the chat-turn semaphore — see `AppState::dispatch_permits`), the
//! pre-existing empty-string checks, a NUL-byte rejection (a `\0` inside a
//! `String` handed to `Command::args` is nonsensical input, not something
//! that should ever reach a subprocess argv), and a length cap on `mission`
//! ([`MAX_MISSION_LEN`]). `req.agent`, when present, is validated against
//! the real `omega_core::agents::Agent::all()` roster before anything is
//! spawned — an unknown name is a 400, not a 502 surfaced from the CLI's own
//! rejection.

use crate::protocol::{DispatchRequest, DispatchResponse};
use crate::server::AppState;
use axum::{extract::State, http::StatusCode, Json};
use serde_json::json;

type ApiError = (StatusCode, Json<serde_json::Value>);

/// Byte-length cap on `mission` — well past any real mission brief, but
/// small enough to keep a runaway/malicious payload from ever reaching a
/// subprocess argv or the underlying CLI's own storage.
const MAX_MISSION_LEN: usize = 8000;

fn bad_request(msg: impl Into<String>) -> ApiError {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": msg.into() })),
    )
}

fn too_many_requests(msg: impl Into<String>) -> ApiError {
    (
        StatusCode::TOO_MANY_REQUESTS,
        Json(json!({ "error": msg.into() })),
    )
}

fn gateway_timeout(msg: impl Into<String>) -> ApiError {
    (
        StatusCode::GATEWAY_TIMEOUT,
        Json(json!({ "error": msg.into() })),
    )
}

/// Parses `omega dispatch`'s stdout into a `DispatchResponse`, per the
/// contract documented at the top of this file. Returns `None` on any
/// shape mismatch — the caller treats that as a 502, never a guess.
fn parse_dispatch_stdout(stdout: &str) -> Option<DispatchResponse> {
    let mut lines = stdout.lines();
    let first = lines.next()?;
    // Line 0: find "Oracle dispatched" anywhere in the line (real output is
    // "◆ Oracle dispatched: <name>"), optional colon, whitespace, then the
    // oracle-<name> token.
    let idx = first.find("Oracle dispatched")?;
    let after = &first[idx + "Oracle dispatched".len()..];
    let after = after.strip_prefix(':').unwrap_or(after);
    let oracle = after.trim();
    if oracle.is_empty() {
        return None;
    }
    let oracle = oracle.to_string();

    // The DISPATCH_DELIVERY= line can appear anywhere in the remaining
    // output (a followup annotation line, and the CLI's own trailing
    // "  Mission: ..." line, both come after it) — scan every line rather
    // than trusting position.
    let delivery = stdout
        .lines()
        .find_map(|l| l.trim().strip_prefix("DISPATCH_DELIVERY="))?
        .trim()
        .to_string();
    if delivery.is_empty() {
        return None;
    }

    Some(DispatchResponse { oracle, delivery })
}

pub async fn create(
    State(state): State<AppState>,
    Json(req): Json<DispatchRequest>,
) -> Result<Json<DispatchResponse>, ApiError> {
    // Step 0: acquire a concurrency permit as the FIRST thing this handler
    // does — the cheapest possible short-circuit, before even the
    // empty-project/mission check. `POST /v1/dispatch` is a single
    // request/response (unlike the chat WS, which holds its permit across a
    // whole streaming turn spawned in a background task), so the permit is
    // just a local variable here: it releases automatically when `create`
    // returns.
    let Ok(_permit) = state.dispatch_permits.clone().try_acquire_owned() else {
        return Err(too_many_requests(
            "too many concurrent dispatches, try again shortly",
        ));
    };

    // Step 1: reject empty/whitespace-only project/mission BEFORE any other
    // check — this is free and catches the common client bug of an empty
    // form field without even touching the filesystem.
    if req.project.trim().is_empty() {
        return Err(bad_request("project must not be empty"));
    }
    if req.mission.trim().is_empty() {
        return Err(bad_request("mission must not be empty"));
    }
    // A NUL byte in a `String` handed to a subprocess argv is nonsensical
    // input — reject both fields outright rather than let it reach
    // `Command::args`, where its behavior at the OS syscall boundary is
    // unpredictable (today it would surface as a confusing 502).
    if req.project.contains('\0') {
        return Err(bad_request("project must not contain a NUL byte"));
    }
    if req.mission.contains('\0') {
        return Err(bad_request("mission must not contain a NUL byte"));
    }
    // `mission` is free-form text with no other bound, so cap it explicitly
    // rather than silently truncate or hand an unbounded payload to a
    // subprocess. `project` is NOT separately capped here: it is already
    // implicitly bounded by the discovered-project allowlist check below (a
    // project name that isn't a real, short, on-disk directory name is
    // rejected there regardless of its length).
    if req.mission.len() > MAX_MISSION_LEN {
        return Err(bad_request(format!(
            "mission too long (max {MAX_MISSION_LEN} bytes)"
        )));
    }

    // Step 2: validate `agent`, when given, against the real roster BEFORE
    // spawning anything — an unknown name is a clean 400, not a 502 surfaced
    // from the CLI's own rejection of a garbage `--agent` value.
    if let Some(name) = req.agent.clone() {
        let is_known = tokio::task::spawn_blocking(move || {
            omega_core::agents::Agent::all()
                .iter()
                .any(|a| a.name() == name)
        })
        .await
        .unwrap_or(false);
        if !is_known {
            return Err(bad_request(format!(
                "unknown agent: {}",
                req.agent.unwrap()
            )));
        }
    }

    // Step 3: validate the project against the discovered-project list
    // BEFORE spawning anything. This is the security-critical gate: no
    // subprocess is spawned for an unknown project, full stop.
    let project = req.project.clone();
    let known = tokio::task::spawn_blocking(move || {
        let home = crate::config::home_dir();
        omega_core::projects::discover(&home)
            .into_iter()
            .any(|p| p.name == project)
    })
    .await
    .unwrap_or(false);
    if !known {
        return Err(bad_request(format!("unknown project: {}", req.project)));
    }

    // Step 4: build argv (never a shell string) and run `omega dispatch`.
    // Named flags come first, then a bare `--` separator, then the two
    // positional values LAST — everything after `--` is treated as a
    // positional value by clap even if it starts with `-`.
    let project = req.project.clone();
    let mission = req.mission.clone();
    let agent = req.agent.clone();
    let force_new = req.new == Some(true);
    let output = tokio::task::spawn_blocking(move || {
        let mut args: Vec<&str> = vec!["dispatch"];
        if let Some(agent) = agent.as_deref() {
            args.push("--agent");
            args.push(agent);
        }
        if force_new {
            args.push("--new");
        }
        args.push("--");
        args.push(&project);
        args.push(&mission);
        // I-2 (Codex cross-model review, 2026-08-11): see
        // routes_sessions.rs::create's identical comment.
        crate::omega_cli::run_with_timeout(&args, crate::omega_cli::cli_timeout())
    })
    .await
    .map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": format!("dispatch task panicked: {e}") })),
        )
    })?
    .map_err(|e| {
        if crate::omega_cli::is_timeout(&e) {
            gateway_timeout(e.to_string())
        } else {
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "error": format!("failed to spawn omega: {e}") })),
            )
        }
    })?;

    // Step 4: non-zero exit or unparseable stdout → 502, never fabricate an
    // oracle name.
    //
    // M-1 (Codex cross-model review, 2026-08-11): the raw stdout/stderr used
    // to be echoed straight into the HTTP response, which can leak
    // environment-derived secrets, file contents, or other sensitive text a
    // future CLI diagnostic writes. The FULL raw text still goes to the
    // gateway's own tracing log for operator debugging; the client only
    // ever sees a generic, sanitized message.
    if !output.success {
        tracing::error!(
            project = %req.project,
            stdout = %output.stdout,
            stderr = %output.stderr,
            "omega dispatch failed"
        );
        return Err((
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": "omega dispatch failed (see gateway logs)" })),
        ));
    }
    let Some(parsed) = parse_dispatch_stdout(&output.stdout) else {
        tracing::error!(
            project = %req.project,
            stdout = %output.stdout,
            stderr = %output.stderr,
            "omega dispatch produced unparseable output"
        );
        return Err((
            StatusCode::BAD_GATEWAY,
            Json(
                json!({ "error": "omega dispatch produced unparseable output (see gateway logs)" }),
            ),
        ));
    };

    Ok(Json(parsed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_spawned_output() {
        let stdout = "\u{25c6} Oracle dispatched: oracle-TestProj-1\nDISPATCH_DELIVERY=spawned\n  Mission: do the thing\n";
        let parsed = parse_dispatch_stdout(stdout).unwrap();
        assert_eq!(parsed.oracle, "oracle-TestProj-1");
        assert_eq!(parsed.delivery, "spawned");
    }

    #[test]
    fn parses_followup_output_with_extra_annotation_line() {
        let stdout = "\u{25c6} Oracle dispatched: oracle-Verba\n  (suivi: route dans l oracle vivant, aucun nouvel oracle cree)\nDISPATCH_DELIVERY=followup\n  Mission: more work\n";
        let parsed = parse_dispatch_stdout(stdout).unwrap();
        assert_eq!(parsed.oracle, "oracle-Verba");
        assert_eq!(parsed.delivery, "followup");
    }

    #[test]
    fn rejects_missing_delivery_tag() {
        let stdout = "\u{25c6} Oracle dispatched: oracle-X\n  Mission: x\n";
        assert!(parse_dispatch_stdout(stdout).is_none());
    }

    #[test]
    fn rejects_missing_oracle_line() {
        let stdout = "something unexpected\nDISPATCH_DELIVERY=spawned\n";
        assert!(parse_dispatch_stdout(stdout).is_none());
    }
}

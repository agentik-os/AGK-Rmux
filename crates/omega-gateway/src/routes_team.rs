//! `POST /v1/team` — wraps `omega team [OPTIONS] <PROJECT> [MEMBERS]...`
//! (`crates/omega-cli/src/main.rs::cmd_team`, ~line 7520).
//!
//! ## Ground truth (verified against the live `omega team --help` and
//! `cmd_team` itself, not guessed — see `.superpowers/sdd/progress.md`'s
//! Task A section)
//!
//! - `cmd_team` builds the REAL spawned session name literally as
//!   `format!("Team-{project}")` (~line 7531) — so `project` is validated
//!   here with the SAME strict slug charset
//!   [`crate::routes_sessions::valid_new_session_name`] already enforces for
//!   a brand-new session name, reused verbatim rather than duplicated: this
//!   `project` is about to become a session-name COMPONENT, exactly the
//!   reason that check exists.
//! - There is NO `--layout` flag anywhere in the real CLI (`omega team
//!   --help` lists only `-c/--count` and `-d/--dir`) — this endpoint does
//!   not accept a `layout` field at all. A wave brief's higher-level mention
//!   of one was ground-truth-checked against the CLI's own `--help` and
//!   found not to exist, so it is dropped as a documented gap rather than
//!   silently invented.
//! - `count` bounded `1..=8` here is DEFENSE-IN-DEPTH for an API caller —
//!   the real CLI's own default is 3 with no hard cap of its own.
//! - `members` are optional `"name:prompt"` strings passed positionally;
//!   `cmd_team` parses each permissively via `splitn(2, ':')` with no
//!   charset validation of its own, so this endpoint only NUL-checks and
//!   length-caps them, never re-derives a stricter shape the real CLI
//!   doesn't enforce either.
//!
//! `output` is the raw stdout on success — this endpoint deliberately does
//! NOT hand-parse the CLI's own per-member text, mirroring wave7's
//! `reap`/`resurrect` "honest raw output" precedent
//! ([`crate::protocol::ReapResponse`]/[`crate::protocol::ResurrectResponse`]).

use crate::protocol::{TeamRequest, TeamResponse};
use crate::routes_sessions::{dir_under_home, valid_new_session_name};
use crate::server::AppState;
use axum::{extract::State, http::StatusCode, Json};
use serde_json::json;

type ApiError = (StatusCode, Json<serde_json::Value>);

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

/// Per-member string cap — generous for a `"name:prompt"` spec (the CLI
/// itself parses each permissively, no length bound of its own) while still
/// keeping an unreasonable payload from ever reaching a subprocess argv.
/// Deliberately tighter than `routes_sessions.rs::MAX_PROMPT_LEN` (8000): a
/// team member spec is a short label + a one-line task, not a whole mission
/// brief.
const MAX_MEMBER_LEN: usize = 500;

/// `count` bounds — the real CLI default is 3 with NO hard cap of its own;
/// this endpoint adds one purely as defense-in-depth for an API caller,
/// matching this crate's established "cap every unbounded numeric field"
/// convention.
const MIN_COUNT: u32 = 1;
const MAX_COUNT: u32 = 8;

/// Validates `count` against [`MIN_COUNT`]..=[`MAX_COUNT`] — factored out as
/// a pure function so it has a direct unit test with no HTTP server
/// involved, the same style `routes_files.rs::resolve_scoped_path` and
/// `routes_dispatch.rs::parse_dispatch_stdout` use for their own pure logic.
fn count_in_bounds(count: u32) -> bool {
    (MIN_COUNT..=MAX_COUNT).contains(&count)
}

pub async fn create(
    State(state): State<AppState>,
    Json(req): Json<TeamRequest>,
) -> Result<Json<TeamResponse>, ApiError> {
    // Step 0: acquire a concurrency permit as the FIRST thing this handler
    // does — shared with `POST /v1/sessions` (see
    // `AppState::session_spawn_permits`'s doc comment): a team spawn is at
    // least as heavy (up to `MAX_COUNT` sub-panes per call).
    let Ok(_permit) = state.session_spawn_permits.clone().try_acquire_owned() else {
        return Err(too_many_requests(
            "too many concurrent session spawns, try again shortly",
        ));
    };

    // Step 1: `project` becomes a session-name COMPONENT (see this file's
    // doc comment) — validate it with the exact same strict slug check a
    // brand-new session name gets, BEFORE any subprocess spawn.
    if !valid_new_session_name(&req.project) {
        return Err(bad_request("invalid project name"));
    }

    // `session` is ALREADY known here — `cmd_team` spawns
    // `format!("Team-{project}")` literally (see this file's doc comment).
    // Computed early (rather than at the old Step 5) so the Finding 4
    // sanitize round-trip check below can run BEFORE any subprocess spawn.
    let session = format!("Team-{}", req.project);

    // Finding 4 (adversarial review round): `valid_new_session_name` checks
    // `project` alone, but the REAL spawned session name is `session`
    // above, which then goes through
    // `omega_core::session::sanitize_session_name` (truncates at
    // `MAX_SESSION_NAME_LEN`). The `"Team-"` prefix eats 5 of that budget,
    // so a `project` that individually passes the charset+length check can
    // still make the FULL name diverge after sanitation — checked on the
    // FULL `session` string, not the bare `project` (a bare-`project`
    // check would miss exactly this prefix-budget case).
    if omega_core::session::sanitize_session_name(&session) != session {
        return Err(bad_request(
            "project name would make the real team session name diverge after rmux sanitation (too long once prefixed with \"Team-\") — choose a shorter project name",
        ));
    }

    // Step 2: `count`, when given, is bounded — defense-in-depth, the real
    // CLI has no hard cap of its own.
    if let Some(count) = req.count {
        if !count_in_bounds(count) {
            return Err(bad_request(format!(
                "count must be between {MIN_COUNT} and {MAX_COUNT}"
            )));
        }
    }

    // Step 3: `dir`, when given, reuses `POST /v1/sessions`'s allowed-root
    // check.
    let dir = match req.dir {
        Some(ref d) => Some(dir_under_home(d)?),
        None => None,
    };

    // Step 4: `members`, when given, are safety-checked only (a count cap +
    // NUL byte + per-string length cap) — the CLI itself parses each
    // `"name:prompt"` spec permissively with no charset validation, so this
    // endpoint does not re-derive one either. The count cap reuses
    // `MAX_COUNT` (finding: an unbounded `members` vector was accepted with
    // no cap at all — the reviewer reproduced 200,000 members in one
    // request, which `cmd_team` would spawn one rmux pane per, since it
    // ignores `count` whenever `members` is non-empty): one team member is
    // the same underlying concept as one requested pane, so it shares
    // `count`'s bound rather than inventing a second, separate one.
    if let Some(ref members) = req.members {
        if members.len() > MAX_COUNT as usize {
            return Err(bad_request(format!("too many members (max {MAX_COUNT})")));
        }
        for m in members {
            if m.contains('\0') {
                return Err(bad_request("member must not contain a NUL byte"));
            }
            if m.len() > MAX_MEMBER_LEN {
                return Err(bad_request(format!(
                    "member too long (max {MAX_MEMBER_LEN} bytes)"
                )));
            }
        }
    }

    // Step 5 (echoed back rather than parsed off stdout, the same "we
    // already know it" posture `routes_sessions::rename` uses for
    // `new_name`): `session` was already computed above, ahead of the
    // Finding 4 sanitize check.

    // Step 6: build argv (never a shell string) and run `omega team`. Flags
    // first, then a bare `--` separator, then PROJECT then every MEMBER
    // last — clap-safe, the same shape `routes_dispatch.rs::create` uses.
    let project_arg = req.project.clone();
    let count_arg = req.count.map(|c| c.to_string());
    let dir_arg = dir.as_ref().map(|d| d.to_string_lossy().into_owned());
    let members_arg = req.members.clone().unwrap_or_default();
    let output = tokio::task::spawn_blocking(move || {
        let mut args: Vec<&str> = vec!["team"];
        if let Some(c) = count_arg.as_deref() {
            // `count` is a `u32` from a bounded range (1..=MAX_COUNT), so
            // it can never start with `-` — the two-element form is safe
            // here, unlike `dir` below (Finding 5, adversarial review
            // round).
            args.push("--count");
            args.push(c);
        }
        // Finding 5 (adversarial review round): same `=`-form fix
        // `routes_sessions.rs::create` applies to its own `--dir`/
        // `--prompt` — a bare `--` separator protects POSITIONALS, not
        // flag VALUES, so a `dir` value starting with `-` would otherwise
        // be misparsed as a second flag by clap.
        let dir_flag = dir_arg.as_deref().map(|d| format!("--dir={d}"));
        if let Some(ref f) = dir_flag {
            args.push(f);
        }
        args.push("--");
        args.push(&project_arg);
        for m in &members_arg {
            args.push(m);
        }
        // I-2 (Codex cross-model review, 2026-08-11): see
        // routes_sessions.rs::create's identical comment.
        crate::omega_cli::run_with_timeout(&args, crate::omega_cli::cli_timeout())
    })
    .await
    .map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": format!("team task panicked: {e}") })),
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

    // Non-zero exit → 502, never fabricate a session.
    //
    // M-1 (Codex cross-model review, 2026-08-11): the raw stdout/stderr used
    // to be echoed straight into the HTTP response, which can leak
    // environment-derived secrets or other sensitive text a future CLI
    // diagnostic writes. The FULL raw text still goes to the gateway's own
    // tracing log; the client only ever sees a generic, sanitized message.
    if !output.success {
        tracing::error!(
            project = %req.project,
            stdout = %output.stdout,
            stderr = %output.stderr,
            "omega team failed"
        );
        return Err((
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": "omega team failed (see gateway logs)" })),
        ));
    }

    Ok(Json(TeamResponse {
        session,
        output: output.stdout,
    }))
}

#[cfg(test)]
mod tests {
    use super::count_in_bounds;

    #[test]
    fn accepts_the_lower_bound() {
        assert!(count_in_bounds(1));
    }

    #[test]
    fn accepts_the_upper_bound() {
        assert!(count_in_bounds(8));
    }

    #[test]
    fn accepts_the_real_cli_default() {
        assert!(count_in_bounds(3));
    }

    #[test]
    fn rejects_zero() {
        assert!(!count_in_bounds(0));
    }

    #[test]
    fn rejects_nine() {
        assert!(!count_in_bounds(9));
    }
}

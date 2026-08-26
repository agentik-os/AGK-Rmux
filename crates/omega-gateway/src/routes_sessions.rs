use crate::protocol::{
    CloseSessionResponse, CreateSessionRequest, CreateSessionResponse, RenameSessionRequest,
    RenameSessionResponse, SendKeysRequest, SendKeysResponse, SessionEntry, SessionsResponse,
    StreamFrame,
};
use axum::Json;

pub async fn list() -> Json<SessionsResponse> {
    match tokio::task::spawn_blocking(crate::rmux::list_sessions).await {
        Ok(Ok(names)) => Json(SessionsResponse {
            sessions: names
                .into_iter()
                .map(|name| SessionEntry { name })
                .collect(),
            error: None,
        }),
        Ok(Err(e)) => Json(SessionsResponse {
            sessions: vec![],
            error: Some(e.to_string()),
        }),
        Err(e) => Json(SessionsResponse {
            sessions: vec![],
            error: Some(e.to_string()),
        }),
    }
}

use crate::server::AppState;
use axum::extract::{
    ws::{Message, WebSocket, WebSocketUpgrade},
    Path, Query, State,
};
use axum::http::StatusCode;
use axum::response::Response;
use std::collections::HashMap;

pub async fn stream(
    ws: WebSocketUpgrade,
    Path(name): Path<String>,
    Query(query): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Response {
    let color = query.get("color").map(|v| v == "1").unwrap_or(false);
    ws.on_upgrade(move |socket| stream_loop(socket, name, state, color))
}

async fn stream_loop(mut socket: WebSocket, name: String, state: AppState, color: bool) {
    let interval = std::time::Duration::from_millis(state.cfg.stream_interval_ms);
    let lines = state.cfg.stream_lines;
    let mut last: Option<String> = None;
    // R-STREAM: this loop never exits on error; errors are rendered as frames.
    // KNOWN LIMIT (V1): the only exit is a failed send, which fires instantly
    // on a clean client close but only after the kernel TCP timeout on a
    // silent network death. Plan 2 hardening adds ping/pong liveness.
    // KNOWN LIMIT (V1): revoking a device does not terminate an already-open
    // stream — the socket keeps running on the token that was valid at
    // upgrade time. Re-verification on a live connection lands with the
    // Plan 2 ping/pong liveness pass above.
    // KNOWN LIMIT (V1): no pairing or stream rate limiting yet; that is
    // Plan 2 scope too.
    loop {
        let session = name.clone();
        let captured = tokio::task::spawn_blocking(move || {
            if color {
                crate::rmux::capture_pane_ansi(&session, lines)
            } else {
                crate::rmux::capture_pane(&session, lines)
            }
        })
        .await;
        let frame = match captured {
            Ok(Ok(text)) => {
                if last.as_deref() == Some(text.as_str()) {
                    None
                } else {
                    last = Some(text.clone());
                    Some(StreamFrame::Frame { text })
                }
            }
            Ok(Err(e)) => Some(StreamFrame::Error {
                message: e.to_string(),
            }),
            Err(e) => Some(StreamFrame::Error {
                message: e.to_string(),
            }),
        };
        if let Some(frame) = frame {
            let text = serde_json::to_string(&frame).expect("serialize frame");
            if socket.send(Message::Text(text.into())).await.is_err() {
                return; // client went away: the ONLY exit
            }
        }
        tokio::time::sleep(interval).await;
    }
}

/// Maximum accepted `data` payload for `POST /v1/sessions/{name}/keys`, in
/// bytes. A safety valve, not a hard product requirement: rmux `send-keys`
/// has no documented limit, but an unbounded body lets one request tie up
/// the subprocess for an unreasonable time and is a trivial DoS/typo vector
/// (a client accidentally posting a whole file). 8 KiB comfortably covers
/// any real interactive keystroke burst (a pasted command, a multi-line
/// snippet) while staying far below anything that would matter for argv
/// size or subprocess latency.
const MAX_SEND_KEYS_BYTES: usize = 8192;
const PASTE_SETTLE_MS: u64 = 100;

/// Session names this endpoint will act on: mirrors `routes_chat.rs`'s
/// `valid_chat_id` shape — reject anything that could path-traverse or
/// shell-inject BEFORE it ever reaches a subprocess argv. rmux session names
/// in practice are `oracle-<Project>-<n>`-shaped or similar identifiers, so a
/// conservative charset (letters/digits/`_`/`-`/`.`) with a generous length
/// cap covers every real name without accepting `/`, `..`, or NUL.
///
/// `pub(crate)` (rather than private) so `routes_session_org.rs` reuses this
/// EXACT guard instead of duplicating it: the session-org overlay never
/// touches a live session or a subprocess, but a session name still becomes
/// a JSON map key persisted to disk, and the same path-traversal-shaped
/// input is unsafe there too.
pub(crate) fn valid_session_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 200 {
        return false;
    }
    if name.contains('/') || name.contains("..") || name.contains('\0') {
        return false;
    }
    name.bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b'.')
}

pub async fn send_keys(
    State(_state): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<SendKeysRequest>,
) -> Result<Json<SendKeysResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Validate BEFORE touching any subprocess, same discipline
    // routes_dispatch.rs uses for `project`.
    if !valid_session_name(&name) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "invalid session name" })),
        ));
    }
    if req.data.len() > MAX_SEND_KEYS_BYTES {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "data too large" })),
        ));
    }

    let session = name.clone();
    let data = req.data.clone();
    tokio::task::spawn_blocking(move || crate::rmux::send_keys_literal(&session, &data))
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": format!("send_keys task panicked: {e}") })),
            )
        })?
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    if req.enter {
        // Full-screen agent composers process a literal paste asynchronously.
        // Sending Enter in the immediately following rmux subprocess can land
        // before the composer has accepted the paste, leaving a visibly typed
        // but unsubmitted prompt. Keep Enter as its own key event, and allow a
        // short settle window only when this request actually pasted text.
        if !req.data.is_empty() {
            tokio::time::sleep(std::time::Duration::from_millis(PASTE_SETTLE_MS)).await;
        }
        let session = name.clone();
        tokio::task::spawn_blocking(move || crate::rmux::send_enter(&session))
            .await
            .map_err(|e| {
                (
                    StatusCode::BAD_GATEWAY,
                    Json(serde_json::json!({ "error": format!("send_enter task panicked: {e}") })),
                )
            })?
            .map_err(|e| {
                (
                    StatusCode::BAD_GATEWAY,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
            })?;
    }

    Ok(Json(SendKeysResponse { ok: true }))
}

type ApiError = (StatusCode, Json<serde_json::Value>);

fn bad_request(msg: impl Into<String>) -> ApiError {
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": msg.into() })),
    )
}

fn too_many_requests(msg: impl Into<String>) -> ApiError {
    (
        StatusCode::TOO_MANY_REQUESTS,
        Json(serde_json::json!({ "error": msg.into() })),
    )
}

fn gateway_timeout(msg: impl Into<String>) -> ApiError {
    (
        StatusCode::GATEWAY_TIMEOUT,
        Json(serde_json::json!({ "error": msg.into() })),
    )
}

/// Parses `cmd_kill`'s alias-resolution line (`"[i] {name} resolved to the
/// oracle session {resolved}"`, printed to **stdout** before anything else
/// when the caller-supplied name needed `resolve_oracle_alias` — see
/// `cmd_kill` in `crates/omega-cli/src/main.rs`) and returns the resolved
/// name it names, or `None` when no such line is present (the common case:
/// no resolution happened). Pure/testable, mirroring `routes_dispatch.rs`'s
/// `parse_dispatch_stdout` style.
fn resolved_oracle_name(stdout: &str) -> Option<&str> {
    let line = stdout.lines().find(|l| l.starts_with("[i] "))?;
    let resolved = line.split(" resolved to the oracle session ").nth(1)?;
    let resolved = resolved.trim();
    if resolved.is_empty() {
        None
    } else {
        Some(resolved)
    }
}

/// `POST /v1/sessions/{name}/close` — runs `omega kill <name>` (never
/// `--force`, so the REFUSED-because-live-workers case surfaces to the app
/// explicitly rather than the endpoint silently forcing it away).
///
/// ## Output-contract deviation from a first read of `cmd_kill` (R-VERIFY /
/// L1 — verified against real runtime behavior, not assumed): `cmd_kill`'s
/// REFUSED path is an `anyhow::bail!`, and `omega-cli`'s `main()` is
/// `#[tokio::main] async fn main() -> Result<()>` — Rust's default
/// `Termination` impl for a failing `Result<(), E: Debug>` prints
/// `"Error: {err:?}"` to **stderr** (confirmed with a throwaway
/// `anyhow::bail!` + `#[tokio::main] async fn main() -> anyhow::Result<()>`
/// reproduction: stdout was empty, stderr was exactly `"Error: kill
/// REFUSED — ...\n"`), not stdout. BUT `cmd_kill` may ALSO print an
/// alias-resolution line to stdout BEFORE hitting the REFUSED bail (when the
/// caller-supplied name needed `resolve_oracle_alias`), so stdout emptiness
/// is not a reliable signal of success — `message` switches on
/// `output.success` instead: on success it is `stdout` alone (where the
/// success/already-closed/cascaded-worker lines all live), on failure it is
/// `stdout` + `stderr` concatenated so a resolution line is never dropped
/// alongside the REFUSED text.
///
/// `cascaded_count` / `already_closed` still parse `stdout` only, per the
/// brief: both of those lines are real `println!`s in the non-REFUSED
/// paths, so they are unaffected by this success/failure split.
///
/// `is_oracle` is classified off the name `cmd_kill` actually resolved and
/// acted on — parsed via [`resolved_oracle_name`] — not the raw path param,
/// since an alias-shaped input (e.g. `"Verba-3"`) falls through
/// `OmegaSession::classify` to `SessionRole::Home` while the real resolved
/// session (`"oracle-Verba-3"`) is an oracle.
pub async fn close(
    State(_state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<CloseSessionResponse>, ApiError> {
    // Validate BEFORE touching any subprocess, same discipline as send_keys.
    if !valid_session_name(&name) {
        return Err(bad_request("invalid session name"));
    }

    let session = name.clone();
    // I-7 (Codex cross-model review, 2026-08-11): `valid_session_name`
    // permits a name whose first byte is `-` (unlike `valid_new_session_name`,
    // which rejects a leading `-`/`.` for a brand-new name) -- a `"--"`
    // separator before the positional is REQUIRED so a name like `-x` is
    // never misparsed as a flag by the CLI's own clap parser.
    let output =
        tokio::task::spawn_blocking(move || crate::omega_cli::run(&["kill", "--", &session]))
            .await
            .map_err(|e| {
                (
                    StatusCode::BAD_GATEWAY,
                    Json(serde_json::json!({ "error": format!("kill task panicked: {e}") })),
                )
            })?
            .map_err(|e| {
                (
                    StatusCode::BAD_GATEWAY,
                    Json(serde_json::json!({ "error": format!("failed to spawn omega: {e}") })),
                )
            })?;

    // Classify off the resolved name when `cmd_kill` reported one, else the
    // raw caller-supplied name — never parsed off the CLI otherwise.
    let classify_name = resolved_oracle_name(&output.stdout).unwrap_or(&name);
    let is_oracle = omega_core::session::OmegaSession::classify(classify_name).role
        == omega_core::session::SessionRole::Oracle;

    let cascaded_count = output
        .stdout
        .lines()
        .filter(|l| l.starts_with("  cascaded worker"))
        .count() as u32;
    let already_closed = output
        .stdout
        .contains("is already closed — nothing live to kill.");
    // M-1 (Codex cross-model review, 2026-08-11): the SUCCESS-path `message`
    // (stdout alone) is a documented contract other code/tests depend on
    // (the success/already-closed/cascaded-worker informational text) and
    // stays untouched. Only the FAILURE-path `message` — which used to be
    // raw stdout+stderr concatenated — is sanitized: the full raw text
    // still goes to the gateway's own tracing log, never the HTTP response.
    let message = if output.success {
        output.stdout.clone()
    } else {
        tracing::error!(
            session = %name,
            stdout = %output.stdout,
            stderr = %output.stderr,
            "omega kill failed"
        );
        "omega kill failed (see gateway logs)".to_string()
    };

    Ok(Json(CloseSessionResponse {
        killed: output.success,
        already_closed,
        is_oracle,
        cascaded_count,
        message,
    }))
}

/// Safe-slug charset for a `rename`'s *new* name — a deliberately TIGHTER,
/// FRESH check than [`valid_session_name`] (never weakened to admit `.`,
/// which rmux is documented to silently REWRITE to `_` rather than reject —
/// see `rmux.rs`'s own quoting-discipline doc comments). ASCII alnum + `_` +
/// `-` only: no `.`, `/`, `:`, whitespace, or NUL. A chosen new name is
/// short, so the length cap is intentionally tighter (100) than
/// `valid_session_name`'s 200.
///
/// Also rejects a name whose FIRST byte is `-` or `.`: live-verified against
/// the real `rmux` binary (a `--` argv separator does not help — this is
/// daemon-side trimming, not client argv parsing), `rmux rename-session -t
/// <session> -q` actually renames the session to `q`, silently dropping the
/// leading `-`. A leading `.` is rejected for the same reason this function
/// already bans `.` everywhere else in the name. Left unchecked, the
/// response's echoed `name` would diverge from the real session name.
///
/// `pub(crate)` (rather than private) so `routes_team.rs::create` reuses this
/// EXACT charset check for its own `project` field: `omega team`'s
/// `cmd_team` builds the real spawned session name as literally
/// `format!("Team-{project}")`, so `project` is about to become a session
/// name COMPONENT — the same reasoning that requires this strict check for
/// `rename`'s `new_name`, not the looser [`valid_session_name`] used for
/// referencing an already-existing session.
pub(crate) fn valid_new_session_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 100 {
        return false;
    }
    if name.starts_with('-') || name.starts_with('.') {
        return false;
    }
    name.bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

/// `POST /v1/sessions/{name}/rename` — runs
/// `rmux rename-session -t <name> <new_name>`. Both the path `{name}` and
/// the body's `new_name` are validated BEFORE any subprocess spawn:
/// `new_name` especially, since the caller expects it back verbatim and
/// rmux would otherwise silently rewrite an unsafe character rather than
/// reject it.
pub async fn rename(
    State(_state): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<RenameSessionRequest>,
) -> Result<Json<RenameSessionResponse>, ApiError> {
    if !valid_session_name(&name) {
        return Err(bad_request("invalid session name"));
    }
    if !valid_new_session_name(&req.new_name) {
        return Err(bad_request("invalid new session name"));
    }

    let old = name.clone();
    let new = req.new_name.clone();
    tokio::task::spawn_blocking(move || crate::rmux::rename_session(&old, &new))
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": format!("rename task panicked: {e}") })),
            )
        })?
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    Ok(Json(RenameSessionResponse { name: req.new_name }))
}

/// Byte-length cap on `POST /v1/sessions`'s `prompt` — mirrors
/// `routes_dispatch.rs::MAX_MISSION_LEN`'s cap (that constant is private to
/// its own module, so this is a deliberate mirror, not a shared import): a
/// generous bound past any real prompt, small enough to keep a
/// runaway/malicious payload from ever reaching a subprocess argv.
const MAX_PROMPT_LEN: usize = 8000;

/// Resolves a client-supplied `dir` (`POST /v1/sessions`'s and `POST
/// /v1/team`'s `-d`/`--dir`, both handed straight to `omega`'s own
/// subprocess, which may itself CREATE the directory) to a path proven to
/// lie under this server's real home directory.
///
/// Deliberately uses `dirs::home_dir()` — NOT `crate::config::home_dir()`'s
/// `$OMEGA_HOME`-overridable variant. `crate::config::home_dir()` exists so
/// a hermetic test can redirect PROJECT DISCOVERY away from the operator's
/// real `$HOME`; this check is a different thing, a SAFETY boundary on where
/// an arbitrary API caller may cause a subprocess to write, and that
/// boundary must not be movable by the same env var a test flips for an
/// unrelated reason.
///
/// Unlike `routes_files.rs::resolve_scoped_path` (which resolves an EXISTING
/// file inside an already-known project root), the requested `dir` here is
/// routinely a path that does not exist yet — `omega new -d <dir>` / `omega
/// team -d <dir>` both create it on demand. So rather than requiring `path`
/// itself to resolve, this walks UP from the requested path to its nearest
/// EXISTING ancestor and canonicalizes THAT — the same ancestor-walk idea
/// `resolve_scoped_path` uses for its 403-vs-404 split, adapted here to a
/// single pass/reject boundary rather than two distinct error shapes (a
/// caller-supplied `dir` outside the allowed root is always a 400, there is
/// no analogous "not found" case worth distinguishing for a directory the
/// server may be about to create). The walk always terminates: an absolute
/// path's ancestor chain ends at `/`, which always canonicalizes and — not
/// being under `$HOME` — correctly fails the prefix check; a relative path's
/// chain ends at the empty path, whose `canonicalize` fails, and the walk
/// then reports the same rejection rather than looping forever.
///
/// Returns the ORIGINAL (uncanonicalized) `path` as a `PathBuf` on success —
/// never the canonicalized ancestor — since the caller passes this straight
/// to `--dir`, and canonicalizing a not-yet-existing leaf would silently
/// change what gets created (e.g. resolving a symlinked ancestor to its real
/// target).
pub(crate) fn dir_under_home(path: &str) -> Result<std::path::PathBuf, ApiError> {
    if path.contains('\0') {
        return Err(bad_request("dir must not contain a NUL byte"));
    }
    let requested = std::path::PathBuf::from(path);

    // Finding 1 (adversarial review round): reject any `..` component
    // BEFORE the ancestor walk below ever runs. Without this, a path like
    // `<home>/does-not-exist-yet/../../../tmp/evil/PWNED` defeats the walk:
    // every ancestor containing `does-not-exist-yet` fails to canonicalize
    // (that leading component does not exist), so the walk keeps popping
    // until it reaches `$HOME` itself -- which DOES canonicalize and pass
    // the prefix check below -- at which point the function would return
    // the ORIGINAL, uncanonicalized `path` string, which still contains the
    // escaping `..` sequences and resolves OUTSIDE home if ever lexically
    // normalized or `create_dir_all`'d downstream. Checked structurally via
    // `Path::components()` rather than a substring search for `".."`: a
    // substring search has its own false-positive/false-negative traps
    // (e.g. it would reject a legitimate directory literally named
    // `my..project`, which has no `ParentDir` component at all).
    use std::path::Component;
    for component in requested.components() {
        if matches!(component, Component::ParentDir) {
            return Err(bad_request(
                "dir must not contain a parent-directory (`..`) component",
            ));
        }
    }

    // Cross-task consistency fix (final whole-branch review, wave8): a
    // RELATIVE `dir` used to be validated by canonicalizing it against THIS
    // PROCESS's own current working directory, then returned as the
    // original (still-relative) string -- but the caller of this function
    // (`omega new -d <dir>` / `omega team -d <dir>`) hands that relative
    // string to the rmux DAEMON, which resolves it against ITS OWN cwd, not
    // gatewayd's. So a relative `dir` could pass this "under home" check
    // against one cwd while actually landing somewhere else entirely once
    // the daemon resolves it -- the validated path and the effective path
    // diverge, silently. `routes_duo.rs` independently discovered and
    // guarded against exactly this shape (its own `is_absolute()` check,
    // Finding 3 of that task's adversarial review round) while
    // `routes_sessions.rs`/`routes_team.rs` did not, leaving three callers
    // of one shared helper with different safety guarantees. Requiring an
    // absolute path here closes the gap for every caller at once and makes
    // `routes_duo.rs`'s own local check redundant (left in place there as
    // harmless defense-in-depth, not removed).
    if !requested.is_absolute() {
        return Err(bad_request("dir must be an absolute path"));
    }

    let home = dirs::home_dir().ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "no home directory configured on this server" })),
        )
    })?;
    let canon_home = std::fs::canonicalize(&home).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("failed to resolve home dir: {e}") })),
        )
    })?;

    let mut candidate = requested.clone();
    loop {
        match std::fs::canonicalize(&candidate) {
            Ok(canon_ancestor) => {
                return if canon_ancestor.starts_with(&canon_home) {
                    Ok(requested)
                } else {
                    Err(bad_request("dir must be under the home directory"))
                };
            }
            Err(_) => {
                if !candidate.pop() {
                    // Exhausted the ancestor chain (an empty relative path)
                    // without ever resolving — reject, never assume safe.
                    return Err(bad_request("dir must be under the home directory"));
                }
            }
        }
    }
}

/// `POST /v1/sessions` — wraps `omega new [OPTIONS] <NAME>`
/// (`crates/omega-cli/src/main.rs::cmd_new`). The CLI's `NAME` positional is
/// REQUIRED even though [`CreateSessionRequest::name`] is optional: when the
/// caller omits it, a server-generated slug (`gw-{6 random hex bytes}`)
/// stands in — it trivially passes [`valid_new_session_name`] by
/// construction (lowercase hex + `-` + a `gw` prefix), so it is never
/// special-cased through the check a caller-supplied name gets.
///
/// Validation runs BEFORE any subprocess spawn, in this order: `agent`
/// against the real `omega_core::agents::Agent::all()` roster (same posture
/// `routes_dispatch.rs::create` uses for its own optional `agent` field);
/// the resolved session `name` against [`valid_new_session_name`]; `dir`
/// (when given) against [`dir_under_home`]; `prompt` (when given) against a
/// NUL-byte check and [`MAX_PROMPT_LEN`]. `--cmd` and `--files` are never
/// exposed — see [`CreateSessionRequest`]'s doc comment.
///
/// Argv is built clap-safe: every named flag first, then a bare `--`
/// separator, then NAME last — a `name` or `prompt` value that itself starts
/// with `-` is never misparsed as a flag by `omega`'s own clap parser (the
/// same shape `routes_dispatch.rs::create` uses).
///
/// On success the response echoes back `name`/`agent` as ALREADY KNOWN
/// (chosen/validated before the spawn) rather than parsed off `cmd_new`'s
/// own `"Created session: {name}"` stdout line — see
/// [`CreateSessionResponse`]'s doc comment.
pub async fn create(
    State(state): State<AppState>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<Json<CreateSessionResponse>, ApiError> {
    // Step 0: acquire a concurrency permit as the FIRST thing this handler
    // does — the cheapest possible short-circuit, before even the agent
    // check. Shared with `POST /v1/team` (see
    // `AppState::session_spawn_permits`'s doc comment). Mirrors
    // `routes_dispatch.rs::create`'s own Step 0.
    let Ok(_permit) = state.session_spawn_permits.clone().try_acquire_owned() else {
        return Err(too_many_requests(
            "too many concurrent session spawns, try again shortly",
        ));
    };

    // Step 1: `agent` must be a real, known agent — validated before
    // anything else touches the filesystem or a subprocess.
    let agent_check = req.agent.clone();
    let is_known = tokio::task::spawn_blocking(move || {
        omega_core::agents::Agent::all()
            .iter()
            .any(|a| a.name() == agent_check)
    })
    .await
    .unwrap_or(false);
    if !is_known {
        return Err(bad_request(format!("unknown agent: {}", req.agent)));
    }

    // Step 2: resolve the session name — caller-supplied names must pass the
    // exact same strict slug check `rename` enforces for a brand-new session
    // name; an omitted name is generated server-side (see this function's
    // doc comment).
    let name = match req.name {
        Some(ref n) => {
            if !valid_new_session_name(n) {
                return Err(bad_request("invalid session name"));
            }
            // Finding 4 (adversarial review round): `valid_new_session_name`
            // allows up to 100 bytes and a trailing `-`, but the REAL
            // session name `cmd_new` creates goes through
            // `omega_core::session::sanitize_session_name` (truncates at
            // `MAX_SESSION_NAME_LEN` and trims a trailing `-`/`.`). A name
            // that survives the charset check but diverges after
            // sanitation would make this endpoint echo back a name the
            // caller can never actually address (`/v1/sessions/{name}/...`
            // would 404 against it) — round-tripping through the REAL
            // sanitizer makes "the name we're about to echo is byte-for-
            // byte the name the CLI will actually create" a structural
            // invariant instead of a hand-maintained charset mirror that
            // can drift.
            if omega_core::session::sanitize_session_name(n) != *n {
                return Err(bad_request(
                    "session name would be altered by rmux sanitation (too long, or a trailing `-`/`.`) — choose a shorter, plain name",
                ));
            }
            n.clone()
        }
        None => format!("gw-{}", crate::util::random_hex(6)),
    };

    // Step 3: `dir`, when given, must resolve under this server's home dir.
    let dir = match req.dir {
        Some(ref d) => Some(dir_under_home(d)?),
        None => None,
    };

    // Step 4: `prompt`, when given, is NUL-checked and length-capped — same
    // discipline `routes_dispatch.rs::create` applies to `mission`.
    if let Some(ref p) = req.prompt {
        if p.contains('\0') {
            return Err(bad_request("prompt must not contain a NUL byte"));
        }
        if p.len() > MAX_PROMPT_LEN {
            return Err(bad_request(format!(
                "prompt too long (max {MAX_PROMPT_LEN} bytes)"
            )));
        }
    }

    // Step 5: build argv (never a shell string) and run `omega new`.
    let agent_arg = req.agent.clone();
    let dir_arg = dir.as_ref().map(|d| d.to_string_lossy().into_owned());
    let prompt_arg = req.prompt.clone();
    let name_arg = name.clone();
    let output = tokio::task::spawn_blocking(move || {
        let mut args: Vec<&str> = vec!["new", "--agent", &agent_arg];
        // Finding 5 (adversarial review round): a bare `--` separator
        // protects POSITIONALS, not flag VALUES — clap parses two argv
        // elements `--dir -x` as TWO flags, not one flag plus a
        // hyphen-leading value, since `allow_hyphen_values` isn't set on
        // `--dir`/`--prompt` in the real CLI (`omega new`'s clap
        // definition). A `dir`/`prompt` value starting with `-` is
        // perfectly legitimate input (a real directory name, a real
        // prompt), so it is emitted as a SINGLE argv element via the `=`
        // form, which clap always accepts regardless of what the value
        // starts with.
        let dir_flag = dir_arg.as_deref().map(|d| format!("--dir={d}"));
        if let Some(ref f) = dir_flag {
            args.push(f);
        }
        let prompt_flag = prompt_arg.as_deref().map(|p| format!("--prompt={p}"));
        if let Some(ref f) = prompt_flag {
            args.push(f);
        }
        args.push("--");
        args.push(&name_arg);
        // I-2 (Codex cross-model review, 2026-08-11): a blocking `run`
        // cannot be cancelled once spawned and has no timeout, so a
        // hung/adversarially-slow `omega new` can pin this permit and this
        // spawn_blocking thread forever. `run_with_timeout` bounds it and
        // kills the whole process group past the ceiling.
        crate::omega_cli::run_with_timeout(&args, crate::omega_cli::cli_timeout())
    })
    .await
    .map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({ "error": format!("new task panicked: {e}") })),
        )
    })?
    .map_err(|e| {
        if crate::omega_cli::is_timeout(&e) {
            gateway_timeout(e.to_string())
        } else {
            (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": format!("failed to spawn omega: {e}") })),
            )
        }
    })?;

    // Non-zero exit → 502, never fabricate a session.
    //
    // M-1 (Codex cross-model review, 2026-08-11; gap found during
    // whole-branch review, same vulnerability class as the review's 5
    // originally-listed sites): the raw stdout/stderr used to be echoed
    // straight into the HTTP response, which can leak environment-derived
    // secrets or other sensitive text a future CLI diagnostic writes. The
    // FULL raw text still goes to the gateway's own tracing log; the client
    // only ever sees a generic, sanitized message.
    if !output.success {
        tracing::error!(
            name = %name,
            stdout = %output.stdout,
            stderr = %output.stderr,
            "omega new failed"
        );
        return Err((
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({ "error": "omega new failed (see gateway logs)" })),
        ));
    }

    Ok(Json(CreateSessionResponse {
        name,
        agent: req.agent,
        output: output.stdout,
    }))
}

#[cfg(test)]
mod valid_session_name_tests {
    use super::valid_session_name;

    #[test]
    fn accepts_a_real_session_name() {
        assert!(valid_session_name("oracle-Foo-1"));
    }

    #[test]
    fn rejects_path_traversal() {
        assert!(!valid_session_name("../etc"));
    }

    #[test]
    fn rejects_slash() {
        assert!(!valid_session_name("foo/bar"));
    }

    #[test]
    fn rejects_empty() {
        assert!(!valid_session_name(""));
    }

    #[test]
    fn rejects_very_long_name() {
        let long = "a".repeat(201);
        assert!(!valid_session_name(&long));
    }
}

#[cfg(test)]
mod resolved_oracle_name_tests {
    use super::resolved_oracle_name;

    #[test]
    fn extracts_the_resolved_name_from_the_exact_cmd_kill_line() {
        let stdout = "[i] Verba-3 resolved to the oracle session oracle-Verba-3\nKilled session: oracle-Verba-3\n";
        assert_eq!(resolved_oracle_name(stdout), Some("oracle-Verba-3"));
    }

    #[test]
    fn returns_none_when_no_resolution_line_present() {
        let stdout = "Killed session: worker-Foo-1\n";
        assert_eq!(resolved_oracle_name(stdout), None);
    }

    #[test]
    fn returns_none_on_empty_stdout() {
        assert_eq!(resolved_oracle_name(""), None);
    }
}

#[cfg(test)]
mod valid_new_session_name_tests {
    use super::valid_new_session_name;

    #[test]
    fn accepts_a_plain_slug() {
        assert!(valid_new_session_name("my-renamed-session_1"));
    }

    #[test]
    fn accepts_a_name_with_an_internal_dash() {
        assert!(valid_new_session_name("my-session"));
    }

    #[test]
    fn rejects_leading_dash() {
        assert!(!valid_new_session_name("-q"));
    }

    #[test]
    fn rejects_leading_dot() {
        assert!(!valid_new_session_name(".hidden"));
    }

    #[test]
    fn rejects_dot() {
        assert!(!valid_new_session_name("foo.bar"));
    }

    #[test]
    fn rejects_colon() {
        assert!(!valid_new_session_name("foo:bar"));
    }

    #[test]
    fn rejects_slash() {
        assert!(!valid_new_session_name("foo/bar"));
    }

    #[test]
    fn rejects_whitespace() {
        assert!(!valid_new_session_name("foo bar"));
    }

    #[test]
    fn rejects_empty() {
        assert!(!valid_new_session_name(""));
    }

    #[test]
    fn rejects_over_100_chars() {
        let long = "a".repeat(101);
        assert!(!valid_new_session_name(&long));
    }

    #[test]
    fn accepts_exactly_100_chars() {
        let ok = "a".repeat(100);
        assert!(valid_new_session_name(&ok));
    }
}

#[cfg(test)]
mod dir_under_home_tests {
    use super::dir_under_home;

    // `dir_under_home` reads the process-global `HOME` env var (via
    // `dirs::home_dir()`); every test below mutates it, so they must never
    // run concurrently with each other or with any other test in this
    // binary that touches `HOME` (none currently do — see this crate's
    // other `LOCK` mutexes for the same discipline, e.g. `omega_cli.rs`).
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn accepts_home_itself() {
        let _g = LOCK.lock().unwrap();
        let home = tempfile::tempdir().unwrap();
        std::env::set_var("HOME", home.path());
        let resolved = dir_under_home(&home.path().display().to_string()).unwrap();
        assert_eq!(resolved, home.path());
        std::env::remove_var("HOME");
    }

    #[test]
    fn accepts_a_not_yet_existing_nested_path_under_home() {
        let _g = LOCK.lock().unwrap();
        let home = tempfile::tempdir().unwrap();
        std::env::set_var("HOME", home.path());
        let nested = home.path().join("Station").join("NewProject");
        assert!(!nested.exists());
        let resolved = dir_under_home(&nested.display().to_string()).unwrap();
        assert_eq!(resolved, nested);
        std::env::remove_var("HOME");
    }

    #[test]
    fn accepts_an_already_existing_subdir_of_home() {
        let _g = LOCK.lock().unwrap();
        let home = tempfile::tempdir().unwrap();
        std::env::set_var("HOME", home.path());
        let sub = home.path().join("Station");
        std::fs::create_dir_all(&sub).unwrap();
        let resolved = dir_under_home(&sub.display().to_string()).unwrap();
        assert_eq!(resolved, sub);
        std::env::remove_var("HOME");
    }

    #[test]
    fn rejects_a_path_outside_home() {
        let _g = LOCK.lock().unwrap();
        let home = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::env::set_var("HOME", home.path());
        let err = dir_under_home(&outside.path().display().to_string()).unwrap_err();
        assert_eq!(err.0, axum::http::StatusCode::BAD_REQUEST);
        std::env::remove_var("HOME");
    }

    #[test]
    fn rejects_a_not_yet_existing_nested_path_outside_home() {
        let _g = LOCK.lock().unwrap();
        let home = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::env::set_var("HOME", home.path());
        let nested = outside.path().join("nope").join("nested");
        let err = dir_under_home(&nested.display().to_string()).unwrap_err();
        assert_eq!(err.0, axum::http::StatusCode::BAD_REQUEST);
        std::env::remove_var("HOME");
    }

    #[test]
    fn rejects_nul_byte() {
        let _g = LOCK.lock().unwrap();
        let home = tempfile::tempdir().unwrap();
        std::env::set_var("HOME", home.path());
        let err = dir_under_home("foo\0bar").unwrap_err();
        assert_eq!(err.0, axum::http::StatusCode::BAD_REQUEST);
        std::env::remove_var("HOME");
    }

    #[test]
    fn rejects_root() {
        let _g = LOCK.lock().unwrap();
        let home = tempfile::tempdir().unwrap();
        std::env::set_var("HOME", home.path());
        let err = dir_under_home("/").unwrap_err();
        assert_eq!(err.0, axum::http::StatusCode::BAD_REQUEST);
        std::env::remove_var("HOME");
    }

    /// Cross-task consistency fix (final whole-branch review, wave8): a
    /// relative `dir` used to be validated by canonicalizing it against
    /// THIS PROCESS's own cwd, then returned as the still-relative string —
    /// but the rmux daemon that actually consumes it resolves the relative
    /// string against ITS OWN cwd, which can differ. Even a relative value
    /// that WOULD canonicalize under the current process's home must now be
    /// rejected outright: only an absolute `dir` carries a safety guarantee
    /// that survives being handed to a different process.
    #[test]
    fn rejects_relative_dir_even_when_it_would_resolve_under_home() {
        let _g = LOCK.lock().unwrap();
        let home = tempfile::tempdir().unwrap();
        std::env::set_var("HOME", home.path());
        let prev_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(home.path()).unwrap();
        std::fs::create_dir_all(home.path().join("Station")).unwrap();
        let err = dir_under_home("Station").unwrap_err();
        assert_eq!(err.0, axum::http::StatusCode::BAD_REQUEST);
        assert_eq!(err.1 .0["error"], "dir must be an absolute path");
        std::env::set_current_dir(prev_cwd).unwrap();
        std::env::remove_var("HOME");
    }

    /// Finding 1 (adversarial review round): the ancestor-walk
    /// canonicalization used to be defeatable by a `..` sequence hiding
    /// behind a not-yet-existing leading component -- every ancestor fails
    /// to canonicalize until the walk pops all the way up to `$HOME` itself
    /// (which DOES canonicalize and pass the prefix check), at which point
    /// the function returned the ORIGINAL uncanonicalized string, which
    /// still contains the escaping `..` sequences. Structural
    /// `Path::components()` rejection must catch this BEFORE the ancestor
    /// walk ever runs.
    #[test]
    fn rejects_parent_dir_component_hidden_behind_a_nonexistent_leading_component() {
        let _g = LOCK.lock().unwrap();
        let home = tempfile::tempdir().unwrap();
        std::env::set_var("HOME", home.path());
        let escaping = home
            .path()
            .join("does-not-exist-yet")
            .join("..")
            .join("..")
            .join("..")
            .join("tmp")
            .join("evil")
            .join("PWNED");
        let err = dir_under_home(&escaping.display().to_string()).unwrap_err();
        assert_eq!(err.0, axum::http::StatusCode::BAD_REQUEST);
        std::env::remove_var("HOME");
    }

    #[test]
    fn rejects_a_bare_parent_dir_component_even_under_an_existing_prefix() {
        let _g = LOCK.lock().unwrap();
        let home = tempfile::tempdir().unwrap();
        std::env::set_var("HOME", home.path());
        let escaping = home
            .path()
            .join("Station")
            .join("..")
            .join("..")
            .join("etc");
        let err = dir_under_home(&escaping.display().to_string()).unwrap_err();
        assert_eq!(err.0, axum::http::StatusCode::BAD_REQUEST);
        std::env::remove_var("HOME");
    }
}

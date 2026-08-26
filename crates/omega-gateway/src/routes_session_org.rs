//! Session organization overlay routes: `GET /v1/session-org` /
//! `PUT /v1/session-org/{name}`. See `session_org.rs` for the storage
//! layer. This file owns validation ("route validates, store trusts its
//! caller" -- the same split `routes_sessions.rs::send_keys` uses in front
//! of `rmux.rs`): `{name}` is checked BEFORE any store write, and the store
//! itself never re-validates its caller's input.

use crate::protocol::{SessionOrgEntry, SessionOrgResponse, SessionOrgUpdateRequest};
use crate::routes_sessions::valid_session_name;
use crate::server::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

type ApiError = (StatusCode, Json<serde_json::Value>);

fn bad_request(msg: impl Into<String>) -> ApiError {
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": msg.into() })),
    )
}

/// Byte-length cap on `label`/`folder`. This is app-generated freeform
/// text (a custom label or a folder name typed on a device), not
/// attacker-controlled input reaching a subprocess argv the way
/// `routes_dispatch.rs`'s `MAX_MISSION_LEN` guards -- but an unbounded
/// string still shouldn't be persisted to disk unbounded. 200 chars
/// comfortably covers any real folder path or label a human would type
/// while keeping a runaway/malicious payload from bloating
/// `session_org.json`.
const MAX_LABEL_LEN: usize = 200;
const MAX_FOLDER_LEN: usize = 200;

/// `GET /v1/session-org` — the whole overlay map. Empty when no session has
/// ever been tagged.
pub async fn get_all(State(state): State<AppState>) -> Json<SessionOrgResponse> {
    Json(SessionOrgResponse {
        entries: state.session_org.get_all(),
    })
}

/// `PUT /v1/session-org/{name}` — FULL REPLACE of that session's overlay
/// entry (standard HTTP PUT semantics, per the task brief): fields omitted
/// from the request body land as `None`/`false` in the persisted entry,
/// they are never merged with whatever was there before. `{name}` is
/// validated with the exact guard `routes_sessions.rs` uses for a live
/// session name, even though this overlay never checks the session is
/// live -- the name still becomes a JSON map key persisted to disk, so the
/// same path-traversal-shaped-input guard still applies. Returns the entry
/// actually persisted, so the caller can confirm what was saved.
pub async fn set(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<SessionOrgUpdateRequest>,
) -> Result<Json<SessionOrgEntry>, ApiError> {
    if !valid_session_name(&name) {
        return Err(bad_request("invalid session name"));
    }
    if req.label.as_ref().is_some_and(|s| s.len() > MAX_LABEL_LEN) {
        return Err(bad_request(format!(
            "label too long (max {MAX_LABEL_LEN} bytes)"
        )));
    }
    if req
        .folder
        .as_ref()
        .is_some_and(|s| s.len() > MAX_FOLDER_LEN)
    {
        return Err(bad_request(format!(
            "folder too long (max {MAX_FOLDER_LEN} bytes)"
        )));
    }

    let entry = SessionOrgEntry {
        label: req.label,
        folder: req.folder,
        pinned: req.pinned.unwrap_or(false),
    };
    state.session_org.set(&name, entry.clone());
    Ok(Json(entry))
}

//! Account CRUD + login routes: create/list/delete/default-set an isolated
//! credential slot, drive its browser-OAuth login over a WebSocket (R-STREAM
//! discipline), and the Codex headless API-key login.
//!
//! Every `{slug}` path param is validated with [`accounts::valid_slug`]
//! BEFORE touching the store or the filesystem — a slug flows straight into
//! `AccountStore::slot_dir` (an fs join) and, for the login/apikey routes,
//! into a spawned provider CLI's environment, so an unvalidated slug is a
//! path-traversal surface.

use crate::account_login::{self, AuthStatus, LoginOutcome};
use crate::accounts;
use crate::protocol::{
    Account, AccountCreateRequest, AccountKind, AccountLoginServerMsg, AccountWithStatus,
    ApiKeyRequest,
};
use crate::server::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::time::Duration;

/// Interval between `poll_login_complete` checks while a browser-OAuth login
/// is in flight. Each poll spawns a real (short-lived) provider CLI status
/// call via `spawn_blocking`, so this stays coarse enough not to hammer it.
const LOGIN_POLL_INTERVAL: Duration = Duration::from_millis(200);

fn status_str(status: AuthStatus) -> &'static str {
    match status {
        AuthStatus::LoggedIn => "logged_in",
        AuthStatus::LoggedOut => "logged_out",
        AuthStatus::Unknown => "unknown",
    }
}

/// `GET /v1/accounts` — every known account, with its LIVE auth status
/// merged in (a free call to the provider CLI's own status subcommand,
/// never a paid one). Run via `spawn_blocking`: each status check spawns a
/// real subprocess.
pub async fn list(State(state): State<AppState>) -> Json<serde_json::Value> {
    let store = state.accounts.clone();
    let with_status = tokio::task::spawn_blocking(move || {
        store
            .list()
            .into_iter()
            .map(|account| {
                let slot = store.slot_dir(&account.slug);
                let status = status_str(account_login::status(&account, &slot)).to_string();
                AccountWithStatus { account, status }
            })
            .collect::<Vec<_>>()
    })
    .await
    .unwrap_or_default();
    Json(json!({ "accounts": with_status }))
}

/// `POST /v1/accounts` — creates a new isolated credential slot.
pub async fn create(
    State(state): State<AppState>,
    Json(req): Json<AccountCreateRequest>,
) -> Result<(StatusCode, Json<Account>), StatusCode> {
    if !accounts::valid_slug(&req.slug) {
        return Err(StatusCode::BAD_REQUEST);
    }
    state
        .accounts
        .create_slot(&req.slug, &req.label, req.kind)
        .map(|account| (StatusCode::CREATED, Json(account)))
        .map_err(|_| StatusCode::BAD_REQUEST)
}

/// `DELETE /v1/accounts/{slug}` — removes the slot (registry entry + its
/// hardened credential directory).
pub async fn delete(State(state): State<AppState>, Path(slug): Path<String>) -> StatusCode {
    if !accounts::valid_slug(&slug) {
        return StatusCode::BAD_REQUEST;
    }
    if state.accounts.remove(&slug) {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

/// `POST /v1/accounts/{slug}/default` — makes `slug` the default account
/// for its kind.
pub async fn set_default(State(state): State<AppState>, Path(slug): Path<String>) -> StatusCode {
    if !accounts::valid_slug(&slug) {
        return StatusCode::BAD_REQUEST;
    }
    if state.accounts.set_default(&slug) {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

/// `POST /v1/accounts/{slug}/apikey` — the headless Codex API-key login.
/// The key is piped straight to `codex login --with-api-key`'s stdin
/// (`account_login::codex_login_with_api_key`); it is never persisted,
/// logged, or included in this route's response.
pub async fn apikey(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(req): Json<ApiKeyRequest>,
) -> StatusCode {
    if !accounts::valid_slug(&slug) {
        return StatusCode::BAD_REQUEST;
    }
    let Some(account) = state.accounts.get(&slug) else {
        return StatusCode::NOT_FOUND;
    };
    if account.kind != AccountKind::Codex {
        // codex_login_with_api_key always spawns the codex CLI; running it
        // against a Claude slot would write Codex credentials into a Claude
        // account's directory.
        return StatusCode::BAD_REQUEST;
    }
    let slot = state.accounts.slot_dir(&slug);
    let api_key = req.api_key;
    let result = tokio::task::spawn_blocking(move || {
        account_login::codex_login_with_api_key(&slot, &api_key)
    })
    .await;
    match result {
        Ok(Ok(())) => StatusCode::OK,
        Ok(Err(e)) => {
            // `e` is codex's stderr/exit status, never the key itself
            // (codex_login_with_api_key pipes the key to stdin, it is never
            // echoed into its output) — safe to log as a breadcrumb.
            tracing::warn!("codex api-key login failed for {slug}: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
        Err(e) => {
            tracing::warn!("codex api-key login task panicked for {slug}: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

/// `GET /v1/accounts/{slug}/login` — begins the provider's browser-OAuth
/// login flow and streams its progress.
pub async fn login(
    ws: WebSocketUpgrade,
    Path(slug): Path<String>,
    State(state): State<AppState>,
) -> Response {
    if !accounts::valid_slug(&slug) {
        return StatusCode::BAD_REQUEST.into_response();
    }
    ws.on_upgrade(move |socket| login_loop(socket, slug, state))
}

async fn send_login_frame(
    socket: &mut WebSocket,
    frame: &AccountLoginServerMsg,
) -> Result<(), axum::Error> {
    let text = serde_json::to_string(frame).expect("serialize AccountLoginServerMsg");
    socket.send(Message::Text(text.into())).await
}

/// Guarantees the OAuth child process from `LoginOutcome::Url` is reaped —
/// `kill()` (a no-op if it already exited on its own) then `wait()` (reaps
/// the zombie either way) — no matter which path `login_loop` exits
/// through: normal completion, a send error, or the client disconnecting
/// mid-login. Holding the child inside this guard as a local variable is
/// what makes the reap unconditional: Rust drops locals on every exit path,
/// including an early `return`, so there is no path that leaks the process.
struct ChildReaper(Option<std::process::Child>);

impl Drop for ChildReaper {
    fn drop(&mut self) {
        if let Some(mut child) = self.0.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

async fn login_loop(mut socket: WebSocket, slug: String, state: AppState) {
    let Some(account) = state.accounts.get(&slug) else {
        let _ = send_login_frame(
            &mut socket,
            &AccountLoginServerMsg::Error {
                message: "unknown account".to_string(),
            },
        )
        .await;
        let _ = socket.send(Message::Close(None)).await;
        return;
    };
    let slot = state.accounts.slot_dir(&slug);

    // begin_claude_login / begin_codex_login block (up to LOGIN_URL_WAIT)
    // and spawn a real subprocess, so this runs off the async executor.
    let begin_result = {
        let account = account.clone();
        let slot = slot.clone();
        tokio::task::spawn_blocking(move || match account.kind {
            AccountKind::Claude => account_login::begin_claude_login(&slot),
            AccountKind::Codex => account_login::begin_codex_login(&slot),
        })
        .await
    };

    let outcome = match begin_result {
        Ok(Ok(outcome)) => outcome,
        Ok(Err(e)) => {
            let _ = send_login_frame(
                &mut socket,
                &AccountLoginServerMsg::Error {
                    message: e.to_string(),
                },
            )
            .await;
            let _ = socket.send(Message::Close(None)).await;
            return;
        }
        Err(e) => {
            let _ = send_login_frame(
                &mut socket,
                &AccountLoginServerMsg::Error {
                    message: format!("login task panicked: {e}"),
                },
            )
            .await;
            let _ = socket.send(Message::Close(None)).await;
            return;
        }
    };

    // Holds the OAuth child (when there is one) for the rest of this
    // function's scope; its Drop impl reaps it on every exit path below.
    let _child_reaper;
    match outcome {
        LoginOutcome::NeedsBox => {
            let _ = send_login_frame(&mut socket, &AccountLoginServerMsg::LoginNeedsBox).await;
            let _ = socket.send(Message::Close(None)).await;
            return;
        }
        LoginOutcome::Url(url, child) => {
            _child_reaper = ChildReaper(Some(child));
            if send_login_frame(&mut socket, &AccountLoginServerMsg::LoginUrl { url })
                .await
                .is_err()
            {
                return; // socket already dead; _child_reaper drops here
            }
        }
    }

    // Poll for completion until logged in, or the client goes away.
    loop {
        let logged_in = {
            let account = account.clone();
            let slot = slot.clone();
            tokio::task::spawn_blocking(move || account_login::poll_login_complete(&account, &slot))
                .await
                .unwrap_or(false)
        };
        if logged_in {
            let _ = send_login_frame(&mut socket, &AccountLoginServerMsg::LoginDone).await;
            let _ = socket.send(Message::Close(None)).await;
            return; // _child_reaper drops here
        }

        match tokio::time::timeout(LOGIN_POLL_INTERVAL, socket.recv()).await {
            Ok(None) | Ok(Some(Err(_))) => return, // client gone / socket dead
            Ok(Some(Ok(Message::Close(_)))) => return, // client closed
            Ok(Some(Ok(_))) => {}                  // other client frames: ignore, keep polling
            Err(_) => {}                           // poll interval elapsed: check status again
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_str_maps_every_variant() {
        assert_eq!(status_str(AuthStatus::LoggedIn), "logged_in");
        assert_eq!(status_str(AuthStatus::LoggedOut), "logged_out");
        assert_eq!(status_str(AuthStatus::Unknown), "unknown");
    }
}

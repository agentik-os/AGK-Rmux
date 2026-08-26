//! Chat REST + WebSocket routes: create/list/inspect chats, and stream real
//! agent turns over a persistent WebSocket (R-STREAM discipline — a parse or
//! agent error becomes an `Error` frame, never a closed socket; only a dead
//! socket or an explicit client close ends the loop).

use crate::accounts::{self, AccountStore};
use crate::chat_driver::run_turn;
use crate::protocol::{
    AccountKind, ChatAgent, ChatDetailResponse, ChatMessage, ChatMessagesPage, ChatMeta,
    ChatStreamClientMsg, ChatStreamServerMsg,
};
use crate::server::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::StatusCode,
    response::Response,
    Json,
};
use serde::Deserialize;
use serde_json::json;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct ChatCreateRequest {
    pub agent: ChatAgent,
    /// Working directory the chat's turns run in (`current_dir` for the
    /// spawned agent process, see `chat_driver.rs::agent_command`).
    /// Validated (I-1: confined under `$HOME`, no NUL byte, no `..`
    /// component, must be absolute) via `routes_sessions::dir_under_home`
    /// before the chat is created — an authed device must not be able to
    /// point a chat at `/etc`, `/`, or another project's private tree.
    pub cwd: String,
    #[serde(default)]
    pub title: Option<String>,
    /// The account slot to run this chat's turns under. `None` means
    /// "resolve the agent kind's default account per turn" (see
    /// `resolve_account_dir`). Validated (path-traversal guard) before the
    /// chat is created — an unknown slot dir is created lazily by the
    /// provider CLI, but the slug SHAPE must still be safe to join onto the
    /// accounts dir.
    #[serde(default)]
    pub account_slug: Option<String>,
}

/// Chat ids are `random_hex(8)` (see `util::random_hex`): exactly 16
/// lowercase hex characters. The `{id}` path param is otherwise unvalidated
/// before flowing into `ChatStore`'s filesystem joins, which is a
/// path-traversal surface (`../../etc/passwd`-shaped ids). Reject anything
/// that doesn't match the real id shape BEFORE the store is ever touched.
fn valid_chat_id(id: &str) -> bool {
    id.len() == 16
        && id
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

pub async fn list(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(json!({ "chats": state.chats.list() }))
}

/// Maps a chat's agent to the account kind its pinned slot must have (Claude
/// chat -> Claude account, Codex -> Codex). The single mapping shared by
/// `create`'s pin validation and `resolve_account_dir`'s default-account
/// lookup, so the Claude<->Codex correspondence lives in exactly one place.
fn account_kind_for(agent: ChatAgent) -> AccountKind {
    match agent {
        ChatAgent::Claude => AccountKind::Claude,
        ChatAgent::Codex => AccountKind::Codex,
    }
}

pub async fn create(
    State(state): State<AppState>,
    Json(req): Json<ChatCreateRequest>,
) -> Result<(StatusCode, Json<ChatMeta>), (StatusCode, Json<serde_json::Value>)> {
    // I-1: `cwd` must resolve under `$HOME` before the chat is ever
    // created — reusing the same confinement guard `routes_sessions.rs`
    // already applies to a session's `dir`. `dir_under_home` returns the
    // caller's ORIGINAL, uncanonicalized-but-validated path (never the
    // canonicalized ancestor, see its own doc comment); that is what gets
    // stored, so the persisted `cwd` can never diverge from what was
    // actually checked here.
    let cwd = crate::routes_sessions::dir_under_home(&req.cwd)?
        .to_string_lossy()
        .into_owned();

    if let Some(slug) = &req.account_slug {
        if !accounts::valid_slug(slug) {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "invalid account_slug" })),
            ));
        }
        let Some(account) = state.accounts.get(slug) else {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "account not found" })),
            ));
        };
        if account.kind != account_kind_for(req.agent) {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "account kind does not match agent" })),
            ));
        }
    }
    let meta = state
        .chats
        .create(req.agent, cwd, req.title, req.account_slug);
    Ok((StatusCode::CREATED, Json(meta)))
}

/// How many of the most recent messages `GET /v1/chats/{id}` inlines
/// directly. Older history is reachable by paging through
/// `GET /v1/chats/{id}/messages` with the returned `next_cursor`.
const DETAIL_WINDOW: usize = 50;

/// Default page size for `GET /v1/chats/{id}/messages` when `limit` is
/// omitted from the query string.
const DEFAULT_MESSAGES_LIMIT: usize = 50;

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ChatDetailResponse>, StatusCode> {
    if !valid_chat_id(&id) {
        return Err(StatusCode::NOT_FOUND);
    }
    let meta = state.chats.get(&id).ok_or(StatusCode::NOT_FOUND)?;
    // tail_page hands back newest-first; this endpoint's contract (and the
    // existing HTTP tests) is chronological, oldest-first, so reverse it.
    let (mut messages, next_cursor) = state.chats.tail_page(&id, None, DETAIL_WINDOW);
    messages.reverse();
    Ok(Json(ChatDetailResponse {
        meta,
        messages,
        next_cursor,
    }))
}

#[derive(Deserialize)]
pub struct MessagesQuery {
    #[serde(default)]
    pub before: Option<u64>,
    #[serde(default)]
    pub limit: Option<usize>,
}

pub async fn messages(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<MessagesQuery>,
) -> Result<Json<ChatMessagesPage>, StatusCode> {
    if !valid_chat_id(&id) {
        return Err(StatusCode::NOT_FOUND);
    }
    // Chat must exist at all before paginating it (mirrors `get`'s guard).
    state.chats.get(&id).ok_or(StatusCode::NOT_FOUND)?;
    let limit = query.limit.unwrap_or(DEFAULT_MESSAGES_LIMIT);
    let (messages, next_cursor) = state.chats.tail_page(&id, query.before, limit);
    Ok(Json(ChatMessagesPage {
        messages,
        next_cursor,
    }))
}

pub async fn stream(
    ws: WebSocketUpgrade,
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| stream_loop(socket, id, state))
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Resolves which account credential slot (if any) this chat's turn should
/// run under: the chat's own `account_slug` when it chose one at creation,
/// else the agent kind's current default account, else `None` (the box's
/// ambient provider config is used — today's pre-Task-3 behavior).
fn resolve_account_dir(accounts: &AccountStore, meta: &ChatMeta) -> Option<PathBuf> {
    if let Some(slug) = &meta.account_slug {
        return Some(accounts.slot_dir(slug));
    }
    accounts
        .default_for(account_kind_for(meta.agent))
        .map(|a| accounts.slot_dir(&a.slug))
}

/// Serializes and sends one server frame. `Err` means the socket is dead.
async fn send_frame(
    socket: &mut WebSocket,
    frame: &ChatStreamServerMsg,
) -> Result<(), axum::Error> {
    let text = serde_json::to_string(frame).expect("serialize ChatStreamServerMsg");
    socket.send(Message::Text(text.into())).await
}

/// Sends `Error{message}` then `TurnDone`, the "can't start a turn" pair used
/// by both the unknown-chat and busy-semaphore short-circuits. Returns
/// `Err(())` if the socket died mid-send, so the caller can stop the loop.
async fn send_error_turn_done(
    socket: &mut WebSocket,
    message: impl Into<String>,
) -> Result<(), ()> {
    send_frame(
        socket,
        &ChatStreamServerMsg::Error {
            message: message.into(),
        },
    )
    .await
    .map_err(|_| ())?;
    send_frame(socket, &ChatStreamServerMsg::TurnDone)
        .await
        .map_err(|_| ())
}

/// I-5 RAII guard: marks chat `id`'s turn as ended (`ChatStore::end_turn`)
/// on EVERY exit path from a turn (normal completion, an early `return`
/// out of the read loop, or a panic unwinding through this frame), the same
/// "fires on every exit path including a dropped future" reasoning
/// `routes_duo.rs`'s `KillGroupOnDrop` documents for its own per-process
/// cleanup. Holds an owned `Arc<ChatStore>` clone + owned id so it has no
/// lifetime tied to the loop iteration's other borrows.
struct TurnGuard {
    chats: std::sync::Arc<crate::chat_store::ChatStore>,
    id: String,
}

impl Drop for TurnGuard {
    fn drop(&mut self) {
        self.chats.end_turn(&self.id);
    }
}

async fn stream_loop(mut socket: WebSocket, id: String, state: AppState) {
    if !valid_chat_id(&id) {
        let _ = send_frame(
            &mut socket,
            &ChatStreamServerMsg::Error {
                message: "invalid chat id".to_string(),
            },
        )
        .await;
        let _ = socket.send(Message::Close(None)).await;
        return;
    }
    // R-STREAM: this loop never exits on error; only a dead socket or an
    // explicit client close ends it. The socket supports multiple turns.
    loop {
        let text = match socket.recv().await {
            Some(Ok(Message::Text(text))) => text.to_string(),
            Some(Ok(Message::Close(_))) | None => return, // client closed or gone
            Some(Ok(_)) => continue,                      // ping/pong/binary: not a turn
            Some(Err(_)) => return,                       // socket error: dead
        };

        let client_msg: ChatStreamClientMsg = match serde_json::from_str(&text) {
            Ok(m) => m,
            Err(e) => {
                if send_error_turn_done(&mut socket, format!("bad client message: {e}"))
                    .await
                    .is_err()
                {
                    return;
                }
                continue;
            }
        };
        let ChatStreamClientMsg::UserMessage { text: user_text } = client_msg;

        let Some(meta) = state.chats.get(&id) else {
            if send_error_turn_done(&mut socket, "chat not found")
                .await
                .is_err()
            {
                return;
            }
            continue;
        };

        state.chats.append_message(
            &id,
            &ChatMessage {
                role: "user".to_string(),
                text: user_text.clone(),
                ts: now(),
            },
        );

        let Ok(permit) = state.chat_permits.clone().try_acquire_owned() else {
            if send_error_turn_done(&mut socket, "busy, too many active chats")
                .await
                .is_err()
            {
                return;
            }
            continue;
        };

        // I-5: `chat_permits` above is only a GLOBAL cap — it does not stop
        // two different WebSocket connections to the SAME chat id from each
        // starting a turn concurrently, racing the transcript append and
        // `set_provider_session` writes. This per-chat guard rejects a
        // second concurrent turn on one chat outright rather than letting
        // both run.
        if !state.chats.try_start_turn(&id) {
            if send_error_turn_done(&mut socket, "a turn is already active on this chat")
                .await
                .is_err()
            {
                return;
            }
            continue;
        }
        let _turn_guard = TurnGuard {
            chats: state.chats.clone(),
            id: id.clone(),
        };

        let (tx, mut rx) = tokio::sync::mpsc::channel::<ChatStreamServerMsg>(64);
        let timeout = std::time::Duration::from_millis(state.cfg.chat_turn_timeout_ms);
        let account_dir = resolve_account_dir(&state.accounts, &meta);
        let turn_meta = meta.clone();
        let turn_handle = tokio::spawn(async move {
            let _permit = permit; // held for the whole turn, released on drop
            run_turn(
                &turn_meta,
                &user_text,
                None,
                account_dir.as_deref(),
                timeout,
                tx,
            )
            .await
        });

        let mut assistant_text = String::new();
        let mut turn_done_seen = false;
        let mut socket_dead = false;
        while let Some(frame) = rx.recv().await {
            // Defensive dedupe (hardening from the Task 3 re-review): run_turn
            // may emit a trailing TurnDone after the stream's own. The client
            // must see exactly one TurnDone per turn, and nothing after the
            // first is persisted or forwarded.
            if turn_done_seen {
                continue;
            }
            match &frame {
                ChatStreamServerMsg::Delta { text }
                | ChatStreamServerMsg::AssistantMessage { text } => {
                    assistant_text.push_str(text);
                }
                ChatStreamServerMsg::TurnDone => turn_done_seen = true,
                ChatStreamServerMsg::ToolEvent { .. } | ChatStreamServerMsg::Error { .. } => {}
            }
            if !socket_dead && send_frame(&mut socket, &frame).await.is_err() {
                socket_dead = true;
            }
        }

        let provider_session_id = turn_handle.await.ok().flatten();

        if !assistant_text.is_empty() {
            state.chats.append_message(
                &id,
                &ChatMessage {
                    role: "assistant".to_string(),
                    text: assistant_text,
                    ts: now(),
                },
            );
        }
        if let Some(sid) = provider_session_id {
            state.chats.set_provider_session(&id, &sid);
        }

        if socket_dead {
            return;
        }
    }
}

#[cfg(test)]
mod valid_chat_id_tests {
    use super::valid_chat_id;

    #[test]
    fn accepts_a_real_16_hex_id() {
        assert!(valid_chat_id("0123456789abcdef"));
    }

    #[test]
    fn rejects_path_traversal() {
        assert!(!valid_chat_id("../etc"));
        assert!(!valid_chat_id("../../etc/passwd"));
    }

    #[test]
    fn rejects_uppercase_hex() {
        assert!(!valid_chat_id("ABCDEF0123456789"));
    }

    #[test]
    fn rejects_short_id() {
        assert!(!valid_chat_id("0123456789"));
    }

    #[test]
    fn rejects_long_id() {
        assert!(!valid_chat_id("0123456789abcdef00"));
    }

    #[test]
    fn rejects_non_hex_same_length_id() {
        assert!(!valid_chat_id("zzzzzzzzzzzzzzzz"));
    }
}

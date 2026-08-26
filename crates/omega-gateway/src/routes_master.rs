//! `GET /v1/master/chat` — a WebSocket that lets the app drive the AISB
//! Master conversation the same way `omega aisb-chat`'s local REPL does.
//!
//! GROUND TRUTH (confirmed against `crates/omega-cli/src/main.rs::
//! cmd_aisb_chat`, ~line 8747 — not guessed): `omega aisb-chat` does NOT
//! spawn a subprocess or talk to any brain directly. It (1) appends a JSON
//! line `{"text": "<msg>", "ts": "<rfc3339>"}` to
//! `~/.omega/state/aisb-local-inbox.jsonl`, (2) records the byte length of
//! `~/.omega/state/aisb-conversation.log` BEFORE the append, (3) polls that
//! log every 500ms up to 180 times (90s) for its length to grow, and (4) on
//! growth, prints the delta as "the response"; on no growth within budget,
//! reports "no response within 90s — check the bridge". This endpoint
//! mirrors that exact protocol rather than inventing a different call path
//! (e.g. shelling a headless prompt), which would break the "same brain,
//! same response" contract `cmd_aisb_chat`'s own doc comment describes and
//! diverge from what OmegaOS itself ships.
//!
//! CONFIRMED SEPARATELY (grepped the whole box: every Rust crate and the Bun
//! Telegram bot's brain source): nothing today actually reads
//! `aisb-local-inbox.jsonl`. A real round-trip through this endpoint
//! currently times out with a clean `Timeout` frame, exactly like the CLI
//! itself would — that is an honest, pre-existing wiring gap in what
//! OmegaOS ships today, not a bug in this endpoint. Building the real
//! mechanism anyway is the correct call: it is genuinely what the CLI
//! documents and does, and a fake shortcut here would silently diverge from
//! it the moment the real bridge IS wired up.
//!
//! LIVENESS GATE: `aisb-master` (`omega_core::aisb::MASTER_SESSION_NAME`) is
//! a read-only `tail -F` viewer, not the brain — but its liveness is still
//! the right gate, because if that session isn't running, nobody is
//! watching this conversation at all. Checked via `rmux::list_sessions()`,
//! the same call `routes_oracles.rs` already uses for liveness. When the
//! master isn't live, the inbox file is NEVER touched (create nor append) —
//! only a `NotRunning` frame is sent, and the loop keeps waiting for more
//! client messages rather than closing (the operator might start the master
//! while the app stays connected; a reconnect shouldn't be required).
//!
//! HOME RESOLUTION (load-bearing, read carefully): the CLI resolves both
//! paths via `dirs::home_dir()` falling back to the raw `$HOME` env var —
//! NEVER `crate::config::home_dir()` (which additionally honors
//! `$OMEGA_DIR`/`$OMEGA_HOME`, a DIFFERENT resolution used elsewhere in this
//! gateway crate, e.g. `routes_audit::resolve_project_path`). This endpoint
//! must watch/write the exact files a real `aisb-master` setup uses, so
//! [`aisb_home_dir`] matches the CLI's resolution bit for bit. A hermetic
//! test therefore overrides the `$HOME` env var itself (the same pattern
//! `tests/box_test.rs` uses for `UsageSnapshot::read()`), LOCK-guarded
//! against every other test in this crate mutating the same global env var.

use crate::protocol::MasterChatMsg;
use crate::server::AppState;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::OwnedSemaphorePermit;

/// Byte-length cap on one inbound WS text message before it is JSON-encoded
/// into the inbox file — matching `routes_dispatch.rs::MAX_MISSION_LEN`'s
/// precedent (a free-text field with no other bound gets an explicit cap
/// rather than an unbounded write to disk).
const MAX_MASTER_CHAT_MESSAGE_LEN: usize = 8000;

/// Resolves the home directory the SAME way `cmd_aisb_chat` does:
/// `dirs::home_dir()`, falling back to the raw `$HOME` env var, falling back
/// to `"."`. Deliberately duplicated rather than reusing
/// `crate::config::home_dir()` — see this module's doc comment for why the
/// two resolutions must stay distinct.
fn aisb_home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| {
        std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
    })
}

fn aisb_conversation_log_path() -> PathBuf {
    aisb_home_dir().join(".omega/state/aisb-conversation.log")
}

fn aisb_inbox_path() -> PathBuf {
    aisb_home_dir().join(".omega/state/aisb-local-inbox.jsonl")
}

/// Poll budget before giving up on a round-trip — the CLI's real default is
/// 180 attempts (90s at 500ms/tick). Overridable via
/// `OMEGA_AISB_POLL_ATTEMPTS` so a timeout test can run in milliseconds
/// instead of burning 90 real seconds; production (no env var set) matches
/// the CLI exactly.
fn poll_attempts() -> u32 {
    std::env::var("OMEGA_AISB_POLL_ATTEMPTS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(180)
}

/// Poll interval — CLI default 500ms. Overridable via
/// `OMEGA_AISB_POLL_INTERVAL_MS` for the same reason as [`poll_attempts`].
fn poll_interval_ms() -> u64 {
    std::env::var("OMEGA_AISB_POLL_INTERVAL_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(500)
}

/// Acquires ONE permit for the WHOLE connection lifetime BEFORE ever
/// upgrading — mirrors `routes_audit.rs::stream`'s reject-before-upgrade
/// shape (branch between `ws.on_upgrade(...)` and `code.into_response()`
/// rather than upgrading unconditionally and erroring inside the loop). When
/// the pool ([`crate::server::AppState::master_chat_permits`]) is
/// exhausted, this returns a bare `429` and never upgrades. The acquired
/// permit is moved into `master_chat_loop` and released automatically when
/// that future completes (client disconnect, dead socket, or the CLI-mirror
/// loop's own return), so a client doing several turns on one open socket
/// only ever holds one permit.
pub async fn chat(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    let Ok(permit) = state.master_chat_permits.clone().try_acquire_owned() else {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    };
    ws.on_upgrade(move |socket| master_chat_loop(socket, permit))
}

/// Serializes and sends one frame. `Err` means the socket is dead.
async fn send_frame(socket: &mut WebSocket, frame: &MasterChatMsg) -> Result<(), axum::Error> {
    let text = serde_json::to_string(frame).expect("serialize MasterChatMsg");
    socket.send(Message::Text(text.into())).await
}

/// Checks `aisb-master` liveness via `rmux::list_sessions()`, off the async
/// runtime thread (`spawn_blocking`, the same idiom `routes_oracles::list`
/// uses for the identical call). A failed/degraded rmux read is treated as
/// "not live", never a panic or 500 — same posture `routes_oracles.rs`
/// documents for its own liveness probe.
async fn master_is_live() -> bool {
    match tokio::task::spawn_blocking(crate::rmux::list_sessions).await {
        Ok(Ok(names)) => names
            .iter()
            .any(|n| n == omega_core::aisb::MASTER_SESSION_NAME),
        _ => false,
    }
}

/// Appends `{"text": text, "ts": <rfc3339>}` to the inbox JSONL, creating the
/// file and its parent directory if missing — identical shape to
/// `cmd_aisb_chat`'s own inbox write. Runs blocking file I/O, so callers
/// dispatch it via `spawn_blocking`.
fn append_to_inbox(inbox: &std::path::Path, text: &str) -> std::io::Result<()> {
    if let Some(parent) = inbox.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let entry = serde_json::json!({
        "text": text,
        "ts": chrono::Utc::now().to_rfc3339(),
    });
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(inbox)?;
    writeln!(f, "{entry}")
}

/// Current byte length of the conversation log, or 0 if it doesn't exist yet
/// (a missing log is a normal "nothing written yet" state, never an error —
/// same posture `cmd_aisb_chat` takes with `.unwrap_or(0)`).
fn log_len(log: &std::path::Path) -> u64 {
    std::fs::metadata(log).map(|m| m.len()).unwrap_or(0)
}

/// Reads the growth past `before_len`, or `None` if the log hasn't grown (or
/// became unreadable — treated as "no growth yet" so a transient read glitch
/// never crashes the poll loop, per the task brief).
///
/// `.get(start..)` (not `&content[start..]`) because `before_len` is a
/// byte-length snapshot taken on an EARLIER read of this file, not something
/// re-validated against the CURRENT content's char boundaries — this
/// endpoint's normal path is append-only, but if the log is ever truncated
/// and rewritten between the snapshot and this read (e.g. a rotation), a
/// stale `before_len` can land in the middle of a multi-byte UTF-8
/// character. A raw slice there panics ("byte index N is not a char
/// boundary"); `.get` returns `None` on a non-boundary (or out-of-range)
/// start instead, which this function already treats as "no growth yet" —
/// the exact same defensive idiom `routes_box.rs::parse_doctor_output` uses
/// for adversarial subprocess output (see its doc comment).
fn read_growth(log: &std::path::Path, before_len: u64) -> Option<String> {
    let now_len = log_len(log);
    if now_len <= before_len {
        return None;
    }
    let content = std::fs::read_to_string(log).ok()?;
    let start = (before_len.min(content.len() as u64)) as usize;
    let delta = content.get(start..)?.trim_start_matches('\n');
    if delta.is_empty() {
        None
    } else {
        Some(delta.to_string())
    }
}

/// One round-trip: append `text` to the inbox, then poll the conversation
/// log for growth up to [`poll_attempts`] times at [`poll_interval_ms`]
/// spacing. Returns the reply text, or `None` on timeout. Blocking file I/O
/// is dispatched via `spawn_blocking` per iteration (the same "keep the
/// async runtime thread free" discipline every other route in this crate
/// follows for filesystem/subprocess work).
async fn run_round_trip(text: String) -> Option<String> {
    let inbox = aisb_inbox_path();
    let log = aisb_conversation_log_path();

    let log_for_len = log.clone();
    let before_len = tokio::task::spawn_blocking(move || log_len(&log_for_len))
        .await
        .unwrap_or(0);

    let inbox_for_write = inbox.clone();
    let text_for_write = text.clone();
    if tokio::task::spawn_blocking(move || append_to_inbox(&inbox_for_write, &text_for_write))
        .await
        .is_err()
    {
        return None;
    }

    let attempts = poll_attempts();
    let interval = Duration::from_millis(poll_interval_ms());
    for _ in 0..attempts {
        tokio::time::sleep(interval).await;
        let log_for_read = log.clone();
        let growth = tokio::task::spawn_blocking(move || read_growth(&log_for_read, before_len))
            .await
            .ok()
            .flatten();
        if let Some(delta) = growth {
            return Some(delta);
        }
    }
    None
}

/// R-STREAM discipline: never exits on error, only on a dead/closed socket.
/// Each inbound client text message is one round-trip (liveness check ->
/// inbox append -> log poll -> reply/timeout frame); the loop then waits for
/// the next client message, so the socket supports many turns, exactly like
/// the CLI's REPL. No unbounded background task outlives the connection —
/// the poll for one round-trip runs to completion inside this same loop
/// iteration and nothing is spawned that keeps running after `chat` returns.
///
/// `_permit` is held for the WHOLE lifetime of this function (i.e. the whole
/// WebSocket connection, however many turns it carries) and released
/// automatically when this future completes, whatever the reason — see
/// [`chat`]'s doc comment.
async fn master_chat_loop(mut socket: WebSocket, _permit: OwnedSemaphorePermit) {
    loop {
        let text = match socket.recv().await {
            Some(Ok(Message::Text(text))) => text.to_string(),
            Some(Ok(Message::Close(_))) | None => return, // client closed or gone
            Some(Ok(_)) => continue,                      // ping/pong/binary: not a line
            Some(Err(_)) => return,                       // socket error: dead
        };
        if text.is_empty() {
            continue;
        }
        // Oversized inbound message: never touch the inbox file for it. Send
        // a rejection frame and keep the loop open for the next client
        // message — same "don't close over one bad message" posture as the
        // `NotRunning` branch below, not a hard disconnect.
        if text.len() > MAX_MASTER_CHAT_MESSAGE_LEN {
            let frame = MasterChatMsg::Error {
                message: format!("message too long (max {MAX_MASTER_CHAT_MESSAGE_LEN} bytes)"),
            };
            if send_frame(&mut socket, &frame).await.is_err() {
                return;
            }
            continue;
        }

        if !master_is_live().await {
            if send_frame(&mut socket, &MasterChatMsg::NotRunning)
                .await
                .is_err()
            {
                return;
            }
            continue; // don't touch the inbox; wait for more client messages
        }

        let frame = match run_round_trip(text).await {
            Some(reply) => MasterChatMsg::Reply { text: reply },
            None => MasterChatMsg::Timeout,
        };
        if send_frame(&mut socket, &frame).await.is_err() {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_growth_none_when_log_has_not_grown() {
        let dir = tempfile::tempdir().unwrap();
        let log = dir.path().join("log");
        std::fs::write(&log, "hello").unwrap();
        assert_eq!(read_growth(&log, 5), None);
    }

    #[test]
    fn read_growth_some_delta_when_log_grew() {
        let dir = tempfile::tempdir().unwrap();
        let log = dir.path().join("log");
        std::fs::write(&log, "hello world").unwrap();
        assert_eq!(read_growth(&log, 5), Some(" world".to_string()));
    }

    #[test]
    fn read_growth_trims_leading_newline_noise() {
        let dir = tempfile::tempdir().unwrap();
        let log = dir.path().join("log");
        std::fs::write(&log, "hello\nworld").unwrap();
        assert_eq!(read_growth(&log, 5), Some("world".to_string()));
    }

    #[test]
    fn read_growth_none_when_log_missing() {
        let dir = tempfile::tempdir().unwrap();
        let log = dir.path().join("does-not-exist");
        assert_eq!(read_growth(&log, 0), None);
    }

    /// Regression for the UTF-8 char-boundary panic: `before_len` is a
    /// byte-length snapshot from an EARLIER read, so if the log is ever
    /// truncated and rewritten before this read (a rotation, not the normal
    /// append-only path), the snapshot can land in the middle of a
    /// multi-byte character. "a\u{e9}" ("aé") is 3 bytes — 'a' at index 0,
    /// then the two bytes of 'é' at indices 1 and 2 — so byte index 2 is NOT
    /// a char boundary. Before the fix, `content[2..]` panicked here with
    /// "byte index 2 is not a char boundary"; `read_growth` must return
    /// `None` instead.
    #[test]
    fn read_growth_none_on_stale_snapshot_mid_char_boundary() {
        let dir = tempfile::tempdir().unwrap();
        let log = dir.path().join("log");
        std::fs::write(&log, "a\u{e9}").unwrap(); // "aé", 3 bytes, boundaries at 0/1/3
        assert_eq!(read_growth(&log, 2), None);
    }

    #[test]
    fn log_len_zero_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(log_len(&dir.path().join("nope")), 0);
    }

    #[test]
    fn append_to_inbox_creates_parent_dir_and_file() {
        let dir = tempfile::tempdir().unwrap();
        let inbox = dir.path().join("nested/inbox.jsonl");
        append_to_inbox(&inbox, "hi").unwrap();
        let content = std::fs::read_to_string(&inbox).unwrap();
        assert!(content.contains("\"text\":\"hi\""), "got: {content}");
        let parsed: serde_json::Value = serde_json::from_str(content.trim()).unwrap();
        assert_eq!(parsed["text"], "hi");
        assert!(parsed["ts"].as_str().is_some());
    }

    #[test]
    fn append_to_inbox_appends_multiple_lines() {
        let dir = tempfile::tempdir().unwrap();
        let inbox = dir.path().join("inbox.jsonl");
        append_to_inbox(&inbox, "one").unwrap();
        append_to_inbox(&inbox, "two").unwrap();
        let content = std::fs::read_to_string(&inbox).unwrap();
        assert_eq!(content.lines().count(), 2);
    }
}

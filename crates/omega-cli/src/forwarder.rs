//! Ordered keystroke forwarder.
//!
//! The TUI chat-focus path forwards one keystroke per event to a target rmux
//! session. A previous optimization fired each forward on its own
//! `tokio::spawn`, which — on the multi-threaded `#[tokio::main]` runtime —
//! let N independent tasks race to the SDK transport's single FIFO actor on
//! different worker threads. Fast typing / drain batches could therefore be
//! delivered OUT OF ORDER (e.g. "abc" landing as "acb").
//!
//! This module replaces those racing spawns with ONE consumer task draining a
//! single FIFO mpsc channel. Because exactly one task reaches the transport,
//! ordering is guaranteed. Adjacent `Text` messages for the same session are
//! coalesced into a single send for efficiency; a `Key` or a session switch
//! flushes pending text first, preserving exact interleaving order.

use omega_core::session::SessionManager;
use tokio::sync::mpsc::{self, error::TryRecvError, UnboundedSender};

/// Error sink shared with the TUI loop; the consumer writes here on failure
/// and the loop drains it into `app.status_message` each tick.
pub type StatusSink = std::sync::Arc<std::sync::Mutex<Option<String>>>;

/// A single forward request. `Text` is coalesce-able; `Key` is a discrete
/// named token (Enter, Up, Space, BSpace…) that must not be merged or reordered.
pub enum ForwardMsg {
    Text {
        session: String,
        text: String,
    },
    Key {
        session: String,
        key: String,
    },
    /// A user paste — forwarded as ONE bracketed-paste block (no auto-Enter)
    /// so the target app buffers it instead of submitting on every embedded
    /// newline. Flushes pending coalesced text first to preserve order.
    Paste {
        session: String,
        text: String,
    },
}

fn set_err(sink: &StatusSink, msg: String) {
    if let Ok(mut g) = sink.lock() {
        *g = Some(msg);
    }
}

/// Flush any pending coalesced text for a session, in order, before the next
/// non-text or different-session message is dispatched.
async fn flush_text(pend: &mut Option<(String, String)>, mgr: &SessionManager, sink: &StatusSink) {
    if let Some((session, text)) = pend.take() {
        if let Err(e) = mgr.send_text_raw(&session, &text).await {
            set_err(sink, format!("Forward failed: {}", e));
        }
    }
}

/// Spawn the single ordered consumer task and return the channel sender.
/// The event loop only calls `tx.send(..)` (synchronous, non-blocking on an
/// unbounded channel), so the hot path stays instant while delivery order is
/// guaranteed by the lone consumer.
pub fn spawn_forwarder(mgr: SessionManager, sink: StatusSink) -> UnboundedSender<ForwardMsg> {
    // A host terminal / SSH client may split ONE user paste into several
    // back-to-back bracketed-paste segments (observed with mobile clients such
    // as Termius). Each segment reaches us as its own `ForwardMsg::Paste`;
    // forwarded separately they become multiple "[Pasted text]" placeholders in
    // the target app (Claude Code).
    //
    // Tuning: the segments of a single user paste arrive within MICROSECONDS
    // of each other (it's the same physical event, just the client chunked it).
    // The original 80ms window was wildly over-conservative — it imposed a
    // perceptible 80ms delay on EVERY paste while catching genuine splits with
    // 100x more headroom than needed. 12ms is plenty for the legitimate split
    // case (still single-digit ms after the chunker), and invisible to the
    // human eye. A deliberate second paste is far more than 12ms apart, so
    // it never gets wrongly merged.
    const PASTE_COALESCE_WINDOW: std::time::Duration = std::time::Duration::from_millis(12);

    let (tx, mut rx) = mpsc::unbounded_channel::<ForwardMsg>();
    tokio::spawn(async move {
        // Pending coalesced text: (session, accumulated_text).
        let mut pend: Option<(String, String)> = None;
        // A message pulled off the channel while coalescing a paste that turned
        // out to belong to the NEXT unit of work — handled first on the
        // following iteration instead of being dropped.
        let mut carry: Option<ForwardMsg> = None;
        loop {
            // With pending text, drain greedily (try_recv) to coalesce; when
            // the queue is momentarily empty, flush and block on the next recv.
            let next = if let Some(m) = carry.take() {
                m
            } else if pend.is_some() {
                match rx.try_recv() {
                    Ok(m) => m,
                    Err(TryRecvError::Empty) => {
                        flush_text(&mut pend, &mgr, &sink).await;
                        continue;
                    }
                    Err(TryRecvError::Disconnected) => {
                        flush_text(&mut pend, &mgr, &sink).await;
                        break;
                    }
                }
            } else {
                match rx.recv().await {
                    Some(m) => m,
                    None => break,
                }
            };

            match next {
                ForwardMsg::Text { session, text } => match pend.as_mut() {
                    Some((s, buf)) if *s == session => buf.push_str(&text),
                    _ => {
                        flush_text(&mut pend, &mgr, &sink).await;
                        pend = Some((session, text));
                    }
                },
                ForwardMsg::Key { session, key } => {
                    // Flush queued text BEFORE the key so interleaving order
                    // (e.g. "ab" then BackSpace) is preserved exactly.
                    flush_text(&mut pend, &mgr, &sink).await;
                    if let Err(e) = mgr.send_key(&session, &key).await {
                        set_err(&sink, format!("Forward {} failed: {}", key, e));
                    }
                }
                ForwardMsg::Paste { session, text } => {
                    // Flush queued text first so the paste lands in order.
                    flush_text(&mut pend, &mgr, &sink).await;
                    // Coalesce split paste segments (see PASTE_COALESCE_WINDOW):
                    // keep absorbing same-session pastes that arrive within the
                    // debounce, then emit the whole thing as ONE bracketed block.
                    let mut body = text;
                    let mut closed = false;
                    loop {
                        match tokio::time::timeout(PASTE_COALESCE_WINDOW, rx.recv()).await {
                            Ok(Some(ForwardMsg::Paste {
                                session: s,
                                text: t,
                            })) if s == session => {
                                body.push_str(&t);
                            }
                            Ok(Some(other)) => {
                                carry = Some(other);
                                break;
                            }
                            Ok(None) => {
                                closed = true;
                                break;
                            }
                            Err(_) => break, // debounce expired → no more segments
                        }
                    }
                    if let Err(e) = mgr.send_paste_raw(&session, &body).await {
                        set_err(&sink, format!("Paste failed: {}", e));
                    }
                    if closed {
                        break;
                    }
                }
            }
        }
    });
    tx
}

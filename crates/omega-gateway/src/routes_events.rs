//! `GET /v1/events` — protected WebSocket forwarding every [`GatewayEvent`]
//! published on the gateway's [`crate::events::EventHub`] as a JSON text
//! frame (mission updates, alerts, heartbeat). R-STREAM shape: the loop
//! never exits on a lag error, only on a dead socket or a closed channel.

use crate::protocol::GatewayEvent;
use crate::server::AppState;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Response;
use tokio::sync::broadcast;

pub async fn events(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| forward_loop(socket, state))
}

async fn forward_loop(mut socket: WebSocket, state: AppState) {
    let mut rx = state.events.subscribe();
    loop {
        match rx.recv().await {
            Ok(ev) => {
                let text = serde_json::to_string(&ev).expect("serialize GatewayEvent");
                if socket.send(Message::Text(text.into())).await.is_err() {
                    return; // client went away: the ONLY exit on a healthy channel
                }
            }
            // A slow subscriber fell behind the broadcast buffer: this is
            // not fatal (R-STREAM never exits on error) — tell the client
            // it missed `n` events so it can resync, then keep reading.
            Err(broadcast::error::RecvError::Lagged(n)) => {
                let hint = GatewayEvent::Alert {
                    message: format!("resync: missed {n} event(s)"),
                    ts: chrono::Utc::now().to_rfc3339(),
                };
                let text = serde_json::to_string(&hint).expect("serialize GatewayEvent");
                if socket.send(Message::Text(text.into())).await.is_err() {
                    return;
                }
            }
            // The hub itself is gone (process shutting down): nothing left
            // to forward.
            Err(broadcast::error::RecvError::Closed) => return,
        }
    }
}

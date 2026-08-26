//! In-process event hub for the `/v1/events` WebSocket: mission updates,
//! alerts, and a heartbeat, fanned out to every connected subscriber via a
//! [`tokio::sync::broadcast`] channel (R-STREAM: pull-shaped for each
//! subscriber, never push-blocking the emitter on a slow reader).
//!
//! KNOWN LIMIT (V2): [`GatewayEvent::Alert`] is only ever emitted by an
//! in-process caller (a test today; a future in-process alert source
//! later). There is no external alert ingestion — the real alert path is
//! `~/.omega/bin/omega-alert-send.sh`, which has no local success log to
//! tail. Wiring a real external alert source into the hub is a later plan.

use crate::config::GatewayConfig;
use crate::missions;
use crate::protocol;
use crate::protocol::GatewayEvent;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::broadcast;

/// Broadcast capacity: generous enough that a briefly-lagging subscriber
/// (a slow client) doesn't miss a burst of mission updates before it can
/// drain them; a subscriber that falls further behind than this gets a
/// `Lagged` error and resyncs (see `routes_events::forward_loop`).
const CHANNEL_CAPACITY: usize = 256;

/// The mission-ledger poll interval is `cfg.stream_interval_ms` × this
/// multiplier, floored at [`MISSION_POLL_MIN_MS`] — mission ledgers change
/// far less often than a terminal pane, so polling them at the pane-stream
/// cadence would be wasted file I/O.
const MISSION_POLL_MULTIPLIER: u64 = 5;
const MISSION_POLL_MIN_MS: u64 = 3000;

/// How often a [`GatewayEvent::Heartbeat`] is emitted regardless of mission
/// activity, so a client can distinguish "quiet" from "gateway is gone".
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

/// Cloneable handle to the event bus. Cloning shares the same underlying
/// broadcast channel — every clone's `emit` reaches every `subscribe`r.
#[derive(Clone)]
pub struct EventHub {
    tx: broadcast::Sender<GatewayEvent>,
}

impl EventHub {
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(CHANNEL_CAPACITY);
        Self { tx }
    }

    /// A fresh receiver, positioned at "now" (it does not see events sent
    /// before this call).
    pub fn subscribe(&self) -> broadcast::Receiver<GatewayEvent> {
        self.tx.subscribe()
    }

    /// Publishes `ev` to every current subscriber. A `SendError` (no
    /// subscribers currently connected) is not an error condition here —
    /// there is simply nobody listening yet, so it is silently ignored.
    pub fn emit(&self, ev: GatewayEvent) {
        let _ = self.tx.send(ev);
    }
}

impl Default for EventHub {
    fn default() -> Self {
        Self::new()
    }
}

/// Pure diff: compares `current` mission ledgers against `prev`'s cached
/// `key -> updated_at` snapshot and returns one `MissionUpdated` per key
/// that is new or whose `updated_at` changed. Never re-fires for a key
/// whose `updated_at` is unchanged (R-MONITOR: one variable per signal, no
/// re-fire on frozen state). Does NOT mutate `prev` — the caller applies
/// the update after acting on the returned events, so the poller stays a
/// thin wrapper around this pure function.
pub fn diff_missions(
    prev: &HashMap<String, String>,
    current: &[protocol::Mission],
) -> Vec<GatewayEvent> {
    current
        .iter()
        .filter(|m| prev.get(&m.key).map(|u| u != &m.updated_at).unwrap_or(true))
        .map(|m| GatewayEvent::MissionUpdated {
            key: m.key.clone(),
            updated_at: m.updated_at.clone(),
        })
        .collect()
}

/// `cfg.stream_interval_ms` × [`MISSION_POLL_MULTIPLIER`], floored at
/// [`MISSION_POLL_MIN_MS`].
fn mission_poll_interval(cfg: &GatewayConfig) -> Duration {
    Duration::from_millis(
        (cfg.stream_interval_ms * MISSION_POLL_MULTIPLIER).max(MISSION_POLL_MIN_MS),
    )
}

/// Spawns the two long-lived background loops that keep `hub` alive for
/// the rest of the process: the mission-ledger poller (diffs
/// `missions::list()` against a cache it owns and emits `MissionUpdated`
/// only for new/changed keys, via the pure [`diff_missions`]) and the
/// heartbeat emitter. Neither loop ever exits on its own (R-STREAM: an
/// emitter loop dying silently is worse than one that keeps running) —
/// they live and die with the process, exactly like `main`'s server loop.
///
/// Not called from [`crate::server::build_router`]: router construction
/// stays a pure state-to-router mapping so tests that build a router to
/// exercise unrelated routes don't also spin up ledger-polling background
/// tasks. The gateway binary's `Serve` command is the one real caller.
pub fn spawn_background_emitters(hub: EventHub, cfg: &GatewayConfig) {
    let poll_interval = mission_poll_interval(cfg);
    let mission_hub = hub.clone();
    tokio::spawn(async move {
        let mut cache: HashMap<String, String> = HashMap::new();
        loop {
            let current = tokio::task::spawn_blocking(missions::list)
                .await
                .unwrap_or_default();
            for ev in diff_missions(&cache, &current) {
                if let GatewayEvent::MissionUpdated { key, updated_at } = &ev {
                    cache.insert(key.clone(), updated_at.clone());
                }
                mission_hub.emit(ev);
            }
            tokio::time::sleep(poll_interval).await;
        }
    });

    tokio::spawn(async move {
        loop {
            hub.emit(GatewayEvent::Heartbeat {
                ts: chrono::Utc::now().to_rfc3339(),
            });
            tokio::time::sleep(HEARTBEAT_INTERVAL).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn subscriber_receives_emitted_alert() {
        let hub = EventHub::new();
        let mut rx = hub.subscribe();
        hub.emit(GatewayEvent::Alert {
            message: "boom".into(),
            ts: "t1".into(),
        });
        let ev = rx.recv().await.unwrap();
        match ev {
            GatewayEvent::Alert { message, ts } => {
                assert_eq!(message, "boom");
                assert_eq!(ts, "t1");
            }
            other => panic!("expected Alert, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn two_subscribers_both_receive_the_same_event() {
        let hub = EventHub::new();
        let mut rx1 = hub.subscribe();
        let mut rx2 = hub.subscribe();
        hub.emit(GatewayEvent::Heartbeat { ts: "t2".into() });
        let e1 = rx1.recv().await.unwrap();
        let e2 = rx2.recv().await.unwrap();
        assert!(matches!(e1, GatewayEvent::Heartbeat { ts } if ts == "t2"));
        assert!(matches!(e2, GatewayEvent::Heartbeat { ts } if ts == "t2"));
    }

    fn mission(key: &str, updated_at: &str) -> protocol::Mission {
        protocol::Mission {
            key: key.to_string(),
            project: None,
            title: None,
            done: 0,
            total: 0,
            tasks: vec![],
            updated_at: updated_at.to_string(),
        }
    }

    #[test]
    fn diff_missions_unchanged_yields_no_events() {
        let mut prev = HashMap::new();
        prev.insert("oracle-a".to_string(), "t1".to_string());
        let current = vec![mission("oracle-a", "t1")];
        assert!(diff_missions(&prev, &current).is_empty());
    }

    #[test]
    fn diff_missions_changed_updated_at_yields_one_event() {
        let mut prev = HashMap::new();
        prev.insert("oracle-a".to_string(), "t1".to_string());
        let current = vec![mission("oracle-a", "t2")];
        let events = diff_missions(&prev, &current);
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            GatewayEvent::MissionUpdated { key, updated_at }
                if key == "oracle-a" && updated_at == "t2"
        ));
    }

    #[test]
    fn diff_missions_new_key_yields_one_event() {
        let prev = HashMap::new();
        let current = vec![mission("oracle-b", "t1")];
        let events = diff_missions(&prev, &current);
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            GatewayEvent::MissionUpdated { key, .. } if key == "oracle-b"
        ));
    }

    #[test]
    fn diff_missions_mixed_only_reports_new_and_changed() {
        let mut prev = HashMap::new();
        prev.insert("oracle-a".to_string(), "t1".to_string()); // unchanged
        prev.insert("oracle-c".to_string(), "old".to_string()); // changed
        let current = vec![
            mission("oracle-a", "t1"),  // unchanged: no event
            mission("oracle-b", "t1"),  // new: event
            mission("oracle-c", "new"), // changed: event
        ];
        let mut keys: Vec<String> = diff_missions(&prev, &current)
            .into_iter()
            .map(|e| match e {
                GatewayEvent::MissionUpdated { key, .. } => key,
                _ => unreachable!(),
            })
            .collect();
        keys.sort();
        assert_eq!(keys, vec!["oracle-b".to_string(), "oracle-c".to_string()]);
    }
}

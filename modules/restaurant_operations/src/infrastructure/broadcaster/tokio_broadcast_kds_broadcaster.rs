//! Tokio-backed implementation of `KdsBroadcaster` — one
//! `tokio::sync::broadcast::Sender` per kitchen station, lazily created on
//! first publish/subscribe. Lookups are guarded by an `RwLock` so the steady
//! state (publish to an existing station) only takes a read lock.
//!
//! Capacity is intentionally small (default 64): the broadcast channel will
//! drop the oldest event on overflow and emit a `Lagged` error to receivers.
//! The SSE handler in the gateway maps `Lagged` to "skip" — the kitchen
//! display can re-fetch the active tickets list to recover.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{RwLock, broadcast};

use crate::application::broadcaster::{KdsBroadcaster, KdsEvent};
use crate::domain::value_objects::KitchenStationId;

const DEFAULT_CHANNEL_CAPACITY: usize = 64;

#[derive(Clone)]
pub struct TokioBroadcastKdsBroadcaster {
    inner: Arc<RwLock<HashMap<KitchenStationId, broadcast::Sender<KdsEvent>>>>,
    capacity: usize,
}

impl TokioBroadcastKdsBroadcaster {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            capacity: DEFAULT_CHANNEL_CAPACITY,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            capacity,
        }
    }

    /// Hands out a fresh `Receiver` for `station_id`; creates the channel if
    /// no one has published or subscribed before.
    pub async fn subscribe(&self, station_id: KitchenStationId) -> broadcast::Receiver<KdsEvent> {
        // Fast path: existing sender.
        if let Some(sender) = self.inner.read().await.get(&station_id) {
            return sender.subscribe();
        }
        // Slow path: insert under write lock. Re-check to avoid races.
        let mut guard = self.inner.write().await;
        let sender = guard
            .entry(station_id)
            .or_insert_with(|| broadcast::channel(self.capacity).0)
            .clone();
        sender.subscribe()
    }
}

impl Default for TokioBroadcastKdsBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl KdsBroadcaster for TokioBroadcastKdsBroadcaster {
    async fn publish(&self, station_id: KitchenStationId, event: KdsEvent) {
        // Fast path.
        if let Some(sender) = self.inner.read().await.get(&station_id) {
            // `send` returns Err only when there are no receivers — that is
            // the normal "no kitchen screen connected yet" case, ignore it.
            let _ = sender.send(event);
            return;
        }
        // Slow path: create the channel even with no subscribers so the next
        // subscriber starts with an empty buffer (broadcast channels do not
        // replay history).
        let mut guard = self.inner.write().await;
        let sender = guard
            .entry(station_id)
            .or_insert_with(|| broadcast::channel(self.capacity).0)
            .clone();
        let _ = sender.send(event);
    }
}

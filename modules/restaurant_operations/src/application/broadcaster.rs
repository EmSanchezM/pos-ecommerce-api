//! KdsBroadcaster — abstracts the realtime fan-out used by the SSE handler in
//! the API gateway. The default implementation lives in `infrastructure` and
//! is backed by `tokio::sync::broadcast` per kitchen station; tests can swap
//! in `NoopKdsBroadcaster`.
//!
//! The broadcaster is intentionally fire-and-forget: if no kitchen display is
//! currently subscribed to a station, the event is dropped (the screens will
//! catch up on reconnect by re-listing active tickets).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::value_objects::{Course, KdsItemStatus, KdsTicketStatus, KitchenStationId};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum KdsEvent {
    TicketCreated {
        ticket_id: Uuid,
        ticket_number: i32,
        station_id: Uuid,
        table_label: Option<String>,
        items_count: usize,
        course: Course,
    },
    TicketStatusChanged {
        ticket_id: Uuid,
        ticket_number: i32,
        status: KdsTicketStatus,
    },
    ItemStatusChanged {
        ticket_id: Uuid,
        item_id: Uuid,
        status: KdsItemStatus,
    },
    TicketCanceled {
        ticket_id: Uuid,
        ticket_number: i32,
        reason: String,
    },
}

#[async_trait]
pub trait KdsBroadcaster: Send + Sync {
    async fn publish(&self, station_id: KitchenStationId, event: KdsEvent);
}

/// Default no-op for tests / silent runs. Production wires
/// `TokioBroadcastKdsBroadcaster` (in `infrastructure::broadcaster`).
#[derive(Debug, Clone, Default)]
pub struct NoopKdsBroadcaster;

#[async_trait]
impl KdsBroadcaster for NoopKdsBroadcaster {
    async fn publish(&self, _station_id: KitchenStationId, _event: KdsEvent) {}
}

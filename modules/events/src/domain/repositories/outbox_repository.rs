//! Outbox repository trait.
//!
//! Two write paths:
//!   * `enqueue_in_tx` is called by other modules from inside an open
//!     `sqlx::Transaction` to atomically persist a domain event together with
//!     the aggregate change. This is the heart of the transactional outbox
//!     pattern.
//!   * `update_after_dispatch` is called by the dispatcher worker after it has
//!     fanned out an event to local subscribers, to mark it processed/failed.

use async_trait::async_trait;
use sqlx::{Postgres, Transaction};

use crate::EventsError;
use crate::domain::entities::OutboxEvent;
use crate::domain::value_objects::OutboxEventId;

#[async_trait]
pub trait OutboxRepository: Send + Sync {
    /// Persist a new event inside an open transaction.
    async fn enqueue_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        event: &OutboxEvent,
    ) -> Result<(), EventsError>;

    /// Pull the next batch of `pending` events ordered by `occurred_at`.
    async fn fetch_pending(&self, batch_size: i64) -> Result<Vec<OutboxEvent>, EventsError>;

    /// Persist an updated event (status / attempts / processed_at / last_error).
    async fn update_after_dispatch(&self, event: &OutboxEvent) -> Result<(), EventsError>;

    async fn find_by_id(&self, id: OutboxEventId) -> Result<Option<OutboxEvent>, EventsError>;
}

//! In-transaction event publishing helper.
//!
//! Other modules build their domain event payload, then call
//! `PublishEventUseCase::execute` with their already-open `sqlx::Transaction`.
//! The event is inserted into `outbox_events` as part of the same commit,
//! guaranteeing that the event exists if and only if the aggregate change
//! was persisted.

use std::sync::Arc;

use serde_json::Value as JsonValue;
use sqlx::{Postgres, Transaction};

use crate::EventsError;
use crate::domain::entities::OutboxEvent;
use crate::domain::repositories::OutboxRepository;

pub struct PublishEventUseCase {
    repo: Arc<dyn OutboxRepository>,
}

impl PublishEventUseCase {
    pub fn new(repo: Arc<dyn OutboxRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        aggregate_type: &str,
        aggregate_id: &str,
        event_type: &str,
        payload: JsonValue,
    ) -> Result<(), EventsError> {
        let event = OutboxEvent::create(aggregate_type, aggregate_id, event_type, payload);
        self.repo.enqueue_in_tx(tx, &event).await
    }
}

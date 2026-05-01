//! OutboxEvent entity — a domain event persisted in the same transaction as
//! the aggregate that produced it. The dispatcher worker picks it up and
//! delivers it to local subscribers (analytics, notifications, accounting).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::domain::value_objects::{EventStatus, OutboxEventId};

/// Maximum delivery attempts before the event is marked `failed`.
pub const MAX_DELIVERY_ATTEMPTS: i32 = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxEvent {
    id: OutboxEventId,
    aggregate_type: String,
    aggregate_id: String,
    event_type: String,
    payload: JsonValue,
    status: EventStatus,
    attempts: i32,
    last_error: Option<String>,
    occurred_at: DateTime<Utc>,
    processed_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl OutboxEvent {
    pub fn create(
        aggregate_type: impl Into<String>,
        aggregate_id: impl Into<String>,
        event_type: impl Into<String>,
        payload: JsonValue,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: OutboxEventId::new(),
            aggregate_type: aggregate_type.into(),
            aggregate_id: aggregate_id.into(),
            event_type: event_type.into(),
            payload,
            status: EventStatus::Pending,
            attempts: 0,
            last_error: None,
            occurred_at: now,
            processed_at: None,
            created_at: now,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: OutboxEventId,
        aggregate_type: String,
        aggregate_id: String,
        event_type: String,
        payload: JsonValue,
        status: EventStatus,
        attempts: i32,
        last_error: Option<String>,
        occurred_at: DateTime<Utc>,
        processed_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            aggregate_type,
            aggregate_id,
            event_type,
            payload,
            status,
            attempts,
            last_error,
            occurred_at,
            processed_at,
            created_at,
        }
    }

    pub fn mark_processed(&mut self) {
        self.status = EventStatus::Processed;
        self.processed_at = Some(Utc::now());
        self.last_error = None;
    }

    pub fn mark_attempt_failed(&mut self, error: impl Into<String>) {
        self.attempts += 1;
        self.last_error = Some(error.into());
        if self.attempts >= MAX_DELIVERY_ATTEMPTS {
            self.status = EventStatus::Failed;
        }
    }

    // Getters
    pub fn id(&self) -> OutboxEventId {
        self.id
    }
    pub fn aggregate_type(&self) -> &str {
        &self.aggregate_type
    }
    pub fn aggregate_id(&self) -> &str {
        &self.aggregate_id
    }
    pub fn event_type(&self) -> &str {
        &self.event_type
    }
    pub fn payload(&self) -> &JsonValue {
        &self.payload
    }
    pub fn status(&self) -> EventStatus {
        self.status
    }
    pub fn attempts(&self) -> i32 {
        self.attempts
    }
    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }
    pub fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
    pub fn processed_at(&self) -> Option<DateTime<Utc>> {
        self.processed_at
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn new_event_is_pending_with_zero_attempts() {
        let evt = OutboxEvent::create("sale", "abc", "sale.completed", json!({"id": "abc"}));
        assert_eq!(evt.status(), EventStatus::Pending);
        assert_eq!(evt.attempts(), 0);
        assert!(evt.processed_at().is_none());
    }

    #[test]
    fn mark_processed_sets_processed_at_and_clears_error() {
        let mut evt = OutboxEvent::create("sale", "abc", "sale.completed", json!({}));
        evt.mark_attempt_failed("boom");
        evt.mark_processed();
        assert_eq!(evt.status(), EventStatus::Processed);
        assert!(evt.processed_at().is_some());
        assert!(evt.last_error().is_none());
    }

    #[test]
    fn marking_attempt_failed_eventually_marks_failed() {
        let mut evt = OutboxEvent::create("sale", "abc", "sale.completed", json!({}));
        for _ in 0..MAX_DELIVERY_ATTEMPTS {
            evt.mark_attempt_failed("downstream timeout");
        }
        assert_eq!(evt.status(), EventStatus::Failed);
        assert_eq!(evt.attempts(), MAX_DELIVERY_ATTEMPTS);
    }
}

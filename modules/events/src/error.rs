//! Events module error types.

use thiserror::Error;
use uuid::Uuid;

/// Error type for all events module operations.
#[derive(Debug, Error)]
pub enum EventsError {
    #[error("Outbox event not found: {0}")]
    EventNotFound(Uuid),

    #[error("Invalid event payload: {0}")]
    InvalidPayload(String),

    #[error("Subscriber failed: {0}")]
    SubscriberFailed(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

//! Notifications module error types.

use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum NotificationsError {
    #[error("Notification not found: {0}")]
    NotFound(Uuid),

    #[error("Unsupported channel: {0}")]
    UnsupportedChannel(String),

    #[error("Invalid channel: {0}")]
    InvalidChannel(String),

    #[error("Invalid status: {0}")]
    InvalidStatus(String),

    #[error("Adapter not configured for channel: {0}")]
    AdapterNotConfigured(String),

    #[error("Adapter error: {0}")]
    Adapter(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

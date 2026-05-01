//! EventStatus value object — lifecycle of an outbox event.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::EventsError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventStatus {
    /// Persisted in `outbox_events`, not yet picked up by the dispatcher.
    Pending,
    /// Dispatcher has handed it to all registered subscribers successfully.
    Processed,
    /// All retries exhausted; needs manual intervention.
    Failed,
}

impl fmt::Display for EventStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventStatus::Pending => write!(f, "pending"),
            EventStatus::Processed => write!(f, "processed"),
            EventStatus::Failed => write!(f, "failed"),
        }
    }
}

impl FromStr for EventStatus {
    type Err = EventsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(EventStatus::Pending),
            "processed" => Ok(EventStatus::Processed),
            "failed" => Ok(EventStatus::Failed),
            other => Err(EventsError::InvalidPayload(format!(
                "unknown event status: {other}"
            ))),
        }
    }
}

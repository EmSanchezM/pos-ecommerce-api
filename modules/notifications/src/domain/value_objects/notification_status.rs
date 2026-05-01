//! NotificationStatus — lifecycle states for a notification record.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::NotificationsError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationStatus {
    Queued,
    Sent,
    Delivered,
    Failed,
}

impl fmt::Display for NotificationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotificationStatus::Queued => write!(f, "queued"),
            NotificationStatus::Sent => write!(f, "sent"),
            NotificationStatus::Delivered => write!(f, "delivered"),
            NotificationStatus::Failed => write!(f, "failed"),
        }
    }
}

impl FromStr for NotificationStatus {
    type Err = NotificationsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "queued" => Ok(Self::Queued),
            "sent" => Ok(Self::Sent),
            "delivered" => Ok(Self::Delivered),
            "failed" => Ok(Self::Failed),
            other => Err(NotificationsError::InvalidStatus(other.into())),
        }
    }
}

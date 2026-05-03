use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::RestaurantOperationsError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KdsItemStatus {
    Pending,
    InProgress,
    Ready,
    Served,
    Canceled,
}

impl KdsItemStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            KdsItemStatus::Pending => "pending",
            KdsItemStatus::InProgress => "in_progress",
            KdsItemStatus::Ready => "ready",
            KdsItemStatus::Served => "served",
            KdsItemStatus::Canceled => "canceled",
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, KdsItemStatus::Served | KdsItemStatus::Canceled)
    }

    pub fn can_transition_to(self, other: KdsItemStatus) -> bool {
        use KdsItemStatus::*;
        if other == Canceled {
            return matches!(self, Pending | InProgress);
        }
        matches!(
            (self, other),
            (Pending, InProgress) | (InProgress, Ready) | (Ready, Served)
        )
    }
}

impl FromStr for KdsItemStatus {
    type Err = RestaurantOperationsError;
    fn from_str(s: &str) -> Result<Self, RestaurantOperationsError> {
        match s {
            "pending" => Ok(KdsItemStatus::Pending),
            "in_progress" => Ok(KdsItemStatus::InProgress),
            "ready" => Ok(KdsItemStatus::Ready),
            "served" => Ok(KdsItemStatus::Served),
            "canceled" => Ok(KdsItemStatus::Canceled),
            other => Err(RestaurantOperationsError::InvalidItemStatus(
                other.to_string(),
            )),
        }
    }
}

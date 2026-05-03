use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::RestaurantOperationsError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KdsTicketStatus {
    Pending,
    InProgress,
    Ready,
    Served,
    Canceled,
}

impl KdsTicketStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            KdsTicketStatus::Pending => "pending",
            KdsTicketStatus::InProgress => "in_progress",
            KdsTicketStatus::Ready => "ready",
            KdsTicketStatus::Served => "served",
            KdsTicketStatus::Canceled => "canceled",
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, KdsTicketStatus::Served | KdsTicketStatus::Canceled)
    }

    pub fn can_transition_to(self, other: KdsTicketStatus) -> bool {
        use KdsTicketStatus::*;
        if other == Canceled {
            return matches!(self, Pending | InProgress);
        }
        matches!(
            (self, other),
            (Pending, InProgress) | (InProgress, Ready) | (Ready, Served)
        )
    }
}

impl FromStr for KdsTicketStatus {
    type Err = RestaurantOperationsError;
    fn from_str(s: &str) -> Result<Self, RestaurantOperationsError> {
        match s {
            "pending" => Ok(KdsTicketStatus::Pending),
            "in_progress" => Ok(KdsTicketStatus::InProgress),
            "ready" => Ok(KdsTicketStatus::Ready),
            "served" => Ok(KdsTicketStatus::Served),
            "canceled" => Ok(KdsTicketStatus::Canceled),
            other => Err(RestaurantOperationsError::InvalidTicketStatus(
                other.to_string(),
            )),
        }
    }
}

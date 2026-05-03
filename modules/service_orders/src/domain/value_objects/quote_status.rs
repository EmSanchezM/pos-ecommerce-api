use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::ServiceOrdersError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuoteStatus {
    Draft,
    Sent,
    Approved,
    Rejected,
    /// A newer quote has been created for the same order.
    Superseded,
}

impl QuoteStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            QuoteStatus::Draft => "draft",
            QuoteStatus::Sent => "sent",
            QuoteStatus::Approved => "approved",
            QuoteStatus::Rejected => "rejected",
            QuoteStatus::Superseded => "superseded",
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            QuoteStatus::Approved | QuoteStatus::Rejected | QuoteStatus::Superseded
        )
    }

    pub fn can_transition_to(self, other: QuoteStatus) -> bool {
        use QuoteStatus::*;
        // Anyone can be marked superseded (a newer quote replaces it).
        if other == Superseded {
            return matches!(self, Draft | Sent);
        }
        matches!(
            (self, other),
            (Draft, Sent) | (Sent, Approved) | (Sent, Rejected)
        )
    }
}

impl FromStr for QuoteStatus {
    type Err = ServiceOrdersError;
    fn from_str(s: &str) -> Result<Self, ServiceOrdersError> {
        match s {
            "draft" => Ok(QuoteStatus::Draft),
            "sent" => Ok(QuoteStatus::Sent),
            "approved" => Ok(QuoteStatus::Approved),
            "rejected" => Ok(QuoteStatus::Rejected),
            "superseded" => Ok(QuoteStatus::Superseded),
            other => Err(ServiceOrdersError::InvalidQuoteStatus(other.to_string())),
        }
    }
}

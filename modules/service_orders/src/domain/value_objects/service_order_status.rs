use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::ServiceOrdersError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceOrderStatus {
    Intake,
    Diagnosis,
    QuoteSent,
    QuoteApproved,
    InRepair,
    Testing,
    ReadyForPickup,
    Delivered,
    Canceled,
}

impl ServiceOrderStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            ServiceOrderStatus::Intake => "intake",
            ServiceOrderStatus::Diagnosis => "diagnosis",
            ServiceOrderStatus::QuoteSent => "quote_sent",
            ServiceOrderStatus::QuoteApproved => "quote_approved",
            ServiceOrderStatus::InRepair => "in_repair",
            ServiceOrderStatus::Testing => "testing",
            ServiceOrderStatus::ReadyForPickup => "ready_for_pickup",
            ServiceOrderStatus::Delivered => "delivered",
            ServiceOrderStatus::Canceled => "canceled",
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            ServiceOrderStatus::Delivered | ServiceOrderStatus::Canceled
        )
    }

    pub fn can_transition_to(self, other: ServiceOrderStatus) -> bool {
        use ServiceOrderStatus::*;
        // Cancellation: any non-terminal → Canceled.
        if other == Canceled {
            return !self.is_terminal();
        }
        matches!(
            (self, other),
            (Intake, Diagnosis)
                | (Diagnosis, QuoteSent)
                | (QuoteSent, QuoteApproved)
                | (QuoteSent, Diagnosis) // quote rejected → back to Diagnosis for a new quote
                | (QuoteApproved, InRepair)
                | (InRepair, Testing)
                | (Testing, ReadyForPickup)
                | (ReadyForPickup, Delivered)
        )
    }
}

impl FromStr for ServiceOrderStatus {
    type Err = ServiceOrdersError;
    fn from_str(s: &str) -> Result<Self, ServiceOrdersError> {
        match s {
            "intake" => Ok(ServiceOrderStatus::Intake),
            "diagnosis" => Ok(ServiceOrderStatus::Diagnosis),
            "quote_sent" => Ok(ServiceOrderStatus::QuoteSent),
            "quote_approved" => Ok(ServiceOrderStatus::QuoteApproved),
            "in_repair" => Ok(ServiceOrderStatus::InRepair),
            "testing" => Ok(ServiceOrderStatus::Testing),
            "ready_for_pickup" => Ok(ServiceOrderStatus::ReadyForPickup),
            "delivered" => Ok(ServiceOrderStatus::Delivered),
            "canceled" => Ok(ServiceOrderStatus::Canceled),
            other => Err(ServiceOrdersError::InvalidServiceOrderStatus(
                other.to_string(),
            )),
        }
    }
}

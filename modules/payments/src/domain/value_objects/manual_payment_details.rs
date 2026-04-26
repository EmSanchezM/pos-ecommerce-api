//! ManualPaymentDetails - structured metadata for offline payment kinds.
//!
//! Stored in `payment_transactions.metadata` as a JSON blob. The `kind` field
//! discriminates which subset of the optional fields is meaningful.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::ManualPaymentKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualPaymentDetails {
    pub kind: ManualPaymentKind,

    // BankTransfer / AgencyDeposit
    pub bank_name: Option<String>,
    pub bank_account_last_four: Option<String>,
    pub depositor_name: Option<String>,
    pub deposit_date: Option<DateTime<Utc>>,

    // CashOnDelivery
    pub delivery_address: Option<String>,
    pub delivery_recipient_name: Option<String>,
    pub delivery_phone: Option<String>,
    pub expected_delivery_date: Option<DateTime<Utc>>,

    pub notes: Option<String>,
}

impl ManualPaymentDetails {
    pub fn new(kind: ManualPaymentKind) -> Self {
        Self {
            kind,
            bank_name: None,
            bank_account_last_four: None,
            depositor_name: None,
            deposit_date: None,
            delivery_address: None,
            delivery_recipient_name: None,
            delivery_phone: None,
            expected_delivery_date: None,
            notes: None,
        }
    }

    /// Serialize for storage in the `metadata` column.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Best-effort parse from the `metadata` column. Returns None when the
    /// payload doesn't look like a structured manual-payment record.
    pub fn try_from_json(raw: &str) -> Option<Self> {
        serde_json::from_str(raw).ok()
    }
}

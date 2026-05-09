//! `BillingCycleStatus` — lifecycle of a single billing period.
//!
//! Allowed transitions:
//!
//! - `Pending  → Invoiced` (cron job generated invoice + transaction)
//! - `Pending  → Skipped`  (administrative skip)
//! - `Trialing → Skipped`  (trial period; no invoice generated)
//! - `Invoiced → Paid`     (payment confirmed by gateway webhook)
//! - `Invoiced → Failed`   (payment failed; dunning kicks in)
//! - `Failed   → Paid`     (dunning attempt succeeded)
//!
//! `Paid` and `Skipped` are terminal.

use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::SubscriptionError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BillingCycleStatus {
    Pending,
    Trialing,
    Invoiced,
    Paid,
    Failed,
    Skipped,
}

impl BillingCycleStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            BillingCycleStatus::Pending => "pending",
            BillingCycleStatus::Trialing => "trialing",
            BillingCycleStatus::Invoiced => "invoiced",
            BillingCycleStatus::Paid => "paid",
            BillingCycleStatus::Failed => "failed",
            BillingCycleStatus::Skipped => "skipped",
        }
    }

    pub fn can_transition_to(&self, to: &BillingCycleStatus) -> bool {
        use BillingCycleStatus::*;
        matches!(
            (self, to),
            (Pending, Invoiced)
                | (Pending, Skipped)
                | (Trialing, Skipped)
                | (Invoiced, Paid)
                | (Invoiced, Failed)
                | (Failed, Paid)
        )
    }
}

impl FromStr for BillingCycleStatus {
    type Err = SubscriptionError;
    fn from_str(s: &str) -> Result<Self, SubscriptionError> {
        match s {
            "pending" => Ok(BillingCycleStatus::Pending),
            "trialing" => Ok(BillingCycleStatus::Trialing),
            "invoiced" => Ok(BillingCycleStatus::Invoiced),
            "paid" => Ok(BillingCycleStatus::Paid),
            "failed" => Ok(BillingCycleStatus::Failed),
            "skipped" => Ok(BillingCycleStatus::Skipped),
            other => Err(SubscriptionError::InvalidCycleStatus(other.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowed_transitions() {
        for (from, to) in [
            (BillingCycleStatus::Pending, BillingCycleStatus::Invoiced),
            (BillingCycleStatus::Pending, BillingCycleStatus::Skipped),
            (BillingCycleStatus::Trialing, BillingCycleStatus::Skipped),
            (BillingCycleStatus::Invoiced, BillingCycleStatus::Paid),
            (BillingCycleStatus::Invoiced, BillingCycleStatus::Failed),
            (BillingCycleStatus::Failed, BillingCycleStatus::Paid),
        ] {
            assert!(from.can_transition_to(&to), "{from:?} -> {to:?}");
        }
    }

    #[test]
    fn rejects_paid_terminal() {
        assert!(!BillingCycleStatus::Paid.can_transition_to(&BillingCycleStatus::Failed));
        assert!(!BillingCycleStatus::Skipped.can_transition_to(&BillingCycleStatus::Paid));
    }

    #[test]
    fn round_trip_str() {
        for s in [
            BillingCycleStatus::Pending,
            BillingCycleStatus::Trialing,
            BillingCycleStatus::Invoiced,
            BillingCycleStatus::Paid,
            BillingCycleStatus::Failed,
            BillingCycleStatus::Skipped,
        ] {
            assert_eq!(BillingCycleStatus::from_str(s.as_str()).unwrap(), s);
        }
        assert!(BillingCycleStatus::from_str("xx").is_err());
    }
}

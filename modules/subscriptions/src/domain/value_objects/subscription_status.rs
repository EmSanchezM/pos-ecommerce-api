//! `SubscriptionStatus` — the lifecycle of a `Subscription`.
//!
//! Allowed transitions (also enforced by `Subscription::cancel`,
//! `mark_past_due`, `resume_active`, `activate`):
//!
//! - `Trialing → Active` (trial expired or activated manually)
//! - `Trialing → Canceled`
//! - `Active   → PastDue` (payment failed)
//! - `Active   → Canceled`
//! - `PastDue  → Active` (payment recovered via dunning)
//! - `PastDue  → Canceled` (grace period exhausted)
//!
//! `Canceled` is terminal.

use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::SubscriptionError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    Trialing,
    Active,
    PastDue,
    Canceled,
}

impl SubscriptionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            SubscriptionStatus::Trialing => "trialing",
            SubscriptionStatus::Active => "active",
            SubscriptionStatus::PastDue => "past_due",
            SubscriptionStatus::Canceled => "canceled",
        }
    }

    pub fn can_transition_to(&self, to: &SubscriptionStatus) -> bool {
        use SubscriptionStatus::*;
        matches!(
            (self, to),
            (Trialing, Active)
                | (Trialing, Canceled)
                | (Active, PastDue)
                | (Active, Canceled)
                | (PastDue, Active)
                | (PastDue, Canceled)
        )
    }
}

impl FromStr for SubscriptionStatus {
    type Err = SubscriptionError;
    fn from_str(s: &str) -> Result<Self, SubscriptionError> {
        match s {
            "trialing" => Ok(SubscriptionStatus::Trialing),
            "active" => Ok(SubscriptionStatus::Active),
            "past_due" => Ok(SubscriptionStatus::PastDue),
            "canceled" => Ok(SubscriptionStatus::Canceled),
            other => Err(SubscriptionError::InvalidStatus(other.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowed_transitions() {
        let allowed = [
            (SubscriptionStatus::Trialing, SubscriptionStatus::Active),
            (SubscriptionStatus::Trialing, SubscriptionStatus::Canceled),
            (SubscriptionStatus::Active, SubscriptionStatus::PastDue),
            (SubscriptionStatus::Active, SubscriptionStatus::Canceled),
            (SubscriptionStatus::PastDue, SubscriptionStatus::Active),
            (SubscriptionStatus::PastDue, SubscriptionStatus::Canceled),
        ];
        for (from, to) in allowed {
            assert!(from.can_transition_to(&to), "{from:?} -> {to:?}");
        }
    }

    #[test]
    fn rejects_terminal_and_loops() {
        assert!(
            !SubscriptionStatus::Canceled.can_transition_to(&SubscriptionStatus::Active),
            "canceled is terminal"
        );
        assert!(
            !SubscriptionStatus::Active.can_transition_to(&SubscriptionStatus::Trialing),
            "cannot go back to trial"
        );
        assert!(
            !SubscriptionStatus::Active.can_transition_to(&SubscriptionStatus::Active),
            "self-transition disallowed"
        );
    }

    #[test]
    fn round_trip_str() {
        for s in [
            SubscriptionStatus::Trialing,
            SubscriptionStatus::Active,
            SubscriptionStatus::PastDue,
            SubscriptionStatus::Canceled,
        ] {
            assert_eq!(SubscriptionStatus::from_str(s.as_str()).unwrap(), s);
        }
        assert!(SubscriptionStatus::from_str("nope").is_err());
    }
}

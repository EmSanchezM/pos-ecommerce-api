//! `BillingInterval` — cadence at which a `Subscription` is billed.
//!
//! v1.0 only supports `Monthly`. `Quarterly` and `Annual` are deferred to
//! v1.1; once added, `next_period_end` should branch on the variant.

use std::str::FromStr;

use chrono::{DateTime, Months, Utc};
use serde::{Deserialize, Serialize};

use crate::SubscriptionError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BillingInterval {
    Monthly,
}

impl BillingInterval {
    pub fn as_str(self) -> &'static str {
        match self {
            BillingInterval::Monthly => "monthly",
        }
    }

    /// Returns the timestamp at which the next billing period ends, given the
    /// start of the current one. For `Monthly`, this is `start + 1 month` per
    /// `chrono::Months::new(1)` semantics (which preserves day-of-month and
    /// clamps overflow — e.g. Jan 31 + 1 month = Feb 28/29).
    pub fn next_period_end(&self, start: DateTime<Utc>) -> DateTime<Utc> {
        match self {
            BillingInterval::Monthly => start
                .checked_add_months(Months::new(1))
                .expect("monthly interval shift overflowed DateTime<Utc>"),
        }
    }
}

impl FromStr for BillingInterval {
    type Err = SubscriptionError;
    fn from_str(s: &str) -> Result<Self, SubscriptionError> {
        match s {
            "monthly" => Ok(BillingInterval::Monthly),
            other => Err(SubscriptionError::InvalidInterval(other.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn monthly_advances_one_month() {
        let start = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let end = BillingInterval::Monthly.next_period_end(start);
        assert_eq!(end, Utc.with_ymd_and_hms(2026, 2, 15, 10, 0, 0).unwrap());
    }

    #[test]
    fn monthly_handles_end_of_month_overflow() {
        // Jan 31 + 1 month → Feb 28 (or Feb 29 in a leap year).
        let start = Utc.with_ymd_and_hms(2026, 1, 31, 0, 0, 0).unwrap();
        let end = BillingInterval::Monthly.next_period_end(start);
        assert_eq!(end, Utc.with_ymd_and_hms(2026, 2, 28, 0, 0, 0).unwrap());
    }

    #[test]
    fn round_trip_str() {
        assert_eq!(
            BillingInterval::from_str(BillingInterval::Monthly.as_str()).unwrap(),
            BillingInterval::Monthly
        );
        assert!(BillingInterval::from_str("yearly").is_err());
    }
}

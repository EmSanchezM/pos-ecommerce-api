//! `DunningOutcome` — terminal status of a single retry attempt.

use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::SubscriptionError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DunningOutcome {
    Pending,
    Succeeded,
    Failed,
    Skipped,
}

impl DunningOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            DunningOutcome::Pending => "pending",
            DunningOutcome::Succeeded => "succeeded",
            DunningOutcome::Failed => "failed",
            DunningOutcome::Skipped => "skipped",
        }
    }
}

impl FromStr for DunningOutcome {
    type Err = SubscriptionError;
    fn from_str(s: &str) -> Result<Self, SubscriptionError> {
        match s {
            "pending" => Ok(DunningOutcome::Pending),
            "succeeded" => Ok(DunningOutcome::Succeeded),
            "failed" => Ok(DunningOutcome::Failed),
            "skipped" => Ok(DunningOutcome::Skipped),
            other => Err(SubscriptionError::InvalidDunningOutcome(other.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_str() {
        for s in [
            DunningOutcome::Pending,
            DunningOutcome::Succeeded,
            DunningOutcome::Failed,
            DunningOutcome::Skipped,
        ] {
            assert_eq!(DunningOutcome::from_str(s.as_str()).unwrap(), s);
        }
        assert!(DunningOutcome::from_str("unknown").is_err());
    }
}

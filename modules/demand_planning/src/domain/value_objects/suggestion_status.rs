//! SuggestionStatus — workflow for a `ReplenishmentSuggestion`:
//! `pending → approved → ordered`, or `pending → dismissed`.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::DemandPlanningError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionStatus {
    Pending,
    Approved,
    Ordered,
    Dismissed,
}

impl SuggestionStatus {
    pub fn can_transition_to(self, next: SuggestionStatus) -> bool {
        use SuggestionStatus::*;
        matches!(
            (self, next),
            (Pending, Approved) | (Pending, Dismissed) | (Approved, Ordered)
        )
    }
}

impl fmt::Display for SuggestionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SuggestionStatus::Pending => "pending",
            SuggestionStatus::Approved => "approved",
            SuggestionStatus::Ordered => "ordered",
            SuggestionStatus::Dismissed => "dismissed",
        };
        f.write_str(s)
    }
}

impl FromStr for SuggestionStatus {
    type Err = DemandPlanningError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "approved" => Ok(Self::Approved),
            "ordered" => Ok(Self::Ordered),
            "dismissed" => Ok(Self::Dismissed),
            other => Err(DemandPlanningError::InvalidSuggestionStatus(other.into())),
        }
    }
}

use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::TenancyError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrganizationStatus {
    #[default]
    Active,
    Suspended,
    /// Reserved for v1.1 self-service signup flow.
    PendingSetup,
}

impl OrganizationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            OrganizationStatus::Active => "active",
            OrganizationStatus::Suspended => "suspended",
            OrganizationStatus::PendingSetup => "pending_setup",
        }
    }

    pub fn can_transition_to(self, other: OrganizationStatus) -> bool {
        use OrganizationStatus::*;
        matches!(
            (self, other),
            (Active, Suspended)
                | (Suspended, Active)
                | (PendingSetup, Active)
                | (PendingSetup, Suspended)
        )
    }
}

impl FromStr for OrganizationStatus {
    type Err = TenancyError;
    fn from_str(s: &str) -> Result<Self, TenancyError> {
        match s {
            "active" => Ok(OrganizationStatus::Active),
            "suspended" => Ok(OrganizationStatus::Suspended),
            "pending_setup" => Ok(OrganizationStatus::PendingSetup),
            other => Err(TenancyError::InvalidStatus(other.to_string())),
        }
    }
}

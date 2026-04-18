//! PromotionStatus value object

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::SalesError;

/// Status of a promotion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromotionStatus {
    Active,
    Inactive,
    Expired,
}

impl PromotionStatus {
    pub fn is_active(self) -> bool {
        self == Self::Active
    }
}

impl fmt::Display for PromotionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Inactive => write!(f, "inactive"),
            Self::Expired => write!(f, "expired"),
        }
    }
}

impl FromStr for PromotionStatus {
    type Err = SalesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(Self::Active),
            "inactive" => Ok(Self::Inactive),
            "expired" => Ok(Self::Expired),
            _ => Err(SalesError::InvalidPromotionStatus),
        }
    }
}

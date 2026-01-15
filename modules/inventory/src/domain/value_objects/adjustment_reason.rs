// AdjustmentReason enum - reason for stock adjustment

use crate::InventoryError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Reason for stock adjustment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdjustmentReason {
    /// Product damaged
    Damage,
    /// Product stolen
    Theft,
    /// Product lost (unknown cause)
    Loss,
    /// Product found (previously unaccounted)
    Found,
    /// Manual correction/count adjustment
    Correction,
    /// Product expired
    Expiration,
}

impl AdjustmentReason {
    /// Returns all available adjustment reasons
    pub fn all() -> &'static [AdjustmentReason] {
        &[
            AdjustmentReason::Damage,
            AdjustmentReason::Theft,
            AdjustmentReason::Loss,
            AdjustmentReason::Found,
            AdjustmentReason::Correction,
            AdjustmentReason::Expiration,
        ]
    }

    /// Returns true if this reason typically results in a decrease
    pub fn is_typically_decrease(&self) -> bool {
        matches!(
            self,
            AdjustmentReason::Damage
                | AdjustmentReason::Theft
                | AdjustmentReason::Loss
                | AdjustmentReason::Expiration
        )
    }

    /// Returns true if this reason typically results in an increase
    pub fn is_typically_increase(&self) -> bool {
        matches!(self, AdjustmentReason::Found)
    }
}

impl FromStr for AdjustmentReason {
    type Err = InventoryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "damage" | "damaged" => Ok(AdjustmentReason::Damage),
            "theft" | "stolen" => Ok(AdjustmentReason::Theft),
            "loss" | "lost" => Ok(AdjustmentReason::Loss),
            "found" => Ok(AdjustmentReason::Found),
            "correction" | "count" | "recount" => Ok(AdjustmentReason::Correction),
            "expiration" | "expired" => Ok(AdjustmentReason::Expiration),
            _ => Err(InventoryError::InvalidAdjustmentReason),
        }
    }
}

impl fmt::Display for AdjustmentReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdjustmentReason::Damage => write!(f, "damage"),
            AdjustmentReason::Theft => write!(f, "theft"),
            AdjustmentReason::Loss => write!(f, "loss"),
            AdjustmentReason::Found => write!(f, "found"),
            AdjustmentReason::Correction => write!(f, "correction"),
            AdjustmentReason::Expiration => write!(f, "expiration"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(AdjustmentReason::from_str("damage").unwrap(), AdjustmentReason::Damage);
        assert_eq!(AdjustmentReason::from_str("theft").unwrap(), AdjustmentReason::Theft);
        assert_eq!(AdjustmentReason::from_str("loss").unwrap(), AdjustmentReason::Loss);
        assert_eq!(AdjustmentReason::from_str("found").unwrap(), AdjustmentReason::Found);
        assert_eq!(AdjustmentReason::from_str("correction").unwrap(), AdjustmentReason::Correction);
        assert_eq!(AdjustmentReason::from_str("expiration").unwrap(), AdjustmentReason::Expiration);
    }

    #[test]
    fn test_from_str_aliases() {
        assert_eq!(AdjustmentReason::from_str("damaged").unwrap(), AdjustmentReason::Damage);
        assert_eq!(AdjustmentReason::from_str("stolen").unwrap(), AdjustmentReason::Theft);
        assert_eq!(AdjustmentReason::from_str("lost").unwrap(), AdjustmentReason::Loss);
        assert_eq!(AdjustmentReason::from_str("count").unwrap(), AdjustmentReason::Correction);
        assert_eq!(AdjustmentReason::from_str("expired").unwrap(), AdjustmentReason::Expiration);
    }

    #[test]
    fn test_invalid() {
        let result = AdjustmentReason::from_str("invalid");
        assert!(matches!(result, Err(InventoryError::InvalidAdjustmentReason)));
    }

    #[test]
    fn test_display() {
        assert_eq!(AdjustmentReason::Damage.to_string(), "damage");
        assert_eq!(AdjustmentReason::Correction.to_string(), "correction");
    }

    #[test]
    fn test_typically_decrease() {
        assert!(AdjustmentReason::Damage.is_typically_decrease());
        assert!(AdjustmentReason::Theft.is_typically_decrease());
        assert!(AdjustmentReason::Loss.is_typically_decrease());
        assert!(AdjustmentReason::Expiration.is_typically_decrease());
        assert!(!AdjustmentReason::Found.is_typically_decrease());
        assert!(!AdjustmentReason::Correction.is_typically_decrease());
    }

    #[test]
    fn test_typically_increase() {
        assert!(AdjustmentReason::Found.is_typically_increase());
        assert!(!AdjustmentReason::Damage.is_typically_increase());
    }
}

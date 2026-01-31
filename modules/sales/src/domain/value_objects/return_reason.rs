//! ReturnReason enum - reasons for product returns

use crate::SalesError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Reasons for product returns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReturnReason {
    /// Product is defective
    Defective,
    /// Wrong item received
    WrongItem,
    /// Product not as described
    NotAsDescribed,
    /// Customer changed their mind
    ChangedMind,
    /// Product no longer needed
    NoLongerNeeded,
    /// Found better price elsewhere
    BetterPriceElsewhere,
    /// Product damaged during shipping
    DamagedInShipping,
    /// Other reason
    Other,
}

impl ReturnReason {
    /// Returns all available return reasons
    pub fn all() -> &'static [ReturnReason] {
        &[
            ReturnReason::Defective,
            ReturnReason::WrongItem,
            ReturnReason::NotAsDescribed,
            ReturnReason::ChangedMind,
            ReturnReason::NoLongerNeeded,
            ReturnReason::BetterPriceElsewhere,
            ReturnReason::DamagedInShipping,
            ReturnReason::Other,
        ]
    }

    /// Returns true if this is a seller-fault reason (typically qualifies for full refund)
    pub fn is_seller_fault(&self) -> bool {
        matches!(
            self,
            ReturnReason::Defective
                | ReturnReason::WrongItem
                | ReturnReason::NotAsDescribed
                | ReturnReason::DamagedInShipping
        )
    }

    /// Returns true if this is a customer-initiated reason
    pub fn is_customer_initiated(&self) -> bool {
        matches!(
            self,
            ReturnReason::ChangedMind
                | ReturnReason::NoLongerNeeded
                | ReturnReason::BetterPriceElsewhere
        )
    }
}

impl FromStr for ReturnReason {
    type Err = SalesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "defective" | "broken" | "faulty" => Ok(ReturnReason::Defective),
            "wrong_item" | "wrongitem" | "wrong" => Ok(ReturnReason::WrongItem),
            "not_as_described" | "notasdescribed" | "misdescribed" => {
                Ok(ReturnReason::NotAsDescribed)
            }
            "changed_mind" | "changedmind" => Ok(ReturnReason::ChangedMind),
            "no_longer_needed" | "nolongerneeded" | "not_needed" => {
                Ok(ReturnReason::NoLongerNeeded)
            }
            "better_price_elsewhere" | "betterpriceelsewhere" | "better_price" => {
                Ok(ReturnReason::BetterPriceElsewhere)
            }
            "damaged_in_shipping" | "damagedinshipping" | "shipping_damage" => {
                Ok(ReturnReason::DamagedInShipping)
            }
            "other" => Ok(ReturnReason::Other),
            _ => Err(SalesError::InvalidReturnReason),
        }
    }
}

impl fmt::Display for ReturnReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReturnReason::Defective => write!(f, "defective"),
            ReturnReason::WrongItem => write!(f, "wrong_item"),
            ReturnReason::NotAsDescribed => write!(f, "not_as_described"),
            ReturnReason::ChangedMind => write!(f, "changed_mind"),
            ReturnReason::NoLongerNeeded => write!(f, "no_longer_needed"),
            ReturnReason::BetterPriceElsewhere => write!(f, "better_price_elsewhere"),
            ReturnReason::DamagedInShipping => write!(f, "damaged_in_shipping"),
            ReturnReason::Other => write!(f, "other"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(
            ReturnReason::from_str("defective").unwrap(),
            ReturnReason::Defective
        );
        assert_eq!(
            ReturnReason::from_str("broken").unwrap(),
            ReturnReason::Defective
        );
        assert_eq!(
            ReturnReason::from_str("wrong_item").unwrap(),
            ReturnReason::WrongItem
        );
        assert_eq!(
            ReturnReason::from_str("changed_mind").unwrap(),
            ReturnReason::ChangedMind
        );
    }

    #[test]
    fn test_display() {
        assert_eq!(ReturnReason::Defective.to_string(), "defective");
        assert_eq!(ReturnReason::WrongItem.to_string(), "wrong_item");
    }

    #[test]
    fn test_predicates() {
        // Seller fault reasons
        assert!(ReturnReason::Defective.is_seller_fault());
        assert!(ReturnReason::WrongItem.is_seller_fault());
        assert!(ReturnReason::NotAsDescribed.is_seller_fault());
        assert!(ReturnReason::DamagedInShipping.is_seller_fault());

        // Customer initiated reasons
        assert!(ReturnReason::ChangedMind.is_customer_initiated());
        assert!(ReturnReason::NoLongerNeeded.is_customer_initiated());
        assert!(ReturnReason::BetterPriceElsewhere.is_customer_initiated());

        // Other
        assert!(!ReturnReason::Other.is_seller_fault());
        assert!(!ReturnReason::Other.is_customer_initiated());
    }
}

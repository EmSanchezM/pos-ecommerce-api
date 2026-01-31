//! PaymentMethod enum - available payment methods

use crate::SalesError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Available payment methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethod {
    /// Cash payment
    Cash,
    /// Credit card payment
    CreditCard,
    /// Debit card payment
    DebitCard,
    /// Bank transfer
    BankTransfer,
    /// PayPal payment
    PayPal,
    /// Store credit
    StoreCredit,
    /// Gift card
    GiftCard,
    /// Other payment method
    Other,
}

impl PaymentMethod {
    /// Returns all available payment methods
    pub fn all() -> &'static [PaymentMethod] {
        &[
            PaymentMethod::Cash,
            PaymentMethod::CreditCard,
            PaymentMethod::DebitCard,
            PaymentMethod::BankTransfer,
            PaymentMethod::PayPal,
            PaymentMethod::StoreCredit,
            PaymentMethod::GiftCard,
            PaymentMethod::Other,
        ]
    }

    /// Returns true if this is a cash payment
    pub fn is_cash(&self) -> bool {
        matches!(self, PaymentMethod::Cash)
    }

    /// Returns true if this is a card payment
    pub fn is_card(&self) -> bool {
        matches!(self, PaymentMethod::CreditCard | PaymentMethod::DebitCard)
    }

    /// Returns true if this payment method requires online processing
    pub fn requires_online_processing(&self) -> bool {
        matches!(
            self,
            PaymentMethod::CreditCard
                | PaymentMethod::DebitCard
                | PaymentMethod::PayPal
                | PaymentMethod::BankTransfer
        )
    }

    /// Returns true if change can be given for this payment method
    pub fn can_give_change(&self) -> bool {
        matches!(self, PaymentMethod::Cash)
    }
}

impl FromStr for PaymentMethod {
    type Err = SalesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "cash" => Ok(PaymentMethod::Cash),
            "credit_card" | "creditcard" | "credit" => Ok(PaymentMethod::CreditCard),
            "debit_card" | "debitcard" | "debit" => Ok(PaymentMethod::DebitCard),
            "bank_transfer" | "banktransfer" | "transfer" => Ok(PaymentMethod::BankTransfer),
            "paypal" => Ok(PaymentMethod::PayPal),
            "store_credit" | "storecredit" => Ok(PaymentMethod::StoreCredit),
            "gift_card" | "giftcard" | "gift" => Ok(PaymentMethod::GiftCard),
            "other" => Ok(PaymentMethod::Other),
            _ => Err(SalesError::InvalidPaymentMethod),
        }
    }
}

impl fmt::Display for PaymentMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaymentMethod::Cash => write!(f, "cash"),
            PaymentMethod::CreditCard => write!(f, "credit_card"),
            PaymentMethod::DebitCard => write!(f, "debit_card"),
            PaymentMethod::BankTransfer => write!(f, "bank_transfer"),
            PaymentMethod::PayPal => write!(f, "paypal"),
            PaymentMethod::StoreCredit => write!(f, "store_credit"),
            PaymentMethod::GiftCard => write!(f, "gift_card"),
            PaymentMethod::Other => write!(f, "other"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(PaymentMethod::from_str("cash").unwrap(), PaymentMethod::Cash);
        assert_eq!(
            PaymentMethod::from_str("credit_card").unwrap(),
            PaymentMethod::CreditCard
        );
        assert_eq!(
            PaymentMethod::from_str("creditcard").unwrap(),
            PaymentMethod::CreditCard
        );
        assert_eq!(
            PaymentMethod::from_str("credit").unwrap(),
            PaymentMethod::CreditCard
        );
    }

    #[test]
    fn test_display() {
        assert_eq!(PaymentMethod::Cash.to_string(), "cash");
        assert_eq!(PaymentMethod::CreditCard.to_string(), "credit_card");
    }

    #[test]
    fn test_predicates() {
        assert!(PaymentMethod::Cash.is_cash());
        assert!(PaymentMethod::Cash.can_give_change());
        assert!(!PaymentMethod::Cash.is_card());

        assert!(PaymentMethod::CreditCard.is_card());
        assert!(PaymentMethod::CreditCard.requires_online_processing());
        assert!(!PaymentMethod::CreditCard.can_give_change());

        assert!(PaymentMethod::DebitCard.is_card());
    }
}

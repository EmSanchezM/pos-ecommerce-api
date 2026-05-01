//! ManualPaymentKind enum - flavour of a manual (offline) payment.
//!
//! Covers the common Honduras cases:
//! - `BankTransfer`     - SPEI/ACH between bank accounts
//! - `AgencyDeposit`    - cash deposited at a bank branch
//! - `CashOnDelivery`   - eCommerce contra-entrega
//! - `Other`            - anything else (Western Union, Tigo Money, etc.)

use crate::PaymentsError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManualPaymentKind {
    BankTransfer,
    AgencyDeposit,
    CashOnDelivery,
    Other,
}

impl FromStr for ManualPaymentKind {
    type Err = PaymentsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "bank_transfer" | "transfer" | "transferencia" => Ok(Self::BankTransfer),
            "agency_deposit" | "deposit" | "deposito" => Ok(Self::AgencyDeposit),
            "cash_on_delivery" | "cod" | "contra_entrega" => Ok(Self::CashOnDelivery),
            "other" => Ok(Self::Other),
            _ => Err(PaymentsError::InvalidManualPaymentKind),
        }
    }
}

impl fmt::Display for ManualPaymentKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BankTransfer => write!(f, "bank_transfer"),
            Self::AgencyDeposit => write!(f, "agency_deposit"),
            Self::CashOnDelivery => write!(f, "cash_on_delivery"),
            Self::Other => write!(f, "other"),
        }
    }
}

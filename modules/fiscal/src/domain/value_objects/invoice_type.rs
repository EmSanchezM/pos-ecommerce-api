//! InvoiceType enum - classification of fiscal invoices

use crate::FiscalError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Classification of fiscal invoices
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvoiceType {
    /// Standard sales invoice
    Standard,
    /// Credit note (reversal of a previous invoice)
    CreditNote,
    /// Debit note (additional charge to a previous invoice)
    DebitNote,
    /// Proforma invoice (not fiscally binding)
    Proforma,
}

impl InvoiceType {
    /// Returns all available invoice types
    pub fn all() -> &'static [InvoiceType] {
        &[
            InvoiceType::Standard,
            InvoiceType::CreditNote,
            InvoiceType::DebitNote,
            InvoiceType::Proforma,
        ]
    }

    /// Returns true if this type requires a fiscal sequence number
    pub fn requires_fiscal_number(&self) -> bool {
        matches!(
            self,
            InvoiceType::Standard | InvoiceType::CreditNote | InvoiceType::DebitNote
        )
    }

    /// Returns true if this is a credit note
    pub fn is_credit_note(&self) -> bool {
        matches!(self, InvoiceType::CreditNote)
    }

    /// Returns true if this is a proforma
    pub fn is_proforma(&self) -> bool {
        matches!(self, InvoiceType::Proforma)
    }
}

impl FromStr for InvoiceType {
    type Err = FiscalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "standard" => Ok(InvoiceType::Standard),
            "credit_note" | "creditnote" => Ok(InvoiceType::CreditNote),
            "debit_note" | "debitnote" => Ok(InvoiceType::DebitNote),
            "proforma" => Ok(InvoiceType::Proforma),
            _ => Err(FiscalError::InvalidInvoiceType),
        }
    }
}

impl fmt::Display for InvoiceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvoiceType::Standard => write!(f, "standard"),
            InvoiceType::CreditNote => write!(f, "credit_note"),
            InvoiceType::DebitNote => write!(f, "debit_note"),
            InvoiceType::Proforma => write!(f, "proforma"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(
            InvoiceType::from_str("standard").unwrap(),
            InvoiceType::Standard
        );
        assert_eq!(
            InvoiceType::from_str("credit_note").unwrap(),
            InvoiceType::CreditNote
        );
        assert_eq!(
            InvoiceType::from_str("debit_note").unwrap(),
            InvoiceType::DebitNote
        );
        assert_eq!(
            InvoiceType::from_str("proforma").unwrap(),
            InvoiceType::Proforma
        );
    }

    #[test]
    fn test_display() {
        assert_eq!(InvoiceType::Standard.to_string(), "standard");
        assert_eq!(InvoiceType::CreditNote.to_string(), "credit_note");
    }

    #[test]
    fn test_requires_fiscal_number() {
        assert!(InvoiceType::Standard.requires_fiscal_number());
        assert!(InvoiceType::CreditNote.requires_fiscal_number());
        assert!(InvoiceType::DebitNote.requires_fiscal_number());
        assert!(!InvoiceType::Proforma.requires_fiscal_number());
    }
}

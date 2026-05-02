//! BankTransaction — one line on a bank statement that we've recorded into
//! our books. `amount` follows accounting convention (positive = money in,
//! negative = money out); the `txn_type` field disambiguates "why".
//!
//! `reconciled` flips to true when a `CashDeposit` matches this row (or when
//! manually marked during reconciliation closing).

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::CashManagementError;
use crate::domain::value_objects::{BankAccountId, BankTransactionId, BankTransactionType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankTransaction {
    id: BankTransactionId,
    bank_account_id: BankAccountId,
    txn_type: BankTransactionType,
    amount: Decimal,
    reference: Option<String>,
    description: Option<String>,
    occurred_at: DateTime<Utc>,
    reconciled: bool,
    reconciliation_id: Option<Uuid>,
    created_by: Option<Uuid>,
    created_at: DateTime<Utc>,
}

impl BankTransaction {
    pub fn record(
        bank_account_id: BankAccountId,
        txn_type: BankTransactionType,
        amount: Decimal,
        reference: Option<String>,
        description: Option<String>,
        occurred_at: DateTime<Utc>,
        created_by: Option<Uuid>,
    ) -> Result<Self, CashManagementError> {
        // Domain rule: type and sign must agree. Inflow types must be
        // non-negative; outflow types must be non-positive. Adjustment is the
        // escape hatch — accountants need it for corrections, signed either way.
        if !matches!(txn_type, BankTransactionType::Adjustment) {
            let inflow = txn_type.is_inflow();
            if inflow && amount < Decimal::ZERO {
                return Err(CashManagementError::NegativeAmount);
            }
            if !inflow && amount > Decimal::ZERO {
                return Err(CashManagementError::NegativeAmount);
            }
        }
        Ok(Self {
            id: BankTransactionId::new(),
            bank_account_id,
            txn_type,
            amount,
            reference,
            description,
            occurred_at,
            reconciled: false,
            reconciliation_id: None,
            created_by,
            created_at: Utc::now(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: BankTransactionId,
        bank_account_id: BankAccountId,
        txn_type: BankTransactionType,
        amount: Decimal,
        reference: Option<String>,
        description: Option<String>,
        occurred_at: DateTime<Utc>,
        reconciled: bool,
        reconciliation_id: Option<Uuid>,
        created_by: Option<Uuid>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            bank_account_id,
            txn_type,
            amount,
            reference,
            description,
            occurred_at,
            reconciled,
            reconciliation_id,
            created_by,
            created_at,
        }
    }

    pub fn mark_reconciled(&mut self, reconciliation_id: Option<Uuid>) {
        self.reconciled = true;
        self.reconciliation_id = reconciliation_id;
    }

    pub fn id(&self) -> BankTransactionId {
        self.id
    }
    pub fn bank_account_id(&self) -> BankAccountId {
        self.bank_account_id
    }
    pub fn txn_type(&self) -> BankTransactionType {
        self.txn_type
    }
    pub fn amount(&self) -> Decimal {
        self.amount
    }
    pub fn reference(&self) -> Option<&str> {
        self.reference.as_deref()
    }
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
    pub fn reconciled(&self) -> bool {
        self.reconciled
    }
    pub fn reconciliation_id(&self) -> Option<Uuid> {
        self.reconciliation_id
    }
    pub fn created_by(&self) -> Option<Uuid> {
        self.created_by
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn account() -> BankAccountId {
        BankAccountId::new()
    }

    #[test]
    fn deposit_with_negative_amount_is_rejected() {
        let err = BankTransaction::record(
            account(),
            BankTransactionType::Deposit,
            dec!(-50),
            None,
            None,
            Utc::now(),
            None,
        )
        .unwrap_err();
        assert!(matches!(err, CashManagementError::NegativeAmount));
    }

    #[test]
    fn fee_with_positive_amount_is_rejected() {
        let err = BankTransaction::record(
            account(),
            BankTransactionType::Fee,
            dec!(25),
            None,
            None,
            Utc::now(),
            None,
        )
        .unwrap_err();
        assert!(matches!(err, CashManagementError::NegativeAmount));
    }

    #[test]
    fn adjustment_accepts_either_sign() {
        BankTransaction::record(
            account(),
            BankTransactionType::Adjustment,
            dec!(-10),
            None,
            None,
            Utc::now(),
            None,
        )
        .unwrap();
        BankTransaction::record(
            account(),
            BankTransactionType::Adjustment,
            dec!(10),
            None,
            None,
            Utc::now(),
            None,
        )
        .unwrap();
    }
}

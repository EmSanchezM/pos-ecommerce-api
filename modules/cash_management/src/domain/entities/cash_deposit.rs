//! CashDeposit — manager records that money from a closed `cashier_shift`
//! is being moved to the bank. The aggregate enforces the
//! `pending → deposited → reconciled` workflow.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::CashManagementError;
use crate::domain::value_objects::{
    BankAccountId, BankTransactionId, CashDepositId, CashDepositStatus,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashDeposit {
    id: CashDepositId,
    cashier_shift_id: Uuid,
    bank_account_id: BankAccountId,
    amount: Decimal,
    deposit_date: NaiveDate,
    deposit_slip_number: Option<String>,
    deposited_by_user_id: Option<Uuid>,
    bank_transaction_id: Option<BankTransactionId>,
    status: CashDepositStatus,
    notes: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl CashDeposit {
    pub fn create(
        cashier_shift_id: Uuid,
        bank_account_id: BankAccountId,
        amount: Decimal,
        deposit_date: NaiveDate,
        notes: Option<String>,
    ) -> Result<Self, CashManagementError> {
        if amount <= Decimal::ZERO {
            return Err(CashManagementError::NegativeAmount);
        }
        let now = Utc::now();
        Ok(Self {
            id: CashDepositId::new(),
            cashier_shift_id,
            bank_account_id,
            amount,
            deposit_date,
            deposit_slip_number: None,
            deposited_by_user_id: None,
            bank_transaction_id: None,
            status: CashDepositStatus::Pending,
            notes,
            created_at: now,
            updated_at: now,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: CashDepositId,
        cashier_shift_id: Uuid,
        bank_account_id: BankAccountId,
        amount: Decimal,
        deposit_date: NaiveDate,
        deposit_slip_number: Option<String>,
        deposited_by_user_id: Option<Uuid>,
        bank_transaction_id: Option<BankTransactionId>,
        status: CashDepositStatus,
        notes: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            cashier_shift_id,
            bank_account_id,
            amount,
            deposit_date,
            deposit_slip_number,
            deposited_by_user_id,
            bank_transaction_id,
            status,
            notes,
            created_at,
            updated_at,
        }
    }

    /// Move from `pending` to `deposited` after the manager confirms the
    /// money reached the bank. Slip number is required at this point —
    /// otherwise auditing has no paper trail.
    pub fn mark_deposited(
        &mut self,
        slip_number: String,
        deposited_by: Uuid,
    ) -> Result<(), CashManagementError> {
        if !self.status.can_transition_to(CashDepositStatus::Deposited) {
            return Err(CashManagementError::InvalidDepositTransition {
                from: self.status.to_string(),
                to: CashDepositStatus::Deposited.to_string(),
            });
        }
        self.deposit_slip_number = Some(slip_number);
        self.deposited_by_user_id = Some(deposited_by);
        self.status = CashDepositStatus::Deposited;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Move from `deposited` to `reconciled` once linked to a `BankTransaction`.
    pub fn mark_reconciled(
        &mut self,
        bank_transaction_id: BankTransactionId,
    ) -> Result<(), CashManagementError> {
        if !self.status.can_transition_to(CashDepositStatus::Reconciled) {
            return Err(CashManagementError::InvalidDepositTransition {
                from: self.status.to_string(),
                to: CashDepositStatus::Reconciled.to_string(),
            });
        }
        self.bank_transaction_id = Some(bank_transaction_id);
        self.status = CashDepositStatus::Reconciled;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn id(&self) -> CashDepositId {
        self.id
    }
    pub fn cashier_shift_id(&self) -> Uuid {
        self.cashier_shift_id
    }
    pub fn bank_account_id(&self) -> BankAccountId {
        self.bank_account_id
    }
    pub fn amount(&self) -> Decimal {
        self.amount
    }
    pub fn deposit_date(&self) -> NaiveDate {
        self.deposit_date
    }
    pub fn deposit_slip_number(&self) -> Option<&str> {
        self.deposit_slip_number.as_deref()
    }
    pub fn deposited_by_user_id(&self) -> Option<Uuid> {
        self.deposited_by_user_id
    }
    pub fn bank_transaction_id(&self) -> Option<BankTransactionId> {
        self.bank_transaction_id
    }
    pub fn status(&self) -> CashDepositStatus {
        self.status
    }
    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;
    use uuid::{NoContext, Timestamp};

    fn fresh() -> Uuid {
        Uuid::new_v7(Timestamp::now(NoContext))
    }

    fn build() -> CashDeposit {
        CashDeposit::create(
            fresh(),
            BankAccountId::new(),
            dec!(2500),
            NaiveDate::from_ymd_opt(2026, 5, 2).unwrap(),
            None,
        )
        .unwrap()
    }

    #[test]
    fn full_workflow() {
        let mut d = build();
        assert_eq!(d.status(), CashDepositStatus::Pending);
        d.mark_deposited("CD-001".into(), fresh()).unwrap();
        assert_eq!(d.status(), CashDepositStatus::Deposited);
        assert!(d.deposit_slip_number().is_some());
        d.mark_reconciled(BankTransactionId::new()).unwrap();
        assert_eq!(d.status(), CashDepositStatus::Reconciled);
    }

    #[test]
    fn cannot_reconcile_before_depositing() {
        let mut d = build();
        let err = d.mark_reconciled(BankTransactionId::new()).unwrap_err();
        assert!(matches!(
            err,
            CashManagementError::InvalidDepositTransition { .. }
        ));
    }

    #[test]
    fn cannot_create_with_zero_amount() {
        let err = CashDeposit::create(
            fresh(),
            BankAccountId::new(),
            dec!(0),
            NaiveDate::from_ymd_opt(2026, 5, 2).unwrap(),
            None,
        )
        .unwrap_err();
        assert!(matches!(err, CashManagementError::NegativeAmount));
    }
}

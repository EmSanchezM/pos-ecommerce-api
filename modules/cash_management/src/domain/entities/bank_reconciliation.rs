//! BankReconciliation — periodic check that the book balance, computed from
//! the entries we recorded, matches the bank statement balance. The aggregate
//! enforces the `in_progress → completed` transition; closing requires the
//! caller to provide the actual statement and book balances and tolerates a
//! configurable difference (zero in v1).

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::CashManagementError;
use crate::domain::value_objects::{BankAccountId, BankReconciliationId, BankReconciliationStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankReconciliation {
    id: BankReconciliationId,
    bank_account_id: BankAccountId,
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
    opening_balance: Decimal,
    closing_book_balance: Option<Decimal>,
    statement_balance: Option<Decimal>,
    status: BankReconciliationStatus,
    completed_at: Option<DateTime<Utc>>,
    completed_by: Option<Uuid>,
    notes: Option<String>,
    created_at: DateTime<Utc>,
}

impl BankReconciliation {
    pub fn start(
        bank_account_id: BankAccountId,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        opening_balance: Decimal,
    ) -> Result<Self, CashManagementError> {
        if period_start >= period_end {
            return Err(CashManagementError::InvalidReconciliationRange);
        }
        Ok(Self {
            id: BankReconciliationId::new(),
            bank_account_id,
            period_start,
            period_end,
            opening_balance,
            closing_book_balance: None,
            statement_balance: None,
            status: BankReconciliationStatus::InProgress,
            completed_at: None,
            completed_by: None,
            notes: None,
            created_at: Utc::now(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: BankReconciliationId,
        bank_account_id: BankAccountId,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        opening_balance: Decimal,
        closing_book_balance: Option<Decimal>,
        statement_balance: Option<Decimal>,
        status: BankReconciliationStatus,
        completed_at: Option<DateTime<Utc>>,
        completed_by: Option<Uuid>,
        notes: Option<String>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            bank_account_id,
            period_start,
            period_end,
            opening_balance,
            closing_book_balance,
            statement_balance,
            status,
            completed_at,
            completed_by,
            notes,
            created_at,
        }
    }

    /// Close the reconciliation. Requires statement and book balances; rejects
    /// if they don't match (v1 tolerates zero variance — accountants want to
    /// see the actual mismatch and post an `Adjustment` transaction first).
    pub fn close(
        &mut self,
        book_balance: Decimal,
        statement_balance: Decimal,
        completed_by: Uuid,
        notes: Option<String>,
    ) -> Result<(), CashManagementError> {
        if !matches!(self.status, BankReconciliationStatus::InProgress) {
            return Err(CashManagementError::InvalidReconciliationTransition {
                from: self.status.to_string(),
                to: BankReconciliationStatus::Completed.to_string(),
            });
        }
        let difference = statement_balance - book_balance;
        if !difference.is_zero() {
            return Err(CashManagementError::ReconciliationUnbalanced {
                statement: statement_balance,
                book: book_balance,
                difference,
            });
        }
        self.closing_book_balance = Some(book_balance);
        self.statement_balance = Some(statement_balance);
        self.status = BankReconciliationStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.completed_by = Some(completed_by);
        if notes.is_some() {
            self.notes = notes;
        }
        Ok(())
    }

    pub fn id(&self) -> BankReconciliationId {
        self.id
    }
    pub fn bank_account_id(&self) -> BankAccountId {
        self.bank_account_id
    }
    pub fn period_start(&self) -> DateTime<Utc> {
        self.period_start
    }
    pub fn period_end(&self) -> DateTime<Utc> {
        self.period_end
    }
    pub fn opening_balance(&self) -> Decimal {
        self.opening_balance
    }
    pub fn closing_book_balance(&self) -> Option<Decimal> {
        self.closing_book_balance
    }
    pub fn statement_balance(&self) -> Option<Decimal> {
        self.statement_balance
    }
    pub fn status(&self) -> BankReconciliationStatus {
        self.status
    }
    pub fn completed_at(&self) -> Option<DateTime<Utc>> {
        self.completed_at
    }
    pub fn completed_by(&self) -> Option<Uuid> {
        self.completed_by
    }
    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use rust_decimal_macros::dec;
    use uuid::{NoContext, Timestamp};

    fn fresh() -> Uuid {
        Uuid::new_v7(Timestamp::now(NoContext))
    }

    fn build() -> BankReconciliation {
        BankReconciliation::start(
            BankAccountId::new(),
            Utc.with_ymd_and_hms(2026, 5, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap(),
            dec!(10000),
        )
        .unwrap()
    }

    #[test]
    fn close_balanced_succeeds() {
        let mut r = build();
        r.close(dec!(12500), dec!(12500), fresh(), None).unwrap();
        assert_eq!(r.status(), BankReconciliationStatus::Completed);
    }

    #[test]
    fn close_unbalanced_is_rejected() {
        let mut r = build();
        let err = r
            .close(dec!(12500), dec!(12800), fresh(), None)
            .unwrap_err();
        assert!(matches!(
            err,
            CashManagementError::ReconciliationUnbalanced { .. }
        ));
        assert_eq!(r.status(), BankReconciliationStatus::InProgress);
    }

    #[test]
    fn cannot_close_twice() {
        let mut r = build();
        r.close(dec!(100), dec!(100), fresh(), None).unwrap();
        let err = r.close(dec!(200), dec!(200), fresh(), None).unwrap_err();
        assert!(matches!(
            err,
            CashManagementError::InvalidReconciliationTransition { .. }
        ));
    }

    #[test]
    fn invalid_range_is_rejected() {
        let err = BankReconciliation::start(
            BankAccountId::new(),
            Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 5, 1, 0, 0, 0).unwrap(),
            dec!(0),
        )
        .unwrap_err();
        assert!(matches!(
            err,
            CashManagementError::InvalidReconciliationRange
        ));
    }
}

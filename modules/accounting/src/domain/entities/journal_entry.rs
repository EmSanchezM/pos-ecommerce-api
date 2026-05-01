//! JournalEntry + JournalLine — the heart of double-entry bookkeeping.
//!
//! A `JournalEntry` is a balanced set of debit and credit lines. The aggregate
//! enforces three invariants in `posted` state:
//!   1. At least two lines.
//!   2. Each line has either debit > 0 OR credit > 0 (never both, never zero).
//!   3. Sum of debits == sum of credits.
//!
//! Lines also carry an optional `store_id` so multi-store deployments can
//! filter reports per location without joining back to source documents.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AccountingError;
use crate::domain::value_objects::{
    AccountId, AccountingPeriodId, JournalEntryId, JournalEntryStatus, JournalLineId,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalLine {
    id: JournalLineId,
    account_id: AccountId,
    store_id: Option<Uuid>,
    line_number: i32,
    debit: Decimal,
    credit: Decimal,
    description: Option<String>,
}

impl JournalLine {
    pub fn debit_line(
        account_id: AccountId,
        store_id: Option<Uuid>,
        line_number: i32,
        amount: Decimal,
        description: Option<String>,
    ) -> Result<Self, AccountingError> {
        if amount <= Decimal::ZERO {
            return Err(AccountingError::NegativeAmount);
        }
        Ok(Self {
            id: JournalLineId::new(),
            account_id,
            store_id,
            line_number,
            debit: amount,
            credit: Decimal::ZERO,
            description,
        })
    }

    pub fn credit_line(
        account_id: AccountId,
        store_id: Option<Uuid>,
        line_number: i32,
        amount: Decimal,
        description: Option<String>,
    ) -> Result<Self, AccountingError> {
        if amount <= Decimal::ZERO {
            return Err(AccountingError::NegativeAmount);
        }
        Ok(Self {
            id: JournalLineId::new(),
            account_id,
            store_id,
            line_number,
            debit: Decimal::ZERO,
            credit: amount,
            description,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: JournalLineId,
        account_id: AccountId,
        store_id: Option<Uuid>,
        line_number: i32,
        debit: Decimal,
        credit: Decimal,
        description: Option<String>,
    ) -> Self {
        Self {
            id,
            account_id,
            store_id,
            line_number,
            debit,
            credit,
            description,
        }
    }

    pub fn id(&self) -> JournalLineId {
        self.id
    }
    pub fn account_id(&self) -> AccountId {
        self.account_id
    }
    pub fn store_id(&self) -> Option<Uuid> {
        self.store_id
    }
    pub fn line_number(&self) -> i32 {
        self.line_number
    }
    pub fn debit(&self) -> Decimal {
        self.debit
    }
    pub fn credit(&self) -> Decimal {
        self.credit
    }
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn validate_amounts(&self) -> Result<(), AccountingError> {
        if self.debit < Decimal::ZERO || self.credit < Decimal::ZERO {
            return Err(AccountingError::NegativeAmount);
        }
        // Exactly one side must be > 0.
        let debit_set = self.debit > Decimal::ZERO;
        let credit_set = self.credit > Decimal::ZERO;
        if debit_set == credit_set {
            return Err(AccountingError::InvalidLineAmounts);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    id: JournalEntryId,
    period_id: AccountingPeriodId,
    entry_number: i64,
    description: String,
    /// Optional source document type (e.g. `"sale"`, `"goods_receipt"`).
    source_type: Option<String>,
    /// Optional source document id.
    source_id: Option<Uuid>,
    status: JournalEntryStatus,
    posted_at: Option<DateTime<Utc>>,
    created_by: Option<Uuid>,
    lines: Vec<JournalLine>,
    created_at: DateTime<Utc>,
}

impl JournalEntry {
    /// Build a new entry in `draft` state and validate its lines balance.
    pub fn create(
        period_id: AccountingPeriodId,
        entry_number: i64,
        description: impl Into<String>,
        source_type: Option<String>,
        source_id: Option<Uuid>,
        created_by: Option<Uuid>,
        lines: Vec<JournalLine>,
    ) -> Result<Self, AccountingError> {
        let entry = Self {
            id: JournalEntryId::new(),
            period_id,
            entry_number,
            description: description.into(),
            source_type,
            source_id,
            status: JournalEntryStatus::Draft,
            posted_at: None,
            created_by,
            lines,
            created_at: Utc::now(),
        };
        entry.validate()?;
        Ok(entry)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: JournalEntryId,
        period_id: AccountingPeriodId,
        entry_number: i64,
        description: String,
        source_type: Option<String>,
        source_id: Option<Uuid>,
        status: JournalEntryStatus,
        posted_at: Option<DateTime<Utc>>,
        created_by: Option<Uuid>,
        lines: Vec<JournalLine>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            period_id,
            entry_number,
            description,
            source_type,
            source_id,
            status,
            posted_at,
            created_by,
            lines,
            created_at,
        }
    }

    /// Move the entry to `posted`. Re-validates balance and stamps `posted_at`.
    pub fn post(&mut self) -> Result<(), AccountingError> {
        if self.status != JournalEntryStatus::Draft {
            return Err(AccountingError::InvalidStatusTransition {
                from: self.status.to_string(),
                to: "posted".into(),
            });
        }
        self.validate()?;
        self.status = JournalEntryStatus::Posted;
        self.posted_at = Some(Utc::now());
        Ok(())
    }

    /// Mark this entry as voided. Voiding does not delete the rows; a reversing
    /// entry must also be created so reports stay consistent. Voiding only
    /// makes sense on a posted entry.
    pub fn void(&mut self) -> Result<(), AccountingError> {
        if self.status != JournalEntryStatus::Posted {
            return Err(AccountingError::InvalidStatusTransition {
                from: self.status.to_string(),
                to: "voided".into(),
            });
        }
        self.status = JournalEntryStatus::Voided;
        Ok(())
    }

    fn validate(&self) -> Result<(), AccountingError> {
        if self.lines.len() < 2 {
            return Err(AccountingError::NotEnoughLines);
        }
        for line in &self.lines {
            line.validate_amounts()?;
        }
        let (debits, credits) = self.totals();
        if debits != credits {
            return Err(AccountingError::Unbalanced { debits, credits });
        }
        Ok(())
    }

    /// Sum of debits and credits across all lines.
    pub fn totals(&self) -> (Decimal, Decimal) {
        let debits: Decimal = self.lines.iter().map(|l| l.debit).sum();
        let credits: Decimal = self.lines.iter().map(|l| l.credit).sum();
        (debits, credits)
    }

    pub fn id(&self) -> JournalEntryId {
        self.id
    }
    pub fn period_id(&self) -> AccountingPeriodId {
        self.period_id
    }
    pub fn entry_number(&self) -> i64 {
        self.entry_number
    }
    pub fn description(&self) -> &str {
        &self.description
    }
    pub fn source_type(&self) -> Option<&str> {
        self.source_type.as_deref()
    }
    pub fn source_id(&self) -> Option<Uuid> {
        self.source_id
    }
    pub fn status(&self) -> JournalEntryStatus {
        self.status
    }
    pub fn posted_at(&self) -> Option<DateTime<Utc>> {
        self.posted_at
    }
    pub fn created_by(&self) -> Option<Uuid> {
        self.created_by
    }
    pub fn lines(&self) -> &[JournalLine] {
        &self.lines
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn cash() -> AccountId {
        AccountId::new()
    }
    fn revenue() -> AccountId {
        AccountId::new()
    }

    fn balanced_lines() -> Vec<JournalLine> {
        vec![
            JournalLine::debit_line(cash(), None, 1, dec!(100), None).unwrap(),
            JournalLine::credit_line(revenue(), None, 2, dec!(100), None).unwrap(),
        ]
    }

    #[test]
    fn create_balanced_entry_succeeds() {
        let e = JournalEntry::create(
            AccountingPeriodId::new(),
            1,
            "Test sale",
            Some("sale".into()),
            None,
            None,
            balanced_lines(),
        )
        .unwrap();
        assert_eq!(e.status(), JournalEntryStatus::Draft);
        let (d, c) = e.totals();
        assert_eq!(d, c);
    }

    #[test]
    fn unbalanced_entry_is_rejected() {
        let lines = vec![
            JournalLine::debit_line(cash(), None, 1, dec!(100), None).unwrap(),
            JournalLine::credit_line(revenue(), None, 2, dec!(99), None).unwrap(),
        ];
        let err =
            JournalEntry::create(AccountingPeriodId::new(), 1, "Bad", None, None, None, lines)
                .unwrap_err();
        assert!(matches!(err, AccountingError::Unbalanced { .. }));
    }

    #[test]
    fn single_line_is_rejected() {
        let lines = vec![JournalLine::debit_line(cash(), None, 1, dec!(100), None).unwrap()];
        let err =
            JournalEntry::create(AccountingPeriodId::new(), 1, "Bad", None, None, None, lines)
                .unwrap_err();
        assert!(matches!(err, AccountingError::NotEnoughLines));
    }

    #[test]
    fn post_transitions_draft_to_posted() {
        let mut e = JournalEntry::create(
            AccountingPeriodId::new(),
            1,
            "x",
            None,
            None,
            None,
            balanced_lines(),
        )
        .unwrap();
        e.post().unwrap();
        assert_eq!(e.status(), JournalEntryStatus::Posted);
        assert!(e.posted_at().is_some());
        // Cannot post again.
        assert!(e.post().is_err());
    }

    #[test]
    fn debit_line_rejects_zero_amount() {
        assert!(JournalLine::debit_line(cash(), None, 1, dec!(0), None).is_err());
    }
}

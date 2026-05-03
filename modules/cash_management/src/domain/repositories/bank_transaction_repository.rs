use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::CashManagementError;
use crate::domain::entities::BankTransaction;
use crate::domain::value_objects::{BankAccountId, BankTransactionId};

#[derive(Debug, Clone, Default)]
pub struct BankTransactionFilter {
    pub bank_account_id: Option<BankAccountId>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub reconciled: Option<bool>,
}

#[async_trait]
pub trait BankTransactionRepository: Send + Sync {
    async fn save(&self, txn: &BankTransaction) -> Result<(), CashManagementError>;

    async fn update(&self, txn: &BankTransaction) -> Result<(), CashManagementError>;

    async fn find_by_id(
        &self,
        id: BankTransactionId,
    ) -> Result<Option<BankTransaction>, CashManagementError>;

    async fn list(
        &self,
        filter: BankTransactionFilter,
    ) -> Result<Vec<BankTransaction>, CashManagementError>;

    /// Compute the book balance for a reconciliation: `opening_balance` +
    /// sum of every transaction whose `occurred_at` falls in
    /// `[period_start, period_end]`. The opening is supplied by the caller —
    /// it's typically the previous reconciliation's closing balance.
    async fn book_balance(
        &self,
        bank_account_id: BankAccountId,
        opening_balance: Decimal,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<Decimal, CashManagementError>;

    /// True if the transaction already has a deposit linked to it; the
    /// link-deposit flow refuses to overwrite that link.
    async fn has_linked_deposit(
        &self,
        bank_transaction_id: BankTransactionId,
    ) -> Result<bool, CashManagementError>;

    /// Marks every transaction in [from, to] for the account as reconciled
    /// against the given reconciliation id. Used at close time.
    async fn mark_range_reconciled(
        &self,
        bank_account_id: BankAccountId,
        reconciliation_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<u64, CashManagementError>;
}

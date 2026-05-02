use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::{BankAccount, BankReconciliation, BankTransaction, CashDeposit};
use crate::domain::value_objects::{
    BankAccountType, BankReconciliationStatus, BankTransactionType, CashDepositStatus,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankAccountResponse {
    pub id: Uuid,
    pub store_id: Uuid,
    pub bank_name: String,
    pub account_number: String,
    pub account_type: BankAccountType,
    pub currency: String,
    pub current_balance: Decimal,
    pub is_active: bool,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&BankAccount> for BankAccountResponse {
    fn from(a: &BankAccount) -> Self {
        Self {
            id: a.id().into_uuid(),
            store_id: a.store_id(),
            bank_name: a.bank_name().to_string(),
            account_number: a.account_number().to_string(),
            account_type: a.account_type(),
            currency: a.currency().to_string(),
            current_balance: a.current_balance(),
            is_active: a.is_active(),
            version: a.version(),
            created_at: a.created_at(),
            updated_at: a.updated_at(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankTransactionResponse {
    pub id: Uuid,
    pub bank_account_id: Uuid,
    pub txn_type: BankTransactionType,
    pub amount: Decimal,
    pub reference: Option<String>,
    pub description: Option<String>,
    pub occurred_at: DateTime<Utc>,
    pub reconciled: bool,
    pub reconciliation_id: Option<Uuid>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl From<&BankTransaction> for BankTransactionResponse {
    fn from(t: &BankTransaction) -> Self {
        Self {
            id: t.id().into_uuid(),
            bank_account_id: t.bank_account_id().into_uuid(),
            txn_type: t.txn_type(),
            amount: t.amount(),
            reference: t.reference().map(|s| s.to_string()),
            description: t.description().map(|s| s.to_string()),
            occurred_at: t.occurred_at(),
            reconciled: t.reconciled(),
            reconciliation_id: t.reconciliation_id(),
            created_by: t.created_by(),
            created_at: t.created_at(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashDepositResponse {
    pub id: Uuid,
    pub cashier_shift_id: Uuid,
    pub bank_account_id: Uuid,
    pub amount: Decimal,
    pub deposit_date: NaiveDate,
    pub deposit_slip_number: Option<String>,
    pub deposited_by_user_id: Option<Uuid>,
    pub bank_transaction_id: Option<Uuid>,
    pub status: CashDepositStatus,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&CashDeposit> for CashDepositResponse {
    fn from(d: &CashDeposit) -> Self {
        Self {
            id: d.id().into_uuid(),
            cashier_shift_id: d.cashier_shift_id(),
            bank_account_id: d.bank_account_id().into_uuid(),
            amount: d.amount(),
            deposit_date: d.deposit_date(),
            deposit_slip_number: d.deposit_slip_number().map(|s| s.to_string()),
            deposited_by_user_id: d.deposited_by_user_id(),
            bank_transaction_id: d.bank_transaction_id().map(|t| t.into_uuid()),
            status: d.status(),
            notes: d.notes().map(|s| s.to_string()),
            created_at: d.created_at(),
            updated_at: d.updated_at(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankReconciliationResponse {
    pub id: Uuid,
    pub bank_account_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub opening_balance: Decimal,
    pub closing_book_balance: Option<Decimal>,
    pub statement_balance: Option<Decimal>,
    pub status: BankReconciliationStatus,
    pub completed_at: Option<DateTime<Utc>>,
    pub completed_by: Option<Uuid>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<&BankReconciliation> for BankReconciliationResponse {
    fn from(r: &BankReconciliation) -> Self {
        Self {
            id: r.id().into_uuid(),
            bank_account_id: r.bank_account_id().into_uuid(),
            period_start: r.period_start(),
            period_end: r.period_end(),
            opening_balance: r.opening_balance(),
            closing_book_balance: r.closing_book_balance(),
            statement_balance: r.statement_balance(),
            status: r.status(),
            completed_at: r.completed_at(),
            completed_by: r.completed_by(),
            notes: r.notes().map(|s| s.to_string()),
            created_at: r.created_at(),
        }
    }
}

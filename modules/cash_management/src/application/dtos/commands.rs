use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::value_objects::{BankAccountId, BankAccountType, BankTransactionType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBankAccountCommand {
    pub store_id: Uuid,
    pub bank_name: String,
    pub account_number: String,
    pub account_type: BankAccountType,
    #[serde(default = "default_currency")]
    pub currency: String,
    #[serde(default)]
    pub opening_balance: Decimal,
}

fn default_currency() -> String {
    "HNL".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBankAccountCommand {
    pub bank_name: Option<String>,
    pub account_type: Option<BankAccountType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordBankTransactionCommand {
    pub bank_account_id: BankAccountId,
    pub txn_type: BankTransactionType,
    /// Signed amount: positive for inflows, negative for outflows.
    pub amount: Decimal,
    pub reference: Option<String>,
    pub description: Option<String>,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCashDepositCommand {
    pub cashier_shift_id: Uuid,
    pub bank_account_id: BankAccountId,
    pub amount: Decimal,
    pub deposit_date: NaiveDate,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkDepositSentCommand {
    pub deposit_slip_number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkDepositCommand {
    pub bank_transaction_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartReconciliationCommand {
    pub bank_account_id: BankAccountId,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub opening_balance: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseReconciliationCommand {
    /// Statement balance the bank reports for `period_end`.
    pub statement_balance: Decimal,
    pub notes: Option<String>,
}

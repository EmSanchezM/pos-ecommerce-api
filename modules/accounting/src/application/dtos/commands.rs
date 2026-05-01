use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::value_objects::{AccountId, AccountType, AccountingPeriodId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccountCommand {
    pub code: String,
    pub name: String,
    pub account_type: AccountType,
    pub parent_id: Option<AccountId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenPeriodCommand {
    pub name: String,
    pub fiscal_year: i32,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
}

/// Side a line should book to. The HTTP DTO uses this so callers don't have
/// to send `debit=0` / `credit=0` explicitly.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineSide {
    Debit,
    Credit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalLineCommand {
    pub account_id: AccountId,
    pub store_id: Option<Uuid>,
    pub side: LineSide,
    pub amount: Decimal,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostJournalEntryCommand {
    pub period_id: AccountingPeriodId,
    pub description: String,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub created_by: Option<Uuid>,
    pub lines: Vec<JournalLineCommand>,
}

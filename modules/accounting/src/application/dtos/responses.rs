use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::{
    Account, AccountingPeriod, JournalEntry, JournalLine, ProfitAndLossLine,
};
use crate::domain::value_objects::{AccountType, JournalEntryStatus, PeriodStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountResponse {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub account_type: AccountType,
    pub parent_id: Option<Uuid>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&Account> for AccountResponse {
    fn from(a: &Account) -> Self {
        Self {
            id: a.id().into_uuid(),
            code: a.code().to_string(),
            name: a.name().to_string(),
            account_type: a.account_type(),
            parent_id: a.parent_id().map(|p| p.into_uuid()),
            is_active: a.is_active(),
            created_at: a.created_at(),
            updated_at: a.updated_at(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountingPeriodResponse {
    pub id: Uuid,
    pub name: String,
    pub fiscal_year: i32,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub status: PeriodStatus,
    pub closed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&AccountingPeriod> for AccountingPeriodResponse {
    fn from(p: &AccountingPeriod) -> Self {
        Self {
            id: p.id().into_uuid(),
            name: p.name().to_string(),
            fiscal_year: p.fiscal_year(),
            starts_at: p.starts_at(),
            ends_at: p.ends_at(),
            status: p.status(),
            closed_at: p.closed_at(),
            created_at: p.created_at(),
            updated_at: p.updated_at(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalLineResponse {
    pub id: Uuid,
    pub account_id: Uuid,
    pub store_id: Option<Uuid>,
    pub line_number: i32,
    pub debit: Decimal,
    pub credit: Decimal,
    pub description: Option<String>,
}

impl From<&JournalLine> for JournalLineResponse {
    fn from(l: &JournalLine) -> Self {
        Self {
            id: l.id().into_uuid(),
            account_id: l.account_id().into_uuid(),
            store_id: l.store_id(),
            line_number: l.line_number(),
            debit: l.debit(),
            credit: l.credit(),
            description: l.description().map(|s| s.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntryResponse {
    pub id: Uuid,
    pub period_id: Uuid,
    pub entry_number: i64,
    pub description: String,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub status: JournalEntryStatus,
    pub posted_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub lines: Vec<JournalLineResponse>,
    pub total_debit: Decimal,
    pub total_credit: Decimal,
    pub created_at: DateTime<Utc>,
}

impl From<&JournalEntry> for JournalEntryResponse {
    fn from(e: &JournalEntry) -> Self {
        let (total_debit, total_credit) = e.totals();
        Self {
            id: e.id().into_uuid(),
            period_id: e.period_id().into_uuid(),
            entry_number: e.entry_number(),
            description: e.description().to_string(),
            source_type: e.source_type().map(|s| s.to_string()),
            source_id: e.source_id(),
            status: e.status(),
            posted_at: e.posted_at(),
            created_by: e.created_by(),
            lines: e.lines().iter().map(JournalLineResponse::from).collect(),
            total_debit,
            total_credit,
            created_at: e.created_at(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfitAndLossLineResponse {
    pub account_id: Uuid,
    pub account_code: String,
    pub account_name: String,
    pub account_type: AccountType,
    pub net_amount: Decimal,
}

impl From<&ProfitAndLossLine> for ProfitAndLossLineResponse {
    fn from(l: &ProfitAndLossLine) -> Self {
        Self {
            account_id: l.account_id,
            account_code: l.account_code.clone(),
            account_name: l.account_name.clone(),
            account_type: l.account_type,
            net_amount: l.net_amount,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfitAndLossResponse {
    pub period_id: Uuid,
    pub store_id: Option<Uuid>,
    pub revenue: Vec<ProfitAndLossLineResponse>,
    pub expenses: Vec<ProfitAndLossLineResponse>,
    pub total_revenue: Decimal,
    pub total_expense: Decimal,
    pub net_income: Decimal,
}

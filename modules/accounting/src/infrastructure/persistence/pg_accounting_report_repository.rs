//! PostgreSQL implementation of AccountingReportRepository.
//!
//! Aggregates posted journal lines per account within a period. The P&L row
//! convention: revenue = sum(credit) - sum(debit), expense = sum(debit) -
//! sum(credit), so `net_amount` always reads as the line's contribution to
//! net income.

use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::AccountingError;
use crate::domain::entities::ProfitAndLossLine;
use crate::domain::repositories::AccountingReportRepository;
use crate::domain::value_objects::{AccountType, AccountingPeriodId};

pub struct PgAccountingReportRepository {
    pool: PgPool,
}

impl PgAccountingReportRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AccountingReportRepository for PgAccountingReportRepository {
    async fn profit_and_loss(
        &self,
        period_id: AccountingPeriodId,
        store_id: Option<Uuid>,
    ) -> Result<Vec<ProfitAndLossLine>, AccountingError> {
        let rows = sqlx::query_as::<_, PnlRowDb>(
            r#"
            SELECT
                a.id   AS account_id,
                a.code AS account_code,
                a.name AS account_name,
                a.account_type AS account_type,
                CASE
                    WHEN a.account_type = 'revenue' THEN COALESCE(SUM(jl.credit) - SUM(jl.debit), 0)
                    WHEN a.account_type = 'expense' THEN COALESCE(SUM(jl.debit) - SUM(jl.credit), 0)
                    ELSE 0
                END::NUMERIC AS net_amount
            FROM chart_of_accounts a
            JOIN journal_lines jl   ON jl.account_id = a.id
            JOIN journal_entries je ON je.id = jl.journal_entry_id
            WHERE je.period_id = $1
              AND je.status = 'posted'
              AND a.account_type IN ('revenue', 'expense')
              AND ($2::uuid IS NULL OR jl.store_id = $2)
            GROUP BY a.id, a.code, a.name, a.account_type
            ORDER BY a.account_type, a.code
            "#,
        )
        .bind(period_id.into_uuid())
        .bind(store_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(ProfitAndLossLine::try_from).collect()
    }
}

#[derive(sqlx::FromRow)]
struct PnlRowDb {
    account_id: Uuid,
    account_code: String,
    account_name: String,
    account_type: String,
    net_amount: Decimal,
}

impl TryFrom<PnlRowDb> for ProfitAndLossLine {
    type Error = AccountingError;

    fn try_from(row: PnlRowDb) -> Result<Self, Self::Error> {
        let account_type: AccountType = row.account_type.parse()?;
        Ok(ProfitAndLossLine {
            account_id: row.account_id,
            account_code: row.account_code,
            account_name: row.account_name,
            account_type,
            net_amount: row.net_amount,
        })
    }
}

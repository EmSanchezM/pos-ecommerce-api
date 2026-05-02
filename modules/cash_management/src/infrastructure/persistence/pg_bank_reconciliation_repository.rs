use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::CashManagementError;
use crate::domain::entities::BankReconciliation;
use crate::domain::repositories::BankReconciliationRepository;
use crate::domain::value_objects::{BankAccountId, BankReconciliationId, BankReconciliationStatus};

pub struct PgBankReconciliationRepository {
    pool: PgPool,
}

impl PgBankReconciliationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BankReconciliationRepository for PgBankReconciliationRepository {
    async fn save(&self, r: &BankReconciliation) -> Result<(), CashManagementError> {
        sqlx::query(
            r#"
            INSERT INTO bank_reconciliations (
                id, bank_account_id, period_start, period_end,
                opening_balance, closing_book_balance, statement_balance,
                status, completed_at, completed_by, notes, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(r.id().into_uuid())
        .bind(r.bank_account_id().into_uuid())
        .bind(r.period_start())
        .bind(r.period_end())
        .bind(r.opening_balance())
        .bind(r.closing_book_balance())
        .bind(r.statement_balance())
        .bind(r.status().to_string())
        .bind(r.completed_at())
        .bind(r.completed_by())
        .bind(r.notes())
        .bind(r.created_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, r: &BankReconciliation) -> Result<(), CashManagementError> {
        let result = sqlx::query(
            r#"
            UPDATE bank_reconciliations
            SET closing_book_balance = $2,
                statement_balance    = $3,
                status               = $4,
                completed_at         = $5,
                completed_by         = $6,
                notes                = $7
            WHERE id = $1
            "#,
        )
        .bind(r.id().into_uuid())
        .bind(r.closing_book_balance())
        .bind(r.statement_balance())
        .bind(r.status().to_string())
        .bind(r.completed_at())
        .bind(r.completed_by())
        .bind(r.notes())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(CashManagementError::ReconciliationNotFound(
                r.id().into_uuid(),
            ));
        }
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: BankReconciliationId,
    ) -> Result<Option<BankReconciliation>, CashManagementError> {
        let row = sqlx::query_as::<_, ReconRow>(SELECT_BY_ID)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        row.map(BankReconciliation::try_from).transpose()
    }

    async fn list_by_account(
        &self,
        bank_account_id: BankAccountId,
    ) -> Result<Vec<BankReconciliation>, CashManagementError> {
        let rows = sqlx::query_as::<_, ReconRow>(LIST_BY_ACCOUNT)
            .bind(bank_account_id.into_uuid())
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(BankReconciliation::try_from).collect()
    }
}

const SELECT_BY_ID: &str = r#"
SELECT id, bank_account_id, period_start, period_end,
       opening_balance, closing_book_balance, statement_balance,
       status, completed_at, completed_by, notes, created_at
FROM bank_reconciliations
WHERE id = $1
"#;

const LIST_BY_ACCOUNT: &str = r#"
SELECT id, bank_account_id, period_start, period_end,
       opening_balance, closing_book_balance, statement_balance,
       status, completed_at, completed_by, notes, created_at
FROM bank_reconciliations
WHERE bank_account_id = $1
ORDER BY period_start DESC
"#;

#[derive(sqlx::FromRow)]
struct ReconRow {
    id: Uuid,
    bank_account_id: Uuid,
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
    opening_balance: Decimal,
    closing_book_balance: Option<Decimal>,
    statement_balance: Option<Decimal>,
    status: String,
    completed_at: Option<DateTime<Utc>>,
    completed_by: Option<Uuid>,
    notes: Option<String>,
    created_at: DateTime<Utc>,
}

impl TryFrom<ReconRow> for BankReconciliation {
    type Error = CashManagementError;

    fn try_from(row: ReconRow) -> Result<Self, Self::Error> {
        let s = BankReconciliationStatus::from_str(&row.status)?;
        Ok(BankReconciliation::reconstitute(
            BankReconciliationId::from_uuid(row.id),
            BankAccountId::from_uuid(row.bank_account_id),
            row.period_start,
            row.period_end,
            row.opening_balance,
            row.closing_book_balance,
            row.statement_balance,
            s,
            row.completed_at,
            row.completed_by,
            row.notes,
            row.created_at,
        ))
    }
}

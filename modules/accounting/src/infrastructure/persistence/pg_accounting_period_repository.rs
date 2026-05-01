use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::AccountingError;
use crate::domain::entities::AccountingPeriod;
use crate::domain::repositories::AccountingPeriodRepository;
use crate::domain::value_objects::{AccountingPeriodId, PeriodStatus};

pub struct PgAccountingPeriodRepository {
    pool: PgPool,
}

impl PgAccountingPeriodRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AccountingPeriodRepository for PgAccountingPeriodRepository {
    async fn save(&self, p: &AccountingPeriod) -> Result<(), AccountingError> {
        sqlx::query(
            r#"
            INSERT INTO accounting_periods (
                id, name, fiscal_year, starts_at, ends_at,
                status, closed_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(p.id().into_uuid())
        .bind(p.name())
        .bind(p.fiscal_year())
        .bind(p.starts_at())
        .bind(p.ends_at())
        .bind(p.status().to_string())
        .bind(p.closed_at())
        .bind(p.created_at())
        .bind(p.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, p: &AccountingPeriod) -> Result<(), AccountingError> {
        let result = sqlx::query(
            r#"
            UPDATE accounting_periods
            SET name = $2,
                status = $3,
                closed_at = $4,
                updated_at = $5
            WHERE id = $1
            "#,
        )
        .bind(p.id().into_uuid())
        .bind(p.name())
        .bind(p.status().to_string())
        .bind(p.closed_at())
        .bind(p.updated_at())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AccountingError::PeriodNotFound(p.id().into_uuid()));
        }
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: AccountingPeriodId,
    ) -> Result<Option<AccountingPeriod>, AccountingError> {
        let row = sqlx::query_as::<_, PeriodRow>(SELECT_BY_ID)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        row.map(AccountingPeriod::try_from).transpose()
    }

    async fn find_containing(
        &self,
        at: DateTime<Utc>,
    ) -> Result<Option<AccountingPeriod>, AccountingError> {
        let row = sqlx::query_as::<_, PeriodRow>(
            r#"
            SELECT id, name, fiscal_year, starts_at, ends_at,
                   status, closed_at, created_at, updated_at
            FROM accounting_periods
            WHERE starts_at <= $1 AND ends_at > $1
            LIMIT 1
            "#,
        )
        .bind(at)
        .fetch_optional(&self.pool)
        .await?;
        row.map(AccountingPeriod::try_from).transpose()
    }

    async fn list(&self) -> Result<Vec<AccountingPeriod>, AccountingError> {
        let rows = sqlx::query_as::<_, PeriodRow>(
            r#"
            SELECT id, name, fiscal_year, starts_at, ends_at,
                   status, closed_at, created_at, updated_at
            FROM accounting_periods
            ORDER BY starts_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(AccountingPeriod::try_from).collect()
    }
}

const SELECT_BY_ID: &str = r#"
SELECT id, name, fiscal_year, starts_at, ends_at,
       status, closed_at, created_at, updated_at
FROM accounting_periods
WHERE id = $1
"#;

#[derive(sqlx::FromRow)]
struct PeriodRow {
    id: Uuid,
    name: String,
    fiscal_year: i32,
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
    status: String,
    closed_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<PeriodRow> for AccountingPeriod {
    type Error = AccountingError;

    fn try_from(row: PeriodRow) -> Result<Self, Self::Error> {
        let status: PeriodStatus = row.status.parse()?;
        Ok(AccountingPeriod::reconstitute(
            AccountingPeriodId::from_uuid(row.id),
            row.name,
            row.fiscal_year,
            row.starts_at,
            row.ends_at,
            status,
            row.closed_at,
            row.created_at,
            row.updated_at,
        ))
    }
}

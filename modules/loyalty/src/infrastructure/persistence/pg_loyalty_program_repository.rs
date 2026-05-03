use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::LoyaltyError;
use crate::domain::entities::LoyaltyProgram;
use crate::domain::repositories::LoyaltyProgramRepository;
use crate::domain::value_objects::LoyaltyProgramId;

pub struct PgLoyaltyProgramRepository {
    pool: PgPool,
}

impl PgLoyaltyProgramRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LoyaltyProgramRepository for PgLoyaltyProgramRepository {
    async fn save(&self, p: &LoyaltyProgram) -> Result<(), LoyaltyError> {
        sqlx::query(
            r#"
            INSERT INTO loyalty_programs (
                id, store_id, name, description,
                points_per_currency_unit, expiration_days,
                is_active, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(p.id().into_uuid())
        .bind(p.store_id())
        .bind(p.name())
        .bind(p.description())
        .bind(p.points_per_currency_unit())
        .bind(p.expiration_days())
        .bind(p.is_active())
        .bind(p.created_at())
        .bind(p.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, p: &LoyaltyProgram) -> Result<(), LoyaltyError> {
        let result = sqlx::query(
            r#"
            UPDATE loyalty_programs
            SET name                     = $2,
                description              = $3,
                points_per_currency_unit = $4,
                expiration_days          = $5,
                is_active                = $6,
                updated_at               = $7
            WHERE id = $1
            "#,
        )
        .bind(p.id().into_uuid())
        .bind(p.name())
        .bind(p.description())
        .bind(p.points_per_currency_unit())
        .bind(p.expiration_days())
        .bind(p.is_active())
        .bind(p.updated_at())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(LoyaltyError::ProgramNotFound(p.id().into_uuid()));
        }
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: LoyaltyProgramId,
    ) -> Result<Option<LoyaltyProgram>, LoyaltyError> {
        let row = sqlx::query_as::<_, ProgramRow>(SELECT_BY_ID)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(LoyaltyProgram::from))
    }

    async fn list(&self, store_id: Option<Uuid>) -> Result<Vec<LoyaltyProgram>, LoyaltyError> {
        let rows = match store_id {
            Some(s) => {
                sqlx::query_as::<_, ProgramRow>(LIST_BY_STORE)
                    .bind(s)
                    .fetch_all(&self.pool)
                    .await?
            }
            None => {
                sqlx::query_as::<_, ProgramRow>(LIST_ALL)
                    .fetch_all(&self.pool)
                    .await?
            }
        };
        Ok(rows.into_iter().map(LoyaltyProgram::from).collect())
    }
}

const SELECT_BY_ID: &str = r#"
SELECT id, store_id, name, description, points_per_currency_unit,
       expiration_days, is_active, created_at, updated_at
FROM loyalty_programs
WHERE id = $1
"#;

const LIST_ALL: &str = r#"
SELECT id, store_id, name, description, points_per_currency_unit,
       expiration_days, is_active, created_at, updated_at
FROM loyalty_programs
ORDER BY created_at DESC
"#;

const LIST_BY_STORE: &str = r#"
SELECT id, store_id, name, description, points_per_currency_unit,
       expiration_days, is_active, created_at, updated_at
FROM loyalty_programs
WHERE store_id = $1
ORDER BY created_at DESC
"#;

#[derive(sqlx::FromRow)]
struct ProgramRow {
    id: Uuid,
    store_id: Uuid,
    name: String,
    description: Option<String>,
    points_per_currency_unit: Decimal,
    expiration_days: Option<i32>,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<ProgramRow> for LoyaltyProgram {
    fn from(row: ProgramRow) -> Self {
        LoyaltyProgram::reconstitute(
            LoyaltyProgramId::from_uuid(row.id),
            row.store_id,
            row.name,
            row.description,
            row.points_per_currency_unit,
            row.expiration_days,
            row.is_active,
            row.created_at,
            row.updated_at,
        )
    }
}

//! PostgreSQL FiscalSequenceRepository implementation

use async_trait::async_trait;
use sqlx::PgPool;

use crate::FiscalError;
use crate::domain::entities::FiscalSequence;
use crate::domain::repositories::FiscalSequenceRepository;
use crate::domain::value_objects::FiscalSequenceId;
use identity::StoreId;
use pos_core::TerminalId;

/// PostgreSQL implementation of FiscalSequenceRepository
pub struct PgFiscalSequenceRepository {
    pool: PgPool,
}

impl PgFiscalSequenceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FiscalSequenceRepository for PgFiscalSequenceRepository {
    async fn save(&self, seq: &FiscalSequence) -> Result<(), FiscalError> {
        sqlx::query(
            r#"
            INSERT INTO fiscal_sequences (
                id, store_id, terminal_id, cai_range_id, prefix, current_number,
                range_start, range_end, is_active, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(seq.id().into_uuid())
        .bind(seq.store_id().into_uuid())
        .bind(seq.terminal_id().into_uuid())
        .bind(seq.cai_range_id())
        .bind(seq.prefix())
        .bind(seq.current_number())
        .bind(seq.range_start())
        .bind(seq.range_end())
        .bind(seq.is_active())
        .bind(seq.created_at())
        .bind(seq.updated_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_active(
        &self,
        store_id: StoreId,
        terminal_id: TerminalId,
    ) -> Result<Option<FiscalSequence>, FiscalError> {
        let row = sqlx::query_as::<_, FiscalSequenceRow>(
            r#"
            SELECT id, store_id, terminal_id, cai_range_id, prefix, current_number,
                   range_start, range_end, is_active, created_at, updated_at
            FROM fiscal_sequences
            WHERE store_id = $1 AND terminal_id = $2 AND is_active = true
            "#,
        )
        .bind(store_id.into_uuid())
        .bind(terminal_id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(r.try_into()?)),
            None => Ok(None),
        }
    }

    async fn increment_and_get(&self, id: FiscalSequenceId) -> Result<String, FiscalError> {
        let row = sqlx::query_as::<_, IncrementResultRow>(
            r#"
            UPDATE fiscal_sequences
            SET current_number = current_number + 1,
                updated_at = NOW()
            WHERE id = $1
              AND is_active = true
              AND current_number < range_end
            RETURNING prefix, current_number
            "#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(format!("{}{:08}", r.prefix, r.current_number)),
            None => Err(FiscalError::FiscalSequenceExhausted(id.into_uuid())),
        }
    }

    async fn update(&self, seq: &FiscalSequence) -> Result<(), FiscalError> {
        let result = sqlx::query(
            r#"
            UPDATE fiscal_sequences
            SET current_number = $2, is_active = $3, updated_at = $4
            WHERE id = $1
            "#,
        )
        .bind(seq.id().into_uuid())
        .bind(seq.current_number())
        .bind(seq.is_active())
        .bind(seq.updated_at())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(FiscalError::FiscalSequenceNotFound);
        }

        Ok(())
    }
}

// =============================================================================
// Row types
// =============================================================================

#[derive(sqlx::FromRow)]
struct FiscalSequenceRow {
    id: uuid::Uuid,
    store_id: uuid::Uuid,
    terminal_id: uuid::Uuid,
    cai_range_id: uuid::Uuid,
    prefix: String,
    current_number: i64,
    range_start: i64,
    range_end: i64,
    is_active: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<FiscalSequenceRow> for FiscalSequence {
    type Error = FiscalError;

    fn try_from(row: FiscalSequenceRow) -> Result<Self, Self::Error> {
        Ok(FiscalSequence::reconstitute(
            FiscalSequenceId::from_uuid(row.id),
            StoreId::from_uuid(row.store_id),
            TerminalId::from_uuid(row.terminal_id),
            row.cai_range_id,
            row.prefix,
            row.current_number,
            row.range_start,
            row.range_end,
            row.is_active,
            row.created_at,
            row.updated_at,
        ))
    }
}

#[derive(sqlx::FromRow)]
struct IncrementResultRow {
    prefix: String,
    current_number: i64,
}

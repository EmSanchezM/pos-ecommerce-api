// PgTerminalRepository - PostgreSQL implementation of TerminalRepository
// Requirements: 5.4

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use identity::StoreId;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entities::{CaiRange, Terminal};
use crate::domain::repositories::TerminalRepository;
use crate::domain::value_objects::{CaiNumber, TerminalCode, TerminalId};
use crate::error::CoreError;

/// PostgreSQL implementation of TerminalRepository
pub struct PgTerminalRepository {
    pool: Arc<PgPool>,
}

impl PgTerminalRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

/// Internal row type for mapping terminal database results
#[derive(sqlx::FromRow)]
struct TerminalRow {
    id: Uuid,
    store_id: Uuid,
    code: String,
    name: String,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// Internal row type for mapping CAI range database results
#[derive(sqlx::FromRow)]
struct CaiRangeRow {
    id: Uuid,
    cai_number: String,
    range_start: i64,
    range_end: i64,
    current_number: i64,
    expiration_date: NaiveDate,
    is_exhausted: bool,
    created_at: DateTime<Utc>,
}

impl CaiRangeRow {
    fn try_into_cai_range(self) -> Result<CaiRange, CoreError> {
        let cai_number = CaiNumber::new(&self.cai_number)?;
        Ok(CaiRange::new(
            self.id,
            cai_number,
            self.range_start,
            self.range_end,
            self.current_number,
            self.expiration_date,
            self.is_exhausted,
            self.created_at,
        ))
    }
}

#[async_trait]
impl TerminalRepository for PgTerminalRepository {
    async fn save(&self, terminal: &Terminal) -> Result<(), CoreError> {
        sqlx::query(
            r#"
            INSERT INTO terminals (id, store_id, code, name, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(terminal.id().into_uuid())
        .bind(terminal.store_id().into_uuid())
        .bind(terminal.code().as_str())
        .bind(terminal.name())
        .bind(terminal.is_active())
        .bind(terminal.created_at())
        .bind(terminal.updated_at())
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    async fn find_by_id(&self, id: TerminalId) -> Result<Option<Terminal>, CoreError> {
        let row = sqlx::query_as::<_, TerminalRow>(
            r#"
            SELECT id, store_id, code, name, is_active, created_at, updated_at
            FROM terminals
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .fetch_optional(self.pool.as_ref())
        .await?;

        match row {
            Some(terminal_row) => {
                // Fetch the current (most recent non-exhausted, non-expired) CAI range
                let cai = self.get_current_cai(id).await?;
                let terminal_code = TerminalCode::new(&terminal_row.code)?;
                
                Ok(Some(Terminal::reconstitute(
                    TerminalId::from_uuid(terminal_row.id),
                    StoreId::from_uuid(terminal_row.store_id),
                    terminal_code,
                    terminal_row.name,
                    terminal_row.is_active,
                    cai,
                    terminal_row.created_at,
                    terminal_row.updated_at,
                )))
            }
            None => Ok(None),
        }
    }

    async fn find_by_store(&self, store_id: StoreId) -> Result<Vec<Terminal>, CoreError> {
        let rows = sqlx::query_as::<_, TerminalRow>(
            r#"
            SELECT id, store_id, code, name, is_active, created_at, updated_at
            FROM terminals
            WHERE store_id = $1
            ORDER BY code
            "#,
        )
        .bind(store_id.into_uuid())
        .fetch_all(self.pool.as_ref())
        .await?;

        let mut terminals = Vec::with_capacity(rows.len());
        for row in rows {
            let terminal_id = TerminalId::from_uuid(row.id);
            let cai = self.get_current_cai(terminal_id).await?;
            let terminal_code = TerminalCode::new(&row.code)?;
            
            terminals.push(Terminal::reconstitute(
                terminal_id,
                StoreId::from_uuid(row.store_id),
                terminal_code,
                row.name,
                row.is_active,
                cai,
                row.created_at,
                row.updated_at,
            ));
        }

        Ok(terminals)
    }

    async fn find_by_code(
        &self,
        store_id: StoreId,
        code: &TerminalCode,
    ) -> Result<Option<Terminal>, CoreError> {
        let row = sqlx::query_as::<_, TerminalRow>(
            r#"
            SELECT id, store_id, code, name, is_active, created_at, updated_at
            FROM terminals
            WHERE store_id = $1 AND code = $2
            "#,
        )
        .bind(store_id.into_uuid())
        .bind(code.as_str())
        .fetch_optional(self.pool.as_ref())
        .await?;

        match row {
            Some(terminal_row) => {
                let terminal_id = TerminalId::from_uuid(terminal_row.id);
                let cai = self.get_current_cai(terminal_id).await?;
                let terminal_code = TerminalCode::new(&terminal_row.code)?;
                
                Ok(Some(Terminal::reconstitute(
                    terminal_id,
                    StoreId::from_uuid(terminal_row.store_id),
                    terminal_code,
                    terminal_row.name,
                    terminal_row.is_active,
                    cai,
                    terminal_row.created_at,
                    terminal_row.updated_at,
                )))
            }
            None => Ok(None),
        }
    }

    async fn update(&self, terminal: &Terminal) -> Result<(), CoreError> {
        let result = sqlx::query(
            r#"
            UPDATE terminals
            SET name = $2, is_active = $3, updated_at = $4
            WHERE id = $1
            "#,
        )
        .bind(terminal.id().into_uuid())
        .bind(terminal.name())
        .bind(terminal.is_active())
        .bind(terminal.updated_at())
        .execute(self.pool.as_ref())
        .await?;

        if result.rows_affected() == 0 {
            return Err(CoreError::TerminalNotFound(terminal.id().into_uuid()));
        }

        Ok(())
    }

    async fn save_cai_range(
        &self,
        terminal_id: TerminalId,
        cai: &CaiRange,
    ) -> Result<(), CoreError> {
        sqlx::query(
            r#"
            INSERT INTO cai_ranges (id, terminal_id, cai_number, range_start, range_end, current_number, expiration_date, is_exhausted, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(cai.id())
        .bind(terminal_id.into_uuid())
        .bind(cai.cai_number().as_str())
        .bind(cai.range_start())
        .bind(cai.range_end())
        .bind(cai.current_number())
        .bind(cai.expiration_date())
        .bind(cai.is_exhausted_flag())
        .bind(cai.created_at())
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    async fn get_cai_history(&self, terminal_id: TerminalId) -> Result<Vec<CaiRange>, CoreError> {
        let rows = sqlx::query_as::<_, CaiRangeRow>(
            r#"
            SELECT id, cai_number, range_start, range_end, current_number, expiration_date, is_exhausted, created_at
            FROM cai_ranges
            WHERE terminal_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(terminal_id.into_uuid())
        .fetch_all(self.pool.as_ref())
        .await?;

        rows.into_iter()
            .map(|row| row.try_into_cai_range())
            .collect()
    }

    async fn increment_and_get_invoice_number(
        &self,
        terminal_id: TerminalId,
    ) -> Result<i64, CoreError> {
        // Use a transaction to ensure atomicity
        let mut tx = self.pool.begin().await?;

        // Get the current active CAI range with FOR UPDATE lock
        let cai_row = sqlx::query_as::<_, CaiRangeRow>(
            r#"
            SELECT id, cai_number, range_start, range_end, current_number, expiration_date, is_exhausted, created_at
            FROM cai_ranges
            WHERE terminal_id = $1
              AND is_exhausted = FALSE
              AND expiration_date >= CURRENT_DATE
            ORDER BY created_at DESC
            LIMIT 1
            FOR UPDATE
            "#,
        )
        .bind(terminal_id.into_uuid())
        .fetch_optional(&mut *tx)
        .await?;

        let cai_row = match cai_row {
            Some(row) => row,
            None => {
                // Check if there's any CAI at all
                let any_cai = sqlx::query_scalar::<_, i64>(
                    r#"SELECT COUNT(*) FROM cai_ranges WHERE terminal_id = $1"#,
                )
                .bind(terminal_id.into_uuid())
                .fetch_one(&mut *tx)
                .await?;

                if any_cai == 0 {
                    return Err(CoreError::NoCaiAssigned(terminal_id.into_uuid()));
                }

                // Check if expired
                let expired = sqlx::query_scalar::<_, i64>(
                    r#"
                    SELECT COUNT(*) FROM cai_ranges 
                    WHERE terminal_id = $1 
                      AND is_exhausted = FALSE 
                      AND expiration_date < CURRENT_DATE
                    "#,
                )
                .bind(terminal_id.into_uuid())
                .fetch_one(&mut *tx)
                .await?;

                if expired > 0 {
                    return Err(CoreError::CaiExpired(terminal_id.into_uuid()));
                }

                return Err(CoreError::CaiRangeExhausted(terminal_id.into_uuid()));
            }
        };

        // Check if range is exhausted
        if cai_row.current_number > cai_row.range_end {
            // Mark as exhausted
            sqlx::query(
                r#"UPDATE cai_ranges SET is_exhausted = TRUE WHERE id = $1"#,
            )
            .bind(cai_row.id)
            .execute(&mut *tx)
            .await?;

            tx.commit().await?;
            return Err(CoreError::CaiRangeExhausted(terminal_id.into_uuid()));
        }

        // Get the current number and increment
        let invoice_number = cai_row.current_number;
        let new_current = invoice_number + 1;
        let is_exhausted = new_current > cai_row.range_end;

        // Update the current number
        sqlx::query(
            r#"
            UPDATE cai_ranges 
            SET current_number = $2, is_exhausted = $3
            WHERE id = $1
            "#,
        )
        .bind(cai_row.id)
        .bind(new_current)
        .bind(is_exhausted)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(invoice_number)
    }

    async fn count_active_by_store(&self, store_id: StoreId) -> Result<i64, CoreError> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM terminals
            WHERE store_id = $1 AND is_active = TRUE
            "#,
        )
        .bind(store_id.into_uuid())
        .fetch_one(self.pool.as_ref())
        .await?;

        Ok(count)
    }

    async fn deactivate_by_store(&self, store_id: StoreId) -> Result<(), CoreError> {
        sqlx::query(
            r#"
            UPDATE terminals
            SET is_active = FALSE, updated_at = NOW()
            WHERE store_id = $1
            "#,
        )
        .bind(store_id.into_uuid())
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }
}

impl PgTerminalRepository {
    /// Helper method to get the current active CAI range for a terminal
    async fn get_current_cai(&self, terminal_id: TerminalId) -> Result<Option<CaiRange>, CoreError> {
        let row = sqlx::query_as::<_, CaiRangeRow>(
            r#"
            SELECT id, cai_number, range_start, range_end, current_number, expiration_date, is_exhausted, created_at
            FROM cai_ranges
            WHERE terminal_id = $1
              AND is_exhausted = FALSE
              AND expiration_date >= CURRENT_DATE
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(terminal_id.into_uuid())
        .fetch_optional(self.pool.as_ref())
        .await?;

        match row {
            Some(cai_row) => Ok(Some(cai_row.try_into_cai_range()?)),
            None => Ok(None),
        }
    }
}

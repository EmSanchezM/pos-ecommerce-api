//! PostgreSQL implementation of JournalEntryRepository.
//!
//! `save` writes the header and all lines inside one transaction so a partial
//! entry never appears in the database. `update_status` only mutates the
//! header — lines are immutable to preserve the audit trail.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::AccountingError;
use crate::domain::entities::{JournalEntry, JournalLine};
use crate::domain::repositories::JournalEntryRepository;
use crate::domain::value_objects::{
    AccountId, AccountingPeriodId, JournalEntryId, JournalEntryStatus, JournalLineId,
};

pub struct PgJournalEntryRepository {
    pool: PgPool,
}

impl PgJournalEntryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JournalEntryRepository for PgJournalEntryRepository {
    async fn save(&self, entry: &JournalEntry) -> Result<(), AccountingError> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"
            INSERT INTO journal_entries (
                id, period_id, entry_number, description,
                source_type, source_id, status, posted_at,
                created_by, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(entry.id().into_uuid())
        .bind(entry.period_id().into_uuid())
        .bind(entry.entry_number())
        .bind(entry.description())
        .bind(entry.source_type())
        .bind(entry.source_id())
        .bind(entry.status().to_string())
        .bind(entry.posted_at())
        .bind(entry.created_by())
        .bind(entry.created_at())
        .execute(&mut *tx)
        .await?;

        for line in entry.lines() {
            sqlx::query(
                r#"
                INSERT INTO journal_lines (
                    id, journal_entry_id, account_id, store_id,
                    line_number, debit, credit, description
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
            )
            .bind(line.id().into_uuid())
            .bind(entry.id().into_uuid())
            .bind(line.account_id().into_uuid())
            .bind(line.store_id())
            .bind(line.line_number())
            .bind(line.debit())
            .bind(line.credit())
            .bind(line.description())
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn update_status(&self, entry: &JournalEntry) -> Result<(), AccountingError> {
        let result = sqlx::query(
            r#"
            UPDATE journal_entries
            SET status = $2,
                posted_at = $3
            WHERE id = $1
            "#,
        )
        .bind(entry.id().into_uuid())
        .bind(entry.status().to_string())
        .bind(entry.posted_at())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AccountingError::JournalEntryNotFound(
                entry.id().into_uuid(),
            ));
        }
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: JournalEntryId,
    ) -> Result<Option<JournalEntry>, AccountingError> {
        let header = sqlx::query_as::<_, EntryRow>(SELECT_HEADER_BY_ID)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;

        let Some(header) = header else {
            return Ok(None);
        };

        let lines = sqlx::query_as::<_, LineRow>(SELECT_LINES_BY_ENTRY)
            .bind(id.into_uuid())
            .fetch_all(&self.pool)
            .await?;

        Ok(Some(reconstitute(header, lines)?))
    }

    async fn list_by_period(
        &self,
        period_id: AccountingPeriodId,
    ) -> Result<Vec<JournalEntry>, AccountingError> {
        let headers = sqlx::query_as::<_, EntryRow>(
            r#"
            SELECT id, period_id, entry_number, description,
                   source_type, source_id, status, posted_at,
                   created_by, created_at
            FROM journal_entries
            WHERE period_id = $1
            ORDER BY entry_number ASC
            "#,
        )
        .bind(period_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;

        let mut entries = Vec::with_capacity(headers.len());
        for header in headers {
            let header_id = header.id;
            let lines = sqlx::query_as::<_, LineRow>(SELECT_LINES_BY_ENTRY)
                .bind(header_id)
                .fetch_all(&self.pool)
                .await?;
            entries.push(reconstitute(header, lines)?);
        }
        Ok(entries)
    }

    async fn next_entry_number(
        &self,
        period_id: AccountingPeriodId,
    ) -> Result<i64, AccountingError> {
        let max: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT MAX(entry_number)::BIGINT
            FROM journal_entries
            WHERE period_id = $1
            "#,
        )
        .bind(period_id.into_uuid())
        .fetch_one(&self.pool)
        .await?;
        Ok(max.unwrap_or(0) + 1)
    }
}

const SELECT_HEADER_BY_ID: &str = r#"
SELECT id, period_id, entry_number, description,
       source_type, source_id, status, posted_at,
       created_by, created_at
FROM journal_entries
WHERE id = $1
"#;

const SELECT_LINES_BY_ENTRY: &str = r#"
SELECT id, journal_entry_id, account_id, store_id,
       line_number, debit, credit, description
FROM journal_lines
WHERE journal_entry_id = $1
ORDER BY line_number ASC
"#;

#[derive(sqlx::FromRow)]
struct EntryRow {
    id: Uuid,
    period_id: Uuid,
    entry_number: i64,
    description: String,
    source_type: Option<String>,
    source_id: Option<Uuid>,
    status: String,
    posted_at: Option<DateTime<Utc>>,
    created_by: Option<Uuid>,
    created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct LineRow {
    id: Uuid,
    #[allow(dead_code)]
    journal_entry_id: Uuid,
    account_id: Uuid,
    store_id: Option<Uuid>,
    line_number: i32,
    debit: Decimal,
    credit: Decimal,
    description: Option<String>,
}

fn reconstitute(
    header: EntryRow,
    line_rows: Vec<LineRow>,
) -> Result<JournalEntry, AccountingError> {
    let status: JournalEntryStatus = header.status.parse()?;
    let lines = line_rows
        .into_iter()
        .map(|r| {
            JournalLine::reconstitute(
                JournalLineId::from_uuid(r.id),
                AccountId::from_uuid(r.account_id),
                r.store_id,
                r.line_number,
                r.debit,
                r.credit,
                r.description,
            )
        })
        .collect();

    Ok(JournalEntry::reconstitute(
        JournalEntryId::from_uuid(header.id),
        AccountingPeriodId::from_uuid(header.period_id),
        header.entry_number,
        header.description,
        header.source_type,
        header.source_id,
        status,
        header.posted_at,
        header.created_by,
        lines,
        header.created_at,
    ))
}

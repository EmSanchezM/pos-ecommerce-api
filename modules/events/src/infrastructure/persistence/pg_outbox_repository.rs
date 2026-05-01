//! PostgreSQL implementation of OutboxRepository.

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};

use crate::EventsError;
use crate::domain::entities::OutboxEvent;
use crate::domain::repositories::OutboxRepository;
use crate::domain::value_objects::{EventStatus, OutboxEventId};

pub struct PgOutboxRepository {
    pool: PgPool,
}

impl PgOutboxRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OutboxRepository for PgOutboxRepository {
    async fn enqueue_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        event: &OutboxEvent,
    ) -> Result<(), EventsError> {
        sqlx::query(
            r#"
            INSERT INTO outbox_events (
                id, aggregate_type, aggregate_id, event_type, payload,
                status, attempts, last_error,
                occurred_at, processed_at, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(event.id().into_uuid())
        .bind(event.aggregate_type())
        .bind(event.aggregate_id())
        .bind(event.event_type())
        .bind(event.payload())
        .bind(event.status().to_string())
        .bind(event.attempts())
        .bind(event.last_error())
        .bind(event.occurred_at())
        .bind(event.processed_at())
        .bind(event.created_at())
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    async fn fetch_pending(&self, batch_size: i64) -> Result<Vec<OutboxEvent>, EventsError> {
        let rows = sqlx::query_as::<_, OutboxEventRow>(
            r#"
            SELECT id, aggregate_type, aggregate_id, event_type, payload,
                   status, attempts, last_error,
                   occurred_at, processed_at, created_at
            FROM outbox_events
            WHERE status = 'pending'
            ORDER BY occurred_at ASC
            LIMIT $1
            "#,
        )
        .bind(batch_size)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(OutboxEvent::try_from).collect()
    }

    async fn update_after_dispatch(&self, event: &OutboxEvent) -> Result<(), EventsError> {
        let result = sqlx::query(
            r#"
            UPDATE outbox_events
            SET status = $2,
                attempts = $3,
                last_error = $4,
                processed_at = $5
            WHERE id = $1
            "#,
        )
        .bind(event.id().into_uuid())
        .bind(event.status().to_string())
        .bind(event.attempts())
        .bind(event.last_error())
        .bind(event.processed_at())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(EventsError::EventNotFound(event.id().into_uuid()));
        }
        Ok(())
    }

    async fn find_by_id(&self, id: OutboxEventId) -> Result<Option<OutboxEvent>, EventsError> {
        let row = sqlx::query_as::<_, OutboxEventRow>(
            r#"
            SELECT id, aggregate_type, aggregate_id, event_type, payload,
                   status, attempts, last_error,
                   occurred_at, processed_at, created_at
            FROM outbox_events
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        row.map(OutboxEvent::try_from).transpose()
    }
}

#[derive(sqlx::FromRow)]
struct OutboxEventRow {
    id: uuid::Uuid,
    aggregate_type: String,
    aggregate_id: String,
    event_type: String,
    payload: serde_json::Value,
    status: String,
    attempts: i32,
    last_error: Option<String>,
    occurred_at: chrono::DateTime<chrono::Utc>,
    processed_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<OutboxEventRow> for OutboxEvent {
    type Error = EventsError;

    fn try_from(row: OutboxEventRow) -> Result<Self, Self::Error> {
        let status: EventStatus = row.status.parse()?;
        Ok(OutboxEvent::reconstitute(
            OutboxEventId::from_uuid(row.id),
            row.aggregate_type,
            row.aggregate_id,
            row.event_type,
            row.payload,
            status,
            row.attempts,
            row.last_error,
            row.occurred_at,
            row.processed_at,
            row.created_at,
        ))
    }
}

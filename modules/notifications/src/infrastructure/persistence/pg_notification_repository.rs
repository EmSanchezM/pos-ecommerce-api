//! PostgreSQL implementation of NotificationRepository.

use async_trait::async_trait;
use sqlx::PgPool;

use crate::NotificationsError;
use crate::domain::entities::Notification;
use crate::domain::repositories::NotificationRepository;
use crate::domain::value_objects::{NotificationChannel, NotificationId, NotificationStatus};

pub struct PgNotificationRepository {
    pool: PgPool,
}

impl PgNotificationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl NotificationRepository for PgNotificationRepository {
    async fn save(&self, n: &Notification) -> Result<(), NotificationsError> {
        sqlx::query(
            r#"
            INSERT INTO notifications (
                id, channel, recipient, subject, body, metadata,
                status, attempts, last_error, sent_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(n.id().into_uuid())
        .bind(n.channel().to_string())
        .bind(n.recipient())
        .bind(n.subject())
        .bind(n.body())
        .bind(n.metadata())
        .bind(n.status().to_string())
        .bind(n.attempts())
        .bind(n.last_error())
        .bind(n.sent_at())
        .bind(n.created_at())
        .bind(n.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, n: &Notification) -> Result<(), NotificationsError> {
        let result = sqlx::query(
            r#"
            UPDATE notifications
            SET status = $2,
                attempts = $3,
                last_error = $4,
                sent_at = $5,
                updated_at = $6
            WHERE id = $1
            "#,
        )
        .bind(n.id().into_uuid())
        .bind(n.status().to_string())
        .bind(n.attempts())
        .bind(n.last_error())
        .bind(n.sent_at())
        .bind(n.updated_at())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(NotificationsError::NotFound(n.id().into_uuid()));
        }
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: NotificationId,
    ) -> Result<Option<Notification>, NotificationsError> {
        let row = sqlx::query_as::<_, NotificationRow>(SELECT_BY_ID)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        row.map(Notification::try_from).transpose()
    }

    async fn find_retryable(
        &self,
        max_attempts: i32,
        limit: i64,
    ) -> Result<Vec<Notification>, NotificationsError> {
        let rows = sqlx::query_as::<_, NotificationRow>(
            r#"
            SELECT id, channel, recipient, subject, body, metadata,
                   status, attempts, last_error, sent_at, created_at, updated_at
            FROM notifications
            WHERE status = 'failed' AND attempts < $1
            ORDER BY updated_at ASC
            LIMIT $2
            "#,
        )
        .bind(max_attempts)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(Notification::try_from).collect()
    }
}

const SELECT_BY_ID: &str = r#"
SELECT id, channel, recipient, subject, body, metadata,
       status, attempts, last_error, sent_at, created_at, updated_at
FROM notifications
WHERE id = $1
"#;

#[derive(sqlx::FromRow)]
struct NotificationRow {
    id: uuid::Uuid,
    channel: String,
    recipient: String,
    subject: Option<String>,
    body: String,
    metadata: serde_json::Value,
    status: String,
    attempts: i32,
    last_error: Option<String>,
    sent_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<NotificationRow> for Notification {
    type Error = NotificationsError;

    fn try_from(row: NotificationRow) -> Result<Self, Self::Error> {
        let channel: NotificationChannel = row.channel.parse()?;
        let status: NotificationStatus = row.status.parse()?;
        Ok(Notification::reconstitute(
            NotificationId::from_uuid(row.id),
            channel,
            row.recipient,
            row.subject,
            row.body,
            row.metadata,
            status,
            row.attempts,
            row.last_error,
            row.sent_at,
            row.created_at,
            row.updated_at,
        ))
    }
}

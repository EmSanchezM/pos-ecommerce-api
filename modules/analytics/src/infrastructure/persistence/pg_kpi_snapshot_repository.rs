//! PostgreSQL implementation of KpiSnapshotRepository.
//!
//! `upsert` deletes the previous (kpi_key, store_id, time_window) row if any
//! and inserts the new one inside a single transaction. `IS NOT DISTINCT FROM`
//! matches NULL store_id rows correctly without needing two code paths.

use async_trait::async_trait;
use chrono::DateTime;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::AnalyticsError;
use crate::domain::entities::KpiSnapshot;
use crate::domain::repositories::KpiSnapshotRepository;
use crate::domain::value_objects::{KpiKey, KpiSnapshotId, TimeWindow};

pub struct PgKpiSnapshotRepository {
    pool: PgPool,
}

impl PgKpiSnapshotRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl KpiSnapshotRepository for PgKpiSnapshotRepository {
    async fn upsert(&self, snapshot: &KpiSnapshot) -> Result<(), AnalyticsError> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"
            DELETE FROM kpi_snapshots
            WHERE kpi_key = $1
              AND time_window = $2
              AND store_id IS NOT DISTINCT FROM $3
            "#,
        )
        .bind(snapshot.kpi_key().as_str())
        .bind(snapshot.time_window().to_string())
        .bind(snapshot.store_id())
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO kpi_snapshots (
                id, kpi_key, store_id, time_window, value, metadata, computed_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(snapshot.id().into_uuid())
        .bind(snapshot.kpi_key().as_str())
        .bind(snapshot.store_id())
        .bind(snapshot.time_window().to_string())
        .bind(snapshot.value())
        .bind(snapshot.metadata())
        .bind(snapshot.computed_at())
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    async fn find_latest(
        &self,
        kpi_key: &KpiKey,
        store_id: Option<Uuid>,
        time_window: TimeWindow,
    ) -> Result<Option<KpiSnapshot>, AnalyticsError> {
        let row = sqlx::query_as::<_, KpiSnapshotRow>(
            r#"
            SELECT id, kpi_key, store_id, time_window, value, metadata, computed_at
            FROM kpi_snapshots
            WHERE kpi_key = $1
              AND time_window = $2
              AND store_id IS NOT DISTINCT FROM $3
            ORDER BY computed_at DESC
            LIMIT 1
            "#,
        )
        .bind(kpi_key.as_str())
        .bind(time_window.to_string())
        .bind(store_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(KpiSnapshot::try_from).transpose()
    }

    async fn list_for_store(
        &self,
        store_id: Option<Uuid>,
    ) -> Result<Vec<KpiSnapshot>, AnalyticsError> {
        let rows = sqlx::query_as::<_, KpiSnapshotRow>(
            r#"
            SELECT id, kpi_key, store_id, time_window, value, metadata, computed_at
            FROM kpi_snapshots
            WHERE store_id IS NOT DISTINCT FROM $1
            ORDER BY computed_at DESC
            "#,
        )
        .bind(store_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(KpiSnapshot::try_from).collect()
    }
}

#[derive(sqlx::FromRow)]
struct KpiSnapshotRow {
    id: Uuid,
    kpi_key: String,
    store_id: Option<Uuid>,
    time_window: String,
    value: Decimal,
    metadata: serde_json::Value,
    computed_at: DateTime<chrono::Utc>,
}

impl TryFrom<KpiSnapshotRow> for KpiSnapshot {
    type Error = AnalyticsError;

    fn try_from(row: KpiSnapshotRow) -> Result<Self, Self::Error> {
        let window: TimeWindow = row.time_window.parse()?;
        Ok(KpiSnapshot::reconstitute(
            KpiSnapshotId::from_uuid(row.id),
            KpiKey::new(row.kpi_key),
            row.store_id,
            window,
            row.value,
            row.metadata,
            row.computed_at,
        ))
    }
}

use async_trait::async_trait;
use uuid::Uuid;

use crate::AnalyticsError;
use crate::domain::entities::KpiSnapshot;
use crate::domain::value_objects::{KpiKey, TimeWindow};

#[async_trait]
pub trait KpiSnapshotRepository: Send + Sync {
    /// Insert or replace the snapshot for a (kpi_key, store_id, time_window) tuple.
    /// The repository keeps only the latest value per tuple.
    async fn upsert(&self, snapshot: &KpiSnapshot) -> Result<(), AnalyticsError>;

    /// Returns the latest snapshot for the given identity, or `None`.
    async fn find_latest(
        &self,
        kpi_key: &KpiKey,
        store_id: Option<Uuid>,
        time_window: TimeWindow,
    ) -> Result<Option<KpiSnapshot>, AnalyticsError>;

    /// Returns all snapshots for a store, useful for bulk dashboard rendering.
    async fn list_for_store(
        &self,
        store_id: Option<Uuid>,
    ) -> Result<Vec<KpiSnapshot>, AnalyticsError>;
}

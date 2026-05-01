use std::sync::Arc;

use uuid::Uuid;

use crate::AnalyticsError;
use crate::domain::entities::KpiSnapshot;
use crate::domain::repositories::KpiSnapshotRepository;
use crate::domain::value_objects::{KpiKey, TimeWindow};

pub struct GetKpiSnapshotUseCase {
    repo: Arc<dyn KpiSnapshotRepository>,
}

impl GetKpiSnapshotUseCase {
    pub fn new(repo: Arc<dyn KpiSnapshotRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        kpi_key: &KpiKey,
        store_id: Option<Uuid>,
        time_window: TimeWindow,
    ) -> Result<KpiSnapshot, AnalyticsError> {
        self.repo
            .find_latest(kpi_key, store_id, time_window)
            .await?
            .ok_or_else(|| AnalyticsError::SnapshotNotFound(kpi_key.as_str().to_string()))
    }
}

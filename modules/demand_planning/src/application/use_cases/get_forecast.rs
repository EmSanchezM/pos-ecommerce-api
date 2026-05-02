use std::sync::Arc;
use uuid::Uuid;

use crate::DemandPlanningError;
use crate::domain::entities::DemandForecast;
use crate::domain::repositories::DemandForecastRepository;

pub struct GetForecastUseCase {
    repo: Arc<dyn DemandForecastRepository>,
}

impl GetForecastUseCase {
    pub fn new(repo: Arc<dyn DemandForecastRepository>) -> Self {
        Self { repo }
    }

    /// Returns every forecast on file for the (variant, store) tuple, newest
    /// first. The handler can pick the freshest one or render a small history.
    pub async fn execute(
        &self,
        product_variant_id: Uuid,
        store_id: Uuid,
    ) -> Result<Vec<DemandForecast>, DemandPlanningError> {
        self.repo
            .list_for_variant(product_variant_id, store_id)
            .await
    }
}

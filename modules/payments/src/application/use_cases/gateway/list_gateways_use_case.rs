//! List payment gateways for a store.

use std::sync::Arc;

use uuid::Uuid;

use crate::PaymentsError;
use crate::application::dtos::GatewayResponse;
use crate::domain::repositories::PaymentGatewayRepository;
use identity::StoreId;

pub struct ListGatewaysUseCase {
    gateway_repo: Arc<dyn PaymentGatewayRepository>,
}

impl ListGatewaysUseCase {
    pub fn new(gateway_repo: Arc<dyn PaymentGatewayRepository>) -> Self {
        Self { gateway_repo }
    }

    pub async fn execute(&self, store_id: Uuid) -> Result<Vec<GatewayResponse>, PaymentsError> {
        let store_id = StoreId::from_uuid(store_id);
        let gateways = self.gateway_repo.find_by_store(store_id).await?;
        Ok(gateways.into_iter().map(GatewayResponse::from).collect())
    }
}

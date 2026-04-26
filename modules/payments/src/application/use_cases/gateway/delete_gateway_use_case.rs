//! Delete a payment gateway. Super-admin only at the API layer.

use std::sync::Arc;

use uuid::Uuid;

use crate::PaymentsError;
use crate::domain::repositories::PaymentGatewayRepository;
use crate::domain::value_objects::PaymentGatewayId;

pub struct DeleteGatewayUseCase {
    gateway_repo: Arc<dyn PaymentGatewayRepository>,
}

impl DeleteGatewayUseCase {
    pub fn new(gateway_repo: Arc<dyn PaymentGatewayRepository>) -> Self {
        Self { gateway_repo }
    }

    pub async fn execute(&self, gateway_id: Uuid) -> Result<(), PaymentsError> {
        let id = PaymentGatewayId::from_uuid(gateway_id);
        // Surface a clear 404 when the gateway is unknown.
        if self.gateway_repo.find_by_id(id).await?.is_none() {
            return Err(PaymentsError::GatewayNotFound(gateway_id));
        }
        self.gateway_repo.delete(id).await
    }
}

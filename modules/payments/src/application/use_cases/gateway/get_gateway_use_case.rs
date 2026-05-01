//! Read a single payment gateway.

use std::sync::Arc;

use uuid::Uuid;

use crate::PaymentsError;
use crate::application::dtos::GatewayResponse;
use crate::domain::repositories::PaymentGatewayRepository;
use crate::domain::value_objects::PaymentGatewayId;

pub struct GetGatewayUseCase {
    gateway_repo: Arc<dyn PaymentGatewayRepository>,
}

impl GetGatewayUseCase {
    pub fn new(gateway_repo: Arc<dyn PaymentGatewayRepository>) -> Self {
        Self { gateway_repo }
    }

    pub async fn execute(&self, id: Uuid) -> Result<GatewayResponse, PaymentsError> {
        let gateway = self
            .gateway_repo
            .find_by_id(PaymentGatewayId::from_uuid(id))
            .await?
            .ok_or(PaymentsError::GatewayNotFound(id))?;
        Ok(GatewayResponse::from(gateway))
    }
}

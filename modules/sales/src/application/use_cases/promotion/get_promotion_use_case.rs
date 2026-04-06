use std::sync::Arc;

use crate::SalesError;
use crate::application::dtos::promotion::PromotionResponse;
use crate::domain::repositories::PromotionRepository;
use crate::domain::value_objects::PromotionId;

pub struct GetPromotionUseCase<P: PromotionRepository> {
    promotion_repo: Arc<P>,
}

impl<P: PromotionRepository> GetPromotionUseCase<P> {
    pub fn new(promotion_repo: Arc<P>) -> Self {
        Self { promotion_repo }
    }

    pub async fn execute(&self, id: uuid::Uuid) -> Result<PromotionResponse, SalesError> {
        let promotion = self
            .promotion_repo
            .find_by_id(PromotionId::from_uuid(id))
            .await?
            .ok_or(SalesError::PromotionNotFound(id))?;

        Ok(PromotionResponse::from(&promotion))
    }
}

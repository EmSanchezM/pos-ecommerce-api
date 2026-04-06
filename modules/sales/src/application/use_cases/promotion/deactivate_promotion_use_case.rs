use std::sync::Arc;

use crate::SalesError;
use crate::application::dtos::promotion::PromotionResponse;
use crate::domain::repositories::PromotionRepository;
use crate::domain::value_objects::PromotionId;

pub struct DeactivatePromotionUseCase<P: PromotionRepository> {
    promotion_repo: Arc<P>,
}

impl<P: PromotionRepository> DeactivatePromotionUseCase<P> {
    pub fn new(promotion_repo: Arc<P>) -> Self {
        Self { promotion_repo }
    }

    pub async fn execute(&self, id: uuid::Uuid) -> Result<PromotionResponse, SalesError> {
        let mut promotion = self
            .promotion_repo
            .find_by_id(PromotionId::from_uuid(id))
            .await?
            .ok_or(SalesError::PromotionNotFound(id))?;

        promotion.deactivate();
        self.promotion_repo.update(&promotion).await?;

        Ok(PromotionResponse::from(&promotion))
    }
}

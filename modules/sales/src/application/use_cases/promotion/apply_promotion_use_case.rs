use std::sync::Arc;

use crate::SalesError;
use crate::application::dtos::SaleDetailResponse;
use crate::domain::repositories::{PromotionRepository, SaleRepository};
use crate::domain::value_objects::SaleId;

pub struct ApplyPromotionUseCase<P: PromotionRepository, S: SaleRepository> {
    promotion_repo: Arc<P>,
    sale_repo: Arc<S>,
}

impl<P: PromotionRepository, S: SaleRepository> ApplyPromotionUseCase<P, S> {
    pub fn new(promotion_repo: Arc<P>, sale_repo: Arc<S>) -> Self {
        Self {
            promotion_repo,
            sale_repo,
        }
    }

    pub async fn execute(
        &self,
        sale_id: uuid::Uuid,
        promotion_code: String,
    ) -> Result<SaleDetailResponse, SalesError> {
        // Find the promotion by code
        let mut promotion = self
            .promotion_repo
            .find_by_code(&promotion_code)
            .await?
            .ok_or(SalesError::PromotionNotFound(uuid::Uuid::nil()))?;

        // Find the sale
        let sale_id_vo = SaleId::from_uuid(sale_id);
        let mut sale = self
            .sale_repo
            .find_by_id_with_details(sale_id_vo)
            .await?
            .ok_or(SalesError::SaleNotFound(sale_id))?;

        // Validate promotion applicability
        promotion.validate_applicable(sale.total())?;

        // Calculate and apply discount
        let discount_amount = promotion.calculate_discount(sale.subtotal());

        if discount_amount > rust_decimal::Decimal::ZERO {
            sale.apply_fixed_discount(discount_amount)?;
        }

        // Increment usage
        promotion.increment_usage();

        // Persist changes
        self.sale_repo.update(&sale).await?;
        self.promotion_repo.update(&promotion).await?;

        Ok(SaleDetailResponse::from(sale))
    }
}

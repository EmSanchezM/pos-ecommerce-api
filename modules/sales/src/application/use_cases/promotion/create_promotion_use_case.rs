use std::str::FromStr;
use std::sync::Arc;

use crate::SalesError;
use crate::application::dtos::promotion::{CreatePromotionCommand, PromotionResponse};
use crate::domain::entities::Promotion;
use crate::domain::repositories::PromotionRepository;
use crate::domain::value_objects::PromotionType;
use identity::UserId;

pub struct CreatePromotionUseCase<P: PromotionRepository> {
    promotion_repo: Arc<P>,
}

impl<P: PromotionRepository> CreatePromotionUseCase<P> {
    pub fn new(promotion_repo: Arc<P>) -> Self {
        Self { promotion_repo }
    }

    pub async fn execute(
        &self,
        command: CreatePromotionCommand,
        actor_id: UserId,
    ) -> Result<PromotionResponse, SalesError> {
        // Validate code uniqueness
        if self
            .promotion_repo
            .find_by_code(&command.code)
            .await?
            .is_some()
        {
            return Err(SalesError::DuplicatePromotionCode(command.code));
        }

        let promotion_type = PromotionType::from_str(&command.promotion_type)?;

        let mut promotion = Promotion::create(
            command.code,
            command.name,
            promotion_type,
            command.discount_value,
            command.start_date,
            actor_id,
        );

        // Apply optional fields
        if let Some(desc) = command.description {
            promotion.set_description(Some(desc));
        }
        promotion.set_buy_quantity(command.buy_quantity);
        promotion.set_get_quantity(command.get_quantity);
        promotion.set_minimum_purchase(command.minimum_purchase);
        promotion.set_maximum_discount(command.maximum_discount);
        promotion.set_usage_limit(command.usage_limit);
        promotion.set_per_customer_limit(command.per_customer_limit);
        promotion.set_applies_to(command.applies_to);
        promotion.set_product_ids(command.product_ids);
        promotion.set_category_ids(command.category_ids);
        promotion.set_end_date(command.end_date);
        promotion.set_store_id(command.store_id);

        self.promotion_repo.save(&promotion).await?;

        Ok(PromotionResponse::from(&promotion))
    }
}

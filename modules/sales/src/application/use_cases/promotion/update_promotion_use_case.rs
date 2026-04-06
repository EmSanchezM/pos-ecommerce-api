use std::sync::Arc;

use crate::SalesError;
use crate::application::dtos::promotion::{PromotionResponse, UpdatePromotionCommand};
use crate::domain::repositories::PromotionRepository;
use crate::domain::value_objects::PromotionId;

pub struct UpdatePromotionUseCase<P: PromotionRepository> {
    promotion_repo: Arc<P>,
}

impl<P: PromotionRepository> UpdatePromotionUseCase<P> {
    pub fn new(promotion_repo: Arc<P>) -> Self {
        Self { promotion_repo }
    }

    pub async fn execute(
        &self,
        id: uuid::Uuid,
        command: UpdatePromotionCommand,
    ) -> Result<PromotionResponse, SalesError> {
        let mut promotion = self
            .promotion_repo
            .find_by_id(PromotionId::from_uuid(id))
            .await?
            .ok_or(SalesError::PromotionNotFound(id))?;

        if let Some(name) = command.name {
            promotion.set_name(name);
        }
        if let Some(description) = command.description {
            promotion.set_description(Some(description));
        }
        if let Some(value) = command.discount_value {
            promotion.set_discount_value(value);
        }
        if let Some(qty) = command.buy_quantity {
            promotion.set_buy_quantity(Some(qty));
        }
        if let Some(qty) = command.get_quantity {
            promotion.set_get_quantity(Some(qty));
        }
        if let Some(amount) = command.minimum_purchase {
            promotion.set_minimum_purchase(amount);
        }
        if let Some(amount) = command.maximum_discount {
            promotion.set_maximum_discount(Some(amount));
        }
        if let Some(limit) = command.usage_limit {
            promotion.set_usage_limit(Some(limit));
        }
        if let Some(limit) = command.per_customer_limit {
            promotion.set_per_customer_limit(Some(limit));
        }
        if let Some(applies_to) = command.applies_to {
            promotion.set_applies_to(applies_to);
        }
        if let Some(ids) = command.product_ids {
            promotion.set_product_ids(ids);
        }
        if let Some(ids) = command.category_ids {
            promotion.set_category_ids(ids);
        }
        if let Some(date) = command.start_date {
            promotion.set_start_date(date);
        }
        if let Some(date) = command.end_date {
            promotion.set_end_date(Some(date));
        }
        if let Some(store_id) = command.store_id {
            promotion.set_store_id(Some(store_id));
        }

        self.promotion_repo.update(&promotion).await?;

        Ok(PromotionResponse::from(&promotion))
    }
}

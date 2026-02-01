//! Apply discount use case

use std::str::FromStr;
use std::sync::Arc;

use crate::application::dtos::{ApplyDiscountCommand, SaleDetailResponse};
use crate::domain::repositories::SaleRepository;
use crate::domain::value_objects::{DiscountType, SaleId, SaleItemId};
use crate::SalesError;

/// Use case for applying a discount to a sale or item
pub struct ApplyDiscountUseCase {
    sale_repo: Arc<dyn SaleRepository>,
}

impl ApplyDiscountUseCase {
    pub fn new(sale_repo: Arc<dyn SaleRepository>) -> Self {
        Self { sale_repo }
    }

    pub async fn execute(&self, cmd: ApplyDiscountCommand) -> Result<SaleDetailResponse, SalesError> {
        let sale_id = SaleId::from_uuid(cmd.sale_id);
        let discount_type = DiscountType::from_str(&cmd.discount_type)
            .map_err(|_| SalesError::InvalidDiscountType)?;

        let mut sale = self
            .sale_repo
            .find_by_id_with_items(sale_id)
            .await?
            .ok_or(SalesError::SaleNotFound(cmd.sale_id))?;

        // Verify sale is in draft status
        if !sale.status().is_draft() {
            return Err(SalesError::SaleNotEditable);
        }

        if let Some(item_uuid) = cmd.item_id {
            // Apply discount to specific item
            let item_id = SaleItemId::from_uuid(item_uuid);
            sale.apply_item_discount(item_id, discount_type, cmd.discount_value)?;

            // Get the updated item
            let updated_item = sale
                .items()
                .iter()
                .find(|i| i.id() == item_id)
                .ok_or(SalesError::SaleItemNotFound(item_uuid))?;

            // Save the updated item
            self.sale_repo.update_item(updated_item).await?;
        } else {
            // Apply discount to entire sale
            sale.apply_sale_discount(discount_type, cmd.discount_value)?;
        }

        // Update sale
        self.sale_repo.update(&sale).await?;

        Ok(SaleDetailResponse::from(sale))
    }
}

//! Update sale item use case

use std::sync::Arc;

use crate::application::dtos::{SaleDetailResponse, UpdateSaleItemCommand};
use crate::domain::repositories::SaleRepository;
use crate::domain::value_objects::SaleItemId;
use crate::SalesError;

/// Use case for updating a sale item
pub struct UpdateSaleItemUseCase {
    sale_repo: Arc<dyn SaleRepository>,
}

impl UpdateSaleItemUseCase {
    pub fn new(sale_repo: Arc<dyn SaleRepository>) -> Self {
        Self { sale_repo }
    }

    pub async fn execute(&self, cmd: UpdateSaleItemCommand) -> Result<SaleDetailResponse, SalesError> {
        let item_id = SaleItemId::from_uuid(cmd.item_id);

        // Find the item to get the sale_id
        let item = self
            .sale_repo
            .find_item_by_id(item_id)
            .await?
            .ok_or(SalesError::SaleItemNotFound(cmd.item_id))?;

        let sale_uuid = item.sale_id().into_uuid();
        let mut sale = self
            .sale_repo
            .find_by_id_with_items(item.sale_id())
            .await?
            .ok_or(SalesError::SaleNotFound(sale_uuid))?;

        // Verify sale is in draft status
        if !sale.status().is_draft() {
            return Err(SalesError::SaleNotEditable);
        }

        // Update the item in the sale
        sale.update_item(item_id, cmd.quantity, cmd.unit_price, cmd.notes)?;

        // Get the updated item
        let updated_item = sale
            .items()
            .iter()
            .find(|i| i.id() == item_id)
            .ok_or(SalesError::SaleItemNotFound(cmd.item_id))?;

        // Save the updated item
        self.sale_repo.update_item(updated_item).await?;

        // Update sale totals
        self.sale_repo.update(&sale).await?;

        Ok(SaleDetailResponse::from(sale))
    }
}

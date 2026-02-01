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
            .find_by_id_with_details(item.sale_id())
            .await?
            .ok_or(SalesError::SaleNotFound(sale_uuid))?;

        // Verify sale is editable
        if !sale.is_editable() {
            return Err(SalesError::SaleNotEditable);
        }

        // Find the item in the sale and update it
        let sale_item = sale
            .items_mut()
            .iter_mut()
            .find(|i| i.id() == item_id)
            .ok_or(SalesError::SaleItemNotFound(cmd.item_id))?;

        if let Some(qty) = cmd.quantity {
            sale_item.set_quantity(qty)?;
        }
        if let Some(price) = cmd.unit_price {
            sale_item.set_unit_price(price)?;
        }
        if cmd.notes.is_some() {
            sale_item.set_notes(cmd.notes);
        }

        // Recalculate sale totals
        sale.recalculate_totals();

        // Find the updated item for saving
        let item_to_save = sale
            .items()
            .iter()
            .find(|i| i.id() == item_id)
            .ok_or(SalesError::SaleItemNotFound(cmd.item_id))?;

        self.sale_repo.update_item(item_to_save).await?;
        self.sale_repo.update(&sale).await?;

        Ok(SaleDetailResponse::from(sale))
    }
}

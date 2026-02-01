//! Remove sale item use case

use std::sync::Arc;
use uuid::Uuid;

use crate::application::dtos::SaleDetailResponse;
use crate::domain::repositories::SaleRepository;
use crate::domain::value_objects::SaleItemId;
use crate::SalesError;

/// Use case for removing an item from a sale
pub struct RemoveSaleItemUseCase {
    sale_repo: Arc<dyn SaleRepository>,
}

impl RemoveSaleItemUseCase {
    pub fn new(sale_repo: Arc<dyn SaleRepository>) -> Self {
        Self { sale_repo }
    }

    pub async fn execute(&self, item_id: Uuid) -> Result<SaleDetailResponse, SalesError> {
        let item_id_vo = SaleItemId::from_uuid(item_id);

        // Find the item to get the sale_id
        let item = self
            .sale_repo
            .find_item_by_id(item_id_vo)
            .await?
            .ok_or(SalesError::SaleItemNotFound(item_id))?;

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

        // Remove the item from the sale
        sale.remove_item(item_id_vo)?;

        // Delete from database
        self.sale_repo.delete_item(item_id_vo).await?;

        // Update sale totals
        self.sale_repo.update(&sale).await?;

        Ok(SaleDetailResponse::from(sale))
    }
}
